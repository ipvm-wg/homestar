//! Styled, output response for console table.

use std::{
    fmt,
    io::{self, Write},
};
use tabled::{
    settings::{
        object::Rows,
        style::{BorderColor, BorderSpanCorrection},
        Alignment, Color, Modify, Panel, Style,
    },
    Table,
};

/// Panel title for the output table.
pub(crate) const TABLE_TITLE: &str = "homestar(╯°□°)╯";

/// Output response wrapper.
#[derive(Debug, Clone, PartialEq)]
pub struct Output(String);

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.trim_end())
    }
}

impl Output {
    /// Create a new output response.
    pub(crate) fn new(table: String) -> Self {
        Self(table)
    }

    /// Get the inner string as a reference.
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &str {
        &self.0
    }

    /// Print ouput response to console via [io::stdout].
    pub(crate) fn echo(&self) -> Result<(), io::Error> {
        let stdout = io::stdout();
        let mut handle = io::BufWriter::new(stdout);
        writeln!(handle, "{}", self.0)
    }
}

/// Trait for console table output responses.
pub trait ConsoleTable {
    /// Get the table as an output response.
    fn table(&self) -> Output;
    /// Print the table to console.
    fn echo_table(&self) -> Result<(), io::Error>;
}

/// Style trait for console table output responses.
#[allow(dead_code)]
pub(crate) trait ApplyStyle {
    fn default(&mut self) -> Output;
    fn default_with_title(&mut self, ext_title: &str) -> Output;
}

impl ApplyStyle for Table {
    fn default(&mut self) -> Output {
        let table = self
            .with(Style::modern())
            .with(Panel::header(TABLE_TITLE))
            .with(Modify::new(Rows::first()).with(Alignment::left()))
            .with(BorderColor::filled(Color::FG_WHITE))
            .with(BorderSpanCorrection)
            .to_string();

        Output(table)
    }

    fn default_with_title(&mut self, ext_title: &str) -> Output {
        let table = self
            .with(Style::modern())
            .with(Panel::header(format!("{TABLE_TITLE} - {ext_title}")))
            .with(Modify::new(Rows::first()).with(Alignment::left()))
            .with(BorderColor::filled(Color::FG_WHITE))
            .with(BorderSpanCorrection)
            .to_string();

        Output(table)
    }
}
