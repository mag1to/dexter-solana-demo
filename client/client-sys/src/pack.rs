use solana_sdk::account::Account;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;

use dexter_client_api::base::setter::SetAccount;
use dexter_client_api::Client;

pub trait PackingSetter: Client {
    fn packing_set_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        lamports: u64,
        owner: Pubkey,
        account_data: &T,
    ) -> Account
    where
        Self: SetAccount,
    {
        let mut account = Account::new(lamports, T::get_packed_len(), &owner);
        account_data.pack_into_slice(&mut account.data);
        self.set_account(pubkey, account.clone());
        account
    }
}

impl<C: ?Sized + Client> PackingSetter for C {}
