# HANDOFF DE INTEGRAÇÃO — `line/FLIP`: o teto de vizinhos era uma CONTAGEM para cobrir um ALCANCE

**Data:** 2026-07-28 · **Branch:** `line/FLIP` · **Commits desta rodada:** `c0d88eaed` ·
`f09bbbc28` · `e583bbf03` · `8c377ce73` · **`7ca83d6fb`** (o conserto).

**Estado:** fechada, **pendente de smoke**. Não integrar sem ordem explícita do Enio.

---

## 1. O report, e por que três rodadas erraram o alvo

> *"Continua a mesma bosta. Mas o problema só aparece se o cruzamento é feito com traço único
> (sem mouse up). Se cruzo vários traços diferentes (após mouse up) esse aspecto 3d não aparece
> e o traço fica melhor."* — Enio, 4ª rodada

As três rodadas anteriores atacaram a **LEI** (o perfil de dureza · a composição de passagens ·
o destino do transbordo) e **a lei já estava certa**: medido contra o depósito REAL do Painter, o
Flip pinta a estrela com desvio de **−3 de 255**. O que estava errado era **QUANTO CAMINHO
chegava ao fragment**.

⚠️ **A frase do Enio era o diagnóstico, não o sintoma** — e é ela que aponta a peça: dois traços
distintos têm depth diferente e compõem por `over`, então o parceiro do cruzamento **não precisa
estar na lista de vizinhos**. A lista só existe para o traço que volta sobre si mesmo. Se o
defeito some com traços separados, o defeito É a lista.

## 2. O mecanismo, e ele é aritmético

O alcance de influência de um segmento é `≈ 3 × raio`. A lista de vizinhos era capeada por
**CONTAGEM** (`MAX_RIBBON_EXTRAS = 16` **segmentos**). A contagem necessária é `alcance ÷ passo`
⇒ a cerca fica em `3r/16 = 0,1875·r`. Passando dela a lista **trunca**, o pixel volta ao
first-wins do Grease Pencil, e a cauda macia de um quad é pintada sobre o **NÚCLEO** do vizinho.

**Medido contra o depósito do Painter, mesma figura, variando SÓ a amostragem:**

| passo / raio | antes | agora |
|---|---|---|
| 0,80 | −3 | −3 |
| 0,40 | −3 | −3 |
| 0,20 | −3 | −3 |
| **0,10** | **−184** | **−3** |
| **0,05** | **−255** (a tinta SOME) | **−3** |
| 0,04 | — | **−3** |

⚠️ **E o produto atravessa a cerca DESENHANDO DEVAGAR** — não é caso patológico. O RDP tem
tolerância `0,05 × espessura = 0,1·r` e a reamostragem **só ACRESCENTA** pontos. Medido
(`flip_draw_tests::the_real_pipeline_step_in_radii`):

| gesto | passo mínimo | segmentos abaixo da cerca |
|---|---|---|
| arco de mão, 400 amostras | **0,137·r** | 10 |
| arco de mão, 1200 amostras (mão LENTA) | **0,108·r** | **125 de 251** |

## 3. A cura: o teto conta CÁPSULAS, e uma cápsula é um PEDAÇO DE CAMINHO

Segmentos consecutivos quase-colineares descrevem o mesmo pedaço de caminho; uma única cápsula
ligando as pontas cobre a mesma tinta, com erro igual à **FLECHA** da corda. Fundir enquanto a
flecha ficar abaixo de `MERGE_SAGITTA × raio` (**1/32**) torna o número necessário função da
**CURVATURA e do alcance**, nunca da amostragem:

| cenário (`measure_ribbon_budget`) | antes | agora |
|---|---|---|
| reta | 4 | **2** |
| arco raio 10·r | 12 | **4** |
| arco raio 1·r (o limite do pincel) | 11 | **6** |
| **entrada 4× densa (mão LENTA)** | **16, SATURADO** | **8** |

⚠️ **O shader NÃO mudou um byte** — nem a BGL, nem o formato do buffer, nem o varying. Ele já
montava a cápsula de `points[a]` a `points[b]` e nunca precisou saber se `b = a+1`. **A wave
inteira mora na CPU** (`neighbors.rs` + duas linhas de `pack.rs`).

## 4. As duas coisas que a implementação aprendeu

⚠️ **A CONTIGUIDADE é conferida, nunca assumida.** Índice consecutivo não é caminho contíguo, e
fundir dois segmentos que não compartilham ponto desenharia uma cápsula sobre tela nua. A
polilinha do produto é contígua **por construção** — é exatamente por isso que a premissa
passaria despercebida. Quem a prova é a fixture do **pente** (segmentos desconexos), que existia
por outro motivo.

⚠️ **O mecanismo de DOIS CARIMBOS da §8.7.1 MORREU, por medição.** Com as cápsulas fundidas a
caminhada absorve a passagem inteira, então o grid nunca reencontra um segmento próprio: as duas
mutações que o par de carimbos equilibrava (`walked` · `stamp`) passaram a **não sangrar em
fixture nenhuma**. Código morto MENTE, então saiu — e o `+63` que ele evitava virou
estruturalmente impossível (o que a caminhada visitou é da própria passagem, e o grid não o vê).

## 5. Gates

**Novo, red-first:** `crates/ph2d-flip-render/tests/sampling_invariance.rs::`
**`the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled`** — a MESMA figura, de
`0,80·r` a `0,04·r`, contra o **depósito REAL do Painter** (não uma cópia da regra do produto),
com a **mesma barra em toda densidade**. Nasceu vermelho em **−184**.

É a **quinta** vez que esta lei é pinada no projeto (as quatro anteriores no relevo do Painter) e
a primeira no rasterizador do Flip.

**Mutações — 5, todas sangram:**

| # | mutação | sangra em |
|---|---|---|
| M1 | a fusão morre (1 cápsula por segmento) | 2 gates GPU |
| M2 | a contiguidade deixa de ser conferida | 2 gates de unidade (o pente) |
| M3 | a caminhada não carimba (o grid recolhe como cruzamento) | 2 unidade + 3 GPU |
| M4 | a fronteira `ribbon` zera (a fita compõe consigo mesma) | 3 GPU |
| M5 | a tolerância de fusão explode (funde por cima da curva) | 4 GPU |

**Suítes:** workspace impactada **1701/1701** · gates GPU do Flip **65/65** (`--ignored`, na RTX)
· clippy `--all-targets` limpo · fmt limpo · LOC caps verdes.

## 6. Custo, medido e nomeado

| | antes | agora |
|---|---|---|
| pack, traço real de 4000 pontos | 1,62 ms · **47.546** vizinhos | 1,6–1,9 ms · **11.954** |
| pack, rabisco PATOLÓGICO de 4000 pontos | 15,9 ms | **20,1 ms** (teto 120) |

⚠️ O caso normal **não pagou nada** e a lista encolheu **4×** — o que barateia o **laço do
FRAGMENT**, que roda por pixel a cada frame. O rabisco patológico subiu ~30 %: a fusão colhe
todos os candidatos antes de agrupá-los em runs, em vez de rejeitar em O(1) pelos 16 mais
próximos. É o preço de o teto ser de CÁPSULAS.

## 7. O smoke

**`env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release`**

⚠️ **A cena mudou: são QUATRO grupos agora**, e o 4º é o que faltava.

1. X duro (hardness 1.0) — **o CONTROLE**, byte-idêntico.
2. X macio (0.4), DOIS traços cruzados.
3. Estrela de UM traço, **mão RÁPIDA** (passo mínimo **1,495·r**, medido).
4. Estrela de UM traço, **mão LENTA** (passo mínimo **0,106·r**) — **a sua foto**.

**O que olhar: as duas estrelas têm de ser a MESMA figura.** Era ali que o defeito vivia — a da
direita perdia tinta em cada quina e cada cruzamento, e o buraco lia como dobra 3D.

⚠️ **A cena PROVA que contém o fenômeno**, por gate próprio
(`flip_hardness_smoke::tests::the_slow_hand_star_is_denser_than_the_old_neighbour_fence`): a
lenta abaixo da cerca, a rápida acima. As quatro rodadas anteriores encenavam **só** o caso do
lado seguro — por isso o smoke passava e o produto não.

E o gesto do report, feito por você: **desenhe devagar, cruzando o próprio traço, sem levantar a
caneta.**

## 8. Superfície / schema

**Nada.** Sem `PROJECT_SCHEMA`, sem `FLIP_SCHEMA`, sem contrato congelado, sem ADR, sem id, sem
token, sem `Cargo.toml`, sem dep nova. O diff inteiro é `ph2d-flip-render` (3 arquivos de `src`,
3 de `tests`) + 2 arquivos do shell + 1 doc.

⚠️ **API interna alterada** (tudo `pub(crate)`): `SegExtras.list` passou de `Vec<u32>` (índices de
SEGMENTO) para `Vec<Capsule>` = `Vec<(u32, u32)>` (índices de PONTO). O único consumidor é
`pack.rs`.

## 9. Aberto, nomeado

- **O resíduo do TIP convexo** (`+140` de 255 na ponta de 36°) **fica, e é do ORÁCULO**: o
  depósito do Painter carimba dabs a cada `0,2·r` de arco, então a ponta dele é até `0,2·r` mais
  curta que a cobertura contínua do Flip. É a direção OPOSTA à queixa (a ponta fica mais redonda,
  não mordida) e reproduzir a quantização do Painter seria copiar um artefato dele.
- **Joins & caps** (miter/bevel/round + butt/round/square) segue como wave dedicada.
- **Congelar o contrato do `ph2d-flip`** segue aberto.
