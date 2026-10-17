# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 15/10 — F16c (tracejado em até três pares, seta por ponta) feita

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `0aab2c1cb6`. Anterior:
  [F16 a/b](HANDOFF_CONTINUACAO_line_UIUX_2026-10-15_F16_BARRA_DE_CONTROLE.md).
- Plano e decisões: [`26_plano_F16_barra_de_controle.md`](../vetor_nova_ui/26_plano_F16_barra_de_controle.md) §4b (mapa) e §4c (feito).

## 1 — Commits desta janela

| commit | o quê |
|---|---|
| `d138b96401` | os BYTES de referência gravados pelo código ANTES da mudança (v22 da cena, v189 do projecto) |
| `695c786c7b` | F16c inteira — lista completa na mensagem (`git show -s 695c786c7b`) |
| `ff16b2cb9b` | os 8 vermelhos da suíte afectada (§4), 4 desta onda e 4 herdados da F16a/b |
| `4a2776c969` | report do dono (faixa branca sob o retângulo da cena 24, conforme o pan/zoom): o fecho de um tracejado ajustado caía EXACTAMENTE no início ⇒ o arredondamento do `kurbo::dash` emendava uma lasca de `3,07e-7` numa cúbica degenerada, e o Vello pintava-a como faixa. `dash_fit::fit_traced` (folga `FOLGA_DO_AJUSTE` também no caminho LOCAL). Anterior à F16c (um par também). Gates: `seam_tests` (CPU) e `a_cena_24_nao_pinta_a_faixa_do_fecho_em_vista_nenhuma` (GPU, 0/144 vs 117/144). Resto do Vello: BUGS_vector #42 |
| `24bf91be29` | report do dono (*«Align inner e outer não ficaram corretos»*, cena 24): o Inner/Outer TRACEJADO era a banda a `2w` cortada — a caneta dupla dobrava as pontas (ponto = meio disco de raio `w`) e fechava todo vão `< 2w`: Centre 45 peças, Inner/Outer **1 peça** de `117 w²`. Lei nova: o tracejado percorre o TRILHO (`região ∓ banda(w)`) com a caneta autorada, a começar no início do contorno e no sentido dele; o sólido fica com o recorte (ao bit). Outline Stroke e SVG passam pela mesma porta (assavam/exportavam CENTRADO). Custo smoke: `2,37 → 0,75 ms`. Cena 24 ganha o rectângulo Inner (x = 3,3) e Outer (x = 5,1). Anterior à F16c. Gates: `align::dash_tests::*`, `outlining_an_aligned_stroke_bakes_the_aligned_band`, `an_aligned_stroke_exports_the_band_it_draws`. BUGS_vector #43 |

## 2 — Decisões que um merge/leitor pode partir

- **Formato**: `VEC_SCENE_SCHEMA_VERSION 22 → 23`, `PROJECT_SCHEMA 189 → 190`, tripla `(190, 13, 23)`. ⚠️ Na
  integração o `190` RECONTA-SE (`python3 scripts/schema-recount.py`) e a constante irmã
  `PROJECT_SCHEMA_TRACO_V22` (= o degrau anterior a este, hoje `189`) move-se com ele — ela é o `match` que
  lê os ficheiros antigos pela forma v22 do traço (`project_load_read.rs`).
- **Leitura do antigo sem tipo congelado**: só o `StrokeSpec` tem forma antiga (`ph2d-vec-scene/src/stroke_wire.rs`,
  `Deserialize` escrito à mão), escolhida por um âmbito thread-local (`StrokeLayoutV22`). Os blobs de `VecPath`
  dentro de componentes (Envelope, `SkinBind`) são REGRAVADOS no load (`WorldSnapshot::rewrite_component_blobs`).
  ⛔ Linha nova que mude a forma do `StrokeSpec` outra vez: congela o v23 ao lado do v22, não o substitui.
- **Fixtures de bytes** (`ph2d-vec-scene/tests/it/fixtures/vec_scene_v22_traco.bin`,
  `shells/desktop/tests/fixtures/projecto_v189_traco.ph2dproj`): ⛔ nunca regenerar.
- **Piso do tracejado** `MIN_DASH_FRACTION = 1e-3` da largura (era `f64::EPSILON`): a foto da cena 24 mostrou
  os PONTOS do «traço-ponto» a sumir.
- **Painel**: «Dash / Gap 1..3» (uma linha por par) — a 1.ª redacção (Dash [3 caixas] / Gap [3 caixas]) partia
  no painel solto (foto). A linha do par seguinte só aparece quando o anterior traça (`visible_dash_rows`, a lei
  do `DashPattern::active`) ⇒ a abertura do painel do vetor DESCEU `916 → 887`. Os sliders
  `VECTOR_DASH`/`VECTOR_GAP` saíram; `*_NUM` são as caixas do 1.º par.
- Lápis com farpas: cada ponta com o tamanho da SUA farpa (era a média das duas).

## 3 — Smoke (por fazer pelo dono)

Cena 24: `PH2D_OBJECT_MODE_SMOKE=24` (`crates/ph2d-app-vec/src/vector_mode_tracejado.rs`, estágios 240–245), foto
[`F16c_cena24_tracejado_em_pares.png`](../vetor_nova_ui/fotos/F16c_cena24_tracejado_em_pares.png). A cena 23 continua por
fazer também. Os `.log` irmãos das fotos ficam não rastreados.

## 4 — Verificação

- Provas novas, por nome, verdes: `stroke_v22_fixture` (2), `stroke_wire` (3), `dash_pattern` (3) em vec-scene;
  `traco_v22_tests` (2) em app-vec; `traco_v189` (2), `project_schema` (1), `project_migrate` (13) na shell.
- Os testes dos 28 ficheiros adaptados, por nome: verdes (lista na conversa do agente; 7 testes novos, ver commit).
- `cargo check --workspace --tests`: sem erros nem avisos.
- Suíte afectada (`nextest-impacted`, motivo: formato gravado): 20 180 corridos, 8 vermelhos, todos
  determinísticos (reproduzem sozinhos), todos curados na causa e re-corridos por nome:
  - desta onda: `clamp_calls_in_ui_are_safe_or_justified` e `no_magic_numeric_in_widget_or_screens`
    (`paint_dashes.rs`); `a_altura_de_abertura_de_um_painel_so_encolhe` (937 com as três linhas à vista ⇒
    linhas progressivas, 887); `the_layout_drain_asks_the_type_before_converting` (janela de bytes cortada num
    `ó` — os cinco cortes do ficheiro passam por `janela()`);
  - herdados da F16a/b (nunca tinham passado pela suíte): `bars.rs` 699 > 600 (cortado em `bars_actions.rs`);
    `vector_mode.rs` 733 > 700 (tabelas das cenas em `vector_mode_scenes.rs`, com gate da equivalência);
    `set_dropdown_popover` à mão no popover do traço (agora `publish_dropdown_popover`, a porta que o
    `paint_dropdown_list` usa); `paint_snap.rs` sem AccessKit (agora pelos `NodeId`).
- Clippy: NÃO correu (só no fecho, R7).

## 5 — ABERTO

1. ~~Popover do traço sem recorte nem rolagem~~ — FEITO 16/10 (`f62d175678`, §6).
2. Smoke do dono: cena 24 OK (16/10, «smoke ok», depois das correções da faixa e do Inner/Outer); a 23 sem confirmação explícita.
3. No fecho: clippy + suíte `PH2D_SUITE=fecho`; CLAUDE.md §5.1 UI/UX (lista de smoke `6..24` e o «Último:»).
4. A placa (`ph2d-shape-gpu`, `tracejado_do_eixo`) só percorre UM par; 2–3 pares caem na cerca `FLAG_SO_CONFORME`
   (mesma imagem; mais lento só sob escala não uniforme). Estender o shader é uma onda própria, medida.
5. ~~Testes GPU do Motion não correram~~ — CORRIDOS 16/10: os 11 de `motion_shape_placa::gpu_tests` verdes
   (7 de paridade em 3,2 s, as 4 sondas/custos em 8 s, um de cada vez; os 10 sem a `intercalada` também
   verdes em paralelo). ⚠️ Achado da linha do MOTION (anterior a esta onda, não curado aqui): o comando do
   cabeçalho de `motion_shape_placa_gpu_tests.rs` (`… gpu_tests -- --ignored`) apanha a submódulo
   `intercalada::sonda_intercalada`, que faz `pass_profiler::init_forced` — medidor GLOBAL do processo,
   ligado ao device DELA. Em paralelo, os outros testes (cada um com o seu device) caem (8 FAILED,
   reproduzido 2×) e a sonda pendura em `futex` a 0 % de CPU segurando a placa (14 min até `kill`). As
   sondas de `ph2d-fx-cells/tests/it/celulas_gpu.rs` têm a mesma forma. Cura certa: o profiler só regista
   passes do device com que nasceu (ou as sondas correm sob um cadeado de processo) — é do `ph2d-gpu`.
6. ~~`frame_margin()` com a ④ em duas linhas; ids órfãos `VECTOR_RULERS_OFF/ON`~~ — FEITOS 16/10 (§6).
7. Formato: a linha tinha deixado 41 ficheiros fora do `cargo fmt` (o `main` limpo) — `4b33991e07`.

## 6 — Janela 16/10: as pendências pequenas (`f62d175678`, `4b33991e07`)

- **Margem de cima do enquadramento** (medido: margem fixa `44` px = UMA linha da ④ a `Lg`; a ④ do vetor
  tem duas a `selection_gap` = `51` px de altura e, sem lugar em cima, era empurrada contra a borda de
  baixo, tapando `7` px da arte enquadrada; com uma linha já não cabia em cima, `48 > 44`):
  `float_bar::frame_top_margin(rows, s, pinned)` = `max(frame_margin, afastamento + linhas + 2·Xs)`, lido
  das linhas REAIS da ④ (`tool_bars::frame_top_margin(hero)`); `canvas_area::frame_target` tira o excesso
  de cima da área enquadrável (o resto da margem fica simétrico). Gate
  `the_frame_top_margin_keeps_the_strip_above_the_framed_selection`; mutação «margem fixa» → vermelho.
- **Painel do traço**: fica do lado do chip que a fila deixa, encolhe ao que sobra na região, ROLA (roda
  e polegar pela `publish_dropdown_popover` de sempre), recorta cena e hits. Porta nova
  `panel::paint_popover_scrollbar` (o censo `architecture_dropdown_lists_use_the_door` proíbe a barra à
  mão num painel). Gate `on_a_short_window_the_stroke_panel_scrolls_inside_the_region` (herói real,
  janela a meia altura, controlo de transbordo); mutações «sem recorte dos hits» e «sem encolher» → vermelho.
- **A roda sobre o painel do traço dava ZOOM no canvas** (report do dono, `d2e190b0c7`): a shell só dá a roda
  ao herói quando `chrome_panel_at` → `panel_at` acha um `panel_rect`, e os dropdowns pintados sobre o canvas
  (o painel do traço, as listas da fila) vivem no slot `dropdown_popover`. `WidgetStore::open_dropdown_popover_at`
  é agora a pergunta única (o despacho da roda e o `chrome_panel_at` usam-na). O gate da janela baixa pergunta
  `chrome_panel_at` como a shell; mutação → vermelho. Lição: o teste anterior chamava `handle_wheel` e saltava
  a pergunta da shell — um gate de roda tem de passar pela porta que a SHELL consulta.
- **`VECTOR_RULERS_OFF/ON`** saíram com o canal `pending_rulers` inteiro (as réguas são do menu View, que
  escreve `hero.view` direto); o nome do ficheiro `fase_compound_snap_rulers.rs` ficou (só o cabeçalho mudou).
- Verificação: os 3 gates novos + 10 por nome dos censos tocados (`clamp_calls…`, `no_magic_numeric…`,
  `architecture_dropdown_lists_use_the_door`, `the_shell_only_shrinks`, `float_bar` 15, os dois do snap
  na shell, `a_altura_de_abertura…`, `view_request` 4, `canvas_area`) verdes; `cargo check --workspace
  --tests` sem erros nem avisos; `cargo fmt --all -- --check` limpo. Suíte e clippy: no fecho (R7).
