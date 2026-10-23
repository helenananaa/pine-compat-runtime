# WASM snapshot byte readers

`RealtimeSession.resultChunks()` and `RuntimeReplica.resultChunks()` capture the
same complete public JSON as `result()`. They return an independent
`SnapshotChunks` handle. `RuntimeReplica.intoResultChunks()` also consumes the
replica. Existing String-returning methods retain their behavior.

The reader owns all encoded snapshot bytes in WASM. It does not hold a borrow
of the session: the session may advance or be freed while the snapshot is being
read. Its contents retain the state captured at creation. Serialization does
not call a JavaScript sink while holding a borrowed runtime.

- `byteLength` is the complete UTF-8 byte length.
- `remainingBytes` decreases as bytes are read.
- `readInto(target)` fills the prefix of a caller-owned `Uint8Array`, returning
  its length. Reuse a nonempty buffer for bounded JavaScript transfer storage.
  It returns zero at EOF or for an empty target; the unused suffix is unchanged.
- `nextChunk()` returns a new, independently owned `Uint8Array` of at most
  65,536 bytes, or `undefined` at EOF. It can be mixed with `readInto`.
- `free()` releases all unread bytes. Read handles consume their contents and
  cannot rewind; create a new snapshot from the source to start over.

Byte blocks may split a UTF-8 code point. Use a streaming decoder if text is
needed. A consumer retaining every block or concatenating a complete String
still pays for that JavaScript storage.

For example, with an already-open Node file descriptor:

```js
const snapshot = session.resultChunks();
const buffer = new Uint8Array(65536);
try {
  for (let length; (length = snapshot.readInto(buffer)) !== 0;) {
    for (let offset = 0; offset < length;) {
      const written = fs.writeSync(fd, buffer, offset, length - offset);
      if (written <= 0) throw new Error('snapshot write made no progress');
      offset += written;
    }
  }
} finally {
  snapshot.free();
}
```

On sink failure discard the incomplete file and free the reader. The source is
unchanged by snapshot creation or consumption; a later new snapshot reflects
the source's state at that later time. `resultDiagnosticCount` on sessions and
replicas returns the total current top-level and strategy diagnostic count.
Snapshot JSON retains all diagnostics, including nonempty ones.

The resource worker uses a reusable buffer and retains every complete session
and independent replica output. Snapshot timing includes capture, reader
transfers and bookkeeping, excluding directly measured file IO; serialization
timing is that measured IO. Their sum accounts for the full export. Runtime
version/executable metadata and worker/artifact hashes bind each receipt.
CSV conversion groups at most 1,024 source rows into a temporary block while
preserving its original full input bytes.

The original 0.3.3-candidate Linux/Node 24.22.1 resource run exceeded its frozen
512-MiB single-session process budget in four 100,000-bar RSI trials. Full
encoded output Strings contributed to the peak; simply releasing parsed input
or switching file writes to async did not qualify the workload. The failed
receipts and intermediate controls remain retained. The new reader and reusable
buffer pass a focused Linux/Node 24.22.1 repair control: both original RSI inputs,
100,000 history bars plus the 10,000-bar tail, one session and two fresh repeats.
All four complete session/replica output files remain byte-identical to the
original controls. Process high-water memory is 461,185,024 / 461,529,088 bytes
for default RSI and 477,773,824 / 476,258,304 bytes for alternate RSI, against
the unchanged 536,870,912-byte ceiling. Snapshot plus file IO takes 1.18-1.42
seconds against 10 seconds. The retained focused receipt SHA-256 is
`4854265aac91422f12cdff48b0aa03a789118307b32d680fe51e5a8b02b5c3c2`.
This control does not replace the complete 216-trial resource qualification.
Complete encoded
snapshots still occupy memory proportional to output size in WASM; this API
does not promise constant overall memory or indefinite-session bounds.

The actual generated Node-binding tests compare complete Unicode JSON bytes,
short reads, mixed read methods, EOF/counters, interruption and retry, source
advancement, source destruction and diagnostic preservation. Numerical/script
semantics, full-matrix resources, other bindings and packaged host acceptance
remain distinct qualification scopes.
