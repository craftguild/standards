//! Compile the language-neutral conformance values into the tool without runtime files.

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    pub counter: String,
    pub version: String,
    pub metric: String,
    pub implementation_limit: usize,
}

pub fn measurement() -> &'static Measurement {
    static MEASUREMENT: OnceLock<Measurement> = OnceLock::new();
    MEASUREMENT.get_or_init(|| {
        let contract: serde_json::Value =
            serde_json::from_str(include_str!("../../../conformance/contracts.json"))
                .expect("Bundled conformance data must parse");
        let measurement: Measurement = serde_json::from_value(contract["measurement"].clone())
            .expect("Bundled measurement contract must have its exact schema");
        assert_eq!(measurement.counter, "tokei");
        assert_eq!(measurement.metric, "Code");
        measurement
    })
}
