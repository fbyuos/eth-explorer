use std::num::ParseFloatError;

use thiserror::Error;
use mongodb::bson; 
use ethers::core::utils::*;
use ethers::prelude::Http;


#[derive(Error, Debug)]
pub enum Error {
    #[error("error mongodb query: {0}")]
    MongoQueryError(mongodb::error::Error),
    #[error("mongodb error:{0}")]
    MongoError(#[from] mongodb::error::Error), 
    #[error("could not access file in document: {0}")]
    MongoBsonError(#[from] bson::de::Error),
    #[error("cannot convert: {0}")]
    EthConvErr(ConversionError),
    #[error("cannot convert string: {0}")]
    EthConvStrErr(ParseFloatError),
    #[error("get blocks error: {0}")]
    EthProviderErr(ethers::providers::ProviderError),
    #[error("get oracle error: {0}")]
    EthOracleErr(ethers::contract::ContractError<ethers::providers::Provider<Http>>),
}