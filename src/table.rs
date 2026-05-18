use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, Color, Table};
use task_hookrs::task::Task;

use crate::config::ProjwarriorConfig;
use crate::project_table_item::ProjectTableItem;

pub enum Column {
    Id,
    Uuid,
    Name,
    Status,
    Tasks,
    Entry,
}

impl Column {
    fn header(&self) -> &str {
        match self {
            Column::Id => "ID",
            Column::Uuid => "UUID",
            Column::Name => "Description",
            Column::Status => "Status",
            Column::Tasks => "Tasks",
            Column::Entry => "Entry",
        }
    }

    fn row(&self, item: &ProjectTableItem) -> String {
        match self {
            Column::Id => match item.id {
                Some(id) => id.to_string(),
                None => "-".to_string(),
            },
            Column::Uuid => item.uuid.to_string(),
            Column::Name => item.name.to_string(),
            Column::Status => item.status.to_string(),
            Column::Tasks => item.tasks.to_string(),
            Column::Entry => match item.entry {
                Some(entry) => entry.format("%Y-%m-%d").to_string(),
                None => "-".to_string(),
            },
        }
    }
}

pub fn project_list_table(
    cfg: &ProjwarriorConfig,
    projects: &[ProjectTableItem],
    columns: &[Column],
) {
    let headers: Vec<&str> = columns.iter().map(|col| col.header()).collect();
    let mut table = create_table(&headers);

    for (index, item) in projects.iter().enumerate() {
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

        let rows: Vec<Cell> = columns
            .iter()
            .map(|col| Cell::new(col.row(item)).fg(color).bg(bg_color))
            .collect();

        table.add_row(rows);
    }
    if table.row_count() > 0 {
        println!("{table}");
    }
}

pub fn project_details_table(
    cfg: &ProjwarriorConfig,
    project: &taskchampion::Task,
    tasks: &[Task],
    id: Option<usize>,
) {
    let headers = vec!["Name", "Value"];
    let mut table = create_table(&headers);
    let color = if cfg.color {
        determine_proj_color(tasks.len())
    } else {
        Color::Reset
    };
    table.add_row(vec![
        Cell::new("ID"),
        Cell::new(match id {
            Some(id) => id.to_string(),
            None => "-".to_string(),
        }),
    ]);
    table.add_row(vec![Cell::new("UUID"), Cell::new(project.get_uuid())]);
    table.add_row(vec![
        Cell::new("Description"),
        Cell::new(project.get_description()).fg(color),
    ]);
    table.add_row(vec![
        Cell::new("Status"),
        Cell::new(format!("{:?}", &project.get_status())),
    ]);

    println!("{table}");
    if !tasks.is_empty() {
        task_list_table(cfg, tasks);
    }
}

fn task_list_table(cfg: &ProjwarriorConfig, tasks: &[Task]) {
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
