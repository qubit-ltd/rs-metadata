#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
package_tmp=$(mktemp -d)
trap 'rm -rf -- "$package_tmp"' EXIT
export CARGO_TARGET_DIR="$package_tmp/target"

python3 -c 'import sys; raise SystemExit(sys.version_info < (3, 12))' || {
    echo "error: Python 3.12 or later is required to safely extract the package" >&2
    exit 2
}

cargo package --manifest-path "$project_root/Cargo.toml" --allow-dirty --no-verify
python3 - "$package_tmp" <<'PYCODE'
import pathlib
import sys
import tarfile

base = pathlib.Path(sys.argv[1])
archives = list((base / 'target/package').glob('qubit-metadata-*.crate'))
assert len(archives) == 1, archives
with tarfile.open(archives[0]) as archive:
    members = archive.getmembers()
    roots = {pathlib.PurePosixPath(member.name).parts[0] for member in members}
    assert len(roots) == 1, roots
    root = roots.pop()
    names = {member.name for member in members}
    for path in ['fuzz/support/bounded_encode.rs', 'tests/support/bounded_encode.rs']:
        assert f'{root}/{path}' in names, f'package missing {path}'
    assert f'{root}/fuzz/Cargo.toml' not in names
    assert not any(name.startswith(f'{root}/fuzz/fuzz_targets/') for name in names)
    destination = base / 'unpacked'
    destination.mkdir()
    archive.extractall(destination, filter='data')
(base / 'package-root').write_text(str(destination / root))
PYCODE

package_root=$(cat "$package_tmp/package-root")
cargo test --manifest-path "$package_root/Cargo.toml" --locked --no-default-features --features json --test metadata_fuzz_roundtrip_tests
cargo test --manifest-path "$package_root/Cargo.toml" --locked --all-features --all-targets
