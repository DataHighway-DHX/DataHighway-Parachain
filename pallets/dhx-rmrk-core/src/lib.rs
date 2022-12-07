#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::OptionQuery;
    use frame_support::pallet_prelude::*;
    use frame_system::ensure_root;
    use frame_system::pallet_prelude::*;
    use frame_support::{transactional, sp_runtime::Permill};
    use pallet_rmrk_core::BoundedResourceInfoTypeOf;
    use pallet_rmrk_core::BoundedCollectionSymbolOf;
    use pallet_rmrk_core::WeightInfo;

    type NftId<T> = <T as pallet_uniques::pallet::Config>::ItemId;
    type CollectionId<T> = <T as pallet_uniques::pallet::Config>::CollectionId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub allowed_producers: Vec<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
	    fn default() -> Self {
		    Self {
                allowed_producers: vec![],
            }
	    }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
	    fn build(&self) {
	        for producer in &self.allowed_producers {
                <AuthorisedProducers<T>>::insert(producer, ());
            }
        }
    }

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_rmrk_core::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient permission
        InsufficientPermission,
        /// Given producer already exists in producer list
        AlreadyAProducer,
        /// Given producer is not in producer list
        NotAProducer,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New authorised producer have been added to list
        AddedAuthorisedProducer(T::AccountId),
        /// Given authorised producer have been removed from list
        RemovedAuthorisedProducer(T::AccountId),
    }


    #[pallet::storage]
    #[pallet::getter(fn get_authorised_producer)]
    pub type AuthorisedProducers<T> = StorageMap<_, Twox64Concat, <T as frame_system::Config>::AccountId, (), OptionQuery>;

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
            Self::ensure_authorised(origin.clone())?;

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

        /// Mints an NFT in the specified collection directly to another NFT
		/// Sets metadata and the royalty attribute
		///
		/// Parameters:
		/// - `collection_id`: The class of the asset to be minted.
		/// - `nft_id`: The nft value of the asset to be minted.
		/// - `recipient`: Receiver of the royalty
		/// - `royalty`: Permillage reward from each trade for the Recipient
		/// - `metadata`: Arbitrary data about an nft, e.g. IPFS hash
		#[pallet::weight(<T as pallet_rmrk_core::Config>::WeightInfo::mint_nft_directly_to_nft())]
		#[transactional]
		pub fn mint_nft_directly_to_nft(
			origin: OriginFor<T>,
			owner: (T::CollectionId, T::ItemId),
			nft_id: T::ItemId,
			collection_id: T::CollectionId,
			royalty_recipient: Option<T::AccountId>,
			royalty: Option<Permill>,
			metadata: BoundedVec<u8, T::StringLimit>,
			transferable: bool,
			resources: Option<BoundedResourceInfoTypeOf<T>>,
		) -> DispatchResult {
            Self::ensure_authorised(origin.clone())?;

            pallet_rmrk_core::Pallet::<T>::mint_nft_directly_to_nft(
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

        /// Create a collection
		#[pallet::weight(<T as pallet_rmrk_core::Config>::WeightInfo::create_collection())]
		#[transactional]
		pub fn create_collection(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			metadata: BoundedVec<u8, T::StringLimit>,
			max: Option<u32>,
			symbol: BoundedCollectionSymbolOf<T>,
		) -> DispatchResult {
            Self::ensure_authorised(origin.clone())?;

            pallet_rmrk_core::Pallet::<T>::create_collection(origin, collection_id, metadata, max, symbol)
        }

        /// Add a account to producer list
        #[pallet::weight(T::DbWeight::get().writes(2))]
        pub fn add_producer(origin: OriginFor<T>, producer: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(!<AuthorisedProducers<T>>::contains_key(&producer), <Error<T>>::AlreadyAProducer);
            <AuthorisedProducers<T>>::insert(&producer, ());

            Self::deposit_event(<Event<T>>::AddedAuthorisedProducer(producer));
            Ok(())
        }

        /// Add given account to producer list
        #[pallet::weight(T::DbWeight::get().writes(2))]
        pub fn remove_producer(origin: OriginFor<T>, producer: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(<AuthorisedProducers<T>>::contains_key(&producer), <Error<T>>::NotAProducer);
            <AuthorisedProducers<T>>::remove(&producer);

            Self::deposit_event(<Event<T>>::RemovedAuthorisedProducer(producer));
            Ok(())
        }
	}

	impl<T: Config> Pallet<T> {
        fn ensure_authorised(origin: OriginFor<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            Self::get_authorised_producer(caller)
                .ok_or(Error::<T>::InsufficientPermission.into())
        }
    }
}
