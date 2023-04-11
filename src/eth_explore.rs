use crate::db::Db;
use crate::{error::Error::*, Result};

use std::time::Duration;
use std::{
    ops::{Div, Mul},
    sync::Arc,
};
use eyre;
use std::time::Instant;
use serde::{Serialize, Deserialize};

use ethers::providers::{Middleware, Http, Provider};
use ethers::{
    contract::abigen,
    core::{utils::format_units},
    types::{U64, U256, I256, H256, Address, Transaction, Block}
};

//const RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/GkJhEJRYGzTnVM0AmRZZ_TgzIesntlDR";
const RPC_URL: &str = "https://eth.llamarpc.com";
const ETH_DECIMALS: u32 = 18;
const USD_PRICE_DECIMALS: u32 = 8;
const ETH_USD_FEED: &str = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419";
    

abigen!(
    AggregatorInterface,
    r#"[
        latestAnswer() public view virtual override returns (int256 answer)
    ]"#,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthTransaction{
    pub hash: H256,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub gas_price: Option<U256>,
    pub gas: U256,
}


impl EthTransaction{
    fn copy_transaction(transaction: &Transaction) -> EthTransaction {
        let new_transaction: EthTransaction= EthTransaction{
            hash:transaction.hash,
            from:transaction.from,
            to:transaction.to,
            value:transaction.value,
            gas_price:transaction.gas_price,
            gas:transaction.gas
        };
        new_transaction
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthBlocks{
    pub number: Option<U64>,
    pub hash: Option<H256>,
    pub miner_author: Option<Address>,
    pub timestamp: U256,
    pub transaction_number: u64,
    pub transactions: Vec<EthTransaction>
}

impl EthBlocks {
    fn copy_blocks(blocks: &Block<H256>) -> EthBlocks {
        let new_blocks: EthBlocks = EthBlocks { 
            number: blocks.number, 
            hash: blocks.hash, 
            miner_author: blocks.author, 
            timestamp: blocks.timestamp, 
            transaction_number: blocks.transactions.len() as u64,
            transactions: vec![]
        };
        new_blocks
    }
    fn copy_blocks_txs(blocks: &Block<Transaction>) -> EthBlocks {
        let new_blocks: EthBlocks = EthBlocks { 
            number: blocks.number, 
            hash: blocks.hash, 
            miner_author: blocks.author, 
            timestamp: blocks.timestamp, 
            transaction_number: blocks.transactions.len() as u64,
            transactions: vec![]
        };
        new_blocks
    }
}


// Initialize a new Http provider
pub async fn get_eth_provider() -> Provider<Http> {
    // Initialize a new Http provider
    let eth_provider = Provider::try_from(RPC_URL).unwrap();
    println!("Connecting with the provider...");
    eth_provider
}


// Download the lastest month history 
pub async fn get_transactions_history( 
    provider: Arc<Provider<Http>>, 
    db: Db, 
    from: u64 
) -> eyre::Result<()> {
    
    let eth_provider = Arc::new(provider);
    let from_block_number = from;
    let to_block_number = eth_provider.get_block_number().await?;
    println!(
        "Downloading {} blocks...", 
        to_block_number - from_block_number
    );

    // Start time
    let start = Instant::now();

    for block_number in from_block_number..=to_block_number.as_u64() {

        // Http retry
        let mut retries = 3;
        while retries > 0 {

            // Research in DB
            if !db.found_one_ethblocks(U64::from(block_number)).await {
                let block_option: Option<Block<Transaction>> ;

                // Download the block (takes a lot of time)
                // It seems like Ethereum block is growing bigger and bigger, 
                // There are more and more transactions inside each block (MEV bot, L2, etc.)
                match eth_provider.get_block_with_txs(block_number).await {
                    Ok(block) => {
                        block_option = Some(block.unwrap());
                        
                        // Create EthTransactions
                        let mut vec_transactions: Vec<EthTransaction>= vec![];
                        if let Some(block) = block_option.as_ref()  {
                            for i_transaction in block.transactions.iter() {
                                vec_transactions.push(EthTransaction::copy_transaction(i_transaction));        
                            }
                        } else {
                            println!("No Transactions");
                        }
                        // Create EthBlocks
                        let block_option2 = block_option.unwrap().clone();
                        let mut blocks = EthBlocks::copy_blocks_txs(&block_option2);
                        blocks.transactions = vec_transactions;
                        db.create_ethblocks(&blocks).await?;
                        
                        print!("{}", (8u8 as char));
                        print!(
                            "Downloading... Block {} ({:.2} Blocks/s):\r", 
                            block_number, 
                            ((block_number-from_block_number) as f64) / start.elapsed().as_secs_f64()
                        );

                        break;
                    }, 
                    Err(err) => {
                        eprintln!("Error downloading block {}: {}. Retrying...", block_number, err);
                        retries -= 1;
                        tokio::time::sleep(Duration::from_millis(10)).await; // Wait for a second before retrying
                    }
                }
                
            } else { // Found in Database

                print!("{}", (8u8 as char));
                print!(
                    "Found... Block {} ({:.2} Blocks/s):\r", 
                    block_number, 
                    ((block_number-from_block_number) as f64) / start.elapsed().as_secs_f64()
                );
                break;

            }
        }   
    }
    println!("{} Blocks downloaded", to_block_number - from_block_number);
    Ok(())
}

// Connect to oracle, get ETH/USD price
fn get_oracle(client: &Arc<Provider<Http>>) -> AggregatorInterface<Provider<Http>> {
    let address: Address = ETH_USD_FEED.parse().expect("Valid address");
    AggregatorInterface::new(address, Arc::clone(client))
}


/// Retrieves the USD amount per gas unit, using a Chainlink price oracle.
/// Function gets the amount of `wei` to be spent per gas unit then multiplies
/// for the ETH USD value.
pub async fn get_gas_price(provider: Arc<Provider<Http>>) -> Result<(f64,f64,f64)>{

    let client = Arc::new(provider);
    let oracle = get_oracle(&client);

    let usd_per_eth: I256 = oracle.latest_answer().call().await.map_err(EthOracleErr)?;
    let usd_per_eth: U256 = U256::from(usd_per_eth.as_u128());
    let wei_per_gas: U256 = client.get_gas_price().await.map_err(EthProviderErr)?;

    // Gas stations use to report gas price in gwei units (1 gwei = 10^9 wei)
    let gwei: f64 = format_units(wei_per_gas, "gwei")
        .map_err(EthConvErr)?
        .parse::<f64>()
        .map_err(EthConvStrErr)?;

    // Let's convert the gas price to USD
    let usd_per_gas: f64 = usd_value(wei_per_gas, usd_per_eth)?;
    let gas_value: f64 = usd_per_gas*21000.0;
    Ok((gwei,usd_per_gas,gas_value))
}



/// `amount`: Number of wei per gas unit (18 decimals)
/// `price_usd`: USD price per ETH (8 decimals)
fn usd_value(amount: U256, price_usd: U256) -> Result<f64> {
    let base: U256 = U256::from(10).pow(ETH_DECIMALS.into());
    let value: U256 = amount.mul(price_usd).div(base);
    let f: String = format_units(value, USD_PRICE_DECIMALS).map_err(EthConvErr)?;
    Ok(f.parse::<f64>().map_err(EthConvStrErr)?)
}



/// Get the last 10 blocks
/// 
/// 
pub async fn get_last_10_eth_blocks(provider: Arc<Provider<Http>>) -> Result<Vec<EthBlocks>>{

    // provider for interacting with the [Ethereum JSON RPC API]
    let eth_provider = Arc::new(provider);

    // Get the last block number
    let to_block_number = eth_provider.get_block_number().await.map_err(EthProviderErr)?.as_u64();
    println!("Lastest block : {}", to_block_number);

    // Get the last ten blocks number
    let from_block_number = to_block_number - 10;

    // Start time
    let start = Instant::now();

    // Mutable blocks vector
    let mut vec_blocks : Vec<EthBlocks> = vec![];

    // Request the last 10 blocks
    for block_number in from_block_number..=to_block_number {
        match eth_provider.get_block(block_number).await.map_err(EthProviderErr)? {

            Some(block) => {
                //print_block(&block, true);
                vec_blocks.push(EthBlocks::copy_blocks(&block));
            },
            None => {}
        }
    }
    println!("Downloaded the 10 last blocks in {:.3}s", start.elapsed().as_secs_f64());
    Ok(vec_blocks)
}




/// Get the last 10 transactions
/// 
/// 
pub async fn get_last_10_eth_transactions(provider: Arc<Provider<Http>>) -> Result<Vec<EthTransaction>>{

    // provider for interacting with the [Ethereum JSON RPC API]
    let eth_provider = Arc::new(provider);
    
    // Get the last block number
    let block_number = eth_provider.get_block_number()
        .await
        .map_err(EthProviderErr)?
        .as_u64();
    println!("Lastest block : {}", block_number);

    // Build Vec<EthTransaction>
    let mut vec_transactions: Vec<EthTransaction>= vec![];

    // Start time
    let start = Instant::now();
    let mut transaction_number: usize = 0;

    // Get the last block 
    match eth_provider.get_block_with_txs(block_number)
        .await
        .map_err(EthProviderErr)? {

        Some(block) => {
            // Transactions[]
            if !block.transactions.is_empty() {
                transaction_number = block.transactions.len();

                for (i,transaction) in block.transactions.iter().enumerate() {
                    vec_transactions.push(EthTransaction::copy_transaction(transaction));
                    if i == 9 {break;}
                }
            } else {
                println!("No Transactions");
            }
        },
        None => {}
    }
    
    println!(
        "Downloaded the last block ({}) with {} transactions in {:.3}s", 
        block_number, 
        transaction_number, 
        start.elapsed().as_secs_f64()
    );

    Ok(vec_transactions)

}


#[allow(dead_code)]
pub async fn fetch_transaction_history(
    db: Db, 
    from_blocks: u64, 
    number: u64 
) -> eyre::Result<()>{
    for iblock in from_blocks..(from_blocks+number) {
        if let Some(block) = db.fetch_ethblocks(U64::from(iblock)).await.unwrap() {
            println!("{:#?}", block);
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn print_transaction(transaction: &Transaction, istrue: bool) {
    if istrue {
        println!("{}", "=".repeat(50));
        //println!("{:#?}", i_transaction);
        println!("Transaction hash : {:?}", transaction.hash);
        println!("index : {}", transaction.transaction_index.unwrap());
        println!("from : {}", transaction.from);
        println!("to : {}", transaction.to.unwrap());
        println!("value : {} Ξ", format_units(transaction.value, "ether").unwrap());
        println!("gas price : {} Gwei", format_units(
            transaction.gas_price.unwrap(), 
            "gwei"
        ).unwrap()); 
        println!("gas: {} unit", transaction.gas);
        println!("{}", "=".repeat(50));
    }
}


#[allow(dead_code)]
fn print_block(block: &Block<H256>, istrue: bool) {
    if istrue {
        println!("{}", "=".repeat(50));
        //println!("Block {}: {:#?}", block_number, block);
        println!("Block {}", block.number.unwrap());
        println!("Block time: {:?}", block.time().unwrap());
        println!("Block hash: {:?}", block.hash.unwrap());
        println!("Block Autor: {:?}", block.author.unwrap());
        println!("{}", "=".repeat(50));
        println!("");
    }
}

pub fn print_gas_value(gwei: f64,usd_per_gas: f64,gas_value: f64) {
    println!(
        r#"
        Gas price
        ---------------
        {gwei:>10.2} gwei
        {usd_per_gas:>10.8} usd

        Total gas estimated
        ---------------
        {gas_value:>5.2} usd (for 21000 unit)
        "#
    );
}

// Subscribe to a websocket provider, 
// Wait for new block (with transaction) using get_block_with_txs
// Update DB
// pub async fn subscribe_blocks_and_update(){}