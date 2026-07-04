---
name: project_brush_audit_2026_06_18
description: "Auditoria multiagente do brush engine (2026-06-18) — 2 HIGH corrigidos + os claims de paridade CPU↔GPU do código MENTEM (latentes, não-gateados)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 141ce813-6352-4f60-9687-4c1b43fa526c
---

Auditoria multiagente total do código do pincel (~38k LOC, 4 crates) em 2026-06-18: 18 clusters, 112 achados → verificação adversarial → síntese. Veredito **minor-issues**, nenhum critical. ABI W2.9 (`_pad→roundness`, 96B align16) **íntegra** (confirmado por gates: ABI 78, parity 3, det_random 7, wiring 3).

**Corrigidos na sessão (com testes):**
- HIGH: `move_into_group` ([stack.rs](../../../../crates/ph2d-tool-painter/src/layers/stack.rs)) ignorava `subtree_height` → árvore de layers insalvável por drag-drop. Fix: somar `subtree_height` ao guard (espelha `move_to_sibling_of`).
- HIGH: cap `MAX_RECOVERED_STROKES` ([recovery.rs](../../../../crates/ph2d-painter-stroke/src/durability/recovery.rs)) burlável no branch de replay-attack. Fix: aplicar o cap também lá + `reconstruct_capped` injetável p/ teste barato.
- SCHED-1 (meu gap W2.9): spacing anti-veneziana usava só `shape_roundness` estático, ignorando squash de pressão/tilt. Fix: roundness efetivo determinístico no `seg_spacing_px`.

**Lição durável (NÃO-óbvia — o código afirma o contrário):** os comentários de paridade CPU↔GPU do brush **mentem em 4 frentes** — grain procedural ("Mirror of"), pigmento ("bit-identical"/"EXACT mirror" em `stamp.wgsl:423/478` — mas o CPU vivo usa LUT `mix_prepared`, o shader espelha `mix_prepared_exact`, ~1.5e-2 de divergência), chromatic aberration ("single source of truth", diverge em SINAL), intense_glaze. São **LATENTES** porque o render vivo é CPU-only ([ADR-0097](../../../../docs/architecture/decisions/0097-brush-engine-procreate-parity-cpu-first-dab-pipeline.md)) e nenhum gate as trava — ao contrário de shape-kernels e dos 6 rendering modes que TÊM gate. **Quem ligar o GPU path (W5+) NÃO pode confiar nesses claims** — meça antes. Mesmo padrão em docs que afirmam que features W11+ (det-painter força Linear, OnDisk undo, paper_base, resolve_pigment_mode do ADR-0044 §2.5.1) estão honradas quando NÃO estão/não existem. Ver também [[feedback_no_industrial_claims_without_verification]].

**Follow-up FECHADO na mesma sessão:** claims de paridade falsos corrigidos (docs dizem a verdade agora) + 2 alinhamentos de código (grain shader arg → pixel-center, intense_glaze inv_aa→divisão); ~8 comentários stale/invertidos limpos (square_hard "Hermite"→Van Verth, LookFn sRGB, mass≈0→branco, atlas budget); hardening WAL (flush restaura batch em erro de I/O; placeholder poisoned removido). Validado: brush 335 + stroke 100 verdes, clippy limpo, WGSL naga-valida, 4 gates congelados verdes. A divergência de CÓDIGO CPU↔GPU (pigment LUT vs exact, chroma sinal) permanece LATENTE — só os docs foram honestados + gate-notes adicionados; quem ligar o GPU path ainda precisa gatear. **TODOS os deferidos também FECHADOS (2026-06-19):** should_rotate sem syscall (contador in-memory `journal_bytes` seedado do file size, +test); consts `TILT_TARGET_*`/`BARREL_TARGET_*` (+test travando a landmine barrel Bleed=4 ≠ pressure Bleed=8); resync exige `payload_size` plausível (corta falsos-positivos); undo/redo OnDisk peek-before-pop via re-push (+test); paper_base segue o canvas em undo/redo (simetria com restore_model). Validado: brush 337 + stroke 151 + tool 248 verdes, clippy limpo, 4 gates congelados verdes. **Auditoria 100% resolvida** (nenhum achado em aberto; divergência de CÓDIGO CPU↔GPU permanece latente por design até o GPU path ligar).
