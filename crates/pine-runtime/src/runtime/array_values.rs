//! Array payloads keep small arrays flat and share pages in large checkpoints.
//! A sparse write copies one page and the page directory, not every element.
use std::{ops::Index, sync::Arc};

const PAGE_SIZE: usize = 128;

#[derive(Clone, Debug)]
pub(crate) struct ArrayValues<T = crate::PineValue> {
    storage: Storage<T>,
}

#[derive(Clone, Debug)]
enum Storage<T> {
    Small(Vec<T>),
    Paged {
        pages: Arc<Vec<Arc<Vec<T>>>>,
        len: usize,
    },
}

impl<T> From<Vec<T>> for ArrayValues<T> {
    fn from(values: Vec<T>) -> Self {
        let len = values.len();
        let storage = if len <= PAGE_SIZE {
            Storage::Small(values)
        } else {
            let mut values = values.into_iter();
            let mut pages = Vec::with_capacity(len.div_ceil(PAGE_SIZE));
            while values.len() != 0 {
                pages.push(Arc::new(values.by_ref().take(PAGE_SIZE).collect()));
            }
            Storage::Paged {
                pages: Arc::new(pages),
                len,
            }
        };
        Self { storage }
    }
}

impl<T> ArrayValues<T> {
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

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        match &self.storage {
            Storage::Small(values) => values.get(index),
            Storage::Paged { pages, len } => {
                (index < *len).then(|| &pages[index / PAGE_SIZE][index % PAGE_SIZE])
            }
        }
    }

    pub(crate) fn view(&self, start: usize, len: usize) -> ArrayView<'_, T> {
        assert!(start <= self.len() && len <= self.len() - start);
        ArrayView {
            values: self,
            start,
            len,
        }
    }

    pub(crate) fn iter(&self) -> ArrayIter<'_, T> {
        self.view(0, self.len()).iter()
    }
}

impl<T: Clone> ArrayValues<T> {
    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match &mut self.storage {
            Storage::Small(values) => values.get_mut(index),
            Storage::Paged { pages, len } => {
                if index >= *len {
                    return None;
                }
                Arc::make_mut(&mut Arc::make_mut(pages)[index / PAGE_SIZE])
                    .get_mut(index % PAGE_SIZE)
            }
        }
    }

    pub(crate) fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    pub(crate) fn insert(&mut self, index: usize, value: T) {
        let len = self.len();
        assert!(index <= len);
        // Middle insertions need to shift the suffix. Keep their owned, flat
        // operation separate from the common checkpoint + tail append path.
        if index != len {
            if let Storage::Small(values) = &mut self.storage {
                values.insert(index, value);
                if values.len() > PAGE_SIZE {
                    *self = std::mem::take(values).into();
                }
                return;
            }
            let mut values = self.to_vec();
            values.insert(index, value);
            *self = values.into();
            return;
        }
        match &mut self.storage {
            Storage::Small(values) if values.len() < PAGE_SIZE => values.push(value),
            Storage::Small(values) => {
                let first = Arc::new(std::mem::take(values));
                self.storage = Storage::Paged {
                    pages: Arc::new(vec![first, Arc::new(vec![value])]),
                    len: len + 1,
                };
            }
            Storage::Paged { pages, len } => {
                let pages = Arc::make_mut(pages);
                if *len % PAGE_SIZE == 0 {
                    pages.push(Arc::new(vec![value]));
                } else {
                    Arc::make_mut(pages.last_mut().expect("nonempty array")).push(value);
                }
                *len += 1;
            }
        }
    }

    pub(crate) fn remove(&mut self, index: usize) -> T {
        let len = self.len();
        assert!(index < len);
        if index + 1 != len {
            if let Storage::Small(values) = &mut self.storage {
                return values.remove(index);
            }
            let mut values = self.to_vec();
            let removed = values.remove(index);
            *self = values.into();
            return removed;
        }
        match &mut self.storage {
            Storage::Small(values) => values.pop().expect("nonempty array"),
            Storage::Paged { pages, len } => {
                let pages = Arc::make_mut(pages);
                let last = Arc::make_mut(pages.last_mut().expect("nonempty array"));
                let removed = last.pop().expect("nonempty page");
                if last.is_empty() {
                    pages.pop();
                }
                *len -= 1;
                if *len == 0 {
                    self.storage = Storage::Small(Vec::new());
                }
                removed
            }
        }
    }
}

pub(crate) struct ArrayView<'a, T = crate::PineValue> {
    values: &'a ArrayValues<T>,
    start: usize,
    len: usize,
}

impl<'a, T> ArrayView<'a, T> {
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub(crate) fn get(&self, index: usize) -> Option<&'a T> {
        (index < self.len).then(|| self.values.get(self.start + index).expect("valid view"))
    }
    pub(crate) fn iter(&self) -> ArrayIter<'a, T> {
        ArrayIter {
            values: self.values,
            front: self.start,
            back: self.start + self.len,
        }
    }
}

impl<T> Index<usize> for ArrayView<'_, T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.get(index).expect("array view index")
    }
}

impl<'a, T> IntoIterator for ArrayView<'a, T> {
    type Item = &'a T;
    type IntoIter = ArrayIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a ArrayValues<T> {
    type Item = &'a T;
    type IntoIter = ArrayIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub(crate) struct ArrayIter<'a, T> {
    values: &'a ArrayValues<T>,
    front: usize,
    back: usize,
}

impl<'a, T> Iterator for ArrayIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        let index = self.front;
        self.front += 1;
        self.values.get(index)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for ArrayIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.values.get(self.back)
    }
}
impl<T> ExactSizeIterator for ArrayIter<'_, T> {}

#[cfg(test)]
mod tests;
