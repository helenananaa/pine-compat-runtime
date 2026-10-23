'use strict';
const assert = require('node:assert/strict');

module.exports = function(modulePath) {
  const pine = require(modulePath);
  const title = '图😀é'.repeat(12000);
  const program = pine.compileScript(`//@version=6\nindicator("chunks")\nplot(close,title="${title}")\n`);
  const session = program.realtimeSessionWithRequestBarsAndInputOverrides('{}','{}');
  session.seed('time,open,high,low,close,volume\n0,1,2,0,1,1\n60000,2,3,1,2,1\n');
  const expected = Buffer.from(session.result());
  assert(expected.length > 65536);

  function readSnapshot(reader, expectedBytes, sourceAlive = true) {
    const chunks = [];
    const length = reader.byteLength;
    let read = 0;
    try {
      assert.equal(reader.remainingBytes, length);
      assert.equal(reader.readInto(new Uint8Array(0)),0);
      assert.equal(reader.remainingBytes,length);
      const target = new Uint8Array(4093);
      target.fill(170);
      for(let count;(count=reader.readInto(target))!==0;) {
        assert(count > 0 && count <= target.length);
        read += count;
        assert.equal(reader.remainingBytes, length-read);
        assert(target.subarray(count).every(byte=>byte===170));
        const copy = Buffer.from(target.subarray(0,count));
        // Returned blocks own their JS memory across unrelated WASM calls.
        if(sourceAlive)session.result();
        assert(Buffer.from(target.subarray(0,count)).equals(copy));
        chunks.push(copy);
        target.fill(170);
      }
      const combined = Buffer.concat(chunks);
      assert.equal(length, combined.length);
      assert(chunks.length > 1);
      assert(combined.equals(expectedBytes), 'full UTF-8 bytes differ from result()');
      assert.equal(JSON.parse(combined.toString('utf8')).plots[0].title, title);
      assert.equal(reader.nextChunk(), undefined);
      assert.equal(reader.remainingBytes, 0);
    } finally {reader.free();}
  }

  function exported(owner) {
    const text = owner.result();
    const publicResult = JSON.parse(text);
    assert.equal(owner.resultDiagnosticCount, publicResult.diagnostics.length
      + (publicResult.strategy?.diagnostics.length ?? 0));
    readSnapshot(owner.resultChunks(), Buffer.from(text));
  }

  exported(session);
  const mixed = session.resultChunks();
  const prefix = new Uint8Array(13);
  assert.equal(mixed.readInto(prefix),13);
  const second = mixed.nextChunk();
  assert.equal(second.length,65536-13);
  const mixedChunks = [Buffer.from(prefix),Buffer.from(second)];
  for(let chunk;(chunk=mixed.nextChunk())!==undefined;)mixedChunks.push(Buffer.from(chunk));
  assert(Buffer.concat(mixedChunks).equals(expected));
  assert.equal(mixed.remainingBytes,0);
  mixed.free();
  const abandoned = session.resultChunks();
  assert.equal(abandoned.nextChunk().length, 65536);
  abandoned.free();
  assert.equal(session.result(), expected.toString('utf8'));
  exported(session);

  const replica = session.replica();
  exported(replica);
  const owned = replica.intoResultChunks();
  assert.throws(() => replica.result(), 'consumed replica must be invalid');
  readSnapshot(owned, expected);
  const disposable = session.replica();
  const independent = disposable.resultChunks();
  disposable.free();
  readSnapshot(independent, expected);

  const captured = session.resultChunks();
  const bar = JSON.stringify({time:120000,open:3,high:4,low:2,close:3,volume:1});
  assert.notEqual(session.applyConfirmed(bar), 'null');
  assert.equal(session.confirmedBars, 3);
  readSnapshot(captured, expected);
  exported(session);

  const withDiagnostics = JSON.parse(session.result());
  withDiagnostics.diagnostics = [{code:'top-level',message:'retain me'}];
  withDiagnostics.strategy = {diagnostics:[{code:'strategy-one',message:'one'},{code:'strategy-two',message:'two'}]};
  const diagnosticReplica = new pine.RuntimeReplica(JSON.stringify(withDiagnostics),0n,0);
  assert.equal(diagnosticReplica.resultDiagnosticCount,3);
  exported(diagnosticReplica);
  diagnosticReplica.free();
  const afterSourceFree = session.resultChunks();
  const finalBytes = Buffer.from(session.result());
  session.free();
  readSnapshot(afterSourceFree,finalBytes,false);
  program.free();
};
