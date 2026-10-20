# 15 — O desfazer das operações de aba (W23 B5(4))

> Leitor: a LLM que implementa ou audita. Oráculo corrido em 19/10: tldraw 5.5.2, as PÁGINAS (o análogo das nossas
> abas de quadro) — [`ferramentas/tldraw_oracle/corre_paginas.mjs`](ferramentas/tldraw_oracle/corre_paginas.mjs),
> 23 fixtures `saidas/paginas_P*.json` com cabeçalho. Antes: criar, duplicar, renomear, arrastar e apagar um quadro
> não se desfaziam; apagar perguntava antes e deitava fora o desfazer do quadro. Commit `c5e1e7a612`.

## 1. Estado da arte — MEDIDO (tldraw, `paginas_P*`)

| caso | o tldraw |
|---|---|
| P1 criar | a corrente NÃO muda; undo tira, redo devolve |
| P2 renomear · P4 reordenar | um passo cada, a corrente não muda |
| P3 duplicar | «X Copy» logo a seguir, e fica corrente; undo volta à original |
| P5 apagar a corrente | volta com as formas, o id e a posição, e **corrente outra vez**; ao apagar, a corrente vai para a ANTERIOR |
| P6 apagar outra | volta, e a corrente não muda |
| P7/P8 cruzar páginas | UM histórico para o documento inteiro; o undo leva à página onde a mudança foi |
| P9 | qualquer mudança depois do undo mata o redo, mesmo noutra página; trocar de página não mata |
| P10 | trocar de página ENTRA no histórico |
| P11/P11b | a última página não se apaga; o gesto vazio fica como passo morto (`canUndo` acende para nada) |

## 2. Escolhas

- **A lei**: o `Ctrl+Z` de um documento (a cena, cada quadro) desfaz o **mais recente** entre o último passo DELE e a
  última operação da barra de abas. Os documentos continuam separados (plano §1.4: o `Ctrl+Z` num quadro nunca desfaz
  outro quadro nem a cena); as abas são a barra partilhada, com pilha própria. Do tldraw (um histórico só) fica a
  ORDEM entre as abas e o documento aberto.
- **O apagado volta inteiro** — conteúdo, id, posição E o desfazer dele (P5); aberto só se o estava (P5/P6).
- **Desfazer criar/duplicar/importar/restaurar versão** tira o quadro e volta à aba de antes.
- **O refazer de abas morre** quando um quadro ou a cena recebem um passo novo depois (P9) — refazer um «apagar» por
  cima de trabalho novo apagava-o.
- **Gesto sem efeito não é passo** (P11b): renomear para o mesmo nome, arrastar sem mudar de lugar.
- **Diferenças deliberadas do tldraw**: trocar de aba NÃO é passo (P10 — Miro/Figma/navegadores não o fazem, e com
  documentos separados seria um `Ctrl+Z` gasto); criar abre o quadro novo e apagar abre a da DIREITA (decisões da W0b,
  o idioma das abas do navegador); o refazer do PRÓPRIO quadro sobrevive a uma operação de abas (é do conteúdo dele).
- **A pergunta antes de apagar fica** (o Miro pergunta; o tldraw não) — com o desfazer, tirá-la é decisão do dono.

## 3. Desenho

- **Relógio** `ph2d_board_model::undo_tick()` (um `AtomicU64` que só cresce): cada passo de cada pilha leva a hora em
  que lá entrou — feito, desfeito ou refeito. `History::{undo_at, redo_at, last_push}`.
- **Pilha das abas** `ph2d-editor-core/src/documents_tabs.rs`: `TabStep::{Rename, Move, Remove{board, back},
  Restore{board, at, history, focus}}`; cada operação de `Documents` grava o inverso (`create` — também `import_board`
  —, `duplicate`, `restore_version`, `rename`, `move_tab`, `remove`). `undo_tab` aplica o topo, empurra o inverso com
  hora nova e salta um passo cujo quadro saiu por outra via. `BoardSet::restore` repõe o mesmo id.
- **Fora da pilha**: `adopt_board` (entrar numa sala ao vivo — tirá-la seria sair da sala), `replace` (abrir projecto:
  os ids recomeçam, `forget_tabs`).
- **Com um quadro aberto**: `board_keys::tabs_first` compara `tab_undo_at` com `History::undo_at` (no refazer, as horas
  de quando foram desfeitos); o texto a meio de se escrever é sempre do quadro. Entra pelas duas portas — a tecla
  (`board_keys::key`) e o Editar da shell (`board_command`).
- **Na cena** (shell): `ProjectUndo` leva a hora em cada degrau (`top_at`); `UndoOwner::Tabs` responde quando a
  operação de abas é mais recente, depois dos editores modais (Áudio, Painter, Colorize) e antes da fila global. A
  shell avisa `Documents::scene_stepped()` a cada degrau novo (o refazer de abas morre).
- `document_tabs_menu::undo_tab` acerta a barra: a aba que volta nasce clicável (`register_board`), a que sai sai.
- **Sessão ao vivo**: renomear o quadro partilhado e desfazê-lo viaja (o `Replica` compara o nome com a sombra);
  apagar o partilhado fecha a sala (`live_view.part`) e o desfazer devolve o quadro SEM a sala. O cronómetro e a
  votação a decorrer de um quadro apagado não voltam (são da sessão, `fac.forget`).

## 4. Gates (todos no código)

- editor (pelo menu, pelo arrasto e pelo `Ctrl+Z` real do quadro, `document_tabs_undo_tests.rs`):
  `ctrl_z_brings_a_deleted_board_back_whole_with_its_own_undo` · `undoing_the_delete_of_another_tab_keeps_the_open_one` ·
  `the_newer_of_the_board_step_and_the_tab_step_answers_ctrl_z` · `create_duplicate_and_drag_come_back_with_ctrl_z` ·
  `a_tab_redo_dies_when_the_board_changes_after_it` · `a_rename_to_the_same_name_or_a_nudge_is_not_a_step`
- documentos (`documents_tabs_tests.rs`): `loading_a_project_forgets_the_tab_steps` ·
  `a_board_that_arrived_from_a_live_room_is_not_a_step` · `a_step_whose_board_is_gone_is_spent_and_the_next_answers`
- shell: `on_the_scene_a_newer_tab_step_answers_before_the_global_queue` (`undo_route.rs`)
- app: `the_w23_tabs_scene_brings_the_archive_back_with_ctrl_z` (cena 27)
- `undoing_a_create_reopens_the_tab_it_was_made_from` (editor)

**Provas de mutação (19/10)**: 14 corridas, 12 sangraram à 1.ª; M11 (sem `register_board` na aba que volta — pintada mas
morta) e M13 (o `back` ignorado — a vizinha coincidia com a de antes no teste) sobreviviam: mortas pela asserção do
`button_state` e pelo teste do «+» a partir da 1.ª de três (`dec9bb6112`).

## 5. ⛔ Recusas MEDIDAS

| recusa | porquê |
|---|---|
| UM histórico para tudo (o tldraw) | quebra o plano §1.4: o `Ctrl+Z` num quadro desfaria outro quadro ou a cena |
| guardar a operação de aba no desfazer do quadro que ela toca | apagar deixa outro quadro aberto: o `Ctrl+Z` dele desfazia um traço seu em vez de devolver o apagado |
| trocar de aba como passo (tldraw P10) | os documentos são separados; seria um `Ctrl+Z` gasto a mudar de aba |
| comparar profundidades das pilhas em vez de horas | no tecto (500 passos) a profundidade não muda e um passo novo lê-se como velho |

## 6. Smoke — cena 27

`PH2D_BOARD_SMOKE=27`: três quadros (Ideas · Plan · Archive), o Plan aberto com as quatro lições. Fotos
`ferramentas/w23_fotos/cena27_desfazer_abas_{1366x768,1930x1040}.png`.
