#![deny(clippy::unwrap_used)]
#![forbid(unsafe_code)]
#![deny(clippy::expect_used)]
#![forbid(clippy::panic)]

mod common_types;

pub mod v1 {
    pub mod common {
        tonic::include_proto!("phantom_store.v1.common");
    }
    pub mod admin {
        tonic::include_proto!("phantom_store.v1.admin");
    }
    pub mod auth {
        pub mod admin {
            // it's empty
            // tonic::include_proto!("phantom_store.v1.auth.admin");
        }
        pub mod common {
            tonic::include_proto!("phantom_store.v1.auth.common");
        }
        pub mod user {
            tonic::include_proto!("phantom_store.v1.auth.user");
        }
    }
}
