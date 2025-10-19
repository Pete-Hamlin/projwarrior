use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use task_hookrs::task::Task;

use crate::config::GtdConfig;
use crate::project::Project;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTableItem {
    pub project: Project,
    pub tasks: i32,
}

pub enum Column {
    Id,
    Uuid,
    Name,
    State,
    Tasks,
    Entry,
}

impl Column {
    fn header(&self) -> &str {
        match self {
            Column::Id => "ID",
            Column::Uuid => "UUID",
            Column::Name => "Name",
            Column::State => "Status",
            Column::Tasks => "Tasks",
            Column::Entry => "Entry",
        }
    }

    fn row(&self, item: &ProjectTableItem) -> String {
        match self {
            Column::Id => match item.project.id {
                Some(id) => id.to_string(),
                None => "-".to_string(),
            },
            Column::Uuid => item.project.uuid.to_string(),
            Column::Name => item.project.name.to_string(),
            Column::State => item.project.state.to_string(),
            Column::Tasks => item.tasks.to_string(),
            Column::Entry => item.project.created_at.format("%Y-%m-%d").to_string(),
        }
    }
}

pub fn project_list_table(
    cfg: &GtdConfig,
    tasks: &[Task],
    projects: &[Project],
    columns: &[Column],
) {
    // let headers = vec!["ID", "Name", "Tasks", "Entry"];
    let headers: Vec<&str> = columns.iter().map(|col| col.header()).collect();
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
            let rows: Vec<Cell> = columns
                .iter()
                .map(|col| Cell::new(col.row(&item)).fg(color).bg(bg_color))
                .collect();

            table.add_row(rows);
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
        Cell::new("ID"),
        Cell::new(match &project.id {
            Some(id) => id.to_string(),
            None => "-".to_string(),
        }),
    ]);
    table.add_row(vec![Cell::new("UUID"), Cell::new(project.uuid)]);
    table.add_row(vec![
        Cell::new("Status"),
        Cell::new(format!("{:?}", &project.state)),
    ]);

    println!("{table}");
    if !tasks.is_empty() {
        task_list_table(cfg, tasks);
    }
}

fn task_list_table(cfg: &GtdConfig, tasks: &[Task]) {
    let headers = vec!["ID", "Entry", "Description", "Status", "Tags"];
    let mut table = create_table(&headers);
    for (index, item) in tasks.iter().enumerate() {
        let bg_color = if cfg.color && index % 2 == 0 {
            Color::Black
        } else {
            Color::Reset
        };
        let tags: String = match item.tags() {
            Some(tag_vec) => tag_vec.join(", "),
            None => "".to_string(),
        };
        table.add_row(vec![
            Cell::new(match item.id() {
                Some(id) => id.to_string(),
                None => "-".to_string(),
            })
            .bg(bg_color),
            Cell::new(item.entry().to_string()).bg(bg_color),
            Cell::new(item.description()).bg(bg_color),
            Cell::new(item.status().to_string()).bg(bg_color),
            Cell::new(tags).bg(bg_color),
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
        .map(|project| {
            let count = project.get_tasks(tasks);
            ProjectTableItem {
                project: project.clone(),
                tasks: count,
            }
        })
        .collect();
    result
}

fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.load_preset(NOTHING);
    let table_headers: Vec<Cell> = headers
        .iter()
        .map(|header| Cell::new(header).add_attribute(Attribute::Underlined))
        .collect();

    table.set_header(table_headers);
    table
}

fn determine_proj_color(task_count: usize) -> Color {
    if task_count == 0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
