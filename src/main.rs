use std::io;
use std::process;
use std::io::Write;

use actix_cors::Cors;
use actix_web::{
    HttpServer, 
    HttpResponse,
    get,
    App,  
    Responder,
};

use ethers::providers::{Middleware};

use std::{
    error::Error,
    sync::Arc,
};
use db::Db;

mod db;
mod error;
mod eth_explore;

type Result<T> = std::result::Result<T, error::Error>;

const FROM_ETH_BLOCK: u64 = 16_976_395; // Let's take a ethereum block

// Actix server side (GET latest Transactions)
#[get("/transactions")]
async fn get_latest_transactions() -> impl Responder {
    // Get Eth Provider
    let eth_provider = Arc::new(eth_explore::get_eth_provider().await);

    // Get transaction from ethers-rs
    let transactions = eth_explore::get_last_10_eth_transactions(eth_provider)
        .await
        .expect("error get last 10 transactions");
    println!("GET latest transactions : {:?}", transactions);

    // response with transactions in JSON
    HttpResponse::Ok().json(transactions)
}

// Actix server side (GET latest blocks)
#[get("/blocks")]
async fn get_latest_blocks() -> impl Responder {
    // Get Eth Provider
    let eth_provider = Arc::new(eth_explore::get_eth_provider().await);
    
    // Get blocks from ethers-rs
    let blocks = eth_explore::get_last_10_eth_blocks(eth_provider)
        .await
        .expect("error get last 10 blocks");
    println!("GET latest blocks : {:?}", blocks);
    
    // Response with blocks in JSON
    HttpResponse::Ok().json(blocks)
}

// Actix server side (GET historic data)
#[get("/historic-data")]
async fn get_chart_info() -> impl Responder {
    // Get Eth Provider
    let eth_provider = Arc::new(eth_explore::get_eth_provider().await);
    
    // Connect to db 
    let db= Db::init().await.expect("db init error");

    // Get blocks from ethers-rs
    let mut blocks_vec = db.fetch_all_ethblocks().await.expect("fetch all eth blocks error");

    if blocks_vec.is_empty() {

        println!("No data in database — Downloading...");

        eth_explore::get_transactions_history(eth_provider, db.clone(), FROM_ETH_BLOCK)
            .await
            .expect("get transaction history err");

        blocks_vec = db.fetch_all_ethblocks()
            .await
            .unwrap();

    }
    
    // Response with blocks in JSON
    HttpResponse::Ok().json(blocks_vec)
}

// Menu
fn menu(choice : &mut String) {
    println!("");
    println!("Menu");
    println!("1) Gas Price");
    println!("2) Get the latest blocks");
    println!("3) Get the latest transactions");
    println!("4) Download history data to MongoDB (takes some times)");
    println!("5) Fetch history from MongoDB");
    println!("6) Clear data from MongoDB");
    println!("7) Run the webserver with Actix");
    println!("0) Quit");
    println!("Please enter your choice");
    io_stdout_flush_e();
    choice.clear();
    io_stdin_read_line_e(choice);
    println!("");//space
}

#[tokio::main]
async fn main() -> eyre::Result<(), Box<dyn Error>> {

    // Menu variable
    let mut choice = String::new();

    // Get Eth Provider
    let eth_provider = Arc::new(eth_explore::get_eth_provider().await);
    println!("{:?}", eth_provider);

    // Connect to db 
    let db= Db::init().await.expect("db init should work");

    // Get the last 500 blocks number
    let to_block_number = eth_provider.get_block_number().await?.as_u64();
    let from_block_number = to_block_number - 500;

    loop {
        
        // Display Menu
        menu(&mut choice);

        match choice.trim().parse().unwrap() {
            0 => {
                println!("exiting");
                process::exit(0);
            }, 
            1 => {
                // Gas price (w/ ETH price from Chainlink oracle)
                let (gwei,usd_per_gas,gas_value) = eth_explore::get_gas_price(eth_provider.clone()).await?;
                eth_explore::print_gas_value(gwei,usd_per_gas,gas_value);
            }, 
            2 => {
                // Get the 10 latest eth blocks
                let blocks_vec = eth_explore::get_last_10_eth_blocks(eth_provider.clone()).await?;
                println!("{:#?}", blocks_vec);

            },
            3 => {
                // Get the 10 latest eth transactions
                let transactions_vec = eth_explore::get_last_10_eth_transactions(eth_provider.clone()).await?;
                println!("{:#?}", transactions_vec);
            }, 
            4 => {
                // Downloading eth blockchain data
                eth_explore::get_transactions_history(eth_provider.clone(), db.clone(), from_block_number).await?;
            }, 
            5 => {
                // Fetch data from database
                let blocks_vec = db.fetch_all_ethblocks().await?;
                println!("{:#?}", blocks_vec);

            }
            6 => {
                let result = db.delete_collection().await?;
                println!("{} Delected", result.deleted_count);
            }
            7 => {
                println!("Waiting for JS Client... Please open frontend/ethscan.html");
                
                // Actix server
                HttpServer::new(|| {
                    let cors = Cors::permissive();
                    App::new()
                    .wrap(cors)
                    .service(get_latest_transactions)
                    .service(get_latest_blocks)
                    .service(get_chart_info)
                })
                .bind(("127.0.0.1", 8080))?
                .run()
                .await?;

            }
            _ => {
                println!("invalid choice, please try again");
            }
        }

    }
}


fn io_stdout_flush_e() {
    match io::stdout().flush() { 
        Ok(_) => {}, 
        Err(error) =>  {
            eprintln!("Error flush io std : {}", error);
            std::process::exit(1);
        }
    }
}

fn io_stdin_read_line_e(buf: &mut String){
    match io::stdin().read_line(buf) {
        Ok(_) => {}
        Err(error) => {
            eprintln!("Error when reading standard entry: {}", error);
            std::process::exit(1);
        }
    }
}

