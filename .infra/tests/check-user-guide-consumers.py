#!/usr/bin/env python3
"""Run representative documentation examples as independent Cargo consumers."""

import os
import pathlib
import re
import subprocess
import sys
import tempfile

# Each entry binds code and dependencies in the same document. Keep this list
# explicit: a broad consumer with merged features would hide missing docs deps.
CASES = (
    ("README.md", "readme-json-roundtrip-example", "readme-json-dependencies"),
    ("README.zh_CN.md", "readme-json-roundtrip-example", "readme-json-dependencies"),
    ("doc/user_guide.md", "core-read-example", "core-dependencies"),
    ("doc/user_guide.zh_CN.md", "core-read-example", "core-dependencies"),
    ("doc/user_guide.md", "chunk-workflow-example", "installation-dependencies"),
    ("doc/user_guide.zh_CN.md", "chunk-workflow-example", "installation-dependencies"),
    ("doc/user_guide.md", "empty-set-example", "installation-dependencies"),
    ("doc/user_guide.zh_CN.md", "empty-set-example", "installation-dependencies"),
    ("doc/user_guide.md", "query-translation-example", "filter-dependencies"),
    ("doc/user_guide.zh_CN.md", "query-translation-example", "filter-dependencies"),
    ("doc/user_guide.md", "bounded-json-decoding-example", "bounded-json-decode-dependencies"),
    ("doc/user_guide.zh_CN.md", "bounded-json-decoding-example", "bounded-json-decode-dependencies"),
    ("doc/user_guide.md", "directional-json-limits-example", "budget-json-dependencies"),
    ("doc/user_guide.zh_CN.md", "directional-json-limits-example", "budget-json-dependencies"),
    ("doc/user_guide.md", "bounded-json-encoding-example", "budget-json-dependencies"),
    ("doc/user_guide.zh_CN.md", "bounded-json-encoding-example", "budget-json-dependencies"),
    ("doc/user_guide.md", "bounded-filter-json-encoding-example", "filter-json-dependencies"),
    ("doc/user_guide.zh_CN.md", "bounded-filter-json-encoding-example", "filter-json-dependencies"),
)


def fenced_block(document: str, marker: str, language: str) -> str:
    """Extract exactly one complete fenced block from an ordered marker region."""
    start = f"<!-- {marker}:start -->"
    end = f"<!-- {marker}:end -->"
    if document.count(start) != 1 or document.count(end) != 1:
        raise ValueError(f"expected exactly one ordered {marker} region")
    start_at = document.index(start)
    end_at = document.index(end)
    if start_at >= end_at:
        raise ValueError(f"reversed {marker} region")
    region = document[start_at + len(start) : end_at].strip()
    blocks = re.findall(r"(?m)^```([^\n]*)\n.*?^```\s*$", region, re.DOTALL)
    if len(blocks) != 1 or blocks[0] != language:
        raise ValueError(f"expected exactly one {language} fenced block in {marker}")
    prefix = f"```{language}\n"
    if not region.startswith(prefix) or not region.endswith("\n```"):
        raise ValueError(f"expected a complete {language} block in {marker}")
    return region[len(prefix) : -len("\n```")] + "\n"


def execute(package_root: pathlib.Path, cargo_args: list[str], run=subprocess.run) -> None:
    """Generate and run every independent consumer under a temporary directory."""
    with tempfile.TemporaryDirectory(prefix="metadata-guide-consumers-") as temporary:
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(pathlib.Path(temporary) / "target"))
        for index, (document_name, code_marker, dependency_marker) in enumerate(CASES):
            document = (package_root / document_name).read_text(encoding="utf-8")
            try:
                code = fenced_block(document, code_marker, "rust")
                dependencies = fenced_block(document, dependency_marker, "toml")
            except ValueError as error:
                raise ValueError(f"{document_name}: {error}") from error
            consumer = pathlib.Path(temporary) / f"{index:02d}-{re.sub(r'[^a-z0-9]+', '-', code_marker.lower()).strip('-')}"
            (consumer / "src").mkdir(parents=True)
            package_name = f"metadata-guide-consumer-{index:02d}"
            (consumer / "Cargo.toml").write_text(
                f'[package]\nname = "{package_name}"\nversion = "0.0.0"\nedition = "2024"\n\n'
                + dependencies,
                encoding="utf-8",
            )
            (consumer / "src" / "main.rs").write_text(code, encoding="utf-8")
            print(
                f"Checking {document_name}: code={code_marker}, dependencies={dependency_marker}",
                flush=True,
            )
            # Patch only the tested package; its own dependencies never become
            # direct dependencies of the generated consumer.
            run(
                [
                    "cargo",
                    *cargo_args,
                    "--config",
                    f'patch.crates-io.qubit-metadata.path="{package_root}"',
                    "run",
                    "--manifest-path",
                    str(consumer / "Cargo.toml"),
                ],
                check=True,
                env=environment,
            )


def main() -> None:
    """Test a checkout or unpacked package, forwarding optional Cargo flags."""
    if len(sys.argv) < 2:
        raise SystemExit("usage: check-user-guide-consumers.py package_root [Cargo flags...]")
    execute(pathlib.Path(sys.argv[1]).resolve(), sys.argv[2:])


if __name__ == "__main__":
    main()
