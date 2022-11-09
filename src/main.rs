use clap;
use structopt::StructOpt;

use notflix_backend::collections;
use notflix_backend::config;
use notflix_backend::db;
use notflix_backend::models;
use notflix_backend::kodifs;
use notflix_backend::server;

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
    /// Serve http(s)
    Serve(ServeOpts),

    #[structopt(display_order = 2)]
    /// Scan directory.
    ScanDir(ScanDirOpts),

    #[structopt(display_order = 2)]
    /// Update movie or tvshow in database.
    Update(UpdateOpts),

    #[structopt(display_order = 3)]
    /// Dump database
    DumpDb(DumpDbOpts),

    #[structopt(display_order = 4)]
    /// Read NFO
    ReadNfo(ReadNfoOpts),
}

#[derive(StructOpt, Debug)]
pub struct ServeOpts {
    #[structopt(short, long)]
    /// Configuration file.
    pub config: String,
}

#[derive(StructOpt, Debug)]
pub struct ScanDirOpts {
    #[structopt(long)]
    /// Scan movie directory.
    pub movie: bool,

    #[structopt(long)]
    /// Scan movie directories.
    pub movies: bool,

    #[structopt(long)]
    /// Scan tvshow directory.
    pub tvshow: bool,

    #[structopt(long)]
    /// Scan tvshow directories.
    pub tvshows: bool,

    /// Directory name.
    pub directory:  String,
}

#[derive(StructOpt, Debug)]
pub struct UpdateOpts {
    #[structopt(long)]
    /// Update single movie.
    pub movie: bool,

    #[structopt(long)]
    /// Update movie collection
    pub movies: bool,

    #[structopt(long)]
    /// Update single tvshow
    pub tvshow: bool,

    #[structopt(long)]
    /// Update tvshow directory
    pub tvshows: bool,

    #[structopt(long)]
    /// Update tvshow directory
    pub database: String,

    /// Directory name.
    pub directory:  String,
}

#[derive(StructOpt, Debug)]
pub struct DumpDbOpts {
    /// Database name.
    pub database:  String,
}

#[derive(StructOpt, Debug)]
pub struct ReadNfoOpts {
    /// NFO name.
    pub filename:  String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = MainOpts::from_args();
    match opts.cmd {
        Command::Serve(opts) => return serve(opts).await,
        Command::ScanDir(opts) => return scandir(opts).await,
        Command::Update(opts) => return update(opts).await,
        Command::DumpDb(opts) => return dumpdb(opts).await,
        Command::ReadNfo(opts) => return readnfo(opts).await,
    }
}

async fn serve(opts: ServeOpts) -> anyhow::Result<()> {
    let cfg = config::from_file(&opts.config)?;

    // FIXME: move somewhere else.
    for coll in &cfg.collections {
        coll.scan().await;
    }

    let handle = db::connect_db(&cfg.server.database).await?;
    server::serve(cfg, handle).await
}

async fn dumpdb(_opts: DumpDbOpts) -> anyhow::Result<()> {
    /*
    let handle = db::connect_db(&opts.database).await?;
    let items = db::get_items(&handle).await?;
    println!("{}", serde_json::to_string_pretty(&items)?);
    */
    eprintln!("not implemented");
    Ok(())
}

async fn scandir(opts: ScanDirOpts) -> anyhow::Result<()> {
    if opts.movie || opts.movies {
        let mut coll = collections::Collection {
            name: "Movies".to_string(),
            type_: "movies".to_string(),
            directory: opts.directory.clone(),
            baseurl: "/".to_string(),
            ..collections::Collection::default()
        };
        if opts.movie {
            let mut m = opts.directory.rsplitn(2, '/');
            let file_name = m.next().unwrap();
            coll.directory = m.next().unwrap_or(".").to_string();

            let mv = models::Movie::default();
            match kodifs::update_movie(&coll, file_name, &mv).await {
                Some(item) => println!("{}", serde_json::to_string_pretty(&item)?),
                None => println!("no movie found"),
            }
        }
        if opts.movies {
            eprintln!("not implemented");
        }
    }
    if opts.tvshow || opts.tvshows {
        let mut coll = collections::Collection {
            name: "TV Shows".to_string(),
            type_: "shows".to_string(),
            directory: opts.directory.clone(),
            baseurl: "/".to_string(),
            ..collections::Collection::default()
        };
        if opts.tvshow {
            let mut m = opts.directory.rsplitn(2, '/');
            let file_name = m.next().unwrap();
            coll.directory = m.next().unwrap_or(".").to_string();

            match kodifs::build_show(&mut coll, file_name).await {
                Some(item) => println!("{}", serde_json::to_string_pretty(&item)?),
                None => println!("no show found"),
            }
        }
        if opts.tvshows {
            kodifs::build_shows(&coll, 0).await;
            let items = coll.get_items().await;
            match items.len() {
                0 => println!("no shows found"),
                _ => println!("{}", serde_json::to_string_pretty(&items)?),
            }
        }
    }
    Ok(())
}

async fn update(opts: UpdateOpts) -> anyhow::Result<()> {
    let db = db::Db::connect(&opts.database).await?;

    if opts.movie || opts.movies {
        let mut coll = collections::Collection {
            name: "Movies".to_string(),
            type_: "movies".to_string(),
            directory: opts.directory.clone(),
            baseurl: "/".to_string(),
            ..collections::Collection::default()
        };
        if opts.movie {
            let mut m = opts.directory.rsplitn(2, '/');
            let file_name = m.next().unwrap();
            coll.directory = m.next().unwrap_or(".").to_string();
            coll.collection_id = 1;

            db.update_movie(&coll, file_name).await?;
            println!("movie updated!");
        }
        if opts.movies {
            eprintln!("not implemented");
        }
    }
    if opts.tvshow || opts.tvshows {
            eprintln!("not implemented");
    }
    Ok(())
}

async fn readnfo(opts: ReadNfoOpts) -> anyhow::Result<()> {
    let mut file = tokio::fs::File::open(&opts.filename).await?;
    let items = kodifs::Nfo::read(&mut file).await?;
    println!("{}", serde_json::to_string_pretty(&items)?);
    Ok(())
}
