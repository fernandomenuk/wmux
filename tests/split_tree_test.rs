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
