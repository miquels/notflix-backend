use clap;
use structopt::StructOpt;

use notflix_backend::collections;
use notflix_backend::config;
use notflix_backend::db;
use notflix_backend::kodifs;
use notflix_backend::server;
use notflix_backend::models::{Movie, TVShow};

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
    env_logger::init();

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

            match kodifs::scan_movie_dir(&coll, file_name, None, false).await {
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
            collection_id: 2,
            type_: "shows".to_string(),
            directory: opts.directory.clone(),
            baseurl: "/".to_string(),
            ..collections::Collection::default()
        };
        if opts.tvshow {
            let mut m = opts.directory.rsplitn(2, '/');
            let file_name = m.next().unwrap();
            coll.directory = m.next().unwrap_or(".").to_string();

            match kodifs::scan_tvshow_dir(&mut coll, file_name, None, false).await {
                Some(item) => println!("{}", serde_json::to_string_pretty(&item)?),
                None => println!("no show found"),
            }
        }
        if opts.tvshows {
            eprintln!("not implemented");
        }
    }
    Ok(())
}

async fn update(opts: UpdateOpts) -> anyhow::Result<()> {
    let db = db::Db::connect(&opts.database).await?;

    let mut coll = collections::Collection {
        name: "Movies".to_string(),
        type_: "movies".to_string(),
        directory: opts.directory.clone(),
        collection_id: 1,
        baseurl: "/".to_string(),
        ..collections::Collection::default()
    };

    if opts.movie || opts.movies {
        if opts.movie {
            let mut m = opts.directory.rsplitn(2, '/');
            let file_name = m.next().unwrap();
            coll.directory = m.next().unwrap_or(".").to_string();

            let mut txn = db.handle.begin().await?;
            db.update_movie::<Movie>(&coll, file_name, &mut txn).await?;
            txn.commit().await?;
            println!("movie {} updated!", file_name);
        }
        if opts.movies {
            db.update_collection(&coll).await?;
            println!("collection {} updated!", opts.directory);
        }
    }
    if opts.tvshow || opts.tvshows {
        coll.name = "TVShows".to_string();
        coll.type_ = "tvshows".to_string();
        coll.collection_id = 2;

        if opts.tvshow {
            let mut m = opts.directory.rsplitn(2, '/');
            let file_name = m.next().unwrap();
            coll.directory = m.next().unwrap_or(".").to_string();

            let mut txn = db.handle.begin().await?;
            db.update_movie::<TVShow>(&coll, file_name, &mut txn).await?;
            txn.commit().await?;
            println!("movie {} updated!", file_name);
        }
        if opts.movies {
            db.update_collection(&coll).await?;
            println!("collection {} updated!", opts.directory);
        }
    }
    Ok(())
}

async fn readnfo(opts: ReadNfoOpts) -> anyhow::Result<()> {
    let mut file = tokio::fs::File::open(&opts.filename).await?;
    let items = kodifs::Nfo::read(&mut file).await?;
    println!("{}", serde_json::to_string_pretty(&items)?);
    Ok(())
}
