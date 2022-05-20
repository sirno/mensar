use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Opts {
    /// Name of mensa
    #[clap(default_value = "polyterrasse")]
    pub mensa: String,

    /// Language (de / en)
    #[clap(long, default_value = "de")]
    pub lang: String,

    /// Show prices
    #[clap(short, long)]
    pub prices: bool,

    /// List available mensas
    #[clap(short, long)]
    pub list: bool,
}
