# RSM-002 Metadata Validation Cost Investigation

Status: three release measurement windows, three allocation runs, and a
structured candidate-equivalence probe completed. The performance and probe
gates pass; T2 recommends the minimal validation-only optimization for T3.
T2 did not change the public API or downstream source.

## Scope and downstream context

The downstream `rs-task` request encoder currently calls
`Metadata::to_json_vec_with_limits` only to validate request metadata, then
drops the encoded bytes. Read-only inspection of the original `rs-task` tree at
HEAD `fa3461a079178c4e0f17d7db8d2565b0c02ca3a0` confirmed:

- `MAX_TASK_METADATA_ENTRIES` is 32 and
  `MAX_TASK_METADATA_BYTES` is `16 * 1024`.
- `TaskRequest::encode` rejects excess entries before JSON validation and maps
  output-budget errors to `MetadataTooLarge`.
- Model tests cover 33 entries, an oversized value, and exact encoded byte
  lengths at the limit and one byte above it using
  `metadata_with_encoded_size(target)`.

No downstream source was modified. Its original worktree was clean apart from
two pre-existing untracked design/plan documents.

## Prepared measurement

`benches/metadata_benchmarks.rs` adds the Criterion group
`metadata/validation_only`. Inputs are constructed before timed iterations. It
compares the current `to_json_vec_with_limits` path with a candidate that
performs the same wire-contract and `ValueWireEncodePreflight` checks, then
calls `JsonEncoder::write_incremental` against `std::io::sink()`.

Samples include empty metadata; scalar and ordinary text values at 1, 8, and
32 entries; escaped/control text, multibyte text, and nested JSON; and successful
near-cap plus over-cap payloads. The near/over-cap names use lengths measured
from actual serialized bytes. `examples/metadata_validation_allocations.rs`
uses 100 independent allocation windows per path and sample, with input setup,
output-length calculation, warm-up, and reporting outside each counting window.
It reports allocation calls and requested bytes (min, median, p95, max).

The incremental sink is only a performance candidate. Although both paths use
the `qubit-json` shared serializer and accounting, a separate structured probe
is required before promoting it to a public API. That probe and its result are
recorded below.

## Environment and dependency baseline

- `rs-metadata` HEAD: `e189da16b9be6cb3ed4ec4132818b9a635f34574`.
- Rust: `rustc 1.94.0 (4a4ef493e 2026-03-02)`; Cargo `1.94.0`.
- Host: Linux x86_64; Intel Xeon E5-2680 v4 @ 2.40 GHz, 2 sockets, 14 cores
  per socket, 2 threads per core (56 logical CPUs). Frequency scaling was
  enabled at observation time.
- Locked and resolved package versions: `qubit-budget 0.10.2`,
  `qubit-json 0.12.2`, `qubit-redact 0.10.1`, `qubit-value 0.13.0`;
  `cargo metadata --format-version 1 --features json,filter --locked` exited 0
  and confirmed crates.io registry sources for all four packages.
- The isolated dependency worktrees are pinned at `rs-value`
  `6299b12d7b145199f7d4e718531dec4581a68631` and `rs-redact`
  `d3116d19fc40d6e23db0fdca4db3748d0dede7d8`; both were clean and detached at
  inspection. The benchmark dependency graph resolved from crates.io; the
  local candidate worktrees were used only by the package-consumption hook.

## Measurement record

The three Criterion runs were serial and each exited 0. Each was followed by
one release allocation run, which also exited 0. The shared machine was not
exclusive: unrelated multi-repository tests/coverage builds continued during
the windows. It has 56 logical CPUs; load averages remained high and varied.
The timing values below are evidence from this loaded host, not stable latency
promises. The Criterion artifacts retain per-sample `estimates.json`,
`sample.json`, `benchmark.json`, and `tukey.json` under
`target/criterion/metadata_validation_only/{to_vec,incremental_sink}/<sample>/rsm-validation-window-N/`.

| Window | First/last Criterion sample artifact (UTC) | Load average snapshot (1/5/15 min) | Criterion command exit | Allocation command exit |
| --- | --- | --- | --- | --- |
| 1 | 04:35:14.590–04:38:34.788 | 39.38 / 37.68 / 29.52 at 04:33:07, before compile/run | 0 | 0 |
| 2 | 04:40:05.493–04:43:30.326 | 52.66 / 51.42 / 39.52 at 04:41:38 | 0 | 0 |
| 3 | 04:44:13.274–04:47:24.337 | 30.10 / 43.78 / 38.65 at 04:44:19 | 0 | 0 |

Exact commands, run serially:

```text
cargo bench --features json,filter --bench metadata_benchmarks -- metadata/validation_only --save-baseline rsm-validation-window-1
cargo run --release --features json --example metadata_validation_allocations
cargo bench --features json,filter --bench metadata_benchmarks -- metadata/validation_only --save-baseline rsm-validation-window-2
cargo run --release --features json --example metadata_validation_allocations
cargo bench --features json,filter --bench metadata_benchmarks -- metadata/validation_only --save-baseline rsm-validation-window-3
cargo run --release --features json --example metadata_validation_allocations
```

The first bench command compiled the optimized bench profile in 1m41s, after
waiting for Cargo package-cache locks; the first allocation command compiled
the optimized release example in 1m01s. Later commands reused those artifacts.
Compilation is outside Criterion's measured iteration windows. The only
compiler warning was the existing unconditional `DataType` import being unused
without the `schema` feature.

Criterion slope point estimates are reported below in microseconds as
`to_vec / incremental_sink`. The six ordinary nonempty samples are the typical
set used for the gate. For each window, the median of their six per-sample
percentage improvements was 19.9%, 47.8%, and 37.1%, respectively; every one of
the six samples was faster with the candidate in all three windows. No ordinary
nonempty sample had a repeated regression above 5%.

| Sample (encoded bytes) | Window 1 (µs) | Window 2 (µs) | Window 3 (µs) |
| --- | ---: | ---: | ---: |
| empty (25) | 0.385 / 0.211 | 0.325 / 0.307 | 0.335 / 0.217 |
| scalar, 1 entry (53) | 1.259 / 0.548 | 1.220 / 0.502 | 0.657 / 0.476 |
| text, 1 entry (83) | 0.745 / 0.552 | 1.244 / 0.610 | 0.983 / 0.520 |
| scalar, 8 entries (256) | 4.853 / 2.652 | 8.130 / 2.187 | 5.580 / 2.151 |
| text, 8 entries (496) | 3.262 / 2.807 | 3.126 / 2.516 | 3.247 / 2.345 |
| scalar, 32 entries (974) | 10.215 / 9.543 | 19.393 / 10.741 | 10.182 / 8.273 |
| text, 32 entries (1,912) | 10.915 / 10.658 | 11.792 / 11.661 | 16.313 / 8.723 |
| escaped, multibyte, nested JSON (204) | 3.094 / 2.029 | 3.195 / 2.715 | 2.988 / 1.922 |
| near-cap success (16,376) | 31.544 / 31.523 | 20.678 / 16.509 | 23.666 / 22.158 |
| over-cap rejection (17,336) | 22.102 / 25.761 | 29.836 / 16.649 | 21.933 / 17.438 |

The allocation command performs 100 independent counting windows for each
path/sample. Results were identical in all three runs; the table records the
per-window median (min, p95, and max matched these values too). Requested bytes
are allocator request sizes, not retained heap size.

| Sample | Bytes | `to_vec` alloc calls / requested bytes | `incremental_sink` alloc calls / requested bytes |
| --- | ---: | ---: | ---: |
| empty | 25 | 3 / 56 | 0 / 0 |
| scalar, 1 entry | 53 | 4 / 120 | 0 / 0 |
| text, 1 entry | 83 | 5 / 248 | 0 / 0 |
| scalar, 8 entries | 256 | 6 / 504 | 0 / 0 |
| text, 8 entries | 496 | 7 / 1,016 | 0 / 0 |
| scalar, 32 entries | 974 | 8 / 2,040 | 0 / 0 |
| text, 32 entries | 1,912 | 9 / 4,088 | 0 / 0 |
| escaped, multibyte, nested JSON | 204 | 9 / 1,192 | 3 / 688 |
| near-cap success | 16,376 | 10 / 33,510 | 0 / 0 |
| over-cap rejection | 17,336 | 14 / 35,515 | 4 / 115 |

## Structured equivalence probe and decision

The measured candidate clears the quantitative gates on these samples: the
median improvement across six typical nonempty samples exceeded 10% in every
window (19.9%, 47.8%, 37.1%), direction was consistent, allocation cost fell,
and no ordinary nonempty sample had a repeated regression above 5%. The
near-cap successful sample was neutral or faster in every window. The
over-cap rejection candidate was slower in window 1 but faster in windows 2
and 3; this was not a repeated regression and is not a typical successful
request.

The release allocation example also runs a structured equivalence probe before
its allocation windows. It passed 20 cases with exit status 0. It compares
successful candidate output bytes and structured error signatures: budget
resource, quantity resource, V1 wire kind/value/maximum, syntax category, and
exact `JsonSerializationErrorKind`. Covered cases include empty/scalar data,
escaping, multibyte text, nested JSON, non-finite float, 4097 V1 entries,
257-byte V1 key, all eight JSON value-budget resources, output cap-1/cap/cap+1,
wire plus value-budget violations, value-budget plus output violations, and
both orderings of output overflow versus later non-finite serialization. For
the Metadata V1 DTO, NaN is reported as the exact
`CustomSerialization` kind. The 4097-entry result is `Entries / 4097 / 4096`;
the separate downstream rs-task limit remains 32 entries.

The first probe compile attempt failed because the key fixture passed `String`
to `Metadata::with(&str, ...)`; it was corrected to borrow a local string. The
first probe run then exposed an incorrect assumption that 33 was the Metadata
wire limit; the fixture was corrected to 4097. The next run exposed an
incorrect expected NaN error kind; inspection showed the V1 DTO surfaces
`CustomSerialization`, and the expected exact kind was corrected. No comparison
was weakened. The final command
`cargo run --release --features json --example metadata_validation_allocations`
exited 0 and printed `semantic_probe=pass cases=20
comparison=structured_error_variant_and_resource` before the allocation
results.

**Decision: proceed with the minimal optimization in T3.** The performance
thresholds and the structured probe both pass. T3 may add the validation-only
API, retain the existing buffered writer contract, and migrate only the
downstream call that currently drops its `Vec`. If T3's fuller regression suite
finds a mismatch, remove the candidate and close RSM-002 as “核查完成、暂不
优化”. No cross-repository change was made in T2.

## Verification notes

- `cargo metadata --format-version 1 --features json,filter --locked`: exit 0;
  confirmed locked registry versions and sources shown above.
- `cargo run --release --features json --example metadata_validation_allocations`:
  all three allocation windows exited 0; the final semantic-probe rerun also
  exited 0.
- `rustfmt --edition 2024 --check examples/metadata_validation_allocations.rs`:
  exit 0. `git diff --check` on the T2 files: exit 0.
- `cargo fmt --all -- --check`: exit 1 because the repository baseline has
  numerous pre-existing formatting diffs across unrelated source, tests, and
  documentation. No unrelated formatting was applied.

## T3 formal API measurement follow-up

The public API and the downstream `rs-task` call were measured after T3
implementation. The first formal three-window run used the API without
`#[inline]` and exited 0 in all windows. It retained the six T2-defined
scalar/text samples' median gains (23.7%, 39.4%, 22.6%) and had lower
allocation cost. However, the escaped/multibyte/nested JSON stress sample was
slower in windows 2 and 3 (7.2% and 20.9%; absolute differences 0.194 us and
0.691 us), while window 1 was faster. This was investigated as a cross-crate
public API call cost: the T2 benchmark's local candidate helper was compiled in
the benchmark crate, while downstream code calls the library method.

The implementation adds `#[inline]` to the public validation method without
changing its checks or serializer path. One exploratory inline Criterion run
was interrupted with Ctrl-C and exited 130 before completing a window; it is
excluded. A complete inline follow-up then ran serially:

- Start: 2026-11-11 05:34:50 UTC; load average 38.94 / 41.04 / 34.00.
- End: 2026-11-11 05:39:07 UTC; load average 27.42 / 30.69 / 31.28.
- `cargo bench --features json,filter --bench metadata_benchmarks -- metadata/validation_only --save-baseline rsm-validation-inline-final-window-1`: exit 0.
- `cargo run --release --features json --example metadata_validation_allocations`: exit 0; semantic probe passed all 20 cases.
- Criterion raw artifacts are under `target/criterion/metadata_validation_only/{to_vec,validate_api}/<sample>/rsm-validation-inline-final-window-1/`.

Slope point estimates below are microseconds as `to_vec / validate_json_with_limits`.
The six typical samples are the same scalar/text 1/8/32-entry set selected
before T3; their median improvement in this inline follow-up was 27.3%.

| Sample | Earlier API window 1 | Earlier API window 2 | Earlier API window 3 | Inline follow-up |
| --- | ---: | ---: | ---: | ---: |
| scalar, 1 entry | 0.652 / 0.422 | 0.665 / 0.435 | 0.666 / 0.535 | 0.665 / 0.466 |
| text, 1 entry | 0.814 / 0.509 | 0.957 / 0.534 | 0.762 / 0.639 | 0.798 / 0.456 |
| scalar, 8 entries | 2.797 / 3.201 | 4.268 / 2.298 | 2.801 / 2.082 | 2.989 / 2.097 |
| text, 8 entries | 3.417 / 2.318 | 4.513 / 2.969 | 3.193 / 2.313 | 3.145 / 2.366 |
| scalar, 32 entries | 10.120 / 8.581 | 10.716 / 8.427 | 9.910 / 8.010 | 11.669 / 9.100 |
| text, 32 entries | 10.735 / 10.993 | 28.571 / 11.162 | 19.517 / 9.509 | 12.569 / 10.187 |
| escaped, multibyte, nested JSON | 2.323 / 1.843 | 2.693 / 2.887 | 3.303 / 3.994 | 2.472 / 1.915 |
| near-cap success, 16,376 bytes | 19.571 / 16.962 | 23.657 / 18.295 | 19.560 / 18.541 | 19.215 / 16.938 |
| over-cap rejection, 17,336 bytes | 19.623 / 16.464 | 29.432 / 16.665 | 20.232 / 17.226 | 21.317 / 17.156 |

In the final inline follow-up, all six T2-defined typical samples improved;
near-cap success improved 11.9%, and the special escaped/nested case improved
22.5%. Across the first formal three windows plus this inline follow-up, no
one of those seven successful samples has a repeated greater-than-5%
regression. The earlier nested regressions are retained above rather than
omitted; their direction changed in the inline follow-up. Criterion reported
sample outliers throughout (for example, 20/100 for the inline 32-entry
scalar `to_vec`, and 19/100 for the inline nested validation API), consistent
with the changing host load. The inline result is one follow-up window, not a
replacement for the three complete windows or a claim of a stable latency
promise.

Allocation results were unchanged across the three formal allocation runs
and the inline follow-up. Values are median allocation calls / requested
bytes per operation (`to_vec` / validation API): empty 3/56 vs 0/0; scalar 1
entry 4/120 vs 0/0; text 1 entry 5/248 vs 0/0; scalar 8 entries 6/504 vs
0/0; text 8 entries 7/1,016 vs 0/0; scalar 32 entries 8/2,040 vs 0/0; text
32 entries 9/4,088 vs 0/0; escaped/multibyte/nested 9/1,192 vs 3/688;
near-cap 10/33,510 vs 0/0; over-cap 14/35,515 vs 4/115. These are allocator
request sizes, not retained heap sizes.

**T3 decision: retain the validation-only API and the single `rs-task`
validation call migration.** The complete no-inline windows and the inline
follow-up preserve the previously selected six-sample performance threshold,
show reduced allocations, and pass the structured equivalence probe. The
inline follow-up confirms the earlier nested-sample regression does not persist
with the public cross-crate call path used by consumers. No writer contract or
wire representation changed.
