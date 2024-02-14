mod app;

use app::*;

pub enum EventHandle {
    Quit,
    Move,
    None,
}

fn main() -> Result<(), std::io::Error> {
    crossterm::terminal::enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
    )?;

    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::terminal::Terminal::new(backend)?;

    let tick_rate = std::time::Duration::from_millis(25); //40Hz
    let mut app = App::new();

    run_app(&mut terminal, &mut app, tick_rate)?;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
    )?;

    Ok(())
}

fn run_app<B: tui::backend::Backend>(
    terminal: &mut tui::terminal::Terminal<B>,
    app: &mut App,
    tick_rate: std::time::Duration,
) -> Result<(), std::io::Error> {
    let args = std::env::args().collect();
    if !handle_arguments(app, &args) {
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    }

    app.init()?;

    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        if let EventHandle::Quit = handle_event(app, &mut last_tick, &tick_rate)? {
            break;
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}

fn handle_arguments(app: &mut App, args: &Vec<String>) -> bool {
    let mut i = 0;
    let length = args.len();
    while i < length {
        let arg = args[i].as_str();
        match arg {
            "--path" => {
                if i + 1 < length {
                    i += 1;
                    let path = std::path::PathBuf::from(&args[i]);
                    if path.exists() && path.is_dir() {
                        app.curr_location = match path.canonicalize() {
                            Ok(ap) => ap,
                            Err(_) => {
                                return false;
                            }
                        };
                    } else {
                        return false;
                    }
                }
            }
            _ => {}
        }

        i += 1;
    }
    true
}

fn handle_event(
    app: &mut App,
    last_tick: &mut std::time::Instant,
    tick_rate: &std::time::Duration,
) -> Result<EventHandle, std::io::Error> {
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| std::time::Duration::from_secs(0));

    if crossterm::event::poll(timeout)? {
        match crossterm::event::read()? {
            crossterm::event::Event::Key(key) => match key.code {
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                    return Ok(EventHandle::Quit);
                }
                crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                    let length = app.contents.len() as u16;
                    if app.curr_line != length - 1 {
                        app.curr_line += 1;
                    }
                    return Ok(EventHandle::Move);
                }
                crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                    if app.curr_line > 0 {
                        app.curr_line -= 1;
                    }
                    return Ok(EventHandle::Move);
                }
                crossterm::event::KeyCode::Enter => {
                    if app.change_directory()? {
                        // 1) renew contents list with changed current location
                        app.contents = App::create_list_by_location(&app.curr_location).unwrap();

                        // 2) sort
                        let sort_start_idx = if app.curr_location.as_os_str()
                            == std::path::Component::RootDir.as_os_str()
                        {
                            0
                        } else {
                            1
                        };
                        let slice = &mut app.contents[sort_start_idx..];
                        slice.sort_by(|a, b| a.0.cmp(&b.0));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(EventHandle::None)
}

fn get_fg_color_by_file_type(file_type: &FileType) -> tui::style::Color {
    match file_type {
        FileType::File => tui::style::Color::White,
        FileType::Directory => tui::style::Color::LightBlue,
        FileType::SymbolicFile => tui::style::Color::LightCyan,
        FileType::Socket => tui::style::Color::Magenta,
        FileType::Fifo => tui::style::Color::LightMagenta,
        FileType::BlockDevice => tui::style::Color::Yellow,
        FileType::CharDevice => tui::style::Color::LightYellow,
        FileType::Other => tui::style::Color::LightRed,
    }
}

fn ui<B: tui::backend::Backend>(f: &mut tui::terminal::Frame<B>, app: &mut App) {
    let frame_size = f.size();
    let margin = 5 as u16;

    // 1) render a frame Block (border)
    let block = tui::widgets::Block::default().style(
        tui::style::Style::default()
            .bg(tui::style::Color::Black)
            .fg(tui::style::Color::White),
    );
    f.render_widget(block, frame_size);

    // 2) layout
    let chunks = tui::layout::Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .margin(margin)
        .constraints([tui::layout::Constraint::Percentage(100)].as_ref())
        .split(frame_size);

    // 3) get closure creating paragraph block
    let closure_create_block = |title| {
        tui::widgets::Block::default()
            .borders(tui::widgets::Borders::ALL)
            .style(
                tui::style::Style::default()
                    .bg(tui::style::Color::Black)
                    .fg(tui::style::Color::White),
            )
            .title(tui::text::Span::styled(
                title,
                tui::style::Style::default().add_modifier(tui::style::Modifier::BOLD),
            ))
    };

    // 4) render paragraph for contents

    // calculate begining view index of contents list vector
    let view_height = chunks[0].height - 2; //2: border block size
    if app.view_line_start + view_height <= app.curr_line {
        app.view_line_start = app.curr_line - view_height + 1;
    } else if app.view_line_start > app.curr_line {
        app.view_line_start = app.curr_line;
    }

    // get view index
    let curr_i = app.curr_line as usize;
    let start_i = app.view_line_start as usize;
    let mut end_i = start_i + view_height as usize;
    if end_i >= app.contents.len() {
        end_i = app.contents.len();
    }

    // create view contents vector
    let mut i = 0 as usize;
    let mut text: Vec<tui::text::Spans> = vec![];
    for content in &app.contents[start_i..end_i] {
        let fg_color = get_fg_color_by_file_type(&content.1);

        // create view-text for a path
        let path = match &content.1 {
            FileType::SymbolicFile => {
                let mut real_path = app.curr_location.canonicalize().unwrap();
                real_path.push(content.0.as_str());
                if real_path.is_symlink() {
                    format!(
                        "{} -> {}",
                        &content.0,
                        real_path.read_link().unwrap().to_str().unwrap()
                    )
                } else {
                    format!("{} -> {}", &content.0, real_path.to_str().unwrap())
                }
            }
            _ => content.0.clone(),
        };

        if i == curr_i - start_i as usize {
            text.push(tui::text::Spans::from(vec![tui::text::Span::styled(
                path,
                tui::style::Style::default()
                    .bg(tui::style::Color::White)
                    .fg(tui::style::Color::Black),
            )]));
        } else {
            text.push(tui::text::Spans::from(vec![tui::text::Span::styled(
                path,
                tui::style::Style::default()
                    .bg(tui::style::Color::Black)
                    .fg(fg_color),
            )]));
        }

        i += 1;
    }

    // fill view buffer of terminal (=render)
    let contents_paragraph = tui::widgets::Paragraph::new(text.clone())
        .style(
            tui::style::Style::default()
                .bg(tui::style::Color::Black)
                .fg(tui::style::Color::White),
        )
        .block(closure_create_block(String::from(
            app.curr_location.to_str().unwrap(),
        )))
        .alignment(tui::layout::Alignment::Left);
    f.render_widget(contents_paragraph, chunks[0]);

    // 5) render paragraph for inforamtion
}
