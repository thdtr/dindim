//! Plain CSV export and import. The format is the same in every language:
//! ISO dates, type codes in English and amounts with a dot as the decimal
//! separator, so files move between computers without surprises.

use std::collections::HashMap;
use std::io::{Read, Write};

use dindim_core::{Category, CategoryId, Currency, Date, Entry, Kind, Money, NumberStyle, amount};

use crate::error::CsvProblem;
use crate::{Result, StoreError};

pub const HEADER: [&str; 5] = ["date", "type", "amount", "category", "note"];

/// One parsed line of a CSV file, not yet saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvRow {
    pub date: Date,
    pub kind: Kind,
    pub amount: Money,
    pub category: Option<String>,
    pub note: String,
}

pub fn write<W: Write>(
    out: W,
    entries: &[Entry],
    categories: &[Category],
    currency: Currency,
) -> Result<()> {
    let names: HashMap<CategoryId, &str> =
        categories.iter().map(|c| (c.id, c.name.as_str())).collect();
    let mut writer = ::csv::Writer::from_writer(out);
    writer.write_record(HEADER).map_err(csv_io)?;
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by_key(|e| (e.date, e.id));
    for e in sorted {
        let category = e
            .category_id
            .and_then(|id| names.get(&id).copied())
            .unwrap_or("");
        writer
            .write_record([
                e.date.to_string().as_str(),
                e.kind.code(),
                &plain_amount(e.amount, currency),
                category,
                &e.note,
            ])
            .map_err(csv_io)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn read<R: Read>(mut input: R, currency: Currency) -> Result<Vec<CsvRow>> {
    let mut text = Vec::new();
    input.read_to_end(&mut text)?;
    let line_at = |byte: u64| {
        let mut end = usize::try_from(byte).unwrap_or(usize::MAX).min(text.len());
        while text.get(end).is_some_and(|b| *b == b'\n' || *b == b'\r') {
            end += 1;
        }
        text[..end].iter().filter(|b| **b == b'\n').count() as u64 + 1
    };
    let mut reader = ::csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(::csv::Trim::All)
        .from_reader(text.as_slice());
    let mut rows = Vec::new();
    for (index, record) in reader.records().enumerate() {
        let record = record.map_err(|e| StoreError::InvalidCsv {
            line: e.position().map_or(index as u64 + 1, |p| line_at(p.byte())),
            problem: CsvProblem::Columns,
        })?;
        let line = record
            .position()
            .map_or(index as u64 + 1, |p| line_at(p.byte()));
        let fail = |problem| StoreError::InvalidCsv { line, problem };
        if index == 0 {
            let header: Vec<String> = record.iter().map(str::to_ascii_lowercase).collect();
            if header != HEADER {
                return Err(fail(CsvProblem::Header));
            }
            continue;
        }
        if record.iter().all(str::is_empty) {
            continue;
        }
        if record.len() != HEADER.len() {
            return Err(fail(CsvProblem::Columns));
        }
        let date = record[0]
            .parse::<Date>()
            .map_err(|_| fail(CsvProblem::Date))?;
        let kind = Kind::from_code(&record[1]).ok_or_else(|| fail(CsvProblem::Type))?;
        let amount = amount::parse(&record[2], currency, NumberStyle::POINT)
            .ok()
            .filter(|m| m.is_positive())
            .ok_or_else(|| fail(CsvProblem::Amount))?;
        let category = Some(record[3].to_string()).filter(|c| !c.is_empty());
        if category
            .as_deref()
            .is_some_and(|c| crate::ledger::clean_name(c).is_err())
        {
            return Err(fail(CsvProblem::Category));
        }
        rows.push(CsvRow {
            date,
            kind,
            amount,
            category: category.filter(|_| !kind.is_reserve()),
            note: record[4].to_string(),
        });
    }
    Ok(rows)
}

fn plain_amount(money: Money, currency: Currency) -> String {
    let scale = currency.scale();
    let major = money.minor() / scale;
    if currency.minor_units == 0 {
        major.to_string()
    } else {
        let width = usize::from(currency.minor_units);
        format!("{major}.{:0width$}", money.minor() % scale)
    }
}

fn csv_io(e: ::csv::Error) -> StoreError {
    match e.into_kind() {
        ::csv::ErrorKind::Io(io) => StoreError::Io(io),
        other => StoreError::Io(std::io::Error::other(format!("{other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ledger;
    use dindim_core::{CategoryKind, NewEntry};
    use jiff::civil::date;

    #[test]
    fn export_then_import_round_trips() {
        let mut source = Ledger::in_memory(Currency::USD).unwrap();
        let rent = source.add_category("Rent", CategoryKind::Expense).unwrap();
        for (kind, minor, category, note) in [
            (Kind::Expense, 120_000, Some(rent.id), "November, \"late\""),
            (Kind::Income, 350_050, None, ""),
            (Kind::ReserveDeposit, 20_000, None, "rainy day"),
        ] {
            source
                .add_entry(&NewEntry {
                    date: date(2026, 11, 3),
                    kind,
                    amount: Money::from_minor(minor),
                    category_id: category,
                    note: note.into(),
                })
                .unwrap();
        }
        let mut file = Vec::new();
        write(
            &mut file,
            &source.entries().unwrap(),
            &source.categories().unwrap(),
            Currency::USD,
        )
        .unwrap();
        let text = String::from_utf8(file.clone()).unwrap();
        assert!(text.starts_with("date,type,amount,category,note\n"));
        assert!(text.contains("2026-11-03,expense,1200.00,Rent,\"November, \"\"late\"\"\""));

        let rows = read(file.as_slice(), Currency::USD).unwrap();
        assert_eq!(rows.len(), 3);
        let mut target = Ledger::in_memory(Currency::USD).unwrap();
        let summary = target.import_rows(&rows).unwrap();
        assert_eq!(summary.entries, 3);
        assert_eq!(summary.new_categories, 1);
        assert_eq!(target.entries().unwrap().len(), 3);
        assert_eq!(target.categories().unwrap()[0].name, "Rent");
    }

    #[test]
    fn reuses_existing_categories_ignoring_case() {
        let mut ledger = Ledger::in_memory(Currency::USD).unwrap();
        ledger
            .add_category("Groceries", CategoryKind::Expense)
            .unwrap();
        let csv = "date,type,amount,category,note\n2026-01-02,expense,5,groceries,\n";
        let rows = read(csv.as_bytes(), Currency::USD).unwrap();
        assert_eq!(ledger.import_rows(&rows).unwrap().new_categories, 0);
    }

    #[test]
    fn reports_the_line_of_the_first_problem() {
        let line_of = |csv: &str| match read(csv.as_bytes(), Currency::USD) {
            Err(StoreError::InvalidCsv { line, problem }) => (line, problem),
            other => panic!("unexpected {other:?}"),
        };
        assert_eq!(line_of("when,what\n"), (1, CsvProblem::Header));
        assert_eq!(
            line_of("date,type,amount,category,note\n2026-13-01,expense,1,,\n"),
            (2, CsvProblem::Date)
        );
        assert_eq!(
            line_of("date,type,amount,category,note\n2026-01-01,gift,1,,\n"),
            (2, CsvProblem::Type)
        );
        assert_eq!(
            line_of("date,type,amount,category,note\n\n2026-01-01,expense,0,,\n"),
            (3, CsvProblem::Amount)
        );
        assert_eq!(
            line_of("date,type,amount,category,note\n2026-01-01,expense,1\n"),
            (2, CsvProblem::Columns)
        );
    }
}
