use crate::view::Console;

pub enum Alignment {
    Left,
    Right,
}

#[derive(Default)]
pub struct Row {
    pub entries: Vec<(String, Alignment)>,
}

impl Row {
    pub fn add(&mut self, entry: &str, alignment: Alignment) -> &mut Self {
        self.entries.push((entry.to_string(), alignment));
        self
    }
}

#[derive(Default)]
pub struct TableView {
    rows: Vec<Row>,
}

impl TableView {
    pub fn new() -> Self {
        TableView::default()
    }

    pub fn add_row(&mut self) -> &mut Row {
        let row = Row::default();
        self.rows.push(row);
        self.rows.last_mut().unwrap()
    }

    pub fn print(&self, console: &mut dyn Console) {
        let mut column_size = Vec::new();
        for row in &self.rows {
            for (index, (entry, _)) in row.entries.iter().enumerate() {
                while index >= column_size.len() {
                    column_size.push(0);
                }

                column_size[index] = column_size[index].max(entry.len());
            }
        }

        for row in &self.rows {
            let line = row
                .entries
                .iter()
                .enumerate()
                .map(|(index, (entry, alignment))| match alignment {
                    Alignment::Left => format!("{entry:<width$}", width = column_size[index]),
                    Alignment::Right => format!("{entry:>width$}", width = column_size[index]),
                })
                .collect::<Vec<_>>()
                .join("   ");
            console.info(line.trim_end());
        }
    }
}
