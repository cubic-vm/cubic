use crate::view::Console;

#[derive(Default)]
pub struct MapView {
    items: Vec<(String, String)>,
}

impl MapView {
    pub fn new() -> Self {
        MapView::default()
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.items.push((key.to_string(), value.to_string()));
    }

    pub fn print(self, console: &mut dyn Console) {
        let max_key_length = self
            .items
            .iter()
            .map(|(key, _)| key.len() + 1)
            .max()
            .unwrap_or(0);

        self.items.iter().for_each(|(key, value)| {
            let mut key = key.clone();
            if !key.is_empty() {
                key += ":";
            }
            console.info(&format!("{key:max_key_length$} {value}"));
        });
    }
}
