# twinned

One struct definition, two generated types. The internal type talks to
[SurrealDB](https://surrealdb.com) and holds `RecordId` fields. The external
type goes over your API and holds plain keys. The macro also writes the
conversions between the two, so the two halves cannot drift apart.

## This code is AI slop

Claude (Anthropic) wrote every line of this crate, including this readme and
the tests. No human wrote any of it. No human has read it line by line. The
tests pass, which tells you the tests pass and nothing else.

Treat it the way you would treat a snippet from a forum post by a stranger:
read it, decide for yourself, and keep your own copy if you depend on it. Do
not expect support, a stable API, or a fix for your bug. If it breaks your
build, you get to keep both pieces.

## What it does

```rust
use twinned::{define_id, dual_struct};

define_id!("apps", IAppID, AppID, i64);

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct App {
    #[dual_type(IAppID)]
    pub id: AppID,
    pub title: String,
}
```

This gives you:

- `InternalApp { id: IAppID, title: String }`, with `SurrealValue` derived
- `ExternalApp { id: AppID, title: String }`
- `impl TryFrom<InternalApp> for ExternalApp`
- `impl From<ExternalApp> for InternalApp`

`define_id!` builds the ID pair itself: an external newtype over a plain key
type and an internal newtype over a `RecordId` that knows its table name.

Fields without `#[dual_type(..)]` are copied into both structs unchanged.

### Field options

`#[dual_type(Internal)]` uses the field's declared type as the external type.
`#[dual_type(Internal, External)]` sets both types.

Add `to_external = path::to::fn` or `to_internal = path::to::fn` when the
default `TryFrom`/`From` conversion is not correct. The `to_external` function
must return `Result<_, surrealdb_types::Error>`.

Add `surreal(..)` to put an attribute on the internal field only. The
`surrealdb-types` derive reads it, and the external struct never sees it:
`#[dual_type(Vec<IItemID>, surreal(wrap))]`.

## Dependencies you must have

The macros expand to paths that resolve in your crate, not in this one. Your
`Cargo.toml` needs:

- `serde` with the `derive` feature
- `surrealdb-types`
- `salvo` with the `oapi` feature, but only if you turn on the `salvo` feature

The module that calls `define_id!` must also have the `SurrealValue` trait in
scope:

```rust
use surrealdb_types::SurrealValue;
```

This is a real limitation, not a design choice, and a proper version of this
crate would re-export its own dependencies instead.

## Features

`salvo`, off by default. It adds `salvo::prelude::ToSchema` to every external
type and to every external ID, so the API half of each pair shows up in your
OpenAPI document. The feature is read inside `twinned`, so turning it on in
your `Cargo.toml` is enough:

```toml
twinned = { version = "0.1", features = ["salvo"] }
```

## Layout

`twinned` holds `define_id!` and re-exports the attribute macro. `twinned-macros`
holds the proc macro. Depend on `twinned`.

## Licence

MPL-2.0. See [LICENSE.md](LICENSE.md).
