pub mod async_usage;
pub mod float_usage;
pub mod no_std;
pub mod wasm_target;

use crate::types::{CheckResult, CrateInfo};

pub trait CrateCheck {
    fn name(&self) -> &str;
    fn run(&self, crate_info: &CrateInfo) -> CheckResult;
}

pub fn all_checks() -> Vec<Box<dyn CrateCheck>> {
    vec![
        Box::new(no_std::NoStdCheck),
        Box::new(wasm_target::WasmTargetCheck),
        Box::new(float_usage::FloatUsageCheck),
        Box::new(async_usage::AsyncUsageCheck),
    ]
}

pub fn run_all_checks(crate_info: &CrateInfo) -> Vec<CheckResult> {
    all_checks().iter().map(|c| c.run(crate_info)).collect()
}
