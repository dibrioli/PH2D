# 99 — Estudo sério do Mini Cavalry V2

> **Ordem do Enio, 2026-09-02:** *«MiniCavalry é nosso MVP referência. Mais belo e fácil de usar
> que nosso produto. O objetivo é ser superior e não inferior a ele. Faça um estudo sério sobre
> ele. Vamos tentar superá-lo em tudo.»*

⚠️ **Método.** As duas fontes são **derivadas**, nunca de memória: o lado dele lido do fonte
(`/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2`, 189 ficheiros JS reais, 22 584 linhas) e o
nosso lido do **registry em execução** por sondas (`what_the_socket_encoding_carries`,
`what_our_visual_channels_carry`, `how_much_reads_writes_we_can_derive`). É a disciplina que a
varredura de 28/08 já pagou: ali uma leitura por NOME publicou um item falso (`pivot` «faltava» e
não faltava).

⛔ **Isto NÃO reabre o [doc 92](92_o_que_o_mini_cavalry_tem_e_nos_nao.md)**, que comparou o
CATÁLOGO de nós. Este estudo é sobre o que o Enio nomeou: **belo e fácil de usar**.

⚠️ **E uma coisa que o estudo tem de dizer alto:** o `visual-tokens.js` dele abre com
*«Doc PH2D §6»*. **O sistema visual dele foi derivado de uma spec NOSSA** — o que ele fez foi
**implementá-la**; o que nós fizemos foi declará-la e deixar metade por pintar.

---

## §1 — A assimetria de tamanho, e por que ela importa

| | Mini Cavalry | PH2D |
|---|---:|---:|
| app inteiro | **22 584 LOC** | — |
| só `ph2d-tool-painter` | — | **136 093 LOC** |
| nós registados | **134** | **134** |

⭐ **Empate exacto no número de nós.** Nenhuma vantagem nossa está na contagem, e nenhuma
desvantagem dele está em ser pequeno: ele faz o mesmo catálogo em **1/6** do código de uma só
crate nossa. *O que ele tem é acabamento, não escala.*

---

## §2 — O SOCKET: o achado central, medido dos dois lados

### Dele — 7 tipos, cada um com cor **e** forma própria

`src/core/helpers.js` + o CSS do `mini-cavalry-v2.html` (linhas 871-877):

| tipo | rótulo | cor | forma |
|---|---|---|---|
| `shape` | Shape (Forma) | `#8b5cf6` violeta | ● círculo |
| `point` | Position (Posição) | `#38bdf8` azul | ■ quadrado |
| `value` | Value (Valor) | `#5eead4` teal | ▬ barra horizontal |
| `color` | Color (Cor) | `#facc15` amarelo | ◆ losango |
| `gradient` | Gradient (Gradiente) | `#fb923c` laranja | ▭ barra em degradê |
| `pulse` | Pulse (Pulso) | `#a3e635` verde-limão | ▶ triângulo |
| `skeleton` | Skeleton (Esqueleto) | `#ec4899` magenta | ▮ barra vertical |

### Nosso — 2 formas, e a cor **não distingue nada**

`what_the_socket_encoding_carries`, sobre o registry:

```
134 tipos · 138 portas de saída · 3 tipos com mais de UMA saída
canal COR   (Domain) │ Instances 138 (100,0%)      ⇒ UM valor só
canal FORMA (Dim)    │ ○ escalar 45 (32,6%) · ◇ vector 93 (67,4%)
```

⛔⛔ **O canal da cor é tinta gasta.** O vocabulário inteiro do nosso grafo são **duas** coisas —
*um número* ou *uma corrente* — codificadas num glifo que **nada no ecrã nomeia**.

⚠️ **E a lei do RÓTULO é a MESMA dos dois lados**, medida no fonte dele
(`render-nodes.js:90`): saída única ⇒ um socket sem rótulo; **duas ou mais** ⇒ empilhadas com
rótulo. *A nossa regra não está errada* — o que falta é outra coisa.

⭐ **O que ele tem e nós não, na saída ÚNICA:** o socket carrega `title="Value (Valor) — <descrição>"`.
**Um balão, em todo socket, sempre.** É a superfície que responde *«o que é que isto carrega?»*
sem gastar um pixel permanente.

---

## §3 — Os CHIPS «lê / escreve»: a resposta aos bugs dos últimos dois dias

`src/editor/chips.js`: cada cartão mostra os atributos que o nó **lê** (cinza) e **escreve**
(dourado), de `def.reads_attrs` / `def.writes_attrs`.

⛔⛔⛔ **É exactamente a informação cuja ausência custou os três reports de 01/09:** o
`motion.duplicator` a deitar fora `id`/`vel`/`age`/`life` (a simulação morria), o `size random`
que «parava de funcionar», e a varredura que teve de ser feita por sonda porque **nenhuma
superfície do app responde a essa pergunta**.

### ⭐ E a nossa versão pode ser ESTRITAMENTE melhor — onde existir

Ele **declara** as duas listas à mão em cada nó: uma segunda fonte que pode divergir do que o nó
faz. Nós temos `GpuKernel::bindings`, que diz o [`ColumnAccess`] por coluna e é **a mesma lista
de que o gerador de código vive** — se estivesse errada, o nó computava errado.

`how_much_reads_writes_we_can_derive`:

```
134 tipos
  com kernel de device        │  73 (54,5%)
  ⭐ com BINDINGS deriváveis  │  67 (50,0%) · 197 colunas declaradas
  ⚠️ cuja forma depende de PARAM (variant) │ 9
  ⛔ SEM nenhuma declaração    │  67 (50,0%)
```

⚠️ **Metade, e o `motion.duplicator` está entre os mudos** — ou seja **derivar sozinho não teria
apanhado o bug de ontem**. ⇒ superá-lo aqui é *derivar onde há kernel* **e** *declarar os outros
67*, com um gate que confronte a declaração com o `eval` onde for testável. A parte derivada é
melhor que a dele; a declarada empata.

---

## §4 — A SILHUETA: temos o vocabulário e não o pintamos

Ele tem 7 silhuetas semânticas (`inferSilhouette`): rect = modificador · circle = terminal ·
diamond = decisão · cigar = junção · trapezoidDown = fonte · trapezoidUp = sink · tabbed = I/O
externo. **A forma do nó diz o que ele faz no grafo.**

Nós temos **as mesmas sete** (`NodeSilhouette` em `ph2d-node-registry/src/ui.rs`), e elas estão
**preenchidas** (`what_our_visual_channels_carry`):

```
132 tipos com metadados de UI · 2 sem
Rect 106 (80,3%) · TrapezoidDown 12 (9,1%) · Diamond 7 (5,3%) · Circle 5 (3,8%)
Cigar 1 · TrapezoidUp 1 · Tabbed 0
```

⛔⛔⛔ **E o pintor NUNCA a lê.** `snapshot.rs:139` declara o campo, `snapshot_build.rs:46`
preenche-o do registry, e `paint.rs` **não o menciona uma vez**: todos os 132 cartões desenham-se
como o mesmo rectângulo arredondado. *É a forma pura do knob morto que o [doc 90](90_caca_aos_knobs_mortos.md)
caça — declarado, transportado, e sem consumidor.*

⚠️ Só o `Tabbed` está por atribuir, e hoje já temos os candidatos dele (`source.table`,
`source.text`, `source.object`).

✅ **A categoria, essa, nós pintamos** (`paint.rs:368`, `cat_token`) — 7 famílias bem
distribuídas. Neste canal estamos em pé de igualdade.

---

## §5 — A CONVERSÃO automática: a maior diferença de «fácil de usar», e é um TRADE

`src/core/helpers.js` traz **23 conversões** entre os 7 tipos (`shape->point`, `shape->value`,
`value->color`, `point->skeleton`, …). Ligar tipos diferentes **converte**; quase nada é recusado.

⚠️ **É isto que faz o app dele parecer que «sempre funciona»** — e é o oposto exacto da nossa
disciplina, que ontem passou a **recusar** um fio que não pode conduzir.

⛔ **Não é obviamente melhor, e o estudo não deve fingir que é.** Uma conversão automática
`value->shape` inventa uma forma que o artista não pediu; a nossa recusa diz-lhe porquê. O que
ele compra é *nunca ficar preso*; o que ele paga é *resultados que ninguém autorou*.
⇒ **A saída superior existe e não é nenhuma das duas:** recusar **e oferecer a conversão pelo
nome** («este fio é uma corrente; quer inserir um `value.reduce`?»), que é a forma do
`Deficit::MissingInput` que já temos, com `Fix::Offer`. *Nós temos a maquinaria da oferta; falta
o catálogo de conversões.*

---

## §6 — Os TUTORIAIS: o buraco onde não temos nada

| | Mini Cavalry | PH2D |
|---|---:|---:|
| tutoriais | **20, progressivos, 150 KB** | **0** |
| norma de autoria | `DIRETRIZES.md`, 9 KB, com checklist | — |

⚠️ **E a norma dele é boa o suficiente para ser copiada tal como está.** Ela exige que cada
tutorial: produza algo **que se mova** (*«se o canvas final pode ser confundido com paint.exe, o
tuto não vale»*), cite **2+ casos de uso reais nomeados**, tenha **variações que são receitas
completas** (nó + param + valor concreto), e seja **testado no app antes de publicar**.

⇒ Este é o item onde «superior» custa **conteúdo**, não código — e é o mais barato de todos os
achados deste estudo, porque não toca numa linha de produto.

---

## §7 — Onde NÓS somos superiores (medido, para o estudo ser honesto)

| dimensão | nós | ele |
|---|---|---|
| **Teto de objectos** | **4,19 M em 3,85 ms** no device ([doc 98 §1](98_auditoria_de_performance_2026-09-01.md)) | DOM + canvas 2D num browser |
| **Determinismo** | hash de replay em 3 OS, `BTreeMap` como espinha | — |
| **Undo** | por DIFF, uma fila, snapshot canónico | `history.js` |
| **Espessura de fio** | por **contagem viva** de elementos (`flow::wire_width`) | por tipo (pulse fino / resto grosso) |
| **Diagnóstico no cartão** | selo ⚠ com **cura clicável** (`Deficit` + `Fix::Offer`) | — |
| **Gates** | ~20 300 testes | — |

⭐ **A espessura de fio é a única em que já o superamos no MESMO canal:** a dele diz o *tipo*
(dois valores, que o pino já diz); a nossa diz **quantos elementos estão a passar agora**.

---

## §8 — A tensão que só o dono resolve

O cartão dele é **bilingue** — `socketLabel` devolve `"Value (Valor)"`, e é daí que sai o
«Oscillator (Oscilador)» da foto. Para um utilizador brasileiro isso é «mais fácil de usar»
directo.

⛔ **E a nossa lei diz o contrário, e a lei é dele:** [[feedback_app_ui_english_only]] —
*«Tire todo PT do app… tudo em inglês»* (Enio, 2026-05-26), com um incidente registado de um
auditor que a violou. ⇒ **nomeado, não mexido.**

---

## §9 — O que «superar em tudo» custa, por ordem de preço

| # | item | preço | porquê agora |
|---|---|---|---|
| 1 | **Pintar a SILHUETA** | ~nada — o dado já viaja até ao pintor | 132 nós já a declaram; hoje é um knob morto |
| 2 | **Dar sentido à COR do pino** | ~nada — o canal existe e não distingue nada | é a causa directa do bug de ontem |
| 3 | **Balão no socket** («o que isto carrega») | pequena — os sockets do canvas não estão no `HitIndex` do chrome | responde a pergunta que o rótulo não responde |
| 4 | **Chips «lê / escreve»** | média — derivar 67, declarar 67, gate a ligá-los | teria apanhado os 3 reports de 01/09 |
| 5 | **Catálogo de CONVERSÕES como OFERTA** | média — a maquinaria do `Fix::Offer` já existe | é o «fácil de usar» dele sem o preço dele |
| 6 | **20 tutoriais** | conteúdo, zero código | o único buraco em que temos **nada** |

⚠️ **Os itens 1 e 2 são grátis e atacam a causa medida do report de ontem.** O 6 não toca no
produto. Os 3-5 são waves com desenho próprio.

---

## §10 — FEITO em 2026-09-02: os dois canais grátis

Ordem do Enio no mesmo dia: *«siga como sugere»*.

### (a) A COR do socket passa a dizer a ESPÉCIE

`socket_token` (`paint.rs`) substitui o `domain_token` no glifo. As **três** espécies que o
módulo de facto tem, contadas:

| espécie | portas | token |
|---|---:|---|
| pulso (`Clock::Event`) | 8 (5,8%) | `PortEvent` ⭐ **já existia e nunca fora usado** |
| número (`Dim::Scalar`) | 45 (32,6%) | `PortValue` ⭐ **novo**, teal, matiz 190 |
| corrente (vector) | 93 (67,4%) | `PortInstances` — **a cor de sempre** |

⚠️ **O relógio pergunta-se PRIMEIRO**: um pulso escalar é um pulso, não um número. Gate
`a_scalar_pulse_reads_as_a_pulse_and_not_as_a_number`, e a mutação que troca a ordem morre.
⚠️ **93 das 138 portas não mudam um pixel** — uma cura que repintasse tudo seria uma mudança
de tema disfarçada de correcção (gate `the_stream_keeps_the_colour_it_always_had`, mutação
morta). ⚠️ **A colisão de matiz com o `port-signal` (195) está NOMEADA no token**, não
descoberta: os dois separam-se por luminosidade e croma, e o `Domain::Signal` nunca aparece
num grafo de Motion.
⏳ **O FIO ainda não** — a `GraphEdgeView` só carrega `out_domain`; pintá-lo pela espécie pede
um campo novo no snapshot.

### (b) A SILHUETA passa a ser PINTADA — como selo no cabeçalho

⚠️⚠️ **E a minha palavra «grátis» estava errada, corrigida por medição.** A primeira ideia era
o contorno do CARTÃO, como ele faz — mas o corpo define os rectângulos de hit **e a posição de
cada socket** (`geom`/`hits`), então mudar-lhe a forma arrasta a geometria toda. **O cabeçalho
é a única superfície onde de facto é grátis.**

⭐ **E o censo escolheu a lei:** `Rect` são **106 de 132 nós (80,3%)** — o modificador genérico.
Um selo neles seria o mesmo ruído que a lei do rótulo de porta já recusou. ⇒ **`Rect` não veste
selo**, e 106 cartões saem byte a byte como antes; os **26** que declaram outro papel pagam
`15 px` do título (`178 → 163`, `−8,4%`), medido e gateado.

Seis formas, com a tinta do TÍTULO (não uma sexta cor a competir com a categoria): ○ terminal ·
◇ decisão · cápsula junção · trapézio-baixo fonte · trapézio-cima sink · aba I/O externo.
⚠️ O `match` é **exaustivo sem braço `_`**: um papel novo no registry passa a ser **erro de
compilação** no único sítio onde alguém se lembraria de lhe dar forma.

⭐ Nasceu uma primitiva partilhada, `ph2d_editor_core::paint_shapes::fill_polygon` — *uma geral
custa menos que três especiais e não convida a uma quarta*.

### (c) O BALÃO no socket — a pergunta que o rótulo não responde

`socket_tip` (`paint_role.rs`): passar o rato num socket diz **`Target X · a number`**,
**`P · a stream`**, **`Pulse · a pulse`**.

⚠️⚠️ **E a minha estimativa de preço estava errada outra vez** — eu escrevera *«pequena — os
sockets do canvas não estão no `HitIndex` do chrome»*. **Estão**: o `hits.rs` já os regista no
`HitIndex` **e** no `WidgetStore` (é de lá que sai o a11y deles), e o `paint_hover_tooltip` lê
exactamente esses dois. ⇒ o balão custou **um `set_tooltip`**. *Duas estimativas de custo
minhas erradas no mesmo dia, as duas por afirmar a ausência de uma peça sem a procurar.*

⭐ **Uma lista, uma condição:** o balão sai do **mesmo laço** que empurra o hit, porque quem
decide se um socket existe é o `clip_rect` — um socket panado para fora do canvas não tem hit,
e uma segunda travessia deixaria balões órfãos no store. Gate no teste de recorte que já
existia; a mutação que tira o `tips.push` de dentro do `if` morre.

⛔ **A COR e o TEXTO são a mesma lei em dois canais**, com gate a percorrer as combinações
(`the_colour_and_the_words_never_disagree`) — duas partições separadas divergiriam no dia em
que uma mudasse.

⚠️ Em inglês ([[feedback_app_ui_english_only]]), e o nome sai da **mesma** derivação que o
rótulo do cartão (`PortLabel`) — uma segunda tabela de nomes seria a que envelhece.

---

## ⛔ Recusas MEDIDAS deste estudo

| recusa | mecanismo |
|---|---|
| *«ele rotula todas as saídas e nós não»* | **Falso**: `render-nodes.js:90` — saída única não leva rótulo, igual a nós. O que ele tem a mais é o **balão** e a **cor por tipo** |
| *«ele tem mais nós»* | **Falso**: `registerNode` conta **134**; nós também |
| *«a conversão automática é obviamente melhor»* | **Não**: ela inventa resultado que ninguém autorou. A saída superior é recusar **e oferecer** |
| *«devíamos ficar bilingues como ele»* | ⛔ contraria a lei do próprio dono (§8) — nomeado, não proposto |
| *«a nossa silhueta não existe»* | **Falso**: existe, está preenchida em 132 nós e tem 6 valores — só não era **pintada** (§10b) |
| *«pintar a silhueta é grátis»* | **Meu, e falso**: o contorno do CARTÃO arrasta hits e sockets. Grátis só no **cabeçalho** (§10b) |
| *«a cor do socket é um canal com dois valores»* | **Três**: o `Clock::Event` distingue **8 portas** que nem o domínio nem a dimensão vêem (§10a) |
| *«o balão no socket é uma wave — eles não estão no `HitIndex`»* | **Meu, e falso**: estão, e o a11y deles sai de lá. Custou **um `set_tooltip`** (§10c) |
