# 12 — A cache de fitas contra o CASCO (W148, 2026-09-13)

> **O item ⏳ do §83.9 do [doc 06](06_resultados_cena_e_gizmo.md)**: *«a cache contra o CASCO, e não
> contra a caixa — o `1,11×` de `74,6 → 67,2` arestas que ela deixa na mesa»*. Medido antes de
> construir, o prémio era outro, o desenho que a nota prescrevia **perdia**, e o que shipa tem duas
> metades que nenhuma nota tinha.
>
> Instrumento: [`hull_cache_probe.rs`](../../crates/ph2d-field-render/src/tests/hull_cache_probe.rs)
> (contagens — valem sob carga). Código: [`region_hulls.rs`](../../crates/ph2d-field-eval/src/region_hulls.rs)
> · [`tape_cache.rs`](../../crates/ph2d-field-render/src/tape_cache.rs) ·
> [`tiles.rs`](../../crates/ph2d-field-render/src/tiles.rs).

## §12.1 — A régua, e porque é uma contagem

A cache (W82) guarda cada fita compilada para uma região **crescida**, e serve-a a toda região que
caiba dentro. Um desenho de cache tem **duas** metades, e um número só esconde a troca entre elas:

| coluna | o que mede | porque importa |
|---|---|---|
| **acerto** | fracção das regiões servidas pela cache | cada falha é uma compilação de JIT (`~1,3 ms` de thread, e satura às 16 threads — §82.9) |
| **compila/quadro** | falhas por quadro | o mesmo, em unidades absolutas |
| **÷ sem cache** | arestas da fita SERVIDA ÷ arestas da fita que o caminho sem cache usaria na mesma região | ⭐ o custo de uma amostra da marcha segue as arestas quase à letra (§82.12: `88,3/74,6 = 1,18×` arestas ⇒ `1,18×` no custo) |

A sonda simula a política de cache sobre as regiões **do produto** (`tiles::slab_region`, a semente
de fase do `tiled_trace`, o corte `probe_cull`/`probe_cull_hull`, o casco `probe_hull_uv`), num
arrasto de 12 quadros com o quadro `0` descartado. ⚠️ Tudo contagens: nenhuma coluna desta secção é
um relógio, e por isso todas foram medidas com a máquina a `load` 4–30 sem perder valor.

## §12.2 — ⛔⛔ A caixa serve `1,9×`–`2,4×` as arestas, e não `1,18×`–`1,31×`

A caixa W82 (`f = 1,25`, a mais velha servida primeiro — o que shipava):

| tamanho | peça | `1°` | `2°` | `4°` |
|---|---|---:|---:|---:|
| `426×240` | círculo 168 | `1,889×` | `1,925×` | `2,061×` |
| `426×240` | estrela 168 | `2,093×` | `2,138×` | `2,295×` |
| `640×360` | círculo 168 | `1,997×` | `2,020×` | `2,133×` |
| `640×360` | estrela 168 | `2,200×` | `2,233×` | `2,369×` |

⚠️ **O `1,18×`–`1,31×` do §83.8 estava certo no dia em que foi medido — com o ladrilho a `64`.** A W88
passou-o a `24`, e o tubo de um ladrilho fino e oblíquo tem uma caixa quase toda vazia: a diagonal
da caixa cresce com a profundidade da fatia, a largura do tubo não. *Quem move o número que sustenta
uma nota tem de a reconferir* (`CLAUDE.md §0.0`) — e a nota do §83.9 sobreviveu à W88 sem ninguém a
remedir. ⇒ **a mesa não tinha `1,11×`: tinha até `~2×` da marcha.**

## §12.3 — ⛔ O desenho que a nota prescrevia PERDE os acertos

A nota dizia: guardar a fita para o **casco** inflado, testar em dois níveis (a caixa rejeita, o
casco confirma). Construído na sonda, com o casco escalado pelo mesmo `f` em torno do centro
(`426×240`, círculo; `acerto · compila/quadro · ÷ sem cache`):

| | `1°` | `2°` | `4°` |
|---|---|---|---|
| caixa `f = 1,25` | `91,1 %` · `48,7` · `1,889×` | `85,8 %` · `75,2` · `1,925×` | `82,3 %` · `88,4` · `2,061×` |
| casco `f = 1,25` | `70,0 %` · **`163,3`** · `1,298×` | `61,8 %` · **`202,5`** · `1,311×` | `54,8 %` · **`225,5`** · `1,370×` |
| casco `f = 1,50` | `83,0 %` · `92,3` · `1,525×` | `81,6 %` · `97,3` · `1,555×` | `79,5 %` · `102,2` · `1,609×` |
| casco `f = 2,00` | `90,6 %` · `51,0` · `1,981×` | `91,2 %` · `46,5` · `2,021×` | `90,4 %` · `48,2` · `2,100×` |

⭐ **O mecanismo:** a folga de um casco escalado por `f` é proporcional à **largura** do tubo, e o
que tira uma região de dentro da fita do quadro anterior é o **movimento** da câmera — `braço ×
ângulo` numa órbita —, que não sabe a largura de tubo nenhum. Um tubo fino sai do seu casco muito
antes de sair da caixa oblíqua que o envolve. A `f = 1,25` o casco corta as arestas a `1,3×` e
**triplica** as compilações; a `f = 2,0` volta a ser a caixa. *Uma fita que guarda menos arestas e
se compila mais vezes pode custar mais.*

## §12.4 — ⛔ A ORDEM de consulta não era o suspeito

O `TapeCache::get` devolve a **primeira** fita que contém a região, e o vector vai da mais velha para
a mais nova (o despejo ordena por idade e tira a metade da frente): uma fita gorda que continua a ser
acertada nunca morre. Medido (`426×240`, círculo, caixa `f = 1,25`, `÷ sem cache`):

| servida | `1°` | `2°` | `4°` |
|---|---:|---:|---:|
| a mais velha (o que shipava) | `1,889×` | `1,925×` | `2,061×` |
| a mais nova | `1,953×` | `1,971×` | `2,119×` |
| a de menor volume | `1,859×` | `1,881×` | `1,960×` |

Menos de `5 %` entre as três, com o acerto igual à décima. ⇒ o peso é **da caixa**, e não de **qual**
caixa.

## §12.5 — ⭐⭐⭐ O casco crescido por uma DISTÂNCIA domina a caixa

Se o que move a região é uma distância, a folga tem de ser uma distância: a caixa cresce `δ` de cada
lado (com a mesma fase W89), os cantos do tubo **não** se mexem, e o casco herda a folga porque o
`hull_uv` a lê como o que a caixa tem além dos pontos.

| tamanho | peça | variante | `1°` | `2°` | `4°` |
|---|---|---|---|---|---|
| `426×240` | círculo | caixa `f 1,25` | `91,1 %` · `48,7` · `1,889×` | `85,8 %` · `75,2` · `1,925×` | `82,3 %` · `88,4` · `2,061×` |
| | | casco `δ 0,04` | `89,3 %` · `58,2` · **`1,448×`** | `82,8 %` · `91,1` · **`1,485×`** | `79,4 %` · `103,1` · **`1,557×`** |
| | | casco `δ 0,08` | `97,3 %` · **`14,9`** · `1,818×` | `95,4 %` · **`24,5`** · `1,844×` | `94,5 %` · **`27,7`** · `1,946×` |
| `640×360` | círculo | caixa `f 1,25` | `88,6 %` · `130,8` · `1,997×` | `82,2 %` · `197,6` · `2,020×` | `78,7 %` · `225,0` · `2,133×` |
| | | casco `δ 0,04` | `91,4 %` · **`98,7`** · **`1,550×`** | `86,1 %` · **`154,4`** · **`1,560×`** | `83,9 %` · **`169,8`** · **`1,626×`** |
| | | casco `δ 0,08` | `98,3 %` · **`19,9`** · `2,016×` | `97,2 %` · **`30,9`** · `2,031×` | `96,7 %` · **`35,1`** · `2,106×` |

⭐ A `640×360` o `δ 0,04` é melhor **nas duas colunas ao mesmo tempo**; o `δ 0,08` serve as mesmas
arestas com **`3×`–`6×` menos compilações**. A estrela (168 arestas, as mesmas regiões) repete o
padrão nas 12 células. ⚠️ **E a caixa com folga absoluta não serve** (`426×240`, `δ 0,04`: `92,9 %`
de acerto, `1,975×`): a folga absoluta compra acertos às duas, e só o casco a converte em arestas a
menos.

## §12.6 — ⭐⭐ O `δ` sai do ALCANCE, e não do ladrilho

Um `δ` de mundo precisa de uma escala. Duas contas puxam por escalas diferentes — o **acerto** segue o
movimento (`braço × ângulo`, que não sabe o zoom: a órbita roda um ângulo por pixel de mão), e as
**arestas** seguem `δ` contra a largura do tubo (o ladrilho em mundo, que escala com o zoom). A
varredura (`measure_where_the_pad_should_come_from`, `426×240`) muda as duas **separadamente**: zoom
`half_extent ∈ {0,4; 0,8; 1,6}` × peça `½`/`1`/`2×`, com o `δ` em fracção do **alcance**
(`tape_cache::reach`: a distância do alvo ao canto mais afastado da caixa da peça).

| a `2°`, nas 9 células | acerto | `÷ sem cache` |
|---|---:|---:|
| caixa `f 1,25` | `75,6 %`–`91,8 %` | `1,607×`–`2,065×` |
| casco `δ = 0,04 × alcance` | `78,9 %`–`93,7 %` | `1,405×`–`1,521×` |
| casco `δ = 0,08 × alcance` | `89,9 %`–`99,2 %` | `1,648×`–`1,840×` |

⛔ **Em ladrilhos a mesma folga não é a mesma coisa:** `δ = 0,24` ladrilho dá `81,5 %` numa célula
(zoom `0,4`, peça `½`) e `91,8 %` noutra (zoom `0,8`, peça `½`). Em fracção do alcance as nove células
ficam na mesma faixa. ⇒ **o recurso que o `δ` mede é o braço da órbita**, e o alcance é lido **do
alvo** (um pan leva o alvo para fora do centro da peça e alonga o braço).

## §12.7 — ⭐⭐ O ganho SOBREVIVE a pan e zoom

A folga absoluta tem um ponto cego que a caixa escalada não tem: a caixa `f` cresce com a região, e
um zoom muda o tamanho de todas as regiões de uma vez. Com as leis de câmera do módulo (órbita
`0,01 rad/px` por `turn_local` · pan de `input_law::pan` · zoom `1,1` por passo), `426×240`, círculo
(`compila/quadro · ÷ sem cache`):

| gesto | caixa `f 1,25` | casco `0,06 × alcance` | **casco `0,08 × alcance`** |
|---|---|---|---|
| órbita 4 px | `61` · `1,93×` | `42` · `1,65×` | **`20` · `1,84×`** |
| órbita 12 px | `108` · `2,04×` | `111` · `1,69×` | **`67` · `1,88×`** |
| pan 4 px | `99` · `1,91×` | `45` · `1,67×` | **`20` · `1,83×`** |
| pan 12 px | `61` · `1,94×` | `35` · `1,69×` | **`17` · `1,87×`** |
| zoom `+0,5` | `74` · `2,09×` | `30` · `1,82×` | **`10` · `1,98×`** |
| zoom `−0,5` | `76` · `1,83×` | `64` · `1,57×` | **`39` · `1,71×`** |
| zoom `+1` | `57` · `2,32×` | `27` · `2,01×` | **`9` · `2,14×`** |
| zoom `−1` | `78` · `1,82×` | `80` · `1,50×` | **`52` · `1,64×`** |

⭐ **`0,08` é melhor que a caixa nas duas colunas em todo gesto**, e na estrela também. `0,06` corta
mais arestas (`−17 %`) e em duas células compila `+3 %`. ⇒ **nasce `0,08`**, a opção que a contagem
prova não perder em lado nenhum; ⏳ o relógio decide se `0,06` compra mais (§12.10).

## §12.8 — A obra

**`ph2d-field-eval`** — o que a compilação consome passa a ser um **valor**:

- ⭐ [`RegionHulls`](../../crates/ph2d-field-eval/src/region_hulls.rs): a caixa local e o casco em `(u, v)`
  de **cada** folha especializada, com `contains` (a caixa local contém, e todo vértice do casco
  interior está no exterior — convexo em convexo). Sem alocar: corre por candidata, ~600× por quadro.
- ⭐ `RegionCompiler::hulls` / `compile_hulled`: os cascos de uma região, e a compilação com eles já
  feitos. **`compile_at` passa a ser `compile_hulled` com os cascos calculados na hora** — a conta que
  especializa e a que decide servir são UMA.
- ⚠️ `RegionCompiler::specialised_leaf`: *quem é especializado* escrito **uma** vez, com dois leitores
  (a compilação e os cascos). Um casco a mais só torna a cache mais exigente; um a menos serve uma fita
  onde ela não vale.
- ⚠️ `affine::local_maps`: o mapa mundo→local de cada nó sobe da compilação para a folha do mapa, e
  compõe-se **uma** vez por `RegionCompiler` (era por região).

**`ph2d-field-render`** — a cache ganha política:

- ⭐ `Growth::Hull { pad_of_reach }` (o que shipa): `grow` = `pad_phased(PAD_OF_REACH × reach)`, a fita
  é compilada para `rc.hulls` da caixa crescida (os cantos do tubo **não** crescem), e o `get` serve só
  quando a caixa contém **e** os cascos guardados contêm os da consulta.
- ⚠️ **Os cascos da consulta calculam-se FORA do cadeado**, no `tiles.rs`, e só quando a política os
  pergunta: debaixo do cadeado de leitura as 32 threads não esperam por eles (a lição do §83.4).
- ⚠️ `Growth::Box { inflate }` (a W82) **fica viva**: é o caminho de bissecção
  (`PH2D_FIELD_TAPE_BOX=1`) e o lado A de todo relógio que decida este módulo — as duas políticas têm
  de poder correr no mesmo processo.
- ⭐ `phase_u`: a dispersão de fase W89 é uma função com dois leitores (`inflate_phased`, `pad_phased`),
  e o `inflate_phased` sai **byte-idêntico** (mesmas operações, mesma ordem).

## §12.9 — Gates

| gate | o que prende |
|---|---|
| `the_containment_of_hulls_is_sound_in_any_pose` | ⭐ **por AMOSTRAGEM**, numa peça com rotação oblíqua, escala e translação em cada folha e na raiz, mais um torno e uma folha debaixo de um espelho: se o `contains` diz sim, nenhum ponto do casco da consulta cai fora do da fita. ⛔ Com o controlo que o torna gate: consultas que a CAIXA serviria e o casco recusa, **com pontos que de facto fogem** (≥ 30) — sem elas, apagar a metade do casco ficaria verde |
| `a_posed_leaf_region_holds_what_the_march_evaluates` | a outra metade da composição, e a que os gates da W59 só mediam na identidade: o que a marcha avalia numa fatia (o tubo + a sonda da normal, dentro da caixa) cai dentro da região de cada folha **posta** |
| `the_padded_region_still_contains_its_query` | a caixa crescida guarda a folga **mínima** `pad·(1 − amp)` de cada lado — não só a contenção, porque é essa folga que o casco herda |
| `the_hull_cache_never_changes_the_image_of_a_posed_piece` | peça com poses, **30 quadros de órbita + pan + zoom**: 0 pixels de acerto diferentes, a normal não mexe mais do que a especialização sozinha — **e a cache serviu** (`> 1 000` fitas; medido `3 991`). *Uma cache que nunca acerta passa num gate de imagem com nota máxima* |
| `a_drag_stops_recompiling_the_tapes_it_already_has` (W82) | continua a exigir `4×` menos compilações com cache — agora sobre a política do casco |

| `a_hull_tape_is_never_served_to_a_region_its_hulls_do_not_contain` | ⭐⭐ **a propriedade no `TapeCache::get`**: uma região no canto da caixa de um tubo oblíquo (a caixa contém, o casco não) não leva a fita; uma consulta **sem** cascos também não. Com o controlo positivo (a própria região é servida). Nasceu da M2 (abaixo) |

⚠️ **As duas metades compõem-se**: *o que a marcha avalia ⊆ casco da consulta ⊆ casco da fita* ⇒ a fita
servida responde certo. Nenhuma das duas sozinha o garante.

### §12.9.1 — As provas de mutação

Instrumento versionado: [`ferramentas/w148_provas_de_mutacao.sh`](ferramentas/w148_provas_de_mutacao.sh)
(agulha com contagem exigida, `touch` depois de restaurar, e quatro veredictos que não se confundem).

| mutação | veredicto | quem a mata |
|---|---|---|
| M1 — o `contains` só pergunta pela caixa local | MORTA | `the_containment_of_hulls_is_sound_in_any_pose` |
| M2 — a cache serve uma fita de casco só pela contenção de **caixa** | ⛔ **SOBREVIVEU** aos três gates de imagem · MORTA pelo gate de propriedade | `a_hull_tape_is_never_served_to_a_region_its_hulls_do_not_contain` (escrito por causa dela) |
| M3 — a consulta chega sem os cascos dela | MORTA | `a_drag_stops_recompiling_the_tapes_it_already_has` **e** `the_hull_cache_never_changes_the_image_of_a_posed_piece` (a metade dos acertos) |
| M4 — a folga da caixa perde o lado de baixo | MORTA | `the_padded_region_still_contains_its_query` |
| M5 — o mapa mundo→local esquece a pose do pai | MORTA | `the_specialised_document_agrees_inside_its_region` (W56) — ⚠️ **e só ele** |

⛔⛔ **A M2 é o achado da secção.** Servir uma fita de casco a uma região que só a caixa contém deixou
**verdes** o `the_cache_never_changes_the_image`, o `a_cached_tape_is_never_served_to_another_document`
e o gate da peça com poses. Não é cegueira de fixtura: o corte guarda toda aresta a menos de `dmax`, e
o `dmax` é um majorante **generoso** (`82 %` de folga, §65.3) — um ponto pouco fora do casco quase
sempre ainda tem a aresta vencedora na fita. A imagem sai igual *quase sempre*. ⇒ *a propriedade
gateia-se onde é definida*, a mesma lição que a §65.3 pagou com três mutações sobreviventes.

⚠️ **E a M5 diz o limite do gate por folha, por escrito:** o oráculo dele mapeia os pontos com o
MESMO mapa que os cascos usam, logo um mapa errado passa-lhe ao lado — *um controlo que partilha o
defeito não é um controlo*. Quem prende o mapa é a paridade da especialização contra o documento
inteiro (W56), que não o partilha.

⚠️ **E a 1.ª redacção do script mentia sobre quatro delas:** lia `^error:` como «não compilou», e o
nextest imprime `error: test run failed` quando um teste REPROVA — quatro mutações MORTAS saíram
classificadas como «NÃO COMPILOU». *Uma régua de mutação que confunde vermelho de teste com vermelho
de compilação não prova nada nos dois sentidos.*

## §12.9-bis — ⛔⛔ O defeito que a auditoria achou: a folha debaixo de uma OPERAÇÃO que dobra

⚠️ **Anterior a esta wave** (a W56 escreveu a especialização por região; a W79 fez do espelho numa
operação um gesto do produto; nenhum gate os juntou) — e a cache do casco **herdava-o**, porque os
cascos saem do mesmo mapa.

**O mecanismo.** A especialização leva a caixa do mundo ao plano de cada folha pelo mapa
`affine::local_maps`, que descia compondo só **poses**; o `specialised_leaf` desistia só debaixo dos
modificadores **da própria folha**. Um espelho, uma matriz ou uma torção numa **operação** dobram os
filhos — e a região da cópia dobrada chegava à folha num sítio onde a especialização guardou as
arestas de outra região.

**A lente que o apanhou** foi a pergunta da auditoria *«quem decide que uma folha é especializada, e
o que essa regra não vê?»* — e o `grep` que respondeu: o gate da W56 põe a matriz **na folha**, que ali
é a raiz. Nenhuma fixture punha uma folha de perfil debaixo de uma operação que remapeia.

**Vermelho antes da cura** — com a fixture que contém o fenómeno, e são as duas condições que a W56
já tinha pago: uma elipse **fina e densa** (numa forma de quatro arestas o corte guarda tudo) e regiões
**centradas na cópia dobrada** (numa região ao acaso o corte guarda quase todas as arestas):

| gate | espelho na operação | matriz na operação |
|---|---:|---:|
| `the_specialisation_gives_up_under_a_remapping_ancestor` (o documento, dentro da região) | `0,496` | `0,496` |
| `the_folded_leaf_draws_like_the_row_march` (a imagem, contra a marcha por linha) | **`84` px** | **`586` px** |
| o gémeo sem o modificador (o controlo) | `0` px | `0` px |

⭐ **A cura é a regra que a folha já seguia, levada à descida:** debaixo de um nó com um modificador que
remapeia, os filhos ficam **sem mapa**, e quem pergunta por ele — a especialização e os cascos da
cache — desiste. Um só sítio, dois leitores. Os dois gates ficam verdes, e as suítes das três crates do
campo vão a **732/732**.

⚠️ **O preço, declarado:** uma folha de perfil debaixo de uma dobra deixa de ser especializada — ela é
**certa e mais lenta**, exactamente como já era debaixo de um modificador próprio. Calcular a
pré-imagem de cada dobra é a wave que a W56 já nomeava («é possível e é uma wave própria»), agora com o
alcance maior.

## §12.10 — ⏳ O relógio

`measure_what_the_hull_cache_buys_on_the_clock`: caixa `f 1,25` contra casco `0,06`/`0,08`/`0,10` do
alcance, **intercaladas ronda a ronda no mesmo processo**, cada cache a continuar o SEU arrasto, com as
leis de câmera do módulo e o quadro de movimento (sem anti-serrilhado).

### §12.10.1 — ⛔ A 1.ª corrida NÃO VALE — e as contagens dela valem

O laço que esperava por `load < 5` disparou a **`4,63`**, no mesmo segundo em que a MINHA suíte das três
crates arrancou: a média de 1 minuto ainda não a tinha visto. A medição correu toda a **`37`–`60`**. *Um
laço que espera pela máquina calma dispara no instante em que a nossa própria corrida começa* ⇒ a 2.ª
exige duas leituras calmas seguidas (1 min `< 4` **e** 5 min `< 8`) e nada meu a correr ao lado.

⚠️ **As colunas de CONTAGEM não dependem da carga**, e são do produto (o traçado real compila só as
fatias que algum raio alcança, então são menores que as da simulação do §12.7). `426×240`, círculo,
compilações por quadro:

| gesto | caixa `f 1,25` | casco `0,06` | casco `0,08` | casco `0,10` |
|---|---:|---:|---:|---:|
| órbita 4 px | `32,2` | `19,5` | `10,9` | `6,7` |
| pan 4 px | `10,9` | `3,2` | `2,1` | `1,6` |
| zoom `+0,5` | `2,7` | `1,8` | `1,2` | `1,1` |

⏳ *(a 2.ª corrida, a preencher)*

## §12.11 — ⏳ O que fica aberto

- ⏳ **`0,06` contra `0,08` do alcance** — decide o relógio (§12.10).
- ⏳ **`FRAMES_KEPT = 3` e `PHASE = 0,3` foram medidos com a CAIXA** (W89). A política do casco serve
  mais regiões com menos fitas, e os dois pedem reconferência contra ela. ⚠️ As sondas
  `how_many_frames_to_keep` e `cohort_dispersion` constroem a cache por `TapeCache::new()` — **passam a
  medir o casco**, e as tabelas nos doc-comments delas continuam a ser as da caixa.
- ⏳ **O custo do teste de casco no `get`** entra no `GET_NS`; o cálculo dos cascos da CONSULTA corre
  fora dele (no `tiles.rs`) e não tem contador próprio.
- ⏳ **A pré-imagem de cada dobra** (§12.9-bis) — uma folha de perfil debaixo de uma operação que
  remapeia é hoje certa e **não especializada**. Especializá-la outra vez é calcular a pré-imagem de cada
  um dos seis modificadores que dobram; sem preço medido.

## ⛔ Recusas MEDIDAS

| recusa | mecanismo | § |
|---|---|---|
| guardar a fita para o casco ESCALADO por `f` | a folga segue a largura do tubo, o arrasto move uma distância: `f = 1,25` triplica as compilações | [§12.3](#123--o-desenho-que-a-nota-prescrevia-perde-os-acertos) |
| servir a fita mais nova, ou a de menor volume | muda as arestas servidas em `< 5 %` | [§12.4](#124--a-ordem-de-consulta-não-era-o-suspeito) |
| a CAIXA com folga absoluta | compra acertos e não converte a folga em arestas a menos (`1,975×`) | [§12.5](#125--o-casco-crescido-por-uma-distância-domina-a-caixa) |
| normalizar o `δ` pelo LADRILHO | a mesma folga em ladrilhos dá `81,5 %` numa célula e `91,8 %` noutra | [§12.6](#126--o-δ-sai-do-alcance-e-não-do-ladrilho) |
