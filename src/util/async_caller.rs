pub struct AsyncCaller {
    runtime: tokio::runtime::Runtime,
}

impl AsyncCaller {
    pub fn new() -> Self {
        Self {
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        }
    }

    pub fn call<F: Future>(self, future: F) -> F::Output {
        let output = self.runtime.block_on(future);
        self.runtime.shutdown_background();
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn sum(a: u32, b: u32) -> u32 {
        a + b
    }

    #[test]
    fn test_single_async_call() {
        assert_eq!(5, AsyncCaller::new().call(sum(2, 3)));
    }

    #[test]
    fn test_multiple_runtime_async_calls() {
        assert_eq!(5, AsyncCaller::new().call(sum(2, 3)));
        assert_eq!(3, AsyncCaller::new().call(sum(1, 2)));
    }
}
