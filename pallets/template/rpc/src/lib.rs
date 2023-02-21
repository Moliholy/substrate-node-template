use std::sync::Arc;

use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as BlockT;

pub use pallet_template_runtime_api::TemplateApi as TemplateRuntimeApi;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct HashItem {
    hash: String,
    pieces: u32,
}

#[rpc(client, server)]
pub trait TemplateApi<BlockHash> {
    #[method(name = "template_getFiles")]
    fn get_files(&self, at: Option<BlockHash>) -> RpcResult<Vec<HashItem>>;
}

/// A struct that implements the `TemplateApi`.
pub struct TemplatePallet<C, Block> {
    // If you have more generics, no need to TemplatePallet<C, M, N, P, ...>
    // just use a tuple like TemplatePallet<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> TemplatePallet<C, Block> {
    /// Create new `TemplatePallet` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block> TemplateApiServer<<Block as BlockT>::Hash> for TemplatePallet<C, Block>
    where
        Block: BlockT,
        C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
        C::Api: TemplateRuntimeApi<Block>,
{
    fn get_files(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<HashItem>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let result = api.get_files(&at).map_err(runtime_error_into_rpc_err)?;
        let hashes = result.into_iter().map(|item| HashItem {
            pieces: item.1,
            hash: item.0.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join(""),
        }).collect();
        Ok(hashes)
    }
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        RUNTIME_ERROR,
        "Runtime error",
        Some(format!("{:?}", err)),
    ))
        .into()
}