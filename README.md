# rsomics-closeness-vitality

Closeness vitality of every node in an undirected, unweighted graph — value-exact
to NetworkX 3.6.1.

**Closeness vitality** of a node v (Brandes & Erlebach 2005, §3.6.2):

```
vitality(v) = Wiener(G) − Wiener(G − v)
```

where `Wiener(G)` is the sum of shortest-path distances over all **unordered**
node pairs (i.e. each pair counted once). Removing a cut vertex disconnects G−v,
so `Wiener(G−v) = ∞` and `vitality(v) = −∞`.

## Input / output

Graph: an undirected edge list on **stdin** or a file argument — one `u v`
(whitespace-separated string labels) per line. `#` comments and blank lines are
ignored, self-loops dropped, parallel edges collapsed (`nx.Graph` semantics).

Default output (all nodes, sorted by label):

```
0 -inf
1 64
2 19
...
```

Single-node query: `--node LABEL` prints one value. `--json` emits the
rsomics-common envelope.

```bash
rsomics-closeness-vitality karate.edgelist
cat graph.edgelist | rsomics-closeness-vitality --node 5
```

## Performance

Each of the V+1 Wiener computations is a full BFS traversal; total work is
`O(V(V + E))`. Adjacency is `Vec<Vec<u32>>` (interned labels), BFS buffers are
reused across nodes. No allocator pressure in the hot loop.

Pure-Python networkx builds a distance dict-of-dicts then sums — O(V²) memory
and dominated by Python interpreter overhead. On gnm(150, 600) our Rust
implementation is several hundred× faster compute-only.

## Value-exactness

For connected unweighted graphs all vitalities are exact integers (BFS produces
integer distances; Wiener sums and their difference are integers). Stored as
`f64`, they are bit-exact to networkx 3.6.1 across all tested graphs (karate
club, path, cycle, cut-vertex graph, two random gnm graphs) and verified 0-error
against networkx on further independent connected graphs (Watts-Strogatz, random
trees, cycle, complete). The `−∞` case for a vertex whose removal disconnects the
graph matches `float('-inf')` in Python.

The input graph must be **connected**. On a disconnected graph `wiener_index(G)`
is infinite, so networkx's `wiener(G) − wiener(G − v)` collapses to a mix of
`nan`/`±inf`; an edge list also cannot represent the isolated nodes that would
have made it disconnected. Rather than emit a plausible wrong number, this crate
bails loud on disconnected input. (Removing a single cut vertex *from a connected
graph* still yields the correct `−inf` — that case is fully supported.)

Wiener index convention: `nx.wiener_index` for undirected graphs sums over all
ordered pairs then divides by 2 (source-confirmed from networkx 3.6.1). This
crate uses the same convention.

## Origin

This crate is an independent Rust reimplementation of `networkx.closeness_vitality`
based on:

- Brandes, U. & Erlebach, T. (eds.). *Network Analysis: Methodological Foundations*.
  Springer, 2005. §3.6.2.
- The NetworkX 3.6.1 source (`networkx.algorithms.vitality`, BSD-3-Clause) — read
  and cited directly, which its permissive license allows.

Test fixtures and expected outputs are hardcoded full-precision constants captured
from NetworkX 3.6.1; no Python is invoked at test time.

License: MIT OR Apache-2.0.
Upstream credit: NetworkX (https://networkx.org/, BSD-3-Clause).
