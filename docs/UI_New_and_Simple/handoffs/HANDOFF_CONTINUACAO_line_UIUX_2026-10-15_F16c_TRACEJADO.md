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

1. Popover do traço: a altura é cortada à região (`min(corpo, região)`) mas as linhas não têm recorte nem
   rolagem — num ecrã muito baixo transbordariam (achado pelo agente; anterior a esta onda).
2. Smoke do dono das cenas 23 e 24.
3. No fecho: clippy + suíte `PH2D_SUITE=fecho`; CLAUDE.md §5.1 UI/UX (lista de smoke `6..24` e o «Último:»).
4. A placa (`ph2d-shape-gpu`, `tracejado_do_eixo`) só percorre UM par; 2–3 pares caem na cerca `FLAG_SO_CONFORME`
   (mesma imagem; mais lento só sob escala não uniforme). Estender o shader é uma onda própria, medida.
5. Testes GPU do Motion (`motion_shape_placa_gpu*`) não correram (a placa estava presa pela `line_luz2d`);
   compilam, a mudança neles é só `DashPattern::single`.
6. Herdados do handoff anterior: `frame_margin()` com a ④ em duas linhas; ids órfãos `VECTOR_RULERS_OFF/ON`.
