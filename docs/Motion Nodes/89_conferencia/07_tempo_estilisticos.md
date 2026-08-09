# 07 — TEMPO / ESTILÍSTICOS (5 nós) — conferência contra a referência

**Família 7 do [plano 89](../89_plano_conferencia_dos_nos.md) §2** · `motion.trail` · `motion.delay` ·
`motion.strobe` · `motion.step` · `motion.morph` · **Data:** 2026-08-09 · **Status:** conferência,
nada implementado, nenhuma prioridade final decidida (§9 lei 9).

**Params lidos do `MANIFEST`** de cada crate, não do doc. **Referências:** AE (Echo · Posterize
Time · CC Wide Time · Strobe Light) · [`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) ·
[`referencia_catalogo_nodes_minicavalry.md`](../referencia_catalogo_nodes_minicavalry.md) ·
[`referencia_pesquisa_c4d_fields.md`](../referencia_pesquisa_c4d_fields.md) ·
[`referencia_pesquisa_houdini_mops.md`](../referencia_pesquisa_houdini_mops.md) ·
[`referencia_pesquisa_niagara_stardust.md`](../referencia_pesquisa_niagara_stardust.md).

---

## §0 — O FATO QUE DECIDE A FAMÍLIA (leia antes da tabela)

O brief aponta o eixo de superação para **o tempo por sub-árvore** (*"um Echo do AE tem um relógio;
o nosso trail vive num grafo onde cada ramo pode ter o seu"*). ⚠️ **Medido no substrato, a resposta
hoje é NÃO — e por uma recusa explícita**, não por acaso:

```rust
// crates/ph2d-nodegraph/src/cook.rs:580-584
// A recurrence over the outer tick cannot run on a rewritten clock
if consumes_pre && key != SCOPE_ROOT {
    return Err(CookError::SequentialInTimeScope { node });
}
```

O motivo está escrito na linha 255 do mesmo arquivo (*"sob `Loop` ele reviveria ticks que já
integrou; sob `Freeze` avançaria com o tempo parado"*). **Quatro dos cinco nós desta família
consomem `pre`** (`trail`/`delay`/`strobe`/`step`) ⇒ **nenhum deles pode viver num ramo com relógio
próprio**; o `motion.trail` até documenta a recusa no próprio header. O único da família que entra
num escopo é o `motion.morph` (Pure).

Isso não mata o eixo — ele **muda de forma**, e vira o `SUPERAR:` S1 desta folha: o caminho não é
*pôr o rastro num escopo*, é **inverter o rastro** para que ele não precise de estado.

Um segundo fato transversal: **os cinco nós são `LoweringKind::Cpu`**. Nenhum tem kernel de GPU.

---

## §1 — A TABELA

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.trail` | **7** (`length 8`·`fade .10`·`shrink .65`·`spacing 1`·`hue_shift 0`·`saturation 1`·`spin 0`), 7 hints, 7 unidades, 2 seções · **não lê `falloff`** | **máscara por CAMPO** — todo effector do C4D tem Strength + Fields ([c4d_fields §Effector base](../referencia_pesquisa_c4d_fields.md), "Use Alpha/Strength"); doc 63 §3.1 cluster **STRENGTH/weight**: *"todo modificador multiplicável por campo de peso"* | **NÃO** — o nó não lê a coluna. Tentado `field.box → motion.cull → trail`: o cull remove a LINHA do stream inteiro (some o elemento, não só o rastro dele). Tentado `trail → field.*`: o campo pousa depois, e não há como distinguir eco de cabeça a jusante (`trail_age` existe, mas nenhum nó a lê como máscara) | **omissão** — é o ÚNICO behaviour da família sem falloff | **P0** | `falloff` ausente ⇒ `1.0` (a lei do `falloff_at` de `delay`/`step`/`strobe`) |
| `motion.trail` | idem | **eco para a FRENTE**: AE **Echo** *Echo Time* **positivo = ecos de frames FUTUROS**; AE **CC Wide Time** *Forward Steps* + *Backward Steps* | **NÃO neste nó** (o estado É o passado, por construção). **PARCIAL fora dele** para sub-árvore PURA: `time_remap(offset > 0)` cozinha o futuro — ver `SUPERAR:` S1 | omissão (estrutural) | **P1** | `forward = 0` ⇒ só passado, o de hoje |
| `motion.trail` | idem | **Echo Operator** (AE): o modo de composição de cada eco (Add/Screen/Maximum/…) — o look canônico de rastro de LUZ | **NÃO** — hoje é sempre alpha-over com a cauda atrás (`concat(carried, head)`). O blend mode vive no renderer (`lower_to_instances`), não há coluna de convenção para ele (≠ `texture_id`/`geometry_id`, que existem) | omissão | **P1** | operador `Over` ⇒ o `concat` de hoje |
| `motion.trail` | idem | **`includeOriginal`** e **`opacityMax`/`satMax`** ([minicavalry §motionTrail](../referencia_catalogo_nodes_minicavalry.md): `length 12`·`spacing 2`·`scaleFade .6`·`includeOriginal true`·`opacityMax/Min 1/0`·`satMax/Min 1/1`) — a cabeça viva é sempre desenhada e sempre a 1.0 | **PARCIAL** — `trail → motion.tint` escurece tudo, **inclusive a cabeça**; o alvo do eco mais NOVO é inalcançável (só o mais VELHO é autorado) | omissão | **P2** | `include_original = on`, `alpha_max = 1` ⇒ hoje |
| `motion.trail` | idem | **Path Type: linha contínua / ribbon** ([cavalry §Trails](../referencia_pesquisa_cavalry.md): *Path Type: Béziers/Lines*); [niagara §113](../referencia_pesquisa_niagara_stardust.md): *"Ribbon/Strip renderer — **PARCIAL** (`motion.trail` = eco; ribbon contínuo?)"*; doc 63 §3.2 linha 200 | **NÃO** — não existe primitiva de fita no motion; a saída é sempre N instâncias | **natureza** (falta a primitiva, não o knob) — mecanismo: o stream é `(Instances, Vec2)`, uma fita é geometria | **P2** | — (não é param) |
| `motion.trail` | idem | **compute velocity** ([houdini_mops §65](../referencia_pesquisa_houdini_mops.md): *"Trail — rastro/eco + **computa `v` por diferença** entre frames; o 'compute velocity' como serviço FALTA"*) | **NÃO** — o rastro tem `P` de dois ticks na mão, mas não há subtração de streams: `motion.mixer` Add SOMA, `value.math` opera na coluna `v` (valor), não em `P`. Tentado `delay(Delay,1) + mixer(Add)` ⇒ dá a soma, não a diferença | omissão (barato: já tem os dois lados no mesmo tick) | **P1** | coluna `v` ausente hoje ⇒ escrevê-la é aditivo |
| `motion.trail` | idem | ⚠️ **`MAX_INSTANCES = 65_536` é um teto NÃO-MEDIDO** — CLAUDE.md §0. O comentário justifica com *"4096 vivas × 32 ecos já é 131k quads"*; a `line/gpu-nodes` **mediu 4,19 M partículas em 3,6 ms na GPU** | n/a (é medição, não composição) | **omissão** — o teto do rastro é o do caminho lento (CPU), no nó cujo custo é multiplicativo | **P1** | o número que a MEDIÇÃO der |
| `motion.delay` | **2** (`mode 2=Blend` Enum[Delay/Average/Blend] · `ticks 8`) · lê `falloff` + declara `Coupling` · **sem `ParamUnit`** | **atrasar mais que a POSIÇÃO**: C4D Delay Effector ([OEDELAY](https://help.maxon.net/c4d/en-us/Content/html/OEDELAY.html), citado em [c4d_fields §127](../referencia_pesquisa_c4d_fields.md)) — *"…with regard to **position, scale and rotation**"*; o **Delay FIELD** ([FLDELAY](https://help.maxon.net/c4d/en-us/Content/html/FLDELAY.html), §163) inclui *"**color** changes"* | **NÃO** — o nó tem um canal só (`P`) e um segundo `motion.delay` a jusante volta a atrasar `P`. A cura já tem vocabulário no repo: o enum `channel` X/Y/Rotation/Size de `step`/`noise`/`oscillator` | **omissão** | **P0** | `channel = Position` ⇒ o de hoje |
| `motion.delay` | idem | **attack ≠ release** — TD **Lag CHOP** (a referência que o próprio `value.smooth` cita: *"O Filter/Lag CHOP de TouchDesigner"*): *Lag up / Lag down* separados | **NÃO** — um `ticks` só governa os dois sentidos | omissão (barato) | **P1** | `ticks_down = ticks_up` ⇒ simétrico, o de hoje |
| `motion.delay` | idem | **Strength [%]** global (C4D Delay: *"quanto do passado segura"*) | ⛔ **REFUTADO** — o `falloff` já É isto, por elemento (`lerp(live, delayed, falloff_at(i))`), e um `field.*` de valor constante dá a versão global | — | ⛔ | — |
| `motion.delay` | idem | **delay em CASCATA por índice** ([minicavalry §delay](../referencia_catalogo_nodes_minicavalry.md): `perIndex 0.05s`) e **por falloff** ([MOPs Delay §115/§308](../referencia_pesquisa_houdini_mops.md)) | ⛔ **REFUTADO** — é o `motion.slit_scan` (`lag · i/(n−1)`, e o `falloff` dele atenua o delay). A tabela do próprio `motion.delay` já mapeia isso, linhas 17-23 | — | ⛔ | — |
| `motion.delay` | idem | **modo Spring** (C4D tem os três num nó) | ⛔ **REFUTADO** — `motion.spring`, e o header do nó declara: *"we already have a spring and it is a better one"* | — | ⛔ | — |
| `motion.delay` | idem | `ticks` **sem `ParamUnit::Count`**, enquanto `trail.spacing` e `strobe.decay` — a MESMA grandeza — declaram (doc 88, lei da unidade) | n/a | omissão (higiene) | **P2** | a unidade não muda o número |
| `motion.strobe` | **6** (`decay 34` "Flash Length" ticks·`size_boost .8`·`flash_r/g/b`+`flash_amount` num `ParamWidget::Color`) · lê `falloff` · **NÃO declara `Coupling`** | **ATTACK** — MOPs/Houdini **Trigger** ([houdini_mops §82/§287](../referencia_pesquisa_houdini_mops.md)): envelope *Delay→**Attack**(len+shape)→Peak→Decay→Sustain(level)→Release(len+shape)* + **retrigger delay** | **NÃO** — `glow_of` crava `1.0` no pulso; o envelope sobe em 1 tick sempre. O nó se auto-declara *"the minimal ADSR"* | omissão | **P1** | `attack = 0` ⇒ subida instantânea, a de hoje |
| `motion.strobe` | idem | **HOLD/SUSTAIN** (o platô antes de decair) — mesma fonte | **NÃO** | omissão | **P1** | `hold = 0` |
| `motion.strobe` | idem | **forma do decay** (hoje exponencial fixo; o Trigger tem *shape* por trecho) | **PARCIAL, a 3 nós** — o `glow` SAI como coluna, então `value.attribute("glow") → value.curve → motion.drive` reconstrói a curva. Três nós para um knob = o critério P1 da §7 | omissão | **P1** | curva não-setada = identidade (a lei do `value.curve`) |
| `motion.strobe` | idem | **Random Strobe Probability** (AE **Strobe Light**: *Strobe Duration · Strobe Period · **Random Strobe Probability** · Strobe Color · Blend With Original · Strobe Operator*) | **NÃO** — tentado `pulse.beat → pulse.sample_hold(aleatório) → pulse.compare(rise) → strobe.pulse`: o `pulse.compare` é de **BORDA com histerese**, então dois sorteios altos consecutivos disparam **UMA** vez ⇒ a probabilidade sai errada. Não há nó que multiplique um PULSO por uma máscara | omissão | **P1** | `probability = 1` ⇒ todo pulso acende |
| `motion.strobe` | idem | **Strobe Operator / Blend With Original** (AE) | **NÃO** — mesma causa do Echo Operator do trail (o blend é do renderer). **Um conserto serve os dois nós** | omissão | **P2** | operador `Mix` ⇒ o lerp de hoje |
| `motion.strobe` | idem | **Strobe Period / Strobe Duration** (AE) | ⛔ **REFUTADO** — `pulse.beat(period, offset)` dá o período e `Flash Length` a duração; e o nosso é **melhor** por ser dirigido por PULSO (o mesmo strobe aceita `pulse.threshold`, `pulse.compare` ou uma colisão do `sim.collide`) | — | ⛔ | — |
| `motion.strobe` | idem | ⚠️ **defeito de side-metadata, não de param:** o nó **lê `falloff`** (`step`, linha 159) e **não** chama `register_couplings(Consumes("falloff"))`, enquanto `delay` e `step` chamam — com o comentário ADR-0155 idêntico (*"CPU-only … o diagnoser não consegue derivar o papel de um `ColumnBinding` — declare"*) | n/a | **omissão** (achado da conferência; ~4 linhas) | **P1** | declarar não muda nenhum pixel |
| `motion.step` | **4** (`channel` Enum[X/Y/Rot/Size]·`step .5`·`count_max 6`·`mode` Enum[Wrap/Clamp/Zigzag]) · lê `falloff` + declara `Coupling` · sem unidades | **RESET** — TD **Count CHOP** (a referência nomeada no próprio header): entrada de *Reset* + *Reset Condition* + *Reset Value*; [minicavalry §counter](../referencia_catalogo_nodes_minicavalry.md) e §stateMachine: *"Inputs: trigger(pulse); **reset(pulse)**"* — **os dois têm** | **NÃO** — o `count_tick` monotônico vive no `pre` self-loop e **nada mais escreve nele**; baixar `count_max` só re-dobra o mesmo tick. Não há nó que zere o estado de outro | **omissão** | **P0** | 4ª porta `reset: PULSE` vazia ⇒ byte-idêntico |
| `motion.step` | idem | **Increment ≠ 1** e **Limit Min ≠ 0** (TD Count CHOP) | Increment: **PARCIAL** — `step` escala o DESLOCAMENTO, mas a coluna `count` publicada (que `value.attribute("count")` lê) anda de 1 em 1; escalá-la custa `value.attribute → value.gain`. Limit Min: **NÃO** | omissão | **P2** | `increment = 1`, `min = 0` |
| `motion.step` | idem | **contar para BAIXO** (TD tem uma 2ª entrada de count-down) | **PARCIAL** — `step` negativo desce o DESLOCAMENTO, mas o `count` publicado continua subindo ⇒ quem consome o índice (`value.attribute("count") → value.switch`) não desce | omissão | **P2** | direção `Up` |
| `motion.step` | idem | ⚠️ **limitação estrutural AUTO-DECLARADA** (header, linhas 38-41): pareamento **POSICIONAL** — *"uma mudança de CONTAGEM desalinha as linhas … uma stream que roda (o emitter) dessincroniza um beat 'global'. Id-keyed pairing … é o follow-up v2"* | **NÃO** contornável no grafo | omissão — e a cura já existe no repo (`motion.integrate`/`motion.spring` pareiam por id) | **P1** | pareamento por id é byte-idêntico onde a contagem é estável |
| `motion.step` | idem | `count_max` **sem `ParamUnit::Count`** (doc 88) | n/a | omissão (higiene) | **P2** | — |
| `motion.morph` | **0** — o `blend` é PORTA de valor (animável) | ⚠️ **o nó emite APENAS `P`**: `ctx.emit(Stream::new(n).with("P", …))` descarta `size`·`tint`·`rot`·`id`·`uv_rect`·**`texture_id`**·**`geometry_id`**. Referência: [Cavalry Blend Shape/Morph §56](../referencia_pesquisa_cavalry.md) e [Houdini Blend Shapes §66](../referencia_pesquisa_houdini_mops.md) interpolam a GEOMETRIA inteira; o irmão `motion.mixer` do nosso repo **"reduz toda coluna presente em todas as entradas"** | **NÃO** — o descarte é do nó; a jusante dá para RE-pintar, nunca para recuperar identidade. ⚠️ Consequência medível: morfar dois `source.object`/`source.shape` **perde a aparência** (as convenções `texture_id`/`geometry_id` do doc 86 / ADR-0154 somem, e o fallback é a tile 0) | **omissão** — não é param, é contrato de stream | ✅ **FEITO** (era P0 DEFEITO) | interpolar/propagar as colunas **é** o que a ausência delas significava (o `min` de comprimento não muda) |
| `motion.morph` | idem | **N formas com PESO** ([Houdini Blend Shapes §66](../referencia_pesquisa_houdini_mops.md): *"interpola N formas com pesos"*) | **PARCIAL** — `motion.mixer` tem 4 entradas, mas `Blend` dele é lerp de DUAS e `Avg` é média SEM pesos; `morph(morph(a,b,t1),c,t2)` funciona geometricamente mas os `t` **compõem** (não são pesos independentes). ⚠️ **A casa é o `motion.mixer`**, e o doc 63 §3.2 linha 205 já pede "peso POR entrada" lá — **não duplicar no morph** | omissão (de outro nó) | **P2** | peso ausente ⇒ média, a de hoje |
| `motion.morph` | idem | **morph guiado por CAMPO** ([c4d_fields §130, Inheritance](../referencia_pesquisa_c4d_fields.md): *"Morph entre dois arranjos MoGraph (origem→destino guiado pelo campo)"*) | ⛔ **REFUTADO** — `field.box → value.attribute("falloff") → morph.blend` fecha: o `value.attribute` lê qualquer coluna nomeada (doc 50) e emite VALUE, que é o tipo da porta `blend`, e o `blend` já é **por elemento** | — | ⛔ | — |
| `motion.morph` | idem | **easing** e **modo `switch`** ([minicavalry §morph](../referencia_catalogo_nodes_minicavalry.md): `t`·`easing`·`mode vertex/switch/crossfade`·resolution·threshold) | ⛔ **REFUTADO** — easing: `value.lfo → value.curve → morph.blend` (curva arbitrária, `ParamWidget::Curve`, LUT no device); switch: `value.step`/`value.quantize` no mesmo fio | — | ⛔ | — |
| `motion.morph` | idem | pareamento por ROW ORDER + `min` de comprimento (C4D Inheritance tem *Step Gap* para o caso) | ⛔ **RECUSADO COM MOTIVO** — é a convenção Sequence-Blend do repo, a mesma do `motion.mixer` (*"o rabo do input mais longo é descartado"*), declarada no header do nó. Mudá-la num nó só criaria duas leis de pareamento | **natureza** | ⛔ | — |

---

## §2 — `SUPERAR:` (derivado do que só nós temos)

**S1 — O ECO SEM MEMÓRIA (`motion.trail`, modo `Source: Remembered | Resampled`).**
Em vez de **lembrar** onde o elemento esteve, **re-cozinhar a sub-árvore de entrada em `t − k·spacing`** —
que é exatamente o que `Cook::cook_scoped` + `motion.time_remap(offset)` já fazem, e o que a tabela do
próprio `motion.delay` chama de *"exact, stateless, scrub-perfect, free"*. O que isso destrava e
**nenhuma referência tem**:
- **eco para a FRENTE** — AE *Echo Time* positivo e *CC Wide Time · Forward Steps* existem porque a AE
  **re-renderiza**; um ring de estado não consegue por construção, e o nosso escopo de tempo consegue;
- **`length` sem teto de memória** (o que sobra é orçamento de instância, e ele **precisa ser medido** — ver a linha do `MAX_INSTANCES`);
- **espaçamento NÃO-UNIFORME** dirigido por curva (param dirigido, doc 58): ecos que se adensam perto da cabeça;
- **scrub exato sem depender do `CheckpointRing`** — um eco puro é função do playhead.
⚠️ **O limite honesto já está escrito no repo** (`motion.delay`, linhas 27-29): *"`time_remap` cannot
delay a **simulation**: a sim is not a function of `t`"* ⇒ é um **MODO**, nunca uma substituição, com
`Remembered` como default (reduz literalmente).
⚠️ **E o custo prova que o item é P1 e não P2:** hoje isso é exprimível à mão —
`4 × time_remap + 4 × tint + 4 × scale + combine` = **13 nós para um eco de 4**, e `motion.combine`
tem 4 entradas, então `length 8` pede combines aninhados.

**S2 — `motion.time_remap` ganha o modo `Quantize` ("anima em 2s") — adjacência, casa em OUTRA família.**
A AE resolve isso com um efeito próprio (**Posterize Time**, param único *Frame Rate*) que trava a
**CAMADA** num frame rate menor. `t' = floor(t·r)/r` é **um braço a mais no enum de modos** de um nó
que já existe — e como o nosso escopo é por **SUB-ÁRVORE**, dá o fundo em 24, o personagem em 12 e o
efeito em 60 **no mesmo documento e no mesmo playhead**. Nenhuma referência faz isso por ramo (a AE
faz por camada; o C4D não faz). ⚠️ O `motion.time_remap` é da **família 6**: reportado como
adjacência para não nascer duplicado.

**S3 — A FAMÍLIA JÁ SUPERA NUM PONTO, e ele não está escrito em lugar nenhum.**
A gotcha do catálogo de origem para o `motionTrail` é literalmente **"Reseta em time-scrub"**
([minicavalry §motionTrail](../referencia_catalogo_nodes_minicavalry.md)), e a Cavalry Trails precisa
de um *Start Frame*. O nosso rastro **rebobina bit-exato** (M2.N2: `Cook::checkpoint`/`restore` +
`CheckpointRing`, GGPO). É uma vantagem **entregue e não-nomeada** ⇒ deveria virar **cena de smoke**
(arrastar a régua para trás e para a frente sobre um rastro de 32 ecos: o desenho tem de ser
idêntico), porque é precisamente o tipo de propriedade que uma wave futura quebra sem perceber.

---

## §3 — `CERCAS:` (grepadas antes de propor — não derrubar por engano)

1. **`motion.strobe` · `flash_amount` sem row própria** — é o **4º canal** do `ParamWidget::Color`, e o
   kernel faz `a = flash_amount · glow` compondo por `over`: ele **É** o alfa da mistura. O bridge de
   params **SUPRIME** a row de qualquer param dobrado num grupo Color (`motion_bridge_params.rs`,
   `consumed`) ⇒ uma row dedicada seria **código morto**. O arquivo já explica (linhas 289-293).
2. **TICKS e não segundos**, nos dois nós que decaem — `ctx.dt()` é **`0.0` dentro de um time scope**, e
   um `dt` zero faria a taxa virar `1.0`, isto é *"o flash nunca apaga"* (`decay_per_tick`, linhas
   132-137). A escolha é sobre RISCO, não sobre gosto.
3. **`motion.trail` · os cinco knobs de decaimento são ALVOS NA PONTA da cauda, não taxas** — a forma
   anterior (o knob *era* a taxa) foi **REPROVADA no smoke de 2026-08-08**, com medição: a faixa útil
   cabia em **5,2%** do curso e o `spacing` **multiplicava** todo decaimento (`fade 0.80` dava 0.21 no
   default e **0.0010** em `length 32`). Não propor voltar.
4. **`motion.strobe` · o param mantém o nome de fio `decay`** (o rótulo é "Flash Length") de propósito:
   renomeá-lo faria o `validate` **recusar todo grafo salvo** que o sobrescreve — a cicatriz do
   `motion.color_ramp` na integração de 2026-07-30.
5. **`strobe`/`step` · o `falloff` mascara a APLICAÇÃO, nunca a MEMÓRIA** (o envelope e a contagem
   seguem correndo), para que um campo animado sobre um flash vivo o **apague sem retriggerar**.
6. **`motion.delay` · a tabela "quem já faz o quê"** (header, linhas 17-23) é uma cerca inteira:
   `time_remap`/`trail`/`slit_scan`/`spring` cobrem quatro pedidos vizinhos, e o nó existe para o
   quinto — **lag SEM overshoot**. Quatro dos gaps "óbvios" desta família morrem nela.
7. **`motion.step` · o nome `pulse.counter` foi deixado LIVRE de propósito** para o redutor puro
   (doc 09 §4.3) — não colonizar.
8. **`motion.morph` · zero params é DECISÃO**: o `blend` é porta de valor para poder ser animado.
   Não converter em param.
9. **`motion.trail` · `materialize_render_columns` parte de `SIZE_IDENTITY`/branco opaco** — é o fix do
   smoke de 08/08 (*"Fade e Shrink não têm efeito algum"*): num stream posicional puro as colunas não
   existiam e os dois knobs eram **no-ops silenciosos**. Não "otimizar" removendo.
10. **`step`/`strobe` · aplicam ao `in` FRESCO, nunca ao estado** — gates
    `the_displacement_never_compounds_across_ticks` e `the_size_boost_does_not_compound_across_ticks`.
11. **`motion.trail` · a janela é `(k−1)·s + 1`, não `k·s`** — um `k·s` ingênuo deixaria passar um
    fantasma A MAIS e `length` passaria a significar duas coisas conforme o espaçamento.

---

## §4 — `O DOC 63 ERROU EM:`

1. **§3.2 linha 200 — `motion.trail` (length·fade·shrink)** → **ENVELHECEU**: hoje são **sete** params
   (`+spacing`, `+hue_shift`, `+saturation`, `+spin`), com unidades, seções e alvos-na-ponta (wave do
   doc 88, 2026-08-08). Dos três itens que ele manda ADICIONAR, **`time offset` está ENTREGUE** (é o
   `spacing`) — a linha manda construir o que existe. Sobram *limite por tempo* e *path type/ribbon*.
2. **§3.2 linha 196 — `motion.spring` | ADICIONAR: "modos Average/Blend/Spring (o Delay do C4D
   generalizado)"** → **ENTREGUE, NOUTRO NÓ**: é o `motion.delay` (Delay/Average/Blend), criado
   **depois** do doc 63 (`23b1efb8d`, cujo próprio commit cita o doc 63). O item está fechado e a
   tabela ainda o pede no `motion.spring`.
3. **§2.6 linha 145 — `motion.sequencer` P1 *"(step/strobe cobrem metade)"*** → a metade que falta é
   **nomeável agora**: nem `step` nem `strobe` escrevem **visibilidade**; e a metade que eles cobrem
   depende de o `motion.step` ter um **reset**, que ele **não tem** (P0 desta folha).
4. **Quatro dos cinco nós desta família não aparecem na §3.2** (`delay`, `strobe`, `step`, `morph`) —
   a tabela endereça 18 dos 87 nós que ela diz cobrir. Não é erro de conteúdo: é exatamente o buraco
   de cobertura que a §0 do doc 89 nomeia, e por isso a conferência **começa** do doc 63 sem confiar nele.
5. **§2.4 linha 124 — `pulse.adsr` P1 (nó NOVO)** → o **consumidor de envelope já existe** e se
   auto-declara *"the minimal ADSR"* (`motion.strobe`). *Upgrade do strobe* × *nó novo* é escolha de
   produto; o doc só oferece a segunda, e as duas dividiriam a mesma lei de envelope.

---

## §6 — O que a W4 fechou (2026-08-09)

### O primeiro P0 — **o morph descartava o stream inteiro menos `P`**

`ctx.emit(Stream::new(n).with("P", …))` jogava fora `size` · `rot` · `tint` · `id` ·
`uv_rect` · **`texture_id`** · **`geometry_id`**. ⚠️ **Não era param faltando, era perda de
dados**, e a consequência estava medida: morfar dois `source.object` **perdia a aparência**
(as convenções do doc 86 / ADR-0154 sumiam e o lowering caía na tile 0 — quads brancos).

**A lei que ficou:** as quantidades que o lowering desenha (`P` · `size` · `rot` · `tint`)
**desvanecem**; toda outra coluna é carregada pelo **VIZINHO MAIS PRÓXIMO**, por elemento.

- ⚠️ **A lista é BRANCA, e a assimetria é o argumento inteiro.** As duas listas possíveis
  apodrecem, mas o modo de falha é oposto: uma lista NEGRA (*"não interpole os `_id`"*)
  medeia em silêncio a coluna de identidade que alguém acrescentar amanhã — e o lowering lê
  `texture_id` com `as u32`, então a média de 0 e 7 vira a textura **3**, uma que ninguém
  pediu; a lista BRANCA faz a quantidade nova **PULAR** em vez de desvanecer, o que se vê e
  se conserta numa linha. *Escolha a lista cujo apodrecimento se VÊ.*
- ⚠️ **`uv_rect` fica fora dela de propósito** — ele nomeia QUAIS pixels junto com o
  `texture_id`, e interpolá-lo varre o átlas. Uma regra por SUFIXO `_id` o teria perdido, e
  é por isso que a régua não é o nome.
- ⚠️ **O vizinho próximo não é gosto, é o contrato do próprio nó:** o doc dele promete
  *"`blend = 0` is all `a`, `1` all `b`"*, e segurar `a` faria `blend = 1` **não** ser `b`.
  O corte no meio é o preço honesto de uma identidade não desvanecer.
- ⚠️ **A resposta do irmão `motion.mixer` não servia inteira:** ele reduz toda coluna
  presente em todas as entradas — **numericamente**, `texture_id` incluso. Copiá-lo teria
  trocado o descarte por uma média silenciosa.

⚠️ **E a wave achou um espelho:** a `fn morph` (a especialização `Vec2`) ficou **sem
chamador de produto**, e os quatro gates de crossfade que já existiam chamavam ELA — eles
teriam seguido verdes enquanto o `morph_stream` quebrava. Agora entram pela mesma porta que
o `eval`, e a função morreu.

**5 gates novos, 2 mutações, 2 sangram** (reinstalar o descarte derruba os cinco; pôr o
`texture_id` na lista branca dá textura **1,4 e 5,6** — que o `as u32` trunca em 1 e 5).

**Seguem P0 nesta família:** a máscara por campo do `motion.trail` (o único behaviour da
família que não lê `falloff`) · o canal do `motion.delay` (hoje só atrasa `P`) · e o
**reset** do `motion.step`.

---

## §5 — O que esta folha NÃO decidiu

Prioridade final, ordem de wave e implementação (plano 89 §9, lei 9). Os `P` da tabela são a leitura
da régua da §7 do plano aplicada a cada item, não a ordenação global.

**Fatos que a §5 (verificação do Enio) deveria conferir primeiro**, por serem os que viram trabalho:
o `motion.trail` de fato não lê `falloff` (grep) · o `motion.morph` de fato emite só `P`
(`lib.rs:115`) · o `motion.step` de fato não tem porta de reset (`MANIFEST.inputs`, 3 portas) · o
`motion.strobe` de fato não chama `register_couplings` · e o `MAX_INSTANCES = 65_536` de fato não tem
medição ao lado.
