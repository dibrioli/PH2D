# 100 — Estudo dos outputs: um nó com muitos pinos, ou muitos nós com um pino?

> **Ordem do Enio, 2026-09-04:** *«Porque no oscillator do minicavalry temos outputs no nó que aqui
> não temos? … quero uma avaliação completa de ambos, quero que descubra qual a abordagem é
> melhor. Buscamos o padrão ouro, o estado da arte. Faça análise séria.»*

⚠️ **Método.** Os dois catálogos são lidos do FONTE, nunca de memória: o dele de
`src/nodes/*.js` (134 `registerNode`) e de `src/core/registry.js` (o que um nó ganha quando não
declara pinos); o nosso dos 131 `MANIFEST` em `crates/ph2d-node-*/src/lib.rs`, batidos contra o
censo do registry em execução (`what_the_socket_encoding_carries`: 134 tipos · 138 portas · 3 com
mais de uma). O estado da arte vem com fonte ao lado de cada afirmação (§3). E a pergunta que
decide o veredito — *os dois caminhos da casa dão os mesmos bits, chegam ao device, e custam
quanto?* — foi **medida** por uma sonda nova, `the_tee_against_the_fused_node` (§5).

⛔ **Isto NÃO reabre o [doc 99](99_estudo_do_mini_cavalry_2026-09-02.md)** (o sistema visual,
os chips, a silhueta). Este estudo é sobre UMA pergunta de desenho: **onde vive o que um nó
calculou e não é a saída principal** — num segundo pino, numa coluna da corrente, ou noutro nó.

---

## §1 — O facto que abriu a pergunta

O oscilador dele (`src/nodes/oscillator.js`) tem três pinos; o nosso (`motion.oscillator`) tem um.

| pino | Mini Cavalry | PH2D |
|---|---|---|
| a corrente já a balançar | `out` (shape) | `out` (`Instances/Vec2/Frame`) |
| o número da onda | `value` (value) | — (é outro nó: `value.lfo`) |
| o disparo ao cruzar o zero | `pulse` (pulse) | — (é outro nó: `pulse.compare` / `pulse.on_change`) |

⚠️ **E o `value` dele NÃO é o número que o nó aplicou.** Lido no `process`:

```js
const phase0 = time * frequency * TAU;                 // sem instâncias
let baseV; switch(wave) { … baseV = Math.sin(phase0) … }
…
const shaped = ins.map((inst, i) => {                  // por instância
    const phase = time * frequency * TAU + i * phaseStagger;
    …
    const value = (v * amplitude) * getAttr(inst, 'FalloffStrength');
    return channelSet(inst, ch, cur + value);
});
return { out: shaped, value: [baseV * amplitude], pulse: [{ value: baseV > 0 ? 1 : 0, t: time, edge }] };
```

O pino `value` é a onda **do elemento 0, sem o `falloff`, recomputada** — um segundo cálculo com
os mesmos knobs, não um subproduto do primeiro. E o `pulse` sai de estado guardado no nó
(`node._oscPrev`, reposto quando `time < node._lastTime`): o disparo em `t` depende da ORDEM em
que os quadros foram avaliados, o que um scrub não reproduz.

*Isto importa porque é a diferença entre um pino que ENTREGA algo que o nó já tinha e um pino
que EMPACOTA um segundo nó dentro do primeiro.* A §2 mede quantos de cada.

---

## §2 — Censo dos dois catálogos (medido 2026-09-04)

| | Mini Cavalry | PH2D |
|---|---:|---:|
| nós registados | **134** | **134** |
| nós com MAIS de um pino de saída | **30** (22,4%) | **3** (2,2%) |
| pinos extra (além do primeiro) | **49** | **4** |
| nós que não declaram pinos e ganham `out: shape` por omissão | 76 (`_normalizeIO`) | — (todo manifesto declara) |

### Os 49 pinos extra dele, classificados pelo que os produz

| classe | o que é | pinos | exemplos |
|---|---|---:|---|
| **decomposição** | um dispositivo lido uma vez, partido em campos | **20** | `mouse` (6), `gamepad` (5), `viewportSize` (4), `keyboard`, `touchMulti`, `mousePosition`, `extractXY`, `colorPicker.alpha` |
| **subproduto** | um número que o cálculo principal já tinha | **17** | `lookAt.angles`, `collisionEvent.pulse/contact/count`, `loopSequencer.pulse/step`, `celAnimation.frame`, `vectorMath.scalar`, `threshold.pulse`, `hitTest.pulse`, `audioReact.value/bass` |
| **projecção** | a mesma saída sem a forma (só `x,y,index,count`) | **7** | `distribute*.points` (5), `numberRangeToColor.colors`, `lookAt.target` |
| **tipo duplo** | o mesmo valor em dois tipos de socket | **3** | `lag.outPoint`, `sampleHold.outPoint`, `springDriver.outPoint` |
| **recomputação** | um segundo cálculo com os mesmos knobs | **2** | `oscillator.value`, `oscillator.pulse` |

⭐⭐ **Só 2 dos 49 são recomputações — e os dois estão no oscilador.** O nó que o Enio apontou
é a **excepção** do catálogo dele, não a regra: 47 de 49 pinos extra entregam algo que o nó já
tinha na mão (ou uma partição de uma leitura só). *A pergunta certa não é «porque é que ele tem
três pinos e nós um», é «porque é que nós temos 4 pinos extra num catálogo do mesmo tamanho».*

### Os 4 pinos extra nossos

| nó | pino | classe | por que existe (do próprio código) |
|---|---|---|---|
| `pulse.counter` | `carry` | subproduto | *«o carry dispara se a contagem AVANÇOU e cruzou um limite»* — `beat → counter(4) → carry → counter(4)` é um divisor |
| `sim.lifetime` | `died` · `pulse` | subproduto | *«o sistema de tipos FORÇA a separação: a CARGA (`Instances/Vec2/Frame`) e o GATILHO (`Instances/Scalar/Event`) não cabem no mesmo fio»* |
| `value.cursor` | `x` · `y` | decomposição | um dispositivo, dois números |

⇒ **4 de 4 obedecem à mesma regra que 47 de 49 dele.** Os dois catálogos seguem a MESMA lei sem
a terem escrito; a diferença é quantas vezes cada um a exerce (49 contra 4) — e a razão dessa
diferença está na §4.

---

## §3 — O estado da arte: quatro modelos, e o padrão-ouro é uma COMPOSIÇÃO deles

Não há UM modelo vencedor no estado da arte. Há quatro, e as ferramentas de referência
combinam-nos — cada uma numa mistura própria.

### Modelo 1 — pinos empacotados (*bundled outputs*)
O nó expõe cada coisa que produziu como um pino. **Grasshopper**: o *Divide Curve* devolve
Pontos, Tangentes e Parâmetros — os três do MESMO corte, e *«all curve division components
have these three outputs»* ([Hopific](https://hopific.com/how-to-divide-a-curve-in-grasshopper/)).
**Max/MSP**: o `counter` tem saídas separadas para a contagem, o *underflow* e o *carry*
([Cycling '74](https://docs.cycling74.com/max8/refpages/counter)). O Mini Cavalry vive aqui.

### Modelo 2 — atributos na corrente (*attributes on the stream*)
O que o nó calculou viaja **na própria geometria**, com nome; quem quer lê-o a jusante.
**Houdini** é o caso canónico — a informação vive em atributos, e a superfície que responde
*«o que é que este fio carrega?»* é o **Info** do botão do meio (*«shows the number of points,
primitives … as well as the groups and attributes in the node's geometry»*,
[SideFX](https://www.sidefx.com/docs/houdini/network/info.html)) e a **Geometry Spreadsheet**
([SideFX](https://www.sidefx.com/docs/houdini/ref/panes/geosheet.html)). SOPs com mais de um
pino existem e são raros — o *Split* parte *«into two streams: the portion that matches the
group and the portion that doesn't»* ([SideFX](https://www.sidefx.com/docs/houdini/nodes/sop/split.html)),
e HDAs só ganharam mais de uma saída no 17.5
([80.lv](https://80.lv/articles/006sdf-17-5-houdini-review-from-olaf-finkbeiner)).
**Niagara** (Unreal) é o mesmo modelo levado ao extremo: um módulo lê e escreve
`Particles.Position` num *parameter map* — *«the particle payload, the data we have asked to
come along»* — e a UI mostra por parâmetro a contagem de **leituras|escritas** (`0|0` para o que
ninguém usa) ([tokeru](https://tokeru.com/cgwiki/Niagara.html),
[ibbles](https://github.com/ibbles/LearningUnrealEngine/blob/master/Niagara.md)).
**Nuke**: um fio carrega `rgba`, `depth`, `motion`, `disparity`… como *layers* nomeadas
([Foundry](https://learn.foundry.com/nuke/developers/latest/pythondevguide/channels.html)).
**TouchDesigner**: um CHOP *«contains a set of Channels»* e passa-os ao seguinte
([Derivative](https://docs.derivative.ca/CHOP)) — um pino, N canais nomeados.

### Modelo 3 — a tee: uma fonte de valor, N consumidores
O número nasce num nó SEU e vai por fio a quem o quiser. **Cavalry**: *«An attribute can have
several outputs but can only have one input»*
([Cavalry](https://cavalry.studio/docs/getting-started/key-concepts/connections)) — o Oscillator
dele é um *behaviour* que se liga a `position.x`, a um *Blur Amount*, ao que for, e o MESMO
oscilador alimenta vários alvos ([Cavalry](https://cavalry.studio/docs/nodes/behaviours/oscillator/)).
É o modelo do nosso `value.lfo → motion.drive`.

### Modelo 4 — tudo é saída (*every attribute is an output*)
Qualquer atributo de qualquer nó pode conduzir qualquer outro, sem pino desenhado: os
*«outputs are indicated by a right facing purple icon»* em cada linha do editor de atributos do
Cavalry (mesma fonte), e o **pick-whip** do After Effects
([Adobe](https://helpx.adobe.com/after-effects/using/expression-basics.html)). Aqui a pergunta
«que pinos tem o nó?» deixa de existir — o custo é que o grafo não MOSTRA o que está ligado a quê.

### ⭐⭐⭐ O padrão-ouro: o Blender Geometry Nodes combina 1 + 2 + preguiça

O *Distribute Points on Faces* mostra os pinos `Points`, `Normal`, `Rotation` (modelo 1) — e por
baixo, `Normal` e `Rotation` são **atributos anónimos gravados na nuvem de pontos** (modelo 2),
que **só são calculados se alguém a jusante os ligou**. Lido no fonte
([`node_geo_distribute_points_on_faces.cc`](https://raw.githubusercontent.com/blender/blender/main/source/blender/nodes/geometry/nodes/node_geo_distribute_points_on_faces.cc)):

```cpp
attribute_outputs.rotation_id = params.get_output_anonymous_attribute_id_if_needed("Rotation");
attribute_outputs.normal_id   = params.get_output_anonymous_attribute_id_if_needed("Normal", bool(attribute_outputs.rotation_id));
…
if (attribute_outputs.normal_id) { normals = point_attributes.lookup_or_add_for_write_only_span<float3>(*attribute_outputs.normal_id, AttrDomain::Point); }
```

E a superfície: ao passar o rato num socket o Blender mostra *«the value, type, and any
errors»* — e *«socket inspection only works for sockets that have been computed already»*
([devtalk](https://devtalk.blender.org/t/tooltips-for-shader-and-geometry-nodes/22758),
[T91605](https://developer.blender.org/T91605)).

### A lei que sai dos quatro

> **Um segundo pino é legítimo quando entrega um SUBPRODUTO** — algo que o nó já tinha e que a
> jusante não reconstrói (barato, ou de todo: a normal no ponto amostrado, o cadáver que a
> reaper apanhou, o *carry* de um contador) — **ou uma ESPÉCIE diferente** que não cabe no mesmo
> fio (um pulso ao lado de uma corrente). **Não é legítimo para um valor recomputável dos mesmos
> knobs**: para esse, a tee é estritamente melhor — uma fonte, N consumidores, e nenhum knob
> escrito duas vezes. E o padrão-ouro ainda exige duas coisas do SUBSTRATO: que um pino
> desligado **não custe** (preguiça por porta) e que exista uma **superfície que diga o que o fio
> carrega** (Info, spreadsheet, inspecção de socket).

A §2 já mediu que os dois catálogos obedecem à primeira metade (47/49 e 4/4). A §4 mede a casa
contra a segunda.

---

## §4 — O que a casa JÁ É, medido contra a lei

### 4.1 O modelo: atributos na corrente + tee — o mesmo do Houdini e do Niagara

- **A espécie está no tipo, e o tipo força o pino.** `PortType::connects_directly` exige
  domínio + dimensão + relógio iguais, então um pulso (`Instances/Scalar/Event`) e uma corrente
  (`Instances/Vec2/Frame`) **não cabem no mesmo fio** — é por isso que o `sim.lifetime` tem três
  pinos e não um. A lei da §3 não precisa de ser decorada aqui: o compilador aplica-a.
- **A corrente carrega colunas com nome, e qualquer nó lê qualquer uma.** Medido nos fontes:
  **44 colunas distintas** escritas por **101 crates-nó** (`P` 385 sítios · `size` 76 ·
  `falloff` 62 · `rot` 47 · `tint` 46 · `vel` 32 · `age` 10 · `life` 5 · …), e o
  `value.attribute` lê **qualquer** delas por nome (*«Blender's Named Attribute; Houdini's
  `@age`, `@id`, `@speed`»*). O subproduto de um nó da casa já tem onde viver sem um pino:
  o `motion.look_at` escreve `rot`, o `motion.spring` escreve `spring_vel`, o `sim.lifetime`
  escreve `life` — e o próprio doc dele diz *«`life` is the point, not a by-product»*.
- **A tee existe e é de primeira classe.** `value.lfo` é *«the pure producer form of
  `motion.oscillator`»* — a MESMA lei da onda (parábola + correcção de Capens, copiada por
  crate-folha), `phase_stagger` por elemento, a porta `in` lida só pela contagem, kernel de
  device. `motion.drive` é *«the Cavalry "connect this value to that attribute" made a
  first-class node»* — oito modos (`Add`/`Set`/`Multiply`/…/`Remap`), a regra de broadcast
  `1 → N`, kernel de device, e o mesmo valor *«can fan out to several drives (one count → X and
  Rotation at once), which no bundled node can»*. O pulso da mesma onda é `pulse.compare` ou
  `pulse.on_change` (entrada `VALUE`), com Schmitt e histerese; o `pulse.threshold` lê um canal
  da corrente.

⇒ **Em MODELO, a casa é o padrão-ouro da §3** (modelo 2 + modelo 3, o par Houdini/Cavalry).
O que se segue é onde ela NÃO é.

### 4.2 O cook: todo pino é cozido sempre — e o substrato só sabe ser preguiçoso com ENTRADAS

`Cook::cook_node` exige que o `eval` emita **exactamente** `manifest.outputs.len()` correntes
(`CookError::OutputCountMismatch`), então um pino desligado custa o mesmo que um ligado. A única
preguiça do substrato é o [`LazySelect`](../../crates/ph2d-nodegraph/src/cook_lazy.rs) — *quais
ENTRADAS de um nó de selecção o cook pode não cozinhar* (medido: `3,83×` no `value.switch`). Não
há o equivalente do `get_output_anonymous_attribute_id_if_needed` do Blender: um nó não tem como
perguntar *«alguém ligou o meu pino 1?»* (o `EvalCtx` não o expõe). O *bypass* passa a porta 0 e
deixa as outras `Empty` — o único sítio em que um pino extra sai de graça.

⚠️ Hoje isto custa pouco porque os 4 pinos extra são baratos (o `died` é um *gather* sobre os
mortos; o `carry` é um `Vec<f32>` por elemento). Custaria a partir do momento em que um pino extra
pedisse um segundo passe sobre a corrente inteira — que é exactamente o que o `value` do
oscilador dele é.

### 4.3 ⛔⛔ O device: um estágio produz UM buffer, e o segundo pino DERRUBA o nó para a CPU

Lido no planeador ([`plan.rs`](../../crates/ph2d-gpu-cook/src/plan.rs)):

```rust
// **A GPU stage produces ONE buffer.** [`GpuStage`] holds a `node`, never a
// `(node, port)`, and `source_of` resolves an input to `GpuSource::Stage(src)`
// with the port dropped — so a consumer reading port 1 of a staged node would
// be handed port **0**: the wrong stream, silently …
if graph.edges().iter().any(|e| e.from.0 == node && e.from.1 != 0) { return false; }
```

⇒ **Ligar o `died` ou o `pulse` do `sim.lifetime` tira o `sim.lifetime` — e tudo a montante
dele — do caminho de `50,9×`** ([doc 98](98_auditoria_de_performance_2026-09-01.md)). Hoje
isso acontece a um nó; um oscilador com `value` e `pulse` ligados aconteceria ao nó mais comum
do módulo. *Esta é a razão MEDIDA pela qual a casa tem 4 pinos extra e não 49: o substrato
que corre a milhões de objectos pune o segundo pino, e os 49 dele correm num `Array.map` da CPU.*

### 4.4 O documento: o pino é um ÍNDICE, e por isso apendar é grátis e inserir é veneno

`Edge { from: (NodeId, u16), to: (NodeId, u16) }`, gravado como
`e <from_id> <from_port> <to_id> <to_port>`. Um pino novo **no fim** de `outputs` deixa toda
cena gravada a apontar para o mesmo sítio; um pino inserido antes do `out` re-aponta em silêncio
— a mesma lei que a porta `time` do oscilador já cumpre (*«APENDADA, nunca inserida»*).

### 4.5 A superfície: o que cada app diz ao artista sobre o que um fio carrega

Lido nos dois fontes (o dele: `render-nodes.js` · `chips.js` · `preview.js` ·
`geo-spreadsheet.js` · `probe.js` · `pull-evaluator.js`; o nosso: `paint_role.rs` ·
`paint_port_label.rs` · `geom.rs` · `motion_bridge_subgraph.rs` · `motion_bridge_columns.rs`).

| superfície | Mini Cavalry | PH2D |
|---|---|---|
| cor do pino | por **tipo** (7 cores, 7 formas) | por **espécie** (pulso · número · corrente — [doc 99](99_estudo_do_mini_cavalry_2026-09-02.md), 3 tokens) |
| rótulo dos pinos | 1 saída ⇒ sem rótulo · ≥2 ⇒ empilhados com rótulo | **a mesma lei** (`labelled_outputs = n.outputs.len() > 1`) |
| balão do pino | rótulo **+ descrição** do nó | rótulo **+ espécie** (`Out · a stream`) — ⛔ **não diz as colunas** |
| chips «lê / escreve» | ✅ de `reads_attrs`/`writes_attrs` — ⚠️ **declarados à mão, ou INFERIDOS pela categoria** (`inferAttrsForDef`), e um `⚠` no cartão quando a execução toca fora do declarado (`_attrViolations`, `pull-evaluator.js:234`) | ⛔ construído, medido em 40 cenas e **revertido** ([handoff 03/09 §6](handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-03.md)): *quase sempre certo, quase sempre irrelevante* |
| pré-visualização no cartão | mini-canvas da saída, por nó, com *throttle* (seleccionado a cada quadro; os outros 1/5; 30 redesenhos/quadro) | **moldura** acima/abaixo do cartão com as posições da corrente ([doc 86](86_plano_objetos_engine_render_e_preview.md), Feature B) — só para nós posicionais; um nó de valor mostra o **número** vivo |
| spreadsheet de geometria | ✅ instâncias × atributos do nó seleccionado, ao vivo (*«estilo Houdini»*) | ⛔ **não existe** (nenhum ficheiro do repo diz *spreadsheet*) |
| sondas de valor | ✅ `probe.js` (lê o `_evalCache`) | ⛔ não existe |
| recusa que nomeia a cura | — | ✅ o toast diz **qual nó converte** (`converter_from`, derivado do registry) |
| rota de cada cena (device/CPU) | — | ✅ `PH2D_MOTION_ROUTE_LOG=1` (todo *fall-through* nomeia-se) |

⭐ **A leitura honesta:** em pinos e tipos estamos empatados (a lei do rótulo é a mesma; a
cor dele distingue mais porque o CATÁLOGO dele tem 7 espécies e o nosso Motion tem 3). Onde
ele está à frente é na **superfície de dados** — chips, spreadsheet, sondas — que responde à
pergunta que os três reports de 01/09 fizeram: *«o que é que esta corrente carrega neste
ponto?»* ⚠️ E a resposta dele é feita da segunda fonte (declaração à mão / inferência por
categoria) que o [doc 99 §3](99_estudo_do_mini_cavalry_2026-09-02.md) já mediu ser o defeito —
com a diferença de que ele **confronta a declaração com a execução** (`_attrViolations`), que
é exactamente o gate que a nossa versão derivada pedia.

---

## §5 — A sonda: a tee CONTRA o nó fundido (medido 2026-09-04)

`the_tee_against_the_fused_node` ([`motion_bridge_tee_probe_tests.rs`](../../shells/desktop/src/render_loop/motion_bridge_tee_probe_tests.rs)):
`grid → motion.oscillator(Y)` contra `grid → value.lfo(in = grid) → motion.drive(in = grid,
value, Add, Y)`, amplitude 37, stagger 0,013 por elemento, sem `falloff`.

**Os mesmos bits?** Grelha 100×100, seis instantes.

| | `t` | diferem | max \|Δy\| | max ULP |
|---|---:|---:|---:|---:|
| 2 Hz ≡ período 0,5 s (exactos em binário) | 0 · 0,1 · 0,37 · 1,234 · 7,5 · 33,3 | **0 / 10 000** em todos | 0 | 0 |
| 3 Hz ≡ período ⅓ s | 0,1 | **53 / 10 000** | 3,05e-5 | 8 |
| | os outros cinco | 0 / 10 000 | 0 | 0 |

⭐⭐ **A tee É o oscilador ao bit** — a onda, o *stagger* por elemento, a máscara de `falloff`
(`resolve` com `f = 1` é `base + ((base + v) − base)`, que nas 60 000 amostras deu o mesmo bit
que `base + v`). O único desvio é o **par de knobs**: o oscilador mede a onda em `frequency`
(`t·f`) e o LFO em `period` (`t/p`), e para `f = 3` o `f32` não tem `1/3` — **8 ULP em 53 de
10 000 elementos, num instante de seis**. *Dois nós, uma lei, dois nomes para o mesmo número:
é a deriva de que a §1 acusou o Mini Cavalry, um nível abaixo — e só a medição a vê.*

**Chegam ao device?**

| | `fully_gpu` | estágios | passes |
|---|---|---:|---:|
| fundido | ✅ | 3 | 2 |
| tee | ✅ | 4 | 3 |

Os dois inteiros no device: a tee paga **um passe a mais** por quadro (o LFO escreve a coluna
`v`, o drive lê-a), contra um nó que faz as duas coisas num passe. O contrário — o oscilador
com `value`/`pulse` como pinos — **cairia da GPU no momento em que alguém os ligasse** (§4.3).

**E na CPU, a 1 000 000 de elementos?** (carga no arranque `7,11`, no fim `2,85` — a corrida
é o segundo `cook`, com a máquina a esvaziar; §5.0)

| | frio | segundo instante |
|---|---:|---:|
| fundido | 6,67 ms | 6,65 ms |
| tee | 14,82 ms | 11,15 ms |

A tee custa **1,7×** na CPU — e a razão está escrita no `value.lfo`: o laço dele é
`(0..n).map(...)` **serial**, enquanto o oscilador é `par_build` (é um dos ~29 nós por-elemento
seriais que o [doc 98 §5](98_auditoria_de_performance_2026-09-01.md) contou). Não é o preço da
tee; é o preço de um nó por paralelizar. ⚠️ E nenhum dos dois números é o do produto — o
produto corre isto no device, onde os dois são `fully_gpu`.

---

## §6 — Veredito

**Nenhuma das duas abordagens, tal como o Enio as nomeou, é o padrão-ouro.** O estado da arte
(§3) não escolhe entre «um nó com muitos pinos» e «muitos nós com um pino»: ele **expõe pinos
para SUBPRODUTOS, guarda o resto como ATRIBUTOS da corrente, não cozinha o pino que ninguém
ligou, e mostra ao artista o que o fio carrega.**

Medido contra isso:

| critério do padrão-ouro | Mini Cavalry | PH2D |
|---|---|---|
| pinos extra são subprodutos / espécies | ✅ **47 de 49** — ⛔ e as 2 recomputações são o oscilador | ✅ **4 de 4** |
| o resto viaja como atributo com nome | ✅ (`getAttr`/`setAttr`, schema `PH2D_Attr`) | ✅ 44 colunas · 101 crates · `value.attribute` |
| a tee (uma fonte, N consumidores) é de primeira classe | ✅ `lfo` + slots promovidos | ✅ `value.*` → `motion.drive` (8 modos, broadcast) — **ao bit** igual ao nó fundido |
| um pino desligado não custa | ⛔ `process` devolve tudo, sempre | ⛔ todo pino é cozido (`OutputCountMismatch`) |
| o segundo pino corre onde o primeiro corre | ✅ (tudo é CPU) | ⛔ **derruba o nó para a CPU** (`GpuStage` sem porta) |
| o artista vê o que o fio carrega | ✅ chips + spreadsheet + sondas (**da declaração**, confrontada em execução) | ⛔ só a espécie no balão; sem spreadsheet; sem sondas |
| determinismo do pulso | ⛔ `node._oscPrev` — depende da ordem dos quadros | ✅ `pre` sobre o tique (Schmitt, histerese, *debounce*) |
| quem lê por-elemento, lê no device | ⛔ | ✅ 4,19 M em 3,85 ms |

⭐⭐⭐ **A resposta à pergunta do Enio:** o oscilador dele tem três pinos porque ele **empacotou
um LFO e um threshold dentro do oscilador** — e é o único sítio do catálogo dele onde ele faz
isso. O nosso tem um porque a casa entrega essas duas coisas por **outros dois nós que, ligados,
dão exactamente os mesmos bits e correm no device**. Em modelo, estamos do lado certo da lei
(e ele também, 47 vezes em 49). **Onde ele nos bate não é o pino: é a SUPERFÍCIE** — chips,
spreadsheet, sondas, o `⚠` que diz *«este nó tocou fora do que declarou»*. E onde nós lhe batemos
é o **substrato** — o pulso determinístico, a corrente por-elemento no device, a recusa que
nomeia a cura.

⛔ **Copiar os três pinos do oscilador seria copiar a EXCEPÇÃO dele e pagar com o nosso ponto
forte:** os pinos `value`/`pulse` tirariam o nó mais comum do módulo do caminho de `50,9×` no
instante em que fossem ligados (§4.3), e recomputariam (na GPU e na CPU) o que a tee já entrega
sem um knob a mais.

---

## §7 — O que fazer, por ordem — e o que NÃO fazer

1. **⛔ Não dar `value`/`pulse` ao `motion.oscillator`.** É a recomputação (§2), cai do device
   (§4.3), e a tee já é ele ao bit (§5). *Recusa medida.*
2. **O substrato: `GpuStage` passa a `(nó, porta)`.** É o pré-requisito de QUALQUER segundo pino
   no device — hoje o `died`/`pulse` do `sim.lifetime` (o evento de morte de todo sistema de
   partículas) tira a simulação inteira da GPU quando alguém o liga. É wave de substrato
   (`ph2d-gpu-cook`), não de nó; medir o preço antes (quantas cenas das 109 ligam um pino ≠ 0).
3. **Preguiça por PORTA, se e quando um pino extra custar um passe.** Hoje não custa (os 4 são
   baratos); o mecanismo é o do Blender (`get_output_anonymous_attribute_id_if_needed`) e o
   `EvalCtx` já tem onde nascer (`lazy_skip_mask` faz o mesmo para ENTRADAS). ⏸️ Só quando o
   item 2 permitir pinos no device — antes disso não há pino extra caro para poupar.
4. **A superfície de dados — é aqui que se supera, e é barato:**
   - **as COLUNAS no balão do pino** (*Blender socket inspection*): o `stream_at(node, port)`
     já existe (`motion_bridge_columns.rs`) e o balão já é só do pino quente (`tip_for_hot`);
     é dizer `Out · a stream · P size rot falloff (400 rows)` em vez de `Out · a stream`.
     ⚠️ É a espécie que **DESCREVE**, não a que ACUSA — o chip `drops` foi recusado por acusar
     quase sempre sem ter razão para o artista; um balão que lista o que HÁ é a outra metade da
     mesma lei (handoff 03/09 §6).
   - **a spreadsheet de geometria** (Houdini / ele): instâncias × colunas do nó seleccionado.
     É um painel novo sobre o `peek` do cook — e é a superfície que teria respondido aos três
     reports de 01/09 sem uma sonda.
   - **o `⚠` de execução contra declaração** — a versão derivada dos chips (doc 99 §3): onde há
     `GpuKernel::bindings`, o `ColumnAccess` É a declaração; um gate que confronta o `eval` da
     CPU com os bindings fecha o buraco sem uma segunda lista.
5. **A tee tem de ser ENCONTRÁVEL.** O artista que vê um pino no cartão dele aprende que o
   oscilador dá um número; o nosso aprende só se já souber que existe `value.lfo`. Três
   candidatos, por ordem de custo: (a) o balão do `Out` do oscilador nomear o irmão (*«the same
   wave as a value: `value.lfo`»* — o `converter_from` já deriva um irmão do registry para as
   recusas); (b) um **preset da paleta** *«Oscillator + value + pulse»* que larga
   `lfo → drive` + `compare` já ligados (a casa tem clipboard e grupos, não tem presets de
   sub-grafo — medir); (c) um gesto *split* sobre um nó fundido que o troca pela tee. ⏸️ Decisão
   de produto do Enio.
6. **`value.lfo` paralelo** (`par_build`, como o oscilador): `14,82 → ~6,7 ms` a 1 M na CPU. Um
   dos ~29 do doc 98 §5 — pequeno, e a medição já está feita aqui.
7. **Os dois knobs para uma lei** (`frequency` no oscilador, `period` no LFO): 8 ULP a 3 Hz.
   Nomeado, não curado — unificar é escolher UMA régua para os dois nós, e o `period` do LFO
   está em cenas gravadas.

⛔ **Recusas medidas deste estudo:** os pinos `value`/`pulse` no `motion.oscillator` (§6) ·
ler o «22% contra 2%» da §2 como atraso nosso (47 dos 49 são subprodutos que a casa entrega
como colunas ou como tipos separados) · copiar os chips dele tal como estão (segunda fonte,
inferida por categoria — o doc 99 §3 já o mediu).
