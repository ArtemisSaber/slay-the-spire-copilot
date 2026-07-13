pub(crate) fn log_map_paths(raw: &serde_json::Value, map_nodes: &[crate::state::MapCoord]) {
    let room_phase = raw
        .pointer("/game_state/room_phase")
        .and_then(|value| value.as_str())
        .unwrap_or("?");
    if room_phase != "COMPLETE" {
        tracing::debug!("MAP screen but room_phase={room_phase} or no current_node");
        return;
    }

    let first_chosen = raw
        .pointer("/game_state/screen_state/first_node_chosen")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);

    if first_chosen {
        let paths = raw
            .pointer("/game_state/screen_state/current_node")
            .and_then(|value| Some((value.get("x")?.as_i64()?, value.get("y")?.as_i64()?)))
            .map(|(x, y)| crate::prompt::enumerate_paths(x, y, map_nodes))
            .unwrap_or_default();
        tracing::info!(
            "MAP: {} paths from current_node, room_phase=COMPLETE",
            paths.len(),
        );
        for (index, path) in paths.iter().enumerate() {
            let route: Vec<String> = path
                .iter()
                .map(|node| format!("{}({},{})", node.symbol, node.x, node.y))
                .collect();
            let summary = crate::prompt::summarize_path(path);
            tracing::info!(
                "  Path {}: {}  [{}]",
                (b'A' + index as u8) as char,
                route.join(" → "),
                summary,
            );
        }
    } else {
        let roots = crate::prompt::enumerate_paths_from_roots(map_nodes);
        let total: usize = roots.iter().map(|root| root.paths.len()).sum();
        tracing::info!(
            "MAP: {} roots, {} paths total, room_phase=COMPLETE",
            roots.len(),
            total,
        );
        for (root_index, root_group) in roots.iter().enumerate() {
            let root_label = format!(
                "{}({},{})",
                root_group.root.symbol, root_group.root.x, root_group.root.y
            );
            tracing::info!(
                "  Root {} {}: {} paths",
                (b'A' + root_index as u8) as char,
                root_label,
                root_group.paths.len(),
            );
            for (path_index, path) in root_group.paths.iter().enumerate() {
                let route: Vec<String> = path
                    .iter()
                    .map(|node| format!("{}({},{})", node.symbol, node.x, node.y))
                    .collect();
                let summary = crate::prompt::summarize_path(path);
                tracing::info!(
                    "    Path {}.{}: {}  [{}]",
                    (b'A' + root_index as u8) as char,
                    path_index + 1,
                    route.join(" → "),
                    summary,
                );
            }
        }
    }
}
