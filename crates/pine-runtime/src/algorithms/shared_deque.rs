use std::{
    collections::VecDeque,
    ops::{Bound, Deref, DerefMut, Index, RangeBounds},
    sync::Arc,
};

const PAGE_SIZE: usize = 128;

#[derive(Debug)]
struct Page<T>(Vec<T>);

impl<T: Clone> Clone for Page<T> {
    fn clone(&self) -> Self {
        // Vec::clone uses capacity=len. An ensuing 127 -> 128 append would
        // double that to 254 and leave oversized pages throughout a sliding
        // queue. Allocate its fixed capacity once when making the page private.
        let mut values = Vec::with_capacity(PAGE_SIZE);
        values.extend(self.0.iter().cloned());
        Self(values)
    }
}

impl<T> Deref for Page<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Page<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Short queues keep their ordinary flat path. Larger checkpoints share the
/// page directory; an endpoint write copies references and one bounded page,
/// rather than postponing a copy of the complete payload until the first write.
#[derive(Debug, Clone)]
pub(crate) struct SharedDeque<T> {
    storage: Storage<T>,
}

#[derive(Debug, Clone)]
enum Storage<T> {
    Small(VecDeque<T>),
    Paged {
        pages: Arc<VecDeque<Arc<Page<T>>>>,
        front_offset: usize,
        back_len: usize,
        len: usize,
    },
}

impl<T> Default for SharedDeque<T> {
    fn default() -> Self {
        Self {
            storage: Storage::Small(VecDeque::new()),
        }
    }
}

impl<T> SharedDeque<T> {
    pub(crate) fn len(&self) -> usize {
        match &self.storage {
            Storage::Small(values) => values.len(),
            Storage::Paged { len, .. } => *len,
        }
    }

    pub(crate) fn capacity(&self) -> usize {
        match &self.storage {
            Storage::Small(values) => values.capacity(),
            Storage::Paged { pages, .. } => pages.iter().map(|page| page.capacity()).sum(),
        }
    }

    pub(crate) fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub(crate) fn back(&self) -> Option<&T> {
        self.len().checked_sub(1).and_then(|index| self.get(index))
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        match &self.storage {
            Storage::Small(values) => values.get(index),
            Storage::Paged {
                pages,
                front_offset,
                len,
                ..
            } => {
                if index >= *len {
                    return None;
                }
                let (page, offset) = locate(pages, *front_offset, index);
                Some(&pages[page][offset])
            }
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, T> {
        self.range(..)
    }

    pub(crate) fn range(&self, range: impl RangeBounds<usize>) -> Iter<'_, T> {
        match &self.storage {
            Storage::Small(values) => Iter::Small(values.range(range)),
            Storage::Paged {
                pages,
                front_offset,
                len,
                ..
            } => {
                let start = match range.start_bound() {
                    Bound::Included(&index) => index,
                    Bound::Excluded(&index) => index.checked_add(1).expect("range start overflow"),
                    Bound::Unbounded => 0,
                };
                let end = match range.end_bound() {
                    Bound::Included(&index) => index.checked_add(1).expect("range end overflow"),
                    Bound::Excluded(&index) => index,
                    Bound::Unbounded => *len,
                };
                assert!(start <= end && end <= *len, "deque range out of bounds");
                let remaining = end - start;
                if remaining == 0 {
                    return Iter::Paged(PagedIter {
                        root: pages,
                        front_offset: *front_offset,
                        front_index: start,
                        back_index: end,
                        pages: pages.range(..0),
                        current: [].iter(),
                        remaining: 0,
                    });
                }
                let (page, offset) = locate(pages, *front_offset, start);
                Iter::Paged(PagedIter {
                    root: pages,
                    front_offset: *front_offset,
                    front_index: start,
                    back_index: end,
                    pages: pages.range(page + 1..),
                    current: pages[page][offset..].iter(),
                    remaining,
                })
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        match &mut self.storage {
            Storage::Small(values) => values.clear(),
            Storage::Paged { .. } => *self = Self::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[cfg(test)]
    pub(crate) fn has_split_storage(&self) -> bool {
        match &self.storage {
            Storage::Small(values) => !values.as_slices().1.is_empty(),
            Storage::Paged { pages, .. } => pages.len() > 1,
        }
    }

    #[cfg(test)]
    fn page_directory_capacity(&self) -> usize {
        match &self.storage {
            Storage::Small(_) => 0,
            Storage::Paged { pages, .. } => pages.capacity(),
        }
    }
}

// Only the first and last pages may be partial. All interior pages have exactly
// PAGE_SIZE cells, so a range starts at its first cell without walking its prefix.
fn locate<T>(pages: &VecDeque<Arc<Page<T>>>, front_offset: usize, index: usize) -> (usize, usize) {
    let first = pages[0].len() - front_offset;
    if index < first {
        (0, front_offset + index)
    } else {
        let rest = index - first;
        (1 + rest / PAGE_SIZE, rest % PAGE_SIZE)
    }
}

fn prune_directory<T>(pages: &mut VecDeque<Arc<Page<T>>>) {
    // Repeated shrink/grow sessions must not retain a directory sized for the
    // session's historical maximum. Avoid reallocating at every page boundary.
    if pages.capacity() > 16 && pages.len() < pages.capacity() / 4 {
        pages.shrink_to(pages.len().saturating_mul(2).max(4));
    }
}

impl<T: Clone> SharedDeque<T> {
    fn promote(&mut self) {
        let Storage::Small(values) = &mut self.storage else {
            return;
        };
        let len = values.len();
        let mut values = std::mem::take(values).into_iter();
        let mut pages = VecDeque::with_capacity(len.div_ceil(PAGE_SIZE));
        while values.len() != 0 {
            let mut page = Vec::with_capacity(PAGE_SIZE);
            page.extend(values.by_ref().take(PAGE_SIZE));
            pages.push_back(Arc::new(Page(page)));
        }
        let back_len = pages.back().expect("nonempty promoted deque").len();
        self.storage = Storage::Paged {
            pages: Arc::new(pages),
            front_offset: 0,
            back_len,
            len,
        };
    }

    pub(crate) fn push_back(&mut self, value: T) {
        if matches!(&self.storage, Storage::Small(values) if values.len() == PAGE_SIZE) {
            self.promote();
        }
        match &mut self.storage {
            Storage::Small(values) => values.push_back(value),
            Storage::Paged {
                pages,
                back_len,
                len,
                ..
            } => {
                let pages = Arc::make_mut(pages);
                if *back_len == PAGE_SIZE {
                    let mut page = Vec::with_capacity(PAGE_SIZE);
                    page.push(value);
                    pages.push_back(Arc::new(Page(page)));
                    *back_len = 1;
                } else {
                    let page = Arc::make_mut(pages.back_mut().expect("nonempty paged deque"));
                    if *back_len < page.len() {
                        page[*back_len] = value;
                    } else {
                        page.push(value);
                    }
                    *back_len += 1;
                }
                *len += 1;
            }
        }
    }

    pub(crate) fn push_front(&mut self, value: T) {
        if matches!(&self.storage, Storage::Small(values) if values.len() == PAGE_SIZE) {
            self.promote();
        }
        match &mut self.storage {
            Storage::Small(values) => values.push_front(value),
            Storage::Paged {
                pages,
                front_offset,
                back_len,
                len,
            } => {
                let pages = Arc::make_mut(pages);
                if *front_offset != 0 {
                    *front_offset -= 1;
                    Arc::make_mut(pages.front_mut().expect("nonempty paged deque"))
                        [*front_offset] = value;
                } else if pages[0].len() < PAGE_SIZE {
                    let single = pages.len() == 1;
                    Arc::make_mut(pages.front_mut().expect("nonempty paged deque"))
                        .insert(0, value);
                    if single {
                        *back_len += 1;
                    }
                } else {
                    let mut page = Vec::with_capacity(PAGE_SIZE);
                    page.push(value);
                    pages.push_front(Arc::new(Page(page)));
                }
                *len += 1;
            }
        }
    }

    pub(crate) fn pop_front(&mut self) -> Option<T> {
        match &mut self.storage {
            Storage::Small(values) => values.pop_front(),
            Storage::Paged {
                pages,
                front_offset,
                back_len,
                len,
            } => {
                let value = pages[0][*front_offset].clone();
                *front_offset += 1;
                *len -= 1;
                if *len == 0 {
                    *self = Self::default();
                } else if *front_offset == pages[0].len() {
                    let pages = Arc::make_mut(pages);
                    pages.pop_front();
                    *front_offset = 0;
                    if pages.len() == 1 {
                        debug_assert!(*back_len <= pages[0].len());
                    }
                    prune_directory(pages);
                }
                Some(value)
            }
        }
    }

    pub(crate) fn pop_back(&mut self) -> Option<T> {
        match &mut self.storage {
            Storage::Small(values) => values.pop_back(),
            Storage::Paged {
                pages,
                back_len,
                len,
                ..
            } => {
                let value = pages.back().expect("nonempty paged deque")[*back_len - 1].clone();
                *back_len -= 1;
                *len -= 1;
                if *len == 0 {
                    *self = Self::default();
                } else if *back_len == 0 {
                    let pages = Arc::make_mut(pages);
                    pages.pop_back();
                    *back_len = pages.back().expect("surviving paged deque").len();
                    prune_directory(pages);
                }
                Some(value)
            }
        }
    }
}

impl<T: Clone> From<VecDeque<T>> for SharedDeque<T> {
    fn from(values: VecDeque<T>) -> Self {
        let mut deque = Self {
            storage: Storage::Small(values),
        };
        if deque.len() > PAGE_SIZE {
            deque.promote();
        }
        deque
    }
}

impl<T: Clone> FromIterator<T> for SharedDeque<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut values = Self::default();
        for value in iter {
            values.push_back(value);
        }
        values
    }
}

impl<T: PartialEq> PartialEq for SharedDeque<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T: PartialEq> PartialEq<VecDeque<T>> for SharedDeque<T> {
    fn eq(&self, other: &VecDeque<T>) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T> Index<usize> for SharedDeque<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("deque index out of bounds")
    }
}

impl<'a, T> IntoIterator for &'a SharedDeque<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub(crate) enum Iter<'a, T> {
    Small(std::collections::vec_deque::Iter<'a, T>),
    Paged(PagedIter<'a, T>),
}

pub(crate) struct PagedIter<'a, T> {
    root: &'a VecDeque<Arc<Page<T>>>,
    front_offset: usize,
    front_index: usize,
    back_index: usize,
    pages: std::collections::vec_deque::Iter<'a, Arc<Page<T>>>,
    current: std::slice::Iter<'a, T>,
    remaining: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Small(values) => values.next(),
            Self::Paged(iter) => {
                if iter.remaining == 0 {
                    return None;
                }
                loop {
                    if let Some(value) = iter.current.next() {
                        iter.remaining -= 1;
                        iter.front_index += 1;
                        return Some(value);
                    }
                    iter.current = iter.pages.next()?.iter();
                }
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Self::Small(values) => values.nth(n),
            Self::Paged(iter) => {
                if n >= iter.remaining {
                    iter.front_index += iter.remaining;
                    iter.remaining = 0;
                    return None;
                }
                iter.front_index += n;
                iter.remaining -= n;
                let (page, offset) = locate(iter.root, iter.front_offset, iter.front_index);
                iter.current = iter.root[page][offset..].iter();
                iter.pages = iter.root.range(page + 1..);
                self.next()
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = match self {
            Self::Small(values) => values.len(),
            Self::Paged(iter) => iter.remaining,
        };
        (remaining, Some(remaining))
    }

    fn fold<B, F>(self, init: B, mut fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Small(values) => values.fold(init, fold),
            Self::Paged(iter) => {
                // Carry the same accumulator across slices. Combining separate
                // per-page sums would change the floating-point addition order.
                let mut remaining = iter.remaining;
                let take = remaining.min(iter.current.len());
                let mut accumulator = iter.current.as_slice()[..take].iter().fold(init, &mut fold);
                remaining -= take;
                for page in iter.pages {
                    if remaining == 0 {
                        break;
                    }
                    let take = remaining.min(page.len());
                    accumulator = page[..take].iter().fold(accumulator, &mut fold);
                    remaining -= take;
                }
                debug_assert_eq!(remaining, 0);
                accumulator
            }
        }
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Small(values) => values.next_back(),
            Self::Paged(iter) => {
                if iter.remaining == 0 {
                    return None;
                }
                iter.back_index -= 1;
                iter.remaining -= 1;
                let (page, offset) = locate(iter.root, iter.front_offset, iter.back_index);
                Some(&iter.root[page][offset])
            }
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Self::Small(values) => values.nth_back(n),
            Self::Paged(iter) => {
                if n >= iter.remaining {
                    iter.back_index -= iter.remaining;
                    iter.remaining = 0;
                    return None;
                }
                iter.back_index -= n;
                iter.remaining -= n;
                self.next_back()
            }
        }
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

#[cfg(test)]
#[path = "shared_deque_tests.rs"]
mod tests;
