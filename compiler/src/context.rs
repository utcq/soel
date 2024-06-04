use reports::{sourcemap::SourceMap, ReportContext};

pub struct CompilerContext {
    pub report_context: ReportContext,
    pub source_context: SourceMap,
}
