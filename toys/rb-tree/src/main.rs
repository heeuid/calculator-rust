mod tree;
use rand::prelude::*;
use tree::*;

fn main() {
    let mut rb_tree = RBTree::<u64>::new();
    let mut rng = rand::thread_rng();
    let mut nums: Vec<u64> = vec![];
    for _ in 0..10000 {
        let n = rng.gen();
        if rb_tree.insert(n) {
            nums.push(n);
        }
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

    println!("-----------------------");

    let mut pass = true;
    for (i, n) in nums.iter().enumerate() {
        if !rb_tree.find(n) {
            println!("failed to find {n} at {i}");
            pass = false;
            break;
        }
    }

    if pass {
        println!("complete to find all numbers");
    }

    println!("-----------------------");

    pass = true;
    for (i, n) in nums.iter().enumerate() {
        rb_tree.delete(n);
        let (is_rbt, _, _, _) = rb_tree.check();
        if !is_rbt {
            println!("failed to delete {n} at {i}");
            pass = false;
        }
    }
    
    if pass {
        println!("complete to delete all");
    }

}
