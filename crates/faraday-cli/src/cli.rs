use clap::{Parser, Subcommand, ValueEnum};
use faraday_core::Module;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "faraday",
    version,
    about = "OBD-II diagnostics for Ford Fusion 2017 SEL",
    long_about = "A command-line tool for automotive diagnostics and configuration through FORScan-compatible OBD-II adapters.\n\nSupports vLinker FS, ELM327-compatible adapters, and other FORScan-compatible hardware."
)]
pub struct Args {
    #[arg(
        short,
        long,
        global = true,
        help = "Increase verbosity (-v, -vv, -vvv)",
        action = clap::ArgAction::Count
    )]
    pub verbose: u8,

    #[arg(
        long,
        global = true,
        env = "FARADAY_ADAPTER",
        default_value = "/dev/ttyUSB0",
        help = "OBD-II adapter device path or port"
    )]
    pub adapter: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "read-dtc", about = "Read diagnostic trouble codes")]
    ReadDtc {
        #[arg(long, value_enum, default_value = "pcm", help = "Target module")]
        module: ModuleArg,

        #[arg(long, help = "Read stored DTCs", action = clap::ArgAction::SetTrue)]
        stored: bool,

        #[arg(long, help = "Read pending DTCs", action = clap::ArgAction::SetTrue)]
        pending: bool,

        #[arg(long, help = "Read permanent DTCs", action = clap::ArgAction::SetTrue)]
        permanent: bool,
    },

    #[command(name = "clear-dtc", about = "Clear diagnostic trouble codes")]
    ClearDtc {
        #[arg(long, value_enum, default_value = "pcm", help = "Target module")]
        module: ModuleArg,
    },

    #[command(name = "live", about = "Read live data PIDs")]
    Live {
        #[arg(
            help = "Comma-separated list of PID hex values (e.g., 0C,0D,05)",
            value_delimiter = ','
        )]
        pids: Vec<String>,

        #[arg(long, value_enum, default_value = "pcm", help = "Target module")]
        module: ModuleArg,

        #[arg(
            long,
            short,
            default_value = "1000",
            help = "Read interval in milliseconds"
        )]
        interval: u64,
    },

    #[command(name = "vin", about = "Read vehicle identification number")]
    Vin {
        #[arg(
            long,
            value_enum,
            default_value = "j1979",
            help = "Method to use for reading VIN"
        )]
        method: VinMethod,
    },

    #[command(name = "read-did", about = "Read data identifier via UDS")]
    ReadDid {
        #[arg(long, value_enum, help = "Target module")]
        module: ModuleArg,

        #[arg(help = "Data identifier in hex format (e.g., F190)")]
        did: String,
    },

    #[command(name = "session", about = "Control diagnostic session")]
    Session {
        #[arg(long, value_enum, help = "Target module")]
        module: ModuleArg,

        #[arg(value_enum, help = "Diagnostic session type")]
        session: SessionArg,
    },

    #[command(name = "asbuilt", about = "Read as-built module configuration")]
    Asbuilt {
        #[command(subcommand)]
        action: AsBuiltAction,
    },

    #[command(
        name = "profile",
        about = "Apply or validate YAML configuration profiles"
    )]
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Subcommand, Clone)]
pub enum AsBuiltAction {
    #[command(name = "dump", about = "Dump all as-built blocks for a module as YAML")]
    Dump {
        #[arg(long, value_enum, help = "Target module (bcm, ipc, pcm)")]
        module: ModuleArg,
    },

    #[command(name = "show", about = "Show a specific feature value from a module")]
    Show {
        #[arg(long, value_enum, help = "Target module (bcm, ipc, pcm)")]
        module: ModuleArg,

        #[arg(long, help = "Feature name (e.g. drl_enabled)")]
        feature: String,
    },

    #[command(name = "write", about = "Write a feature value to a module")]
    Write {
        #[arg(long, value_enum, help = "Target module (bcm, ipc, pcm)")]
        module: ModuleArg,

        #[arg(long, help = "Feature name (e.g. drl_enabled)")]
        feature: String,

        #[arg(long, help = "New value (e.g. true, false, 0, 1, mph)")]
        value: String,

        #[arg(long, help = "Preview changes without writing to the vehicle")]
        dry_run: bool,

        #[arg(long, short = 'y', help = "Skip confirmation prompt")]
        yes: bool,
    },

    #[command(name = "snapshot", about = "Capture an as-built snapshot for a module")]
    Snapshot {
        #[arg(long, value_enum, help = "Target module (bcm, ipc, pcm)")]
        module: ModuleArg,

        #[arg(
            long,
            help = "Output file path (default: ~/.local/share/faraday/snapshots/)"
        )]
        output: Option<PathBuf>,
    },

    #[command(
        name = "restore",
        about = "Restore module configuration from a snapshot file"
    )]
    Restore {
        #[arg(help = "Path to snapshot JSON file")]
        snapshot: PathBuf,

        #[arg(long, short = 'y', help = "Skip confirmation prompt")]
        yes: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum ProfileAction {
    #[command(name = "apply", about = "Apply a YAML profile to the vehicle")]
    Apply {
        #[arg(help = "Path to YAML profile file")]
        file: PathBuf,

        #[arg(long, help = "Preview changes without writing to the vehicle")]
        dry_run: bool,

        #[arg(long, short = 'y', help = "Skip confirmation prompt")]
        yes: bool,
    },

    #[command(
        name = "validate",
        about = "Validate a YAML profile without connecting to the vehicle"
    )]
    Validate {
        #[arg(help = "Path to YAML profile file")]
        file: PathBuf,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ModuleArg {
    Pcm,
    Tcm,
    Abs,
    Rcm,
    Pscm,
    Bcm,
    Ipc,
    Apim,
    Pam,
    Dsm,
}

impl From<ModuleArg> for Module {
    fn from(arg: ModuleArg) -> Self {
        match arg {
            ModuleArg::Pcm => Module::Pcm,
            ModuleArg::Tcm => Module::Tcm,
            ModuleArg::Abs => Module::Abs,
            ModuleArg::Rcm => Module::Rcm,
            ModuleArg::Pscm => Module::Pscm,
            ModuleArg::Bcm => Module::Bcm,
            ModuleArg::Ipc => Module::Ipc,
            ModuleArg::Apim => Module::Apim,
            ModuleArg::Pam => Module::Pam,
            ModuleArg::Dsm => Module::Dsm,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum VinMethod {
    J1979,
    Uds,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SessionArg {
    Default,
    Programming,
    Extended,
}

impl From<SessionArg> for faraday_core::protocol::uds::DiagnosticSession {
    fn from(arg: SessionArg) -> Self {
        match arg {
            SessionArg::Default => faraday_core::protocol::uds::DiagnosticSession::Default,
            SessionArg::Programming => faraday_core::protocol::uds::DiagnosticSession::Programming,
            SessionArg::Extended => faraday_core::protocol::uds::DiagnosticSession::Extended,
        }
    }
}
