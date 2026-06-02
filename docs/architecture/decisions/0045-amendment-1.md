# ADR-0045 Amendment 1 — AdjustmentLayer crate-placement reconciliation

**Status:** Accepted · 2026-06-02 · Coordenador
**Amends:** [0045-adjustment-layers.md](0045-adjustment-layers.md) §2.1–2.2

## Contexto

Ao landar T4.1 (materializar a superfície de contrato `adjustments`) o Coord
encontrou 3 conflitos entre o ADR-0045 escrito e a árvore real. O ADR fixou os
caps/variants corretamente, mas §2.1–2.2 deixaram detalhes de **placement de
tipo** que, como escritos, não compilam. Este amendment os reconcilia sem mudar
nenhum cap, variant, ou semântica de adjustment.

## Conflitos

1. **Ciclo de dependência (`LayerId`).** O gate `adjustment_layer_field_count_is_capped`
   exige `AdjustmentLayer` em **`ph2d-painter-brush`** (§2.1). Mas §2.2 deu a ele
   `id: LayerId` + `clipped_by: Option<LayerId>`, e `LayerId` mora em
   **`ph2d-tool-painter`**, que **depende de** `ph2d-painter-brush`. Referência
   impossível (ciclo de crate).
2. **`MaskData` é vapor.** §2.2 listou `mask: Option<MaskData>`; `MaskData` não
   existe. O `LayerStack` real mascara via `Option<LayerId>` (mask = um layer-id).
3. **Duplicação `AdjustmentLayer` ⟷ `Layer`.** §2.2 congelou `AdjustmentLayer`
   como layer completa (id/name/opacity/blend_mode/visible/locked/…). T4.2 a embute
   como `LayerKind::Adjustment(AdjustmentLayer)`, duplicando os metadados do
   `Layer` externo do `LayerStack`.

## Decisões (não mudam cap nem variant; gates §2.10 inalterados)

1. **`AdjustmentLayer.{id, clipped_by, mask}` usam `u64` cru** (valor-LayerId cru),
   não o tipo `LayerId`. `LayerId` é newtype `(u64)`; a conversão na fronteira do
   `LayerStack` é `LayerId(x)` / `x.0`. Isto mata o ciclo SEM mover `LayerId` nem
   tocar `ph2d-tool-painter/layers.rs` (crate quente do impl — evita colisão).
   - `id: u64`, `clipped_by: Option<u64>`, `mask: Option<u64>`.
   - Contagem de fields idêntica → gate `adjustment_layer_field_count_is_capped` (≤12) inalterado.
2. **`mask: Option<u64>`** (ref de mask-layer-id, padrão atual do `LayerStack`).
   `MaskData` removido do contrato.
3. **Integração `LayerKind::Adjustment(AdjustmentLayer)`, campos internos
   autoritativos.** Para uma layer de kind=Adjustment, os campos de
   `AdjustmentLayer` (opacity/blend_mode/visible/locked/mask) são a fonte de
   verdade; os campos homônimos do `Layer` externo são espelhados/ignorados. O
   `LayerStack` documenta isso no ponto de criação. (Alternativa "slim
   AdjustmentLayer" foi rejeitada: quebraria o gate que exige o struct completo.)

## Consequências

- T4.1 vira landing mecânico contra estes shapes (zero decisão em runtime).
- Zero churn em `ph2d-tool-painter/layers.rs` para T4.1 (só T4.2 toca, via
  `LayerKind::Adjustment` aditivo — coordenado com a janela do impl).
- Trade-off aceito: `id`/`clipped_by`/`mask` perdem a tipagem `LayerId` dentro de
  `AdjustmentLayer` (são `u64` crus). Ganho: sem ciclo, sem colisão, sem mover um
  tipo foundational da crate quente do impl. Conversão trivial na fronteira.
