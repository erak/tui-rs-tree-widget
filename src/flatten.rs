use std::cell::RefCell;

use tui::style::Style;
use tui::text::Spans;

use crate::identifier::{TreeIdentifier, TreeIdentifierVec};
use crate::{SharedTreeItem, TreeItem};

#[derive(Clone)]
pub struct Flattened<'a> {
    identifier: Vec<usize>,
    item: SharedTreeItem<'a>,
}

impl<'a> Flattened<'a> {
    pub fn identifier(&self) -> &Vec<usize> {
        &self.identifier
    }

    pub fn item(&self) -> SharedTreeItem<'a> {
        self.item.clone()
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }
}

/// Get a flat list of all visible [`TreeItem`s](TreeItem)
#[must_use]
pub fn flatten<'a>(opened: &[TreeIdentifierVec], items: &[SharedTreeItem<'a>]) -> Vec<Flattened<'a>> {
    internal(opened, items, &[])
}

#[must_use]
fn internal<'a>(
    opened: &[TreeIdentifierVec],
    items: &[SharedTreeItem<'a>],
    current: TreeIdentifier,
) -> Vec<Flattened<'a>> {
    let mut result = Vec::new();

    for (index, item) in items.into_iter().enumerate() {
        let mut child_identifier = current.to_vec();
        child_identifier.push(index);

        result.push(Flattened {
            item: item.clone(),
            identifier: child_identifier.clone(),
        });

        if opened.contains(&child_identifier) {
            let mut child_result = internal(opened, &item.borrow().children, &child_identifier);
            result.append(&mut child_result);
        }
    }

    result
}

#[cfg(test)]
fn get_naive_string_from_text(text: &tui::text::Text<'_>) -> String {
    text.lines
        .first()
        .unwrap()
        .0
        .first()
        .unwrap()
        .content
        .to_string()
}

#[cfg(test)]
fn get_example_tree_items() -> Vec<TreeItem<'static>> {
    // use tui::text::Spans;

    let a = vec![Spans::from("a")];
    let b = vec![Spans::from("b")];
    let c = vec![Spans::from("c")];
    let d = vec![Spans::from("d")];
    let e = vec![Spans::from("e")];
    let f = vec![Spans::from("f")];
    let g = vec![Spans::from("g")];
    let h = vec![Spans::from("h")];

    vec![
        TreeItem::new_leaf(a),
        TreeItem::new(
            b,
            vec![
                TreeItem::new_leaf(c),
                TreeItem::new(d, vec![TreeItem::new_leaf(e), TreeItem::new_leaf(f)]),
                TreeItem::new_leaf(g),
            ],
        ),
        TreeItem::new_leaf(h),
    ]
}

#[test]
fn get_opened_nothing_opened_is_top_level() {
    let items = get_example_tree_items();
    let result = flatten(&[], &items);
    let result_text = result
        .into_iter()
        .map(|o| get_naive_string_from_text(&o.item.lines().into()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_wrong_opened_is_only_top_level() {
    let items = get_example_tree_items();
    let opened = [vec![0], vec![1, 1]];
    let result = flatten(&opened, &items);
    let result_text = result
        .iter()
        .map(|o| get_naive_string_from_text(&o.item.lines().into()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_one_is_opened() {
    let items = get_example_tree_items();
    let opened = [vec![1]];
    let result = flatten(&opened, &items);
    let result_text = result
        .iter()
        .map(|o| get_naive_string_from_text(&o.item.lines().into()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "g", "h"]);
}

#[test]
fn get_opened_all_opened() {
    let items = get_example_tree_items();
    let opened = [vec![1], vec![1, 1]];
    let result = flatten(&opened, &items);
    let result_text = result
        .iter()
        .map(|o| get_naive_string_from_text(&o.item.lines().into()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "e", "f", "g", "h"]);
}
