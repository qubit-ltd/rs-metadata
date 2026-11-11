#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 || "$1" != /* || ! -f "$1" ]]; then
    echo "usage: $0 <absolute-path-to-crate-archive>" >&2
    exit 2
fi
archive=$(cd "$(dirname "$1")" && printf '%s/%s\n' "$PWD" "$(basename "$1")")
python_cmd=${RS_INFRA_PYTHON:-python3}
command -v "$python_cmd" >/dev/null 2>&1 || {
    echo "error: Python 3.12 or later is required to safely extract the package" >&2
    exit 2
}
if ! "$python_cmd" -c 'import sys; raise SystemExit(sys.version_info < (3, 12))'; then
    echo "error: Python 3.12 or later is required to safely extract the package" >&2
    exit 2
fi
cargo_bin=$(command -v cargo) || {
    echo "error: Cargo is required to verify the package" >&2
    exit 2
}

# These variables can replace registry sources or inject compiler flags. Reject
# them explicitly so a successful run always means the plain published graph.
for variable in $(compgen -e); do
    if [[ "$variable" == CARGO_SOURCE_* || "$variable" == CARGO_REGISTRIES_*_INDEX || "$variable" == RUSTFLAGS || "$variable" == CARGO_ENCODED_RUSTFLAGS || "$variable" == CARGO_BUILD_RUSTFLAGS || "$variable" == RUSTC || "$variable" == RUSTC_WRAPPER || "$variable" == RUSTC_WORKSPACE_WRAPPER || "$variable" == RUSTC_BOOTSTRAP || "$variable" == RUSTDOC || "$variable" == RUSTDOCFLAGS || "$variable" == RUSTUP_TOOLCHAIN ]]; then
        echo "error: inherited $variable would alter registry resolution or build execution" >&2
        exit 2
    fi
done
for variable in $(compgen -e); do
    if [[ "$variable" == CARGO_* && "$variable" != CARGO_HOME && "$variable" != CARGO_TARGET_DIR ]]; then
        unset "$variable"
    fi
done

# A fresh home ignores user and workspace Cargo config while preserving normal
# proxy, certificate, and other non-Cargo network environment settings.
tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT
export CARGO_HOME="$tmp_dir/cargo-home"
export CARGO_TARGET_DIR="$tmp_dir/target"
mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR"
echo "$("$cargo_bin" --version)"
rustc_bin=$(command -v rustc) || {
    echo "error: rustc is required to verify the package" >&2
    exit 2
}
echo "$("$rustc_bin" --version)"

"$python_cmd" - "$archive" "$tmp_dir/package-root" <<'PYCODE'
import pathlib
import sys
import tarfile

archive_path = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2]).parent / "unpacked"
with tarfile.open(archive_path) as archive:
    members = archive.getmembers()
    roots = set()
    for member in members:
        parts = pathlib.PurePosixPath(member.name).parts
        if not parts or parts[0] in (".", "..") or pathlib.PurePosixPath(member.name).is_absolute():
            raise SystemExit(f"error: unsafe archive member path: {member.name!r}")
        roots.add(parts[0])
    if len(roots) != 1:
        raise SystemExit(f"error: expected one crate archive root, found {sorted(roots)}")
    root = roots.pop()
    destination.mkdir()
    try:
        archive.extractall(destination, filter="data")
    except (tarfile.TarError, OSError, ValueError) as error:
        raise SystemExit(f"error: unsafe crate archive: {error}") from error
package_root = destination / root
manifest = package_root / "Cargo.toml"
if not manifest.is_file():
    raise SystemExit("error: crate archive does not contain Cargo.toml")
pathlib.Path(sys.argv[2]).write_text(str(package_root), encoding="utf-8")
PYCODE

package_root=$(cat "$tmp_dir/package-root")
# A package archive can contain a lockfile from its producer checkout. Resolve
# the normalized published manifest afresh, without patches or source overrides.
rm -f -- "$package_root/Cargo.lock"
cd "$package_root"
metadata_file="$tmp_dir/metadata.json"
check_metadata() {
    "$python_cmd" - "$metadata_file" <<'PYCODE'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    metadata = json.load(source)
packages = metadata.get("packages", [])
resolve = metadata.get("resolve") or {}
root_id = resolve.get("root")
root_matches = [package for package in packages if package.get("id") == root_id]
if len(root_matches) != 1:
    raise SystemExit("error: Cargo metadata did not identify exactly one unpacked crate root")
root = root_matches[0]
if root.get("name") != "qubit-metadata" or root.get("source") is not None:
    raise SystemExit("error: Cargo metadata root is not the local qubit-metadata package")
by_name = {}
for package in packages:
    by_name.setdefault(package.get("name"), []).append(package)
for required in ("qubit-value", "qubit-redact"):
    if not by_name.get(required):
        raise SystemExit(f"error: expected {required} in resolved registry package metadata")
for package in packages:
    if package.get("id") == root_id:
        continue
    name = package.get("name", "<unknown>")
    source = package.get("source")
    if not isinstance(source, str) or not source.startswith("registry+"):
        raise SystemExit(f"error: dependency {name} resolved outside a registry: {source}")
    if source != "registry+https://github.com/rust-lang/crates.io-index":
        raise SystemExit(f"error: dependency {name} resolved from a non-crates.io registry: {source}")
print("Resolved external dependencies from crates.io:")
for package in sorted(packages, key=lambda item: (item.get("name", ""), item.get("version", ""))):
    if package.get("id") != root_id:
        print(f"  {package['name']} {package['version']} {package['source']}")
PYCODE
}

echo "Checking $(basename "$archive") with published crates.io dependencies"
cargo metadata --manifest-path "$package_root/Cargo.toml" --format-version 1 --all-features > "$metadata_file"
check_metadata

run_cargo() {
    echo "+ cargo $*"
    "$cargo_bin" "$@"
}
run_cargo test --manifest-path "$package_root/Cargo.toml"
run_cargo test --locked --manifest-path "$package_root/Cargo.toml" --all-features --all-targets
run_cargo test --locked --manifest-path "$package_root/Cargo.toml" --all-features --doc
# Check the generated lockfile's resolved graph after all tests as well.
cargo metadata --locked --manifest-path "$package_root/Cargo.toml" --format-version 1 --all-features > "$metadata_file"
check_metadata
