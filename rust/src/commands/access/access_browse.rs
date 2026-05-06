//! Access-wide interactive browse model and TUI shell.

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::Result;

use super::render::{
    map_get_text, normalize_org_role, normalize_service_account_row, normalize_team_row,
    normalize_user_row, scalar_text,
};
use super::{request_array, AccessBrowseArgs, Scope};

#[cfg(feature = "tui")]
use std::time::Duration;

#[cfg(feature = "tui")]
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[cfg(feature = "tui")]
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

#[cfg(feature = "tui")]
use super::browse_terminal::TerminalSession;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccessBrowseItem {
    pub(crate) kind: String,
    pub(crate) identity: String,
    pub(crate) summary: String,
    pub(crate) review: String,
}

impl AccessBrowseItem {
    fn matches_query(&self, query: &str) -> bool {
        let query = query.to_ascii_lowercase();
        self.kind.to_ascii_lowercase().contains(&query)
            || self.identity.to_ascii_lowercase().contains(&query)
            || self.summary.to_ascii_lowercase().contains(&query)
            || self.review.to_ascii_lowercase().contains(&query)
    }
}

fn enabled(args: &AccessBrowseArgs, flag: bool) -> bool {
    if args.include_users
        || args.include_teams
        || args.include_orgs
        || args.include_service_accounts
    {
        flag
    } else {
        true
    }
}

fn user_identity(row: &Map<String, Value>) -> String {
    let login = map_get_text(row, "login");
    if !login.is_empty() {
        return login;
    }
    let email = map_get_text(row, "email");
    if !email.is_empty() {
        return email;
    }
    scalar_text(row.get("id"))
}

fn shape_user_item(user: &Map<String, Value>, scope: &Scope) -> AccessBrowseItem {
    let row = normalize_user_row(user, scope);
    AccessBrowseItem {
        kind: "user".to_string(),
        identity: user_identity(&row),
        summary: format!(
            "email={} role={} admin={} scope={}",
            map_get_text(&row, "email"),
            map_get_text(&row, "orgRole"),
            map_get_text(&row, "grafanaAdmin"),
            map_get_text(&row, "scope")
        ),
        review: "review user org role, admin state, identity, and team membership".to_string(),
    }
}

fn shape_team_item(team: &Map<String, Value>) -> AccessBrowseItem {
    let row = normalize_team_row(team);
    AccessBrowseItem {
        kind: "team".to_string(),
        identity: map_get_text(&row, "name"),
        summary: format!(
            "id={} email={} members={}",
            map_get_text(&row, "id"),
            map_get_text(&row, "email"),
            map_get_text(&row, "memberCount")
        ),
        review: "review team contact and membership count before import or sync".to_string(),
    }
}

fn shape_org_item(org: &Map<String, Value>) -> AccessBrowseItem {
    let id = scalar_text(org.get("id").or_else(|| org.get("orgId")));
    let name = map_get_text(org, "name");
    let user_count = {
        let count = scalar_text(org.get("userCount"));
        if count.is_empty() {
            match org.get("users") {
                Some(Value::Array(users)) => users.len().to_string(),
                _ => "0".to_string(),
            }
        } else {
            count
        }
    };
    AccessBrowseItem {
        kind: "org".to_string(),
        identity: if name.is_empty() { id.clone() } else { name },
        summary: format!("id={id} users={user_count}"),
        review: "review org identity and membership count before global changes".to_string(),
    }
}

fn shape_service_account_item(account: &Map<String, Value>) -> AccessBrowseItem {
    let row = normalize_service_account_row(account);
    AccessBrowseItem {
        kind: "service-account".to_string(),
        identity: map_get_text(&row, "name"),
        summary: format!(
            "login={} role={} disabled={} tokens={} orgId={}",
            map_get_text(&row, "login"),
            normalize_org_role(row.get("role")),
            map_get_text(&row, "disabled"),
            map_get_text(&row, "tokens"),
            map_get_text(&row, "orgId")
        ),
        review: "review service-account role, disabled state, and token metadata only".to_string(),
    }
}

pub(crate) fn build_access_browse_items<F>(
    mut request_json: F,
    args: &AccessBrowseArgs,
) -> Result<Vec<AccessBrowseItem>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut items = Vec::new();
    if enabled(args, args.include_users) {
        let users = super::user::iter_global_users_with_request(&mut request_json, args.per_page)?;
        items.extend(
            users
                .iter()
                .map(|user| shape_user_item(user, &Scope::Global)),
        );
    }
    if enabled(args, args.include_teams) {
        let teams = super::team::iter_teams_with_request(&mut request_json, args.query.as_deref())?;
        items.extend(teams.iter().map(shape_team_item));
    }
    if enabled(args, args.include_orgs) {
        let orgs = request_array(
            &mut request_json,
            Method::GET,
            "/api/orgs",
            &[],
            None,
            "Unexpected organization list response from Grafana.",
        )?;
        items.extend(orgs.iter().map(shape_org_item));
    }
    if enabled(args, args.include_service_accounts) {
        let accounts =
            super::service_account::list_all_service_accounts_with_request(&mut request_json)?;
        items.extend(accounts.iter().map(shape_service_account_item));
    }
    if let Some(query) = args.query.as_deref() {
        let query = query.trim();
        if !query.is_empty() {
            items.retain(|item| item.matches_query(query));
        }
    }
    Ok(items)
}

#[cfg(feature = "tui")]
pub(crate) fn browse_access_with_request<F>(request_json: F, args: &AccessBrowseArgs) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let items = build_access_browse_items(request_json, args)?;
    let mut session = TerminalSession::enter()?;
    let mut selected = 0usize;
    loop {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(selected.min(items.len().saturating_sub(1))));
        }
        session.terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(5)])
                .split(frame.area());
            let rows = items
                .iter()
                .map(|item| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<15}", item.kind),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!(" {:<32} ", item.identity)),
                        Span::raw(&item.summary),
                    ]))
                })
                .collect::<Vec<_>>();
            let list = List::new(rows)
                .block(
                    Block::default()
                        .title("Access Browse")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            frame.render_stateful_widget(list, chunks[0], &mut state);

            let detail = items
                .get(selected)
                .map(|item| format!("{}: {}\n{}", item.kind, item.identity, item.review))
                .unwrap_or_else(|| "No access inventory rows matched.".to_string());
            frame.render_widget(
                Paragraph::new(detail)
                    .block(Block::default().title("Review").borders(Borders::ALL)),
                chunks[1],
            );
        })?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1).min(items.len().saturating_sub(1));
            }
            KeyCode::Up | KeyCode::Char('k') => selected = selected.saturating_sub(1),
            _ => {}
        }
    }
}

#[cfg(not(feature = "tui"))]
pub(crate) fn browse_access_with_request<F>(
    _request_json: F,
    _args: &AccessBrowseArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    Err(crate::common::tui(
        "Access browse requires the `tui` feature.",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::message;
    use serde_json::json;

    #[test]
    fn service_account_item_shows_token_metadata_without_secrets() {
        let account = json!({
            "id": 7,
            "name": "deploy",
            "login": "sa-deploy",
            "role": "Editor",
            "isDisabled": false,
            "tokens": 2,
            "key": "secret-token-value"
        });
        let item = shape_service_account_item(account.as_object().unwrap());

        assert_eq!(item.kind, "service-account");
        assert_eq!(item.identity, "deploy");
        assert!(item.summary.contains("tokens=2"));
        assert!(item.review.contains("token metadata only"));
        assert!(!item.summary.contains("secret-token-value"));
        assert!(!item.review.contains("secret-token-value"));
    }

    #[test]
    fn access_browse_items_can_consolidate_all_live_resource_rows() {
        let args = AccessBrowseArgs {
            common: super::super::CommonCliArgs {
                profile: None,
                url: "http://127.0.0.1:3000".to_string(),
                api_token: Some("token".to_string()),
                username: None,
                password: None,
                prompt_password: false,
                prompt_token: false,
                org_id: None,
                timeout: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            query: None,
            include_users: true,
            include_teams: true,
            include_orgs: true,
            include_service_accounts: true,
            per_page: 100,
        };
        let mut calls = Vec::new();
        let items = build_access_browse_items(
            |method, path, _params, _payload| {
                calls.push((method, path.to_string()));
                match path {
                    "/api/users" => Ok(Some(json!([
                        {"id": 1, "login": "alice", "email": "alice@example.com", "isAdmin": true}
                    ]))),
                    "/api/teams/search" => Ok(Some(json!({
                        "teams": [{"id": 2, "name": "ops", "email": "ops@example.com", "memberCount": 3}]
                    }))),
                    "/api/orgs" => Ok(Some(json!([
                        {"id": 3, "name": "Main Org.", "userCount": 9}
                    ]))),
                    "/api/serviceaccounts/search" => Ok(Some(json!({
                        "serviceAccounts": [{"id": 4, "name": "deploy", "login": "sa-deploy", "role": "Viewer", "tokens": 1}]
                    }))),
                    _ => Err(message(format!("unexpected request {path}"))),
                }
            },
            &args,
        )
        .unwrap();

        assert_eq!(
            items
                .iter()
                .map(|item| item.kind.as_str())
                .collect::<Vec<_>>(),
            vec!["user", "team", "org", "service-account"]
        );
        assert!(calls.iter().any(|(_, path)| path == "/api/orgs"));
    }
}
