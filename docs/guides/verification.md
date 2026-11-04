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
| Signing | OpenPGP verification for every real/test commit, missing-signature detection and isolated simulated failure with no publication |
| Platforms | macOS/Linux CI and Rust 1.85 consumer libraries |
| Publication | Reviewed inventory, public HTTPS clone/build, full-revision consumer, green CI, v0.1.0 |

No release result is claimed until these checks run against the final source.
Record unavailable environments and unimplemented coverage explicitly.
