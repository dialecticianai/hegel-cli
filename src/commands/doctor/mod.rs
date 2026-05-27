#[cfg(test)]
mod tests;

mod fix_cowboy_durations;
mod fix_ddd;
mod fix_phases;
mod fix_state;

use anyhow::Result;
use clap::Args;

use crate::storage::FileStorage;

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Apply fixes (without this flag, only detection is performed)
    #[arg(long)]
    pub apply: bool,

    /// Show verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn doctor_command(args: DoctorArgs, storage: &FileStorage) -> Result<()> {
    // Check and fix state file first
    if let Some(report) =
        fix_state::check_and_fix_state(storage, args.apply, args.verbose, args.json)?
    {
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    // Check and fix DDD artifacts
    fix_ddd::check_and_fix_ddd(args.apply, args.json)?;

    // Check and fix archived phase metrics (dedup + prune empty phases)
    fix_phases::check_and_fix_phases(storage, args.apply, args.json)?;

    // Trim synthetic cowboy rides to their last captured activity
    fix_cowboy_durations::check_and_fix_cowboy_durations(storage, args.apply, args.json)?;

    Ok(())
}
