//! 测试公共模块
//! 
//! 这个模块包含了所有测试共享的公共函数、工具和测试数据生成器

pub mod test_utils;
pub mod test_data;
pub mod mock_services;
pub mod assertions;

// 重新导出常用的测试工具
pub use test_utils::*;
pub use test_data::*;
pub use mock_services::*;
pub use assertions::*;