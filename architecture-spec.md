# Compatibility Pointer: Use `MASTER_DOC.md`

This repository's source-of-truth product requirements document, architecture spec, domain contract, safety-gate contract, and bead-decomposition contract is:

```text
MASTER_DOC.md
```

Do not treat this file as the canonical specification.

Historical tools and prompts may still reference `architecture-spec.md`; those tools must load `MASTER_DOC.md` before reviewing, decomposing, or implementing Clarity work.
