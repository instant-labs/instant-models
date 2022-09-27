use clap::Parser;
use instant_models::*;
use postgres::{Config, NoTls}; // Client

/// Generate Rust code from postgres table.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the table to generate
    #[clap(short, long, value_parser)]
    table_name: String,

    /// Postgres username
    #[clap(long, value_parser, default_value = "postgres")]
    pg_username: String,

    /// Postgres password
    #[clap(long, value_parser, default_value = "postgres")]
    pg_password: String,

    /// Postgres host
    #[clap(long, value_parser, default_value = "127.0.0.1")]
    pg_host: String,

    /// Postgres port
    #[clap(long, value_parser, default_value_t = 5432)]
    pg_port: u16,

    /// Postgres db name
    #[clap(long, value_parser, default_value = "postgres")]
    pg_dbname: String,
}

fn main() {
    let args = Args::parse();
    let client = &mut Config::new()
        .user(&args.pg_username)
        .password(&args.pg_password)
        .host(&args.pg_host)
        .port(args.pg_port)
        .dbname(&args.pg_dbname)
        .connect(NoTls)
        .unwrap();
    let struct_bldr = StructBuilder::new_from_conn(client, &args.table_name).unwrap();
    println!("{}", struct_bldr.build_type());
    println!("\n{}", struct_bldr.build_new_type());
    println!("\n{}", struct_bldr.build_type_methods());
    #[cfg(feature = "sql")]
    println!("\n{}", struct_bldr.build_field_identifiers());
}
