---
name: feedback-an-axis-aligned-fixture-cannot-measure-a-basis
description: "Uma fixtura alinhada aos eixos não mede uma BASE — os termos fora da diagonal são zero, e um erro de convenção (y para cima × v para baixo) passa nas duas metades e só aparece no produto rodado"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-14T19:59:24.476Z
---

Medido em 2026-09-14 (W11/W11b do plano `docs/Skeleton/03`). O pincel tinha de pintar uma elipse que
a deformação da malha endireita, para sair **redondo no ecrã**. Duas leis, em duas crates:

1. a malha publica a deformação local (`2×2`, adimensional);
2. o pincel decompõe `W⁻¹·E` nos três números que ele consome.

As duas tinham gate. As duas passavam. E o dono devolveu a **mesma foto**: *«sem melhorias»*.

⛔⛔ **A matriz nascia numa base MISTA** — as *linhas* em coordenadas locais (`y` para CIMA) e as
*colunas* em coordenadas de imagem (`v` para BAIXO). Conjugar pelo espelho do `y` **nega os termos
fora da diagonal** (`E = D·T·D`): numa deformação **diagonal** `E = T` e não se vê nada; numa
**rodada** a elipse sai espelhada, esticada na diagonal errada.

⚠️ **As fixturas das duas metades eram alinhadas aos eixos** (uma só transladava, a outra comprimia
em `x`). Ali os termos fora da diagonal são **zero**: a fixtura não tinha como distinguir as duas
convenções.

**Why:** um erro de BASE (orientação, mão, ordem linha/coluna) vive inteiramente nos termos fora da
diagonal. Toda fixtura «simples» — transladada, escalada, alinhada — os põe a zero, e por isso é
exactamente a fixtura que não mede a coisa que mais se erra. E como cada metade passa, ninguém
suspeita da junção: *duas metades verdes não fazem uma junção verde*.

**How to apply:**
- ao gatear qualquer matriz/transformação, a fixtura tem de ser **rodada ou cortada** (com termos
  fora da diagonal não nulos), e de preferência **construída a partir da resposta**: escolha a
  transformação que quer, derive a geometria que a produz, e meça a IDA e a VOLTA;
- gateie a **JUNÇÃO**, não só as metades — se as duas leis vivem em crates diferentes, o gate vai
  para a crate que pode chamar as duas (um `dev-dependency` para a lei canónica é mais barato que
  re-implementá-la: foi o que se fez aqui);
- e escreva a convenção no tipo ou no doc (*«linhas em ecrã, colunas em imagem»*): uma `[[f32;2];2]`
  não diz em que base está. Ver [[feedback-a-gate-anchored-on-a-byte-distance-is-a-proxy-that-expires]]
  e [[reference-topic-gate-discipline]].

---

## ⚠️ A volta SEGUINTE, medida em 2026-09-14 (W12): **uma SONDA SIMÉTRICA não mede a base**

A cura acima pôs a fixtura **rodada**. Na wave seguinte a mesma mutação (trocar a base do ajuste por
um espelho) **sobreviveu outra vez** — e desta vez a fixtura estava rodada.

⛔⛔ **O que faltava era do lado do CONSUMIDOR, não da fixtura:** o gate media a marca de um pincel
**REDONDO**. Trocar a base multiplica a matriz por um factor **ORTOGONAL**, e um factor ortogonal tem
os **mesmos valores singulares e os mesmos eixos** ⇒ sobre o círculo unitário a resposta é a mesma
**ao bit**. O erro existe, é real no produto, e é literalmente invisível àquela sonda.

**How to apply (acrescento):** ao gatear uma transformação, a sonda tem de ser **assimétrica** —
aqui, uma elipse **autorada** (`flatten 0,4`, `angle 30°`), que separou `29,8°`–`38,3°` (certo) de
`152,6°`–`164,2°` (espelhado). *Uma fixtura rodada com uma sonda redonda ainda não mede uma base:
o que se pergunta tem de distinguir a orientação.*
