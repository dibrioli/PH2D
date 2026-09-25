---
name: forcing-a-property-after-the-optimum-is-what-ripples
description: "Impor uma propriedade a seguir a um óptimo por-peça não é resolvê-la dentro dele — o desvio é diferente em cada peça e as vizinhas ficam empurradas para lados opostos, que é o que se lê como ondulação"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ff68af47-4a05-499f-83ce-4bb390d09ae2
  modified: 2026-09-21T00:54:44.533Z
---

⛔⛔⛔ **Um passe que «conserta» uma propriedade DEPOIS de um óptimo por-peça é a causa de ondulação
mais cara que esta casa mediu** (2026-09-20, `ph2d-vec-skin/curva.rs`).

O caso: a deformação vectorial ajusta as duas alças de cada segmento por mínimos quadrados contra a
curva verdadeira. Isso parte a tangente nos nós (`13,6°`), porque cada segmento resolve o seu
sistema sozinho. Escrevi um segundo passe que rodava as duas alças de cada nó para um ângulo comum.
Ele **curava a quina** e era ele, sozinho, a serpentina que o dono fotografou:

| | serpentina p50 | desvio ao padrão-ouro (máx) | quebra da tangente |
|---|---:|---:|---:|
| óptimo **+ o passe** | `0,016016` | `0,00546` | `0,000°` |
| **só o óptimo** | `0,001727` | `0,00341` | `13,65°` |
| o **CHÃO** (o melhor que a representação permite) | `0,001752` | `0,00334` | — |

**O mecanismo, e ele é geral:** o passe pega na solução óptima e **move-a para fora do óptimo** por
um desvio que é diferente em cada peça. Peças vizinhas ficam empurradas para lados opostos ⇒ o
resultado alterna de sinal ao longo do caminho. *Uma correcção correcta aplicada peça a peça, mas
com magnitude livre, é indistinguível de ruído estruturado.*

⭐⭐ **A alternativa certa — resolver o óptimo DENTRO do subespaço em que a propriedade já é
verdade — também foi construída e MEDIDA, e perde:** prender cada alça ao eixo do nó dá `G¹` exacto
e paga `4,6×` de excesso de curvatura (`14,48°` contra `3,16°`). *Porque a VERDADE que se persegue
não tinha a propriedade*: o campo de pesos é `C⁰`, logo a deformação correcta **tem mesmo** pequenos
bicos, e o óptimo livre estava a reproduzi-los.

⇒ **antes de forçar uma suavidade, meça se a VERDADE a tem.** Se não tiver, forçá-la afasta o
resultado do padrão-ouro por construção — e a régua que o revela tem de comparar contra o
padrão-ouro, não contra uma curva ideal imaginada ([[measurement-discipline]]).

⚠️ **E isto refutou a alavanca que eu próprio tinha nomeado na véspera** (*«a cura que falta é um
ajuste com continuidade GLOBAL»*): a cura não era mais continuidade, era **menos** — apagar o passe.
A remoção veio com desconto: `−21 %` de relógio, porque saíram também as consultas de eixo que só
ele lia.
