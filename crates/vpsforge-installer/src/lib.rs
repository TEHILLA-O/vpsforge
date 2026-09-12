//! Plan construction, idempotent apply, recommendations, and doctor.

mod doctor;
mod executor;
mod planner;
mod recommend;
mod status;

use vpsforge_core::{ForgeResult, HostFacts, InstallPlan};
use vpsforge_profiles::{Profile, ProfileFlags};

pub use doctor::{ai_doctor, system_doctor};
pub use executor::{apply_plan, ApplyOptions};
pub use planner::{build_custom_plan, build_plan, custom_catalog};
pub use recommend::{postgres_tuning, PostgresTuning};
pub use status::status_report;

pub fn plan_profiles(
    host: &HostFacts,
    profiles: &[Profile],
    flags: &ProfileFlags,
) -> ForgeResult<InstallPlan> {
    planner::build_plan(host, profiles, flags)
}
