# Corpus do PENTE DE TOPOLOGIA — proveniência e a RÉGUA

> ⚠️ **O CABEÇALHO DE CADA FICHEIRO É A FONTE; esta prosa nunca é.** Uma grandeza nova que não
> entre no cabeçalho esconde-se debaixo da frase que descreve o corpus, e **nenhuma varredura a
> acusa**.

## §1 — Proveniência

| | |
|---|---|
| **Entradas** | **NOSSAS**, geradas pelo harness: um plano `28×28` células de lado `2,4`, triangulado com diagonais **alternadas** e com abanão determinístico de `0,55` da célula nos vértices interiores (a entrada canónica, `entrada_9fb3d9dea0d0.txt.gz`); mais um plano de **quads**, um plano com **furo** (bordo aberto interior) e uma esfera UV. ⛔ Nenhum asset do alvo entra como entrada. |
| **Saídas** | Lista de posições e de faces devolvida pelo alvo, corrido **sem interface** por script sobre as entradas acima. Saída de programa é **dado** (GPLv2 §0). |
| **Alvo corrido** | binário **5.2.2 LTS**, pacote `17:5.2.2-1`, build **2026-09-15**. ⚠️ **As obras anteriores desta linha correram `5.2.1-2`** — este corpus e os delas **não vêm do mesmo binário**, e é por isso que a versão está no cabeçalho de cada célula. |
| **Data da colheita** | 2026-09-17 |
| **Quem colheu** | o subagente **E**; o harness vive **fora da árvore** e não viaja com ela. |

Cada célula é `<família>/<nome>.txt.gz`, com o cabeçalho `# chave=valor`, depois o percurso do cursor
(uma linha `# ponto x y z` por ponto), depois `V n` + `n` posições e `F m` + `m` faces.
A **entrada** de cada célula é nomeada no cabeçalho (`malha_de_entrada=`) e vive em `entrada/`.

⚠️ Os valores como `DRAW`, `SMOOTH`, `CONSTANT`, `SUBDIVIDE_COLLAPSE` são **identificadores públicos
de enumeração** da interface do alvo, presentes só porque são a **chave de regeneração** da célula —
é por eles que uma corrida futura reproduz o mesmo enquadramento.

## §2 — A RÉGUA, e porque ela é esta

A grandeza desta obra é a **direcção do fluxo de arestas**, não a posição. Para cada aresta cujo
ponto médio caia dentro da pegada, mede-se `α ∈ [0°, 90°]`, o ângulo entre a aresta e a direcção do
traço, ambos projectados no plano do ecrã (uma aresta não tem sentido, logo o ângulo dobra-se). A
régua é o **parâmetro de ordem de quatro dobras**:

```
Q = média( cos 4α )           Q = +1  toda aresta a 0° OU a 90° do traço  (uma GRADE alinhada)
                              Q =  0  isotrópico
                              Q = −1  toda aresta a 45° do traço
```

⛔⛔ **A régua ÓBVIA — a de duas dobras, `S = média(cos 2α)` — é CEGA a este fenómeno, por
construção**, e foi a primeira que se escreveu. Ela vale `+1` a 0° e `−1` a 90°, logo uma grade
alinhada, que tem as duas famílias, dá-lhe `≈ 0`. Medido nas quatro rotações do traço:

| rotação do traço | `S` desligado | `S` ligado | `Q` desligado | `Q` ligado |
|---|---|---|---|---|
| 0° | +0,0412 | +0,0595 | −0,0456 | **+0,1243** |
| 22,5° | +0,0327 | **+0,0215** | −0,0154 | **+0,0901** |
| 45° | +0,0157 | +0,0368 | +0,0273 | **+0,1491** |
| 67,5° | +0,0535 | +0,0547 | −0,0123 | **+0,0940** |

*A régua de duas dobras **troca de sinal** entre rotações do mesmo traço (a `22,5°` ela DESCE) e o
seu `Δ` é `5×` a `90×` mais pequeno; a de quatro separa nas quatro rotações, sempre no mesmo sentido
e nunca por menos de `+0,10`.* ⇒ **a régua escolheu-se MEDINDO**, e o que ela mede é uma **grade**, não uma
direcção só.

⚠️ **A PEGADA É UM DISCO DE ECRÃ, e medi-la em 3D dá `None`.** A primeira redacção media a distância
do ponto médio ao percurso em três dimensões. Com duas passagens o relevo sobe a `0,24` e a região
de raio `0,175` fica **vazia** — a régua devolvia *«não há arestas»* sobre uma célula perfeita. As
fixturas planas são vistas de topo em projecção ortográfica, logo **a projecção XY é o ecrã**.

⚠️ **A região usada nos números abaixo é `raio/2`** e não o raio inteiro. Medido: a `raio/2` a
separação é `2`–`3×` maior (0° ligado: `+0,0353` no raio inteiro contra `+0,1243` a `raio/2`; desligado, `−0,0632` contra `−0,0456`) —
**o efeito concentra-se na faixa central do traço**, e o raio inteiro dilui-o com a orla.

⛔ **A régua é PLANAR.** Numa esfera ela lê o mesmo valor com o pente ligado e desligado
(`+0,0453` nos dois) enquanto a malha mede `1 917` faces diferentes: ali as arestas junto da
silhueta projectam curtas e o ângulo projectado não é o ângulo na superfície. ⇒ **dívida NOMEADA:
uma régua para malha curva projecta a aresta e a direcção do traço no plano tangente local.** Os
números deste corpus valem para as fixturas **planas**.

## §3 — O VALE, e de onde sai a barra

⛔ **A população tem de ser UMA COISA.** A primeira tentativa varreu «tudo o que tem detalhe
constante» e devolveu um vale **invertido**, porque lá dentro estavam células de outra malha de
entrada, células sem passe de refino e a esfera. A população é:

> a malha de entrada `9fb3d9dea0d0` · topologia dinâmica **armada** · detalhe `CONSTANT` ·
> refino `SUBDIVIDE_COLLAPSE` · resolução `18` · **uma** passagem · percurso inteiro ·
> e o **controlo**: o verbo subdividiu (`v_saída > 1,5 × v_entrada`).

| | n | o pior da população | célula |
|---|---|---|---|
| **pente desligado** | 37 | `Q = +0,0298` (o MAIOR) | `escada/k_a0450_p0000` |
| **pente no máximo** | 37 | `Q = +0,0632` (o MENOR) | `verbos/v_blob_p100` |

**VALE = `[+0,0298 , +0,0632]`**, largura `0,0334`.
⭐ **Os dois lados são saída do PRÓPRIO alvo** — o lado «aprovado» é o alvo com o pente ligado —,
que é a condição que duas barras do corpus do tecido desta casa foram retiradas por não cumprir.

**BARRA = `Q ≥ +0,0465`** (o meio do vale) para *«o pente está no máximo»*, e `Q ≤ +0,0298` para
*«o pente está desligado»*.

⚠️ **A barra separa DESLIGADO de MÁXIMO, não desligado de qualquer posição.** A meio curso os
valores atravessam o vale (ver a escada no `SPEC`), e é isso que a torna honesta.

## §4 — O ORÁCULO NÃO REPETE, e a banda está medida

Seis corridas da **mesma** célula (`banda/b_con*`):

| | v de saída | `Q` | amplitude |
|---|---|---|---|
| **pente desligado** | `2525` nas seis | `−0,04555619` .. `−0,04555623` | `4e-8` |
| **pente no máximo** | `2444` .. `2448` | `+0,11428` .. `+0,12542` | **`0,0111`** |

⇒ **vale / banda = `3,0`.**

⛔⛔ **E a primeira leitura desta não-repetição estava ERRADA, de uma forma que vale para toda
bancada futura desta casa.** Comparando as duas corridas **por índice**, `522` de `~4 200` arestas
«diferiam» com o pente **desligado** — e a contagem de vértices, a contagem de arestas da região e o
`Q` eram idênticos a sete casas. A causa: **o passe de refino emite a mesma geometria com os
vértices por OUTRA ORDEM**. Ordenadas as posições, `10` de `2 525` diferem (`0,4 %`).
⇒ *uma comparação por índice mede a ORDEM, não a malha* — e é por isso que **toda régua deste corpus
é invariante à ordem**. Com o pente ligado a contagem de vértices ela própria muda (`2444`–`2448`),
e aí a variação é real.

⭐ **No regime SEM passe de refino o alvo é determinístico nos dois lados**: o conjunto de arestas é
idêntico entre corridas e `Q` repete à 4.ª casa. É esse o regime em que as leis de mecanismo da
`composicao/` e da `mecanismo/` foram fixadas, e é por isso que elas se podem cobrar **exactamente**.

## §5 — As famílias

| pasta | células | o que ela responde |
|---|---|---|
| `entrada/` | 14 | as malhas de entrada, todas nossas |
| `porta/` | 6 | a **pré-condição**: com a topologia dinâmica desarmada a saída é **byte-idêntica** com o pente a `0` e a `1` |
| `escada/` | 21 | a **escada do botão** — 7 posições × 3 rotações do traço |
| `rotacao/` | 10 | o **discriminador da direcção**: 4 rotações do traço, mais passagens repetidas |
| `verbos/` | 42 | o **censo**: 21 verbos do catálogo × pente `0`/`1`, cada um com o controlo *«o verbo agiu?»* |
| `composicao/` | 26 | como compõe com o refino, a resolução, o modo de detalhe, a simetria; e os degenerados de direcção e de malha |
| `mecanismo/` | 30 | o regime **determinístico** (sem passe de refino): é aqui que se prova o que o pente faz |
| `banda/` | 18 | a repetibilidade, nos dois regimes |

⚠️ **Cada célula do censo de verbos tem o CONTROLO dentro**: o número de vértices que o próprio
verbo moveu contra o repouso. Sem ele, um verbo **inerte** lê-se exactamente como um verbo que
ignora o pente — e duas células deste corpus (a torção com um percurso recto, e a esfera sem passe
de refino) foram apanhadas assim, **medidas como inertes e refeitas** com a fixtura que contém o
fenómeno (uma varredura angular; o passe armado).
