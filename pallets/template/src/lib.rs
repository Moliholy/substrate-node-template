#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

mod file_merkle_tree;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
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


    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when a claim has been created.
        FileUploaded { who: T::AccountId, merkle_root: T::Hash, pieces: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Could not obtain the merkle root hash
        Unhasheable,
    }

    #[pallet::storage]
    pub(super) type Files<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, FileMerkleTree), OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        #[pallet::call_index(0)]
        pub fn upload_file(origin: OriginFor<T>, file_bytes: Vec<u8>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            let file_merkle_tree = FileMerkleTree::new(file_bytes.clone());
            let merkle_root = T::Hash::decode(
                &mut file_merkle_tree.merkle_root()
            ).or(Err(Error::<T>::Unhasheable))?;

            // Store the claim with the sender and block number.
            Files::<T>::insert(&merkle_root, (&who, &file_merkle_tree));

            // Emit an event that the claim was created.
            Self::deposit_event(Event::FileUploaded {
                who,
                merkle_root,
                pieces: file_merkle_tree.pieces,
            });

            Ok(())
        }
    }

    // RPC methods

    impl<T: Config> Pallet<T> {
        /// Get all file hashes ever submitted
        pub fn get_files() -> Vec<(Vec<u8>, u32)> {
            let result = Files::<T>::iter()
                .map(|(_, (_, tree))| (tree.merkle_root().to_vec(), tree.pieces))
                .collect::<Vec<(Vec<u8>, u32)>>();
            result
        }
    }
}