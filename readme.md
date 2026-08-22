# Quadtree

A generic, high-performance 2D Quadtree implementation in Rust for spatial indexing and spatial queries.

The crate is designed for use in simulation engines, games, collision detection, visibility queries, and other applications that need efficient spatial partitioning.

## Features

- Generic `Quadtree<T>` implementation
- 2D axis-aligned bounding boxes (`AABB`)
- Recursive spatial subdivision
- Configurable maximum tree depth
- AABB containment and overlap tests
- Spatial overlap queries
- Insert, remove, and relocate operations
- Persistent element locations
- Efficient relocation of dynamic objects
- Doubly linked list storage for elements
- Thread-safe node references using `Arc<RwLock<_>>`
- No runtime dependencies beyond the Rust standard library and the linked-list crate
- Unit tests and documentation tests

---

## Installation

Add the crate to your `Cargo.toml`.

### Git dependency

```toml
[dependencies]
quadtree = { git = "https://github.com/dasaniket20002/quadtree.git" }
````

Then import the types you need:

```rust
use quadtree::{AABB, Element, Quadtree};
```

---

# Quick Start

Create a world-space AABB and initialize the quadtree:

```rust
use quadtree::{AABB, Quadtree};

fn main() {
    let world = AABB::from_center_size(
        (0.0, 0.0),
        1000.0,
    );

    let mut tree = Quadtree::new(world);

    let location = tree.insert(
        "player",
        AABB::from_center_size(
            (100.0, 100.0),
            20.0,
        ),
    );

    let results = tree.search(
        AABB::from_center_size(
            (100.0, 100.0),
            50.0,
        ),
    );

    for node in results {
        let node = node.read().unwrap();

        let (element, bounds) = node.value();

        println!("element: {element:?}");
        println!("bounds: {bounds:?}");
    }

    tree.remove(location);
}
```

---

# AABB

`AABB` represents a square, axis-aligned bounding box in 2D space.

```rust
use quadtree::AABB;
```

An AABB contains:

```rust
pub struct AABB {
    pub min: (f32, f32),
    pub max: (f32, f32),
    pub size: f32,
    pub center: (f32, f32),
}
```

## Creating an AABB

### From center and size

```rust
let bounds = AABB::from_center_size(
    (0.0, 0.0),
    100.0,
);
```

This creates:

```text
min = (-50, -50)
max = ( 50,  50)
center = (0, 0)
size = 100
```

### From center and half-size

```rust
let bounds = AABB::from_center_half_size(
    (0.0, 0.0),
    50.0,
);
```

Both constructors above produce an AABB with a total side length of `100.0`.

---

## AABB Containment

Use `contains` to determine whether one AABB completely contains another:

```rust
let world = AABB::from_center_size(
    (0.0, 0.0),
    100.0,
);

let object = AABB::from_center_size(
    (10.0, 10.0),
    10.0,
);

assert!(world.contains(&object));
```

Boundary contact is considered containment.

---

## AABB Overlap

Use `overlaps` to check whether two AABBs overlap:

```rust
let first = AABB::from_center_size(
    (0.0, 0.0),
    10.0,
);

let second = AABB::from_center_size(
    (5.0, 0.0),
    10.0,
);

assert!(first.overlaps(&second));
```

AABBs that touch at an edge or corner are considered overlapping.

---

## Splitting an AABB

An AABB can be split into four equally sized quadrants:

```rust
let bounds = AABB::from_center_size(
    (0.0, 0.0),
    100.0,
);

let quadrants = bounds.split_into_quadrants();
```

The quadrants are returned in the following order:

```text
+-------------------+
|         |         |
|    2    |    3    |
|         |         |
+---------+---------+
|         |         |
|    0    |    1    |
|         |         |
+---------+---------+

0 = bottom-left
1 = bottom-right
2 = top-left
3 = top-right
```

---

# Quadtree

The main data structure is:

```rust
Quadtree<T>
```

Create a quadtree by providing the bounds of the world:

```rust
use quadtree::{AABB, Quadtree};

let world = AABB::from_center_size(
    (0.0, 0.0),
    1000.0,
);

let mut tree = Quadtree::new(world);
```

The default maximum depth is `16`.

---

## Custom Maximum Depth

Use `new_with_depth` if you want to specify the maximum tree depth:

```rust
let mut tree = Quadtree::new_with_depth(
    world,
    8,
);
```

A smaller maximum depth produces a shallower tree.

A larger maximum depth allows the tree to subdivide further.

The maximum depth also prevents unbounded subdivision.

---

# Elements

The quadtree is generic:

```rust
Quadtree<T>
```

Elements are stored internally as:

```rust
pub type Element<T> = (T, AABB);
```

For example:

```rust
use quadtree::{AABB, Quadtree};

let world = AABB::from_center_size(
    (0.0, 0.0),
    1000.0,
);

let mut tree = Quadtree::new(world);

tree.insert(
    "player",
    AABB::from_center_size(
        (100.0, 100.0),
        20.0,
    ),
);

tree.insert(
    "enemy",
    AABB::from_center_size(
        (200.0, 150.0),
        10.0,
    ),
);
```

You can store any Rust type.

For example:

```rust
struct Entity {
    id: u32,
    name: String,
}

let mut tree = Quadtree::new(world);

tree.insert(
    Entity {
        id: 1,
        name: "Player".to_string(),
    },
    AABB::from_center_size(
        (100.0, 100.0),
        20.0,
    ),
);
```

---

# Inserting Elements

Use `insert` to add an element to the quadtree:

```rust
let location = tree.insert(
    42,
    AABB::from_center_size(
        (100.0, 100.0),
        10.0,
    ),
);
```

The returned value is a:

```rust
QuadtreeElementLocation<T>
```

Keep this value if you need to relocate or remove the element later.

---

## How Elements Are Placed

When inserting an element, the quadtree attempts to place it in the deepest child node whose bounds completely contain the element.

For example:

```text
+-----------------------+
|           |           |
|           |           |
|     2     |     3     |
|           |           |
|-----------+-----------|
|           |           |
|     0     |     1     |
|           |           |
+-----------------------+
```

If an element completely fits inside quadrant `3`, the quadtree continues searching that quadrant.

Subdivision continues until:

1. The element no longer fits completely inside a child.
2. The maximum depth is reached.

If an element spans multiple quadrants, it remains in the current node rather than being duplicated into multiple children.

This means each element is stored in exactly one quadtree node.

---

# Searching

Use `search` to find elements whose AABBs overlap a query region:

```rust
let results = tree.search(
    AABB::from_center_size(
        (100.0, 100.0),
        50.0,
    ),
);
```

The returned value is:

```rust
Vec<DoubleLinkedListNodeRef<Element<T>>>
```

Access the stored element through the linked-list node:

```rust
for node in results {
    let node = node.read().unwrap();

    let (element, bounds) = node.value();

    println!("element: {element:?}");
    println!("bounds: {bounds:?}");
}
```

For example:

```rust
let results = tree.search(
    AABB::from_center_size(
        (100.0, 100.0),
        50.0,
    ),
);

for node in results {
    let node = node.read().unwrap();

    println!("{:?}", node.value().0);
}
```

---

## Search Behavior

The search operation checks:

1. Whether the query overlaps the current node.
2. Elements stored directly in the current node.
3. Relevant child nodes.

If the query completely contains a quadtree node, the implementation can collect all elements beneath that node without individually testing every element's AABB.

This allows large spatial queries to avoid unnecessary per-element overlap tests.

---

# Removing Elements

When an element is inserted, save its location:

```rust
let location = tree.insert(
    42,
    AABB::from_center_size(
        (100.0, 100.0),
        10.0,
    ),
);
```

Remove it later using:

```rust
let node = tree.remove(location);
```

The removed linked-list node is returned.

You can access the element:

```rust
let node = tree.remove(location);

let node = node.read().unwrap();

let (element, bounds) = node.value();

println!("element: {element:?}");
println!("bounds: {bounds:?}");
```

The `QuadtreeElementLocation` is consumed by `remove`.

---

# Relocating Elements

For dynamic objects, use `relocate` when an element's bounds change.

```rust
let mut location = tree.insert(
    42,
    AABB::from_center_size(
        (100.0, 100.0),
        10.0,
    ),
);

tree.relocate(
    &mut location,
    AABB::from_center_size(
        (500.0, 500.0),
        10.0,
    ),
);
```

The linked-list node containing the element is preserved.

The quadtree updates its node location according to the new bounds.

---

## Dynamic Object Example

A typical simulation can maintain an element location alongside the object:

```rust
use quadtree::{AABB, Quadtree};

struct Entity {
    id: u32,
}

let world = AABB::from_center_size(
    (0.0, 0.0),
    2000.0,
);

let mut tree = Quadtree::new(world);

let mut player = tree.insert(
    Entity { id: 1 },
    AABB::from_center_size(
        (100.0, 100.0),
        20.0,
    ),
);

// The player moves.
tree.relocate(
    &mut player,
    AABB::from_center_size(
        (300.0, 300.0),
        20.0,
    ),
);
```

This makes `QuadtreeElementLocation<T>` useful for objects that frequently move through the world.

---

# Typical Game / Simulation Usage

A common use case is broad-phase collision detection.

```rust
use quadtree::{AABB, Quadtree};

struct Entity {
    id: u32,
}

let world = AABB::from_center_size(
    (0.0, 0.0),
    2000.0,
);

let mut tree = Quadtree::new(world);

let mut player = tree.insert(
    Entity { id: 1 },
    AABB::from_center_size(
        (100.0, 100.0),
        20.0,
    ),
);

tree.insert(
    Entity { id: 2 },
    AABB::from_center_size(
        (120.0, 100.0),
        20.0,
    ),
);

// Find potential collision candidates.
let candidates = tree.search(
    AABB::from_center_size(
        (100.0, 100.0),
        50.0,
    ),
);

for node in candidates {
    let node = node.read().unwrap();

    let (entity, bounds) = node.value();

    println!(
        "Potential collision with entity {}",
        entity.id
    );
}

// Move the player.
tree.relocate(
    &mut player,
    AABB::from_center_size(
        (300.0, 300.0),
        20.0,
    ),
);
```

The quadtree should generally be treated as a **broad-phase spatial index**.

Use the results of `search` as potential candidates, then perform more precise collision or intersection tests as necessary.

---

# Architecture

The quadtree is composed of three main concepts:

```text
Quadtree
   |
   +-- QuadtreeNode
   |      |
   |      +-- AABB
   |      +-- Elements
   |      +-- Child Nodes
   |
   +-- QuadtreeElementLocation
          |
          +-- Linked-list node
          +-- Quadtree node
```

Each `QuadtreeNode` contains:

```rust
pub struct QuadtreeNode<T> {
    pub self_depth: u8,
    pub bounds: AABB,
    pub elements: DoubleLinkedList<Element<T>>,
    pub children: Option<[QuadtreeNodeRef<T>; 4]>,
}
```

A `QuadtreeElementLocation<T>` keeps track of:

```rust
pub struct QuadtreeElementLocation<T> {
    pub ll_node: DoubleLinkedListNodeRef<Element<T>>,
    pub qt_node: QuadtreeNodeRef<T>,
}
```

This allows an element to be moved between quadtree nodes while retaining the same underlying linked-list node.

---

# Spatial Layout

The quadtree recursively subdivides the world:

```text
Depth 0

+-----------------------+
|                       |
|                       |
|        ROOT           |
|                       |
|                       |
+-----------------------+


Depth 1

+-----------+-----------+
|           |           |
|     2     |     3     |
|           |           |
+-----------+-----------+
|           |           |
|     0     |     1     |
|           |           |
+-----------+-----------+


Depth 2

+-----+-----+-----+-----+
|     |     |     |     |
|  2  |  3  |  2  |  3  |
|     |     |     |     |
+-----+-----+-----+-----+
|  0  |  1  |  0  |  1  |
+-----+-----+-----+-----+
|     |     |     |     |
|  2  |  3  |  2  |  3  |
|     |     |     |     |
+-----+-----+-----+-----+
|  0  |  1  |  0  |  1  |
+-----+-----+-----+-----+
```

Only nodes that are required are created.

Child nodes are lazily initialized when an element needs to descend into a quadrant.

---

# Dynamic Updates

The intended workflow for dynamic objects is:

```text
Insert
  |
  v
QuadtreeElementLocation
  |
  +----> Update object position
  |
  +----> relocate()
  |
  v
Search
  |
  v
Potential spatial candidates
```

For example:

```rust
let mut object = tree.insert(
    entity,
    initial_bounds,
);

loop {
    update_entity();

    tree.relocate(
        &mut object,
        new_bounds,
    );

    let nearby = tree.search(query_bounds);

    // Process nearby objects...
}
```

---

# Performance Considerations

The quadtree is intended for spatial workloads where querying a subset of a large world is more common than scanning every object.

Performance depends on:

* Number of elements
* Spatial distribution
* Element sizes
* Query sizes
* Maximum tree depth
* How frequently objects are relocated

Large AABBs may remain higher in the tree because they cannot completely fit inside a child quadrant.

This is intentional: storing such an object in multiple child nodes would require duplication and make updates more expensive.

---

# Maximum Depth

The default maximum depth is:

```rust
16
```

You can override it:

```rust
let tree = Quadtree::new_with_depth(
    world,
    8,
);
```

The maximum depth limits how far the quadtree can subdivide.

A very small depth can result in too many elements being stored in the same node.

An unnecessarily large depth can create many small nodes without providing meaningful benefits.

The appropriate value depends on the size and distribution of the world and its objects.

---

# Thread Safety

Quadtree nodes use:

```rust
Arc<RwLock<_>>
```

for shared ownership and synchronized access.

This allows node references to be shared between different parts of an application while protecting access to the underlying data.

For example:

```rust
let node = location.qt_node.clone();

let node = node.read().unwrap();

println!("{:?}", node.bounds);
```

Write access uses:

```rust
let mut node = location.qt_node.write().unwrap();

node.bounds = new_bounds;
```

The locking strategy is part of the data structure's implementation and should be considered when designing highly concurrent workloads.

---

# Testing

Run the complete test suite:

```bash
cargo test
```

Run library tests:

```bash
cargo test --lib
```

Run documentation tests:

```bash
cargo test --doc
```

Check formatting:

```bash
cargo fmt --check
```

Run Clippy:

```bash
cargo clippy
```

A useful local verification command is:

```bash
cargo fmt --check \
    && cargo check \
    && cargo test \
    && cargo test --doc \
    && cargo clippy
```

---

# Documentation

Generate the Rust API documentation:

```bash
cargo doc --open
```

The generated documentation contains the API documentation for the public types and functions, including:

* `AABB`
* `Quadtree`
* `QuadtreeNode`
* `QuadtreeElementLocation`
* `Element`
* AABB operations
* Quadtree operations

---

# Example

A complete example combining insertion, searching, relocation, and removal:

```rust
use quadtree::{AABB, Quadtree};

#[derive(Debug)]
struct Entity {
    id: u32,
}

fn main() {
    let world = AABB::from_center_size(
        (0.0, 0.0),
        1000.0,
    );

    let mut tree = Quadtree::new(world);

    // Insert an entity.
    let mut entity = tree.insert(
        Entity { id: 1 },
        AABB::from_center_size(
            (100.0, 100.0),
            20.0,
        ),
    );

    // Search the surrounding area.
    let results = tree.search(
        AABB::from_center_size(
            (100.0, 100.0),
            100.0,
        ),
    );

    for node in results {
        let node = node.read().unwrap();

        let (entity, bounds) = node.value();

        println!(
            "Found entity {:?} at {:?}",
            entity,
            bounds
        );
    }

    // Move the entity.
    tree.relocate(
        &mut entity,
        AABB::from_center_size(
            (300.0, 300.0),
            20.0,
        ),
    );

    // Remove the entity.
    tree.remove(entity);
}
```
