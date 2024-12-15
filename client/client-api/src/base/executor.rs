use std::sync::Arc;

use solana_sdk::transaction::VersionedTransaction;

use crate::client::Client;
use crate::errors::ClientResult;

pub trait ProcessTransaction<T>: Client {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T>;
}

impl<T, C: ?Sized + ProcessTransaction<T>> ProcessTransaction<T> for &C {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).process_transaction(transaction)
    }
}

impl<T, C: ?Sized + ProcessTransaction<T>> ProcessTransaction<T> for &mut C {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).process_transaction(transaction)
    }
}

impl<T, C: ?Sized + ProcessTransaction<T>> ProcessTransaction<T> for Box<C> {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).process_transaction(transaction)
    }
}

impl<T, C: ?Sized + ProcessTransaction<T>> ProcessTransaction<T> for Arc<C> {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).process_transaction(transaction)
    }
}

pub trait SimulateTransaction<T>: Client {
    fn simulate_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T>;
}

impl<T, C: ?Sized + SimulateTransaction<T>> SimulateTransaction<T> for &C {
    fn simulate_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).simulate_transaction(transaction)
    }
}

impl<T, C: ?Sized + SimulateTransaction<T>> SimulateTransaction<T> for &mut C {
    fn simulate_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).simulate_transaction(transaction)
    }
}

impl<T, C: ?Sized + SimulateTransaction<T>> SimulateTransaction<T> for Box<C> {
    fn simulate_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).simulate_transaction(transaction)
    }
}

impl<T, C: ?Sized + SimulateTransaction<T>> SimulateTransaction<T> for Arc<C> {
    fn simulate_transaction(&self, transaction: VersionedTransaction) -> ClientResult<T> {
        (**self).simulate_transaction(transaction)
    }
}
