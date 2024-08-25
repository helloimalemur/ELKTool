pub mod alerts_api_funcs;
pub mod app_index;
pub mod common;
pub mod ilm_api;
pub mod lifetime_api;
pub mod notifications;
pub mod replicate;
pub mod sanitize;
pub mod search_settings;
pub mod snapshot_api;
pub mod snapshot_repository;
pub mod tests;
pub mod transform;

//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
