mod api;
mod cli;
mod data;
mod event;
mod ui;
mod workflow;

use crate::intent::interview::types::Profile;

pub(super) fn template_for_profile(profile: Profile) -> String {
    match profile {
        Profile::Api => api::template(),
        Profile::Cli => cli::template(),
        Profile::Event => event::template(),
        Profile::Data => data::template(),
        Profile::Workflow => workflow::template(),
        Profile::Ui => ui::template(),
    }
}
