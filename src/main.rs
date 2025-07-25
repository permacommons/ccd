use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

// Constants
const LOCATE_LIMIT: &str = "100";
const PAGE_SIZE: usize = 10;
const FREQUENCY_FILE_NAME: &str = ".ccd_frequency";

// View modes for the interactive interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Search,
    Frequent,
}

// Custom error types
#[derive(Debug)]
enum CddError {
    LocateCommand(String),
    NoDirectoriesFound,
    DirectoryNotFound(String),
    IoError(io::Error),
}

impl fmt::Display for CddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CddError::LocateCommand(msg) => write!(f, "Locate command error: {msg}"),
            CddError::NoDirectoriesFound => write!(f, "No directories found"),
            CddError::DirectoryNotFound(path) => write!(f, "Directory not found: {path}"),
            CddError::IoError(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl Error for CddError {}

impl From<io::Error> for CddError {
    fn from(err: io::Error) -> Self {
        CddError::IoError(err)
    }
}

// Data structures
#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectoryEntry {
    path: String,
    count: u32,
}

impl DirectoryEntry {
    fn new(path: String, count: u32) -> Self {
        Self { path, count }
    }
}

#[derive(Debug, Clone)]
struct SearchResult {
    directories: Vec<DirectoryEntry>,
    files_filtered: usize,
}

impl SearchResult {
    fn new(directories: Vec<DirectoryEntry>, files_filtered: usize) -> Self {
        Self {
            directories,
            files_filtered,
        }
    }
}

struct App {
    input: String,
    directories: Vec<DirectoryEntry>,
    list_state: ListState,
    should_quit: bool,
    user_selected: bool,
    frequency_map: HashMap<String, u32>,
    view_mode: ViewMode,
    files_filtered: usize,
}

impl App {
    fn new() -> Result<Self, CddError> {
        let frequency_map = FrequencyManager::load()?;
        Ok(Self {
            input: String::new(),
            directories: Vec::new(),
            list_state: ListState::default(),
            should_quit: false,
            user_selected: false,
            frequency_map,
            view_mode: ViewMode::Search,
            files_filtered: 0,
        })
    }

    fn search_directories(&mut self) -> Result<(), CddError> {
        if self.input.is_empty() {
            self.directories.clear();
            self.files_filtered = 0;
            self.list_state.select(None);
            return Ok(());
        }

        // Search and handle the case where no results are found
        match DirectorySearcher::search(&self.input, &self.frequency_map) {
            Ok(search_result) => {
                self.directories = search_result.directories;
                self.files_filtered = search_result.files_filtered;
            }
            Err(CddError::NoDirectoriesFound) => {
                // Clear results when no directories are found
                self.directories.clear();
                self.files_filtered = 0;
            }
            Err(e) => return Err(e),
        }

        // Reset selection to first item if we have results
        if !self.directories.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }

        Ok(())
    }

    fn navigate(&mut self, direction: NavigationDirection) {
        if self.directories.is_empty() {
            return;
        }

        let new_index = match direction {
            NavigationDirection::Next => self.calculate_next_index(),
            NavigationDirection::Previous => self.calculate_previous_index(),
            NavigationDirection::PageUp => self.calculate_page_up_index(),
            NavigationDirection::PageDown => self.calculate_page_down_index(),
            NavigationDirection::First => 0,
            NavigationDirection::Last => self.directories.len() - 1,
        };

        self.list_state.select(Some(new_index));
    }

    fn calculate_next_index(&self) -> usize {
        match self.list_state.selected() {
            Some(i) if i >= self.directories.len() - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        }
    }

    fn calculate_previous_index(&self) -> usize {
        match self.list_state.selected() {
            Some(0) => self.directories.len() - 1,
            Some(i) => i - 1,
            None => 0,
        }
    }

    fn calculate_page_up_index(&self) -> usize {
        match self.list_state.selected() {
            Some(i) if i >= PAGE_SIZE => i - PAGE_SIZE,
            _ => 0,
        }
    }

    fn calculate_page_down_index(&self) -> usize {
        match self.list_state.selected() {
            Some(i) => {
                let new_pos = i + PAGE_SIZE;
                if new_pos >= self.directories.len() {
                    self.directories.len() - 1
                } else {
                    new_pos
                }
            }
            None => 0,
        }
    }

    fn reset_frequency(&mut self) -> Result<(), CddError> {
        if let Some(selected_dir) = self.get_selected_directory() {
            let path = selected_dir.clone();
            let selected_index = self.list_state.selected().unwrap_or(0);

            // Remove from frequency map and save
            self.frequency_map.remove(&path);
            FrequencyManager::save(&self.frequency_map)?;

            match self.view_mode {
                ViewMode::Frequent => {
                    // In frequent mode, remove the directory from the list entirely
                    self.directories.remove(selected_index);

                    // Adjust selection after removal
                    if self.directories.is_empty() {
                        self.list_state.select(None);
                    } else if selected_index >= self.directories.len() {
                        // If we removed the last item, select the new last item
                        self.list_state.select(Some(self.directories.len() - 1));
                    } else {
                        // Keep the same index, which now points to the next item
                        self.list_state.select(Some(selected_index));
                    }
                }
                ViewMode::Search => {
                    // In search mode, update the entry to show count as 0
                    if let Some(entry) = self.directories.get_mut(selected_index) {
                        entry.count = 0;
                    }

                    // Re-sort the directories since frequency changed
                    DirectorySearcher::sort_directories(&mut self.directories);

                    // Find the new position of the directory we just reset
                    if let Some(new_index) =
                        self.directories.iter().position(|entry| entry.path == path)
                    {
                        self.list_state.select(Some(new_index));
                    }
                }
            }
        }
        Ok(())
    }

    fn get_selected_directory(&self) -> Option<&String> {
        self.list_state
            .selected()
            .and_then(|i| self.directories.get(i))
            .map(|entry| &entry.path)
    }

    fn toggle_view_mode(&mut self) {
        match self.view_mode {
            ViewMode::Search => {
                self.view_mode = ViewMode::Frequent;
                self.show_frequent_directories();
            }
            ViewMode::Frequent => {
                self.view_mode = ViewMode::Search;
                // Return to search mode - if there's input, search, otherwise clear
                if !self.input.is_empty() {
                    let _ = self.search_directories();
                } else {
                    self.directories.clear();
                    self.list_state.select(None);
                }
            }
        }
    }

    fn show_frequent_directories(&mut self) {
        // Get all directories with frequency > 0, sorted by frequency
        let mut frequent_dirs: Vec<DirectoryEntry> = self
            .frequency_map
            .iter()
            .filter(|(path, count)| **count > 0 && Path::new(path).is_dir())
            .map(|(path, count)| DirectoryEntry::new(path.clone(), *count))
            .collect();

        // Sort by frequency (descending), then by path length (ascending)
        frequent_dirs.sort_by(|a, b| b.count.cmp(&a.count).then(a.path.len().cmp(&b.path.len())));

        // Apply search filter if there's input
        if !self.input.is_empty() {
            frequent_dirs.retain(|entry| {
                entry
                    .path
                    .to_lowercase()
                    .contains(&self.input.to_lowercase())
            });
        }

        self.directories = frequent_dirs;

        // Reset selection to first item if we have results
        if !self.directories.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    fn handle_character_input(&mut self, c: char) {
        self.input.push(c);

        // Apply search to current view mode
        match self.view_mode {
            ViewMode::Search => {
                let _ = self.search_directories();
            }
            ViewMode::Frequent => {
                self.show_frequent_directories();
            }
        }
    }

    fn handle_backspace(&mut self) {
        self.input.pop();

        // Apply search to current view mode
        match self.view_mode {
            ViewMode::Search => {
                let _ = self.search_directories();
            }
            ViewMode::Frequent => {
                self.show_frequent_directories();
            }
        }
    }
}

// Navigation enum for better type safety
#[derive(Debug, Clone, Copy)]
enum NavigationDirection {
    Next,
    Previous,
    PageUp,
    PageDown,
    First,
    Last,
}

// Frequency management module
struct FrequencyManager;

impl FrequencyManager {
    fn get_file_path() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Path::new(&home).join(FREQUENCY_FILE_NAME)
    }

    fn load() -> Result<HashMap<String, u32>, CddError> {
        let mut frequency_map = HashMap::new();
        let freq_file = Self::get_file_path();

        if let Ok(file) = fs::File::open(&freq_file) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if let Some((count_str, path)) = line.split_once('\t') {
                    if let Ok(count) = count_str.parse::<u32>() {
                        frequency_map.insert(path.to_string(), count);
                    }
                }
            }
        }

        Ok(frequency_map)
    }

    fn save(frequency_map: &HashMap<String, u32>) -> Result<(), CddError> {
        let freq_file = Self::get_file_path();
        let mut file = fs::File::create(&freq_file)?;

        for (path, count) in frequency_map {
            writeln!(file, "{count}\t{path}")?;
        }

        Ok(())
    }

    fn increment(path: &str) -> Result<(), CddError> {
        let mut frequency_map = Self::load()?;
        let count = frequency_map.get(path).unwrap_or(&0) + 1;
        frequency_map.insert(path.to_string(), count);
        Self::save(&frequency_map)
    }
}

// Directory search module
struct DirectorySearcher;

impl DirectorySearcher {
    fn search(
        pattern: &str,
        frequency_map: &HashMap<String, u32>,
    ) -> Result<SearchResult, CddError> {
        // Use a HashSet to deduplicate paths from both sources
        let mut unique_paths = std::collections::HashSet::new();
        let mut files_filtered = 0;

        // First, search using locate
        let output = Command::new("locate")
            .arg("--limit")
            .arg(LOCATE_LIMIT)
            .arg(pattern)
            .output()
            .map_err(|e| CddError::LocateCommand(format!("Failed to execute locate: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let locate_paths: Vec<&str> = stdout.lines().collect();

        // Count files and add directories to our set
        for path in locate_paths {
            if Path::new(path).is_dir() {
                unique_paths.insert(path.to_string());
            } else {
                files_filtered += 1;
            }
        }

        // Second, search through frequency map for paths that match the pattern
        let pattern_lower = pattern.to_lowercase();
        for path in frequency_map.keys() {
            if path.to_lowercase().contains(&pattern_lower) && Path::new(path).is_dir() {
                unique_paths.insert(path.clone());
            }
        }

        // If no directories found from either source, return error
        if unique_paths.is_empty() {
            return Err(CddError::NoDirectoriesFound);
        }

        // Convert to DirectoryEntry with frequency data
        let mut directories: Vec<DirectoryEntry> = unique_paths
            .into_iter()
            .map(|path| {
                let count = frequency_map.get(&path).unwrap_or(&0);
                DirectoryEntry::new(path, *count)
            })
            .collect();

        Self::sort_directories(&mut directories);
        Ok(SearchResult::new(directories, files_filtered))
    }

    fn sort_directories(directories: &mut [DirectoryEntry]) {
        directories.sort_by(|a, b| b.count.cmp(&a.count).then(a.path.len().cmp(&b.path.len())));
    }
}

// Main application logic
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => print_help(),
        2 => match args[1].as_str() {
            "-i" => run_interactive_mode()?,
            "-b" | "--bookmark" => bookmark_current_directory()?,
            "--help" | "-h" => print_help(),
            pattern => search_and_change_directory(pattern)?,
        },
        3 if args[1] == "--increment" => {
            FrequencyManager::increment(&args[2])?;
        }
        _ => {
            let pattern = &args[1];
            search_and_change_directory(pattern)?;
        }
    }

    Ok(())
}

fn bookmark_current_directory() -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()
        .map_err(CddError::IoError)?
        .to_string_lossy()
        .to_string();

    let mut frequency_map = FrequencyManager::load()?;

    // Only add if not already in the frequency map
    if !frequency_map.contains_key(&current_dir) {
        frequency_map.insert(current_dir.clone(), 1);
        FrequencyManager::save(&frequency_map)?;
        eprintln!("Bookmarked: {current_dir}");
    } else {
        eprintln!("Directory already bookmarked: {current_dir}");
    }

    Ok(())
}

fn search_and_change_directory(search_pattern: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("Searching for directories matching: {search_pattern}");

    let frequency_map = FrequencyManager::load()?;
    let search_result =
        DirectorySearcher::search(search_pattern, &frequency_map).map_err(|e| match e {
            CddError::NoDirectoriesFound => {
                eprintln!("No directories found matching '{search_pattern}'");
                exit(1);
            }
            other => other,
        })?;

    let target_dir = &search_result.directories[0].path;

    // Verify the directory exists and is accessible
    if !Path::new(target_dir).exists() {
        return Err(CddError::DirectoryNotFound(target_dir.clone()).into());
    }

    if !Path::new(target_dir).is_dir() {
        return Err(CddError::DirectoryNotFound(target_dir.clone()).into());
    }

    // Output the directory path for shell integration
    println!("{target_dir}");

    // Provide feedback to stderr
    let freq_info = if search_result.directories[0].count > 0 {
        format!(" (used {} times)", search_result.directories[0].count)
    } else {
        String::new()
    };

    let files_info = if search_result.files_filtered > 0 {
        format!("; {} matching files not shown", search_result.files_filtered)
    } else {
        String::new()
    };

    eprintln!(
        "Found {} directories in first {} results{}, selected: {}{}",
        search_result.directories.len(),
        LOCATE_LIMIT,
        files_info,
        target_dir,
        freq_info
    );

    Ok(())
}

fn run_interactive_mode() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Err(err) => {
            eprintln!("{err:?}");
            exit(1);
        }
        Ok(()) if app.user_selected => {
            if let Some(selected_dir) = app.get_selected_directory() {
                // Increment frequency count for the selected directory
                FrequencyManager::increment(selected_dir)?;

                // Output the selected directory to file descriptor 3 if available, otherwise stdout
                if let Ok(mut fd3) = fs::OpenOptions::new().write(true).open("/proc/self/fd/3") {
                    writeln!(fd3, "{selected_dir}")?;
                } else {
                    println!("{selected_dir}");
                }
            }
        }
        Ok(()) => {
            // User quit without selecting - exit with code 1
            exit(1);
        }
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
                    KeyCode::Tab => {
                        app.toggle_view_mode();
                    }
                    KeyCode::Char(c) => {
                        app.handle_character_input(c);
                    }
                    KeyCode::Backspace => {
                        app.handle_backspace();
                    }
                    KeyCode::Down => app.navigate(NavigationDirection::Next),
                    KeyCode::Up => app.navigate(NavigationDirection::Previous),
                    KeyCode::PageUp => app.navigate(NavigationDirection::PageUp),
                    KeyCode::PageDown => app.navigate(NavigationDirection::PageDown),
                    KeyCode::Home => app.navigate(NavigationDirection::First),
                    KeyCode::End => app.navigate(NavigationDirection::Last),
                    KeyCode::Delete if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        let _ = app.reset_frequency(); // Ignore errors in interactive mode
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

    render_input_box(f, app, chunks[0]);
    render_results_list(f, app, chunks[1]);
    render_help_text(f, chunks[2]);

    // Set cursor position in input box
    f.set_cursor_position((chunks[0].x + app.input.len() as u16 + 1, chunks[0].y + 1));
}

fn render_input_box(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (input_text, input_style) = if app.input.is_empty() {
        let placeholder = match app.view_mode {
            ViewMode::Search => "Start typing or press [Tab] to see frequent choices",
            ViewMode::Frequent => {
                "Search the list below, or press [Tab] to search across all directories"
            }
        };
        (placeholder, Style::default().fg(Color::DarkGray))
    } else {
        (app.input.as_str(), Style::default().fg(Color::Yellow))
    };

    let title = match app.view_mode {
        ViewMode::Search => "Search All Directories",
        ViewMode::Frequent => "Search Frequently Used",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, Style::default().fg(Color::Gray)))
        .border_style(Style::default().fg(Color::Gray));

    let input = Paragraph::new(input_text).style(input_style).block(block);
    f.render_widget(input, area);
}

fn render_results_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = if app.directories.is_empty() && app.view_mode == ViewMode::Frequent
    {
        vec![ListItem::new(Line::from(Span::styled(
            "No frequently used directories found",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )))]
    } else {
        app.directories
            .iter()
            .map(|dir| create_list_item(dir))
            .collect()
    };

    let title = match app.view_mode {
        ViewMode::Search => {
            if app.files_filtered > 0 {
                format!(
                    "Search Results ({} found; {} matching files not shown)",
                    app.directories.len(),
                    app.files_filtered
                )
            } else {
                format!("Search Results ({} found)", app.directories.len())
            }
        }
        ViewMode::Frequent => {
            if app.directories.is_empty() {
                "Frequent Directories (none)".to_string()
            } else {
                format!("Frequent Directories ({} found)", app.directories.len())
            }
        }
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.list_state.clone());
}

fn create_list_item(dir: &DirectoryEntry) -> ListItem {
    if dir.count > 0 {
        let content = Line::from(vec![
            Span::raw(&dir.path),
            Span::styled(
                format!(" [{}]", dir.count),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        ListItem::new(content)
    } else {
        ListItem::new(Line::from(Span::raw(&dir.path)))
    }
}

fn render_help_text(f: &mut Frame, area: ratatui::layout::Rect) {
    let help = Paragraph::new("↑/↓: Navigate | Home/End: First/Last | Shift+Del: Reset Count | Enter: Select | q/Esc: Quit")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, area);
}

fn print_help() {
    println!("ccd-pick - Change Change Directory Picker");
    println!();
    println!("USAGE:");
    println!("    ccd-pick -i                   Enter interactive mode");
    println!("    ccd-pick -b                   Bookmark current directory");
    println!("    ccd-pick <search_pattern>     Search for directories matching pattern");
    println!();
    println!("DESCRIPTION:");
    println!("    Uses the locate database to quickly look up directories to cd into.");
    println!("    Remembers most frequently used directories for faster access.");
    println!("    Usually invoked via the ccd wrapper function.");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Show this help message");
    println!("    -i               Interactive mode (used internally by shell wrapper)");
    println!("    -b, --bookmark   Add current directory to bookmarks with frequency 1");
    println!();
    println!("EXAMPLES:");
    println!("    ccd-pick -i      # Enter interactive mode");
    println!("    ccd-pick -b      # Bookmark current directory");
    println!("    ccd-pick proj    # Find directories containing 'proj'");
    println!("    ccd-pick Docs    # Find directories containing 'Docs'");
    println!();
    println!("INTERACTIVE MODE:");
    println!("    Type to search, use ↑/↓ to navigate, PgUp/PgDn for fast navigation");
    println!("    Home/End to jump to first/last, Tab to toggle frequent/search view");
    println!("    Shift+Del to reset frequency count, Enter to select, q/Esc to quit");
    println!("    Directories are sorted by usage frequency (most used first)");
}
