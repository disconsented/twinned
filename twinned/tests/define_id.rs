use surrealdb_types::SurrealValue;
use twinned::define_id;

define_id!("apps", IAppID, AppID, i64);

#[test]
fn external_to_internal_and_back() {
    let external = AppID::from(7_i64);
    let internal = IAppID::from(external.clone());

    assert_eq!(internal.table.to_string(), "apps");
    assert_eq!(internal.try_into_external().unwrap(), external);
    assert_eq!(i64::from(external), 7);
}

#[test]
fn serde_is_transparent() {
    let external = AppID::from(7_i64);
    assert_eq!(serde_json::to_string(&external).unwrap(), "7");
}

/// Only compiles with the `salvo` feature, which proves the derive is gated.
#[cfg(feature = "salvo")]
#[test]
fn external_has_a_schema() {
    fn needs_schema<T: salvo::oapi::ToSchema>() {}
    needs_schema::<AppID>();
}
