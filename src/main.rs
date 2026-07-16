//use base64::{Engine as _, engine::general_purpose};
use clap::{Parser, Subcommand, ValueEnum};
//use std::fs;
//use std::io::{Write};
//use std::os::unix::fs::PermissionsExt;
//use std::process;
//use crossterm::event::{read, Event};

use std::path::Path;

mod func;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// make secret key and public key
    Key {
        /// keep processing as much as possible
        #[arg(short, long)]
        force: bool,

        /// run without outputting logs
        #[arg(short, long)]
        quiet: bool,
    },

    /// register the other person's public key
    Register {
        /// keep processing as much as possible
        #[arg(short, long)]
        force: bool,

        /// run without outputting logs
        #[arg(short, long)]
        quiet: bool,
    },

    /// encrypt data
    Encrypt {
        #[arg(short, long, value_enum)]
        output: Output,

        #[arg(long = "output-path", alias = "op", requires = "output")]
        output_path: Option<std::path::PathBuf>,

        #[arg(short = 'E', long)]
        encryptors: std::path::PathBuf,

        #[arg(long, alias = "tp", conflicts_with = "target")]
        target_path: Option<std::path::PathBuf>,

        #[arg(short, long, conflicts_with = "target_path")]
        target: Option<String>,

        #[arg(short, long)]
        force: bool,

        #[arg(short, long)]
        quiet: bool,
    },

    /// decode data
    Decode {
        #[arg(short, long, value_enum)]
        output: Output,

        #[arg(long = "output-path", alias = "op", requires = "output")]
        output_path: Option<std::path::PathBuf>,

        #[arg(short = 'E', long)]
        encryptors: std::path::PathBuf,

        #[arg(long, alias = "tp", conflicts_with = "target")]
        target_path: Option<std::path::PathBuf>,

        #[arg(short, long, conflicts_with = "target_path")]
        target: Option<String>,

        #[arg(short, long)]
        force: bool,

        #[arg(short, long)]
        quiet: bool,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
pub enum Output {
    #[value(
        name = "std",
        alias = "stdout",
        alias = "0",
        help = "output to stdout(the --path option is not required)"
    )]
    Std,

    #[value(
        name = "file",
        alias = "1",
        help = "output to a .txt file(requires the --path option)"
    )]
    File,
}

fn main() {
    let args = Args::parse();
    let my_secret_path = Path::new("keys/sec.key");
    let my_public_path = Path::new("keys/pub.key");
    let their_public_path = Path::new("keys/their_pub.key");

    let _ = their_public_path;

    match args.command {
        Command::Key { force, quiet } => {
            func::create_key(force, quiet, &my_secret_path, &my_public_path);
            println!("Key generation completed.");
        }

        Command::Register { force, quiet } => {
            let _ = force;

            // 対話型で公開鍵を入力
            let their_pub = func::read_user_input("please enter public key(Base64): ");

            if their_pub.is_empty() {
                println!("public key is empty.please try again.");
                return;
            }

            func::register_their_public(&their_pub, &their_public_path, force, quiet);
        }

        Command::Encrypt {
            output,
            output_path,
            encryptors,
            target_path,
            target,
            force,
            quiet,
        } => {
            let target = func::get_target(&target_path, &target.unwrap_or_default(), quiet);

            let result =
                func::cryptography(encryptors.as_path(), force, quiet, target, true, false);

            func::handle_output(output, output_path, result, quiet, force);
        }

        Command::Decode {
            output,
            output_path,
            encryptors,
            target_path,
            target,
            force,
            quiet,
        } => {
            let target = func::get_target(&target_path, &target.unwrap_or_default(), quiet);

            let result =
                func::cryptography(encryptors.as_path(), force, quiet, target, false, true);

            func::handle_output(output, output_path, result, quiet, force);
        }
    }
}
