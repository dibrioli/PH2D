# HANDOFF — Smoke do Enio: fechamento do W15 (E4+E5 zero-readback preview)

> W15 (Fluid Brushes/aquarela) **FECHADO**: E1–E5 completos. Este batch (E4 `db9466aa` + E5
> `6547bdd1`) é infra de perf/arquitetura — o LOOK da aquarela **não muda em nada**; o que muda é
> COMO o preview chega na tela (zero round-trip CPU mid-stroke). O smoke é portanto de
> regressão + fluidez, não de feature nova.

## O que mudou por baixo

| Modo | Antes (por frame) | Agora (mid-stroke) |
|---|---|---|
| 1 camada (stack trivial) | composite GPU → readback → canvas_rgba → premultiply CPU O(canvas) → re-upload textura | composite → textura premultiplicada GPU → cópia GPU→GPU → sprite (zero CPU) |
| Multi-camada (GPU-representável) | + reflatten CPU do stack | composite → textura straight → inject GPU→GPU no slice da camada ativa no LayerCompositor → recomposite GPU → premul GPU (zero CPU) |
| Pós pen-up (secagem) | igual | caminho readback antigo + catch-up (canvas_rgba fica atual antes do próximo traço) |

Medido (bench, full-res): imposto de readback removido ~1ms/frame banda típica, **10ms** full-wash
4K — mais o premultiply CPU + upload de textura que sumiram do loop do app.

## Smoke (roteiro)

1. **Regressão visual 1 camada** — pinte aquarela normal (wash, capilaridade, lift, branching).
   Deve estar **idêntico** ao de ontem, só mais fluido. Atenção especial ao **pen-up**: não deve
   haver flicker/salto no instante que solta o traço (hand-off textura→readback coberto por 2
   frames de override).
2. **Multi-camada** — crie 2-3 camadas (opacidades/blend diferentes), pinte aquarela na camada do
   meio. O traço deve aparecer **corretamente composto entre as camadas em tempo real** (acima das
   de baixo, abaixo das de cima, com blend/opacity respeitados). Pós pen-up + secagem: trocar de
   camada, undo, thumbnail — tudo deve refletir o estado correto.
3. **FPS** — `PH2D_FLUID_PROFILE=1` no terminal imprime a média por frame. Compare com a sessão
   anterior; mid-stroke deve estar igual ou melhor (em canvas grande, visivelmente melhor).
4. **Undo/commit** — terminar traço, esperar secar, undo → restaura; redo; commit — sem corrupção.

## Janelas conhecidas (aceitas, documentadas no commit)

- Commit por TECLADO com o ponteiro ainda pressionado bakea sem o traço em curso (ele entra no
  pen-up). Undo mid-stroke re-aplica o wash vivo por cima do canvas restaurado (classe de hazard
  pré-existente; janela agora é o traço inteiro).
- Pressão extrema de VRAM (4K + 8+ camadas vivas) pode evictar o slice injetado por 1 frame
  (snap transitório; follow-up: pinar o slice da camada ativa durante o traço).
- 2 gates de perf `#[ignore]` do ph2d-render estão borderline NESTE Mac 8GB (também em
  origin/main — `scales_linearly` 6.8× lá vs 6.1× aqui; variância de carga). Não rodam na CI.

## Follow-ups registrados (fora do W15)

dirty-rect no recomposite E5; pinar slice ativo contra LRU; LBM/MoXi dendrítico (ADR-0082 §2.3);
tiling esparso 4K (ADR-0083 §4). Próxima etapa da escada: **W7** (color modes + ColorDrop +
Eyedropper) ou **W8** (Selection/Transform).
