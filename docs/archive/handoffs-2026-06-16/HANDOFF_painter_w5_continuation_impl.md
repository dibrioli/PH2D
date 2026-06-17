# HANDOFF — Painter W5 continuation (novo implementador, START HERE)

> Você é o **Implementador Painter**. Leia o `CLAUDE.md` (raiz) e este doc antes de agir.
> Esta é a entrada; o plano detalhado do que falta está em
> [`HANDOFF_painter_w5_brush_studio_impl.md`](HANDOFF_painter_w5_brush_studio_impl.md).

## §0 — Regras inegociáveis (do CLAUDE.md)
- **Isolamento:** edite só as SUAS crates (`ph2d-painter-brush`, `ph2d-tool-painter`,
  `ph2d-panel-painter-sidebar`, `ph2d-panel-painter-layers`, e a futura `ph2d-panel-brush-studio`;
  no shell, só o `painter_bridge`). Precisou de algo foundational (ex: `ph2d-color`, `ph2d-editor-core`
  ids/widgets, contract gates) → **edite com cuidado e reporte**; se outro agente estiver mexendo, PARE.
- **Git anti-colisão:** `git add -- <seus paths>` (NUNCA `-A`/`.`/`stash`); `git commit --no-verify
  -m "msg" -- <paths>`; **`git status` antes de stage**; se houver `M`/`??` alheio nos seus paths, não
  comite — reporte. **Você NÃO pusha** (o Coordenador faz ship+push 1× por jornada).
- **Velocidade:** inner loop = **só `cargo check/test -p <crate>`**. Slot warm:
  `CARGO_TARGET_DIR=/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/target-slots/slot-1` (prefixe
  TODO cargo). Teste/clippy 1× no fim. ≤3 cargos simultâneos (RAM 8GB).
- **UI canônica:** zero hex/`f32` literal de UI/string hardcoded — tokens + i18n (HR-15). **Labels
  de UI SEMPRE em inglês** mesmo o Enio descrevendo em pt-BR.
- **Decida no padrão-ouro, não pergunte** (o Enio confia que você sabe mais de código). Gaps in-scope
  fecham na sessão. A melhor opção técnica vence cronograma/custo de build.

## §1 — Diretriz de UI do Enio (CRÍTICA, 2026-06-06)
**A fonte da verdade da UI é o `ph2d-panel-widget-gallery` (canon de TODOS os widgets) + o
`ph2d-panel-inspector` (padrão de painel-com-seções).** Ao construir qualquer UI nova (Brush Studio),
**espelhe esses dois** — NÃO improvise chrome/controles. O Inspector tem o molde exato: painel com
seções de params, scroll, row-builders (`check_row`/`number_row`/slider rows), `paint_section_separator`.

## §2 — Estado da W5 (o que JÁ foi feito esta sessão)
Pigmento + grão = **as 2 inovações de render da W5, completas**. Falta o **Brush Studio** (painel editor).

**Pigmento (mistura subtrativa K-M):** `crates/ph2d-painter-brush/src/pigment_mix.rs` (era `mixbox.rs`).
Engine **7-curvas** (reconstrução de reflectância White+CMY+RGB tipo spectral.js, base derivada
clean-room, otimizada contra alvos artísticos públicos) + Kubelka-Munk + re-anchor. **CPU é o path AO
VIVO** (`cpu_render/mod.rs` — `mixbox_lerp`→`pigment_lerp`, `mix_prepared`); `shader/stamp.wgsl` é
paridade (NÃO é o path vivo). azul+amarelo=#468847 verde, azul+vermelho=#600079 violeta. Commits:
`1970740` (engine), `417a676` (rename Mixbox→Subtractive/Pigment — higiene de marca, o nome "Mixbox" é
produto CC-BY-NC da scrtwpns; nosso código é clean-room).

**Modelo wash vs build-up (ortogonal ao pigmento):** `RenderingParams.accumulate` (bool). `false`=wash
(opacidade limita o stroke, `apply_stamps_wash`), `true`=build-up (acumula). Decoupled em
`lifecycle.rs::begin_stroke` (`wash = !accumulate`). Commit `4edfc9a`.

**Grão procedural:** `crates/ph2d-painter-brush/src/grain_noise.rs` — 4 geradores (Simplex/Gabor/
PaperWeave/SprayDot), hash→Perlin→fbm, determinístico (HR-5), retorna multiplicador de cobertura
[0,1]. Codificado no Stamp ABI (tipo+depth em `grain_layer`, escala em `grain_scale`,
`FLAG_GRAIN_PROCEDURAL`). Scheduler (`stamp_scheduler/advance.rs`) baka do brush; `cpu_render` modula
`shape_alpha` (build-up + wash) na posição mundo (Texturized) ou uv (Moving). Paridade WGSL em
`stamp.wgsl` (naga-validada). Commits: `de43c00` (CPU), `5c6992d` (WGSL).

**Controles UI (na sidebar):** 3 toggles empilhados um-por-linha — **Pigment** (checkbox), **Accumulate**
(checkbox), **Grain** (botão que CICLA Off→Simplex→Gabor→Weave→Spray) + slider **Grain Depth** (aparece
só com grão on). Commits: `4ac54cc` (cycler), `f11b54b` (depth slider). Wiring: `PainterUiEdit` +
handler em `lifecycle.rs::apply_ui_edit` + rota em `trait_impls.rs::handle_panel_event` + sidebar
`paint.rs`/`populate.rs`/`event.rs`/`ids.rs`.

## §3 — Gotchas duráveis desta sessão (LEIA antes de mexer)
1. **CPU é o path ao vivo, NÃO o WGSL.** O stamp shader (`stamp.wgsl`) existe mas a pintura ao vivo
   roda em `cpu_render`. Toda mudança visual vai PRIMEIRO no CPU; o WGSL é paridade (sem gate
   automático CPU↔GPU). Não caia na armadilha de "consertei o shader" e o Enio não vê nada.
2. **`PainterUiSnapshot` está no CAP de 18 campos** (gate `painter_ui_snapshot_field_count_is_capped`).
   Não dá pra adicionar campo. Sliders de param novos = **store-driven** (sem campo no snapshot — o
   valor vive no `WidgetStore`, seeded no `populate`, lido via `store.slider(id)`). **Provado** com o
   Grain Depth slider. Por isso o Brush Studio completo é painel separado (snapshot próprio).
3. **`PainterUiEdit` cap ≤24** (hoje em 20). **`RenderingParams` cap ≤14** (hoje 11). **`PigmentMode`
   cap ≤4** (hoje 2: Linear/Subtractive — `=1` ABI freeze). Gate:
   `architecture_painter_contract_surface`. Adicionar variante DENTRO do cap é OK (drop-in ADR-0043).
4. **Stamp ABI 96B congelado** (`stamp.rs`, 17 `offset_of!` asserts + naga). Grão foi codificado nos
   slots existentes (`grain_layer`/`grain_scale`/`grain_offset_uv`/flags) — NÃO adicione campo.
5. **Checkbox despacha `WidgetEvent::Toggled`; Button despacha `Click`.** O Grain virou Button (cicla)
   → roteia Click; Pigment/Accumulate são Checkbox → Toggled. Veja `sidebar/event.rs`.
6. **Slider value display:** size/opacity usam snapshot (`size01`/`opacity01`); grão usa store (cap).
   Padrão store-driven: `register_slider_chip_pair` no populate (seed constante) +
   `link_slider_number_mapped_integer` + ler `store.slider(id)` no paint.
7. **Adicionar painel** = crate + `cargo run -p ph2d-panel-sync` (regenera registry markers, tem gate
   de staleness) + feature no `shells/desktop/Cargo.toml` (hand-edit) + NODE_ID em
   `editor-core/ids/chrome.rs`. Workspace é glob `crates/*` (auto-membro).

## §4 — PRÓXIMA TAREFA: Brush Studio (fecha a W5)
Plano completo + 7 passos do scaffold em
[`HANDOFF_painter_w5_brush_studio_impl.md`](HANDOFF_painter_w5_brush_studio_impl.md). Resumo:
- **Painel separado** `ph2d-panel-brush-studio` (snapshot próprio rico, sem cap), reusando a geometria
  do right-dock da sidebar (`ctx.layout.painter_sidebar`). NODE_ID `PAINTER_BRUSH_STUDIO_PANEL` já
  existe em `chrome.rs`.
- **Template = Inspector** (seções + scroll + row-builders). **Widgets = widget-gallery canon.**
- Seções: **Stroke Path** (spacing/jitter/streamline), **Shape** (count/scatter/rotation/flip),
  **Rendering** (pigment/accumulate/grain tipo+scale+depth + rendering_mode). Os controles de grão/
  pigmento hoje na sidebar migram/espelham aqui.
- Abrir via `PainterUiEdit::OpenBrushStudio` (variante existe, hoje no-op) + flag `show_brush_studio`
  no `PainterTool`; botão no sidebar; X no painel. Shell `painter_bridge` controla visibilidade +
  publica o snapshot.
- **Live preview:** MVP = o canvas (pinta e vê); ou re-render de um stroke num buffer pequeno via
  `cpu_render::apply_stamps`.
- **Marco 1 (faça primeiro):** crate compila + registra + APARECE (placeholder) quando aberto.
  Valide com `cargo check -p ph2d-host-desktop`. Crate nova precisa compilar INTEIRA de uma vez
  (senão quebra o build do workspace) — crie todos os arquivos antes de validar.

## §5 — Build / test / commit
```
SLOT=/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/target-slots/slot-1
CARGO_TARGET_DIR=$SLOT cargo test -p ph2d-painter-brush --lib          # motor
CARGO_TARGET_DIR=$SLOT cargo test -p ph2d-tool-painter --lib           # tool + edits
CARGO_TARGET_DIR=$SLOT cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface  # gates
CARGO_TARGET_DIR=$SLOT cargo check -p ph2d-host-desktop                # app inteiro compila
CARGO_TARGET_DIR=$SLOT cargo clippy -p <crate>                         # 1× no fim
```
Estado atual: **284 brush + 184 tool + 81 contract verdes**, host-desktop compila, clippy limpo,
workspace verde. Smoke: `./play.command` (double-click) → pinta, testa toggles/cycler/depth.
Commit scoped + `--no-verify`, sem push. Reporte ao Coordenador/Enio o commit local pronto.
