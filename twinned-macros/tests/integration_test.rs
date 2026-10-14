use salvo::{oapi::__private::serde_json, prelude::ToSchema};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, RecordIdKey, SurrealValue};
use twinned_macros::dual_struct;

// Mock types to simulate the User's environment
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SurrealValue, ToSchema)]
#[surreal(crate = "surrealdb_types")]
#[surreal(transparent)]
pub struct ItemID(i64);
impl From<i64> for ItemID {
    fn from(v: i64) -> Self {
        Self(v)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SurrealValue, ToSchema)]
#[surreal(crate = "surrealdb_types")]
#[surreal(transparent)]
pub struct AppID(i64);
impl From<i64> for AppID {
    fn from(v: i64) -> Self {
        Self(v)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SurrealValue)]
#[surreal(crate = "surrealdb_types")]
pub struct IItemID(RecordId);
impl From<ItemID> for IItemID {
    fn from(id: ItemID) -> Self {
        Self(RecordId::new("workshop_items", id.0))
    }
}
impl From<IItemID> for ItemID {
    fn from(id: IItemID) -> Self {
        Self(number_key(&id.0))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SurrealValue)]
#[surreal(crate = "surrealdb_types")]
pub struct IAppID(RecordId);
impl From<AppID> for IAppID {
    fn from(id: AppID) -> Self {
        Self(RecordId::new("apps", id.0))
    }
}
impl From<IAppID> for AppID {
    fn from(id: IAppID) -> Self {
        Self(number_key(&id.0))
    }
}

fn number_key(id: &RecordId) -> i64 {
    match id.key {
        RecordIdKey::Number(key) => key,
        ref key => panic!("expected a number key, got {key:?}"),
    }
}

fn to_external_ids(internal: Vec<IItemID>) -> Result<Vec<ItemID>, surrealdb_types::Error> {
    Ok(internal.into_iter().map(ItemID::from).collect())
}

fn to_internal_ids(external: Vec<ItemID>) -> Vec<IItemID> {
    external.into_iter().map(IItemID::from).collect()
}

/// This is a doc comment for ExampleItem
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug, PartialEq))]
struct ExampleItem {
    /// The item's ID
    #[dual_type(IItemID)]
    pub id: ItemID,
    #[dual_type(IAppID)]
    pub appid: AppID, // Inferred external as AppID, internal as IAppID

    #[dual_type(Vec<IItemID>, to_external = to_external_ids, to_internal = to_internal_ids)]
    pub related_items: Vec<ItemID>,

    // Content information
    pub title: String,       // The titles name
    pub description: String, // HTML encoded description from steam
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,

    #[dual_type(Vec<IItemID>, to_external = to_external_ids, to_internal = to_internal_ids, surreal(wrap))]
    #[serde(default)]
    pub wrapped_items: Vec<ItemID>,
}

#[test]
fn test_surreal_attr() {
    // This test primarily checks that it compiles with the surreal(wrap)
    // attribute We can also check if the attribute is present via some
    // reflection or just trust compilation for now
    let _item = InternalExampleItem {
        id: IItemID(RecordId::new("workshop_items", 1)),
        appid: IAppID(RecordId::new("apps", 1)),
        related_items: vec![],
        title: "title".to_string(),
        description: "desc".to_string(),
        preview_url: None,
        wrapped_items: vec![],
    };
}

#[test]
fn test_dual_struct_generation() {
    let external = ExternalExampleItem {
        id: ItemID(123),
        appid: AppID(456),
        related_items: vec![ItemID(789), ItemID(101)],
        title: "Test Title".to_string(),
        description: "Test Description".to_string(),
        preview_url: None,
        wrapped_items: vec![],
    };

    let internal = InternalExampleItem::from(external.clone());
    assert_eq!(internal.id, IItemID::from(ItemID(123)));
    assert_eq!(internal.appid, IAppID::from(AppID(456)));
    assert_eq!(internal.related_items.len(), 2);
    assert_eq!(internal.preview_url, None);
    assert_eq!(internal.title, "Test Title");

    let external_back: ExternalExampleItem = internal.try_into().expect("Conversion failed");
    assert_eq!(external_back.id, ItemID(123));
    assert_eq!(external_back.appid, AppID(456));
    assert_eq!(external_back.related_items, vec![ItemID(789), ItemID(101)]);
    assert_eq!(external_back.title, "Test Title");
}

#[test]
fn test_serde_round_trip() {
    let external = ExternalExampleItem {
        id: ItemID(123),
        appid: AppID(456),
        related_items: vec![ItemID(789)],
        title: "Test Title".to_string(),
        description: "Test Description".to_string(),
        preview_url: None,
        wrapped_items: vec![],
    };

    // serde_json does not write a struct name, so the `#[serde(rename)]` that
    // the macro adds cannot show here. This test checks the field shape only.
    let json = serde_json::to_value(&external).unwrap();
    assert_eq!(json["id"], 123);
    assert_eq!(json["appid"], 456);
    assert_eq!(json["related_items"], serde_json::json!([789]));

    let back: ExternalExampleItem = serde_json::from_value(json).unwrap();
    assert_eq!(back.id, ItemID(123));
    assert_eq!(back.title, "Test Title");
}
