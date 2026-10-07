# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, reaberta → W5 «Organizar» (2026-10-07)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Antecessores:
> [`HANDOFF_INTEGRACAO_line_MiroClone_2026-10-07.md`](HANDOFF_INTEGRACAO_line_MiroClone_2026-10-07.md)
> (o estado da linha inteira W0–W4, e o §10 com o que ficou aberto) ·
> [`HANDOFF_CONTINUACAO_W4_2026-10-07.md`](HANDOFF_CONTINUACAO_W4_2026-10-07.md).
> Plano: [`../02_plano.md`](../02_plano.md) — §3 W5, §6 (recusas MEDIDAS: leia antes de propor desenho).

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`, **base `2a925859d`** (o `main` depois da
  rodada de sete linhas de 07/10, já no GitHub). A W0–W4 está toda no `main` (`126845087`).
- `PROJECT_SCHEMA` no `main` = **185** (o degrau dos quadros foi recontado na integração). Campos novos
  do quadro vão no **`ph2d_board_model::FORMAT_VERSION`** (hoje `4`), nunca no `PROJECT_SCHEMA`; o leitor
  do formato anterior entra em `legacy.rs` com fixture gravada pelo build ANTIGO (o idioma da W4:
  `fixtures/format_v3.bin`).
- Escolha do dono (07/10): **W5 — Organizar** antes de acabamentos, W6 e `.excalidraw`.

## §1 — O alcance (plano §3 W5)

1. **Frames**: levam os filhos (mover/apagar o frame leva-os), nome e cor, aninháveis, pertença
   visível; painel de Frames.
2. Agrupar / desagrupar, travar, alinhar e distribuir, ordem (frente/trás), minimapa (`M`), busca
   (`Ctrl+F`) com filtro por etiqueta, links entre elementos.
3. **Nível de detalhe por zoom** (queixa aberta no Miro): longe, as notas viram blocos de cor e os
   frames mostram só o título.
4. **O dono vê:** um quadro grande que continua legível e rápido de navegar → cena de smoke nova
   `PH2D_BOARD_SMOKE=6` (o número conta-se no roteador `ph2d-app-board/src/smoke.rs`, hoje `max_level 5`),
   fotografada antes de mandar.

## §2 — Método (o que já funcionou nesta linha)

- **Leis do Miro** pelo help público (agente; `help.miro.com/api/v2/help_center/en-us/articles.json?page=1..7`
  — o endpoint de busca com `/en-us/` dá 404, use `…/help_center/articles/search.json`): Frames,
  Group, Lock, Align, Layers/order, Minimap, Search, Links, e o que o Miro faz com o zoom (texto que
  some, «zoom to fit»). Atalhos pela página «Keyboard shortcuts». Tabela lei → fonte, como no §1 da W4.
- **Oráculo que se corre:** o Excalidraw `0.18.1` já instalado em
  `ferramentas/excalidraw_oracle/` (`corre.sh`) tem frames, grupos, alinhar/distribuir e ordem —
  corra-o sobre cenas NOSSAS e compare número a número (as regras de pertença a um frame, recorte
  dos filhos, o que o alinhar faz a um grupo). O Miro é a referência de PRODUTO; o Excalidraw é a
  régua do número onde os dois concordam.
- **Medir antes de limitar (§0.0):** o nível de detalhe e o minimapa existem por desempenho — meça
  primeiro um quadro de 10 000 elementos (régua `ph2d-board-render/tests/it/measure_encode_cost.rs`, em
  final E em rascunho: o rascunho de quadro grande nunca foi medido, §10.4 do handoff de integração)
  e escreva o limiar com a tabela ao lado. Nada de `MAX_*` sem o recurso que o justifica.
- **A barra:** todo botão novo passa pelo arnês `shell_click_bar` (a ordem real da shell — a W4 achou
  por ele a faixa invisível das réguas); o gate «nenhuma zona de gesto da shell sobre um botão da
  barra» já existe e tem de continuar verde.
- Código no quadro vive nas crates `ph2d-board-*` / `ph2d-app-board` e nos ecrãs `board_*` da
  `ph2d-editor-core`; a shell só compõe (catraca `the_shell_only_shrinks`).

## §3 — O que fica aberto de antes (não é W5, não bloqueia)

Do §10 do handoff de integração: pressão da mesa (decisão do dono — dependência do `winit`),
capturas do Miro por medir (P/G das notas, sombra, marcador sobre branco), ponta de seta curta do
Excalidraw (`min(25, comprimento·0,5)`), laço/smart drawing, e os herdados W2–W3 (cotovelo da seta,
rótulo ao longo da linha, abas que transbordam, desfazer das abas, trackpad, menu do botão direito).

## §4 — Fecho da W5

Gate batched 1× sobre `2a925859d..HEAD` (`nextest-impacted`, clippy `--workspace --all-targets -D warnings`,
`file_loc_caps`, auditoria de 2 lentes), prova de mutação nas leis novas (pertença ao frame, ordem,
travar), cena 6 fotografada, smoke do dono em passos numerados, e este diretório ganha o
`HANDOFF_CONTINUACAO_W6_…` ou o de integração. **Não integre nem faça push sem ordem do dono.**
