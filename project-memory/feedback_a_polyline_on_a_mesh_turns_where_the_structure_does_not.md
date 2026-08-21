---
name: feedback-a-polyline-on-a-mesh-turns-where-the-structure-does-not
description: Uma curva de estrutura traçada sobre arestas de malha zigue-zagueia — decidir "aqui há uma quina" pelo ÂNGULO conta o zigue-zague como estrutura
metadata:
  type: feedback
---

Quando uma curva **estrutural** (uma divisa, uma separatriz, um vinco autorado) é
representada como polilinha sobre as arestas de uma malha, ⛔ **não decida
propriedades da estrutura pela geometria local dela.** A polilinha vira onde a
triangulação a obriga a virar, e não onde a estrutura vira.

**Why:** medido em 2026-08-21 (quad remesher, F3). Um canto de patch era decidido
pelo ângulo interno arredondado a quartos de volta. Uma parede que vira 60° num
vértice dá `120°` de ângulo interno → arredonda a **1 quarto** → canto. Censo dos
cantos do layout:

| malha | cantos | singularidade | junção de paredes | ⛔ **artefacto** |
|---|---|---|---|---|
| esfera 96×144 | 52 | 6 | 19 | **27** |
| toro 64×32 | 72 | 8 | 27 | **37** |

⭐ **Metade não tinha estrutura nenhuma por baixo** — eram vértices no INTERIOR de
uma curva (grau de ramificação `2`) — e **todos** viravam vértice defeituoso no
produto final. A lei certa é estrutural: *um canto só pode existir onde a curva se
ramifica.* Corrigido, a contagem de defeitos caiu de **47 para 14**.

⚠️ **A geometria continua a ser necessária, mas como DESEMPATE.** Numa junção em T
os dois lados que ladeiam o pé têm quina e o terceiro tem a fronteira reta: *a
estrutura diz ONDE pode haver canto, a geometria diz para QUEM ele é.*

⛔ **E os artefactos eram load-bearing.** Removê-los sozinho fez a decomposição
colapsar (14 patches → **1, com zero arcos**), porque eram eles que davam a cada
região lados suficientes. ⇒ Quem apertar a regra tem de prever o **degrau** para o
caso em que a estrutura não chega — e **contá-lo**, senão o remendo vira a regra
sem que ninguém decida.

⚠️ **E o degrau é um piso de VALIDADE, nunca um alvo de qualidade.** Subi-lo do
mínimo legal (3) para o desejável (4) tornou o sistema a jusante **inviável** em 2
de 6 fixturas e não melhorou a contagem em nenhuma. *O que a estrutura não dá, o
remendo não inventa.*

**How to apply:** antes de escrever um teste geométrico sobre uma curva discreta,
pergunte que **grandeza estrutural** responde à mesma pergunta — grau num grafo,
identidade de componente, ramificação. Se existir, ela é a porta e a geometria é o
desempate. Irmã de
[[feedback_a_defect_count_without_provenance_names_the_wrong_phase]] (foi a
proveniência que apontou para aqui) e de
[[feedback_documented_decision_chesterton_fence]] — o artefacto era cerca.
