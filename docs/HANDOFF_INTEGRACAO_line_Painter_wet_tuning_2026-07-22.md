# HANDOFF DE INTEGRAÇÃO — `line/Painter` · a UI completa do Wet Paint (doc 22)

> DIRETRIZ §1.5.9. A linha está FECHADA e **NÃO integra nem faz ship** (o Enio cancelou
> explicitamente a integração/ship/CI desta jornada em 2026-07-22 — outros agentes vão
> trabalhar em suas linhas; este handoff espera o integrador da próxima janela).

## 1. Identidade

- **Branch:** `line/Painter` · **base do fork:** `13a04c7aa` (o main integrado de 2026-07-22).
- **Commits:** 12 (W1 engine → W2 tool → W3 seção básica → W4 painel lateral → fechamento →
  **fix pós-smoke: painel arrastável/redimensionável + heading engole o clique** →
  **doc 23: estudo + implementação — o pigmento responde às tools** →
  **fix pós-smoke: Impasto mostra TODOS os cards com Enable ON** (a estreiteza
  selected-tool-only revertida; Material fan-out pros 3 slots de relevo), ver §2/§6;
  checkpoint de reversão do doc 23: tag `checkpoint-pre-wet-tools-rework`).
- **Plano:** [`docs/Painter/22_plano_wet_tuning_ui.md`](Painter/22_plano_wet_tuning_ui.md).
- **Gate batched:** `nextest-impacted` **5031/5031** · clippy `--all-targets` **0 warnings** nas
  8 crates tocadas · engine debug **E** release verdes (a lição do voronoi) · fingerprint pinado
  intacto · 13 arch-gates verdes · 3 mutações dirigidas sangram (porta de tool → RED de paridade
  bit-exata; rota de SetValue do tuning → RED; véu do Show Wet bakando → RED).

## 2. Foundational/compartilhado tocado (tudo aditivo)

| Arquivo | O quê |
|---|---|
| `ph2d-editor-core/src/ids/chrome/painter_wetpaint.rs` | +7 tool ids, tilt pad/toggle/ring/spoke, 4 ações, paper_visual, tuning; `PAINTER_WETPAINT_CLICKS` **2→16** |
| `ph2d-editor-core/src/ids/chrome/wet_tuning.rs` | **NOVO** — painel/scroll/close + headers/resets/eye/km + a família dinâmica `wet_tuning_*_id(key)` (fnv runtime) + `WET_TUNING_DRAG_HANDLE`/`RESIZE_HANDLE`/`RESIZE_HANDLE_BL` (chrome de drag/resize, 1º smoke do Enio) |
| `ph2d-editor-core/src/widget/scrollbar.rs` + `widget/mod.rs` | `WET_TUNING_SCROLLBAR_ID = NodeId(837)` — **próximo livre: 838** |
| `ph2d-editor-core/src/interaction/dispatch/scroll.rs` | braço 837 → `WET_TUNING_PANEL` no `scrollbar_panel_for_id` |
| `ph2d-editor-core/src/screens/hero/paint.rs` | `WET_TUNING_PANEL` na fallback de z-order |
| `ph2d-editor-core/tests/hr12_widgets_a11y.rs` | entrada `PANEL_A11Y_DELEGATE_OK` p/ `paint_wetpaint_tilt.rs` (classe do `paint_shape_dab.rs`) |
| `ph2d-i18n/src/lib.rs` | +51 chaves `panel.wet_tuning.*` (título, 6 grupos, 40 knobs, K–M, nota) |
| `ph2d-panel-registry-init` | via `ph2d-panel-sync` (gerado) + `EXPECTED_TYPED` +1 (braço `panel-wet-tuning`) |
| `shells/desktop/Cargo.toml` | dep `ph2d-panel-wet-tuning` + `default` + feature `panel-wet-tuning` (a lição do physics: registry-init tem `default-features=false`) |
| `shells/desktop/tests/every_panel_the_shell_drives_is_in_its_registry.rs` | row `("wet_tuning", "panel-wet-tuning")` |
| `shells/desktop/src/forwarding.rs` | `WET_TUNING_PANEL` no `cursor_over_hero_panel` (wheel intercept) |
| `shells/desktop/src/render_loop/painter_bridge.rs` | publish do snapshot + espelho de visibilidade (`tuning_open`; OFF escrito FORA do downcast) + z-bump no edge |
| `ph2d-wet-paint` (engine) | portas ADITIVAS: `dispatch_pressure_dab_lane_blend` · `dispatch_pressure_dab_tool` (prev explícito) · `render_pigment_region_visual` (+`PigmentVisual`; `render_pigment_only_region` delega, off byte-idêntico) · `wet_canvas_now`/`dry_canvas_now`/`fast_dry_now` (sem `capture_history` — o clone de grid por aperto seria a doença do ADR-0117) · `tilt_dir_for_spoke` (cardinais EXATOS) · `knob_defaults()` const · `Tuning::default` delega |
| `ph2d-wet-paint` (engine, **doc 23**) | **MUDANÇA DE COMPORTAMENTO dos tools** (P1-P4, [doc 23](Painter/23_estudo_tools_wet_pigmento.md)): Wet dissolve `sett` sob o stamp · Smear arrasta o seco · Blend molhado re-suspende · Erase resiste por staining — porta única `drying::lift_settled` (extração VERBATIM do re-wet passivo) + `tools::active_lift_gain`. **`Knob::WetLift` apendado (`KNOB_COUNT` 53→54)** + `extStaining` Hidden→Paint (painel Tuning 40→42 rows; i18n +2 chaves). ⚠️ **O pin `fingerprint.rs::PINNED` MUDOU** (justificado no histórico do pin; o pin antigo virou o gate `wet_lift_zero_is_the_old_model_to_the_byte`). Gates: `tests/product_rewet.rs` (5, mutação-provados 5/5) |

## 3. Símbolos que podem COLIDIR

- `NodeId(837)` (scrollbar) — **hand-assigned**; se outra linha tomou 837, renumere (próximo 838)
  e o comentário em `scrollbar.rs` diz a regra.
- Família dinâmica `"wet_tuning.*"` (fnv de strings) — gate `wet_tuning_ids_dont_collide` roda
  sobre as chaves REAIS no crate do painel.
- Chaves i18n `panel.wet_tuning.*` (match do `tr()` — Mergiraf funde adições disjuntas).
- Feature cargo `panel-wet-tuning` (registry-init GERADO + shell) e o id de painel `"wet_tuning"`.
- `EXPECTED_TYPED` +1 — se outra linha também adicionou painel, **conte, não escolha**.

## 4. Contratos congelados encostados

**Nenhum.** `Tool=12`/`CanvasPaintTool=1`/`PanelEvent=4` intactos (tudo viaja pelos canais
genéricos); `NodeOp`/`NodeManifest` intocados.

## 5. O que só o `ship.sh` pega

- fmt: rodado com o pin sobre as 8 crates tocadas; risco residual só em arquivo NÃO tocado.
- `cargo-machete`: `ph2d-panel-wet-tuning` usa todas as deps declaradas; o shell USA
  `ph2d_panel_wet_tuning::set_current_brush` (não é dep morta).
- deny/audit: **zero dependência externa nova** (só crates internas).
- typos: prosa nova em i18n/docs — vocabulário do modelo (Kubelka-Munk etc.).

## 6. O que SMOKE-testar (nada foi smokado — tudo pendente)

`env PH2D_WETPAINT_SMOKE=1 cargo run -p ph2d-host-desktop --release` (a cena arma o wet):

1. **Seção básica:** o rádio de tools (Erase acende com o chip do rail e vice-versa — duas vistas
   de um rádio); o TILT dial (arrastar snapa e LIGA; toggle preserva direção; com Gravity>0 o
   pingo corre na direção do dial); Wet canvas (o próximo traço sangra em qualquer lugar — ligue
   Show Wet pra VER a folha úmida); Dry canvas (assenta na hora); Fast dry (seca com os anéis de
   borda); Show Wet (véu frio + brilho de menisco; **desarmar o wet NÃO pode bakar o véu**);
   Paper (o grão entra na tinta; **baka** de propósito); Tuning (abre o painel lateral).
2. **Painel Tuning:** os 40 sliders vivos (ex.: Leveling/Brake mudam o fluxo com água na tela;
   Bristle count re-textura o depósito; Contrast/Fibres/Grooves re-cozem o papel do ENGINE — e
   **somem** quando o Paper slot do artista arma); resets por-knob e por-grupo; o olhinho do
   PAPER = o checkbox Paper; K–M mixing muda a mistura de cores; Glaze muda lavagem-sobre-seco;
   fechar no X = desmarcar Tuning. **Chrome (fix do 1º smoke):** arrastar pela BARRA DE
   TÍTULO move o painel; os grippers dos 2 cantos inferiores redimensionam (mesma maquinaria
   `BlenderHit` do Inspector, deltas sob `WET_TUNING_PANEL`, clamp ao viewport); e o heading
   ENGOLE o clique — com o corpo rolado, a row que fica atrás do título não pode ser
   scrubada (a banda de drag é registrada por ÚLTIMO no paint: last-registered-wins; gates
   em `tests/panel_chrome.rs`, 2 mutações provadas — banda registrada cedo · handle fora do
   `populate`).
3. **As 5 tools novas:** Smear arrasta, Blend remistura tinta SECA, Wet molha sem pigmento, Dry
   sela, Blow empurra o filme (a sim fica viva sob o gesto do Blow). Com Symmetry ligada, Smear/
   Blow deslocam LOCALMENTE em cada cópia (o prev por-lane).
4. **O pigmento responde às tools (doc 23):** pinte com tinta pura (Water 0), Fast dry,
   e passe o **Wet** — a tinta seca DISSOLVE sob o pincel (o reporte original); com a área
   molhada, **Blow** empurra e **Smear** arrasta a cor re-suspensa; **Smear no seco**
   esfrega como smudge de raster (espalha por repetição); **Blend** sobre seco re-mistura
   (como antes) e sobre molhado re-suspende (sangra depois); **Staining** no painel Tuning
   (grupo PAINT) controla tudo — a 1.0 a tinta seca fica pinada; **Rewet lift** (grupo
   TOOLS) é a força do dissolve, 0 = modelo antigo. ⚠️ Molhar e NÃO mexer re-assenta o
   pigmento quando a água morre — é física, não bug.
5. **Zero regressão:** o modo Paint comum segue byte-idêntico (G0b verde); wet Paint padrão
   idem (boot equivalence + fingerprint com pin justificado; `wetLift=0` reproduz o pin
   antigo ao byte).

## 7. Decisões que o smoke pode reabrir (nomeadas, não escondidas)

- Métodos de stroke NÃO-incrementais (Line/shapes) com tool ≠ Paint: o preview flat mostra
  PIGMENTO e o commit aplica a TOOL (o esboço derrete na ação) — coerente com o doc 21, mas o
  preview "mente" a cor para Wet/Dry/Blow; se incomodar, a saída é restringir o Method sob tools.
- O véu do Show Wet sobre camada transparente é uma ARDÓSIA translúcida (adaptação nomeada do
  modelo, que escurece uma folha opaca); constantes em `wetpaint/composite.rs`.
- Tooltips ricos (KNOB_DOCS do modelo) ficaram fora (§4 do plano).
- A visibilidade do painel via bridge não tem gate de shell dedicado (o espelho é 3 linhas no
  `painter_bridge`; o gate de registry cobre a metade estrutural).

*Linha `Painter` pronta (8 commits). Aguardo ordem de integração.*
