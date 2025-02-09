// SPDX-License-Identifier: Apache-2.0

use clap::{Parser, Subcommand};
use std::process;

use git_cmp::*;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Commit {
        /// <other commit> [<our commit>...]
        /// Default: HEAD.
        /// If more than one <our commit>s provided,
        /// they all squashed before comparison with the <other commit>
        #[arg(required = true, num_args(1..))]
        commits: Vec<String>,

        /// autofetch commit IDs from the commit message
        #[arg(long)]
        autofetch: bool,
    },
    Branch {
        /// <old branch> [<common upstream> [<current branch>]]
        /// Default: main, HEAD
        #[arg(required = true, num_args(1..=3))]
        commits: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let repo = repo_open();

    let cmp = match &cli.command {
        Command::Commit { commits, autofetch } => cmp_commits(&repo, commits, *autofetch),
        Command::Branch { commits } => cmp_branches(&repo, commits),
    };

    match cmp {
        Ok((merge, our)) => {
            process::Command::new("git")
                .arg("diff")
                .arg(merge.to_string())
                .arg(our.to_string())
                .status()
                .expect("Failed to execute git diff");
        }
        Err(giterr) => {
            eprintln!("Error: {}", giterr.message());
            std::process::exit(1);
        }
    }
}
