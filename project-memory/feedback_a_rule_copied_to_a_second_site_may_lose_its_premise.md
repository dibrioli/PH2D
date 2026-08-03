---
name: feedback-a-rule-copied-to-a-second-site-may-lose-its-premise
description: "A mesma regra em dois sítios pode ser certa num e pura perda no outro — o que viaja é a CONDIÇÃO, não o que a justificava; ablacione o limiar por sítio"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 39ec3808-26ec-4cf4-b80e-b2291882bc64
  modified: 2026-08-02T18:22:22.803Z
---

Quando um limiar/heurística existe em **dois sítios**, o que foi copiado é a **condição**; a **premissa**
que a justificava fica no primeiro. Vale a pena ablacionar **um sítio por vez** — atribuir ao par é como
se conclui que "a regra está certa" sobre um sítio onde ela só perde.

**Caso medido (PH2D, `from_journal` vs `from_window`, 2026-08-02):** o mesmo limiar de 50% mandava um
delta grande para `Whole`. No `from_window` o `Whole` **MOVE** os `Arc` que já existem (custo zero); no
`from_journal` ele **COPIA** — `par_clone` do plano inteiro + varredura de plano inteiro — **descartando
o `before`/`after` que a própria função já extraiu**. Ablacionando os dois juntos: −99 ms. **Um por
vez:** o journal vale **−121 ms** e o `from_window` **PIORA +24** se ablacionado. E o ramo perdia também
em bytes (8,00× contra 7,66× um plano RGBA por passo) — *não havia trade nenhum*.

**Why:** o doc-comment **declarava** a premissa (*"ali o `Whole` guardaria os dois planos inteiros de
qualquer forma — o `split` clássico faz a mesma escolha, no mesmo limiar"*). Premissa escrita é premissa
auditável; foi ela que nomeou o defeito.

**How to apply:**
1. Grepe o limiar. Se ele aparece 2×, **meça os dois braços separadamente** antes de concluir.
2. Pergunte *o que torna isto barato aqui?* — mover um `Arc` e copiar um plano são a mesma linha de
   código com preços opostos.
3. ⚠️ **O ramo pode nunca ter sido executado por gate nenhum:** ali TODAS as fixtures usavam traço
   curto, cuja janela não alcança o limiar. Ao gatear, o **controle positivo** pode vir do *outro* sítio
   (a rota que mantém o limiar testemunha que a fixture cruza a condição) — ver
   [[reference_topic_gate_discipline]] e [[reference_topic_fixture_discipline]].
