//! Pallet TDG Governance
//! Demonstrates TDG‑driven off‑chain worker governance on Substrate.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::offchain as ocw;

    #[pallet::pallet]
    #[pallet::hooks]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ConfigUpdated { committee_size: u32, threshold: u32 },
        TallyCompleted { round: u64, success: bool },
    }

    #[pallet::storage]
    #[pallet::getter(fn tdg_params)]
    pub type TdgParameters<T: Config> = StorageValue<_, (u32, u32), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_round)]
    pub type CurrentRound<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn committee_size)]
    pub type CommitteeSize<T: Config> = StorageValue<_, u32, ValueQuery, TdgParameters<T>, 0>;

    #[pallet::storage]
    #[pallet::getter(fn threshold)]
    pub type Threshold<T: Config> = StorageValue<_, u32, ValueQuery, TdgParameters<T>, 1>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn update_tdg_parameters(
            origin: OriginFor<T>,
            committee_size: u32,
            threshold: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            CommitteeSize::<T>::put(committee_size);
            Threshold::<T>::put(threshold);
            Self::deposit_event(Event::ConfigUpdated { committee_size, threshold });
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_n: BlockNumberFor<T>) {
            let (committee_size, _threshold) = TdgParameters::<T>::get();
            log::info!("OCW: TDG params {:?}", (committee_size));

            let payload = 42u64;
            if let Err(err) = Self::submit_tally_transaction(payload) {
                log::error!("Failed to submit tally tx: {:?}", err);
            }
        }
    }

    impl<T: Config> Pallet<T> {
        fn submit_tally_transaction(payload: u64) -> Result<(), &'static str> {
            let call = frame_system::Call::<T>::remark { remark: payload.encode() };
            let signed = ocw::SendSignedTransaction::<T, Call<T>>::send_signed_transaction(
                call.into(),
                true,
            );
            match signed {
                Ok(Some(_tx_hash)) => {
                    log::info!("OCW: Submitted tally tx");
                    Ok(())
                }
                Ok(None) => {
                    log::info!("OCW: No local account for signing");
                    Ok(())
                }
                Err(err) => {
                    log::error!("OCW: Failed to submit tx: {:?}", err);
                    Err("offchain_tx_failed")
                }
            }
        }
    }
}
