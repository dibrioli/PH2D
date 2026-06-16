# ADR-0096 — Remoção da simulação de aquarela/fluido; pivô para mixer-brush + Mixbox

- **Status:** ACEITO (Enio, 2026-06-15).
- **Data:** 2026-06-15.
- **Supersede:** toda a linha de aquarela como **simulação de fluido** —
  [ADR-0085](0085-watercolor-v2-gpu-first.md) (GPU-first watercolor v2),
  [ADR-0086](0086-watercolor-minimal-core-wash.md)/[0087](0087-wash-integration-parallel-watercolor-mode.md)
  (núcleo Wash + integração), [ADR-0090](0090-wash-undo-event-driven.md)/[0091](0091-wash-mixbox-residual-faithful-pigment-color.md)
  (undo/cor do Wash), [ADR-0092](0092-wash-capillary-fringe-realistic-deposition-edge.md) (capilar),
  [ADR-0093](0093-gpu-resident-painter-canvas.md)/[0094](0094-wash-gpu-resident-simplified-core.md)
  (canvas GPU-residente + Wash GPU-residente) e [ADR-0095](0095-wash-curtis-gd-deposition-topology.md)
  (topologia Curtis g/d). **MANTÉM como histórico** (não apaga os ADRs; ficam como registro da pesquisa).
- **NÃO afeta:** sistema de layers + efeitos/adjustments + blend modes GPU (ADR-0043..0053/W3-W4),
  o motor de pincel CPU (`apply_stamps*`), Mixbox (a TÉCNICA de cor, ADR-0091 — ver §4).

## 1. Contexto

Três reconstruções sucessivas do modo de aquarela como **simulação de fluido** (shallow-water Curtis/MoXi
na GPU) foram mutuamente irreprodutíveis. A investigação empírica (2026-06-15, `tests/wash_investigation.rs`)
provou que: (a) a diluição-por-água **nunca existiu** no design original (cobertura `1−exp(−massa/0.6)`
não lê o campo de água em nenhuma versão); (b) o espalhamento shallow-water não funciona na prática (a água
satura em 1.0 ⇒ ∇água=0 ⇒ escoamento inerte). A mistura que **funciona** é a do composite mixer-brush
(soma de cobertura + Mixbox), não a da física.

Pesquisa de sistemas de produto (Procreate, Photoshop Mixer Brush, Krita Color Smudge) mostrou que o
caminho **pragmático, previsível, reproduzível e com literatura sólida** (Baxter "DAB"/"IMPaSTo" 2004;
Sochorová & Jamriška "Mixbox" 2021) é o **mixer-brush** (carimbo + mistura subtrativa local), NÃO a
simulação de fluido (que é o caminho Rebelle/Fresco Live — outra classe de produto).

## 2. Decisão

1. **Remover toda a simulação de fluido/aquarela** do código (mantendo os backups
   `backups/wash_2026-06-14/` e `backups/watercolor_v2_2026-06-12/` intactos):
   - crate `ph2d-painter-wash` (solver shallow-water + shaders + km GPU);
   - sessões GPU do shell `painter_wash_gpu.rs` (solver) e `painter_canvas_gpu.rs` (canvas GPU-residente);
   - `WashPipeline`/`wash.wgsl` em `ph2d-painter-brush` (pincel default na GPU);
   - sliders de fluido (Diffusion/Flow/Evaporation/Water/Load), `WashUiParams`, modo Wash (tecla **W**);
   - o **edge-darkening / bordas molhadas no pen-up** (`apply_wash_settle` / `cpu_render/settle.rs`),
     por decisão explícita do Enio (slate limpo).
2. **Reverter ao canvas CPU-residente.** O pincel default volta a renderizar por `apply_stamps_wash`
   (CPU, cobertura-cap + Mixbox) — a topologia provada. A maquinaria GPU-residente do tool
   (`gpu_stamps`/`gpu_resident_stroke`/`reset_gen`/backdrop/readback) sai junto.
3. **Preservar** o sistema de layers + efeitos GPU + o motor de pincel CPU (`apply_stamps`,
   `apply_stamps_buildup`, `apply_stamps_wash`, `RenderingMode`, paper-tooth/grain) e o compositor GPU de
   preview de efeitos (`painter_gpu_preview`).

## 3. ABI / contratos

- Campos removidos do `PainterTool` (`gpu_*`, `wash_mode`, `wash_resident_stroke`) são **runtime, não
  serializados** — remoção sem impacto de persistência.
- `PainterParams::wash` (sub-struct `WashUiParams`, `#[serde(default)]`) é removido; projetos antigos
  continuam carregando (campo ausente = default; o dado de wash é simplesmente ignorado). Reduz a contagem
  de campos de `PainterParams` (continua ≤12, ADR-0043 §2.3).
- `RenderingParams.wet_edges`/`burnt_edges`/`edge_intensity` (parte do `Brush` CONGELADO) **ficam como
  campos inertes** para não tocar o ABI serializado do Brush nem o gate `architecture_painter_contract_surface`;
  apenas o comportamento (settle) e os toggles de UI saem. Remoção formal desses campos = ADR-amendment futuro.

## 4. Norte futuro (não implementado aqui — só registro)

Reconstruir aquarela/molhado, se desejado, como **mixer-brush**: Wet Mix no dab (Dilution/Charge/Attack/
Pull amostrando o canvas), cor por **Mixbox** (já presente em `pigment_mix`). É o modelo Procreate/IMPaSTo —
determinístico e reproduzível. A simulação de fluido (Rebelle/Fresco Live) fica como possível modo "Live"
opcional, jamais como default.

## 5. Alternativas rejeitadas

- **Continuar a topologia Curtis g/d (ADR-0095):** é a fonte das 3 versões irreprodutíveis. Rejeitado.
- **Manter o canvas GPU-residente sem o fluido:** é substrato construído para o GPU-first watercolor
  abandonado; adiciona a maquinaria `gpu_stamps` sem consumidor após a remoção. Rejeitado em favor do
  slate CPU-residente mais limpo (perf de pincel-gigante via GPU = rebuild deliberado futuro).
