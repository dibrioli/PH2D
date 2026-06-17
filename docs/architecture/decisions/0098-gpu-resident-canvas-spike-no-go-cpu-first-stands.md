# ADR-0098 — Spike canvas GPU-residente: NO-GO agora; CPU-first (ADR-0097) mantido

- **Status:** ACEITO (Enio, 2026-06-16 — escolheu "spike de viabilidade primeiro").
- **Data:** 2026-06-16.
- **Contexto:** A saga da aquarela (ADR-0085..0096, ~3 semanas, deletada) foi construída sobre a
  premissa de que o canvas precisa ser GPU-residente. Existe um `StampPipeline` (GPU stamp compute,
  828 LOC de WGSL, naga-validado, 9 gates de ABI-freeze) que **nunca é dispatchado** fora dos próprios
  testes — o caminho vivo pinta na CPU (`apply_stamps_wash` → `canvas_rgba: Arc<Vec<u8>>`) e sobe o
  dirty-rect por frame. A investigação 2026-06-16 apontou essa inversão como o maior erro de eficiência
  vs. estado-da-arte (Krita/MyPaint mantêm o canvas na GPU). A diretriz manda: **spike + go/no-go ANTES
  de qualquer build foundational** (não repetir o over-reach da aquarela).

## Medição (spike)

Caminho CPU vivo real (`PainterTool` begin→queue→end, scheduler + `apply_stamps_wash` + composite trivial),
canvas **4096×4096**, `--release`, Mac M-series. Teste: `spike_cpu_stroke_cost_4k`
([`golden_tests.rs`](../../crates/ph2d-tool-painter/src/tool/golden_tests.rs)).

| Brush | traço (24 samples) | ms/sample | vs 16.7ms/frame (60fps) |
|---|---|---|---|
| 64px | 66.7ms | 2.78 | ✅ folgado |
| 256px | 93.3ms | 3.89 | ✅ ok |
| 1024px | 361ms | 15.0 | ⚠️ no limite (≈1 dab/frame) |
| 2048px | 675ms | 28.1 | ❌ estoura (jank) |

(1 frame de input ≈ 1–3 dabs; ms/sample ≈ custo de aplicar um dab quando o spacing dispara.)

## Decisão

**NO-GO** na migração para canvas GPU-residente **agora**. Racional:
1. O CPU-first (ADR-0097) **atende os tamanhos relevantes à paridade Procreate** (pequeno–médio) com
   folga de frame. A paridade de pincel (14 painéis, dab dynamics) **não exige brush gigante em 4K**.
2. A GPU-residência só se justifica no caso **brush > ~1024px em 4K real-time**, que **não é requisito
   ratificado** do norte atual. Fazer a reescrita foundational sem esse requisito é exatamente o
   over-reach que custou as 3 semanas de aquarela (ADR-0096 §1).
3. "A melhor topologia" é a **comprovadamente barata** até bater uma parede real (diretriz).

## Gatilho de revisita (kill-criterion invertido)

Reabrir a migração GPU-residente **somente** quando **brush > 1024px em 4K real-time** virar requisito
ratificado pelo Enio. Aí: re-rodar este spike p/ confirmar os números, e então decidir wire do
`StampPipeline` vs. tiles GPU (plano arquivado `wise-seeking-gem.md` é o ponto de partida).

## StampPipeline (disposição)

Mantido (código validado é o substrato provado pro caso grande). Mas seus **9 gates de ABI-freeze
congelam uma ABI com ZERO consumidores** = freeze prematuro (puro imposto). **Follow-up:** relaxar
esses gates de "frozen" p/ "validated-dormant" (não bloquear evolução de algo que nada usa) — mudança
de contrato, fora deste ADR. Não deletar: é o caminho do gatilho de revisita acima.

## Consequências

- O norte do Painter é **inequivocamente CPU-first** (ADR-0097); GPU-residência é perf adiada com
  gatilho escrito, não aposta inicial.
- Fecha a questão que dirigiu a saga da aquarela: **a GPU-residência não era pré-requisito** — era uma
  aposta de topologia sem número. Agora há número.
