# 91 · OS TETOS QUE NINGUÉM MEDIU — o bloco Z

**Data:** 2026-08-23 · **Linha:** `line/motion-value` · **Lei-mãe:** [`CLAUDE.md` §0.0](../../CLAUDE.md)
— *antes de escrever qualquer limite, MEÇA; e depois escreva o número que a MEDIÇÃO deu, com a
tabela ao lado dele. Um limite legítimo diz de que RECURSO ele é.*

> **Este doc é o registro de uma varredura, não um plano.** O que ele guarda são as **medições** e
> as **três espécies** que elas separaram. As curas estão nos doc-comments das crates; os números
> re-derivam-se a cada corrida nos gates `param_ceilings` e `integrator_ceilings`
> (`ph2d-node-registry-init`) e em `the_gradient_stop_ceiling_is_the_narrowest_panel_divided_by_a_pointer_target`
> (`ph2d-panel-motion-params`).

---

## §0 — De onde veio a lista (ela não foi escrita a olho)

Dentro do `motion_gpu_coverage.rs` da shell vivia, `#[ignore]`, a sonda
**`what_the_corpus_authors_and_no_one_can_type`**. Ela varre **todo grafo que o repo constrói**
(`corpus`) e, para cada param com override, pergunta uma coisa só:

> *o valor que a cena autora cai dentro do que uma sessão de autoria NOVA consegue escrever?*

A régua dela é a do painel, verbatim: `param_hard_max.unwrap_or(hint.max).max(hint.max)`,
**sem** o `contain` que alarga a faixa para conter o valor do documento — porque é justamente
esse alargamento que esconde o defeito.

O doc-comment dela dizia, há meses:

> *"É a varredura por família da §9 do doc 88, não desta wave, e o teto de cada um se MEDE. Por
> isso isto é SONDA e não gate: transformá-la em vermelho hoje só ofereceria duas saídas ruins —
> fazer a wave alheia por dentro desta, ou shipar uma allowlist de catorze nomes."*

**Este bloco é aquela wave.** Ela acusava **22 valores em 13 params de 10 nós**; hoje acusa
**zero**.

⚠️ **Ela nomeia o irmão de cada acusado, e isso é deliberado.** Curar `motion.move::dx` sem o
`dy`, ou um canto de quatro sem os outros sete, é precisamente a inconsistência que fez este
defeito ficar invisível: quando metade da família tem teto e metade não, ninguém nota que a
faixa que se vê descreve a comodidade do dedo e não o nó.

---

## §1 — As três espécies (separá-las É o achado)

Um teto pode ser de três coisas, e **qual delas decide o número**:

| espécie | de que recurso | quem manda | exemplo |
|---|---|---|---|
| **PRECISÃO** | o `f32` | acima de `2^e`, somar o `step` do slider não move o número | `field.box::width` |
| **LEI** | a aritmética do nó | acima daqui o nó **desiste**, e o excedente é perdido | `sim.spawn::rate` |
| **SIGNIFICADO** | o domínio | acima daqui a escolha **repete-se** | `force.wind::angle` |

⚠️ **A espécie errada dá o número errado por ordens de grandeza.** O `sim.spawn::rate` seria
`2²⁵` pela precisão e é **15 360** pela lei; o `force.wind::angle` seria astronómico pela
precisão e é **360** pelo significado.

---

## §2 — PRECISÃO: 25 params, 9 crates

**A lei.** Dentro de um binado `[2^e, 2^{e+1})` o `ulp` é constante (`2^{e-23}`), então
`v + step != v` vale para o binado inteiro ou para nenhum dele. O teto é **o maior representável
do último binado que sobrevive** — acima dele, dois valores autoráveis vizinhos são o mesmo campo,
e um campo digitável que os aceitasse *aceitaria e mentiria* (doc 88 §B2).

**A tabela, DERIVADA** (`measure_precision_ceilings`, 2026-08-23):

| step do slider | teto medido | binado |
|---|---|---|
| `0,01` | **262 143,984** | 2¹⁸ |
| `0,05` | **1 048 575,938** | 2²⁰ |
| `0,1` | **2 097 151,875** | 2²¹ |
| `1` | **16 777 215** | 2²⁴ |

Os 25 params: `field.box::width`/`height` · `field.radial_sweep::radius`/`inner_radius` ·
`field.remap::steps` · `force.vortex::radius` · `force.attractor::radius` ·
`motion.spherize::radius` · `force.buoyancy::depth` · `motion.voronoi::width`/`height` ·
`value.lfo::period`/`amplitude`/`offset` · `motion.emitter::speed` · `motion.move::dx`/`dy` ·
os **oito** cantos do `motion.four_point_warp`.

⚠️ **Um deslocamento com sinal leva as DUAS pontas.** A sonda acusou `br_dx = −40` ao lado de
`tl_dx = 160`: um teto generoso com o piso de ontem deixa metade do gesto inalcançável, e um
gesto que só funciona para um lado lê-se como bug do nó, nunca como faixa de slider.

⚠️ **E o NEUTRO de dois nós era inalcançável.** O doc-comment do `field.box` promete *"a box
larger than the scene with `soft = 0`"* e o teste dele usa `width = 100` — sobre um campo que
digitava até **40**, porque sem `ParamHardMax` o `ui.rs:206` é explícito: *"a param with no entry
here types to its soft `max`"*. O nó documentava um estado que a UI recusava.

**Os piores casos medidos**, por razão entre o autorado e o digitável:

| nó · param | a cena autora | o campo digitava | razão |
|---|---|---|---|
| `motion.move::dx` (cena `=15`) | 260 | 10 | **26×** |
| `motion.spherize::radius` (cena `=13`) | 320 | 20 | **16×** |
| `motion.four_point_warp::tr_dy` (cena `=14`) | 180 | 10 | **18×** |
| `value.lfo::amplitude` (cena `=15`) | 180 | 10 | **18×** |

---

## §3 — LEI: `sim.spawn::rate`

`born_in` grampeia a janela em `first + MAX_PER_TICK` (256), e o app cozinha uma vez por tique de
`ph2d_core::time::DEFAULT_HZ` ⇒ o nó honra **15 360 nascimentos/s**. Acima disso os nascimentos
devidos **não são adiados: são saltados** — o único ponto lossy do modelo, dito em voz alta no
doc-comment do `MAX_PER_TICK`.

**Medido pela porta do produto** (`the_spawn_rate_ceiling_is_the_one_the_law_honours`), a `rate =
15 360` durante `0,25 s`:

| | devidos | entregues |
|---|---|---|
| **no teto** | 3 840 | **3 840** |
| **ao dobro** | 7 680 | **3 840** |

⚠️ **Os dois entregam o MESMO número**, e é isso que prova que o teto é o certo: o excedente é
perdido, não adiado.

O campo digitava até **60**. Duzentas e cinquenta e seis vezes menos.

---

## §4 — SIGNIFICADO: `force.wind::angle`

A cena `=24` autora `angle = −90` sobre um campo `[0, 360]`. **O nó sempre honrou o valor** —
`frac(p) = p − p.floor()` leva `−0,25` a `0,75`, então `−90° ≡ 270°` ao bit —; o campo é que
recusava, e escrever `−90` para *"para baixo"* é o gesto de quem vem de qualquer outra ferramenta.

O teto é **uma volta**: `450°` desenha o mesmo vento que `90°`, então aceitar mais seria oferecer
uma escolha que não existe. É a lei do `sim.spawn::probability`, que para em `1` porque acima dali
todo nascimento acontece.

---

## §5 — O GRAMPO DOS INTEGRADORES (folhas 13 e 17)

### §5.1 — O que estava escrito

| nó | `MAX_DT` | o que o doc-comment afirmava |
|---|---|---|
| `motion.integrate` | `0,100` | *"guards a pathological playhead jump from becoming one giant unstable step"* |
| `sim.step` | `0,050` | *"otherwise arrives as one enormous `dt` and the sim explodes"* |

Duas constantes, o mesmo papel, valores diferentes, **nenhuma medição**.

### §5.2 — O que a medição diz

`measure_the_step_that_a_closed_loop_survives` corre a MESMA malha fechada nos dois integradores
— `motion.grid → (integrador)`, com o `pre` de volta por uma `force.attractor` — e mede a maior
excursão de uma grelha que **nasce dentro de raio 1,0**:

| `strength` | dt=1/60 | dt=1/30 | dt=0,05 | dt=0,075 | dt=0,1 |
|---|---|---|---|---|---|
| 5 | 0,71 | 0,73 | 0,91 | 0,96 | 2,48 |
| 10 | 0,80 | 0,83 | 1,52 | 1,89 | 3,52 |
| 20 | 0,73 | 0,78 | 2,43 | 33,10 | 32,75 |
| **40** (fim do arrasto) | 0,83 | 0,89 | 4,43 | 33,48 | **127,19** |
| — o mesmo pela `sim.zone`/`sim.step` — | 0,83 | 0,89 | 2,49 | 2,48 | 2,48 |

⚠️ **As duas malhas são IDÊNTICAS até 1/30** — é o mesmo Euler semi-implícito —, e o que as separa
dali para a frente é só o grampo. **O dissidente era o número mais certo dos dois**: `0,05`
segurava a mesma cena em `2,49` onde `0,1` a deixava chegar a **127,19**, ou seja **127 vezes o
raio em que ela nasceu**, com uma força que o artista alcança *arrastando*.

⚠️ **A sonda confirma-se a si própria:** acima de `0,1` a excursão do `motion.integrate` **congela
em 127,03** e não muda mais. É o grampo a morder — o instrumento está a medir o que diz medir.

### §5.3 — O joelho, e a régua que o achou

O critério está escrito, não sentido: ***um passo legítimo não muda a RESPOSTA, só a
resolução***. A régua é a excursão em regime (`dt = 1/60`) e a barra é o **dobro** dela.

**O maior `dt` em que TODO passo até ele fica dentro da barra é `0,0300`** (1/33,3). A `0,0325`
já mede `3,57`.

⚠️⚠️ **A RESPOSTA É RESSONANTE, E A PRIMEIRA VERSÃO DA SONDA LEU-A AO CONTRÁRIO.** Procurando o
**primeiro cruzamento** ela parava em `0,0300` — e `0,0333` (um quadro perdido a 30 fps, o caso
mais comum de todos) mede `0,89` e passa folgadamente. Um laço fechado com força central tem
**ressonâncias**: a excursão não é monótona em `dt`, então *o primeiro cruzamento é uma
ressonância, não a fronteira*. A pergunta certa não é *"este passo sobrevive?"* mas ***"todos os
passos até este sobrevivem?"*** — e o **prefixo-máximo**, monótono por construção, é a única forma
de a fazer. A mesma correcção teve de ser feita na varredura de `strength` (120 mede 1,63 e 160
mede 0,89).

**Cura:** os dois passam a `0,03`, cada um spelling o seu (um grampo é do SOLVER), com a tabela ao
lado. **Em regime é byte-idêntico** — o tique fixo é `1/60 = 0,0167`, e o `FixedStep` da casa já
entrega um tique por cozedura mesmo num ecrã lento; o que muda é só quanto de um SCRUB o sim
absorve, e absorver é a resposta certa ali.

### §5.4 — ⛔ O que NÃO se mexeu, e porquê

`MAX_DT = 0,1` está copiado em **cinco** nós: `motion.integrate` · `sim.step` · `motion.boids` ·
`motion.wave` · `motion.spring`.

⛔ **O `motion.spring` fica onde está, e ele é o bom exemplo desta casa:** o `MAX_DT` dele é a
premissa de **três outros tetos medidos** — `friction = 2/MAX_DT = 20` (e o achado de que o slider
já estava *sentado* no teto de estabilidade sem ninguém saber), a saturação do sub-passo adaptativo
e o teto do `tension` em 1 600 000, cada um com a sua tabela e um oráculo de três braços
(*sadia · EXPLODE · SALTA*). Mexer no dele move aquela tabela inteira, e isso é wave própria.

⏳ **`motion.boids` e `motion.wave` continuam com `0,1` por medir.** Eles copiaram o número sem
derivação, como os dois que este bloco curou — a sonda `excursion` já está escrita e serve-lhes com
outro grafo. É a dívida que este doc regista.

---

## §6 — O teto de PAINEL: as paradas do gradiente (folha 09)

`MAX_GRADIENT_STOPS = 8`, com a justificativa *"o painel é estreito e a faixa tem de ficar
legível"* — uma frase, não um recurso, sobre um modelo que admite 32.

⭐ **A medição CONFIRMOU o 8, e é esse o resultado.** O número nunca esteve errado; o que não
existia era a razão. *Um teto certo sem derivação e um teto errado leem exactamente igual no dia
em que alguém precisa de o mover.*

| grandeza | de onde vem | px |
|---|---|---|
| painel mais estreito | `ph2d_tokens::PANEL_MIN_W_PX` | 220 |
| recuo, dos dois lados | `PANEL_HEAD_PAD_PX × 2` | 36 |
| **faixa útil** | | **184** |
| alvo de ponteiro | `GRAB_R × 2` — a caixa de agarrar que o próprio editor declara | 18 |
| folga da célula | `SWATCH_PAD × 2` | 4 |
| **por parada** | | **22** |

`184 / 22 = 8,36` ⇒ **8**.

⚠️ **O recurso não é a legibilidade, é o ALVO DE PONTEIRO**, e a distinção decide o número: uma
amostra de 14 px lê-se perfeitamente — o que ela deixa de ser é **clicável**, e cada amostra abre o
seletor de cor.

⚠️ **Contra o painel MAIS ESTREITO**, não contra o de hoje: um teto que só vale na largura
confortável parte-se quando o artista aperta a janela — a lei do pior caso que o `motion.spring` já
aplica ao relógio.

⚠️ **`PANEL_MIN_W` era um `const` privado DENTRO de uma função.** Ele é o pior caso de *quem
desenha dentro de um painel*, então virou token (`chrome.panel-min-w`), ao lado do `panel-head-pad`
que já morava lá. A primeira tentativa foi expô-lo no `panel_chrome.rs`, e o cap de 500 LOC dos
widgets mordeu — apontando para o sítio certo: ele não é do widget, é do design system.

---

## §7 — O que este bloco custou, em erros meus

1. **Uma busca de primeiro-cruzamento sobre uma resposta ressonante** (§5.3). Ela devolveu um
   número plausível e errado, e o sinal de que estava errado foi a tabela grossa **discordar** dela
   — `0,0333` media melhor que `0,0300`. *Duas réguas que discordam é informação; uma régua só é
   uma opinião.*
2. **Alargar o percurso de uma fixtura CAÓTICA** (a paridade CPU/GPU do mar). Com o grampo mais
   apertado, dois tiques deixaram de mover o campo acima do piso de vacuidade (`0,0923` contra
   `0,1`). Tentei 4 tiques (paridade `0,00526 > ε`) e 3 (`0,00266`) antes de reler o comentário da
   própria fixtura, que já dizia que ela é caótica **em número de passos**. ⇒ *Quando a régua
   encolhe, alargue a FORÇA* (density 40 → 90, paridade em `2,3e-4`), nunca o número de passos de
   um sistema condicionado no número de passos.
3. **Três testes pinados a literais que este bloco moveu.** Passam a DERIVAR do `MAX_DT` — *um
   teste que fixa o valor de uma constante que ele não possui envelhece com ela.*

---

## ⛔ Recusas MEDIDAS

| o que foi tentado | por que caiu | onde |
|---|---|---|
| baixar o `MAX_DT` do `motion.spring` junto com os outros | ele deriva **três** tetos medidos do dele; um grampo é do SOLVER | §5.4 |
| dar 4 (e depois 3) tiques à fixtura do mar para repor a guarda de vacuidade | a paridade sai do ε (`0,00526` e `0,00266`) — a fixtura é caótica em nº de passos | §7.2 |
| pôr `force.*::strength` na lista de PRECISÃO | o limite dele é de estabilidade, e estabilidade é de uma **composição** (`strength × dt × radius`), não de um param | §1 |
| expor `PANEL_MIN_W` no `panel_chrome.rs` | cap de 500 LOC dos widgets — e ele é do design system, não do widget | §6 |
| subir `MAX_GRADIENT_STOPS` | a medição **confirmou** o 8 | §6 |
