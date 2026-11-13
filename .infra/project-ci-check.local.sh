#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
package_tmp=$(mktemp -d)
trap 'rm -rf -- "$package_tmp"' EXIT
export CARGO_TARGET_DIR="$package_tmp/target"

# Sibling checkouts may provide candidate versions newer than crates.io. Use
# them when available; otherwise resolve the published registry versions.
local_dependency_patches=()
for dependency in qubit-redact qubit-value; do
    sibling_crate=${dependency#qubit-}
    if [[ -f "$project_root/../rs-$sibling_crate/Cargo.toml" ]]; then
        local_dependency_patches+=(
            --config "patch.crates-io.$dependency.path=\"$project_root/../rs-$sibling_crate\""
        )
    fi
done

python3 -c 'import sys; raise SystemExit(sys.version_info < (3, 12))' || {
    echo "error: Python 3.12 or later is required to safely extract the package" >&2
    exit 2
}

bash "$project_root/.infra/tests/check-package-with-registry-deps-tests.sh"
cargo "${local_dependency_patches[@]}" package --manifest-path "$project_root/Cargo.toml" --allow-dirty --no-verify
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
    assert f'{root}/tests/support/bounded_encode.rs' in names, 'package missing the shared bounded-encoding oracle'
    assert not any(name.endswith('.tar.gz') for name in names), 'package includes raw benchmark archives'
    assert f'{root}/doc/design.md' in names
    assert f'{root}/doc/design.zh_CN.md' in names
    assert f'{root}/fuzz/Cargo.toml' not in names
    assert not any(name.startswith(f'{root}/fuzz/fuzz_targets/') for name in names)
    destination = base / 'unpacked'
    destination.mkdir()
    archive.extractall(destination, filter='data')
(base / 'package-root').write_text(str(destination / root))
(base / 'package-archive').write_text(str(archives[0]))
PYCODE

package_root=$(cat "$package_tmp/package-root")
python3 "$project_root/.infra/tests/check-user-guide-consumers-tests.py"
python3 "$project_root/.infra/tests/check-user-guide-consumers.py" "$package_root" "${local_dependency_patches[@]}"
cargo "${local_dependency_patches[@]}" test --manifest-path "$package_root/Cargo.toml" --locked --no-default-features --features json --test metadata_fuzz_roundtrip_tests
cargo "${local_dependency_patches[@]}" test --manifest-path "$package_root/Cargo.toml" --locked --all-features --all-targets
if [[ -f "$project_root/../rs-value/Cargo.toml" ]]; then
    echo "Checking archive with latest candidate qubit-value"
    "$project_root/.infra/check-package-with-latest-value-path.sh" "$(cat "$package_tmp/package-archive")"
else
    echo "No sibling qubit-value checkout; registry archive check will verify the published dependency"
fi
echo "Checking the same archive with published registry dependencies"
"$project_root/.infra/check-package-with-registry-deps.sh" "$(cat "$package_tmp/package-archive")"
