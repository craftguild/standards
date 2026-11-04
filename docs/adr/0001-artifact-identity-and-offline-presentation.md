# ADR 0001: Artifact identity and offline presentation

Status: Accepted

Language-neutral contracts and explicit distribution identities are authoritative.
Runtime presentation receives data; application build scripts capture that data;
preparation creates and verifies the artifact's SBOM. These paths share core
identity/seal validation. This avoids runtime discovery and competing lockfile
interpretations. Rust 1.85 consumer libraries remain separate from tool requirements.

The build preparation sequence is transient, with atomic publication of complete
prepared files and no domain database. The architecture records why transition
history and business Header-Detail tables do not apply to technical inventories.
Product domain models remain subject to those baselines during later adoption.
