═══════════════════════════════════════════════════════════════════
AUDITORIA T5.3 — fechamento do Vector W5 (variable-width stroke + SDF Hybrid full)
Auditor: Coordenador · 2026-06-04 · gates executáveis (smoke visual indisponível — sem hardware de pressão)
═══════════════════════════════════════════════════════════════════

## Veredito: APPROVE — W5 fecha. Data-path verificado por teste. Lente VISUAL-com-pixels deferida (sem Wacom/pen).

Escopo W5 (plano §8): T5.1 variable-width stroke + T5.2 SDF Hybrid full + T5.3 audit.
**Divisão:** foundational (Coord) — primitiva render + WidthProfile + gate SDF; integração
(impl Vector) — pressão→largura no Pencil (commit `19cd7e4`). Método: **gates executáveis**
(o Enio não tem device de pressão → o smoke de pixels não roda; aterro no data-path testável).

## Lens A — variable-width data-path (input→render) · VERDE

A pressão flui de ponta a ponta, **fiação verificada real** (não claim — lição
`feedback_tool_unit_green_integration_dead`):
- **Captura:** `StrokeSample.pressure` (0..=1) — hook do W2, já existente.
- **Live preview:** `vector_pencil_bridge.rs:122` → `draw_variable_width_stroke(centerline,
  widths = line_width×pressure, …)`. Substituiu o `scene.stroke` constante.
- **Commit:** `tool.rs:327` → `StrokeStyle.width_profile = Some(WidthProfile{start,end = pressões
  dos knots})` **só quando a pressão varia** (`tool.rs:323`, senão style compartilhado); o
  `draw_vector_network` expande `width_profile` automático (renderer foundational `8dca426`).
- **Gates (rodados pelo Coord):** **25 testes do pencil verdes**, incl. os 2 que provam o path:
  `constant_pressure_keeps_one_shared_constant_width_style` + `pressure_variation_assigns_per_
  segment_width_profiles`. Foundational: `variable_width_band` + `scale_at` verdes.

## Lens B — SDF Hybrid full / real-time · VERDE

Gate `vector_sdf_real_time` (`7b03e48`): **GPU 64-path boolean draft = 5.33 ms/frame < 8.33 ms
(120 FPS)** medido no Metal; CPU = 140 ms (confirma GPU necessária). Critério T5.2 atingido.

## Lens C — regressão + no-bloat · VERDE

- **Regressão-safe:** device sem pressão (mouse/trackpad-sem-força → pressure 1.0) = **largura
  constante = look exato do W2** (tested: `constant_pressure_keeps_one_shared_constant_width_style`).
- **Sem bloat de StyleTable:** profile per-segmento só quando a pressão varia; pressão constante =
  1 style compartilhado. StyleTable fresco por-asset (não acumula).
- **Perf:** variable-width = band-fill no GPU (barato, mesmo path do fill). Sem readback no hot-path.

## Lens D — contrato + serialização · VERDE

`StrokeStyle.width_profile` (5º campo, `Option`): contract gate `architecture_vector_contract_
surface` verde (StrokeStyle não-gated); postcard `triangle_round_trip` (23) verde; **zero schema
bump** (Option apendado, precedente `dormant_fractures`, PH2D pré-release). VectorNetwork carrega
`style_ref:u32` não a StrokeStyle → **zero impacto em cook-hash**.

## Lente VISUAL (pixels na tela) — DEFERIDA (limitação de hardware, não gap de código)

O smoke final (traço afina/engrossa com Apple Pencil/Wacom/trackpad-force) **não rodou — o Enio
não tem device de pressão disponível**. O **data-path está provado por teste de ponta a ponta**
(pressão → largura → render), então o risco é só "os pixels finais batem com a intenção", não a
lógica. Confirmar quando houver hardware; **não-bloqueante** pra fechar o W5.

## Findings
| # | Sev | Item | Ação |
|---|---|---|---|
| DEFER-1 | — | Smoke visual com device de pressão | Aguarda hardware. Data-path test-coberto. |
| DEFER-2 | — | Piece C: richness do nó width-profile (bulge/contrast) | Deferido (W10 anima width; `WidthProfile` já existe, drop-in). |

Zero CRITICAL/HIGH/MEDIUM. Nenhum bug. Data-path completo + verificado.

## Conclusão
**Vector W5 FECHADO** (T5.1 + T5.2 + T5.3). Variable-width stroke (foundational + pressão→largura
no Pencil) + SDF Hybrid real-time (gate 120 FPS) — corretos, regressão-safe, dentro de budget,
data-path verificado input→render. Visual-com-hardware quando disponível. Próximo wave do impl
Vector = **W6** (procedural fill / shader graph, §9) quando o Enio liberar.
═══════════════════════════════════════════════════════════════════
