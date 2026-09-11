use super::append_history::AppendHistory;
use crate::{
    BoxOutput, BoxSnapshot, LabelOutput, LabelSnapshot, LineFillOutput, LineFillSnapshot,
    LineOutput, LineSnapshot, PineValue, PolylineOutput, PolylineSnapshot, TableOutput,
    TableSnapshot,
};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDrawing<S> {
    pub(crate) id: u32,
    pub(crate) snapshots: AppendHistory<S>,
}

impl<S: Clone> RuntimeDrawing<S> {
    pub(crate) fn from_snapshot(id: u32, snapshot: S) -> Self {
        let mut snapshots = AppendHistory::default();
        snapshots.push(snapshot);
        Self { id, snapshots }
    }
}

pub(crate) type RuntimeLabel = RuntimeDrawing<LabelSnapshot>;
pub(crate) type RuntimeLine = RuntimeDrawing<LineSnapshot>;
pub(crate) type RuntimeLineFill = RuntimeDrawing<LineFillSnapshot>;
pub(crate) type RuntimePolyline = RuntimeDrawing<PolylineSnapshot>;
pub(crate) type RuntimeBox = RuntimeDrawing<BoxSnapshot>;

impl RuntimeLabel {
    pub(crate) fn snapshot(&self) -> LabelOutput {
        LabelOutput {
            id: self.id,
            snapshots: self.snapshots.to_vec(),
        }
    }
}

impl RuntimeLine {
    pub(crate) fn snapshot(&self) -> LineOutput {
        LineOutput {
            id: self.id,
            snapshots: self.snapshots.to_vec(),
        }
    }
}

impl RuntimeLineFill {
    pub(crate) fn snapshot(&self) -> LineFillOutput {
        LineFillOutput {
            id: self.id,
            snapshots: self.snapshots.to_vec(),
        }
    }
}

impl RuntimePolyline {
    pub(crate) fn snapshot(&self) -> PolylineOutput {
        PolylineOutput {
            id: self.id,
            snapshots: self.snapshots.to_vec(),
        }
    }
}

impl RuntimeBox {
    pub(crate) fn snapshot(&self) -> BoxOutput {
        BoxOutput {
            id: self.id,
            snapshots: self.snapshots.to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTable {
    pub(crate) id: u32,
    pub(crate) position: PineValue,
    pub(crate) bg_color: PineValue,
    pub(crate) frame_color: PineValue,
    pub(crate) frame_width: PineValue,
    pub(crate) border_color: PineValue,
    pub(crate) border_width: PineValue,
    pub(crate) columns: i64,
    pub(crate) rows: i64,
    pub(crate) snapshots: AppendHistory<TableSnapshot>,
}

impl RuntimeTable {
    pub(crate) fn snapshot(&self) -> TableOutput {
        TableOutput {
            id: self.id,
            position: self.position.clone(),
            bg_color: self.bg_color.clone(),
            frame_color: self.frame_color.clone(),
            frame_width: self.frame_width.clone(),
            border_color: self.border_color.clone(),
            border_width: self.border_width.clone(),
            columns: self.columns,
            rows: self.rows,
            snapshots: self.snapshots.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn drawing_checkpoint_append_copies_only_a_bounded_leaf() {
        #[derive(Debug)]
        struct Counted(Arc<AtomicUsize>);
        impl Clone for Counted {
            fn clone(&self) -> Self {
                self.0.fetch_add(1, Ordering::Relaxed);
                Self(self.0.clone())
            }
        }
        let count = Arc::new(AtomicUsize::new(0));
        let mut drawing = RuntimeDrawing::from_snapshot(1, Counted(count.clone()));
        for _ in 0..1_000 {
            drawing.snapshots.push(Counted(count.clone()));
        }
        let checkpoint = drawing.clone();
        count.store(0, Ordering::Relaxed);
        drawing.snapshots.push(Counted(count.clone()));
        assert!(count.load(Ordering::Relaxed) <= 128);
        assert_eq!(checkpoint.snapshots.len(), 1_001);
        assert_eq!(drawing.snapshots.len(), 1_002);
    }
}
