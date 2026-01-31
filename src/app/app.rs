use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line}, widgets::{Block, Paragraph, Widget}};

use crate::process::Process;

pub struct App {
    process: Process,
    exit: bool,
}

impl App {
    pub fn new(process: Process) -> Self {
        Self {
            process: process,
            exit: false,
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    self.handle_key_event(key)
                }
                Event::Resize(_, _) => {
                    // Terminal is resized
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {                                                                                                                                
          KeyCode::Char('q') | KeyCode::Char('Q') => {                                                                                                
              self.exit = true;                                                                                                                       
          }                                                                                                                                                                                                                                                                              
          _ => {}                                                                                                                                     
        } 
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(
            vec![
                format!(" Process: {}: ", self.process.pid).yellow(),
                format!("{}", self.process.cmd_line).white()
            ]);
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Paragraph::new("")
        .centered()
        .block(block)
        .render(area, buf);
    }
}