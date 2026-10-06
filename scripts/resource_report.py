"""Validate compact resource reports without loading public output payloads.

Version 2 reports name complete sibling spool files. Runtime output schemas and
the independently recorded file hashes remain unchanged.
"""
from pathlib import Path

from requalify_core_scripts import read


def expected_spool_names(count, require_historical=False):
    kinds = ['live', 'replica']
    if require_historical:
        kinds += ['batch', 'incremental', 'same-context']
    return {f'{kind}-{index}.json' for kind in kinds for index in range(count)}


def validate_report(output, count, confirmed, require_historical=False, seed_count=None):
    output = Path(output)
    report = read(output)
    assert type(report.get('resourceReportVersion')) is int
    assert report['resourceReportVersion'] == 2, 'unsupported resource report version'
    assert type(report.get('confirmedBars')) is int and report['confirmedBars'] == confirmed

    def reference(value, relative):
        assert value == {'path': relative}, f'incorrect resource report reference: {value!r}'
        target = output.parent / relative
        # A sibling symlink must not redirect receipt verification elsewhere.
        assert target.resolve().is_relative_to(output.parent.resolve())
        assert target.is_file(), f'missing complete resource output: {relative}'

    metadata_name = output.name + '.metadata.json'
    reference(report.get('metadata'), metadata_name)
    metadata = read(output.parent / metadata_name)
    assert type(metadata.get('resourceMetadataVersion')) is int
    assert metadata['resourceMetadataVersion'] == 1
    assert type(metadata.get('resultCount')) is int and metadata['resultCount'] == count
    assert type(metadata.get('confirmedBars')) is int and metadata['confirmedBars'] == confirmed
    assert isinstance(report.get('results'), list) and len(report['results']) == count
    parts = output.name + '.parts/'
    for index, value in enumerate(report['results']):
        reference(value, parts + f'live-{index}.json')
    for name in expected_spool_names(count, require_historical):
        reference({'path': parts + name}, parts + name)
    if require_historical:
        controls = report.get('historicalContexts')
        assert isinstance(controls, list) and len(controls) == count
        assert report.get('historicalSameContextMatches') is True
        assert metadata.get('historicalSameContextMatches') is True
        assert type(report.get('historicalAppendMatches')) is bool
        assert report['historicalAppendMatches'] == metadata.get('historicalAppendMatches')
        matches = []
        for index, control in enumerate(controls):
            assert type(control.get('stream')) is int and control['stream'] == index
            assert type(control.get('matches')) is bool
            assert control.get('sameContextMatches') is True
            assert type(control.get('batchDatasetEnd')) is int
            assert control['batchDatasetEnd'] == confirmed - 1
            assert type(control.get('initialSeedDatasetEnd')) is int
            if seed_count is not None:
                assert control['initialSeedDatasetEnd'] == seed_count - 1
            matches.append(control['matches'])
            for field, kind in [('batch', 'batch'), ('incremental', 'incremental'),
                                ('sameContextIncremental', 'same-context')]:
                reference(control.get(field), parts + f'{kind}-{index}.json')
        assert report['historicalAppendMatches'] == all(matches)
    return report
