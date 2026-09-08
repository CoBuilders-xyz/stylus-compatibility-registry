pub mod async_usage;
pub mod float_usage;
pub mod no_std;
pub mod wasm_target;

use crate::registry::KnownCrateEntry;
use crate::types::{CheckResult, CrateInfo};

pub trait CrateCheck {
    fn name(&self) -> &str;
    fn run(&self, crate_info: &CrateInfo) -> CheckResult;
}

/// Runs every compatibility check against a crate.
///
/// `entry` is the crate's registry record when it has one. Three of the four checks
/// prefer it over their built-in blocklist, so passing `None` means blocklists only.
pub fn run_all_checks(crate_info: &CrateInfo, entry: Option<&KnownCrateEntry>) -> Vec<CheckResult> {
    vec![
        no_std::NoStdCheck.check_against_registry(crate_info, entry),
        wasm_target::WasmTargetCheck.run(crate_info),
        float_usage::FloatUsageCheck.check_against_registry(crate_info, entry),
        async_usage::AsyncUsageCheck.check_against_registry(crate_info, entry),
    ]
}
