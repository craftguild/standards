# Release contract

Use [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)
and [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html). Breaking `!` and uppercase
`BREAKING CHANGE` / `BREAKING-CHANGE` footers select MINOR while the selected
group is at 0.x, and MAJOR from 1.0.0 onward. Otherwise `feat` selects MINOR and
`fix` PATCH. Other types alone do not increment. Apply the greatest effective
increment once. Normal 0.x releases never cross to 1.x; overflow is an error.
Invalid messages are failures, never guessed patches.

Explicit release groups own version authorities, synchronized Cargo/npm manifests
and lockfiles, literal source paths, initial versions, tag prefixes and public
notes directories. Independent frontend/backend releases use separate groups.
The current implementation supports stable versions; prerelease/build metadata
requires an explicit separate procedure and fails automatic preparation.

With no previous tag, normal preparation retains the configured initial version and include scoped
history from the root. Later use the greatest reachable group SemVer tag and the
observed full HEAD. Require full history/all tags and valid OpenPGP signatures.

`make release RELEASE_ARGS=--json` previews current/next versions, comparison
revisions, affected files and English notes. `make release` applies local
preparation. The common tool equivalents are `release --json` and `release --apply`.
No command implicitly commits, tags, pushes, publishes or installs anything.

Refuse unrelated dirty work, changed inputs and existing target tags. Retain an
immutable preparation receipt with exact file hashes so reexecution cannot
double-increment or duplicate notes, including after committing only prepared
edits. New source work during pending preparation requires finishing or explicitly
reverting it. After signed/verified preparation commits, build final artifacts
from the clean commit and run acceptance before authorized tagging/publication.
The publication contract governs signing failures: stop, never bypass or continue
to tag/push. Simulated failure tests use isolated resources only.


Explicit `release --major` selects current major + 1 with minor and patch zero,
including first releases (using `initial_version`), empty post-tag ranges and
non-bumping history. It overrides increment selection only: message/signature,
full-history, ownership, dirty-input and tag protections still apply. Breaking
explanations and full commit identities remain in notes. Use `--major` for preview,
apply and every retry, for example `make release RELEASE_ARGS="--major --json"`
then `make release RELEASE_ARGS="--major --apply"`. Receipt schema 2 records
`mode` (`normal` or `explicit-major`) and the effective `bump`; initial normal
preparation has no increment. Conflicting pending modes fail before writes.
Historical receipts do not authorize an implicit 0.x to 1.x transition.
