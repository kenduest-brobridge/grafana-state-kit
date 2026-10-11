use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{message, string_field, value_as_object, Result};
use crate::dashboard::live::{
    fetch_dashboard_if_exists_with_request, fetch_folder_if_exists_with_request,
};
use crate::dashboard::DEFAULT_PAGE_SIZE;
use crate::grafana_api::DashboardResourceClient;

#[derive(Default)]
pub(crate) struct ImportLookupCache {
    pub dashboards_by_uid: BTreeMap<String, Option<Value>>,
    pub dashboard_uids_from_search: Option<BTreeSet<String>>,
    pub dashboard_summary_folder_uids: BTreeMap<String, String>,
    pub resolved_existing_dashboard_folder_paths: BTreeMap<String, String>,
    pub resolved_dashboard_import_folder_paths: BTreeMap<(String, bool), String>,
    pub folders_by_uid: BTreeMap<String, Option<Map<String, Value>>>,
    pub ensured_folder_uids: BTreeSet<String>,
    pub current_org_id: Option<String>,
    pub orgs: Option<Vec<Map<String, Value>>>,
}

fn load_dashboard_uid_summary_cache<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if cache.dashboard_uids_from_search.is_some() {
        return Ok(());
    }
    let summaries =
        crate::dashboard::list_dashboard_summaries_with_request(request_json, DEFAULT_PAGE_SIZE)?;
    store_dashboard_uid_summary_cache(summaries, cache);
    Ok(())
}

fn store_dashboard_uid_summary_cache(
    summaries: Vec<Map<String, Value>>,
    cache: &mut ImportLookupCache,
) {
    let mut dashboard_uids = BTreeSet::new();
    let mut folder_uids = BTreeMap::new();
    for summary in summaries {
        let uid = string_field(&summary, "uid", "");
        if uid.is_empty() {
            continue;
        }
        dashboard_uids.insert(uid.clone());
        let folder_uid = string_field(&summary, "folderUid", "");
        if !folder_uid.is_empty() {
            folder_uids.insert(uid, folder_uid);
        }
    }
    cache.dashboard_uids_from_search = Some(dashboard_uids);
    cache.dashboard_summary_folder_uids = folder_uids;
}

fn load_dashboard_uid_summary_cache_for_dashboard_client(
    client: &DashboardResourceClient<'_>,
    cache: &mut ImportLookupCache,
) -> Result<()> {
    if cache.dashboard_uids_from_search.is_some() {
        return Ok(());
    }
    let summaries = client.list_dashboard_summaries(DEFAULT_PAGE_SIZE)?;
    store_dashboard_uid_summary_cache(summaries, cache);
    Ok(())
}

pub(crate) fn dashboard_exists_with_summary<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
    uid: &str,
) -> Result<bool>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if cache.dashboards_by_uid.contains_key(uid) {
        let result = cache
            .dashboards_by_uid
            .get(uid)
            .is_some_and(|value| value.is_some());
        return Ok(result);
    }
    load_dashboard_uid_summary_cache(request_json, cache)?;
    let exists = cache
        .dashboard_uids_from_search
        .as_ref()
        .is_some_and(|known| known.contains(uid));
    Ok(exists)
}

pub(crate) fn dashboard_summary_folder_uid<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
    uid: &str,
) -> Result<Option<String>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    load_dashboard_uid_summary_cache(request_json, cache)?;
    Ok(cache.dashboard_summary_folder_uids.get(uid).cloned())
}

pub(crate) fn dashboard_summary_folder_uid_with_client(
    client: &DashboardResourceClient<'_>,
    cache: &mut ImportLookupCache,
    uid: &str,
) -> Result<Option<String>> {
    load_dashboard_uid_summary_cache_for_dashboard_client(client, cache)?;
    Ok(cache.dashboard_summary_folder_uids.get(uid).cloned())
}

pub(crate) fn fetch_dashboard_if_exists_cached<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
    uid: &str,
) -> Result<Option<Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if uid.is_empty() {
        return Ok(None);
    }
    if let Some(cached) = cache.dashboards_by_uid.get(uid) {
        return Ok(cached.clone());
    }
    if let Ok(exists) = dashboard_exists_with_summary(request_json, cache, uid) {
        if !exists {
            cache.dashboards_by_uid.insert(uid.to_string(), None);
            return Ok(None);
        }
    }
    let fetched = fetch_dashboard_if_exists_with_request(request_json, uid)?;
    cache
        .dashboards_by_uid
        .insert(uid.to_string(), fetched.clone());
    Ok(fetched)
}

pub(crate) fn fetch_dashboard_if_exists_cached_with_client(
    client: &DashboardResourceClient<'_>,
    cache: &mut ImportLookupCache,
    uid: &str,
) -> Result<Option<Value>> {
    if uid.is_empty() {
        return Ok(None);
    }
    if let Some(cached) = cache.dashboards_by_uid.get(uid) {
        return Ok(cached.clone());
    }
    if let Ok(exists) =
        load_dashboard_uid_summary_cache_for_dashboard_client(client, cache).map(|_| {
            cache
                .dashboard_uids_from_search
                .as_ref()
                .is_some_and(|known| known.contains(uid))
        })
    {
        if !exists {
            cache.dashboards_by_uid.insert(uid.to_string(), None);
            return Ok(None);
        }
    }
    let fetched = client.fetch_dashboard_if_exists(uid)?;
    cache
        .dashboards_by_uid
        .insert(uid.to_string(), fetched.clone());
    Ok(fetched)
}

pub(crate) fn fetch_folder_if_exists_cached<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
    uid: &str,
) -> Result<Option<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if uid.is_empty() {
        return Ok(None);
    }
    if let Some(cached) = cache.folders_by_uid.get(uid) {
        return Ok(cached.clone());
    }
    let fetched = fetch_folder_if_exists_with_request(request_json, uid)?;
    cache
        .folders_by_uid
        .insert(uid.to_string(), fetched.clone());
    Ok(fetched)
}

pub(crate) fn fetch_folder_if_exists_cached_with_client(
    client: &DashboardResourceClient<'_>,
    cache: &mut ImportLookupCache,
    uid: &str,
) -> Result<Option<Map<String, Value>>> {
    if uid.is_empty() {
        return Ok(None);
    }
    if let Some(cached) = cache.folders_by_uid.get(uid) {
        return Ok(cached.clone());
    }
    let fetched = client.fetch_folder_if_exists(uid)?;
    cache
        .folders_by_uid
        .insert(uid.to_string(), fetched.clone());
    Ok(fetched)
}

pub(crate) fn determine_dashboard_import_action_with_request<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
    payload: &Value,
    replace_existing: bool,
    update_existing_only: bool,
) -> Result<&'static str>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let payload_object =
        value_as_object(payload, "Dashboard import payload must be a JSON object.")?;
    let dashboard = payload_object
        .get("dashboard")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Dashboard import payload is missing dashboard."))?;
    let uid = string_field(dashboard, "uid", "");
    if uid.is_empty() {
        return Ok(if update_existing_only {
            "would-skip-missing"
        } else {
            "would-create"
        });
    }
    if !dashboard_exists_with_summary(request_json, cache, &uid)? {
        if update_existing_only {
            return Ok("would-skip-missing");
        }
        return Ok("would-create");
    }
    if replace_existing || update_existing_only {
        Ok("would-update")
    } else {
        Ok("would-fail-existing")
    }
}

pub(crate) fn determine_dashboard_import_action_with_client(
    client: &DashboardResourceClient<'_>,
    cache: &mut ImportLookupCache,
    payload: &Value,
    replace_existing: bool,
    update_existing_only: bool,
) -> Result<&'static str> {
    let payload_object =
        value_as_object(payload, "Dashboard import payload must be a JSON object.")?;
    let dashboard = payload_object
        .get("dashboard")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Dashboard import payload is missing dashboard."))?;
    let uid = string_field(dashboard, "uid", "");
    if uid.is_empty() {
        return Ok(if update_existing_only {
            "would-skip-missing"
        } else {
            "would-create"
        });
    }
    let exists = match fetch_dashboard_if_exists_cached_with_client(client, cache, &uid)? {
        Some(_) => true,
        None => {
            if let Some(known) = cache.dashboard_uids_from_search.as_ref() {
                known.contains(&uid)
            } else {
                false
            }
        }
    };
    if !exists {
        if update_existing_only {
            return Ok("would-skip-missing");
        }
        return Ok("would-create");
    }
    if replace_existing || update_existing_only {
        Ok("would-update")
    } else {
        Ok("would-fail-existing")
    }
}
