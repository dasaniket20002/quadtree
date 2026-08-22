#[cfg(test)]
mod tests {
    use crate::quadtree::{
        AABB, {Element, Quadtree},
    };
    use double_linked_list::DoubleLinkedListNodeRef;
    use std::sync::Arc;

    fn world() -> AABB {
        AABB::from_center_size((0.0, 0.0), 100.0)
    }

    fn box_at(x: f32, y: f32, size: f32) -> AABB {
        AABB::from_center_size((x, y), size)
    }

    fn result_values<T: Clone>(results: &[DoubleLinkedListNodeRef<Element<T>>]) -> Vec<T> {
        results
            .iter()
            .map(|node| node.read().unwrap().value().0.clone())
            .collect()
    }

    #[test]
    fn new_creates_empty_tree() {
        let tree: Quadtree<i32> = Quadtree::new(world());

        assert_eq!(tree.max_depth, 16);
        assert_eq!(tree.root.read().unwrap().self_depth, 0);
        assert!(tree.root.read().unwrap().elements.is_empty());
        assert!(tree.root.read().unwrap().children.is_none());
    }

    #[test]
    fn new_with_depth_uses_custom_depth() {
        let tree: Quadtree<i32> = Quadtree::new_with_depth(world(), 4);

        assert_eq!(tree.max_depth, 4);
    }

    #[test]
    fn insert_adds_element() {
        let mut tree = Quadtree::new(world());

        let location = tree.insert(42, box_at(10.0, 10.0, 2.0));

        assert_eq!(location.ll_node.read().unwrap().value().0, 42);

        assert_eq!(
            location.ll_node.read().unwrap().value().1,
            box_at(10.0, 10.0, 2.0)
        );
    }

    #[test]
    fn inserted_element_can_be_found() {
        let mut tree = Quadtree::new(world());

        tree.insert(42, box_at(10.0, 10.0, 2.0));

        let results = tree.search(box_at(10.0, 10.0, 5.0));

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].read().unwrap().value().0, 42);
    }

    #[test]
    fn search_does_not_return_non_overlapping_elements() {
        let mut tree = Quadtree::new(world());

        tree.insert(1, box_at(-30.0, -30.0, 2.0));

        tree.insert(2, box_at(30.0, 30.0, 2.0));

        let results = tree.search(box_at(-30.0, -30.0, 5.0));

        assert_eq!(result_values(&results), vec![1]);
    }

    #[test]
    fn search_returns_multiple_overlapping_elements() {
        let mut tree = Quadtree::new(world());

        tree.insert(1, box_at(0.0, 0.0, 10.0));

        tree.insert(2, box_at(2.0, 0.0, 10.0));

        tree.insert(3, box_at(50.0, 50.0, 2.0));

        let results = tree.search(box_at(1.0, 0.0, 10.0));

        let mut values = result_values(&results);
        values.sort();

        assert_eq!(values, vec![1, 2]);
    }

    #[test]
    fn search_returns_element_that_touches_boundary() {
        let mut tree = Quadtree::new(world());

        tree.insert(42, box_at(10.0, 0.0, 10.0));

        let results = tree.search(box_at(0.0, 0.0, 10.0));

        assert_eq!(result_values(&results), vec![42]);
    }

    #[test]
    fn search_empty_tree_returns_empty_result() {
        let tree: Quadtree<i32> = Quadtree::new(world());

        let results = tree.search(box_at(0.0, 0.0, 10.0));

        assert!(results.is_empty());
    }

    #[test]
    fn search_outside_world_returns_empty_result() {
        let mut tree = Quadtree::new(world());

        tree.insert(42, box_at(0.0, 0.0, 2.0));

        let results = tree.search(box_at(200.0, 200.0, 10.0));

        assert!(results.is_empty());
    }

    #[test]
    fn element_outside_all_quadrants_stays_at_current_node() {
        let mut tree = Quadtree::new(world());

        let location = tree.insert(42, box_at(0.0, 0.0, 60.0));

        assert_eq!(location.qt_node.read().unwrap().self_depth, 0);

        assert_eq!(location.qt_node.read().unwrap().bounds, world());
    }

    #[test]
    fn small_element_is_inserted_into_descendant() {
        let mut tree = Quadtree::new_with_depth(world(), 4);

        let location = tree.insert(42, box_at(-40.0, -40.0, 2.0));

        assert!(location.qt_node.read().unwrap().self_depth > 0);

        assert!(
            location
                .qt_node
                .read()
                .unwrap()
                .bounds
                .contains(&box_at(-40.0, -40.0, 2.0))
        );
    }

    #[test]
    fn quadtree_creates_children_when_subdividing() {
        let mut tree = Quadtree::new_with_depth(world(), 4);

        tree.insert(42, box_at(-40.0, -40.0, 2.0));

        assert!(tree.root.read().unwrap().children.is_some());
    }

    #[test]
    fn max_depth_limits_subdivision() {
        let mut tree = Quadtree::new_with_depth(world(), 1);

        let location = tree.insert(42, box_at(-40.0, -40.0, 0.01));

        assert_eq!(location.qt_node.read().unwrap().self_depth, 1);

        assert!(location.qt_node.read().unwrap().children.is_none());
    }

    #[test]
    fn remove_removes_element_from_search_results() {
        let mut tree = Quadtree::new(world());

        let location = tree.insert(42, box_at(10.0, 10.0, 2.0));

        assert_eq!(tree.search(box_at(10.0, 10.0, 5.0)).len(), 1);

        tree.remove(location);

        assert!(tree.search(box_at(10.0, 10.0, 5.0)).is_empty());
    }

    #[test]
    fn remove_returns_removed_node() {
        let mut tree = Quadtree::new(world());

        let location = tree.insert(42, box_at(10.0, 10.0, 2.0));

        let node = tree.remove(location);

        assert_eq!(node.read().unwrap().value().0, 42);

        assert_eq!(node.read().unwrap().value().1, box_at(10.0, 10.0, 2.0));
    }

    #[test]
    fn relocate_updates_element_bounds() {
        let mut tree = Quadtree::new(world());

        let mut location = tree.insert(42, box_at(10.0, 10.0, 2.0));

        let new_bounds = box_at(30.0, 30.0, 4.0);

        tree.relocate(&mut location, new_bounds);

        assert_eq!(location.ll_node.read().unwrap().value().1, new_bounds);
    }

    #[test]
    fn relocate_moves_element_to_new_quadtree_node() {
        let mut tree = Quadtree::new_with_depth(world(), 4);

        let mut location = tree.insert(42, box_at(-40.0, -40.0, 2.0));

        let old_node = location.qt_node.clone();

        tree.relocate(&mut location, box_at(40.0, 40.0, 2.0));

        assert!(!Arc::ptr_eq(&old_node, &location.qt_node));

        assert!(
            !old_node
                .read()
                .unwrap()
                .elements
                .contains(&location.ll_node)
        );

        assert!(
            location
                .qt_node
                .read()
                .unwrap()
                .elements
                .contains(&location.ll_node)
        );
    }

    #[test]
    fn relocate_keeps_same_node_when_possible() {
        let mut tree = Quadtree::new(world());

        let mut location = tree.insert(42, box_at(10.0, 10.0, 2.0));

        tree.relocate(&mut location, box_at(11.0, 11.0, 2.0));

        let node = location.qt_node.read().unwrap();

        assert!(node.bounds.contains(&box_at(11.0, 11.0, 2.0)));
    }

    #[test]
    fn relocate_can_move_element_out_of_current_subtree() {
        let mut tree = Quadtree::new_with_depth(world(), 4);

        let mut location = tree.insert(42, box_at(-40.0, -40.0, 2.0));

        let original_node = location.qt_node.clone();

        tree.relocate(&mut location, box_at(40.0, 40.0, 2.0));

        assert!(!Arc::ptr_eq(&original_node, &location.qt_node));

        let results = tree.search(box_at(40.0, 40.0, 5.0));

        assert_eq!(result_values(&results), vec![42]);

        let old_results = tree.search(box_at(-40.0, -40.0, 5.0));

        assert!(old_results.is_empty());
    }

    #[test]
    fn relocate_keeps_same_linked_list_node() {
        let mut tree = Quadtree::new(world());

        let mut location = tree.insert(42, box_at(-30.0, -30.0, 2.0));

        let original_ll_node = location.ll_node.clone();

        tree.relocate(&mut location, box_at(30.0, 30.0, 2.0));

        assert!(Arc::ptr_eq(&original_ll_node, &location.ll_node));

        assert_eq!(location.ll_node.read().unwrap().value().0, 42);
    }

    #[test]
    fn search_finds_elements_stored_at_parent_nodes() {
        let mut tree = Quadtree::new(world());

        let location = tree.insert(42, box_at(0.0, 0.0, 60.0));

        assert_eq!(location.qt_node.read().unwrap().self_depth, 0);

        let results = tree.search(box_at(0.0, 0.0, 10.0));

        assert_eq!(result_values(&results), vec![42]);
    }

    #[test]
    fn search_finds_elements_in_children() {
        let mut tree = Quadtree::new_with_depth(world(), 4);

        tree.insert(1, box_at(-40.0, -40.0, 2.0));

        tree.insert(2, box_at(40.0, 40.0, 2.0));

        let results = tree.search(box_at(-40.0, -40.0, 5.0));

        assert_eq!(result_values(&results), vec![1]);
    }

    #[test]
    fn search_collects_all_when_query_contains_node() {
        let mut tree = Quadtree::new_with_depth(world(), 4);

        tree.insert(1, box_at(-40.0, -40.0, 2.0));

        tree.insert(2, box_at(40.0, 40.0, 2.0));

        let results = tree.search(world());

        let mut values = result_values(&results);
        values.sort();

        assert_eq!(values, vec![1, 2]);
    }

    #[test]
    fn multiple_insertions_and_removals_work() {
        let mut tree = Quadtree::new(world());

        let first = tree.insert(1, box_at(-10.0, -10.0, 2.0));

        let second = tree.insert(2, box_at(0.0, 0.0, 2.0));

        let third = tree.insert(3, box_at(10.0, 10.0, 2.0));

        let results = tree.search(world());

        let mut values = result_values(&results);
        values.sort();

        assert_eq!(values, vec![1, 2, 3]);

        tree.remove(second);

        let results = tree.search(world());

        let mut values = result_values(&results);
        values.sort();

        assert_eq!(values, vec![1, 3]);

        tree.remove(first);

        let results = tree.search(world());

        assert_eq!(result_values(&results), vec![3]);

        tree.remove(third);

        assert!(tree.search(world()).is_empty());
    }

    #[test]
    fn elements_on_quadrant_boundaries_are_handled_correctly() {
        let mut tree = Quadtree::new_with_depth(world(), 4);

        let location = tree.insert(42, box_at(0.0, 0.0, 2.0));

        assert!(
            location
                .qt_node
                .read()
                .unwrap()
                .bounds
                .contains(&box_at(0.0, 0.0, 2.0))
        );

        let results = tree.search(box_at(0.0, 0.0, 5.0));

        assert_eq!(result_values(&results), vec![42]);
    }
}
