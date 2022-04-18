#![doc = include_str!("../README.md")]
pub mod error;
pub mod external;

pub use durandal_derives::{CliDispatch, CliMetaDispatch};

/// This exists for the purpose of proving subcommands with a standard interface.
///
/// Generally, it should be possible to do something with `enum_dispatch`, to
/// make actually utilizing this trait more egronomic, but that's less possible
/// if allowing external subcommands. For commands that need to take additional
/// metadata, see [CliMetaCommand]
pub trait CliCommand {
    fn run(&self) -> anyhow::Result<()>;
}

/// Like the [CliCommand] trait, but additionally allowing metadata to be passed
/// to the underlying command.
///
/// One potential use would be for passing on top-level application config to
/// subcommands. Regardless of which trait is used, the choice should be
/// consistent for all subcommands for a given application to ease in making
/// calls to enum variants.
pub trait CliMetaCommand {
    type Meta;

    fn run(&self, meta: &Self::Meta) -> anyhow::Result<()>;
}
