---
name: feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless
description: Julguei uma cura pela peça mais ATÍPICA, ela não continha o defeito que a cura ataca, e eu quase apaguei 400 linhas certas
metadata:
  type: feedback
---

A lei *"a fixtura tem de conter o fenómeno"* tem um **segundo gume**, e é o que
mata trabalho bom: uma fixtura que **não** contém o fenómeno faz a cura certa ler
como inútil. ⇒ **Antes de rejeitar uma cura, meça a fração do defeito que ela
sequer alcança.**

**Why:** medido em 2026-08-21 (quad remesher). Construí a parametrização por patch
— achatamento de Tutte + coordenadas de valor médio — para curar faces dobradas, e
medi-a na `hooked_sphere`, a peça que reproduzia a foto do artista. Resultado: **as
dobras não caíram** (17 → 18). Escrevi a nota de recusa e ia apagar o ficheiro.

⭐ **O que salvou foi correr o gate da CRATE**, que mede outras três malhas:

| fixtura | antes | com a cura |
|---|---|---|
| esfera 48×72 | 25,9 % | **0,0 %** |
| esfera 24×36 | 12,2 % | **1,8 %** |
| ⛔ `hooked_sphere` | 6,9 % | 6,9 % ← *a peça em que eu media* |

E a razão estava num instrumento que eu tinha acabado de construir: a
**proveniência** dos vértices das faces dobradas. Na `hooked_sphere`, das 28 pontas
de face dobrada, **1** era do interior de grade — que é a única coisa que a cura
reconstrói. As outras 27 eram de arco, canto e raio. *A peça mais difícil do corpus
era a que menos continha o fenómeno.*

⚠️ **E a cura estava a 3 linhas de funcionar em toda a parte.** O centro do leque
estava na origem do domínio em vez do centróide dos cortes; corrigido, a 48×72 foi
de 2,1 % a **0,0 %** e a aresta máxima de `24,81×` a **`4,24×`** o alvo. *Eu ia
rejeitar a versão meio-feita de uma coisa certa.*

**How to apply:**

1. ⭐ **Meça a FRAÇÃO ALCANÇÁVEL antes do resultado.** *"Quantos dos defeitos que
   existem hoje esta mudança sequer pode tocar?"* Se a resposta for 4 %, um
   resultado nulo não diz nada sobre a cura.
2. **Uma cura julga-se no CORPUS, nunca numa peça** — e sobretudo não na peça mais
   extrema, que costuma ser dominada por outro mecanismo.
3. ⚠️ **Um resultado nulo pode ser uma implementação incompleta.** Antes de
   escrever a recusa, pergunte que parte da construção ainda é a antiga; aqui era
   um único ponto (o centro do leque) que continuava a ser escolhido no espaço.
4. ⭐ Se rejeitar mesmo assim, **rejeite com a tabela** — a recusa medida vale mais
   que o código, e é o que impede reconstruí-la ([[feedback_archiving_without_indexing_the_refusals_deletes_them]]).

Irmã de [[feedback_a_defect_count_without_provenance_names_the_wrong_phase]] (foi
a proveniência que explicou o zero) e de
[[feedback_a_suite_of_topological_assertions_is_blind_to_geometry]].
