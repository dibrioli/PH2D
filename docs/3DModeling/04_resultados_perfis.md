# W3 — Os perfis vêm do editor vetorial: o que foi MEDIDO (2026-08-19)

> A wave que faz o desenho da caneta virar sólido. `Extrude` e `Revolve` sobre uma figura 2D cozida
> do `VecPath` — [ADR-0161](../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md).
>
> **Ler antes:** [`03_plano_implicito.md`](03_plano_implicito.md) (o plano vivo) ·
> [`01_resultados_spike.md`](01_resultados_spike.md) (as medições da W0, que são o baseline daqui).

---

## §1 — O que existe agora

| Crate | O que é |
|---|---|
| [`ph2d-field`](../../crates/ph2d-field/) | `Profile` (contornos fechados + regra de preenchimento + **a tolerância com que foi cozido**) e as duas primitivas: `Extrude`/`Revolve`. `FIELD_DOC_VERSION` = **2** |
| [`ph2d-field-eval`](../../crates/ph2d-field-eval/) | [`profile.rs`](../../crates/ph2d-field-eval/src/profile.rs) — a distância 2D com sinal como **árvore**, e as duas formas |
| [`ph2d-field-profile`](../../crates/ph2d-field-profile/) | ⭐ **A costura**: `VecPath` → `Profile`. É a única crate que conhece os dois documentos |

Smoke: cena **`=4`** (cantoneira desenhada, extrudada e furada) e **`=5`** (o torno: o mesmo tipo de
contorno girado em torno de Y).

---

## §2 — A parte que exigiu derivação: o sinal, sem `if`

Um winding number é um `for` com um `if` e um acumulador. Uma árvore de avaliação **não tem `if`** —
tem `compare`, que devolve −1/0/+1. A tradução, por aresta `a→b` e ponto `(u,v)`:

```text
acima_i = max(compare(y_i, v), 0)      1 se o vértice i está acima do ponto
dir     = acima_j − acima_i            +1 subindo · −1 descendo · 0 sem cruzar
cross   = e_x·w_y − e_y·w_x            de que lado da aresta o ponto está
hit     = max(compare(dir·cross, 0), 0)  1 sse cruza E o raio +x o alcança
```

⭐ **`dir · cross > 0` casa os dois sentidos de uma vez** — uma aresta que sobe é cruzada quando o
ponto está à esquerda dela, uma que desce quando está à direita. Os dois `if` do algoritmo original
viram **um** `compare`.

⚠️ E, de quebra, **elimina a divisão**: a forma ingénua acha o `x` do cruzamento com
`t = (v − a_y)/(b_y − a_y)`, que numa árvore seria avaliada em **todas** as arestas — inclusive nas
horizontais, onde é `0/0`. Numa linguagem com `if` isso nunca acontece; numa árvore, é a única coisa
que acontece.

`acima_i` é calculado **uma vez por vértice** e usado pelas duas arestas que o tocam.

---

## §3 — Quanto custa: as duas tabelas que mandam na tolerância

### §3.1 — Tamanho da árvore (`ph2d-field-eval::measure_profile_tree_size`)

| arestas | nós da árvore | nós por aresta |
|---:|---:|---:|
| 16 | 459 | 28,7 |
| 32 | 877 | 27,4 |
| 64 | 1 713 | 26,8 |
| 128 | 3 385 | 26,4 |
| 256 | 6 729 | 26,3 |

**~26 nós por aresta**, e a constante estabiliza — não há termo quadrático escondido.

### §3.2 — Custo de traçado (`ph2d-field-render::measure_profile_trace_cost`, 640×480)

⚠️ **Re-medido depois do anti-serrilhado** ([doc 05](05_resultados_imagem.md)) — a coluna que manda
hoje é a última, porque é a que o app usa.

| arestas | serial s/ AA | paralelo s/ AA | paralelo **c/ AA** |
|---:|---:|---:|---:|
| 8 | 126,7 ms | 9,6 ms | 10,7 ms |
| 16 | 100,8 ms | 11,4 ms | 12,6 ms |
| 32 | 175,2 ms | 16,3 ms | 19,4 ms |
| **64** | 384,7 ms | 26,0 ms | **29,9 ms** |
| 128 | 784,9 ms | 47,3 ms | 55,4 ms |
| 256 | 1 579,6 ms | 86,8 ms | 100,1 ms |
| 512 | 3 055,8 ms | 168,7 ms | 195,5 ms |

**Baseline** (a junção de três cilindros, a mesma janela, com AA): **7,3 ms**.

⭐ **Logo o orçamento de um perfil é ~64 arestas** — 30 ms num quadro de 640×480, que é a ordem de
grandeza de uma peça de primitivas ao mesmo tamanho.

### §3.3 — E é daí que sai o `TOLERANCE_RATIO = 1e-3`

Para um contorno redondo, a tolerância `ε` produz `n ≈ 2,22·√(R/ε)` arestas. Com `ε = 10⁻³·D` (D = a
maior dimensão do desenho, R ≈ D/2) isso dá **≈ 50 arestas** — dentro do orçamento medido.

⚠️ **É uma FRAÇÃO e não um absoluto**: a mesma peça desenhada em milímetros ou em metros tem de sair
com a mesma qualidade. Uma tolerância absoluta faria a **unidade do documento** decidir a suavidade
da forma *e* o custo do traçado, 1000× em cada direção. Há gate
(`the_automatic_tolerance_follows_the_size_of_the_drawing`).

---

## §4 — Os dois gates de ORÁCULO INDEPENDENTE

O risco desta wave era escrever uma fórmula que concorda consigo própria. A defesa foi comparar
contra código **completamente diferente**:

| Gate | O oráculo | O que ele mata |
|---|---|---|
| `an_extruded_polygon_is_the_cylinder_it_approximates` | `Primitive::Cylinder` (fórmula analítica de `ops.rs`) | Sinal trocado em qualquer região; distância errada em qualquer lugar |
| `a_revolved_polygon_is_the_torus_it_traces` | `Primitive::Torus` | O mesmo, e mais: prova que `x → √(x²+z²)` dá a distância **exata** |

Os dois afirmam o erro **exato** que a geometria prevê: um `n`-gono inscrito erra o círculo pela
flecha `R·(1 − cos(π/n))`, e o gate exige que o meio da aresta esteja a **exatamente** isso — não
"perto".

---

## §5 — Os três vermelhos que ensinaram alguma coisa

### 5.1 — Um erro da ORDEM DA PEÇA é assinatura de eixo, não de fórmula

O gate do toro reprovou com **0,834** numa peça de raio 0,8. Um erro dessa magnitude nunca é uma
fórmula com um sinal trocado — é um **sistema de coordenadas diferente**.

E era: o `Torus` da casa tem o anel no plano XY (eixo de revolução = **Z**), como o `Cylinder`; o
`Revolve` novo gira em torno de **Y**.

⭐ **A divergência FICOU, e por decisão.** A regra que manda não é a coerência entre primitivas, é a
coerência com o **plano de desenho**: o perfil vem do editor vetorial, que desenha em XY, e o eixo de
uma revolução tem de estar **dentro** do plano do perfil. A extrusão sai do plano (por Z), a
revolução gira em torno de uma reta do plano (o Y). Quem quiser outro eixo roda o nó — é para isso
que o `Xform` existe. O gate documenta o quarto de volta que reconcilia os dois.

### 5.2 — Um oráculo que aproxima o que mede deixa de ser oráculo quando a tolerância desce até ele

O gate do achatamento media a polilinha contra o **círculo verdadeiro** e reprovava a `ε = 10⁻⁴` com
1,86·10⁻⁴. Não era o achatador: um círculo feito de quatro cúbicas com `κ = 0,5523` **já é
~2,7·10⁻⁴ diferente do círculo** por construção. Acima de 10⁻³ isso some no ruído; a 10⁻⁴ ele *é* a
medição.

A cura foi medir contra a **curva que se deu ao achatador**, que é o que ele promete — 400 amostras
por arco contra a polilinha.

### 5.3 — O ponto de fecho repetido

Quem constrói uma polilinha fechada à mão repete o primeiro ponto no fim; é o hábito de todo formato
de desenho. Aqui isso é uma aresta de comprimento **zero**, e a distância ponto-segmento divide pelo
comprimento ao quadrado: `0/0`, e o campo inteiro vira `NaN` a partir dali. O construtor **limpa** em
vez de recusar — a entrada é legítima, só a representação é que não.

---

## §6 — As decisões, escritas para não ficarem implícitas

| Decisão | Por quê |
|---|---|
| O perfil é **polilinha**, não Bézier | A distância exata a uma cúbica exige resolver uma quíntica, que não é exprimível na árvore. Nem o `libfive` o faz. A cura é tolerância **declarada**, não uma fórmula melhor |
| A **tolerância viaja dentro** do `Profile` | Sem ela, *"este perfil está bom?"* não tem resposta, e re-cozinhar com outro número muda a forma em silêncio |
| `Primitive` **perdeu o `Copy`** | Um perfil é um `Vec`. A alternativa (segunda arena + índice) mantinha o `Copy` e comprava uma classe inteira de erro novo — índice pendente — que a arena de nós existe para tornar impossível |
| Na extrusão, o `round` é limitado **só pela meia-altura** | Um `round` maior que a meia-largura do perfil é uma **abertura morfológica**: o pescoço fino desaparece, que é o que arredondar com esse raio significa. Na altura é diferente: o termo axial inverte e o sólido deixa de existir |
| Um `Revolve` cujo perfil **cruza o eixo** é recusado | A superfície auto-intersecta e o campo deixa de ser distância. Tocar o eixo (`x = 0`) é legítimo — é como um sólido de revolução se fecha |
| O arredondamento das arestas **verticais** vem do editor vetorial | O cozimento parte de `VecPath::cooked()`, com Live Corners já aplicados. *Uma quina, um dono* — o módulo 3D não escreve uma segunda resposta para "arredondar a quina de um contorno" |
| A conversão **não espelha o Y** | Se um plano de desenho tiver o Y para baixo, quem espelha é a **ferramenta** que escolhe o plano — não a costura, que não sabe de que plano o desenho veio. Há gate a pinar |
| Um contorno **aberto** é recusado, não ignorado | Saltá-lo em silêncio daria um sólido *quase* igual ao desenho, e a diferença apareceria como uma parede que não fechou |

---

## §7 — ⚠️ O gatilho MEDIDO que esta wave abriu (e que NÃO foi construído)

O custo é **linear no número de arestas**, e a razão é estrutural: a soma do winding number toca
**todas** as arestas em **toda** amostra. Um `min` de distâncias a `fidget` poda por intervalo; uma
**soma** não.

| Fica bem | Aperta | Fica caro |
|---|---|---|
| ≤ 64 arestas (24 ms) | 128 (44 ms) | ≥ 256 (81 ms), 512 (162 ms) |

Um contorno de letra ou um SVG importado passa de 256 arestas com facilidade. **Quando isso chegar**,
as duas direções candidatas, por ordem de valor:

1. **Aceleração espacial dentro da árvore** — partir o perfil numa hierarquia de `min`/`max` por
   caixa, para que a poda por intervalo volte a morder. É algorítmico e não muda hardware.
2. **GPU** — a W0 disse "só se a medição pedir". Esta é a primeira medição que aponta para lá.

⛔ **Nenhuma das duas foi feita**, e é de propósito: o número que as pediria (um perfil real acima de
128 arestas, num fluxo real) ainda não existe. O que existe é a tabela, e ela é o gatilho.

⚠️ Um segundo número, para quem for medir: o traçado é **ponto a ponto** (`float_slice_tape`), então
o sinal descontínuo **não** o afeta. Quem paga o intervalo frouxo é a **malhagem** — e a malha é o
artefato de exportação, não o que o artista vê.
