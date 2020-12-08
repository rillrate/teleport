use crate::logparser::LogFormat;
use clap::Clap;
use strum_macros::EnumString;

#[derive(Clap, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Format {
    EnvLogger,
    PrettyEnvLogger,
}

impl Into<LogFormat> for Format {
    fn into(self) -> LogFormat {
        match self {
            Self::EnvLogger => LogFormat {
                pattern: r"^\[(?P<ts>\S+) (?P<lvl>\S+) (?P<path>\S+)\] (?P<msg>.+)$",
                separator: "::",
            },
            Self::PrettyEnvLogger => LogFormat {
                pattern: r"^(?P<ts>) (?P<lvl>\S+) (?P<path>\S+)\s+> (?P<msg>.+)$",
                separator: "::",
            },
        }
    }
}

#[derive(Clap)]
pub struct Opts {
    #[clap(long)]
    pub name: Option<String>,
    #[clap(long)]
    pub format: Format,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Stdin,
}
