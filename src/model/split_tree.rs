use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize)]
pub struct SurfaceLayout {
    pub surface_id: Uuid,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub enum SplitNode {
    Leaf {
        surface_id: Uuid,
    },
    Split {
        direction: Direction,
        ratio: f64,
        first: Box<SplitNode>,
        second: Box<SplitNode>,
    },
}

impl SplitNode {
    /// Calculate pixel layout for every leaf in the tree.
    pub fn layout(&self, x: u16, y: u16, width: u16, height: u16) -> Vec<SurfaceLayout> {
        match self {
            SplitNode::Leaf { surface_id } => vec![SurfaceLayout {
                surface_id: *surface_id,
                x,
                y,
                width,
                height,
            }],
            SplitNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let mut layouts = Vec::new();
                match direction {
                    Direction::Vertical => {
                        let first_w = (width as f64 * ratio) as u16;
                        let second_w = width - first_w;
                        layouts.extend(first.layout(x, y, first_w, height));
                        layouts.extend(second.layout(x + first_w, y, second_w, height));
                    }
                    Direction::Horizontal => {
                        let first_h = (height as f64 * ratio) as u16;
                        let second_h = height - first_h;
                        layouts.extend(first.layout(x, y, width, first_h));
                        layouts.extend(second.layout(x, y + first_h, width, second_h));
                    }
                }
                layouts
            }
        }
    }

    /// Navigate from a surface in a given direction. Returns the target surface ID.
    pub fn navigate(&self, from: Uuid, direction: Direction) -> Option<Uuid> {
        let ids = self.surface_ids();
        let idx = ids.iter().position(|id| *id == from)?;
        match direction {
            Direction::Vertical => {
                if idx + 1 < ids.len() {
                    Some(ids[idx + 1])
                } else {
                    None
                }
            }
            Direction::Horizontal => {
                if idx + 1 < ids.len() {
                    Some(ids[idx + 1])
                } else {
                    None
                }
            }
        }
    }

    /// Split the leaf identified by `target` into a Split with `target` as first and `new_id` as second.
    pub fn split_at(&mut self, target: Uuid, new_id: Uuid, direction: Direction) {
        match self {
            SplitNode::Leaf { surface_id } if *surface_id == target => {
                let old = SplitNode::Leaf { surface_id: target };
                let new = SplitNode::Leaf { surface_id: new_id };
                *self = SplitNode::Split {
                    direction,
                    ratio: 0.5,
                    first: Box::new(old),
                    second: Box::new(new),
                };
            }
            SplitNode::Split { first, second, .. } => {
                first.split_at(target, new_id, direction);
                second.split_at(target, new_id, direction);
            }
            _ => {}
        }
    }

    /// Remove a leaf from the tree. Returns Some(removed_id) on success.
    pub fn remove(&mut self, target: Uuid) -> Option<Uuid> {
        match self {
            SplitNode::Split { first, second, .. } => {
                if matches!(first.as_ref(), SplitNode::Leaf { surface_id } if *surface_id == target) {
                    let sibling = *second.clone();
                    *self = sibling;
                    return Some(target);
                }
                if matches!(second.as_ref(), SplitNode::Leaf { surface_id } if *surface_id == target) {
                    let sibling = *first.clone();
                    *self = sibling;
                    return Some(target);
                }
                first.remove(target).or_else(|| second.remove(target))
            }
            SplitNode::Leaf { .. } => None,
        }
    }

    /// Collect all surface IDs in tree order (left-to-right, top-to-bottom).
    pub fn surface_ids(&self) -> Vec<Uuid> {
        match self {
            SplitNode::Leaf { surface_id } => vec![*surface_id],
            SplitNode::Split { first, second, .. } => {
                let mut ids = first.surface_ids();
                ids.extend(second.surface_ids());
                ids
            }
        }
    }

    /// Find first leaf surface ID.
    pub fn first_surface(&self) -> Uuid {
        match self {
            SplitNode::Leaf { surface_id } => *surface_id,
            SplitNode::Split { first, .. } => first.first_surface(),
        }
    }
}
