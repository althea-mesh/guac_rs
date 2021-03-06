use crate::types::Counterparty;
use clarity::Address;
use failure::Error;

use futures;

use futures::Future;

use qutex::{Guard, QrwLock, Qutex};
use std::collections::HashMap;

/// Storage contains a futures aware RwLock (QrwLock) which controls access to the inner data
/// This outer Rwlock should only be mutated very rarely, only to insert and remove counterparties
pub struct Storage {
    inner: QrwLock<HashMap<Address, Qutex<Counterparty>>>,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            inner: QrwLock::new(HashMap::new()),
        }
    }

    pub fn get_counterparty(
        &self,
        k: Address,
    ) -> impl Future<Item = Option<Guard<Counterparty>>, Error = Error> {
        self.inner
            .clone()
            .read()
            .from_err()
            .and_then(move |data| match data.get(&k) {
                Some(v) => Box::new(v.clone().lock().from_err().and_then(|v| Ok(Some(v)))),
                None => Box::new(futures::future::ok(None))
                    as Box<Future<Item = Option<Guard<Counterparty>>, Error = Error>>,
            })
    }

    pub fn new_counterparty(
        &self,
        k: Address,
        v: Counterparty,
    ) -> impl Future<Item = (), Error = Error> {
        self.inner
            .clone()
            .write()
            .from_err()
            .and_then(move |mut data| {
                if !data.contains_key(&k) {
                    data.insert(k.clone(), Qutex::new(v.clone()));
                } else {
                    bail!("Counterparty already exists");
                }
                Ok(())
            })
    }
}
