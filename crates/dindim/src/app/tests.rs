use std::time::{Duration, Instant};

use dindim_core::{CategoryKind, Date, Money, YearMonth};
use dindim_i18n::Locale;
use iced::widget::text_editor;

use super::*;
use crate::preferences::Preferences;

fn today() -> Date {
    Date::constant(2026, 11, 19)
}

fn config(dir: &std::path::Path) -> Config {
    Config {
        data_dir: dir.to_path_buf(),
        preferences_file: None,
        preferences: Preferences {
            language: Language::EnUs,
            ..Preferences::default()
        },
        logs_dir: None,
        today: today(),
        system: false,
    }
}

fn app() -> (tempfile::TempDir, App) {
    let dir = tempfile::tempdir().unwrap();
    let app = App::new(config(dir.path())).0;
    (dir, app)
}

fn reopen(dir: &tempfile::TempDir) -> App {
    App::new(config(dir.path())).0
}

fn send(app: &mut App, messages: impl IntoIterator<Item = Message>) {
    for m in messages {
        let _ = app.update(m);
    }
}

fn create(app: &mut App, name: &str, password: &str) {
    send(
        app,
        [
            Message::NewName(name.into()),
            Message::NewPassword(password.into()),
            Message::NewPasswordAgain(password.into()),
            Message::CreateAccount,
        ],
    );
}

fn add_category(app: &mut App, name: &str) -> CategoryId {
    send(
        app,
        [
            Message::ManagerKind(CategoryKind::Expense),
            Message::ManagerSearch(name.into()),
            Message::CreateCategory,
        ],
    );
    app.active()
        .unwrap()
        .categories
        .iter()
        .find(|c| c.name == name)
        .unwrap()
        .id
}

fn type_note(text: &str) -> Message {
    Message::Note(text_editor::Action::Edit(text_editor::Edit::Paste(
        std::sync::Arc::new(text.into()),
    )))
}

#[test]
fn new_accounts_start_empty_and_keep_their_own_categories() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    assert_eq!(app.screen, Screen::Account("joe".into()));
    assert!(app.active().unwrap().categories.is_empty());
    add_category(&mut app, "Rent");

    create(&mut app, "Ann", "");
    assert_eq!(app.screen, Screen::Account("ann".into()));
    assert!(app.active().unwrap().categories.is_empty());
}

#[test]
fn keyboard_flow_adds_an_expense() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    add_category(&mut app, "Groceries");
    send(
        &mut app,
        [
            Message::SetTab(Tab::Entries),
            Message::Amount("45.90".into()),
            Message::CategoryQuery("groc".into()),
            Message::FocusNext(true),
            type_note("milk"),
            Message::SubmitEntry,
        ],
    );
    let acc = app.active().unwrap();
    assert_eq!(acc.entries.len(), 1);
    assert_eq!(acc.entries[0].amount, Money::from_minor(4590));
    assert_eq!(
        acc.category_name(acc.entries[0].category_id),
        Some("Groceries")
    );
    assert_eq!(acc.entries[0].note, "milk");
    assert!(
        acc.form.amount.is_empty(),
        "form is cleared for the next one"
    );
}

#[test]
fn the_category_menu_creates_a_category_on_the_spot() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    send(
        &mut app,
        [
            Message::Amount("12".into()),
            Message::CategoryQuery("Coffee".into()),
        ],
    );
    assert_eq!(app.active().unwrap().category_menu, Some(0));
    assert_eq!(app.active().unwrap().menu_len(), 1, "only the create row");
    send(&mut app, [Message::CategorySubmit, Message::SubmitEntry]);
    let acc = app.active().unwrap();
    assert_eq!(acc.categories.len(), 1);
    assert_eq!(
        acc.category_name(acc.entries[0].category_id),
        Some("Coffee")
    );
}

#[test]
fn arrow_keys_move_through_the_menu() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    let gas = add_category(&mut app, "Gas");
    add_category(&mut app, "Groceries");
    send(
        &mut app,
        [
            Message::SetTab(Tab::Entries),
            Message::CategoryQuery("g".into()),
            Message::MenuMove(1),
            Message::MenuMove(1),
            Message::MenuMove(1),
        ],
    );
    let acc = app.active().unwrap();
    assert_eq!(acc.menu_len(), 3, "two matches and the create row");
    assert_eq!(acc.category_menu, Some(0), "wraps around");
    send(&mut app, [Message::CategorySubmit]);
    assert_eq!(app.active().unwrap().form.category, Some(gas));
    assert_eq!(app.active().unwrap().category_menu, None);
}

#[test]
fn validation_errors_keep_the_form() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    send(
        &mut app,
        [Message::Amount("abc".into()), Message::SubmitEntry],
    );
    let acc = app.active().unwrap();
    assert!(acc.entries.is_empty());
    assert!(acc.form.error.is_some());
    assert_eq!(acc.form.amount, "abc");
}

#[test]
fn saving_jumps_to_the_month_of_the_entry() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    send(
        &mut app,
        [
            Message::Amount("10".into()),
            Message::EntryDate(Date::constant(2026, 3, 5)),
            Message::SubmitEntry,
        ],
    );
    assert_eq!(app.month, YearMonth::new(2026, 3).unwrap());
}

#[test]
fn edits_and_deletes_with_confirmation_then_undoes() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    send(
        &mut app,
        [Message::Amount("10".into()), Message::SubmitEntry],
    );
    let id = app.active().unwrap().entries[0].id;

    send(
        &mut app,
        [
            Message::EditEntry(id),
            Message::Amount("12.50".into()),
            Message::SubmitEntry,
        ],
    );
    assert_eq!(
        app.active().unwrap().entries[0].amount,
        Money::from_minor(1250)
    );

    send(&mut app, [Message::EditEntry(id), Message::DeleteEntry]);
    assert_eq!(app.active().unwrap().entries.len(), 1, "first click asks");
    send(&mut app, [Message::DeleteEntry]);
    assert!(app.active().unwrap().entries.is_empty());

    send(&mut app, [Message::Undo]);
    assert_eq!(
        app.active().unwrap().entries[0].amount,
        Money::from_minor(1250)
    );
    send(&mut app, [Message::Undo]);
    assert_eq!(
        app.active().unwrap().entries[0].amount,
        Money::from_minor(1000)
    );
    send(&mut app, [Message::Redo, Message::Redo]);
    assert!(app.active().unwrap().entries.is_empty());
}

#[test]
fn search_finds_entries_in_any_month() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    send(
        &mut app,
        [
            Message::Amount("30".into()),
            Message::EntryDate(Date::constant(2025, 1, 10)),
            type_note("Birthday gift"),
            Message::SubmitEntry,
            Message::GoToMonth(YearMonth::of(today())),
            Message::Search("gift".into()),
        ],
    );
    let acc = app.active().unwrap();
    let query = dindim_core::search::EntryQuery::new(&acc.search, None);
    assert!(acc.entries.iter().any(|e| query.matches(e, None)));
}

#[test]
fn protected_accounts_need_their_password() {
    let (dir, mut app) = app();
    create(&mut app, "Joe", "1234");
    send(&mut app, [Message::CloseAccount]);
    assert_eq!(app.screen, Screen::Welcome);

    let mut app = reopen(&dir);
    assert_eq!(
        app.screen,
        Screen::Welcome,
        "a protected account never opens alone"
    );
    send(&mut app, [Message::SelectAccount("joe".into())]);
    assert_eq!(app.screen, Screen::Unlock("joe".into()));

    let vault = app.vault.clone().unwrap();
    let wrong = Handoff::new(vault.unlock("joe", Some("nope")));
    send(&mut app, [Message::Unlocked("joe".into(), wrong)]);
    assert!(app.unlock.error.is_some());
    assert_eq!(app.screen, Screen::Unlock("joe".into()));

    let right = Handoff::new(vault.unlock("joe", Some("1234")));
    send(&mut app, [Message::Unlocked("joe".into(), right)]);
    assert_eq!(app.screen, Screen::Account("joe".into()));
}

#[test]
fn reopens_the_last_account_without_a_password() {
    let (dir, mut app) = app();
    create(&mut app, "Joe", "");
    create(&mut app, "Ann", "");
    let mut prefs = app.prefs.clone();
    prefs.last_account = Some("joe".into());
    let app = App::new(Config {
        preferences: prefs,
        ..config(dir.path())
    })
    .0;
    assert_eq!(app.screen, Screen::Account("joe".into()));
}

#[test]
fn several_accounts_stay_open() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    create(&mut app, "Ann", "");
    send(
        &mut app,
        [Message::Amount("7".into()), Message::SubmitEntry],
    );
    send(&mut app, [Message::SelectAccount("joe".into())]);
    assert!(app.active().unwrap().entries.is_empty());
    send(&mut app, [Message::SelectAccount("ann".into())]);
    assert_eq!(app.active().unwrap().entries.len(), 1);
    assert_eq!(app.open.len(), 2);
}

#[test]
fn mismatched_passwords_are_rejected() {
    let (_d, mut app) = app();
    send(
        &mut app,
        [
            Message::NewName("X".into()),
            Message::NewPassword("a".into()),
            Message::NewPasswordAgain("b".into()),
            Message::CreateAccount,
        ],
    );
    assert!(app.new_account.error.is_some());
    assert_eq!(app.new_account.name, "X", "the form keeps what was typed");
    assert!(app.accounts.is_empty());
}

#[test]
fn account_currency_follows_the_choice() {
    let (_d, mut app) = app();
    let brl = Currency::from_code("BRL").unwrap();
    send(&mut app, [Message::NewCurrency(brl)]);
    create(&mut app, "Joe", "");
    assert_eq!(app.active().unwrap().currency, brl);
}

#[test]
fn settings_manage_password_categories_and_opening_balance() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    let pets = add_category(&mut app, "Pets");
    send(
        &mut app,
        [
            Message::SettingsPassword("abc".into()),
            Message::SettingsPasswordAgain("abc".into()),
            Message::SavePassword,
            Message::OpeningAmount("1,000".into()),
            Message::OpeningDate(Date::constant(2026, 1, 1)),
            Message::SaveOpening,
        ],
    );
    let acc = app.active().unwrap();
    assert!(acc.info.protected);
    assert!(app.accounts[0].protected);
    assert_eq!(acc.opening.unwrap().amount, Money::from_minor(100_000));

    send(
        &mut app,
        [
            Message::StartRename(pets),
            Message::RenameText("Animals".into()),
            Message::SaveRename,
        ],
    );
    assert_eq!(
        app.active().unwrap().category_name(Some(pets)),
        Some("Animals")
    );

    send(&mut app, [Message::AskDeleteCategory(pets)]);
    assert!(app.active().unwrap().category(Some(pets)).is_some());
    send(&mut app, [Message::ConfirmDeleteCategory]);
    assert!(app.active().unwrap().category(Some(pets)).is_none());
    send(&mut app, [Message::Undo]);
    assert_eq!(
        app.active().unwrap().category_name(Some(pets)),
        Some("Animals")
    );
}

#[test]
fn escape_closes_one_thing_at_a_time() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    send(
        &mut app,
        [
            Message::Amount("12".into()),
            Message::TogglePicker(Picker::EntryDate),
        ],
    );
    assert_eq!(app.picker, Some(Picker::EntryDate));
    send(&mut app, [Message::Escape]);
    assert_eq!(app.picker, None);
    assert_eq!(app.active().unwrap().form.amount, "12");
    send(&mut app, [Message::Escape]);
    assert!(app.active().unwrap().form.amount.is_empty());

    let day = Date::constant(2026, 11, 3);
    send(&mut app, [Message::EntryDate(day)]);
    assert_eq!(app.active().unwrap().form.date, Some(day));
    send(&mut app, [Message::EntryDate(today())]);
    assert_eq!(
        app.active().unwrap().form.date,
        None,
        "today is the default"
    );
}

#[test]
fn removing_an_account_needs_confirmation() {
    let (_d, mut app) = app();
    create(&mut app, "Joe", "");
    send(&mut app, [Message::RemoveAccount]);
    assert_eq!(app.accounts.len(), 1);
    send(&mut app, [Message::RemoveAccount]);
    assert!(app.accounts.is_empty());
    assert_eq!(app.screen, Screen::Welcome);
}

#[test]
fn idle_protected_accounts_lock_themselves() {
    let (_d, mut app) = app();
    create(&mut app, "Open", "");
    create(&mut app, "Locked", "pw");
    app.last_activity = Instant::now() - Duration::from_secs(60 * 60);
    app.lock_if_idle();
    assert_eq!(app.screen, Screen::Unlock("locked".into()));
    assert_eq!(app.open.len(), 1);
    assert_eq!(app.open[0].info.id, "open");
    assert!(app.toast.is_some());
}

#[test]
fn switching_language_translates_on_the_spot() {
    let (_d, mut app) = app();
    assert_eq!(app.i18n.locale(), Locale::EnUs);
    send(&mut app, [Message::SetLanguage(Language::PtBr)]);
    assert_eq!(app.i18n.locale(), Locale::PtBr);
    assert_eq!(app.prefs.language, Language::PtBr);
}

#[test]
fn zoom_and_theme_are_preferences() {
    let (_d, mut app) = app();
    send(&mut app, [Message::ZoomIn, Message::ZoomIn]);
    assert!((app.scale_factor() - 1.2).abs() < 0.001);
    send(&mut app, [Message::ZoomReset]);
    assert_eq!(app.scale_factor(), 1.0);
    send(&mut app, [Message::SetTheme(ThemeChoice::Dark)]);
    assert!(app.is_dark());
}

/// Every message id used in the code exists in the reference language.
#[test]
fn every_translation_key_in_the_code_exists() {
    let i18n = I18n::new(Locale::EnUs);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut missing = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            for pattern in ["t(\"", "t_with(\""] {
                for chunk in source.split(pattern).skip(1) {
                    let Some(key) = chunk.split('"').next() else {
                        continue;
                    };
                    let looks_like_key = !key.is_empty()
                        && key.contains('-')
                        && key.chars().all(|c| c.is_ascii_lowercase() || c == '-');
                    if looks_like_key && !i18n.has(key) {
                        missing.push(format!("{}: {key}", path.display()));
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "missing translations: {missing:#?}");
}
