pub trait Animation: Send {
    fn render(&mut self) -> String;
}

#[cfg(test)]
mod tests {
    use super::Animation;

    struct Counter {
        count: u32,
    }

    impl Animation for Counter {
        fn render(&mut self) -> String {
            self.count += 1;
            format!("frame {}", self.count)
        }
    }

    #[test]
    fn test_render_advances_each_frame() {
        let mut counter = Counter { count: 0 };
        assert_eq!(counter.render(), "frame 1");
        assert_eq!(counter.render(), "frame 2");
    }
}
