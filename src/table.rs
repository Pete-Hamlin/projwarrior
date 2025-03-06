use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, Color, Table};
use serde::{Deserialize, Serialize};

use crate::project::Project;
use crate::{config::GtdConfig, parser::Task};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTableItem {
    pub index: usize,
    pub name: String,
    pub tasks: i32,
}

pub fn project_list_table(cfg: &GtdConfig, tasks: &[Task], projects: &[Project]) {
    let headers = vec!["ID", "Name", "Tasks"];
    let mut table = create_table(&headers);
    let mut output = generate_project_list(tasks, projects);

    output.sort_by(|a, b| a.tasks.cmp(&b.tasks));
    for (index, item) in output.into_iter().enumerate() {
        // Set colors
        let color = if cfg.color {
            determine_proj_color(item.tasks as usize)
        } else {
            Color::Reset
        };
        let bg_color = if cfg.color && index % 2 == 0 {
            Color::Black
        } else {
            Color::Reset
        };

        if !cfg.short || item.tasks == 0 {
            table.add_row(vec![
                Cell::new(item.index.to_string()).fg(color).bg(bg_color),
                Cell::new(item.name.to_string()).fg(color).bg(bg_color),
                Cell::new(item.tasks.to_string()).fg(color).bg(bg_color),
            ]);
        }
    }
    if table.row_count() > 0 {
        println!("{table}");
    }
}

pub fn project_details_table(cfg: &GtdConfig, project: &Project, tasks: &[Task]) {
    let headers = vec!["Name", "Value"];
    let mut table = create_table(&headers);
    let color = if cfg.color {
        determine_proj_color(tasks.len())
    } else {
        Color::Reset
    };
    table.add_row(vec![Cell::new("Name"), Cell::new(&project.name).fg(color)]);
    table.add_row(vec![
        Cell::new("Status"),
        Cell::new(format!("{:?}", &project.state)),
    ]);

    println!("{table}");
    if tasks.len() > 0 {
        task_list_table(cfg, tasks);
    }
}

fn task_list_table(cfg: &GtdConfig, tasks: &[Task]) {
    let headers = vec!["ID", "Entry", "Description", "Status", "Tags"];
    let mut table = create_table(&headers);
    for (index, item) in tasks.into_iter().enumerate() {
        let bg_color = if cfg.color && index % 2 == 0 {
            Color::Black
        } else {
            Color::Reset
        };
        table.add_row(vec![
            Cell::new(item.id.to_string()).bg(bg_color),
            Cell::new(item.entry.to_string()).bg(bg_color),
            Cell::new(item.description.to_string()).bg(bg_color),
            Cell::new(item.status.to_string()).bg(bg_color),
            Cell::new(item.tags.clone().unwrap_or(vec![]).join(", ")).bg(bg_color),
        ]);
    }
    println!("{table}");
}

/// Converts the imported JSON `Project` struct to a `ProjectListItem` (the data we wish to display).
/// Currently attaches the following data:
/// - Current pending task count
///
/// * `tasks`: Parsed task list JSON
/// * `projects`: Parsed project list JSON
pub fn generate_project_list(tasks: &[Task], projects: &[Project]) -> Vec<ProjectTableItem> {
    let result: Vec<ProjectTableItem> = projects
        .iter()
        .enumerate()
        .map(|(index, project)| {
            let count = project.get_tasks(tasks);
            return ProjectTableItem {
                index,
                name: project.name.clone(),
                tasks: count,
            };
        })
        .collect();
    return result;
}

fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.load_preset(NOTHING);
    let table_headers: Vec<Cell> = headers
        .iter()
        .map(|header| Cell::new(header).add_attribute(Attribute::Underlined))
        .collect();

    table.set_header(table_headers);
    return table;
}

fn determine_proj_color(task_count: usize) -> Color {
    if task_count == 0 {
        return Color::Yellow;
    } else {
        return Color::Green;
    }
}
