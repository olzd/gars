use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;
use crate::genome::Genome;
use crate::simulation;
use crate::simulation::Simulation;

pub fn run_watch<R: Rng>(mut sim: Simulation, genome: &Genome, rng: &mut R) -> Result<()> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // conf
    let mut auto_run = false;
    let auto_delay = Duration::from_millis(120);
    // draw initial state
    terminal.draw(|f| draw_frame(f, &sim))?;

    loop {
        if auto_run {
            // Check for interrupt key
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q' | 'Q') => break,
                        KeyCode::Char('a' | 'A') => auto_run = false,
                        _ => {}
                    }
                }
            }
            let _ = sim.step(genome, rng);
            terminal.draw(|f| draw_frame(f, &sim))?;
            std::thread::sleep(auto_delay);
        } else if let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char(' ') => {
                    let _ = sim.step(genome, rng);
                    terminal.draw(|f| draw_frame(f, &sim))?;
                },
                KeyCode::Char('a' | 'A') => auto_run = true,
                KeyCode::Char('q' | 'Q') => break,
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_frame(f: &mut Frame, sim: &Simulation) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    // Stats bar (unchanged)
    let stats = format!(
        "Steps: {} | Length: {} | Food eaten: {}",
        sim.steps,
        sim.snake.len(),
        sim.snake.len() - 3,
    );
    f.render_widget(
        Paragraph::new(stats).block(Block::default().borders(Borders::ALL).title("Stats")),
        chunks[0],
    );

    // Grid drawn with box‑drawing characters
    let width = sim.width;
    let height = sim.height;
    let cell_width = 3;

    // Helper: choose tile symbol and style
    let tile_char = |x: usize, y: usize| -> (char, Style) {
        let pos = (x, y);
        if Some(&pos) == sim.snake.front() {
            let c = match sim.direction {
                simulation::Direction::Up => '^',
                simulation::Direction::Down => 'v',
                simulation::Direction::Left => '<',
                simulation::Direction::Right => '>',
            };
            (c, Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else if sim.snake.contains(&pos) {
            (' ', Style::default().bg(Color::Green))
        } else if pos == sim.food {
            ('◆', Style::default().fg(Color::Red))
        } else {
            (' ', Style::default())
        }
    };

    let mut lines = vec![];

    // Top border
    let top = "┌".to_string()
        + &(0..width)
        .map(|_| "─".repeat(cell_width))
        .collect::<Vec<_>>()
        .join("")
        + "┐";
    lines.push(ratatui::text::Line::from(Span::raw(top)));

    for y in 0..height {
        // Content row
        let mut row = vec![Span::raw("│")];
        for x in 0..width {
            let (ch, style) = tile_char(x, y);
            let content = format!(" {} ", ch);
            row.push(Span::styled(content, style));
        }
        row.push(Span::raw("│"));
        lines.push(ratatui::text::Line::from(row));
    }

    // Bottom border
    let bottom = "└".to_string()
        + &(0..width)
        .map(|_| "─".repeat(cell_width))
        .collect::<Vec<_>>()
        .join("")
        + "┘";
    lines.push(ratatui::text::Line::from(Span::raw(bottom)));

    let grid_para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Grid ({}x{})", width, height)),
    );
    f.render_widget(grid_para, chunks[1]);
}
