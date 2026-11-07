# Development contract

| Entry | Required behavior |
| --- | --- |
| `make`, `make build` | Locked release build including required assets and SBOM |
| `make install` | Copy a built executable; never start a build |
| `make test` | Ordinary automated tests for every product language |
| `make check` | Tests, common quality, language and repository checks |
| `make acceptance` | Explicit configured artifact and actual-host acceptance |
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

## Repository initialization and execution

The installed `craftguild-standards` exposes `init [--force]` and offline
help/version/SBOM presentation. The separate `craftguild-standards-engine` owns
all operations; consumers normally call it through their fixed Make/Python entry.

Init uses exactly cwd and selects its own clean published version/full revision.
An absent pin is initialized; newer versions upgrade; the same version/revision
revalidates without mutable lookup or identical-file rewrites. Downgrades and
same-version revision replacement require `--force`. Force never bypasses
integrity, compatibility, ownership or concurrent-edit checks.

Init preserves product config, Make recipes, contracts and Cargo/npm inputs.
The generated starter is deliberately unfinished. Finish artifact configuration,
Make wiring and product adaptation; align common-crate Git revisions manually.
Updates replace only proven managed baselines, publish the pin last and retain
prior bytes/modes for rollback. Initialization is not conformance certification.

Only init and explicit `make setup` may acquire an engine. A clone needs the
committed entry and setup, without installing a global CLI. Setup acquires only
the current pin. Ordinary operations and prime require its verified cache, with
no Latest, compilation or sibling fallback. Prime returns matching embedded
context as read-only offline Markdown, even before product configuration is ready.

`make installer` (`INSTALLER_ARGS`) generates or checks standalone scripts.
`make package` (`PACKAGE_ARGS`) verifies and packages release receipts without
building or running setup. Download installation is distinct from copy-only
source install. The [execution guide](../docs/guides/orchestration.md) describes
these workflows. Historical v0.4.0 uses its single-CLI grammar; bundle references
require explicit migration.
