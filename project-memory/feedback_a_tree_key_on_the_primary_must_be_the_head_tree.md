---
name: feedback-a-tree-key-on-the-primary-must-be-the-head-tree
description: Selo/registo «esta árvore já passou» no checkout PRIMÁRIO nunca casa pela pasta — ela vive suja de project-memory/; o que se envia é o HEAD^{tree}
metadata:
  type: feedback
---

⛔ Um salto do tipo «a mesma árvore já passou a suíte» (B1 da comissão 29/10, `scripts/suite-completa.sh`) que
calcula a chave pela PASTA (commitado + por commitar + por rastrear) nunca casa no checkout primário: ele vive
com `project-memory/` por comitar, escrito por outras sessões, e nunca está limpo. Medido no ensaio do A
(29/10): o integrador passou a árvore num worktree limpo e o `ship.sh` no primário correria outra suíte de
25–35 min.

**Why:** o que o push envia é o `HEAD^{tree}`; o por comitar não vai no push. A pasta continua a ser a régua
de quem CORRE (testa o que está lá), mas a de quem SALTA é o que se envia.

**How to apply:** em qualquer selo novo que deva valer no primário, ofereça a chave do `HEAD^{tree}` ao lado da
da pasta (`suite-completa.sh --enviar`). Nunca «limpe» o `project-memory/` alheio para fazer casar a chave.
Relaciona-se com [[feedback-full-suite-only-before-integration]].
