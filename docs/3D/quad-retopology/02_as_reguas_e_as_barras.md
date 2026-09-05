# 02 — As réguas, as barras, e de onde cada barra veio

> **A lição que este módulo pagou seis vezes:** *cada report do dono foi, primeiro, uma régua
> que não existia.* Nenhuma linha de algoritmo mudou até a medição concordar com a foto — e
> quatro vezes a régua ANTIGA dizia o contrário do que a foto mostrava.

## §1 — O censo: o que é uma PONTA

[`ph2d_quadfill::apices`] é a **única** lei do ápice, e três réguas a consultam. Ela devolve os
máximos locais de raio, filtrados por:

| cerca | valor | por quê |
|---|---|---|
| piso do raio | `0,25 × raio máximo` | ⛔ o piso anterior (`0,55`) **escondia as pontas da foto**: as de que o dono se queixava estavam a `0,43`–`0,47` do raio, e **nenhuma régua desta linha alguma vez as mediu** |
| forma (o **cone**) | `CONE_MAX = 1,0` | baixar o piso sem filtro de forma chamaria «ponta» a cada bossa do corpo (`42` na peça dele) — e as bossas lêem grade `1,0`–`1,47` **na malha que ele aprovou** |
| corte | `MAX_TIPS = 32` | custo (um Dijkstra por ápice), não lei |

O **cone** é o pior `Σ raio / Σ profundidade` das faixas de `2 h` entre `3` e `9 h` do bico, ao
longo do eixo local. *Um espinho é cónico até fundo; um botão ou cúpula salta para o corpo.*

⚠️ **Ele é relativo à RESOLUÇÃO, de propósito** — uma cúpula que a `9 h` ainda é cónica é,
*àquela* resolução, um bico que a grade devia resolver. Uma versão sem `h` foi medida e **não
separa**.

⚠️ **A unidade muda o censo, e por isso ela é fixada por quem chama:** no produto é o **alvo do
slider** (o censo tem de ser o mesmo em todas as candidatas de um clique — senão o selector
compara *«3 pontas más de 8»* com *«2 de 7»*); na bancada é a aresta **mediana** da saída,
porque a saída de outra ferramenta não tem alvo.

## §2 — A tabela por ponta, de que as agregadas são dobras

[`ph2d_quadfill::tip_rows`] devolve **uma linha por ápice**. As agregadas
([`tip_deviation`], [`tip_density`]) são **dobras** dela, e há gate que dobra a tabela à mão e
exige o mesmo número: *uma régua nova que muda o veredito da anterior não é a mesma régua.*

| coluna | o que mede | barra |
|---|---|---|
| `gap` | do **ápice** da escultura à **superfície** da saída (ponto→FACE) | `TIP_GAP_MAX = 0,5` |
| `dev` `p50`/`p90`/máx | dos vértices da entrada a `≤ 3 h` do ápice à superfície da saída | `TIP_DEVIATION_MAX = 1,0` |
| `grade` | a aresta média da saída na bola de `3 h` do bico | `TIP_DENSITY_MAX = 1,0` |
| `shaft` | grade **e** aspecto na faixa `3`–`12 h` — o **corpo** do espinho | — |
| `faces` | o aspecto `p50` das faces do bico, e quantas são | — |
| `cone` | a forma do espinho | `CONE_MAX = 1,0` |
| `pole` | irregulares a `≤ 2 h` e a valência do bico | — |
| `blind` | não há superfície nenhuma junto do ápice ⇒ o número é um **PISO** | — |

## §3 — De onde vem cada barra (e a que custou)

- **`TIP_GAP_MAX = 0,5` — meia célula.** Medido nos DOIS lados que o dono julgou: a
  retopologia que ele **aprovou** (QRemeshify) lê pior `gap 0,19`; as nossas pontas que não o
  incomodaram, `0,31`; as que ele **reprovou**, `1,02` · `1,11` · `3,17` · `4,08` · `10,4`. A
  barra vive no vazio `0,31`…`1,02`.
- **`TIP_DENSITY_MAX = 1,0` — a ponta não recebe um quad mais grosso que o mediano da própria
  malha.** ⛔ A primeira barra (`1,5`) foi calibrada **só com a nossa saída** e deixava passar,
  a `1,10`–`1,40`, exactamente as pontas de que ele se queixava. *Uma barra calibrada sem o
  lado aprovado mede a distância entre os nossos próprios defeitos.*
- **`TIP_DEVIATION_MAX = 1,0` — o CHÃO DA DISCRETIZAÇÃO**, não um número escolhido: uma grade de
  passo `h` não pode seguir uma superfície melhor que `h`.
- **A régua é ponto→FACE, não ponto→vértice**: com vértices a população sã lê `p50 0,28`–`0,35`
  (metade de uma aresta da saída) e com faces lê `0,08`–`0,30`. *Uma régua cujo valor «são» é
  feito do artefacto dela própria não tem onde pôr uma barra.*

## §4 — As réguas globais, e o buraco que cada uma tinha

| régua | o que ela vê | ⛔ o que ela **não** vê |
|---|---|---|
| `χ` (Euler) | topologia fechada | a **almofada** (o mesmo quad emitido duas vezes, um virado) — ela conta os dois lados e dá `2` |
| bordo | arestas com uma face só | o **não-manifold** — por isso `open_edges = bordo + não-manifold` |
| `edge_max` / `edge_median` | extremos globais | o quad de `0,02 × 0,30`: nenhuma das duas se move |
| `QuadShape` (aspecto, enviesamento, `>60°`) | a forma de cada face | **qual ponta** — uma mediana sobre milhares não vê três quads emaranhados num bico |
| `ENTREGA` (`tip_body_ratio`) | cinco coroas radiais, média das pontas | a ponta que colapsou — cinco pontas certas afogam uma, e ela imprimia `0,553` (*«afina na ponta»*) sobre a peça da foto |
| `reach` (alcance) | até onde a peça vai | ⚠️ com o centroide dos **vértices** ela media a AMOSTRAGEM e tinha o **sinal invertido**; hoje o centroide é o da **ÁREA** |
| `tip_rows.grade` | a aresta média no bico | a forma: um quad de `2 h × 0,5 h` tem aresta média `1,25 h` |
| **o anel** (quantas faces dão a volta) | o que o olho lê como densidade | — (é a régua mais nova, 04/09) |

⭐⭐⭐ **O padrão, escrito uma vez para não voltar:** *um extremo ou uma média sobre a peça
inteira nunca vê UM item — e, quando só um está mau, ela também não diz QUAL.* Foi essa a forma
do `edge_max`, do `χ`, da `ENTREGA` e das três réguas de ponta agregadas.

## §5 — A barra do ORÁCULO, e o dia em que ela foi lida a 1/9 da densidade

A comparação com o `quadwild-bimdf` (o oráculo, **fora da árvore** — ADR-0167) corria com
`370`–`576` quads contra os `3 352`–`4 696` da saída dele: **mais fina é mais fácil**. À
densidade dele, a mesma cadeia sem uma linha mudada deu `3,8°`–`6,5°` de enviesamento — dentro
da barra (`4,8°`–`7,1°`) — e *uma semana de trabalho tinha perseguido um buraco da RÉGUA*.

⇒ **toda comparação com o oráculo nomeia a contagem de faces dos DOIS lados**, e o
`piece_report` imprime-a.
