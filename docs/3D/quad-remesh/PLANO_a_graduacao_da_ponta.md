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
