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


For authorized adoption, follow the [execution guide](orchestration.md). Install
the initializer, run init from the product root, then read matching prime topics.
Preserve existing Make/config/contracts, wire the shared entry and configure real
artifacts. Common-crate Git declarations must use the full revision reported by
the pin; init does not edit dependencies or lockfiles.

The initializer selects its own published version/revision. Upgrades are allowed;
downgrades and same-version revision changes require `--force`. Force retains
integrity, compatibility and local-edit protection. Clones need only the committed
entry and `make setup`; setup never updates. Historical v0.4.0 keeps its single
CLI, and old bundle references require explicit migration rather than a guessed pin.

Standards owns font-renderer, measurement and common-engine tests. Retire obsolete
adapter tests instead of moving them into product core or retaining figlet-rs/vendor.
Keep SDK/ABI, canonical generation and domain gates; add only uncovered connection
checks. Generic measurement gaps belong upstream. Product adoption and publication
still require the active request's scope.
