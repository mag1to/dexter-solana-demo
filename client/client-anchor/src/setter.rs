use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

use anchor_lang::{AccountSerialize, Owner, ZeroCopy};

use dexter_client_api::base::setter::{HasRent, SetAccount};
use dexter_client_api::Client;

pub trait AnchorSetter: Client {
    fn set_anchor_account_zero_copy<T>(
        &mut self,
        pubkey: Pubkey,
        account: &T,
        owner: Option<Pubkey>,
    ) where
        Self: SetAccount + HasRent,
        T: ZeroCopy + Owner,
    {
        let data = {
            let disc = T::discriminator();
            let bytes = bytemuck::bytes_of(account);

            let mut data = Vec::with_capacity(disc.len() + bytes.len());
            data.extend_from_slice(&disc);
            data.extend_from_slice(bytes);
            data
        };

        let account = Account {
            lamports: self.minimum_balance_for_rent_exemption(data.len()),
            data,
            owner: owner.unwrap_or(T::owner()),
            executable: false,
            rent_epoch: u64::MAX,
        };

        self.set_account(pubkey, account);
    }

    fn set_anchor_account<T>(&mut self, pubkey: Pubkey, account: &T, owner: Option<Pubkey>)
    where
        Self: SetAccount + HasRent,
        T: AccountSerialize + Owner,
    {
        let mut data = Vec::new();
        account.try_serialize(&mut data).unwrap();

        let account = Account {
            lamports: self.minimum_balance_for_rent_exemption(data.len()),
            data,
            owner: owner.unwrap_or(T::owner()),
            executable: false,
            rent_epoch: u64::MAX,
        };

        self.set_account(pubkey, account);
    }
}

impl<C: ?Sized + Client> AnchorSetter for C {}
