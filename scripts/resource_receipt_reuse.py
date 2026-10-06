"""Prove that a Python-only collector repair leaves Rust/WASM receipts valid."""
import ast
import copy
import hashlib
import zipfile
from pathlib import Path

from requalify_core_scripts import read, sha


RETAINABLE = {'Windows/rust', 'Windows/wasm', 'Linux/rust', 'Linux/wasm'}
RUNNER = 'scripts/product_resource_acceptance.py'


def validate_surfaces(surfaces):
    assert len(surfaces) == len(RETAINABLE) and set(surfaces) == RETAINABLE


def compatible_plans(previous, current):
    for key in ['coreCommit', 'sourceIdentitySha256', 'sourceState', 'budgetSha256',
            'budgets', 'cases', 'repetitions', 'processTimeoutSeconds', 'artifactPaths']:
        assert previous[key] == current[key], f'reuse changes {key}'
    assert previous['workerSourceHashes'].keys() == current['workerSourceHashes'].keys()
    for key in previous['workerSourceHashes']:
        if key != RUNNER:
            assert previous['workerSourceHashes'][key] == current['workerSourceHashes'][key]


def python_only_revision(previous, current):
    old = ast.parse(previous)
    new = ast.parse(current)
    worker = next(node for node in new.body
        if isinstance(node, ast.FunctionDef) and node.name == 'python_worker')
    original_worker = copy.deepcopy(worker)
    additions = ast.parse('''from resource_json import dump_public_json
serializer_sha = sha(Path(dump_public_json.__code__.co_filename))
assert serializer_sha == sha(Path(dump_public_json.__code__.co_filename)), 'serializer changed during measurement'
''').body
    counts = [0, 0, 0, 0, 0]
    body = []
    for node in worker.body:
        matched = False
        for index, addition in enumerate(additions):
            if ast.dump(node) == ast.dump(addition):
                counts[index] += 1
                matched = True
                break
        if not matched:
            body.append(node)
    worker.body = body
    for node in ast.walk(worker):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            if node.func.id == 'dump_public_json':
                assert not node.keywords and len(node.args) == 2
                assert ast.dump(node.args[1]) == ast.dump(ast.Name(id='stream', ctx=ast.Load()))
                assert isinstance(node.args[0], ast.Name) and node.args[0].id in ('result', 'replica_result')
                node.func = ast.Attribute(value=ast.Name(id='json', ctx=ast.Load()), attr='dump', ctx=ast.Load())
                node.keywords = ast.parse("json.dump(x, y, allow_nan=False, separators=(',', ':'))").body[0].value.keywords
                counts[3] += 1
            elif node.func.id == 'dict':
                for keyword in list(node.keywords):
                    if keyword.arg == 'serializerSourceSha256':
                        assert ast.dump(keyword.value) == ast.dump(ast.Name(id='serializer_sha', ctx=ast.Load()))
                        node.keywords.remove(keyword)
                        counts[4] += 1
    assert counts == [1, 1, 1, 2, 1], counts
    assert ast.dump(old) == ast.dump(new), 'changes extend beyond Python snapshot serialization'
    untouched = ast.Module(body=[node for node in old.body
        if not (isinstance(node, ast.FunctionDef) and node.name == 'python_worker')], type_ignores=[])
    return dict(untouchedControllerAstSha256=hashlib.sha256(ast.dump(untouched).encode()).hexdigest(),
        pythonWorkerSha256=hashlib.sha256(ast.dump(original_worker).encode()).hexdigest(), normalizations=counts)


def verify_reuse(repo, plan):
    if not plan.get('retainedSurfaces'):
        return None
    validate_surfaces(plan['retainedSurfaces'])
    previous_root = repo / plan['previousMatrixRoot']
    previous_path = previous_root / 'plan.json'
    assert sha(previous_path) == plan['previousPlanSha256']
    previous = read(previous_path)
    compatible_plans(previous, plan)
    archive = repo / plan['previousSourceArchive']
    assert sha(archive) == plan['previousSourceArchiveSha256']
    dependency_hashes = {}
    with zipfile.ZipFile(archive) as source:
        old_runner = source.read(RUNNER)
        for name in ['scripts/requalify_core_scripts.py']:
            digest = hashlib.sha256(source.read(name)).hexdigest()
            assert sha(repo / name) == digest
            dependency_hashes[name] = digest
    assert hashlib.sha256(old_runner).hexdigest() == previous['workerSourceHashes'][RUNNER]
    current_runner = (repo / RUNNER).read_bytes()
    assert hashlib.sha256(current_runner).hexdigest() == plan['workerSourceHashes'][RUNNER]
    proof = python_only_revision(old_runner.decode('utf-8-sig'), current_runner.decode('utf-8-sig'))
    proof.update(previousPlanSha256=sha(previous_path), previousSourceArchiveSha256=sha(archive),
        previousRunnerSha256=hashlib.sha256(old_runner).hexdigest(), currentRunnerSha256=sha(repo / RUNNER),
        unchangedControllerDependencyHashes=dependency_hashes)
    for name, digest in plan['workerAuxiliarySourceHashes'].items():
        assert sha(repo / name) == digest
    return proof


def receipt_root(repo, root, plan, system, surface):
    return repo / plan['previousMatrixRoot'] if system + '/' + surface in plan.get('retainedSurfaces', []) else root
