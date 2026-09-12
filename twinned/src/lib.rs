//! # twinned
//!
//! > **Entirely AI-generated code.** Every line of this crate was written by
//! > Claude (Anthropic). No human wrote it and no human has audited it line by
//! > line. Read [the readme](https://github.com/disconsented/twinned) before
//! > you put it near anything you care about.
//!
//! One struct definition, two generated types: an *internal* type that talks
//! to [SurrealDB](https://surrealdb.com) with `RecordId` fields, and an
//! *external* type that you expose over an API with plain keys. The crate
//! also generates the conversions between the two.
//!
//! ```rust,ignore
//! use twinned::{define_id, dual_struct};
//!
//! define_id!("apps", IAppID, AppID, i64);
//!
//! #[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
//! pub struct App {
//!     #[dual_type(IAppID)]
//!     pub id: AppID,
//!     pub title: String,
//! }
//! // -> InternalApp { id: IAppID, title: String }
//! // -> ExternalApp { id: AppID,  title: String }
//! // -> impl TryFrom<InternalApp> for ExternalApp
//! // -> impl From<ExternalApp> for InternalApp
//! ```
//!
//! ## Dependencies in your crate
//!
//! Both macros expand to paths that resolve in *your* crate, not in this one.
//! Your `Cargo.toml` must therefore have `serde` (with the `derive` feature),
//! `surrealdb-types`, and, with the `salvo` feature on, `salvo` (with the
//! `oapi` feature). The module that calls [`define_id`] must also have
//! `surrealdb_types::SurrealValue` in scope.
//!
//! ## Features
//!
//! - `salvo` (off by default): adds `salvo::prelude::ToSchema` to every
//!   external type, so the API side of each pair appears in your OpenAPI
//!   document.

pub use twinned_macros::dual_struct;

/// Declares the external half of an ID pair.
///
/// Internal helper for [`define_id`]. The `salvo` feature is read here, in
/// `twinned`, and not in the crate that calls the macro.
#[cfg(feature = "salvo")]
#[doc(hidden)]
#[macro_export]
macro_rules! __twinned_external_id {
    ($external:ident, $external_type:ty) => {
        /// Externally facing newtype over the ID's plain key type.
        #[derive(
            serde::Serialize,
            serde::Deserialize,
            Clone,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            salvo::prelude::ToSchema,
        )]
        #[serde(transparent)]
        pub struct $external($external_type);
    };
}

/// Declares the external half of an ID pair, without the salvo schema derive.
#[cfg(not(feature = "salvo"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __twinned_external_id {
    ($external:ident, $external_type:ty) => {
        /// Externally facing newtype over the ID's plain key type.
        #[derive(
            serde::Serialize,
            serde::Deserialize,
            Clone,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
        )]
        #[serde(transparent)]
        pub struct $external($external_type);
    };
}

/// A helper macro to define a newtype for a SurrealDB record ID.
/// Makes it not a total pain in the ass to deal with.
///
/// Implemented with 2 structs because, we can't maintain a transparent newtype
/// with an enum, which, makes sense as its a different type.
///
/// ## Args:
/// Table name: The name of the SurrealDB table\
/// Internal name: The name of the internal newtype (interfaces with the DB)\
/// External name: The name of the external newtype (exposed to users)\
/// External type: The underlying type of the external newtype
#[macro_export]
macro_rules! define_id {
    ($table:literal, $internal:ident, $external:ident, $external_type:ty) => {
        $crate::__twinned_external_id!($external, $external_type);

        impl From<$external> for $external_type {
            fn from(id: $external) -> $external_type {
                id.0
            }
        }

        impl From<$external_type> for $external {
            fn from(id: $external_type) -> Self {
                Self(id)
            }
        }
        /// Internally facing newtype over RecordID
        #[derive(
            serde::Serialize,
            serde::Deserialize,
            Clone,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            surrealdb_types::SurrealValue,
        )]
        #[serde(transparent)]
        pub struct $internal(surrealdb_types::RecordId);
        impl $internal {
            const TABLE_NAME: &'static str = $table;

            // An alias of TryInto<$external> for $internal to make it easier to use
            pub fn try_into_external(self) -> Result<$external, surrealdb_types::Error> {
                self.try_into()
            }
        }
        impl From<$external> for $internal {
            fn from(id: $external) -> Self {
                #[allow(
                    clippy::disallowed_methods,
                    reason = "Macro use only, this is considered correct"
                )]
                Self(surrealdb_types::RecordId::new(Self::TABLE_NAME, id.0))
            }
        }
        impl From<$external_type> for $internal {
            fn from(id: $external_type) -> Self {
                #[allow(
                    clippy::disallowed_methods,
                    reason = "Macro use only, this is considered correct"
                )]
                Self(surrealdb_types::RecordId::new(Self::TABLE_NAME, id))
            }
        }

        impl From<$internal> for surrealdb_types::RecordId {
            fn from(id: $internal) -> surrealdb_types::RecordId {
                id.0
            }
        }

        impl TryFrom<$internal> for $external {
            type Error = surrealdb_types::Error;

            fn try_from(id: $internal) -> Result<$external, Self::Error> {
                Ok($external(id.0.key.into_value().into_t()?))
            }
        }

        impl std::ops::Deref for $internal {
            type Target = surrealdb_types::RecordId;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
