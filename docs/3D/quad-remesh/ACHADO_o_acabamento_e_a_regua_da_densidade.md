# ACHADO — a barra do oráculo estava a ser lida a **1/9 da densidade dele**, e o que falta é o ACABAMENTO

> **Data:** 2026-08-28 · **Linha:** `line/quadextract` · **Instrumentos:**
> `cargo run --release -p ph2d-quadextract --example chain_info -- <peca>.obj <alvo>` (com
> `PH2D_RELAX_SCAN=1`) e `… --example piece_report -- <saida>.obj` (com `PH2D_REF=<peca>.obj`).
>
> ⚠️ **Este doc é um ACHADO de RÉGUA antes de ser um de algoritmo.** A primeira metade dele
> desfaz uma comparação errada que estava escrita no `CLAUDE.md` §5 e em toda esta pasta; a
> segunda escolhe, por medição, o acabamento que a cadeia de extracção passa a levar.

---

## §1 — ⛔⛔⛔ O defeito da régua: **a barra `4,8°–7,1°` nunca foi medida na nossa densidade**

Durante toda a semana de 2026-08-22..28 a cadeia foi julgada assim:

> *«enviesamento p50 `7,4°`–`10,4°` contra a barra do oráculo `4,8°`–`7,1°` ⇒ estamos fora.»*

⛔ **Os dois números não são da mesma peça.** A nossa medição corria a `alvo 2` e entregava
**`370`–`576` quads**; a saída do oráculo guardada em `ph2d-quadbench/ref/` tem **`3 352`–`4 696`**.
⇒ a comparação era entre uma malha e outra **nove vezes mais fina**, e mais fina é mais fácil:
cada quad cobre menos curvatura, logo desvia-se menos de `90°`.

⭐ **A mesma cadeia, sem uma linha mudada, à densidade DELE** (`alvo 0,667`):

| peça | quads | `χ` | aspecto p50 | ⭐ envies. p50 | envies. p99 | `>60°` |
|---|---|---|---|---|---|---|
| `sphere_uv_96x144` | 5 005 | 2 | 1,10 | **3,8°** | 17,3° | 0 |
| `sculpt_wrinkled` | 4 557 | 2 | 1,12 | **5,2°** | 35,5° | 0 |
| `sculpt_eared` | 4 492 | 2 | 1,10 | **6,3°** | 27,2° | 0 |
| `sculpt_hooked` | 3 208 | 2 | 1,11 | **6,5°** | 33,0° | 1 |

⇒ **`3,8°`–`6,5°`. Estávamos DENTRO da barra desde 2026-08-25** — e a linha inteira do
`PH2D_GRIDMAP_ARCLINE` (as amarras dos arcos, §23 do
[`ACHADO_ordem_das_fases.md`](ACHADO_ordem_das_fases.md)) foi paga a perseguir um buraco que
era da **régua**.

⚠️ **A lição não é «medimos mal uma vez».** É que *uma barra copiada de outro documento não
carrega as condições em que foi medida*, e nesta pasta ela foi citada **dezenas** de vezes sem
que ninguém contasse as faces dos dois lados. ⇒ **toda comparação com o oráculo passa a nomear
a contagem de faces dos dois lados**, e o `piece_report` imprime-a na primeira linha.

---

## §2 — A tabela HONESTA, com o oráculo medido pela NOSSA régua

O oráculo grava **duas** saídas por peça — a crua e a `_smooth`. ⚠️ **A barra que este repo
citava misturava as duas.** Medidas as duas com o `ph2d_quadfill::quad_shape`:

| peça | quem | faces | aspecto p50 | envies. p50 | envies. p99 | `>60°` |
|---|---|---|---|---|---|---|
| `sphere_uv` | oráculo cru | 3 352 | 1,26 | 7,1° | 29,3° | 0 |
| | ⭐ oráculo `_smooth` | 3 352 | 1,22 | 5,9° | 20,0° | 0 |
| | **nós** (`0,667`, hoje) | 5 005 | **1,10** | **3,8°** | **17,3°** | 0 |
| `sculpt_wrinkled` | oráculo cru | 4 696 | 1,10 | 5,1° | 27,9° | 0 |
| | ⭐ oráculo `_smooth` | 4 696 | **1,08** | **4,8°** | **17,0°** | 0 |
| | **nós** (`0,667`, hoje) | 4 557 | 1,12 | 5,2° | 35,5° | 0 |
| `sculpt_eared` | oráculo cru | 4 658 | 1,11 | 6,4° | 27,7° | 0 |
| | ⭐ oráculo `_smooth` | 4 658 | **1,08** | **5,7°** | **20,2°** | 0 |
| | **nós** (`0,667`, hoje) | 4 492 | 1,10 | 6,3° | 27,2° | 0 |
| `sculpt_hooked` | oráculo cru | 4 262 | 1,21 | 7,3° | 57,1° | **13** |
| | ⭐ oráculo `_smooth` | 4 262 | 1,19 | 5,8° | 48,1° | **4** |
| | **nós** (`0,667`, hoje) | 3 208 | **1,11** | 6,5° | **33,0°** | **1** |

⭐⭐ **Duas coisas saem daqui:**

1. **Nós já batemos a saída CRUA dele em três peças** e ficamos a `0,1°` na quarta.
2. ⭐⭐⭐ **A diferença que sobra é EXACTAMENTE o passe de acabamento dele** — `_smooth` compra-lhe
   `−0,3°` a `−1,5°` na mediana e **`−8°` a `−11°` no p99**. E o nosso acabamento é
   **`6` rondas de Laplaciano tangencial** (`ph2d_quadfill::SMOOTHING_ROUNDS`), uma constante
   herdada da montagem por patches e **nunca re-medida para a extracção**.

---

## §3 — A escada do acabamento, sobre a MESMA extracção

`PH2D_RELAX_SCAN=1` corre todas as variantes a partir da **mesma** malha extraída — *uma
varredura que re-corresse a cadeia mediria também o ruído da cadeia.*

### `sculpt_eared` (4 492 quads) — o Laplaciano não estava saturado em `6`

| acabamento | aspecto p50 | envies. p50 | envies. p99 | area spread | fid máx | relevo |
|---|---|---|---|---|---|---|
| nenhum | 1,10 | 6,5° | 38,3° | 1,45 | 0,825 % | 20,4° |
| ⭐ **lap 6 (hoje)** | 1,10 | 6,3° | 27,2° | 1,45 | 0,764 % | 20,3° |
| lap 40 | 1,08 | 5,6° | 15,9° | 1,46 | 0,727 % | 20,0° |
| lap 160 | 1,05 | 4,5° | 12,1° | 1,54 | 0,706 % | 20,3° |
| lap 320 | 1,04 | 3,7° | 11,2° | 1,60 | 0,685 % | 19,9° |

⚠️ **E o ajuste de quadrado dá a MESMA linha a metade do ritmo** (`quad 2N ≡ lap N` em toda
coluna, nas três peças lisas). *Numa peça lisa as duas leis têm o mesmo ponto fixo.*

### ⭐⭐⭐ `sculpt_hooked` (3 208 quads) — a peça com PONTA separa as duas leis

| acabamento | aspecto p99 | envies. p50 | envies. p99 | `>60°` | ⛔ **dobras** | ⛔ **fid MÁX** |
|---|---|---|---|---|---|---|
| nenhum | 2,06 | 6,9° | 42,2° | 7 | **0** | 2,247 % |
| lap 6 (hoje) | 1,69 | 6,5° | 33,0° | 1 | **0** | ⛔ **3,248 %** |
| lap 80 | 1,49 | 4,0° | 27,7° | ⛔ 4 | ⛔ **2** | ⛔ **4,065 %** |
| lap 160 | 1,43 | 3,0° | 28,9° | ⛔ 5 | ⛔ **4** | ⛔ **3,952 %** |
| lap 320 | 1,42 | 2,2° | 29,0° | 3 | ⛔ **3** | ⛔ **4,189 %** |
| ⭐ **quad 160** | 1,44 | 4,0° | 27,3° | **0** | **0** | ⭐ **1,367 %** |
| ⭐ **quad 320** | 1,39 | 3,0° | 25,1° | **0** | **0** | ⭐ **1,399 %** |

⭐⭐⭐ **À MESMA mediana de enviesamento (`3,0°`), o Laplaciano entrega `4` dobras, `5` faces
péssimas e `3,95 %` de perda de ponta; o ajuste de quadrado entrega `0`, `0` e `1,40 %` — e
ainda um `p99` melhor.** As duas leis não são a mesma; a peça lisa é que não sabia distingui-las.

### ⭐ O MECANISMO, e é ele que fecha a escolha

O Laplaciano manda cada vértice para o **centróide dos vizinhos**. Numa ponta, *todos* os
vizinhos estão para trás — logo a ponta é puxada para dentro, a reprojecção aterra do lado
errado do vinco, e nasce dobra. ⭐ **O ajuste de quadrado nunca pede um centróide:** ele pede
que a FACE seja um quadrado, e uma face na ponta pode ser um quadrado **sem sair da ponta**.

⇒ *O alisador cego ao ângulo estraga onde a forma tem informação; o que olha para o ângulo não
precisa de mover o vértice para longe.*

⚠️ **E isto INVERTE a recusa de 2026-08-22** (`ph2d_quadfill::SQUARE_ROUNDS`), que mediu esta
mesma lei sobre a montagem por **patches** e concluiu, correctamente para aquela conectividade,
que ela *«compra cauda e paga dobras»*. Aqui ela compra cauda e **não paga dobra nenhuma** —
quem as paga é o Laplaciano. *Uma recusa medida responde UMA pergunta; a conectividade mudou,
e a pergunta é outra.*

---

## §4 — O preço, e o que ele NÃO é

- **`area spread` sobe** (`1,45` → `1,60` na orelha a 320 rondas): quadrados de tamanho
  variável em vez de rectângulos de tamanho uniforme. ⚠️ *É a troca que se quer* — a régua do
  artista e a do oráculo são forma, não uniformidade de área.
- ⛔ **`relevo` (obediência às direcções principais) DEGRADA-SE, e as duas leis pagam o mesmo.**
  Fica inerte na orelha (`20,3°` → `19,9°`) e na esfera (onde `22,5°` = cega, porque uma esfera
  não tem direcção preferida — a régua é pesada pela anisotropia e ali cala-se por construção),
  e **move-se na `wrinkled`**, que é a peça com relevo a sério: `11,8°` → `12,5°` (160) →
  `13,6°` (320) → `15,4°` (1280). ⚠️ **A `lap 160` e a `quad 320` dão `13,6°` as duas** — ao
  mesmo grau de avanço o preço é idêntico, e **não** há aqui uma vantagem do ajuste de
  quadrado. *A vantagem dele é a ponta (§3), não o relevo.*
- **Fidelidade `p95`** não se mexe em peça nenhuma; o que se mexe é o **máximo**, e é lá que o
  Laplaciano perde a ponta.

---

## §5 — ⛔ O que NÃO explica o resto

- ⛔ **Não é o campo (F2).** Ele já foi ilibado por resultado em 2026-08-24, e a tabela do §2
  volta a dizê-lo: a nossa saída CRUA bate a saída CRUA dele.
- ⛔ **Não é o endurecimento local** (MIQ §5.4): construído, medido e rejeitado em 2026-08-25
  com mecanismo (`ph2d_gridmap::STIFFEN_PASSES`) — a nossa energia é *seguir o campo*, e pesar
  mais um triângulo virado manda-o obedecer com mais força exactamente onde obedecer é o que o
  vira.
- ⛔ **Não são as amarras dos arcos** (`PH2D_GRIDMAP_ARCLINE`): medidas em 2026-08-28, pioram a
  forma nas quatro peças (§23.35 do [`ACHADO_ordem_das_fases.md`](ACHADO_ordem_das_fases.md)).

---

## §6 — ⛔⛔⛔ REFUTADO: o oráculo **NÃO** refina por curvatura. A malha dele é UNIFORME.

O doc do [`ph2d-remesh-iso`](../../../crates/ph2d-remesh-iso/src/lib.rs) declara, desde
2026-08-20, um **item aberto** do F1:

> *«Nas fixturas CURVAS o oráculo termina mais FINO que `alpha × diag` (0,0566 contra 0,0693 na
> esfera; 0,0588 contra 0,0859 na `sculpt_hooked`) — ou seja, **ele refina abaixo do alvo onde a
> curvatura pede**. Essa metade da lei ainda não está portada, e é o primeiro item aberto do F1.»*

⭐ E era o candidato natural para «estado da arte», porque é exactamente a manchete do
**ZRemesher/QuadRemesher** (o *Adaptive Size*: quads menores onde a curvatura é alta).

⛔ **Medido 2026-08-28, sobre a malha remalhada DELE** (`ref/<peça>/<peça>_rem.obj`, saída, e
portanto lícita), com a curvatura média estimada por vértice (`κ ≈ 2·|L(p)|/h²`, a lei do
Laplaciano uniforme que dá `1/R` numa esfera) e os vértices repartidos em **oito bandas de
curvatura**:

| peça | `κ` da banda 0 | `κ` da banda 7 | aresta na banda 0 | aresta na banda 7 | ⇒ expoente `h ~ κ^e` |
|---|---|---|---|---|---|
| `sculpt_eared` | 0,97 | **7,70** | 0,0564 | 0,0490 | **`−0,029`** |
| `sculpt_hooked` | 0,73 | **6,64** | 0,0579 | 0,0552 | **`−0,007`** |
| `sculpt_wrinkled` | 0,97 | **6,20** | 0,0566 | 0,0558 | **`+0,009`** |

⇒ **Sobre uma faixa de `8×` na curvatura, a aresta dele não se mexe** (`e ≈ 0`; `−0,5` seria
erro geométrico constante e `−1` seria ângulo constante). A malha do oráculo é **isotrópica e
uniforme**, tal como a nossa.

⭐⭐⭐ **A inferência de 2026-08-20 confundiu duas afirmações.** *«O alvo GLOBAL dele numa peça
curva é menor que `alpha × diag`»* é verdade — e não implica *«ele refina LOCALMENTE onde a
curvatura pede»*, que é falso. A primeira é sobre como ele escolhe **um** número por peça; a
segunda é sobre a **variação dentro** da peça, e só a segunda seria trabalho de porte.

⇒ **O item aberto do F1 fecha como REFUTADO.** ⚠️ Isto **não** diz que densidade adaptativa
seja má ideia — diz que ela é uma **feature de produto** (o *Adaptive Size* do ZBrush, com o
artista a decidir), e **não** o que separa a nossa saída da dele. *Construí-la à espera de
fechar esta distância teria sido pagar por uma diferença que não existe.*
