# Distribution contract

Every shipped artifact has a CycloneDX 1.5 JSON SBOM for its selected target,
features and runtime dependency closure. A lockfile-wide package list is
insufficient. Exclude development/build-only dependencies, unrelated workspace
members and independently distributed frontends. Include actually embedded Web
code, fonts, native components and assets. A separate frontend can generate a
separate SBOM. Explicit supplemental records cover components without automated
discovery; disclose evidence limits rather than imply source-level completeness.

The canonical build fixes identity and locked inputs, builds embedded assets,
maps bundle evidence to package identities, creates/schema-validates the SBOM,
then verifies and embeds it. Record application name/version, revision, target
and features in the SBOM. Keep source file inventories and hashes in local
prepared seals and receipts for developer-side integrity checks; do not include
them or an aggregate source-input digest in consumer-facing SBOM metadata.
Actually shipped asset/component evidence remains part of the SBOM.
Reject missing inputs, stale seals,
schema-invalid JSON and mismatched assets. Direct compiler builds also verify
prepared inputs and explain the canonical preparation command on failure.

Never recurse into Cargo from a build script. Tooling may use a non-distributed
bootstrap executable to prepare standards' own final SBOM. The final shipped
executable always includes its verified real SBOM. Runtime `sbom` needs only the
binary and writes the saved JSON verbatim.

The current engine writes prepared seal schema 2 for this separation.
Published v0.1.0 and its schema 1 artifacts remain unchanged and readable.

Schema 2 supports standard Rust output layout, receipt-backed `package` and
declarative POSIX shell `installer` generation. Standard layout fixes all Rust final outputs
at `<target-dir>/<target-triple>/<profile>/<file>`, including native builds and
library formats. Explicit Rust output overrides are refused in this mode.
Frontend locations and configurations without the mode keep their contracts.

Package requires fresh output, clean matching full HEAD, release receipts and the
owning group's version/target agreement, plus complete seal/SBOM/output validation.
It emits binaries, consumer SBOMs and target checksums without building or acquiring
an engine. Download installers verify an exact SHA-256 entry before atomic command
replacement and use sudo only for destination operations requiring it. Uninstall
removes only the regular command file and preserves configuration/data. Schema 2
installer declarations select executable artifacts, their owning release group,
GitHub repository and safe relative script output. Runtime options select a stable
version and absolute destination; ordinary checks reject generated script drift.


Fixed-reference delivery replaces execution/skill archives with flat host binaries,
matching SBOMs, target checksums and versioned release metadata. The release index
names exact full revision, baseline, entry/execution compatibility and exact HTTPS
asset SHA-256 values by supported host. The engine embeds its matching canonical
contracts and adoption/configuration guidance as owned, sealed build inputs.
Repositories track a fixed reference, common entry, short local skill and product
configuration. Init does not create Git commits or relabel old assets as new features. Published
v0.3.0 and earlier assets/notes/receipts retain their original contracts.


Standards delivery has two artifact identities in the same release group:
`craftguild-standards` (initializer) and `craftguild-standards-engine` (full engine),
with one version and full clean revision. Each has independent embedded/sidecar
SBOMs and checksums. Release metadata schema 3 explicitly names initializer and
engine assets for each qualified host. The public installer selects the initializer;
init verifies and acquires the matching engine. Old full-CLI assets and another
artifact's SBOM cannot substitute for the engine. Published assets remain unchanged.
