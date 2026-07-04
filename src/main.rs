//use base64::{Engine as _, engine::general_purpose};
use clap::{Parser, ValueEnum, Subcommand};
use std::fs;
use std::io::{Write};
//use std::os::unix::fs::PermissionsExt;
use std::process;
//use crossterm::event::{read, Event};

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
        /// plugin folder
        #[arg(short='E', long)]
        encryptors: std::path::PathBuf,

        /// keep processing as much as possible
        #[arg(short, long)]
        force: bool,

        /// run without outputting logs
        #[arg(short, long)]
        quiet: bool,
    },

    /// register the other person's public key
    Register {
        /// plugin folder
        #[arg(short='E', long)]
        encryptors: std::path::PathBuf,

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

        #[arg(short='E', long)]
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

        #[arg(short='E', long)]
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

    match args.command {
        Command::Key { encryptors, force, quiet } => {
            let order_files = func::find_order_files(
                encryptors.as_path(),
                "order",
                force,
                quiet,
            ).unwrap_or_else(|e| {
                eprintln!("Error: failed to read order files: {}", e);
                process::exit(1);
            });

            if order_files.is_empty() {
                eprintln!("Error: no order file found in {:?}", encryptors);
                process::exit(1);
            }

            let content = fs::read_to_string(&order_files[0]).unwrap_or_else(|e| {
                eprintln!("failed to read order file: {}", e);
                process::exit(1);
            });

            let first_plugin = content
                .split(";")
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .next()
                .map(|name| {
                    let mut p = encryptors.clone();
                    p.push(name);
                    p
                })
                .unwrap_or_else(|| {
                    eprintln!("Error: order file is empty");
                    process::exit(1);
                });

            func::create_key(force, quiet, true, first_plugin.as_path());
            println!("Key generation completed.");
        }

       Command::Register { encryptors, force, quiet } => {
    // order ファイルを読む
    let order_files = func::find_order_files(
        encryptors.as_path(),
        "order",
        force,
        quiet,
    ).unwrap_or_else(|e| {
        eprintln!("Error: failed to read order files: {}", e);
        process::exit(1);
    });

    if order_files.is_empty() {
        eprintln!("Error: no order file found in {:?}", encryptors);
        process::exit(1);
    }

    let content = fs::read_to_string(&order_files[0]).unwrap_or_else(|e| {
        eprintln!("failed to read order file: {}", e);
        process::exit(1);
    });

    // 最初のプラグイン名を取得
    let first_plugin = content
        .split(";")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .next()
        .map(|name| {
            let mut p = encryptors.clone();
            p.push(name);
            p
        })
        .unwrap_or_else(|| {
            eprintln!("Error: order file is empty");
            process::exit(1);
        });

    // 対話型で公開鍵を入力
    let their_pub = func::read_user_input("please enter public key(Base64): ");

    if their_pub.is_empty() {
        println!("public key is empty.please try again.");
        return;
    }

    // plugin の stdin に公開鍵を渡す
    let mut child = std::process::Command::new(&first_plugin)
        .arg("boo")
        .arg("false")
        .arg(force.to_string())
        .arg(quiet.to_string())
        .arg("foo")
        .arg("true") // register モード
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute plugin");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(their_pub.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
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

            let result = func::cryptography(
                encryptors.as_path(),
                force,
                quiet,
                target,
                true,
                false,
            );

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

            let result = func::cryptography(
                encryptors.as_path(),
                force,
                quiet,
                target,
                false,
                true,
            );

            func::handle_output(output, output_path, result, quiet, force);
        }
    }
}



