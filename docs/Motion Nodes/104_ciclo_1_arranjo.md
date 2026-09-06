# 104 — CICLO 1 · ARRANJO: pôr muitos objectos na tela

> O primeiro ciclo da [dinâmica](103_dinamica_dos_ciclos.md). **Tutorial:** *«Do primeiro
> objecto ao milhão»*. ⚠️ Este é o único ciclo que carrega o **substrato** (os params passam a
> viver no cartão e o painel lateral sai); os ciclos 2+ só pagam o grupo deles.

## §1 — O grupo (10 nós), e por que estes

Os nós que **põem objectos na tela e decidem onde eles ficam** — a primeira coisa que um artista
faz, e por isso o primeiro tutorial. Contagens do registry em 2026-09-05.

| nó | nome na tela | params | categoria hoje |
|---|---|---:|---|
| `motion.grid` | Grid | 4 | Source |
| `motion.scatter` | Scatter | 4 | Source |
| `motion.distribute_radial` | Radial Array | 7 | Source |
| `motion.fibonacci` | Fibonacci Spiral | 3 | Source |
| `motion.lattice` | Lattice | 4 | Source |
| `motion.voronoi` | Voronoi | 5 | Source |
| `motion.distribute_poisson` | Poisson Disk | 4 | Distribute |
| `motion.distribute_curve` | Curve Points | 12 | Source |
| `motion.path` | Path | 8 | Distribute |
| `motion.clone` | Clone | 10 | Distribute |

⚠️ **Primeiro achado, e é de facilidade de uso:** para o artista estes dez são **uma** família
(*«onde é que as coisas ficam»*), e a paleta parte-os em **duas** categorias — e mete os sete de
`Source` no mesmo saco que o `Emitter`, o `Boids`, o `Soft Body` e o `Verlet Rope`, que são
**simulações**, e que o `Shape`/`Object`/`Text`/`Table`, que são **entradas**. As duas
categorias sobrecarregadas (`Transform` 43 e `Utility` 43) já têm sub-clusters
(`palette_subgroups`); `Source` (19) não tem. ⇒ item do ciclo.

## §2 — Estado do substrato (o que já ficou pronto)

| passo | estado |
|---|---|
| medir o custo de uma row e de um cartão | ✅ [doc 103 §7](103_dinamica_dos_ciclos.md) — **11,7 µs/cartão · 2,8 µs/row** |
| `CardParam` + a shell a preencher pela porta única | ✅ `stamp_card_params` ← `shown_params` |
| a faixa na geometria (`param_band_top` · `param_row_rect` · `readout_top`) | ✅ 6 gates, 2 provas de mutação |
| o pintor da row (faixa inteira, valor à direita, nível como preenchimento) | ✅ `paint_card_params.rs` |
| o pintor é **exaustivo** nas 14 espécies de widget (nível · estado · amostra · editor) | ✅ `match` sem `_` |
| a cor é uma **amostra**, e a porta `fill_rounded_rect_srgb8` no `editor-core` | ✅ + censo |
| LOD por legibilidade (`11 px × zoom ≥ 9 px`) | ✅ |
| gates: 6 de geometria + 3 da shell · 4 provas de mutação (1 sobreviveu e apagou código) | ✅ |
| **arrastar** a row para editar (hit-test + intent) | ⏳ |
| **secções dobráveis** no cartão (`ParamGroup`) | ⏳ |
| **os editores ricos** (curva, gradiente, paleta, texto, ficheiro, cor) | ⏳ |
| **o painel lateral sai** | ⏳ |
| auditoria + upgrade dos 10 nós | ✅ §3 (scatter 22× · fibonacci e radial no device) |
| o tutorial em PDF | ✅ [`tutoriais/01_arranjo.pdf`](tutoriais/01_arranjo.pdf) — 6 páginas |

## §2-bis — O que a construção do substrato ENSINOU (e corrigiu)

1. ⛔ **A row do painel e a row do cartão não custam o mesmo número** — 13,5 µs contra **2,8 µs**
   (4,8×). Uma é um widget vivo (slider + chip + reset + registo + balão), a outra são dois
   textos e dois rectângulos. *Extrapolar o custo de um controlo de uma superfície para outra é
   medir a pergunta errada* — e foi o que a minha tabela do doc 103 §7 fez antes de a
   construção a refutar.
2. ⛔ **O param-ÂNCORA de uma cor É um dos canais dela** (`motion.tint` ancora em `r` com
   `channels: [r,g,b,a]`): suprimir «os canais» apagava a própria amostra.
3. ⛔⛔ **E depois de corrigida, a supressão inteira era CÓDIGO MORTO** — a mutação que a
   removia sobreviveu, e o censo disse porquê: das **5** cores do registry, **0** têm um canal
   não-âncora com hint próprio. Apagada; o gate ficou, como **censo com a contagem**.
   *A terceira leitura de uma mutação sobrevivente — «o mundo ainda não contém o caso» — é a que
   manda apagar a linha e guardar a contagem.*
4. ⚠️ **A altura do cartão não pode seguir o zoom**, senão os hit-rects saltam debaixo do dedo a
   meio de um pinch. O LOD decide o CONTEÚDO da row, nunca o espaço dela.

## §2-ter — ⭐ A medição que decidiu PARAR o substrato aqui

Censo das espécies de controlo (`what_species_of_control_the_catalogue_has`, **700 rows**):

| espécie | rows | % | nós |
|---|---:|---:|---:|
| Slider | 431 | 61,6 % | 104 |
| **Enum** | **138** | **19,7 %** | **85** |
| Toggle · Angle · IntSlider · Seed | 102 | 14,6 % | — |
| **os RICOS** (Text 9 · Source 7 · Color 4 · File 3 · Curve 2 · Gradient 2 · Channels 1 · Palette 1) | **29** | **4,1 %** | ~29 |

⇒ **o prémio nunca foram os editores ricos: era o ENUM** (20 %, em 85 nós). Com o arrasto e o
clique, o cartão alcança **~81 %** das rows do catálogo sem um editor novo.

⭐⭐ **E a pergunta que fechou o assunto** (`what_the_open_cycle_group_needs`): dos **10 nós
deste ciclo**, os controlos ricos são **UM** (o `Source` do `motion.path`). Construir agora um
balão de editores seria construir para um ciclo mais à frente — e o ciclo 1 ainda deve a
auditoria, o upgrade e o tutorial, que é o que a dinâmica pede.

⏳ **O que fica nomeado para quando um ciclo precisar** (Fx, animadores, fontes): o balão
ancorado na row, hospedando as rows que o painel já sabe pintar. ⚠️ **E o obstáculo está
medido:** o id de uma amostra de cor é `param_swatch_id(nome_do_param)` — keyed pelo NOME, o que
supõe **um nó de cada vez** (o painel). No cartão, dois `motion.tint` colidiriam no mesmo id;
o balão exige um id com o NÓ dentro, e a leitura de volta do picker a saber de quem é.

## §3 — A auditoria do grupo: **onde cada nó corre, e o que custa**

Sonda `measure_the_arranjo_group` (`#[ignore]`, na shell). ⚠️ **A primeira versão desta tabela
foi deitada fora:** ela cronometrava um segundo `cook` no MESMO `Cook`, e como estes nós são
`Effect::Pure` o memo respondia — **as dez linhas leram `0,00 ms`**. *A régua media o memo.*
Hoje cada corrida tem um `Cook` novo, e o `clone`/`path` recebem uma grelha (sem entrada
multiplicavam zero).

| nó | elementos | device | passes | CPU | nota |
|---|---:|---|---:|---:|---|
| `motion.grid` | 10 000 | ✅ | 1 | 0,09 ms | |
| `motion.fibonacci` | 10 000 | ✅ | 1 | 0,05 ms | ⭐ **kernel escrito neste ciclo** |
| `motion.voronoi` | 2 000 | ✅ | 1 | **19,68 ms** | ⏳ caro para a contagem |
| `motion.scatter` | 10 000 | ⛔ | — | **153,15 → 6,87 ms** | ⭐⭐ **22×, ao bit** |
| `motion.distribute_radial` | 10 000 | ✅ | 2 | 0,06 ms | ⭐ **kernel escrito neste ciclo** |
| `motion.lattice` | 10 000 | ⛔ | — | 0,02 ms | sem kernel |
| `motion.distribute_poisson` | 91 | ⛔ | — | 0,10 ms | sem kernel |
| `motion.distribute_curve` | 10 000 | ⛔ | — | 0,11 ms | sem kernel |
| `motion.clone` | 100 000 | ⛔ | — | 0,29 ms | sem kernel |
| `motion.path` | 0 | ⛔ | — | — | ⏳ precisa de um caminho VECTORIAL, não de pontos |

⚠️ **Leitura tirada a `load 6,18`** (§5.0 pede ≤ 5) — as diferenças são de ordens de grandeza,
logo direccionais; re-confirmar calmo no fecho do ciclo.

### 3.1 ⭐⭐⭐ O defeito que a auditoria achou: o `motion.scatter` era `O(n²)`

**153 ms para 10 000 pontos** — nove quadros a 60 fps, no nó que um artista põe primeiro. O
critério de Mitchell na forma ingénua: 12 dardos por ponto, e **cada dardo varria todos os
pontos já colocados** ⇒ ~600 milhões de distâncias. Curado com uma grelha de vizinhança
(~1 ponto por célula, consulta em anéis com paragem conservadora):

| pontos | antes | depois | µs/ponto |
|---:|---:|---:|---:|
| 500 | — | 0,31 ms | 0,626 |
| 5 000 | — | 3,42 ms | 0,684 |
| **10 000** | **153,15 ms** | **6,87 ms** | 0,687 |
| 50 000 | (~3,8 s) | 37,74 ms | 0,755 |

⭐ O custo por ponto passa a ser **plano** — a curva era quadrática e é linear. E a nuvem é
**bit-idêntica**, com dois gates (a consulta ponto a ponto contra a varredura, que fica viva sob
`cfg(test)` como **oráculo**; e a nuvem inteira nas três formas de região).
⚠️ **A minha primeira grelha ainda era lenta** (15 ms): ela varria o **rectângulo** de cada anel
e saltava o miolo com um `if` (`O(r²)` por anel) e não parava quando o anel já cobria a grelha
— com a grelha quase vazia isso percorria-a toda por cada um dos primeiros pontos.

### 3.1-bis ⭐ O leque radial no device — e a peça que parecia impedi-lo

*«Qual é o anel do elemento `i`?»* parecia pedir uma soma de prefixo (logo, um segundo passe ou
um readback). **Tem forma fechada:** `ring_counts` reparte `count` por `rings` como
`base = count/rings` mais um extra nos primeiros `rem = count % rings`, então os anéis cheios
ocupam o prefixo `rem·(base+1)` e o resto é uniforme — nenhum laço, nenhuma soma, nenhum
readback.

⚠️ **Duas armadilhas medidas:** a coluna `rot` só existe com `align`, e o conjunto de colunas de
um kernel é ESTÁTICO ⇒ duas listas de bindings e um `variant_by_param` (emitir `rot = 0` sempre
seria outra corrente). E `count` é um **uniform interno**: o codegen renomeia o param para
`params.count_`, e sem isso o naga recusa com *«wrong type passed to `floor`»*.

### 3.2 ⏳ O que fica, com o mecanismo nomeado

| nó | por que ainda não está no device | tamanho |
|---|---|---|
| `distribute_curve` | fórmula fechada (cúbica amostrada) — falta o kernel | pequeno |
| `lattice` | ⚠️ **recorta por REGIÃO**, logo a contagem depende de dados: é a mesma cerca do *problema do círculo de Gauss* que o `motion.grid` já documenta ⇒ kernel com `applicable = (shape == Rect)`, como ele | médio |
| `clone` | multiplica a entrada: pede `count_law` sobre a contagem de entrada + `StreamOp` | médio |
| `scatter` · `distribute_poisson` | ⛔ **sequenciais por construção** (cada ponto depende de todos os anteriores). Um kernel exige OUTRO algoritmo (Bridson paralelo, ou tiles de Poisson) — decisão de produto, porque a nuvem deixa de ser bit-idêntica | grande |
| `path` | ⏳ precisa de um caminho **vectorial** vivo; é a mesma fronteira do `field.shape` (CPU-only enquanto o canal de porta-template no device só existir emparelhado com `StreamOp::SourceRows`) | grande |
| `voronoi` | está no device e custa **19,68 ms** para 2 000 — medir onde (relaxação de Lloyd?) antes de tocar | médio |

---

## §4 — O TUTORIAL, e as duas coisas que ele ensinou a quem o escreveu

**Fonte:** `tutoriais/src/01_arranjo.html` · **PDF:** `tutoriais/01_arranjo.pdf` (6 páginas) ·
**figuras:** `tutoriais/fig/*.svg`.

⭐⭐ **As nove figuras não são desenhos: são a saída COZIDA de cada nó**
(`dump_arranjo_figures`, no shell). É a lei do doc 103 §3 — *as imagens saem do próprio app* —,
e a sonda **falha** se alguma nuvem sair vazia. A tabela dos controlos é **derivada do registry**
(`param_ui` + `param_units` + `param_hard_max`), então um param novo ganha a linha sozinho.

⛔⛔ **DOIS defeitos que só olhar o PDF impresso apanhou:**
1. A tabela vinha por `fetch()` — e num `file://` o browser **não a busca**: a secção 6 do
   primeiro PDF prometia uma tabela e entregava **branco**. *Um tutorial que promete e não
   entrega é pior que um que não promete.* Hoje o `tutorial-pdf.sh` expande
   `<!--#include …-->` antes de imprimir e **aborta** se o include vier vazio ou ausente.
2. A tabela mostrava só `min..max` do hint — a faixa do **deslizante** — e o PDF ensinava que o
   `Radius` de um leque vai «até 20 px», quando a caixa aceita `4000` e a **figura deste mesmo
   tutorial** usa `200`. É a família do *controlo que mente*. Hoje a coluna diz
   `1 a 20 (digitável até 1 000 000)` quando o tecto duro difere.

⚠️ **E o caminho de saída do gerador estava errado sem dar erro:** `cargo test` corre com a cwd
na raiz do **pacote**, então as figuras foram parar a `shells/desktop/docs/…` e a sonda disse
*ok*. *Um gerador que escreve no sítio errado é pior que um que falha.*

---

## §5 — O SMOKE DO ENIO (2026-09-05), e o que ele devolveu

Três reports, três espécies diferentes de defeito. Os dois primeiros do substrato do cartão; o
terceiro é sobre um nó de **outro** grupo, e a resposta dele é uma MEDIÇÃO, não uma cura.

### §5.1 — *«vários números só aparecem com zoom muito agressivo»*

**Causa:** a largura era medida em `FontWeight::NORMAL` e o texto pintado em `SEMI_BOLD`, que é
mais largo — o número não cabia na largura que ele próprio tinha medido e o elidor cortava-o
(`0....`). A diferença cresce quando a fonte encolhe, e é por isso que piorava ao afastar.

**Cura:** a medição mudou-se para **ao lado do pintor** (`ph2d_editor_core::text_elide::
title_elided_width`), no mesmo peso. ⚠️ *O módulo já tinha a lição para o CORTE do texto e não
para a LARGURA* — [memória](../../project-memory/feedback_measuring_a_text_at_one_weight_and_painting_it_at_another_elides_the_text.md).

### §5.2 — *«vários nós não permitem clicar no número para usar o teclado para escrever»*

⭐⭐⭐ **Um clique num número abre uma caixa de escrita sobre a row** (`param_edit.rs`, irmã do
`rename.rs`: campo efémero, buffer no `WidgetStore`, `Enter` comita, `Esc` desiste, `Blur` fecha).
Arrastar continua a varrer — é o *number field* do Blender inteiro: *arrastar dá «um pouco mais»,
clicar dá «exactamente isto»*. Um enum e um interruptor mantêm o clique que já tinham.

⚠️ **Quem responde *«isto é um número?»* é o PINTOR** (`paint_card_params::shows_a_level`), pela
MESMA função que decide o que a row desenha — duas listas de espécies é como uma variante nova
ganha um número no ecrã e não o deixa escrever.

⭐⭐⭐ **E construir a caixa expôs uma divergência MUITO maior, que o painel a sair traria calada:
o cartão lia `hint.min`/`hint.max` e mostrava o valor CRU.** Três coisas em falta, todas já
resolvidas no painel:

| o que faltava | mecanismo | quantos |
|---|---|---|
| a faixa do **canal** | uma magnitude mede graus numa Rotation e unidades de mundo num X/Y (`channel_range_override`) | os 6 nós que declaram `param_channel_range` |
| a faixa do **fio** e o `contain` | um `value.*` veste a faixa de quem alimenta; e a faixa tem de conter o valor vivo | — |
| a **FACE** (`mostrado = guardado × escala`) | um comprimento guarda-se em metros e mostra-se em px | **109 de 454** rows escalares (24 %), e **135** têm sufixo |

⛔ Sem isto a caixa nova escreveria `94` onde o painel escreve `0,94`, num quarto dos controlos —
*eu teria shipado o defeito no mesmo commit que curou o report*. Hoje `CardParam` carrega
`min`/`max`/`step`/`hard_min`/`hard_max`/`value` **já na face**, e a volta ao documento acontece
num sítio só (`CardParam::to_stored`), com o gate
`the_card_shows_and_drags_the_same_numbers_the_panel_does` a compará-los **row a row sobre todo o
catálogo** (a mutação que apaga a face acusa as 109 pelo nome).

### §5.3 — *«em Duplicator não vejo o efeito de Pick, Point Scale e Transfer»*

⭐ **Os três estão VIVOS — a sonda é a régua do PRODUTO** (`measure_what_each_param_needs`, em
`ph2d-node-motion-duplicator`): varrer o param pela faixa do hint e contar quantas saídas
**distintas ao bit** o nó emite. `1` = o artista não vê nada mexer.

| entrada | Pick | Point Scale | Transfer |
|---|---|---|---|
| 1 forma · pontos NUS | 1 | 1 | 1 |
| 3 formas · pontos NUS | **3** | 1 | 1 |
| 1 forma · pontos AUTORADOS (`size`+`tint`) | 1 | **5** | 1 |
| 3 formas · pontos AUTORADOS | **3** | **5** | 1 |
| 3 formas **COM TINT** · pontos AUTORADOS | **3** | **5** | **4** |

⇒ cada um precisa de algo na ENTRADA, e o mecanismo é o próprio desenho do nó:

- **Pick** escolhe *qual forma pousa em qual ponto* ⇒ com **uma** forma os três modos são a mesma
  coisa. Precisa de ≥ 2 formas na porta `shape` (um `motion.combine` de duas fontes).
- **Point Scale** compõe a escala do PONTO com a da forma ⇒ precisa que os pontos **tragam
  `size`** (um `motion.scale` sobre o arranjo antes do carimbo). Sem coluna não há o que compor,
  e escrever `1` em toda a linha criaria uma coluna que não existia — está no doc da função.
- **Transfer** decide *de quem é o valor quando a coluna existe dos DOIS lados* ⇒ precisa de uma
  coluna **disputada**. Uma coluna que só o ponto tem chega em **todo** modo desde 2026-09-01
  (foi a cura do report *«o nó entrou em points e a simulação morreu»*), e por isso um arranjo
  colorido contra uma forma sem cor dá o mesmo nos quatro.

⛔⛔ **Isto NÃO é «funciona como desenhado» e fica assim:** *um knob cujo sujeito não existe nesta
entrada lê-se exactamente como um knob morto* — é a mesma família do doc 90 e a mesma lição da
wave do painel do L-System (*«um param cujo sujeito outro cria mede-se morto no default»*). A
diferença é que ali a condição era outro PARAM (e o `ParamGate` resolve-a) e aqui é a **forma da
ENTRADA**, que muda a cada quadro e que nenhum mecanismo do registry sabe exprimir hoje.
⏳ **Aberto, com o desenho nomeado:** um `register_param_needs` (side-metadata: *este param só
fala quando a porta `k` traz a coluna `X`* / *quando ela traz ≥ 2 elementos*), lido pela shell —
que já calcula o `inert` do nó inteiro — para a row **dizer** que não tem sujeito. É wave própria,
no ciclo que possuir o grupo do CARIMBO. ⛔ Esconder a row está fora: o artista precisa de a ver
para saber que entrada ligar.

### §5.3-ter — A TABELA DO TUTORIAL passou a mentir, e a FACE foi a causa

⛔ **O tutorial do ciclo 1 tem uma tabela de controlos DERIVADA, e ela ficou errada no dia em que
o cartão passou a vestir a unidade do artista** (§5.2): ela saía do `param_ui` **cru**, e das
`39` células dos nove nós do tutorial **`23` têm escala ou sufixo**. A tabela imprimia
*«Gap X: 0 a 10 px»* sobre um controlo que o cartão mostra como *«0 a 1000 px»* — cem vezes.
*Uma tabela derivada não é honesta por ser derivada: é honesta se for derivada da MESMA porta.*

⇒ ela sai agora do `build_params_snapshot`, a porta de onde o painel e o cartão tiram os números.
⭐ E a troca melhorou-a de três maneiras que a porta trouxe de graça: um **enum** deixa de ser
*«0 a 2»* e passa a listar as opções (`Rect · Circle · Ring`), os **teclos digitáveis** ficam
corretos, e uma linha nova diz **o que só aparece noutro modo** (`Hole aparece quando Shape é
Ring`) — derivada dos `ParamGate` do registry, porque a porta do painel esconde o que está
gateado e numa tabela de REFERÊNCIA a omissão é pior que no painel: ali o controlo está a um
clique e vê-se aparecer; aqui ele simplesmente não existiria.

### §5.3-bis — *«Point Scale faz exatamente o que o nó Scale faz? E Transfer faz o quê?»*

Perguntas do Enio (2026-09-06), respondidas por medição — as duas sondas vivem na crate do nó
(`measure_what_point_scale_does_that_scale_cannot` · `measure_what_transfer_decides`).

**(a) Não, e a diferença é INFORMAÇÃO PERDIDA, não conveniência.** O `motion.scale` é *«fique N
vezes maior»* — um número, o mesmo para todos. O `point_scale` é *«o arranjo já traz um tamanho
por ponto: quanto dele eu obedeço?»* — a variação vem dos PONTOS. Medido com um arranjo que traz
`size = [1, 2, 3, 4]` sobre uma forma sem `size`:

| `point_scale` | o `size` de cada cópia |
|---|---|
| `0` | **a coluna NÃO EXISTE na saída** |
| `0,5` | `[1 · 1,5 · 2 · 2,5]` |
| `1` | `[1 · 2 · 3 · 4]` |

⛔ **A metade que prova que não é um `scale`:** com `point_scale = 0`, trocar os tamanhos do
arranjo de `[1,2,3,4]` para `[10,17,24,31]` **não muda a saída ao bit** — a coluna do ponto é
deitada fora, e um `motion.scale` a jusante não a pode recuperar porque ela já não existe. *Um é
um multiplicador; o outro é uma comporta.*

⚠️ **E o default é `0`, ou seja: DEITAR FORA.** Quem espalha com tamanhos variados e carimba
perde a variação **em silêncio** — é o mundo de sempre preservado de propósito (mudá-lo mexeria
em arte já autorada), mas é a coisa que um artista descobre tarde.

**(b) O `Transfer` é a regra de quem GANHA quando os dois lados trazem a MESMA coluna** — e não é
sobre cor: a cor é só o caso comum. Medido com um atributo qualquer (`heat`), forma `10` contra
arranjo `[1,2,3]`:

| modo | disputada (forma `10`, arranjo `[1,2,3]`) | só no ARRANJO |
|---|---|---|
| `Shape Wins` | `[10, 10, 10]` | `[1, 2, 3]` |
| `Point Wins` | `[1, 2, 3]` | `[1, 2, 3]` |
| `Add` | `[11, 12, 13]` | `[1, 2, 3]` |
| `Multiply` | `[10, 20, 30]` | `[1, 2, 3]` |

⚠️ **A coluna de UM lado só chega em todos os modos** — não é o `Transfer` a decidir, é a cura de
2026-09-01: *um modo que resolve conflito não decide sobre uma coluna que ninguém disputa*.
⛔ **E há três que ele NÃO toca**, porque já têm lei própria: `P` e `rot` **somam sempre** (medido:
`rot` sai `[8, 9, 10]` nos quatro modos), `Index`/`Count` são renumerados contínuos, e o `size` é
do `point_scale`. *Uma grandeza, uma porta.*

⭐⭐ **E a metade que se podia entregar HOJE foi entregue: a cena `=110`** (pedido do Enio no
mesmo dia — *«crie uma cena de smoke com exemplos de todos os usos do duplicator»*). Se um
controlo só fala com a entrada certa, então a resposta imediata é **montar a entrada** — treze
bandas numa tela, uma por caso:

| fileira | bandas | o que ela responde |
|---|---|---|
| 1 | 3 | **o que o carimbo É** — a forma inteira pousa em cada ponto, e `P`/`rot` SOMAM |
| 2 | 3 | **`Pick`** — `Off` (o produto, 15 cópias) · `Cycle` · `Random` |
| 3 | 3 | **`Point Scale`** — `0` · `0,5` · `1` |
| 4 | 4 | **`Transfer`** — `Shape Wins` · `Point Wins` · `Add` · `Multiply` |

⛔⛔ **E a 1.ª versão errou o alvo, com o report a chegar no dia seguinte** — *«o cenário que
você construiu tem tantos nós interligados que não pude entender. Crie uma cadeia de nós por
output»* (Enio, 06/09). Ela construía as formas **uma vez** e partilhava-as entre as bandas de
cada fileira, para a comparação medir o modo e não a entrada. ⭐ **A propriedade estava certa e o
preço era o GRAFO:** um nó de forma alimentava quatro carimbos em quatro alturas, e os fios
atravessavam a tela. *Um grafo que ninguém consegue seguir não ensina nada, por mais correcta que
seja a corrente que ele desenha.*

⇒ hoje cada banda é uma **cadeia FECHADA**, e a
propriedade não se perdeu — **mudou de dono**: as três bandas do `Pick` carimbam as mesmas formas
porque saem da MESMA função, não porque partilhem um nó. ⭐ *Uma igualdade por CONSTRUÇÃO é tão
forte quanto uma por referência, e não custa um fio a atravessar a tela.* Os dois gates novos:
`each_output_is_its_own_closed_chain` (conta as **componentes ligadas** — a régua é a TOPOLOGIA,
não a contagem de nós) e `the_three_pick_bands_stamp_the_same_shapes` (mede a igualdade na
SAÍDA, que é onde a afirmação vive). A mutação que repõe a partilha lê **`11` ilhas para `13`
saídas**.

⛔⛔⛔ **E o report seguinte apanhou um erro de IDIOMA que nenhum gate podia ver** — *«você
colocou grid entrando em Shape de Duplicator! Essa aplicação é correta?»* (Enio, 06/09). Não
era. As duas portas do carimbo são o mesmo tipo (`Instances`), então **só a semântica as
distingue**, e um `motion.grid` de uma célula compila, cozinha e desenha o ladrilho de omissão.
⚠️ *Numa cena de demonstração isso não é «funciona»: é ENSINAR que uma forma se faz com uma
grelha de 1×1*, e o artista leva o erro para o trabalho dele. A resposta estava escrita no nó
que eu não procurei — o doc do **`source.shape`** nomeia a composição à letra: *«cross it with a
`motion.grid` through a `motion.duplicator` and the shape is stamped, crisp, at every point»*.

⭐⭐ **O idioma certo saiu MAIS BARATO, que é o sinal de que era mesmo o certo:** o ladrilho
precisava de `grid + transform + scale + tint` para ser «uma forma»; o `source.shape` é **um** nó
(`kind` e `size` são params dele). A cena foi de **140 para 102 nós** (`7,8` por cadeia), ficou
**vectorial** — nítida em qualquer zoom — e ganhou silhuetas: o `Pick` deixou de escolher entre
três quadrados coloridos e passa a escolher entre um **círculo, uma estrela e um coração**, que é
a pergunta que ele responde. ⇒ o oráculo dos gates passou a ser o **`geometry_id`** (literalmente
*qual forma*) em vez da cor, que era um substituto.

⚠️ **Dois preços que a troca traz, os dois nomeados:** um `source.shape` lê um EXTERNAL que a
shell publica, então **num cook nu ele emite zero** — os gates cozinham através de um
`MotionState` depois de `motion_shape_gen::publish`; e a geometria vive em **raio 1**, então a
meia-extensão de uma cópia **é** o `size` e não metade dele (o gate da sobreposição depende
disso). O gate novo, `the_shape_port_is_fed_by_a_shape_source_and_the_points_port_by_an_arrangement`,
sobe pela entrada 0 de cada carimbo até à **origem** do braço — entre a fonte e o carimbo há
transformes e tints, e olhar só o vizinho imediato não responderia.

⛔⛔ **E o report era UMA de TRÊS.** O censo sobre as cenas do roteador (`19` carimbos) achou
mais duas a fazer o mesmo — **`=62`** (a junção) e **`=98`** (*«o vocabulário do carimbo»*, a
cena que é literalmente sobre este nó). Curar a que ele apontou e deixar as irmãs é curar
**metade de uma família**; as três estão convertidas, e o censo virou o gate
`every_stamps_shape_port_is_fed_by_a_thing_to_draw`, com a **catraca de duas metades**: a lista
de fontes legítimas cresce de propósito, e uma entrada que cena nenhuma usa tem de sair.

⚠️ **Duas coisas que a conversão do `=62` mudou por baixo de um gate, e o gate é que estava a
descrever o mundo antigo:** ele afirmava *«com Point Scale 0 o carimbo não emite `size`»* — o que
era verdade porque uma grelha nua não tem `size` nenhum. Uma FORMA a sério tem, então o que a
cena ensina agora é o que ela de facto mostra: **todas do mesmo tamanho, o da forma**. ⭐ E a
conversão descobriu um defeito ao lado: o `motion.drive(Size, **Set**)` sobre uma rampa `0..1`
dava `size = 0` ao primeiro ponto — **a primeira cópia saía invisível**, sete pontos e seis peças
na tela. Passou a `Add` (a escala vai de `1` a `1 + scale`), com gate a exigir que nenhuma cópia
tenha tamanho zero.
⚠️ **A semente do `Random` é uma CALIBRAÇÃO com gate:** cinco sorteios sobre três formas deixam
uma de fora com facilidade — **9 das 24** primeiras sementes fazem-no, e a banda passaria a
ler-se como *«Random escolhe entre duas»*. O `PICK_SEED = 9` é o número que a varredura deu, e o
`the_random_band_shows_every_shape` defende-o.
⚠️ **A banda 2 e a banda 4 são a MESMA lei vista de dois lados** — *«uma forma feita de três
peças»* e *«três formas alternativas»* são a mesma corrente para este nó; o que as separa é o
`Pick`, que trata cada elemento da forma como uma candidata. Está no anúncio, porque é
exactamente a coisa que um artista entende ao contrário.

**Os nove gates medem o que a cena DESENHA** (cada banda é cozida e a afirmação é sobre as
colunas que saem), e três deles nasceram de uma medição: a contagem por banda (`15` no produto,
`5` nos dois modos de variante), a **sequência de cores** do `Cycle` contra a do `Random` (a
contagem não os separa), e o `no_band_runs_into_its_neighbour` — que teve de aprender que **uma
peça girada é `√2` mais larga**, senão a banda do giro lia-se 41 % mais estreita do que desenha.
