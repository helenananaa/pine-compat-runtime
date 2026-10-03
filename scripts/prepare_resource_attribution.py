"""Prepare an offline staged-memory probe from the public resource collector.

The derived tail is an exact event prefix, including provider updates. This is
a diagnostic slice, not a replacement for the frozen resource acceptance plan.
Build the emitted Cargo project in an initialized native toolchain environment.
"""
import argparse
import hashlib
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError(f"probe instrumentation anchor changed: {before}")
    return source.replace(before, after, 1)


def prefix(payload, tail_bars):
    # Daily provider events carry the day's opening timestamp, so filtering
    # events by timestamp would accidentally retain later forming observations.
    wanted = min(tail_bars, len(payload['tail']))
    if wanted < 1:
        raise ValueError('need a nonempty diagnostic tail')
    confirmed = 0
    events = []
    for event in payload['events']:
        events.append(event)
        confirmed += event['kind'] == 'confirmed'
        if confirmed == wanted:
            break
    if confirmed != wanted:
        raise ValueError('tail and event confirmation counts disagree')
    return dict(payload, tail=payload['tail'][:wanted], events=events)


def prepare(input_root, root, tail_bars):
    root.mkdir(parents=True, exist_ok=False)
    probe = root / 'probe'
    probe.mkdir()
    source_path = REPO / 'scripts/product_resource_probe.rs'
    source = source_path.read_text(encoding='utf-8')
    helper = REPO / 'crates/pine-runtime/examples/benchmark_support/memory.rs'
    (probe / 'memory.rs').write_bytes(helper.read_bytes())
    stage = '''
mod memory;
fn stage(name: &str, session: Option<&RealtimeRuntime<'_>>) {
    let m = memory::read();
    let profile = session.map(|s| format!("{:?}", s.confirmed_profile()));
    eprintln!("{}", serde_json::json!({"stage":name,"rssKiB":m.resident_kib,
        "commitKiB":m.commit_kib,"peakRssKiB":m.peak_resident_kib,
        "peakCommitKiB":m.peak_commit_kib,"memorySource":memory::SOURCE,"profile":profile}));
}
'''
    source = replace_once(source, 'fn main() -> Result', stage + '\nfn main() -> Result')
    source = replace_once(source, 'let args: Vec<String>', 'stage("startup",None);\n    let args: Vec<String>')
    source = replace_once(source, 'let count: usize', 'stage("parsedInput",None);\n    let count: usize')
    source = replace_once(source, 'replicas.push(session.replica()); sessions.push(session);',
        'stage("seeded",Some(&session));\n        replicas.push(session.replica());\n        stage("replicaCreated",Some(&session)); sessions.push(session);')
    source = replace_once(source, '// attribution: tailComplete', 'stage("tailComplete",sessions.first());')
    source = replace_once(source, '// attribution: replicasReleased', 'stage("replicasReleased",sessions.first());')
    source = replace_once(source, 'metrics.entry("snapshot".into())', 'stage("snapshotCreated",Some(session));\n        metrics.entry("snapshot".into())')
    source = replace_once(source, 'metrics.entry("serialization".into())', 'stage("serialized",Some(session));\n        metrics.entry("serialization".into())')
    source = replace_once(source, '// attribution: publicResultsCompared', 'stage("publicResultsCompared",sessions.first());')
    source = replace_once(source, '// attribution: sessionsReleased', 'stage("sessionsReleased",None);')
    source = replace_once(source, '// attribution: historicalControlsExecuted', 'stage("historicalControlsExecuted",None);')
    source = replace_once(source, '// attribution: reportAssembly', 'stage("reportAssembly",None);')
    (probe / 'main.rs').write_text(source, encoding='utf-8')
    manifest = '[package]\nname="attribution-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="attribution-probe"\npath="main.rs"\n[dependencies]\n'
    for crate in ('pine-runtime', 'pine-sema', 'pine-syntax'):
        manifest += f'{crate}={{path={json.dumps((REPO / "crates" / crate).as_posix())}}}\n'
    manifest += 'serde_json="1"\n'
    (probe / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    inputs = []
    for path in sorted(input_root.glob('*.json')):
        payload = json.loads(path.read_text(encoding='utf-8-sig'))
        if not isinstance(payload, dict) or not {'source', 'bars', 'tail', 'events', 'request'} <= payload.keys():
            continue
        output = root / path.name
        sliced = prefix(payload, tail_bars)
        output.write_text(json.dumps(sliced, separators=(',', ':')), encoding='utf-8')
        inputs.append(dict(file=path.name, originalSha256=sha(path), derivedSha256=sha(output),
            history=len(sliced['bars']), tail=len(sliced['tail']), events=len(sliced['events'])))
    receipt = dict(scope='diagnostic event prefixes; no resource acceptance', inputs=inputs,
        collectorSha256=sha(source_path), memoryHelperSha256=sha(helper), probeSha256=sha(probe / 'main.rs'))
    (root / 'preparation.json').write_text(json.dumps(receipt, indent=2), encoding='utf-8')
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-root', type=Path, required=True)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--tail-bars', type=int, default=64)
    args = parser.parse_args()
    if args.tail_bars < 1:
        parser.error('tail-bars must be positive')
    receipt = prepare(args.input_root, args.root, args.tail_bars)
    print(json.dumps(dict(prepared=len(receipt['inputs']), root=str(args.root))))


if __name__ == '__main__':
    main()
