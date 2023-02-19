#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

mod file_merkle_tree;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_256;
    use sp_std::vec::Vec;
    use crate::file_merkle_tree::FileMerkleTree;

	#[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // Pallets use events to inform users when important changes are made.
    // Event documentation should end with an array that provides descriptive names for parameters.
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when a claim has been created.
        ClaimCreated { who: T::AccountId, claim: T::Hash },
        /// Event emitted when a claim is revoked by the owner.
        ClaimRevoked { who: T::AccountId, claim: T::Hash },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The claim already exists.
        AlreadyClaimed,
        /// The claim does not exist, so it cannot be revoked.
        NoSuchClaim,
        /// The claim is owned by another account, so caller can't revoke it.
        NotClaimOwner,
    }

    #[pallet::storage]
    pub(super) type Claims<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, T::BlockNumber)>;

    // Dispatchable functions allow users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        #[pallet::call_index(0)]
        pub fn create_claim(origin: OriginFor<T>, file_bytes: Vec<u8>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let sender = ensure_signed(origin)?;

            let file_merkle_tree = FileMerkleTree::new(file_bytes.clone());
            let hash = file_merkle_tree.root_hash().to_vec();
            let claim = T::Hash::decode(
                &mut hash.as_slice()
            ).unwrap();

            // Verify that the specified claim has not already been stored.
            ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

            // Get the block number from the FRAME System pallet.
            let current_block = <frame_system::Pallet<T>>::block_number();

            // Store the claim with the sender and block number.
            Claims::<T>::insert(&claim, (&sender, current_block));

            // Emit an event that the claim was created.
            Self::deposit_event(Event::ClaimCreated { who: sender, claim });

            Ok(())
        }
    }
}