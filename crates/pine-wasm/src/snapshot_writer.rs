use std::collections::VecDeque;
use std::io::{self, Write};

use pine_runtime::{RuntimeResultView, write_public_runtime_result_view_json};
use wasm_bindgen::prelude::*;

const CHUNK_BYTES: usize = 64 * 1024;

pub(crate) fn diagnostic_count(result: &RuntimeResultView<'_>) -> usize {
    result.diagnostics.len()
        + result
            .strategy
            .as_ref()
            .map_or(0, |strategy| strategy.diagnostics.len())
}

/// An immutable, complete JSON snapshot. Encoded bytes are owned by this handle,
/// independently of the source session. Pulling a chunk consumes its bytes;
/// free the handle to discard any unread remainder.
#[wasm_bindgen]
pub struct SnapshotChunks {
    chunks: VecDeque<Vec<u8>>,
    length: usize,
    remaining: usize,
    offset: usize,
}

#[wasm_bindgen]
impl SnapshotChunks {
    /// Copy the next at-most-64-KiB block to JavaScript, or return undefined at
    /// EOF. Blocks may split UTF-8 code points; decode using a streaming decoder
    /// or concatenate all blocks before decoding.
    #[wasm_bindgen(js_name = nextChunk)]
    pub fn next_chunk(&mut self) -> Option<Box<[u8]>> {
        self.chunks.pop_front().map(|mut chunk| {
            if self.offset != 0 {
                chunk = chunk.split_off(self.offset);
                self.offset = 0;
            }
            self.remaining -= chunk.len();
            chunk.into_boxed_slice()
        })
    }

    /// Fill a caller-owned buffer and return bytes copied. Reuse one buffer to
    /// avoid allocating a JavaScript array for every block. Zero means EOF or
    /// an empty target; bytes outside the returned prefix are left unchanged.
    #[wasm_bindgen(js_name = readInto)]
    pub fn read_into(&mut self, target: &mut [u8]) -> usize {
        let mut written = 0;
        while written < target.len() {
            let Some(chunk) = self.chunks.front() else {
                break;
            };
            let length = (chunk.len() - self.offset).min(target.len() - written);
            target[written..written + length]
                .copy_from_slice(&chunk[self.offset..self.offset + length]);
            self.offset += length;
            written += length;
            if self.offset == chunk.len() {
                self.chunks.pop_front();
                self.offset = 0;
            }
        }
        self.remaining -= written;
        written
    }

    #[wasm_bindgen(getter, js_name = byteLength)]
    pub fn byte_length(&self) -> usize {
        self.length
    }

    #[wasm_bindgen(getter, js_name = remainingBytes)]
    pub fn remaining_bytes(&self) -> usize {
        self.remaining
    }
}

struct ChunkWriter {
    chunks: VecDeque<Vec<u8>>,
    written: usize,
}

impl Write for ChunkWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.written = self
            .written
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("snapshot byte count exceeds addressable size"))?;
        let mut remaining = bytes;
        while !remaining.is_empty() {
            if self
                .chunks
                .back()
                .is_none_or(|chunk| chunk.len() == CHUNK_BYTES)
            {
                self.chunks.push_back(Vec::with_capacity(CHUNK_BYTES));
            }
            let chunk = self.chunks.back_mut().expect("chunk was allocated");
            let length = remaining.len().min(CHUNK_BYTES - chunk.len());
            chunk.extend_from_slice(&remaining[..length]);
            remaining = &remaining[length..];
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn snapshot_chunks(result: &RuntimeResultView<'_>) -> Result<SnapshotChunks, String> {
    let mut writer = ChunkWriter {
        chunks: VecDeque::new(),
        written: 0,
    };
    write_public_runtime_result_view_json(result, &mut writer)
        .map_err(|error| error.to_string())?;
    Ok(SnapshotChunks {
        chunks: writer.chunks,
        length: writer.written,
        remaining: writer.written,
        offset: 0,
    })
}
