use crate::quadtree::aabb::AABB;
use double_linked_list::{DoubleLinkedList, DoubleLinkedListNodeRef};
use std::sync::{Arc, RwLock};

/// Maximum depth used by [`Quadtree::new`].
///
/// A quadtree created with [`Quadtree::new`] will subdivide the world until
/// this depth is reached.
const MAX_DEPTH: u8 = 16;

/// An element stored in a [`Quadtree`].
///
/// Each element consists of the user's value and the [`AABB`] describing its
/// spatial bounds.
///
/// # Examples
///
/// ```
/// use quadtree::{AABB, Element};
///
/// let bounds = AABB::from_center_size((10.0, 10.0), 4.0);
/// let element: Element<u32> = (42, bounds);
///
/// assert_eq!(element.0, 42);
/// assert_eq!(element.1, bounds);
/// ```
pub type Element<T> = (T, AABB);

/// Stores the location of an element inside a [`Quadtree`].
///
/// A location contains two references:
///
/// - `ll_node` points to the element's node in the doubly linked list.
/// - `qt_node` points to the quadtree node containing that list node.
///
/// Keeping both references allows operations such as [`Quadtree::remove`] and
/// [`Quadtree::relocate`] to operate directly on the element without first
/// searching the entire tree.
///
/// # Examples
///
/// ```
/// use quadtree::{AABB, Quadtree};
///
/// let mut tree = Quadtree::new(
///     AABB::from_center_size((0.0, 0.0), 100.0)
/// );
///
/// let location = tree.insert(
///     42,
///     AABB::from_center_size((10.0, 10.0), 2.0),
/// );
///
/// assert_eq!(
///     location.ll_node.read().unwrap().value().0,
///     42
/// );
/// ```
pub struct QuadtreeElementLocation<T> {
    /// Reference to the element's node in the doubly linked list.
    pub ll_node: DoubleLinkedListNodeRef<Element<T>>,

    /// Reference to the quadtree node containing the element.
    pub qt_node: QuadtreeNodeRef<T>,
}

/// A reference-counted, thread-safe reference to a [`QuadtreeNode`].
///
/// The `Arc` allows multiple owners of a node, while `RwLock` allows multiple
/// concurrent readers or one writer.
///
/// # Examples
///
/// ```
/// use quadtree::{AABB, QuadtreeNode};
///
/// let node = QuadtreeNode::<i32>::new_ref(
///     AABB::from_center_size((0.0, 0.0), 100.0),
///     0,
/// );
///
/// assert_eq!(node.read().unwrap().self_depth, 0);
/// ```
pub type QuadtreeNodeRef<T> = Arc<RwLock<QuadtreeNode<T>>>;

/// A node in a [`Quadtree`].
///
/// Each node represents a square region of space. Elements that cannot fit
/// completely inside one of its child quadrants remain directly in the node.
///
/// A node may have up to four children:
///
/// 1. Bottom-left
/// 2. Bottom-right
/// 3. Top-left
/// 4. Top-right
///
/// Elements are stored in a [`DoubleLinkedList`] so that an element can be
/// removed or relocated efficiently when its node is already known.
///
/// # Examples
///
/// ```
/// use quadtree::{AABB, QuadtreeNode};
///
/// let node = QuadtreeNode::<i32>::new(
///     AABB::from_center_size((0.0, 0.0), 100.0),
///     0,
/// );
///
/// assert_eq!(node.self_depth, 0);
/// assert!(node.children.is_none());
/// assert!(node.elements.is_empty());
/// ```
pub struct QuadtreeNode<T> {
    /// Depth of this node relative to the root.
    ///
    /// The root has depth `0`.
    pub self_depth: u8,

    /// Spatial bounds represented by this node.
    pub bounds: AABB,

    /// Elements that belong directly to this node.
    ///
    /// Elements are stored here when they do not completely fit inside one
    /// of the node's four child quadrants, or when the maximum depth has
    /// been reached.
    pub elements: DoubleLinkedList<Element<T>>,

    /// The four child quadrants of this node, if the node has been split.
    ///
    /// The array order is:
    ///
    /// ```text
    /// 0 = bottom-left
    /// 1 = bottom-right
    /// 2 = top-left
    /// 3 = top-right
    /// ```
    pub children: Option<[QuadtreeNodeRef<T>; 4]>,
}

impl<T> QuadtreeNode<T> {
    /// Creates a new quadtree node.
    ///
    /// The node starts without children and without elements.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The spatial region represented by the node.
    /// * `depth` - The node's depth within the quadtree.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, QuadtreeNode};
    ///
    /// let bounds = AABB::from_center_size((0.0, 0.0), 100.0);
    /// let node = QuadtreeNode::<i32>::new(bounds, 3);
    ///
    /// assert_eq!(node.self_depth, 3);
    /// assert_eq!(node.bounds, bounds);
    /// assert!(node.children.is_none());
    /// assert!(node.elements.is_empty());
    /// ```

    pub fn new(bounds: AABB, depth: u8) -> Self {
        Self {
            self_depth: depth,
            bounds,
            elements: DoubleLinkedList::new(),
            children: None,
        }
    }

    /// Creates a new reference-counted quadtree node.
    ///
    /// This is equivalent to [`QuadtreeNode::new`] followed by wrapping the
    /// node in `Arc<RwLock<_>>`.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The spatial region represented by the node.
    /// * `depth` - The node's depth within the quadtree.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, QuadtreeNode};
    ///
    /// let node = QuadtreeNode::<i32>::new_ref(
    ///     AABB::from_center_size((0.0, 0.0), 100.0),
    ///     0,
    /// );
    ///
    /// assert_eq!(node.read().unwrap().self_depth, 0);
    /// ```

    pub fn new_ref(bounds: AABB, depth: u8) -> QuadtreeNodeRef<T> {
        Arc::new(RwLock::new(Self::new(bounds, depth)))
    }
}

/// A spatial quadtree for storing elements with axis-aligned bounding boxes.
///
/// The quadtree recursively subdivides a square region into four quadrants.
/// An element is placed into the deepest node whose bounds completely contain
/// the element's bounds.
///
/// Elements that overlap multiple quadrants remain in their current node
/// instead of being duplicated across children.
///
/// # Examples
///
/// ```
/// use quadtree::{AABB, Quadtree};
///
/// let mut tree = Quadtree::new(
///     AABB::from_center_size((0.0, 0.0), 100.0)
/// );
///
/// tree.insert(
///     "player",
///     AABB::from_center_size((10.0, 10.0), 2.0),
/// );
///
/// let results = tree.search(
///     AABB::from_center_size((10.0, 10.0), 5.0)
/// );
///
/// assert_eq!(results.len(), 1);
/// ```
pub struct Quadtree<T> {
    /// The root node of the quadtree.
    pub root: QuadtreeNodeRef<T>,

    /// Maximum depth to which the tree may subdivide.
    pub max_depth: u8,
}

impl<T> Quadtree<T> {
    /// Creates a new quadtree using the default maximum depth.
    ///
    /// The default maximum depth is [`MAX_DEPTH`].
    ///
    /// # Arguments
    ///
    /// * `world_bounds` - The total spatial region covered by the quadtree.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, Quadtree};
    ///
    /// let bounds = AABB::from_center_size((0.0, 0.0), 100.0);
    /// let tree: Quadtree<i32> = Quadtree::new(bounds);
    ///
    /// assert_eq!(tree.max_depth, 16);
    /// assert_eq!(tree.root.read().unwrap().self_depth, 0);
    /// ```

    pub fn new(world_bounds: AABB) -> Self {
        Self {
            root: QuadtreeNode::new_ref(world_bounds, 0),
            max_depth: MAX_DEPTH,
        }
    }

    /// Creates a new quadtree with a custom maximum depth.
    ///
    /// A smaller maximum depth limits the amount of subdivision and can be
    /// useful when controlling memory usage or limiting tree traversal.
    ///
    /// # Arguments
    ///
    /// * `world_bounds` - The total spatial region covered by the quadtree.
    /// * `max_depth` - Maximum depth the tree may reach.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, Quadtree};
    ///
    /// let bounds = AABB::from_center_size((0.0, 0.0), 100.0);
    /// let tree: Quadtree<i32> = Quadtree::new_with_depth(bounds, 8);
    ///
    /// assert_eq!(tree.max_depth, 8);
    /// ```

    pub fn new_with_depth(world_bounds: AABB, max_depth: u8) -> Self {
        Self {
            root: QuadtreeNode::new_ref(world_bounds, 0),
            max_depth,
        }
    }

    /// Inserts an element into the quadtree.
    ///
    /// The element is placed into the deepest node whose bounds completely
    /// contain its [`AABB`]. If the element cannot completely fit inside any
    /// child quadrant, it remains in the current node.
    ///
    /// The returned [`QuadtreeElementLocation`] can later be used to remove
    /// or relocate the element without searching for it.
    ///
    /// # Arguments
    ///
    /// * `element` - The value to store.
    /// * `bounds` - The spatial bounds of the element.
    ///
    /// # Returns
    ///
    /// A location handle containing references to both the list node and
    /// quadtree node containing the element.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, Quadtree};
    ///
    /// let mut tree = Quadtree::new(
    ///     AABB::from_center_size((0.0, 0.0), 100.0)
    /// );
    ///
    /// let location = tree.insert(
    ///     "player",
    ///     AABB::from_center_size((10.0, 10.0), 2.0),
    /// );
    ///
    /// assert_eq!(
    ///     location.ll_node.read().unwrap().value().0,
    ///     "player"
    /// );
    /// ```

    pub fn insert(&self, element: T, bounds: AABB) -> QuadtreeElementLocation<T> {
        let qt_node = Self::find_node_for(self.root.clone(), &bounds, self.max_depth);
        let ll_node = qt_node
            .write()
            .unwrap()
            .elements
            .push_back((element, bounds));

        QuadtreeElementLocation { ll_node, qt_node }
    }

    /// Removes an element from the quadtree.
    ///
    /// The supplied location must correspond to an element currently stored
    /// in the quadtree.
    ///
    /// Removal is efficient because the location already contains both the
    /// quadtree node and linked-list node containing the element.
    ///
    /// # Arguments
    ///
    /// * `element` - The location handle returned by [`Quadtree::insert`].
    ///
    /// # Returns
    ///
    /// The removed doubly linked-list node.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, Quadtree};
    ///
    /// let mut tree = Quadtree::new(
    ///     AABB::from_center_size((0.0, 0.0), 100.0)
    /// );
    ///
    /// let location = tree.insert(
    ///     42,
    ///     AABB::from_center_size((10.0, 10.0), 2.0)
    /// );
    ///
    /// let node = tree.remove(location);
    ///
    /// assert_eq!(
    ///     node.read().unwrap().value().0,
    ///     42
    /// );
    ///
    /// assert!(tree.search(
    ///     AABB::from_center_size((10.0, 10.0), 5.0)
    /// ).is_empty());
    /// ```

    pub fn remove(
        &self,
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

    /// Updates the spatial bounds and location of an existing element.
    ///
    /// If the new bounds still fit inside the element's current quadtree node,
    /// the search starts from that node. Otherwise, the search restarts from
    /// the root.
    ///
    /// This optimization avoids traversing the entire tree when an element
    /// moves within its current spatial region.
    ///
    /// If the element belongs in a different node, its existing linked-list
    /// node is detached from the old node and inserted into the new node.
    ///
    /// # Arguments
    ///
    /// * `element` - A mutable location handle previously returned by
    ///   [`Quadtree::insert`].
    /// * `new_bounds` - The element's new spatial bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, Quadtree};
    ///
    /// let mut tree = Quadtree::new(
    ///     AABB::from_center_size((0.0, 0.0), 100.0)
    /// );
    ///
    /// let mut location = tree.insert(
    ///     42,
    ///     AABB::from_center_size((10.0, 10.0), 2.0)
    /// );
    ///
    /// tree.relocate(
    ///     &mut location,
    ///     AABB::from_center_size((30.0, 30.0), 2.0)
    /// );
    ///
    /// let results = tree.search(
    ///     AABB::from_center_size((30.0, 30.0), 5.0)
    /// );
    ///
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].read().unwrap().value().0, 42);
    /// ```

    pub fn relocate(&self, element: &mut QuadtreeElementLocation<T>, new_bounds: AABB) {
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

    /// Searches the quadtree for elements whose bounds overlap the supplied
    /// bounds.
    ///
    /// The search traverses only nodes whose spatial bounds overlap the query
    /// region.
    ///
    /// If the query completely contains a node's bounds, all elements in that
    /// node and its descendants are returned without individually testing
    /// each element's bounds.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The query region.
    ///
    /// # Returns
    ///
    /// A vector containing references to all element nodes whose bounds
    /// overlap the query region.
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{AABB, Quadtree};
    ///
    /// let mut tree = Quadtree::new(
    ///     AABB::from_center_size((0.0, 0.0), 100.0)
    /// );
    ///
    /// tree.insert(
    ///     "inside",
    ///     AABB::from_center_size((10.0, 10.0), 2.0)
    /// );
    ///
    /// tree.insert(
    ///     "outside",
    ///     AABB::from_center_size((40.0, 40.0), 2.0)
    /// );
    ///
    /// let results = tree.search(
    ///     AABB::from_center_size((10.0, 10.0), 5.0)
    /// );
    ///
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].read().unwrap().value().0, "inside");
    /// ```

    pub fn search(&self, bounds: AABB) -> Vec<DoubleLinkedListNodeRef<Element<T>>> {
        let mut result = Vec::new();
        Self::search_node(&self.root, &bounds, &mut result);
        result
    }

    /// Finds the deepest quadtree node that completely contains the supplied
    /// bounds.
    ///
    /// Starting at `node`, this method repeatedly checks the four child
    /// quadrants. If exactly one quadrant completely contains the bounds, the
    /// search continues into that child.
    ///
    /// If no quadrant contains the bounds, the current node is returned.
    /// The search also stops when `max_depth` is reached.
    ///
    /// Child nodes are created lazily only when subdivision is required.
    ///
    /// # Arguments
    ///
    /// * `node` - Node from which to begin the search.
    /// * `bounds` - Bounds that must fit completely within the resulting node.
    /// * `max_depth` - Maximum allowed depth.
    ///
    /// # Returns
    ///
    /// The deepest node that can contain `bounds` completely.
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

    /// Recursively searches a quadtree node and its descendants.
    ///
    /// Nodes that do not overlap the query bounds are skipped immediately.
    ///
    /// When the query completely contains a node, [`Self::collect_all`] is
    /// used to collect every element beneath that node without performing
    /// individual element overlap tests.
    ///
    /// Otherwise, elements stored directly in the node are tested
    /// individually and overlapping child nodes are searched recursively.
    ///
    /// This method is used internally by [`Quadtree::search`].
    ///
    /// # Arguments
    ///
    /// * `node` - Node currently being searched.
    /// * `bounds` - Query bounds.
    /// * `result` - Output vector to which matching elements are appended.
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

    /// Collects all elements stored in a node and all of its descendants.
    ///
    /// This method does not perform bounds checks. It is used when a search
    /// query completely contains a node, making individual overlap checks
    /// unnecessary.
    ///
    /// This method is used internally by [`Quadtree::search`] through
    /// [`Self::search_node`].
    ///
    /// # Arguments
    ///
    /// * `node` - Node whose elements should be collected.
    /// * `result` - Output vector to which element references are appended.
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
