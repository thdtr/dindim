## Dates and months

date-today = Today
date-yesterday = Yesterday
date-long = { $month } { $day }, { $year }
date-month-year = { $month } { $year }
date-short = { $weekday }, { $month } { $day }
date-short-year = { $month } { $day }, { $year }

month-name = { $month ->
    [1] January
    [2] February
    [3] March
    [4] April
    [5] May
    [6] June
    [7] July
    [8] August
    [9] September
    [10] October
    [11] November
   *[12] December
}

month-short = { $month ->
    [1] Jan
    [2] Feb
    [3] Mar
    [4] Apr
    [5] May
    [6] Jun
    [7] Jul
    [8] Aug
    [9] Sep
    [10] Oct
    [11] Nov
   *[12] Dec
}

weekday-short = { $day ->
    [0] Sun
    [1] Mon
    [2] Tue
    [3] Wed
    [4] Thu
    [5] Fri
   *[6] Sat
}

weekday-initial = { $day ->
    [0] S
    [1] M
    [2] T
    [3] W
    [4] T
    [5] F
   *[6] S
}

## Sidebar and navigation

rail-accounts = Accounts
rail-new-account = New account
rail-backup = Backup
rail-preferences = Preferences
rail-about = About
rail-update-available = Version { $version } is out

tab-entries = Transactions
tab-report = Report
tab-categories = Categories
tab-settings = Settings

action-undo = Undo
action-redo = Redo
action-this-month = This month
action-cancel = Cancel
action-delete = Delete
action-save = Save
action-remove = Remove
action-rename = Rename

## Transaction form

kind-expense = Expense
kind-income = Income
kind-reserve = Savings
reserve-deposit = Put in
reserve-withdrawal = Take out

form-edit-title = Edit transaction
form-amount = Amount
form-category = Category
form-category-placeholder = Pick or type…
form-category-placeholder-empty = Type a name to create one
form-category-menu-empty = No categories yet
form-create-category = Create “{ $name }”
form-movement = Direction
form-date = Date
form-other-day = Pick a date
form-note = Note
form-optional = optional
form-note-placeholder = Milk, bread and coffee
form-save-changes = Save changes
form-add-expense = Add expense
form-add-income = Add income
form-add-deposit = Add to savings
form-add-withdrawal = Take from savings
form-delete = Delete
form-confirm-delete = Click again to delete
form-keyboard-hint = Enter saves · Tab moves on · Esc clears

## Month summary and list

summary-in = Money in
summary-out = Money out
summary-left = Left over
summary-short = Overspent

list-title = { $month } transactions
list-search-results = Search results
list-count = { $count ->
    [0] { "" }
    [one] 1 transaction
   *[other] { $count } transactions
}
list-search-placeholder = Search transactions
list-empty = Nothing in { $month } yet
list-empty-hint = Expenses and income for the month show up here, grouped by day.
list-search-empty = No matches
list-search-empty-hint = Try another word or the exact amount.
list-reserve-in = { $amount } added to savings
list-reserve-out = { $amount } taken from savings

entry-expense-uncategorized = Uncategorized expense
entry-income-uncategorized = Uncategorized income
entry-reserve-deposit = Added to savings
entry-reserve-withdrawal = Taken from savings

## Report

report-headline-empty = No transactions in { $month } yet.
report-headline-no-income = You spent { $amount } in { $month } and nothing came in.
report-headline-surplus = You had { $amount } left over in { $month }.
report-headline-even = Everything that came in during { $month } went right back out.
report-headline-deficit = You spent { $amount } more than you made in { $month }.
report-ratio = You spent { $percent }% of what came in.
report-change-more = That’s { $amount } more than in { $month }.
report-change-less = That’s { $amount } less than in { $month }.
report-change-same = That’s the same as in { $month }.
report-where-it-went = Where the money went
report-where-it-came-from = Where the money came from
report-uncategorized = Uncategorized
report-uncategorized-income = Uncategorized
report-reserve-title = Emergency savings
report-reserve-balance = You have { $amount } set aside.
report-reserve-saved = You put in { $amount } this month.
report-reserve-taken = You took out { $amount } this month.
report-balance-title = Cash at the end of the month
report-balance-note = Your starting balance plus everything that came in, minus everything that went out, through the last day of the month.
report-balance-tip = Add your starting balance in Settings so this number is right.
report-year-title = { $year } at a glance
report-year-hint = Click a month to see its report.
report-year-in = { $amount } in this year
report-year-out = { $amount } out this year

## Categories tab

categories-expenses = Expenses  { $count }
categories-income = Income  { $count }
categories-delete-note = Deleting a category keeps its transactions. They just lose the label.
categories-search-expense = Find or create an expense category
categories-search-income = Find or create an income category
categories-create = New category
categories-create-named = Create “{ $name }”
categories-column-name = Category
categories-column-entries = Transactions
categories-column-total = Total in { $year }
categories-empty = No categories yet. Type a name above and press Enter.
categories-no-match = Nothing matches “{ $name }”. Press Enter to create it.
categories-delete-question = Delete “{ $name }”?
categories-delete-consequence = { $count ->
    [0] No transactions use this category.
    [one] 1 transaction will lose its category.
   *[other] { $count } transactions will lose their category.
}
categories-rename-placeholder = Category name

## Account settings

settings-name-title = Account name
settings-name-description = Shown in the sidebar and in the window title.
settings-name-placeholder = Name
settings-password-title = Password
settings-password-description = With a password, the account only opens with it and the file on disk is encrypted. If you forget it, there is no way to get the data back.
settings-password-on = Protected with a password. The file is encrypted.
settings-password-off = No password. Anyone using this computer can open it.
settings-password-new = New password
settings-password-repeat = Type it again
settings-password-set = Set password
settings-password-change = Change password
settings-password-remove = Remove password
settings-opening-title = Starting balance
settings-opening-description = How much money you had when you started tracking. The report uses it to show how much cash you have.
settings-opening-date = As of
settings-opening-save = Save starting balance
settings-currency-title = Currency
settings-currency-description = Picked when the account was created. Every amount in this account uses it.
settings-csv-title = Spreadsheets
settings-csv-description = Export every transaction to a CSV file or bring transactions in from one. Categories that don’t exist yet are created along the way.
settings-csv-export = Export CSV…
settings-csv-import = Import CSV…
settings-close-title = Close or delete
settings-close-description = Closing locks the account again, so it asks for the password next time if it has one. Deleting takes it off the list and keeps a copy in your data folder.
settings-close = Close account
settings-remove = Delete account
settings-remove-confirm = Click again to delete

## Start screen and unlock

welcome-tagline = Jot down what comes in and what goes out. dindim tells you where your money went, in plain words.
welcome-your-accounts = Your accounts
welcome-needs-password = Password
welcome-new-account = New account
welcome-new-account-hint = Make one for each person or goal. Every account has its own categories.
welcome-name = Name
welcome-name-placeholder = Personal
welcome-currency = Currency
welcome-password = Password (optional)
welcome-password-placeholder = No password
welcome-password-repeat = Type it again
welcome-password-hint = With a password, the account file is encrypted. Without one, it opens right away.
welcome-create = Create account

unlock-prompt = This account has a password.
unlock-placeholder = Password
unlock-open = Open
unlock-opening = Opening…

## Backup

backup-title = Backup
backup-intro = Put every account in one .zip file. Keep it on a flash drive, in your email or in the cloud, then import it on your new computer.
backup-export-title = Export a backup
backup-export-description = Saves every account, with or without a password, to one .zip file.
backup-export = Export…
backup-import-title = Import a backup
backup-import-description = Nothing gets overwritten. An account whose name is taken comes in as “Name (2)”. Accounts with a password still ask for the same one.
backup-import = Import…
backup-folder = Your data is in
backup-open-folder = Open folder
backup-auto-on = Daily automatic backups are on. The latest { $count } are kept in the backups folder.
backup-auto-off = Automatic backups are off. You can turn them on in Preferences.

## Preferences

prefs-title = Preferences
prefs-language-title = Language
prefs-language-description = The language used throughout the app.
prefs-language-system = Same as my system
prefs-theme-title = Appearance
prefs-theme-description = Light, dark, or whatever your system is set to.
prefs-theme-system = Match my system
prefs-theme-light = Light
prefs-theme-dark = Dark
prefs-zoom-title = Zoom
prefs-zoom-description = Makes everything bigger or smaller.
prefs-zoom-reset = Reset
prefs-lock-title = Auto lock
prefs-lock-description = Accounts with a password close on their own after this long without use.
prefs-lock-never = Never
prefs-lock-minutes = After { $minutes } minutes
prefs-backup-title = Automatic backups
prefs-backup-description = Once a day, dindim saves a copy of every account in your data folder.
prefs-backup-auto = Back up every day
prefs-backup-keep = Keep the latest { $count }
prefs-updates-title = Updates
prefs-updates-description = When this is on, dindim asks GitHub for a newer version each time it starts. Nothing else is sent.
prefs-updates-toggle = Check for new versions
prefs-shortcuts-title = Keyboard shortcuts
prefs-saved-note = Changes are saved right away.

shortcut-new-entry = New transaction
shortcut-find = Search
shortcut-undo = Undo
shortcut-redo = Redo
shortcut-tabs = Switch tabs
shortcut-months = Previous or next month
shortcut-lock = Close the account
shortcut-preferences = Preferences
shortcut-zoom = Zoom in or out
shortcut-escape = Close a panel, cancel an edit, clear the form
shortcut-quit = Quit

## About

about-version = Version { $version }
about-description = A fast, private expense tracker. Your data never leaves your computer.
about-source = Source code
about-releases = All releases
about-report-issue = Report a problem
about-license = License
about-open-logs = Open the log folder
about-credits-title = Credits
about-credits = Built with Rust, iced and SQLCipher. The typefaces are Figtree and Bricolage Grotesque, released under the SIL Open Font License.
about-license-line = Free software, released under the Apache License 2.0.

## Dialogs

dialog-already-running = dindim is already open. Look for its window on your screen.
dialog-export-csv = Export transactions
dialog-import-csv = Import transactions
dialog-csv-files = CSV files
dialog-export-backup = Save backup
dialog-import-backup = Open backup
dialog-backup-files = dindim backups

## Short confirmations

toast-undone = Undone
toast-redone = Redone
toast-nothing-to-undo = Nothing to undo
toast-nothing-to-redo = Nothing to redo
toast-entry-deleted = Transaction deleted. You can undo this.
toast-csv-exported = Transactions exported to { $path }
toast-locked-idle = Accounts with a password were locked after a while without use.

## Notices

notice-account-list-damaged = The account list couldn’t be read, so it was set aside as { $path }. Your account files are fine and show up under their file names. You can rename them in Settings.
notice-account-removed = Account deleted. A copy was kept at { $path }
notice-backup-empty = That backup didn’t have any accounts in it.
notice-backup-imported = { $count ->
    [one] Imported 1 account ({ $names }).
   *[other] Imported { $count } accounts ({ $names }).
}
notice-backup-saved = { $count ->
    [one] Backup of 1 account saved to { $path }
   *[other] Backup of { $count } accounts saved to { $path }
}
notice-category-created = Category “{ $name }” created.
notice-category-renamed = “{ $old }” is now called “{ $new }”.
notice-category-deleted = { $count ->
    [0] Category “{ $name }” deleted.
    [one] Category “{ $name }” deleted. 1 transaction no longer has a category.
   *[other] Category “{ $name }” deleted. { $count } transactions no longer have a category.
}
notice-csv-imported = { $entries ->
    [0] No transactions found in that file.
    [one] Imported 1 transaction.
   *[other] Imported { $entries } transactions.
}
notice-csv-new-categories = { $count ->
    [one] 1 new category was created.
   *[other] { $count } new categories were created.
}
notice-name-saved = Name saved.
notice-password-saved = Password saved. This account’s file is now encrypted.
notice-password-removed = Password removed. The account now opens without asking.
notice-opening-saved = Starting balance saved.
notice-opening-removed = Starting balance removed.

## Errors

error-amount-empty = Enter an amount.
error-amount-invalid = That doesn’t look like an amount. Use only digits, commas and periods.
error-amount-decimals = Too many decimal places for this currency.
error-amount-too-large = That amount is too large.
error-amount-not-positive = The amount has to be more than zero.
error-unknown-category = There’s no category called “{ $name }”. Pick one from the list or create it.
error-wrong-password = Wrong password. Try again.
error-account-not-found = This account couldn’t be found.
error-entry-not-found = This transaction no longer exists.
error-category-not-found = This category no longer exists.
error-name-taken = That name is already taken.
error-name-empty = Give it a name.
error-name-too-long = Names can be up to { $max } characters long.
error-category-kind = That category doesn’t go with this type of transaction.
error-category-name-needed = Type the category name in the search box first.
error-too-new = This file was made by a newer version of dindim. Update the app to open it.
error-backup-not-backup = This file isn’t a dindim backup.
error-backup-damaged = This backup is damaged.
error-backup-missing-account = An account file is missing from this backup.
error-backup-not-database = An account in this backup isn’t a valid dindim file.
error-backup-too-large = An account in this backup is too big to import.
error-csv-line = Problem on line { $line }. { $reason }
error-csv-header = The first line has to be date,type,amount,category,note.
error-csv-columns = Every line needs five columns.
error-csv-date = Dates have to look like 2026-11-20.
error-csv-type = The type has to be expense, income, reserve_deposit or reserve_withdrawal.
error-csv-amount = The amount has to be a number above zero with a period for decimals, like 12.50.
error-csv-category = That category name is too long.
error-database = Something went wrong while saving. The log has the details.
error-io = A file couldn’t be read or saved. Check that you have permission to use that folder.
error-data-folder = dindim couldn’t open its data folder. The log has the details.
error-passwords-differ = The passwords don’t match.
error-password-empty = Type the new password.
