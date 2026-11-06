# Development contract

| Entry | Required behavior |
| --- | --- |
| `make`, `make build` | Locked release build including required assets and SBOM |
| `make install` | Copy a built executable; never start a build |
| `make test` | Ordinary automated tests for every product language |
| `make check` | Tests, common quality, language and repository checks |
| `make acceptance` | Isolated external-service/actual-host acceptance |
| `make release` | Prepare version updates and English notes from Conventional Commits |

Install honors `PREFIX`, `BINDIR`, `DESTDIR`, with default `/usr/local/bin`.
Missing binaries fail with build instructions. `TARGET` and `CARGO_TARGET_DIR`
select the matching build and install artifact. Privileged install never runs
Cargo/npm. Focused development commands remain available. Report unavailable,
skipped and unexecuted acceptance separately from success.

The versioned execution extension resolves artifact, toolchain, target,
profile, features and outputs once. `PROFILE=debug` selects debug from preparation
onward; release remains the default. Make, direct launcher and CI dispatch through
one compatible pinned engine. Configured prerequisites and bounded argv hooks
surround shared assets, dependency evidence, SBOM seal, compiler and verification.
Required toolchains/SDKs must already exist. Publish outputs and success receipts
only after verification; failures and cancellation retain the previous verified
output and record the failed invocation separately. Install requires matching
receipts and an existing engine/cache, without acquisition or bootstrap.

Release preparation uses the [common release contract](releases.md).
Preview precedes application; explicit configuration owns synchronized manifests,
lockfiles and artifact scopes. Breaking changes select MINOR at 0.x and MAJOR
from 1.0.0 onward; `feat` selects MINOR and `fix` PATCH. Use the greatest effective
increment once since the previous release tag. Other types alone do not increment.
Normal first releases retain the configured initial version. Explicit `--major`
requests current major + 1 with minor/patch zero, including on first releases.
Normal 0.x preparation never crosses to 1.x. Retry with the same mode.
Invalid messages and unavailable signatures fail. Preserve unrelated work and
make repeated preparations idempotent. Commit, tag, push and publication remain
separate authorized operations under the OpenPGP signature/stop contract.

README order: project title/description, Quick start, System requirements, Build
requirements, Usage/Public API, Known limitations, Community, Security, License.
Include only applicable sections backed by real behavior. Separate runtime and
build tools; do not invent support, minimum versions, packages or contacts.

Public prose is English. Preserve multilingual behavioral fixtures. `docs/README.md`
indexes `docs/guides/`, `docs/design/` and `docs/adr/`. During migration retain ADR
numbers, statuses and supersession; update links and checks in the same change.

AGENTS.md has a short versioned common reference block and a repository-specific
section for design, product and operational contracts. Do not duplicate numerical
quality rules. Other agent instructions reference the same authority. A standalone
clone keeps a fixed engine reference, common entry and short local skill;
no sibling checkout or personal absolute path may be required.

Apply design-baselines before modeling state machines/lifecycles or business line
items: record current state plus atomic append-only transition history, and
Header 1:N Detail ownership/cardinality. Explain justified departures. Adoption
alone does not authorize changing unrelated domain models.

The ADR 0003 source extension adds `make installer` (`INSTALLER_ARGS`) and
`make package` (`PACKAGE_ARGS`), forwarding to the engine's same CLI commands.
Installer checks compare without writing, including during `make check`. Package
requires an existing verified engine/cache and release receipts; it never runs
setup or compilers. Published v0.2.0 does not provide this extension. Download
installation is distinct from the copy-only source-build install contract above.


The ADR 0004 source extension adds `init` and `prime [--topic NAME]` (not available
in published v0.3.0). Init uses cwd, a clean published implementing engine and
exact host assets. It preserves existing product config/recipes/contracts and
generates an explicitly unfinished starter when config is absent. The shared
entry is standards-owned; only init and explicit `make setup` acquire an engine.
All ordinary operations and prime require its verified fixed repository-local
cache. No Latest, compile or sibling fallback is provided. Prime presents the
same engine's embedded context offline without writes or product execution;
missing/malformed config is preparation status, never conformance success.


ADR 0006 separates the installed `craftguild-standards` initializer from
`craftguild-standards-engine`. Only `init [--force]` is an initializer operation;
help/version/SBOM remain offline. Init selects its own exact clean published
version/revision. Stable SemVer upgrades are allowed; downgrades and equal-version
revision replacement require `--force`. Exact retries reuse the recorded pin and
avoid mutable lookup or identical-file rewrites. Force never bypasses integrity,
compatibility, ownership or concurrent-edit checks. Updates replace only proven
managed baselines and publish the pin last, preserving the old pin/cache on failure.
`make setup` acquires the current pin only. Manually align common-crate Git revisions
and complete product adaptation/checks after updating. Published v0.4.0 retains
its single-CLI and existing-pin behavior; no new release is implied by source work.
