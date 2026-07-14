---
name: feedback-an-anchor-is-not-a-feature
description: Casar duas formas por ÂNCORA força uma rotação — num contorno suave a âncora é artefato da parametrização, não geometria que alguém autorou
metadata:
  type: feedback
---

Correspondência de formas (Blend/morph). O motor casava **âncora com âncora**. O Enio: *"o porquê
da rotação?"* — um quadrado virando círculo **girava 45°** no caminho.

**Medido:** as quinas do quadrado estão a −135°/−45°/45°/135°; as 4 âncoras do círculo, a
0°/90°/180°/270°, todas com virada `(sen, cos) = (0, 1)` — **perfeitamente suaves**. Elas existem
só porque a elipse é cozida em 4 cúbicas. **O artista nunca as autorou.** Forçado a casar quina com
âncora, o motor escolheu o melhor de quatro casamentos ruins: 45° de giro.

**Why:** a resposta certa (a quina a 45° casa com o ponto do círculo a 45°, **no meio de um
segmento**) **não estava no conjunto de candidatos**. Não era bug de implementação — era o
*conjunto de candidatos* estar errado.

**How to apply:**
1. **Âncora ≠ feature.** Feature é o que **sobrevive a um refit**: uma quina (virada acima de um
   limiar), um extremo de curvatura. Uma âncora de contorno suave é **parametrização**, e casar por
   ela injeta uma rotação que ninguém pediu.
2. **Quando NENHUMA das formas tem feature** (dois círculos, dois blobs), a correspondência é uma
   **fase contínua** — e um conjunto de candidatos discreto (âncora↔âncora) não a alcança. Varra a
   fase, não as âncoras.
3. **O custo precisa de POSIÇÃO e de FORMA.** Só com posição, uma quina **convexa** casa com um
   vértice **reentrante** (medido: 2 das 4 quinas do quadrado casaram com os VALES da estrela) — e
   a forma **colapsa pelas quinas** enquanto as pontas nascem do meio das arestas. O termo de
   *bending* (Sederberg & Greenwood 1992) é o que separa: convexa com convexa é diferença de
   **grau**; convexa com reentrante é diferença de **TIPO**. Compare a virada como o par
   `(sen, cos)` — dois vetores unitários, monótono no ângulo, **sem `atan2`** (HR-5).
4. **Um escape manual que às vezes é inerte é pior que escape nenhum.** O botão que rotaciona a
   correspondência re-decidia o SENTIDO junto; numa forma simétrica os dois se cancelavam e o botão
   **não fazia nada** — o artista conclui que a ferramenta travou. Escape rotaciona; o sentido é
   outra decisão.

Relacionadas: [[feedback_oracle_must_model_appearance_not_implementation]] ·
[[feedback_ergonomics_verdict_is_a_design_bug]]
