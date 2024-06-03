pub mod sourcemap;

use std::{borrow::Cow, convert::Infallible, ops::Range};

use sourcemap::SourceKey;

pub type Span = Range<usize>;

pub trait IntoReport {
    fn into_report(self) -> Report;
}

impl IntoReport for Report {
    fn into_report(self) -> Report {
        self
    }
}

impl IntoReport for Infallible {
    fn into_report(self) -> Report {
        match self {}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    span: Span,
    source_key: SourceKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    level: Level,
    span: Span,
    source_key: SourceKey,
    title: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    infos: Vec<Label>,
}

impl Report {
    pub fn new(
        level: Level,
        location: Span,
        source_key: impl Into<SourceKey>,
        title: impl Into<Cow<'static, str>>,
        description: Option<impl Into<Cow<'static, str>>>,
    ) -> Self {
        Self {
            level,
            span: location,
            source_key: source_key.into(),
            title: title.into(),
            description: description.map(Into::into),
            infos: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: impl Into<Label>) -> Self {
        self.infos.push(label.into());

        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub info: Cow<'static, str>,
    pub span: Option<Span>,
    pub src_key: SourceKey,
}

impl Label {
    pub fn new(info: impl Into<Cow<'static, str>>, span: Option<Span>, src_key: SourceKey) -> Self {
        Self {
            info: info.into(),
            span,
            src_key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    Error,
    Warn,
    Advice,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Position {
    Span(Span),
    Line(usize),
}

#[derive(Debug, Clone, Default)]
pub struct ReportContext {
    reports: Vec<Report>,
}

impl ReportContext {
    pub fn has_errors(&self) -> bool {
        self.reports
            .iter()
            .any(|report| matches!(report.level, Level::Error))
    }

    pub fn has_reports(&self) -> bool {
        !self.reports.is_empty()
    }
}

impl core::ops::DerefMut for ReportContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reports
    }
}

impl core::ops::Deref for ReportContext {
    type Target = Vec<Report>;

    fn deref(&self) -> &Self::Target {
        &self.reports
    }
}

impl Extend<Report> for ReportContext {
    fn extend<T: IntoIterator<Item = Report>>(&mut self, iter: T) {
        self.reports.extend(iter)
    }
}

pub mod realtime_logger {
    pub fn loading_const(cst: i16, reg: swriter::Registers) {
        println!("[SOEL_RT] Loading {} into {:?}", cst, reg);
    }
    pub fn loading_var(var: String, offset:u16, reg: swriter::Registers) {
        println!("[SOEL_RT] Moving var {} [Y+{}] into {:?}", var, offset, reg);
    }
}