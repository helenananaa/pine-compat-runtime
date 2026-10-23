"""Promote immutable, resource-qualified CI wheels without rebuilding them.

Only release tooling and qualification documents may differ from the producer's
source commit. Runtime, package metadata, licenses and compiler inputs must be
the same Git objects. Normal branch/PR builds retain their existing build route.
"""
from __future__ import annotations

import argparse
from email.parser import BytesParser
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import urllib.request
import zipfile


PLATFORMS = {"windows-x86_64", "linux-x86_64", "macos-arm64", "macos-x86_64"}
RELEASE_FILES = {
    ".github/workflows/wheels.yml",
    "scripts/promote_qualified_wheels.py",
    "scripts/tests/test_promote_qualified_wheels.py",
}
RELEASE_DIRECTORIES = ("release-inputs/", "docs/releases/", "docs/qualifications/")


class PromotionError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise PromotionError(message)


def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def safe_relative(value):
    require(isinstance(value, str) and "\\" not in value and ":" not in value,
            "invalid relative evidence path")
    path = PurePosixPath(value)
    require(value and not path.is_absolute() and ".." not in path.parts
            and str(path) == value.rstrip("/"), "unsafe evidence path")
    return value


def positive_integer(value):
    return type(value) is int and value > 0


def digest_value(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def source_changes(repo, source_commit):
    require(re.fullmatch(r"[0-9a-f]{40}", source_commit) is not None, "invalid producer commit")
    dirty = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=no"], cwd=repo, text=True,
    ).strip()
    require(not dirty, "tracked release checkout has uncommitted changes")
    changed = subprocess.check_output(
        ["git", "diff", "--name-only", "--no-renames", source_commit, "HEAD", "--"],
        cwd=repo, text=True,
    ).splitlines()
    unexpected = [name for name in changed if name not in RELEASE_FILES
                  and not name.startswith(RELEASE_DIRECTORIES)]
    require(not unexpected, "runtime or build inputs changed since qualification: " + ", ".join(unexpected))
    return sorted(changed)


def load_plan(repo: Path, tag: str, repository: str):
    require(re.fullmatch(r"v[0-9][0-9A-Za-z.+-]*", tag) is not None, "invalid release tag")
    path = repo / "release-inputs" / (tag + ".json")
    if not path.exists():
        return None
    plan = read(path)
    require(type(plan.get("schemaVersion")) is int and plan["schemaVersion"] == 1
            and plan.get("tag") == tag, "invalid promotion schema/tag")
    require(plan.get("repository") == repository, "promotion repository mismatch")
    require(positive_integer(plan.get("workflowRun")), "invalid producer workflow run")
    require(set(plan.get("wheels", {})) == PLATFORMS, "promotion requires exactly four platform wheels")
    require(re.fullmatch(r"[0-9a-f]{40}", plan.get("sourceCommit", "")) is not None, "invalid producer commit")
    resource = plan["resourceQualification"]
    resource_path = (repo / safe_relative(resource["path"])).resolve()
    require(repo.resolve() in resource_path.parents, "qualification escapes repository")
    require(digest_value(resource.get("sha256")) and sha(resource_path) == resource["sha256"], "changed resource qualification")
    qualification = read(resource_path)
    require(qualification.get("status") == "passed" and qualification.get("sourceCommit") == plan["sourceCommit"], "resource source/status mismatch")
    require(qualification.get("freshTrials") == 216 and qualification.get("crossSurfaceMismatchGroups") == 0,
            "incomplete resource qualification")
    require(qualification.get("resourcePlatforms") == ["Windows", "Linux"]
            and qualification.get("resourceSurfaces") == ["rust", "python", "wasm"],
            "unexpected resource coverage")
    require(qualification.get("macOSBoundary") == "Installed-wheel compatibility tested in CI; no macOS resource measurement",
            "macOS qualification boundary missing")
    require(qualification.get("unverified") == [] and qualification.get("budgetAndExecutionFailures") == [],
            "resource qualification contains failures")
    for label, wheel in plan["wheels"].items():
        require(wheel.get("artifactName") == "wheel-" + label and positive_integer(wheel.get("artifactId")), "invalid artifact identity")
        name = safe_relative(wheel["filename"])
        require("/" not in name and name.endswith(".whl"), "invalid wheel filename")
        require(re.fullmatch(r"[A-Za-z0-9_.+-]+\.whl", name) is not None, "invalid wheel filename")
        require(digest_value(wheel.get("sha256")) and digest_value(wheel.get("archiveSha256")), "invalid artifact digest")
        require(positive_integer(wheel.get("size")), "invalid wheel size")
        require(isinstance(wheel.get("native"), dict) and wheel["native"], "native wheel identity missing")
        require(all(safe_relative(name) and digest_value(value) for name, value in wheel["native"].items()), "invalid native identity")
        require(qualification.get("pythonWheelHashes", {}).get(label) == wheel["sha256"], "qualification wheel mismatch")
    source_changes(repo, plan["sourceCommit"])
    return plan


def api_json(repository, suffix, token):
    require(re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) is not None, "invalid repository")
    require(bool(token), "GitHub token required for producer verification")
    request = urllib.request.Request(
        f"https://api.github.com/repos/{repository}/{suffix}",
        headers={"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def verify_producer(plan, label, fetch):
    run = fetch(f"actions/runs/{plan['workflowRun']}")
    require(run.get("repository", {}).get("full_name") == plan["repository"], "producer repository mismatch")
    require(run.get("name") == "Wheels" and run.get("head_sha") == plan["sourceCommit"], "producer source/workflow mismatch")
    require(run.get("path") == ".github/workflows/wheels.yml", "producer workflow path mismatch")
    require(run.get("status") == "completed" and run.get("conclusion") == "success", "producer did not complete successfully")
    wheel = plan["wheels"][label]
    artifact = fetch(f"actions/artifacts/{wheel['artifactId']}")
    require(artifact.get("id") == wheel["artifactId"] and artifact.get("name") == wheel["artifactName"], "producer artifact mismatch")
    require(artifact.get("expired") is False, "qualified artifact expired")
    require(artifact.get("digest") == "sha256:" + wheel["archiveSha256"], "qualified archive digest changed")
    origin = artifact.get("workflow_run", {})
    require(origin.get("id") == plan["workflowRun"] and origin.get("head_sha") == plan["sourceCommit"], "artifact source run mismatch")


def verify_wheel(plan, label, dist):
    expected = plan["wheels"][label]
    wheel = dist / expected["filename"]
    require(wheel.is_file() and wheel.stat().st_size == expected["size"] and sha(wheel) == expected["sha256"], "qualified wheel bytes changed")
    with zipfile.ZipFile(wheel) as archive:
        names = archive.namelist()
        require(len(set(names)) == len(names), "duplicate wheel members")
        require(all(safe_relative(name) for name in names), "unsafe wheel member")
        metadata_files = [name for name in names if name.count("/") == 1 and name.endswith(".dist-info/METADATA")]
        require(len(metadata_files) == 1, "wheel must have exactly one metadata file")
        metadata = metadata_files[0]
        value = BytesParser().parsebytes(archive.read(metadata))
        require(re.sub(r"[-_.]+", "-", value.get("Name", "")).lower() == "pine-compat-runtime", "wheel distribution mismatch")
        require(value.get("Version") == plan["tag"][1:], "wheel version mismatch")
        native = {name: hashlib.sha256(archive.read(name)).hexdigest() for name in names if name.endswith((".pyd", ".so"))}
        require(native == expected["native"], "qualified native bytes changed")
    return wheel


def record_release(repo, tag, plan, dist):
    if plan is None:
        return
    expected = {wheel["filename"] for wheel in plan["wheels"].values()}
    require({path.name for path in dist.glob("*.whl")} == expected, "release wheel set differs from qualified set")
    for label in PLATFORMS:
        verify_wheel(plan, label, dist)
    manifest_path = dist / "manifest.json"
    manifest = read(manifest_path)
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    require(manifest.get("tag") == tag and manifest.get("commit") == head, "release manifest source mismatch")
    expected_assets = {wheel["filename"]: (wheel["sha256"], wheel["size"])
                       for wheel in plan["wheels"].values()}
    assets = manifest.get("assets", [])
    require(len(assets) == len(expected_assets)
            and {row["filename"]: (row["sha256"], row["size"]) for row in assets} == expected_assets,
            "release manifest assets differ from qualified wheels")
    changes = source_changes(repo, plan["sourceCommit"])
    input_path = repo / "release-inputs" / (tag + ".json")
    resource_path = repo / plan["resourceQualification"]["path"]
    shutil.copyfile(input_path, dist / "qualified-inputs.json")
    shutil.copyfile(resource_path, dist / "resource-qualification.json")
    manifest["artifactPromotion"] = dict(
        sourceCommit=plan["sourceCommit"], producerRepository=plan["repository"], producerWorkflowRun=plan["workflowRun"],
        releaseCommit=head, releaseOnlyChanges=changes, inputsSha256=sha(input_path),
        resourceQualificationSha256=sha(resource_path), boundary="Immutable qualified artifacts; runtime/build inputs identical across producer and release source snapshots",
    )
    manifest_path.write_bytes((json.dumps(manifest, indent=2) + "\n").encode("utf-8"))
    assets = sorted([*dist.glob("*.whl"), manifest_path, dist / "qualified-inputs.json", dist / "resource-qualification.json"], key=lambda path: path.name)
    (dist / "SHA256SUMS").write_bytes("".join(f"{sha(path)}  {path.name}\n" for path in assets).encode("utf-8"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("select", "verify", "record"))
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--tag", default=os.environ.get("GITHUB_REF_NAME", ""))
    parser.add_argument("--platform", choices=sorted(PLATFORMS))
    parser.add_argument("--dist", type=Path, default=Path("dist"))
    parser.add_argument("--ref", default=os.environ.get("GITHUB_REF", ""))
    parser.add_argument("--repository", default=os.environ.get("GITHUB_REPOSITORY", ""))
    args = parser.parse_args()
    if args.ref.startswith("refs/tags/"):
        require(args.ref == "refs/tags/" + args.tag, "tag/ref mismatch")
    if args.mode == "select" and not args.ref.startswith("refs/tags/"):
        plan = None
    else:
        plan = load_plan(args.repo.resolve(), args.tag, args.repository)
    if args.mode == "select":
        values = {"enabled": "false"}
        if plan is not None:
            require(args.platform is not None, "platform required")
            verify_producer(plan, args.platform, lambda suffix: api_json(args.repository, suffix, os.environ.get("GH_TOKEN", "")))
            values.update(enabled="true", run_id=str(plan["workflowRun"]),
                          artifact_id=str(plan["wheels"][args.platform]["artifactId"]),
                          artifact_name=plan["wheels"][args.platform]["artifactName"], source_commit=plan["sourceCommit"])
        destination = os.environ.get("GITHUB_OUTPUT")
        if destination:
            with Path(destination).open("a", encoding="utf-8") as stream:
                stream.write("".join(f"{key}={value}\n" for key, value in values.items()))
        print(json.dumps(values))
    elif args.mode == "verify":
        require(plan is not None and args.platform is not None, "qualified promotion plan/platform required")
        require({path.name for path in args.dist.glob("*.whl")} == {plan["wheels"][args.platform]["filename"]}, "downloaded wheel set mismatch")
        verify_wheel(plan, args.platform, args.dist)
        print("Qualified immutable wheel verified:", args.platform)
    else:
        record_release(args.repo.resolve(), args.tag, plan, args.dist)


if __name__ == "__main__":
    main()
