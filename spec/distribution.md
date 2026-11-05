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
