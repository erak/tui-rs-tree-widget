use crate::identifier::{TreeIdentifier, TreeIdentifierVec};
use crate::{HasChildren, Visible};

/// Get a flat list of all visible [`TreeItem`s](TreeItem)
#[must_use]
pub fn flatten<'a, T: HasChildren<T> + Clone>(
    opened: &[TreeIdentifierVec],
    items: &'a [T],
) -> Vec<Visible<T>> {
    internal(opened, items, &[])
}

#[must_use]
fn internal<'a, T: HasChildren<T> + Clone>(
    opened: &[TreeIdentifierVec],
    items: &'a [T],
    current: TreeIdentifier,
) -> Vec<Visible<T>> {
    let mut result = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let mut child_identifier = current.to_vec();
        child_identifier.push(index);

        result.push(Visible {
            item: item.clone(),
            identifier: child_identifier.clone(),
        });

        if opened.contains(&child_identifier) {
            let mut child_result = internal(opened, item.children(), &child_identifier);
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
fn get_example_tree_items() -> Vec<crate::DefaultTreeItem<'static>> {
    use crate::DefaultTreeItem;

    vec![
        DefaultTreeItem::new_leaf("a"),
        DefaultTreeItem::new(
            "b",
            vec![
                DefaultTreeItem::new_leaf("c"),
                DefaultTreeItem::new(
                    "d",
                    vec![
                        DefaultTreeItem::new_leaf("e"),
                        DefaultTreeItem::new_leaf("f"),
                    ],
                ),
                DefaultTreeItem::new_leaf("g"),
            ],
        ),
        DefaultTreeItem::new_leaf("h"),
    ]
}

#[test]
fn get_opened_nothing_opened_is_top_level() {
    use crate::TreeItem;

    let items = get_example_tree_items();
    let result = flatten(&[], &items);
    let result_text = result
        .iter()
        .map(|o| get_naive_string_from_text(o.item.text()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_wrong_opened_is_only_top_level() {
    use crate::TreeItem;

    let items = get_example_tree_items();
    let opened = [vec![0], vec![1, 1]];
    let result = flatten(&opened, &items);
    let result_text = result
        .iter()
        .map(|o| get_naive_string_from_text(o.item.text()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_one_is_opened() {
    use crate::TreeItem;

    let items = get_example_tree_items();
    let opened = [vec![1]];
    let result = flatten(&opened, &items);
    let result_text = result
        .iter()
        .map(|o| get_naive_string_from_text(o.item.text()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "g", "h"]);
}

#[test]
fn get_opened_all_opened() {
    use crate::TreeItem;

    let items = get_example_tree_items();
    let opened = [vec![1], vec![1, 1]];
    let result = flatten(&opened, &items);
    let result_text = result
        .iter()
        .map(|o| get_naive_string_from_text(o.item.text()))
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "e", "f", "g", "h"]);
}
