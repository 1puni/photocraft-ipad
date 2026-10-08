//! Typed and spoken commands share the actual editor catalogue. No second command registry.
use photocraft_ui_egui::{PhotocraftApp, menus, state::Tool};
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Tool(Tool),
    Command(String),
}
#[derive(Clone, Debug)]
pub struct Choice {
    pub action: Action,
    pub label: String,
    pub path: String,
    pub enabled: bool,
    pub exact: bool,
    score: u16,
}
fn normalized(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if !out.ends_with(' ') {
            out.push(' ');
        }
    }
    out.trim().to_owned()
}
fn query(s: &str) -> String {
    let mut s = normalized(s);
    for prefix in [
        "please ",
        "switch to ",
        "change to ",
        "activate ",
        "use ",
        "run ",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_owned();
        }
    }
    s
}
fn score(query: &str, label: &str, path: &str, id: &str) -> u16 {
    if query.is_empty() {
        return 0;
    }
    let label = normalized(label);
    let path = normalized(&format!("{path} {label}"));
    if query == normalized(id) {
        400
    } else if query == label || query == path {
        300
    } else if label.starts_with(query) {
        200
    } else if query.split_whitespace().all(|word| path.contains(word)) {
        100
    } else {
        0
    }
}
pub fn find(app: &PhotocraftApp, text: &str, last_selection: Tool) -> Vec<Choice> {
    let q = query(text);
    if q.is_empty() {
        return Vec::new();
    }
    let mut choices = Vec::new();
    for tool in Tool::ALL {
        let name = normalized(tool.label());
        let short = name.strip_suffix(" tool").unwrap_or(&name);
        let rank = if q == name {
            400
        } else if q == short {
            300
        } else {
            score(&q, tool.label(), "Tools", "")
        };
        if rank > 0 {
            choices.push(Choice {
                action: Action::Tool(tool),
                label: tool.label().into(),
                path: "Tools".into(),
                enabled: true,
                exact: rank >= 300,
                score: rank,
            });
        }
    }
    // The broad Selection instrument recalls the last concrete selection tool, as a group slot
    // does in a conventional toolbar. Individual tools still resolve by their actual names.
    if q == "selection" || q == "selection tool" {
        choices.retain(|c| c.action != Action::Tool(last_selection));
        choices.push(Choice {
            action: Action::Tool(last_selection),
            label: last_selection.label().into(),
            path: "Last selection tool".into(),
            enabled: true,
            exact: true,
            score: 500,
        });
    }
    let mut seen = std::collections::HashSet::new();
    for item in menus::menu_items(app) {
        if item.id.is_empty()
            || item.label == "---"
            || !menus::is_live(&item.id)
            || !seen.insert(item.id.clone())
        {
            continue;
        }
        let path = item.path.join(" / ");
        let rank = score(&q, &item.label, &path, &item.id);
        if rank > 0 {
            choices.push(Choice {
                action: Action::Command(item.id),
                label: item.label,
                path,
                enabled: item.enabled,
                exact: rank >= 300,
                score: rank,
            });
        }
    }
    choices.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.label.cmp(&b.label))
            .then(a.path.cmp(&b.path))
    });
    choices
}
/// Only an unambiguous, exact tool switch happens without a tap on a result. All document/menu
/// actions remain reviewable with their real label and menu path, including destructive names.
pub fn immediate_tool(choices: &[Choice]) -> Option<Tool> {
    let first = choices.first()?;
    if !first.exact || choices.get(1).is_some_and(|next| next.score == first.score) {
        return None;
    }
    match first.action {
        Action::Tool(tool) => Some(tool),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> PhotocraftApp {
        PhotocraftApp::new(
            photocraft_engine::Session::new(),
            photocraft_ui_egui::Services::default(),
        )
    }
    #[test]
    fn every_tool_resolves_and_selection_remembers_its_instrument() {
        let app = app();
        for tool in Tool::ALL {
            assert_eq!(
                immediate_tool(&find(&app, tool.label(), Tool::Lasso)),
                Some(tool),
                "{}",
                tool.label()
            );
        }
        assert_eq!(
            immediate_tool(&find(&app, "Please switch to brush tool.", Tool::Lasso)),
            Some(Tool::Brush)
        );
        assert_eq!(
            immediate_tool(&find(&app, "selection tool", Tool::Lasso)),
            Some(Tool::Lasso)
        );
    }
    #[test]
    fn document_actions_are_catalogued_and_never_inferred_as_tool_switches() {
        let mut app = app();
        app.run("file.new", serde_json::json!({"width":32,"height":32}))
            .unwrap();
        for id in [
            "select.all",
            "filter.blur.gaussianBlur",
            "layer.delete",
            "file.save",
            "image.adjustments.levels",
        ] {
            let found = find(&app, id, Tool::RectMarquee);
            assert!(
                found.iter().any(|c| c.action == Action::Command(id.into())),
                "missing {id}"
            );
            assert_eq!(immediate_tool(&found), None);
        }
        assert!(find(&app, "nothing with this improbable name", Tool::RectMarquee).is_empty());
        assert!(find(&app, "delete", Tool::RectMarquee).len() > 1);
    }

    #[test]
    fn blur_is_ambiguous_and_exposes_distinct_filters() {
        let mut app = app();
        app.run("file.new", serde_json::json!({"width":32,"height":32}))
            .unwrap();
        let found = find(&app, "blur", Tool::RectMarquee);
        assert_eq!(
            immediate_tool(&found),
            None,
            "Blur names a tool and a filter"
        );
        for id in [
            "filter.blur.boxBlur",
            "filter.blur.gaussianBlur",
            "filter.blur.motionBlur",
            "filter.blur.lensBlur",
        ] {
            assert!(
                found
                    .iter()
                    .any(|c| c.action == Action::Command(id.into()) && c.enabled),
                "{id}"
            );
        }
        assert_eq!(
            immediate_tool(&find(&app, "blur tool", Tool::RectMarquee)),
            Some(Tool::Blur)
        );
    }
}
