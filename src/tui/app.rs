use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::commands::DuckDbInspector;

use super::views;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Home,
    FileBrowser,
    DataInspector,
    JsonInspector,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorTab {
    Schema,
    Preview,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonInspectorTab {
    Tree,
    Raw,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeoJsonTab {
    Summary,
    Features,
    Tree,
}

pub const FILTER_OPERATORS: &[&str] = &[
    "=", "!=", ">", "<", ">=", "<=", "LIKE", "IS NULL", "IS NOT NULL",
];

#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub column: String,
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterField {
    Column,
    Operator,
    Value,
}

#[derive(Debug, Clone)]
pub struct FilterEditorState {
    pub conditions: Vec<FilterCondition>,
    pub column_idx: usize,
    pub operator_idx: usize,
    pub value_input: String,
    pub active_field: FilterField,
}

#[derive(Debug, Clone)]
pub enum Popup {
    None,
    ConvertConfirm { target_format: String },
    Message { title: String, body: String },
    FilterEditor(FilterEditorState),
}

#[derive(Debug)]
pub enum Message {
    Quit,
    NavigateUp,
    NavigateDown,
    Enter,
    Back,
    SwitchTab,
    ScrollUp,
    ScrollDown,
    ConvertFile,
    ConfirmConvert,
    ClosePopup,
    ToggleTreeNode,
    SwitchGeoTab,
    Noop,
    NextPage,
    PrevPage,
    OpenFilterPopup,
    FilterTabNext,
    FilterNavUp,
    FilterNavDown,
    FilterChar(char),
    FilterBackspace,
    FilterAddCondition,
    FilterRemoveLast,
    FilterApply,
}

pub struct DirEntryInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

pub struct App {
    pub should_quit: bool,
    pub current_screen: Screen,
    // Home
    pub home_selected: usize,
    // File browser
    pub current_dir: PathBuf,
    pub dir_entries: Vec<DirEntryInfo>,
    pub browser_selected: usize,
    // Data inspector
    pub inspector_file: Option<PathBuf>,
    pub inspector_tab: InspectorTab,
    pub inspector_schema: Vec<(String, String)>,
    pub inspector_null_counts: Vec<usize>,
    pub inspector_mean_values: Vec<String>,
    pub inspector_min_values: Vec<String>,
    pub inspector_max_values: Vec<String>,
    pub inspector_preview_headers: Vec<String>,
    pub inspector_preview_data: Vec<Vec<String>>,
    pub inspector_row_count: usize,
    pub inspector_scroll: usize,
    pub inspector_page: usize,
    pub inspector_filters: Vec<FilterCondition>,
    // Popup
    pub popup: Popup,
    // Json inspector
    pub json_file: Option<PathBuf>,
    pub json_root: Option<serde_json::Value>,
    pub json_kind: Option<crate::commands::json_inspector::FileKind>,
    pub json_tab: JsonInspectorTab,
    pub geo_tab: GeoJsonTab,
    pub json_scroll: usize,
    pub json_tree_nodes: Vec<(String, crate::tui::tree::TreeNode)>,
    pub json_collapsed: std::collections::HashSet<String>,
    pub json_features_headers: Vec<String>,
    pub json_features_data: Vec<Vec<String>>,
    pub json_geosummary: Option<(usize, Vec<String>, Option<(f64, f64, f64, f64)>)>,
    pub json_raw: String,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let mut app = Self {
            should_quit: false,
            current_screen: Screen::Home,
            home_selected: 0,
            current_dir: std::env::current_dir()?,
            dir_entries: Vec::new(),
            browser_selected: 0,
            inspector_file: None,
            inspector_tab: InspectorTab::Schema,
            inspector_schema: Vec::new(),
            inspector_null_counts: Vec::new(),
            inspector_mean_values: Vec::new(),
            inspector_min_values: Vec::new(),
            inspector_max_values: Vec::new(),
            inspector_preview_headers: Vec::new(),
            inspector_preview_data: Vec::new(),
            inspector_row_count: 0,
            inspector_scroll: 0,
            inspector_page: 0,
            inspector_filters: Vec::new(),
            popup: Popup::None,
            json_file: None,
            json_root: None,
            json_kind: None,
            json_tab: JsonInspectorTab::Tree,
            geo_tab: GeoJsonTab::Summary,
            json_scroll: 0,
            json_tree_nodes: Vec::new(),
            json_collapsed: std::collections::HashSet::new(),
            json_features_headers: Vec::new(),
            json_features_data: Vec::new(),
            json_geosummary: None,
            json_raw: String::new(),
        };

        if let Some(p) = path {
            let p = std::fs::canonicalize(&p).unwrap_or(p);
            if p.is_dir() {
                app.current_dir = p;
                app.load_dir_entries()?;
                app.current_screen = Screen::FileBrowser;
            } else {
                match p.extension().and_then(|e| e.to_str()) {
                    Some("csv") | Some("parquet") => {
                        // Set file browser dir to parent for Back navigation
                        if let Some(parent) = p.parent() {
                            app.current_dir = parent.to_path_buf();
                            app.load_dir_entries()?;
                        }
                        app.inspector_file = Some(p.clone());
                        app.load_inspector_data(&p)?;
                        app.current_screen = Screen::DataInspector;
                    }
                    Some("json") | Some("geojson") => {
                        if let Some(parent) = p.parent() {
                            app.current_dir = parent.to_path_buf();
                            app.load_dir_entries()?;
                        }
                        app.load_json_data(&p)?;
                        app.current_screen = Screen::JsonInspector;
                    }
                    _ => {
                        // Unknown file type - open browser in parent dir
                        if let Some(parent) = p.parent() {
                            app.current_dir = parent.to_path_buf();
                        }
                        app.load_dir_entries()?;
                        app.current_screen = Screen::FileBrowser;
                    }
                }
            }
        }

        Ok(app)
    }

    pub fn handle_event(&self, event: Event) -> Message {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key),
            _ => Message::Noop,
        }
    }

    fn handle_key(&self, key: crossterm::event::KeyEvent) -> Message {
        // Popup handling takes priority
        match &self.popup {
            Popup::ConvertConfirm { .. } => {
                return match key.code {
                    KeyCode::Enter => Message::ConfirmConvert,
                    KeyCode::Esc => Message::ClosePopup,
                    _ => Message::Noop,
                };
            }
            Popup::Message { .. } => {
                return match key.code {
                    KeyCode::Enter | KeyCode::Esc => Message::ClosePopup,
                    _ => Message::Noop,
                };
            }
            Popup::FilterEditor(state) => {
                return match key.code {
                    KeyCode::Esc => Message::ClosePopup,
                    KeyCode::Tab => Message::FilterTabNext,
                    KeyCode::Up => Message::FilterNavUp,
                    KeyCode::Down => Message::FilterNavDown,
                    KeyCode::Backspace => Message::FilterBackspace,
                    KeyCode::Enter => {
                        if state.active_field == FilterField::Value {
                            if state.value_input.is_empty() {
                                Message::FilterApply
                            } else {
                                Message::FilterAddCondition
                            }
                        } else {
                            Message::FilterTabNext
                        }
                    }
                    KeyCode::Char('d') if state.active_field != FilterField::Value => {
                        Message::FilterRemoveLast
                    }
                    KeyCode::Char(c) => {
                        if state.active_field == FilterField::Value {
                            Message::FilterChar(c)
                        } else {
                            Message::Noop
                        }
                    }
                    _ => Message::Noop,
                };
            }
            Popup::None => {}
        }

        // Global quit
        if key.code == KeyCode::Char('q') {
            return Message::Quit;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Message::Quit;
        }

        // Screen-specific
        match self.current_screen {
            Screen::Home => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Message::NavigateUp,
                KeyCode::Down | KeyCode::Char('j') => Message::NavigateDown,
                KeyCode::Enter => Message::Enter,
                _ => Message::Noop,
            },
            Screen::FileBrowser => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Message::NavigateUp,
                KeyCode::Down | KeyCode::Char('j') => Message::NavigateDown,
                KeyCode::Enter => Message::Enter,
                KeyCode::Esc => Message::Back,
                _ => Message::Noop,
            },
            Screen::DataInspector => match key.code {
                KeyCode::Tab => Message::SwitchTab,
                KeyCode::Up | KeyCode::Char('k') => Message::ScrollUp,
                KeyCode::Down | KeyCode::Char('j') => Message::ScrollDown,
                KeyCode::Char('c') => Message::ConvertFile,
                KeyCode::Char('f') => Message::OpenFilterPopup,
                KeyCode::Esc => Message::Back,
                KeyCode::Right => Message::NextPage,
                KeyCode::Left => Message::PrevPage,
                _ => Message::Noop,
            },
            Screen::JsonInspector => match key.code {
                KeyCode::Tab => {
                    if self.json_kind == Some(crate::commands::json_inspector::FileKind::GeoJson) {
                        Message::SwitchGeoTab
                    } else {
                        Message::SwitchTab
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => Message::ScrollUp,
                KeyCode::Down | KeyCode::Char('j') => Message::ScrollDown,
                KeyCode::Enter => Message::ToggleTreeNode,
                KeyCode::Esc => Message::Back,
                _ => Message::Noop,
            },
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Quit => self.should_quit = true,
            Message::NavigateUp => self.navigate_up(),
            Message::NavigateDown => self.navigate_down(),
            Message::Enter => self.enter(),
            Message::Back => self.back(),
            Message::SwitchTab => self.switch_tab(),
            Message::ScrollUp => self.scroll_up(),
            Message::ScrollDown => self.scroll_down(),
            Message::ConvertFile => self.convert_file(),
            Message::ConfirmConvert => self.confirm_convert(),
            Message::ClosePopup => self.popup = Popup::None,
            Message::ToggleTreeNode => self.toggle_tree_node(),
            Message::SwitchGeoTab => self.switch_geo_tab(),
            Message::NextPage => self.next_page(),
            Message::PrevPage => self.prev_page(),
            Message::OpenFilterPopup => self.open_filter_popup(),
            Message::FilterTabNext => self.filter_tab_next(),
            Message::FilterNavUp => self.filter_nav_up(),
            Message::FilterNavDown => self.filter_nav_down(),
            Message::FilterChar(c) => self.filter_char(c),
            Message::FilterBackspace => self.filter_backspace(),
            Message::FilterAddCondition => self.filter_add_condition(),
            Message::FilterRemoveLast => self.filter_remove_last(),
            Message::FilterApply => self.filter_apply(),
            Message::Noop => {}
        }
    }

    fn navigate_up(&mut self) {
        match self.current_screen {
            Screen::Home => {
                if self.home_selected > 0 {
                    self.home_selected -= 1;
                }
            }
            Screen::FileBrowser => {
                if self.browser_selected > 0 {
                    self.browser_selected -= 1;
                }
            }
            _ => {}
        }
    }

    fn navigate_down(&mut self) {
        match self.current_screen {
            Screen::Home => {
                if self.home_selected < 1 {
                    self.home_selected += 1;
                }
            }
            Screen::FileBrowser => {
                if self.browser_selected + 1 < self.dir_entries.len() {
                    self.browser_selected += 1;
                }
            }
            _ => {}
        }
    }

    fn enter(&mut self) {
        match self.current_screen {
            Screen::Home => {
                // Both options go to file browser
                if let Err(e) = self.load_dir_entries() {
                    self.popup = Popup::Message {
                        title: "Error".to_string(),
                        body: e.to_string(),
                    };
                    return;
                }
                self.current_screen = Screen::FileBrowser;
            }
            Screen::FileBrowser => {
                let entry_path;
                let entry_is_dir;
                if let Some(entry) = self.dir_entries.get(self.browser_selected) {
                    entry_path = entry.path.clone();
                    entry_is_dir = entry.is_dir;
                } else {
                    return;
                }

                if entry_is_dir {
                    self.current_dir = entry_path;
                    self.browser_selected = 0;
                    if let Err(e) = self.load_dir_entries() {
                        self.popup = Popup::Message {
                            title: "Error".to_string(),
                            body: e.to_string(),
                        };
                    }
                } else {
                    // Check if data file
                    match entry_path.extension().and_then(|e| e.to_str()) {
                        Some("csv") | Some("parquet") => {
                            self.inspector_file = Some(entry_path.clone());
                            match self.load_inspector_data(&entry_path) {
                                Ok(()) => self.current_screen = Screen::DataInspector,
                                Err(e) => {
                                    self.popup = Popup::Message {
                                        title: "Error".to_string(),
                                        body: e.to_string(),
                                    };
                                }
                            }
                        }
                        Some("json") | Some("geojson") => match self.load_json_data(&entry_path) {
                            Ok(()) => self.current_screen = Screen::JsonInspector,
                            Err(e) => {
                                self.popup = Popup::Message {
                                    title: "Error".to_string(),
                                    body: e.to_string(),
                                };
                            }
                        },
                        _ => {} // Can't open non-data files
                    }
                }
            }
            Screen::DataInspector => {}
            Screen::JsonInspector => {}
        }
    }

    fn back(&mut self) {
        match self.current_screen {
            Screen::JsonInspector => {
                self.current_screen = Screen::FileBrowser;
            }
            Screen::DataInspector => {
                // Go back to file browser
                if self.dir_entries.is_empty() {
                    if let Some(ref file) = self.inspector_file {
                        if let Some(parent) = file.parent() {
                            self.current_dir = parent.to_path_buf();
                            let _ = self.load_dir_entries();
                        }
                    }
                }
                self.current_screen = Screen::FileBrowser;
            }
            Screen::FileBrowser => {
                self.current_screen = Screen::Home;
            }
            Screen::Home => {}
        }
    }

    fn switch_tab(&mut self) {
        match self.current_screen {
            Screen::JsonInspector => {
                self.json_scroll = 0;
                self.json_tab = match self.json_tab {
                    JsonInspectorTab::Tree => JsonInspectorTab::Raw,
                    JsonInspectorTab::Raw => JsonInspectorTab::Tree,
                };
            }
            _ => {
                self.inspector_scroll = 0;
                self.inspector_tab = match self.inspector_tab {
                    InspectorTab::Schema => InspectorTab::Preview,
                    InspectorTab::Preview => InspectorTab::Schema,
                };
            }
        }
    }

    fn scroll_up(&mut self) {
        match self.current_screen {
            Screen::JsonInspector => {
                if self.json_scroll > 0 {
                    self.json_scroll -= 1;
                }
            }
            _ => {
                if self.inspector_scroll > 0 {
                    self.inspector_scroll -= 1;
                }
            }
        }
    }

    fn scroll_down(&mut self) {
        match self.current_screen {
            Screen::JsonInspector => {
                let max = match self.geo_tab {
                    GeoJsonTab::Features => self.json_features_data.len(),
                    _ => self.json_tree_nodes.len(),
                };
                if self.json_scroll + 1 < max {
                    self.json_scroll += 1;
                }
            }
            _ => {
                let max = match self.inspector_tab {
                    InspectorTab::Schema => self.inspector_schema.len(),
                    InspectorTab::Preview => self.inspector_preview_data.len(),
                };
                if self.inspector_scroll + 1 < max {
                    self.inspector_scroll += 1;
                }
            }
        }
    }

    fn next_page(&mut self) {
        if self.inspector_tab != InspectorTab::Preview {
            return;
        }
        const PAGE_SIZE: usize = 50;
        let total_pages = (self.inspector_row_count + PAGE_SIZE - 1) / PAGE_SIZE;
        if self.inspector_page + 1 < total_pages {
            self.inspector_page += 1;
            self.load_preview_page();
        }
    }

    fn prev_page(&mut self) {
        if self.inspector_tab != InspectorTab::Preview {
            return;
        }
        if self.inspector_page > 0 {
            self.inspector_page -= 1;
            self.load_preview_page();
        }
    }

    fn load_preview_page(&mut self) {
        let file = match self.inspector_file.clone() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return,
        };
        let where_clause = Self::build_where_clause(&self.inspector_filters);
        match DuckDbInspector::new(file) {
            Ok(inspector) => match inspector.preview(50, self.inspector_page * 50, &where_clause) {
                Ok((headers, data)) => {
                    self.inspector_preview_headers = headers;
                    self.inspector_preview_data = data;
                    self.inspector_scroll = 0;
                }
                Err(e) => {
                    self.popup = Popup::Message {
                        title: "Error".to_string(),
                        body: e.to_string(),
                    };
                }
            }, Err(e) => {
                self.popup = Popup::Message {
                    title: "Error".to_string(),
                    body: e.to_string(),
                }
            }
        }
    }

    fn open_filter_popup(&mut self) {
        if self.inspector_tab != InspectorTab::Preview {
            return;
        }
        self.popup = Popup::FilterEditor(FilterEditorState {
            conditions: self.inspector_filters.clone(),
            column_idx: 0,
            operator_idx: 0,
            value_input: String::new(),
            active_field: FilterField::Column,
        });
    }

    fn filter_tab_next(&mut self) {
        if let Popup::FilterEditor(ref mut state) = self.popup {
            let op = FILTER_OPERATORS[state.operator_idx];
            state.active_field = match state.active_field {
                FilterField::Column => FilterField::Operator,
                FilterField::Operator => {
                    if op == "IS NULL" || op == "IS NOT NULL" {
                        FilterField::Column
                    } else {
                        FilterField::Value
                    }
                }
                FilterField::Value => FilterField::Column,
            };
        }
    }

    fn filter_nav_up(&mut self) {
        if let Popup::FilterEditor(ref mut state) = self.popup {
            match state.active_field {
                FilterField::Column => {
                    if state.column_idx > 0 {
                        state.column_idx -= 1;
                    }
                }
                FilterField::Operator => {
                    if state.operator_idx > 0 {
                        state.operator_idx -= 1;
                    }
                }
                FilterField::Value => {}
            }
        }
    }

    fn filter_nav_down(&mut self) {
        if let Popup::FilterEditor(ref mut state) = self.popup {
            match state.active_field {
                FilterField::Column => {
                    if state.column_idx + 1 < self.inspector_schema.len() {
                        state.column_idx += 1;
                    }
                }
                FilterField::Operator => {
                    if state.operator_idx + 1 < FILTER_OPERATORS.len() {
                        state.operator_idx += 1;
                    }
                }
                FilterField::Value => {}
            }
        }
    }

    fn filter_char(&mut self, c: char) {
        if let Popup::FilterEditor(ref mut state) = self.popup {
            state.value_input.push(c);
        }
    }

    fn filter_backspace(&mut self) {
        if let Popup::FilterEditor(ref mut state) = self.popup {
            state.value_input.pop();
        }
    }

    fn filter_add_condition(&mut self) {
        if let Popup::FilterEditor(ref mut state) = self.popup {
            if let Some((col_name, _)) = self.inspector_schema.get(state.column_idx) {
                let op = FILTER_OPERATORS[state.operator_idx];
                state.conditions.push(FilterCondition {
                    column: col_name.clone(),
                    operator: op.to_string(),
                    value: state.value_input.clone(),
                });
                state.value_input.clear();
                state.active_field = FilterField::Column;
            }
        }
    }

    fn filter_remove_last(&mut self) {
        if let Popup::FilterEditor(ref mut state) = self.popup {
            state.conditions.pop();
        }
    }

    fn filter_apply(&mut self) {
        let conditions = if let Popup::FilterEditor(ref state) = self.popup {
            state.conditions.clone()
        } else {
            return;
        };
        self.inspector_filters = conditions;
        self.inspector_page = 0;
        self.inspector_scroll = 0;
        self.popup = Popup::None;
        self.reload_preview_with_filters();
    }

    fn build_where_clause(filters: &[FilterCondition]) -> String {
        if filters.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = filters.iter().map(|f| {
            let col = f.column.replace('"', "\"\"");
            match f.operator.as_str() {
                "IS NULL"     => format!("\"{}\" IS NULL", col),
                "IS NOT NULL" => format!("\"{}\" IS NOT NULL", col),
                "LIKE" => {
                    let v = f.value.replace('\'', "''");
                    format!("\"{}\"::VARCHAR LIKE '%{}%'", col, v)
                }
                op => {
                    let v = f.value.replace('\'', "''");
                    format!("\"{}\" {} '{}'", col, op, v)
                }
            }
        }).collect();
        format!("WHERE {}", parts.join(" AND "))
    }

    fn reload_preview_with_filters(&mut self) {
        let file = match self.inspector_file.clone() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return,
        };
        let where_clause = Self::build_where_clause(&self.inspector_filters);
        match DuckDbInspector::new(file) {
            Ok(inspector) => {
                match inspector.row_count_filtered(&where_clause) {
                    Ok(count) => self.inspector_row_count = count,
                    Err(e) => {
                        self.popup = Popup::Message {
                            title: "Error".to_string(),
                            body: e.to_string(),
                        };
                        return;
                    }
                }
                match inspector.preview(50, 0, &where_clause) {
                    Ok((headers, data)) => {
                        self.inspector_preview_headers = headers;
                        self.inspector_preview_data = data;
                        self.inspector_scroll = 0;
                    }
                    Err(e) => {
                        self.popup = Popup::Message {
                            title: "Error".to_string(),
                            body: e.to_string(),
                        };
                    }
                }
            }
            Err(e) => {
                self.popup = Popup::Message {
                    title: "Error".to_string(),
                    body: e.to_string(),
                };
            }
        }
    }

    fn convert_file(&mut self) {
        if let Some(ref file) = self.inspector_file {
            let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
            let target = if ext == "csv" { "parquet" } else { "csv" };
            self.popup = Popup::ConvertConfirm {
                target_format: target.to_string(),
            };
        }
    }

    fn confirm_convert(&mut self) {
        let target_format = match &self.popup {
            Popup::ConvertConfirm { target_format } => target_format.clone(),
            _ => return,
        };

        let file = match &self.inspector_file {
            Some(f) => f.to_string_lossy().to_string(),
            None => return,
        };

        match DuckDbInspector::new(file) {
            Ok(inspector) => match inspector.convert(&target_format) {
                Ok(path) => {
                    self.popup = Popup::Message {
                        title: "Success".to_string(),
                        body: format!("Converted to {}", path),
                    };
                }
                Err(e) => {
                    self.popup = Popup::Message {
                        title: "Error".to_string(),
                        body: e.to_string(),
                    };
                }
            },
            Err(e) => {
                self.popup = Popup::Message {
                    title: "Error".to_string(),
                    body: e.to_string(),
                };
            }
        }
    }

    pub fn view(&self, frame: &mut Frame) {
        match self.current_screen {
            Screen::Home => views::home::render(frame, self),
            Screen::FileBrowser => views::file_browser::render(frame, self),
            Screen::DataInspector => views::data_inspector::render(frame, self),
            Screen::JsonInspector => views::json_inspector::render(frame, self),
        }
    }

    pub fn load_json_data(&mut self, path: &Path) -> anyhow::Result<()> {
        use crate::commands::JsonInspector;
        use crate::tui::tree::build_tree;

        let inspector = JsonInspector::new(path)?;
        self.json_raw = serde_json::to_string_pretty(&inspector.root)?;
        self.json_kind = Some(inspector.kind.clone());
        self.json_collapsed = std::collections::HashSet::new();
        self.json_tree_nodes = build_tree(&inspector.root, &self.json_collapsed);

        if inspector.kind == crate::commands::json_inspector::FileKind::GeoJson {
            let (count, types, bbox) = inspector.geojson_summary();
            self.json_geosummary = Some((count, types, bbox));
            let (headers, rows) = inspector.features_table();
            self.json_features_headers = headers;
            self.json_features_data = rows;
            self.geo_tab = GeoJsonTab::Summary;
        } else {
            self.json_tab = JsonInspectorTab::Tree;
            self.json_geosummary = None;
            self.json_features_headers = vec![];
            self.json_features_data = vec![];
        }

        self.json_root = Some(inspector.root);
        self.json_scroll = 0;
        self.json_file = Some(path.to_path_buf());
        Ok(())
    }

    fn toggle_tree_node(&mut self) {
        if let Some((path, node)) = self.json_tree_nodes.get(self.json_scroll) {
            use crate::tui::tree::NodeKind;
            match &node.kind {
                NodeKind::Object | NodeKind::Array => {
                    let path = path.clone();
                    if self.json_collapsed.contains(&path) {
                        self.json_collapsed.remove(&path);
                    } else {
                        self.json_collapsed.insert(path);
                    }
                    if let Some(ref root) = self.json_root.clone() {
                        self.json_tree_nodes =
                            crate::tui::tree::build_tree(root, &self.json_collapsed);
                    }
                }
                _ => {}
            }
        }
    }

    fn switch_geo_tab(&mut self) {
        self.json_scroll = 0;
        self.geo_tab = match self.geo_tab {
            GeoJsonTab::Summary => GeoJsonTab::Features,
            GeoJsonTab::Features => GeoJsonTab::Tree,
            GeoJsonTab::Tree => GeoJsonTab::Summary,
        };
    }

    fn load_dir_entries(&mut self) -> anyhow::Result<()> {
        let mut entries = Vec::new();

        // Parent directory entry
        if let Some(parent) = self.current_dir.parent() {
            entries.push(DirEntryInfo {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                size: 0,
                modified: None,
            });
        }

        let mut file_entries: Vec<DirEntryInfo> = Vec::new();
        for entry in std::fs::read_dir(&self.current_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            file_entries.push(DirEntryInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified().ok(),
            });
        }

        // Sort: directories first, then alphabetical
        file_entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        entries.extend(file_entries);
        self.dir_entries = entries;
        self.browser_selected = 0;
        Ok(())
    }

    fn load_inspector_data(&mut self, path: &Path) -> anyhow::Result<()> {
        let inspector = DuckDbInspector::new(path.to_string_lossy().to_string())?;

        self.inspector_schema = inspector.schema()?;
        self.inspector_row_count = inspector.row_count()?;

        // Null counts per column
        self.inspector_null_counts = Vec::new();
        for (name, _) in &self.inspector_schema {
            match inspector.null_count(name) {
                Ok(count) => self.inspector_null_counts.push(count),
                Err(_) => self.inspector_null_counts.push(0),
            }
        }

        self.inspector_mean_values = Vec::new();
        self.inspector_min_values = Vec::new();
        self.inspector_max_values = Vec::new();

        for (name, _) in &self.inspector_schema {
            match inspector.mean_value(name) {
                Ok(value) => self.inspector_mean_values.push(value),
                Err(_) => self.inspector_mean_values.push("-".to_string()),
            }
            match inspector.min_value(name) {
                Ok(value) => self.inspector_min_values.push(value),
                Err(_) => self.inspector_min_values.push("-".to_string()),
            }
            match inspector.max_value(name) {
                Ok(value) => self.inspector_max_values.push(value),
                Err(_) => self.inspector_max_values.push("-".to_string()),
            }
        }

        // Preview data
        let (headers, data) = inspector.preview(50, 0, "")?;
        self.inspector_preview_headers = headers;
        self.inspector_preview_data = data;

        self.inspector_scroll = 0;
        self.inspector_page = 0;
        self.inspector_filters = Vec::new();
        self.inspector_tab = InspectorTab::Schema;

        Ok(())
    }
}
