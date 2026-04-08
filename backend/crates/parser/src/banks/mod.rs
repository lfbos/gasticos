//! Bank-specific parsers for Colombian banks.

mod bancolombia;
mod nequi;
mod nu;

pub use bancolombia::BancolombiaParser;
pub use nequi::NequiParser;
pub use nu::NuParser;
