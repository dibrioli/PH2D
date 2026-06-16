# ADR-0093 — Canvas residente na GPU (Painter)

- **Status:** ACEITO (Enio, 2026-06-14); implementação faseada em andamento.
- **Contexto:** desfizemos o sistema de aquarela (fluid/wash, [ADR-0049](0049-fluid-brushes-extension.md)/[0085](0085-watercolor-v2-gpu-first-realtime.md)/0086–0092) para estabilizar a base. A avaliação do módulo apontou o alicerce que falta: **o canvas é CPU-residente**. Cada traço é pintado por `cpu_render::apply_stamps_*` na CPU (`PainterTool.canvas_rgba: Arc<Vec<u8>>`) e só o dirty-rect sobe à GPU como preview. O renderizador de stamp em GPU (`StampPipeline`, [ADR-0044 §2.9](0044-brush-engine-gpu.md)) já existe e é naga-validado, mas **nunca é despachado pelo shell**. Essa topologia (canvas CPU + upload/readback por frame) é a raiz da lentidão submit/copy-bound que inviabilizou o watercolor GPU-first ([ADR-0085 §0](0085-watercolor-v2-gpu-first-realtime.md)) e limita brush a ~256px.
- **Coord-only + ADR** (CLAUDE.md §4/§6): muda o modelo fundacional do Painter.

## 1. Decisão

O **canvas do Painter passa a residir na GPU como fonte-da-verdade.** A pintura inteira (wash default + build-up) roda na GPU; o preview é a própria textura residente (zero upload por frame); o undo vira tiles GPU; o `canvas_rgba` da CPU é **materializado lazy** (só save/export/MCP/thumbnail).

Supersede o invariante implícito "`canvas_rgba` (CPU, straight-sRGB8) é a fonte-da-verdade do canvas". A nova verdade é uma textura `Rgba8Unorm` (premul-sRGB) residente na GPU; a CPU mantém um espelho lazy.

## 2. Arquitetura

- **Sessão shell `PainterCanvasGpu`** (irmã de `painter_gpu_preview`) é dona das texturas residentes e dirige a pintura — espelha a topologia comprovada do bridge de wash deletado: texturas persistentes, seed do backdrop uma vez, **um encoder/submit por frame**, copy GPU→GPU no preview slot, retorna `PreviewOverride`.
- **Separação de papéis:** o tool continua produtor de stamps (scheduler CPU determinístico, HR-5); o shell é dono das texturas + dispatcher. O tool **não ganha dep de GPU** — expõe `Stamp` (POD 96B) por um buffer drainável (como `fluid_take_dabs`).
- **Dois pipelines:** `StampPipeline` (build-up, Porter-Duff "over", já existe) + `WashPipeline` novo (splat de cobertura monotônica + composite `mix(backdrop, cor, opacity·min(cov,1))`) para o pincel PADRÃO (`accumulate=false`).
- **Cobertura/cor** ficam em texturas GPU; no pen-up são lidas de volta junto com o canvas para reconstruir os buffers CPU que o `apply_wash_settle` (edge-darkening) consome inalterado.

## 3. Determinismo (HR-5)

O contrato de det-replay é sobre o **scheduler de stamp** (PRNG/ABI/aritmética inteira bit-idêntica cross-OS) e o WAL grava **pointer samples (Q16.16), não pixels** — ambos permanecem na CPU e inalterados. Mover o **blend** para a GPU é **ULP-bounded**, não bit-idêntico. **Não há gate de bit-paridade do canvas vivo** (`architecture_painter_contract_surface` é só caps estruturais; os gates `gpu_parity`/`composite_parity` do watercolor saíram com a remoção). Os testes de paridade GPU usam **banda ULP**, não igualdade bit.

## 4. Fases

- **Fase 1 (IMPLEMENTADA):** canvas residente (wash) para a stack **trivial** (layer ativa única, caso comum); CPU-sync nas bordas do traço (readback do canvas no pen-up → undo/settle/Apply inalterados). Stacks não-triviais ficam no caminho CPU.
- **Fase 2 (IMPLEMENTADA):** a textura **straight-sRGB8** da layer ativa (novo output `canvas_straight_out` do `cs_wash`) alimenta o `LayerCompositor` direto via `inject_slice_from_texture` (mecanismo herdado do fluid E5, device-testado) — stacks **não-triviais representáveis** compõem na GPU sem round-trip de readback. A invariante de versão mantém o slice injetado até o readback de pen-up bumpar a versão. O gate de paintability (`is_gpu_paintable` = trivial OU `flatten_for_gpu().is_some()`) é capturado no pointer-down: stack não-representável (máscara/clip/reference/ajuste não-portado) fica no caminho CPU. Compositor reusado é o mesmo `PainterGpuPreview` do preview de hover (cache de slices compartilhado).
- **Fase 3:** undo em tiles GPU; canvas residente entre traços; `canvas_rgba` lazy — a verdade fica de fato na GPU.

## 5. Consequências

- **+** GPU stamp render, brush grande, 4K real-time, preview sem upload por frame; substrato para fluidos GPU-first futuros.
- **−** dois caminhos de pintura coexistem durante as fases (CPU mantido como fallback/stacks não-triviais até Fase 2/3); custo de readback por traço na Fase 1 (eliminado na Fase 3); drift premul round-trip ±1/canal (bounded, não acumula com re-seed por traço).
- **Rollback:** o caminho CPU permanece compilado e correto; desligar o modo GPU (flag de capacidade) reverte ao comportamento atual sem perda.
