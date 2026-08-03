---
name: feedback-dropping-ownership-blinds-every-comparator-that-reads-that-side
description: Tirar a posse de um dado de um lado do sistema faz TODO comparador que lê aquele lado passar a mentir — varra os detectores antes de medir o ganho
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 39ec3808-26ec-4cf4-b80e-b2291882bc64
  modified: 2026-08-02T00:47:32.618Z
---

Quando uma otimização faz um componente **deixar de segurar** um dado (elidir, delegar, mover para um
handle fraco), todo lugar que **compara** aquele lado com outro passa a comparar *ausência* com
*presença* — e responde "mudou" para sempre.

**Caso medido (PH2D, degrau 4 do S3, 2026-08-01):** o `cursor` do undo passou a DESCREVER os planos de
relevo em vez de os segurar. O detector de escritas estrangeiras (`absorb_foreign_writes`) compara o
cursor com o estado que chega, e passou a ver toda camada como *"apareceu agora"* ⇒ **disparava em todo
pen-down**, fazendo um re-split completo por traço: **pen-down 5,7 → 36,2 ms a 4096²**, o que teria
tornado a wave uma perda líquida.

**Por que é insidioso:** o comparador continua *correto no seu próprio termo* e a suíte fica **verde** —
o que muda é só o custo, e só na tela grande. Ninguém o atribui ao lado que largou a posse.

**Why:** posse e comparação parecem eixos independentes e não são. Um comparador é uma afirmação sobre
DOIS estados materializados; retirar a materialização de um deles muda o que ele afirma, em silêncio.

**How to apply:**
1. Antes de medir o ganho, **grepe quem compara aquele lado** (`split`/`diff`/`==`/`ptr_eq` sobre o
   campo que perdeu a posse) e decida, por comparador, se ele ainda pode testemunhar.
2. Quem não pode, **sai do detector com o porquê escrito** — não é atalho: um lado que não segura o
   dado não tem como falar sobre ele.
3. O gate desse comparador **não pode observar tamanho nem profundidade** se a operação faz pop+push:
   [[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]] — a pergunta é
   *"ela disparou?"* e a forma honesta é **CONTAR** (um contador `cfg(test)`, o idioma que este repo já
   usa para o `RELIEF_FROM_JOURNAL`).
4. E a hipótese bonita não substitui a refutação: a explicação natural aqui (o alocador reciclando os
   ~200 MB liberados, mecanismo que o próprio arquivo documentava, com escala e aritmética fechando)
   foi **REFUTADA por medição** em cinco linhas de sonda. Ver [[reference_topic_repro_discipline]].
