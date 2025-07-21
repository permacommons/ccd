use std::env;
use std::process::{Command, exit};
use std::path::Path;
use std::io;
use std::fs;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check for interactive mode flag
    if args.len() > 1 && args[1] == "-i" {
        if let Err(e) = run_interactive_mode() {
            eprintln!("Error in interactive mode: {}", e);
            exit(1);
        }
        return;
    }
    
    // Check for increment frequency flag
    if args.len() > 2 && args[1] == "--increment" {
        let path = &args[2];
        increment_frequency(path);
        return;
    }
    
    // If no arguments provided or help requested, show help
    if args.len() == 1 || (args.len() > 1 && (args[1] == "--help" || args[1] == "-h")) {
        print_help();
        return;
    }
    
    // If search pattern provided
    if args.len() > 1 {
        let search_pattern = &args[1];
        search_and_change_directory(search_pattern);
        return;
    }
}

fn search_and_change_directory(search_pattern: &str) {
    eprintln!("Searching for directories matching: {}", search_pattern);
    
    // Run locate with limit
    let output = match Command::new("locate")
        .arg("--limit")
        .arg("100")
        .arg(search_pattern)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error running locate command: {}", e);
            eprintln!("Make sure 'locate' is installed and the database is updated (run 'sudo updatedb')");
            exit(1);
        }
    };
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<&str> = stdout.lines().collect();
    
    // locate returns exit status 1 when no matches are found, which is normal
    if !output.status.success() && !paths.is_empty() {
        eprintln!("locate command failed with status: {}", output.status);
        exit(1);
    }
    
    if paths.is_empty() {
        println!("No files or directories found matching '{}'", search_pattern);
        exit(1);
    }
    
    // Load frequency data for sorting
    let frequency_map = load_frequency_data();
    
    // Filter for directories only and create DirectoryEntry with frequency data
    let mut directories: Vec<DirectoryEntry> = paths
        .iter()
        .filter(|&&path| Path::new(path).is_dir())
        .map(|&path| {
            let count = frequency_map.get(path).unwrap_or(&0);
            DirectoryEntry {
                path: path.to_string(),
                count: *count,
            }
        })
        .collect();
    
    if directories.is_empty() {
        println!("Found {} matches in first 100 results, but none are directories:", paths.len());
        for path in paths.iter().take(5) {
            println!("  {}", path);
        }
        if paths.len() > 5 {
            println!("  ... and {} more", paths.len() - 5);
        }
        println!("Try a more specific search pattern or update the locate database with 'sudo updatedb'");
        exit(1);
    }
    
    // Sort by frequency (highest first), then by shortest path length
    directories.sort_by(|a, b| {
        b.count.cmp(&a.count).then(a.path.len().cmp(&b.path.len()))
    });
    
    let target_dir = &directories[0].path;
    
    // Verify the directory exists and is accessible
    if !Path::new(target_dir).exists() {
        eprintln!("Directory '{}' does not exist", target_dir);
        exit(1);
    }
    
    if !Path::new(target_dir).is_dir() {
        eprintln!("'{}' is not a directory", target_dir);
        exit(1);
    }
    
    // Output the directory path for shell integration
    // The shell wrapper should capture this and execute: cd "$output"
    println!("{}", target_dir);
    
    // Also provide feedback to stderr so it doesn't interfere with the path output
    let freq_info = if directories[0].count > 0 {
        format!(" (used {} times)", directories[0].count)
    } else {
        String::new()
    };
    eprintln!("Found {} directories in first 100 results, selected: {}{}", directories.len(), target_dir, freq_info);
}

#[derive(Debug, Clone)]
struct DirectoryEntry {
    path: String,
    count: u32,
}

struct App {
    input: String,
    directories: Vec<DirectoryEntry>,
    list_state: ListState,
    should_quit: bool,
    user_selected: bool,
    frequency_map: HashMap<String, u32>,
}

impl App {
    fn new() -> App {
        let frequency_map = load_frequency_data();
        App {
            input: String::new(),
            directories: Vec::new(),
            list_state: ListState::default(),
            should_quit: false,
            user_selected: false,
            frequency_map,
        }
    }

    fn search_directories(&mut self) {
        if self.input.is_empty() {
            self.directories.clear();
            self.list_state.select(None);
            return;
        }

        // Run locate command
        let output = Command::new("locate")
            .arg("--limit")
            .arg("100")
            .arg(&self.input)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let paths: Vec<&str> = stdout.lines().collect();
                
                // Filter for directories only and create DirectoryEntry with frequency data
                let mut directories: Vec<DirectoryEntry> = paths
                    .iter()
                    .filter(|&&path| Path::new(path).is_dir())
                    .map(|&path| {
                        let count = self.frequency_map.get(path).unwrap_or(&0);
                        DirectoryEntry {
                            path: path.to_string(),
                            count: *count,
                        }
                    })
                    .collect();
                
                // Sort by frequency (highest first), then by shortest path length
                directories.sort_by(|a, b| {
                    b.count.cmp(&a.count).then(a.path.len().cmp(&b.path.len()))
                });
                
                self.directories = directories;
                
                // Reset selection to first item if we have results
                if !self.directories.is_empty() {
                    self.list_state.select(Some(0));
                } else {
                    self.list_state.select(None);
                }
            }
            Err(_) => {
                self.directories.clear();
                self.list_state.select(None);
            }
        }
    }

    fn next(&mut self) {
        if self.directories.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.directories.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.directories.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.directories.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn page_up(&mut self) {
        if self.directories.is_empty() {
            return;
        }
        let page_size = 10; // Move 10 items at a time
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= page_size {
                    i - page_size
                } else {
                    0
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn page_down(&mut self) {
        if self.directories.is_empty() {
            return;
        }
        let page_size = 10; // Move 10 items at a time
        let i = match self.list_state.selected() {
            Some(i) => {
                let new_pos = i + page_size;
                if new_pos >= self.directories.len() {
                    self.directories.len() - 1
                } else {
                    new_pos
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn go_to_first(&mut self) {
        if !self.directories.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn go_to_last(&mut self) {
        if !self.directories.is_empty() {
            self.list_state.select(Some(self.directories.len() - 1));
        }
    }

    fn reset_frequency(&mut self) {
        if let Some(selected_dir) = self.get_selected_directory() {
            let path = selected_dir.clone();
            
            // Remove from frequency map and save
            self.frequency_map.remove(&path);
            save_frequency_data(&self.frequency_map);
            
            // Update the current directory entry to show count as 0
            if let Some(i) = self.list_state.selected() {
                if let Some(entry) = self.directories.get_mut(i) {
                    entry.count = 0;
                }
            }
            
            // Re-sort the directories since frequency changed
            self.directories.sort_by(|a, b| {
                b.count.cmp(&a.count).then(a.path.len().cmp(&b.path.len()))
            });
            
            // Find the new position of the directory we just reset
            let new_index = self.directories.iter().position(|entry| entry.path == path);
            if let Some(new_index) = new_index {
                self.list_state.select(Some(new_index));
            }
        }
    }

    fn get_selected_directory(&self) -> Option<&String> {
        if let Some(i) = self.list_state.selected() {
            self.directories.get(i).map(|entry| &entry.path)
        } else {
            None
        }
    }
}

fn run_interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
        exit(1);
    } else if app.user_selected {
        if let Some(selected_dir) = app.get_selected_directory() {
            // Increment frequency count for the selected directory
            increment_frequency(selected_dir);
            
            // Output the selected directory to file descriptor 3 if available, otherwise stdout
            if let Ok(mut fd3) = std::fs::OpenOptions::new().write(true).open("/proc/self/fd/3") {
                let _ = writeln!(fd3, "{}", selected_dir);
            } else {
                println!("{}", selected_dir);
            }
        }
    } else {
        // User quit without selecting - exit with code 1
        exit(1);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Enter => {
                        if app.get_selected_directory().is_some() {
                            app.user_selected = true;
                            app.should_quit = true;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.search_directories();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.search_directories();
                    }
                    KeyCode::Down => {
                        app.next();
                    }
                    KeyCode::Up => {
                        app.previous();
                    }
                    KeyCode::PageUp => {
                        app.page_up();
                    }
                    KeyCode::PageDown => {
                        app.page_down();
                    }
                    KeyCode::Home => {
                        app.go_to_first();
                    }
                    KeyCode::End => {
                        app.go_to_last();
                    }
                    KeyCode::Delete => {
                        app.reset_frequency();
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Input box
            Constraint::Min(0),    // Results list
            Constraint::Length(3), // Help text
        ])
        .split(f.area());

    // Input box
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search Pattern"));
    f.render_widget(input, chunks[0]);

    // Results list
    let items: Vec<ListItem> = app
        .directories
        .iter()
        .map(|dir| {
            if dir.count > 0 {
                // Use spans to create visually distinct formatting
                let content = Line::from(vec![
                    Span::raw(&dir.path),
                    Span::styled(
                        format!(" [{}]", dir.count),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    ),
                ]);
                ListItem::new(content)
            } else {
                let content = Line::from(Span::raw(&dir.path));
                ListItem::new(content)
            }
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Directories ({} found)",
            app.directories.len()
        )))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, chunks[1], &mut app.list_state.clone());

    // Help text
    let help = Paragraph::new("↑/↓: Navigate | PgUp/PgDn: Page | Home/End: First/Last | Del: Reset Count | Enter: Select | q/Esc: Quit")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[2]);

    // Set cursor position in input box
    f.set_cursor_position((chunks[0].x + app.input.len() as u16 + 1, chunks[0].y + 1));
}

fn get_frequency_file_path() -> std::path::PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    Path::new(&home).join(".cdd_frequency")
}

fn load_frequency_data() -> HashMap<String, u32> {
    let mut frequency_map = HashMap::new();
    let freq_file = get_frequency_file_path();
    
    if let Ok(file) = fs::File::open(&freq_file) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.len() == 2 {
                    if let Ok(count) = parts[0].parse::<u32>() {
                        frequency_map.insert(parts[1].to_string(), count);
                    }
                }
            }
        }
    }
    
    frequency_map
}

fn save_frequency_data(frequency_map: &HashMap<String, u32>) {
    let freq_file = get_frequency_file_path();
    if let Ok(mut file) = fs::File::create(&freq_file) {
        for (path, count) in frequency_map {
            let _ = writeln!(file, "{}\t{}", count, path);
        }
    }
}

fn increment_frequency(path: &str) {
    let mut frequency_map = load_frequency_data();
    let count = frequency_map.get(path).unwrap_or(&0) + 1;
    frequency_map.insert(path.to_string(), count);
    save_frequency_data(&frequency_map);
}

fn print_help() {
    println!("cdd - Change Directory Directory");
    println!();
    println!("USAGE:");
    println!("    cdd                    Enter interactive mode");
    println!("    cdd <search_pattern>   Search for directories matching pattern (case-insensitive)");
    println!();
    println!("DESCRIPTION:");
    println!("    Uses the locate database to quickly look up directories to cd into.");
    println!("    Remembers most frequently used directories for faster access.");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help    Show this help message");
    println!("    -i            Interactive mode (used internally by shell wrapper)");
    println!();
    println!("EXAMPLES:");
    println!("    cdd           # Enter interactive mode");
    println!("    cdd proj      # Find directories containing 'proj'");
    println!("    cdd docs      # Find directories containing 'docs'");
    println!();
    println!("INTERACTIVE MODE:");
    println!("    Type to search, use ↑/↓ to navigate, PgUp/PgDn for fast navigation");
    println!("    Home/End to jump to first/last, Del to reset frequency count");
    println!("    Enter to select, q/Esc to quit");
    println!("    Directories are sorted by usage frequency (most used first)");
}
