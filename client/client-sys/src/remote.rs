use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

use dexter_client_api::base::setter::SetAccount;
use dexter_client_api::errors::ClientResult;
use dexter_client_api::exts::getter::GetAccountExt;
use dexter_client_api::Client;

pub trait RemoteSetter: Client {
    fn set_account_from_remote<U: ToString>(
        &mut self,
        pubkey: Pubkey,
        rpcurl: U,
    ) -> ClientResult<Account>
    where
        Self: SetAccount,
    {
        let account = RpcClient::new(rpcurl).try_get_account(&pubkey)?;

        self.set_account(pubkey, account.clone());

        Ok(account)
    }
}

impl<C: ?Sized + Client> RemoteSetter for C {}
