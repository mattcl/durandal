pub mod error;
pub mod external;

/// This exists for the purpose of proving subcommands with a standard interface.
///
/// Generally, it should be possible to do something with `enum_dispatch`, to
/// make actually utilizing this trait more egronomic, but that's less possible
/// if allowing external subcommands.
pub trait CliCommand {
    fn run(&self) -> anyhow::Result<()>;
}
