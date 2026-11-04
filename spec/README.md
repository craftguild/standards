# Craftguild standards 0.1.0

These contracts apply to products in the Craftguild organization regardless of
implementation language. Public product names do not require a language suffix.
Machine configuration and agent instructions must refer to these same rules.

| Contract | Scope |
| --- | --- |
| [CLI](cli.md) | Offline presentation and build identity |
| [Distribution](distribution.md) | Artifact-specific CycloneDX and input seals |
| [Quality](quality.md) | Roles, measurement, adapters and policies |
| [Development](development.md) | Make, documentation and agent instructions |
| [Publication](publication.md) | Independent clones, archives, licensing and history |

Conformance data in `conformance/` is the executable companion to these rules.
Adapters must report unsupported or unmeasured cases, never silently pass them.
