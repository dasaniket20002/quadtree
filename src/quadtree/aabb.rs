/// An axis-aligned bounding box in two-dimensional space.
///
/// The bounding box is represented by its minimum and maximum coordinates,
/// its side length, and its center point.
///
/// `AABB` is intended for use with spatial data structures such as
/// [`Quadtree`](crate::Quadtree).
///
/// # Coordinate System
///
/// The box is defined by:
///
/// - `min`: bottom-left corner
/// - `max`: top-right corner
/// - `center`: center point
/// - `size`: length of each side
///
/// For example, an AABB centered at `(0, 0)` with a size of `10` has:
///
/// ```text
/// min = (-5, -5)
/// max = ( 5,  5)
/// ```
///
/// # Examples
///
/// ```
/// use quadtree::AABB;
///
/// let bounds = AABB::from_center_size((0.0, 0.0), 10.0);
///
/// assert_eq!(bounds.min, (-5.0, -5.0));
/// assert_eq!(bounds.max, (5.0, 5.0));
/// assert_eq!(bounds.center, (0.0, 0.0));
/// assert_eq!(bounds.size, 10.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    /// Minimum `(x, y)` coordinate of the bounding box.
    ///
    /// This represents the bottom-left corner.
    pub min: (f32, f32),

    /// Maximum `(x, y)` coordinate of the bounding box.
    ///
    /// This represents the top-right corner.
    pub max: (f32, f32),

    /// Length of each side of the bounding box.
    pub size: f32,

    /// Center point of the bounding box.
    pub center: (f32, f32),
}

impl AABB {
    /// Creates an AABB from a center point and half-size.
    ///
    /// The `half_size` represents the distance from the center to each edge.
    ///
    /// Therefore, the resulting AABB has a total side length of
    /// `half_size * 2.0`.
    ///
    /// # Arguments
    ///
    /// * `center` - Center `(x, y)` coordinate of the AABB.
    /// * `half_size` - Distance from the center to any edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::AABB;
    ///
    /// let bounds = AABB::from_center_half_size((10.0, 20.0), 5.0);
    ///
    /// assert_eq!(bounds.min, (5.0, 15.0));
    /// assert_eq!(bounds.max, (15.0, 25.0));
    /// assert_eq!(bounds.center, (10.0, 20.0));
    /// assert_eq!(bounds.size, 10.0);
    /// ```

    pub fn from_center_half_size(center: (f32, f32), half_size: f32) -> Self {
        Self {
            min: (center.0 - half_size, center.1 - half_size),
            max: (center.0 + half_size, center.1 + half_size),
            size: half_size * 2.0,
            center,
        }
    }

    /// Creates an AABB from a center point and full side length.
    ///
    /// The supplied `size` represents the total width and height of the
    /// resulting square bounding box.
    ///
    /// # Arguments
    ///
    /// * `center` - Center `(x, y)` coordinate of the AABB.
    /// * `size` - Total side length of the AABB.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::AABB;
    ///
    /// let bounds = AABB::from_center_size((10.0, 20.0), 10.0);
    ///
    /// assert_eq!(bounds.min, (5.0, 15.0));
    /// assert_eq!(bounds.max, (15.0, 25.0));
    /// assert_eq!(bounds.center, (10.0, 20.0));
    /// assert_eq!(bounds.size, 10.0);
    /// ```

    pub fn from_center_size(center: (f32, f32), size: f32) -> Self {
        let half_size = size * 0.5;

        Self {
            min: (center.0 - half_size, center.1 - half_size),
            max: (center.0 + half_size, center.1 + half_size),
            size,
            center,
        }
    }

    /// Determines whether this AABB completely contains another AABB.
    ///
    /// An AABB is considered contained when all four edges of `other` are
    /// inside or exactly on the edges of `self`.
    ///
    /// Touching the boundary is considered containment.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::AABB;
    ///
    /// let outer = AABB::from_center_size((0.0, 0.0), 10.0);
    /// let inner = AABB::from_center_size((0.0, 0.0), 4.0);
    ///
    /// assert!(outer.contains(&inner));
    /// assert!(!inner.contains(&outer));
    /// ```
    ///
    /// Boundary-touching boxes are also considered contained:
    ///
    /// ```
    /// use quadtree::AABB;
    ///
    /// let outer = AABB::from_center_size((0.0, 0.0), 10.0);
    /// let touching = AABB::from_center_size((2.5, 0.0), 5.0);
    ///
    /// assert!(outer.contains(&touching));
    /// ```

    pub fn contains(&self, other: &AABB) -> bool {
        other.min.0 >= self.min.0
            && other.min.1 >= self.min.1
            && other.max.0 <= self.max.0
            && other.max.1 <= self.max.1
    }

    /// Determines whether this AABB overlaps another AABB.
    ///
    /// Two AABBs overlap when their projections on both the x and y axes
    /// intersect.
    ///
    /// Touching edges or corners are considered overlapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::AABB;
    ///
    /// let first = AABB::from_center_size((0.0, 0.0), 10.0);
    /// let second = AABB::from_center_size((5.0, 0.0), 10.0);
    ///
    /// assert!(first.overlaps(&second));
    /// ```
    ///
    /// Non-overlapping boxes return `false`:
    ///
    /// ```
    /// use quadtree::AABB;
    ///
    /// let first = AABB::from_center_size((0.0, 0.0), 10.0);
    /// let second = AABB::from_center_size((20.0, 0.0), 10.0);
    ///
    /// assert!(!first.overlaps(&second));
    /// ```

    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.0 <= other.max.0
            && self.max.0 >= other.min.0
            && self.min.1 <= other.max.1
            && self.max.1 >= other.min.1
    }

    /// Splits the AABB into four equally sized quadrants.
    ///
    /// Each child has half the side length of the original AABB.
    ///
    /// The quadrants are returned in the following order:
    ///
    /// ```text
    /// +-------------------+
    /// |         |         |
    /// |    2    |    3    |
    /// |         |         |
    /// +---------+---------+
    /// |         |         |
    /// |    0    |    1    |
    /// |         |         |
    /// +-------------------+
    /// ```
    ///
    /// Therefore:
    ///
    /// - `0` = bottom-left
    /// - `1` = bottom-right
    /// - `2` = top-left
    /// - `3` = top-right
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::AABB;
    ///
    /// let bounds = AABB::from_center_size((0.0, 0.0), 100.0);
    /// let quadrants = bounds.split_into_quadrants();
    ///
    /// assert_eq!(quadrants[0].center, (-25.0, -25.0));
    /// assert_eq!(quadrants[1].center, (25.0, -25.0));
    /// assert_eq!(quadrants[2].center, (-25.0, 25.0));
    /// assert_eq!(quadrants[3].center, (25.0, 25.0));
    ///
    /// assert_eq!(quadrants[0].size, 50.0);
    /// ```

    pub fn split_into_quadrants(&self) -> [AABB; 4] {
        let half_size = self.size * 0.5;
        let quarter_size = self.size * 0.25;

        [
            AABB::from_center_size(
                (self.center.0 - quarter_size, self.center.1 - quarter_size),
                half_size,
            ),
            AABB::from_center_size(
                (self.center.0 + quarter_size, self.center.1 - quarter_size),
                half_size,
            ),
            AABB::from_center_size(
                (self.center.0 - quarter_size, self.center.1 + quarter_size),
                half_size,
            ),
            AABB::from_center_size(
                (self.center.0 + quarter_size, self.center.1 + quarter_size),
                half_size,
            ),
        ]
    }
}
