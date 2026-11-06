# Initialization and pinned context

ADR 0004 replaces archive delivery in the source engine. Published v0.3.0 does
not implement it. The normative baseline remains 0.1.0 and the execution receipt
protocol remains `craftguild-execution-1`.

## Reference and entry ownership

`standards/engine.json` is the one committed provenance authority. Reference
schema 2 records the engine version, full clean source revision, baseline,
execution protocol, entry protocol `craftguild-entry-1`, exact HTTPS binary/SBOM
assets by supported host, and hashes of the two managed entry files:
`standards/launcher.py` and `standards/make/common.mk`. No archive or source ledger
is distributed. A versioned release metadata index combines independently
qualified host deliveries; unavailable hosts are absent, never inferred from
another host's asset. Init checks the index against the invoking clean tool.

Init's root is exactly cwd. New references require published version metadata;
source builds without matching publication are refused. Existing references are
reused without a mutable lookup or implicit upgrade. A different installed tool
delegates to the existing verified pinned engine for initialization and prime.
Consumers later use that same full revision in common-crate declarations.
Known old pins at `standards/manifest.json` or `standards/execution/engine.json`
require explicit migration. Init checks them before release lookup, engine
acquisition or delegation, and refuses to create a second pin authority. It leaves
the legacy files and product Make recipes untouched and provides no legacy loader.

The small common Python launcher owns host selection, HTTPS acquisition and
verification. It acquires only on explicit setup (also called by init). Products
do not implement acquisition. Every operational dispatch verifies entry hashes,
binary/SBOM hashes, revision, version, host and compatibility. Downloaded code is
never executed before its bytes match the reference. Cache entries are identified
by reference digest and host under `.craftguild/engine/` and published from owned
staging. Invalid existing entries fail closed; a failed acquisition preserves
any valid entry. Normal operations and prime have no acquisition fallback.

## Initialization consistency boundary

The generated set consists of the reference and common entry, a short local
`.agents/skills/craftguild-standardize/SKILL.md`, the known AGENTS common block,
a starter config only if absent, a Makefile only if absent, and a deduplicated
root `/.craftguild/` ignore rule. Existing config and Make recipes remain owned
by the product. Init reports the shared include needed for manual Make wiring.
The common AGENTS block names the normative baseline and keeps product contracts
outside that managed block, matching the ordinary instruction quality gate.
The starter has an empty artifacts array and commented examples; operational
validation remains strict.

Plan all writes and check conflicts/symlinks before acquisition or publication.
Hold an exclusive init lock, stage replacements, recheck observed bytes before
each write and publish the reference last. On failure restore only still-owned
written bytes, in reverse order; preserve unrelated/concurrently changed data
and valid cache entries. Identical files require no writes. Never initialize Git,
search parent/sibling checkouts, untrack files or modify global settings.

This is a transient technical operation: immutable references, staging and cache
identity retain provenance. Rejected/no-op attempts are not transitions. There
is no business entity or business line-item aggregate, so no persistent business
transition table or Header-Detail records are introduced (design-baselines).

## Prime

Canonical contracts and adoption/configuration guidance are embedded and sealed
engine inputs. Public prime delegates to the pin's verified cache. The internal
context renderer checks that its own identity matches the pin; it reads product
configuration without running product code. Missing, malformed or incomplete
configuration is preparation status, never conformance success.

Output is banner-free Markdown on stdout. Topics render the matching embedded
document with repository-relative links resolved to full-revision source links.
Prime never downloads, writes, compiles, generates or checks products. The
overview assigns common engine/font/measurement tests to standards and SDK/ABI,
generator, domain and actual integration checks to products. Generic counter gaps
remain upstream work; no product-specific counter is generated.
