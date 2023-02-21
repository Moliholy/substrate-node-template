#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    #[api_version(1)]
    pub trait TemplateApi {
        fn get_files() -> Vec<(Vec<u8>, u32)>;
    }
}