mod util;

use crate::util::StatefulTree;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, vec};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, StyledGrapheme, Text},
    widgets::{Block, Borders},
    Terminal,
};
use tui_tree_widget::reflow::WordWrapper;
use tui_tree_widget::{reflow::LineComposer, HasChildren, Tree, TreeItem};

#[derive(Debug, Clone)]
pub struct CustomTreeItem<'a> {
    title: Spans<'a>,
    content: Option<Text<'a>>,
    style: Style,
    children: Vec<CustomTreeItem<'a>>,
}

impl<'a> CustomTreeItem<'a> {
    #[must_use]
    pub fn new_leaf<S>(title: S) -> Self
    where
        S: Into<Spans<'a>>,
    {
        Self {
            title: title.into(),
            content: None,
            style: Style::default(),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn new<S, Children>(title: S, children: Children) -> Self
    where
        S: Into<Spans<'a>> + Clone,
        Children: Into<Vec<CustomTreeItem<'a>>>,
    {
        Self {
            title: title.clone().into(),
            content: None,
            style: Style::default(),
            children: children.into(),
        }
    }

    #[must_use]
    pub fn with_content<T>(mut self, content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        self.content = Some(content.into());
        self
    }

    #[must_use]
    pub fn children(&self) -> &Vec<CustomTreeItem<'a>> {
        &self.children
    }

    #[must_use]
    pub fn child(&self, index: usize) -> Option<&Self> {
        self.children.get(index)
    }

    #[must_use]
    pub fn child_mut(&mut self, index: usize) -> Option<&mut Self> {
        self.children.get_mut(index)
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn add_child(&mut self, child: CustomTreeItem<'a>) {
        self.children.push(child);
    }
}

impl<'a> TreeItem<'a> for CustomTreeItem<'a> {
    // fn height(&self) -> usize {
    //     // let title_h = 1_usize;
    //     // let content_h = match &self.content {
    //     //     Some(content) => content.height(),
    //     //     _ => 0,
    //     // };
    //     // title_h.saturating_add(content_h)
    //     self.graphemes.len()
    // }

    fn style(&self) -> Style {
        self.style
    }

    fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    // fn text(&self) -> &Text<'a> {
    //     // match &self.content {
    //     //     Some(content) => {
    //     //         let mut spans = vec![self.title.clone()];
    //     //         spans.extend(content.clone());
    //     //         spans.into()
    //     //     }
    //     //     _ => self.title.clone().into(),
    //     // }
    //     &self.text
    // }
    fn graphemes(&self, area: Rect) -> Vec<Vec<StyledGrapheme<'a>>> {
        // self.graphemes.clone()
        vec![]
    }

    // fn wrap(self, area: Rect) -> Self {
    //     if let Some(content) = &self.content {
            // let styled = content.lines.iter().map(|line| {
            //     (
            //         line.0
            //             .iter()
            //             .flat_map(|span| span.styled_graphemes(self.style)),
            //         Alignment::Left,
            //     )
            // });
            // let mut wrapper = Box::new(WordWrapper::new(styled, 20, true));

            // let mut lines = vec![];
            // while let Some((line, _, _)) = wrapper.next_line() {
            //     let spans = line
            //         .iter()
            //         .map(|StyledGrapheme { symbol, style }| Span::styled(*symbol, *style))
            //         .collect::<Vec<_>>();
            //     lines.push(Spans::from(spans));
            // }

    //         // println!("{:?}", content);
    //         let text: Text<'a> = lines.into();

    //         return self.with_content(text);
    //     }
    //     self
    // }

    fn wrap(&'a mut self, area: Rect) {
        // self.graphemes.clear();

        if let Some(content) = &self.content {
            let styled = content.lines.iter().map(|line| {
                (
                    line.0
                        .iter()
                        .flat_map(|span| span.styled_graphemes(self.style)),
                    Alignment::Left,
                )
            });
            let mut wrapper = Box::new(WordWrapper::new(styled, 20, true));

            // let mut lines = vec![];
            while let Some((line, _, _)) = wrapper.next_line() {
                // let spans = line
                //     .iter()
                //     .map(|StyledGrapheme { symbol, style }| Span::styled(*symbol, *style))
                //     .collect::<Vec<_>>();
                // lines.push(Spans::from(spans));
                // self.graphemes.push(line.to_vec())
            }

            // self.text = lines.into();
            // 
        }
    }
}

impl<'a> HasChildren<CustomTreeItem<'a>> for CustomTreeItem<'a> {
    fn children(&self) -> &Vec<CustomTreeItem<'a>> {
        &self.children
    }
}

const NODE_CONTENT: &str = r"Devops terminal linker XML greenfield tl;dr. Transpile JSON object library reactive DSL private. Font security Stack Overflow terminal bike-shedding inheritance. XML contribute scalable SOAP freelancer first in first out compiler. Off-by-one error CSS-in-JS OOP S3 static container resolve internet button.";

struct App<'a> {
    tree: StatefulTree<CustomTreeItem<'a>>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            tree: StatefulTree::with_items(vec![
                CustomTreeItem::new_leaf("a").with_content(NODE_CONTENT),
                CustomTreeItem::new(
                    "b",
                    vec![
                        CustomTreeItem::new_leaf("c"),
                        CustomTreeItem::new(
                            "d",
                            vec![CustomTreeItem::new_leaf("e"), CustomTreeItem::new_leaf("f")],
                        ),
                        CustomTreeItem::new_leaf("g"),
                    ],
                ),
                CustomTreeItem::new_leaf("h"),
            ]),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.size();

            let items = Tree::new(app.tree.items.clone())
                .wrap(true)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Tree Widget {:?}", app.tree.state)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(items, area, &mut app.tree.state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('\n' | ' ') => app.tree.toggle(),
                KeyCode::Left => app.tree.left(),
                KeyCode::Right => app.tree.right(),
                KeyCode::Down => app.tree.down(),
                KeyCode::Up => app.tree.up(),
                KeyCode::Home => app.tree.first(),
                KeyCode::End => app.tree.last(),
                _ => {}
            }
        }
    }
}
