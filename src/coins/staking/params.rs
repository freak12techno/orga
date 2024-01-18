use orga_macros::orga;
use crate::coins::Decimal;

#[orga(skip(Serialize, Deserialize))]
pub struct StakingParams {
    pub unbonding_seconds: u64,
    pub max_validators: u64,
}

#[orga(skip(Serialize, Deserialize))]
pub struct SlashingParams {
    pub max_offline_blocks: u64,
    pub slash_fraction_double_sign: Decimal,
    pub slash_fraction_downtime: Decimal,
    pub downtime_jail_seconds: u64,
}