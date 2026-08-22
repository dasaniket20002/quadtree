#[cfg(test)]
mod tests {
    use crate::quadtree::AABB;

    #[test]
    fn from_center_half_size_creates_correct_bounds() {
        let aabb = AABB::from_center_half_size((10.0, 20.0), 5.0);

        assert_eq!(aabb.min, (5.0, 15.0));
        assert_eq!(aabb.max, (15.0, 25.0));
        assert_eq!(aabb.center, (10.0, 20.0));
        assert_eq!(aabb.size, 10.0);
    }

    #[test]
    fn from_center_size_creates_correct_bounds() {
        let aabb = AABB::from_center_size((10.0, 20.0), 10.0);

        assert_eq!(aabb.min, (5.0, 15.0));
        assert_eq!(aabb.max, (15.0, 25.0));
        assert_eq!(aabb.center, (10.0, 20.0));
        assert_eq!(aabb.size, 10.0);
    }

    #[test]
    fn constructors_produce_equivalent_aabbs() {
        let from_half_size = AABB::from_center_half_size((10.0, 20.0), 5.0);

        let from_size = AABB::from_center_size((10.0, 20.0), 10.0);

        assert_eq!(from_half_size, from_size);
    }

    #[test]
    fn contains_smaller_aabb() {
        let outer = AABB::from_center_size((0.0, 0.0), 10.0);
        let inner = AABB::from_center_size((0.0, 0.0), 4.0);

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn contains_offset_aabb() {
        let outer = AABB::from_center_size((0.0, 0.0), 10.0);
        let inner = AABB::from_center_size((2.0, 2.0), 2.0);

        assert!(outer.contains(&inner));
    }

    #[test]
    fn contains_boundary_aabb() {
        let outer = AABB::from_center_size((0.0, 0.0), 10.0);
        let touching = AABB::from_center_size((2.5, 0.0), 5.0);

        assert!(outer.contains(&touching));
    }

    #[test]
    fn does_not_contain_partially_outside_aabb() {
        let outer = AABB::from_center_size((0.0, 0.0), 10.0);
        let outside = AABB::from_center_size((5.0, 0.0), 10.0);

        assert!(!outer.contains(&outside));
    }

    #[test]
    fn aabb_contains_itself() {
        let aabb = AABB::from_center_size((10.0, 20.0), 10.0);

        assert!(aabb.contains(&aabb));
    }

    #[test]
    fn overlaps_intersecting_aabbs() {
        let first = AABB::from_center_size((0.0, 0.0), 10.0);
        let second = AABB::from_center_size((5.0, 0.0), 10.0);

        assert!(first.overlaps(&second));
        assert!(second.overlaps(&first));
    }

    #[test]
    fn overlaps_when_one_contains_the_other() {
        let outer = AABB::from_center_size((0.0, 0.0), 10.0);
        let inner = AABB::from_center_size((0.0, 0.0), 2.0);

        assert!(outer.overlaps(&inner));
        assert!(inner.overlaps(&outer));
    }

    #[test]
    fn touching_edges_are_overlapping() {
        let first = AABB::from_center_size((0.0, 0.0), 10.0);
        let second = AABB::from_center_size((10.0, 0.0), 10.0);

        assert!(first.overlaps(&second));
    }

    #[test]
    fn touching_corners_are_overlapping() {
        let first = AABB::from_center_size((0.0, 0.0), 10.0);
        let second = AABB::from_center_size((10.0, 10.0), 10.0);

        assert!(first.overlaps(&second));
    }

    #[test]
    fn separated_aabbs_do_not_overlap() {
        let first = AABB::from_center_size((0.0, 0.0), 10.0);
        let second = AABB::from_center_size((20.0, 0.0), 10.0);

        assert!(!first.overlaps(&second));
    }

    #[test]
    fn separated_on_y_axis_aabbs_do_not_overlap() {
        let first = AABB::from_center_size((0.0, 0.0), 10.0);
        let second = AABB::from_center_size((0.0, 20.0), 10.0);

        assert!(!first.overlaps(&second));
    }

    #[test]
    fn split_produces_four_quadrants() {
        let aabb = AABB::from_center_size((0.0, 0.0), 100.0);

        let quadrants = aabb.split_into_quadrants();

        assert_eq!(quadrants[0].center, (-25.0, -25.0));
        assert_eq!(quadrants[1].center, (25.0, -25.0));
        assert_eq!(quadrants[2].center, (-25.0, 25.0));
        assert_eq!(quadrants[3].center, (25.0, 25.0));
    }

    #[test]
    fn split_quadrants_have_half_the_size() {
        let aabb = AABB::from_center_size((0.0, 0.0), 100.0);

        for quadrant in aabb.split_into_quadrants() {
            assert_eq!(quadrant.size, 50.0);
        }
    }

    #[test]
    fn split_quadrants_cover_original_bounds() {
        let aabb = AABB::from_center_size((0.0, 0.0), 100.0);
        let quadrants = aabb.split_into_quadrants();

        for quadrant in &quadrants {
            assert!(aabb.contains(quadrant));
        }
    }

    #[test]
    fn split_quadrants_have_expected_bounds() {
        let aabb = AABB::from_center_size((0.0, 0.0), 100.0);
        let quadrants = aabb.split_into_quadrants();

        assert_eq!(quadrants[0].min, (-50.0, -50.0));
        assert_eq!(quadrants[0].max, (0.0, 0.0));

        assert_eq!(quadrants[1].min, (0.0, -50.0));
        assert_eq!(quadrants[1].max, (50.0, 0.0));

        assert_eq!(quadrants[2].min, (-50.0, 0.0));
        assert_eq!(quadrants[2].max, (0.0, 50.0));

        assert_eq!(quadrants[3].min, (0.0, 0.0));
        assert_eq!(quadrants[3].max, (50.0, 50.0));
    }

    #[test]
    fn split_quadrants_are_mutually_distinct() {
        let aabb = AABB::from_center_size((0.0, 0.0), 100.0);
        let quadrants = aabb.split_into_quadrants();

        assert_ne!(quadrants[0], quadrants[1]);
        assert_ne!(quadrants[0], quadrants[2]);
        assert_ne!(quadrants[0], quadrants[3]);
        assert_ne!(quadrants[1], quadrants[2]);
        assert_ne!(quadrants[1], quadrants[3]);
        assert_ne!(quadrants[2], quadrants[3]);
    }
}
