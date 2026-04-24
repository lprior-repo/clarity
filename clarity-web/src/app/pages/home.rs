#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::components::quality::{QualityScoreBar, MINIMUM_GATE};
use crate::components::PlanningCoach;
use crate::lattice::quality::{
  calculate_quality, EarsRequirementRef, InversionControl, QualityScore,
};
use crate::types::{prompt_steps, Answer, RightTab, PHASES, TABS};

use super::header::{render_header, HeaderRenderData};
use super::phase_nav::{is_phase_done, render_phase_button, PhaseButtonData};
use super::tab_panel::{render_tab_button, render_tab_content, TabButtonData};

#[component]
pub fn HomePage() -> Element {
  let active_phase = use_signal(|| String::from("discover"));
  let answers = use_signal(Vec::<Answer>::new);
  let right_tab = use_signal(|| RightTab::Plan);
  let ears_requirements = use_signal(Vec::<EarsRequirementRef>::new);
  let quality_score = use_signal(|| Option::<QualityScore>::None);

  use_effect({
    let mut quality_score = quality_score;
    move || {
      let answers_clone = answers.read().clone();
      let ears_clone = ears_requirements.read().clone();
      if answers_clone.is_empty() {
        *quality_score.write() = None;
        return;
      }
      let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
      };
      match calculate_quality(&answers_clone, &ears_clone, &inversion) {
        Ok(score) => quality_score.set(Some(score)),
        Err(_) => quality_score.set(None),
      }
    }
  });

  let total_required = prompt_steps().iter().filter(|s| s.required).count();
  let total_done = answers
    .read()
    .iter()
    .filter(|a| {
      prompt_steps()
        .iter()
        .any(|s| s.id == a.step_id && s.required && a.value != "(skipped)")
    })
    .count();

  let passes_gate: bool = quality_score
    .read()
    .as_ref()
    .is_some_and(|s: &QualityScore| s.passes(MINIMUM_GATE));

  let active_phase_val = active_phase.read();
  let phase_buttons_data: Vec<PhaseButtonData> = PHASES
    .iter()
    .enumerate()
    .map(|(i, phase)| {
      let is_done = is_phase_done(phase.key, &answers.read());
      let is_active = *active_phase_val == phase.key;
      let (is_disabled, disabled_reason) = if phase.key == "develop"
        && is_phase_done("discover", &answers.read())
        && !passes_gate
      {
        (
          true,
          Some(format!(
            "Quality score must be at least {MINIMUM_GATE} to proceed"
          )),
        )
      } else {
        (false, None)
      };
      PhaseButtonData {
        key: phase.key.to_string(),
        label: phase.label.to_string(),
        index: i,
        is_done,
        is_active,
        is_disabled,
        disabled_reason,
      }
    })
    .collect();
  drop(active_phase_val);

  let active_phase_for_buttons = active_phase;
  let phase_buttons: Vec<Element> = phase_buttons_data
    .iter()
    .map(|data| render_phase_button(data, active_phase_for_buttons))
    .collect();

  let right_tab_val = right_tab();
  let tab_buttons: Vec<Element> = TABS
    .iter()
    .map(|tab| TabButtonData {
      key: tab.key,
      label: tab.label.to_string(),
      is_active: right_tab_val == tab.key,
      right_tab_signal: right_tab,
    })
    .map(render_tab_button)
    .collect();

  let current_tab = right_tab();
  let show_quality_details = use_signal(|| false);
  let overall_score: Option<u8> = quality_score.read().as_ref().map(|s| s.overall);
  let quality_badge_class: &'static str = overall_score.map_or("", |score| {
    if score >= MINIMUM_GATE {
      "text-chart-2 border-chart-2/30 bg-chart-2/10"
    } else if score >= 50 {
      "text-chart-3 border-chart-3/30 bg-chart-3/10"
    } else {
      "text-chart-4 border-chart-4/30 bg-chart-4/10"
    }
  });

  rsx! {
      div { class: "flex h-screen flex-col overflow-hidden bg-background",
          {render_header(HeaderRenderData {
              phase_buttons,
              total_done,
              total_required,
              overall_score,
              quality_badge_class,
              active_phase,
          })}
          div { class: "flex flex-1 overflow-hidden",
              main { class: "flex-1 overflow-hidden border-r border-border flex flex-col",
                  if *active_phase.read() == "discover" {
                      div {
                          class: "shrink-0 border-b border-border bg-muted/30 px-6 py-4",
                          QualityScoreBar {
                              score: quality_score,
                              minimum_gate: MINIMUM_GATE,
                              show_details: show_quality_details,
                          }
                      }
                  }
                  div {
                      class: "flex-1 overflow-hidden",
                      PlanningCoach {
                          active_phase: active_phase,
                          answers: answers,
                          mut_answers: answers,
                          mut_active_phase: active_phase
                      }
                  }
              }
              div { class: "flex w-[440px] shrink-0 flex-col lg:w-[500px]",
                  div { class: "flex shrink-0 items-center border-b border-border",
                      for button in tab_buttons.iter() {
                          {button.clone()}
                      }
                  }
                  div { class: "flex-1 overflow-hidden",
                      {render_tab_content(current_tab, answers, active_phase)}
                  }
              }
          }
      }
  }
}
