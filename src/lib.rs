pub mod dip;
pub mod isp;
pub mod lsp;
pub mod ocp;
pub mod srp;

#[derive(Debug, PartialEq, Eq)]
pub struct RuleError(pub &'static str);
impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for RuleError {}

pub mod console;
