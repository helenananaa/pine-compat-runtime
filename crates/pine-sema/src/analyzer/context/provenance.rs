use super::Analyzer;
use pine_ir::CallSiteId;
use pine_syntax::Span;

impl Analyzer {
    pub(crate) fn alloc_call_site_at(&mut self, span: Span) -> CallSiteId {
        let id = self.alloc_call_site();
        if let Some((source_id, library_key)) = self
            .source_context_origins
            .get(&self.current_source_context_id())
        {
            self.call_site_sources.push(pine_ir::HirCallSiteSource {
                call_site_id: id,
                source_id: source_id.get(),
                library_key: library_key.clone(),
                start: span.start,
                end: span.end,
            });
        }
        id
    }
}
