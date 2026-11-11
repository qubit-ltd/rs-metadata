# Verification Record: RSM-001 and RSM-002

Date: 2026-11-11
Target: `rs-metadata`
Implementation worktree: `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-metadata`
Baseline commit: `e189da16b9be6cb3ed4ec4132818b9a635f34574`

## Outcome

- **RSM-001 — published dependency acceptance:** implemented a registry-only archive check and connected it to the existing package hook. The final `rs-metadata` CI passed both the existing candidate-path check and consumption of the same archive with published crates.io dependencies.
- **RSM-002 — validation cost:** the measured candidate passed the plan's timing, allocation, and semantic-equivalence gates. Added `Metadata::validate_json_with_limits` and migrated the one `rs-task` call that discarded the encoded `Vec`. The API preserves the existing structured errors and validation order while avoiding the output buffer.

## RSM-001 implementation and checks

Added `.infra/check-package-with-registry-deps.sh`, its controlled test script, and calls in `.infra/project-ci-check.local.sh`. The hook still runs the candidate local-path check, then runs the registry-only check against the same `.crate` archive. The new script checks archive paths, rejects non-registry external dependencies while permitting the archived root package itself to be a path package, uses fresh Cargo home/target directories, removes the archive's lockfile before resolution, and runs default tests, locked all-features/all-targets tests, and locked doctests.

Recorded commands and results:

| Command (working directory is the `rs-metadata` implementation worktree) | Result |
| --- | --- |
| `bash -n .infra/check-package-with-registry-deps.sh` | exit 0 |
| `bash -n .infra/tests/check-package-with-registry-deps-tests.sh` | exit 0 |
| `bash -n .infra/project-ci-check.local.sh` | exit 0 |
| `bash .infra/tests/check-package-with-registry-deps-tests.sh` | exit 0; controlled archive, metadata, Cargo failure, environment isolation, and cleanup cases passed |
| `git diff --check -- .infra/check-package-with-registry-deps.sh .infra/tests/check-package-with-registry-deps-tests.sh .infra/project-ci-check.local.sh` | exit 0 |
| `.infra/bin/align-ci.sh` | final run exit 0; style operation and `rs-infra-ci check` succeeded |
| `.infra/bin/ci-check.sh` | exit 0; package suite and registry archive consumers succeeded |

The real archive check resolved `qubit-redact 0.10.1` and `qubit-value 0.13.0` from `registry+https://github.com/rust-lang/crates.io-index`; metadata confirmed external dependencies were registry sourced. The archive's default tests, locked all-features/all-targets tests, and doctests passed. The existing candidate check also succeeded. Published guide consumers compiled and ran for both README languages and both user-guide languages.

## RSM-002 measurement and semantic evidence

The downstream inspection found `rs-task::TaskRequest::encode` using 32 metadata entries and a 16 KiB JSON output budget, and dropping the `Vec` returned by `to_json_vec_with_limits`. Its existing model tests cover the entry and encoded-byte boundaries. No downstream source was changed during the T2 measurement phase.

Three serial release windows and the allocation runs passed. The six typical scalar/text samples (1, 8, and 32 entries) had median time improvements of 19.9%, 47.8%, and 37.1% across the three T2 windows. Allocation requests fell to zero for those typical cases; nested JSON fell from 9 calls / 1,192 requested bytes to 3 / 688. Near-cap success fell from 10 / 33,510 to 0 / 0. The 20-case structured probe compared successful output bytes and structured error signatures, including all eight value budgets, V1 entry/key limits, exact output boundaries, non-finite values, and competing-failure ordering.

After the public method was implemented, three complete formal windows without `#[inline]` met the six-sample gate. The nested stress sample regressed in two of those windows, so a single complete inline follow-up was measured for the public cross-crate call path. In that follow-up the six typical cases had a 27.3% median improvement, nested JSON improved 22.5%, and near-cap success improved 11.9%. Allocation counts matched the T2 result. A separate exploratory inline run was interrupted before one window completed (exit 130) and is excluded from the results. Measurements ran on a shared, heavily loaded host; the investigation document retains window data, load observations, raw Criterion artifact paths, and this limitation.

The full measurement record, sample data, commands, semantic probe details, and T3 decision are in [`metadata-validation-investigation-20261111.md`](metadata-validation-investigation-20261111.md).

## T3 API and downstream checks

The public API checks the metadata V1 wire contract first, performs the same `ValueWireEncodePreflight`, and sends the shared incremental JSON encoder to `std::io::sink()` so it need not retain output bytes. It remains gated by the existing `json` feature and takes the existing explicit `JsonEncodeLimits` value. The encoder/writer behavior and wire representation did not change.

Recorded commands and results:

| Command | Result |
| --- | --- |
| `cargo test --no-default-features --features json --test metadata_validation_encoding_tests` (`rs-metadata` worktree) | exit 0; 1 equivalence test passed |
| `cargo test --manifest-path /data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-task/Cargo.toml --test model_api_tests` | exit 0; 15 tests passed |
| `cargo test --manifest-path /data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-task/Cargo.toml --test typed_payload_encode_tests` | exit 0; 5 tests passed |
| `cargo metadata --manifest-path /data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-task/Cargo.toml --format-version 1 --locked` | exit 0 after the lockfile repair below; local packages resolved from the mapped task worktrees |
| `.infra/bin/align-ci.sh` (`rs-task` worktree) | final run exit 0; style and `rs-infra-ci check` succeeded |
| `.infra/bin/ci-check.sh` (`rs-task` worktree) | final run exit 0; feature matrix, tests, doctests, package verification, coverage, and security audit succeeded |

The downstream mapping is unchanged: output-budget errors still map to `MetadataTooLarge`; other metadata errors still map to `TaskRequestEncodeError::Metadata`. A NaN regression test verifies serialization error propagation.

## T4 repository checks

The final `rs-metadata` `.infra/bin/ci-check.sh` exited 0. Its build, default and feature-matrix test suites, doctests, fuzz targets, guide consumers, package verification, coverage, and security audit all passed. Coverage was functions **99.76%** (421/422), lines **97.11%** (3095/3187), and regions **94.66%** (4504/4758), above the configured thresholds. The security audit scanned 101 locked dependencies and returned success. The CI's coverage result was complete, so a separate coverage invocation was not needed.

The final `rs-task` `.infra/bin/ci-check.sh` exited 0. Its configured feature matrix, tests, doctests, package verification, coverage, and security audit passed. Coverage was functions **94.99%** (588/619), lines **94.74%** (5913/6241), and regions **92.52%** (8307/8979), above its configured thresholds. The security audit scanned 204 locked dependencies and returned success.

### Failures diagnosed and repaired

- The first `rs-metadata` alignment run rejected the two new Rust files for not using the repository's seven-line license header and rejected the integration-test name. Those were corrected; the final alignment and CI passed.
- The first `rs-task` alignment check found `clippy::let_unit_value` on the discarded unit result. The unnecessary `let _ =` binding was removed; final alignment and CI passed.
- The pre-existing `rs-task/Cargo.lock` lacked the `async-io` dependency edge listed by `qubit-event-bus-redis`. `cargo metadata --locked` failed on both the implementation worktree and the original baseline. `cargo metadata --offline` resolved the local graph and added only the missing `async-io` edge; locked metadata and the final CI then passed. The lockfile repair is included with the downstream change.
- `cargo fmt --all -- --check` had previously reported numerous baseline formatting differences in unrelated files. No unrelated files were manually reformatted; the repository's required alignment script completed successfully on the final worktree.

During the first `rs-task` alignment attempts, its bootstrap script tried to create shallow clones for local-path dependency directories that were not yet present in the isolated layout. The attempts were interrupted. The generated shallow clones (`rs-codec`, `rs-datatype`, `rs-progress`, and `rs-validation-rules`) were moved aside, verified as task-created shallow clones, and removed. All required directories were then created with `git worktree add` from their existing complete local source repositories, and alignment was rerun successfully. No source worktree was modified by this recovery.

## Dependency worktree record

All entries below are read-only, detached worktrees created for this implementation. Each worktree's Git common directory points to the named existing source repository; no dependency repository was committed, synchronized, or pushed.

| Repository | Existing source repository | Commit | Worktree path |
| --- | --- | --- | --- |
| `rs-value` | `/data/working/qubit/rust-common/rs-value` | `6299b12d7b145199f7d4e718531dec4581a68631` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-value` |
| `rs-redact` | `/data/working/qubit/rust-common/rs-redact` | `d3116d19fc40d6e23db0fdca4db3748d0dede7d8` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-redact` |
| `rs-model-metadata` | `/data/working/qubit/rust-common/rs-model-metadata` | `19e0c9e40b82e50b684966726d538b147caf2b74` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-model-metadata` |
| `rs-event-bus` | `/data/working/qubit/rust-common/rs-event-bus` | `43ee17c8477ccb9b51bd8a28ecb199ecc3164d13` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-event-bus` |
| `rs-event-bus-redis` | `/data/working/qubit/rust-common/rs-event-bus-redis` | `30cfd469734664ff5d4770604abda9fdd8674289` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-event-bus-redis` |
| `rs-reflect` | `/data/working/qubit/rust-common/rs-reflect` | `18a0717228af63eba66ef3520d2823841a4e0c48` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-reflect` |
| `rs-codec` | `/data/working/qubit/rust-common/rs-codec` | `1f4c243d422fa6514583ea3facd2a51ddc60001d` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-codec` |
| `rs-datatype` | `/data/working/qubit/rust-common/rs-datatype` | `35d2921f9ae9d8931a6ec9f173324e82bf677743` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-datatype` |
| `rs-id` | `/data/working/qubit/rust-common/rs-id` | `82a1457045c9fb14fb968a38b3fbfb9da90a1489` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-id` |
| `rs-progress` | `/data/working/qubit/rust-common/rs-progress` | `853ec97df471223107b13faa6e6b5e71fa3cb71b` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-progress` |
| `rs-validation-rules` | `/data/working/qubit/rust-common/rs-validation-rules` | `87e1d602371f758a2d346193f0ec3b6a4f1644ad` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-validation-rules` |
| `rs-validator` | `/data/working/qubit/rust-common/rs-validator` | `9674fa169fe0071f1fb36df2d70fb0b8c37cf406` | `/data/working/qubit/rust-common/.worktrees/rs-metadata-rsm-20261111/rs-validator` |

## Scope and handoff

The original design and implementation-plan documents were not copied into or staged from the implementation worktree. Their original files remain in the user's target checkout. No README changes, wire-format changes, writer-contract changes, lockfile changes to `rs-metadata`, dependency source commits, or out-of-scope API changes were made. Git commits, fast-forward integration, branch synchronization, and worktree cleanup are recorded in the implementation handoff after those operations complete.
