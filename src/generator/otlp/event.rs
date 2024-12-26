use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// MetricsData is the data structure for the metrics data.
///
/// Ref. https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/metrics/v1/metrics.proto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub resource_metrics: Vec<ResourceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub resource: Resource,
    pub scope_metrics: Vec<ScopeMetrics>,
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub attributes: serde_json::Value,
    pub dropped_attributes_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeMetrics {
    pub scope: Scope,
    pub metrics: Vec<Metric>,
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    pub version: String,
    pub attributes: serde_json::Value,
    pub dropped_attributes_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub gauge: Gauge,
    pub sum: Sum,
    pub histogram: Histogram,
    pub exponential_histogram: ExponentialHistogram,
    pub summary: Summary,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gauge {
    pub data_points: Vec<NumberDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sum {
    pub data_points: Vec<NumberDataPoint>,
    pub aggregation_temporality: i32,
    pub is_monotonic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub data_points: Vec<HistogramDataPoint>,
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialHistogram {
    pub data_points: Vec<ExponentialHistogramDataPoint>,
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub data_points: Vec<SummaryDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberDataPoint {
    pub attributes: serde_json::Value,
    pub start_time_unix_nano: i64,
    pub time_unix_nano: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_double: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_int: Option<i64>,
    pub exemplars: Vec<Exemplar>,
    pub flags: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramDataPoint {
    pub attributes: serde_json::Value,
    pub start_time_unix_nano: i64,
    pub time_unix_nano: i64,
    pub count: String,
    pub sum: f64,
    pub bucket_counts: Vec<String>,
    pub explicit_bounds: Vec<f64>,
    pub exemplars: Vec<Exemplar>,
    pub flags: i64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialHistogramDataPoint {
    pub attributes: serde_json::Value,
    pub start_time_unix_nano: i64,
    pub time_unix_nano: i64,
    pub count: String,
    pub sum: f64,
    pub scale: i32,
    pub zero_count: String,
    pub positive: ExponentialBucket,
    pub negative: ExponentialBucket,
    pub flags: i64,
    pub exemplars: Vec<Exemplar>,
    pub min: f64,
    pub max: f64,
    pub zero_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialBucket {
    pub offset: i32,
    pub bucket_counts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryDataPoint {
    pub attributes: serde_json::Value,
    pub start_time_unix_nano: i64,
    pub time_unix_nano: i64,
    pub count: String,
    pub sum: f64,
    pub quantile_values: Vec<QuantileValue>,
    pub flags: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantileValue {
    pub quantile: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exemplar {
    pub filtered_attributes: serde_json::Value,
    pub time_unix_nano: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_double: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_int: Option<i64>,
    pub span_id: Vec<u8>,
    pub trace_id: Vec<u8>,
}

impl ResourceMetrics {
    pub fn generate(event_id: usize) -> Self {
        let mut rng = SmallRng::seed_from_u64(event_id as u64);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        ResourceMetrics {
            resource: Resource {
                attributes: serde_json::json!({
                    "service.name": "example-service",
                    "service.version": "1.0.0",
                    "host.name": format!("host-{}", rng.gen_range(1..100))
                }),
                dropped_attributes_count: 0,
            },
            scope_metrics: vec![ScopeMetrics {
                scope: Scope {
                    name: "example.scope".to_string(),
                    version: "1.0.0".to_string(),
                    attributes: serde_json::json!({}),
                    dropped_attributes_count: 0,
                },
                metrics: vec![
                    Self::generate_gauge_metric(&mut rng, now),
                    Self::generate_histogram_metric(&mut rng, now),
                ],
                schema_url: "https://opentelemetry.io/schemas/1.0.0".to_string(),
            }],
            schema_url: "https://opentelemetry.io/schemas/1.0.0".to_string(),
        }
    }

    fn generate_gauge_metric(rng: &mut SmallRng, now: i64) -> Metric {
        let cpu_id = rng.gen_range(0..16);
        let states = ["idle", "user", "system", "iowait", "steal"];
        let state = states[rng.gen_range(0..states.len())];

        Metric {
            name: "system.cpu.usage".to_string(),
            description: "CPU usage percentage".to_string(),
            unit: "%".to_string(),
            gauge: Gauge {
                data_points: vec![NumberDataPoint {
                    attributes: serde_json::json!({
                        "cpu": format!("cpu{}", cpu_id),
                        "state": state
                    }),
                    start_time_unix_nano: now - rng.gen_range(30_000_000_000..120_000_000_000),
                    time_unix_nano: now,
                    as_double: Some(rng.gen_range(0.0..100.0)),
                    as_int: Some(rng.gen_range(0..100)),
                    exemplars: vec![],
                    flags: rng.gen_range(0..4),
                }],
            },
            sum: Sum {
                data_points: vec![],
                aggregation_temporality: rng.gen_range(1..4),
                is_monotonic: rng.gen_bool(0.5),
            },
            histogram: Histogram {
                data_points: vec![],
                aggregation_temporality: rng.gen_range(1..4),
            },
            exponential_histogram: ExponentialHistogram {
                data_points: vec![],
                aggregation_temporality: rng.gen_range(1..4),
            },
            summary: Summary {
                data_points: vec![],
            },
            metadata: serde_json::json!({
                "collector_name": format!("collector-{}", rng.gen_range(1..10)),
                "region": format!("region-{}", rng.gen_range(1..5))
            }),
        }
    }

    fn generate_histogram_metric(rng: &mut SmallRng, now: i64) -> Metric {
        let endpoints = ["/api/v1", "/api/v2", "/health", "/metrics", "/status"];
        let endpoint = endpoints[rng.gen_range(0..endpoints.len())];
        let count = rng.gen_range(50..1000);
        let sum: f64 = rng.gen_range(500.0..5000.0);

        let method = ["GET", "POST", "PUT", "DELETE"][rng.gen_range(0..4)];
        let status_code = [200, 201, 400, 401, 403, 404, 500][rng.gen_range(0..7)];

        Metric {
            name: "http.request.duration".to_string(),
            description: "HTTP request duration histogram".to_string(),
            unit: "ms".to_string(),
            gauge: Gauge {
                data_points: vec![],
            },
            sum: Sum {
                data_points: vec![],
                aggregation_temporality: rng.gen_range(1..4),
                is_monotonic: rng.gen_bool(0.5),
            },
            histogram: Histogram {
                data_points: vec![HistogramDataPoint {
                    attributes: serde_json::json!({
                        "endpoint": endpoint,
                        "method": method,
                        "status_code": status_code,
                    }),
                    start_time_unix_nano: now - rng.gen_range(30_000_000_000..120_000_000_000),
                    time_unix_nano: now,
                    count: count.to_string(),
                    sum,
                    bucket_counts: (0..5)
                        .map(|_| rng.gen_range(1..count / 4).to_string())
                        .collect(),
                    explicit_bounds: vec![0.0, 50.0, 100.0, 250.0, 500.0],
                    exemplars: vec![],
                    flags: rng.gen_range(0..4),
                    min: rng.gen_range(1.0..10.0),
                    max: rng.gen_range(400.0..1000.0),
                }],
                aggregation_temporality: rng.gen_range(1..4),
            },
            exponential_histogram: ExponentialHistogram {
                data_points: vec![],
                aggregation_temporality: rng.gen_range(1..4),
            },
            summary: Summary {
                data_points: vec![],
            },
            metadata: serde_json::json!({
                "collector_name": format!("collector-{}", rng.gen_range(1..10)),
                "region": format!("region-{}", rng.gen_range(1..5))
            }),
        }
    }
}

impl MetricsData {
    pub fn generate(event_id: usize, batch_size: usize) -> Self {
        let mut resource_metrics = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            resource_metrics.push(ResourceMetrics::generate(event_id + i));
        }
        MetricsData { resource_metrics }
    }
}
