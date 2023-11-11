use clap::Parser;
use instant_models::Schema;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Options::parse();
    let (client, conn) = tokio_postgres::connect(&opts.db, NoTls).await?;
    tokio::spawn(conn);

    let schema = Schema::from_postgres(&client).await?;
    println!("{schema}");

    Ok(())
}

/// Generate Rust code from Postgres database
#[derive(Parser, Debug)]
struct Options {
    /// Database to inspect (as URL)
    db: String,
}
