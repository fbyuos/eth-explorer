use crate::db::bson::to_bson;
use crate::eth_explore::{EthBlocks};
use crate::{error::Error::*, Result};
use mongodb::results::DeleteResult;
use mongodb::{options::ClientOptions, Client, Database};
use mongodb::bson::{self, document::Document};
use ethers::types::{U64};
use futures::stream::StreamExt;


const DB_NAME: &str = "ethereum-blockchain";
const COLLECTION: &str = "eth_blocks";
const DB_URL: &str = "mongodb://localhost:27017";

#[derive(Clone, Debug)]
pub struct Db {
    db: Database,
}

impl Db {

    pub async fn init() -> Result<Self> {
        let client_options = ClientOptions::parse(DB_URL).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(DB_NAME);
        Ok(Self { db })
    }

    fn get_collection(&self) -> mongodb::Collection<bson::Document> {
        self.db.collection::<bson::Document>(COLLECTION)
    }

    pub async fn delete_collection(&self) -> Result<DeleteResult> {
        let filter = bson::doc! {};
        let result = self.get_collection()
            .delete_many(filter, None)
            .await
            .map_err(MongoQueryError)?;
        Ok(result)
    }

    async fn doc_to_ethblocks(&self, doc: Document) -> Result<EthBlocks> {
        bson::from_document(doc).map_err(MongoBsonError)
    }

    pub async fn fetch_ethblocks(&self, block_number: U64) -> Result<Option<EthBlocks>> {
        let block_number_hex = format!("0x{:x}", block_number);
        let filter = bson::doc!{ "number": block_number_hex };
        let result = self.get_collection()
            .find_one(filter, None)
            .await?;

        match result {
            Some(doc) => Ok(Some(self.doc_to_ethblocks(doc).await?)),
            None => Ok(None),
        }
    }

    pub async fn create_ethblocks(&self, eth_blocks: &EthBlocks) -> Result<()> {
        let doc = bson::to_document(&eth_blocks).unwrap();
        self.get_collection()
            .insert_one(doc, None)
            .await
            .map_err(MongoQueryError)?;
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn edit_ethblocks(&self, eth_blocks: &EthBlocks) -> Result<()> {
        let block_number_hex = format!("0x{:x}", eth_blocks.number.unwrap());
        let filter = bson::doc! { "number": block_number_hex };
        let doc = to_bson(&eth_blocks).unwrap();
        let doc = doc.as_document().ok_or("erreur eth block -> doc").unwrap().clone();
        self.get_collection()
            .find_one_and_replace(filter, doc, None)
            .await
            .map_err(MongoQueryError)?;
        Ok(())
    }

    pub async fn found_one_ethblocks(&self, block_number: U64) -> bool {
        let block_number_hex = format!("0x{:x}", block_number);
        let filter = bson::doc! { 
            "number": block_number_hex 
        };
        let result = self.get_collection().find_one(filter, None).await;
        match result {
            Ok(document) => document.is_some(),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                false
            }
        }
        
    }

    #[allow(dead_code)]
    pub async fn delete_ethblocks(&self, block_number: U64) -> Result<()> {
        let block_number_hex = format!("0x{:x}", block_number);
        let filter = bson::doc! { 
            "number": block_number_hex 
        };
        self.get_collection()
            .delete_one(filter, None)
            .await
            .map_err(MongoQueryError)?;
        Ok(())
    }

    pub async fn fetch_all_ethblocks(&self) -> Result<Vec<EthBlocks>> {
        let mut cursor = self.get_collection().find(None, None).await?;
        let mut eth_blocks: Vec<EthBlocks> = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => {
                    let block: EthBlocks = bson::from_bson(bson::Bson::Document(document))
                        .unwrap();
                    eth_blocks.push(block);
                }
                Err(e) => return Err(MongoQueryError(e)),
            }
        }
    
        Ok(eth_blocks)
    }
    
}














