enum FileType {
    File,
    Directory,
    SymbolicFile,
    SymbolicDirectory,
    Other,
}

enum EventHandle {
    Quit,
    Move,
    None,
}

struct App {
    contents: Vec<(String, FileType)>,
    curr_location: String,
    curr_line: u16,
    view_line_start: u16,
    //information: Vec<String>, // TODO
}

impl App {
    fn new() -> Self {
        App {
            contents: vec![], // contents list
            curr_location: String::from(
                //current direction
                std::fs::canonicalize("./")
                    .unwrap()
                    .as_path()
                    .to_str()
                    .unwrap(),
            ),
            curr_line: 0, // line (index) of contents list
            view_line_start: 0,
            //information: Vec<String>, // TODO
        }
    }

    fn create_list_by_location(
        location: &String,
    ) -> Result<Vec<(String, FileType)>, std::io::Error> {
        // 1) get list(files, dirs) in this location
        let mut paths = std::fs::read_dir(location)?;

        // 2) vector push: all stuffs in this location
        let mut contents_vec = Vec::<(String, FileType)>::new();
        if std::fs::canonicalize(location).unwrap().to_str().unwrap() != "/" {
            contents_vec.push((String::from(".."), FileType::Directory));
        }
        for path in &mut paths {
            let rel_path = path.unwrap().path();
            let abs_path = std::fs::canonicalize(std::path::Path::new(&rel_path))?;
            let file_name = abs_path.file_name().unwrap().to_str().unwrap();
            let symlink_meta = std::fs::symlink_metadata(&rel_path)?;

            let mut displable_name = String::from(file_name);
            let mut file_type = FileType::File;

            if symlink_meta.file_type().is_symlink() {
                let real_path = std::fs::read_link(&rel_path).unwrap();
                displable_name = format!(
                    "{} -> {}",
                    rel_path.file_name().unwrap().to_str().unwrap(),
                    real_path.to_str().unwrap(),
                );
                file_type = FileType::SymbolicFile;
            }

            if abs_path.is_dir() {
                displable_name += "/";
                if let FileType::SymbolicFile = file_type {
                    file_type = FileType::SymbolicDirectory;
                } else {
                    file_type = FileType::Directory;
                }
            } else if !abs_path.is_file() {
                displable_name += "?";
                file_type = FileType::Other;
            }

            contents_vec.push((displable_name, file_type));
        }

        Ok(contents_vec)
    }

    fn init(&mut self) -> Result<(), std::io::Error> {
        // 1) create current location's contents list
        self.contents = App::create_list_by_location(&self.curr_location)?;

        // 2) sort vector
        let slice = &mut self.contents[2..];
        slice.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(())
    }

    fn change_directory(&mut self) -> Result<bool, std::io::Error> {
        let next_content = &self.contents[self.curr_line as usize];
        let mut path = String::new();
        let ret = match next_content.1 {
            FileType::Directory => {
                path = format!("{}/{}", &self.curr_location, &next_content.0);
                Ok(true)
            }
            FileType::SymbolicDirectory => {
                path = format!(
                    "{}/{}",
                    &self.curr_location,
                    next_content
                        .0
                        .split_whitespace()
                        .next()
                        .unwrap()
                        .to_string()
                );
                Ok(true)
            }
            _ => Ok(false),
        };

        if let Ok(true) = ret {
            let abs_path = std::fs::canonicalize(std::path::Path::new(&path))?;
            self.curr_location = String::from(abs_path.to_str().unwrap());
            self.curr_line = 0;
            self.view_line_start = 0;
        }
        ret
    }
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
    handle_arguments(app, &args);

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

fn handle_arguments(app: &mut App, args: &Vec<String>) {
    let mut i = 0;
    let length = args.len();
    while i < length {
        let arg = &args[i];
        if arg == "--path" && i + 1 < length {
            i += 1;
            let path = &args[i];
            if let Ok(_) = std::fs::metadata(path) {
                app.curr_location =
                    String::from(std::fs::canonicalize(path).unwrap().to_str().unwrap());
            }
        }

        i += 1;
    }
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
                        app.contents = App::create_list_by_location(&app.curr_location)?;

                        // 2) display information: // TODO
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(EventHandle::None)
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
        let path = &content.0;
        //let file_type = &content.1;
        if i == curr_i - start_i as usize {
            text.push(tui::text::Spans::from(vec![tui::text::Span::styled(
                path.as_str(),
                tui::style::Style::default()
                    .bg(tui::style::Color::White)
                    .fg(tui::style::Color::Black),
            )]));
        } else {
            text.push(tui::text::Spans::from(vec![tui::text::Span::styled(
                path.as_str(),
                tui::style::Style::default()
                    .bg(tui::style::Color::Black)
                    .fg(tui::style::Color::White),
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
        .block(closure_create_block(&app.curr_location))
        .alignment(tui::layout::Alignment::Left);
    f.render_widget(contents_paragraph, chunks[0]);

    // 5) render paragraph for inforamtion
}
