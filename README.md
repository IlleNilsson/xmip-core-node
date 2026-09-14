# xmip-core-node

The Node: one machine in an Xmip Cluster, its `NodeRole` and what it can do.
It is the one declaration of what a node is for; anything else that needs it
reads it here rather than carrying a copy (ADR-0044).

A Node is not the Xmip Service that runs on it and not a Host Service; those
are `xmip-core-runtime`'s. A Node does not hold cluster membership — the
Cluster does — and it does not decide where work runs.

`doc/architecture/deployment-model.md` section 3 governs the roles a node
combines; `architecture.toml` carries the maturity.
