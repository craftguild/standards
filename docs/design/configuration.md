# Configuration

The versioned repository configuration is `craftguild.toml`, schema version 1.
Unknown fields, unknown adapters/policies, unsafe paths, overlapping source roles,
empty checks and absent required tools are errors. Paths are repository-relative;
absolute paths, `..` traversal and escaping symlinks are forbidden.

Artifacts declare their public command, package/version source, distribution kind,
manifest, locked dependency inputs, feature selection and assets. Source roles
describe actual responsibility: implementation, test, test-helper, fixture,
documentation, vendored or verified-generated. A filename is not role evidence.
Repository checks declare argv and purpose; no custom plugin language is used.

The [architecture](architecture.md) is the design authority. The parser is
[`config.rs`](../../crates/craftguild-standards/src/config.rs); the repository's
[`craftguild.toml`](../../craftguild.toml) is an executable example.

## Root

`schema_version = 1` and `standards_version = "0.1.0"` are required. `artifacts`
is nonempty. Optional arrays are `sources`, `adapters`, `policies`, `generated`,
and `commands`. Quality checks reject missing source roles/adapters/test gates;
build-only operations do not require running unrelated quality gates.

## Artifacts

| Field | Meaning |
| --- | --- |
| id | Unique artifact selection name |
| command | Unique public binary/distribution command and prepared-record name |
| project | Organization project name, without a mandatory language suffix |
| kind | `rust` or `frontend` |
| manifest | Cargo.toml or frontend package.json, relative to repository root |
| lockfile | Exact Cargo workspace lockfile or frontend package-lock.json |
| package | Selected Cargo/npm package name |
| features | Requested Cargo features; default empty |
| no_default_features | Cargo default-feature selection; default false |
| inputs | Additional required files, hashed individually |
| input_dirs | Complete required trees, inventoried and hashed |
| bundle | Embedded/independent Web evidence, required for frontend artifacts |
| supplemental | JSON array of explicit native/font/asset component records |

Cargo workspaces may be nested below the explicit source root; local package
dependencies outside that root fail. Selected normal dependencies become SBOM
components. Local build inputs are also sealed even when build-only packages are
excluded from the distribution's component list. Run builds from the repository
root so its configuration and source-root boundary are unambiguous.

`bundle` requires `root` (bundler working-root path), `lockfile`, `metafile`
(esbuild JSON), `directory` (exact output tree), and a nonempty build `command`
argv executed from the repository root. Install locked npm inputs before building.
Every metafile output must match a real output file and recorded byte length;
every positive byte contribution must match a real input. Node-module inputs map
to package-lock v3 entries and installed package name/version. The actual bundle
may include a development dependency; an unused dependency is never included
merely because it exists in the lockfile. Linked npm packages need supplemental
records. Supplemental records require `name`, `version`, CycloneDX `type`, unique
`reference`, actual `files` and human-readable `evidence` describing inclusion.

Prepared files live under `.craftguild/<command>/`. The SBOM includes shared build
identity, effective root features, all sealed input hashes and directory inventories
as standard CycloneDX properties. Build scripts recapture identity, compare Cargo
features, verify every inventory/hash and schema, then embed those exact bytes.

## Source roles and adapters

Each source entry has a glob `pattern`, `role` and nonempty responsibility
`reason`. Roles are `implementation`, `test`, `test-helper`, `fixture`,
`documentation`, `vendored` and `generated`. Patterns must not overlap. Executable
sources with no role fail. Exclusion reasons are reviewable assertions about use;
they do not grant permission to put product code in a test tree.

Each adapter has `language = "rust"` or `"npm"`, nonempty `test` and `lint` argv,
and optional `format` and `typecheck` argv. Rust requires `format`; npm requires
`typecheck`. Rust and npm source inventories require their respective adapters.
All configured adapter gates execute during `check`, including changed-scope checks.
Only structural source measurement is narrowed by changed scope; global ownership
and dependency invariants remain global.

## Policies, generated outputs and repository gates

`kind = "dependencies"` takes `manifest`, exhaustive first-party `layers` allowed
edges, and optional `restricted` library-to-permitted-package lists. Missing layer
packages and workspace packages without a layer fail. Every declared dependency
kind is inspected, so aliases/dev/build declarations cannot evade a layer rule.

`kind = "sea-orm"` takes `manifest`, nonempty source-glob `sources`, and
`allowed_packages`. It rejects direct drivers/query libraries, raw query/Statement
API syntax and schema-sync opt-in. The syntax policy includes macro identifiers
but does not expand arbitrary external macros or perform compiler type resolution.
Comments and literal strings are not API invocations. Repository checks must retain
stronger product contracts where this guarantee is insufficient.

Generated entries take `owner`, common ownership `manifest`, `output_root`, a
`reproduce` argv containing a separate `{output}` argument, and optional
`rust_size_exception` (false by default). The ownership JSON requires
`schema_version: 1`, matching `owner`, full `generator_revision`, nonempty `inputs`
and `files` hash maps. All paths are relative to the repository. The exact output
inventory/hashes must agree; isolated reproduction must yield identical files.
Only verified `.rs` outputs receive the configured size exception. Existing
generator-specific ownership/ABI rules remain connected through a small converter
and repository gate, rather than being replaced with weaker assertions.

Repository `commands` have unique `id`, nonempty `purpose`, nonempty `argv` and
`phase` (`test`, `check` or `acceptance`). Ordinary tests and checks execute in
`check`; acceptance remains an explicit separate entry. See the
[skill policy example](../../skills/craftguild-standardize/references/policies.toml).
