use std::time::SystemTime;

#[cfg(any(feature = "tonic", feature = "http"))]
pub mod tonic {
    use opentelemetry_api::global;
    use opentelemetry_api::metrics::MetricsError;
    use opentelemetry_proto::tonic::common::v1::InstrumentationScope as TonicInstrumentationScope;
    use opentelemetry_proto::tonic::metrics::v1::exemplar::Value as TonicExemplarValue;
    use opentelemetry_proto::tonic::metrics::v1::number_data_point::Value as TonicNumberDataPointValue;
    use opentelemetry_proto::tonic::metrics::v1::{
        metric::Data as TonicData, Metric as TonicMetric, ResourceMetrics as TonicResourceMetrics,
        ScopeMetrics as TonicScopeMetrics,
    };
    use opentelemetry_proto::tonic::metrics::v1::{
        AggregationTemporality as TonicTemporality, DataPointFlags as TonicDataPointFlags,
        Exemplar as TonicExemplar, Gauge as TonicGauge, Histogram as TonicHistogram,
        HistogramDataPoint as TonicHistogramDataPoint, NumberDataPoint as TonicNumberDataPoint,
        Sum as TonicSum,
    };
    use opentelemetry_proto::tonic::resource::v1::Resource as TonicResource;
    use opentelemetry_sdk::metrics::data::{Aggregation, Exemplar, Gauge, Histogram, Metric, Sum};
    use opentelemetry_sdk::{
        metrics::data::{ResourceMetrics, ScopeMetrics},
        Resource,
    };

    use super::to_nanos;

    pub fn convert_metrics(metrics: &ResourceMetrics) -> TonicResourceMetrics {
        TonicResourceMetrics {
            resource: convert_resource(&metrics.resource),
            scope_metrics: metrics
                .scope_metrics
                .iter()
                .map(convert_scope_metrics)
                .collect(),
            schema_url: metrics
                .resource
                .schema_url()
                .map(Into::into)
                .unwrap_or_default(),
        }
    }

    pub fn convert_resource(resource: &Resource) -> Option<TonicResource> {
        if resource.is_empty() {
            return None;
        }
        Some(TonicResource {
            attributes: resource.iter().map(Into::into).collect(),
            dropped_attributes_count: 0,
        })
    }

    pub fn convert_scope_metrics(scope_metrics: &ScopeMetrics) -> TonicScopeMetrics {
        TonicScopeMetrics {
            scope: Some(TonicInstrumentationScope::from(&scope_metrics.scope)),
            metrics: scope_metrics.metrics.iter().map(convert_metric).collect(),
            schema_url: scope_metrics
                .scope
                .schema_url
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
        }
    }

    pub fn convert_metric(metric: &Metric) -> TonicMetric {
        TonicMetric {
            name: metric.name.to_string(),
            description: metric.description.to_string(),
            unit: metric.unit.as_str().to_owned(),
            data: convert_data(&*metric.data),
        }
    }

    pub fn convert_data(data: &dyn Aggregation) -> Option<TonicData> {
        let any = data.as_any();
        if let Some(gauge) = any.downcast_ref::<Gauge<i64>>() {
            Some(TonicData::Gauge(convert_data_gauge(gauge)))
        } else if let Some(gauge) = any.downcast_ref::<Gauge<u64>>() {
            Some(TonicData::Gauge(convert_data_gauge(gauge)))
        } else if let Some(gauge) = any.downcast_ref::<Gauge<f64>>() {
            Some(TonicData::Gauge(convert_data_gauge(gauge)))
        } else if let Some(histogram) = any.downcast_ref::<Histogram<i64>>() {
            Some(TonicData::Histogram(convert_data_histogram(histogram)))
        } else if let Some(histogram) = any.downcast_ref::<Histogram<u64>>() {
            Some(TonicData::Histogram(convert_data_histogram(histogram)))
        } else if let Some(histogram) = any.downcast_ref::<Histogram<f64>>() {
            Some(TonicData::Histogram(convert_data_histogram(histogram)))
        } else if let Some(sum) = any.downcast_ref::<Sum<i64>>() {
            Some(TonicData::Sum(convert_data_sum(sum)))
        } else if let Some(sum) = any.downcast_ref::<Sum<u64>>() {
            Some(TonicData::Sum(convert_data_sum(sum)))
        } else if let Some(sum) = any.downcast_ref::<Sum<f64>>() {
            Some(TonicData::Sum(convert_data_sum(sum)))
        } else {
            global::handle_error(MetricsError::Other(format!("Unsupported data: {:?}", data)));
            None
        }
    }

    trait IntoF64 {
        fn into_f64(self) -> f64;
    }

    impl IntoF64 for f64 {
        fn into_f64(self) -> f64 {
            self
        }
    }

    impl IntoF64 for i64 {
        fn into_f64(self) -> f64 {
            self as f64
        }
    }

    impl IntoF64 for u64 {
        fn into_f64(self) -> f64 {
            self as f64
        }
    }

    fn convert_data_gauge<T>(gauge: &Gauge<T>) -> TonicGauge
    where
        T: Into<TonicExemplarValue> + Into<TonicNumberDataPointValue> + Copy,
    {
        TonicGauge {
            data_points: gauge
                .data_points
                .iter()
                .map(|p| TonicNumberDataPoint {
                    attributes: p.attributes.iter().map(Into::into).collect(),
                    start_time_unix_nano: p.start_time.and_then(to_nanos).unwrap_or_default(),
                    time_unix_nano: p.time.and_then(to_nanos).unwrap_or_default(),
                    exemplars: p.exemplars.iter().map(convert_exemplar).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(p.value.into()),
                })
                .collect(),
        }
    }

    fn convert_data_histogram<T>(histogram: &Histogram<T>) -> TonicHistogram
    where
        T: IntoF64 + Into<TonicExemplarValue> + Copy,
    {
        TonicHistogram {
            data_points: histogram
                .data_points
                .iter()
                .map(|p| TonicHistogramDataPoint {
                    attributes: p.attributes.iter().map(Into::into).collect(),
                    start_time_unix_nano: to_nanos(p.start_time).unwrap_or_default(),
                    time_unix_nano: to_nanos(p.time).unwrap_or_default(),
                    count: p.count,
                    sum: Some(p.sum.into_f64()),
                    bucket_counts: p.bucket_counts.clone(),
                    explicit_bounds: p.bounds.clone(),
                    exemplars: p.exemplars.iter().map(convert_exemplar).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    min: p.min.map(IntoF64::into_f64),
                    max: p.max.map(IntoF64::into_f64),
                })
                .collect(),
            aggregation_temporality: TonicTemporality::from(histogram.temporality).into(),
        }
    }

    fn convert_data_sum<T>(sum: &Sum<T>) -> TonicSum
    where
        T: Into<TonicExemplarValue> + Into<TonicNumberDataPointValue> + Copy,
    {
        TonicSum {
            data_points: sum
                .data_points
                .iter()
                .map(|p| TonicNumberDataPoint {
                    attributes: p.attributes.iter().map(Into::into).collect(),
                    start_time_unix_nano: p.start_time.and_then(to_nanos).unwrap_or_default(),
                    time_unix_nano: p.time.and_then(to_nanos).unwrap_or_default(),
                    exemplars: p.exemplars.iter().map(convert_exemplar).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(p.value.into()),
                })
                .collect(),
            aggregation_temporality: TonicTemporality::from(sum.temporality).into(),
            is_monotonic: sum.is_monotonic,
        }
    }

    fn convert_exemplar<T>(exemplar: &Exemplar<T>) -> TonicExemplar
    where
        T: Into<TonicExemplarValue> + Copy,
    {
        TonicExemplar {
            filtered_attributes: exemplar
                .filtered_attributes
                .iter()
                .map(Into::into)
                .collect(),
            time_unix_nano: to_nanos(exemplar.time).unwrap_or_default(),
            span_id: exemplar.span_id.into(),
            trace_id: exemplar.trace_id.into(),
            value: Some(exemplar.value.into()),
        }
    }
}

fn to_nanos(time: SystemTime) -> Option<u64> {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .ok()
}
