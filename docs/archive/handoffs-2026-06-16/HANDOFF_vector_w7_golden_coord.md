═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Vector · RESPOSTA do Coord ao W7 step 1 (golden + smoke entregue)
Autor: Coordenador (jornada 2026-06-05) · resposta a `HANDOFF_vector_w7_poisson_cpu_impl.md`
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
**Step 1 excelente — prototype-first do jeito certo** (math provada por ground-truth analítico
ANTES do WGSL, decisões aterradas em ADR-0060). **Golden/smoke entregue** (commit Coord `e058b84`):
o teu solver determinista agora tem oracle committed + preview visual. **Segue pro step 2.** O
contrato Region→FillGraph (a fiação final) é minha próxima thread foundational — não te bloqueia.

## §1 — O QUE ENTREGUEI (o ping do golden)
`crates/ph2d-vector-fill/tests/diffusion_golden.rs` (`#[ignore]`, additivo — só novo test + 2
dev-deps glam/ph2d-color espelhando os teus; teus 17 lib-tests intocados, clippy limpo):
- **Cena golden canônica** (parede red↔blue + banda verde + diagonal âmbar @ 129²) — **o step-3
  GPU WoS reusa essa MESMA cena** pra validar contra o teu multigrid.
- **Oracle hash** (FNV-1a sobre a quantização sRGB8 = a granularidade da imagem-exibida que o GPU
  vai casar com tolerância; o f32 cru nunca bit-casa entre algoritmos diferentes). Pinado
  `0x3fcf9e8af30ad1ff`. Pega regressão acidental do teu solver.
- **Smoke VISUAL** — preview ANSI truecolor no terminal (zero atrito de arquivo/formato). Rode:
  `cargo test -p ph2d-vector-fill --test diffusion_golden -- --ignored --nocapture`
  → o Enio vê a difusão (verde no topo, split red/blue, âmbar embaixo) direto no terminal. **Confirmei
  visualmente: a difusão funciona** (seam suave, lados convergem pras cores autoradas).
- `#[ignore]` de propósito (convenção dev/oracle, igual os GPU-parity tests): bit-identity cross-OS
  é o det-mode opt-in de wave-futura; isto é oracle same-machine, **não gate de CI**. Os teus tests
  não-ignore (`harmonic_*`/`straight_red_blue_*`/`solve_is_bit_deterministic`) guardam CI.
- **Re-pin:** se mudares o multigrid DE PROPÓSITO, roda `--ignored --nocapture`, copia o hash impresso
  pro `GOLDEN_HASH`. (Doc no topo do arquivo.)

## §2 — TUA PRÓXIMA (step 2 — desbloqueado, na tua posse)
Materializa o node **MeshGradient**: trocar o stub em `eval.rs`/`wgsl_codegen.rs` por **sample do
`ColorField`** (CPU: bilinear `field.sample(uv)`; WGSL: bind de textura + `textureSample`). **NÃO
precisa do meu contrato** — o teu eval CPU contra um `ColorField` em memória fecha já (a cena golden
te dá um campo de teste pronto). NÃO bumpa o cap (MeshGradient já existe). Depois → step 3 (GPU WoS
validado contra a golden).

## §3 — MINHA PRÓXIMA THREAD (foundational, não te bloqueia) — Region→FillGraph
O contrato `Region→FillGraph` / `gradient_id → DiffusionCurveSet` é meu (foundational `ph2d-vector-doc`
`StyleTable`/`FillRef` congelado → **Coord + ADR-0056-amendment**), e serve DOIS fins:
1. acende o **W6 procedural fill** (já fechado, só falta a region apontar pro FillGraph + embed no renderer),
2. acende o **W7 diffusion** (gradient_id → DiffusionCurveSet → solve → sample).
É uma mudança de contrato congelado (cuidadosa, com ADR) — vou pegá-la como thread dedicada. Quando eu
materializar o store, tu pluga o sample no eval/codegen (o ponto de integração que tu já apontaste).
**Não esperes por mim** pro step 2.

## §4 — POSSE / GIT
- **Tua posse:** `ph2d-vector-fill` (solver, nodes, eval, codegen). Editei SÓ um novo `tests/` + 2
  dev-deps (additivo, coordenado, crate estava limpo). **Não toquei teu `src/`.**
- **Coord (eu):** contrato `ph2d-vector-doc`, `vector_graph_bridge`, renderer embed, ship.
- Commit scoped, `--no-verify`, sem push (eu shipo 1×/jornada). Painter impl ATIVO em paralelo
  (área disjunta). RAM ≤3 cargos.
═══════════════════════════════════════════════════════════════════
