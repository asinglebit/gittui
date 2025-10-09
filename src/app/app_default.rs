use std::{collections::HashMap, env, path::PathBuf};

use git2::Repository;

use crate::app::app::{App, Layout};

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

        App {
            // General
            path: absolute_path.display().to_string(),
            repo,

            // Data
            oids: Vec::new(),
            tips: HashMap::new(),
            tip_colors: HashMap::new(),
            branch_oid_map: HashMap::new(),
            oid_branch_map: HashMap::new(),
            lines_graph: Vec::new(),
            lines_branches: Vec::new(),
            lines_messages: Vec::new(),
            lines_buffers: Vec::new(),

            // Interface
            layout: Layout::default(),
            
            // Panes
            is_inspector: true,
            is_status: true,

            // Rest
            scroll: 0.into(),
            status_scroll: 0.into(),
            selected: 0,
            is_minimal: false,
            is_exit: false,            
            is_modal: false,
            modal_selected: 0
        }
    }
}