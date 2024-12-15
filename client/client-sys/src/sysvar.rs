use solana_sdk::sysvar::clock::Clock;
#[allow(deprecated)]
use solana_sdk::sysvar::recent_blockhashes::RecentBlockhashes;
use solana_sdk::sysvar::rent::Rent;
use solana_sdk::sysvar::slot_hashes::SlotHashes;
use solana_sdk::sysvar::slot_history::SlotHistory;
use solana_sdk::sysvar::Sysvar;

use dexter_client_api::base::getter::GetAccount;
use dexter_client_api::errors::{ClientError, ClientResult};
use dexter_client_api::exts::getter::GetAccountExt;
use dexter_client_api::Client;

pub trait SysvarGetter: Client + GetAccount {
    fn get_sysvar<T: Sysvar>(&self) -> ClientResult<Option<T>> {
        let Some(account) = self.get_account(&T::id())? else {
            return Ok(None);
        };

        let sysvar = bincode::deserialize(&account.data)
            .map_err(|_| ClientError::AccountDidNotDeserialize(T::id()))?;

        Ok(Some(sysvar))
    }

    fn get_sysvar_clock(&self) -> ClientResult<Option<Clock>> {
        self.get_sysvar()
    }

    fn get_sysvar_rent(&self) -> ClientResult<Option<Rent>> {
        self.get_sysvar()
    }

    fn get_sysvar_slothashes(&self) -> ClientResult<Option<SlotHashes>> {
        self.get_sysvar()
    }

    fn get_sysvar_slothistory(&self) -> ClientResult<Option<SlotHistory>> {
        self.get_sysvar()
    }

    #[allow(deprecated)]
    fn get_sysvar_recent_blockhashes(&self) -> ClientResult<Option<RecentBlockhashes>> {
        self.get_sysvar()
    }

    fn try_get_sysvar<T: Sysvar>(&self) -> ClientResult<T> {
        let account = self.try_get_account(&T::id())?;

        let sysvar = bincode::deserialize(&account.data)
            .map_err(|_| ClientError::AccountDidNotDeserialize(T::id()))?;

        Ok(sysvar)
    }

    fn try_get_sysvar_clock(&self) -> ClientResult<Clock> {
        self.try_get_sysvar()
    }

    fn try_get_sysvar_rent(&self) -> ClientResult<Rent> {
        self.try_get_sysvar()
    }

    fn try_get_sysvar_slothashes(&self) -> ClientResult<SlotHashes> {
        self.try_get_sysvar()
    }

    fn try_get_sysvar_slothistory(&self) -> ClientResult<SlotHistory> {
        self.try_get_sysvar()
    }

    #[allow(deprecated)]
    fn try_get_sysvar_recent_blockhashes(&self) -> ClientResult<RecentBlockhashes> {
        self.try_get_sysvar()
    }
}

impl<C: ?Sized + Client + GetAccount> SysvarGetter for C {}
