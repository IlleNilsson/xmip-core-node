# xmip-core-node

The Node: one machine in an Xmip Cluster, its `NodeRole` and what it can do.
It is the one declaration of what a node is for; anything else that needs it
reads it here rather than carrying a copy (ADR-0044).

`Stage` is the one parse of what a node declares it serves on the message
path — `receive`, `process`, `send`, one or more (ADR-0056). `Stage::declared`
reads a declaration separated by commas or `+`, takes each word exactly and
in lowercase only (the owner, 2026-09-24: `RECEIVE` or `Send` is an unknown
word), returns the stages in path order, and refuses an unknown word by name
(ADR-0055);
nothing is inferred from a node's name. The Playground reads every
declaration through it, and every other surface calls it: the runtime's
library forwards `Stage::WORDS` and `Stage::declared` as
`xmip_stage_words_v1` and `xmip_stage_declared_v1` (`xmip_operate.h`
section 7), which `Xmip.Surface` and the estate's PowerShell module call
rather than keep a copy (ADR-0056, amendment 2026-09-24, corrected). A stage
also says whether an operator may pause it (`pausable`) and what a thing
configured at it is called (`location`), forwarded the same way.

`Capability` is what a node declares — its stages and whether it may assume
the internet — and the two forms it is said in: the evidence a node
publishes about itself (`evidence`, `from_evidence`) and the entry a run
lists it by (`entry`, `from_entry`: `edge-01=receive+send`). The Playground
declares through it and the surfaces read through it
(`xmip_capability_published_v1`, `xmip_capability_entry_v1`). Where a
declaration places work is not decided here (open problem 25, row o).

A System Process's declaration (ADR-0053) is TOML, its strings quoted by
`xmip-core-library-codec`: every control character is escaped, so any name,
location or path is one line a TOML reader takes back exactly.

A Node is not the Xmip Service that runs on it and not a Host Service; those
are `xmip-core-runtime`'s. A Node does not hold cluster membership — the
Cluster does — and it does not decide where work runs.

`doc/architecture/deployment-model.md` section 3 governs the roles a node
combines; `architecture.toml` carries the maturity.
