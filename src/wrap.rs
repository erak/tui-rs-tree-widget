use tui::layout::Rect;

use crate::{TreeItem, Visible};

pub fn wrap<'a, T: TreeItem<'a> + Clone>(items: &[Visible<T>], area: Rect) -> Vec<Visible<T>> {
    items
        .iter()
        .map(|visible| Visible {
            identifier: visible.identifier.clone(),
            item: visible.item.clone().wrap(area),
        })
        .collect::<Vec<_>>()
}
