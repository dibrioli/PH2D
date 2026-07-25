---
name: feedback-final-product-every-node-ships-the-full-pro-param-set
description: "Past the MVP — every node carries the COMPLETE pro-app param set for its type, reviewed against the reference catalog (not memory) at build AND at revision."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bbfea52f-73bd-4cf1-8a2b-7250ed5b7f1d
  modified: 2026-07-25T01:35:58.181Z
---

**Não é o MVP; é o produto final** (Enio, 2026-07-24). Todo nó carrega o conjunto **COMPLETO** de parâmetros que os apps pro expõem para o SEU tipo — conferido no **CATÁLOGO** (as pesquisas `referencia_pesquisa_*.md`, no módulo em questão), **não na memória/lembrança** — ANTES de fechar o nó E ao TOCÁ-LO de novo. Um subconjunto mínimo é dívida disfarçada de "v1".

**Why:** shipei `field.box` **axis-aligned** (reflexo de MVP) e TODOS os apps pro (C4D/Cavalry/Houdini) rotacionam os fields — Enio pegou na hora ("não vejo modo de rotacionar os fields"). E o catálogo ainda disse **COMO**: o transform (position+rotation+scale) vive **NO field, com gizmo de canvas** — nunca num nó de transform separado (isso é idioma TouchDesigner/DAG-geral, não mograph). Consultar o catálogo corrigiu **os dois** reflexos (o meu MVP e o de "fazer um nó de rotação").

**How to apply:** toda task de nó abre por (1) abrir o dump do(s) app(s) que têm aquele tipo e LISTAR os params **verbatim**; (2) implementar o **superset**, OU escrever no doc-comment do nó por que um param é fatorado noutro nó / diferido (senão vira "esqueci" vestido de "v1"); (3) conferir Coordinates (position/**rotation**/scale), o neutro byte-idêntico (D12) e a rota GPU. Canonizado no plano em [[reference_canonical_files]] → `docs/Motion Nodes/63_*` §0.1 e D13. Vale para **qualquer módulo**, não só motion nodes. Relaciona com [[feedback_ergonomics_verdict_is_a_design_bug]] (questione o modelo) e [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]] (mire no extraordinário).
