# Continuação — line/Macros-Automação, W1 (núcleo) fechada → abrir W2 (trilha do Painter)

> Para a PRÓXIMA janela desta linha. Branch `line/Macros-Automação`, worktree
> `Worktrees/line-Macros-Automação/`, base `757afebd58`. Docs do módulo em `docs/Macros/` (o nome da pasta
> é `Macros`, não o da linha). Leia antes de agir: [plano da onda 1](../01_plano_onda1.md) (o desenho, as
> portas únicas, os gates e os números) e o [estudo](../00_estudo_inicial.md) §4 (decisões do dono).

## Estado

| commit | o quê |
|---|---|
| `fbbc084970` · `b5efbfdcb1` | estudo + decisões do dono |
| `84523b5c6e` | sondas `#[ignore]`: `crates/ph2d-tool-painter/src/tool/paint/wetpaint/sonda_macro_tests.rs` (Wet Paint) · `crates/ph2d-app-motion/src/motion_bridge_macro_replay_probe.rs` (Motion) |
| `adaf9d5bf4` | plano da onda 1 |
| `7711955529` | **W1**: crate `ph2d-macro` + ADR-0186 (número provisório; `0185` era o máximo) |

**W1 — `crates/ph2d-macro`** (zero dependências, sem relógio de parede):
`MacroDoc { reference: RefRect, tracks: Vec<Track{ family, target: u64 (StableId; 0 = nenhum), initial, events: Vec<Event{ t: MacroTime(µs), payload }> }> }` ·
`codec::{write, read}` (`PH2DMACRO` + major 1/minor 0, campos etiquetados; desconhecido salta, major ≠ recusa) ·
`MacroRecorder::{new(reference, armed_at_us), open_track, push(track, now_us, payload)}` (recusa tempo para trás) ·
`MacroPlayer::{advance_to, seek, is_finished, dispatched}` (ordem global; empate → trilha menor; falha não salta) ·
`Door/Recordability` + `undecided`/`orphans` (censo de gravabilidade).
Prova: 19 testes em `tests/it/`; **mutação 9/9 sangraram** (formato ×3 níveis, major, ordem, retry, recorder, relógio de
parede, escritor congelado, seek, monotonia). Portões verdes no fecho da W1: `ph2d-editor-core` (2209), `ph2d-app-registry-init`,
clippy `-D warnings` de `ph2d-macro`/`ph2d-tool-painter`/`ph2d-app-motion` (`--all-targets`), `cargo machete`.

**Não feito de propósito na W1:** portões de tutorial (`StepGate`) — entram na W5 como campo NOVO (minor 1.1:
o leitor tolerante já os aguenta); o registo das famílias (`TrackKind`) — é composição da W4.

## W2 — a trilha do Painter (o que fazer)

1. **Carga** (em `ph2d-tool-painter`, módulo irmão novo, p.ex. `src/tool/paint/macro_track/`): enum de passos
   com codec próprio por dentro da `payload` — `Pointer{pos normalizada pelo RefRect, pressure, phase}` ·
   `Tick{dt}` · `Panel(PanelEvent resolvido)` · `LayerEdit` · `Undo`/`Redo` · setters diretos ·
   `StrokeBegin{seed}` · `WetMark{frame}`. Estado inicial = preset/meio/cor/papel/tamanho do canvas/semente.
2. **Gravação nas portas** (fronteira medida, várias portas): `on_canvas_pointer`
   (`src/tool/paint/canvas_pointer.rs:11`) · `Tool::on_tick` (`src/tool/trait_impls.rs:383`) ·
   `handle_panel_event` (`trait_impls.rs:50`) · `apply_layer_edit` (`layer_edit.rs:237`) ·
   `undo_last/redo_last` (`src/tool/runtime.rs:48,94`) · `set_line_constrain`/`set_canvas_warp`
   (`canvas_pointer.rs:120,135`) · `set_wet_*`. Os setters de preset/meio/cor/tamanho **não foram
   enumerados** — enumerar e declarar cada um no censo (`Door`).
3. **Lei da água** (vale SEMPRE): no 1.º lote molhado de um traço (`!sess.stroke_open`,
   `src/tool/paint/wetpaint/dab_route.rs:73`, antes de `begin_direct_stroke` em `:200-215`):
   `bring_home` → `engine.drain_step()` (`crates/ph2d-wet-paint/src/painter/stage.rs:13`) → gravar
   `engine.sim.frame` (`crates/ph2d-wet-paint/src/sim.rs:77`). ⛔ NÃO `sim_steps()` (não vê passos fechados
   por outra porta). Reprodução: relógio fixo desligado (a sonda usa `relogio_fixo = Some(NaN)`) e
   `wet_step_sync` (`offthread.rs:430`) até `sim.frame` == marca; no fim, drenar e conferir o frame final.
4. **Semente**: `paint.seed` (`state.rs:47`, avança em `stroke_lifecycle.rs:186` e nas formas) gravada no
   `StrokeBegin`; ela NÃO é persistida no projeto hoje.
5. **Tocador headless** em `ph2d-app-painter`: um `PainterTool` PRÓPRIO por alvo (não a ferramenta ativa do
   editor; molde `tests.rs:258 white_canvas`), alimentado pelo `MacroPlayer`. Ligação à cena (sprite) é W4.
6. **Gates** (plano §5): 1 (água ao byte, fixture worker com esperas reais — vermelho primeiro; a sonda já tem o
   roteiro e os números: 24 239 / 29 891 texels → 0), 2 (semente + `dt` irregular do Airbrush), 4 (censo das portas do
   Painter), 6 (resolução 2×). Cada um com controlo positivo.
7. **Composite uma vez por quadro** para 2048²+ (medido `29,7 ms` por passo com composite vs `8,08` só o passo):
   medir antes/depois no MESMO processo (`docs/DevOps/MEDIR_VELOCIDADE.md`), sem tecto inventado.

## Cuidados medidos
- As sondas mediram sob carga (loadavg 11–46): use os mínimos.
- A rota `sim_while_painting`/`agua_viva` (água a correr durante o traço) muda a intercalação dab×passo — a marca
  por traço não chega aí; decidir na W2 (gravar marca por LOTE quando `agua_viva`, ou recusar com o porquê).
- `apply_layer_edit`/undo do Painter gravam pixels: o tocador reproduz pela PORTA, nunca copiando pixels.
