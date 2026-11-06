# Release assets and standalone installers

These commands are implemented in the source engine extending ADR 0003. They are
not available in published v0.2.0. Keep a matching engine and fixed entry together;
use a source export during development, without inventing a public revision pin.

Declare an executable artifact, its release group and an installer in
[configuration](../design/configuration.md#standard-rust-outputs-and-installers-source-extension).
Generate and check the script with the common engine:

```sh
craftguild-standards-engine installer
craftguild-standards-engine installer --artifact standards --check
make installer INSTALLER_ARGS=--check
```

Ordinary `check` also rejects script drift. The template is embedded in the engine,
so consumers need no generator script or sibling standards checkout.

After building clean release artifacts at the current full Git HEAD, package all
installer artifacts in the group into a fresh directory:

```sh
craftguild-standards-engine package --group standards --output target/release-assets
make package PACKAGE_ARGS="--group standards --output target/release-assets"
```

Group selection may be omitted only when installer declarations refer to one
group. Receipts must share version/target, match the group's version authority and
the clean current revision, and pass complete input, exact inventory, SBOM and
output validation. The result contains `<command>-v<version>-<target>`, the same
name plus `.bom.json`, and `SHA256SUMS-<target>.txt`. Package reads output paths
from receipts and never builds, resolves dependencies or acquires an engine.
The launcher requires an existing verified cache; run setup/build first.
Standards declares both initializer and engine artifacts in one release group,
with independent receipts, embedded/sidecar SBOMs and checksums. The public
`scripts/install.sh` installs only `craftguild-standards`; its `init` acquires
`craftguild-standards-engine` for the repository. The engine installer declaration
also makes the second artifact available to common packaging. Source metadata
schema 3 explicitly labels both roles per host. Published v0.4.0 remains a single
full CLI and cannot be substituted for the new engine.
Standards' delivery wrapper adds flat versioned host release metadata, then
hashes every asset except checksum files. Local source ledgers stay local.

## Download installation

Serve the generated script as raw source. Once a release containing these assets
has been published, its installation form is:

```sh
curl -fsSL https://raw.githubusercontent.com/craftguild/standards/main/scripts/install.sh | sh -s --
curl -fsSL https://raw.githubusercontent.com/craftguild/standards/main/scripts/install.sh | sh -s -- --version 0.2.0
curl -fsSL https://raw.githubusercontent.com/craftguild/standards/main/scripts/install.sh | sh -s -- --bindir /opt/tools/bin
curl -fsSL https://raw.githubusercontent.com/craftguild/standards/main/scripts/install.sh | sh -s -- --uninstall
```

The explicit version is stable `X.Y.Z`, without the group's tag prefix. The
version example identifies an existing release format; publishing this source
script and future assets is separate work. Check asset availability for the chosen
release and target. `--help` explains the flags without network access. Duplicate,
unknown and incomplete options fail; `--version` cannot accompany `--uninstall`.

The installer needs POSIX sh, curl, standard utilities (including mktemp and awk),
and sha256sum or shasum. Darwin/Linux and x86_64/amd64/arm64/aarch64 map to
`x86_64-apple-darwin`, `aarch64-apple-darwin`,
`x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. Other systems fail;
Linux does not fall back to musl.

Without `--version`, one GitHub Latest Release redirect selects a stable tag with
the configured group's prefix. A different group's Latest fails with an explicit
version instruction. Every asset request then uses that fixed tag. The script
downloads the binary and checksums without elevation, requires one well-formed
SHA-256 entry for the exact asset basename and verifies its bytes before placement.
It never runs a downloaded binary to verify it.

Destination priority is `--bindir`, environment `BINDIR`, then `$HOME/.local/bin`.
Destinations must be nonempty absolute paths; spaces are supported. HOME is needed
only for the default. The installed filename is the artifact's command. Verified
bytes are staged on the destination filesystem, set to 0755 and renamed over the
old binary only after success. Failure or interruption cleans owned temporary
files. Writable destinations and root need no sudo; otherwise only necessary
creation, placement, deletion and cleanup commands use sudo with separate argv.
Authentication or placement failure preserves the old binary. The script reports
a missing PATH entry without editing shell startup files.

Uninstall needs no curl, hash tool or OS/architecture probe. It removes only the
regular file at the command destination; absence succeeds, directories and
symlinks fail. Settings, data, unrelated commands and directories remain.

Source-build `make install` is a separate copy-only operation with the existing
`PREFIX`/`BINDIR`/`DESTDIR` contract and `/usr/local/bin` default.
