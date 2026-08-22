use std::sync::{Arc, RwLock};

use super::AABB;
use double_linked_list::{DoubleLinkedList, DoubleLinkedListNodeRef};

const MAX_DEPTH: u8 = 16;

pub type Element<T> = (T, AABB);

pub struct QuadtreeElementLocation<T> {
    pub ll_node: DoubleLinkedListNodeRef<Element<T>>,
    pub qt_node: QuadtreeNodeRef<T>,
}

pub type QuadtreeNodeRef<T> = Arc<RwLock<QuadtreeNode<T>>>;

pub struct QuadtreeNode<T> {
    pub self_depth: u8,
    pub bounds: AABB,
    pub elements: DoubleLinkedList<Element<T>>,
    pub children: Option<[QuadtreeNodeRef<T>; 4]>,
}

impl<T> QuadtreeNode<T> {
    pub fn new(bounds: AABB, depth: u8) -> Self {
        Self {
            self_depth: depth,
            bounds,
            elements: DoubleLinkedList::new(),
            children: None,
        }
    }

    pub fn new_ref(bounds: AABB, depth: u8) -> QuadtreeNodeRef<T> {
        Arc::new(RwLock::new(Self::new(bounds, depth)))
    }
}

pub struct Quadtree<T> {
    pub root: QuadtreeNodeRef<T>,
    pub max_depth: u8,
}

impl<T> Quadtree<T> {
    pub fn new(world_bounds: AABB) -> Self {
        Self {
            root: QuadtreeNode::new_ref(world_bounds, 0),
            max_depth: MAX_DEPTH,
        }
    }

    pub fn new_with_depth(world_bounds: AABB, max_depth: u8) -> Self {
        Self {
            root: QuadtreeNode::new_ref(world_bounds, 0),
            max_depth,
        }
    }

    pub fn insert(&mut self, element: T, bounds: AABB) -> QuadtreeElementLocation<T> {
        let qt_node = Self::find_node_for(self.root.clone(), &bounds, self.max_depth);
        let ll_node = qt_node
            .write()
            .unwrap()
            .elements
            .push_back((element, bounds));
        QuadtreeElementLocation { ll_node, qt_node }
    }

    pub fn remove(
        &mut self,
        element: QuadtreeElementLocation<T>,
    ) -> DoubleLinkedListNodeRef<Element<T>> {
        element
            .qt_node
            .write()
            .unwrap()
            .elements
            .remove_node(element.ll_node.clone());
        element.ll_node
    }

    pub fn relocate(&mut self, element: &mut QuadtreeElementLocation<T>, new_bounds: AABB) {
        element.ll_node.write().unwrap().value_mut().1 = new_bounds;

        // If the new bounds still fit inside the current node's region, the
        // correct node (if different at all) must be a descendant of the
        // current node, so we can resume the search from there instead of
        // the root. Otherwise we have to restart from the root.
        let still_fits_current = element.qt_node.read().unwrap().bounds.contains(&new_bounds);

        let start = if still_fits_current {
            element.qt_node.clone()
        } else {
            self.root.clone()
        };

        let best_node = Self::find_node_for(start, &new_bounds, self.max_depth);

        if Arc::ptr_eq(&best_node, &element.qt_node) {
            return;
        }

        element
            .qt_node
            .write()
            .unwrap()
            .elements
            .remove_node(element.ll_node.clone());

        best_node
            .write()
            .unwrap()
            .elements
            .insert_node(element.ll_node.clone());

        element.qt_node = best_node;
    }

    pub fn search(&self, bounds: AABB) -> Vec<DoubleLinkedListNodeRef<Element<T>>> {
        let mut result = Vec::new();
        Self::search_node(&self.root, &bounds, &mut result);
        result
    }

    fn find_node_for(
        mut node: QuadtreeNodeRef<T>,
        bounds: &AABB,
        max_depth: u8,
    ) -> QuadtreeNodeRef<T> {
        loop {
            let (depth, quadrants) = {
                let node_ref = node.read().unwrap();
                (node_ref.self_depth, node_ref.bounds.split_into_quadrants())
            };

            if depth >= max_depth {
                return node;
            }

            let Some(index) = quadrants.iter().position(|q| q.contains(bounds)) else {
                return node;
            };

            let child = {
                let mut node_mut = node.write().unwrap();
                let children = node_mut
                    .children
                    .get_or_insert_with(|| quadrants.map(|q| QuadtreeNode::new_ref(q, depth + 1)));
                children[index].clone()
            };

            node = child;
        }
    }

    fn search_node(
        node: &QuadtreeNodeRef<T>,
        bounds: &AABB,
        result: &mut Vec<DoubleLinkedListNodeRef<Element<T>>>,
    ) {
        let (node_bounds, fully_contains) = {
            let node_ref = node.read().unwrap();
            (node_ref.bounds, bounds.contains(&node_ref.bounds))
        };

        if !bounds.overlaps(&node_bounds) {
            return;
        }

        if fully_contains {
            Self::collect_all(node, result);
            return;
        }

        let node_ref = node.read().unwrap();

        for elem in node_ref.elements.iter_nodes() {
            let elem_bounds = elem.read().unwrap().value().1;
            if bounds.overlaps(&elem_bounds) {
                result.push(elem.clone());
            }
        }

        if let Some(children) = &node_ref.children {
            for child in children {
                Self::search_node(child, bounds, result);
            }
        }
    }

    fn collect_all(
        node: &QuadtreeNodeRef<T>,
        result: &mut Vec<DoubleLinkedListNodeRef<Element<T>>>,
    ) {
        let node_ref = node.read().unwrap();

        for elem in node_ref.elements.iter_nodes() {
            result.push(elem);
        }

        if let Some(children) = &node_ref.children {
            for child in children {
                Self::collect_all(child, result);
            }
        }
    }
}
