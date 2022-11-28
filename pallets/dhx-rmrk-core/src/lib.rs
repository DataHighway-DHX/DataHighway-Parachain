#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::*;
    use frame_support::{transactional, sp_runtime::Permill};
    use pallet_rmrk_core::BoundedResourceInfoTypeOf;
    use pallet_rmrk_core::WeightInfo;

    type NftId<T> = <T as pallet_uniques::pallet::Config>::ItemId;
    type CollectionId<T> = <T as pallet_uniques::pallet::Config>::CollectionId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_rmrk_core::Config {
	    /// Who can mint nft
		type ProducerOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

        /// EVent
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
    }


	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
        // Call signature, fee & else should be same as actual rmrk-core
        //
        /// Mints an NFT in the specified collection
		/// Sets metadata and the royalty attribute
		///
		/// Parameters:
		/// - `collection_id`: The collection of the asset to be minted.
		/// - `nft_id`: The nft value of the asset to be minted.
		/// - `recipient`: Receiver of the royalty
		/// - `royalty`: Permillage reward from each trade for the Recipient
		/// - `metadata`: Arbitrary data about an nft, e.g. IPFS hash
        #[pallet::weight(<T as pallet_rmrk_core::Config>::WeightInfo::mint_nft() )]
		#[transactional]
        pub fn mint_nft(
			origin: OriginFor<T>,
			owner: Option<T::AccountId>,
			nft_id: NftId<T>,
			collection_id: CollectionId<T>,
			royalty_recipient: Option<T::AccountId>,
			royalty: Option<Permill>,
			metadata: BoundedVec<u8, T::StringLimit>,
			transferable: bool,
			resources: Option<BoundedResourceInfoTypeOf<T>>,
		) -> DispatchResult {
            let _minter: T::AccountId = T::ProducerOrigin::ensure_origin(origin.clone())?;
            pallet_rmrk_core::Pallet::<T>::mint_nft(
                origin,
                owner,
                nft_id,
                collection_id,
                royalty_recipient,
                royalty,
                metadata,
                transferable,
                resources
            )
        }
	}
}
