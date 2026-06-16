# ADR-0097 — Brush Engine: paridade Procreate, dab pipeline CPU-first

- **Status:** PROPOSTO — aguarda ratificação do Enio (decisão fundacional, Coord-only, CLAUDE.md §4/§6).
- **Data:** 2026-06-16.
- **Decisor:** Enio (direção: "desenvolver de forma plena o Brush Tool/Brush Engine usando cada parâmetro
  do Procreate como fonte definitiva; paridade primeiro, diferencial depois").
- **Relaciona:** segue [ADR-0096](0096-remove-watercolor-fluid-pivot-mixer-brush.md) (pivô p/ mixer-brush);
  formaliza a arquitetura do modelo de dados de [ADR-0044](0044-brush-engine-gpu.md)
  (`Brush` = 12+ sub-structs Brush Studio, `Stamp=96B`, `RenderingMode=6`).
- **NÃO altera contratos congelados** aqui: `Brush≤168`/`Stamp=96B align(16)`/`RenderingMode=6`/
  `PainterParams≤12` ficam intactos (gate `architecture_painter_contract_surface`). Mudanças de superfície
  para fechar paridade vêm num **ADR-amendment único** futuro (§4).
- **Plano vivo:** [`docs/Novo Painter/`](../../Novo%20Painter/) (pesquisa + referência exaustiva + plano
  passo-a-passo W0-W11; cada feature do Procreate = uma etapa).

## 1. Contexto

Após a remoção da simulação de fluido (ADR-0096), o norte é um **brush engine raster stamp-based** com
**paridade de features do Procreate Brush Studio** (14 painéis). A pesquisa profunda (2026-06-15/16,
3 frentes: manual oficial, literatura acadêmica, formato `.brush`) e a **auditoria de cobertura do código**
(2026-06-16, W0.1) estabeleceram dois fatos fundacionais:

1. **O modelo de dados já espelha o Procreate.** O `Brush` (ADR-0044) já tem as 14 categorias como
   sub-structs; `ph2d-color::pigment_space` já tem Mixbox/Kubelka–Munk (ADR-0080/0091); `RenderingMode`
   já são os 6 modos Glaze/Blending do Procreate. **Não é greenfield** — é completar a paridade.

2. **O render vivo é CPU; o GPU está validado mas morto.** O tool chama `apply_stamps_*`
   (`cpu_render/mod.rs`) — caminho CPU-residente. O `StampPipeline` + `shader/stamp.wgsl` é naga-validado
   mas **não é despachado** pelo shell (revertido pós-ADR-0096). Params "wired" só no WGSL (tilt/azimuth/
   barrel) estão **dormentes**. Wet Mix está 100% morto (`wet_amount = 0.0` hardcoded). A matriz `[E]`
   completa está em [`03_plano_implementacao.md §W0.1`](../../Novo%20Painter/03_plano_implementacao.md#w01-resultado--matriz-de-cobertura-e-auditoria-2026-06-16).

## 2. Decisão

### 2.1 Arquitetura: o dab pipeline em 3 estágios ortogonais

A geração de um traço é uma função determinística (HR-5) decomposta em três estágios independentes — a
**ortogonalidade é o invariante de design** (cada estágio é um caminho de código separado e testável):

```
input (path + pressão/tilt/azimuth/velocidade)
   │
   ▼  ESTÁGIO 1 — GEOMETRIA  (stamp_scheduler)
   │   path → Vec<Stamp> : spacing/jitter/falloff, taper, shape (count/scatter/roundness/flip),
   │   dynamics (speed/jitter), stabilization (streamline/FFT), curvas de pressão/tilt.
   ▼  ESTÁGIO 2 — COBERTURA / COMPOSIÇÃO  (cpu_render)
   │   por-pixel: shape×grain×flow, build-up vs wash (os 6 RenderingMode), wet/burnt edges,
   │   blend mode, alpha threshold, luz-linear.   [ "quanto e como deposita" ]
   ▼  ESTÁGIO 3 — COR  (pigment_mix / color_dynamics)
       mistura de matiz: Mixbox/K-M, color dynamics, Wet Mix (reservatório pickup/deposit).
       [ "qual cor" — independente da cobertura ]
```

`Stamp` (96B POD, congelado) é a fronteira Estágio-1→2. A spec de paridade é o
[`02_referencia_parametros_procreate.md`](../../Novo%20Painter/02_referencia_parametros_procreate.md).

### 2.2 CPU-first: o caminho CPU é a fonte-da-verdade do render

O render vivo (`cpu_render`) é a **referência canônica**. Toda nova avaliação de parâmetro (Wet Mix,
roundness, grain detalhado, edges, color dynamics não-stamp, tilt/barrel) é implementada **primeiro no CPU**,
com teste headless. O GPU (`StampPipeline`/`stamp.wgsl`) é **otimização de perf futura** (brush gigante,
4K real-time), reconciliada contra o CPU por **paridade ULP** (não bit-a-bit — sem gate de paridade de
canvas vivo). Mixbox no CPU é a referência; o WGSL usa os mesmos coeficientes.

**Justificativa:** menos superfície, destrava os 7 buracos da matriz no caminho vivo imediatamente, evita
reanimar a maquinaria GPU-residente recém-removida (ADR-0096). O GPU-dispatch (parte do plano
`wise-seeking-gem`, **sem fluido**) fica como rebuild deliberado quando perf exigir.

### 2.3 Paridade primeiro, diferencial depois

A ordem é: ter **tudo o que o Procreate tem** antes de qualquer melhoramento. O diferencial — quando vier —
ancora no que já é forte: mistura de cor Mixbox/K-M espectral acima do baseline; perf de brush grande via
GPU. Nada de "melhorar" antes da paridade.

## 3. Roteiro (ondas — detalhe no plano)

W0 fundação (este ADR + scheduler consolidado + estágio de cor) → W1 Stroke Path + espinha de input →
W3 Rendering (6 modos + edges) → W2 Shape (roundness/filtering) → W4 Grain → W5 Dynamics/Stabilization →
W6 Taper → **W7 Wet Mix (maior valor)** → W8 Color Dynamics → W9 Apple Pencil/Properties/Preview →
W10 UI Brush Studio (14 seções) → W11 persistência + import `.brush`. **Fora de escopo:** Materials (3D),
Dual Brush (commodity, pós-paridade).

## 4. ABI / contratos

- **Nada muda agora.** `Brush`/`Stamp`/`RenderingMode`/`PainterParams` ficam congelados.
- A auditoria identificou **~12 params que faltam ou divergem** no `Brush` para paridade exata
  (`jitter_linear`; taper tip/classic/split; `shape_angle`; pencil per-target amounts + tilt angle/compression;
  `wet_edges`/`burnt_edges` bool→slider; `shape_rotation_follow` bool→slider; `PreviewParams`). Estes serão
  fechados num **único ADR-amendment** ("Brush param surface — paridade Procreate completa") quando a onda
  correspondente os exigir, em vez de N amendments. Há **1 slot top-level de headroom** no `Brush` (cap ≤ 14).
- `wet_amount` no `Stamp` deixa de ser `0.0` hardcoded quando W7 (Wet Mix) for implementado — é campo já
  existente do ABI congelado, sem mudança de superfície.

## 5. Alternativas rejeitadas

- **GPU-first (despachar o `StampPipeline` agora):** reanima a topologia GPU-residente removida no ADR-0096
  antes de termos paridade de features; adia o valor (Wet Mix/grain/edges) por trabalho de perf prematuro.
  Rejeitado — perf vem depois da paridade (CLAUDE.md §0.6 não obriga prematuridade de perf).
- **Reescrever o engine do zero:** descartaria o modelo de dados ADR-0044 já-Procreate-shaped, o scheduler
  determinístico vivo, e o Mixbox vivo. Rejeitado — a auditoria provou que ~40% dos params já são avaliados.
- **Mudar todos os ~12 campos de `Brush` já:** quebraria o ABI congelado antes de saber a forma final de cada
  feature. Rejeitado — amendment único, just-in-time por onda.

## 6. Decisão que pede ratificação explícita do Enio

A escolha **CPU-first (§2.2)** é a única consequente e reversível: ela define que o GPU-dispatch é trabalho
futuro de perf, não agora. Recomendação forte = **CPU-first**. Se o Enio preferir GPU-dispatch já (brush
gigante/4K como requisito imediato), o roteiro reordena (W0 inclui despachar o `StampPipeline`). Tudo o mais
(arquitetura de 3 estágios, paridade-primeiro, amendment único) é consenso técnico e não precisa de fork.
