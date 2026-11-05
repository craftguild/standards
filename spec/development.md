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

The source-development execution extension resolves artifact, toolchain, target,
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
lockfiles and artifact scopes. Breaking changes select MAJOR, `feat` MINOR and
`fix` PATCH, with no implicit 0.x exception. Use the greatest required increment
once since the previous release tag; other types alone do not increment. An
explicit first version avoids double-incrementing prepared initial manifests.
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
clone must contain the required standards/skill assets or pinned accessible links;
no sibling checkout or personal absolute path may be required.

Apply design-baselines before modeling state machines/lifecycles or business line
items: record current state plus atomic append-only transition history, and
Header 1:N Detail ownership/cardinality. Explain justified departures. Adoption
alone does not authorize changing unrelated domain models.
