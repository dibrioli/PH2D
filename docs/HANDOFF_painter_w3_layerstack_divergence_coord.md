═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · DIVERGÊNCIA de modelo LayerStack/LayerId (W3) — decisão tua
Autor: Implementador Painter (sessão 2026-05-31) · descoberto ao iniciar Bloco 1
Regras: DIRETRIZ §4 (contrato congelado = Coord+ADR) · CLAUDE.md §6 (ph2d-painter-stroke persistência)
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ Existem DOIS modelos de layer. O meu (T3.1, runtime) e o canon de   ║
║ persistência (`device::LayerStack`, savefile CONGELADO ADR-0046).   ║
║ O doc do stub canon diz que É ELE que W3 preenche. Reconciliar toca ║
║ o contrato congelado (teu/ADR) E o compositor GPU que você acabou   ║
║ de construir sobre o MEU `LayerId(u64)`. PAREI antes de aprofundar. ║
║ Recomendo Opção A (runtime=meu, persistência serializa). Ratifique. ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
OS DOIS MODELOS
───────────────────────────────────────────────────────────────────
EXISTENTE (canon, congelado):
- `ph2d_painter_stroke::device::LayerId(pub u32)` — id canônico usado em
  stroke records, journal, undo (`undo.rs` usa `ph2d_painter_stroke::LayerId`),
  e no `PainterTool::layer_target` / `set_layer_target`.
- `ph2d_painter_stroke::device::LayerStack { layers: Vec<LayerStackEntry> }`
  (bottom-up) + `LayerStackEntry::Reserved(bytes)` (stub).
- **Está no SAVEFILE congelado:** `persistence.rs:98 PaintProject.layer_stack:
  LayerStack` (validado, MAX_LAYERS, reserved-bytes) — ADR-0046 §2.7.1.
- **O doc do stub (`device.rs`) DIZ explicitamente:** *"Stub W3: ... W3
  (ADR-0045 + 02_layers.md spec) preenche com `enum LayerKind { Raster, Group,
  Mask, ClippingMask, Reference, AlphaLocked, Adjustment }`"* — ou seja, o lar
  pretendido do modelo W3 é ESTE entry.

MEU T3.1 (runtime, commit `33fb4eb`):
- `ph2d_tool_painter::layers::{LayerStack, LayerId(pub u64), LayerKind,
  Layer, RasterLayer, MaskLayer, GroupLayer}` — modelo completo (blend/
  opacity/visible/grupo/cap-8/cap-999) que o compositor CPU (`1bed40e`),
  o **compositor GPU do Coord (`6ba3ed7`, `key: u64`)** e o scaffold do
  painel (`efe59b9`) já consomem.

**Meu erro:** criei o modelo paralelo sem fazer o grep de estado interno
(`device::LayerStack` já existia). Reconhecido. Não está perdido (vira o
modelo runtime sob a Opção A) — mas os NOMES iguais (dois `LayerStack`, dois
`LayerId`) são um smell, e já coexistem no `PainterTool` (`layer_target:
device::LayerId(u32)` vs o `layers: my::LayerStack` que eu IA adicionar).

───────────────────────────────────────────────────────────────────
RECOMENDAÇÃO — Opção A (mínimo rework; preserva teu Bloco 2)
───────────────────────────────────────────────────────────────────
- `ph2d_tool_painter::layers::LayerStack` = modelo **RUNTIME canônico** (o que
  CPU compositor + teu GPU compositor + painel já usam). Fonte de verdade em
  memória.
- VOCÊ (via o ADR de persistência que já reivindicou — ADR-0046/0045
  amendment): preenche `LayerStackEntry` (hoje `Reserved(bytes)`) para
  **serializar** o modelo runtime: por layer { id, name, kind, blend_mode (u8
  via `BlendMode::to_u8`), opacity, visible, locked, alpha_locked, clipping,
  is_reference, mask }. É a ponte (de)serialização no crate de persistência —
  NÃO muda o runtime nem o GPU.
- **Id width:** runtime `LayerId(u64)` ↔ savefile. O savefile hoje implica
  u32 (`device::LayerId`). Decida: (i) bump o savefile pra u64 (ADR), ou
  (ii) o serializer mapeia u64→u32 (counter < 2³² sempre cabe; meu `next_id`
  começa em 1). Teu GPU usa `key: u64` então u32→u64 widening é trivial lá.
- Resultado: TODO o W3 (fundação + GPU + scaffold + o render read-only do
  painel que estou fazendo agora) fica intacto; só ganha a ponte de persistência.

Opção B (unificar em `device`, `LayerId(u32)`, deletar o meu): quebra teu
compositor GPU (u64) + o meu + o painel, e edita o crate congelado de forma
mais profunda. Não recomendo.

Decisão cosmética (tua): renomear o meu `LayerStack`/`LayerId` pra algo tipo
`RuntimeLayerStack`/`RtLayerId` pra matar o shadowing dos nomes do `device`?
Eu topo fazer no meu crate se você quiser (não toca congelado). Caso contrário
documento a coexistência (runtime vs savefile) e seguimos.

───────────────────────────────────────────────────────────────────
ONDE PAREI / O QUE SIGO FAZENDO (não-bloqueado)
───────────────────────────────────────────────────────────────────
- NÃO fiei `layers` no `PainterTool` (evitei aprofundar a divergência), NÃO
  toquei o contrato congelado.
- Estou fazendo o **render read-only do painel de layers (T3.4 1ª passada)** —
  lê o snapshot do modelo runtime; independe desta decisão.
- BLOQUEADO até tua ratificação: a integração tool↔LayerStack interna (campo
  `layers` na tool + composite-no-preview + edição via handle_panel_event),
  porque a forma final do `LayerId`/serialização depende da tua escolha A/B.

Responde com A (+ id-width i/ii + rename sim/não) e eu sigo a integração interna.
═══════════════════════════════════════════════════════════════════
