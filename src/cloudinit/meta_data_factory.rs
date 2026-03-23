#[derive(Default)]
pub struct MetaDataFactory;

impl MetaDataFactory {
    pub fn create(&self, name: &str) -> String {
        format!("instance-id: {name}\nlocal-hostname: {name}\n")
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_meta_user_data() {
        assert_eq!(
            &MetaDataFactory::default().create("myinstance"),
            "instance-id: myinstance\nlocal-hostname: myinstance\n"
        );
    }
}
