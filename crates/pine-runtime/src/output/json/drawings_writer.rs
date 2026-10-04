//! Sink encoding of drawing histories with at most one snapshot buffer.
use super::*;
use std::io::{self, Write};

fn write_history<'a, W: Write + ?Sized, T: 'a>(
    output: &mut W,
    prefix: &str,
    snapshots: HistoryView<'a, T>,
    mut serialize_snapshot: impl FnMut(&'a T) -> String,
) -> io::Result<()> {
    // The prefix and each snapshot come from our typed legacy serializers. The
    // caller supplies exactly one borrowed snapshot, so this is trusted wrapper
    // removal, never parsing or searching arbitrary JSON payloads.
    assert!(prefix.starts_with("[{\"id\":"));
    assert!(prefix.ends_with("\"snapshots\":["));
    output.write_all(&prefix.as_bytes()[1..])?;
    for (index, snapshot) in snapshots.iter().enumerate() {
        if index > 0 {
            output.write_all(b",")?;
        }
        let text = serialize_snapshot(snapshot);
        let snapshot = text
            .strip_prefix(prefix)
            .and_then(|text| text.strip_suffix("]}]"))
            .expect("single-snapshot serializer must preserve its exact wrapper");
        assert!(snapshot.starts_with("{\"barIndex\":"));
        assert!(snapshot.ends_with('}'));
        output.write_all(snapshot.as_bytes())?;
    }
    output.write_all(b"]}")
}

macro_rules! drawing {
    ($function:ident, $view:ident, $serializer:ident) => {
        pub(super) fn $function<W: Write + ?Sized>(
            drawing: &$view<'_>,
            output: &mut W,
        ) -> io::Result<()> {
            let prefix = format!("[{{\"id\":{},\"snapshots\":[", drawing.id);
            write_history(output, &prefix, drawing.snapshots, |snapshot| {
                $serializer([$view {
                    id: drawing.id,
                    snapshots: HistoryView::from_slice(std::slice::from_ref(snapshot)),
                }])
            })
        }
    };
}

drawing!(labels, LabelOutputView, labels_json);
drawing!(lines, LineOutputView, lines_json);
drawing!(line_fills, LineFillOutputView, line_fills_json);
drawing!(polylines, PolylineOutputView, polylines_json);
drawing!(boxes, BoxOutputView, boxes_json);

pub(super) fn tables<W: Write + ?Sized>(
    table: &TableOutputView<'_>,
    output: &mut W,
) -> io::Result<()> {
    // Tables have extra metadata before snapshots. Ask the established serializer
    // for their exact header once rather than duplicating its field/escaping rules.
    let header = tables_json([TableOutputView {
        snapshots: HistoryView::from_slice(&[]),
        ..*table
    }]);
    let prefix = header
        .strip_suffix("]}]")
        .expect("empty table serializer must preserve its exact wrapper");
    write_history(output, prefix, table.snapshots, |snapshot| {
        tables_json([TableOutputView {
            snapshots: HistoryView::from_slice(std::slice::from_ref(snapshot)),
            ..*table
        }])
    })
}
