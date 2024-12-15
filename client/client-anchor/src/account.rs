use std::mem::size_of;
use std::ops::{Deref, DerefMut};

use solana_sdk::account::{Account, AccountSharedData, ReadableAccount};
use solana_sdk::clock::Epoch;
use solana_sdk::pubkey::Pubkey;

use anchor_lang::{AccountDeserialize, AccountSerialize, Key, ZeroCopy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorAccount<T> {
    key: Pubkey,
    account: Account,
    data: T,
}

impl<T> AnchorAccount<T> {
    pub fn try_from_account(key: Pubkey, account: Account) -> anchor_lang::Result<Self>
    where
        T: AccountDeserialize,
    {
        let data = T::try_deserialize(&mut account.data())?;
        Ok(Self { key, account, data })
    }

    pub fn as_parts(this: &Self) -> (&Pubkey, &Account, &T) {
        (&this.key, &this.account, &this.data)
    }

    pub fn into_account(this: Self) -> Account {
        this.account
    }

    pub fn into_data(this: Self) -> T {
        this.data
    }

    pub fn into_parts(this: Self) -> (Pubkey, Account, T) {
        (this.key, this.account, this.data)
    }

    pub fn serializable_mut(&mut self) -> SerializableMut<T>
    where
        T: AccountSerialize,
    {
        SerializableMut {
            key: &self.key,
            account: &mut self.account,
            data: &mut self.data,
        }
    }

    pub fn loadable_mut(&mut self) -> LoadableMut<T>
    where
        T: ZeroCopy,
    {
        LoadableMut {
            key: &self.key,
            account: &mut self.account,
            data: &mut self.data,
        }
    }
}

impl<T: AccountDeserialize> TryFrom<(Pubkey, Account)> for AnchorAccount<T> {
    type Error = anchor_lang::error::Error;

    fn try_from((key, account): (Pubkey, Account)) -> anchor_lang::Result<Self> {
        Self::try_from_account(key, account)
    }
}

impl<T: AccountDeserialize> TryFrom<(Pubkey, AccountSharedData)> for AnchorAccount<T> {
    type Error = anchor_lang::error::Error;

    fn try_from((key, account): (Pubkey, AccountSharedData)) -> anchor_lang::Result<Self> {
        Self::try_from_account(key, account.into())
    }
}

impl<T> From<AnchorAccount<T>> for Account {
    fn from(account: AnchorAccount<T>) -> Self {
        account.account
    }
}

impl<T> From<AnchorAccount<T>> for AccountSharedData {
    fn from(account: AnchorAccount<T>) -> Self {
        account.account.into()
    }
}

impl<T> AsRef<Pubkey> for AnchorAccount<T> {
    fn as_ref(&self) -> &Pubkey {
        &self.key
    }
}

impl<T> AsRef<Account> for AnchorAccount<T> {
    fn as_ref(&self) -> &Account {
        &self.account
    }
}

impl<T> Deref for AnchorAccount<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Key for AnchorAccount<T> {
    fn key(&self) -> Pubkey {
        self.key
    }
}

impl<T> ReadableAccount for AnchorAccount<T> {
    fn lamports(&self) -> u64 {
        self.account.lamports()
    }

    fn data(&self) -> &[u8] {
        self.account.data()
    }

    fn owner(&self) -> &Pubkey {
        self.account.owner()
    }

    fn executable(&self) -> bool {
        self.account.executable()
    }

    fn rent_epoch(&self) -> Epoch {
        self.account.rent_epoch()
    }

    fn to_account_shared_data(&self) -> AccountSharedData {
        self.account.to_account_shared_data()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SerializableMut<'a, T: AccountSerialize> {
    key: &'a Pubkey,
    account: &'a mut Account,
    data: &'a mut T,
}

impl<'a, T: AccountSerialize> Deref for SerializableMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T: AccountSerialize> DerefMut for SerializableMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T: AccountSerialize> Key for SerializableMut<'a, T> {
    fn key(&self) -> Pubkey {
        *self.key
    }
}

impl<'a, T: AccountSerialize> Drop for SerializableMut<'a, T> {
    fn drop(&mut self) {
        self.account.data.clear();
        self.data.try_serialize(&mut self.account.data).unwrap();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoadableMut<'a, T: ZeroCopy> {
    key: &'a Pubkey,
    account: &'a mut Account,
    data: &'a mut T,
}

impl<'a, T: ZeroCopy> Deref for LoadableMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T: ZeroCopy> DerefMut for LoadableMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T: ZeroCopy> Key for LoadableMut<'a, T> {
    fn key(&self) -> Pubkey {
        *self.key
    }
}

impl<'a, T: ZeroCopy> Drop for LoadableMut<'a, T> {
    fn drop(&mut self) {
        let written = bytemuck::from_bytes_mut::<T>(&mut self.account.data[8..size_of::<T>() + 8]);
        *written = *self.data;
    }
}
