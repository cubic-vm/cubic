pub trait Animation: Send {
    fn render(&mut self, width: usize) -> String;
}

#[cfg(test)]
mod tests {
    use super::Animation;

    struct Counter {
        count: u32,
    }

    impl Animation for Counter {
        fn render(&mut self, _width: usize) -> String {
            self.count += 1;
            format!("frame {}", self.count)
        }
    }

    #[test]
    fn test_render_advances_each_frame() {
        let mut counter = Counter { count: 0 };
        assert_eq!(counter.render(80), "frame 1");
        assert_eq!(counter.render(80), "frame 2");
    }
}
