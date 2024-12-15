use std::sync::Arc;

use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;

use crate::client::Client;

pub trait SetAccount: Client {
    fn set_account(&mut self, pubkey: Pubkey, account: Account);
}

impl<C: ?Sized + SetAccount> SetAccount for &mut C {
    fn set_account(&mut self, pubkey: Pubkey, account: Account) {
        (**self).set_account(pubkey, account)
    }
}

impl<C: ?Sized + SetAccount> SetAccount for Box<C> {
    fn set_account(&mut self, pubkey: Pubkey, account: Account) {
        (**self).set_account(pubkey, account)
    }
}

pub trait HasRent: Client {
    fn rent(&self) -> Rent;

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        self.rent().minimum_balance(data_len).max(1)
    }
}

impl<T: ?Sized + HasRent> HasRent for &T {
    fn rent(&self) -> Rent {
        (**self).rent()
    }

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        (**self).minimum_balance_for_rent_exemption(data_len)
    }
}

impl<T: ?Sized + HasRent> HasRent for &mut T {
    fn rent(&self) -> Rent {
        (**self).rent()
    }

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        (**self).minimum_balance_for_rent_exemption(data_len)
    }
}

impl<T: ?Sized + HasRent> HasRent for Box<T> {
    fn rent(&self) -> Rent {
        (**self).rent()
    }

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        (**self).minimum_balance_for_rent_exemption(data_len)
    }
}

impl<T: ?Sized + HasRent> HasRent for Arc<T> {
    fn rent(&self) -> Rent {
        (**self).rent()
    }

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        (**self).minimum_balance_for_rent_exemption(data_len)
    }
}
