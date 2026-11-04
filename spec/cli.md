# CLI contract

Root invocation, `-h` and `--help` write identical banner, centered
`craftguild/<project_name>` affiliation and root help to stdout. `-V` and
`--version` write identical banner, affiliation and application build details.
`sbom` writes the embedded CycloneDX JSON bytes unchanged, with no envelope.
All succeed with exit 0 only if output succeeds. Handle these paths before
configuration, discovery, state, daemon startup or communication.

Subcommand help has no large banner. Existing machine output, including structured
version metadata, retains its product contract. The common library does not parse
product arguments. Affiliation centers against the longest banner line, using
plain ASCII with no colors, control sequences or terminal-width adjustment.

Build details identify the public executable command and application version,
full source commit, dirty state, actual compiler, compilation target and profile.
Unknown values are `unknown`; unknown dirty state is distinct from clean. Include
staged, unstaged and untracked changes, excluding ignored paths. Add `(dirty)`
only for known dirty. Capture at application build time, including normal
rebuilds and linked worktrees. Never inherit a library's metadata or an archive's
unrelated parent repository. Runtime output invokes neither Git nor compilers.

Banners use pinned FIGlet Small Slant font data and a Rust renderer. Accept only
nonempty printable ASCII input and reject unsupported characters. Remove trailing
spaces, leading/trailing empty rows, and retain exactly one final newline; preserve
leading spaces within nonempty rows. Generation supports stdout, `--output` and
nonmutating `--check`. Products commit and embed the generated text, requiring
neither Node.js, fonts nor generation tools at build/run time.
