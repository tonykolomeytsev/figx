use crate::MetricsCollector;
use dashmap::DashMap;
use std::{fs::File, io::Write, path::Path, sync::Arc};

impl MetricsCollector {
    pub fn export_as_prometheus(
        &self,
        labels: Option<&[(&'static str, &'static str)]>,
        path: &Path,
    ) -> std::io::Result<()> {
        let mut buf = String::with_capacity(8192);
        to_prometheus_string(&mut buf, &self.durations, labels, |d| {
            d.get().as_millis().to_string()
        });
        to_prometheus_string(&mut buf, &self.counters, labels, |c| c.get().to_string());
        let mut file = File::create(path)?;
        file.write_all(buf.as_bytes())
    }
}

fn to_prometheus_string<T>(
    buf: &mut String,
    metrics: &DashMap<&'static str, Arc<T>>,
    labels: Option<&[(&'static str, &'static str)]>,
    ser: impl Fn(&T) -> String,
) {
    for entry in metrics.iter() {
        let key = entry.key();
        let value = entry.value();

        buf.push_str(key);
        if let Some(labels) = labels {
            buf.push('{');
            for (idx, (k, v)) in labels.iter().enumerate() {
                if idx > 0 {
                    buf.push(',');
                }
                buf.push_str(k);
                buf.push_str(r#"=""#);
                buf.push_str(v);
                buf.push('"');
            }
            buf.push_str("} ");
        } else {
            buf.push(' ');
        }
        buf.push_str(&ser(&value));
        buf.push('\n');
    }
}
