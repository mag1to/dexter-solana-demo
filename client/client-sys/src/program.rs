use solana_sdk::bpf_loader;
use solana_sdk::bpf_loader_deprecated;
use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};
use solana_sdk::loader_v4::{self, LoaderV4State};
use solana_sdk::pubkey::Pubkey;

use dexter_client_api::base::getter::GetAccount;
use dexter_client_api::errors::{ClientError, ClientResult};
use dexter_client_api::exts::getter::GetAccountExt;
use dexter_client_api::Client;

pub trait ProgramGetter: Client {
    fn get_program(&self, program_id: &Pubkey) -> ClientResult<Option<Vec<u8>>>
    where
        Self: GetAccount,
    {
        let Some(program_account) = self.get_account(program_id)? else {
            return Ok(None);
        };

        let loader_id = program_account.owner;
        let program = if loader_id == bpf_loader_upgradeable::id() {
            let program_state: UpgradeableLoaderState = bincode::deserialize(&program_account.data)
                .map_err(|_| ClientError::AccountDidNotDeserialize(*program_id))?;

            let UpgradeableLoaderState::Program {
                programdata_address,
            } = program_state
            else {
                return Err(ClientError::AccountDidNotDeserialize(*program_id));
            };

            let programdata_account = self.try_get_account(&programdata_address)?;

            programdata_account.data[UpgradeableLoaderState::size_of_programdata_metadata()..]
                .to_vec()
        } else if loader_id == loader_v4::id() {
            program_account.data[LoaderV4State::program_data_offset()..].to_vec()
        } else {
            assert!(loader_id == bpf_loader::id() || loader_id == bpf_loader_deprecated::id());
            program_account.data
        };

        Ok(Some(program))
    }

    fn try_get_program(&self, program_id: &Pubkey) -> ClientResult<Vec<u8>>
    where
        Self: GetAccount,
    {
        match self.get_program(program_id)? {
            Some(program) => Ok(program),
            None => Err(ClientError::AccountNotFound(*program_id)),
        }
    }
}

impl<C: ?Sized + Client> ProgramGetter for C {}
