---
name: feedback-a-suite-of-topological-assertions-is-blind-to-geometry
description: 10.515 gates verdes sobre um produto destruído — todas as asserções eram de CONTAGEM, e o defeito era de POSIÇÃO
metadata:
  type: feedback
---

Quando uma fase produz **geometria**, conte quantas das suas asserções olham uma
**coordenada**. Se a resposta for zero, a suíte é cega ao defeito que mais importa —
e ela vai ficar verde sobre um produto inutilizável.

**Why:** medido em 2026-08-21 (quad remesher). O artista clicou no botão e a malha
voltou em lascas, com arestas a atravessar a peça de lado a lado. O log dizia
`100 % quads · casca FECHADA · 22 irregulares`, e **10.515 testes passavam**.

⭐ **Não foi azar: era estrutural.** O relatório da fase era **função pura dos
índices** — `quads` da aridade das faces, `boundary_edges` de um mapa de pares,
`irregular` da valência. As faces saíam da estrutura combinatória a montante.
*Nenhuma posição escolhia um índice* ⇒ embaralhar todas as coordenadas deixava o
relatório **byte-idêntico**. Medido lado a lado, mesmo layout, só a malha de
amostragem trocada:

| | quads | não-quads | bordo | irreg. | **aresta mediana** | **aresta MAX** |
|---|---|---|---|---|---|---|
| destruído | 5 978 | 0 | 0 | 21 | **4,6× o alvo** | **2,01 = o DIÂMETRO** |
| correto | 5 978 | 0 | 0 | 21 | 1,0× o alvo | 0,41 |

E o único `assert` que tocava posições era um `assert_ne!(saída, entrada)` — que
passa trivialmente: *a malha mudou, só que para lixo.*

**How to apply:**

1. **Classifique cada asserção** em TOPOLÓGICA (sobrevive a posições arbitrárias)
   ou GEOMÉTRICA. Um relatório sem nenhuma da segunda espécie não é uma régua de
   geometria, é um contador.
2. **A régua mais barata costuma ser o COMPRIMENTO DE ARESTA contra o alvo** — ela
   já está em memória, é `O(E)`, e não precisa de octree nem de normais.
3. ⚠️ **Use DUAS, não uma.** Com o defeito reintroduzido, numa fixtura a aresta
   **máxima** ficou *debaixo* da barra e quem apanhou foi a **mediana**; noutra foi
   o inverso. *O dano geométrico não escolhe sempre a mesma régua.*
4. ⛔ **Cuidado com a régua TAUTOLÓGICA.** A distância vértice→superfície numa
   direção só dava **0,0000** na malha destruída contra 0,0015 na boa — porque a
   última operação da fase era reprojetar sobre essa superfície. *A malha destruída
   pontuava melhor.* Meça no sentido contrário, ou no interior das faces.
5. ⭐ **Melhor que o gate de saída: a PRÉ-CONDIÇÃO de entrada.** Aqui, o
   comprimento de cada arco medido na malha recebida contra o comprimento que a
   fase anterior declarou — coerente `1,000` exacto, defeituoso `5,40×`. Ela nomeia
   a causa em vez de detectar o sintoma.

Irmã de [[feedback_a_conserved_invariant_cannot_grade_quality]] e de
[[feedback_a_defect_count_without_provenance_names_the_wrong_phase]] — a mesma
família: *o instrumento media uma coisa e a promessa era outra*. E a costura que
ninguém testava é [[feedback_painter_inefficiency_4_causes]] (causa nº 1).
