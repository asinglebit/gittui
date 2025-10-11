#[rustfmt::skip]
use std::{
    collections::HashMap,
    env,
    path::PathBuf
};
#[rustfmt::skip]
use git2::Repository;
#[rustfmt::skip]
use ratatui::{
    style::Style,
    text::{
        Span
    }
};
#[rustfmt::skip]
use edtui::{
    EditorEventHandler,
    EditorState
};
use crate::app::app::Viewport;
#[rustfmt::skip]
use crate::{
    app::app::{
        App,
        Layout,
        Focus
    },
    git::{
        queries::UncommittedChanges
    }
};
#[rustfmt::skip]
use crate::utils::colors::*;

impl Default for App {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let path = if args.len() > 1 {
            &args[1]
        } else {
            &".".to_string()
        };
        let absolute_path: PathBuf = std::fs::canonicalize(path)
            .unwrap_or_else(|_| PathBuf::from(path));
        let repo = Repository::open(absolute_path.clone()).expect("Could not open repo");

        let logo = vec![
            Span::styled(" g", Style::default().fg(COLOR_GRASS)),
            Span::styled("u", Style::default().fg(COLOR_GRASS)),
            Span::styled("i", Style::default().fg(COLOR_GRASS)),
            Span::styled("t", Style::default().fg(COLOR_GRASS)),
            Span::styled("a", Style::default().fg(COLOR_GRASS)),
            Span::styled("╭", Style::default().fg(COLOR_GREEN))
        ];

        App {
            // General
            path: absolute_path.display().to_string(),
            repo,
            logo,

            // User
            name: "Single Bit".to_string(),
            email: "asinglebit@gmail.com".to_string(),

            // Walker data
            oids: Vec::new(),
            tips: HashMap::new(),
            oid_colors: HashMap::new(),
            tip_colors: HashMap::new(),
            branch_oid_map: HashMap::new(),
            oid_branch_map: HashMap::new(),
            uncommitted: UncommittedChanges::default(),

            // Walker lines
            lines_graph: Vec::new(),
            lines_branches: Vec::new(),
            lines_messages: Vec::new(),
            lines_buffers: Vec::new(),

            // Cache
            current_diff: Vec::new(),
            file_name: None,
            viewer_lines: Vec::new(),

            // Interface
            layout: Layout::default(),
            
            // Focus
            is_minimal: false,
            is_status: true,
            is_inspector: true,
            viewport: Viewport::Graph,
            focus: Focus::Viewport,
            
            // Graph
            graph_selected: 0,
            graph_scroll: 0.into(),
    
            // Viewer
            viewer_selected: 0,
            viewer_scroll: 0.into(),

            // Editor
            file_editor: EditorState::default(),
            file_editor_event_handler: EditorEventHandler::default(),
    
            // Inspector
            inspector_selected: 0,
            inspector_scroll: 0.into(),
            
            // Status top
            status_top_selected: 0,
            status_top_scroll: 0.into(),
            
            // Status bottom
            status_bottom_selected: 0,
            status_bottom_scroll: 0.into(),

            // Modal branch
            modal_checkout_selected: 0,

            // Modal commit
            commit_editor: EditorState::default(),
            commit_editor_event_handler: EditorEventHandler::default(),

            // Exit
            is_exit: false,   
        }
    }
}
