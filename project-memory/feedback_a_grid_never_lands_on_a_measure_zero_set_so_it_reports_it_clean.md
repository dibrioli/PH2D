---
name: feedback-a-grid-never-lands-on-a-measure-zero-set-so-it-reports-it-clean
description: Uma sonda que amostra por GRELHA e depois parte o corpus por uma condicao que vive num conjunto de medida nula reporta essa populacao LIMPA — os pontos poem-se, nao se procuram
metadata:
  type: feedback
---

⛔⛔ **Uma sonda que amostra por GRELHA e depois PARTE o corpus por uma condição que só vale num
conjunto de medida nula lê essa população como LIMPA — e o veredito sai ao contrário.**

Medido 2026-09-10 (`line/3DModeling`, W147). A pergunta era se o gradiente **analítico** da `fidget`
podia substituir a diferença central `f64` nas réguas do modelador. As duas respostas divergem por
construção **num vinco** (ali a derivada não existe: a diferença central devolve a MÉDIA dos dois
gradientes laterais, o analítico escolhe UM ramo). A sonda varria uma grelha junto da superfície e
separava «liso» de «vinco» pela segunda diferença:

| região | pontos | mediana | pior |
|---|---:|---:|---:|
| liso | 2 323 | `1,101e-13` | `1,567e-7` |
| vinco (**achado por grelha**) | 122 | `2,174e-6` | `3,058e-6` |
| vinco (**POSTO na aresta**) | 360 | **`1,876e-1`** | **`1,876e-1`** |

Folga do módulo: `2,0e-2`. ⇒ a grelha reportou a população do vinco **~60 000× mais limpa do que ela
é**, e a primeira redacção do veredito dizia *«o bloqueio dissolve-se»*. A leitura certa é a oposta:
a discordância é **`9,4×` a folga inteira**, e a cura publicada fica **recusada**.

**Why:** um vinco é uma **superfície** dentro de um volume — medida nula. Com passo `0,041` e
`eps = 1e-4`, a chance de um ponto da grelha cair a menos de `eps` de uma aresta é ~`0,5 %`; os `122`
que o detector apanhou eram, na prática, **curvatura**, não vincos. *O filtro não estava errado — a
população que ele filtrava é que nunca continha o fenómeno.* É a terceira leitura de
[[feedback-a-fixture-where-the-two-are-siblings-cannot-produce-a-cycle]]: **a fixtura não produz o
fenómeno**.

**How to apply:** quando uma hipótese diz *«X e Y divergem NA CONDIÇÃO C»*, pergunte primeiro **de
que dimensão é o conjunto onde C vale**. Se for menor que a do domínio amostrado, ⛔ **não filtre uma
grelha: CONSTRUA os pontos.** Escolha uma peça cuja condição se **escreva** (aqui: uma esfera a
atravessar a face de cima de uma caixa ⇒ o vinco é a circunferência da intersecção, com equação
fechada) e amostre **em cima** dela. E mantenha as DUAS populações na tabela — é a linha da grelha ao
lado da linha posta que mostra o tamanho da cegueira.

⚠️ Sinal de alarme barato: **a população suspeita saiu pequena** (`122` de `2 445`) **e limpa**.
Pequena-e-limpa quase nunca é «não há defeito»; é «não amostrei aquilo».

Ver [[reference-topic-measurement-discipline]] · [[feedback-a-bar-calibrated-without-the-approved-side-measures-our-own-defects]]
