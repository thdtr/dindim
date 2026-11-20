## Dates and months

date-today = Hoje
date-yesterday = Ontem
date-long = { $day } de { $month } de { $year }
date-month-year = { $month } de { $year }
date-short = { $weekday }, { $day-padded }/{ $month-number }
date-short-year = { $day-padded }/{ $month-number }/{ $year }

month-name = { $month ->
    [1] janeiro
    [2] fevereiro
    [3] março
    [4] abril
    [5] maio
    [6] junho
    [7] julho
    [8] agosto
    [9] setembro
    [10] outubro
    [11] novembro
   *[12] dezembro
}

month-short = { $month ->
    [1] jan
    [2] fev
    [3] mar
    [4] abr
    [5] mai
    [6] jun
    [7] jul
    [8] ago
    [9] set
    [10] out
    [11] nov
   *[12] dez
}

weekday-short = { $day ->
    [0] dom
    [1] seg
    [2] ter
    [3] qua
    [4] qui
    [5] sex
   *[6] sáb
}

weekday-initial = { $day ->
    [0] D
    [1] S
    [2] T
    [3] Q
    [4] Q
    [5] S
   *[6] S
}

## Sidebar and navigation

rail-accounts = Contas
rail-new-account = Nova conta
rail-backup = Backup
rail-preferences = Preferências
rail-about = Sobre
rail-update-available = Saiu a versão { $version }

tab-entries = Lançamentos
tab-report = Relatório
tab-categories = Categorias
tab-settings = Ajustes

action-undo = Desfazer
action-redo = Refazer
action-this-month = Mês atual
action-cancel = Cancelar
action-delete = Apagar
action-save = Salvar
action-remove = Remover
action-rename = Renomear

## Transaction form

kind-expense = Despesa
kind-income = Receita
kind-reserve = Reserva
reserve-deposit = Guardar
reserve-withdrawal = Tirar

form-edit-title = Editar lançamento
form-amount = Valor
form-category = Categoria
form-category-placeholder = Escolha ou digite…
form-category-placeholder-empty = Digite um nome para criar
form-category-menu-empty = Nenhuma categoria ainda
form-create-category = Criar “{ $name }”
form-movement = Movimento
form-date = Data
form-other-day = Outro dia
form-note = Descrição
form-optional = opcional
form-note-placeholder = Pão, leite e café
form-save-changes = Salvar alterações
form-add-expense = Lançar despesa
form-add-income = Lançar receita
form-add-deposit = Guardar na reserva
form-add-withdrawal = Tirar da reserva
form-delete = Excluir
form-confirm-delete = Clique de novo para excluir
form-keyboard-hint = Enter lança · Tab passa de campo · Esc limpa

## Month summary and list

summary-in = Entrou
summary-out = Saiu
summary-left = Sobrou
summary-short = Faltou

list-title = Lançamentos de { $month }
list-search-results = Resultado da busca
list-count = { $count ->
    [0] { "" }
    [1] 1 lançamento
   *[other] { $count } lançamentos
}
list-search-placeholder = Buscar lançamentos
list-empty = Nada lançado em { $month }
list-empty-hint = As despesas e receitas do mês aparecem aqui, separadas por dia.
list-search-empty = Nada encontrado
list-search-empty-hint = Tente outra palavra ou o valor exato.
list-reserve-in = { $amount } guardados na reserva
list-reserve-out = { $amount } tirados da reserva

entry-expense-uncategorized = Despesa sem categoria
entry-income-uncategorized = Receita sem categoria
entry-reserve-deposit = Guardado na reserva
entry-reserve-withdrawal = Tirado da reserva

## Report

report-headline-empty = Nenhum lançamento em { $month } ainda.
report-headline-no-income = Em { $month } saíram { $amount } e não entrou nada.
report-headline-surplus = Sobraram { $amount } em { $month }.
report-headline-even = Em { $month }, tudo o que entrou foi gasto.
report-headline-deficit = Em { $month } saiu { $amount } a mais do que entrou.
report-ratio = Você gastou { $percent }% do que entrou.
report-change-more = Gastou { $amount } a mais que em { $month }.
report-change-less = Gastou { $amount } a menos que em { $month }.
report-change-same = Gastou o mesmo que em { $month }.
report-where-it-went = Para onde foi o dinheiro
report-where-it-came-from = De onde veio o dinheiro
report-uncategorized = Sem categoria
report-uncategorized-income = Sem categoria
report-reserve-title = Reserva de emergência
report-reserve-balance = A reserva tem { $amount }.
report-reserve-saved = Neste mês você guardou { $amount }.
report-reserve-taken = Neste mês você tirou { $amount }.
report-balance-title = Dinheiro disponível no fim do mês
report-balance-note = Saldo inicial mais tudo o que entrou, menos tudo o que saiu, até o último dia do mês.
report-balance-tip = Informe o saldo inicial em Ajustes para este número ficar certinho.
report-year-title = O ano de { $year }
report-year-hint = Clique num mês para ver o relatório dele.
report-year-in = { $amount } entraram no ano
report-year-out = { $amount } saíram no ano

## Categories tab

categories-expenses = Despesas  { $count }
categories-income = Receitas  { $count }
categories-delete-note = Apagar uma categoria não apaga os lançamentos. Eles só ficam sem categoria.
categories-search-expense = Buscar ou criar categoria de despesa
categories-search-income = Buscar ou criar categoria de receita
categories-create = Nova categoria
categories-create-named = Criar “{ $name }”
categories-column-name = Categoria
categories-column-entries = Lançamentos
categories-column-total = Total em { $year }
categories-empty = Nenhuma categoria ainda. Digite um nome acima e aperte Enter.
categories-no-match = Nenhuma categoria com “{ $name }”. Aperte Enter para criar.
categories-delete-question = Apagar “{ $name }”?
categories-delete-consequence = { $count ->
    [0] Nenhum lançamento usa esta categoria.
    [1] 1 lançamento vai ficar sem categoria.
   *[other] { $count } lançamentos vão ficar sem categoria.
}
categories-rename-placeholder = Nome da categoria

## Account settings

settings-name-title = Nome da conta
settings-name-description = Aparece na barra lateral e no título da janela.
settings-name-placeholder = Nome
settings-password-title = Senha
settings-password-description = Com senha, a conta só abre com ela e o arquivo fica criptografado. Se você esquecer a senha, não tem como recuperar os dados.
settings-password-on = Protegida com senha. O arquivo é criptografado.
settings-password-off = Sem senha. Qualquer pessoa que use este computador consegue abrir.
settings-password-new = Nova senha
settings-password-repeat = Repita a senha
settings-password-set = Colocar senha
settings-password-change = Trocar senha
settings-password-remove = Tirar a senha
settings-opening-title = Saldo inicial
settings-opening-description = Quanto dinheiro você tinha quando começou a anotar. O relatório usa esse valor para mostrar quanto você tem disponível.
settings-opening-date = Na data
settings-opening-save = Salvar saldo inicial
settings-currency-title = Moeda
settings-currency-description = Escolhida quando a conta foi criada. Todos os valores desta conta usam essa moeda.
settings-csv-title = Planilha
settings-csv-description = Exporte todos os lançamentos para um arquivo CSV ou traga lançamentos de um. Categorias que ainda não existem são criadas na hora.
settings-csv-export = Exportar CSV…
settings-csv-import = Importar CSV…
settings-close-title = Fechar ou apagar
settings-close-description = Fechar tranca a conta de novo e, se ela tiver senha, a senha vai ser pedida na próxima vez. Apagar tira a conta da lista e guarda uma cópia na sua pasta de dados.
settings-close = Fechar conta
settings-remove = Apagar conta
settings-remove-confirm = Clique de novo para apagar

## Start screen and unlock

welcome-tagline = Anote o que entra e o que sai. O dindim mostra, em palavras simples, para onde foi o seu dinheiro.
welcome-your-accounts = Suas contas
welcome-needs-password = pede senha
welcome-new-account = Nova conta
welcome-new-account-hint = Crie uma para cada pessoa ou objetivo. Cada conta tem as próprias categorias.
welcome-name = Nome
welcome-name-placeholder = Pessoal
welcome-currency = Moeda
welcome-password = Senha (opcional)
welcome-password-placeholder = Sem senha
welcome-password-repeat = Repita a senha
welcome-password-hint = Com senha, o arquivo da conta fica criptografado. Sem senha, ela abre direto.
welcome-create = Criar conta

unlock-prompt = Esta conta tem senha. Digite para abrir.
unlock-placeholder = Senha
unlock-open = Abrir
unlock-opening = Abrindo…

## Backup

backup-title = Backup
backup-intro = Junte todas as contas num único arquivo .zip. Guarde no pendrive, no e-mail ou na nuvem e depois é só importar no computador novo.
backup-export-title = Exportar backup
backup-export-description = Salva todas as contas, com ou sem senha, num arquivo .zip.
backup-export = Exportar…
backup-import-title = Importar backup
backup-import-description = Nada é apagado. Contas com nome repetido entram como “Nome (2)”. Contas com senha continuam pedindo a mesma senha.
backup-import = Importar…
backup-folder = Seus dados ficam em
backup-open-folder = Abrir pasta
backup-auto-on = O backup automático diário está ligado. Os { $count } mais recentes ficam guardados na pasta backups.
backup-auto-off = O backup automático está desligado. Dá para ligar em Preferências.

## Preferences

prefs-title = Preferências
prefs-language-title = Idioma
prefs-language-description = O idioma usado em todo o app.
prefs-language-system = Igual ao do sistema
prefs-theme-title = Aparência
prefs-theme-description = Clara, escura ou igual à do sistema.
prefs-theme-system = Igual à do sistema
prefs-theme-light = Clara
prefs-theme-dark = Escura
prefs-zoom-title = Zoom
prefs-zoom-description = Deixa tudo maior ou menor.
prefs-zoom-reset = Voltar ao normal
prefs-lock-title = Bloqueio automático
prefs-lock-description = Contas com senha fecham sozinhas depois desse tempo sem uso.
prefs-lock-never = Nunca
prefs-lock-minutes = Depois de { $minutes } minutos
prefs-backup-title = Backup automático
prefs-backup-description = Uma vez por dia, o dindim guarda uma cópia de todas as contas na sua pasta de dados.
prefs-backup-auto = Fazer backup todo dia
prefs-backup-keep = Guardar os { $count } mais recentes
prefs-updates-title = Atualizações
prefs-updates-description = Com isso ligado, o dindim pergunta ao GitHub se há uma versão nova toda vez que abre. Nada mais é enviado.
prefs-updates-toggle = Procurar versões novas
prefs-shortcuts-title = Atalhos de teclado
prefs-saved-note = As mudanças são salvas na hora.

shortcut-new-entry = Novo lançamento
shortcut-find = Buscar
shortcut-undo = Desfazer
shortcut-redo = Refazer
shortcut-tabs = Trocar de aba
shortcut-months = Mês anterior ou seguinte
shortcut-lock = Fechar a conta
shortcut-preferences = Preferências
shortcut-zoom = Aumentar ou diminuir o zoom
shortcut-escape = Fecha um painel, cancela a edição e limpa o formulário
shortcut-quit = Sair

## About

about-version = Versão { $version }
about-description = Controle de gastos rápido e privado. Seus dados não saem do seu computador.
about-source = Código-fonte
about-releases = Todas as versões
about-report-issue = Relatar um problema
about-license = Licença
about-open-logs = Abrir a pasta de logs
about-credits-title = Créditos
about-credits = Feito com Rust, iced e SQLCipher. As fontes são a Figtree e a Bricolage Grotesque, distribuídas sob a SIL Open Font License.
about-license-line = Software livre, distribuído sob a Licença Apache 2.0.

## Dialogs

dialog-already-running = O dindim já está aberto. Procure a janela dele na tela.
dialog-export-csv = Exportar lançamentos
dialog-import-csv = Importar lançamentos
dialog-csv-files = Arquivos CSV
dialog-export-backup = Salvar backup
dialog-import-backup = Abrir backup
dialog-backup-files = Backups do dindim

## Short confirmations

toast-undone = Desfeito
toast-redone = Refeito
toast-nothing-to-undo = Nada para desfazer
toast-nothing-to-redo = Nada para refazer
toast-entry-deleted = Lançamento excluído. Dá para desfazer.
toast-csv-exported = Lançamentos exportados para { $path }
toast-locked-idle = As contas com senha foram trancadas depois de um tempo sem uso.

## Notices

notice-account-list-damaged = Não deu para ler a lista de contas, então ela foi guardada à parte como { $path }. Os arquivos das contas estão intactos e aparecem com o nome do arquivo. Dá para renomear em Ajustes.
notice-account-removed = Conta apagada. Uma cópia ficou guardada em { $path }
notice-backup-empty = Esse backup não tinha nenhuma conta.
notice-backup-imported = { $count ->
    [1] 1 conta importada ({ $names }).
   *[other] { $count } contas importadas ({ $names }).
}
notice-backup-saved = { $count ->
    [1] Backup com 1 conta salvo em { $path }
   *[other] Backup com { $count } contas salvo em { $path }
}
notice-category-created = Categoria “{ $name }” criada.
notice-category-renamed = “{ $old }” agora se chama “{ $new }”.
notice-category-deleted = { $count ->
    [0] Categoria “{ $name }” apagada.
    [1] Categoria “{ $name }” apagada. 1 lançamento ficou sem categoria.
   *[other] Categoria “{ $name }” apagada. { $count } lançamentos ficaram sem categoria.
}
notice-csv-imported = { $entries ->
    [0] Nenhum lançamento encontrado nesse arquivo.
    [1] 1 lançamento importado.
   *[other] { $entries } lançamentos importados.
}
notice-csv-new-categories = { $count ->
    [1] 1 categoria nova foi criada.
   *[other] { $count } categorias novas foram criadas.
}
notice-name-saved = Nome salvo.
notice-password-saved = Senha salva. O arquivo desta conta agora é criptografado.
notice-password-removed = Senha removida. A conta abre sem pedir senha.
notice-opening-saved = Saldo inicial salvo.
notice-opening-removed = Saldo inicial removido.

## Errors

error-amount-empty = Digite um valor.
error-amount-invalid = Valor inválido. Use só números, vírgula e ponto.
error-amount-decimals = Casas decimais demais para esta moeda.
error-amount-too-large = Valor alto demais.
error-amount-not-positive = O valor precisa ser maior que zero.
error-unknown-category = Não existe a categoria “{ $name }”. Escolha uma da lista ou crie uma nova.
error-wrong-password = Senha incorreta. Tente de novo.
error-account-not-found = Conta não encontrada.
error-entry-not-found = Esse lançamento não existe mais.
error-category-not-found = Essa categoria não existe mais.
error-name-taken = Esse nome já está em uso.
error-name-empty = Dê um nome.
error-name-too-long = O nome pode ter no máximo { $max } caracteres.
error-category-kind = Essa categoria não combina com o tipo do lançamento.
error-category-name-needed = Digite o nome da categoria no campo de busca.
error-too-new = Este arquivo foi criado por uma versão mais nova do dindim. Atualize o app para abrir.
error-backup-not-backup = Este arquivo não é um backup do dindim.
error-backup-damaged = Este backup está corrompido.
error-backup-missing-account = Está faltando o arquivo de uma conta neste backup.
error-backup-not-database = Uma conta deste backup não é um arquivo válido do dindim.
error-backup-too-large = Uma conta deste backup é grande demais para importar.
error-csv-line = Problema na linha { $line }. { $reason }
error-csv-header = A primeira linha precisa ser date,type,amount,category,note.
error-csv-columns = Cada linha precisa ter cinco colunas.
error-csv-date = A data precisa estar no formato 2026-11-20.
error-csv-type = O tipo precisa ser expense, income, reserve_deposit ou reserve_withdrawal.
error-csv-amount = O valor precisa ser um número maior que zero, com ponto nos centavos, como 12.50.
error-csv-category = O nome da categoria é longo demais.
error-database = Algo deu errado ao salvar. Os detalhes estão no log.
error-io = Não foi possível ler ou gravar um arquivo. Confira se você tem permissão nessa pasta.
error-data-folder = O dindim não conseguiu abrir a pasta de dados. Os detalhes estão no log.
error-passwords-differ = As duas senhas estão diferentes.
error-password-empty = Digite a nova senha.
