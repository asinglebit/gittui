#[rustfmt::skip]
use std::{
    collections::{
        HashMap
    },
    env,
    path::PathBuf,
    rc::Rc,
    cell::RefCell
};
#[rustfmt::skip]
use git2::Repository;
#[rustfmt::skip]
use indexmap::IndexMap;
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
#[rustfmt::skip]
use crate::{
    layers,
    app::app::{
        App,
        Layout,
        Viewport,
        Focus
    },
    core::{
        buffer::{
            Buffer
        }
    },
    helpers::{
        palette::*,
        colors::ColorPicker,
        spinner::Spinner
    },
    git::{
        queries::{
            helpers::{
                UncommittedChanges
            }
        }
    }
};

impl Default for App {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let path = if args.len() > 1 {
            &args[1]
        } else {
            &".".to_string()
        };
        let theme = Theme::default();
        let color = Rc::new(RefCell::new(ColorPicker::from_theme(&theme)));
        let layers = layers!(Rc::new(RefCell::new(ColorPicker::from_theme(&theme))));
        let absolute_path: PathBuf = std::fs::canonicalize(path)
            .unwrap_or_else(|_| PathBuf::from(path));
        let repo = Rc::new(Repository::open(absolute_path.clone()).expect("Could not open repo"));

        let logo = vec![
            Span::styled("  g", Style::default().fg(theme.COLOR_GRASS)),
            Span::styled("u", Style::default().fg(theme.COLOR_GRASS)),
            Span::styled("i", Style::default().fg(theme.COLOR_GRASS)),
            Span::styled("t", Style::default().fg(theme.COLOR_GRASS)),
            Span::styled("a", Style::default().fg(theme.COLOR_GRASS)),
            Span::styled("╭", Style::default().fg(theme.COLOR_GREEN))
        ];

        App {
            // General
            logo,
            path: absolute_path.display().to_string(),
            repo,
            hint: String::new(),
            spinner: Spinner::new(),
            keymap: IndexMap::new(),
            last_input_direction: None,
            theme,

            // User
            name: String::new(),
            email: String::new(),

            // Walker utilities    
            color,
            layers,
            buffer: RefCell::new(Buffer::default()),
            walker_rx: None,
            walker_cancel: None,
            walker_handle: None,

            // Walker data
            oids: Vec::new(),
            tips_local: HashMap::new(),
            tips_remote: HashMap::new(),
            tips: HashMap::new(),
            oid_colors: HashMap::new(),
            tip_colors: HashMap::new(),
            branch_oid_map: HashMap::new(),
            oid_branch_map: HashMap::new(),
            uncommitted: UncommittedChanges::default(),

            // Cache
            current_diff: Vec::new(),
            file_name: None,
            viewer_lines: Vec::new(),
            oid_branch_vec: Vec::new(),
            oid_branch_vec_chronological: Vec::new(),
            oid_branch_indices: Vec::new(),
            visible_branches: HashMap::new(),

            // Interface
            layout: Layout::default(),
            
            // Focus
            is_minimal: false,
            is_branches: false,
            is_status: false,
            is_inspector: false,
            viewport: Viewport::Splash,
            focus: Focus::Viewport,

            // Branches
            branches_selected: 0,
            branches_scroll: 0.into(),
            
            // Graph
            graph_selected: 0,
            graph_scroll: 0.into(),
            
            // Settings
            settings_selected: 0,
            settings_selections: Vec::new(),
    
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

            // Modal checkout
            modal_checkout_selected: 0,

            // Modal solo
            modal_solo_selected: 0,

            // Modal commit
            commit_editor: EditorState::default(),
            commit_editor_event_handler: EditorEventHandler::default(),

            // Exit
            is_exit: false,   
        }
    }
}
