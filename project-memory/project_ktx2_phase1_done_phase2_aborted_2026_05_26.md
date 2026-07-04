---
name: ktx2-phase1-done-phase2-aborted-2026-05-26
description: "KTX2 Fase 1 (codec-only crate ph2d-asset-ktx2) entregue + ratificada; Fase 2 (Basis Universal runtime + ACEScg + BC6H HDR) ABORTADA pós-auditoria 4-lente (5.67/10 média, 12 CRITICAL). Caminho recomendado para Coord-A futura = Opção E (cooking offline nativo per-platform, sem Basis runtime, ph2d-color ADR-0051 expandido)."
metadata: 
  node_type: memory
  type: project
  originSessionId: 85df73a6-feb3-48ba-96a8-47365c0f1f69
---

## Estado em 2026-05-26 noite

**Fase 1 — ENTREGUE.** Crate `crates/ph2d-asset-ktx2/` (1207 LOC lib.rs + 22 LOC Cargo.toml).
4 commits locais: `f30e225` → `db96f28` → `7806369` → `b276cef`. 24 unit + 2 doctests = 26 verdes.
clippy `--all-targets -- -D warnings` + fmt clean. Deps: `ktx2 = "0.5"` (pure-Rust parser) + `thiserror = "2"`.
Cobertura: parse container, 25 Ktx2Format variants (RGBA8/16/32 + BC1/3/4/5/6H/7 + ASTC 4×4..8×8 + ETC2),
limits defensivos, reject 3D/cubemap/array, exhaustive error paths. **Não pushado** — fast mode, decisão do
Enio sobre quando.

**Fase 2 — ABORTADA.** Rascunho ADR-0055 (KTX2 + UASTC/BasisLZ + BC6H HDR + ACEScg + 2 crates novos) foi
escrito e auditado por 4 agentes paralelos adversariais. Resultado: score médio **5.67/10** (Painter
ratificado hoje pós-4 audits = 9.0/10). **12 CRITICAL findings**. 2 de 3 audits adversariais recomendaram
REJECT/REWORK. Avaliador final 4 confirmou e ofereceu Opção E.

ADR-0055 file deletado (untracked, sem histórico git). Zero referências cruzadas externas — apagar o file
zerou as menções.

## Caminho recomendado para Fase 2 (Opção E — quando Coord-A retomar)

**Princípio:** "melhor pra 2D pro tool" ≠ "stack mais complexo de 3D AAA". O melhor é zero CPU spike em
load + pixel-perfect + WASM portable + builds puro-Rust estáveis. Não é Basis Universal runtime.

| Aspecto | Decisão recomendada |
|---|---|
| Container | KTX2 — Fase 1 codec é o alicerce |
| Compressão | **Nativo per-platform offline** (cooker): BC7 desktop, ASTC LDR mobile, ETC2 fallback |
| HDR | BC6H_UFLOAT desktop + ASTC HDR mobile, **cooked offline** (sem Basis layer) |
| Apple iOS/iPadOS | **ASTC apenas** — Metal-iOS NÃO expõe BC. Verificar em runtime via wgpu feature query. |
| Runtime transcoder | **NÃO criar `ph2d-asset-basisu`.** Renderer lê KTX2 → `queue.write_texture` direto. |
| Color pipeline | **NÃO criar `ph2d-color-pipeline`.** Usar `ph2d-color` expandido (mandato ADR-0051 em curso) |
| ACES | **Tonemap operator apenas** (shader output), sobre Linear sRGB working space. **NÃO ACEScg gamut** — 2D games shippados zero. |
| ColorProfile | Reusar 8 variants FROZEN da ADR-0051. **NÃO amendar.** Conversor `imageio → painter` ADR-0054 ↔ ADR-0051. |
| Cooker | `tools/asset-cooker` chama CLIs externas (toktx/Compressonator), não FFI in-process |
| Painter wins | Brush atlases R8 → BC4 (4× saving), UI assets → ASTC LDR (priorizar W1) |

## Lições para próxima ADR de texture/asset

- **Pesquisa industrial verificada SEMPRE antes de afirmar adoção.** Vide [[no-industrial-claims-without-verification]].
- **Ler TODAS as ADRs ratificadas nas últimas 24-48h antes de escrever nova ADR.** Painter cascade
  0050-0053 + ADR-0054 imageio ratificadas 2026-05-26 contêm `ColorProfile`, mandato `ph2d-color` expansão,
  caps `*_count_is_exact_N`. Ignorei e derrotei FREEZE silenciosamente.
- **Override de HR-N exige critério objetivo aplicável a futuras ADRs.** Argumento case-by-case
  ("HEIC não pode FFI C++ mas Basis pode") abre slippery slope.
- **Auditoria N-lente paralela ANTES de marcar Proposed, não apenas pré-Accepted.** ADR-0054 §W0.T6.5
  é o pattern: 5 audits inline durante W0. ADR-0055 escrito sem isso → 12 CRITICAL.

## Briefing-pronto para Coord-A retomar Fase 2

Quando Coord-A futura quiser endereçar texture compression, ler nesta ordem:

1. Esta memória (caminho recomendado + decisões parking)
2. `crates/ph2d-asset-ktx2/src/lib.rs` (Fase 1 entregue + #![forbid(unsafe_code)] precedent)
3. ADRs 0040, 0042, 0051, 0053, 0054 (especialmente caps FROZEN e `ColorProfile` dualidade)
4. [[no-industrial-claims-without-verification]] (checklist pre-flight)

Pesquisa fresca obrigatória antes de escrever ADR-0055-Revised:
- `cargo search basis-universal` — confirmar versão real (não inferir)
- WebFetch Unity 6 / Unreal 5.7 / Godot 4.x docs sobre texture compression default (não recitar)
- `ls docs/architecture/decisions/0009-*.md` — ADR-0009 Radiance Cascades não existia em 2026-05-26 (alucinei dependência)
- libbasisu v2.10 (mai/2026) API surface — UASTC HDR estável?

Estado canônico no fim da sessão 2026-05-26: Fase 1 commitada local, Fase 2 ABORTADA, sem push, working
tree do PH2D limpo do meu lado.

Vide também [[feedback-perfection-no-deferrals]] (princípio que mandou os 12 CRITICAL serem trabalhados
agora, não diferidos), [[project-painter-w0-ratified-2026-05-26]] (Painter cascade que invadi sem ler).
