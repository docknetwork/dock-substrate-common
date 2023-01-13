use core::fmt::Debug;
use jsonrpsee::{
    core::{async_trait, Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::{error::CallError, ErrorObject},
};
pub use price_feed::runtime_api::PriceFeedApi as PriceFeedRuntimeApi;
use price_feed::{CurrencySymbolPair, PriceRecord};
use sp_api::{NumberFor, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, Get},
};
use std::sync::Arc;

#[rpc(server, client)]
pub trait PriceFeedApi<BlockHash, Number, MaxSymbolBytesLen: Get<u32>> {
    /// Gets the price of currency pair
    #[method(name = "price_feed_price")]
    async fn price(
        &self,
        at: Option<BlockHash>,
        currency_pair: CurrencySymbolPair<String, String>,
    ) -> RpcResult<Option<PriceRecord<Number>>>;
}

#[derive(Debug, Clone)]
struct RuntimeError<T>(T);

impl<T: Debug> From<RuntimeError<T>> for JsonRpseeError {
    fn from(error: RuntimeError<T>) -> Self {
        let data = format!("{:?}", error);

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            1,
            "Runtime error",
            Some(data),
        )))
    }
}

/// A struct that implements the [`PriceFeedApi`].
pub struct PriceFeed<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> PriceFeed<C, P> {
    /// Create new `PriceFeed` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        PriceFeed {
            client,
            _marker: Default::default(),
        }
    }
}

#[async_trait]
impl<C, Block, MaxSymbolBytesLen>
    PriceFeedApiServer<<Block as BlockT>::Hash, NumberFor<Block>, MaxSymbolBytesLen>
    for PriceFeed<C, Block>
where
    Block: BlockT,
    MaxSymbolBytesLen: Get<u32> + Send + Sync + 'static,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: PriceFeedRuntimeApi<Block, NumberFor<Block>, MaxSymbolBytesLen>,
{
    async fn price(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        pair: CurrencySymbolPair<String, String>,
    ) -> RpcResult<Option<PriceRecord<NumberFor<Block>>>> {
        let api = self.client.runtime_api();

        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        api.price(&at, pair)
            .map_err(RuntimeError)
            .map_err(Into::into)
    }
}
