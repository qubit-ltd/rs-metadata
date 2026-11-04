# Metadata encoding performance investigation

## Scope and downstream call path

This investigation checks the local metadata costs in the typed task request path. It does not measure a performance defect or an end-to-end improvement.

`TaskExecutionService::submit` calls `TaskRequest::encode` before allocating the task ID and calling the store. `TaskRequest::encode` rejects more than 32 metadata entries, then runs `Metadata::to_json_vec_with_limits` with the default JSON budgets except that `OutputBytes` is set to 16 KiB. It drops that encoded vector after validation and retains the original `Metadata` in `StoredTaskRequest`. SQLite `accept_encoded` converts the stored request into `StoredRequestInfo`; that conversion clones metadata and the other immutable request fields. It then calls `serde_json::to_string(&request_info)` to produce `request_info_json`, which is inserted into SQLite. This second serialization contains metadata along with request identifiers, resource limits, and optional fields.

The benchmark below isolates metadata encoding and cloning. Its combination case models bounded encode and drop, metadata clone, then a second Serde encoding. It excludes payload encoding, identifier generation, request-field cloning, async/store dispatch, SQLite contention and I/O, transaction work, and the serialized non-metadata request fields. Thus it is a local cost proxy, not a measurement of `submit` latency. Two serialization operations do not imply twice the request latency.

## Benchmark design

The Criterion group is `metadata/request_encode_investigation`. It measures four operations per sample:

- `encode_and_drop`: bounded metadata JSON encoding using the downstream 16 KiB output budget, then drop the vector.
- `clone`: clone the metadata value.
- `serde_reencode`: `serde_json::to_vec` on metadata, then drop the vector.
- `validation_clone_reencode`: bounded encode and drop, metadata clone, Serde encode and drop.

The group uses `iter_batched`; input construction and setup cloning happen outside the timed routine. The timed routines contain only the named clone/encoding work. Sample encoding and the `<= 16 KiB` assertion run before Criterion starts timing. The same JSON encode defaults are retained except for the 16 KiB output limit.

Samples use 1, 8, or 32 entries with short keys `k00`… and value `v`, plus 32 entries with keys `field_00`…`field_31` and 470-character ASCII string values. Encoded lengths are exact compact JSON byte lengths; all samples passed the bounded encoder and the limit assertion:

| Sample | Entries | Encoded bytes |
| --- | ---: | ---: |
| `short_1_entries` | 1 | 56 |
| `short_8_entries` | 8 | 280 |
| `short_32_entries` | 32 | 1,048 |
| `near_16k_32_entries` | 32 | 16,216 |

## Environment and commands

- Worktree base: `9b64f41f818f8970584058e8cc7d50e49decbee6`.
- Toolchain: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, LLVM 21.1.8; `cargo 1.94.0 (85eff7c80 2026-01-15)`.
- Host: x86_64 Linux, Intel Xeon E5-2680 v4 @ 2.40 GHz, 2 sockets × 14 cores × 2 threads (56 online CPUs).
- Build profile: Cargo `bench` / optimized profile; `opt-level = 3`, LTO enabled, `codegen-units = 1` from `[profile.bench]`; all features enabled; locked dependencies.
- Criterion used 3 seconds warmup and 100 measurement samples per case. Iteration counts varied automatically by case and are not operation counts for an entire request.
- The shared host had concurrent Cargo/LLVM work from other repositories. CPU contention is visible in the changing intervals, high and severe outlier counts, and unstable comparisons between runs. CPU frequency scaling was enabled. The intervals should not be treated as an isolated-machine baseline.

The exact benchmark command was run three times serially:

1. `rustfmt --edition 2024 benches/metadata_benchmarks.rs && cargo bench --locked --all-features --bench metadata_benchmarks -- metadata/request_encode_investigation` — exit code **0**. Criterion output was captured in the execution log; this invocation did not tee stdout to a separate file.
2. `set -o pipefail; cargo bench --locked --all-features --bench metadata_benchmarks -- metadata/request_encode_investigation 2>&1 | tee /tmp/metadata-encoding-investigation-run2.log` — exit code **0**.
3. `set -o pipefail; cargo bench --locked --all-features --bench metadata_benchmarks -- metadata/request_encode_investigation 2>&1 | tee /tmp/metadata-encoding-investigation-run3.log` — exit code **0**.

All 16 benchmark cases completed in each run. Each row below gives Criterion's reported time interval and its outlier count for runs 1/2/3. `—` means Criterion reported no outliers. The intervals are shown as `[lower, upper]`; units are included in each cell.

## Results

| Operation / sample | Run 1 | Run 2 | Run 3 |
| --- | --- | --- | --- |
| encode/drop · short 1 | [860.70, 917.59] ns; 5/100 | [860.07, 877.46] ns; 19/100 | [827.11, 837.33] ns; 6/100 |
| clone · short 1 | [80.704, 88.477] ns; 14/100 | [78.680, 79.296] ns; 11/100 | [79.478, 80.640] ns; 12/100 |
| Serde · short 1 | [214.05, 226.52] ns; 2/100 | [221.03, 223.66] ns; 13/100 | [215.93, 222.54] ns; — |
| combination · short 1 | [1.2543, 1.4076] µs; 17/100 | [1.0601, 1.0824] µs; 16/100 | [1.0331, 1.0498] µs; 8/100 |
| encode/drop · short 8 | [3.3436, 3.7591] µs; 14/100 | [3.2801, 3.3219] µs; 5/100 | [3.1784, 3.2381] µs; 8/100 |
| clone · short 8 | [422.80, 478.13] ns; 13/100 | [424.41, 432.62] ns; 11/100 | [413.20, 421.06] ns; 11/100 |
| Serde · short 8 | [1.2137, 1.2989] µs; 12/100 | [1.1960, 1.2147] µs; — | [2.0364, 2.2709] µs; 18/100 |
| combination · short 8 | [4.7580, 4.7856] µs; — | [4.7961, 4.9037] µs; 7/100 | [4.5967, 4.6368] µs; 10/100 |
| encode/drop · short 32 | [11.730, 11.805] µs; 5/100 | [11.827, 11.884] µs; — | [11.535, 11.900] µs; 8/100 |
| clone · short 32 | [3.0982, 3.1185] µs; 6/100 | [3.0563, 3.0997] µs; 8/100 | [3.0136, 3.0303] µs; 5/100 |
| Serde · short 32 | [4.1591, 4.1910] µs; 3/100 | [4.5503, 5.1208] µs; 19/100 | [4.1945, 4.2141] µs; 4/100 |
| combination · short 32 | [17.827, 18.563] µs; 14/100 | [18.647, 20.034] µs; 16/100 | [17.735, 17.880] µs; 4/100 |
| encode/drop · near 16 KiB | [22.145, 24.015] µs; 18/100 | [22.552, 23.979] µs; 17/100 | [22.694, 26.771] µs; 9/100 |
| clone · near 16 KiB | [4.0993, 4.2893] µs; 15/100 | [4.0181, 4.0432] µs; 12/100 | [9.0710, 9.3875] µs; — |
| Serde · near 16 KiB | [18.495, 21.047] µs; — | [14.094, 14.267] µs; 4/100 | [14.589, 15.235] µs; 12/100 |
| combination · near 16 KiB | [38.663, 41.295] µs; 17/100 | [39.156, 43.407] µs; 6/100 | [39.041, 40.827] µs; 10/100 |

The combined path ranged from about 1.03–1.41 µs for the one-entry sample, 4.60–4.90 µs for eight short entries, 17.7–20.0 µs for 32 short entries, and 38.7–43.4 µs near 16 KiB. These are metadata-only operation timings. In particular, they do not state that a SQLite-backed request costs that amount, or that avoiding one serialization would reduce request latency by the same amount.

## Assessment

The local measurements show that the two encodings and the intervening clone have measurable cost in this isolated proxy, especially for larger metadata. They do not provide reliable evidence of an end-to-end bottleneck or of a likely user-visible gain: there is no `rs-task` plus SQLite submit benchmark here, and the shared-host results vary substantially (for example, the short-eight-entry Serde interval changes from 1.20–1.21 µs in run 2 to 2.04–2.27 µs in run 3; the near-limit clone changes from 4.02–4.04 µs to 9.07–9.39 µs). This spread and the outliers preclude interpreting apparent gains or regressions as stable.

Allocation counts/bytes were **not measured**; no reliable allocation profiler was used. No `to_json_writer(io::sink())` comparison was made, since that would not establish removal of the encoder's internal buffering.

**Disposition:** retain the current API and leave the performance question open. These results do not justify adding a public size-validation API, changing downstream production calls, or claiming an optimization benefit. If the question becomes decision-critical, measure typed request submission through the real SQLite store under a controlled host, including request composition and database conditions, before designing an optimization.
