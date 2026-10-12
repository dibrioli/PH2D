# INTEGRAÇÃO — `line/MiroClone` (W12…W19) no `main` (2026-10-12, por ordem do dono)

> O que cada onda trouxe está no handoff dela ([W19](HANDOFF_INTEGRACAO_line_MiroClone_W19_2026-10-12.md), que lista
> W18…W12). Aqui só o que a INTEGRAÇÃO fez. Próxima onda: [briefing W20](HANDOFF_CONTINUACAO_W20_VIDEO_PAN_E_DECORACAO_2026-10-12.md).

## 1. A ordem e a base

Uma linha só ⇒ sem par a medir. `collision-surface.sh` contra o `main` do dia (`b1dd77e477`, +53 commits desde a base
`f4a6df7a34`: a Luz2D W5d–W5m): schemas e registos iguais à base (`PROJECT_SCHEMA` 188, componentes 107/108/108);
contratos congelados intocados; ADR: o `main` acaba no **0181** e nenhuma outra linha aberta cria ADR ⇒ **0182–0185
da linha ficam como estão**; `main.rs` da shell 1057 → 1061 (`file_loc_caps` e `the_shell_only_shrinks` verdes).

## 2. O rebase (126 commits, sem `Solved`)

- **`CLAUDE.md` ×5** (a entrada do Quadro ao lado da da Luz 2D): resolvidos pelos ESTÁGIOS por um script que recusa
  se os dois lados mudam a mesma região — a da linha sobre a do `main`.
- **`crates/ph2d-app-flip/src/pass.rs` — colisão de DESENHO na mesma função.** A W15 partira o `composite_layers` num
  `compose` sem blit (o «Send to board…» lê a saída); a Luz2D W5j passou o passe 3 do ecrã a compor em TRECHOS com a
  luz de cada objecto (`blit_com`). Fusão: `stage_slices` (passes 1–2, comum) → o ecrã `composite_lit` (a lei da
  W5j, igual ao `main`) · o `compose` do «Send to board…» faz UMA composição inteira sem luz (o quadro do desenho), e
  mora agora no `pass_raster.rs` (o `pass.rs` passava o tecto: 710 → 673).
- `range-diff`: só os 6 commits resolvidos diferem dos originais; os outros 120 iguais.

## 3. O portão da árvore combinada

Registos (sync sem diferença; staleness ✓) · `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` ✓ ·
clippy do `ship.sh` ✓ · `fmt`/`machete`/`deny` ✓ · censos 117/117 ✓ · `file_loc_caps` + `the_shell_only_shrinks` ✓ ·
Flip `--ignored` na placa 21/21 ✓ · `nextest-impacted` (BASE=main): 18 801 verdes; os ✗ meus curados (o tecto do
`pass.rs`; o índice da memória 22 167 > 22 000 — o ponteiro do YouTube desceu para o tópico «Comunicação & decisão»,
67). **Os vermelhos que ficam são HERDADOS do `main`**, a lista da Luz2D ([W5m §5](../../Luz2D/handoffs/HANDOFF_CONTINUACAO_W5m_2026-10-12.md)):
- `ph2d-app-light2d`: `as_formas_redondas` ×3, `o_miolo_das_formas…`, cenas 5/6/7 «ensina», `o_traco_do_flip_recebe_a_luz_como_o_sprite`,
  `o_interior_do_opaco_nao_depende_da_posicao_na_grade`, `o_rebote_nao_depende_da_oitava`. **Provado ao bit** para os
  8 primeiros: com o caminho do ecrã do Flip VERBATIM o do `main` (ablação da fusão) as mensagens saem iguais, linha a
  linha (o do Flip: `3.12e-3` nos dois).
- `ph2d-light2d-gpu` (em série, na placa, 898 s): 65 verdes, 3 vermelhos = os da lista (`c3_grade_fina` ×2, `g5…`).
- Os 6 timeouts e os 25 que o `nextest` não chegou a correr (sem placa reservada, prazo de 1 800 s): corridos em série
  na placa — verdes, salvo os herdados acima.

## 4. A memória

A decisão do YouTube (W18) que o handoff da W18 mandava recolher do primário entrou pela linha
(`project_youtube_in_the_board_uses_the_persons_own_browser_2026_10_12.md`, com a nota da W19). O avanço do `main`
devolveu ao primário, por fusão a três vias, o que outras sessões tinham por gravar em `MEMORY.md` e no tópico dos
gotchas (a contagem dos gotchas conta-se no ficheiro).

## 5. Não feito aqui

Ship/push: só por ordem do dono.
