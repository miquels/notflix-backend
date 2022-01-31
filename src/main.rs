use clap;
use structopt::StructOpt;

use notflix_backend::collections;
use notflix_backend::db;
use notflix_backend::kodifs;

#[derive(StructOpt, Debug)]
#[structopt(setting = clap::AppSettings::VersionlessSubcommands)]
pub struct MainOpts {
    #[structopt(long)]
    /// Log options (like RUSTLOG; trace, debug, info etc)
    pub log: Option<String>,
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub enum Command {
    #[structopt(display_order = 1)]
    /// Scan directory.
    ScanDir(ScanDirOpts),

    #[structopt(display_order = 2)]
    /// Dump database
    DumpDb(DumpDbOpts),
}

#[derive(StructOpt, Debug)]
pub struct ScanDirOpts {
    #[structopt(short, long)]
    /// Scan movie directory.
    pub movie: bool,

    #[structopt(short, long)]
    /// Scan movie directories.
    pub movies: bool,

    /// Directory name.
    pub directory:  String,
}

#[derive(StructOpt, Debug)]
pub struct DumpDbOpts {
    /// Database name.
    pub database:  String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = MainOpts::from_args();
    match opts.cmd {
        Command::ScanDir(opts) => return scandir(opts).await,
        Command::DumpDb(opts) => return dumpdb(opts).await,
    }
}

async fn dumpdb(opts: DumpDbOpts) -> anyhow::Result<()> {
    let handle = db::connect_db(&opts.database).await?;
    let items = db::get_items(&handle).await;
    println!("{}", serde_json::to_string_pretty(&items)?);
    Ok(())
}

async fn scandir(opts: ScanDirOpts) -> anyhow::Result<()> {
    let mut coll = collections::Collection {
        name: "Movies".to_string(),
        type_: "movies",
        directory: opts.directory.clone(),
        baseurl: "/".to_string(),
        ..collections::Collection::default()
    };
    if opts.movie {
        let mut m = opts.directory.rsplitn(2, '/');
        let file_name = m.next().unwrap();
        coll.directory = m.next().unwrap_or(".").to_string();

        match kodifs::build_movie(&mut coll, file_name).await {
            Some(item) => println!("{:#?}", item),
            None => println!("no movie found"),
        }
    }
    if opts.movies {
        kodifs::build_movies(&mut coll, 0).await;
        match coll.items.len() {
            0 => println!("no movie found"),
            _ => println!("{:#?}", &coll.items),
        }
    }
    Ok(())
}



