#[derive(Subcommand, Debug)]
pub enum Command {
    /// make secret key and public key
    Key {
        /// plugin folder
        #[arg(short = 'E', long)]
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
        #[arg(short = 'E', long)]
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
