use std::cmp::Ordering;

use solana_sdk::clock::Slot;
use solana_sdk::feature::{self, Feature};
use solana_sdk::feature_set::{FeatureSet, FEATURE_NAMES};
use solana_sdk::pubkey::Pubkey;

use dexter_client_api::base::getter::{GetAccount, GetProgramAccounts, ProgramAccountsFilter};
use dexter_client_api::errors::{ClientError, ClientResult};
use dexter_client_api::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureStatus {
    Inactive,
    Pending,
    Active(Slot),
}

impl PartialOrd for FeatureStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Same as `solana_cli::feature::CliFeatureStatus`
impl Ord for FeatureStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Inactive, Self::Inactive) => Ordering::Equal,
            (Self::Inactive, _) => Ordering::Greater,
            (_, Self::Inactive) => Ordering::Less,
            (Self::Pending, Self::Pending) => Ordering::Equal,
            (Self::Pending, _) => Ordering::Greater,
            (_, Self::Pending) => Ordering::Less,
            (Self::Active(active_slot), Self::Active(other_active_slot)) => {
                active_slot.cmp(other_active_slot)
            }
        }
    }
}

impl From<Feature> for FeatureStatus {
    fn from(feature: Feature) -> Self {
        match feature.activated_at {
            Some(slot) => Self::Active(slot),
            None => Self::Pending,
        }
    }
}

impl From<FeatureStatus> for Option<Feature> {
    fn from(status: FeatureStatus) -> Self {
        match status {
            FeatureStatus::Inactive => None,
            FeatureStatus::Pending => Some(Feature { activated_at: None }),
            FeatureStatus::Active(slot) => Some(Feature {
                activated_at: Some(slot),
            }),
        }
    }
}

pub trait FeatureGetter: Client {
    fn get_feature(&self, feature_id: &Pubkey) -> ClientResult<Option<Feature>>
    where
        Self: GetAccount,
    {
        let Some(feature_account) = self.get_account(feature_id)? else {
            return Ok(None);
        };

        let feature = feature::from_account(&feature_account)
            .ok_or(ClientError::AccountDidNotDeserialize(*feature_id))?;

        Ok(Some(feature))
    }

    fn try_get_feature(&self, feature_id: &Pubkey) -> ClientResult<Feature>
    where
        Self: GetAccount,
    {
        match self.get_feature(feature_id)? {
            Some(feature) => Ok(feature),
            None => Err(ClientError::AccountNotFound(*feature_id)),
        }
    }

    fn get_feature_status(&self, feature_id: &Pubkey) -> ClientResult<Option<FeatureStatus>>
    where
        Self: GetAccount,
    {
        let status = match self.get_feature(feature_id)? {
            Some(feature) => Some(FeatureStatus::from(feature)),
            None if FEATURE_NAMES.contains_key(feature_id) => Some(FeatureStatus::Inactive),
            None => None,
        };

        Ok(status)
    }

    fn try_get_feature_status(&self, feature_id: &Pubkey) -> ClientResult<FeatureStatus>
    where
        Self: GetAccount,
    {
        match self.get_feature_status(feature_id)? {
            Some(status) => Ok(status),
            None => Err(ClientError::AccountNotFound(*feature_id)),
        }
    }

    fn get_features(&self) -> ClientResult<Vec<(Pubkey, Feature)>>
    where
        Self: GetProgramAccounts,
    {
        let feature_accounts = self.get_program_accounts(
            &feature::id(),
            Some(vec![ProgramAccountsFilter::DataSize(
                Feature::size_of() as u64
            )]),
        )?;

        feature_accounts
            .into_iter()
            .map(|(id, acc)| {
                let feature =
                    feature::from_account(&acc).ok_or(ClientError::AccountDidNotDeserialize(id))?;
                Ok((id, feature))
            })
            .collect()
    }

    fn get_feature_set(&self) -> ClientResult<FeatureSet>
    where
        Self: GetProgramAccounts,
    {
        let mut feature_set = FeatureSet::default();

        for (feature_id, feature) in self.get_features()? {
            let Some(slot) = feature.activated_at else {
                continue;
            };

            if !feature_set.inactive.contains(&feature_id) {
                continue;
            }

            feature_set.activate(&feature_id, slot);
        }

        Ok(feature_set)
    }
}

impl<C: ?Sized + Client> FeatureGetter for C {}
