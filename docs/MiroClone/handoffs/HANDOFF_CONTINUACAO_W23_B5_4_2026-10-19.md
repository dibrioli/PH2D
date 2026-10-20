# CONTINUAÇÃO — `line/MiroClone` W23: B5(4) o desfazer das abas feito; mutações do B5(2) e do B5(4) corridas — 2026-10-19

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_B5_2](HANDOFF_CONTINUACAO_W23_B5_2_2026-10-19.md) ·
> lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md). Base = main `100d34c8e0`.
> Sem integração nem push: só por ordem do Enio. ✅ **Smoke do dono da cena 27: OK (20/10).** O dono decidiu: a
> pergunta «Delete board» antes de apagar FICA.

## 1. B5(4) — FEITO

Commits: `c5e1e7a612` a feature + cena 27 · `9c20b94567` oráculo tldraw das páginas (23 casos) · `dec9bb6112` gates
reforçados pelas mutações · + o commit de docs (doc 15, doc 14, README, este handoff).

Criar, duplicar, importar, restaurar versão, renomear, arrastar e apagar um quadro desfazem-se. O `Ctrl+Z` de cada
documento (a cena, cada quadro) desfaz o MAIS RECENTE entre o último passo dele e a última operação da barra de abas
(relógio `ph2d_board_model::undo_tick`); o apagado volta inteiro — id, posição, conteúdo e o desfazer dele —, aberto só
se o estava. Mecanismo, leis do oráculo, diferenças deliberadas e recusas: **[15_desfazer_as_abas.md](../15_desfazer_as_abas.md)**.

## 2. O que um merge pode partir (para o integrador)

- **Shell** (`undo.rs`, `undo_app.rs`, `undo_route.rs`): `ProjectUndo` passou a `(estado, marca, hora)`;
  `undo_owner` ganhou o 5.º argumento `tabs_newer` e o dono `UndoOwner::Tabs`. Outra linha que mexa no roteamento do
  `Ctrl+Z` colide aqui — reconte os argumentos. A shell cresceu ~76 linhas (187 727 contra o tecto 189 041).
- `Documents::remove` devolve `bool` (era `Option<Board>`); `adopt_board` usa `boards.create` (não grava passo).
- `History` ganhou a hora por passo (`undo_at`/`redo_at`/`last_push`) — sem mudança de formato (é sessão).
- Sem formato nem `PROTOCOL` novos. `FORMAT_VERSION` 15 / `PROTOCOL` 9 continuam os do B5(2).
- `CLAUDE.md` §5 (Quadro) conta-se **1..27** (23 = lado de quem entra, 24–27 desta linha).
- i18n: `board.smoke.tabs.*` (cena 27).

## 3. Prova

- Gates por nome verdes: `undo_tests` (7), `documents::tabs::tests` (3), `undo_route` (4), `w23_tabs` (1),
  `history` (model, 6), `smoke` (app-board, 36), `frase` (i18n, 3).
- **Mutações (`mutacao`, 19/10):** B5(4) 14 + B5(2) 13 = 27; 21 sangraram à 1.ª; as 6 sobreviventes (M11, M13, B7,
  B8, B10, B12) mortas reforçando os gates (`dec9bb6112`) — tabela nos docs 15 §4 e 14 §4.
- `cargo fmt --check` limpo nas 5 crates tocadas. ⚠️ clippy `--all-targets` e as suítes inteiras NÃO corridas nesta
  onda (cadência: no fecho da linha, `PH2D_SUITE=fecho`).
- Fotos da cena 27 conferidas (1366x768 e 1930x1040).

## 4. ABERTO (nesta ordem)

1. ~~Smoke do dono da cena 27~~ ✅ OK 20/10.
2. ~~Pergunta ao dono sobre «Delete board»~~ ✅ 20/10: fica.
3. Bloco 4 (B6, B7, B10, B11 — MEDIR antes) · bloco 5 (B9, A11) · bloco 6 (B12, B13, B14 com `/pd-feature`).
   ⇒ **janela nova** (outra onda).
4. `textAlignVertical` (top/bottom do Miro): por medir, só existe na API — sem UI nem oráculo.
5. Perguntas ao dono sem resposta: YouTube Premium; «Rascunho/Final» e a letra importada.
6. Não feitos, nomeados: as abas do editor de scripts (aba de família) não entram nesta pilha — o `Ctrl+Z` delas é o
   do texto; com o Painter activo na cena o `Ctrl+Z` é dele, antes das abas (editor modal).

## 5. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=27 ./target/smoke/ph2d-host-desktop`
2. Na barra de cima: botão direito na aba «Archive» → «Delete…» → «Delete board». Ctrl+Z: ela volta no mesmo lugar, com
   a caixa «Archive board».
3. Duplo clique numa aba, escreva outro nome, Enter. Ctrl+Z devolve o nome antigo; Ctrl+Shift+Z o novo.
4. Arraste uma aba para outro lugar da fila; carregue «+». Ctrl+Z desfaz cada um.
5. No «Plan», desenhe uma caixa e depois renomeie uma aba: o 1.º Ctrl+Z devolve o nome, o 2.º tira a caixa.

Deu errado se: a aba que volta não abre ao clicar, volta noutro lugar ou vazia, o Ctrl+Z apaga um desenho antes de
devolver a aba, ou o Ctrl+Shift+Z apaga um quadro em que você mexeu depois. Fotos:
`../ferramentas/w23_fotos/cena27_desfazer_abas_{1366x768,1930x1040}.png`.
