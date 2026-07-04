---
name: audit-scope-discipline
description: "Auditor adversarial acha bug em crate adjacent → handoff/issue pro owner, NÃO fixo eu mesmo. Escopo de implementor é a SUA pasta exclusiva + bits específicos de integração."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 341efa42-4ad5-477d-827a-4cfdef5f0653
---

Quando auditores adversariais (multi-lens parallel agents) acham bugs
em código que está FORA do crate/pasta exclusiva do meu trabalho atual,
a ação correta é **handoff documentado pro owner do crate**, NÃO fixo
eu mesmo. Isto vale mesmo quando o auditor afirma severidade
CRITICAL/HIGH e mesmo quando o fix técnico é trivial.

**Why:** registado no incidente 2026-05-27 R7→R8→R9 do T1.6 Painter:

- R7 (5 lenses) trouxe achados em `bgremoval/algorithm` (J1-4
  panic→Result), `bgremoval/params` (I1-1 non_exhaustive), `shells/
  desktop/bgremoval_preview` (J1-3 GPU toast). Eu fixei tudo "porque
  já tinha capturado esses arquivos via CI recovery sweep".
- R8 (5 lenses) herdou + R9 (6 lenses) escalou pra `ph2d-color`
  (canon OklchColor serde), `bgremoval/scratch.rs` overflow, `bgremoval/
  tool.rs` migração try_run_pipeline, `padding/upscale/color_eq/
  equalize-sizes/painter params` non_exhaustive, 4 drain files pt-BR.
- Enio: "Por que vc que está implementando o painter está vendo
  outros módulos como BGRemoval, CEQ dentre outros?" Scope creep
  flagrante. Revert via commit `7fed63b`.

**Consequências do scope creep:**
1. Compete com agente owner do crate alheio (parallel-agent collision).
2. Dilui foco do meu work próprio (Painter T1.6).
3. Cria commits "feat(painter): ... mas também 8 outros tools".
4. Auditor adversarial naturalmente expande escopo lens-a-lens —
   sem disciplina explícita, audit drift acumula geometricamente.

**How to apply:**

1. **Antes de fixar qualquer achado, perguntar:** "este arquivo está
   na minha pasta exclusiva do trabalho atual?" Se não → handoff.
2. Painter T1.6 scope explícito:
   - `crates/ph2d-painter-brush/*` ✓
   - `crates/ph2d-painter-contracts/*` ✓
   - `crates/ph2d-tool-painter/*` ✓
   - `docs/Painter_projeto/*` ✓
   - **Painter-específicos** em shells: `painter_bridge.rs`,
     `hero_intents/image_edit/painter.rs`, `app_state.rs` campos
     Painter, `render_loop/mod.rs` lines que mexem com PainterTool
     downcast (e.g. L1-1 destructive deactivate warn).
3. **Fora do escopo, mesmo com finding CRITICAL:** anotar em
   `docs/HANDOFF_<scope>_<lens>.md` OU em memory project_* com nome
   + severity + sketch da fix. **Não tocar o arquivo.**
4. **Auditor que sugere fix cross-crate** → questionar o briefing
   da próxima rodada de auditoria: "lens X deve apenas relatar
   achados em crates {owned_list}; achados em outros crates devem
   sair como handoff bullet, não como remediation target."
5. Memory `feedback-parallel-agent-collision` reforça: cada commit
   próprio deve usar `git commit -- <paths>` atômico restrito ao
   escopo. Se a remediação requer arquivos fora do escopo, é um
   sinal pra repensar a remediação, não pra expandir o commit.

**Excepções:**
- Coordenador (papel multi-agente v6.0) opera cross-crate por design.
  Esta regra é para Implementador / agente owner-de-feature.
- Bugs que TRAVAM o build/test do meu próprio crate (e.g., trait
  impl ausente em crate dependency) precisam ser fixed pra eu poder
  validar — mas mesmo aí, fix mínimo + handoff registrando "fix
  cosmético, owner deve revisar".

Linka com [[feedback-parallel-agent-collision]] (mesma família — escopo
disciplinado evita colisão).
