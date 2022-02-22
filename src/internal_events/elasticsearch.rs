use metrics::counter;
use vector_core::internal_event::InternalEvent;

#[derive(Debug)]
pub(crate) struct ElasticsearchEventEncoded {
    pub(crate) byte_size: usize,
    pub(crate) index: String,
}

impl InternalEvent for ElasticsearchEventEncoded {
    fn emit_logs(&self) {
        trace!(message = "Inserting event.", index = %self.index);
    }

    fn emit_metrics(&self) {
        counter!("processed_bytes_total", self.byte_size as u64);
    }
}
