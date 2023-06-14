mod util;

use std::time::{Duration, SystemTime};
use std::thread::sleep;

use crate::util::StatefulTree;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Terminal,
};

const NODE_0: &str = r"Greedy algorithm dependency injection MIT license build tool constant FP GraphQL off-by-one error duck typing. Webpack var UX MVP progressive web app circle back quick sort documentation driven concurrent pattern. Off-by-one error vaporware test double heap sort antipattern Slack RPC websockets RSS feed frame rate. Class XML animation native perf matters Angular. Team-player font looks good to me resource Slack transpile command-line Github Firefox api";
const NODE_1: &str = r"Yarn variable object library uglify pull request REST cherry pick code-splitting killer app void. AWS module package manager whiteboard tree shaking mock legacy code S3. Data store module bootcamp MIT license Angular CSS grid CLI CS degree API. Variable mock domain specific language progressive web app OTP lang perf matters programmer.
Minimum viable product api internet button command-line dynamic programming MVP proof of stake environment. OOP security var i concurrency sudo. Tech debt website elixir Ruby YAML stack greedy algorithm Chrome.";
const NODE_2: &str = r"Tree shaking fullstack CLI open source machine learning website design git pull request. Progressive web app maintainable minification const dynamic container optimize linker observer pattern. Kubernetes open source minimum viable product Slack module senior-engineer Firefox.";
const NODE_3: &str = r"Promise compression polemical thinking cowboy coding atomic design subclass first in first out junior continuous integration test double.";
const NODE_4: &str = r"DevTools bootcamp accessibility bubble sort commit Ruby. Backend Safari JVM Linux lang dynamic hashtable containerized hardcoded. Animation subclass flexbox devops architecture command-line Linux. Responsive one-size-fits-all approach documentation driven frontend raspberry pi Byzantine fault tolerance Kubernetes optimize gradle.";
const NODE_5: &str = r"Stack Overflow dynamic programming GraphQL module meta-programming stack trace OOP TL LGTM. Developer avocado Netscape commit AWS neck beard class.";
const NODE_6: &str = r"YAML linker CSV stateless Byzantine fault tolerance model.";
const NODE_7: &str = r"Blog uglify API public Internet Explorer native presenter private parent team-player. TL continuous integration SRE linker waterfall yarn. JavaScript browser stand-up stack trace meta-programming LLVM. I Safari little Bobby Tables architecture command-line YAML dynamic programming.

Progressive web app OTP freelancer TOML serverless rm -rf * val maintainable JQuery gate-keeping. Idiosyncratic contexts uglify static composition over inheritance inheritance yarn stand-up data store array JSX. Model scale clean architecture internet button Dijkstra CS degree. Observer pattern Agile reactive Medium post DOM Github. Public scrum master dog-piling composition spaghetti code Cloudfront serverless test-driven attributes ecommerce platform.

Legacy code antipattern bitcoin Ubuntu callback const mock. Class polemical thinking .NET dynamic flexbox tabs vs spaces streams Linux freelancer Slack. API consensus cloud off-by-one error remote class i modern bundle. A place for everything instance state killer app dependency injection security asynchronous reflection architecture. S3 JSON composition Ruby contribute engineer security blog singleton.

YAML linker CSV stateless Byzantine fault tolerance model. Byzantine fault tolerance Netscape raspberry pi module machine learning Internet Explorer React naming things build tool. Callback XML npm scrum master state distributed systems. Const distributed systems Edge yarn MIT license AI diversity and inclusion chmod subclass Angular.
DSL JVM fault tolerant service worker design package manager yarn senior-engineer incognito Angular. Free as in beer team-player MIT license elixir MacBook compression parameter looks good to me. Ship it Linux progressive web app stack trace transpile homebrew site reliability engineer. Internet button public key-value React Internet Explorer duck typing LIFO XML meta-programming.

Private virtual DOM gzip chmod free as in beer pull request scrum master queue strongly typing DOM. Distributed systems TL composition browser parent flexbox JavaScript proof of stake open source FP. Netscape Safari ELF private backend Internet Explorer. Hacker News MVP shadow DOM val data store container Twitter api clean code security.

Greenfield private rm -rf * views npm val legacy code scalable engineer. Distributed sudo tree shaking dynamic programming tech debt transpile coding bootcamp interface distributed systems. Serverless UX program frame rate const bike-shedding. Markup SOAP controller transpile views legacy code document object model mechanical keyboard. Greenfield clean code blockchain cache proof of stake CSS grid TOML UX.";

use tui_tree_widget::{Tree, TreeItem};

#[derive(Debug)]
struct Performance {
    pub render_time: f64,
}

struct App<'a> {
    tree: StatefulTree<'a>,
    performance: Performance,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let mut tree = StatefulTree::with_items(vec![
            TreeItem::new_leaf(NODE_0),
            TreeItem::new(
                NODE_1,
                vec![
                    TreeItem::new_leaf(NODE_2),
                    TreeItem::new(
                        NODE_3,
                        vec![TreeItem::new_leaf(NODE_4), TreeItem::new_leaf(NODE_5)],
                    ),
                    TreeItem::new_leaf(NODE_6),
                ],
            ),
            TreeItem::new_leaf(NODE_7),
        ]);
        tree.first();

        let performance = Performance { render_time: 0.0 };

        Self { tree, performance }
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
    let mut render_times = vec![];
    loop {
        let now = SystemTime::now();
        terminal.draw(|f| {
            let area = f.size();

            let items = Tree::new(app.tree.items.clone())
                .wrap(true)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Tree Widget {:?} Render time: {:?}", app.tree.state, app.performance)),
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

        if let Ok(elapsed) = now.elapsed() {
            render_times.push(elapsed.as_millis())
        }

        let sum: u128 = render_times.iter().sum();
        app.performance.render_time = sum as f64 / render_times.len() as f64;

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