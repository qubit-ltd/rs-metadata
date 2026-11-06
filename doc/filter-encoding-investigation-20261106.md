# Filter JSON encoding investigation (RM-002)

## Scope and decision

This investigation measures filter JSON encoding for RM-002. It does not
confirm an end-to-end performance defect. RM-001 remains excluded. Public API,
V1 wire data, encoding budgets, and writer behavior were not changed. No
production serialization candidate was retained because the repeated timing
measurements were too noisy to establish the required benefit.

The allocator experiment shows that the current borrowed wire representation
still creates temporary allocations. It does not establish that removing them
would improve encoding time: the ordinary Serde path and bounded helper include
different work, and the Criterion measurements varied substantially across
runs. The evidence therefore does not meet the decision threshold for an
optimization.

## Environment and reproducibility

- Worktree: `codex/rm002-filter-encoding-20261106`, initial HEAD
  `e46fea9ae96f2ea44a49078c93d649dd0d6491a5`.
- Toolchain: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, target
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8.
- Host: Linux 6.8.0-147-generic, Intel Xeon E5-2680 v4 @ 2.40 GHz, 56 logical
  CPUs, 2 sockets, 14 cores per socket.
- Features: `json,filter`; release/bench profile (`opt-level=3`, LTO, one codegen
  unit); dependencies pinned by `Cargo.lock`.
- Local sibling path dependencies were resolved through the existing project
  Cargo config patches to the temporary worktree symlinks
  `/data/working/qubit/.worktrees/rs-metadata/rs-redact` and `rs-value`.
- Criterion raw results are in the ignored build output under
  `target/criterion/metadata_filter_json_encode/`. Five exploratory 127-leaf
  groups are saved as `baseline-1`, `baseline-2`, `pinned-1`, `isolated18-1`,
  and `pinned12-1`. Three formal maximum-valid 255-leaf groups are saved as
  `pinned12-255-1`, `pinned12-255-2`, and `pinned12-255-3`. Each group contains
  22 cases (11 samples × 2 paths). Earlier interrupted runs are excluded.
- All eight complete groups are preserved in
  `doc/filter-encoding-criterion-results.tar.gz`: 528 files (each group's 22
  `benchmark.json`, 22 `estimates.json`, and 22 `sample.json` files), 211,249
  compressed bytes. The archive contains only these eight complete labels.

The five exploratory runs used this base command, changing only the label:

```sh
cargo --locked --frozen \
  --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' \
  --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-metadata/rs-value"' \
  bench --features json,filter --bench metadata_benchmarks -- \
  metadata/filter_json_encode --save-baseline baseline-1
```

`baseline-1` and `baseline-2` ran without affinity. The other exploratory
labels used `taskset -c 16` (`pinned-1`), `taskset -c 18` (`isolated18-1`), and
`taskset -c 12` (`pinned12-1`). Each exited 0 with 22/22 cases.

The formal 255-leaf runs used the same configuration with `taskset -c 12`,
changing only the label:

```sh
taskset -c 12 cargo --locked --frozen \
  --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' \
  --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-metadata/rs-value"' \
  bench --features json,filter --bench metadata_benchmarks -- \
  metadata/filter_json_encode --save-baseline pinned12-255-1
```

The same command exited 0 with labels `pinned12-255-2` and `pinned12-255-3`;
each covered 22/22 cases. The CPU 18 exploratory run recorded
`mpstat -P 18,46 1 1`
at start (CPU 18 idle 92.00%, CPU 46 idle 98.02%) and finish (95.96%, 98.99%).
The earlier exploratory CPU 12 run recorded CPU 12/40 idle at 91.00%/95.96%
at start and 92.86%/94.00% at finish. The formal CPU 12 runs recorded
98.02%/98.00% idle at start and 92.08%/95.96% at finish for group 1;
100%/100% at start and 27.84%/91.00% at finish for group 2; and 98.00%/97.00%
at start and 83.67%/96.97% at finish for group 3. An external workspace test
was active during group 2, consistent with its CPU 12 iowait spike. Many cases
continued to report severe outliers despite the fixed affinity.

At several points Cargo unexpectedly rewrote `Cargo.lock` while these commands
were running, even with `--locked --frozen`. The repeated diff added a registry
`qubit-redact 0.10.0` entry and qualified two dependency references. The
worktree owner inspected and restored that generated diff to the initial clean
lockfile after each occurrence. No lockfile change is part of this
investigation. The Cargo metadata check using the symlink patch identities
passed with `--locked --offline` and left the lockfile unchanged.

## Samples and constraints

For each prepared filter, both `serde_json::to_vec(&filter)` and
`filter.to_json_vec_with_limits(default_json_encode_limits())` were verified to
produce the same output bytes before timing. Sample construction and validation
are outside the timed closure; the closure only encodes and consumes its output.

The requested 256-leaf flat Boolean sample cannot be built within the existing
hard 256-node limit: the operator counts as a node, so 256 leaves require 257
nodes. The largest valid flat sample has 255 leaves and one operator node.
Likewise, the requested 256-value membership sample exceeds the existing
128-value hard limit; the largest valid sample uses 128 values. Limits were
not widened.

The completed target matrix has 11 samples: 1 and 8 leaves; flat AND and OR
with 64 and 255 leaves; nested depth 4 and 16; and membership with 8, 64, and
128 values. Every sample is measured via both encoding paths. Five earlier
complete exploratory groups used 127 leaves before the maximum legal size was
confirmed; those results are retained separately and not presented as the
255-leaf matrix.

## Allocator experiment

`examples/filter_encode_allocations.rs` installs a forwarding `System`
allocator, verifies its counting window with a known allocation, warms each
encoding path, then measures 100 serial encodes per sample. Each encode has an
independent counting window. The summary below gives the per-encode median
calls/requested bytes; in this run, min, p95, and max equal the median for each
value. These are not peak or net live memory. `realloc` bytes count the new
requested size. Construction, preflight checks, and printing occur outside the
measurement windows.

| Sample | Output bytes | ordinary calls / bytes median | bounded calls / bytes median |
| --- | ---: | ---: | ---: |
| small_1 | 108 | 1 / 128 | 5 / 248 |
| small_8 | 374 | 5 / 1,472 | 9 / 1,592 |
| flat_and_64 | 2,332 | 11 / 14,016 | 15 / 14,136 |
| flat_or_64 | 2,331 | 11 / 14,016 | 15 / 14,136 |
| flat_and_255 | 9,172 | 15 / 57,024 | 19 / 57,144 |
| flat_or_255 | 9,171 | 15 / 57,024 | 19 / 57,144 |
| nested_4 | 287 | 7 / 1,376 | 11 / 1,496 |
| nested_16 | 902 | 20 / 4,128 | 24 / 4,248 |
| membership_8 | 298 | 5 / 1,088 | 9 / 1,208 |
| membership_64 | 1,640 | 10 / 5,952 | 14 / 6,072 |
| membership_128 | 3,204 | 12 / 12,096 | 16 / 12,216 |

Across all 100 individual trials of each sample, the bounded helper made four
more allocation requests and requested 120 more bytes per encode than ordinary Serde. The current wire
conversion allocates in both paths. These observations describe this isolated
experiment only; they do not attribute all requests to the wire tree or
predict end-to-end performance.

The allocator command was:

```sh
cargo --locked --frozen \
  --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' \
  --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-metadata/rs-value"' \
  run --release --features json,filter --example filter_encode_allocations
```

It exited 0. An earlier attempt failed because a 256-leaf sample exceeded the
hard node limit, and a later attempt failed because membership 256 exceeded
the hard 128-value limit. Both samples were corrected to the largest legal
inputs; these failures did not relax or modify any filter limits.

## Criterion results and interpretation

The five complete exploratory runs used 127-leaf flat samples. Their raw
estimates illustrate timing variability and are not mixed with the formal
255-leaf matrix:

| 127-leaf exploratory ordinary Serde median (µs) | baseline-1 | baseline-2 | pinned-1 (CPU 16) | isolated18-1 | pinned12-1 |
| --- | ---: | ---: | ---: | ---: | ---: |
| small_1 | 0.188 | 0.188 | 0.187 | 0.185 | 0.389 |
| small_8 | 1.151 | 1.970 | 1.136 | 1.117 | 1.138 |
| flat_and_64 | 10.128 | 9.069 | 6.663 | 6.263 | 6.325 |
| flat_and_127 | 13.157 | 20.747 | 12.043 | 12.259 | 12.256 |
| flat_or_127 | 12.212 | 21.854 | 23.868 | 12.447 | 22.480 |
| nested_16 | 4.078 | 3.185 | 2.994 | 3.004 | 3.010 |
| membership_128 | 6.350 | 6.525 | 6.257 | 6.337 | 6.479 |

The formal 255-leaf matrix has these exact `median.point_estimate` values from
Criterion `estimates.json` files:

| Sample and path | pinned12-255-1 median µs (outliers) | pinned12-255-2 median µs (outliers) | pinned12-255-3 median µs (outliers) |
| --- | ---: | ---: | ---: |
| flat_and_255 ordinary | 23.220 (13%) | 23.512 (11%) | 24.746 (9%) |
| flat_and_255 bounded | 65.987 (9%) | 64.439 (12%) | 64.712 (10%) |
| flat_or_255 ordinary | 24.515 (4%) | 24.257 (8%) | 24.956 (13%) |
| flat_or_255 bounded | 66.096 (16%) | 64.527 (8%) | 65.047 (17%) |
| nested_16 ordinary | 3.288 (1%) | 2.978 (12%) | 2.988 (1%) |
| nested_16 bounded | 7.888 (13%) | 7.934 (16%) | 7.816 (9%) |
| membership_128 ordinary | 6.308 (8%) | 6.947 (13%) | 6.439 (4%) |
| membership_128 bounded | 28.808 (7%) | 40.881 (0%) | 28.959 (6%) |

Outlier percentages come from Criterion output; 0% means no outliers were
reported for the sample. Complete raw estimates and samples are in the archive
described below.

Criterion reported repeated outliers in exploratory and formal runs. The
255-leaf table shows ordinary AND increasing about 6.6% across runs while
bounded AND varies little; ordinary and bounded OR show different trends.
Other samples show additional variation, including bounded membership_128 at
40.881 µs in group 2 versus 28.808 and 28.959 µs in groups 1 and 3. The
direction and size of the apparent changes are not consistent, and outliers
remain common. Thus the data do not establish the required benefit or absence
of regression.

## Decision against the optimization threshold

No candidate implementation was built. While the allocator experiment confirms
temporary allocation requests, the timing evidence is too noisy to establish
the required minimum 10% time improvement, consistent direction across three
groups, and Criterion-supported improvement. Therefore there is no basis to
claim the candidate would satisfy the required 20% reduction in large-sample
allocation calls or the no-repeatable-regression guard. No production code,
wire encoding, API, budget, or writer behavior was changed.

This is a conclusion of “optimization benefit not demonstrated under the
available measurements,” not a finding that optimization is unnecessary and
not a claim that an end-to-end bottleneck has been fixed. The benchmark measures
only these encoding calls; request-level or downstream end-to-end gains were
not measured.

## Verification record

| Command or check | Exit | Result |
| --- | ---: | --- |
| `rustc -Vv` | 0 | Rust 1.94.0, x86_64 Linux, LLVM 21.1.8. |
| `uname -a` and `lscpu` | 0 | Recorded host and CPU above. |
| Plain allocator command above, corrected sample matrix and independent windows | 0 | 11 samples × 2 paths; self-check and 100 windows per sample passed. Each per-encode call/byte distribution was constant in this run. |
| Eight complete Criterion commands above | 0 each | Five exploratory 127-leaf groups and three formal 255-leaf groups; each produced 22/22 raw `benchmark.json` files. |
| Criterion archive generation and validation | 0 | 528 files (eight complete groups × 22 cases × three JSON files), 211,249 compressed bytes; archive reopened and entry count/uniqueness checked. |
| Criterion run with invalid 256-leaf sample | 101 | Public hard node limit rejected 257 nodes; formal matrix uses the maximum legal 255 leaves. |
| Allocator run with invalid membership-256 sample | 101 | Public hard value limit rejected 129; sample corrected to 128 values. |
| First no-patch Cargo invocation | 101 | Worktree sibling `rs-redact` path missing; temporary symlinks were supplied by the worktree owner. |
| `cargo metadata --locked --offline --features json,filter` with symlink patches | 0 | Dependency metadata resolved; lockfile stayed clean at check time. |
| Initial `git diff --check` | 0 | No whitespace errors in the T1 changes. |
| `.infra/bin/align-ci.sh` (first run) | 1 | Style check rejected the allocator example for missing the repository license header; the required header was added. |
| `.infra/bin/align-ci.sh` (rerun) | 0 | Rust style and infra-ci alignment passed. The generated root lockfile diff was restored to the initial HEAD. |
| `cargo --locked --frozen --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-metadata/rs-value"' test` | 0 | Default feature tests passed; output reported 73 passing tests across enabled test and documentation suites. |
| `cargo --locked --frozen --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-value"' test --no-default-features --features filter` | 0 | Filter-only feature matrix passed, including filter, metadata, wire, redaction, and doctest suites. |
| `cargo --locked --frozen --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-value"' test --no-default-features --features json,filter` | 0 | JSON plus filter feature matrix passed, including filter, metadata writer, wire, redaction, and doctest suites. |
| `cargo --locked --frozen --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-value"' test --all-features --all-targets` | 0 | All targets passed, including the Criterion bench harness and all 22 final encoding cases. Cargo rewrote the root lockfile during this run; the generated diff was restored to the initial HEAD. |
| `cargo --locked --frozen --config 'patch.crates-io.qubit-redact.path="/data/working/qubit/.worktrees/rs-metadata/rs-redact"' --config 'patch.crates-io.qubit-value.path="/data/working/qubit/.worktrees/rs-value"' test --all-features --doc` (first attempt) | 101 | Frozen lock mismatch after the preceding all-targets lock rewrite; the lockfile was restored before retry. |
| Same all-features doc command (retry after restoring lockfile) | 0 | All 32 all-feature doctests passed. |
| `.infra/bin/ci-check.sh` | 0 | CI checks passed, including build and feature test matrices, three 10-second fuzz runs (`decode_metadata`, `decode_metadata_filter`, schema), packaged archive checks, dependency security scan, and its built-in coverage summary. Built-in summary: functions 99.75% (406/407), lines 96.97% (3075/3171), regions 94.20% (4463/4738). The script regenerated root and `fuzz/Cargo.lock` diffs; both were restored to the initial HEAD. It also generated ignored fuzz corpus files, which were removed after preserving tracked seed files. |
| `.infra/bin/coverage.sh` | 0 | Independent coverage run passed thresholds: functions ≥95% (99.75%, 406/407), lines >90% (96.97%, 3075/3171), regions >85% (94.20%, 4463/4738). It emitted a warning that 77 functions have mismatched data. This is separate from the `ci-check.sh` coverage summary above. Generated ignored coverage reports were removed after recording these results. |
| Final `git diff --check` | 0 | No whitespace errors after report and T3 updates. |

Cargo validation commands used the symlink path patches shown above, with
`--locked --frozen` before the subcommand. Lockfile rewrites described here
were generated by the validation tools, restored by the worktree owner to the
initial clean HEAD, and are not part of this change.
