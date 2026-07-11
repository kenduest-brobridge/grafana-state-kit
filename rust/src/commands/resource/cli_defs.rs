use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

use super::resource_help::{
    RESOURCE_DESCRIBE_AFTER_HELP, RESOURCE_GET_AFTER_HELP, RESOURCE_KINDS_AFTER_HELP,
    RESOURCE_LIST_AFTER_HELP,
};
use crate::common::CliColorChoice;
use crate::dashboard::CommonCliArgs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ResourceOutputFormat {
    Text,
    Table,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ResourceApiMode {
    Auto,
    Legacy,
    V1,
    V2,
    #[value(name = "v1beta1", alias = "v1-beta1")]
    V1Beta1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
pub enum ResourceKind {
    Dashboards,
    Folders,
    Datasources,
    #[value(name = "alert-rules")]
    AlertRules,
    Orgs,
    Receivers,
    #[value(name = "routing-trees")]
    RoutingTrees,
    #[value(name = "template-groups")]
    TemplateGroups,
    #[value(name = "time-intervals")]
    TimeIntervals,
    #[value(name = "inhibition-rules")]
    InhibitionRules,
    #[value(name = "secure-values")]
    SecureValues,
}

impl ResourceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Dashboards => "dashboards",
            Self::Folders => "folders",
            Self::Datasources => "datasources",
            Self::AlertRules => "alert-rules",
            Self::Orgs => "orgs",
            Self::Receivers => "receivers",
            Self::RoutingTrees => "routing-trees",
            Self::TemplateGroups => "template-groups",
            Self::TimeIntervals => "time-intervals",
            Self::InhibitionRules => "inhibition-rules",
            Self::SecureValues => "secure-values",
        }
    }

    pub(crate) fn singular_label(self) -> &'static str {
        match self {
            Self::Dashboards => "dashboard",
            Self::Folders => "folder",
            Self::Datasources => "datasource",
            Self::AlertRules => "alert-rule",
            Self::Orgs => "org",
            Self::Receivers => "receiver",
            Self::RoutingTrees => "routing-tree",
            Self::TemplateGroups => "template-group",
            Self::TimeIntervals => "time-interval",
            Self::InhibitionRules => "inhibition-rule",
            Self::SecureValues => "secure-value",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Dashboards => "Grafana dashboards from /api/search and /api/dashboards/uid/{uid}.",
            Self::Folders => "Grafana folders from /api/folders and /api/folders/{uid}.",
            Self::Datasources => "Grafana datasources from /api/datasources and /api/datasources/uid/{uid}.",
            Self::AlertRules => {
                "Grafana alert rules from /api/v1/provisioning/alert-rules and /api/v1/provisioning/alert-rules/{uid}."
            }
            Self::Orgs => "Grafana org inventory from /api/orgs and /api/orgs/{id}.",
            Self::Receivers => "Grafana Alerting receivers from notifications.alerting.grafana.app/v1beta1.",
            Self::RoutingTrees => "Grafana Alerting routing trees from notifications.alerting.grafana.app/v1beta1.",
            Self::TemplateGroups => "Grafana Alerting template groups from notifications.alerting.grafana.app/v1beta1.",
            Self::TimeIntervals => "Grafana Alerting time intervals from notifications.alerting.grafana.app/v1beta1.",
            Self::InhibitionRules => "Grafana Alerting inhibition rules from notifications.alerting.grafana.app/v1beta1.",
            Self::SecureValues => "Grafana secret metadata from secret.grafana.app/v1beta1; secret values are never returned.",
        }
    }

    pub(crate) fn selector_pattern(self) -> &'static str {
        match self {
            Self::Dashboards => "dashboards/<uid>",
            Self::Folders => "folders/<uid>",
            Self::Datasources => "datasources/<uid>",
            Self::AlertRules => "alert-rules/<uid>",
            Self::Orgs => "orgs/<id>",
            Self::Receivers => "receivers/<name>",
            Self::RoutingTrees => "routing-trees/<name>",
            Self::TemplateGroups => "template-groups/<name>",
            Self::TimeIntervals => "time-intervals/<name>",
            Self::InhibitionRules => "inhibition-rules/<name>",
            Self::SecureValues => "secure-values/<name>",
        }
    }

    pub(crate) fn list_endpoint(self) -> &'static str {
        match self {
            Self::Dashboards => "GET /api/search",
            Self::Folders => "GET /api/folders",
            Self::Datasources => "GET /api/datasources",
            Self::AlertRules => "GET /api/v1/provisioning/alert-rules",
            Self::Orgs => "GET /api/orgs",
            Self::Receivers => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/receivers",
            Self::RoutingTrees => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/routingtrees",
            Self::TemplateGroups => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/templategroups",
            Self::TimeIntervals => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/timeintervals",
            Self::InhibitionRules => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/inhibitionrules",
            Self::SecureValues => "GET /apis/secret.grafana.app/v1beta1/namespaces/{namespace}/securevalues",
        }
    }

    pub(crate) fn get_endpoint(self) -> &'static str {
        match self {
            Self::Dashboards => "GET /api/dashboards/uid/{uid}",
            Self::Folders => "GET /api/folders/{uid}",
            Self::Datasources => "GET /api/datasources/uid/{uid}",
            Self::AlertRules => "GET /api/v1/provisioning/alert-rules/{uid}",
            Self::Orgs => "GET /api/orgs/{id}",
            Self::Receivers => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/receivers/{name}",
            Self::RoutingTrees => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/routingtrees/{name}",
            Self::TemplateGroups => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/templategroups/{name}",
            Self::TimeIntervals => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/timeintervals/{name}",
            Self::InhibitionRules => "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/{namespace}/inhibitionrules/{name}",
            Self::SecureValues => "GET /apis/secret.grafana.app/v1beta1/namespaces/{namespace}/securevalues/{name}",
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct ResourceKindsArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Table,
        help = "Render supported resource kinds as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct ResourceDescribeArgs {
    #[arg(
        value_enum,
        help = "Optional resource kind to describe. Omit this to describe every supported kind."
    )]
    pub kind: Option<ResourceKind>,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Table,
        help = "Render resource descriptions as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct ResourceListArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        value_enum,
        help = "Grafana resource kind to list. Use grafana-util status resource describe to see the current selector patterns and endpoints."
    )]
    pub kind: ResourceKind,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Table,
        help = "Render live resource inventory as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceApiMode::Auto,
        help = "Choose API probing mode. auto uses primary endpoints and falls back to legacy alternatives where available; legacy prefers those alternatives when supported."
    )]
    pub api_mode: ResourceApiMode,
    #[arg(
        long,
        default_value = "default",
        help = "App Platform namespace used by v1, v2, or v1beta1 resource APIs."
    )]
    pub namespace: String,
}

#[derive(Debug, Clone, Args)]
pub struct ResourceGetArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        value_name = "SELECTOR",
        help = "Fetch one live resource by selector. Use <kind>/<identity>, for example dashboards/cpu-main or folders/infra. Run grafana-util status resource describe first if you need the supported selector patterns."
    )]
    pub selector: String,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Json,
        help = "Render the fetched live resource as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceApiMode::Auto,
        help = "Choose API probing mode. auto uses primary endpoints and falls back to legacy alternatives where available; legacy prefers those alternatives when supported."
    )]
    pub api_mode: ResourceApiMode,
    #[arg(
        long,
        default_value = "default",
        help = "App Platform namespace used by v1, v2, or v1beta1 resource APIs."
    )]
    pub namespace: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ResourceCommand {
    #[command(
        name = "kinds",
        about = "List the resource kinds supported by the generic read-only resource query surface.",
        after_help = RESOURCE_KINDS_AFTER_HELP
    )]
    Kinds(ResourceKindsArgs),
    #[command(
        name = "describe",
        about = "Describe the supported live Grafana resource kinds and selector patterns.",
        after_help = RESOURCE_DESCRIBE_AFTER_HELP
    )]
    Describe(ResourceDescribeArgs),
    #[command(
        name = "list",
        about = "List one supported live Grafana resource kind.",
        after_help = RESOURCE_LIST_AFTER_HELP
    )]
    List(ResourceListArgs),
    #[command(
        name = "get",
        about = "Fetch one supported live Grafana resource by selector.",
        after_help = RESOURCE_GET_AFTER_HELP
    )]
    Get(ResourceGetArgs),
}

#[derive(Debug, Clone, Parser)]
pub struct ResourceCliArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = CliColorChoice::Auto,
        help = "Colorize JSON and YAML output. Use auto, always, or never."
    )]
    pub color: CliColorChoice,
    #[command(subcommand)]
    pub command: ResourceCommand,
}
