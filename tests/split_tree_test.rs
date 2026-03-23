use wmux::model::split_tree::{SplitNode, Direction};
use uuid::Uuid;

#[test]
fn single_leaf_layout() {
    let id = Uuid::new_v4();
    let node = SplitNode::Leaf { surface_id: id };
    let layouts = node.layout(0, 0, 120, 40);
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].surface_id, id);
    assert_eq!(layouts[0].x, 0);
    assert_eq!(layouts[0].y, 0);
    assert_eq!(layouts[0].width, 120);
    assert_eq!(layouts[0].height, 40);
}

#[test]
fn vertical_split_layout() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    let layouts = node.layout(0, 0, 120, 40);
    assert_eq!(layouts.len(), 2);
    assert_eq!(layouts[0].width, 60);
    assert_eq!(layouts[0].height, 40);
    assert_eq!(layouts[1].x, 60);
    assert_eq!(layouts[1].width, 60);
}

#[test]
fn horizontal_split_layout() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Horizontal,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    let layouts = node.layout(0, 0, 120, 40);
    assert_eq!(layouts.len(), 2);
    assert_eq!(layouts[0].height, 20);
    assert_eq!(layouts[1].y, 20);
    assert_eq!(layouts[1].height, 20);
}

#[test]
fn focus_navigation_vertical() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    assert_eq!(node.navigate(id1, Direction::Vertical), Some(id2));
}

#[test]
fn split_at_leaf() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut node = SplitNode::Leaf { surface_id: id1 };
    node.split_at(id1, id2, Direction::Vertical);
    match &node {
        SplitNode::Split { first, second, .. } => {
            assert!(matches!(first.as_ref(), SplitNode::Leaf { surface_id } if *surface_id == id1));
            assert!(matches!(second.as_ref(), SplitNode::Leaf { surface_id } if *surface_id == id2));
        }
        _ => panic!("Expected Split node"),
    }
}

#[test]
fn remove_leaf() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    let result = node.remove(id1);
    assert!(result.is_some());
    assert!(matches!(node, SplitNode::Leaf { surface_id } if surface_id == id2));
}

#[test]
fn collect_surface_ids() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    let ids = node.surface_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// === Edge cases ===

#[test]
fn nested_split_layout_three_panes() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    // Vertical split, then horizontal split on the right side
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { surface_id: id2 }),
            second: Box::new(SplitNode::Leaf { surface_id: id3 }),
        }),
    };
    let layouts = node.layout(0, 0, 120, 40);
    assert_eq!(layouts.len(), 3);
    // Left pane: full height, half width
    assert_eq!(layouts[0].surface_id, id1);
    assert_eq!(layouts[0].x, 0);
    assert_eq!(layouts[0].width, 60);
    assert_eq!(layouts[0].height, 40);
    // Top-right: half width, half height
    assert_eq!(layouts[1].surface_id, id2);
    assert_eq!(layouts[1].x, 60);
    assert_eq!(layouts[1].y, 0);
    assert_eq!(layouts[1].width, 60);
    assert_eq!(layouts[1].height, 20);
    // Bottom-right: half width, half height
    assert_eq!(layouts[2].surface_id, id3);
    assert_eq!(layouts[2].x, 60);
    assert_eq!(layouts[2].y, 20);
    assert_eq!(layouts[2].width, 60);
    assert_eq!(layouts[2].height, 20);
}

#[test]
fn layout_with_non_half_ratio() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.3,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    let layouts = node.layout(0, 0, 100, 40);
    assert_eq!(layouts[0].width, 30);
    assert_eq!(layouts[1].x, 30);
    assert_eq!(layouts[1].width, 70);
}

#[test]
fn layout_with_offset() {
    let id = Uuid::new_v4();
    let node = SplitNode::Leaf { surface_id: id };
    let layouts = node.layout(10, 5, 80, 30);
    assert_eq!(layouts[0].x, 10);
    assert_eq!(layouts[0].y, 5);
    assert_eq!(layouts[0].width, 80);
    assert_eq!(layouts[0].height, 30);
}

#[test]
fn first_surface_returns_leftmost_leaf() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { surface_id: id1 }),
            second: Box::new(SplitNode::Leaf { surface_id: id2 }),
        }),
        second: Box::new(SplitNode::Leaf { surface_id: id3 }),
    };
    assert_eq!(node.first_surface(), id1);
}

#[test]
fn navigate_from_last_returns_none() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    assert_eq!(node.navigate(id2, Direction::Vertical), None);
}

#[test]
fn navigate_nonexistent_surface_returns_none() {
    let id1 = Uuid::new_v4();
    let node = SplitNode::Leaf { surface_id: id1 };
    let fake = Uuid::new_v4();
    assert_eq!(node.navigate(fake, Direction::Vertical), None);
}

#[test]
fn remove_from_deep_tree() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    // Tree: Split(Leaf(id1), Split(Leaf(id2), Leaf(id3)))
    let mut node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { surface_id: id2 }),
            second: Box::new(SplitNode::Leaf { surface_id: id3 }),
        }),
    };
    // Remove id2 — the right subtree should collapse to just id3
    let result = node.remove(id2);
    assert_eq!(result, Some(id2));
    let ids = node.surface_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id3));
}

#[test]
fn remove_nonexistent_returns_none() {
    let id1 = Uuid::new_v4();
    let mut node = SplitNode::Leaf { surface_id: id1 };
    let fake = Uuid::new_v4();
    assert_eq!(node.remove(fake), None);
}

#[test]
fn split_at_in_nested_tree() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let mut node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    // Split id2 horizontally, adding id3
    node.split_at(id2, id3, Direction::Horizontal);
    let ids = node.surface_ids();
    assert_eq!(ids.len(), 3);
    assert_eq!(ids, vec![id1, id2, id3]);
}

#[test]
fn split_at_nonexistent_is_noop() {
    let id1 = Uuid::new_v4();
    let fake = Uuid::new_v4();
    let new_id = Uuid::new_v4();
    let mut node = SplitNode::Leaf { surface_id: id1 };
    node.split_at(fake, new_id, Direction::Vertical);
    // Should still be a single leaf
    assert_eq!(node.surface_ids(), vec![id1]);
}

#[test]
fn surface_ids_order_matches_layout_order() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { surface_id: id2 }),
            second: Box::new(SplitNode::Leaf { surface_id: id3 }),
        }),
    };
    let ids = node.surface_ids();
    let layouts = node.layout(0, 0, 120, 40);
    let layout_ids: Vec<Uuid> = layouts.iter().map(|l| l.surface_id).collect();
    assert_eq!(ids, layout_ids);
}

// === Mouse hit-test ===

#[test]
fn surface_at_single_leaf() {
    let id = Uuid::new_v4();
    let node = SplitNode::Leaf { surface_id: id };
    assert_eq!(node.surface_at(10, 5, 0, 0, 120, 40), Some(id));
    assert_eq!(node.surface_at(200, 5, 0, 0, 120, 40), None); // outside
}

#[test]
fn surface_at_vertical_split() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    // Left half (0..59)
    assert_eq!(node.surface_at(10, 10, 0, 0, 120, 40), Some(id1));
    // Right half (60..119)
    assert_eq!(node.surface_at(80, 10, 0, 0, 120, 40), Some(id2));
}

#[test]
fn surface_at_nested_split() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { surface_id: id2 }),
            second: Box::new(SplitNode::Leaf { surface_id: id3 }),
        }),
    };
    assert_eq!(node.surface_at(10, 10, 0, 0, 120, 40), Some(id1));  // left
    assert_eq!(node.surface_at(80, 5, 0, 0, 120, 40), Some(id2));   // top-right
    assert_eq!(node.surface_at(80, 30, 0, 0, 120, 40), Some(id3));  // bottom-right
}

#[test]
fn border_hit_vertical() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    // Border at x=60 for 120-wide split
    let hit = node.border_hit(60, 10, 0, 0, 120, 40);
    assert!(hit.is_some());
    let (path, dir, _, _, _, _) = hit.unwrap();
    assert!(path.is_empty()); // root split
    assert_eq!(dir, Direction::Vertical);
}

#[test]
fn border_hit_horizontal() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Horizontal,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    // Border at y=20 for 40-high split
    let hit = node.border_hit(10, 20, 0, 0, 120, 40);
    assert!(hit.is_some());
    let (_, dir, _, _, _, _) = hit.unwrap();
    assert_eq!(dir, Direction::Horizontal);
}

#[test]
fn border_hit_miss() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    // Click well inside a pane, not on border
    assert!(node.border_hit(10, 10, 0, 0, 120, 40).is_none());
}

#[test]
fn set_ratio_at_root() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    node.set_ratio_at(&[], 0.7);
    let layouts = node.layout(0, 0, 100, 40);
    assert_eq!(layouts[0].width, 70);
    assert_eq!(layouts[1].width, 30);
}

#[test]
fn set_ratio_clamps() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut node = SplitNode::Split {
        direction: Direction::Vertical,
        ratio: 0.5,
        first: Box::new(SplitNode::Leaf { surface_id: id1 }),
        second: Box::new(SplitNode::Leaf { surface_id: id2 }),
    };
    node.set_ratio_at(&[], 0.0); // should clamp to 0.1
    let layouts = node.layout(0, 0, 100, 40);
    assert_eq!(layouts[0].width, 10);

    node.set_ratio_at(&[], 1.0); // should clamp to 0.9
    let layouts = node.layout(0, 0, 100, 40);
    assert_eq!(layouts[0].width, 90);
}
