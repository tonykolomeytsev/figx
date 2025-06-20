use crate::MetricsCollector;

impl MetricsCollector {
    pub fn to_prom(&self, labels: Option<&[(&'static str, &'static str)]>) -> String {
        let mut buf = String::with_capacity(8192);
        for entry in self.durations.iter() {
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
                    buf.push('=');
                    buf.push('"');
                    buf.push_str(v);
                    buf.push('"');
                }
                buf.push('}');
            } else {
                buf.push(' ');
            }
            buf.push_str(&value.get().as_millis().to_string());
        }
        for entry in self.counters.iter() {
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
                    buf.push('=');
                    buf.push('"');
                    buf.push_str(v);
                    buf.push('"');
                }
                buf.push('}');
            } else {
                buf.push(' ');
            }
            buf.push_str(&value.get().to_string());
        }

        buf
    }
}
