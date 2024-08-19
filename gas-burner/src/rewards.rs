use crate::week_timekeeping::{Week, INVALID_WEEK_ERR_MSG};

multiversx_sc::imports!();

pub const TOP_LEADERBOARD_USERS_FOR_PRIZES: usize = 10;

#[multiversx_sc::module]
pub trait RewardsModule:
    crate::leaderboard::LeaderboardModule
    + crate::week_timekeeping::WeekTimekeepingModule
    + crate::signature::SignatureModule
{
    #[endpoint(claimRewards)]
    fn claim_rewards(&self, start_week: Week) -> BigUint {
        require!(start_week > 0, INVALID_WEEK_ERR_MSG);

        let current_week = self.get_current_week();
        require!(start_week < current_week, "Invalid start week");

        let previous_week = current_week - 1;
        self.claim_developer_rewards(previous_week);

        let caller = self.blockchain().get_caller();
        let user_id = self.user_id().get_id_non_zero(&caller);
        let total_rewards = self.claim_user_rewards(start_week, current_week, user_id);
        self.send().direct_non_zero_egld(&caller, &total_rewards);

        total_rewards
    }

    fn claim_developer_rewards(&self, previous_week: Week) {
        let developer_rewards_claimed_for_week_mapper =
            self.developer_rewards_claimed_for_week(previous_week);
        if developer_rewards_claimed_for_week_mapper.get() {
            return;
        }

        let own_sc_address = self.blockchain().get_sc_address();
        let egld_balance_before = self.blockchain().get_balance(&own_sc_address);
        self.send()
            .claim_developer_rewards(own_sc_address.clone())
            .sync_call();

        let egld_balance_after = self.blockchain().get_balance(&own_sc_address);
        let claimed_rewards = egld_balance_after - egld_balance_before;
        self.total_rewards_week(previous_week).set(claimed_rewards);

        developer_rewards_claimed_for_week_mapper.set(true);
    }

    fn claim_user_rewards(
        &self,
        start_week: Week,
        current_week: Week,
        user_id: AddressId,
    ) -> BigUint {
        let mut total_rewards = BigUint::zero();
        for week in start_week..current_week {
            let user_index_in_lb = self.user_index_in_leaderboard(user_id, week).get();
            if user_index_in_lb == 0 || user_index_in_lb > TOP_LEADERBOARD_USERS_FOR_PRIZES {
                continue;
            }

            let user_claimed_for_week_mapper = self.user_claimed_for_week(week);
            if user_claimed_for_week_mapper.get() {
                continue;
            }

            let total_work_for_week = self.calculate_top_total_user_work_for_week(week);
            let total_rewards_for_week = self.total_rewards_week(week).get();

            let user_entry = self.leaderboard(week).get_unchecked(user_index_in_lb);
            let user_rewards =
                total_rewards_for_week * user_entry.work_amount / total_work_for_week;
            total_rewards += user_rewards;

            user_claimed_for_week_mapper.set(true);
        }

        total_rewards
    }

    fn calculate_top_total_user_work_for_week(&self, week: Week) -> BigUint {
        let total_work_mapper = self.total_work_for_week(week);
        if !total_work_mapper.is_empty() {
            return total_work_mapper.get();
        }

        let leaderboard_mapper = self.leaderboard(week);
        let last_entry_index =
            core::cmp::min(leaderboard_mapper.len(), TOP_LEADERBOARD_USERS_FOR_PRIZES);
        let mut total_work = BigUint::zero();
        for i in 1..=last_entry_index {
            let lb_entry = leaderboard_mapper.get_unchecked(i);
            total_work += lb_entry.work_amount;
        }

        total_work_mapper.set(&total_work);

        total_work
    }

    #[storage_mapper("devRewardsClaimedForWeek")]
    fn developer_rewards_claimed_for_week(&self, week: Week) -> SingleValueMapper<bool>;

    #[storage_mapper("totalRewardsWeek")]
    fn total_rewards_week(&self, week: Week) -> SingleValueMapper<BigUint>;

    #[storage_mapper("totalWorkForWeek")]
    fn total_work_for_week(&self, week: Week) -> SingleValueMapper<BigUint>;

    #[storage_mapper("userClaimedForWeek")]
    fn user_claimed_for_week(&self, week: Week) -> SingleValueMapper<bool>;
}
