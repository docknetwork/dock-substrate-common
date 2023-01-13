use crate::*;

pub fn migrate_to_candidacy_delay<T: Config>() -> Weight {
    let _ = <Candidates<T>>::translate::<Vec<(T::AccountId, BalanceOf<T>)>, _>(
        |maybe_old_candidates| {
            maybe_old_candidates.map(|old_candidates| {
                log::info!(
                    target: "runtime::elections-phragmen",
                    "migrated {} candidate accounts.",
                    old_candidates.len(),
                );

                old_candidates
                    .into_iter()
                    .map(|(candidate, deposit)| (candidate, deposit, 0u32.into()))
                    .collect::<Vec<_>>()
            })
        },
    );

    T::DbWeight::get().reads_writes(1, 1)
}
