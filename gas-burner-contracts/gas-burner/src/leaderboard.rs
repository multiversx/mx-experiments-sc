use crate::week_timekeeping::Week;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub struct LeaderboardEntry<M: ManagedTypeApi> {
    pub user_id: AddressId,
    pub work_amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait LeaderboardModule:
    crate::week_timekeeping::WeekTimekeepingModule + crate::signature::SignatureModule
{
    /// A placement of 0 means the user does not exist for the given week
    #[view(getUserLeaderboardPlacement)]
    fn get_user_leaderboard_placement(&self, user: ManagedAddress, week: Week) -> usize {
        let user_id = self.user_id().get_id(&user);
        if user_id == 0 {
            return 0;
        }

        self.user_index_in_leaderboard(user_id, week).get()
    }

    fn increase_leaderboard_entry(&self, user: &ManagedAddress, amount: u64) {
        let current_week = self.get_current_week();
        let user_id = self.user_id().get_id_or_insert(user);
        let user_index = self.user_index_in_leaderboard(user_id, current_week).get();
        if user_index != 0 {
            self.increase_work_amount(user_index, amount, current_week);
        } else {
            self.add_new_user_entry(user_id, amount, current_week);
        }
    }

    fn add_new_user_entry(&self, user_id: AddressId, amount: u64, current_week: Week) {
        let mut leaderboard_mapper = self.leaderboard(current_week);
        let new_user_entry = LeaderboardEntry {
            user_id,
            work_amount: amount.into(),
        };
        let _ = leaderboard_mapper.push(&new_user_entry);

        let leaderboard_len = leaderboard_mapper.len();
        self.user_index_in_leaderboard(user_id, current_week)
            .set(leaderboard_len);

        if leaderboard_len == 1 {
            return;
        }

        self.set_new_user_entry_in_mapper(
            current_week,
            &new_user_entry,
            leaderboard_len,
            &mut leaderboard_mapper,
        );
    }

    fn set_new_user_entry_in_mapper(
        &self,
        current_week: Week,
        new_user_entry: &LeaderboardEntry<Self::Api>,
        leaderboard_len: usize,
        leaderboard_mapper: &mut VecMapper<LeaderboardEntry<Self::Api>>,
    ) {
        let mut new_user_index = leaderboard_len;
        for user_index in (1..=leaderboard_len - 1).rev() {
            let existing_user_entry = leaderboard_mapper.get_unchecked(user_index);
            if new_user_entry.work_amount <= existing_user_entry.work_amount {
                break;
            }

            self.move_user_down_lb(current_week, &existing_user_entry, leaderboard_mapper);
            new_user_index -= 1;
        }

        if new_user_index != leaderboard_len {
            leaderboard_mapper.set(new_user_index, new_user_entry);
        }

        self.user_index_in_leaderboard(new_user_entry.user_id, current_week)
            .set(new_user_index);
    }

    fn move_user_down_lb(
        &self,
        current_week: Week,
        user_entry: &LeaderboardEntry<Self::Api>,
        leaderboard_mapper: &mut VecMapper<LeaderboardEntry<Self::Api>>,
    ) {
        let prev_user_index = self
            .user_index_in_leaderboard(user_entry.user_id, current_week)
            .update(|user_index| {
                let prev_user_index = *user_index;
                *user_index += 1;

                prev_user_index
            });

        let new_user_index = prev_user_index + 1;
        leaderboard_mapper.set(new_user_index, user_entry);
    }

    fn increase_work_amount(&self, user_index: usize, work_amount: u64, current_week: Week) {
        let mut leaderboard_mapper = self.leaderboard(current_week);
        let mut user_entry = leaderboard_mapper.get(user_index);
        user_entry.work_amount += work_amount;

        self.update_leaderboard_placement(
            current_week,
            &user_entry,
            user_index,
            &mut leaderboard_mapper,
        );
    }

    fn update_leaderboard_placement(
        &self,
        current_week: Week,
        updated_user_entry: &LeaderboardEntry<Self::Api>,
        current_user_index: usize,
        leaderboard_mapper: &mut VecMapper<LeaderboardEntry<Self::Api>>,
    ) {
        let mut new_user_index = current_user_index;
        for user_index in (1..=current_user_index - 1).rev() {
            let existing_user_entry = leaderboard_mapper.get_unchecked(user_index);
            if updated_user_entry.work_amount <= existing_user_entry.work_amount {
                break;
            }

            self.move_user_down_lb(current_week, &existing_user_entry, leaderboard_mapper);
            new_user_index -= 1;
        }

        leaderboard_mapper.set(new_user_index, updated_user_entry);
        self.user_index_in_leaderboard(updated_user_entry.user_id, current_week)
            .set(new_user_index);
    }

    #[storage_mapper("userIndexInLb")]
    fn user_index_in_leaderboard(&self, user_id: AddressId, week: Week)
        -> SingleValueMapper<usize>;

    #[view(getLeaderboardForWeek)]
    #[storage_mapper("leaderboard")]
    fn leaderboard(&self, week: Week) -> VecMapper<LeaderboardEntry<Self::Api>>;
}
