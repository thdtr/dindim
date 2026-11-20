//! Domain model for dindim. Pure logic with no I/O and no user-facing text.

pub mod amount;
pub mod currency;
pub mod model;
pub mod money;
pub mod month;
pub mod report;
pub mod search;
pub mod text;

pub use amount::{AmountError, NumberStyle};
pub use currency::Currency;
pub use jiff::civil::Date;
pub use model::{
    Category, CategoryId, CategoryKind, Entry, EntryId, Kind, NewEntry, OpeningBalance,
};
pub use money::Money;
pub use month::YearMonth;
