use std::{
    cell::RefCell,
    cmp::Ordering,
    fmt::{self, Debug},
    rc::Rc,
};

#[derive(Copy, Clone, Debug)]
enum Color {
    Red,
    Black,
}

struct RBTreeNode<T: PartialOrd + Debug> {
    data: T,
    color: Color,
    parent: Option<Rc<RefCell<RBTreeNode<T>>>>,
    lchild: Option<Rc<RefCell<RBTreeNode<T>>>>,
    rchild: Option<Rc<RefCell<RBTreeNode<T>>>>,
}

enum Child {
    Left,
    Right,
    None,
}

impl<T: PartialOrd + Debug> RBTreeNode<T> {
    pub fn new(data: T) -> RBTreeNode<T> {
        RBTreeNode {
            data,
            color: Color::Red,
            parent: None,
            lchild: None,
            rchild: None,
        }
    }

    // is node left child or right child of its parent
    fn which_child(node: Rc<RefCell<RBTreeNode<T>>>) -> Child {
        if let Some(parent) = &node.borrow().parent {
            let left = parent.borrow().lchild.clone();
            let right = parent.borrow().rchild.clone();
            match (left, right) {
                (Some(child), None) if Rc::ptr_eq(&child, &node) => Child::Left,
                (None, Some(child)) if Rc::ptr_eq(&child, &node) => Child::Right,
                (Some(lchild), Some(rchild)) => {
                    if Rc::ptr_eq(&lchild, &node) {
                        Child::Left
                    } else if Rc::ptr_eq(&rchild, &node) {
                        Child::Right
                    } else {
                        Child::None
                    }
                }
                (_, _) => Child::None,
            }
        } else {
            Child::None
        }
    }

    fn get_childs_string(&self) -> String {
        let data = &self.data;
        let parent = match &self.parent {
            Some(var) => format!("{:?}", var.borrow().data),
            None => "_".to_string(),
        };
        let lchild = match &self.lchild {
            Some(child) => format!("{:?}", child.borrow().data),
            None => "_".to_string(),
        };
        let rchild = match &self.rchild {
            Some(child) => format!("{:?}", child.borrow().data),
            None => "_".to_string(),
        };
        format!(
            "[{:?}({:?}): (p{:?},l{:?},r{:?})]",
            data, self.color, parent, lchild, rchild
        )
    }

    fn get_childs_string_chain(&self) -> String {
        let mut result = String::new();
        result.push_str(self.get_childs_string().as_str());
        result.push('\n');
        if let Some(ref child) = self.lchild {
            result.push_str(child.borrow().get_childs_string_chain().as_str());
        }
        if let Some(ref child) = self.rchild {
            result.push_str(child.borrow().get_childs_string_chain().as_str());
        }
        result
    }

    fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    // get parent's sibling
    fn get_uncle(&self) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        if let Some(parent) = self.parent.clone() {
            if let Some(grand_parent) = parent.borrow().parent.clone() {
                match Self::which_child(parent.clone()) {
                    Child::Left => return grand_parent.borrow().rchild.clone(),
                    Child::Right => return grand_parent.borrow().lchild.clone(),
                    Child::None => return None,
                }
            }
        }

        None
    }
}

pub struct RBTree<T: PartialOrd + Debug> {
    root: Option<Rc<RefCell<RBTreeNode<T>>>>,
    cnt: u32,
}

impl<T: PartialOrd + Debug> Debug for RBTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref root) = &self.root {
            write!(
                f,
                "{}\n{} nodes",
                root.borrow().get_childs_string_chain().trim(),
                self.cnt,
            )
        } else {
            write!(f, "{} nodes: None", self.cnt)
        }
    }
}

impl<T: PartialOrd + Debug> RBTree<T> {
    pub fn new() -> RBTree<T> {
        RBTree { root: None, cnt: 0 }
    }

    pub fn len(&self) -> u32 {
        self.cnt
    }

    pub fn insert(&mut self, data: T) -> bool {
        let new_node = Rc::new(RefCell::new(RBTreeNode::new(data)));
        let mut p = None; // tracing parent node for new node
        let mut c = self.root.clone(); // tracing current node

        // find out parent node for new node
        while let Some(current) = c.clone() {
            p = Some(current.clone());
            let current_borrowed = current.borrow();
            match new_node.borrow().data.partial_cmp(&current_borrowed.data) {
                Some(order) => match order {
                    Ordering::Less => c = current_borrowed.lchild.clone(),
                    Ordering::Greater => c = current_borrowed.rchild.clone(),
                    Ordering::Equal => c = current_borrowed.rchild.clone(),
                },
                None => return false,
            }
        }

        // set new node to parent as a child
        if let Some(parent) = p {
            let mut new_node_borrowed = (*new_node).borrow_mut();
            let mut parent_borrowed = (*parent).borrow_mut();
            if new_node_borrowed.data < parent_borrowed.data {
                parent_borrowed.lchild = Some(new_node.clone());
                new_node_borrowed.parent = Some(parent.clone());
            } else {
                parent_borrowed.rchild = Some(new_node.clone());
                new_node_borrowed.parent = Some(parent.clone());
            }
        } else {
            // root has no parent
            self.root = Some(new_node.clone());
        }

        self.cnt += 1;

        // fix balance of rb tree
        self.fix_insert(new_node.clone());

        true
    }

    //   P           X
    //     X  =>   P
    //   C           C
    fn rotate_left(&mut self, node: Rc<RefCell<RBTreeNode<T>>>) {
        let parent = match node.borrow().parent.clone() {
            Some(p) => p.clone(),
            None => return,
        };
        let grand_parent = parent.borrow().parent.clone();
        let lchild_of_node = node.borrow().lchild.clone();

        // P<--C
        if let Some(lc) = &lchild_of_node {
            (*lc).borrow_mut().parent = Some(parent.clone());
        }

        // GP<-->X
        if let Some(gp) = grand_parent.clone() {
            match RBTreeNode::<T>::which_child(parent.clone()) {
                Child::Left => (*gp).borrow_mut().lchild = Some(node.clone()),
                Child::Right => (*gp).borrow_mut().rchild = Some(node.clone()),
                _ => {}
            }
        } else {
            self.root = Some(node.clone());
        }
        (*node).borrow_mut().parent = grand_parent.clone();

        // X<-->P-->C
        (*parent).borrow_mut().parent = Some(node.clone());
        (*parent).borrow_mut().rchild = lchild_of_node.clone();
        (*node).borrow_mut().lchild = Some(parent.clone());
    }

    //   P      X
    // X    =>    P
    //   C      C
    fn rotate_right(&mut self, node: Rc<RefCell<RBTreeNode<T>>>) {
        let parent = match node.borrow().parent.clone() {
            Some(p) => p.clone(),
            None => return,
        };
        let grand_parent = parent.borrow().parent.clone();
        let rchild_of_node = node.borrow().rchild.clone();

        // P<--C
        if let Some(rc) = &rchild_of_node {
            (*rc).borrow_mut().parent = Some(parent.clone());
        }

        // GP<-->X
        if let Some(gp) = grand_parent.clone() {
            match RBTreeNode::<T>::which_child(parent.clone()) {
                Child::Left => (*gp).borrow_mut().lchild = Some(node.clone()),
                Child::Right => (*gp).borrow_mut().rchild = Some(node.clone()),
                _ => {}
            }
        } else {
            self.root = Some(node.clone());
        }
        (*node).borrow_mut().parent = grand_parent.clone();

        // X<-->P-->C
        (*parent).borrow_mut().parent = Some(node.clone());
        (*parent).borrow_mut().lchild = rchild_of_node.clone();
        (*node).borrow_mut().rchild = Some(parent.clone());
    }

    fn fix_insert(&mut self, node: Rc<RefCell<RBTreeNode<T>>>) {
        if node.borrow().is_root() {
            (*node).borrow_mut().color = Color::Black;
            return;
        }

        let parent = node.borrow().parent.clone();
        let uncle = node.borrow().get_uncle();

        let (pcolor, ucolor) = match (&parent, &uncle) {
            (Some(p), Some(u)) => (p.borrow().color, u.borrow().color),
            (Some(p), None) => (p.borrow().color, Color::Black),
            (None, _) => return,
        };

        match (pcolor, ucolor) {
            // Recoloring (case1)
            (Color::Red, Color::Red) => {
                if let Some(u) = uncle {
                    (*u).borrow_mut().color = Color::Black;
                }
                if let Some(p) = parent {
                    (*p).borrow_mut().color = Color::Black;
                    let pp = (*p).borrow().parent.clone();
                    if let Some(gp) = pp {
                        (*gp).borrow_mut().color = Color::Red;
                        self.fix_insert(gp.clone());
                    }
                }
            }
            // Rotating (case2)
            (Color::Red, Color::Black) => {
                let new_node_dir = RBTreeNode::<T>::which_child(node.clone());
                let parent_dir = if let Some(p) = &parent {
                    RBTreeNode::<T>::which_child(p.clone())
                } else {
                    return;
                };
                match (parent_dir, new_node_dir) {
                    // case2-1-1 (LL)
                    (Child::Left, Child::Left) => {
                        if let Some(p) = &parent {
                            if let Some(gp) = &(*p).borrow().parent {
                                (*gp).borrow_mut().color = Color::Red;
                            }
                            (*p).borrow_mut().color = Color::Black;
                            self.rotate_right(p.clone());
                        }
                    }
                    // case2-1-2 (RR)
                    (Child::Right, Child::Right) => {
                        if let Some(p) = &parent {
                            if let Some(gp) = &(*p).borrow().parent {
                                (*gp).borrow_mut().color = Color::Red;
                            }
                            (*p).borrow_mut().color = Color::Black;
                            self.rotate_left(p.clone());
                        }
                    }
                    // case2-2-1 (LR)
                    (Child::Left, Child::Right) => {
                        // make this case2-1-1 (LL)
                        self.rotate_left(node.clone());
                        if let Some(p) = &parent {
                            // for case2-1-1 (LL)
                            self.fix_insert(p.clone());
                        }
                    }
                    // case2-2-2 (RL)
                    (Child::Right, Child::Left) => {
                        // make this case2-1-2 (RR)
                        self.rotate_right(node.clone());
                        if let Some(p) = &parent {
                            // for case2-1-2 (RR)
                            self.fix_insert(p.clone());
                        }
                    }
                    (_, _) => {}
                }
            }
            (Color::Black, _) => { /* O.K. */ }
        }
    }

    fn _find_node(
        current_node: Rc<RefCell<RBTreeNode<T>>>,
        data: &T,
    ) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        match &current_node.borrow().data.partial_cmp(data) {
            Some(res) => match res {
                Ordering::Greater => {
                    if let Some(lchild) = &current_node.borrow().lchild {
                        Self::_find_node(lchild.clone(), data)
                    } else {
                        None
                    }
                }
                Ordering::Less => {
                    if let Some(rchild) = &current_node.borrow().rchild {
                        Self::_find_node(rchild.clone(), data)
                    } else {
                        None
                    }
                }
                Ordering::Equal => Some(current_node.clone()),
            },
            None => None,
        }
    }

    fn find_node(&self, data: &T) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        if let Some(root) = &self.root {
            Self::_find_node(root.clone(), data)
        } else {
            None
        }
    }

    fn find_successor(&self, node: Rc<RefCell<RBTreeNode<T>>>) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        if let Some(rchild) = &node.borrow().rchild {
            let mut current_node = rchild.clone();
            while let Some(lchild) = current_node.clone().borrow().lchild.clone() {
                current_node = lchild;
            }
            Some(current_node)
        } else {
            None
        }
    }

    fn _check(
        node: Rc<RefCell<RBTreeNode<T>>>,
        blacks: u32,
        vec: &mut Vec<u32>,
        depth: u32,
        min_depth: &mut u32,
        max_depth: &mut u32,
    ) -> bool {
        let lchild = node.borrow().lchild.clone();
        let rchild = node.borrow().rchild.clone();
        let mut res = true;
        if let Some(left) = &lchild {
            if left.borrow().data >= node.borrow().data {
                return false;
            }
            if let Color::Black = left.borrow().color {
                res = Self::_check(
                    left.clone(),
                    blacks + 1,
                    vec,
                    depth + 1,
                    min_depth,
                    max_depth,
                );
            } else {
                res = Self::_check(left.clone(), blacks, vec, depth + 1, min_depth, max_depth);
            }
        } else {
            vec.push(blacks);
            if *min_depth > depth {
                *min_depth = depth;
            }
            if *max_depth < depth {
                *max_depth = depth;
            }
        }
        if let Some(right) = &rchild {
            if right.borrow().data <= node.borrow().data {
                return false;
            }
            if let Color::Black = right.borrow().color {
                res = Self::_check(
                    right.clone(),
                    blacks + 1,
                    vec,
                    depth + 1,
                    min_depth,
                    max_depth,
                );
            } else {
                res = Self::_check(right.clone(), blacks, vec, depth + 1, min_depth, max_depth);
            }
        } else {
            vec.push(blacks);
            if *min_depth > depth {
                *min_depth = depth;
            }
            if *max_depth < depth {
                *max_depth = depth;
            }
        }
        res
    }

    // returns (rb-tree or not, black cnt, min_depth, max depth)
    pub fn check(&self) -> (bool, u32, u32, u32) {
        let mut cnt = 0;
        let mut max_depth = 0;
        let mut min_depth = u32::MAX;
        let mut vec = vec![];
        if let Some(root) = self.root.clone() {
            if !Self::_check(root, 1, &mut vec, 1, &mut min_depth, &mut max_depth) {
                return (false, 0, 0, 0);
            }
            if !vec.is_empty() {
                let a = vec[0];
                for x in &vec[1..] {
                    if a != *x {
                        return (false, 0, 0, 0);
                    }
                }
                cnt = a;
            }
        }
        (true, cnt, min_depth, max_depth)
    }
}

#[cfg(test)]
mod test {
    use crate::RBTree;
    use rand::prelude::*;
    #[test]
    fn insertion_test() {
        let mut rbt = RBTree::<u64>::new();
        let mut rng = rand::thread_rng();
        let mut nums: Vec<u64> = (0..1000000).collect();
        nums.shuffle(&mut rng);
        for n in &nums {
            rbt.insert(*n);
        }
        let (is_rbt, blacks, min_depth, max_depth) = rbt.check();
        assert!(is_rbt);
        assert!(blacks <= min_depth);
        assert!(blacks * 2 >= max_depth);
    }
    #[test]
    fn find_test() {
        let mut rbt = RBTree::<u64>::new();
        let mut rng = rand::thread_rng();
        let (min, max) = (0, 1000000);
        let mut nums: Vec<u64> = (min..max).collect();
        nums.shuffle(&mut rng);
        for n in &nums {
            rbt.insert(*n);
        }
        let num = nums[rng.gen_range(0..max - min) as usize];
        let test_result = if let Some(found) = rbt.find_node(&num) {
            num == found.borrow().data
        } else {
            false
        };
        assert!(test_result);
    }
    #[test]
    fn find_successor_test() {
        let mut rbt = RBTree::<u64>::new();
        for n in 0..20 {
            rbt.insert(n);
        }

        let num = 11;
        let found_node = rbt.find_node(&num);
        let test_result = if let Some(found) = &found_node {
            num == found.borrow().data
        } else {
            false
        };
        assert!(test_result);

        let test_result = if let Some(found) = rbt.find_successor(found_node.unwrap()) {
            found.borrow().data == 12
        } else {
            false
        };
        assert!(test_result);
    }
}
