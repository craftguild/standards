# Source-development build execution

This branch implements ADR 0002 for a later release. Configuration schema 2,
prepared seal schema 2 and the `craftguild-execution-1` entry protocol are distinct
from package versions and the published v0.1.0 revision. The
[execution contract](../design/orchestration.md) defines all fields and precedence.

Subprocess execution currently requires a Unix host. Non-Unix execution is refused
before spawning tools; Windows engine qualification is separate from consumer
library compatibility.

Build and export a host-specific source bundle inside the standards checkout:

```sh
make build
python3 scripts/package_entry.py --output target/execution-entry
```

Packaging verifies the engine receipt, sealed inputs and exact embedded SBOM.
Its common receipt verifier includes file additions within sealed directories;
rebuild before packaging when any sealed input or inventory changes.
The export identifies local source provenance by digest; it does not claim a
released revision. Its full source inventory remains under `.craftguild` in the
producer checkout. Keep the complete export together when copying it to a
standalone consumer as `standards/execution`:

```makefile
ARTIFACT ?= example-cli
include standards/execution/make/common.mk
```

`make setup` verifies and copies the vendored engine into a repository-local cache.
Build can perform that setup on first use. No sibling checkout, global engine or
remote include is required. Entry/engine protocol mismatch, changed hashes and
unsafe paths fail closed. Install requires an existing verified cache and output;
it never sets up, downloads or builds anything.

Declare artifacts and bounded hooks in schema 2. CLI selectors override their
environment equivalents, then configuration, then defaults. `ARTIFACT`, `VARIANT`,
`TOOLCHAIN`, `TARGET`, `PROFILE`, `FEATURES`, `NO_DEFAULT_FEATURES` and
`CARGO_TARGET_DIR` work with the common Make entry. Examples:

```sh
make build PROFILE=debug
python3 standards/execution/launcher.py build --artifact example-cli --profile release
make install PREFIX=/opt/example DESTDIR=/tmp/example-stage
```

CI may invoke the verified cached engine directly with the same arguments from
the consumer root. A named variant or target setting can retain platform-specific
compiler prefixes and output layouts. Libraries declare target name and formats;
frontend artifacts use actual bundler evidence and do not probe Rust tools.

The engine records the resolved plan and ordered results under
`.craftguild/<command>/runs/`. `process.jsonl` captures argv, output and failure or
cancellation. A successful build publishes the verified output and `receipt.json`
last. Failed attempts retain the previous verified output/receipt and their own
failure history. Source inventories and hashes stay in these local records;
consumer SBOM metadata carries build identity and effective features.

Operation declarations prepare/build the needed artifacts before ordinary gates.
Keep acceptance explicit. Product hooks retain SDK, ABI and exact generation
contracts; they do not repeat the engine's orchestration. Actual toolchains, SDKs
and locked dependencies are prerequisites. Record unavailable platforms separately
from successful local checks; exporting this bundle does not authorize product
adoption or publication.

Use `acceptance --gate ID` (or `make acceptance ACCEPTANCE_ARGS="--gate ID"`) when a public
alias must run only one declared acceptance gate. Ordinary test/check entries do
not include acceptance commands.
