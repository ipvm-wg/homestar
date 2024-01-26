/// A module to parse prometheus metrics data into json
///
/// Influenced by https://crates.io/crates/prom2jsonrs/0.1.0.
use anyhow::{anyhow, bail, Result};
use dyn_clone::DynClone;
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap},
};

const HISTOGRAM_TYPE: &str = "HISTOGRAM";
const SUMMARY_TYPE: &str = "SUMMARY";

static METRIC_REGEX_NO_LABEL: Lazy<&Regex> = Lazy::new(|| {
    static RE: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| Regex::new(r"([a-zA-Z_:][a-zA-Z0-9_:]*)\s(-?[\d.]+(?:e-?\d+)?|NaN)").unwrap())
});

static METRIC_REGEX_WITH_LABEL: Lazy<&Regex> = Lazy::new(|| {
    static RE: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| {
        Regex::new(r"[a-zA-Z_:][a-zA-Z0-9_:]*\{(.*)\}\s(-?[\d.]+(?:e-?\d+)?|NaN)").unwrap()
    })
});

static LABELS_REGEX: Lazy<&Regex> = Lazy::new(|| {
    static RE: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| Regex::new("([a-zA-Z0-9_:]*)=\"([^\"]+)\"").unwrap())
});

static MULTI_NEWLINE: Lazy<&Regex> = Lazy::new(|| {
    static RE: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| Regex::new(r"\n\n").unwrap())
});

type Labels = HashMap<String, String>;
type Value = String;

#[derive(Clone, Serialize, JsonSchema)]
/// A parsed representation of the prometheus metrics data
#[allow(missing_debug_implementations)]
#[schemars(title = "Metrics data", description = "Prometheus metrics data")]
pub struct PrometheusData {
    metrics: Vec<MetricFamily>,
}

impl PrometheusData {
    /// Parse promethues metric data from string
    pub(crate) fn from_string(s: &str) -> Result<PrometheusData> {
        let text = MULTI_NEWLINE.replace_all(s, "\n");
        let mut metrics = Vec::new();
        let mut metric_lines = Vec::new();
        let mut num_comment_lines = 0;
        for line in text.lines() {
            if line.starts_with('#') {
                if num_comment_lines == 2 {
                    // One set complete
                    metrics.push(MetricFamily::from_raw(&metric_lines)?);
                    metric_lines = vec![line];
                    num_comment_lines = 1;
                } else {
                    num_comment_lines += 1;
                    metric_lines.push(line);
                }
            } else {
                metric_lines.push(line)
            }
        }
        Ok(PrometheusData { metrics })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Metric {
    labels: Option<Labels>,
    value: Value,
}

impl JsonSchema for Metric {
    fn schema_name() -> String {
        "gauge".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-runtime::network::webserver::prom::Metric")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let type_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
            const_value: Some(json!("metric")),
            ..Default::default()
        };

        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Gauge data".to_string()),
                description: Some("A gauge metric".to_string()),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([
                    ("type".to_string(), Schema::Object(type_schema)),
                    ("labels".to_string(), <Option<Labels>>::json_schema(gen)),
                    ("value".to_string(), <String>::json_schema(gen)),
                ]),
                required: BTreeSet::from(["type".to_string(), "value".to_string()]),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Summary {
    labels: Option<Labels>,
    quantiles: Labels,
    count: Value,
    sum: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Histogram {
    labels: Option<HashMap<String, String>>,
    buckets: Labels,
    count: Value,
    sum: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(title = "Metric type")]
enum MetricType {
    Gauge,
    Histogram,
    Summary,
}

#[derive(Clone, Serialize)]
struct MetricFamily {
    metric_type: MetricType,
    metric_name: String,
    help: String,
    data: Vec<Box<dyn MetricLike>>,
}

impl JsonSchema for MetricFamily {
    fn schema_name() -> String {
        "metric".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-runtime::network::webserver::prom::MetricFamily")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        struct DataConditional {
            if_schema: Schema,
            then_schema: Schema,
            else_schema: Schema,
        }

        fn data_conditional(gen: &mut SchemaGenerator) -> DataConditional {
            let if_schema = SchemaObject {
                instance_type: None,
                object: Some(Box::new(ObjectValidation {
                    properties: BTreeMap::from([(
                        "metric_type".to_owned(),
                        Schema::Object(SchemaObject {
                            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
                            const_value: Some(json!("gauge")),
                            ..Default::default()
                        }),
                    )]),
                    ..Default::default()
                })),
                ..Default::default()
            };

            let then_schema = SchemaObject {
                instance_type: None,
                object: Some(Box::new(ObjectValidation {
                    properties: BTreeMap::from([("data".to_string(), <Metric>::json_schema(gen))]),
                    ..Default::default()
                })),
                ..Default::default()
            };

            DataConditional {
                if_schema: Schema::Object(if_schema),
                then_schema: Schema::Object(then_schema),
                else_schema: Schema::Bool(false),
            }
        }

        let mut schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Metric family".to_string()),
                description: Some("A prometheus gauge, summary, or histogram metric".to_string()),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([
                    ("metric_type".to_string(), <MetricType>::json_schema(gen)),
                    ("metric_name".to_string(), <String>::json_schema(gen)),
                    ("help".to_string(), <String>::json_schema(gen)),
                ]),
                required: BTreeSet::from([
                    "metric_type".to_string(),
                    "metric_name".to_string(),
                    "help".to_string(),
                    "data".to_string(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        };

        let data = data_conditional(gen);
        schema.subschemas().if_schema = Some(Box::new(data.if_schema));
        schema.subschemas().then_schema = Some(Box::new(data.then_schema));
        schema.subschemas().else_schema = Some(Box::new(data.else_schema));

        schema.into()
    }
}

#[typetag::serde(tag = "type")]
trait MetricLike: DynClone {
    fn parse_from_string(s: &str) -> Result<(Value, Option<Labels>)>
    where
        Self: Sized,
    {
        if let Some(caps) = METRIC_REGEX_NO_LABEL.captures(s) {
            Ok((caps[2].to_string(), None))
        } else if let Some(caps) = METRIC_REGEX_WITH_LABEL.captures(s) {
            let value = caps[2].to_string();
            let mut labels: HashMap<String, String> = HashMap::new();
            for cap in LABELS_REGEX.captures_iter(&caps[1]) {
                labels.insert(cap[1].to_string(), cap[2].to_string());
            }
            Ok((value, Some(labels)))
        } else {
            Err(anyhow!("invalid format {}", s))
        }
    }

    fn metric_type() -> String
    where
        Self: Sized;
}

dyn_clone::clone_trait_object!(MetricLike);

impl Metric {
    fn from_string(s: &str) -> Result<Metric> {
        let (value, labels) = Self::parse_from_string(s)?;
        Ok(Metric { labels, value })
    }
}

#[typetag::serde(name = "metric")]
impl MetricLike for Metric {
    fn metric_type() -> String {
        String::from("DEFAULT")
    }
}

impl Summary {
    fn from_raw(metric_name: &str, raw_lines: &Vec<&str>) -> Result<Summary> {
        let mut sum = String::from("");
        let mut count = String::from("");
        let sum_prefix = format!("{}_sum", metric_name);
        let count_prefix = format!("{}_count", metric_name);
        let mut labels = HashMap::new();
        let mut quantiles = HashMap::new();
        for raw_line in raw_lines {
            if raw_line.starts_with(&sum_prefix) {
                sum = Summary::parse_from_string(raw_line)?.0;
            } else if raw_line.starts_with(&count_prefix) {
                count = Summary::parse_from_string(raw_line)?.0;
            } else if let Some(caps) = METRIC_REGEX_WITH_LABEL.captures(raw_line) {
                for cap in LABELS_REGEX.captures_iter(&caps[1]) {
                    let key = &cap[1];
                    let value = &cap[2];
                    match key {
                        "quantile" => quantiles.insert(key.to_string(), value.to_string()),
                        _ => labels.insert(key.to_string(), value.to_string()),
                    };
                }
            } else {
                bail!("invalid format {}", raw_line);
            }
        }

        Ok(Summary {
            sum,
            count,
            labels: Some(labels),
            quantiles,
        })
    }
}

#[typetag::serde]
impl MetricLike for Summary {
    fn metric_type() -> String {
        String::from(SUMMARY_TYPE)
    }
}

impl Histogram {
    fn from_raw(metric_name: &str, raw_lines: &Vec<&str>) -> Result<Histogram> {
        let mut sum = String::from("");
        let mut count = String::from("");
        let sum_prefix = format!("{}_sum", metric_name);
        let count_prefix = format!("{}_count", metric_name);
        let mut labels: HashMap<String, String> = HashMap::new();
        let mut buckets: HashMap<String, String> = HashMap::new();
        for raw_line in raw_lines {
            if raw_line.starts_with(&sum_prefix) {
                sum = Summary::parse_from_string(raw_line)?.0;
            } else if raw_line.starts_with(&count_prefix) {
                count = Summary::parse_from_string(raw_line)?.0;
            } else if let Some(caps) = METRIC_REGEX_WITH_LABEL.captures(raw_line) {
                for cap in LABELS_REGEX.captures_iter(&caps[1]) {
                    let key = &cap[1];
                    let value = &cap[2];
                    match key {
                        "le" => buckets.insert(value.to_string(), caps[2].to_string()),
                        _ => labels.insert(key.to_string(), value.to_string()),
                    };
                }
            } else {
                bail!("invalid format {}", raw_line)
            }
        }

        Ok(Histogram {
            sum,
            count,
            labels: Some(labels),
            buckets,
        })
    }
}

#[typetag::serde]
impl MetricLike for Histogram {
    fn metric_type() -> String {
        String::from(HISTOGRAM_TYPE)
    }
}

impl MetricFamily {
    fn from_raw(raw: &[&str]) -> Result<MetricFamily> {
        let mut raw_iter = raw.iter();
        let help = MetricFamily::metric_help_fron_raw(
            raw_iter
                .next()
                .ok_or(anyhow!("invalid metric help{}", raw.join("\n")))?,
        );
        let (metric_name, metric_type) = MetricFamily::metric_name_and_type(
            raw_iter
                .next()
                .ok_or(anyhow!("invalid metric name/type {}", raw.join("\n")))?,
        )?;
        let mut data: Vec<Box<dyn MetricLike>> = Vec::new();
        match metric_type {
            MetricType::Gauge => {
                for raw_line in raw_iter {
                    data.push(Box::new(Metric::from_string(raw_line)?))
                }
            }
            MetricType::Histogram => {
                let count_prefix = format!("{}_count", metric_name);
                let mut histogram_lines: Vec<&str> = Vec::new();
                for raw_line in raw_iter {
                    histogram_lines.push(raw_line);
                    if raw_line.starts_with(&count_prefix) {
                        data.push(Box::new(Histogram::from_raw(
                            &metric_name,
                            &histogram_lines,
                        )?));
                        histogram_lines = Vec::new();
                    }
                }
            }
            MetricType::Summary => {
                let count_prefix = format!("{}_count", metric_name);
                let mut summary_lines: Vec<&str> = Vec::new();
                for raw_line in raw_iter {
                    summary_lines.push(raw_line);
                    if raw_line.starts_with(&count_prefix) {
                        data.push(Box::new(Summary::from_raw(&metric_name, &summary_lines)?));
                        summary_lines = Vec::new();
                    }
                }
            }
        }
        Ok(MetricFamily {
            metric_type,
            metric_name,
            help,
            data,
        })
    }

    fn metric_name_and_type(type_line: &str) -> Result<(String, MetricType)> {
        let tags: Vec<&str> = type_line.split_whitespace().collect();
        let (name, type_raw) = (tags[2], tags[3]);
        let metric_type = match type_raw {
            "gauge" => MetricType::Gauge,
            "counter" => MetricType::Gauge,
            "histogram" => MetricType::Histogram,
            "summary" => MetricType::Summary,
            _ => bail!("invalid metric type {}", type_raw),
        };

        Ok((name.to_string(), metric_type))
    }

    fn metric_help_fron_raw(help_line: &str) -> String {
        let tags: Vec<&str> = help_line.split_whitespace().collect();
        tags[3..].join(" ").to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn parse_metric() {
        assert_eq!(
            Metric {
                labels: None,
                value: String::from("205632")
            },
            Metric::from_string("go_memstats_mspan_inuse_bytes 205632").unwrap()
        );
        assert_eq!(
            Metric {
                labels: Some(hashmap!{
                    "dialer_name".to_string() => "default".to_string(),
                    "reason".to_string() => "unknown".to_string(),
                }),
                value: String::from("0")
            },
            Metric::from_string("net_conntrack_dialer_conn_failed_total{dialer_name=\"default\",reason=\"unknown\"} 0").unwrap()
        )
    }

    #[test]
    fn parse_metric_raw_data() {
        let raw_data = "# HELP go_goroutines Number of goroutines that currently exist.
# TYPE go_goroutines gauge
go_goroutines 31
# HELP go_info Information about the Go environment.
# TYPE go_info gauge
go_info{version=\"go1.15.5\"} 1";
        let prom_data = PrometheusData::from_string(raw_data).unwrap();
        assert_eq!(MetricType::Gauge, prom_data.metrics[0].metric_type)
    }

    #[test]
    fn parse_metric_summary() {
        let raw_data =
            "prometheus_engine_query_duration_seconds{slice=\"inner_eval\",quantile=\"0.5\"} NaN
prometheus_engine_query_duration_seconds{slice=\"inner_eval\",quantile=\"0.9\"} NaN
prometheus_engine_query_duration_seconds{slice=\"inner_eval\",quantile=\"0.99\"} NaN
prometheus_engine_query_duration_seconds_sum{slice=\"inner_eval\"} 12
prometheus_engine_query_duration_seconds_count{slice=\"inner_eval\"} 0";
        let summary = Summary::from_raw(
            "prometheus_engine_query_duration_seconds",
            &raw_data.lines().collect(),
        )
        .unwrap();
        assert_eq!(summary.sum, "12".to_string());
        assert_eq!(
            summary.labels,
            Some(hashmap! {"slice".to_string() => "inner_eval".to_string()})
        );
    }

    #[test]
    fn parse_metric_histogram() {
        let raw_data = r#"prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="0.1"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="0.2"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="0.4"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="1"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="3"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="8"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="20"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="60"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="120"} 10871
prometheus_http_request_duration_seconds_bucket{handler="/metrics",le="+Inf"} 10871
prometheus_http_request_duration_seconds_sum{handler="/metrics"} 67.48398663499978
prometheus_http_request_duration_seconds_count{handler="/metrics"} 10871"#;
        let histogram = Histogram::from_raw(
            "prometheus_http_request_duration_seconds",
            &raw_data.lines().collect(),
        )
        .unwrap();
        assert_eq!(histogram.sum, "67.48398663499978");
        assert_eq!(
            histogram.labels,
            Some(hashmap! {"handler".to_string() => "/metrics".to_string()})
        );
    }

    #[test]
    fn parse_metric_collection_to_json() {
        let raw_data = r#"# HELP homestar_process_disk_total_read_bytes Total bytes read from disk.
# TYPE homestar_process_disk_total_read_bytes gauge
homestar_process_disk_total_read_bytes 45969408

# HELP homestar_process_virtual_memory_bytes The virtual memory size in bytes.
# TYPE homestar_process_virtual_memory_bytes gauge
homestar_process_virtual_memory_bytes 418935930880

# HELP homestar_network_received_bytes The bytes received since last refresh.
# TYPE homestar_network_received_bytes gauge
homestar_network_received_bytes 0

# HELP homestar_system_available_memory_bytes The amount of available memory.
# TYPE homestar_system_available_memory_bytes gauge
homestar_system_available_memory_bytes 0

# HELP homestar_system_disk_available_space_bytes The total amount of available disk space.
# TYPE homestar_system_disk_available_space_bytes gauge
homestar_system_disk_available_space_bytes 0

# HELP homestar_system_load_average_percentage The load average over a five minute interval.
# TYPE homestar_system_load_average_percentage gauge
homestar_system_load_average_percentage 6.26611328125"#;

        let prom_data = PrometheusData::from_string(raw_data).unwrap();
        let json_string = serde_json::to_string(&prom_data).unwrap();
        let root: serde_json::Value = serde_json::from_str(&json_string).unwrap();

        let check = root
            .get("metrics")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("value"))
            .unwrap();

        assert_eq!(check, &serde_json::Value::String("45969408".to_string()));
    }
}
