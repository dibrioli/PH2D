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
