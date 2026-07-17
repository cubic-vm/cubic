mod system;
mod system_mock;

pub use system::*;
#[cfg(test)]
pub use system_mock::tests::SystemMock;
