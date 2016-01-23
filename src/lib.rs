extern crate nix;
extern crate rand;
extern crate time;
extern crate timely;
extern crate getopts;
extern crate graph_map;

use rand::{Rng, SeedableRng, StdRng};

#[cfg(linux)]
fn pin_to_core(core: usize) {
    let mut cpu_set = ::nix::sched::CpuSet::new();
    let tgt_cpu = index * cpu_stride;
    cpu_set.set(tgt_cpu);
    let result = ::nix::sched::sched_setaffinity(0, &cpu_set);
}

#[cfg(not(linux))]
fn pin_to_core(_core: usize) {

}

// pins the core in the process, which is a bit silly.
pub fn fetch_edges(index: usize, peers: usize) -> (Vec<(u32, u32)>, usize) {

    let mut opts = getopts::Options::new();
    opts.optopt("", "graph", "prefix of graph file", "FILE");
    opts.optopt("", "nodes", "number of nodes in a random graph", "NUM");
    opts.optopt("", "edges", "number of edges in a random graph", "NUM");
    opts.optopt("", "stride", "stride for CPU pinning, or zero for none", "NUM");

    let mut args = ::std::env::args().skip_while(|x| x != "--").collect::<Vec<_>>();
    args.remove(0);

    // this will assert if parsing fails. could be friendlier.
    let matches = opts.parse(args.into_iter()).ok().unwrap();

    // core pinning
    if let Some(stride) = matches.opt_str("stride") {
        if let Ok(cpu_stride) = stride.parse::<usize>() {
            if cpu_stride > 0 {
                pin_to_core(cpu_stride);
            }
        }
        else {
            println!("could not parse <{}> as a stride", stride);
        }
    }
    
    let mut nodes = 0;
    let mut edges = Vec::new();
    if let Some(filename) = matches.opt_str("graph") {

        let graph = graph_map::GraphMMap::new(&filename);
        nodes = graph.nodes();
        for node in 0..graph.nodes() {
            if node % peers == index {
                for &dest in graph.edges(node) {
                    edges.push((node as u32, dest));
                }
            }
        }

    }
    else {
        if let Some(node_cnt) = matches.opt_str("nodes").and_then(|x| x.parse::<usize>().ok()) {
            nodes = node_cnt;
            if let Some(edge_cnt) = matches.opt_str("edges").and_then(|x| x.parse::<usize>().ok()) {

                // rng is seeded with worker index.
                let seed: &[_] = &[1, 2, 3, index];
                let mut rng: StdRng = SeedableRng::from_seed(seed);

                for _index in 0..(edge_cnt / peers) {
                    edges.push((rng.gen_range(0, node_cnt as u32), rng.gen_range(0, node_cnt as u32)));
                }

            }
            else {
                println!("must specify --edges if not --graph")
            }
        }
        else {
            println!("must specify --nodes if not --graph")
        }
    }

    (edges, nodes)
}