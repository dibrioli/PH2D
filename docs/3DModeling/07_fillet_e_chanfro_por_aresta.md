# Fillet e chanfro **por aresta** e **por vértice** — avaliação medida

> **Pergunta do Enio, 2026-08-28:** *"avalie a possibilidade de chamfer e fillet por edge e por
> vertex, em vez do objeto todo. relate"*
>
> Sonda executável: [`spike_per_edge_radius.rs`](../../crates/ph2d-field-eval/tests/spike_per_edge_radius.rs)
> — 3 gates + 1 medição, `0,43 s`. Tudo o que este doc afirma sai dela ou de um doc já medido.

---

## §1 — O veredicto, em três linhas

| Pergunta | Resposta | Onde |
|---|---|---|
| Um raio por **grupo de arestas** de uma primitiva (as 4 verticais de uma caixa) | ⭐ **SIM**, e é barato e exacto | §3 |
| Um raio por **aresta individual** de uma primitiva (as 12 de uma caixa) | ⚠️ **Provavelmente**, com preço por medir | §4 |
| Um raio por **aresta do RESULTADO** de uma booleana (o fluxo do CAD) | ⛔ **NÃO** sem mudar de representação — e traria uma doença junto | §5 |
| Um raio por **vértice** | ⚠️ **Não é um item à parte: é a consequência do de aresta** | §6 |

---

## §2 — ⚠️ Nós e o modelador de referência estamos em pontas OPOSTAS

O estudo do modelador que este plano ia portar já tinha medido isto
([`00_plano_port.md` §1.3](00_plano_port.md)):

> | Fillet em **todas** as arestas de uma taça | **falhou** | Fillet é sempre **por aresta
> selecionada**; sem botão "arredondar tudo" |

⇒ **Ele só faz por aresta e não tem "arredondar tudo". Nós só fazemos "tudo" e não temos por
aresta.** A pergunta do Enio é, literalmente, a distância entre as duas.

⚠️ E a razão não é preguiça: é a **família** que o [ADR-0161](../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)
escolheu, com o preço escrito na altura.

| | B-Rep (Parasolid, `monstertruck`, `brepkit`) | Implícito (nós, nTop, Womp) |
|---|---|---|
| O que é uma **aresta** | uma **entidade** com identidade — a curva de intersecção de duas faces | **não existe**; é uma descontinuidade do gradiente, emergente |
| Fillet por aresta | ✅ nativo (`fillet_edges`, `RadiusSpec` por aresta) | ⛔ não há em que pegar |
| Booleana que nunca falha | ❌ falha em geometria difícil | ✅ é `min`/`max` de dois números |
| «Arredondar tudo» | ❌ **não existe** (medido no referencial) | ✅ é o que temos |

---

## §3 — ⭐ O que É barato: um raio por GRUPO de arestas, MEDIDO

Uma caixa tem 12 arestas em **3 grupos de 4** (as paralelas a X, a Y, a Z). Um raio por grupo
constrói-se como a **intersecção de três barras de secção arredondada** — a barra em X limita
`|y|≤hy, |z|≤hz` com os cantos arredondados em `rx`, e arredonda exactamente as 4 arestas paralelas
a X.

**Medido na sonda:**

| | valor | veredicto |
|---|---|---|
| `‖∇f‖` pior | **`1,0000`** | ⭐ continua a ser uma **distância** — a marcha não abranda |
| Custo, em nós de árvore | `30 → 58` (**`1,93×`** a caixa arredondada) | e a caixa **viva** custa 28 |
| Só o grupo que pediu é arredondado | Δ `0,083` na aresta que pediu · **`< 1e-6`** na que não | ⭐ zero vazamento |

⭐⭐ **E o caso uniforme pode ficar BYTE-IDÊNTICO:** com os três raios iguais, o construtor continua a
ser o `sd_box` de sempre (30 nós); a construção de três barras só entra quando eles **diferem**. ⇒
nenhuma peça existente muda, e ninguém paga os `1,93×` por uma caixa que não pediu nada.

### §3.1 — ⚠️ Mas ela NÃO é a caixa arredondada de hoje, e o sítio onde difere é o achado

| direcção | `sd_box` | por grupo | Δ |
|---|---|---|---|
| **face** `(1,0,0)` | `0,50000` | `0,50000` | `+0,00000` |
| **aresta** `(1,1,0)` | `0,64497` | `0,64497` | `+0,00000` |
| **vértice** `(1,1,1)` | `0,75622` | `0,78993` | **`+0,03371`** |

O `sd_box` arredonda por **deslocamento** — a soma de Minkowski com uma **bola** —, então os 8 cantos
saem **esféricos**. Três barras arredondadas dão **cilindros** nas arestas e um canto de
**Steinmetz**. As duas coincidem na face e na aresta, e **divergem no vértice**.

---

## §4 — Por aresta INDIVIDUAL numa primitiva: provavelmente sim, preço por medir

O passo seguinte é natural: cada barra é uma **secção rectangular 2D**, e um rectângulo 2D com **4
raios de canto diferentes** é forma canónica (é o `border-radius` do CSS, e há SDF publicado). Três
barras × 4 cantos = **as 12 arestas, cada uma com o seu raio**.

⚠️ **O que ainda não está medido**, e tem de estar antes de alguém prometer isto:
- o custo (cada barra passa a ter uma selecção por quadrante);
- se o campo continua **contínuo** na fronteira entre dois quadrantes de raios diferentes — deve
  continuar, porque os arredondamentos não se tocam enquanto `r < h`, mas *«deve»* não é um número.

⛔ E vale para **primitivas**, onde as arestas têm nome **estrutural** (*«a aresta +X+Y desta
caixa»*). Não vale para o resultado de uma booleana — §5.

---

## §5 — ⛔ Por aresta do RESULTADO: não, e o motivo é maior que o custo

É este o fluxo do CAD: *«clico nesta aresta do sólido e arredondo-a com 2 mm»*. Ele precisa de duas
coisas que a representação implícita não tem:

**1. A aresta não existe como coisa.** As arestas de `A − B` são as curvas onde a superfície de `A`
encontra a de `B`. O campo não as enumera — ele responde `f(p)` e mais nada. Dar raios diferentes a
duas delas exige um raio que **varia com a posição**, `r(p)`; e aí o campo deixa de ser uma distância
onde `r` varia, o que custa a promessa central do módulo (*«o raio pedido é o raio entregue, 0,00 %
de erro»*) e obriga a marcha a abrandar.

**2. ⭐⭐⭐ E traria junto o PROBLEMA DO NOME PERSISTENTE.** Para o raio sobreviver a uma edição, o
documento tem de **nomear** a aresta de forma durável. Em B-Rep isto tem nome próprio — *persistent
naming* — e é **o problema não resolvido** de todo kernel de CAD: é a razão de um modelo de histórico
"explodir" quando se muda uma cota lá atrás, e cada kernel tem a sua heurística para adiar o
sintoma.

> ⚠️ **A nossa representação não tem essa doença precisamente por não ter identidade de aresta.**
> Pôr selecção por aresta importaria a doença junto com a feature.

⛔ Há uma terceira saída que **não** recomendo esconder: um raio por junção **já existe** desde a W98
— o artista consegue raios diferentes decompondo a peça nas partes cujas junções são as arestas que
lhe interessam. É o fluxo nativo do SDF (é o que se faz no MagicaCSG e no Womp), e é honesto dizer
que ele pede outra maneira de modelar, não a do CAD.

---

## §6 — ⭐⭐⭐ «Por vértice» não é um item à parte: é a CONSEQUÊNCIA de «por aresta»

Isto saiu da medição, não de uma opinião. Na tabela da §3.1, face e aresta coincidem e **só o vértice
diverge** — porque num canto de caixa encontram-se **três** arestas, e assim que elas têm raios
independentes o canto deixa de ter resposta óbvia.

É exactamente por isso que os kernels de CAD tratam o *vertex blend* como **operação separada**, com
opções próprias (rolling-ball, setback, patch de N lados).

⇒ **A ordem é forçada:** por vértice não se pode desenhar antes de por aresta existir, porque ele é a
pergunta que o por-aresta cria.

---

## §7 — Recomendação

1. ⭐ **FAZER: um raio por grupo de arestas nas primitivas.** Três números numa caixa, dois num
   cilindro (o aro de cima e o de baixo). É exacto, mantém `‖∇f‖ = 1`, custa `1,93×` **só** quando os
   raios diferem, e o caso uniforme fica byte-idêntico. ⭐ E o painel **não muda**: as linhas de
   número saem do `params_of`, como o `Joint` da W98.
2. ⏳ **MEDIR ANTES DE PROMETER: por aresta individual.** O caminho existe (rectângulo 2D com 4
   raios); faltam o custo e a continuidade. Uma sonda de meio dia responde.
3. ⛔ **RECUSAR: por aresta do resultado de uma booleana.** Não é preço, é representação — e traz o
   problema do nome persistente, que é a doença que não temos.
4. ⏸️ **ADIAR: por vértice.** Ele é a consequência do (2), e desenhá-lo antes seria responder a uma
   pergunta que ainda não existe.

---

## ⛔ Recusas MEDIDAS

| Recusa | Motivo medido |
|---|---|
| Fillet por aresta selecionada no **resultado** de uma booleana | a aresta não existe na representação, e nomeá-la de forma durável é o *persistent naming*, não resolvido em CAD nenhum (§5) |
| Substituir o `sd_box` pela construção de três barras | ela diverge no **vértice** (`+0,03371`): canto de Steinmetz contra esférico ⇒ mudaria toda peça já feita. A cura é usá-la **só** quando os raios diferem (§3) |
| Raio que varia com a posição (`r(p)`) para arredondar uma aresta e não a vizinha | mata a exactidão do raio entregue e obriga a marcha a abrandar em toda peça (§5) |
