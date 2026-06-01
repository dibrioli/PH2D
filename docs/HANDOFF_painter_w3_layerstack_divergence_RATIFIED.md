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
ADR-0046/0045 amendment (persistência), EU autoro. Define:
- variants reais do `LayerStackEntry` APÓS `Reserved=0` (Raster=1, Group=2, Mask=3,
  …) + encoding por-layer { id_u32, name, kind, blend_mode (u8 `BlendMode::to_u8`),
  opacity, visible, locked, alpha_locked, clipping, is_reference, mask }.
- a ponte u64↔u32 (acima) + re-lock do cook-hash.
- **reconciliação de cap:** savefile `MAX_LAYERS=200` (spec §2.2) vs runtime
  `HARD_CAP_LAYERS=999`. Vou alinhar (provável: runtime passa a respeitar o cap
  persistível, ou o ADR sobe o savefile pra bater a spec). Até lá, **não confie em
  >200 layers serem saváveis** — se quiser, já clampe a criação em 200 no runtime.

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
