use std::{fmt::format, io, iter::Sum};

use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Span}, widgets::{Block, Borders, Gauge, Paragraph, Widget}};

use crate::process::Process;

const TOP_N_REGIONS: usize = 5;

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
                format!("{} ", self.process.cmd_line).white()
            ]);
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);


        let block = Block::bordered()
            .title(title.left_aligned())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let memory_totals= self.process.get_rss_totals();
        let total: u64 = memory_totals.iter().map(|x| x.1).sum();
        let top_n_totals:Vec<&(String, u64)> = memory_totals.iter().take(TOP_N_REGIONS).collect();
                                                                                                                                                              
        // Build constraints: [gauge, spacer, gauge, spacer, ...]                                                                                           
        let mut constraints = Vec::new();                                                                                                                   
        for _ in 0..top_n_totals.len() {                                                                                                                           
            constraints.push(Constraint::Length(1)); // gauge row                                                                                           
            constraints.push(Constraint::Length(1)); // spacer row                                                                                          
        }    

        // Create vertical chunks for each region's rss 
        let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints).split(inner_area);

    
        let bar_colors = [
            Color::Cyan,
            Color::Green,
            Color::Yellow,
            Color::Magenta,
            Color::Red,
        ];

        for (i, (name, rss_kb)) in top_n_totals.iter().enumerate() {
            let chunk_idx = i * 2;
            let color = bar_colors[i % bar_colors.len()];

            // split each row, horizontally: [name | bar | amount]
            let row_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(50),
                Constraint::Percentage(20)
            ]).split(gauge_chunks[chunk_idx]);

            // render the name (left)
            let name_widget = Paragraph::new(name.as_str()).alignment(Alignment::Left);
            name_widget.render(row_chunk[0], buf);

            // then the bar (middle)
            let gauge = Gauge::default()
            .gauge_style(Style::default()
                .fg(color)
                .bg(Color::DarkGray)
            )
            .ratio(*rss_kb as f64 / total as f64);

            gauge.render(row_chunk[1], buf);

            // then the memory amount (right)
            let amount_widget = Paragraph::new(format!("{} kB", rss_kb)).alignment(Alignment::Right);
            amount_widget.render(row_chunk[2], buf);
        }
    }
}