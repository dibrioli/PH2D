# CONTINUAÇÃO — `line/MiroClone` depois dos acabamentos do §9 da W13 (2026-10-11)

> Para a próxima janela da linha. Rota `/pd-linha-assumir`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current` ANTES de ler.
> As regras R1–R9 da [continuação RÁPIDA](HANDOFF_CONTINUACAO_RAPIDA_FECHO_2026-10-11.md) valem iguais.

## Estado

- **W12 + W13 + acabamentos FECHADOS, por integrar** (não integre, não pushe): [handoff de integração](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md)
  — o §12 dele é a adenda desta janela (3 commits `3c8d8c0a5`, `8aaf833b3`, `9cd70e2f5`). Base = `main` `8f20c60de`.
  Se o integrador já levou a linha (`git cherry main HEAD | grep -c '^+'` = 0), `git reset --keep main` primeiro.
- Smoke da cena 14 e destes acabamentos: do dono, por fazer.

## O que esta janela fechou (não reconstruir)

| item do §9 | o que ficou | gate |
|---|---|---|
| o custo do selo | re-medido em `--release` com a placa: 1,34 ms de encode a 6 px (era 1,30), degrau na placa 0,15 ms — a carga sobe tudo por igual. Nenhum tecto (`05_facilitacao.md` §4.1) | — (régua `measure_marks_paint_cost`) |
| a coluna numa janela baixa | ROLA pela porta `widget::scroll_area`; a barra à esquerda dela; a roda sobre ela nunca dá zoom; listas até 1 fila; o `fit` conta a moldura | 4 em `board_bar_history_tests.rs` |
| fechar a aba do anfitrião | a aba apagada, OUTRO PROJECTO carregado (os ids recomeçam: a sala seguia o quadro novo) e a janela fechada fecham como «End session» | `live_tests::the_room_closes_like_end_session_when_its_board_leaves_the_host` |
| a identidade no `Hello` | presa a uma chave; impostor recusado antes do `Welcome`; protocolo 4; ADR-0182 §10 | 2 em `session_tests.rs` |

## A próxima tarefa: mudar o TIPO de uma forma seleccionada (o que fica do §9)

`Editor::set_shape_type` ([`ph2d-board-edit/src/lib.rs:580`](../../../crates/ph2d-board-edit/src/lib.rs)) existe com
**zero chamadores e zero testes** — a porta com a lei e sem ecrã. O Miro tem-no na barra de contexto de uma forma (o
botão com o contorno actual abre a grelha das formas). O plano:

1. **A lei primeiro (`ph2d-board-edit`, com gate):** `set_shape_type` NUNCA toca uma nota nem faz uma (`t.is_note()` →
   nada; filtra `!kind.is_note()`). ⛔ Não é cosmético: o cofre do modo privado sela o texto por `kind.is_note()`
   (`session_chair.rs` `chair_writes`); uma nota escondida que virasse rectângulo soltava o texto. O Miro só troca nota
   quadrada↔larga (isso já existe: `NoteWide` na barra das notas).
2. **O ecrã (`ph2d-editor-core`):** um botão `Item::Reshape` no 1.º grupo de `style_groups(Selected::Shapes)` (só formas;
   nem `Both`, nem notas); clicá-lo abre a grelha de `ShapeType::ALL` (que não tem notas — conferido) ancorada nele
   (`float_bar::grid_rects(area, rect_do_botão, n, 3)`), e `Item::Pick(t)` com essa grelha aberta chama `set_shape_type`
   em vez de escolher a ferramenta. Fecha ao escolher, ao trocar de selecção ou de ferramenta (nunca fica um estado
   aberto por baixo de outra barra). O «actual» da grelha = o tipo da 1.ª forma seleccionada.
3. ⛔ **`board_bar.rs` está a 686 de 700 linhas** (tecto `architecture_workspace_file_loc_cap`) e é o ficheiro de
   conflito da integração (`TOOLBAR`). Corte ANTES, por assunto — a grelha das formas (o `shapes_open`, o `Pick`, este
   botão) num `board_bar_shapes.rs`, como os irmãos `board_bar_notes.rs`/`_pen.rs`; o `apply_event` só despacha.
   O id de cada `Item` é o índice da tabela `items()`: um `Item` novo só se acrescenta no FIM do grupo.
4. Gates: o clique real (`shell_click_bar` + `shell_click_id`) troca o tipo de duas formas num passo do desfazer; uma
   selecção com nota não mostra o botão; a grelha fecha ao trocar de selecção; e a lei da nota no `board-edit` (com a
   mutação de tirar o filtro). i18n: uma chave `board.style.shape` (o balão).
5. Cena: a cena 2 ou 3 do quadro (formas) ganha a lição, se o roteador de cenas o pedir — conte o número no roteador.

## Lembretes MEDIDOS nesta janela

- **O arnês do quadro tem `T.vp_h`** (a altura da janela): uma janela baixa é `t.vp_h = …; t.paint()`.
- ⛔ **Uma grelha de alturas folgada não cai numa faixa de 4 px** (a M4 da coluna sobreviveu a dois testes): varra
  TODAS as alturas de dois passos de fila (`360..=420`). É a lição «grelha nunca cai num conjunto de medida nula».
- ⛔ O `cargo-test-narrow.sh` não mostra o `stdout` de um teste com `--nocapture`: para uma sonda, `bash
  scripts/ph2d-run.sh cargo test -p <crate> --lib <nome> -- --nocapture`.
- ⛔ O shell é `zsh`: `$P` com vários caminhos NÃO se parte — escreva os caminhos no `git add`/`commit`.
- Um script de troca que filtra por `endswith('live.rs')` apanha `smoke_fac_live.rs` — filtre pelo caminho inteiro.
