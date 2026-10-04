use std::io::{self, Write};

use pine_runtime::{RuntimeResult, RuntimeResultView, write_public_runtime_result_view_json};

/// Retain a complete String without whole-family encoding buffers or growth copies.
pub(crate) fn borrowed_snapshot_json(result: &RuntimeResult) -> String {
    result_view_snapshot_json(&result.view())
}

pub(crate) fn stream_snapshot_json(
    result: &RuntimeResultView<'_>,
    revision: u64,
    retained_from: usize,
) -> String {
    let prefix = format!("{{\"revision\":{revision},\"retainedFrom\":{retained_from},\"result\":");
    encode_snapshot(result, &prefix, "}")
}

pub(crate) fn result_view_snapshot_json(result: &RuntimeResultView<'_>) -> String {
    encode_snapshot(result, "", "")
}

fn encode_snapshot(result: &RuntimeResultView<'_>, prefix: &str, suffix: &str) -> String {
    struct Counter(usize);
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0 += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut count = Counter(0);
    write_public_runtime_result_view_json(result, &mut count).expect("counting cannot fail");
    let total = prefix.len() + count.0 + suffix.len();
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(prefix.as_bytes());
    write_public_runtime_result_view_json(result, &mut bytes)
        .expect("writing to a Vec cannot fail");
    bytes.extend_from_slice(suffix.as_bytes());
    debug_assert_eq!(bytes.len(), total);
    String::from_utf8(bytes).expect("JSON serializers produce UTF-8")
}
