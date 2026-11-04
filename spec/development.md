# Development contract

| Entry | Required behavior |
| --- | --- |
| `make`, `make build` | Locked release build including required assets and SBOM |
| `make install` | Copy a built executable; never start a build |
| `make test` | Ordinary automated tests for every product language |
| `make check` | Tests, common quality, language and repository checks |
| `make acceptance` | Isolated external-service/actual-host acceptance |

Install honors `PREFIX`, `BINDIR`, `DESTDIR`, with default `/usr/local/bin`.
Missing binaries fail with build instructions. `TARGET` and `CARGO_TARGET_DIR`
select the matching build and install artifact. Privileged install never runs
Cargo/npm. Focused development commands remain available. Report unavailable,
skipped and unexecuted acceptance separately from success.

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
