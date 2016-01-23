extern crate time;
extern crate timely;
extern crate graph_scaling;

use graph_scaling::fetch_edges;

use timely::progress::timestamp::RootTimestamp;
use timely::dataflow::*;
use timely::dataflow::operators::{ToStream, Binary, LoopVariable, ConnectLoop};
use timely::dataflow::channels::pact::Exchange;

fn main () {

    timely::execute_from_args(std::env::args(), move |root| {

        let index = root.index() as usize;
        let peers = root.peers() as usize;

        // fetch edges and pin cores (optional).
        let (graph, nodes) = fetch_edges(index, peers);

        let start = time::precise_time_s();

        let mut edges = Vec::new();
        let mut ranks = vec![1.0; (nodes / peers) + 1];   // holds ranks

        let mut going = start;

        root.scoped(|builder| {

            // define a loop variable: messages from nodes to neighbors.
            let (cycle, loopz) = builder.loop_variable::<(u32, f32)>(20, 1);

            graph
                .into_iter()
                .to_stream(builder)
                .binary_notify(&loopz,
                               Exchange::new(|x: &(u32,u32)| x.0 as u64),
                               Exchange::new(|x: &(u32,f32)| x.0 as u64),
                               "pagerank",
                               vec![RootTimestamp::new(0)],
                               move |input1, input2, output, notificator| {

                    // receive incoming edges (should only be iter 0)
                    while let Some((_index, data)) = input1.next() {
                        for (src,dst) in data.drain(..) {
                            edges.push((src / (peers as u32),dst));
                        }
                    }

                    // all inputs received for iter, commence multiplication
                    while let Some((iter, _)) = notificator.next() {

                        // record some timings in order to estimate per-iteration times
                        if iter.inner == 0 { println!("src: {}, dst: {}, edges: {}", ranks.len(), nodes, edges.len()); }
                        if iter.inner == 10 && index == 0 { going = time::precise_time_s(); }
                        if iter.inner == 20 && index == 0 { println!("average: {}", (time::precise_time_s() - going) / 10.0 ); }

                        // wander through destinations
                        let mut session = output.session(&iter);
                        for &(src,dst) in &edges {
                            unsafe {
                                // this happens a lot, so unsafe helps out a fair bit.
                                session.give((dst, *ranks.get_unchecked(src as usize)));
                            }
                        }

                        // clear the values; optional.
                        for s in &mut ranks { *s = 0.0; }
                    }

                    // receive data from workers, accumulate in src
                    while let Some((iter, data)) = input2.next() {
                        notificator.notify_at(&iter);
                        for &(node, rank) in data.iter() {
                            // this could be strength-reduced to a shift if peers is a power of two.
                            unsafe { *ranks.get_unchecked_mut(node as usize / peers) += rank; }
                        }
                    }
                })
                .connect_loop(cycle);
        });
    }); 
}