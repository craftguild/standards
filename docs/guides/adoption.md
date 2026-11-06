# Later product adoption

Stage one ends after standards publication. Do not change a product until a later
request selects it. The seven planned public product names are:

| Product | Public repository |
| --- | --- |
| Oriha | `craftguild/oriha.rs` |
| Chigiri | `craftguild/chigiri.rs` |
| Musubi | `craftguild/musubi.rs` |
| pm9 | `craftguild/pm9.rs` |
| AgentChat | `craftguild/agentchat.rs` |
| Shiori | `craftguild/shiori.rs` |
| Kicker | `craftguild/kicker.rs` |

Start with the repository's own instructions and accepted designs. Preserve
existing machine version formats, CLI grammar, generator metadata, immutable
migrations and operational state. Classify every current gate before replacing
anything: shared policy, language adapter or retained product-specific command.

Current migration decisions include reconciling physical-line and Code-line
budgets, test exclusions and verified canonical generated Rust exceptions in the
same change as instructions/checkers/tests/design rules. Keep format, compile,
Clippy, schema/config/ABI, ownership/hash and exact regeneration checks. Numbers
and exclusions refer to the versioned common contract instead of being copied
into AGENTS.md. Keep other agent files aligned through the same authority.

Shiori currently has sibling-checkout dependencies on its generator and renderer.
AgentChat currently pins generator/renderer Git dependencies using SSH. Resolve
these through the final public names and full immutable HTTPS revisions after
dependency publication. Check anonymous retrieval, regenerate contracts/provenance,
then freeze consumer public commits. Keep independent frontends separate from
backend SBOMs; include only actually embedded assets in embedded-Web products.

Use `.private/oss-archive/` for evaluation originals and preservation records,
excluded from Git/public export. Preserve necessary public fixtures, goldens and
generator provenance by content/role. Remove personal paths and internal names
in context, without blind replacement of legitimate dependency names or fixtures.
Apply MIT OR Apache-2.0 to first-party product content while retaining third-party
rights. Standards introduction does not authorize unrelated domain redesign.

Move public documentation through `docs/README.md`, `guides/`, `design/` and `adr/`.
Preserve ADR numbers, status, supersession and links. Export reviewed public files
to a separate directory and create fresh public Git history there; keep originals
and working history locally. Publish dependency histories first, replace pinned
references and generation provenance, then commit consumers and build final
binaries from clean public commits. Do not remove old hashes before replacing
their references. Ordinary standardization is not publication/install/service
authorization; honor the later request's actual scope.


For later authorized adoption of a release implementing ADR 0004, run init from
that product's explicit root, then prime and relevant topics. Preserve existing
Make/config/contracts; finish real configuration and wire the shared entry.
Standards owns font-renderer, measurement and common-engine tests. Do not retain
an obsolete local adapter's font tests by moving them into product core or adding
figlet-rs/vendor. Keep SDK/ABI, canonical generation and domain gates; add only
uncovered actual connection checks. General measurement gaps remain upstream.
Published v0.3.0 lacks this workflow; wait for an implementing release instead of
inventing a public consumer pin. Optional old bundle-based pilot migration scripts
have been retired. Their common guarantees are covered by standards' execution,
flat entry, packaging and standalone-consumer tests; actual product gates remain
with the unmodified products.


The ADR 0006 source initializer `craftguild-standards` exposes only `init [--force]`
and offline presentation. Repository operations use the pinned full
`craftguild-standards-engine` through Make/launcher. Init selects its invoking
published version/revision: upgrades are allowed; downgrades and same-version
revision replacement need force. Force does not bypass verification or ownership.
Exact retries reuse the reference; `make setup` only restores that fixed engine.
Manually align common-crate Git revisions with the reported full revision and
complete product adaptation/checks. Published v0.4.0 retains its old single-CLI
behavior; do not assume it accepts the new flag or invent a public source pin.
