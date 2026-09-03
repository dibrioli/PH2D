# ⭐⭐⭐ A GRADUAÇÃO DA PONTA — o plano, e as três rotas de que só uma sobrou

> **Estado:** aberto (2026-08-30). Dono: `line/quadextract`.
> **Régua de fecho:** a coluna `ENTREGA razao ponta/corpo`, alvo **`0,59`** (derivado da
> retopologia que o dono APROVOU). Hoje: **`1,338`**.

## §1 — O report, e o que ele de facto é

Dois reports do dono sobre a mesma coisa: *«as pontas finas perdem detalhe»* (28/08) e
*«buracos nas pontas, faces emboladas nas pontas»* (29/08).

⚠️ **A régua que os vê não é nenhuma das globais** — o `edge_max`, o `χ`, o enviesamento
mediano e a contagem de quads passam todos com a ponta grosseira. A que os vê é a razão
entre a mediana de aresta-equivalente na **casca radial exterior** e a da casca **mais
populada** (`ph2d_quadfill::tip_body_ratio`).

## §2 — ⛔ O que estava escrito e está REFUTADO

O §8-octodecies do handoff de 30/08 dizia *«a entrada dele já traz a densidade certa; o
trabalho é parar de a deitar fora»*. ⛔ **A linha da tabela chamava ENTRADA a uma SAÍDA**
(`sculpt_t003.obj`: `0` triângulos, valência máxima `6` — é uma retopologia). Ver o
§8-novendecies do handoff.

⭐ Com as entradas a sério a conclusão **inverte-se**: as esculturas do dono medem
**`2,026`** e **`3,650`** — elas são *anti-adaptativas*, com faces **duas a três vezes
maiores** na ponta. ⇒ **a graduação tem de ser CRIADA.**

## §3 — ⭐⭐⭐ ONDE ela morre, medido com controlo dos dois lados

| peça | `ENTRADA` | **`F1`** | `SAIDA` |
|---|---|---|---|
| `sculpt_antes.obj` | `2,026` | ⛔ **`1,007`** | `1,338` |
| `sculpt_t003.obj` | `0,675` | ⛔ **`1,020`** | `0,884` |

⭐⭐⭐ **A fase zero achata em `1,00` vinda de cima E de baixo.** As cascas da malha de
trabalho saem `0,0649 / 0,0650 / 0,0654`: um campo rigorosamente uniforme, não uma
tendência fraca. ⇒ a `ph2d-remesh-iso` é o sítio.

## §4 — As três rotas, e por que só uma sobra

| rota | estado | mecanismo |
|---|---|---|
| **A** — a F1 gradua (`PH2D_ISO_ADAPT=1`) | ⛔ **medida e recusada** | cura a agulha (alcance `−15,8 % → −0,8 %`) e **parte** a jusante (`χ` `1 → −7`, bordo `4 → 62`, `6×` o relógio) |
| **B** — o mapa gradua (`Follow Curvature`) | ⛔ **medida e recusada** | pede-se `400 %` e a saída move-se `7 %`: com `h` a variar o campo alvo `direcção/h` deixa de ser integrável e a projecção fica com a parte integrável |
| **C** — *preservar* a que chega | ⛔ **não existe** | §2: a entrada é anti-adaptativa |

⇒ **A e B são as únicas rotas, e cada uma foi medida SOZINHA.** ⚠️ *Nenhuma das duas
mediu o PAR*, e as duas recusas dizem, cada uma, que a cura tem de tratar as duas fases
ao mesmo tempo (*«uma fase medida sozinha pode melhorar e piorar o produto»*).

## §5 — ⭐ A hipótese com endereço: a A não RENORMALIZA a contagem

A [`SizingGrid`](../../../crates/ph2d-remesh-iso/src/sizing.rs) multiplica o alvo por
`clamp(mediana/κ, 1/4, 1)` — o tecto é `1`, logo ela **só afina, nunca engrossa**. Sem
compensação a malha de trabalho vai de `3 982` para **`33 156`** faces, e é essa inflação
(e não a graduação) que a jusante não digere.

⭐⭐ **A irmã dela um nível acima já resolve isto:** a `sizing_field` da shell escala o
campo por `√(N_previsto / N_pedido)` com `N = Σ_face área/h²` — *«a adaptação move os
quads; ela não os cria»*. A `SizingGrid` **não tem** esse passo.

⇒ **A wave é:** dar à `SizingGrid` a mesma renormalização, e medir o PAR (A+B) contra as
duas células sozinhas.

⚠️ **O preço declarado:** com renormalização a grelha passa a ser **mais grossa que o
alvo** nas regiões planas — o que contradiz o invariante que o doc dela declara hoje
(*«nunca grosseira»*). *Esse invariante é exactamente o que a fazia inflar.*

## §6 — A matriz que decide

`sculpt_antes.obj`, `Detail 0,85`, as quatro células `PH2D_ISO_ADAPT × PH2D_ADAPT`.

<!-- MATRIZ: preenchida pela corrida -->

## §7 — ⭐⭐⭐ AS TRÊS RECUSAS SÃO A MESMA, e ela nomeia o que falta

Três portas desta cadeia nascem desligadas com tabela ao lado, e **as três medem o mesmo
mecanismo**:

| porta | o que muda | o que a jusante mede |
|---|---|---|
| `PH2D_ISO_ADAPT=1` | malha de trabalho `3 982 → 33 156` faces (**8,3×**) | `χ` `1 → −7` · bordo `4 → 62` |
| `PH2D_ISO_FACING=1` | `3 982 → 9 458` faces (**2,4×**), valência até `23` | `χ` `1 → −16` · bordo `4 → 250` |
| `PH2D_F1_TARGET=1` | malha de trabalho mais fina (segue o alvo) | `χ` `2 → 1` · bordo `0 → 4` · dobras `21 → 123` |

⭐⭐⭐ **A variável comum é a CONTAGEM, não a graduação.** Cada uma das três dá ao campo
cruzado uma malha com mais faces do que a que ele sabe ler — e o doc da terceira já o diz
por palavras: *«a remalha grosseira é o filtro que faz o campo cruzado ver a forma e não o
ruído»*.

⇒ ⚠️ **Nenhuma delas mediu «graduar SEM aumentar a contagem»**, que é a única forma que
respeita as três tabelas ao mesmo tempo. A `SizingGrid` não pode fazê-lo hoje porque o
tecto dela é `1` — ela **só afina**. Uma banda **simétrica** (`[alvo/√R, alvo·√R]`) mais a
renormalização `√(N_previsto/N_pedido)` mantém o orçamento e move-o para onde a forma
aperta.

⚠️ **E isso é exactamente a lei que a irmã um nível acima já tem e já é gateada**
([`ph2d_quadflow::ScaleField::adaptive_between`] + a renormalização da `sizing_field`):
*«a adaptação move os quads; ela não os cria»*. ⭐ A `SizingGrid` é a única das duas que a
não tem — e é a única das duas que infla a malha `8×`.

## §8 — ⭐⭐⭐ A MATRIZ, e o que ela revelou: o knob NÃO é fraco, ele é DESCARTADO

`sculpt_antes.obj`, `Detail 0,85`. ⚠️ Máquina sob carga `26`–`58` (outras linhas a correr):
**as colunas de relógio desta corrida não valem nada**; as de geometria e topologia sim.

| `ISO_ADAPT` | `ADAPT` | malha F1 | `ENTREGA` F1 | quads | `χ` | bordo | **`ENTREGA` saída** | alcance |
|---|---|---|---|---|---|---|---|---|
| `0` | `0` | `3 982` | `1,007` | `9 414` | `1` | `4` | `1,502` | `−12,4 %` |
| `0` | `1` | `3 982` | `1,007` | `9 414` | `1` | `4` | ⛔ **`1,502`** | `−12,4 %` |
| `1` | `0` | ⛔ `33 156` | `0,936` | `9 740` | ⛔ `−7` | ⛔ `62` | ⭐ **`1,075`** | `−17,3 %` |
| ⭐⭐⭐ **`1`** | **`1`** | ⛔ `33 156` | `0,936` | `9 322` | ⛔ `−5` | ⛔ `36` | ⭐⭐⭐ **`0,536`** | `−15,2 %` |

⭐⭐⭐ **A célula 4 é a ÚNICA que chega ao alvo — e ela é o PAR que nenhuma das duas recusas
mediu.** `0,536` contra o alvo `0,59`, com as cascas a afinar `0,0532 → 0,0452 → 0,0382 → 0,0242`
(**`−54 %`**, e a referência aprovada faz `−52 %`). *A graduação da ponta está resolvida no
instante em que a malha fechar.* ⇒ **o problema inteiro colapsou num ponto: a topologia rasga.**

⭐⭐ **E as duas metades sozinhas não chegam lá:** a F1 graduada dá `1,075` e o mapa avisado dá
`1,502` (nada). *Cada recusa mediu meia cura e concluiu que a cura não servia* — que é exactamente
o que os dois docs delas avisam ao dizer, cada um, que *«a cadeia inteira tem de ser consciente do
sizing»*.

⭐⭐⭐ **A célula 2 é BYTE-IDÊNTICA à 1** — os `9 414` quads, as três medianas por casca
(`0,0403 / 0,0471 / 0,0606`), as contagens (`8 544 / 717 / 153`), `dobras 76`, o alcance.
*Não é «a adaptação move 7 %»: é ZERO.*

### §8.1 — ⭐ E a causa não é o solver: é uma REDE de segurança a funcionar

O campo **PEDE** `0,471` (melhor que o alvo `0,59`!) e a cadeia entrega `1,502`. ⛔ **Mas o
mapa adaptativo nunca chega à saída:** a
[`sculpt3d_history_retopo_extract.rs`](../../../shells/desktop/src/sculpt3d_history_retopo_extract.rs)
tem, desde 30/08, a guarda

```rust
let uniforme = if adaptive > 0.0 && still_broken(&out) { /* corre a corrida INTEIRA sem campo */ };
```

⇒ *se a saída adaptativa fica partida, corre-se de novo **sem** campo e o [`worse`] escolhe.*
A nossa saída tem **`4` arestas de bordo** ⇒ a rede arma **sempre** nesta peça, e o
resultado uniforme ganha. ⭐ **A adaptação está construída, medida, e é sistematicamente
deitada fora nesta peça.**

### §8.2 — ⛔⛔⛔ E o `worse` NÃO TEM CHAVE PARA A PONTA

A ordem é `bordo+não-manifold → componentes → gravatas → >60° → enviesamento`. **Nenhuma
delas é a densidade da ponta.** ⇒ uma corrida que cura as pontas e traz *uma* aresta de
bordo a mais perde **antes** de a ponta ser sequer olhada.

⚠️ **É a mesma forma que esta linha já pagou quatro vezes** (o `edge_max` cego ao quad de
`0,02 × 0,30`, o `χ` cego à almofada, o `open_edges` cego à gravata, o `QuadShape` cego às
três faces emaranhadas numa ponta): *a régua que ESCOLHE é cega ao eixo de que o dono se
queixa.*

### §8.3 — ⭐ A varredura que escolheu o alisamento não tinha coluna de TOPOLOGIA

A tabela no doc de `SIZING_SMOOTH_ROUNDS` mede `quads · razão · faces na ponta · >60° ·
envies. p99` — e **não** `bordo` nem `χ`. ⇒ ela mediu a adaptação a funcionar
(`1,533 → 1,044`) sem poder ver que o mapa **rasga**. *A rede de segurança foi acrescentada
depois, noutra wave, e as duas nunca se viram na mesma tabela.*

## §9 — ⭐⭐⭐ O PANIC «SEM ENDEREÇO DESDE 26/08» ESTÁ VIVO, e tem reprodutor

O `CLAUDE.md` §5 diz que ele *«não está provado curado nem provado vivo»*. ⭐ **Está vivo:**

```
panicked at crates/ph2d-gridmap/src/assembly.rs:198:34
```

reproduzido na célula `ISO_ADAPT=1 · ADAPT=0` desta matriz. A linha é
`partners[pb as usize][*lb as usize]` — o **segundo** lado de uma costura indexa um vértice
local que o patch dele não tem (o primeiro lado, três linhas acima, passou).

⚠️ **Ele é apanhado pelo `catch_unwind`** da tentativa, logo o botão não morre — perde uma
candidata em silêncio. *Uma rede que engole um `panic` sem o contar não distingue «esta
tentativa era pior» de «esta tentativa rebentou».*

## §10 — ⭐⭐⭐ O MECANISMO DO PANIC (e, muito provavelmente, das 4 ARESTAS DE BORDO)

Leitura do código, ainda **sem medição** — a medição é o item 1 da §11.

Em [`cut.rs`](../../../crates/ph2d-gridmap/src/cut.rs), a tabela de costuras casa os dois
lados de um arco **pela POSIÇÃO na lista** `across[(a,b)]`, e não pela identidade do patch:

```rust
for (slot_i, &(p, f)) in here.iter().enumerate() {
    if sides.len() <= slot_i { sides.push((p, vec![None; chain.len()])); }
    let Some((sp, marks)) = sides.get_mut(slot_i) else { continue };
    *sp = p;                       // ⛔ sobrescreve o patch do lado
    …  *m = corner_local[p][(f,k)] // …enquanto `marks` já tem locais de OUTRO patch
}
```

⚠️ **A lista `across[(a,b)]` é construída dentro do laço por patch**
(`across.entry((a,b)).or_default().push((pid, f))`), logo a ordem é por `pid` crescente —
**estável só enquanto cada patch contribui exactamente UMA face por aresta**. ⛔ O laço
interior é `for &f in sites`: um patch com **duas** faces a tocar a mesma aresta empurra
duas entradas, e a partir daí o `slot 1` deixa de ser o mesmo patch ao longo da cadeia.

⇒ **O lado fica com o patch da ÚLTIMA aresta e com índices locais de vários patches.** É
exactamente a forma do estouro:

```rust
partners[pb as usize][*lb as usize]   // assembly.rs:198 -- `lb` é local de outro patch
```

⚠️ **E a mesma mistura, quando não estoura, dá uma costura que não fecha** — que é a
assinatura das `4` arestas de bordo que a saída desta peça tem **mesmo sem adaptação
nenhuma**. *Uma hipótese, não um facto: mede-se contando as discordâncias.*

⛔ **A cura NÃO é chavear por patch.** O doc do [`SeamSide`] declara que *«pode ser o MESMO
patch nos dois lados — é o caso da ponte que abre um anel, e não é um erro»*: um mapa
`patch → lado` fundiria os dois lados de uma ponte. ⇒ a identidade de um lado é o par
**(patch, face)**, e a atribuição a `0`/`1` tem de ser fixada **uma vez** e depois
**casada**, nunca reposicionada por aresta.

## §11 — A ordem de trabalho ⚠️ **(executada — ver §12 em diante)**

1. ⭐ **MEDIR primeiro:** um contador em `CutReport` — quantas arestas da cadeia
   discordam do patch que o lado já tinha. ⛔ Se der `0` nesta peça, a §10 está errada.
2. **`RemeshRefusal::Panicked`** — hoje um estouro é devolvido como
   `TooCoarseToResolve`, e o artista lê *«a malha é grossa demais»* sobre um bug nosso.
   *A `ph2d-quadchain` já distingue (`Verdict::Panicked`); esta porta não.*
3. **O `assembly` conta em vez de estourar** (`get_mut` + `rep`), para o número da (1)
   sair mesmo nas corridas que hoje morrem.
4. **Registar cada candidata** (`bordo · não-manifold · gravatas · >60° · ENTREGA`) — sem
   isto não se sabe **porquê** a adaptativa perde, e é a pergunta que decide a wave.

## §12 — ⚠️ A §10 é REAL, e a 1.ª leitura mediu a peça errada

Na `sculpt_antes.obj`: `side_patch_flips = 0` e `mismatched_locals = 0` em **todas** as
candidatas de **todas** as células. ⛔ **E isso foi escrito aqui como «hipótese refutada».**

⭐⭐⭐ **Na `_base_sculpt.obj` — a escultura mais recente do dono — a mesma coluna dá `2`, e a
irmã dá `4`.** É exactamente a peça em que o `map.uv[p][l]` da `solve.rs` estourava. ⇒ *o
casamento dos dois lados por POSIÇÃO é uma causa real; ela só não dispara em todas as peças.*

⛔⛔ **A refutação valia sobre a fixtura em que correu.** É a mesma forma que esta linha já pagou
com a cura medida numa fixtura sem o fenómeno — e a única razão de o número ter aparecido é que
o estouro deixou de matar a candidata **antes** de ela imprimir.

⇒ ⏳ **A cura fica NOMEADA e não construída:** a identidade de um lado é o par **(patch, face)**,
e a atribuição a `0`/`1` tem de ser fixada uma vez e depois **casada**, nunca reposicionada por
aresta. ⛔ Chavear por patch **não serve** — o doc do `SeamSide` declara que os dois lados podem
ser o mesmo patch (a ponte que abre um anel).

## §13 — ⭐⭐⭐ A CURA JÁ ESTAVA A SER PRODUZIDA E ERA DEITADA FORA PELO DESEMPATE

O registo por candidata (⭐ construído nesta janela porque **um knob descartado e um knob fraco
liam-se exactamente igual**) mostra o que a cadeia de facto produz. `sculpt_antes.obj`,
`Detail 0,85`:

**Caminho de OMISSÃO (`ADAPT=0`), as três candidatas:**

| campo | quads | bordo | `>60°` | **`ENTREGA`** | |
|---|---|---|---|---|---|
| liso (`w=0`) | `9 484` | `28` | `2` | `1,585` | |
| alinhado (`w=0,03`) | `9 414` | `4` | `2` | `1,502` | ⛔ **a escolhida** |
| ⭐ **linhas de feição** | `9 121` | **`4`** | **`2`** | ⭐ **`0,851`** | perde |

⭐⭐⭐ **A terceira EMPATA em furos, peças, gravatas e faces `>60°`.** Ela perdia **só** na última
chave — o **enviesamento mediano** —, que é a única das três grandezas que o dono nunca nomeou.
⇒ *o eixo de que ele se queixou três vezes não estava na função que escolhe.*

**Com `Follow Curvature = 1`:**

| campo | quads | bordo | `>60°` | **`ENTREGA`** |
|---|---|---|---|---|
| liso | `8 078` | `20` | `0` | `1,076` |
| alinhado | `7 472` | `8` | `12` | `1,040` |
| ⭐⭐⭐ **linhas de feição** | `8 307` | `6` | `6` | ⭐⭐⭐ **`0,543`** |
| (recaída sem campo, a que ganha) | `9 414` | **`4`** | `2` | `1,502` |

⭐⭐⭐ **`0,543` contra o alvo `0,59`** — e perde por **DUAS arestas de bordo** em ~`16 600`.

⇒ ⛔ **A chave dos FUROS fica à frente** (foi a queixa dele três vezes), logo esta célula continua
a perder — e está certo. ⭐ **Mas a do caminho de omissão empata em tudo**, e é essa que a chave
nova da ponta faz ganhar.

## §14 — O que se ligou, e o que NÃO se ligou

- ⭐ **`worse` ganha a chave da ponta**, entre `>60°` e o enviesamento mediano
  ([`sculpt3d_retopo_rulers.rs`](../../../shells/desktop/src/sculpt3d_retopo_rulers.rs)).
  `PH2D_RETOPO_TIPKEY=0` bissecta. ⛔ **Nunca à frente dos furos.**
- ⛔ **Nada de `ISO_ADAPT`, nada de `Follow Curvature`** — a célula que atinge o alvo continua a
  perder por causa dos furos, e os furos são a queixa mais antiga. *A wave que os fecha é outra.*
- ⛔ **A renormalização da `SizingGrid`** (§5) foi desenhada e **não construída**: ela deixou de
  ser o caminho mais curto quando a medição mostrou que a cura já existia noutro sítio. O desenho
  fica escrito para quando os furos forem o assunto.

## §15 — ⭐⭐ A PROPRIEDADE que torna a chave nova segura

A chave da ponta é a **penúltima**: `furos → peças → gravatas → >60° → PONTA → enviesamento`.

⇒ ⭐⭐⭐ **Ela só pode mudar o resultado quando TODAS as chaves de defeito empatam.** Ela não
compra uma ponta fina com um furo, nem com uma peça solta, nem com uma face cruzada, nem com uma
face de canto pior que `60°` — em qualquer desses casos a candidata perde **antes** de a ponta ser
olhada.

⚠️ **É isso que responde à regressão conhecida das linhas de feição** (*«triplica as faces com
canto pior que 60°»*, medida noutra peça): onde essa regressão acontece, a candidata perde na
chave `>60°`, que vem **antes**. *A chave nova não desarma nenhuma cerca; ela só desempata o que
já estava empatado.*

⇒ E o que ela substitui é o **enviesamento mediano** como desempate — a única das grandezas em jogo
que o dono nunca nomeou em report nenhum.

## §16 — O A/B da chave, nas três peças do dono (`Detail 0,85`)

| peça | `TIPKEY=0` | `TIPKEY=1` | mudou? | porquê |
|---|---|---|---|---|
| ⭐ `sculpt_antes` | `1,502` | ⭐ **`0,851`** | **SIM** | as três candidatas empatam em furos, peças, gravatas e `>60°` |
| `sculpt_t003` | `1,300` | `1,300` | não | a candidata boa (`0,881`) tem `8` furos contra `4` — **os furos ganham** |
| `_base_sculpt` | `0,988` | `0,988` | não | ⛔ **duas das três candidatas ESTOURAVAM** — não havia entre o que escolher |

### ⚠️ E o preço, medido na peça em que ela dispara

| coluna | `TIPKEY=0` | `TIPKEY=1` |
|---|---|---|
| quads | `9 414` | `9 121` |
| `χ` · bordo · não-manifold | `1` · `4` · `0` | `1` · `4` · `0` |
| faces `>60°` | `2` | `2` |
| ⭐ **`ENTREGA`** | ⛔ `1,502` | ⭐ **`0,851`** |
| enviesamento p50 | ⭐ `2,9°` | `5,6°` |
| enviesamento p99 | `20,5°` | `32,3°` |
| aspecto p50 | ⭐ `1,05` | `1,13` |
| defeitos locais | ⭐ `6` (`0,06 %`) | `10` (`0,11 %`) |
| torção máxima | ⭐ `61,7°` | ⛔ `165,4°` |
| defeitos na PONTA | ⭐ `0/239` | `2/513` |
| alcance | `−12,4 %` | `−13,6 %` |

⚠️ **Não é uma troca livre**, e a decisão tem de a nomear:
- ⭐ O enviesamento `5,6°` fica **dentro da barra do oráculo** (`4,8°`–`7,1°`); o `2,9°` era
  melhor que ele. *Passar de «melhor que a referência» para «dentro da referência» não é uma
  regressão de produto.*
- ⚠️ A torção máxima sobe para `165,4°` — **uma** face em `9 121`. Para calibrar: o que a mesma
  cadeia já **shipa** na `sculpt_t003` são `46` faces torcidas com máximo `105,8°` e `3/208`
  defeitos na ponta. *A saída nova é `4,6×` mais limpa que uma que já vai para o artista hoje.*
- ⭐⭐ E o eixo que melhora é o **único** que o dono fotografou e nomeou três vezes.

⇒ **A chave nasce LIGADA.** `PH2D_RETOPO_TIPKEY=0` devolve o comportamento anterior, para o A/B
ser dele.

---

# ⛔⛔⛔ PARTE II — A AMPUTAÇÃO (report do dono, 31/08: *«vamos corrigir as pontas»*)

## §17 — ONDE a ponta morre, medido ponta a ponta

`_base_sculpt.obj` (a escultura mais recente do dono), `Detail 0,85`. A régua é o **suporte
por ponta** (`tips`), que mede cada ápice em separado — ⛔ o ALCANCE é um extremo global e
esconde uma ponta cortada atrás de outra que sobreviveu.

| | fase zero (F1) | saída |
|---|---|---|
| ⛔ **o que shipa** | `3` de `4` cortadas · pior **`−6,9 %`** | `3` de `4` · pior ⛔ **`−41,2 %`** |
| ⭐ `PH2D_ISO_ADAPT=1` | ⭐⭐⭐ **`0` de `4`** · pior `−0,2 %` | `1` de `4` · pior `−4,9 %` |

⭐ **Alcance final: `−41,8 %` contra `−6,3 %`.**

## §18 — ⭐⭐⭐ E a causa está NUMA RAZÃO QUE JÁ ERA IMPRESSA: `ALVO/F1 = 0,34×`

| | aresta média | |
|---|---|---|
| a malha de trabalho que o F1 entrega | `0,1146` | |
| ⭐ o alvo de quad que o slider pede | `0,0386` | **`2,97×` mais fino** |

⛔⛔⛔ **A cadeia é obrigada a produzir quads TRÊS VEZES mais finos que os triângulos que
ela lê.** Uma ponta cujo raio local é menor que `0,1146` **não sobrevive ao passe de
colapso**, e morre antes de o campo cruzado existir.

⚠️ **E não é uma propriedade desta peça: é sistémica.** A `sculpt_antes.obj` no mesmo
`Detail 0,85` mede `ALVO/F1 = 0,41×`. *As duas metades do botão são ancoradas em coisas
diferentes:*

- o **F1** remalha para [`ph2d_remesh_iso::ALPHA`] `× diagonal da caixa` — uma fracção do
  **bounding box**, que **não sabe nada do slider**;
- o **alvo do quad** sai de `edge_for_detail_by_count`, ancorado na **ÁREA** e numa
  **contagem** — a lei que a wave de 28/08 escolheu precisamente por o bounding box não
  servir.

⇒ ⭐⭐ **A âncora da fase zero é a que ficou para trás naquela wave.** ⚠️ E ela é
auto-derrotante numa peça com espinhos: *um espinho longo infla a diagonal, logo uma peça
com espinhos recebe uma malha de trabalho MAIS GROSSA precisamente por ter espinhos.*

## §19 — ⚠️ A recusa que tem de ser RECONFERIDA antes de qualquer cura

`PH2D_F1_TARGET=1` (a fase zero segue o alvo) está registada como **REFUTADA** — e a
medição correu na **fixtura sintética** `espinhos:6`, não nas peças do dono. *Uma refutação
vale sobre a fixtura em que correu*, e esta linha pagou essa lição duas vezes em 30/08.

⇒ **A reconferência corre primeiro.** Só depois se desenha cura nenhuma.

## §20 — ⛔⛔ A RECONFERÊNCIA, e um defeito na PRÓPRIA SONDA que ela expôs

Com `PH2D_F1_TARGET=1` a linha `F1` do relatório saiu **idêntica** à do controlo — e a env
estava a mudar a saída. ⛔ **O bloco de diagnóstico da fase zero não corria o caminho do
produto:** ele chamava sempre `remesh_isotropic(ALPHA)`.

⚠️ **E havia uma segunda divergência no mesmo bloco:** ele calculava o alvo do slider por
`edge_for_detail_with` enquanto o produto usa `edge_for_detail_by_count` desde 28/08 —
imprimia `0,03861` onde o botão usava `0,03961`.

⇒ *Uma sonda que calcula por outra lei mede outro programa.* As duas estão curadas; a linha
`F1` passa a seguir a env, e o alvo passa a ser o do produto.

## §21 — A tabela da reconferência (as linhas `F1` do controlo, as de saída válidas)

`Detail 0,85`, os dois modos contra o que shipa:

| peça | | alcance | `χ` · bordo · não-manif. | pontas cortadas |
|---|---|---|---|---|
| `_base_sculpt` | ⛔ o que shipa | ⛔ `−41,8 %` | ⭐ `2` · `0` · `0` | `3` de `4`, pior `−41,2 %` |
| | `PH2D_F1_TARGET=1` | `−9,4 %` | `−1` · `24` · `4` | `3` de `4`, pior `−14,4 %` |
| | ⭐ `PH2D_ISO_ADAPT=1` | ⭐ **`−6,3 %`** | `0` · `8` · `1` | ⭐ **`1` de `4`**, pior `−4,9 %` |
| `sculpt_antes` | ⛔ o que shipa | `−13,6 %` | ⭐ `1` · `4` · `0` | `3` de `6`, pior `−34,0 %` |
| | `PH2D_F1_TARGET=1` | `−11,0 %` | ⛔ `−5` · `46` · `2` | `3` de `6`, pior `−23,5 %` |
| | `PH2D_ISO_ADAPT=1` | — | ⛔ `−7` · `62` · `2` | — |

⭐⭐ **A recusa do `PH2D_F1_TARGET` HOLDS na `sculpt_antes`** (troca `4` bordo por `46` para
ganhar `2,6` pontos de alcance) — e na `_base_sculpt` ela é uma **troca**, não uma recusa.

⭐⭐⭐ **E a graduada bate a uniforme-fina nas DUAS colunas** na `_base_sculpt`: melhor alcance
(`−6,3` contra `−9,4`) **e** melhor topologia (`8` bordo contra `24`), com contagens de malha
de trabalho parecidas (`21 038` contra `~27 000`). ⇒ *não é «quantas faces»: é se a malha de
trabalho está adaptada à forma.* A que fica grossa onde a forma é chapada é a que o campo
cruzado sabe ler.

⇒ **A cura tem a forma que a §5 desenhou:** graduar **sem inflar** — banda simétrica mais
renormalização da contagem. Na `sculpt_antes` a grelha adaptativa entregava `ALVO/F1 = 1,28×`
(uma malha de trabalho **mais fina que os quads**, `33 156` faces); com o orçamento reposto ela
tem de aterrar na contagem que a cadeia já digere.

## §22 — ⭐⭐⭐ A RENORMALIZAÇÃO: a topologia fica PERFEITA, e a amputação muda de dono

Construída em 31/08 ([`SizingGrid::build`](../../../crates/ph2d-remesh-iso/src/sizing.rs)):
banda **simétrica** (`[alvo/√R, alvo·√R]`) mais o factor `√(N_previsto/N_pedido)` medido
**pela própria grelha** (o `at()` leva o mínimo dos 27 vizinhos, logo normalizar o campo por
vértice e consultar a grelha deixaria a inflação de pé).

`Detail 0,85`:

| peça | | malha F1 | pontas cortadas no F1 | saída | alcance |
|---|---|---|---|---|---|
| `_base_sculpt` | o que shipa | `3 036` | `3` de `4`, pior `−6,9 %` | `χ 2` · `0` bordo | ⛔ `−41,8 %` |
| | ⭐ **graduada + renorm.** | `3 544` (**`+17 %`**) | ⭐ `1` de `4`, pior `−4,2 %` | ⭐ `χ 2` · `0` · `>60 = 0` | `−26,0 %` |
| | graduada **sem** renorm. | ⛔ `21 038` (`7×`) | `0` de `4` | ⛔ `χ 0` · `8` bordo · `1` n-m | ⭐ `−6,3 %` |
| `sculpt_antes` | o que shipa | `3 982` | `2` de `6`, pior `−11,5 %` | `χ 1` · `4` bordo | `−13,6 %` |
| | ⭐ **graduada + renorm.** | `4 578` (**`+15 %`**) | ⭐ `2` de `6`, pior **`−3,9 %`** | ⭐⭐ `χ 2` · **`0`** · `>60 = 0` | `−13,8 %` |
| | graduada **sem** renorm. | ⛔ `33 156` (`8,3×`) | — | ⛔ `χ −7` · `62` bordo | — |

⭐⭐⭐ **A renormalização faz exactamente o que prometia:** o orçamento fica (`+15/17 %` em vez
de `7`–`8×`) e **a avaria de topologia desaparece** — na `sculpt_antes` a saída fica *melhor
que a linha de base* (`4` bordo → `0`, `χ 1` → `2`, `>60` → `0`).

⛔⛔ **Mas o alcance final só melhora em metade do caminho** (`−41,8 % → −26,0 %`) e na outra
peça fica **plano**. ⇒ *com o mesmo orçamento, graduar não chega:* `3 544` faces não seguram
uma agulha **e** o corpo.

⭐⭐ **E a medição separa os dois efeitos, que estavam colados:**

| | topologia | pontas |
|---|---|---|
| **graduar** (a mesma contagem) | ⭐ melhora | melhora metade |
| **orçamento** (mais faces) | ⛔ piora | ⭐ salva |

⇒ ⚠️ **Graduar COMPRA margem de topologia.** É por isso que a célula seguinte — a fase zero no
**alvo do quad** *e* graduada dentro desse orçamento — é a que falta correr: ela pede o
orçamento que salva a ponta, com a graduação que paga a topologia.

## §23 — ⭐⭐⭐ A célula `(1,1)`: o F1 fica PERFEITO e a saída ainda corta ⇒ o amputador MUDOU DE DONO

Fase zero no **alvo do quad** (`PH2D_F1_TARGET=1`) **e** graduada dentro desse orçamento:

| peça | malha F1 | `ALVO/F1` | pontas cortadas no **F1** | saída | alcance |
|---|---|---|---|---|---|
| `_base_sculpt` | `22 960` | ⭐ `0,99×` | ⭐⭐⭐ **`0` de `4`**, pior `−0,2 %` | `χ 1` · `6` bordo · `3` n-m | ⭐ **`−9,0 %`** |
| `sculpt_antes` | `23 202` | ⭐ `0,99×` | ⭐ `1` de `6`, pior `−3,5 %` | ⛔ `χ −2` · `10` bordo · `2` n-m | `−14,3 %` |

⭐⭐⭐ **A fase zero deixa de cortar e a SAÍDA ainda corta `2` de `4` a `−23,6 %`.** ⇒ *a
amputação que sobra nasce a jusante do F1* — no campo, no traçado ou no mapa.

⚠️ **E parte dela é RESOLUÇÃO, não defeito:** a `_base_sculpt` tem uma agulha de raio local
`≈ 0,037` e o quad pedido a `Detail 0,85` mede `0,0399`. *Uma grade de quads tão grossos como o
raio do tubo não o pode envolver* — `2π·0,037 / 0,04 ≈ 5,8` quads à volta, que é o limite.
⇒ **há uma parte desta perda que só sobe com o `Detail`**, e dizê-lo é honesto, não desistir.

## §24 — O quadro completo, julgado pela ordem do próprio `worse` (furos primeiro)

| peça | configuração | furos | alcance |
|---|---|---|---|
| `_base_sculpt` | hoje | ⭐ `0` | ⛔ `−41,8 %` |
| | ⭐ **graduada + renorm.** | ⭐ `0` | `−26,0 %` |
| | no alvo do quad | `6` | ⭐ `−9,0 %` |
| `sculpt_antes` | hoje | `4` | `−13,6 %` |
| | ⭐ **graduada + renorm.** | ⭐⭐ **`0`** | `−13,8 %` |
| | no alvo do quad | ⛔ `10` | `−14,3 %` |

⇒ ⭐⭐⭐ **A graduada renormalizada é a ÚNICA que nunca piora a chave da frente**, e é melhor ou
igual à linha de base em **todas** as colunas das duas peças. *A do alvo do quad compra a ponta
com buracos, e buracos foram a queixa do dono três vezes.*

## §25 — ⛔ A BANDA SIMÉTRICA foi construída, MEDIDA e REVERTIDA

A mutação que a apagava **sobreviveu aos dois gates**: com a renormalização por cima, o factor
sai `> 1` e empurra tudo para cima do alvo — *o tecto deixa de ser observável no intervalo*.
⇒ **Ela teve de ganhar o lugar com um A/B ponta a ponta**, e perdeu:

| peça | banda simétrica | ⭐ tecto `1` (o original) |
|---|---|---|
| `_base_sculpt` | `3/4` cortadas, pior `−24,3 %` · alcance `−26,0 %` | ⭐ `3/4`, pior **`−8,4 %`** · **`−11,1 %`** |
| `sculpt_antes` | `2/6`, pior `−24,3 %` · `−13,8 %` | ⭐ **`1/6`**, pior `−25,1 %` · `−16,8 %` |

⇒ **Fica só a renormalização.** ⚠️ *Uma mutação que sobrevive pode ser código inerte — ou, como
aqui, código que faz a coisa errada e que a régua certa recusa.*

## §26 — ⭐⭐⭐ A varredura FINAL, com o código que fica

| `σ` | pontas cortadas na fase zero | furos na saída | alcance |
|---|---|---|---|
| `0,30` | `0/6` → `0/6` | `0` → `0` | `+2,8 %` → `+1,8 %` |
| ⭐ `0,14` | **`5/6` → `0/6`** | `0` → `0` | `+3,7 %` → `+0,3 %` |
| ⭐⭐⭐ `0,07` | `6/6` pior `−20,5 %` → **`−7,6 %`** | ⛔ `4` → ⭐ **`0`** | `−15,5 %` → ⭐ **`−3,5 %`** |
| ⭐⭐ `_base_sculpt` | `3/4` pior `−41,2 %` → **`−8,4 %`** | `0` → `0` | ⭐⭐ `−41,8 %` → **`−11,1 %`** |
| ⭐ `sculpt_antes` | **`3/6` → `1/6`** | ⭐ `4` → **`0`** | ⚠️ `−13,6 %` → `−16,8 %` |

⭐⭐⭐ **Cinco de cinco melhoram ou empatam nas pontas cortadas E nos furos**, e a agulha mais
fina — que saía com `χ = 1` e `4` arestas de bordo — passa a **fechar**.

⚠️ **A única coluna que piora numa peça é o ALCANCE da `sculpt_antes`**, e a régua por ponta diz
o contrário na mesma corrida (`3` cortadas → `1`): *um máximo global move-se com a pior ponta, e
a pior ponta mudou de identidade.* ⇒ é a mesma lição que fez esta linha construir o `tips`.

⇒ ⭐ **A porta nasce LIGADA** (`PH2D_ISO_ADAPT=0` desliga).

## §27 — O que fica ABERTO

1. ⏳ **A forma final não é um interruptor global.** A fase zero graduada devia ser mais uma
   **candidata** da corrida do botão, com o `worse` a decidir por peça — e para isso o `worse`
   precisa de uma chave de **amputação**, que ele não tem. ⚠️ A banda medida para ela já existe
   e é do repo: `ph2d_quadfill::TIP_CUT_PCT = −2 %`, cujo doc mostra uma **ordem de grandeza**
   entre pontas intactas (`−0,0 %`..`−0,4 %`) e cortadas (`−5 %`..`−22 %`).
2. ⏳ **A amputação que sobra nasce a JUSANTE do F1** (§23): com a fase zero perfeita a saída
   ainda corta `2` de `4`. Parte disso é **resolução** — na `_base_sculpt` a agulha tem raio
   local `≈ 0,037` e o quad pedido mede `0,0399`.
3. ⏳ **O `ALVO/F1` continua em `0,30`–`0,42`**: a fase zero é ancorada na **diagonal da caixa**
   (`ALPHA`) e o alvo do quad na **área/contagem**. *Um espinho longo infla a diagonal, logo uma
   peça com espinhos recebe uma malha de trabalho mais grossa precisamente por ter espinhos.*
   ⛔ Igualar as duas âncoras (`PH2D_F1_TARGET=1`) está medido e **recusado** (§21).

---

# ⭐⭐⭐ PARTE III — «uma apenas foi amputada, a menos densa em faces» (Enio, 31/08)

## §28 — O report, e por que ele é um DIAGNÓSTICO e não uma queixa

*«Melhor resultado até agora, uma ponta bem comprida com bom resultado. Mas um mistério: dentre
várias pontas uma apenas foi amputada (a menos densa em faces).»* — três fotos, a terceira um
grande plano do **funil**: quads grandes e irregulares a fechar num buraco, com a escultura
original a aparecer por baixo.

⭐⭐ **A observação dele é o mecanismo:** *a que morreu é a que ficou com a grelha grossa*. Isso
distingue as duas leituras possíveis (a graduação falhou **ali**, contra a cadeia falhou ao
acaso) e aponta a uma **saturação**, não a um erro aleatório.

## §29 — ⛔⛔⛔ O TETO DA GRADUAÇÃO, e ele não foi medido para este consumidor

```rust
let h = target * (median / κ).clamp(1.0 / ADAPT_RATIO, 1.0);   // ADAPT_RATIO = 4
```

⚠️ **O `4` é emprestado**, e o doc dele di-lo: *«é a mesma cerca de gradação que a
`ph2d_quadflow::MAX_ADAPTIVE_RATIO` declara noutra crate — duas células cujas escalas diferem por
mais do que isto deixam de ter aresta comum»*. ⛔ **Essa cerca responde a outra pergunta:** ela é
sobre a **grade de quads** transitar sem rasgar. O consumidor aqui é um **remalhador de
triângulos**, que não tem essa restrição.

⇒ ⭐ É o `§0.0` do `CLAUDE.md`: *um limite legítimo diz de que recurso ele é.* Este diz de um
recurso de **outro** subsistema.

### A aritmética que fecha com a observação dele

O alvo da fase zero é `ALPHA × diagonal ≈ 0,11`, logo o mais fino que a grelha alcança é
`0,11 / 4 ≈ 0,029`. Uma agulha de raio local **abaixo disso** satura: *a partir dali, uma agulha
mais fina recebe exactamente a mesma grelha que uma mais grossa.*

| raio local da ponta | quads a dar a volta ao tubo (`2πr / h`) |
|---|---|
| `0,037` (a maior agulha da peça) | `≈ 7` — sobrevive |
| `0,020` | `≈ 3,8` — ⛔ colapsa |

⇒ **É a assinatura exacta de «uma apenas foi amputada, a menos densa em faces».**

## §30 — ⭐⭐ E subir o teto agora é BARATO, porque a renormalização já lá está

Antes desta jornada, subir o teto **multiplicava** a malha de trabalho (o campo só afinava).
⭐ Com o orçamento renormalizado (`√(N_previsto/N_pedido)`), subir o teto **não acrescenta faces:
move-as** — mais orçamento para as agulhas, menos para o corpo chapado.

⇒ *A wave anterior é o que torna esta pergunta respondível.* Varredura a correr:
`ADAPT_RATIO ∈ {4, 8, 16, 32}`, com a coluna nova **`NO PISO %`** (quantos vértices batem no
teto) ao lado das pontas cortadas.

## §31 — ⭐⭐⭐ A varredura do teto, e o SEGUNDO achado que ela devolveu

`_base_sculpt.obj`, `Detail 0,85`, com a coluna nova **`NO PISO %`** (vértices que batem no teto):

| `ADAPT_RATIO` | `NO PISO` (1.ª ronda → pico) | pontas cortadas no F1 | pontas cortadas na SAÍDA | alcance |
|---|---|---|---|---|
| ⛔ **`4`** (o que shipa) | **`8,5 %` → `14,3 %`** | `1/4`, pior `−2,6 %` | `1/4`, pior `−5,9 %` | `−7,3 %` |
| ⚠️ `8` | `2,6 %` | ⭐ `0/4`, pior `−0,5 %` | ⛔ `1/4`, pior **`−43,0 %`** | ⛔ `−37,8 %` |
| ⭐⭐⭐ **`16`** | ⭐ `0,2 %` | ⭐ `0/4`, pior `−0,5 %` | ⭐⭐⭐ **`0/4`**, pior **`−0,4 %`** | ⭐ `−6,5 %` |
| `32` | `0,0 %` | `0/4`, pior `−0,7 %` | `2/4`, pior `−2,9 %` | `−6,1 %` |

⭐⭐ **A saturação existe e é grande:** com `4`, entre `8,5 %` e `14,3 %` dos vértices batem no
teto — *a partir dali uma agulha mais fina recebe exactamente a mesma grelha que uma mais grossa*.
Com `16` ela desaparece (`0,2 %`) e a saída fica com **zero** pontas cortadas.

### ⛔⛔⛔ E a linha do `8` é o segundo achado: o SELECTOR podia escolher a candidata amputada

Com a fase zero **perfeita** (`0/4` cortadas, pior `−0,5 %`) a saída cortou a ponta mais longa em
**`−43 %`** — porque a corrida trocou de vencedora (*campo liso* em vez de *alinhado*) e as duas
estavam limpas na topologia. ⇒ **o `worse` não tinha chave de amputação**, e a `−43 %` é a prova.

⭐ Ela existe agora, **entre as gravatas e as faces `>60°`**, com a banda **medida e do repo**
(`ph2d_quadfill::TIP_CUT_PCT = −2 %`, cujo doc mostra uma ordem de grandeza entre pontas intactas
e cortadas). ⛔ **Nunca à frente dos furos.** ⚠️ Sem referência, de propósito: as duas candidatas
vêm da mesma entrada, logo o alcance de uma contra a outra já é a comparação certa.

⇒ ⚠️ **A varredura de uma peça só não escolhe um número** (o `8` prova que a curva não é monótona:
ela mede a selecção, não só a graduação). A escolha do teto corre em **quatro** peças, já com a
chave de amputação a proteger a escolha.

## §32 — ⭐⭐⭐ A escolha do teto, em QUATRO peças e já com a chave de amputação

| peça (`Detail 0,85`) | teto `4` | teto **`16`** |
|---|---|---|
| ⭐ `_base_sculpt` | `1/4`, pior `−5,9 %` | ⭐⭐⭐ **`0/4`**, pior **`−0,4 %`** |
| `sculpt_antes` | `3/6`, pior `−23,2 %` | ⭐ `2/6`, pior **`−18,9 %`** |
| ⭐⭐ agulha `σ=0,07` | `6/6`, pior `−36,4 %` | `6/6`, pior **`−11,2 %`** |
| `σ=0,14` | `1/6`, pior `−2,0 %` | `1/6`, pior `−2,1 %` |

⭐⭐ **Melhor ou igual nas quatro**, e a topologia é **idêntica** nas oito células (`χ = 2`, zero
bordo, zero não-manifold, valência máxima `5`–`6`). ⇒ `ADAPT_RATIO = 4 → 16`.

⛔ **Subi-lo só é barato porque a renormalização já lá está:** sem ela, `16×` de refinamento
local **multiplicaria** a malha de trabalho; com ela o orçamento é o mesmo e só **muda de sítio**.
*A wave anterior é o que torna esta respondível.*

## §33 — ⚠️ E o que a chave de amputação MUDA, dito com honestidade

Ela maximiza o **ALCANCE**, que é a distância da ponta **mais longa**. ⇒ ela protege o espinho
grande — que é o caso `−43 %` que a fez existir — e ⛔ **não protege os outros**: na
`sculpt_antes` ela troca *«uma ponta cortada a `−25,1 %`»* por *«três cortadas, a pior a
`−23,2 %`»*, porque a segunda tem mais alcance.

⚠️ **É a mesma limitação que esta linha já nomeou três vezes: um extremo global não conta
quantas.** Uma chave **por ponta** precisaria da malha de ENTRADA dentro do `worse` — hoje ele
recebe só as duas candidatas —, e isso é uma mudança de assinatura com o seu próprio preço.

⇒ ⏳ **Fica NOMEADO**, e a régua já existe (`ph2d_quadfill::tip_survival`, que conta pontas
cortadas contra uma referência).

---

# ⭐⭐⭐ PARTE IV — «Estamos perto da perfeição» (Enio, 2026-08-31, duas fotos e uma seta)

## §34 — O report, e o que ele pedia primeiro

Duas fotos da mesma peça em duas densidades, a segunda com uma **seta vermelha** no bico de um
espinho, e uma frase: *«estamos perto da perfeição»*. ⇒ ele não está a acusar a topologia (que
fecha) nem a forma (que bate o oráculo): está a apontar **um bico**.

⛔⛔ **E o §5.0 do `CLAUDE.md` já dizia o que fazer primeiro:** *«NENHUMA RÉGUA O VÊ … a próxima
janela constrói a régua LOCAL antes de tocar em código»*. Esta parte é essa janela.

## §35 — ⛔⛔⛔ A PRIMEIRA MEDIÇÃO ACUSOU A PRÓPRIA RÉGUA: o `ALCANCE` mede a AMOSTRAGEM

A sonda do botão, na escultura do dono a `Detail 0,85`, imprimia **duas linhas que não podem
ser as duas verdade**:

| linha | o que dizia |
|---|---|
| `PONTAS` (suporte por ponta) | `0` de `4` cortadas, a pior **`−0,4 %`** |
| `ALCANCE` | entrada `3,0959` → saída `2,8943`, ⛔ **`−6,5 %`** |

⭐ **A causa é o CENTROIDE.** As duas réguas medem a distância máxima ao centroide, e o
`ALCANCE` tirava-o da **média dos vértices** — que é uma propriedade de *onde estão os
vértices*, não de *que forma eles descrevem*:

| centroide | deriva entrada→saída | alcance lido |
|---|---|---|
| ⛔ média dos **vértices** | `0,2129` | **`−6,5 %`** |
| ⭐ pesado pela **área** | `0,0037` (`58×` menos) | `+0,0 %` |
| a verdade (referencial comum) | — | `−0,1 %` |

⛔⛔ **E isto estava no CAMINHO DO PRODUTO, não numa sonda:** a *chave de amputação* que a
§31 acrescentou ao `worse` compara `reach(a)` com `reach(b)`, com a banda `TIP_CUT_PCT = −2 %`.
Medido: as saídas a `Detail 0,50` e `0,85` da **mesma** peça diferem **`1,06 %`** nessa régua e
`0,09 %` na verdade — *metade da banda, sem uma ponta se mexer.*

⚠️⚠️ **E o sinal é o pior possível.** Uma candidata que **corta** a ponta perde vértices longe
do corpo ⇒ o centroide dela **afasta-se** da ponta ⇒ o alcance medido **sobe**. *A régua
defendia exactamente a candidata que devia acusar.*

⇒ `ph2d_quadfill::reach` (novo, centroide pesado pela **área**), com a média dos vértices como
último recurso para uma malha sem área. As duas cópias — a do `worse` e a da sonda — passam a
chamá-la. **Gate:** `a_mesma_forma_amostrada_de_duas_maneiras_tem_o_mesmo_alcance`, e ele
carrega **o controlo**: a régua velha, na mesma fixtura, erra mais de `5 %`.

## §36 — ⭐⭐⭐ A VARREDURA: sempre a MESMA ponta, e monótona no tamanho do quad

`_base_sculpt.obj`, o botão, sete densidades. `F1` é a fase zero; `pior` é o suporte da pior
ponta.

| `Detail` | alvo do quad | `F1` | saída | pior |
|---|---|---|---|---|
| `0,40` | `0,1383` | `0/4` (`−0,5 %`) | `3/4` | ⛔ `−19,6 %` |
| `0,45` | `0,1205` | `0/4` | `3/4` | `−9,4 %` |
| `0,50` | `0,1049` | `0/4` | `1/4` | `−5,3 %` |
| `0,55` | `0,0914` | `0/4` | `1/4` | `−4,7 %` |
| `0,60` | `0,0796` | `0/4` | `1/4` | `−3,1 %` |
| `0,70` | `0,0604` | `0/4` | ⭐ `0/4` | `−1,2 %` |
| `0,85` | `0,0399` | `0/4` | ⭐ `0/4` | `−0,4 %` |

⭐⭐ **Três leituras que só a varredura dá:**

1. **A fase zero está ILIBADA em todas as sete linhas** — `0/4` sempre, com a pior a `−0,5 %`.
   *A amputação que sobra é 100 % a jusante do F1*, e a wave do `ADAPT_RATIO` fez o seu
   trabalho inteiro.
2. **É sempre a ponta `3`** — a vítima não muda com a densidade. Não é sorteio de fase da
   grade.
3. **O corte vale UMA CÉLULA**: `0,91` · `0,93` · `0,69` · `0,36` quads para `Detail` de `0,50`
   a `0,70` (e `2,55` a `0,40`). *O bico morre porque a última célula não cabe nele.*

## §37 — ⭐⭐⭐ E O SUPORTE NÃO VÊ METADE DO DEFEITO: o bico é curto **e GORDO**

A `Detail 0,50` a ponta `3` lê `−5,3 %` de suporte, e a superfície da saída mais próxima do
ápice está a **`0,2133`** — *duas células*. A saída fecha o espinho com um anel de raio `≈0,19`
onde a escultura tem `≈0,05`. ⛔ **A função de suporte é `max(v·d)`: ela diz *até onde* a peça
vai naquela direcção e NADA sobre a espessura com que lá chega.** É o «funil» das fotos de
30/08, e nenhuma régua desta linha o mede.

⇒ ⭐ **`ph2d_quadfill::tip_deviation`** — para cada ápice da entrada, a distância dos vértices
da entrada a menos de `3 × alvo` do ápice até à **superfície** da saída, dividida pelo `alvo`.

| | `Detail 0,50` | `Detail 0,85` |
|---|---|---|
| ponta 0 | `p50 0,13` `p90 0,22` | `0,19` / `0,37` |
| ponta 1 | `0,08` / `0,15` | `0,08` / `0,16` |
| ponta 2 | `0,13` / `0,30` | `0,30` / `0,44` |
| ⛔ ponta 3 | **`1,15`** / **`2,02`** | `0,20` / `0,30` |

⭐ **A barra é `TIP_DEVIATION_MAX = 1,0` e ela é o CHÃO DA DISCRETIZAÇÃO, não um número
escolhido**: uma grade de passo `h` não pode seguir uma superfície melhor que `h`. As pontas sãs
medem `máximo 0,45`; a partida mede `p50 1,15` — um vazio de `2,6×`, e a barra vive nele.

⚠️ **É ponto→FACE e não ponto→vértice**, e a diferença decidiu o desenho: com vértices a
população sã lê `p50 0,28`–`0,35` (o erro é meia aresta da saída) e com faces lê `0,08`–`0,30`.
*Uma régua cujo valor «são» é feito do artefacto da própria régua não tem onde pôr uma barra.*

⚠️ **E é ADIMENSIONAL** (dividida pelo alvo), que é o que permite pôr as duas densidades na
mesma tabela — uma medida em unidades de mundo diria que a saída mais fina é sempre melhor, o
que é verdade por construção e não informa nada.

## §38 — ⛔⛔ DUAS CURAS CONSTRUÍDAS, MEDIDAS E REFUTADAS — a falta é de CONECTIVIDADE

Com a régua na mão, as duas curas óbvias foram medidas antes de escritas em Rust.

**(a) Puxar o vértice mais avançado até ao ápice.** ⛔ Refutada: para a ponta `3` o défice de
*suporte* é `0,0952`, mas o vértice mais avançado **naquela direcção** está a **`0,6198`** do
ápice — o deslocamento leva o aspecto do quad a `12,11` e o enviesamento a `85°`.

**(b) *Shrinkwrap* da região da ponta** (mover todo vértice da saída a menos de `3` quads de um
ápice para a superfície da entrada). ⛔ Refutada, e o número é duplo: a malha é **destruída**
(aspecto `1,9·10⁸` — dois vértices colapsam no mesmo sítio) e a régua **mal se move**
(`p90 2,03 → 1,55`).

⭐⭐⭐ **É o achado da parte IV:** *mover vértices `76×` não cura, logo o que falta não são
POSIÇÕES — são CÉLULAS.* A grade não tem resolução para representar o último centímetro do
espinho, e nenhuma pós-passagem de posição inventa conectividade.

⇒ a cura de fundo é a que o `CLAUDE.md` §5 já nomeia: o **factor de escala conforme por
construção** (`Δ log h` contra a curvatura de Gauss), que é wave com espec própria. ⛔ Não é
afinação, não é acabamento, e não é o `finish_extracted`.

## §39 — ⭐ O que o dono pode fazer HOJE, medido

`Detail ≥ 0,70` na escultura dele dá **`0` de `4`** pontas cortadas. É a resposta honesta
enquanto a wave do factor conforme não existir — e ⛔ **não** é «suba sempre o slider»: a
`0,85` a peça sai com `9 730` quads contra `1 381` a `0,50`.

## §40 — ⚠️ O que a fixtura dos gates ensinou, e é uma propriedade da LEI

Ao construir o cone de teste, a 1.ª e a 2.ª redacção mediram **uma peça sem pontas**:

- **1.ª:** os anéis densos junto do ápice puxam o centroide para cima ⇒ o próprio ápice cai
  abaixo do piso de `0,55 × raio máximo` e a lei deixa de o ver. *O corpo tem de ter mais
  população que o bico — como uma escultura real tem.*
- **2.ª:** com `12` vértices no anel da base, os doze **empatam** em raio (a lei aceita
  empate), ficam à frente do bico na ordenação e enchem o `MAX_TIPS = 12` — ⛔ **o espinho era
  o 13.º**, a medição saía `12` pontas todas a zero, e lia-se como *«a peça está perfeita»*.

⚠️ **Um corte por posto é uma decisão sobre QUEM não é medido**, e um empate de doze
preenche-o inteiro.

## §41 — As provas de mutação (4 aplicadas, 1 SOBREVIVEU e o gate que faltava)

| mutação | veredito |
|---|---|
| o centroide de área vira a média dos vértices | ✗ morta |
| o `p50` deixa de acumular o pior entre as pontas | ✗ morta |
| a guarda do alvo não positivo desaparece | ✗ morta |
| ⛔ a região **interior** do ponto-triângulo vira «a distância ao canto `a`» | ⚠️ **SOBREVIVEU** |

⭐ **A sobrevivente diz o que faltava:** nenhuma fixtura dos quatro gates põe uma amostra sobre
o **meio** de uma face — e a cadeia real põe quase só isso (o quad a `0,10` contra a escultura a
`0,03`). ⇒ o gate novo é o da **propriedade que a função promete**, medida onde ela é definida:
`a_distancia_e_ao_interior_da_face_e_nao_ao_canto` (a perpendicular sobre o baricentro vale `1`
e a distância ao canto mais próximo vale `1,374`). Com ele, as quatro mutações morrem.

## §42 — ⭐⭐⭐ E A CHAVE DO SELECTOR TROCA DE RÉGUA — medida, e muda uma escolha

A chave de amputação da §31 comparava o **alcance** de duas candidatas. Com a régua nova na
mão, a pergunta *«ela escolhe bem?»* passou a ter resposta: o `log_candidate` passou a
imprimir o alcance **e** o desvio por ponta de cada candidata, e a varredura correu.

⚠️ **Só há um sítio onde a chave chega a falar:** ela vem depois de furos, peças e gravatas,
e nas quatro células medidas as candidatas empatam em bordo **uma vez** — `_base_sculpt` a
`Detail 0,40` (as outras três são decididas por bordo, e a chave nunca é consultada).

| `_base_sculpt`, `Detail 0,40` | quads | bordo | alcance | **pontas acima da barra** |
|---|---|---|---|---|
| ⛔ `w = 0,000` (a que o **alcance** escolhia) | `774` | `4` | `2,8644` | **`2` de `4`** |
| ⭐ `w = 0,030` (a que o **desvio** escolhe) | `804` | `4` | `2,7869` | **`1` de `4`** |

⭐⭐ *A régua velha preferia a candidata com MAIS pontas partidas* — porque a ponta que ela
media (a mais longa) sobrevivia nas duas, e a que morria não entrava na conta.

**Confirmado pelo produto**, correndo o botão depois da troca: o clique passa a devolver
`804` quads, a pior ponta vai de **`−19,6 %` para `−9,9 %`**, e o desvio de `p50 3,01`
(`2` acima) para `p50 2,78` (`1` acima).

⚠️ **O preço, dito inteiro:** o enviesamento mediano sobe de `7,48°` para `9,33°` e o aspecto
de `1,12` para `1,14`. ⭐ É a troca que a **ordem** desta função já declarava desde 30/08 —
*pontas antes da mediana do enviesamento, porque foi das pontas que o dono se queixou três
vezes e a mediana é a única das três que ele nunca nomeou.*

⚠️ **A chave é DISCRETA (a contagem), de propósito.** Um `p50` contínuo competiria com o
enviesamento em **toda** peça, incluindo as que não têm ponta partida nenhuma; a contagem só
fala quando há uma diferença de facto.

⛔ **A amostra vazia não decide** (`tips = 0` é *«não medido»* e lê-se igual a *«perfeito»*),
e o `reach` **fica** — já não como chave, mas como coluna do registo, agora honesta.

⚠️ **O que NÃO foi re-medido, dito:** a célula `ADAPT_RATIO = 8` que fez a chave nascer
(`−43 %` na ponta longa) já não é reproduzível com o teto em `16`. A chave nova cobre-a por
construção — uma candidata que come o espinho passa a barra naquele ápice e conta `+1` —,
mas isso é **raciocínio**, não uma medição, e vai escrito assim.

**Provas de mutação (3, todas mortas):** a chave decide ao contrário · a guarda da amostra
vazia desaparece · a chave desaparece.

---

# ⭐⭐⭐ PARTE V — «tenho usado Detail e Curvature no MÁXIMO» (Enio, 2026-08-31)

## §43 — ⛔⛔⛔ A VARREDURA INTEIRA DA PARTE IV CORREU COM `Curvature 0`

O dono disse, ao devolver o smoke: *«em todos os testes tenho usado Detail e Curvature no
máximo»*. ⛔ **As sete densidades da §36 correram com `PH2D_ADAPT` no default da sonda, que é
`0,0`** — a posição que ele nunca usa. *Uma varredura de sete células mediu um programa que o
dono não corre.*

⚠️ **É a família de §0.0 e das recusas medidas:** a sonda tem os seus próprios defaults, e um
default de sonda que não é o default do produto — nem a posição do dono — produz uma tabela
plausível sobre outra configuração.

## §44 — A tabela nas quatro esquinas (`_base_sculpt.obj`, a peça que ele exportou a 30/08)

| `Detail` | `Curvature` | quads | pontas cortadas | pior | **desvio `p50`** |
|---|---|---|---|---|---|
| `1,00` | `1,0` | `21 735` | `0` de `4` | `−0,2 %` | `0,22` |
| `0,75` | `1,0` | `5 554` | `0` de `4` | `−0,3 %` | ⭐ `0,14` |
| `0,75` | `0,0` | `5 625` | `0` de `4` | `−0,8 %` | `0,26` |
| `1,00` | `0,0` | `22 345` | `0` de `4` | `−0,9 %` | `0,41` |

⭐ **O `Curvature 1` MELHORA as pontas nas duas densidades** (`0,26 → 0,14` e `0,41 → 0,22`),
que é exactamente o que o knob promete.

⛔⛔ **E o `Detail 0,75 · Curvature 1` — a célula em que ele reporta uma ponta amputada — sai
LIMPA aqui.** *Não reproduzo o report na peça que tenho.*

## §45 — ⭐⭐⭐ O QUE A TABELA DAS CANDIDATAS DIZ, e é um aviso

Em **todas as quatro** células, uma das duas candidatas amputa e a outra não — e quem escolhe
a boa é a chave dos **furos**, não a das pontas:

| célula | candidata `w = 0,000` | candidata `w = 0,030` |
|---|---|---|
| `1,00 · 1,0` | bordo `10` · desvio **`2,86`** (`1` acima) | bordo `0` · desvio `0,22` (`0`) |
| `0,75 · 1,0` | bordo `8` · desvio **`3,57`** (`1` acima) | bordo `0` · desvio `0,14` (`0`) |
| `0,75 · 0,0` | bordo `4` · desvio **`4,16`** (`1` acima) | bordo `0` · desvio `0,26` (`0`) |
| `1,00 · 0,0` | bordo `10` · desvio **`3,78`** (`1` acima) | bordo `0` · desvio `0,41` (`0`) |

⚠️ **A candidata que amputa é sempre a do campo LISO, e ela traz sempre bordo.** ⇒ hoje o
resultado bom sai porque as duas coisas andam juntas — *e a margem é um acaso da peça, não um
desenho*. Se numa escultura o campo liso fechar a casca (`bordo 0`), quem decide passa a ser a
chave das pontas — que **só existe desde este commit**.

⇒ ⭐⭐ **Hipótese com endereço para o report do `0,75`:** ele correu o binário **anterior** à
troca da chave (a régua velha era o alcance, contaminado pelo centroide por vértice, e ela
*prefere* a candidata que come a ponta — §35). ⛔ É hipótese e vai escrita como tal: só o
re-smoke no binário de hoje, ou a peça dele, a confirma.

## §46 — ⛔ E o caminho tinha uma AFIRMAÇÃO FALSA a apanhar quem investigasse

`RetopoMode::uses_adaptive()` dizia *«só o motor local consome a densidade adaptativa»* — e o
`Follow Curvature` chega **aos dois** desde 2026-08-21 (o comentário no sítio da chamada
di-lo). ⚠️ Ela **não tinha um único leitor**, nem existia o aviso de painel que o doc dela
descrevia. ⇒ apagada, com o motivo no lugar dela.

⚠️ **É a espécie ÓRFÃ e não a MORTA**, e as curas são opostas. O perigo aqui não era o código:
era o **texto** — quem greppasse o nome lia *«o `Curvature` é inerte no motor de omissão»* e
concluía o contrário do que o produto faz. *Uma afirmação falsa que ninguém executa continua a
ser lida.*

## §47 — ⛔⛔ A HIPÓTESE DO PISO FOI CONSTRUÍDA, MEDIDA E REFUTADA

Ao ver a foto (uma bola com um espinho longo e vários curtos, a seta num **curto**), a
hipótese óbvia era a régua: `apices` tem um piso de `0,55 × raio máximo`, e nesta peça o
espinho longo mede `3,0959` ⇒ o corte fica em `1,70` e **só `4` de `42`** máximos locais
entram na conta. *A ponta apontada seria invisível a todas as réguas desta linha.*

⛔ **Medido, e é falso.** Todos os **42** máximos locais de `_base_sculpt.obj`, a
`Detail 0,75 · Curvature 1`, sem piso nenhum:

| | pior de 42 |
|---|---|
| suporte | **`−0,9 %`** (a barra do corte é `−2 %`) |
| desvio `p50` | **`0,21`** (a barra é `1,0`) |
| desvio `p90` | `0,42` |

*Nenhuma ponta desta peça é amputada nesta configuração — nem as que o piso escondia.*

⇒ ⭐⭐ **A peça que o dono está a testar não é a que ele exportou a 30/08.** As duas
hipóteses de §45 ficam ambas de pé; a que sobra é *outra escultura*, e o passo seguinte é
tê-la: no app, **`Ctrl+Shift+E`** escreve a escultura num ficheiro
([`sculpt3d_export.rs`](../../../shells/desktop/src/sculpt3d_export.rs), o par do
`Ctrl+Shift+O`).

⚠️ **A lição não é «a hipótese estava errada» — é que ela era barata de medir e cara de
acreditar.** Uma régua com piso *é* uma população escolhida, e este repo já pagou isso
quatro vezes; a diferença é que desta vez a suspeita foi medida antes de virar cura.

---

# ⭐⭐⭐ PARTE VI — A PEÇA DELE, e o defeito que ela expôs (2026-08-31, `_remesh_sculpt.obj`)

## §48 — O report REPRODUZ, e a diferença não era a peça: era ONDE ela está

O dono entregou a saída (`_remesh_sculpt.obj`) e disse que a entrada é a mesma
`_base_sculpt.obj` que eu já tinha. ⭐ **A saída dele veio noutro referencial** — `0,582×` o
tamanho e ancorada em `x ≈ 2`. Alinhada, as réguas dizem:

| | pior de 42 pontas |
|---|---|
| ponta **3** (raio `1,8014`) | ⛔ suporte **`−3,9 %`** · desvio `p50` **`2,82`** |
| as outras 41 | `≤ 0,4 %` · `≤ 0,18` |

⇒ o report é **verdadeiro** e a minha medição também era: *nós não estávamos a correr a mesma
coisa.*

⭐ **A causa do referencial está no importador:** `sculpt3d_import::IMPORT_SPAN = 2.0`
normaliza o tamanho e **ancora a peça fora da origem**. A sonda alimenta o ficheiro cru,
centrado; o artista alimenta a peça ancorada.

## §49 — ⛔⛔⛔ A MESMA MALHA, SÓ DESLOCADA, DÁ OUTRA RETOPOLOGIA

`_base_sculpt.obj` normalizada (`2,0` de vão), `Detail 0,75 · Curvature 1`, **só translada em
`x`**:

| `x` | quads | pontas cortadas | pior | desvio `p50` |
|---|---|---|---|---|
| `0` | `5 703` | ⭐ `0` de `4` | `−0,5 %` | `0,17` |
| `0,5` | `5 528` | `0` de `4` | `−0,3 %` | `0,12` |
| `1` | `5 432` | `1` de `4` | `−4,2 %` | `3,27` |
| `2` | `5 301` | ⛔ `2` de `4` | `−6,3 %` | `2,18` |
| `4` | `5 344` | ⛔ `2` de `4` | `−6,0 %` | `3,86` |
| `16` | `5 669` | `0` de `4` | `−0,6 %` | `0,14` |

⛔ **Não é precisão de `f32`:** a `16` volta a ficar limpa. Não é monótono, não é gradual — é
uma decisão discreta a mudar de lado.
⭐ **A fase zero está limpa nas seis** (`0/4`, pior `−0,9 %`), logo a sensibilidade não é dela
sozinha — mas ela **também** muda (`1 797`–`1 902` vértices nas seis posições).

## §50 — ⛔⛔⛔ E É PIOR: O DEFEITO É CAÓTICO NOS ÚLTIMOS BITS, não «translação»

Canonicalizar a pose na porta (correr a cadeia sempre com a peça na origem e devolver a saída
ao sítio) foi **construído e medido**. As seis entradas passam a diferir **só no arredondamento
de `f32`** — `(p − c) + c` não devolve `p` bit a bit — e o resultado foi:

| | quads | pontas cortadas |
|---|---|---|
| `x = 0` | `5 703` | `0` de `4` |
| ⛔ `x = 0,5` | `4 142` | `3` de `4`, pior **`−77,6 %`** |
| ⛔ `x = 1` | `3 950` | `4` de `4`, pior `−77,5 %` |
| ⛔ `x = 2` | `4 435` | `4` de `4`, pior **`−105 %`** |

⭐⭐⭐ **Uma perturbação de `~10⁻⁷` relativos muda a saída inteira, e pode destruí-la.** O tempo
também explode (`15 s → 172 s`). ⇒ *o que a §49 mediu não é uma dependência da POSIÇÃO: é
sensibilidade caótica a qualquer perturbação da entrada, e a posição é só a que o artista
consegue mexer sem querer.*

⛔ **A canonicalização foi REVERTIDA** — ela não pode remover uma perturbação que ela própria
introduz, e piorava o produto.

## §51 — ⛔ As duas curas tentadas, medidas e recusadas

1. **Ancorar a grelha de densidade da fase zero na caixa da peça** (`SizingGrid::key_of` divide
   a coordenada de **mundo** pela célula, logo os baldes movem-se com a peça). Construída,
   medida: o F1 **continuou** a mudar com a posição (`1 841`–`1 902` vértices) e o caso da
   origem **piorou** de `0/4` para `1/4`. *A hipótese estava errada.* Revertida.
2. **Canonicalizar a pose na porta** — §50. Revertida.

## §52 — ⇒ O que a próxima janela tem de construir PRIMEIRO

⭐⭐⭐ **Um gate de SENSIBILIDADE, antes de qualquer cura:** perturbar a entrada em `1` ULP (ou
transladá-la) e exigir que a saída fique dentro de uma barra — contagem de quads, pontas
cortadas, `χ`. ⛔ *Sem ele, toda medição desta linha — as sete densidades, as quatro esquinas,
os A/B de candidatas — mede uma amostra de uma lotaria, e a barra de qualquer cura é ruído.*

⚠️ **E isso reabre, com honestidade, tudo o que esta linha mediu por A/B de uma corrida só.**
As diferenças de `2`–`8 %` que decidiram constantes podem ser ruído desta família. *A
prioridade deixa de ser a ponta: é a REPETIBILIDADE.*

---

# ⭐⭐⭐ PARTE VII — «o remesh deve funcionar perfeitamente em qualquer lugar» (Enio, 31/08)

## §53 — A ordem, e o que ela muda na prioridade

Veredito do dono sobre a §49: **não há contorno**. ⇒ a invariância deixa de ser um achado e
passa a ser requisito, e vem **à frente** da ponta.

## §54 — ⭐⭐⭐ A PRIMEIRA CAUSA, LOCALIZADA E CURADA

⛔⛔ **A §51 declarou esta cura REFUTADA, e a declaração é que estava errada.** Ela foi medida
**através do botão inteiro**, onde o ruído a jusante a afoga. Isolada na crate, em `12 s`, ela
é inequívoca — `uv_sphere(96, 144)`, a mesma malha só transladada em `x`:

| | `0` | `½` | `1` | `2` | dispersão |
|---|---|---|---|---|---|
| ⛔ chave de mundo | `2 633` | `2 712` | `2 679` | `2 586` | **`4,9 %`** |
| ⭐ ancorada na peça | `2 687` | `2 687` | `2 687` | `2 687` | **`0,0 %`** |
| ⚠️ sem graduação (controlo) | `2 608` | `2 608` | `2 608` | `2 608` | `0,0 %` |

⭐ **O controlo é a metade que localiza:** sem campo o remalhador **já era** invariante ⇒ o
defeito era do campo, e não do laço. `SizingGrid` indexava por `p / cell` — coordenada de
**mundo** —, e como cada balde guarda o mínimo e o `at` lê o mínimo de 27, um deslocamento
muda que região herda a finura de uma agulha.

⚠️ **O canto MÍNIMO da caixa e não o centroide:** o centroide dos vértices é propriedade da
amostragem — é o defeito que a `reach` pagou no mesmo dia (§35).

## §55 — ⛔⛔ DUAS MUTAÇÕES SOBREVIVERAM, e a razão é uma lei

Mudar **só** a construção (ou **só** a consulta) faz as chaves nunca casarem, o `at` cai no
`fallback` constante e o campo **morre** — *e um campo morto é perfeitamente invariante.* O
gate passava.

⇒ o gate ganhou a segunda metade: **a graduação tem de MUDAR a malha**. E o filtro de mutação
ganhou o seu próprio controlo: **os dois sítios de uma vez**, senão a mutação não reproduz o
código antigo. *Meia mutação testa uma terceira coisa que nunca existiu.*

## §56 — ⏳ O QUE FICA ABERTO, sem enfeite

⛔ **O botão ainda NÃO é invariante na peça do dono:** com a grelha curada, as seis posições
dão `1 841` · `1 841` · `1 889` · `1 797` · `1 902` · `1 861` vértices de fase zero. A cura
tirou uma causa; **sobra a amplificação**.

⭐ O mecanismo está nomeado e é o mesmo da §50: o remalhador é **iterativo**, `p − origem`
perde bits quando a peça está longe, e **um** bit muda uma decisão de corte que cascateia. Na
esfera lisa isso não aparece (o campo é constante); numa peça com agulhas e `ADAPT_RATIO = 16`
o campo tem gradientes fortes e há fronteiras por toda a parte.

⇒ **A obra seguinte tem duas frentes, por esta ordem:**

1. ⭐⭐⭐ **Tirar o CLIFF do campo.** O `at` devolve o mínimo de 27 baldes — uma função em
   **degrau**. Um campo contínuo (mistura pesada pela distância em vez de `min` duro) faz uma
   perturbação de `10⁻⁷` mudar o alvo em `10⁻⁷` em vez de saltar para o valor do vizinho.
   ⚠️ **Com uma cerca:** o `min` existe para ser conservador (nunca mais grosso que o vizinho
   mais fino), e um contínuo ingénuo perde isso **exactamente na agulha** — é preciso um
   *soft-min*, e a barra é a régua por ponta que já existe.
2. **O gate de sensibilidade do §52 sobre o BOTÃO**, não só sobre a crate — com a peça do dono
   e as seis posições, e a barra em *pontas cortadas*, não em contagem de vértices.

## §57 — ⭐⭐⭐ O CAMPO ESTÁ ILIBADO, e a sonda que o disse custa 30 segundos

`o_campo_e_invariante_nos_mesmos_sitios` constrói a grelha na peça e na mesma peça deslocada,
e lê o `at` nos vértices correspondentes — **sem o remalhador pelo meio**:

| | pior desvio relativo | sítios que diferem acima de `10⁻⁶` |
|---|---|---|
| `x = ½`, `1`, `2` | `~2·10⁻⁷` | ⭐ **`0` de `13 682`** |
| `x = 16` | `1,5·10⁻⁶` | `19` de `13 682` |

⇒ **depois da ancoragem, o campo é invariante à precisão da máquina.** O que sobra na saída é
o **laço** a amplificar `10⁻⁷`: uma decisão de corte exactamente no limiar muda de lado, e a
partir daí os índices são todos outros.

⭐ **E o controlo prova que a amplificação precisa de um campo VARIÁVEL:** com o campo
constante (`graded = false`) o mesmo laço, na mesma peça, é bit-estável nas quatro posições.

## §58 — ⛔⛔⛔ TIRAR O CLIFF DO CAMPO: construído, MEDIDO e REVERTIDO

A §56 mandava tornar o campo contínuo. Foi feito, em três passos, cada um medido:

| passo | dispersão do F1 (esfera 96×144) | o que aconteceu |
|---|---|---|
| planalto + rampa, 1 representante por balde | `4,4 %` | ⛔ pior: *qual* vértice representa a célula é um degrau na **construção** |
| **todas** as amostras, sem escolha | `3,0 %` | melhor, ainda não fecha |
| + alvo **quantizado** em `64` degraus | `1,4 %` | melhor, ainda não fecha |

⛔⛔ **E o produto PIOROU, que é o que decide.** No botão, `Detail 0,75 · Curvature 1`, nas
seis posições: a origem foi de **`0` de `4`** pontas cortadas para **`1` de `4`**, e uma
célula deu **`−40,8 %`**. A fase zero encolheu de `~1 841` para `1 695`–`1 807` vértices.

⭐⭐⭐ **O achado é o mecanismo:** *o `min` duro das 27 células não era um acidente — é ele que
ALIMENTA a agulha.* Ele espalha o valor mais fino por toda a vizinhança sem o diluir; qualquer
mistura suave, por melhor que seja a continuidade, entrega **menos** resolução exactamente
onde a ponta precisa dela. ⇒ *a estabilidade e a ponta pediram coisas opostas, e a ponta é o
que o dono vê.*

⚠️ **Revertido.** Fica a **primeira** cura (a ancoragem da grelha, §54), que é ganho sem
troca, e a sonda do campo.

## §59 — ⏳ O QUE FICA, com a régua certa desta vez

⛔ **A régua desta obra não pode ser a contagem de vértices.** `2 645` contra `2 681` (`1,4 %`)
não é o que o dono vê; `0` de `4` contra `2` de `4` pontas cortadas é. A §52 pedia um gate de
sensibilidade — ele tem de ser sobre a **qualidade**, e sobre o **botão**.

⇒ a obra seguinte tem duas frentes e uma delas já tem mecanismo:

1. ⭐⭐ **Estabilizar o LAÇO, não o campo.** O campo já é invariante; o que vira é uma decisão
   de corte no limiar. As saídas clássicas — histerese na banda de split/collapse, ou uma
   ordem de visita derivada do conteúdo em vez do índice — **não foram tentadas**, e são o
   sítio certo agora que o campo está ilibado.
2. ⏳ **O gate de sensibilidade sobre o botão**, com a barra em *pontas cortadas*.

⛔ **E uma cerca nova, medida:** qualquer cura que suavize o campo tem de mostrar a **régua
por ponta**, não a dispersão — foi a dispersão que fez a cura de §58 parecer progresso
enquanto ela destruía o bico.

## §60 — ⛔ TIRAR OUTRA CARTA: construído, medido, RECUSADO — e o que ele expôs vale mais

A §59 mandava estabilizar o laço. Antes disso foi tentada a saída barata, que usa a máquina
que já existe: **armar mais uma tentativa quando a escolhida tem uma ponta comida**
(`needs_another_try = still_broken || dev.over > 0`, custo **zero** quando a peça sai limpa).

⛔ **Recusado, porque não há ganho líquido.** Nas seis posições (`Detail 0,75 · Curvature 1`):

| | como está | com a carta extra |
|---|---|---|
| `x = 0` · `0,5` · `2` · `16` | — | **iguais** |
| ⭐ `x = 4` | `1/4`, pior `−10,4 %` | `1/4`, pior **`−2,6 %`**, `0` acima da barra |
| ⛔ `x = 1` | `2/4`, pior `−8,4 %` | `1/4`, pior ⛔ **`−46,6 %`** |

*Uma lotaria com mais bilhetes continua a ser uma lotaria*: ganha numa posição e perde noutra,
e a perda é catastrófica. ⇒ revertido.

⚠️ **E a razão de a carta má ganhar é estrutural:** a chave da ponta vive **depois** dos
furos, e a candidata que come o espinho tinha menos bordo. Pô-la à frente contradiz a ordem
que o dono estabeleceu por três reports seguidos — e não existe medição que a reordene.

## §61 — ⭐⭐⭐ E ELE EXPÔS UM PONTO CEGO NA RÉGUA QUE NASCEU HOJE

⛔⛔ **Uma ponta comida POR INTEIRO não tem superfície junto do ápice ⇒ não há amostra ⇒ a
1.ª redacção do `tip_deviation` SALTAVA-A.** O relatório dizia `0 de 3 pontas acima da barra`
sobre a peça com o espinho amputado em **`−46,6 %`**.

⚠️ *É a família do balde vazio — «não medido» e «perfeito» são o mesmo byte — e desta vez foi
construída no ficheiro que nasceu hoje para curar exactamente essa cegueira.*

⇒ **Curado e gateado:** sem faces perto do ápice, a ponta conta como partida e regista o
**raio da busca** (o piso do que se sabe: *«mais longe do que eu olhei»*). ⛔ O caso vizinho —
a **ENTRADA** sem vértices junto do próprio ápice — continua a saltar de propósito: aí é a
fixtura que não tem amostra, e acusar mediria a fixtura.

⚠️ **A lição de método:** *a régua nova tem de ser exercitada pelo caso EXTREMO antes de
alimentar uma decisão.* Ela foi escrita a partir das pontas **parcialmente** cortadas — as que
a foto mostrava — e o caso total, que é o pior, nunca lhe foi apresentado até um selector o
produzir.

## §62 — ⛔ CONSTRUIR O CAMPO UMA VEZ, DA REFERÊNCIA: o amplificador certo, a cura errada

⭐ **O amplificador foi encontrado, e o mecanismo é limpo:** a `SizingGrid` era reconstruída **a
cada ronda**, a partir da malha que o laço está a modificar. *Isso é realimentação*: a ronda 1
vê um campo que difere de `10⁻⁷`, produz uma malha ligeiramente outra, e a ronda 2 constrói o
campo **sobre essa malha** — o desvio deixa de ser `10⁻⁷` e passa a ser a diferença entre duas
malhas.

Construí-lo **uma vez, da referência**, quebra a realimentação — e a esfera grossa passou a
invariante nas **cinco** posições, `x = 16` incluída (a ancoragem sozinha não conseguia).

⛔⛔ **E o produto piorou muito.** No botão, as seis posições deram `1`·`2`·`3`·`3`·`1`·`2`
pontas cortadas, com piores de **`−48,6 %`**, `−47,5 %`, `−43,4 %`, `−21,4 %` — contra
`−4,2 %`..`−10,5 %` de como está. A fase zero subiu de `~1 841` para `~2 182` vértices: o
campo tirado da referência densa está calibrado para a população errada.

⇒ revertido. ⚠️ *O mecanismo estava certo e a cura estava errada — e as duas coisas cabem na
mesma frase sem se contradizerem.*

## §63 — ⭐⭐⭐ A SAÍDA É A OPOSTA: dar FOLGA à ponta (`ADAPT_RATIO` `16 → 64`)

Depois de **três** curas de estabilidade construídas e revertidas — todas a comerem a agulha —,
a leitura é uma só: **a nitidez da ponta vive exactamente do que se estava a suavizar.**

⇒ em vez de tirar o ruído, **tirar a ponta do alcance dele**: com resolução de sobra no bico,
uma decisão de corte que vira deixa de decidir se ele vive. Medido em **8 células**:

| célula | `16` | ⭐ **`64`** |
|---|---|---|
| origem | `1/4` · `−4,2 %` | ⭐ **`0/4` · `−0,5 %`** |
| `x = ½` | `3/4` · `−10,5 %` | ⭐ **`0/4` · `−0,3 %`** |
| `x = 1` | `2/4` · `−8,4 %` | `1/4` · `−4,1 %` |
| `x = 2` (o importador põe a peça aqui) | `2/4` · `−6,3 %` | ⭐ **`0/4` · `−1,9 %`** |
| `x = 4` | `1/4` · `−10,4 %` | ⭐ **`0/4` · `−1,7 %`** |
| ⛔ `x = 16` | `2/4` · `−7,1 %` (casca ABERTA) | `1/4` · `−24,2 %` (fecha, `1` não-manifold) |
| `sculpt_antes` `d = 0,85` | `2/6` · `−18,9 %` | `2/6` · `−17,1 %` |
| `_base_sculpt` `d = 0,85` | `1/4` · `−4,1 %` | `1/4` · `−4,6 %` |

⭐ **Melhor ou igual em `7` de `8`, quatro células a ZERO pontas cortadas**, e o relógio não
sobe — a renormalização mantém o orçamento, logo a folga **move** os quads em vez de os criar.

⚠️ **Não é invariância**: é a ponta a deixar de depender do sorteio. A saída ainda difere entre
posições; o que deixou de diferir, em 5 das 6, é **se o bico sobrevive** — que é a pergunta do
dono.

## §64 — ⏳ O QUE FICA

1. ⛔ **`x = 16` continua mau nas duas configurações** (`16` abre a casca, `64` deixa um
   não-manifold). É a posição extrema e nenhuma peça de artista vive lá — mas é a prova de que
   a folga **mascara** a instabilidade em vez de a curar.
2. ⏳ **O gate de sensibilidade sobre o BOTÃO**, com a barra em *pontas cortadas* — ainda por
   escrever, e agora ele tem um número honesto para exigir (`0` cortadas em `x ≤ 4`).
3. ⏳ **O laço** continua a amplificar. As duas técnicas nomeadas na §59 — histerese na banda de
   split/collapse e ordem de visita por conteúdo — **continuam por tentar**, e agora com a
   vantagem de que a ponta já não depende delas para sobreviver.

---

# ⛔⛔⛔ PARTE VIII — «piorou» (Enio, 31/08) — e a FIXTURA estava errada desde a Parte V

## §65 — O veredito do dono revoga a tabela de 8 células

*«Piorou. Antes amputava uma ponta. Agora amputou 2. Piorou até mesmo com Detail 1.»*
⇒ o `ADAPT_RATIO = 64` foi **revertido no mesmo minuto**, antes de qualquer diagnóstico.
*Nenhuma tabela minha ganha ao smoke dele.*

## §66 — ⛔⛔⛔ EU DERIVEI A TRANSFORMAÇÃO DO FICHEIRO EXPORTADO EM VEZ DE LER O IMPORTADOR

A saída que ele exportou vinha a `0,582×` e ancorada em `x ≈ 2`. **Concluí daí que a peça
vivia ali quando o botão corre** — e as Partes V a VII inteiras foram construídas sobre isso.

⛔ **É falso, e bastava ler [`sculpt3d_import::place`]:**

1. `p.mesh.recenter()` — **a MALHA é recentrada na própria origem**, e isso mexe nos vértices;
2. a escala (`IMPORT_SPAN / span`) e a posição vão para uma **`Pose`**, que só é usada para
   **desenhar e exportar**.

⇒ **o botão vê sempre a peça CENTRADA e na escala ORIGINAL.** O `0,582×` e o `x ≈ 2` do
ficheiro exportado são a pose assada na exportação, e **nunca** o que a cadeia consome.

⚠️ ⇒ **as 8 células da §63 mediram peças que o botão nunca vê.** A fixtura certa é
`_base_sculpt.obj` **recentrado pela caixa** (deslocamento `0,4137 · 0,0950 · 0,5802`), escala
`1`.

## §67 — ⭐ Com a fixtura certa, o report dele reproduz À LETRA

| `base_recentrada` · `Curvature 1` | `ADAPT_RATIO 16` | `64` |
|---|---|---|
| `Detail 1,00` | ⭐ **`0` de `4`** · `−0,5 %` | ⛔ `1` de `4` · `−2,5 %` |
| `Detail 0,75` | `1` de `4` · `−4,1 %` | ⛔ **`2` de `4`** · `−10,1 %` |

*«Antes uma, agora duas, e pior até com Detail 1»* — as duas linhas, na mesma direcção que ele
descreveu.

## §68 — ⭐⭐⭐ E O FANTASMA DA POSIÇÃO DISSOLVE-SE (mas as duas curas ficam)

⚠️ **A sensibilidade à posição É REAL como propriedade do código** — as seis posições dão
saídas diferentes, e isso está medido — ⛔ **mas o artista NUNCA a atinge**, porque a malha
que a cadeia recebe está sempre recentrada. *A ordem «funcionar em qualquer lugar» já era
satisfeita pelo importador, e o defeito que ele vê é o original: a ponta.*

⭐ **O que ficou das Partes V–VII continua a valer, e por outros motivos:**

- a `SizingGrid` indexada por coordenada de **MUNDO** é errada em si (o gate fica, e o
  controlo — o caminho sem graduação — provou que o laço já era invariante);
- o `reach` com centroide por **vértice** media a amostragem (defeito no caminho do produto);
- a régua da ponta comida **por inteiro** tinha o ponto cego curado;
- e as **quatro** curas de estabilidade recusadas continuam recusadas, com o mecanismo que as
  explica: *a nitidez da ponta vive do que se estava a suavizar*.

## §69 — ⏳ A ORDEM DE TRABALHO, corrigida

1. ⛔ **Toda medição futura corre sobre `base_recentrada`** — a peça como o botão a vê. Uma
   sonda que alimente o ficheiro cru mede outro programa (é a 4.ª vez que isto morde).
2. ⭐ **A pergunta volta a ser a original e é mais simples do que parecia:** a `Detail 1,00` a
   peça dele sai **limpa** (`0` de `4`) e a `0,75` perde **uma** ponta a `−4,1 %`. ⇒ o alvo é
   essa **uma** ponta, na densidade média — e não uma lotaria de posições.
3. ⏳ O gate de sensibilidade e o laço continuam na fila, agora **sem** urgência de produto.

---

# ⭐⭐ PARTE IX — «quase perfeito! 1 ponta ruim com detail 1» (Enio, 31/08, foto da TAMPA)

## §70 — ⛔ AS DUAS RÉGUAS QUE EXISTIAM DÃO A PONTA POR BOA

`base_recentrada` (a peça como o botão a vê), `Detail 1,00 · Curvature 1`:

| régua | leitura |
|---|---|
| suporte por ponta | `0` de `4` cortadas, a pior `−0,5 %` |
| desvio local | `p50 0,47` · `p90 0,59` — **barra `1,0`** |

⇒ *o comprimento está certo e a superfície está pousada na escultura.* O que a foto mostra é
outra coisa: **como o bico TERMINA** — uma tampa chata onde a forma pedia um ponto.

## §71 — QUATRO RÉGUAS CONSTRUÍDAS, e o que cada uma NÃO viu

| régua | o que mede | porque não vê a tampa |
|---|---|---|
| suporte (`tip_survival`) | até onde a peça vai | o comprimento está certo |
| desvio (`tip_deviation`) | distância da escultura à saída | a tampa está **pousada** na escultura |
| tampa (aresta do vértice do topo) | `0,22`–`1,20` quads nas quatro | ⛔ **não separa**: todas terminam com ~1 quad |
| coroas 1..5 (raio por distância de grafo) | `1,20 → 2,00 → 2,81` … | ⛔ **não separa**: as quatro crescem como um cone |

⚠️ *Quatro instrumentos e nenhum acusa o que o dono aponta com o dedo.* ⇒ a hipótese «a saída
está no sítio errado» tinha de ser medida directamente.

## §72 — ⭐⭐⭐ E A MEDIÇÃO DIRECTA FECHA A QUESTÃO: a tampa está no sítio CERTO

| ponta | distância da tampa ao bico verdadeiro | preço de a puxar até lá |
|---|---|---|
| 0 | `0,58` quads | aspecto `1,47 → 2,47` · enviesamento `16° → 47°` |
| 1 | `0,91` quads | ⛔ `1,18 → 4,52` · `9° → 43°` |
| 2 | `0,56` quads | ⛔ `1,55 → 5,88` · `23° → 84°` |
| 3 | `0,79` quads | `1,38 → 2,84` · `16° → 38°` |

⭐ **A tampa está a menos de UM QUAD do bico em todas as pontas** — a malha está tão perto
quanto a grade permite. ⛔ E puxá-la destrói a forma das quatro faces que a tocam: *um
movimento de meio quad paga aspecto `4×` e enviesamento `40°`.*

⇒ ⭐⭐⭐ **A tampa chata não é um erro de colocação: é o que uma grade de passo `h` faz com um
cone quando `h` é o que é.** Para o bico convergir, a célula tem de ser mais pequena **ali** —
e isso é o `ADAPT_RATIO`/factor de escala, não uma pós-passagem.

## §73 — ⚠️ E ISSO FECHA O CICLO COM A §63, com a troca explícita

Subir a folga (`16 → 64`) **afina a tampa** — e foi **rejeitado pelo dono** porque piora as
**amputações** (`1 → 2` pontas comidas). ⇒ *a mesma alavanca melhora o que ele vê agora e piora
o que ele viu antes*, e a ordem de preferência dele está declarada: **uma ponta amputada é pior
que uma tampa chata.**

⇒ **a única saída que não paga esse preço é a que o `CLAUDE.md` §5 já nomeia:** o **factor de
escala conforme por construção** — mais resolução só onde a curvatura a pede, sem inflar o
campo em todo o lado e sem a instabilidade que a folga global traz. É wave com espec própria, e
é a primeira da fila.

⛔ **Não voltar a subir o `ADAPT_RATIO` sem re-medir as amputações na peça RECENTRADA** — é o
que foi feito e revertido em 31/08.

---

# Parte X — a AMPUTAÇÃO tem causa, e a cerca que a cura estava DESLIGADA (2026-09-01)

> Ordem do dono, verbatim: *«não está assim. Estava melhor. A ponta tem que ficar boa. Auditoria
> com agentes»*. As duas lentes correram; o que se segue é o que elas acharam, o que a medição
> confirmou e o que a medição **refutou**.

## §74 — A reprodução, e ela é EXACTA

A saída que ele exportou (`_remesh_sculpt.obj`, `Detail 0,75`) trazida ao referencial da entrada
dele (escala `1,7178`, dos três eixos a `±0,3 %`) e medida pela régua da ponta **sobre TODOS os
máximos locais de raio**:

| | ápices medidos | acima da barra | a pior |
|---|---|---|---|
| **a saída DELE** | `42` | **`1`** | ponta `15909` · `p50 1,43` · `p90 3,00` |
| **a nossa, `Detail 0,75`** | `42` | **`1`** | ponta `15909` · `p50 2,39` · `p90 2,82` |
| a nossa, `Detail 1,00` | `42` | `0` | ponta `15909` · `p50 0,47` |

⭐⭐⭐ **É a MESMA ponta nas duas saídas, e é a única partida das 42.** *«Amputa 1 ponta»* é
literal, e a peça não tem nenhum outro defeito de ponta em nenhuma das duas densidades.

## §75 — O que a ponta `15909` tem de diferente: ela é uma AGULHA

Medindo o raio da secção a `d` quads **de caminho sobre a superfície** a partir do ápice
(⛔ uma fatia axial atravessa a peça e apanha o outro lado — a 1.ª régua deu `25,60` numa ponta de
raio `1,32`, que é o corpo):

| ponta | raio | `1q` | `2q` | `3q` | `4q` | `8q` | |
|---|---|---|---|---|---|---|---|
| `9663` | `3,096` | `0,90` | `1,30` | `1,62` | `1,71` | `1,74` | sobrevive |
| `1463` | `1,875` | `1,23` | `1,94` | `2,58` | `3,20` | `5,44` | sobrevive |
| `12074` | `1,832` | `1,24` | `1,52` | `1,62` | `2,13` | `3,71` | sobrevive |
| ⛔ **`15909`** | `1,801` | `0,78` | `1,07` | `1,19` | `1,22` | **`1,53`** | **parte** |
| `15341` | `1,319` | `1,23` | `1,96` | `2,61` | `3,38` | `6,13` | sobrevive |

⇒ A `8` quads de caminho, a agulha ainda mede `1,53` quads de raio contra `3,7`–`6,1` das que
sobrevivem. ⚠️ **E a `Detail 1,00` o mesmo espinho tem o dobro da espessura em unidades do quad —
e sobrevive.** *A amputação é uma função da RESOLUÇÃO relativa ao espinho, não da peça.*

## §76 — ⛔⛔⛔ A causa: a cerca de viagem do acabamento estava a `f32::INFINITY`

[`ph2d_quadfill::square_relax_capped`] intitula-se, no doc dela, *«a relaxação com cerca de
viagem — **a porta do produto**»*, traz a tabela que mostra o relevo a ir de `11,9°` a `19,1°`
sem cerca ao fim de `1 280` rondas, e **não tinha um único chamador**. O produto corria
`finish_extracted_with(..., f32::INFINITY)` com o tecto em `EXTRACT_MAX_ROUNDS = 1 200`.

⚠️ **E a aceitação do acabamento não podia apanhá-lo:** `acceptable`/`better` lêem enviesamento e
aspecto, que é exactamente o que a relaxação sem cerca **melhora** enquanto desliza a grade ponta
abaixo. *Uma ronda que come o espinho e endireita os quads é aceite por unanimidade.*

Varredura na configuração em que o defeito **existe** (peça recentrada, `Detail 0,75`,
`Follow Curvature 1`):

| cerca (arestas) | ponta `p50` | pontas acima da barra | enviesamento `p50` | `>60°` |
|---|---|---|---|---|
| ⛔ `∞` (o que shipava) | **`2,39`** | `1` | `3,2°` | `2` |
| `4` | `2,39` | `1` | `3,2°` | `2` |
| `2` | `1,69` | `1` | `3,8°` | `2` |
| `1` | `1,06` | `1` | `5,2°` | `4` |
| ⭐ **`0,5`** | **`0,67`** | **`0`** | `6,3°` | `5` |
| *(acabamento desligado)* | `0,74` | `0` | `9,4°` | `48` |

⭐⭐ **A cerca é estritamente melhor que o interruptor:** cura a ponta **mais** que desligar o
acabamento e paga **um quinto** das faces `>60°`. ⇒ *o A/B «acabamento ligado/desligado» de 31/08
não ilibou o acabamento — ele comparou duas configurações ambas erradas.*

## §77 — ⛔⛔ E a porta que ARMA as tentativas não conhecia a amputação

A 4.ª chave do [`worse`] (a amputação) nasceu em 31/08 e o [`still_broken`] — a condição que arma
a 3.ª e a 4.ª tentativas — **não foi actualizada com ela**. ⇒ na peça do dono, cuja saída é
**topologicamente impecável** com **uma ponta comida**, o botão entregava a primeira candidata
**sem tentar mais nada**. *É exactamente a forma do report.* Curado: `still_broken` conta
`dev.over > 0`, e o log passa de `2` candidatas para `6`.

## §78 — E a chave da amputação deitava fora a GRAVIDADE

`worse` lia só `dev.over` (**quantas**), nunca `p50/p90/max` (**quão**) — os três eram calculados
e impressos, e nada os lia. Uma candidata que come uma ponta por inteiro (`p90 3,0`, o piso do
*«mais longe do que eu olhei»*) empatava com uma que a arranha (`p90 1,02`), e a escolha caía
para as chaves da beleza. Curado: desempate por `p90` **depois** da contagem.

## §79 — A cura, e o que ela custa

A cerca entra como **5.ª tentativa**, armada pela mesma porta — o molde que a casa já usa, com a
mesma garantia (*só vence onde é melhor*). Medido na peça do dono, um binário, determinista:

| | `Detail 0,75` antes | depois | `Detail 1,00` antes | depois |
|---|---|---|---|---|
| pontas acima da barra | `1` de `4` | ⭐ **`0`** | `0` | `0` |
| desvio `p50` da pior | `2,39` | ⭐ **`0,67`** | `0,47` | `0,47` |
| suporte da pior | `−4,1 %` | `−2,7 %` | `−0,5 %` | `−0,5 %` |
| bordo · não-manifold | `0` · `1` | `0` · `1` | `0` · `0` | `0` · `0` |
| enviesamento `p50` | `3,2°` | `6,3°` | `4,2°` | `4,2°` |

⭐ **A `Detail 1,00` a saída é byte-idêntica** — a tentativa não arma, porque não há ponta partida.
⚠️ **A aresta não-manifold JÁ ESTAVA na candidata antiga** (verificado com
`PH2D_EXTRACT_TRAVEL=1e30`, que reproduz `2,39` / `−4,1 %` exactamente): as duas empatam na chave
da frente, e foi a chave da ponta que decidiu — ⛔ *a cerca não compra a ponta com um furo.*
⚠️ O preço é o enviesamento mediano a dobrar, **dentro da barra do oráculo** (`4,8°`–`7,1°`).

## §80 — ⛔ REFUTADO: o piso de `55 %` da régua não era o culpado

`apices()` só mede máximos locais acima de `0,55 × raio máximo`, depois trunca em `12`. Na peça do
dono isso é **`4` de `42`** — e o piso é uma fracção de **um extremo global** que um único espinho
domina (`3,096` contra `1,875` do segundo). *Parecia a explicação inteira.* ⛔ **Não é:** medidas
as `42`, as `38` invisíveis estão **todas** limpas (a pior a `p90 0,36`, melhor que duas das
visíveis) nas duas saídas e nas duas densidades. ⇒ **o ponto cego existe e está VAZIO nesta peça**;
fica nomeado como dívida da régua, ⛔ e não se constrói cura para um defeito medido como ausente.

## §81 — Erro de método a registar

Numa fase da caça pus o `CARGO_TARGET_DIR` de uma worktree de bissecção a apontar para o `target/`
da worktree principal. As reconstruções de commits antigos passaram a substituir o binário que as
corridas «de agora» executavam ⇒ resultados que pareciam **não-determinismo** (`17016`/`21084`
alternando) e que cheguei a comunicar ao dono como dependência de carga da máquina. **É falso e o
erro é meu.** Três corridas limpas depois: perfeitamente determinista. ⛔ *Uma bissecção partilha o
`target/` com ninguém.*

---

# Parte XI — a GRADE TERMINA ANTES DO BICO, e nenhuma régua o via (2026-09-01, tarde)

> Report do dono, com foto e seta, e ele traz o **diagnóstico**: *«essa área deveria ser levada
> à ponta, mas veja que ela fica a meio caminho e a ponta fica cada vez menos densa em
> polígonos»*. Ficheiro: `sculpt_Depois.obj`, exportado às 16:56 da build nova.

## §82 — Ele tinha razão por um factor de `3,85×`

Medindo, na saída **dele**, a aresta média em anéis de caminho sobre a superfície a partir de
cada bico (`perfil_ponta.py`):

| distância do bico | `1q` | `3q` | `6q` | `12q` | `20q` | `32q` | `48q` |
|---|---|---|---|---|---|---|---|
| ⛔ ponta `3990` | **`3,85`** | `3,65` | `3,15` | `2,50` | `1,54` | `0,75` | `0,74` |
| ponta `128` | `1,44` | `1,46` | `1,33` | `1,00` | `0,85` | `0,99` | `0,96` |
| ✅ ponta `2295` | `0,38` | `0,51` | `0,79` | `0,93` | `0,97` | `1,00` | `1,16` |

*O quad no bico é quase quatro vezes o mediano e **encolhe** à medida que se afasta* — o
contrário do que uma ponta afiada exige, e exactamente o que a foto mostra.

## §83 — ⛔⛔⛔ E o relatório dizia o CONTRÁRIO — pela terceira vez o mesmo mecanismo

A régua que devia vê-lo é a **`ENTREGA`** (`ph2d_quadfill::tip_body_ratio`), e ela mede **cinco
coroas radiais à volta do centroide, com média de todas as pontas**: imprimia `0,553`
(*«afina na ponta»*) sobre a peça da foto. ⇒ *o `edge_max` global era cego ao quad de
`0,02 × 0,30`, o `χ` era cego à almofada, e a `ENTREGA` é cega à ponta que engrossou.* **Um
extremo ou uma média sobre a peça inteira nunca vê UMA ponta.**

## §84 — O mecanismo: a grade não converge, TERMINA

| ponta | quad no bico | vértices a `≤6q` | irregulares aí | valência do ápice |
|---|---|---|---|---|
| ⛔ `3990` | `3,85×` | **`8`** | **`37,5 %`** | `3` |
| ⛔ `128` | `1,43×` | `26` | `11,5 %` | `3` |
| ✅ `2295` | `0,41×` | `246` | `1,2 %` | `4` |
| ✅ `132` | `0,50×` | `183` | `1,6 %` | `4` |

As pontas boas têm **~30× mais vértices** na mesma vizinhança física. Nas más as linhas de
grade **acabam todas de uma vez, a meio do espinho**, deixando uma tampa grosseira. ⚠️ A peça
tem `0,23 %` de irregulares no total — **classe do oráculo**; o defeito não é *quantos*, é
*onde*: concentrados num colapso em vez de escalonados ao longo do cone.

## §85 — O que ficou construído

- ⭐ **`ph2d_quadfill::tip_density`** — por ápice, o quad junto do bico em unidades do quad
  **pedido**, por distância de **caminho** (⛔ uma vizinhança esférica sobre um espinho fino
  apanha o outro lado: uma versão anterior leu `25,60` numa ponta de raio `1,32`). Barra
  `TIP_DENSITY_MAX = 1,5`, do vazio medido entre as duas populações (`0,38`–`0,52` contra
  `1,43`–`3,85`).
- ⭐ **A coluna `GRADE NA PONTA` em cada candidata do log** — *um knob descartado e um knob
  fraco liam-se exactamente igual.*
- ⭐ **A 5.ª chave do `worse`**, a seguir à amputação: ela impede a candidata de `4,14×` de
  vencer (a `Detail 1,00` ela perde por bordo hoje, mas nada o garantia).
- ⭐ **A 4.ª condição de `still_broken`** — uma grade que termina antes do bico arma as
  tentativas de socorro, e a da cerca de viagem é a melhor nessa coluna (`1,70 → 1,11` a
  `Detail 0,90`; `1,60 → 1,12` a `0,85`).

## §86 — ⛔ REFUTADO: carregar repetidamente não piora

*«cada vez menos densa»* lê-se como uma afirmação sobre cliques repetidos, e não é: `1` clique
dá pior bico `1,15×`, `2` cliques dão **`0,89×`**. ⚠️ E a medição apanhou a doença do `take(N)`
**na minha própria régua**: com as `4` pontas mais altas em vez de `6` ela perdia a pior
(`1,15` contra `1,55`).

## §87 — ⏳ ABERTO, e é o que o dono descreve

A coarsening **branda** que fica (`1,28×` a `Detail 1,00`, `1,40×` a `0,95`) está **abaixo da
barra**, logo nenhuma tentativa arma, e é ela que a foto mostra quando a peça está boa no
resto. ⇒ **a cura não é de selecção — é de substrato**: as linhas de grade têm de perder-se
**escalonadamente** ao longo do cone em vez de terminarem juntas. É a wave do **factor de
escala conforme** que o `CLAUDE.md` §5 já nomeia, com espec própria.

## §88 — Cortes de LOC que esta wave forçou (HR-18)

O ficheiro do caminho chegou a `694` linhas. ⛔ Sem tolerância: **três cortes por
responsabilidade** — `sculpt3d_retopo_one.rs` (correr UMA candidata), `sculpt3d_retopo_decide.rs`
(escolher entre duas) e `sculpt3d_retopo_target_tests.rs` (os gates do alvo). ⭐ E a cascata de
cinco comparações de **dez argumentos posicionais** virou uma porta (`decide::melhor`) — *conferir
à mão que nenhum par estava trocado foi trabalho de auditoria neste mesmo dia.* ⚠️ **Quatro gates
textuais reprovaram no corte**, que é o serviço deles: cada um foi repontado com a razão escrita
ao lado.

---

# §89 — A ponta perde a resolução **DEPOIS** da fase zero, e três curas caíram por medição

> Segunda foto do dono no mesmo dia — a tampa chata no bico, com a haste a mostrar uma grade
> fina e regular. *«veja. não é bom»*.

## §89.1 — A localização, com a régua nova nas três fases

`PH2D_DUMP_F1` (porta nova) escreve a malha de trabalho, e o perfil por ponta responde de uma
vez a pergunta que só se respondia por hipótese:

| | pior bico (× a mediana) |
|---|---|
| a **escultura** do dono | `2,40` — ela própria é grossa nos bicos |
| ⭐ a **fase zero** (F1) | **`0,82`** — as seis pontas ficam FINAS |
| ⛔ a **saída** do botão | `1,55` |

⇒ **`100 %` da grossura da ponta nasce a jusante do F1.** A fase zero está ilibada por
resultado, e a hipótese *«o campo de tamanho não pede fino na ponta»* está **refutada**.

## §89.2 — ⛔ REFUTADO: o alisamento do pedido

A tabela que escolheu `SIZING_SMOOTH_ROUNDS = 8` foi feita com a `ENTREGA` — a régua radial que
o §83 mostrou ser cega a uma ponta. Re-medida com a régua por ponta:

| rondas | `0` | `2` | `4` | ⭐ **`8`** | `16` |
|---|---|---|---|---|---|
| pior bico | `1,46` | `1,37` | `1,31` | **`1,28`** | `1,32` |

*O número já estava no óptimo, e a faixa inteira é `1,28`–`1,46`.* ⇒ não é a alavanca.

## §89.3 — ⛔⛔ REFUTADO: o tecto da faixa (`MAX_ADAPTIVE_RATIO = 4`)

O campo que vai ao mapa é grampeado em `alvo/2 .. 2·alvo`: **a ponta não pode receber um quad
menor que metade do pedido, por construção**. Parecia a explicação inteira. Medido pela porta
nova `ScaleField::adaptive_ranged` (`PH2D_SIZING_RATIO`):

| faixa | o que o campo PEDE na ponta | o que a saída ENTREGA | `>60°` | bordo · não-manifold |
|---|---|---|---|---|
| **`4`** | `0,553` | **`1,28`** | `0` | `0` · `0` |
| `8` | `0,508` | `1,25` | `2` | `0` · `0` |
| `16` | `0,490` | `1,24` | `6` | `0` · `0` |
| `32` | `0,476` | `1,33` | `7` | `0` · `0` |

⭐⭐⭐ **O pedido fica `14 %` mais fino e a saída não se move** (`1,28 → 1,24`, dentro do
ruído), pagando `7` faces péssimas. ⇒ *o mapa não honra o pedido na ponta, e alargar o que se
pede não muda o que se recebe.* ⛔ O tecto **fica em `4`**: subi-lo não compra nada e custa.

⭐ **E o «rasga» que o doc do tecto declara NÃO foi reproduzido** — as quatro faixas dão
`χ = 2`, zero bordo e zero não-manifold. ⚠️ *A justificação é sobre a razão entre células
VIZINHAS e o número é um limite GLOBAL*; quem limita o gradiente é o `smooth_in_log`.

## §89.4 — ⏳ O que fica, e é substrato

As três alavancas a montante estão medidas e fechadas. O que resta é o mecanismo que o §84 já
nomeia: **as linhas de grade não se perdem escalonadamente ao longo do cone — terminam juntas,
e o que sobra é a tampa.** ⚠️ Uma pista confirma que não é densidade: na faixa `16`, a candidata
com linhas de feição entrega `p50 0,60` (a **mediana** das pontas fica excelente) e `pior 2,07`
— *melhorar a densidade média das pontas não move a que colapsou.*

⇒ A obra é o **factor de escala conforme** com espec própria, e o critério de aceitação dela é
esta coluna: `GRADE NA PONTA pior` abaixo de `1,0` na peça do dono.

---

# §90 — A DENSIDADE ENTRA NO CAMPO, e a tampa do bico cede (2026-09-01, ordem *«construa»*)

## §90.1 — Por que nada do que se tentou podia funcionar

Num mapa de grade inteira o factor de escala `σ` e o ângulo do campo `θ` são **conjugados**:
`∇σ = J∇θ`. ⇒ **a densidade que um mapa integrável consegue entregar é ditada pelo CAMPO**, e o
mínimo quadrado do G3 projecta fora tudo o que o campo não permite.

*Isto explica de uma vez as três refutações do §89*: pedir um passo mais fino **ao mapa** — por
alisamento, por tecto de faixa, por qualquer via — é pedir uma coisa que o mapa é obrigado a
recusar. Não era implementação: era um teorema.

## §90.2 — A correcção, e ela é exacta

Pedir a métrica `g̃ = g/h²` é pedir `g̃ = e^{2s} g` com `s = −log h`. A curvatura muda por
`K̃ = K − Δs`, e a forma de o impor ao campo é somar ao transporte a 1-forma **`α = −∗ds`**,
cujo rotacional é exactamente `−Δs`. ⭐ **No discreto é uma linha:** a aresta dual cruza a primal
em ângulo recto, logo `∫α` sobre a dual é a diferença de `s` sobre a **primal**
(`ph2d_crossfield::Dual::scale_by_density`).

⚠️ **A força não tem curso — é `1`.** Medido, `1,5` leva o mapa de `17` para `105` dobras e `2`
leva o enviesamento mediano de `4,2°` a `7,7°`. ⛔ *Escolher uma força seria inventar um número
onde a teoria já deu um*, e foi o que eu fiz nas duas primeiras varreduras.

⭐ **O sinal está medido:** o negativo rasga a malha (`100` arestas de bordo).

## §90.3 — Ela é CANDIDATA, e a medição obriga

| `Detail` | pontas cortadas (sem · com) | desvio `p50` | dobras |
|---|---|---|---|
| `1,00` | `0 de 4` · `0 de 4` | `0,47` · ⭐ **`0,22`** | `17` · ⭐ **`6`** |
| `0,95` | `0 de 4` · `0 de 4` | `0,88` · ⭐ **`0,27`** | `18` · ⭐ **`8`** |
| ⛔ `0,75` | `1 de 4` · `1 de 4` | `0,67` · ⛔ **`3,00`** | `20` · ⛔ **`166`** |

⇒ **serve onde a resolução chega para o espinho e destrói onde não chega** (a `0,75` o espinho
mede `1,2` quads de largura e a grade não contrai tão depressa). *É a forma de coisa que o
`worse` existe para decidir, e nunca a de uma constante.*

## §90.4 — ⛔⛔ E a promessa *«só vence onde é melhor»* era FALSA — a guarda que a torna verdadeira

Posta como candidata, a curada **venceu a `Detail 0,75` e o botão piorou** (`2 de 4` cortadas
contra `1 de 4`). ⚠️ **E o critério tinha toda a razão:** ela sai com `0` furos e a de omissão
traz `1` aresta não-manifold — e os furos são a chave da **frente**, por medição e depois de
três queixas do dono. *A curada comprava duas pontas com um furo que não tinha.*

⛔ **A cura não é reordenar o critério** (decisão de produto que ninguém mediu). É fazer valer a
promessa: **uma candidata com densidade que ampute MAIS pontas que a melhor sem correcção não é
oferecida.** Com a guarda, o botão fica **estritamente melhor ou idêntico**:

| `Detail` | antes | agora |
|---|---|---|
| `1,00` | `0/4` · `−0,5 %` · `0,47` · `17` dobras | ⭐ `0/4` · **`−0,1 %`** · **`0,22`** · **`6`** |
| `0,95` | `0/4` · `−1,2 %` · `0,88` · `18` | ⭐ `0/4` · **`−0,3 %`** · **`0,27`** · **`8`** |
| `0,75` | `1/4` · `−2,7 %` · `0,67` · `20` | **idêntico** — a guarda barrou |

⚠️ **A regressão nomeada:** o enviesamento `p99` sobe (`22,9° → 36,3°` a `Detail 1,00`). O `p50`,
o aspecto e as dobras **melhoram**, e as faces com canto pior que `60°` ficam em `1`.

## §90.5 — O que os cortes de LOC forçaram, e foi bom

O ficheiro voltou a estourar duas vezes. ⭐ Os cortes foram **duplicação real**: os quatro sítios
que escreviam o par série/paralelo à mão viraram `one::par`, e as quatro corridas de duas
candidatas viraram `corrida` — *a divergência entre ramos que um gate desta linha vigiava tornou-se
inexprimível*. ⚠️ E um comentário que dizia *«a cadeia corre duas vezes, ~9 s»* **mentia**: são
até oito candidatas e `47`–`71 s`.

---

# Parte XII — A RÉGUA QUE CONCORDA COM O OLHO DO DONO (2026-09-02)

> Ordem da janela: *«o remesh embota e amputa as pontas dos espinhos. O dono reprovou a jornada
> inteira de 2026-09-01 com "absolutamente nenhuma melhoria" sobre medições que diziam o contrário.
> Comece pelo §0 do handoff — a régua que concorde com a foto — e NÃO toque no algoritmo antes de a
> ter.»* O que se segue é a régua, o que ela custou a achar, e o que ela diz do produto de hoje.

## §91 — Duas coisas antes de qualquer número

1. ⛔ **`Sculpt_Blender.obj` NÃO é uma retopologia de `_base_sculpt.obj`.** A caixa dela
   (`2,970 × 2,241 × 2,664`) é a de `sculpt_antes.obj` (13 682 v), exportada cinco minutos antes
   (29/08 10:42 → 10:47). O handoff de 01/09 emparelhava-a com a escultura do dia 30 como se fosse a
   mesma peça. ⇒ a comparação justa é **a mesma entrada pelos dois motores**, e uma régua que valha
   nas duas peças tem de ser normalizada pela própria malha.
2. **As nossas saídas exportadas trazem a pose assada** (`s = 2 / 3,424240 = 0,5840711`, âncora
   `(2, 0, 0)`), e no referencial da entrada alinham a `p50 0,000 h` — o que valida a lei do
   importador que a Parte VII já pagara.

## §92 — As réguas existentes, corridas pela PRIMEIRA vez no lado aprovado

Unidade = aresta mediana da saída; ápices pela lei da casa (piso `0,55`):

| par | suporte pior | desvio `p50` pior | grade a `3 h` pior | `gap` do ápice pior |
|---|---|---|---|---|
| ✅ `antes → Sculpt_Blender` (QRemeshify) | `−0,4 %` | `0,17` | **`0,79`** | **`0,19`** |
| ⛔ `base → _remesh_sculpt` (31/08) | `−4,1 %` | `1,50` | `1,51` | `3,17` |
| ⛔ `base → sculpt_Depois` (01/09) | `−9,8 %` | `∞` | `3,87` | `10,40` |
| HEAD `base`, `Detail 1,00` | `−0,1 %` | `0,20` | `1,10` | `0,23` |
| HEAD `antes`, `Detail 1,00` | **`−24,9 %`** | `∞` | `5,41` | `∞` |

⭐ **As réguas separavam — o que nunca fora feito era corrê-las no lado aprovado.** E a HEAD, na
entrada da malha aprovada, come o espinho principal em **`30` células** (`Detail 1,00`) e em `1,9`
(`0,75`); a fase zero já o corta `−3,0 %` nessa peça.

## §93 — ⛔⛔⛔ O piso de `0,55` ESCONDIA as pontas da foto

Com o piso a `0,25` e um filtro de forma (§97), a HEAD a `Detail 1,00` na peça do dono:

| ápice | raio | grade `3 h` | anéis `0–2 / 2–4 / 4–8 / 8–16 h` | visto pelo piso `0,55`? |
|---|---|---|---|---|
| `9663` | `1,00` | `0,60` | `0,60 / 0,56 / 0,47 / 0,54` | sim |
| `1463` | `0,61` | `1,10` | `1,20 / 1,02 / 1,03 / 1,00` | sim |
| `12074` · `15909` | `0,59` · `0,58` | `0,50` · `0,50` | afina para o bico | sim |
| `10230` | `0,51` | `0,51` | `0,45 / 0,51 / 0,65 / 0,78` | **não** |
| ⛔ **`3138`** | **`0,47`** | **`1,36`** | **`1,58 / 1,36 / 1,22 / 1,01`** — engrossa para o bico | **não** |
| ⛔ `1943` | `0,43` | `1,29` | `1,10 / 1,29 / 1,25 / 1,16` | **não** |

⇒ *a régua «GRADE NA PONTA» do produto media 4 pontas e a da foto não era nenhuma delas.* Na
malha aprovada os cinco espinhos lêem `0,52`–`0,81` no anel `0–2 h` — a grade do QRemeshify
**afina** para o bico; a nossa, nas más, **engrossa**.

## §94 — ⛔ A barra `1,5` foi calibrada sem o lado aprovado

| população (grade a `3 h`) | valores |
|---|---|
| aprovada, 5 espinhos | `0,55`–`0,79` |
| nossas pontas que não o incomodaram | `0,41`–`0,88` |
| nossas pontas reprovadas (3 saídas, 2 peças) | `1,10` · `1,29` · `1,36` · `1,40` · `1,47` · `1,51` · `3,87` · `4,50` · `5,41` |

A barra de 01/09 (`1,5`) saía do vazio entre as **nossas** pontas boas e más, e deixava passar a
`1,10`–`1,40` exactamente o que ele via. ⇒ **`TIP_DENSITY_MAX = 1,0`** — *a ponta não recebe um
quad mais grosso que o mediano da própria malha* — no vazio `0,88 … 1,10`.

## §95 — O ápice medido SOZINHO

A `p50` da vizinhança de `3 h` afoga o bico: a agulha `15909` da saída reprovada lê `p50 0,84`
(verde a `1,0`) com o ápice a **`1,11`** da superfície. ⇒ `TipDeviation::apex_max` / `cut`, barra
**`TIP_GAP_MAX = 0,5`** (meia célula): aprovada `≤ 0,19`, nossas boas `≤ 0,31`, reprovadas
`1,02 · 1,11 · 1,93 · 3,17 · 4,08 · 10,4`. A chave entra no `worse` **antes** da `over`, e arma
o socorro.

## §96 — O que o ARAME mostra e o número não (instrumento novo)

`render_ponta.py` (scratch desta janela) desenha cada ponta de lado e de frente, entrada em cinza e
saída em preto. O que se vê: no QRemeshify a grade **segue o espinho** (meridianos + anéis, a
convergir no bico); nas nossas ela corre **em diagonal** através do espinho e fecha com uma tampa
grosseira (`3138`), ou é um losango fino que chega ao bico (`9663`). Medido o ângulo mediano das
arestas ao meridiano local (dobrado a `0–45°`): aprovada `8°`–`17°`, nossas `4°`–`34°` — **sobrepõe-se,
não é discriminador**; fica como observação e como pista para o algoritmo (§100).

## §97 — A FORMA que separa um espinho de uma bossa, e o que custou

⛔ **Baixar o piso sem filtro é acusar a malha aprovada:** as bossas do corpo lêem grade
`1,0`–`1,47` **no `Sculpt_Blender.obj`** (`8` de `21` acima de `1,0`). O censo é *load-bearing*.

| tentativa | o que a medição deu |
|---|---|
| ⛔ cone sem `h` (anel a `2`–`6 %` do raio da peça) | não separa: o espinho `3810` da aprovada lê `1,76` e uma bossa `1,47` |
| ⛔ razão de ÁREA (superfície a `≤ 6 h` de caminho ÷ `π(6h)²`) | pior que o cone: espinhos `≤ 0,44`, bossas desde `0,30` |
| ⛔ cone a `2,5`–`4,5 h` (a 1.ª redacção Rust) | o botão `7328` da aprovada lê **`0,95`** e entra como espinho — com grade `1,35` **e aprovação do dono** |
| ⭐ **a PIOR faixa de `2 h` entre `3` e `9 h`** | espinhos cónicos até fundo; botões saltam para o corpo a `4`–`5 h` |

O perfil que decidiu (aprovada, `unit 0,046`, raio da secção por unidade de profundidade):
`4454` `3,13 · 3,31 · 3,80 · 3,91 · 4,30` (`r/t` de `0,89` a `0,57`) contra `7328` `3,51 · 3,98 ·`
**`7,28`** `· 7,20 · 7,51` — um botão de cinco células. Tabela completa em 4 densidades no doc de
`ph2d_quadfill::apices`; **`CONE_MAX = 1,0`**. ⚠️ Limite medido: a `unit ≳ 0,15 R` o corpo esférico
lê `cot(θ/2) < 1` e passa a espinho (a `Detail ≈ 0,5` na `sculpt_antes` uma bossa lê `0,90`).

⛔ **E o Dijkstra por PILHA da régua da grade explodia com a bola maior**: o portão das fixturas
levava `71 s` na `_remesh_sculpt` (`unit 0,056`); com heap, `0,5 s`. A bola de `16 h` só foi
possível por isso.

⚠️ **Unidade:** no produto é o **alvo** (censo idêntico entre candidatas — com a mediana de cada
uma, `19 154` e `21 650` quads dariam listas de pontas diferentes e o `worse` compararia *3 de 8*
com *2 de 7*); na bancada e nas sondas é a **mediana** da saída (uma malha de outra ferramenta não
tem alvo). Diferem `~8 %` e o registo diz qual.

## §98 — O produto com a régua honesta (mesmo binário, mesmas peças, `Follow Curvature` no máximo)

| | escolhido | antes dizia | agora diz | relógio |
|---|---|---|---|---|
| `base`, `Detail 1,00` | `21 747` quads (**o mesmo**) | `0/4` · desvio `0,22` · grade `1,28 < 1,5` ⇒ verde | `0/5` amputadas · **grade `1,36` no `3138` ⇒ RED** | `103 s → 251 s` (o socorro arma) |
| `base`, `Detail 0,75` | `5 287` (**o mesmo**) | `1/4` | `1/6` amputadas (`15909`, gap `1,02`) | `130 → 142 s` |
| `antes`, `Detail 1,00` | `19 154` (**o mesmo**) | `1/6` | `1/4` amputadas — o espinho principal, gap `3,00` (piso «mais longe do que olhei») | `211 → 262 s` |

⭐ **A saída não muda e o veredito muda — que é exactamente o que o dono disse.** Das nove
candidatas de `base`/`1,00`, a única com grade limpa (`0,82`) amputa uma ponta (`gap 0,76`) e perde,
correctamente, na chave da frente. Em `antes`/`1,00` **todas as nove** comem o espinho principal
(`gap ≥ 2,85`). ⇒ *o selector já não tem onde escolher: a cura é de substrato.*

## §99 — ⛔ Recusas MEDIDAS desta janela

| o que | por quê |
|---|---|
| `TIP_DENSITY_MAX = 1,5` | calibrada sem o lado aprovado; deixa passar `1,10`–`1,40` |
| piso `0,55` do ápice com corte em `12` | esconde `3138`/`1943`/`10230` (`0,43`–`0,51` do raio) |
| cone sem `h` · razão de área · faixa `2,5`–`4,5 h` · `CONE_MAX = 1,5` | §97 — cada uma põe uma bossa ou um botão aprovado dentro, ou um espinho fora |
| unidade = mediana da candidata **no produto** | censos diferentes entre candidatas do mesmo clique |
| alinhamento ao meridiano como discriminador | `8`–`17°` contra `4`–`34°`: sobrepõe |
| Dijkstra por pilha nas bolas de caminho | `71 s` numa fixtura; heap dá `0,5 s` |

## §100 — O que fica ABERTO, com endereço

1. **`3138` a `Detail 1,00`** (cone `0,63`, alto): a grade termina a meio e fecha com quads
   `1,36`–`1,58×` — é a foto. Nenhuma candidata cura sem amputar.
2. **`sculpt_antes`, espinho `4849`**: a fase zero corta `−3,0 %` (`ALVO/F1 = 0,39×` nessa peça) e
   **todas** as candidatas o perdem por inteiro — é o defeito mais alto que o repo tem, e está na
   peça de que o dono guardou a retopologia aprovada.
3. **Pista de mecanismo, medida no arame (§96):** onde a grade converge (QRemeshify) ela é
   meridiano + anel; onde termina (nossa) ela atravessa o espinho em diagonal. ⇒ antes de tocar no
   solver, **medir o campo cruzado junto de cada ápice contra as direcções principais de
   curvatura** (num cone: meridiano e anel), nas duas peças e no campo do oráculo (`*.rosy`).
4. O relógio a `Detail 1,00` dobrou (`251 s`): o socorro arma porque a régua diz a verdade. Fica.

## §101 — ⭐⭐⭐ O MECANISMO da grade que termina, MEDIDO: onde as singularidades param

`PH2D_SING_DUMP=<dir>` (novo, em `one.rs`) grava as singularidades do CAMPO de cada candidata;
`escada_campo.py` (scratch) lê, por espinho afiado, a profundidade de cada uma ao longo do eixo
local (em quads do alvo), e ao lado a escada de vértices irregulares da SAÍDA. `_base_sculpt`,
`Detail 1,00`, `Follow Curvature` no máximo, candidata escolhida:

| espinho | campo (F2): `+¼` a … | saída: valência `3` a … | grade `3 h` |
|---|---|---|---|
| `9663` | `0,6 · 0,7 · 1,0 · 2,6` | `0,4 · 0,5 · 3,5 · 3,9` | `0,60` |
| `12074` | `0,2 · 0,6 · 0,9` | `0,6 · 0,6 · 0,6 · 10,1` | `0,50` |
| `15909` | `0,6 · 0,6 · 0,7` | `0,2 · 0,2 · 0,2 · 6,8` | `0,50` |
| `10230` | `0,4 · 2,7 · 2,9` | `0,1 · 0,1 · 1,2` | `0,51` |
| ⛔ **`3138`** | `0,2 · 0,9 · 1,9` (e `−¼` a `13,3 · 14,3`) | **`1,0 · 1,2 · 6,1`** | **`1,36`** |

E na malha que o dono aprovou (`Sculpt_Blender.obj`), **todos** os espinhos fecham com um pólo
`+1` — quatro valência-`3` a `≤ 2 h` do bico (`3810`: `1,6 · 1,8 · 1,9 · 1,9`; `8449`: `0,7 ·
0,7 · 1,0 · 1,5`) — com a compensação `−¼` (valência `5`) a `8`–`15 h`. Nas saídas reprovadas
as singularidades estão a `9`–`15 h` do bico (`9663`: `9,4 · 11,0 · 11,2`; `3138`: `9,2 · 14,1 ·
14,9`), que é literalmente *«a grade termina a meio caminho»*.

⭐⭐⭐ **A grade do bico é monótona na profundidade da TERCEIRA singularidade**: `1,2 h → 0,51`
(`10230`) · `2,4 h → 0,88` (`8285`, outra peça) · `6,1 h → 1,36` (`3138`) · nenhuma perto →
`3,87`–`4,50`. A calota entre o bico e a terceira é uma lente de duas linhas, e estica.

⚠️ **O campo já tem as três a `≤ 1,9 h` no `3138`; é a JUSANTE que a terceira desce para `6,1 h`**
— o mapa inteiro e a extracção não a realizam onde o campo a pôs. E a fase zero está a
`1,28`–`2,32 ×` o alvo **nos bicos** (mediana `3,15 ×`): as singularidades vivem em vértices da
fase zero, logo a `1`–`2` células umas das outras — o pólo de quatro do QRemeshify precisa de
`≈ 2 h` de calota resolvida.

## §102 — ⛔ EXPERIMENTO medido e NÃO adoptado: reforçar o alinhamento na CALOTA

`PH2D_TIP_ALIGN=<k>` (novo, `one.rs`; `Dual::boost_align`, aditivo): multiplica a confiança do
alinhamento ao relevo nas faces a `≤ 8 h` de caminho de cada espinho afiado. No flanco de um cone
a anisotropia já é `1` — o que falta ao termo (`0,03 × anisotropia`) é peso, e o global foi
medido a partir em todo o lado. `PH2D_CANDIDATE_DUMP=<dir>` (novo) grava cada candidata, e
`candidatas.py` (scratch) lê-as todas:

| `k` | candidata (`w 0,03 · adapt 1 · densidade 1`) | grade nos 5 espinhos | amputadas | bordo |
|---|---|---|---|---|
| — (HEAD) | `21 649` q | `0,60 · 0,50 · 0,50 · 0,51 ·` ⛔ `1,36` | `0` | `0` |
| `3` | `21 435` q | `0,48 · 1,00 · 0,50 · 0,63 ·` ⭐ `0,66` | `0` | ⛔ `34` |
| `5` | `21 231` q | ⭐ **`0,51 · 0,63 · 0,46 · 0,75 · 0,85`** (gaps `≤ 0,3`) | `0` | ⛔ `14` |
| `10` | `21 294` q | `0,55 · 0,67 · 1,16 · 1,08 · 0,72` | `0` | `5` (+`2` não-manifold) |
| `30` | `18 936` q | `5,83` | `1` | `4`, `>60 = 29` |

⭐ **O campo fecha as pontas** (a `k = 10` o `3138` recebe `+¼` a `0,2 · 0,9 · 1,7 · 1,9` — o pólo
de quatro do QRemeshify) **e as cinco réguas da grade ficam verdes a `k = 5`** — a primeira
candidata verde nas duas réguas que esta peça já teve. ⛔ **E a extracção não fecha a calota:** as
`14` arestas de bordo de `k = 5` são **UM laço, com `14` vértices em `0,6 h` de extensão, a `1,1 h`
do bico da agulha `15909`** — o pólo converge para um ponto e o vértice degenerado fica por fechar.
Os contadores de costura (`costuras soltas · locais trocados · lados a discordar`) são **`0`**: não
é o mapa a rasgar. A `k = 5` sem densidade (`19 187` q) a topologia fecha (`bordo 0`) e a agulha é
**comida** (`gap ∞`), e `10230` também (`4,4`–`14 h`). ⇒ *a cadeia realiza um pólo denso ou com um
furo no bico ou comendo o bico* — o selector escolhe, e bem, a candidata sem furos, que é a de
sempre.

⛔ **Recusas desta secção:** `k ≥ 10` (holes fora dos espinhos, `>60` a subir, `k = 30` destrói) ·
o reforço como cura (fica como instrumento) · reordenar a chave dos furos (decisão de produto que
o dono já tomou três vezes no sentido contrário).

## §103 — ⏳ A obra seguinte, com endereço e experimento desenhado

A calota de um espinho afiado precisa de **`≥ 2` células resolvidas** para receber quatro `+¼`
separadas (o que o QRemeshify tem) — e a fase zero entrega `1,3`–`2,3 ×` o alvo nos bicos. A
`remesh_isotropic_graded` não aceita um passo por vértice; a wave é **dar-lhe uma calota por
espinho afiado** (`apices` da referência, unidade = alvo; passo `= alvo` a `≤ 8 h`, com a
renormalização que a `SizingGrid` já faz), e medir com `PH2D_TIP_ALIGN=5` **e** sem ele:

1. escada do campo a `≤ 2 h` (`escada_campo.py`) — quatro `+¼`?
2. `candidatas.py` — bordo `0` **e** grade `≤ 1,0` **e** gap `≤ 0,5` nos cinco espinhos, na
   MESMA candidata;
3. o portão `pontas_do_dono` com a nossa saída sobre `sculpt_antes` como fixtura nova — o critério
   de aceitação é passar o que a `Sculpt_Blender.obj` passa.

⚠️ `PH2D_F1_TARGET=1` (a fase zero inteira ao alvo) já foi refutado (`χ = 1`, `4` bordo, `123`
dobras) — a afinação tem de ser **local**, e a renormalização por contagem é o que a torna barata.

## §104 — ⛔⛔⛔ A FIXTURA À MÃO ERA OUTRA REALIZAÇÃO — e o destino da ponta maior é sorteado nos últimos bits (2026-09-03)

O dono exportou `sculpt-pre.obj` (a escultura de 30/08, contagem idêntica) e `sculpt-Pos-Remesh.obj`
(`Detail 1`, `Follow Curvature 1`). ⭐ **A régua concorda com a foto dele:** a ponta maior (`8042`
na numeração do importador = `9663`) lê **gap `7,23 h`, grade `3,51`, `7` vértices a `6 h`, ápice
de valência `3`** — cortada sete células abaixo do bico e tapada com faces `3,5×`; as outras quatro
lêem `0,41`–`0,86` e gap `≤ 0,27`. É o segundo report com foto em que a régua fica **RED onde ele
aponta**.

⛔⛔ **E a sonda dava outra malha para os MESMOS knobs:** `21 747` quads com a ponta maior fina. A
diferença era a **fixtura**: `base_recentrada.obj` foi recentrada em Python (`f64`, seis decimais) e
o importador recentra por [`ph2d_mesh::Mesh::recenter`] em `f32`. Com `PH2D_RECENTER=1` (novo na
sonda: recentra pela porta do importador) a sonda devolve **`20 658` quads, a mesma malha dele, ao
bit**. ⇒ *a jornada de 01/09 mediu uma realização diferente da que o dono via — é (também) por isso
que as réguas diziam «melhorou» sobre a foto que dizia o contrário.* ⛔ **Toda medição futura corre
com `PH2D_RECENTER=1` sobre o ficheiro CRU**, nunca sobre uma fixtura recentrada à mão.

⭐⭐ **E a mesma escultura, nos mesmos knobs, dá cinco vereditos em cinco realizações** (a cadeia é
covariante à escala; só o ruído de `f32` muda):

| realização de `_base_sculpt` (`Detail 1` · `Curv 1`) | quads | amputadas | grade pior |
|---|---|---|---|
| a do dono (`recenter` do importador, `s = 1`) | `20 658` | **`1/5`** (a maior, gap `7,2`) | **`3,51`** |
| o export dele re-importado (`s = 0,584`) | `21 882` | `0/5` | ⭐ `0,76` |
| fixtura Python (`f64`, 6 decimais) | `21 747` | `0/5` | `1,36` (`3138`) |
| `s = 0,7` | `21 755` | `0/5` | `1,66` |
| `s = 1,3` | `21 816` | `1/5` (gap `0,56`) | `1,64` |

⛔ **O «segundo sorteio» como rede foi medido e RECUSADO:** na `_base_sculpt` ele troca uma
amputação por três grades grossas, e na `sculpt_antes` o espinho principal é comido em **todas** as
`18` candidatas de dois sorteios (`s = 1` e `0,7`). É lotaria, custa `2×` o relógio (`12 min` a
`Detail 1` nesta máquina sob carga), e a §51 já o tinha recusado noutra forma. ⇒ *a cura é de
substrato* (§103): a calota de cada espinho afiado precisa de resolução na fase zero para receber
o pólo de quatro que o QRemeshify tem — e aí o destino do bico deixa de depender de bits.

⚠️ **Higiene de instrumento paga duas vezes hoje:** `ls -t` no `target/release/deps` apanha o
executável do PROGRAMA (que o dono acabara de construir) e não o da sonda — dez corridas «vazias»;
e a pasta temporária da sessão foi limpa a meio (reinício), levando os quatro scripts e as
fixturas — recriados a partir deste plano. *Um instrumento fora da árvore é um instrumento que
desaparece; o que fica é a lei escrita aqui.*

## §105 — ⭐⭐⭐ A CALOTA na fase zero, e a GRAVATA que deitava fora a candidata boa (2026-09-03)

O §103 pedia *«dar à fase zero uma calota por espinho afiado»*. Está feito
([`ph2d_remesh_iso::Cap`], `remesh_isotropic_graded_capped`), e a medição partiu em **duas**
descobertas — a segunda é que fecha o report do dono.

### §105.1 — A porta, e as duas metades da lei

A calota entra no campo por vértice **antes** da renormalização por contagem (logo a factura é
paga pelo resto da peça: *a adaptação move os quads, não os cria*) e é **reclamada depois** (o
factor sai `> 1` por construção e engrossaria a calota que se acabou de pedir). As calotas vêm de
[`ph2d_quadfill::apices`] com unidade `= alvo` — a **mesma** lei de ápice das réguas, e é a
`phase_zero` do shell que as calcula: *quem grada é quem chama*.

⚠️ **A sonda da porta deixou de ter uma CÓPIA da fase zero** — ela espelhava a escolha do
produto num bloco próprio, com um comentário a dizer que a espelhava, e a cópia envelheceria no
dia em que a fase zero mudasse. Hoje chama `target::phase_zero`.

### §105.2 — O que a calota faz à fase zero (`_base_sculpt`, `Detail 1`, `Curv 1`)

| `PH2D_TIP_CAP` | F1 faces | aresta média | **grade no bico** (passos do alvo) | acima de `1,0` |
|---|---|---|---|---|
| `0` (o que shipa) | `3 642` | `0,0959` | ⛔ **`2,22`** (p50 `1,56`) | `5` de `5` |
| `1,0 h` | `6 146` | `0,0671` | `1,15` (p50 `1,08`) | `5` de `5` |
| `0,75 h` | `8 374` | `0,0520` | ⭐ `0,84` | `0` de `5` |
| `0,5 h` | `15 972` | `0,0323` | ⭐ `0,55` | `0` de `5` |

⭐ A porta faz **exactamente** o que promete. ⚠️ E a coluna que o diz — a grade da calota em
**passos do alvo** — não existia em sonda nenhuma: a `tips` mede a malha contra a mediana **dela
própria**, logo responde *«a ponta é mais grossa que o corpo?»* e nunca *«cabem duas células no
bico?»*, que é a pergunta de que o pólo `+1` depende.

⛔ **E afinar mais não é melhor:** a `0,75` e a `0,5` a cadeia a jusante deixa de digerir a
inflação (candidatas com `7`–`48` arestas de bordo, `1` não-manifold na saída). É a mesma lei que
a `sizing.rs` já escrevia — *o que a jusante não digere é a INFLAÇÃO, não a graduação*.

### §105.3 — ⭐⭐⭐ A descoberta: UMA GRAVATA deitava fora a melhor candidata que esta peça já teve

A `1,0 h` a cadeia passou a **produzir** uma candidata verde nas duas réguas de ponta — a
primeira desta peça — e o selector escolhia outra. Com as três primeiras chaves do `worse`
impressas pela primeira vez:

| candidata | furos | ilhas | **gravatas** | amputadas | grade |
|---|---|---|---|---|---|
| a escolhida | `0` | `1` | **`0`** | ⛔ **`3` de `5`** | ⛔ `2,81` |
| ⭐ a deitada fora | `0` | `1` | **`1`** | ⭐ **`0` de `5`** | ⭐ `0,81` |

A chave das gravatas é a **3.ª** e a da amputação a **4.ª**: *uma face dobrada ganha a três
pontas cortadas*. ⚠️ **E a gravata nem estava na ponta** — a `5,7` células do bico mais próximo
(`gravatas.py`, sobre `PH2D_CANDIDATE_DUMP`), um quad dobrado solto no flanco.

⛔⛔ **O log da decisão imprimia `n−1` das `n` chaves, e a que faltava era a que decidia.** Ele
mostrava `bordo` (`boundary_edges`) enquanto o selector lê `open_edges` (bordo **+**
não-manifold), nunca imprimiu as **ilhas**, e nunca imprimiu as **gravatas**. *Um registo que não
mostra as chaves não explica a escolha* — e foram precisas **três** corridas para descobrir uma
coisa que uma coluna teria dito à primeira.

### §105.4 — A cura é produzir a candidata que tem as duas coisas

⛔ **Reordenar as chaves está fora:** a ordem foi medida em 30/08 sobre um report do dono
(*«destruiu completamente a malha»*, `125` gravatas), e o doc da cascata já escreve a lei — *«a
saída não é reordenar o critério, é produzir a candidata que tem as duas coisas»*.

[`ph2d_quadfill::untangle_bowties`] desfaz a gravata **no sítio**: Laplaciano tangencial sobre os
`4` vértices da face acusada, reprojecção na escultura, cerca de viagem própria
(`UNTANGLE_TRAVEL = 2` arestas — ⛔ *um quad só se auto-cruza quando um vértice passa PARA LÁ do
vizinho*, logo a meia aresta do socorro tornaria a cura impossível por construção), e aceitação
com **duas** metades: as gravatas desceram **e** a forma não piorou (a mesma `acceptable` do
acabamento — duas leis de aceitação seriam duas respostas à mesma pergunta). Onde não há gravata
é a **identidade ao bit**, e há gate.

### §105.5 — ⭐⭐⭐ O resultado, na realização do PRÓPRIO dono

| `_base_sculpt`, `Detail 1` · `Curv 1`, `PH2D_RECENTER=1` | quads | pontas amputadas | grade no bico |
|---|---|---|---|
| o que shipa hoje | `20 658` | ⛔ **`1` de `5`** (pior gap `3,00 h`) | ⛔ **`3,51`** |
| ⭐ calota `1,0 h` + gravata desfeita | `21 928` | ⭐ **`0` de `5`** (pior `0,47`, barra `0,5`) | ⭐ **`0,79`** (barra `1,0`) |

⚠️ **O desembaraçador sozinho não muda o caminho de omissão** (medido: `20 658` quads e o mesmo
veredito ao bit) — a candidata vencedora de hoje não tem gravata nenhuma. *As duas metades são
necessárias: a calota PRODUZ a candidata, o desembaraçador deixa-a GANHAR.*
