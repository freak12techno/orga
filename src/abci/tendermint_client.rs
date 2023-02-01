use std::cell::Cell;
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use tendermint_rpc as tm;
use tm::endpoint::abci_query::AbciQuery;
use tm::Client as _;

use crate::call::Call;
use crate::client::{AsyncCall, AsyncQuery, Client};
use crate::encoding::Encode;
use crate::merk::ABCIPrefixedProofStore;
use crate::query::Query;
use crate::state::State;
use crate::store::{Read, Shared, Store};
use crate::{Error, Result};

pub use tm::endpoint::broadcast::tx_commit::Response as TxResponse;

pub struct TendermintClient<T: Client<TendermintAdapter<T>>> {
    state_client: T::Client,
    tm_client: tm::HttpClient,
}

impl<T> Clone for TendermintClient<T>
where
    T: Client<TendermintAdapter<T>>,
{
    fn clone(&self) -> Self {
        let state_client = T::create_client(TendermintAdapter {
            marker: std::marker::PhantomData,
            client: self.tm_client.clone(),
            res_store: None,
        });

        Self {
            state_client,
            tm_client: self.tm_client.clone(),
        }
    }
}

impl<T: Client<TendermintAdapter<T>>> TendermintClient<T> {
    pub fn new(addr: &str) -> Result<Self> {
        let tm_client = tm::HttpClient::new(addr)?;
        let state_client = T::create_client(TendermintAdapter {
            marker: std::marker::PhantomData,
            client: tm_client.clone(),
            res_store: None,
        });
        Ok(TendermintClient {
            state_client,
            tm_client,
        })
    }

    //this should await something
    pub async fn with_response<F, R, X: std::future::Future<Output = Result<R>>>(
        &self,
        f: F,
    ) -> Result<(R, AbciQuery)>
    where
        F: FnOnce(T::Client) -> X,
    {
        let res_store = Arc::new(Mutex::new(Cell::new(None)));
        let state_client = T::create_client(TendermintAdapter {
            marker: std::marker::PhantomData,
            client: self.tm_client.clone(),
            res_store: Some(res_store.clone()),
        });

        let query_res = f(state_client).await?;

        let response = res_store
            .lock()
            .map_err(|e| Error::Poison(e.to_string()))?
            .take()
            .ok_or_else(|| Error::Query("No query preformed in closure".to_string()))?;
        Ok((query_res, response))
    }
}

impl<T: Client<TendermintAdapter<T>>> Deref for TendermintClient<T> {
    type Target = T::Client;

    fn deref(&self) -> &Self::Target {
        &self.state_client
    }
}

impl<T: Client<TendermintAdapter<T>>> DerefMut for TendermintClient<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_client
    }
}

pub struct TendermintAdapter<T> {
    marker: std::marker::PhantomData<fn() -> T>,
    client: tm::HttpClient,
    res_store: Option<Arc<Mutex<Cell<Option<AbciQuery>>>>>,
}

impl<T> Clone for TendermintAdapter<T> {
    fn clone(&self) -> TendermintAdapter<T> {
        TendermintAdapter {
            marker: self.marker,
            client: self.client.clone(),
            res_store: self.res_store.clone(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<T: Call> AsyncCall for TendermintAdapter<T>
where
    T::Call: Send,
{
    type Call = T::Call;

    async fn call(&self, call: Self::Call) -> Result<()> {
        let tx = call.encode()?.into();
        let tx_res = self.client.broadcast_tx_commit(tx).await?;

        if tx_res.check_tx.code.is_err() {
            Err(Error::ABCI(format!(
                "CheckTx failed: {}",
                tx_res.check_tx.log
            )))
        } else if tx_res.deliver_tx.code.is_err() {
            Err(Error::ABCI(format!(
                "DeliverTx failed: {}",
                tx_res.deliver_tx.log
            )))
        } else {
            Ok(())
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<T: Query + State> AsyncQuery for TendermintAdapter<T> {
    type Query = T::Query;
    type Response<'a> = std::rc::Rc<T>;

    async fn query<F, R>(&self, query: Self::Query, mut check: F) -> Result<R>
    where
        F: FnMut(Self::Response<'_>) -> Result<R>,
    {
        // TODO: attempt query against locally persisted store data for this
        // height, only issue query if we are missing data (belongs in a
        // different type)

        let query_bytes = query.encode()?;
        let res = self
            .client
            .abci_query(None, query_bytes, None, true)
            .await?;

        if let Some(res_store) = &self.res_store {
            res_store
                .lock()
                .map_err(|e| Error::Poison(e.to_string()))?
                .replace(Some(res.clone()));
        }

        if let tendermint::abci::Code::Err(code) = res.code {
            let msg = format!("code {}: {}", code, res.log);
            return Err(Error::Query(msg));
        }

        // TODO: we shouldn't need to include the root hash in the result, it
        // should come from a trusted source
        let root_hash = match res.value[0..32].try_into() {
            Ok(inner) => inner,
            _ => {
                return Err(Error::Tendermint(
                    "Cannot convert result to fixed size array".into(),
                ));
            }
        };
        let proof_bytes = &res.value[32..];

        let map = merk::proofs::query::verify(proof_bytes, root_hash)?;
        // TODO: merge data into locally persisted store data for given height

        let store: Shared<ABCIPrefixedProofStore> = Shared::new(ABCIPrefixedProofStore::new(map));
        let root_value = {
            match store.get(&[])? {
                Some(root_value) => root_value,
                None => return Err(Error::ABCI("Missing root value".into())),
            }
        };
        let mut state = T::load(Store::new(store.clone().into()), &mut root_value.as_slice())?;

        // TODO: remove need for ABCI prefix layer since that should come from
        // ABCIPlugin Client impl and should be part of app type
        state.attach(Store::new(store.into()))?;

        // TODO: retry logic
        check(std::rc::Rc::new(state))
    }
}
