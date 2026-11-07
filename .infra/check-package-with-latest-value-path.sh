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
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
git_dir=$(git -C "$project_root" rev-parse --path-format=absolute --git-dir)
git_common_dir=$(git -C "$project_root" rev-parse --path-format=absolute --git-common-dir)
if [[ "$git_dir" != "$git_common_dir" ]]; then
    main_checkout=$(dirname "$git_common_dir")
else
    main_checkout=$project_root
fi
latest_value_path=$(cd "$main_checkout/../rs-value" && pwd -P)
[[ -f "$latest_value_path/Cargo.toml" ]] || {
    echo "error: latest qubit-value path dependency is unavailable at $latest_value_path" >&2
    exit 2
}

tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT
export CARGO_HOME="$tmp_dir/cargo-home"
export CARGO_TARGET_DIR="$tmp_dir/target"
unset CARGO_NET_OFFLINE
mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR"

"$python_cmd" - "$archive" "$tmp_dir/package-root" <<'PYCODE'
import pathlib
import sys
import tarfile

archive_path = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2]).parent / "unpacked"
with tarfile.open(archive_path) as archive:
    members = archive.getmembers()
    roots = {
        pathlib.PurePosixPath(member.name).parts[0]
        for member in members
        if pathlib.PurePosixPath(member.name).parts
    }
    if len(roots) != 1:
        raise SystemExit(f"error: expected one crate archive root, found {sorted(roots)}")
    root = roots.pop()
    if not all(
        pathlib.PurePosixPath(member.name).parts[0] == root
        for member in members
        if pathlib.PurePosixPath(member.name).parts
    ):
        raise SystemExit("error: crate archive contains entries outside its root")
    destination.mkdir()
    archive.extractall(destination, filter="data")
package_root = destination / root
manifest = package_root / "Cargo.toml"
if not manifest.is_file():
    raise SystemExit("error: crate archive does not contain Cargo.toml")
pathlib.Path(sys.argv[2]).write_text(str(package_root), encoding="utf-8")
PYCODE

package_root=$(cat "$tmp_dir/package-root")
# Cargo archives may carry the producer checkout's lockfile, including local
# path package identities. Resolve the normalized published manifest afresh.
rm -f -- "$package_root/Cargo.lock"
cd "$package_root"
metadata_file="$tmp_dir/metadata.json"
echo "Checking $(basename "$archive") with qubit-value path: $latest_value_path"
"$cargo_bin" --config "patch.crates-io.qubit-value.path=\"$latest_value_path\"" metadata \
    --manifest-path "$package_root/Cargo.toml" --format-version 1 > "$metadata_file"
"$python_cmd" - "$metadata_file" "$latest_value_path" <<'PYCODE'
import json
import pathlib
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    metadata = json.load(source)

packages = metadata.get("packages", [])
by_name = {}
for package in packages:
    by_name.setdefault(package.get("name"), []).append(package)

for name in ("qubit-redact",):
    matches = by_name.get(name, [])
    if not matches:
        raise SystemExit(f"error: expected {name} in resolved registry package metadata")
    if any(not (package.get("source") or "").startswith("registry+") for package in matches):
        raise SystemExit(f"error: {name} did not resolve exclusively from a registry")

latest_value_path = pathlib.Path(sys.argv[2]).resolve()
value_matches = by_name.get("qubit-value", [])
if len(value_matches) != 1:
    raise SystemExit(f"error: expected one qubit-value package, found {len(value_matches)}")
value_manifest = pathlib.Path(value_matches[0].get("manifest_path", "")).resolve()
if value_manifest != latest_value_path / "Cargo.toml":
    raise SystemExit(
        f"error: qubit-value did not resolve from the latest sibling path: {value_manifest}"
    )

root_id = metadata.get("resolve", {}).get("root")
root = next((package for package in packages if package.get("id") == root_id), None)
if root is None:
    raise SystemExit("error: Cargo metadata did not identify the unpacked crate root")
for package in packages:
    if package["id"] == root_id or package.get("name") == "qubit-value":
        continue
    if not (package.get("source") or "").startswith("registry+"):
        raise SystemExit(
            f"error: dependency {package.get('name')} resolved outside a registry: {package.get('source')}"
        )

print("Resolved external dependencies from registries:")
for package in sorted(packages, key=lambda item: (item.get("name", ""), item.get("version", ""))):
    if package["id"] != root_id and package.get("name") == "qubit-redact":
        print(f"  {package['name']} {package['version']} {package['source']}")
print(f"Resolved latest qubit-value path: {latest_value_path} ({value_matches[0]['version']})")
PYCODE

run_cargo() {
    echo "+ cargo $*"
    "$cargo_bin" --config "patch.crates-io.qubit-value.path=\"$latest_value_path\"" "$@"
}

run_cargo test --manifest-path "$package_root/Cargo.toml"
run_cargo test --locked --manifest-path "$package_root/Cargo.toml" --no-default-features --features json --test metadata_fuzz_roundtrip_tests
run_cargo test --locked --manifest-path "$package_root/Cargo.toml" --all-features --all-targets
run_cargo test --locked --manifest-path "$package_root/Cargo.toml" --all-features --doc
