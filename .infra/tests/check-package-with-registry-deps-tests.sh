#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
script="$project_root/.infra/check-package-with-registry-deps.sh"
tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT
fake_bin="$tmp_dir/bin"
mkdir -p "$fake_bin"

cat > "$fake_bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == --version ]]; then
    echo "cargo 1.90.0 (controlled test double)"
    exit 0
fi
prior_calls=0
[[ ! -f "$FAKE_CARGO_LOG" ]] || prior_calls=$(wc -l < "$FAKE_CARGO_LOG")
printf '%s\t%s\t%s\t%s\t%s\n' "$PWD" "${CARGO_HOME:-}" "${CARGO_TARGET_DIR:-}" "${RUSTFLAGS:-}" "$*" >> "$FAKE_CARGO_LOG"
if [[ "$1" == metadata ]]; then
    if [[ "$prior_calls" -eq 0 && -e "$PWD/Cargo.lock" ]]; then
        echo "producer lockfile was not removed" >&2; exit 46
    fi
    touch "$PWD/Cargo.lock"
    printf '%s\n' "$PWD/Cargo.lock" > "$FAKE_LOCK_PATH"
    python3 - "$FAKE_METADATA_CASE" <<'PYJSON'
import json
import sys
case = sys.argv[1]
registry = "registry+https://github.com/rust-lang/crates.io-index"
packages = [
 {"id":"path+file:///tmp/unpacked/qubit-metadata-0.7.0#qubit-metadata@0.7.0","name":"qubit-metadata","version":"0.7.0","source":None},
 {"id":f"{registry}#qubit-redact@0.10.1","name":"qubit-redact","version":"0.10.1","source":registry},
 {"id":f"{registry}#qubit-value@0.13.1","name":"qubit-value","version":"0.13.1","source":registry},
 {"id":f"{registry}#serde@1.0.0","name":"serde","version":"1.0.0","source":registry},
]
if case == "path": packages[3]["id"] = "path+file:///tmp/serde#serde@1.0.0"; packages[3]["source"] = None
if case == "git": packages[3]["id"] = "git+https://example.invalid/serde#serde@1.0.0"; packages[3]["source"] = "git+https://example.invalid/serde"
if case == "other-registry": packages[3]["source"] = "registry+https://example.invalid/index"
if case == "missing-value": packages = [p for p in packages if p["name"] != "qubit-value"]
if case == "missing-redact": packages = [p for p in packages if p["name"] != "qubit-redact"]
print(json.dumps({"packages":packages,"resolve":{"root":packages[0]["id"]}}))
PYJSON
fi
case "${FAKE_CARGO_FAIL_ON:-}" in
  metadata) [[ "$1" != metadata ]] || exit 42 ;;
  test-default) [[ "$*" != *"test"* || "$*" == *"--locked"* ]] || exit 43 ;;
  test-all) [[ "$*" != *"--all-targets"* ]] || exit 44 ;;
  test-doc) [[ "$*" != *"--doc"* ]] || exit 45 ;;
esac
FAKE_CARGO
chmod +x "$fake_bin/cargo"

make_archive() {
    python3 - "$1" "${2:-one}" <<'PYARCHIVE'
import io, sys, tarfile
with tarfile.open(sys.argv[1], "w:gz") as archive:
    names = ["qubit-metadata-0.7.0/Cargo.toml", "qubit-metadata-0.7.0/Cargo.lock"]
    if sys.argv[2] == "two": names.append("second-root/Cargo.toml")
    if sys.argv[2] == "traversal": names.append("qubit-metadata-0.7.0/../../escape")
    for name in names:
        data = (b"[package]\nname='qubit-metadata'\nversion='0.7.0'\nedition='2024'\n" if name.endswith("Cargo.toml") else b"# producer lock\n")
        info = tarfile.TarInfo(name); info.size = len(data)
        archive.addfile(info, io.BytesIO(data))
PYARCHIVE
}
archive="$tmp_dir/qubit-metadata-0.7.0.crate"
make_archive "$archive"
export PATH="$fake_bin:$PATH"
export FAKE_CARGO_LOG="$tmp_dir/cargo.log"
export FAKE_LOCK_PATH="$tmp_dir/lock-path"
export FAKE_METADATA_CASE=valid

expect_failure() {
    local label=$1; shift
    if "$@" >"$tmp_dir/$label.out" 2>&1; then
        echo "expected failure: $label" >&2; exit 1
    fi
}
expect_failure no-args "$script"
expect_failure relative-path "$script" relative.crate
expect_failure missing-archive "$script" "$tmp_dir/missing.crate"

mkdir -p "$tmp_dir/injected-cargo-home"
printf '[source.crates-io]\nreplace-with = "poisoned"\n[source.poisoned]\nregistry = "https://example.invalid/index"\n' > "$tmp_dir/injected-cargo-home/config.toml"
CARGO_HOME="$tmp_dir/injected-cargo-home" CARGO_TARGET_DIR="$tmp_dir/injected-target" "$script" "$archive"
[[ $(wc -l < "$FAKE_CARGO_LOG") -eq 5 ]] || { echo "expected two metadata and three test commands" >&2; exit 1; }
awk -F '\t' '
    $1 !~ /\/unpacked\/qubit-metadata-0[.]7[.]0$/ { exit 1 }
    $2 == "" || $3 == "" || $2 == $3 || $4 != "" { exit 1 }
    $5 ~ /--config|patch[.]crates[.]io/ { exit 1 }
    NR == 1 && $5 !~ /metadata.*--format-version 1.*--all-features/ { exit 1 }
    NR == 2 && ($5 !~ /test/ || $5 ~ /--locked/) { exit 1 }
    NR == 3 && $5 !~ /--all-features.*--all-targets/ { exit 1 }
    NR == 4 && $5 !~ /--all-features.*--doc/ { exit 1 }
    NR == 5 && $5 !~ /metadata.*--locked.*--all-features/ { exit 1 }
    END { if (NR != 5) exit 1 }
' "$FAKE_CARGO_LOG" || { echo "Cargo invocation was not isolated or missing required arguments" >&2; exit 1; }
[[ ! -e "$(cat "$FAKE_LOCK_PATH")" ]] || { echo "producer Cargo.lock was not removed" >&2; exit 1; }

for scenario in path git other-registry missing-value missing-redact; do
    : > "$FAKE_CARGO_LOG"
    if FAKE_METADATA_CASE=$scenario "$script" "$archive" >"$tmp_dir/$scenario.out" 2>&1; then
        echo "expected metadata rejection: $scenario" >&2; exit 1
    fi
    [[ $(wc -l < "$FAKE_CARGO_LOG") -eq 1 ]] || { echo "tests ran after bad metadata: $scenario" >&2; exit 1; }
done

for roots in two traversal; do
    make_archive "$tmp_dir/$roots.crate" "$roots"
    : > "$FAKE_CARGO_LOG"
    expect_failure "$roots-archive" "$script" "$tmp_dir/$roots.crate"
    [[ ! -s "$FAKE_CARGO_LOG" ]] || { echo "Cargo ran for unsafe archive: $roots" >&2; exit 1; }
done

for failing in metadata test-default test-all test-doc; do
    : > "$FAKE_CARGO_LOG"
    if FAKE_CARGO_FAIL_ON=$failing "$script" "$archive" >"$tmp_dir/$failing.out" 2>&1; then
        echo "expected Cargo failure: $failing" >&2; exit 1
    else
        status=$?
        [[ $status -eq 42 || $status -eq 43 || $status -eq 44 || $status -eq 45 ]] || { echo "failure not propagated: $failing ($status)" >&2; exit 1; }
    fi
done

if CARGO_SOURCE_CRATES_IO_REPLACE_WITH=mirror "$script" "$archive" >"$tmp_dir/injected-source.out" 2>&1; then
    echo "expected inherited Cargo source injection to be rejected" >&2; exit 1
fi
if RUSTFLAGS='--cfg poisoned' "$script" "$archive" >"$tmp_dir/injected-rustflags.out" 2>&1; then
    echo "expected inherited RUSTFLAGS to be rejected" >&2; exit 1
fi
if RUSTC_WRAPPER="$tmp_dir/poisoned-wrapper" "$script" "$archive" >"$tmp_dir/injected-rustc-wrapper.out" 2>&1; then
    echo "expected inherited RUSTC_WRAPPER to be rejected" >&2; exit 1
fi
if RUSTC_BOOTSTRAP=1 "$script" "$archive" >"$tmp_dir/injected-rustc-bootstrap.out" 2>&1; then
    echo "expected inherited RUSTC_BOOTSTRAP to be rejected" >&2; exit 1
fi
if RUSTDOCFLAGS='--cfg poisoned' "$script" "$archive" >"$tmp_dir/injected-rustdocflags.out" 2>&1; then
    echo "expected inherited RUSTDOCFLAGS to be rejected" >&2; exit 1
fi

# Every invocation uses a disposable directory; the successful run's temp
# roots can be inferred from the fake Cargo log and must be gone afterward.
while IFS=$'\t' read -r cwd cargo_home target_dir rustflags args; do
    [[ ! -e "$cargo_home" && ! -e "$target_dir" ]] || { echo "temporary Cargo state was not cleaned" >&2; exit 1; }
done < "$FAKE_CARGO_LOG"

echo "package-with-registry-deps script tests passed"
