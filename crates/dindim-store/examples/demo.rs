//! Creates a demo account from a CSV file, for screenshots and trying the app.
//!
//! ```text
//! cargo run -p dindim-store --example demo -- <data folder> <file.csv> <name> <currency> [today]
//! ```
//!
//! With `today` (YYYY-MM-DD), every date moves by the same number of days so
//! the newest entry falls on that day and the current month is never empty.

use std::path::PathBuf;

use dindim_core::{Currency, Date, OpeningBalance};
use dindim_store::Vault;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [root, csv, name, currency, rest @ ..] = args.as_slice() else {
        eprintln!("usage: demo <data folder> <file.csv> <name> <currency> [today]");
        std::process::exit(2);
    };
    let currency = Currency::from_code(currency).ok_or("unknown currency")?;
    let mut rows = dindim_store::csv::read(std::fs::File::open(csv)?, currency)?;

    if let Some(today) = rest.first() {
        let today: Date = today.parse()?;
        let newest = rows.iter().map(|r| r.date).max().ok_or("empty file")?;
        let shift = newest.until(today)?;
        for row in &mut rows {
            row.date = row.date.checked_add(shift)?;
        }
    }

    let (vault, _) = Vault::open(PathBuf::from(root))?;
    let (info, mut ledger) = vault.create_account(name, None, currency)?;
    let summary = ledger.import_rows(&rows)?;
    if let Some(first) = rows.iter().map(|r| r.date).min() {
        ledger.set_opening_balance(Some(OpeningBalance {
            date: first,
            amount: dindim_core::Money::from_minor(250_000),
        }))?;
    }
    println!(
        "{} ({}): {} entries, {} categories",
        info.name, info.id, summary.entries, summary.new_categories
    );
    Ok(())
}
