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
