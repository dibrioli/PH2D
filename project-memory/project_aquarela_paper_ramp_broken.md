---
name: project_aquarela_paper_ramp_broken
description: "Rampa \"Paper Colors\" da aquarela — construída, consertada (Opção A), mas REVERTIDA a pedido do Enio (2026-07-06); papel volta ao grayscale"
metadata: 
  node_type: memory
  type: project
  originSessionId: 62398db0-b6c6-484b-bf31-05088eaaa242
---

**REVERTIDA 2026-07-06 — o Enio preferiu a implementação ORIGINAL sem color ramp.** A seção **Paper**
(papéis procedurais Cold/Rough/Hot como substrato grayscale + Angle + tags "Use as Paper/Granulation" +
preview) **fica**; só a **rampa Paper Colors** (3º ramp que tingia o papel) saiu. Não reconstruir sem ele pedir.

**Como foi revertido (worktree `line/Painter`):** a rampa era o commit `3f227f0c` (integrado ao main, 24
arquivos +707/−19), seguido do fmt largo `5cb9a9f0`. Reverter só `3f227f0c` com `git revert` conflitaria com o
fmt → em vez disso restaurei cada arquivo tocado ao pai `3f227f0c~1` (= `614a0676`, "Angle funciona") +
`git rm` nos 4 arquivos criados (`paper_ramp.rs`, `paint_paper_ramp.rs`, `paper_ramp_picker.rs`,
`ramp_events.rs`) + reaplicei fmt (pin 1.95; o pai era `--no-verify`/fmt-sujo, o 5cb9a9f0 tinha limpado).
Verifiquei que NADA além do fmt tocou os 24 arquivos depois da rampa (senão o checkout-do-pai perderia
mudança boa). 0 refs residuais, 458 testes tool + 40 painel + arch gate verdes.

**Por que reverteu (lições da tentativa):** a rampa funcionava, mas o modelo "papel = substrato colorido" num
render de **escrita-direta** (Beer-Lambert, sem alpha emplumado) expôs problemas: v1 (`sb=paper_col·base-sobre-branco`
em força total até o corte `cw>0`) → **borda serrilhada** (substrato saturado revela o cliff que ≈branco
escondia); Opção A (emplumar por `cw`) consertou a borda mas o **miolo mosqueado** (tooth mapeado pela rampa
inteira) persistia. O Enio achou o resultado inferior ao papel grayscale liso e mandou reverter. **Contraste
que vale lembrar:** o Grain color ramp (`dab.rs`) NÃO usa multiply-substrato — recolore o pigmento com
cobertura emplumada + BrushBlend do topo (`PAINTER_BRUSH_BLEND`, 24 modos); o caminho da aquarela é óptico e
ignora esse BrushBlend. Ver [[feedback_visual_bug_debug]].
