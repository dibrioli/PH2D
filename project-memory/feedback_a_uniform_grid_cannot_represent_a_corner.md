---
name: feedback_a_uniform_grid_cannot_represent_a_corner
description: "Uma tabela amostrada uniformemente converge com 1/n numa ESQUINA e não converge de todo num DEGRAU — quando o erro máximo não cai com a densidade, o defeito é da representação ou da régua, nunca da resolução"
metadata:
  type: feedback
---

Levando a rampa de cor do artista ao halo (doc 89 folha 11), a tabela mediu assim:

| amostras | erro máximo (rampas suaves) |
|---|---|
| 16 | 0,100 |
| 64 | 0,024 |
| 256 | 0,006 |

**Cai com `1/n`, não com `1/n²`** — a assinatura de uma **esquina**. Uma reconstrução
linear é exacta numa recta e erra `O(h)` num ponto anguloso, e uma parada de gradiente
é precisamente isso. ⇒ *o defeito não era a resolução, era a representação*: oito
paradas num uniforme nunca chegam; a tabela tem de ser uma **textura filtrável**, onde
os texels são baratos.

E depois, com o `Constant` (um DEGRAU) no corpus, o erro máximo parou de cair de todo:
`16` texels davam `0,998` e `1024` davam `0,834`. A régua dizia que **nenhuma**
resolução servia.

**Why:** num salto, o erro máximo é metade do salto e é **independente da densidade**.
O que a densidade encolhe é a **LARGURA** da banda errada — e é ela que decide se
aquilo aparece no ecrã. *Um extremo global e uma fracção do percurso respondem a
perguntas diferentes, e sobre uma descontinuidade só a segunda é sobre o produto.*

**How to apply:**
1. Ao escolher a resolução de uma tabela, meça **duas** colunas: o erro máximo **e** a
   fracção do domínio em que ele passa da barra. Uma só delas mente sobre uma das
   duas classes de função.
2. **Leia a taxa de convergência antes de aumentar o número.** `1/n` = há uma esquina
   e a representação é linear; nenhuma convergência = há um salto, e mais resolução só
   estreita a banda. Só `1/n²` justifica «é só pôr mais».
3. **O corpus tem de conter o pior caso que a UI oferece.** Os quatro presets da casa
   nascem todos em `Linear`; o editor oferece cinco interpolações, e foi a que faltava
   que mudou o veredito. [[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]
4. Quando a limitação é estrutural, **registe-a num gate** em vez de a esconder: quem
   vier acrescentar texels a pensar que «ainda está errado» tem de encontrar a prova de
   que a banda é de UMA célula, e que uma célula é mais fina que um passo do ecrã.

*Terceira vez nesta linha que a régua se corrige antes do algoritmo — irmã de
[[feedback_the_first_crossing_of_a_resonant_response_is_not_the_boundary]].*
