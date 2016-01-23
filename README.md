# Graph scaling

A testbed for evaluating the scaling properties of graph algorithms and infrastructure.

This project is designed to provide some measurement tools and methodology for evaluating the scaling properties of graph algorithms and infrastructure. Specifically, we are interested in when and why graph algorithms scale poorly. Initial result indicate that graph algorithms are not fundamentally bad at scaling, but that somewhere along the path from purely random graphs to "real world graphs" scaling falls over. We want to know where and why this happens.

The intended use is to run `cargo` specifying a computation (list will evenutally be in `/src/bin`) followed by timely parameters, followed by application parameters (to be described).

    cargo run --release --bin computation1 -- <timely params> -- <experiment params>

If you don't have any timely parameters, it seems important to use `--` in their place. That is, you should have three (3!) `--` in sequence. Cargo seems to eat the first two, and this causes timely to try and parse the experiment parameters.

Experiment parameters are currently:

*  `--graph <prefix>` indicates a graph to use as input. The format is that used by the [graph_map](https://github.com/frankmcsherry/graph-map) library. There are binaries `parse` and `parse-pairs` in the `src/bin/` directory that can help put graph data in the correct format.
*  `--nodes <number>` indicates a number of nodes to use in a random graph. This is mandatory if `--graph` isn't specified.
*  `--edges <number>` indicates a number of edges to use in a random graph. This is mandatory if `--graph` isn't specified.
*  `--stride <number>` indicates a multiplier to apply to worker indices when pinning them to cores. A value of zero disables core pinning.