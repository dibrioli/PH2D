# INTEGRAÇÃO — `line/MiroClone` W23: o que ficou aberto fechado, laço · desenho inteligente · menções · enviar para outro quadro · Rascunho/Final (2026-10-22)

> Leitor: o integrador. Antecessor no `main`: [W22c](HANDOFF_INTEGRACAO_line_MiroClone_W22c_2026-10-17.md). Lista-mãe da onda:
> [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) (cada bloco abaixo tem o seu handoff, com o mecanismo).
> **A linha PAROU aqui: não integrou nem fez push.**

## 0 — Base e tamanho

- Merge-base = `main` `100d34c8e0`; o `main` NÃO andou (`git log 100d34c8e0..main` vazio). Fast-forward possível se continuar assim.
- 105 commits até `a662e8f984` (último de código) + 1 de docs (este handoff, §5 do `CLAUDE.md`, notas do doc 17) = **106**. Confira com
  `git log --oneline 100d34c8e0..HEAD | wc -l`.
- Diffstat `100d34c8e0..a662e8f984`: 619 ficheiros, +172 622 / −10 158.

## 1 — O que a W23 entregou (um bloco por linha)

| bloco | o quê | handoff |
|---|---|---|
| 1 | enquadramento ao abrir (A1, 12 cenas deixam de compensar à mão), colunas da barra (A6), «+n» dos avatares (A3), fundo do vídeo (A4), frame de longe (A8) | [BLOCO1](HANDOFF_CONTINUACAO_W23_BLOCO1_2026-10-18.md) |
| 2 | rede ao vivo: ritmo (`Pace`) partilhado por presença e escritas (A9), A12, B8, fecho da sala sem esperar 2 s | [BLOCO2](HANDOFF_CONTINUACAO_W23_BLOCO2_2026-10-18.md) |
| cartão Live | pedido do dono: o cartão «Live» separado por caminho de ligação (Internet · Pessoas · sessão); cena 23 | [CARTAO_LIVE](HANDOFF_CONTINUACAO_W23_CARTAO_LIVE_2026-10-18.md) · [doc 12](../12_cartao_live_por_ligacao.md) |
| 3 «quadro» | palavras-chave com maiúscula (A7), foco Orbit (A10), tamanho da letra do mapa (B3), pontas (B5-3), cursor nas pegas (B5-5), trackpad (B5-6), botão direito (B5-7), abas a mais (B5-4 metade); cena 24 | [BLOCO3](HANDOFF_CONTINUACAO_W23_BLOCO3_2026-10-18.md) |
| B5-1 | trecho do meio do cotovelo; cena 25 | [SMOKE19_B5_1](HANDOFF_CONTINUACAO_W23_SMOKE19_B5_1_2026-10-19.md) · [doc 13](../13_trecho_do_cotovelo.md) |
| B5-2 | textos ao longo da seta (dobram com a curva); cena 26 | [B5_2](HANDOFF_CONTINUACAO_W23_B5_2_2026-10-19.md) |
| B5-4 | Ctrl+Z das operações de abas; cena 27 | [B5_4](HANDOFF_CONTINUACAO_W23_B5_4_2026-10-19.md) |
| 4 | velocidade e memória (B6 forma de longe, B7…); medido em [doc 16](../16_bloco4_velocidade.md) | [BLOCO4](HANDOFF_CONTINUACAO_W23_BLOCO4_2026-10-20.md) |
| 5 | fidelidade (B9) e A11 (`send_to_board::gather` fora da shell) | [BLOCO5](HANDOFF_CONTINUACAO_W23_BLOCO5_2026-10-20.md) |
| 6 | B15 Copy/Move to board, B12 laço, B13 desenho inteligente da caneta (QuickShape), B14 menções e avisos; cena 28 | [BLOCO6](HANDOFF_CONTINUACAO_W23_BLOCO6_2026-10-20.md) · [doc 17](../17_bloco6_plano.md) |
| 6, 2.ª onda | M4 (oráculo Excalidraw `next`), 63 mutações, realce azul das menções | [BLOCO6_ONDA2](HANDOFF_CONTINUACAO_W23_BLOCO6_ONDA2_2026-10-20.md) |
| 6 Sketch | «apertei e nada mudou»: 7 achados, uma cura; formato 16 → 17, protocolo 10 → 11 | [BLOCO6_SKETCH](HANDOFF_CONTINUACAO_W23_BLOCO6_SKETCH_2026-10-21.md) |
| 6 Sketch barra | Rascunho/Final também na barra da FORMA seleccionada | [BLOCO6_SKETCH_BARRA](HANDOFF_CONTINUACAO_W23_BLOCO6_SKETCH_BARRA_2026-10-22.md) |
| fecho | auditoria do fecho (duas lentes, §4) e as curas de `a662e8f984` | este |

## 2 — Foundational tocado e porquê é ADITIVO

Medido com `git diff --numstat 100d34c8e0..HEAD`. **Puro acréscimo (0 linhas removidas):** `ph2d-editor-core`: `documents_alert.rs`,
`documents_tabs.rs`, `documents_team.rs`, `menu_tables_board.rs` (novo), `paint_text.rs` (+108), `float_bar.rs` (+31),
`interaction/state/graph_ops.rs` (+29), `types_menu.rs` (+20), `key.rs` (+1); `ph2d-app-host` `clipboard.rs` (+33);
`shells/desktop` `input_dispatch.rs` (+1), `despacho_metodos_janela_e_vetor.rs` (+13), `fase_pointer_subjects.rs` (+2), `undo_app.rs` (+3);
`Cargo.lock` (+2).

**NÃO puramente aditivo (há linhas removidas) — o integrador roda `collision-surface.sh` (caminho ABSOLUTO, em cada worktree) nestes:**

| ficheiro | +/− | porquê |
|---|---|---|
| `ph2d-editor-core/src/documents.rs` | +74/−8 | ações de abas desfazíveis (B5-4) |
| `…/documents_live.rs` · `documents_nav.rs` | +14/−1 · +21/−2 | cartão Live por ligação; abas a mais |
| `…/ids/chrome/documents.rs` | +20/−1 | ids novos do cartão/abas |
| `…/hero/menu_tables.rs` | +0/−12 | **MOVIDO** para `menu_tables_board.rs` (63 linhas novas): as linhas do menu do quadro |
| `…/hero/menu_rows.rs` | +9/−2 | linhas do botão direito do quadro |
| `…/hero/context_menu_overlay.rs` | +3/−1 | idem |
| `…/hero/document_tabs.rs` · `_menu.rs` · `_menu_tests.rs` · `_tests.rs` | +103/−13 · +20/−3 · +6/−6 · +74/−2 | abas (B5-4) |
| `…/hero/send_to_board.rs` · `_tests.rs` | +75/−22 · +2/−2 | Copy/Move to board (B15), A11 |
| `…/hero/paint.rs` | +21/−1 | HUD/canto (já da W22c) e a barra do quadro |
| `…/src/paint.rs` | +2/−2 | — |
| `…/dispatch/pointer_down_menus.rs` | +1/−1 | botão direito do quadro |
| `ph2d-vec-edit` `pencil.rs` | +7/−10 | B13: o laço do traço da caneta passa à porta do QuickShape |
| `ph2d-vec-edit` `quickshape_hold.rs` | +27/−7 | idem (reconhecimento ao segurar) |
| `ph2d-i18n/src/lib.rs` | +3/−1 | módulos de chaves do quadro |
| `ph2d-panel-registry-init/tests/it/nenhum_rotulo_do_app_pinta_nada.rs` | +3/−8 | o teste passa a contar o quadro |
| `shells/desktop` `forwarding.rs` | +2/−7 | encaminha o que passou para `board_view` |
| `…/main.rs` | +2/−1 | — |
| `…/undo.rs` · `undo_route.rs` | +21/−9 · +73/−12 | abas e quadro entram na fila de undo por DIFF; o aviso «Undo» do Move |
| `…/render_loop/fase_send_to_board.rs` | +5/−14 | A11: a escolha vive em `ph2d_app_board::send_to_board::gather`; **a shell encolhe** |
| `…/tests/it/o_send_to_board_leva_a_seleccao_com_a_pose.rs` | +21/−16 | idem |

Todos são mudanças de comportamento DENTRO do ramo do quadro (nenhuma assinatura pública de outra família mudou).
Contratos congelados §6 (`nodegraph`, `ph2d-tool-*`, `ph2d-vector`, `ph2d-imageio`, `project.rs`, `schema`): `git diff --name-only
100d34c8e0..HEAD | grep -E "nodegraph|ph2d-tool|ph2d-vector/|ph2d-imageio|project.rs|schema"` ⇒ **vazio**.

## 3 — Contadores como DELTA (reconte ao integrar)

- `PROJECT_SCHEMA` = **191**, **inalterado** (`shells/desktop/src/project_schema.rs:472`). Os quadros viajam como blob opaco.
- ADR: **nenhum novo** (o último continua 0188). Decisões vivem nos docs 12–17.
- Formato do ficheiro do quadro `FORMAT_VERSION` 13 → **17** (`ph2d-board-model/src/board.rs:23`); protocolo vivo `PROTOCOL` 7 → **11**
  (`ph2d-board-sync/src/session.rs:23`). **Contadores locais da linha**: se outra linha tocar neles, reconte
  (`python3 scripts/schema-recount.py` para o degrau do projeto; estes dois contam-se no código). Leitores antigos e fixtures:
  `legacy_v8/v10/v12/v13/v15.rs` + `legacy_v16.rs` (novo), `fixtures/format_v16.bin` (escrita pelo formato 16 REAL, `b832018d9e`) e as
  da v13–v15.
- Cenas de smoke `max_level` (`ph2d-app-board/src/smoke.rs:110`) 22 → **28** (+6): 23 entrar ao vivo (`smoke_live.rs`) · 24 mapa mental
  em rascunho (`smoke_w23.rs`) · 25 meio do cotovelo (`smoke_w23_elbow.rs`) · 26 textos ao longo da seta (`smoke_w23_text.rs`) · 27 Ctrl+Z
  das abas (`smoke_w23_tabs.rs`) · 28 «Block 6», quatro estações (`smoke_w23_block6.rs`). **Outra linha que acrescente cena: reconte.**
- `Style` ≤ 40 bytes (gate `the_style_stays_in_forty_bytes`); `Field::ALL` 40 → 41.
- Registo de componentes: zero.

## 4 — O que um merge pode partir

- Formato/protocolo (§3): quem também subir um, conta de novo. Leitores `legacy_*` e `legacy.rs` tocados: outra linha ali conflita.
- `Style` ≤ 40 bytes e `Field::ALL` 41: tabelas que contam campos (menus, oráculo) reconferem.
- `board_bar::items()`: o par `Look` (Rascunho/Final) fica no FIM; quem acrescentar `Item` ao fim reordena
  (`images_w16::the_w16_items_come_last_and_move_no_id`).
- Ficheiros novos `ph2d-board-edit/src/look.rs` e `ph2d-board-render/src/sketch/ink.rs`; quem tocar nos mesmos blocos de `lib.rs` /
  `sketch.rs` reaplica nos novos sítios. Hachurado de vídeo e ramos do mapa mental passaram a PREENCHIMENTO (não traço): asserções de
  `Stroke` em `video_tests.rs`, `mind_sketch_tests.rs`, `sketch_tests.rs`, `tests/it/sketch_paint.rs` reescrevem-se.
- `float_bar::strip_rects_fitting` (nova) e `board_bar_layout::style_rects_of`: a barra de estilo já não cobre a de ferramentas ⇒ posições
  mudam junto ao bordo esquerdo e em janelas estreitas.
- ROUTERS `max_level` (cenas) e `Cargo.lock` (+2).
- Oráculo `docs/MiroClone/ferramentas/excalidraw_oracle/{entradas,saidas}`: regenerado, nunca à mão.
- Chaves i18n `board.style.final`/`board.style.sketch`, `board.live.*`, `board.smoke.*`; o gate `no_tofu_glyphs` recusa `→` em testes.

## 5 — Prova de fecho (MEDIDA hoje, 22/10)

| o quê | resultado |
|---|---|
| `PH2D_SUITE=fecho` `nextest-impacted` (`loadavg` 16–50) | 19 269 corridos, 19 267 passaram, **2 falharam** — os dois gates novos red-first da auditoria do fecho, apanhados enquanto a cura estava de lado para os ver vermelhos: `what_the_bar_births_follows_a_board_mode_changed_elsewhere`, `a_far_pen_shapes_width_is_chosen_at_its_birth_scale_and_lit`. Com a cura (`a662e8f984`) passam, 3/3 sozinhos cada; o filtro `board_` inteiro de `ph2d-editor-core` 242/242; `ph2d-board-edit` `ink` 22/22 |
| suíte da onda anterior (22/10) | 19 793 corridos, 2 vermelhos = tecto de LOC (`ph2d-board-edit` `lib.rs` 743, `ph2d-board-render` `sketch.rs` 770) e `no_tofu_glyphs`; curados em `efc211c86a` |
| clippy `--workspace --all-targets --all-features -D warnings` | ✓ (re-corrido em `ph2d-board-edit` + `ph2d-editor-core` depois da cura ✓) |
| `cargo fmt --check` · `file_loc_caps` (4/4) · `arch_safe_clamp_only` (2/2) · `architecture_*` de `ph2d-host-desktop` | ✓ |

**Auditoria do fecho (2 lentes).** Lente A, costuras — 3 defeitos reais curados em `a662e8f984`: (1) o modo do quadro mudado por fora
(colega, Ctrl+Z do aviso) não chegava ao estilo do editor fora do rato ⇒ `board_view::sync_mode` em cada entrada (rato, barra, teclado);
(2) a espessura da forma da caneta escolhida crua ⇒ `choose_width`; (3) a espessura da luz lida crua ⇒ `chosen_width`.
Lente B, docs — `CLAUDE.md` §5 dizia cenas 1..22 (corrigido); doc 17 §7 «`max_level: 27`» velho e §6 com 21 de 23 nomes de gate que nunca
existiram, citando protocolo 9 (código: 11): ambos com nota ⚠️ no topo, plano não reescrito.

## 6 — Smokes (o dono; da pasta da LINHA, antes da integração)

Comando (troque `<n>`): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=<n> ./target/smoke/ph2d-host-desktop`.
Duas janelas (cenas 22 e 23): na 2.ª janela rode o binário sem a variável; sem rede pública, `PH2D_BOARD_INTERNET=local` nas DUAS.
Passos, o que ver e como saber que errou: no «Smoke ao dono» de cada handoff do §1.

Estado dos smokes do dono (só onde está escrito no handoff): cena 22/cópia do link OK 18/10 · 24 e 25 OK 19/10 · 26 OK 19/10 · 27 OK 20/10
(a pergunta «Delete board» FICA) · blocos 4 e 5 OK 20/10 · **cena 28 OK 22/10** (Rascunho/Final na barra da forma). Cena 23 (cartão Live):
sem OK escrito no handoff.

## 7 — Decisões do dono em 22/10

- Rascunho/Final na barra da FORMA seleccionada: **feito**. `Ctrl+Z` com a caneta pousada: **fica como está**.
- Alinhamento vertical do texto nas formas: **não quer** (fecha `textAlignVertical`). A letra NÃO acompanha a escala de nascença no ecrã (fecha).
- «Rascunho/Final troca a letra importada»: já decidido na W10 (`ph2d-board-edit` `look.rs`: `toggle`/`set_sketch` põe `Letter::Auto`) — fecha.
- YouTube Premium: medidos 0 anúncios em 6 (W19) — nada a construir, salvo se o dono vir um anúncio.

## 8 — ABERTO depois da linha

1. A barra de estilo pode cobrir uma gaveta de ferramenta aberta (notas · caneta · modelos · formas): ambas vivem à direita da barra de
   ferramentas e a de estilo pinta-se depois. Classe anterior à W23; **não curado**.
2. Com a sala mais estreita que um grupo (janelas minúsculas) a barra de estilo ainda pode passar à esquerda por cima da de ferramentas. **Não curado.**
3. Elementos antigos de nascença 0 ficam a escala do mundo (desenho: ficheiros de antes abrem nascidos a zoom 1).
4. Otimização do caminho de longe da hachura: sem medição (doc 16, recusas).
5. Doc 17 §6: nomes planeados que nunca existiram (nota ⚠️ posta; os gates reais estão no §9 e nos handoffs).

## 9 — Integração no `main` (22/10, integrador)

- Ordem: uma linha só; `main` parado na base `100d34c8e0` ⇒ rebase vazio, **fast-forward** para `764578e085` (106 commits), sem
  commit de regen (os três geradores não mudaram nada).
- `collision-surface.sh` (caminho absoluto, na worktree): `PROJECT_SCHEMA` 191 = base (tripla `(191, 13, 23)`), registo de componentes
  108/109/109 = base, contratos §6 intocados, ADR nenhum (próximo livre 0189), `Cargo.lock` sem pacote externo novo, zero marcadores.
  Único `✗`: `shells/desktop/src/main.rs` 1061/600 — isento em `file_loc_caps.rs:36`, era 1060 na base.
- Portão da árvore combinada (`foundational-integrate.sh` sob `ph2d-run.sh`): staleness ✓ · `CARGO_BUILD_WARNINGS=deny cargo check
  --workspace --all-targets` ✓ · `PH2D_SUITE=fecho nextest-impacted` **19 269/19 269** (841 s, `loadavg` 5–56).
- Bateria depois do merge: `cargo fmt --all --check` ✓ · `clippy --workspace --all-targets -D warnings` ✓ · `censos-da-arvore-combinada.sh`
  121/121 (12 de 12 censos correram) · `file_loc_caps` 4/4. Nenhum flake, nada curado.
- Binário do smoke compilado no primário (`--profile smoke`). **Sem push** (ship = outra ordem do dono).
