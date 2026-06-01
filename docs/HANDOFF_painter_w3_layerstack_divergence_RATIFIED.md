═══════════════════════════════════════════════════════════════════
RATIFICAÇÃO (Coord) → divergência LayerStack/LayerId W3
Responde a: HANDOFF_painter_w3_layerstack_divergence_coord.md
Autor: Coordenador (2026-05-31)
═══════════════════════════════════════════════════════════════════

VEREDITO: **Opção A ratificada. Id-width (ii). Rename NÃO-forçado.** Você está
DESBLOQUEADO pra integração in-memory agora. Meu Block 2 GPU fica intacto.

───────────────────────────────────────────────────────────────────
1. OPÇÃO A — RATIFICADA
───────────────────────────────────────────────────────────────────
`ph2d_tool_painter::layers::LayerStack` = modelo **RUNTIME canônico** (fonte de
verdade em memória). É o que o compositor CPU, o compositor GPU (`6ba3ed7`) e o
painel já consomem. Não muda.

Confirmei no código que isto é o caminho PROJETADO, não uma quebra:
`device::LayerStackEntry::Reserved(Vec<u8>) = 0` tem doc explícito *"Add novos
variants APÓS Reserved: Raster=1, Group=2, ..."* — o stub foi desenhado pra W3
preencher, com forward-compat (files v1 carregam `Reserved` → migram). Opção B
(unificar em u32, deletar o runtime) está REJEITADA: quebraria 3 peças que já
funcionam + editaria o congelado mais fundo.

───────────────────────────────────────────────────────────────────
2. ID-WIDTH — (ii) runtime u64, savefile/strokes ficam u32
───────────────────────────────────────────────────────────────────
Runtime `LayerId(u64)` NÃO muda (teu T3.1 + meu GPU `key:u64` ficam como estão).
A ponte de largura vive só na fronteira de persistência/stroke:
- **Save / stroke-record:** narrow `device::LayerId(active.0 as u32)`. Defensivo:
  assert `id <= u32::MAX` no save (inalcançável — cap operacional ≤200, `next_id`
  começa em 1 — mas erro explícito > UB silencioso).
- **Load:** widen u32→u64; reconstrua o stack e **`next_id = max(ids)+1`** (preserva
  o invariante "ids nunca reusados" entre sessões).
- Razão p/ (ii) e não (i): bumpar `device::LayerId`/stroke records pra u64 mexeria
  em campos congelados MAIS fundo (migração + re-lock cook-hash dos stroke records).
  (ii) mantém os campos u32 existentes intactos; só DEFINE o payload do
  `Reserved(bytes)` (que já era reservado pra isso).

───────────────────────────────────────────────────────────────────
3. RENAME — não force (type system já protege)
───────────────────────────────────────────────────────────────────
Os dois tipos são DISTINTOS — o compilador recusa passar um pelo outro, então não
há risco de confusão silenciosa. Qualifique por módulo no `PainterTool`
(`device::LayerId` p/ `layer_target` vs `layers::LayerStack` p/ `layers`) e
documente a coexistência (runtime vs savefile). **Pré-aprovo** você renomear os
tipos RUNTIME dentro do TEU crate (ex.: deixe `device::*` como está, renomeie o
teu) SE a integração mostrar fricção real — é contido, não toca meu Block 2 (usa
`key:u64` cru, não o tipo `LayerId`) nem o congelado. Não bloqueie nisso.

───────────────────────────────────────────────────────────────────
4. O QUE É MEU (Coord) — não te bloqueia
───────────────────────────────────────────────────────────────────
**FEITO ✅ (Coord):** ADR-0046-amendment-1 + o formato congelado v2 já estão
implementados + gateados (crate `ph2d-painter-stroke`, `SCHEMA_VERSION=2`). Você
NÃO define formato — só escreve a **ponte** runtime↔savefile no TEU crate:

- Tipos prontos (re-exportados em `ph2d_painter_stroke`): `LayerStackEntry::Node(LayerNode)`,
  `LayerNode { id: LayerId(u32), name, kind: LayerNodeKind, blend_mode: u8, opacity,
  modifiers: u8, mask: Option<Box<LayerNode>> }`, `LayerNodeKind::{Raster{w,h},
  Mask{w,h,inverted},Group{children,collapsed}}`, e os flags `LAYER_FLAG_{VISIBLE,
  LOCKED,ALPHA_LOCKED,CLIPPING,IS_REFERENCE,ACTIVE}`.
- **Contrato da ponte (SAVE):** `device::LayerStack.layers` = teus root em **z-order
  top-first** (índice 0 = topo); cada `Node` = uma `layers::Layer` (id `as u32`,
  bools → bits `modifiers`, `blend_mode.to_u8()`, mask `Option<LayerId>` → resolve a
  layer e aninha como `Box<LayerNode>`); grupos aninham filhos recursivamente; a layer
  ativa recebe `LAYER_FLAG_ACTIVE`. `next_id` NÃO serializa.
- **Contrato da ponte (LOAD):** reconstrói arena/root da árvore; widen id `u32→u64`;
  `next_id = max(id)+1`; a layer com `LAYER_FLAG_ACTIVE` vira `active`.
- Caps que o `load` JÁ valida por você (rejeita file forjado): profundidade de grupo
  ≤8, nome ≤256 B, total de nodes ≤999. `migrate_v1_to_v2` já cria 1 raster default.
- cook-hash re-locka sozinho no `save`. Detalhe completo: ADR-0046-amendment-1.

**Cap: NÃO há conflito (era comentário stale).** Spec §2.2 (linha 145) fixa
`HARD_CAP_LAYERS = 999` (espelha Procreate). Runtime = 999 ✓, savefile
`MAX_LAYERS = 1000` (999 + overflow) ✓, meu Block 2 = 999 ✓ — tudo já alinhado.
O "200" que eu (Coord) tinha flagado era comentário errado em `device.rs:43` +
`persistence.rs:62` citando mal a spec; corrigi ambos (comment-only, sem impacto
de wire/ABI/cook-hash). **Cap fica 999 — pode criar até 999 layers saváveis.**

───────────────────────────────────────────────────────────────────
5. VOCÊ SEGUE AGORA (desbloqueado)
───────────────────────────────────────────────────────────────────
A integração in-memory NÃO depende da persistência: o runtime model já é canônico
as-is. Toque:
- campo `layers: layers::LayerStack` + `images: BTreeMap<LayerId, LayerImage>` no
  `PainterTool` (substitui `canvas_rgba`); `set_source` = stack N=1.
- strokes → layer ATIVA (`layers.active()`); no stroke-record, `layer_target =
  device::LayerId(active.0 as u32)` (regra ii).
- `current_preview()` = composite (CPU ref; o GPU é o caminho real-time via o meu
  Block 2 — API no HANDOFF_painter_w3_block2_done.md).
- edição via `handle_panel_event`.
A persistência (save/load do stack) é follow-up sobre o meu ADR — o caminho
in-memory funciona sem ela.
═══════════════════════════════════════════════════════════════════
