mod tree;
use rand::prelude::*;
use tree::*;

fn main() {
    let mut rb_tree = RBTree::<f64>::new();
    let mut rng = rand::thread_rng();
    // let mut nums: Vec<u64> = (0..10000).collect();
    // nums.shuffle(&mut rng);
    for _ in 0..1000000 {
        rb_tree.insert(rng.gen());
    }
    println!("{:?}", rb_tree);
    let check_res = rb_tree.check();
    println!(
        "[{}proper rbtree]\ntotal: {} nodes, black height: {} nodes\nmin_depth: {} nodes, max_depth: {} nodes",
        if check_res.0 { "" } else { "un" },
        rb_tree.len(),
        check_res.1,
        check_res.2,
        check_res.3
    );
}
