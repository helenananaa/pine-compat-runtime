#!/usr/bin/env python3
"""Build the deterministic wheel release manifest.

Install tool dependencies with: python -m pip install -r scripts/requirements-release.txt
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import zipfile
from email.parser import BytesParser
from pathlib import Path

from packaging.version import InvalidVersion, Version


SCHEMA_VERSION = 1
DEFAULT_DISTRIBUTION = "pine-compat-runtime"
DEFAULT_MODULE = "pine_compat"


class ManifestError(ValueError):
    """Raised when release wheel metadata is inconsistent."""


def normalize_distribution(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def wheel_metadata(path: Path) -> dict[str, str]:
    try:
        with zipfile.ZipFile(path) as archive:
            members = [
                member
                for member in archive.namelist()
                if member.endswith(".dist-info/METADATA")
            ]
            if len(members) != 1:
                raise ManifestError(
                    f"{path.name}: expected one .dist-info/METADATA, found {len(members)}"
                )
            metadata = BytesParser().parsebytes(archive.read(members[0]))
    except zipfile.BadZipFile as exc:
        raise ManifestError(f"{path.name}: invalid wheel archive") from exc

    name = metadata.get("Name")
    version = metadata.get("Version")
    if not name or not version:
        raise ManifestError(f"{path.name}: wheel metadata lacks Name or Version")
    return {
        "distribution": normalize_distribution(name),
        "version": version,
        "python_requires": metadata.get("Requires-Python", ""),
    }


def wheel_tags(path: Path) -> tuple[str, str, str]:
    if path.suffix != ".whl":
        raise ManifestError(f"{path.name}: expected a .whl file")
    parts = path.name[:-4].rsplit("-", 3)
    if len(parts) != 4:
        raise ManifestError(f"{path.name}: malformed wheel filename")
    return parts[1], parts[2], parts[3]


def build_manifest(
    dist: Path,
    *,
    tag: str,
    commit: str,
    expected_wheel_count: int | None = None,
    distribution: str = DEFAULT_DISTRIBUTION,
    module: str = DEFAULT_MODULE,
) -> dict[str, object]:
    wheels = sorted(dist.glob("*.whl"))
    if not wheels:
        raise ManifestError(f"{dist}: no wheel files found")
    if expected_wheel_count is not None and len(wheels) != expected_wheel_count:
        raise ManifestError(
            f"{dist}: expected {expected_wheel_count} wheels, found {len(wheels)}"
        )

    expected_distribution = normalize_distribution(distribution)
    expected_version = tag.removeprefix("v")
    if not tag.startswith("v") or not expected_version:
        raise ManifestError(f"release tag must use v<version> form, got {tag!r}")
    try:
        release_version = Version(expected_version)
    except InvalidVersion as exc:
        raise ManifestError(f"invalid release version in tag {tag!r}") from exc
    expected_version = str(release_version)

    assets: list[dict[str, object]] = []
    python_requires: str | None = None
    seen_tags: set[tuple[str, str, str]] = set()
    for wheel in wheels:
        metadata = wheel_metadata(wheel)
        if metadata["distribution"] != expected_distribution:
            raise ManifestError(
                f"{wheel.name}: expected distribution {expected_distribution!r}, "
                f"got {metadata['distribution']!r}"
            )
        try:
            actual_version = Version(metadata["version"])
        except InvalidVersion as exc:
            raise ManifestError(f"{wheel.name}: invalid wheel version {metadata['version']!r}") from exc
        if actual_version != release_version:
            raise ManifestError(
                f"{wheel.name}: version {metadata['version']!r} does not match tag {tag!r}"
            )
        if python_requires is None:
            python_requires = metadata["python_requires"]
        elif metadata["python_requires"] != python_requires:
            raise ManifestError(f"{wheel.name}: inconsistent Requires-Python metadata")

        python_tag, abi_tag, platform_tag = wheel_tags(wheel)
        tag_set = (python_tag, abi_tag, platform_tag)
        if tag_set in seen_tags:
            raise ManifestError(f"{wheel.name}: duplicate wheel compatibility tags")
        seen_tags.add(tag_set)
        assets.append(
            {
                "filename": wheel.name,
                "python_tag": python_tag,
                "abi_tag": abi_tag,
                "platform_tag": platform_tag,
                "sha256": sha256(wheel),
                "size": wheel.stat().st_size,
            }
        )

    return {
        "schema_version": SCHEMA_VERSION,
        "channel": "prerelease" if release_version.is_prerelease else "stable",
        "distribution": expected_distribution,
        "module": module,
        "version": expected_version,
        "tag": tag,
        "commit": commit,
        "python_requires": python_requires or "",
        "assets": assets,
    }


def write_release_files(
    dist: Path,
    manifest: dict[str, object],
    *,
    manifest_name: str = "manifest.json",
    checksums_name: str = "SHA256SUMS",
) -> tuple[Path, Path]:
    manifest_path = dist / manifest_name
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

    checksum_paths = sorted([*dist.glob("*.whl"), manifest_path], key=lambda p: p.name)
    checksums_path = dist / checksums_name
    checksums_path.write_text(
        "".join(f"{sha256(path)}  {path.name}\n" for path in checksum_paths)
    )
    return manifest_path, checksums_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--expected-wheel-count", type=int)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        manifest = build_manifest(
            args.dist,
            tag=args.tag,
            commit=args.commit,
            expected_wheel_count=args.expected_wheel_count,
        )
        manifest_path, checksums_path = write_release_files(args.dist, manifest)
    except ManifestError as exc:
        raise SystemExit(f"release manifest error: {exc}") from exc

    print(
        f"release manifest passed: {len(manifest['assets'])} wheels; "
        f"wrote {manifest_path} and {checksums_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
