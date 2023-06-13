#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashSet;
use std::str::FromStr;

use textwrap;

use tui::buffer::Buffer;
use tui::layout::{Alignment, Corner, Rect};
use tui::style::Style;
use tui::text::{Span, Spans, StyledGrapheme, Text};
use tui::widgets::{Block, StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

mod flatten;
mod identifier;
mod reflow;

pub use crate::flatten::{flatten, Flattened};
pub use crate::identifier::{
    get_without_leaf as get_identifier_without_leaf, TreeIdentifier, TreeIdentifierVec,
};
use crate::reflow::{LineComposer, WordWrapper};

pub type SharedTreeItem<'a> = Rc<RefCell<TreeItem<'a>>>;

/// Keeps the state of what is currently selected and what was opened in a [`Tree`]
///
/// # Example
///
/// ```
/// # use tui_tree_widget::TreeState;
/// let mut state = TreeState::default();
/// ```
#[derive(Debug, Default, Clone)]
pub struct TreeState {
    offset: usize,
    opened: HashSet<TreeIdentifierVec>,
    selected: TreeIdentifierVec,
}

impl TreeState {
    #[must_use]
    pub const fn get_offset(&self) -> usize {
        self.offset
    }

    #[must_use]
    pub fn get_all_opened(&self) -> Vec<TreeIdentifierVec> {
        self.opened.iter().cloned().collect()
    }

    #[must_use]
    pub fn selected(&self) -> Vec<usize> {
        self.selected.clone()
    }

    pub fn select<I>(&mut self, identifier: I)
    where
        I: Into<Vec<usize>>,
    {
        self.selected = identifier.into();

        // TODO: ListState does this. Is this relevant?
        if self.selected.is_empty() {
            self.offset = 0;
        }
    }

    /// Open a tree node.
    /// Returns `true` if the node was closed and has been opened.
    /// Returns `false` if the node was already open.
    pub fn open(&mut self, identifier: TreeIdentifierVec) -> bool {
        if identifier.is_empty() {
            false
        } else {
            self.opened.insert(identifier)
        }
    }

    /// Close a tree node.
    /// Returns `true` if the node was open and has been closed.
    /// Returns `false` if the node was already closed.
    pub fn close(&mut self, identifier: TreeIdentifier) -> bool {
        self.opened.remove(identifier)
    }

    /// Toggles a tree node.
    /// If the node is in opened then it calls `close()`. Otherwise it calls `open()`.
    pub fn toggle(&mut self, identifier: TreeIdentifierVec) {
        if self.opened.contains(&identifier) {
            self.close(&identifier);
        } else {
            self.open(identifier);
        }
    }

    /// Toggles the currently selected tree node.
    /// See also [`toggle`](TreeState::toggle)
    pub fn toggle_selected(&mut self) {
        self.toggle(self.selected());
    }

    pub fn close_all(&mut self) {
        self.opened.clear();
    }

    /// Select the first node.
    pub fn select_first(&mut self) {
        self.select(vec![0]);
    }

    /// Select the last node.
    pub fn select_last(&mut self, items: &[SharedTreeItem]) {
        let visible = flatten(&self.get_all_opened(), items);
        let new_identifier = visible
            .last()
            .map(|o| o.identifier().clone())
            .unwrap_or_default();
        self.select(new_identifier);
    }

    /// Handles the up arrow key.
    /// Moves up in the current depth or to its parent.
    pub fn key_up(&mut self, items: &[SharedTreeItem]) {
        let visible = flatten(&self.get_all_opened(), items);
        let current_identifier = self.selected();
        let current_index = visible
            .iter()
            .position(|o| *o.identifier() == current_identifier);
        let new_index = current_index.map_or(0, |current_index| {
            current_index.saturating_sub(1).min(visible.len() - 1)
        });
        let new_identifier = visible
            .get(new_index)
            .map(|o| o.identifier().clone())
            .unwrap_or_default();
        self.select(new_identifier);
    }

    /// Handles the down arrow key.
    /// Moves down in the current depth or into a child node.
    pub fn key_down(&mut self, items: &[SharedTreeItem]) {
        let visible = flatten(&self.get_all_opened(), items);
        let current_identifier = self.selected();
        let current_index = visible
            .iter()
            .position(|o| *o.identifier() == current_identifier);
        let new_index = current_index.map_or(0, |current_index| {
            current_index.saturating_add(1).min(visible.len() - 1)
        });
        let new_identifier = visible
            .get(new_index)
            .map(|o| o.identifier().clone())
            .unwrap_or_default();
        self.select(new_identifier);
    }

    /// Handles the left arrow key.
    /// Closes the currently selected or moves to its parent.
    pub fn key_left(&mut self) {
        let selected = self.selected();
        if !self.close(&selected) {
            let (head, _) = get_identifier_without_leaf(&selected);
            self.select(head);
        }
    }

    /// Handles the right arrow key.
    /// Opens the currently selected.
    pub fn key_right(&mut self) {
        self.open(self.selected());
    }
}

pub enum Wrap {
    Width(usize),
    None,
}

// #[derive(Clone, Debug)]
// pub enum Wrap {
//     Width(usize),
//     None,
// }

/// One item inside a [`Tree`]
///
/// Can zero or more `children`.
///
/// # Example
///
/// ```
/// # use tui_tree_widget::TreeItem;
/// let a = TreeItem::new_leaf("leaf");
/// let b = TreeItem::new("root", vec![a]);
/// ```
#[derive(Debug, Clone)]
pub struct TreeItem<'a> {
    text: Text<'a>,
    // wrapped_text: Option<Text<'a>>,
    style: Style,
    children: Vec<SharedTreeItem<'a>>,
}

impl<'a> TreeItem<'a> {
    #[must_use]
    pub fn new_leaf<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            text: text.into(),
            style: Style::default(),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn new<T, Children>(text: T, children: Children) -> Self
    where
        T: Into<Text<'a>>,
        Children: Into<Vec<TreeItem<'a>>>,
    {
        Self {
            text: text.into(),
            style: Style::default(),
            children: children
                .into()
                .into_iter()
                .map(|item| Rc::new(RefCell::new(item)))
                .collect::<Vec<_>>(),
        }
    }

    #[must_use]
    pub fn children(&self) -> &[SharedTreeItem<'a>] {
        &self.children
    }

    #[must_use]
    pub fn child(&self, index: usize) -> Option<SharedTreeItem<'a>> {
        self.children.get(index).cloned()
    }

    fn wrap(&mut self, width: usize) {
        // self.text = self.text.clone();
        // println!("wrapping");
        // let text = self.text.clone();
        // let styled = text.lines.iter().map(|line| {
        //     (
        //         line.0
        //             .iter()
        //             .flat_map(|span| span.styled_graphemes(self.style)),
        //         Alignment::Left,
        //     )
        // });
        // let mut line_composer: Box<dyn LineComposer> =
        //     Box::new(WordWrapper::new(styled, width as u16, true));
        // let mut lines = vec![];
        // for line in self.text.clone() {
        //     let mut spans = vec![];
        //     for span in line.0 {
        //         spans.push(span);
        //     }
        //     let spans = Spans::from(spans);
        //     lines.push(spans);
        // }
        // let mut lines = vec![];
        // for line in self.lines() {
        //     lines.p
        // }
        // self.text = lines.into();
        // self
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.text.lines.len()
    }

    // pub fn text(&self) -> Text<'a> {
    //     self.text.clone()
    // }

    pub fn lines(&mut self, wrap: Wrap) -> &[StyledGrapheme<'a>] {
        let lines = self.text.lines.iter().map(|line| {
            line.0
                .iter()
                .flat_map(|span| span.styled_graphemes(self.style))
        });
        &lines.collect();
        // if let Wrap::Width(width) = wrap {
        //     &lines.collect();
        // } else {
        //     &lines
        // }

        // vec![Spans::from(vec![Span::raw("dsdsd")])]
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn add_child(&mut self, child: SharedTreeItem<'a>) {
        self.children.push(child);
    }
}

/// A `Tree` which can be rendered
///
/// # Example
///
/// ```
/// # use tui_tree_widget::{Tree, TreeItem, TreeState};
/// # use tui::backend::TestBackend;
/// # use tui::Terminal;
/// # use tui::widgets::{Block, Borders};
/// # fn main() -> std::io::Result<()> {
/// #     let mut terminal = Terminal::new(TestBackend::new(32, 32)).unwrap();
/// let mut state = TreeState::default();
///
/// let item = TreeItem::new_leaf("leaf");
/// let items = vec![item];
///
/// terminal.draw(|f| {
///     let area = f.size();
///
///     let tree_widget = Tree::new(items.clone())
///         .block(Block::default().borders(Borders::ALL).title("Tree Widget"));
///
///     f.render_stateful_widget(tree_widget, area, &mut state);
/// })?;
/// #     Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Tree<'a> {
    items: Vec<SharedTreeItem<'a>>,

    block: Option<Block<'a>>,
    /// ..... if item content should be wrapped.
    wrap: bool,
    start_corner: Corner,
    /// Style used as a base style for the widget
    style: Style,

    /// Style used to render selected item
    highlight_style: Style,
    /// Symbol in front of the selected item (Shift all items to the right)
    highlight_symbol: &'a str,

    /// Symbol displayed in front of a closed node (As in the children are currently not visible)
    node_closed_symbol: &'a str,
    /// Symbol displayed in front of an open node. (As in the children are currently visible)
    node_open_symbol: &'a str,
    /// Symbol displayed in front of a node without children.
    node_no_children_symbol: &'a str,
}

impl<'a> Tree<'a> {
    #[must_use]
    pub fn new<T>(items: T) -> Self
    where
        T: Into<Vec<SharedTreeItem<'a>>>,
    {
        Self {
            items: items.into(),
            block: None,
            wrap: false,
            start_corner: Corner::TopLeft,
            style: Style::default(),
            highlight_style: Style::default(),
            highlight_symbol: "",
            node_closed_symbol: "\u{25b6} ", // Arrow to right
            node_open_symbol: "\u{25bc} ",   // Arrow down
            node_no_children_symbol: "  ",
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use]
    pub const fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub const fn start_corner(mut self, corner: Corner) -> Self {
        self.start_corner = corner;
        self
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub const fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    #[must_use]
    pub const fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.highlight_symbol = highlight_symbol;
        self
    }

    #[must_use]
    pub const fn node_closed_symbol(mut self, symbol: &'a str) -> Self {
        self.node_closed_symbol = symbol;
        self
    }

    #[must_use]
    pub const fn node_open_symbol(mut self, symbol: &'a str) -> Self {
        self.node_open_symbol = symbol;
        self
    }

    #[must_use]
    pub const fn node_no_children_symbol(mut self, symbol: &'a str) -> Self {
        self.node_no_children_symbol = symbol;
        self
    }
}

impl<'a> StatefulWidget for Tree<'a> {
    type State = TreeState;

    #[allow(clippy::too_many_lines)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);

        // Get the inner area inside a possible block, otherwise use the full area
        let area = self.block.map_or(area, |b| {
            let inner_area = b.inner(area);
            b.render(area, buf);
            inner_area
        });

        if area.width < 1 || area.height < 1 {
            return;
        }

        // TODO: Optimize to wrap only vivible items.
        // let items = if self.wrap {
        //     self.items
        //         .iter()
        //         .map(|item| {
        //             let max_indent = 6;
        //             item.clone()
        //                 .wrap(area.width.saturating_sub(max_indent) as usize)
        //         })
        //         .collect::<Vec<_>>()
        // } else {
        //     self.items
        // };

        let visible = flatten(&state.get_all_opened(), &self.items);
        if visible.is_empty() {
            return;
        }
        let available_height = area.height as usize;

        let selected_index = if state.selected.is_empty() {
            0
        } else {
            visible
                .iter()
                .position(|o| *o.identifier() == state.selected)
                .unwrap_or(0)
        };

        if self.wrap {
            for flattened in &visible {
                let indent = flattened.depth() * 2;
                let width = area.width as usize;
                flattened.item().borrow_mut().wrap(width.saturating_sub(indent));
            }
        }

        let mut start = state.offset.min(selected_index);
        let mut end = start;
        let mut height = 0;
        for flattened in visible.iter().skip(start) {
            if height + flattened.item().borrow().height() > available_height {
                break;
            }

            height += flattened.item().borrow().height();
            end += 1;
        }

        while selected_index >= end {
            height = height.saturating_add(visible[end].item().borrow().height());
            end += 1;
            while height > available_height {
                height = height.saturating_sub(visible[start].item().borrow().height());
                start += 1;
            }
        }

        state.offset = start;

        let blank_symbol = " ".repeat(self.highlight_symbol.width());

        let mut current_height = 0;
        let has_selection = !state.selected.is_empty();

        #[allow(clippy::cast_possible_truncation)]
        for flattened in visible.into_iter().skip(state.offset).take(end - start) {
            #[allow(clippy::single_match_else)] // Keep same as List impl
            let (x, y) = match self.start_corner {
                Corner::BottomLeft => {
                    current_height += flattened.item().borrow().height() as u16;
                    (area.left(), area.bottom() - current_height)
                }
                _ => {
                    let pos = (area.left(), area.top() + current_height);
                    current_height += flattened.item().borrow().height() as u16;
                    pos
                }
            };
            let area = Rect {
                x,
                y,
                width: area.width,
                height: flattened.item().borrow().height() as u16,
            };

            let item_style = self.style.patch(flattened.item().borrow().style);
            buf.set_style(area, item_style);

            let is_selected = state.selected == *flattened.identifier();
            let after_highlight_symbol_x = if has_selection {
                let symbol = if is_selected {
                    self.highlight_symbol
                } else {
                    &blank_symbol
                };
                let (x, _) = buf.set_stringn(x, y, symbol, area.width as usize, item_style);
                x
            } else {
                x
            };

            let after_depth_x = {
                let indent_width = flattened.depth() * 2;
                let (after_indent_x, _) = buf.set_stringn(
                    after_highlight_symbol_x,
                    y,
                    " ".repeat(indent_width),
                    indent_width,
                    item_style,
                );
                let symbol = if flattened.item().borrow().children().is_empty() {
                    self.node_no_children_symbol
                } else if state.opened.contains(flattened.identifier()) {
                    self.node_open_symbol
                } else {
                    self.node_closed_symbol
                };
                let max_width = area.width.saturating_sub(after_indent_x - x);
                let (x, _) =
                    buf.set_stringn(after_indent_x, y, symbol, max_width as usize, item_style);
                x
            };

            let max_element_width = area.width.saturating_sub(after_depth_x - x);
            for (j, line) in flattened
                .item()
                .borrow_mut()
                .lines()
                .iter()
                .enumerate()
            {
                buf.set_spans(after_depth_x, y + j as u16, line, max_element_width);
            }
            if is_selected {
                buf.set_style(area, self.highlight_style);
            }
        }
    }
}

impl<'a> Widget for Tree<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TreeState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}
