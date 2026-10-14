use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryRowInsert<T> {
    pub insert_id: String,
    pub json_payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryStreamingRequest<T> {
    pub kind: String,
    pub skip_invalid_rows: bool,
    pub ignore_unknown_values: bool,
    pub rows: Vec<BigQueryRowInsert<T>>,
}

pub struct StreamBufferManager;

impl StreamBufferManager {
    /// Wrap rows into an idempotent BigQuery streaming insert payload
    pub fn build_insert_request<T>(
        rows: Vec<(String, T)>, // (event_id/insert_id, payload)
        skip_invalid: bool,
    ) -> BigQueryStreamingRequest<T> {
        let stream_rows = rows
            .into_iter()
            .map(|(id, payload)| BigQueryRowInsert {
                insert_id: id,
                json_payload: payload,
            })
            .collect();

        BigQueryStreamingRequest {
            kind: "bigquery#tableDataInsertAllRequest".to_string(),
            skip_invalid_rows: skip_invalid,
            ignore_unknown_values: true,
            rows: stream_rows,
        }
    }
}
