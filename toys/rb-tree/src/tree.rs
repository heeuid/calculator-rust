// use debug_cell::RefCell;
use std::cell::RefCell;
use std::{
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

impl<T: PartialOrd + Debug> Debug for RBTreeNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parent = if let Some(parent) = &self.parent {
            format!("{:?}", parent.borrow().data)
        } else {
            String::from("none")
        };
        let lchild = if let Some(child) = &self.lchild {
            format!("{:?}", child.borrow().data)
        } else {
            String::from("none")
        };
        let rchild = if let Some(child) = &self.rchild {
            format!("{:?}", child.borrow().data)
        } else {
            String::from("none")
        };
        write!(
            f,
            "Node({:?};{:?};p({});l({});r({}))",
            self.data, self.color, parent, lchild, rchild
        )
    }
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

    pub fn find(&self, data: &T) -> bool {
        if let Some(root) = &self.root {
            if let Some(node) = Self::_find_node(root.clone(), data) {
                if let Some(Ordering::Equal) = node.borrow().data.partial_cmp(data) {
                    return true;
                }
            }
        }
        false
    }

    pub fn insert(&mut self, data: T) -> bool {
        let new_node = Rc::new(RefCell::new(RBTreeNode::new(data)));
        let mut p = None; // tracing parent node for new node
        let mut c = self.root.clone(); // tracing current node

        // find out parent node for new node
        while let Some(current) = c.clone() {
            p = Some(current.clone());
            let mut current_borrowed = current.borrow_mut();
            match new_node.borrow().data.partial_cmp(&current_borrowed.data) {
                Some(order) => match order {
                    Ordering::Less => c = current_borrowed.lchild.clone(),
                    Ordering::Greater => c = current_borrowed.rchild.clone(),
                    Ordering::Equal => {
                        return false;
                    }
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

    pub fn delete(&mut self, data: &T) -> bool {
        let found = self.find_node(data);
        if let Some(found_node) = found.clone() {
            // println!("delete({:?})", found_node.borrow().data);
            let found_node_color = found_node.borrow().color;

            let mut has_2_children = found_node.borrow().lchild.is_some();
            has_2_children = has_2_children && found_node.borrow().rchild.is_some();

            if has_2_children {
                let successor = self.find_successor(found_node.clone());
                let successor_color = Self::get_color(successor.clone());

                let (starting, parent) = if let Some(successor_node) = &successor {
                    successor_node.borrow_mut().color = found_node_color;
                    match RBTreeNode::<T>::which_child(successor_node.clone()) {
                        Child::Left => {
                            let starting = successor_node.borrow().rchild.clone();
                            let parent = successor_node.borrow().parent.clone();
                            // println!("2 children, successor left, st({:?}) p({:?})", starting, parent);
                            self.transplant(found_node.clone(), successor_node.clone(), true);
                            // println!("2 children, successor left, st({:?}) p({:?})", starting, parent);
                            (starting, parent)
                        }
                        Child::Right => {
                            let starting = successor_node.borrow().rchild.clone();
                            let parent = successor.clone();
                            // println!("2 children, successor left, st({:?}) p({:?})", starting, parent);
                            self.transplant(found_node.clone(), successor_node.clone(), false);
                            // println!("2 children, successor left, st({:?}) p({:?})", starting, parent);
                            (starting, parent)
                        }
                        Child::None => return false,
                    }
                } else {
                    return false;
                };

                if let Color::Black = successor_color {
                    self.fix_delete(starting, parent);
                }
            } else {
                // println!("1|0 child");
                let child = if found_node.borrow().lchild.is_some() {
                    found_node.borrow().lchild.clone()
                } else {
                    found_node.borrow().rchild.clone()
                };

                let pfound = found_node.borrow().parent.clone();
                match RBTreeNode::<T>::which_child(found_node.clone()) {
                    Child::Left => {
                        // println!("left p{:?} c{:?}", pfound, child);
                        (*pfound.clone().unwrap()).borrow_mut().lchild = child.clone();
                    }
                    Child::Right => {
                        // println!("right p{:?} c{:?}", pfound, child);
                        (*pfound.clone().unwrap()).borrow_mut().rchild = child.clone();
                    }
                    Child::None => {
                        self.root = child.clone();
                    }
                }
                if let Some(child_node) = &child {
                    (*child_node).borrow_mut().parent = pfound.clone();
                }

                // Red => No broken rules
                if let Color::Black = found_node_color {
                    let pfound = found_node.borrow().parent.clone();
                    self.fix_delete(child, pfound);
                }
            }

            self.cnt -= 1;

            true
        } else {
            false
        }
    }

    //   P           X
    //     X  =>   P
    //   C           C
    fn rotate_left(&mut self, node: Rc<RefCell<RBTreeNode<T>>>) {
        let parent = match node.borrow().parent.clone() {
            Some(p) => p.clone(),
            None => {
                self.root = Some(node.clone());
                return;
            }
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
            None => {
                self.root = Some(node.clone());
                return;
            }
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

    fn fix_delete(
        &mut self,
        starting: Option<Rc<RefCell<RBTreeNode<T>>>>,
        parent: Option<Rc<RefCell<RBTreeNode<T>>>>,
    ) {
        let pnode = if let Some(_pnode) = parent.clone() {
            _pnode
        } else {
            // 1) statring is a root node
            if let Some(snode) = starting.clone() {
                (*snode).borrow_mut().color = Color::Black;
            }
            self.root = starting.clone();
            return;
        };

        // 2) starting is not a root (parent exists)

        // 2-1) starting is a red node
        if let Some(snode) = starting.clone() {
            let color = snode.borrow().color;
            if let Color::Red = color {
                (*snode).borrow_mut().color = Color::Black;
                return;
            }
        }

        // 2-2) starting is a black node
        let sibling = Self::get_sibling(starting.clone(), parent.clone());
        if let Some(sibling_node) = &sibling {
            // 2-2-1). sibling is red
            let sibling_node_color = sibling_node.borrow().color;
            if let Color::Red = sibling_node_color {
                // 1. swap colors of sibling and parent (parent must be black)
                (*sibling_node).borrow_mut().color = Color::Black;
                (*pnode).borrow_mut().color = Color::Red;

                // 2. rotate left or right
                if let Child::Right = RBTreeNode::<T>::which_child(sibling_node.clone()) {
                    self.rotate_left(sibling_node.clone());
                } else {
                    self.rotate_right(sibling_node.clone());
                }

                self.fix_delete(starting.clone(), parent.clone());
                return;
            }

            // 2-2-2). sibling is black
            let close_child = Self::get_close_child_of_sibling(sibling_node.clone());
            let far_child = Self::get_far_child_of_sibling(sibling_node.clone());
            let close_color = Self::get_color(close_child.clone());
            let far_color = Self::get_color(far_child.clone());
            match (far_color, close_color) {
                // 1. children are black or Nil
                (Color::Black, Color::Black) => {
                    // 1-1. raise two blacks to parent
                    // raise extra black of starting node and sibling's black color to parent node, and
                    // recursively fix-up with new starting node which is parent of current starting node.
                    (*sibling_node).borrow_mut().color = Color::Red;
                    // 1-2. fix-up with new starting node
                    let ppnode = pnode.borrow().parent.clone();
                    self.fix_delete(parent.clone(), ppnode);
                }
                // 2. far child is red
                (Color::Red, _) => {
                    // deliver sibling's black to 2 children
                    // swap parent & sibling color
                    // raise 2 blacks (starting, close child) to parent
                    let far_child_node = far_child.unwrap();
                    (*sibling_node).borrow_mut().color = pnode.borrow().color;
                    (*pnode).borrow_mut().color = Color::Black;
                    (*far_child_node).borrow_mut().color = Color::Black;
                    if let Child::Right = RBTreeNode::<T>::which_child(far_child_node.clone()) {
                        self.rotate_left(sibling_node.clone());
                    } else {
                        self.rotate_right(sibling_node.clone());
                    }
                }
                // 3. close child is red (far child is black)
                (Color::Black, Color::Red) => {
                    let close_child_node = close_child.unwrap();
                    // swap colors of sibling and close child
                    (*sibling_node).borrow_mut().color = Color::Red;
                    (*close_child_node).borrow_mut().color = Color::Black;
                    // rotate
                    if let Child::Right = RBTreeNode::<T>::which_child(close_child_node.clone()) {
                        self.rotate_left(close_child_node.clone());
                    } else {
                        self.rotate_right(close_child_node.clone());
                    }
                    // fix-up again (it will be case 3.)
                    self.fix_delete(starting.clone(), parent.clone());
                }
            }
        }
    }

    fn find_node(&self, data: &T) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        if let Some(root) = &self.root {
            Self::_find_node(root.clone(), data)
        } else {
            None
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

    fn find_successor(
        &self,
        node: Rc<RefCell<RBTreeNode<T>>>,
    ) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        let right_child = node.borrow().rchild.clone();
        if let Some(rchild) = &right_child {
            let mut current_node = rchild.clone();
            while let Some(lchild) = current_node.clone().borrow().lchild.clone() {
                current_node = lchild;
            }
            Some(current_node)
        } else {
            None
        }
    }

    fn get_sibling(
        me: Option<Rc<RefCell<RBTreeNode<T>>>>,
        parent: Option<Rc<RefCell<RBTreeNode<T>>>>,
    ) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        match (me, parent) {
            (Some(me_node), Some(pnode)) => {
                if let Some(lchild) = pnode.borrow().lchild.clone() {
                    if Rc::ptr_eq(&me_node, &lchild) {
                        pnode.borrow().rchild.clone()
                    } else {
                        pnode.borrow().lchild.clone()
                    }
                } else {
                    pnode.borrow().rchild.clone()
                }
            }
            (None, Some(pnode)) => {
                if pnode.borrow().lchild.is_none() {
                    pnode.borrow().rchild.clone()
                } else {
                    pnode.borrow().lchild.clone()
                }
            }
            _ => None,
        }
    }

    fn get_far_child_of_sibling(
        sibling: Rc<RefCell<RBTreeNode<T>>>,
    ) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        match RBTreeNode::<T>::which_child(sibling.clone()) {
            Child::Left => sibling.borrow().lchild.clone(),
            Child::Right => sibling.borrow().rchild.clone(),
            Child::None => None,
        }
    }

    fn get_close_child_of_sibling(
        sibling: Rc<RefCell<RBTreeNode<T>>>,
    ) -> Option<Rc<RefCell<RBTreeNode<T>>>> {
        match RBTreeNode::<T>::which_child(sibling.clone()) {
            Child::Left => sibling.borrow().rchild.clone(),
            Child::Right => sibling.borrow().lchild.clone(),
            Child::None => None,
        }
    }

    fn get_color(node: Option<Rc<RefCell<RBTreeNode<T>>>>) -> Color {
        if let Some(_node) = node.clone() {
            _node.borrow().color
        } else {
            Color::Black
        }
    }

    fn transplant(
        &mut self,
        old_node: Rc<RefCell<RBTreeNode<T>>>,
        new_node: Rc<RefCell<RBTreeNode<T>>>,
        care_child_of_new: bool,
    ) {
        if care_child_of_new {
            let new_parent = new_node.borrow().parent.clone();
            let new_rchild = new_node.borrow().rchild.clone();
            // new_node.rchild.parent = new_node.parent
            if let Some(new_child) = &new_rchild {
                (*new_child).borrow_mut().parent = new_parent.clone();
            }
            // new_node.parent.lchild = new_node.rchild
            if let Some(new_pnode) = &new_parent {
                (*new_pnode).borrow_mut().lchild = new_rchild.clone();
            }
        }

        let old_parent = old_node.borrow().parent.clone();
        match RBTreeNode::<T>::which_child(old_node.clone()) {
            Child::Left => {
                // new_node.parent = old_node.parent
                // old_node.parent.lchild = new_node
                new_node.borrow_mut().parent = old_parent.clone();
                old_parent.unwrap().borrow_mut().lchild = Some(new_node.clone());
            }
            Child::Right => {
                // new_node.parent = old_node.parent
                // old_node.parent.rchild = new_node
                new_node.borrow_mut().parent = old_parent.clone();
                old_parent.unwrap().borrow_mut().rchild = Some(new_node.clone());
            }
            Child::None => {
                // self.root = new_node
                new_node.borrow_mut().parent = None;
                self.root = Some(new_node.clone());
            }
        }

        // new_node.lchild = old_node.lchild
        // old_node.lchild.parent = new_node
        let old_lchild = old_node.borrow().lchild.clone();
        new_node.borrow_mut().lchild = old_lchild.clone();
        if let Some(old_child) = &old_lchild {
            (*old_child).borrow_mut().parent = Some(new_node.clone());
        }

        // new_node.rchild = old_node.rchild
        // new_node.rchild.parent = new_node
        if care_child_of_new {
            let old_rchild = old_node.borrow().rchild.clone();
            new_node.borrow_mut().rchild = old_rchild.clone();
            if let Some(rchild) = &old_rchild {
                (*rchild).borrow_mut().parent = Some(new_node.clone());
            }
        }

        // (*old_node).borrow_mut().lchild = None;
        // (*old_node).borrow_mut().rchild = None;
        // (*old_node).borrow_mut().parent = None;
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

        let is_rbt = cnt <= min_depth && cnt * 2 >= max_depth;
        (is_rbt, cnt, min_depth, max_depth)
    }
}

#[cfg(test)]
mod test {
    use crate::RBTree;
    use rand::prelude::*;
    #[test]
    fn insertion_test() {
        let mut rbt = RBTree::<f64>::new();
        let mut rng = rand::thread_rng();
        let mut nums: Vec<f64> = vec![];
        for _ in 0..1000000 {
            nums.push(rng.gen());
        }
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
        for n in &nums {
            assert!(rbt.find(n));
        }
    }
    #[test]
    fn find_successor_test() {
        let mut rbt = RBTree::<u64>::new();
        for n in 0..100 {
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
    #[test]
    fn deletion_test() {
        let mut rbt = RBTree::<u64>::new();
        let mut rng = rand::thread_rng();
        let mut nums: Vec<u64> = vec![];
        for _ in 0..10000 {
            let n = rng.gen();
            if rbt.insert(n) {
                nums.push(n);
            }
        }
        for (i, n) in nums.iter().enumerate() {
            rbt.delete(n);
            println!("{}: delete {}", i, n);
            let (is_rbt, _, _, _) = rbt.check();
            assert!(is_rbt);
        }
        let (is_rbt, blacks, min_depth, max_depth) = rbt.check();
        assert!(is_rbt);
        assert!(blacks <= min_depth);
        assert!(blacks * 2 >= max_depth);
        assert!(rbt.cnt == 0);
        assert!(rbt.root.is_none());
    }
    #[test]
    fn insert_delete_test() {
        let mut rbt = RBTree::<u64>::new();
        let mut backup: Vec<u64> = vec![];
        let mut rng = rand::thread_rng();
        let mut insert_cnt = 0;
        let mut delete_cnt = 0;
        for _ in 0..10000 {
            let n = rng.gen();

            if rbt.insert(n) {
                backup.push(n);
                insert_cnt += 1;
            }

            if rng.gen_bool(0.5) {
                let number = backup.pop();
                if let Some(num) = &number {
                    if rbt.delete(num) {
                        delete_cnt += 1;
                    }
                }
            }

            let (is_rbt, blacks, min_depth, max_depth) = rbt.check();
            assert!(is_rbt);
            assert!(blacks <= min_depth);
            assert!(blacks * 2 >= max_depth);
        }
        assert!(rbt.cnt == insert_cnt - delete_cnt);
    }
    // #[test]
    // fn delete_custom_test() {
    //     let mut rbt = RBTree::<u64>::new();
    //     for n in 0..10 {
    //         rbt.insert(n);
    //     }
    //     println!("{:?}", rbt);
    //     println!("-----------------------------");
    //     let n = 0;
    //     rbt.delete(&n);
    //     println!("{:?}", rbt);
    //     println!("-----------------------------");
    //     let n = 1;
    //     rbt.delete(&n);
    //     println!("{:?}", rbt);
    //     println!("-----------------------------");
    //     let n = 2;
    //     rbt.delete(&n);
    //     println!("{:?}", rbt);
    //     let (is_rbt, _, _, _) = rbt.check();
    //     assert!(is_rbt);
    // }
}
