pub(super) fn refresh_autoplay_control(
    control_path: &std::path::Path,
    last_revision: &mut Option<u64>,
    current_control: &mut Option<crate::autoplay::control::AutoPlayControl>,
) -> crate::autoplay::control::ControlLoad {
    let load = crate::autoplay::control::load_control(control_path, *last_revision);
    if let Some(control) = crate::autoplay::action::active_control(&load) {
        *last_revision = Some(control.revision);
        *current_control = Some(control.clone());
    } else if matches!(load, crate::autoplay::control::ControlLoad::Malformed(_)) {
        *current_control = None;
    }
    load
}

pub(super) fn autoplay_load_status(load: &crate::autoplay::control::ControlLoad) -> String {
    match load {
        crate::autoplay::control::ControlLoad::Updated(_) => "updated".to_string(),
        crate::autoplay::control::ControlLoad::MissingDefault(_) => "missing_default".to_string(),
        crate::autoplay::control::ControlLoad::Stale => "stale".to_string(),
        crate::autoplay::control::ControlLoad::Malformed(error) => format!("malformed: {error}"),
    }
}

pub(super) fn autoplay_mode_name(
    control: Option<&crate::autoplay::control::AutoPlayControl>,
) -> String {
    control
        .map(|control| format!("{:?}", control.mode))
        .unwrap_or_else(|| "disabled".to_string())
}

pub(super) fn autoplay_allows_execution(
    control: Option<&crate::autoplay::control::AutoPlayControl>,
) -> bool {
    control.is_some_and(|control| control.mode == crate::autoplay::control::AutoPlayMode::Auto)
}
