---
name: a-normalising-law-needs-a-quantity-invariant-to-free-motion
description: "Uma lei que iguala uma grandeza MEDIDA exige que ela seja invariante a tudo o que a figura pode fazer de graça — senão a lei normaliza o movimento livre em vez do efeito"
metadata:
  type: feedback
---

**Medido na `line/motion-value`, 2026-08-30 (report do Enio: *"em dragon enquanto cresce
parece piscar"*).**

A lei do crescimento do L-System põe o TAMANHO da figura numa rampa recta, medindo-o por
`max(w, h)` da caixa alinhada aos eixos. A curva do dragão **roda 45° por geração** por
construção. ⇒ quando a caixa trocava de lado longo, a lei passava a fixar a OUTRA dimensão:
o tamanho verdadeiro **estagnava e depois arrancava** (menor passo do arrasto a `4,5 %` do
passo médio, contra `55,2 %` depois da cura).

**Why:** *a lei não estava errada — a GRANDEZA estava.* Uma lei que iguala `f(figura)` a uma
rampa só é uma lei sobre a figura se `f` for invariante a tudo o que a figura pode fazer sem
mudar: rotação, translação, e a mudança de AMOSTRAGEM (aqui a contagem de elementos duplica ao
atravessar uma geração). Duas réguas invariantes à rotação foram medidas e **rejeitadas** por
falharem a terceira condição: o raio de giração e a maior distância ao centroide são medidas de
DISTRIBUIÇÃO, e o centroide salta quando a contagem duplica (Tree: passo `−7 991 %` do médio).
A que fica é a **largura média de Cauchy** — `média_u(max⟨P,u⟩ − min⟨P,u⟩)` sobre `K` direções —,
que é um EXTENSO sem centroide.

**Corolário que custou uma correcção:** o gate estava verde, e a 1.ª explicação («a régua dele
era cega») foi **refutada por mutação**. O observador do gate era a DIAGONAL da caixa, e com o
produto CURADO ele **reprova** (lê `25,6 %` onde o honesto lê `55,2 %`). *Uma régua da mesma
família do defeito não é cega — é falsa acusadora, e as duas leem-se igual num gate verde.*

**How to apply:** antes de escrever `alvo/medido`, pergunte *o que é que este objecto pode fazer
sem mudar, e a minha grandeza sobrevive a isso?* Rotação · translação · re-amostragem. E ao
escolher o observador de um gate, verifique que ele não condena o produto certo.
Relacionado: [[feedback_a_ruler_normalised_by_what_the_cure_zeroes_measures_it_backwards]] ·
[[reference_topic_gate_discipline]].
