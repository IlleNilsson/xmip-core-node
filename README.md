# xmip-core-node

The Node: one machine in an Xmip Cluster, its `NodeRole`s and what it
declares. It is the one declaration of what a node is for; anything else that
needs it reads it here rather than carrying a copy (ADR-0044).

`NodeRole` is that declaration (ADR-0056, amendments 2026-10-01): eight
roles — operational, monitoring, receiving, processing, sending, executing,
development, storage. Receiving, processing and sending serve one stage of the
message path each, and executing is their sum, all three in one process, the
low-latency role; storage is Xmip Storage, the doorway every other node calls
for all storage, and serves no stage; `NodeRole::stages` says which stages a role serves, and
`NodeRole::said` says a set of roles one way, the three together as
executing. `NodeRole::declared` is the one parse: words separated by commas
or `+`, each exact and lowercase only (the owner, 2026-09-24), an unknown word
refused by name (ADR-0055); nothing is inferred from a node's name. The
Essentials reads every declaration through it, and every other surface calls
it: the runtime's library forwards `NodeRole::WORDS`, `declared` and `stages`
as `xmip_role_words_v1`, `xmip_role_declared_v1` and `xmip_role_stages_v1`
(`xmip_operate.h` section 7), which `Xmip.Surface` and the estate's PowerShell
module call rather than keep a copy.

`Stage` is a stage of the message path — `receive`, `process`, `send` — and
the scope segment it is named by; a node does not declare one. A stage says
whether an operator may pause it (`pausable`), what a thing configured at
it is called (`location`: Receive Location, Subscription, Send Port) and
how a person reads it (`title`: Receive, Work Process, Send — never a bare
Process), and its words are `xmip_stage_words_v1`, all forwarded the same
way (`xmip_stage_title_v1`; ADR-0027, amendment 2026-11-11).

A scope is an Xmip URI, and the tree is its path (ADR-0027 clauses 3 and 4).
`Scope` reads one — the scheme and the authority go, a slash at either end
is ignored, empty text is the root — `Scope::segments` splits it, and
`Scope::contains` is the one containment rule: at and beneath, by segment,
never by character. `Scope::node` is the node a scope is on — the segment
after the `node` marker beneath the cluster, `xmip:///<cluster>/node/<name>`
(ADR-0053), and none for a scope on no node, since the cluster is never one
— and `Scope::stage` the stage of the message path it is on, beneath its
node or its cluster, so no name is read as a stage. The Event filter, the
audit, observe's snapshot and the runtime ask it alike, and the runtime's
library forwards it as `xmip_scope_contains_v1`, `xmip_scope_parts_v1` and
`xmip_scope_node_v1` (`xmip_operate.h` section 7; ADR-0027, amendment
2026-11-13).

The one wildcard over scopes is `wildcard::matches`: `*` and `?`, literal
everything else, case-insensitive, `*` crossing a `/`, both sides read as
scopes first (ADR-0052, amendment 2026-09-19). The audit read filters by it,
and the runtime forwards it as `xmip_scope_matches_v1`.

`Capability` is what a node declares — its roles and whether it may assume
the internet — and the two forms it is said in: the evidence a node
publishes about itself (`evidence`, `from_evidence`) and the entry a run
lists it by (`entry`, `from_entry`: `RX1=receiving+sending`). The Essentials
declares through it and the surfaces read through it
(`xmip_capability_published_v1`, `xmip_capability_entry_v1`).

`Requirement` is what a piece of work needs of the node that takes it, in
the same terms (ADR-0056 clause 3, amendment 2026-11-13): the stage it needs
served — a Receive Location the receive stage, a Send Port the send stage —
and whether it needs the online capability. `Capability::satisfies` is the
one match: one of the node's roles serves the stage, executing serving every
stage, and the node is online where the work needs it. Which node takes the
work is `xmip-core-cluster`'s `place`.

A System Process's declaration (ADR-0053) is TOML, its strings quoted by
`xmip-core-library-codec`: every control character is escaped, so any name,
location or path is one line a TOML reader takes back exactly. `Declaration`
is what it says — the name, the location and the purpose, and whatever else
the process says of itself (`with`, a bare key and none of the six every
declaration writes), which is how an Essentials node says its flags — and
`to_toml` is its text; `Standing::read` is the one reading of that text back.
The purpose is `test` or `runtime` exactly (`Purpose::declared`); any other
word is REFUSED, as a stage's is. This crate opens no file (ADR-0058 clause
1, amendment 2026-11-13): the file a process declares itself in, its
directory and the listing of every declaration standing there are
`xmip-core-runtime`'s (`declared`), and no other language writes or reads a
declaration — the runtime's library forwards both as `xmip_process_declare_v1`
and `xmip_process_declarations_v1` (`xmip_operate.h` section 13).

A Node is not the Xmip Service that runs on it and not a Host Service; those
are `xmip-core-runtime`'s. A Node does not hold cluster membership — the
Cluster does — and it does not decide where work runs.

`doc/architecture/deployment-model.md` section 3 governs the roles a node
combines; `architecture.toml` carries the maturity.
