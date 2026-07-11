use std::collections::BTreeSet;

use reqwest::Method;
use serde_json::Value;

use crate::common::{message, Result};
use crate::http::JsonHttpClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppPlatformApiVersion {
    V1,
    V2,
    V1Beta1,
}

impl AppPlatformApiVersion {
    pub(crate) const fn path_segment(self) -> &'static str {
        match self {
            Self::V1 => "v1",
            Self::V2 => "v2",
            Self::V1Beta1 => "v1beta1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AppPlatformResource {
    group: &'static str,
    version: AppPlatformApiVersion,
    plural: &'static str,
}

impl AppPlatformResource {
    pub(crate) const fn new(
        group: &'static str,
        version: AppPlatformApiVersion,
        plural: &'static str,
    ) -> Self {
        Self {
            group,
            version,
            plural,
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn version(self) -> AppPlatformApiVersion {
        self.version
    }

    fn collection_path(self, namespace: &str) -> String {
        format!(
            "/apis/{}/{}/namespaces/{namespace}/{}",
            self.group,
            self.version.path_segment(),
            self.plural
        )
    }

    fn item_path(self, namespace: &str, name: &str) -> String {
        format!("{}/{name}", self.collection_path(namespace))
    }
}

pub(crate) struct AppPlatformResourceClient<'a> {
    http: &'a JsonHttpClient,
}

impl<'a> AppPlatformResourceClient<'a> {
    pub(crate) fn new(http: &'a JsonHttpClient) -> Self {
        Self { http }
    }

    #[allow(dead_code)]
    pub(crate) fn is_served(&self, resource: AppPlatformResource, namespace: &str) -> Result<bool> {
        let path = resource.collection_path(namespace);
        match self.http.request_json(
            Method::GET,
            &path,
            &[("limit".to_string(), "1".to_string())],
            None,
        ) {
            Ok(Some(Value::Object(_))) => Ok(true),
            Ok(Some(_)) => Err(message(format!(
                "Unexpected App Platform discovery payload from {path}."
            ))),
            Ok(None) => Err(message(format!(
                "Unexpected empty App Platform discovery payload from {path}."
            ))),
            Err(error) if error.status_code() == Some(404) => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn list_all(
        &self,
        resource: AppPlatformResource,
        namespace: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Value>> {
        self.list_all_with_params(resource, namespace, limit, Vec::new())
    }

    pub(crate) fn get(
        &self,
        resource: AppPlatformResource,
        namespace: &str,
        name: &str,
    ) -> Result<Value> {
        self.get_with_params(resource, namespace, name, &[])
    }

    #[allow(dead_code)]
    pub(crate) fn get_if_exists(
        &self,
        resource: AppPlatformResource,
        namespace: &str,
        name: &str,
    ) -> Result<Option<Value>> {
        match self.get(resource, namespace, name) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.status_code() == Some(404) => Ok(None),
            Err(error) => Err(error),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn list_history(
        &self,
        resource: AppPlatformResource,
        namespace: &str,
        name: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Value>> {
        if resource.version() == AppPlatformApiVersion::V1Beta1 {
            return Err(message(
                "Resource history requires a stable v1 or v2 App Platform API.",
            ));
        }
        self.list_all_with_params(
            resource,
            namespace,
            limit,
            vec![
                ("fieldSelector".to_string(), format!("metadata.name={name}")),
                (
                    "labelSelector".to_string(),
                    "grafana.app/get-history=true".to_string(),
                ),
            ],
        )
    }

    #[allow(dead_code)]
    pub(crate) fn get_at_resource_version(
        &self,
        resource: AppPlatformResource,
        namespace: &str,
        name: &str,
        resource_version: &str,
    ) -> Result<Value> {
        if resource.version() == AppPlatformApiVersion::V1Beta1 {
            return Err(message(
                "Historical reads require a stable v1 or v2 App Platform API.",
            ));
        }
        self.get_with_params(
            resource,
            namespace,
            name,
            &[("resourceVersion".to_string(), resource_version.to_string())],
        )
    }

    fn list_all_with_params(
        &self,
        resource: AppPlatformResource,
        namespace: &str,
        limit: Option<usize>,
        base_params: Vec<(String, String)>,
    ) -> Result<Vec<Value>> {
        let path = resource.collection_path(namespace);
        let mut items = Vec::new();
        let mut continuation: Option<String> = None;
        let mut seen_tokens = BTreeSet::new();

        loop {
            let mut params = base_params.clone();
            if let Some(limit) = limit {
                params.push(("limit".to_string(), limit.to_string()));
            }
            if let Some(token) = continuation.as_deref() {
                params.push(("continue".to_string(), token.to_string()));
            }
            let response = self.http.request_json(Method::GET, &path, &params, None)?;
            let object = match response {
                Some(Value::Object(object)) => object,
                Some(_) => {
                    return Err(message(format!(
                        "Unexpected App Platform collection payload from {path}."
                    )))
                }
                None => {
                    return Err(message(format!(
                        "Unexpected empty App Platform collection payload from {path}."
                    )))
                }
            };
            let page_items = object
                .get("items")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    message(format!(
                        "App Platform collection payload from {path} is missing an items array."
                    ))
                })?;
            items.extend(page_items.iter().cloned());

            continuation = object
                .get("metadata")
                .and_then(Value::as_object)
                .and_then(|metadata| metadata.get("continue"))
                .and_then(Value::as_str)
                .filter(|token| !token.is_empty())
                .map(str::to_string);
            let Some(token) = continuation.as_ref() else {
                break;
            };
            if !seen_tokens.insert(token.clone()) {
                return Err(message(format!(
                    "App Platform collection from {path} repeated continue token `{token}`."
                )));
            }
        }

        Ok(items)
    }

    fn get_with_params(
        &self,
        resource: AppPlatformResource,
        namespace: &str,
        name: &str,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = resource.item_path(namespace, name);
        match self.http.request_json(Method::GET, &path, params, None)? {
            Some(Value::Object(object)) => Ok(Value::Object(object)),
            Some(_) => Err(message(format!(
                "Unexpected App Platform resource payload from {path}."
            ))),
            None => Err(message(format!(
                "Unexpected empty App Platform resource payload from {path}."
            ))),
        }
    }
}
