---
name: a-geometric-ruler-and-a-pixel-ruler-disagree-and-the-pixel-one-decides
description: "Num LOD, a régua GEOMÉTRICA (desvio da silhueta) e a do PIXEL medem grandezas diferentes — e só a do pixel descreve o que o olho vê"
metadata:
  type: feedback
---

Ao medir se uma simplificação de forma é invisível (LOD do `source.shape`, 2026-09-22), a régua
geométrica — Hausdorff entre as duas silhuetas, normalizado pelo lado — deu **«invisível abaixo de
`4,4 px`»**. A régua do **PIXEL** (rasterizar as duas ao mesmo tamanho e contar níveis de 255)
refutou-a: a `4 px` ainda há `26,9` níveis, a `2 px` há `3,0`, e **nunca** cai abaixo de `1`.

**Why:** o desvio da SILHUETA e o que sobrevive à GRELHA do ecrã não são a mesma grandeza. Um
desvio pequeno concentrado numa ponta fina muda a cobertura de um pixel inteiro; um desvio grande
espalhado por uma borda suave desaparece no anti-serrilhado. *A régua geométrica responde «as duas
formas são parecidas?»; a pergunta de um LOD é «as duas IMAGENS são a mesma?».*

E a diferença muda a CURA, não só o número: com a régua do pixel, *simplificar a forma* fica
refutado (erra `7`–`255` níveis) e *substituí-la por uma REAMOSTRAGEM dela* — uma tile com mipmap —
erra `< 1` nível abaixo de `4 px`. **Uma troca de forma por forma nunca é invisível; uma troca de
forma pela sua média local é.**

**How to apply:** ao decidir qualquer LOD, simplificação ou aproximação visual, a régua é
**rasterizar e comparar píxeis**, nunca a distância entre as geometrias. Ela é barata na CPU (~30
linhas de point-in-polygon com supersampling), não precisa de placa, e leva CONTROLO obrigatório
nos dois lados: a forma contra si mesma (tem de ler `0`) e o supersampling dobrado (se uma coluna
se mexer, é ruído da régua — ver [[a-ruler-cannot-have-the-order-of-magnitude-of-what-it-measures]]).

⚠️ E a barra de uma tile **não é o lado dela**: a de `28 px` erra `17,9` níveis a `28 px` e `0,4` a
`4`. Quem a fixa é a REAMOSTRAGEM, não a resolução guardada — e a medição só vale se o consumidor
tiver a cadeia de mips e amostragem trilinear, o que se verifica antes de escrever o número.

[[reference-topic-measurement-discipline]] · [[feedback-perfection-no-deferrals]]
