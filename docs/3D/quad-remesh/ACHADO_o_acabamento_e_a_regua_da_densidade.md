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

---

## §7 — ⭐⭐⭐ O ACABAMENTO QUE OLHA PARA O RELEVO (e por que ele é o certo)

### §7.1 — A régua que faltava: **o acabamento do oráculo não paga relevo**

Com o `piece_report` a receber a escultura (`PH2D_REF=`), mediu-se pela primeira vez a
obediência ao relevo das saídas **dele**:

| peça | oráculo cru | oráculo `_smooth` | nós (fina, `lap 6`) |
|---|---|---|---|
| `sculpt_wrinkled` | 7,1° | ⭐ **7,0°** | 11,8° |
| `sculpt_eared` | 19,7° | 19,4° | 20,3° |
| `sculpt_hooked` | 15,1° | ⭐ **13,3°** | 17,7° |

⭐⭐⭐ **O passe de acabamento dele compra forma SEM pagar relevo — e no gancho até o
MELHORA.** ⇒ *um acabamento que estraga o alinhamento não é o acabamento certo com rondas a
mais; é outro acabamento.* E as duas tentativas cegas confirmam-no:

- **relaxação sem cerca**: `sculpt_wrinkled` grossa, enviesamento `8,6° → 3,2°` e relevo
  `11,9° → 18,8°` (com `22,5°` = cega). *Ela desliza a grade pela superfície até os quads
  serem quadrados, e apaga a propriedade que distingue uma retopologia por campo cruzado de
  um remesh por voxel.*
- ⛔ **cerca de VIAGEM sozinha, medida e rejeitada**: a `0,35 h` ela guarda o relevo
  (`11,6°`) e paga o `p99` do enviesamento — `52,8°` contra os `34,5°` de hoje. *A cerca
  limita a distância; o defeito não é distância, é direcção.* (A cerca fica na API, por ser
  uma propriedade útil e medida, e nasce **desligada**.)

### §7.2 — A lei: rodar o quadrado para onde a superfície pede, com o peso da CONFIANÇA

O quadrado mais próximo de quatro pontos tem forma fechada (o 1.º harmónico `h`, ver
`ph2d_quadfill::nearest_square`). ⭐ **O tamanho vem dos pontos; a ORIENTAÇÃO não precisa
de vir.** As arestas de `h·iᵏ` correm a `arg(h) + 45° + 90°k`, logo a orientação de uma
grade é um ângulo **módulo 90°** — e rodá-la para a direcção principal da superfície é uma
correcção dobrada em `[−45°, 45°]`, aplicada com peso.

⭐⭐⭐ **O peso é a ANISOTROPIA, tal como ela sai da estimativa, sem constante nenhuma por
cima** (`|k₁ − k₂| / (|k₁| + |k₂|)`, em `[0, 1]`). Numa esfera ela é `0` e a lei degenera
**ao bit** no quadrado puro — e a medição prova-o: na `sphere_uv` as linhas `x0`, `x0.5`,
`x1`, `x2` e `x4` são **idênticas em toda coluna**. *Um alinhamento sem confiança poria
costura onde a forma não pede nenhuma.*

### §7.3 — A tabela, densidade grossa (a que o botão usa), `relevo x1` contra o que shipa hoje

| peça | enviesamento p50 | enviesamento p99 | aspecto p50 | `>60°` | ⭐ **relevo** | ms |
|---|---|---|---|---|---|---|
| `sculpt_wrinkled` | `7,8° →` **`4,5°`** | `34,5° →` **`30,8°`** | `1,19 →` **`1,09`** | `0 → 0` | `12,1° →` **`11,2°`** | `17 → 393` |
| `sculpt_hooked` | `7,7° →` **`4,2°`** | `49,1° →` **`44,0°`** | `1,17 →` **`1,09`** | `2 → 2` | `16,5° →` **`14,7°`** | `8 → 186` |
| `sculpt_eared` | `10,4° →` **`3,8°`** | `33,4° →` **`31,2°`** | `1,14 →` **`1,07`** | `0 → 0` | `17,4° → 17,8°` | `21 → 411` |
| `sphere_uv` | `7,4° →` **`3,0°`** | `34,3° →` **`30,4°`** | `1,11 →` **`1,06`** | `0 → 0` | `25,9° → 14,6°` | `18 → 318` |

⚠️ **A única coluna que não melhora é o relevo da orelha** (`+0,4°`), a peça com a menor
confiança de anisotropia do corpus (`0,05`) — e a relaxação cega ali custava `+3,7°`.

⚠️ **Preço medido:** `190`–`411 ms`, contra os `8`–`21 ms` do Laplaciano de hoje, sobre uma
cadeia de `4`–`10 s`. ⭐ E ele já leva **duas** acelerações que valem `~12×`: o raio da
reprojecção **encolhe com o movimento da ronda** (exacto — depois da 1.ª ronda o vértice
está *sobre* a superfície, e uma esfera de `2×` o que ele andou contém o pé mais próximo) e
a corrida **sai por assentamento** em vez de gastar o tecto (`259`–`394` rondas de `2 000`).

⛔ **O Laplaciano SAI do caminho da extracção.** Ele não é somado: medido, `lap` + quadrado
entrega pior ponta que o quadrado sozinho, e o quadrado sozinho já leva o `>60°` de `8` a
`0` nas peças em que o Laplaciano o levava.

---

## §8 — O PREÇO DA LEI DE PARAGEM, e a regressão que ela expôs

A relaxação sai **por assentamento** (`settle`, em fracções da aresta alvo) e não por um
número de rondas — *a taxa de convergência depende do tamanho da malha, então um tecto de
rondas é uma cerca cujo tamanho muda com a peça.* O que a paragem custa, medido:

| peça · alvo | `settle` | rondas | ms | envies. p50 / p99 | `>60°` | dobras |
|---|---|---|---|---|---|---|
| `wrinkled` · 0,667 | — (`lap 6` hoje) | — | 145 | `5,2°` / `35,5°` | 0 | 0 |
| | `3e-2` | 26 | **322** | `5,2°` / `33,6°` | 0 | 0 |
| | `1e-2` | 93 | 1 005 | `4,5°` / `28,5°` | 0 | 0 |
| | `3e-3` | 360 | 3 664 | `3,7°` / `24,6°` | 0 | 0 |
| | `1e-3` | 940 | 9 360 | `2,8°` / `23,4°` | 0 | 0 |
| `eared` · 2 | — (`lap 6` hoje) | — | 21 | `10,4°` / `33,4°` | 0 | 0 |
| | `1e-2` | 33 | **55** | `9,6°` / `34,1°` | 0 | 0 |
| | `3e-3` | 140 | 172 | `6,3°` / `33,0°` | 0 | 0 |
| | `1e-3` | 369 | 413 | `3,8°` / `31,2°` | 0 | 0 |

⭐ **A escada é regular e o preço é linear nas rondas** — nada aqui satura antes de `1e-3`.

### ⛔⛔ E a régua apanhou uma REGRESSÃO que a densidade grossa escondia

`sculpt_hooked` **fina** (3 208 quads), com o ajuste de quadrado **sozinho**:

| acabamento | envies. p50 / p99 | ⛔ `>60°` | ⛔ **dobras** | fid máx |
|---|---|---|---|---|
| nenhum | `6,9°` / `42,2°` | 7 | **0** | 2,247 % |
| ⭐ `lap 6` (hoje) | `6,5°` / `33,0°` | **1** | **0** | 3,248 % |
| quadrado, `settle 1e-2` | `3,3°` / `39,1°` | ⛔ **6** | ⛔ **3** | 2,728 % |
| quadrado, tecto 100 | `4,8°` / `41,1°` | ⛔ **11** | ⛔ **2** | 2,332 % |

⇒ **as duas leis não são substitutas: elas atacam metades diferentes.** O Laplaciano iguala
**comprimentos** e é ele que mata a face extrema (`>60°` de `7` para `1`); o ajuste de
quadrado endireita o **ângulo** e é ele que move a mediana (`6,5°` para `3,3°`). ⚠️ *A
densidade grossa não distinguia os dois* — ali o quadrado sozinho também levava o `>60°` a
zero, e a conclusão «o Laplaciano sai» teria shipado uma regressão numa peça com ponta.

---

## §9 — ⭐⭐⭐ A/B FINAL, **através da porta do produto**, na densidade que o botão usa

`PH2D_OUT_RELAX=6` reproduz o que shipava (`6` rondas de Laplaciano); sem ele o `chain_info`
chama `ph2d_quadfill::finish_extracted`, que é **a mesma função que o botão chama**.

| peça (`alvo 2`) | aspecto p50 | ⭐ envies. p50 | envies. p99 | `>60°` | relevo | rondas |
|---|---|---|---|---|---|---|
| `sculpt_wrinkled` | `1,19 →` **`1,10`** | `7,8° →` **`4,5°`** | `34,5° →` **`32,0°`** | `0 → 0` | `12,1° →` **`11,5°`** | 308 (ficou 302) |
| `sculpt_eared` | `1,14 →` **`1,07`** | `10,4° →` **`3,8°`** | `33,4° →` **`30,9°`** | `0 → 0` | `17,4° → 18,6°` | 350 |
| `sculpt_hooked` | `1,17 →` **`1,09`** | `7,7° →` **`4,3°`** | `49,1° →` **`45,4°`** | `2 → 2` | `16,5° →` **`14,6°`** | 283 (ficou 273) |
| `sphere_uv_96x144` | `1,11 →` **`1,06`** | `7,4° →` **`3,0°`** | `34,3° →` **`30,4°`** | `0 → 0` | (confiança `0,00`) | 248 |

⭐⭐⭐ **As quatro peças melhoram em aspecto, mediana e `p99`, nenhuma ganha uma face
péssima, e o relevo melhora em duas.** A única coluna que anda para trás é o relevo da
orelha (`+1,2°`), a peça com a **menor confiança de anisotropia do corpus** (`0,04`) — ali o
número é quase ruído, e a relaxação **cega** custava `+3,7°`.

⚠️ **Contra a barra do oráculo** (`aspecto p50 1,08–1,22` · `envies. p50 4,8–7,1°`), medida a
`3 352`–`4 696` quads: estas quatro saídas têm `370`–`576` quads e entregam `1,06`–`1,10` e
`3,0°`–`4,5°`. *A comparação continua a não ser da mesma densidade — é por isso que ela está
no §2 e não aqui.*

### ⛔ O que ficou por curar, nomeado

- **`sculpt_hooked` fina:** o alinhamento nunca bate a ronda zero (aquela peça sai do
  Laplaciano com `1` face péssima e a relaxação alinhada sobe-a para `2` à primeira). A porta
  entrega **exactamente o que shipava** e a paciência corta o desperdício. ⛔ Com o
  alinhamento **desligado** a mesma peça chega a `1,04 / 2,0° / p99 22,8 / >60 0`: *há ali um
  ganho que esta lei não alcança*, e a hipótese seguinte é a direcção principal ser **ruidosa
  por face** (ela vem de `ph2d_mesh::principal_dirs` sem suavização de vizinhança).
- **`sculpt_hooked` grossa:** o `fid máx` sobe de `6,25 %` para `7,61 %` (um vértice; o `p95`
  fica em `0,27 %`) e as dobras de `2` para `3`.

---

## §10 — As TRÊS correcções que o próprio produto impôs à lei, cada uma com a medição

A lei do §7 estava certa sobre o mecanismo e **errada três vezes sobre a forma**. Cada erro
foi apanhado por medir através da porta, e cada um é uma armadilha reutilizável.

### §10.1 — ⛔ O limiar de paragem foi calibrado no programa errado

`EXTRACT_SETTLE` saiu de uma varredura da relaxação **sozinha** (`1e-2` ⇒ 93 rondas,
`7,8° → 4,5°`). Na porta ela corre **depois** do Laplaciano, e a mesma fracção deu **23
rondas** e `7,8° → 6,8°`. *O passo anterior pré-condiciona a malha, o movimento começa menor,
e o mesmo limiar relativo chega muito mais cedo.* ⇒ o valor que shipa (`1e-3`) está medido
**através da porta**, e a forma aberta (`finish_extracted_with`) existe para que a varredura
não possa voltar a medir outro programa.

### §10.2 — ⛔⛔ A comparação de Pareto contra a MELHOR ronda é uma catraca

A relaxação **mergulha antes de subir**: as primeiras rondas melhoram uma coluna e pioram
outra. Comparando cada ronda com a **melhor até então**, bastava uma ronda inicial ganhar
numa coluna para nenhuma das seguintes conseguir dominá-la — e a corrida entregava a ronda
zero. Medido: com a catraca, **as quatro peças na densidade fina saíam intocadas**
(`ficou a 0`, `128` rondas = a paciência).

⭐ A cura é comparar a aceitação com a **ronda zero** e escolher, entre as aceitáveis, pela
mediana. A garantia fica mais simples de dizer — *o que sai nunca é pior que o que shipava em
nenhuma das três colunas* — e o mergulho deixa de a cegar.

### §10.3 — ⭐⭐⭐ Há peças em que a lei alinhada não tem nada a dizer, e a cega tem

Mesmo com a aceitação corrigida, há peças em que a relaxação **alinhada** piora uma coluna
logo à primeira ronda e nunca é aceite. A porta passa então a vez à lei **cega** — e só
então, o que garante que onde o relevo estava em jogo ele fica guardado.

⚠️ **O preço está medido e é o relevo** (`17,7° → 19,3°` no gancho fino). ⭐ A troca faz-se
com os números na mão: o oráculo entrega `13,3°` naquela peça — *já estamos atrás dessa
coluna com ou sem isto* — e as três colunas que a barra dele nomeia são onde a troca nos põe
à frente.

### §10.4 — ⛔ E uma cura CONSTRUÍDA que não foi adoptada: suavizar o campo de direções

A hipótese era que a direção principal **por face** é ruidosa. ⭐ Construiu-se a suavização
4-RoSy (`ph2d_quadfill::quality::smooth_hint`, com a rotação de quarto de volta que impede
duas vizinhas alinhadas de se cancelarem) e mediu-se:

| onde a suavização corre | `wrinkled` fina | `hooked` grossa |
|---|---|---|
| **nenhuma** | não se mexia | `7,7° → 4,3°` · aspecto `1,09` · relevo `14,6°` |
| ⛔ nas faces da **SAÍDA** (8 rondas) | ⭐ `5,2° → 2,6°`, `p99 15,0°` | ⛔ `7,7° → 5,8°` · aspecto `1,13` · **1** face `>4×` · relevo `18,6°` |
| na **superfície** (8 rondas) | — | — |

⛔ **Um raio contado em VIZINHOS é um raio em unidades de mundo diferentes em cada
densidade** — foi por isso que ela ajudava a malha fina e estragava a grossa. Mudá-la para a
superfície corrige o domínio, e a medição seguinte mostrou que ela **deixou de ser
necessária**: com a aceitação do §10.2 a densidade fina passa a melhorar sem suavização
nenhuma. ⇒ `HINT_SMOOTH_ROUNDS = 0`, **o código fica vivo e testado, e a medição é o
resultado**.
