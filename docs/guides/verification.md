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
| Releases | 0.x effective MINOR for breaking, explicit-MAJOR intent/replay, stable MAJOR/MINOR/PATCH, scoped range, first version, none/invalid changes, lockfiles, deterministic notes/replay, dirty work and tag protection |
| Installer/package | Strict schema/layout, deterministic script drift, full receipt/HEAD/target checks, real sh/hash with stubbed network/sudo, atomic replacement and regular-file uninstall |
| Signing | OpenPGP verification for positive histories; intentional unsigned rejection fixtures and isolated simulated failure with no publication |
| Platforms | macOS/Linux CI and Rust 1.85 consumer libraries |
| Publication | Reviewed inventory, public HTTPS clone/build, full-revision consumer, green CI, v0.1.0 |

No release result is claimed until these checks run against the final source.
Record unavailable environments and unimplemented coverage explicitly.

`python3 scripts/tests/verify_initialization.py` builds a disposable signed clean
source fixture, validates flat host delivery and its release index, then exercises
real init/prime and standalone consumer/installer acceptance using stubbed HTTPS
and temporary HOME/destinations. Mapping tests do not qualify unexecuted hosts.
The earlier ADR 0003 records below describe its historical bundle qualification;
ADR 0004 replaces that delivery workflow while retaining its artifact guarantees.

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


## ADR 0005 source qualification (2026-11-06)

The source release engine now applies the selected group's current major version:
breaking changes select MINOR at 0.x and MAJOR from 1.0.0 onward. Explicit
`--major` applies current major + 1 once, including initial releases and empty
ranges. Published v0.3.0 and its receipts, notes, versions and tags are unchanged.

`tests/releases.rs` checks every inferred increment across 0.0.x, high 0.x minor,
1.x and 2.x, normal initial versions and checked overflow. Signed disposable Git
fixtures in `scripts/tests/test_release.py` exercise initial/tagged preview and
apply, all breaking forms and mixed ranges, independent groups with different
major versions, empty/non-bumping ranges, synchronized Cargo/npm manifests and
lock roots, breaking notes with full identities, matching-mode replay before and
after committing only preparation, both directions of mode conflict, legacy
receipt refusal, stale/dirty/tag guards and signature/message rejection. The
intentional unsigned fixture tests rejection; all positive histories use disposable
OpenPGP keys. A product preparation hook fixture proves mode conflicts precede
hook, record and file writes. Actual repository Make forwarding passed isolated
explicit-major preview/apply/replay as well.

Final ordinary `make check` passed all eight gates, including 29 Python tests.
Rust 1.85 consumer-library checks passed offline. Tokei 15.0.0 measured the four
changed implementation files at 180, 201, 214 and 279 Code lines; the complete
repository gate measured 89 files with maximum 362, no unmeasured files or size
exceptions. Increment selection, planning/CLI, guarded receipt persistence and
operation hooks remain separate responsibilities. Immutable receipts retain
technical intent/provenance; signed Git history records publication. There is no
business lifecycle or Header-Detail aggregate.

Local evidence is retained in `.craftguild/a0c-final-check.log`,
`a0c-msrv.log` and `a0c-make.log`. Execution was on macOS arm64; no remote CI or
other host qualification was run. No product, real HOME/system installation,
package version or published artifact was changed. No real-repository release
preparation, stage/commit/tag/push or publication was performed.

## ADR 0004 source qualification (2026-11-06)

The source implements init, fixed verified acquisition and offline prime, replacing
execution/skill archives. Published v0.3.0 and earlier do not provide this model.
No public consumer pin was created. Canonical contracts and adoption/configuration
texts are embedded and explicitly sealed engine inputs; detailed source ledgers
remain producer-local and are absent from delivery metadata and consumer SBOMs.

| Contract | Current evidence |
| --- | --- |
| Initialization | Nine Rust integration tests cover valid unfinished TOML, strict operational rejection, preserved config/Make/contracts, short local skill, ignore/retry/no rewrite, conflicting files/markers/symlinks, locks, interrupted publication, concurrent changes, owned rollback, failed output and legacy-descriptor migration refusal. |
| Acquisition and entry | Nine Python entry tests cover four host targets and aliases through stubbed transport, unavailable/unsupported hosts, exact assets, transport/incomplete/hash/identity/compatibility failures, interruption cleanup, cache reuse/tamper and missing-cache refusal for all ordinary operations. Make forwards release intent and prime. Hash verification precedes executing downloaded code. |
| Context | Rust tests cover overview, every topic, pinned links, unknown topics, missing/malformed/unfinished/structurally configured config and no product gate/hook execution. Real CLI acceptance uses a denied-tool PATH and file snapshots to verify offline, banner-free, read-only matching context. |
| Matching versions | A disposable clean source engine at fixture version 0.3.0 initializes the consumer. A separately rebuilt installed fixture at 9.9.9 delegates prime/init to the unchanged fixed engine; its dirty unpublished build cannot create a fresh public reference. Missing/tampered cache is refused without acquisition. These are test fixtures, not published versions or consumer pins. |
| Standalone consumers | Actual Rust and npm/esbuild consumers use only the thin entry/pin/config plus their product/build inputs. Make/direct/launcher output agreement, standard output layout, offline identity, exact SBOM, copy-only install and standalone/embedded Web runtime inclusion pass. No rules tree, engine archive, sibling checkout or figlet-rs runtime dependency is needed. Common-crate source exports are confined to disposable development fixtures. |
| Flat delivery | A clean signed source fixture produces exactly binary, matching SBOM, host metadata and target checksums. Aggregation produces a versioned index/checksum. Tests reject ignored added/unsealed/changed/missing sealed inputs and SBOM/binary tampering independently of Git dirtiness; restored control output is identical. Metadata tests combine unique matching hosts and reject mixed identities, duplicate hosts and changed assets. |
| Installer and ownership | The real consumer preserves receipt-backed package/generation equivalence and temporary install/uninstall with exact SBOM/identity. Existing common context, cancellation, package, font and measurement suites remain in standards. Retired optional bundle-pilot scripts no longer move font tests into product core; SDK/ABI/generator/domain gates remain with the unchanged products. |
| Release follow-up | The final signed release fixture also rejects missing schema-2 preparation intent; a normal 0.x receipt cannot authorize promotion regardless of receipt schema. Both retry modes remain guarded before product hooks. |
| Required gates | Final ordinary make check passed all eight gates, including 37 Python tests; focused initialization/release tests, offline Rust 1.85 consumer-library checks, document/provenance/whitespace checks and workflow YAML/host-index dependency checks passed. |
| Size and boundaries | Tokei 15.0.0 measured all 16 changed implementation files, maximum 279 Code; the full gate measured 91 files, maximum 362, with no oversized/unmeasured files or size exceptions. Pin validation, initialization planning, transaction/rollback, context rendering, acquisition/dispatch and delivery/index generation have separate responsibilities. Tests/helpers, fixtures, ADRs and design prose are excluded by role. |

Actual execution was macOS arm64 (`aarch64-apple-darwin`). The final isolated
fixture's clean engine revision was `4150ab17efa6655d0e7b6218508ef18fa0d110a3`;
the installer consumer used `64aea23d7ffbfd419a10aef4933eeda27cb6e584`. Both
repositories and disposable keys were removed. Never use these fixture revisions
as public source pins. Final legacy-reference and missing-intent guard additions
are covered by focused regressions and the final full gate; the real standalone
host acceptance separately verifies the flat delivery/init/prime/build workflow.

Local evidence: `.craftguild/g6b-acceptance-pass.log`, `g6b-completion-check.log`,
`g6b-audit-tests.log`, `g6b-entry-final.log`, `g6b-msrv.log` and
`g6b-final-size.json`. Earlier attempts exposed fixture HOME/toolchain routing and
a shared transport-log baseline error; the corrected acceptance completed.

macOS x86_64 and Linux x86_64/arm64 actual runtimes/SDKs remain unexecuted for this
source. Mapping stubs are not host qualification. The existing Linux/macOS CI
host definitions now exercise flat acceptance and combine qualified host metadata;
no remote CI was triggered. Init has no global skill install, and all acceptance
HOME/destinations and transport/signing resources were isolated. There were no
product edits, real user/system installations, dependency/version changes,
real-repository release preparation, stage/commit/tag/push or publication.

The final focused signed-release run passed all 19 tests after extending the
matrix to independent fix-only/non-bumping/empty/breaking ranges and a tagged
1.x explicit-MAJOR preview/apply/replay. Evidence is
`.craftguild/g6b-release-verified.log`; this follows the eight-gate full run above
without further implementation changes. An assertion-placement error in the
first test extension was corrected before this passing run.


## ADR 0004 initialization review corrections: standards-ofl (2026-11-06)

The previous full gates missed two integration cases: the generated AGENTS block
omitted the normative baseline required by CG-AGENT-001, and init recognized only
the old execution descriptor rather than the known manifest-based adoption.

The generated common block now takes its baseline from the engine's BASELINE
constant, with the source AGENTS template aligned. The instruction checker is
unchanged. A regression completes a real Rust product fixture after initialization
and runs the actual quality assessor before and after identical reinitialization.
It preserves product contracts and reports zero findings. Removing only the
baseline remains a CG-AGENT-001 failure.

Init now refuses both standards/manifest.json and
standards/execution/engine.json before release lookup, cache acquisition or
installed-version delegation. The planner shares this guard. It does not load or
remove legacy pins, rewrite product Make recipes or create a second authority.
A fixture matching the reviewed v0.2 manifest and legacy Make include exercises
the real initialization operation in isolated child processes. It covers fresh
legacy adoption and coexistence with a new pin invoked from a different installed
version. A transport marker must remain absent; legacy file snapshots stay exact,
and no new pin or cache appears. Existing fresh/repeated new-format tests remain.

The new regressions failed on the reviewed source (nine passed, two failed) and
all eleven initialization tests passed after correction. The review-authored
probe was rerun unchanged against a separate owned fixture: generated AGENTS
findings were empty, legacy planning returned false and init returned the expected
explicit-migration error before creating any cache or second authority. This is
probe revalidation by the implementer, not independent code re-review approval.

Evidence: .craftguild/ofl-regressions-red.log, ofl-regressions-green.log,
ofl-review-probe.log, ofl-msrv.log and ofl-size.json. The modified implementation
unit initialization.rs measures 192 Code with tokei 15.0.0; unmeasured/oversized
files: zero. Test/helper code, template prose and design documents are excluded
by responsibility. No split is needed: the legacy guard belongs to initialization
preflight, and the common block remains initialization output.

All new fixtures are isolated. No product repository, real HOME/system destination,
source version/dependency, published asset or release history was changed. No
real-repository release preparation, stage/commit/tag/push, live service, remote
CI or other-host runtime was used. The full signed-source host acceptance was not
repeated for these two focused corrections. Independent code re-review remains
the parent issue's final acceptance step.

Final ordinary make check passed all eight gates, including 38 Python tests;
the complete source gate measured 91 implementation files, maximum 362 Code,
with no unmeasured files or size exceptions. Offline Rust 1.85 consumer checks,
rustfmt/Clippy, document/provenance and whitespace checks passed. Full-gate evidence
is .craftguild/ofl-check.log. Executed host: macOS arm64; other hosts and remote CI
remain unexecuted for this correction.


## ADR 0006 source qualification (2026-11-07)

This qualifies the source initializer/engine split and controlled updates for
`standards-2bi`. Published v0.4.0, its tags, release notes, receipts and assets
remain unchanged. Isolated version 0.4.0/9.9.9 builds and full revisions below
are test fixtures, not new public releases or real consumer pins.

| Requirement | Executed evidence |
| --- | --- |
| Command roles and artifact identity | `make build` builds the init-only `craftguild-standards` from `craftguild-initializer` and the full `craftguild-standards-engine` from `craftguild-standards`. Both belong to the same release group/version. `make acceptance` checks each binary alone with empty PATH and malformed config: identical offline help/version aliases, command-specific build/SBOM identity, exact SBOM bytes, no source ledger and no file writes. Initializer global engine operations are refused; temporary source install copies only the initializer. |
| Same clean release and host delivery | `verify_initialization.py` exports a fresh source tree, signs its fixture commit with a disposable key and builds both clean artifacts at full revision `1e90c3898d08a8a91dcab14abd77a134a4c2b2ce`. Packaging verifies both receipts and separate binary/SBOM/checksum assets. Schema 3 metadata labels initializer and engine independently; aggregation checks matching identity, unique hosts, exact role URLs and asset hashes. Added/unsealed/changed/missing inputs and altered SBOM/binary assets are refused before delivery publication. |
| Numeric update policy | The 15-test Rust initialization suite covers all transitions in both force modes, including 0.9.0 versus 0.10.0. Real clean initializer and engine calls cover fresh init with and without force, upgrade, downgrade, same-version revision replacement, and exact retries. Refused transitions preserve the tree and make no transport calls; errors identify both identities and the force option. Successful changes report their classification and identities. Local ownership conflicts are checked before suggesting a version-only force override. No Git ancestry is used. |
| Exact retries and cache restoration | Both real binaries restore a missing cache in normal and force modes using only recorded binary/SBOM URLs, without release-index lookup. Subsequent retries preserve file mtimes and perform no acquisition. Source implementation rejects inconsistent exact-identity ownership/role data instead of guessing a repair. |
| Force boundaries | Real forced calls reject transport failure, hash/revision/SBOM/role/protocol mismatch, missing host assets, unsupported existing reference/protocol, symlink paths and edited managed content. Common entry tests also reject initializer identity and historical identity without an engine command, foreign/unclean identities, tampered cache metadata/SBOMs, entry hashes and unsupported hosts. |
| Managed update and prior flat pin | The planner records entry, Skill and AGENTS common-block hashes in reference schema 3. The frozen schema 2 baseline matches published v0.4.0 (`893a6d4b8a9d67fb57ec9cd0027ddf0cd5f1fdbc`). Rust tests exercise known-flat migration, edited-content refusal and rollback before pin publication. `verify_legacy_update.py` separately uses the same clean real initializer to upgrade a schema 2 consumer, replacing launcher/Make/Skill/AGENTS while retaining its product contract and old valid cache. The final integrated host acceptance also executes this scenario. No historical full CLI is treated as the new engine. |
| Ownership, concurrency and rollback | Tests preserve product config, Make recipes, AGENTS contracts and Cargo/npm inputs; refuse edited managed files and legacy bundle authorities; cover exclusive locking, lock replacement, acquisition-time edits, interrupted publication, acquisition/output failures, and concurrent files that rollback must preserve. The pin is published last after staged acquisition/verification, and previous pin/cache bytes survive failed updates. Version refusal occurs before lock/staging/transport. |
| Clone and pinned operations | A clone-equivalent tree without cache first refuses prime, then runs explicit `make setup` and offline prime using only committed entry files. Real standalone Rust, embedded npm/esbuild and independent frontend consumers execute through the pinned engine. Ordinary operations refuse absent cache without acquiring, and setup never selects another version. Prime runs with denied network/compiler/product tools, reports incomplete config and matching embedded topics, and leaves files unchanged. No global full engine, old archive, copied rules tree, sibling checkout or product font-renderer dependency is required. |
| Canonical guidance and compatibility | Specs, architecture/configuration/initialization design, source/local Skill, README, guides, Make and CI distinguish the initializer from the pinned engine, force policy and manual common-crate Git revision follow-up. Initialization does not certify product conformance. ADR 0005's 0.x/explicit-major behavior is unchanged and its existing release tests pass. Standard Rust outputs and source-ledger-free consumer SBOMs remain verified. |

Passed commands: focused Rust initialization tests (15), `make check` (all eight
gates, including 38 Python tests), `make build`, `make acceptance`, release-profile Clippy for both distribution binaries,
`cargo +1.85.0 check --offline --locked -p craftguild-cli -p craftguild-core -p craftguild-cli-build`,
`python3 scripts/tests/verify_initialization.py`, the additional real flat-pin
upgrade scenario, document/provenance checks and `git diff --check`.
The integrated host suite includes real Rust/npm consumers and temporary
installer/package/install/uninstall acceptance using transport stubs.

Code-size assessment used tokei 15.0.0 with explicit changed implementation paths:
31/31 files measured, maximum **216 Code** in `init_transaction.rs`, no oversized
or unmeasured files. The full source gate measured 97 implementation files,
maximum **362 Code**, no verified size exceptions. Rust, Python, shell, Make and
CI wiring are included; tests/helpers/fixtures, manifests, ADRs and design prose
are excluded according to the repository's executable-source inventory. Policy,
managed-baseline planning, transaction publication, provenance and command roles
have separate cohesive responsibilities; no numerical-only split was needed.
The design-baselines decision remains a transient technical provenance and
consistency boundary, with no business history table or Header-Detail aggregate.

Local evidence: `.craftguild/2bi-check.log`, `2bi-build.log`, `2bi-acceptance.log`,
`2bi-focused.log`, `2bi-msrv.log`, `2bi-initialization.log`,
`2bi-legacy-update.log`, `2bi-binary-clippy.log` and `2bi-size.json` under the same control directory.
Executed runtime host: **aarch64-apple-darwin (macOS arm64)**. Linux arm64/x86_64
and macOS x86_64 runtime acceptance, remote CI and paid runners were not executed;
four-host transport/selection fixtures do not claim runtime qualification.

All acquisitions and installs used isolated roots, temporary destinations and
transport stubs. No real HOME/system installation, product repository, global
Skill, user key, live service, dependency upgrade, real version bump, release
preview/apply, stage/commit/tag/push or registry/publication operation was performed.
The only Cargo.lock addition is the internal initializer workspace package.


## ADR 0006 read-only rollback correction (2026-11-07)

Independent review found a P2 permission failure after the initial qualification:
a managed launcher at mode 0444 was replaced successfully, then a mode 0555
`standards/make` directory refused publication of `common.mk`. Restoring the
launcher with `fs::write` failed too, leaving the new launcher behind the old pin.
The original report and fixtures remain unchanged in
`.craftguild/adr6-independent-review-m2gylfzp/`.

Before publishing, initialization now stages each existing file's previous bytes
and permission mode. Rollback verifies ownership of the current bytes and of the
backup, then renames the backup over the replacement. It does not need write
permission on a read-only destination file. Concurrent replacements remain
untouched. A failed restore no longer stops other owned restores; the error lists
failures and retains the remaining prior files in an owned `init-recovery-*`
directory, reporting its path. Normal rollback removes owned staging. The pin-last
publication rule, exclusive lock and distinct valid engine caches remain intact.
The [initialization design](../design/initialization.md) records this boundary.

Regression coverage in `tests/init_rollback.rs` verifies:

- The natural 0444-launcher/0555-Make-directory failure restores all prior bytes,
  file modes, pin and cache without an injected failure.
- A failure after publishing the read-only pin restores every read-only file
  with its original mode.
- A concurrent read-only replacement is preserved.
- A directory that blocks one rollback does not stop other restores, and the
  blocked file's old bytes/mode remain available at the reported recovery path.

Before the fix, two of the first three regression tests failed with permission
denied. After the fix, all four regressions and the existing 15 initialization
tests pass. The independent review's unchanged `rollback_probe.rs` was also
linked against the corrected library and run on fresh fixture copies. Both its
interruption and natural permission-failure cases restore all files and the old
pin, preserve mode 0444, and leave the entry matching the active pin. Evidence is
`.craftguild/adr6-rollback-recheck-h6b2ezkt/result.json`.

Passed after the correction: `make check` (all eight gates, including 38 Python
tests), `make build`, both-artifact `make acceptance`, the offline Rust 1.85
public-consumer check, and the full signed-source
`python3 scripts/tests/verify_initialization.py` qualification. The latter reruns
real isolated Rust/npm consumers, normal/force updates, failed-update cache
preservation, known flat-pin migration, clone/setup, offline prime and temporary
package/install/uninstall. Its clean revision
`33328d741db02e195154e1a1dd4921e2199fa0df` is a test fixture, not a published pin.
Document/provenance and whitespace checks also pass.

Code-size uses pinned tokei 15.0.0. The correction changes one implementation
file, `init_transaction.rs`: **241 Code**, fully measured and within 400. Backup,
publication and restore remain one cohesive initialization transaction; no split
is needed. Recounting the complete ADR 0006 changed implementation scope measures
31/31 files, maximum 241 Code, no oversized or unmeasured files. Tests/helpers and
design prose are excluded by responsibility. The full source gate measures 97
implementation files, maximum 362 Code, with no verified size exceptions.

Logs are under `.craftguild/adr6-rollback-`: `before.log`, `focused.log`,
`check.log`, `build.log`, `acceptance.log`, `msrv.log`, `consumer.log`, `size.json`
and `adr-scope-size.json`. Executed runtime host: macOS arm64. Linux, macOS x86_64
and remote CI were not executed. No real consumer/product repository, user HOME,
global Skill, user key, live service or system installation was changed. The Git
index remains unchanged; no real commit/tag/push, release preparation or
publication was performed.

The implementation and ownership/error paths received a read-only self-review.
A new independent review has **not** been performed. `standards-2bi` is reopened
and remains in progress pending that review; the previous closure is not treated
as approval of this correction.
