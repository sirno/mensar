use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Opts {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Name of mensa
    #[clap()]
    pub mensa: Option<String>,

    /// Language (de / en)
    #[clap(long)]
    pub lang: Option<String>,

    /// Show prices
    #[clap(short, long)]
    pub prices: bool,

    /// List available mensas
    #[clap(short, long)]
    pub list: bool,
}

    
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Set your default mensa
    SetMensa { name: String },

    /// Set your default language
    SetLanguage { name: String },
}
