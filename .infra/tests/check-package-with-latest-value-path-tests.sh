#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT
fake_bin="$tmp_dir/bin"
mkdir -p "$fake_bin"

cat > "$fake_bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
set -euo pipefail
count_file=${FAKE_CARGO_LOG}.count
count=0
[[ ! -f "$count_file" ]] || count=$(cat "$count_file")
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
printf '%s\t%s\t%s\t%s\n' "$PWD" "$CARGO_HOME" "$CARGO_TARGET_DIR" "$*" >> "$FAKE_CARGO_LOG"
if [[ "$1" == --config ]]; then
    value_path=${2#patch.crates-io.qubit-value.path=}
    value_path=${value_path%\"}
    value_path=${value_path#\"}
    shift 2
fi
if [[ "${1:-}" == metadata ]]; then
    python3 - "$value_path" <<'PYJSON'
import json
import sys
path = sys.argv[1]
print(json.dumps({"packages":[
 {"id":"path+file:///tmp/package#qubit-metadata@0.7.0","name":"qubit-metadata","version":"0.7.0","source":None},
 {"id":"registry+https://github.com/rust-lang/crates.io-index#qubit-redact@0.10.1","name":"qubit-redact","version":"0.10.1","source":"registry+https://github.com/rust-lang/crates.io-index"},
 {"id":f"path+file://{path}#qubit-value@0.13.1","name":"qubit-value","version":"0.13.1","source":None,"manifest_path":f"{path}/Cargo.toml"}],
 "resolve":{"root":"path+file:///tmp/package#qubit-metadata@0.7.0"}}))
PYJSON
fi
if [[ "${FAKE_CARGO_FAIL_ON:-}" == "$count" ]]; then
    exit 42
fi
FAKE_CARGO
chmod +x "$fake_bin/cargo"

make_archive() {
    local archive=$1 roots=${2:-one}
    python3 - "$archive" "$roots" <<'PYCODE'
import io
import sys
import tarfile

with tarfile.open(sys.argv[1], "w:gz") as archive:
    names = ["qubit-metadata-0.7.0/Cargo.toml"]
    if sys.argv[2] == "two":
        names.append("second-root/Cargo.toml")
    for name in names:
        content = b"[package]\nname='qubit-metadata'\nversion='0.7.0'\nedition='2024'\n"
        info = tarfile.TarInfo(name)
        info.size = len(content)
        archive.addfile(info, io.BytesIO(content))
PYCODE
}

archive="$tmp_dir/qubit-metadata-0.7.0.crate"
make_archive "$archive"
export PATH="$fake_bin:$PATH"
export FAKE_CARGO_LOG="$tmp_dir/cargo.log"
"$project_root/.infra/check-package-with-latest-value-path.sh" "$archive"
[[ $(wc -l < "$FAKE_CARGO_LOG") -eq 5 ]] || { echo "expected metadata plus four Cargo test commands" >&2; exit 1; }
awk -F '\t' '
    $1 !~ /\/unpacked\/qubit-metadata-0.7.0$/ { exit 1 }
    $2 == "" || $3 == "" || $2 == $3 { exit 1 }
    $4 !~ /--config patch[.]crates-io[.]qubit-value[.]path=/ { exit 1 }
    END { if (NR != 5) exit 1 }
' "$FAKE_CARGO_LOG" || { echo "Cargo invocation was not isolated or lacked the qubit-value path patch" >&2; exit 1; }

rm -f "$FAKE_CARGO_LOG" "$FAKE_CARGO_LOG.count"
if FAKE_CARGO_FAIL_ON=2 "$project_root/.infra/check-package-with-latest-value-path.sh" "$archive"; then
    echo "expected Cargo failure to propagate" >&2
    exit 1
else
    status=$?
    [[ $status -eq 42 ]] || { echo "expected exit 42, got $status" >&2; exit 1; }
fi
[[ $(wc -l < "$FAKE_CARGO_LOG") -eq 2 ]] || { echo "Cargo continued after a failed test" >&2; exit 1; }

multi_root="$tmp_dir/multi-root.crate"
make_archive "$multi_root" two
rm -f "$FAKE_CARGO_LOG" "$FAKE_CARGO_LOG.count"
if "$project_root/.infra/check-package-with-latest-value-path.sh" "$multi_root"; then
    echo "expected multi-root archive to be rejected" >&2
    exit 1
fi
[[ ! -e "$FAKE_CARGO_LOG" ]] || { echo "Cargo ran for invalid archive" >&2; exit 1; }

if "$project_root/.infra/check-package-with-latest-value-path.sh" relative.crate >/dev/null 2>&1; then
    echo "expected relative archive path to be rejected" >&2
    exit 1
fi

echo "package-with-latest-value-path script tests passed"
