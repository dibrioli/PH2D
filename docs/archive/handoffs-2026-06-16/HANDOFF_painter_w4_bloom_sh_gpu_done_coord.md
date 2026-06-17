═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter · Bloom + Shadows/Highlights GPU — render-side PRONTO
Autor: Coordenador (jornada 2026-06-06) · responde HANDOFF_painter_w4_bloom_sh_coord.md
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
Acelerei os 2 kinds na GPU no pass-graph do `ph2d-render` (W4), reconciliados
**bit-a-bit contra as TUAS refs CPU canônicas** (`apply_bloom` / `apply_shadows_highlights`).
Commits locais (sem push): `ff70ad2` (Bloom), `37a06d4` (S/H). Tudo verde no Metal.

**Falta só a TUA metade — 2 linhas na tua crate** pra ligar end-to-end (§3). Hoje
ambos rodam via CPU fallback porque `gpu_spatial_code` devolve `None` pra eles.

## §1 — Bloom (commit ff70ad2)
Pass-graph: `cs_bloom_bright` (bright-pass premultiplicado: `color·α·smoothstep(
threshold, threshold+falloff, display_luma)`) → blur separável COMPARTILHADO
(`premul_read=0`, já premult) → `COMBINE_BLOOM` (additivo: `base_pm + intensity·glow`,
un-premultiplica). **Feathera cobertura** (glow haloa pra fora) — `cs_combine` adota
o alpha do kernel, igual teu `feathers_coverage()=true`.
- `SPATIAL_BLOOM=4`, params `[threshold, intensity, radius, falloff]`.
- **Não usei mip/Kawase pyramid**: ela NÃO byte-bate com teu `apply_bloom` (Gaussiana
  separável real). Usei a separável exata (reach `MAX_BLUR_HALF=256`, cobre radius
  interativo; default 20). Pyramid = otimização futura CONJUNTA (teu `apply_bloom`
  teria de virar pyramid também pra paridade) — não-bloqueante.
- Gates: `gpu_bloom_matches_cpu_reference` (≤5B, full+dirty-rect⊕halo) +
  `gpu_bloom_haloes_into_transparency` (glow espalha cobertura pra fora).

## §2 — Shadows/Highlights (commit 37a06d4)
Primeiro kind MULTI-MAPA: `cs_sh_luma` (extrai display_luma p/ `.r`) → DOIS blurs
escalares (raios shadows/highlights, reusando `cs_blur` com `premul_read=0`) → mapas
locais → `cs_combine_sh` (correção tonal local + midtone contrast + saturação).
**Cobertura PRESERVADA** (op tonal; combine própria, `run_combine` compartilhada é
pulada) — igual teu `feathers_coverage()=false`.
- `SPATIAL_SHADOWS_HIGHLIGHTS=5`. Precisa dos **8 params** → o contrato
  `SpatialAdjustment.params` alargou `[f32;4]→[f32;8]` (uniforme; os 4-escalares
  zero-padam o tail). Só os testes do render + o flatten do shell constroem
  `SpatialAdjustment` — ambos atualizados; **tua crate intocada**; sem arch-gate.
- Gate: `gpu_shadows_highlights_matches_cpu_reference` — **max byte diff = 0** no
  Metal (full + dirty-rect⊕halo), asserta ≤4.

## §3 — O QUE FALTA (TUA crate, ~2 linhas, padrão idêntico aos 4 kinds atuais)
Em `ph2d-painter-brush/src/adjustments/mod.rs`:

**`gpu_spatial_code()`** — adiciona:
```rust
Self::Bloom => 4,                  // SPATIAL_BLOOM
Self::ShadowsHighlights => 5,      // SPATIAL_SHADOWS_HIGHLIGHTS
```

**`spatial_params()`** — Bloom cabe em 4; S/H precisa de 8 → ALARGA o retorno
(hoje `Option<[f32;4]>`). Opções:
- (a) muda p/ `Option<[f32;8]>` e zero-pada os 4 kinds atuais; OU
- (b) método novo `spatial_params8()` p/ S/H, mantendo os outros em 4.
A ORDEM exata que o render lê pro S/H (NÃO mude):
```rust
Self::Bloom(p)            => [p.threshold, p.intensity, p.radius, p.falloff, 0,0,0,0],
Self::ShadowsHighlights(p) => [
    p.shadows_amount, p.shadows_tonal_width, p.shadows_radius,
    p.highlights_amount, p.highlights_tonal_width, p.highlights_radius,
    p.color_correction, p.midtone_contrast,
],
```
O flatten do shell (`painter_gpu_flatten.rs`, já é meu/Coord) hoje faz
`[p[0..4], 0,0,0,0]`. Se fores pela rota (a), eu ajusto o flatten p/ passar os 8;
me avisa qual rota escolheste (decisão tua — é a tua API).

**`feathers_coverage()`**: nada a fazer no GPU — o render já espelha por construção
(Bloom feathera no `cs_combine`; S/H preserva base.a no `cs_combine_sh`).

## §3.5 — Noise + Halftone GPU (commit 84f559e) — TAMBÉM PRONTOS
Não são spatial (sem vizinhança) mas lêem a coord ABSOLUTA (gx,gy) → vão no caminho
per-pixel `cs_flat` (o `apply_adjustment` agora recebe `coord`, threaded em
cs_flat/cs_grouped/cs_segment; kinds coord-independentes ignoram). Sem pipeline novo.
- **`ADJ_NOISE=9`** `[amount, kind(0=Gaussian,1=Uniform), monochromatic(0/1)]`. O hash
  (`hash_u32`/`rand01`/`noise_value`) é **bit-idêntico** CPU↔GPU (u32 wrapping, zero
  transcendental); só o sRGB `pow` diverge. Gate: **diff 0** no Metal (≤4).
- **`ADJ_HALFTONE=10`** `[dot_size, angle, shape(0=Dot,1=Line,2=Circle)]`. Threshold
  duro + rotação → gate por FRAÇÃO (sin/cos+fract ULP podem flipar boundary); observei
  **0/9216 flips** no Metal (<1% + ink&paper presentes).
- **Wiring (tua crate):** `gpu_code(Noise)→Some(9)`, `gpu_code(Halftone)→Some(10)` +
  `gpu_params` devolver `[amount, kind as u8 as f32, mono as f32]` / `[dot_size, angle,
  shape as u8 as f32]` (a ORDEM dos enums `NoiseKind`/`HalftoneShape` já bate). Sem isso,
  uma layer Noise/Halftone força o preview INTEIRO pro CPU; com, fica no compositor GPU.

## §4 — POSSE / smoke
- Mexi só em `ph2d-render` (+ o flatten do shell, Coord) + o teste do render. Sem push.
  Commits: `ff70ad2` Bloom, `37a06d4` S/H, `84f559e` Noise+Halftone.
- Smoke do Enio: depois do wiring (§3 + §3.5), Bloom numa layer transparente → halo de
  luz; S/H → levanta sombra / recupera highlight sem achatar contraste local; Noise →
  grão; Halftone → trama de pontos — tudo agora na GPU (~sub-ms vs o CPU fallback).
═══════════════════════════════════════════════════════════════════
