#!/usr/bin/env python3
"""Run the empty-set example with only the dependencies listed in each guide."""

import os
import pathlib
import subprocess
import sys
import tempfile


def fenced_block(document: str, marker: str, language: str) -> str:
    """Extract exactly one fenced example inside a named documentation region."""
    start = f"<!-- {marker}:start -->"
    end = f"<!-- {marker}:end -->"
    if document.count(start) != 1 or document.count(end) != 1:
        raise ValueError(f"expected exactly one {marker} region")
    region = document.split(start, 1)[1].split(end, 1)[0].strip()
    prefix = f"```{language}\n"
    if not region.startswith(prefix) or not region.endswith("\n```"):
        raise ValueError(f"expected a complete {language} block in {marker}")
    return region[len(prefix) : -len("\n```")] + "\n"


def main() -> None:
    """Test a checkout or unpacked package, forwarding optional Cargo flags."""
    package_root = pathlib.Path(sys.argv[1]).resolve()
    with tempfile.TemporaryDirectory(prefix="metadata-guide-consumers-") as temporary:
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(pathlib.Path(temporary) / "target"))
        for index, name in enumerate(("user_guide.md", "user_guide.zh_CN.md")):
            document = (package_root / "doc" / name).read_text(encoding="utf-8")
            consumer = pathlib.Path(temporary) / str(index)
            (consumer / "src").mkdir(parents=True)
            dependencies = fenced_block(document, "installation-dependencies", "toml")
            (consumer / "Cargo.toml").write_text(
                '[package]\nname = "metadata-guide-consumer"\nversion = "0.0.0"\n'
                'edition = "2024"\n\n' + dependencies,
                encoding="utf-8",
            )
            (consumer / "src" / "main.rs").write_text(
                fenced_block(document, "empty-set-example", "rust"), encoding="utf-8"
            )
            print(f"Checking consumer from {name}", flush=True)
            # A path patch selects the artifact under test without exposing its
            # own dependencies to the consumer's Rust extern prelude.
            subprocess.run(
                [
                    "cargo",
                    *sys.argv[2:],
                    "--config",
                    f'patch.crates-io.qubit-metadata.path="{package_root}"',
                    "run",
                    "--manifest-path",
                    str(consumer / "Cargo.toml"),
                ],
                check=True,
                env=environment,
            )


if __name__ == "__main__":
    main()
