// src/tui/mod.rs
use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use qdrant_client::prelude::point_id::PointIdOptions;
use qdrant_client::qdrant::PointId;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::{error::AppError, llama::LlamaService};
use crate::{prelude::Result, rag::vectors::VectorDB};

pub struct App {
    pub input: String,
    pub cursor_position: usize,
    pub messages: Vec<String>,
    pub selected_tab: Tab,
    pub search_results: Vec<SearchResult>,
    pub selected_result: Option<usize>,
    pub status_message: Option<String>,
}

pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub content: String,
    pub content_preview: String,
    pub score: f32,
    pub file_path: String,
}

pub enum Tab {
    Chat,
    Search,
    Settings,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            messages: Vec::new(),
            selected_tab: Tab::Chat,
            search_results: Vec::new(),
            selected_result: None,
            status_message: None,
        }
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn next_result(&mut self) {
        if !self.search_results.is_empty() {
            match self.selected_result {
                Some(idx) if idx < self.search_results.len() - 1 => {
                    self.selected_result = Some(idx + 1);
                }
                None => {
                    self.selected_result = Some(0);
                }
                _ => {}
            }
        }
    }

    pub fn previous_result(&mut self) {
        if !self.search_results.is_empty() {
            match self.selected_result {
                Some(idx) if idx > 0 => {
                    self.selected_result = Some(idx - 1);
                }
                None => {
                    self.selected_result = Some(self.search_results.len() - 1);
                }
                _ => {}
            }
        }
    }

    pub fn view_full_content(&self) -> Option<String> {
        match self.selected_result {
            Some(idx) => self.search_results.get(idx).map(|result| {
                format!(
                    "# {}\n\nPath: {}\n\n{}",
                    result.title,
                    result.file_path,
                    result.content_preview
                )
            }),
            None => None,
        }
    }
    pub fn add_search_result(
        &mut self,
        id: String,
        title: String,
        content: String,
        score: f32,
        file_path: String,
    ) {
        // Create a short preview of the content
        let content_preview = if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.clone()
        };

        self.search_results.push(SearchResult {
            id,
            title,
            content,
            content_preview,
            score,
            file_path,
        });
    }

    pub fn clear_search_results(&mut self) {
        self.search_results.clear();
        self.selected_result = None;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn submit_message(&mut self) -> String {
        let message = std::mem::take(&mut self.input);
        self.cursor_position = 0;
        self.messages.push(format!("You: {}", message.clone()));
        message
    }

    pub fn add_ai_response(&mut self, response: String) {
        self.messages.push(format!("AI: {}", response));
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Chat => Tab::Search,
            Tab::Search => Tab::Settings,
            Tab::Settings => Tab::Chat,
        }
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Chat => Tab::Settings,
            Tab::Search => Tab::Chat,
            Tab::Settings => Tab::Search,
        }
    }

}

pub async fn run_app(llama_service: &LlamaService, vector_db: &VectorDB) -> Result<()> {
    // Setup terminal
    enable_raw_mode()
        .map_err(|e| AppError::TUIError(format!("Failed to enable raw mode: {}", e)))?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| AppError::TUIError(format!("Failed to enter alternate screen: {}", e)))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| AppError::TUIError(format!("Failed to create terminal: {}", e)))?;

    // Create app state
    let mut app = App::new();
    app.messages.push(String::from(
        "AI: Welcome to Raggy Notes! How can I help you today?",
    ));
    app.set_status("Connected to Ollama and Qdrant");

    // Use a oneshot channel to handle the status timeout
    let (status_sender, status_receiver) = tokio::sync::oneshot::channel();

    // Spawn a task to clear the status after 3 seconds
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        // Send signal to clear status
        let _ = status_sender.send(());
    });

    // Run the application
    let result = tokio::select! {
        ui_result = run_ui(&mut terminal, &mut app, llama_service, vector_db) => ui_result,
        _ = status_receiver => {
            // Clear status message after timeout
            app.clear_status();
            run_ui(&mut terminal, &mut app, llama_service, vector_db).await
        }
    };

    // Restore terminal
    restore_terminal(&mut terminal).map_err(|e| {
        eprintln!("Error restoring terminal: {}", e);
        AppError::TUIError(format!("Failed to restore terminal: {}", e))
    })?;

    // Return result from UI
    result
}

fn restore_terminal<W: std::io::Write>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    llama_service: &LlamaService,
    vector_db: &VectorDB,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            return Ok(());
                        } else {
                            app.insert_char('q');
                        }
                    }
                    KeyCode::Char('c') => {
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            return Ok(());
                        } else {
                            app.insert_char('c');
                        }
                    }
                    KeyCode::Char(c) => {
                        app.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }

                    KeyCode::Up => {
                        if matches!(app.selected_tab, Tab::Search) {
                            app.previous_result();
                        }
                    },
                    KeyCode::Down => {
                        if matches!(app.selected_tab, Tab::Search) {
                            app.next_result();
                        }
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    }
                    KeyCode::Tab => {
                        app.next_tab();
                    }
                    KeyCode::BackTab => {
                        app.previous_tab();
                    }
                    KeyCode::Enter => {
                        match app.selected_tab {
                            Tab::Chat => {
                                if !app.input.is_empty() {
                                    let user_message = app.submit_message();
                                    app.set_status("Thinking...");

                                    // Redraw UI
                                    terminal.draw(|f| ui(f, app))?;

                                    // Use RAG-enhanced completion
                                    match llama_service.generate_rag_completion(&user_message, vector_db).await {
                                        Ok(response) => {
                                            app.add_ai_response(response);
                                        },
                                        Err(e) => {
                                            app.add_ai_response(format!("Error generating response: {}", e));
                                        }
                                    }

                                    app.clear_status();
                                }
                            }
                            Tab::Search => {
                                if !app.input.is_empty() {
                                    let search_query = app.submit_message();
                                    app.set_status("Searching...");
                                    app.clear_search_results();

                                    terminal.draw(|f| ui(f, app))?;

                                    // Get embedding for search query
                                    match llama_service.get_embedding(&search_query).await {
                                        Ok(embedding) => {
                                            // Search for similar notes
                                            match vector_db
                                                .search_similar_notes(embedding, 10)
                                                .await
                                            {
                                                Ok(results) => {
                                                    if results.result.is_empty() {
                                                        app.add_ai_response("No relevant notes found for your query.".into());
                                                    } else {
                                                        app.add_ai_response(format!(
                                                            "Found {} relevant notes.",
                                                            results.result.len()
                                                        ));

                                                        // Process search results
                                                        for point in results.result {


                                                            let title = point
                                                                .payload
                                                                .get("title")
                                                                .and_then(|v| v.as_str())
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_else(|| {
                                                                    "Untitled".to_string()
                                                                });

                                                            let content = point
                                                                .payload
                                                                .get("content")
                                                                .and_then(|v| v.as_str())
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_else(|| {
                                                                    "No content".to_string()
                                                                });

                                                            let file_path = point
                                                                .payload
                                                                .get("file_path")
                                                                .and_then(|v| v.as_str())
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_else(|| {
                                                                    "Unknown path".to_string()
                                                                });

                                                            let id  =  match point.id.map(|id| id.point_id_options).flatten() {
                                                                None => "Unknown ID".to_string(),
                                                                Some(id_opt) => match id_opt {
                                                                    PointIdOptions::Num(num) => num.to_string(),
                                                                    PointIdOptions::Uuid(uid) => uid,
                                                                }
                                                            };

                                                            // Add to search results
                                                            app.add_search_result(
                                                                id,
                                                                title,
                                                                content,
                                                                point.score,
                                                                file_path,
                                                            );
                                                        }

                                                        // Select first result by default
                                                        if !app.search_results.is_empty() {
                                                            app.selected_result = Some(0);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    app.add_ai_response(format!(
                                                        "Error searching notes: {}",
                                                        e
                                                    ));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            app.add_ai_response(format!(
                                                "Error generating embedding: {}",
                                                e
                                            ));
                                        }
                                    }

                                    app.clear_status();
                                }
                            }
                            Tab::Settings => {
                                // Handle settings tab actions
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // tabs
                Constraint::Min(1),    // content
                Constraint::Length(3), // input
                Constraint::Length(1), // status
            ]
            .as_ref(),
        )
        .split(f.size());

    // Render tab bar
    let tabs = ["Chat", "Search", "Settings"];
    let tab_items: Vec<Line> = tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (first, rest) = t.split_at(1);
            let selected = matches!(
                (&app.selected_tab, i),
                (Tab::Chat, 0) | (Tab::Search, 1) | (Tab::Settings, 2)
            );

            let style = if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(vec![
                Span::styled(first.to_string(), style.add_modifier(Modifier::UNDERLINED)),
                Span::styled(rest.to_string(), style),
            ])
        })
        .collect();

    let tabs_paragraph = Paragraph::new(tab_items);
    f.render_widget(tabs_paragraph, chunks[0]);

    // Render content based on selected tab
    match app.selected_tab {
        Tab::Chat => {
            // Render chat messages
            let messages: Vec<ListItem> = app
                .messages
                .iter()
                .map(|m| {
                    let message = Text::from(m.clone());
                    ListItem::new(message)
                })
                .collect();

            let messages_list =
                List::new(messages).block(Block::default().borders(Borders::ALL).title("Chat"));

            f.render_widget(messages_list, chunks[1]);
        }
        Tab::Search => {
            // Create a split layout for search results and preview
            let search_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(40), // results list
                        Constraint::Percentage(60), // preview
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            // Render search results
            let results: Vec<ListItem> = if app.search_results.is_empty() {
                vec![ListItem::new(
                    "No search results yet. Type a query and press Enter.",
                )]
            } else {
                app.search_results
                    .iter()
                    .enumerate()
                    .map(|(i, result)| {
                        let style = if Some(i) == app.selected_result {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        };

                        let score_text = format!(" ({:.2})", result.score);
                        let spans = vec![
                            Span::styled(&result.title, style.add_modifier(Modifier::BOLD)),
                            Span::styled(score_text, style.fg(Color::DarkGray)),
                        ];

                        ListItem::new(Line::from(spans))
                    })
                    .collect()
            };

            let results_list = List::new(results)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Search Results"),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            f.render_widget(results_list, search_chunks[0]);

            // Render preview of the selected result
            let preview_content = if let Some(selected_idx) = app.selected_result {
                if let Some(selected_result) = app.search_results.get(selected_idx) {
                    let mut content_lines = Vec::new();
                    content_lines.push(Line::from(vec![
                        Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&selected_result.title),
                    ]));
                    content_lines.push(Line::from(vec![
                        Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&selected_result.file_path),
                    ]));
                    content_lines.push(Line::from(vec![
                        Span::styled("Score: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{:.2}", selected_result.score)),
                    ]));
                    content_lines.push(Line::from(Span::raw(""))); // Empty line

                    // Add full content with line breaks preserved
                    for content_line in selected_result.content.lines() {
                        content_lines.push(Line::from(Span::raw(content_line)));
                    }

                    content_lines
                } else {
                    vec![Line::from("No result selected")]
                }
            } else {
                vec![Line::from("Select a result to see preview")]
            };

            let preview = Paragraph::new(preview_content)
                .block(Block::default().borders(Borders::ALL).title("Preview"))
                .wrap(ratatui::widgets::Wrap { trim: false });

            f.render_widget(preview, search_chunks[1]);
        }
        Tab::Settings => {
            // Render settings
            let settings = Paragraph::new("Settings (not yet implemented)")
                .block(Block::default().borders(Borders::ALL).title("Settings"));

            f.render_widget(settings, chunks[1]);
        }
    }

    // Render input
    let input = Paragraph::new(app.input.clone())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input, chunks[2]);
    f.set_cursor(
        chunks[2].x + 1 + app.cursor_position as u16,
        chunks[2].y + 1,
    );

    // Render status message if present
    if let Some(status) = &app.status_message {
        let status_style = Style::default().fg(Color::White).bg(Color::Blue);
        let status_widget = Paragraph::new(status.as_str())
            .style(status_style)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(status_widget, chunks[3]);
    } else {
        // Render help text
        let help_text = match app.selected_tab {
            Tab::Chat => "Ctrl+Q/Ctrl+C: Quit | Tab: Switch tabs | Enter: Send message",
            Tab::Search => {
                "Ctrl+Q/Ctrl+C: Quit | Tab: Switch tabs | Enter: Search | ↑/↓: Navigate results"
            }
            Tab::Settings => "Ctrl+Q/Ctrl+C: Quit | Tab: Switch tabs",
        };

        let help_style = Style::default().fg(Color::DarkGray);
        let help_widget = Paragraph::new(help_text).style(help_style);
        f.render_widget(help_widget, chunks[3]);
    }
}
