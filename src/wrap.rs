use tui::layout::Rect;

use crate::{TreeItem, Visible};

pub fn wrap<'a, T: TreeItem<'a> + Clone>(items: &[Visible<'a, T>], area: Rect) -> Vec<Visible<'a, T>> {
    items
        .iter()
        .map(|visible| Visible {
            identifier: visible.identifier.clone(),
            item: visible.item,
            graphemes: vec![],
        })
        .collect::<Vec<_>>()
}
