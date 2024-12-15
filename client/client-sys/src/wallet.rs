use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_program;

use dexter_client_api::base::setter::SetAccount;
use dexter_client_api::Client;

pub trait WalletSetter: Client {
    fn set_wallet(&mut self, pubkey: Pubkey, lamports: u64)
    where
        Self: SetAccount,
    {
        let account = Account {
            lamports,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: u64::MAX,
        };

        self.set_account(pubkey, account);
    }

    fn new_wallet(&mut self, lamports: u64) -> Keypair
    where
        Self: SetAccount,
    {
        let keypair = Keypair::new();
        self.set_wallet(keypair.pubkey(), lamports);
        keypair
    }
}

impl<C: ?Sized + Client> WalletSetter for C {}
