# Verification

Completion requires executable evidence for each contract, not merely a clean
lint run. Tests use temporary repositories and resources and never modify a
reference product, user configuration or running daemon.

| Area | Required evidence |
| --- | --- |
| Presentation | Root/help/version equivalence, centering, write errors, isolated offline binary |
| Identity | Clean/staged/unstaged/untracked/ignored, linked worktree, archive under unrelated repo, rebuild |
| Banner | Five imported normalized goldens, seven public names, reject invalid input, output/check |
| Distribution | Rust-only, embedded npm/assets, independent frontend/backend, target/features/locks drift |
| Integrity | Official CycloneDX schema, stale/missing inputs, exact runtime JSON, asset inventory |
| Quality | 400/401, role exclusions, mixed syntax, missing measurements/tools, unknown configuration |
| Policies | Dependency layers, parsed SeaORM uses, verified ownership/reproduction, repository checks |
| Workflow | Agent pointers, make contracts, isolated install, target paths, docs/skill links |
| Releases | Stable MAJOR/MINOR/PATCH, scoped range, first version, none/invalid changes, lockfiles, deterministic notes/replay, dirty work and tag protection |
| Installer/package | Strict schema/layout, deterministic script drift, full receipt/HEAD/target checks, real sh/hash with stubbed network/sudo, atomic replacement and regular-file uninstall |
| Signing | OpenPGP verification for every real/test commit, missing-signature detection and isolated simulated failure with no publication |
| Platforms | macOS/Linux CI and Rust 1.85 consumer libraries |
| Publication | Reviewed inventory, public HTTPS clone/build, full-revision consumer, green CI, v0.1.0 |

No release result is claimed until these checks run against the final source.
Record unavailable environments and unimplemented coverage explicitly.

`python3 scripts/tests/verify_installers.py target/execution-entry` runs real host
consumer build/package and temporary installer/uninstall acceptance through the
matching distributed entry. Existing platform runners execute it; target mapping
tests do not certify unexecuted host binaries. ADR 0003 implementation evidence
is recorded separately from the published v0.2.0 result.

## ADR 0003 source qualification (2026-11-06)

These qualification results were collected from development source above
published v0.2.0, `f34211b79ced9c2384ba97fd524ca973c147d299`. They do not certify a
new tagged release; later review corrections and local dynamic results are below.
The normative baseline, consumer SBOM, prepared seal/receipt and entry protocol
formats remain unchanged. Standards opts into schema 2 standard output mode.
Configuration validation, shared context resolution, embedded shell generation and
receipt-backed packaging live in the common engine; the delivery wrapper adds
only standards' entry/skill/release identity and final checksums.

| Contract | Evidence |
| --- | --- |
| Layout and compatibility | `tests/output_layout.rs` covers native/cross, debug/release, executable/library filenames, variants, configured/overridden roots, prohibited output overrides and frontend placement. `tests/execution.rs` retains legacy layout and exercises real build/install selectors. |
| Install target inference | `inferred_install_retains_recorded_explicit_target_layout` passes for explicit-build/inferred-install, omitted native target and explicitly mismatched selectors; tracked in `standards-auf`. |
| Generation | `tests/installers.rs` checks strict references/ownership/paths, deterministic bytes, read-only drift detection and symlink refusal. Ordinary assessment verifies declared script bytes. |
| Package | `tests/package.rs` builds two real commands in both legacy and standard mode. It checks exact assets/hashes and rejects missing/stale/altered receipts, dirty or wrong HEAD, sealed inventory additions, changed/missing sources, SBOM/binary tampering and version/target/profile mismatch. Packaging succeeds with only Git on PATH and no compiler. |
| Standalone shell | `scripts/tests/test_installer.py` runs real sh and hash tools, stubbed curl/sudo, temporary HOME/destinations, four targets/aliases, Latest/explicit versions, strict flags/checksums, upgrade preservation, permission failures, signal cleanup and offline regular-file uninstall. |
| Distributed entry | `scripts/tests/verify_installers.py` passes Make/CLI/launcher generation and package equivalence, missing-cache refusal, actual temporary binary install/uninstall, offline help/version/machine identity and byte-exact embedded SBOM. |
| Consumer boundaries | `scripts/tests/verify_execution.py` passes real standard-mode Rust root/profile/feature precedence, copy-only install, standalone frontend and embedded Web, including runtime-only dependency evidence. |
| Self delivery | A clean, disposable signed source export built standards 0.2.0 for `aarch64-apple-darwin`, packaged the common executable/SBOM, then entry/skill/release identity and final checksums. Exact six-file inventory, every hash, checksum exclusion and standalone skill links/manifest/runtime checks passed. |
| Size | tokei 15.0.0 measured 89 implementation files, maximum 362 Code, no oversized or unmeasured files and no size exceptions. Template and generated installer each measure 150 Code. Context resolution, installer configuration/generation and package publication retain distinct responsibilities. Tests/helpers, fixtures, ADRs and design prose are excluded by responsibility. |
| MSRV | `cargo +1.85.0 check --locked -p craftguild-cli -p craftguild-core -p craftguild-cli-build` passed. |

The actual macOS arm64 consumer package records version `0.1.0`, target
`aarch64-apple-darwin` and disposable signed revision
`2ba0a95d3d0053833145944c1f048d6efc69807b`. It publishes
`target/aarch64-apple-darwin/release/sample`; the package contains
`sample-v0.1.0-aarch64-apple-darwin`, its `.bom.json` and target checksums.
The standards delivery fixture uses disposable signed revision
`3a9a6846462db87dcfe07cc78fb58da89e20848d`, version `0.2.0`, release profile and
the same target. These revisions belong only to removed test repositories.
These temporary fixture revisions must not be used as consumer source pins.

Real native and `wasm32-unknown-unknown` library builds passed in both profiles,
with cdylib, staticlib and rlib outputs under `products/<target>/<profile>/` and
matching embedded SBOM bytes in each file. Native names include `.dylib`, `.a` and
`.rlib`; wasm names include `.wasm`, `.a` and `.rlib`. No toolchain was acquired.
The generated `scripts/install.sh` SHA-256 is
`f7182c9ccf28a43ee6dc37187ba36853eb7f10f111ecb2b88e0c50f9d3bc52f4`.

The recorded sudo boundary has unprivileged curl and SHA-256 verification first,
then separate `sudo mkdir -p <bindir>`, `sudo mktemp <bindir>/.<command>.XXXXXX`,
`sudo cp <download> <stage>`, `sudo chmod 0755 <stage>` and
`sudo mv -f <stage> <command>`. Paths containing spaces remain single arguments.
Copy/rename failure preserves the old binary and runs `sudo rm -f <stage>`;
authentication failure preserves it before staging. Uninstall uses only
`sudo rm -f <command>` when needed. Writable/root paths use no sudo. This evidence
uses a process stub, never real privilege escalation.

Local logs and detailed hashes are retained under `.craftguild/bgd-*`, including
`bgd-delivery-evidence.json`, `bgd-cross-evidence.json`,
`bgd-sudo-evidence.json`, `bgd-real-installer.log`, `bgd-consumers.log`,
`bgd-size.json` and `bgd-msrv.log`. The first skill archive link check exposed a
checkout-relative link; the final standalone archive check passes after correction.

Linux and other actual host/architecture executions have not run for this source.
The four-target shell matrix and cross compilation do not certify those runtimes.
The existing Linux/macOS CI matrix now includes installer acceptance; no runner
was added or remote CI triggered. The implementation has not modified products,
real HOME/system installations, published history or package versions, and has
performed no real-repository stage/commit/tag/push, release preparation or publish.

Full-check outcomes are distinct. An earlier default `make check` passed, but the
final parallel invocation reproduced existing `standards-1v1`: 19 execution tests
passed and signal cancellation failed with EPERM/deferred rollback; seven other
gates completed. `execution_process.rs` is unchanged and no assertion or ownership
check was weakened. A subsequent full `RUST_TEST_THREADS=1 make check` passed all
eight gates, including all Rust and 21 Python repository tests. This is not a
claim of deterministic parallel cancellation or remote CI success. Both logs
(`bgd-check.log` and `bgd-check-serial.log`) remain available. `standards-1v1` stays
open; `standards-auf` is closed for the verified source fix. Disposable fixture
OpenPGP signing and full-revision verification succeeded; no signing failure was
bypassed. Final source skill packaging and standalone archive verification pass.

### Independent review corrections: standards-94s

The runtime results above predate these corrections and do not prove them.
The initial static re-review and subsequent dynamic qualification are distinct.

The platform CI job now creates a fresh job-local Cargo cache and explicitly
fetches locked dependencies for the workspace and all three standalone example
manifests before offline operations. The cache's bin directory is added to PATH
for tokei installation. Existing Linux/macOS jobs exercise this cold-cache path;
ordinary build/install/package still do not fetch dependencies or engines.

Package's final checkout observation excludes only the exact repository-relative
path of its owned temporary stage, using a literal Git pathspec. HEAD agreement,
fresh destination refusal, complete receipt validation and same-filesystem rename
remain required. Parent directories and other `package-*` paths are not excluded.
The package integration test now adds an ignored top-level `dist` destination in
both layout modes, compares its exact output bytes and clean final status, and
checks existing-destination refusal. A Git process stub introduces tracked source
or unrelated sibling changes after staging; both must reject publication and
clean only the owned stage.

Source-only checks passed: workflow YAML parsing and preparation order for all
four existing manifests/lockfiles, rustfmt, Git whitespace checks and tokei 15.0.0.
The two changed implementation files measure 214 Code (`package.rs`) and 69 Code
(CI YAML), with no unmeasured files. Package/checkout validation stays within one
cohesive operation; regression fixtures remain test code. A trusted Git-only
fixture confirmed exact stage exclusion, including literal brackets, while
sibling and source additions remain visible. It did not execute the package tool.

At that static review checkpoint no target build, test, `make check` or cold-cache
fetch had been executed for the corrections. The review initially imposed an
audit-specific resource-isolation condition. The owner subsequently authorized
ordinary local regression validation using the existing isolated fixtures and
deferred actual cold-cache CI qualification to the release stage.

### Dynamic re-review and macOS cancellation correction

The first independent `RUST_TEST_THREADS=1 make check` reproduced
`standards-1v1` in the timeout branch, so the defect is not restricted to parallel
tests. The seven other gates and all 21 Python tests passed. A workspace Rust
regression run excluding only that known case passed, including the package test
for ignored top-level `dist`, exact assets and concurrent source/sibling refusal.

The cancellation root cause was then reproduced without a timing race. Two owned
children share a dedicated process group; the second child exits but remains
unreaped while the leader is terminated and reaped. With only the owned zombie
remaining, both `killpg(group, 0)` and `SIGKILL` return `EPERM` on macOS, while
the scoped `ps` query reports `Z`. Apple's
[XNU implementation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sig.c)
excludes zombies from group signal targets and can return POSIX `EPERM` when no
eligible member is found. The engine previously inspected terminal states only
after a successful signal probe and therefore deferred a safe rollback.

The correction inspects only the owned group on macOS `EPERM`. Confirmed
zombie-only membership permits cleanup; live members, inspection errors and
ambiguous observations still prevent rollback. Empty inspection output requires
an `ESRCH` probe before treating the group as gone. Other Unix behavior is
unchanged. The deterministic unit regression failed with `EPERM` on the old
implementation and passes with the correction; a live leader is also checked
as nonterminal. The test reaps both owned children, including on assertion failure.

The original signal/cancel-file/timeout integration test then passed all three
paths. Final ordinary `make check`, with no test filter or serial override,
passed all eight gates, including all 21 Python tests. The checker measured 89
implementation files, maximum 362 Code, no unmeasured files or size exceptions.
No target source assertion or process-ownership guarantee was weakened.

Local evidence is retained in `.craftguild/review-94s-dynamic-check.log`,
`review-94s-rust-regressions.log`, `1v1-zombie-group-evidence.json`,
`1v1-red-regression.log`, `1v1-green-regression.log`,
`1v1-cancellation-regression.log` and `review-94s-final-check.log`.
Actual cold-cache CI and other host executions remain release qualification,
tracked in `standards-9tx` as explicitly accepted by the owner. These local results do not claim remote CI
or a new released binary. Git publication of the reviewed source is a separately
authorized step; package versions and published tags remain unchanged.
