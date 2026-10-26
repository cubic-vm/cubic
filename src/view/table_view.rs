use crate::platform::Stream;
use crate::view::Console;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

    pub fn print(&self, console: &Arc<Console>) {
        let mut column_size = Vec::new();
        for row in &self.rows {
            for (index, (entry, _)) in row.entries.iter().enumerate() {
                while index >= column_size.len() {
                    column_size.push(0);
                }

                column_size[index] = column_size[index].max(entry.len());
            }
        }

        let styler = console.get_styler(Stream::Stdout);
        for (row_index, row) in self.rows.iter().enumerate() {
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
            let line = line.trim_end();
            if row_index == 0 {
                console.print(&styler.bold(line));
            } else {
                console.print(line);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
