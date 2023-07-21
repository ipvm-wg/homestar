use std::{
    io::{self, Write},
    net::SocketAddr,
};
use tabled::{
    settings::{
        object::Rows,
        style::{BorderColor, BorderSpanCorrection},
        themes::Colorization,
        Alignment, Color, Modify, Panel, Style,
    },
    Table, Tabled,
};

const TABLE_TITLE: &str = "homestar(╯°□°)╯";

/// Output response wrapper.
pub(crate) struct Output(String);

impl Output {
    /// Print ouput response to console via [io::stdout].
    pub(crate) fn echo(&self) -> Result<(), io::Error> {
        let stdout = io::stdout();
        let mut handle = io::BufWriter::new(stdout);
        writeln!(handle, "{}", self.0)
    }
}

/// Ping response for display.
#[derive(Tabled)]
pub(crate) struct Ping {
    address: SocketAddr,
    response: String,
}

trait ApplyStyle {
    fn default(&mut self) -> Output;
}

impl ApplyStyle for Table {
    fn default(&mut self) -> Output {
        let table = self
            .with(Style::modern())
            .with(Panel::header(TABLE_TITLE))
            .with(Modify::new(Rows::first()).with(Alignment::left()))
            .with(Colorization::exact([Color::FG_WHITE], Rows::first()))
            .with(Colorization::exact(
                [Color::FG_BRIGHT_GREEN],
                Rows::single(1),
            ))
            .with(BorderColor::filled(Color::FG_WHITE))
            .with(BorderSpanCorrection)
            .to_string();

        Output(table)
    }
}

impl Ping {
    /// Display a singleton table of a `ping` response.
    pub(crate) fn table(address: SocketAddr, response: String) -> Output {
        Table::new(vec![Self { address, response }]).default()
    }
}
