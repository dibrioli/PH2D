---
name: feedback_a_cache_invalidated_in_another_file_is_a_branch_no_oracle_sees
description: "Uma cache derivada invalidada NOUTRO ficheiro é um ramo que o oráculo da cache nunca exerce — a frescura vive dentro dela (a versão da fonte)"
metadata:
  type: feedback
---

⛔⛔ **Uma cache derivada invalidada NOUTRO ficheiro é um ramo que o oráculo da cache nunca exerce** (2026-10-04,
W11 da navegação, `line/components`). As paredes do desvio por mosaicos (`ParedesDaMalha`) tinham um oráculo
campo a campo contra a construção inteira — e passava. A invalidação vivia em `nav_malha.rs`
(`p.montadas = None` quando a malha mudava). Apagá-la **sobreviveu** à prova de mutação (29/30): o oráculo
chamava `monta` directamente, e os 17 testes da ponte em ponta não olhavam as paredes depois de uma mudança.

**Why:** um oráculo testa a FUNÇÃO que deriva; quem decide QUANDO derivar estava fora dela, num terceiro sítio
que nenhum gate liga ao resultado. O defeito real seria o desvio a contornar uma porta que já saiu, sem erro
nenhum.

**How to apply:** ponha a pergunta «estou em dia?» DENTRO da cache, contra uma VERSÃO da fonte (aqui
`TiledMesh::versao`, que sobe a cada mudança; `ParedesDaMalha::paredes(tm)` compara-a) — e faça o oráculo
passar por essa porta, não pela função de montar. Assim o descarte noutro ficheiro deixa de existir (um sítio
a menos para esquecer), e as duas mutações (nunca refazer · a versão não sobe) sangram no gate que já havia.
Família: [[topic-gate-discipline]] · [[reference_topic_mutation_proofs]].
