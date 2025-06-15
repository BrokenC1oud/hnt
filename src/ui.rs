use crate::api::{HackerNews, Item};
use futures::future::join_all;

use color_eyre::Result;
use crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, List, ListState};
use ratatui::{crossterm, DefaultTerminal, Frame};
use tokio::runtime::Runtime;

pub struct App {
    hacker_news: HackerNews,
    
    exit: bool,

    story_list: Vec<Item>,
    story_list_state: ListState,
    
    thread_list: Vec<Item>,
    thread_list_state: ListState,

    async_runtime: Runtime,
}

impl App {
    pub fn new() -> App {
        let mut state = ListState::default();
        state.select(Some(1));
        App {
            exit: false,
            hacker_news: HackerNews::new("https://hacker-news.firebaseio.com/v0/"),
            async_runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .build().unwrap(),
            story_list: Vec::new(),
            story_list_state: state.clone(),
            thread_list: Vec::new(),
            thread_list_state: state.clone(),
        }
    }

    fn load_stories(&mut self) {
        self.story_list = self.async_runtime.block_on(join_all(
            // TODO: Show-purposed changed this to top stories
            self.async_runtime.block_on(self.hacker_news.get_top_stories()).unwrap().iter()
                .take(100)
                .map(|&id| (async |id| self.hacker_news.get_item(id).await.unwrap())(id))
                .collect::<Vec<_>>()
        ))
    }

    fn load_thread(&mut self) {
        let item = self.story_list.get(self.story_list_state.selected().unwrap()).unwrap();
        if let Item::Story { kids, .. } = item {
            self.thread_list = self.async_runtime.block_on(join_all(
                kids.clone().unwrap_or(vec![]).iter()
                    .map(|&id| (async |id| self.hacker_news.get_item(id).await.unwrap())(id))
                    .collect::<Vec<_>>()
            ));
            self.thread_list.insert(0, item.clone());
        };
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(frame.area());

        // Story list
        let story_list_block = Block::bordered()
            .title("New Stories");
        let story_list = List::new(&self.story_list)
            .block(story_list_block)
            .highlight_style(Style::default().reversed());
        frame.render_stateful_widget(
            story_list,
            layout[0],
            &mut self.story_list_state
        );

        // Story comments
        let thread_block = Block::default()
            .title("Details")
            .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM);
        let thread_list = List::new(&self.thread_list)
            .block(thread_block);
        frame.render_stateful_widget(
            thread_list,
            layout[1],
            &mut self.thread_list_state,
        )
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_events(key_event);
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,

            KeyCode::Char('r') => self.load_stories(),
            
            KeyCode::Left => {
                self.story_list_state.select_previous();
                self.load_thread();
            }
            KeyCode::Right => {
                self.story_list_state.select_next();
                self.load_thread();
            }
            
            KeyCode::Up => {
                self.thread_list_state.select_previous();
            }
            KeyCode::Down => {
                self.thread_list_state.select_next();
            }
            _ => {}
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|f| self.render(f))?;
        while !self.exit {
            self.handle_events()?;
            terminal.draw(|f| self.render(f))?;
        }
        Ok(())
    }
}
