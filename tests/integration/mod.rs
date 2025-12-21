//! Stage 89 Phase 3: 集成测试套件
//! 提供完整的跨模块、跨平台、跨语言集成测试

pub mod test_multilang_integration;
pub mod test_cross_platform;
pub mod test_end_to_end;

pub use test_multilang_integration::tests::*;
pub use test_cross_platform::tests::*;
pub use test_end_to_end::tests::*;
