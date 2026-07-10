# 06 — Pulse: o gatilho de 1ª classe (decisão + evidências)

**Data:** 2026-07-10 · **Linha:** `line/MotionNodes` · **Escopo desta wave:** o TIPO `pulse` +
o produtor `pulse.threshold` (Schmitt) + o consumidor visível `motion.strobe`. A família larga
(`on_change`, `compare`, `switch`, `counter`, `sample_hold`) fica para waves seguintes, agora
que a infra existe.

**Método:** pesquisa-antes-de-implementar (DIRETRIZ). 5 varreduras de fonte primária + o
MiniCavalryV2 clean-room. Fontes no rodapé.

---

## §1 A pergunta central: pulse é um TIPO, ou um `0/1` por convenção?

Aqui as referências **divergem**, e a divergência é a decisão:

- **Cavalry NÃO tem tipo de evento.** Tudo que flui num fio é um value contínuo; "discreto"
  é `1/0` fabricado por utilitários (Comparison: *"output a value of 1 when the result is true
  and 0 when false"*; If Else: *"True for any value ≥ 0.5"*; Logic AND/OR/XOR). O único evento
  real é interno à física (Collision Events), e mesmo esse é "aplique um efeito", não um sinal
  no fio. Varredura dos ~90 Behaviours + ~100 Utilities: **nenhum** nó "Signal/Trigger/Pulse/
  Gate/Event".
- **Rive TEM.** Um Trigger é um input **distinto** de Boolean e Number: no runtime,
  `StateMachineInputType.Trigger` *"has a `fire()` function"* e — ao contrário de Number/Boolean
  — **não tem `value`**. *"Triggers are similar to booleans, but can only become true for a
  short time."* O Unity runtime é o mais cru: *"SMITrigger is a boolean that is set to true for
  one frame by calling the `.Fire()` method"* — auto-reset, o dev nunca zera.
- **Max/Pd**: o evento é o `bang` — token puro "aconteceu", **sem payload**; nunca um tipo de
  fio próprio, sempre emitido por um detector de borda.

**Decisão: pulse é um TIPO** — `PortType(Instances, Scalar, Clock::Event)`. Seguimos Rive (que o
plano cita para "trigger de 1ª classe"), uma **departure deliberada de Cavalry**. E é barato:
o substrato **já** impõe `connects_directly` com o clock no tipo — um pulse `Event` não conecta
numa porta `Frame` por aresta plana (a travessia é uma membrana). Ou seja, o "Trigger ≠ Boolean"
do Rive não é convenção, é **erro de compilação** no PH2D. O type-system já queria isto.

## §2 O que um pulse carrega

| Opção | Fonte |
|---|---|
| Só "disparou" (1.0 no tick, sem payload) | Rive (`fire()` sem arg, sem `value`); Max `bang` (sem payload) |
| `{value, edge, t}` (nível + borda + tempo) | **MiniCavalry** — cada pulse carrega `edge ∈ {enter,exit,idle}`; consumidores checam `value>0.5 && edge==='enter'` |
| Envelope ADSR (shaped) | TouchDesigner Trigger CHOP |

**Decisão: só "disparou".** A coluna `pulse` (Scalar) vale **1.0 apenas no tick da borda de
subida**, 0.0 caso contrário. Isto é o Rive "true for a short time" / o Max rising bang.

Por que não o `{value,edge}` do MiniCavalry? Porque separar nível e borda em campos que o
consumidor precisa combinar (`value>0.5 && edge==='enter'`) é frágil: um consumidor que esqueça
o `&& edge` conta o nível sustentado como N disparos. No PH2D, **o produtor já garante que 1.0 =
borda** — o consumidor só lê "1.0 este tick? dispara". A detecção de borda é responsabilidade de
quem produz, via `pre` (comparar o estado latched do tick anterior). Payload (Unreal Notify /
Unity `AnimationEvent` / Rive Events) é um conceito **out-bound** separado; não é pulse.

## §3 Threshold: um limiar, ou dois (Schmitt)?

**Decisão: dois (Schmitt).** Um único limiar dispara sempre que o sinal treme perto dele — ruído
sozinho produz uma rajada de pulsos espúrios (Wikipedia: *"a noisy input signal near that
threshold could cause the output to switch rapidly back and forth from noise alone"*). Dois
limiares (`rise` > `fall`) dão a banda de histerese = **memória bistável**: uma vez disparado,
o sinal tem de cair abaixo do `fall` separado antes de re-armar.

Nomes e forma vêm direto das referências convergentes:

| Ferramenta | limiar de subida | limiar de descida | extra |
|---|---|---|---|
| **TouchDesigner** Trigger/Count CHOP | `threshup` | `threshdown` | `retrigger` (refractory), Trigger On (dir) |
| **Pure Data** `threshold~` | trigger | rest | debounce ms por borda |
| **Schmitt** (Wikipedia) | upper | lower | banda = upper−lower |

Uso `rise`/`fall` (legível) + um param `edge` de direção (Rise/Fall/Both — o "Trigger On" do TD,
os dois outlets do `edge~`). A memória bistável (o "possesses memory" do Schmitt) é a coluna
`armed` carregada no `pre` self-loop — **o mesmo mecanismo de nó sequencial** de
integrate/spring/trail. HR-5: só comparações, determinístico. O `retrigger`/debounce do TD/Pd
fica deferido (é refinamento; a histerese já mata o chatter).

## §4 O consumidor visível: `motion.strobe` (envelope)

Um pulse sem consumidor visível seria "unit-verde ≠ funciona no produto". O consumidor mais
**visual e canônico** é um envelope disparado pelo pulso:

- TouchDesigner: o Trigger CHOP *"starts an audio-style attack/decay/sustain/release (ADSR)
  envelope to all trigger pulses"*.
- Unreal: o **Notify State** é o Notify com duração — *"a begin, a tick, and an end"* (o
  impulso vira envelope).
- Max/Pd: o padrão clássico é o bang **retriggar** um `line~`/`adsr~`.

**`motion.strobe`** recebe um stream + um pulse e, a cada disparo, **acende** uma propriedade
(size/tint boost) que **decai geometricamente** ao longo dos ticks seguintes — attack instantâneo
+ decay exponencial, o ADSR mínimo. O decay é o mesmo geométrico-via-`pre` do `motion.trail`
(aplicado uma vez por tick à intensidade carregada → nunca compõe duas vezes). Reusa o padrão de
pareamento por `id` do integrate/trail. Resultado na tela: o foco **pisca** no ritmo do gatilho.

## §5 Determinismo — o que NÃO copio do MiniCavalry

O `stateMachine.js` do MiniCavalry, no modo `random`, usa `Math.random()` — não-determinístico,
proibido por HR-5. Todos os nós de estado dele guardam estado oculto por-nó (`node._smState`,
`node._counterValue`) e resetam em regressão de tempo (`time < _lastTime`). No PH2D o estado
viaja no stream via `pre` (visível, serializável, replay bit-exato) e qualquer aleatoriedade
seria `hash(seed, id, …)` (a lição do emitter). Esta wave não tem RNG; quando a família chegar
ao `stateMachine`, o `random` vira `hash`.

## §6 Resumo da decisão

| # | Decisão | Ref dominante |
|---|---|---|
| 1 | pulse é um **tipo** (`Instances,Scalar,Event`), não `0/1` | Rive (vs Cavalry) |
| 2 | carrega só "disparou" (1.0 no tick de borda), sem payload | Rive/Max |
| 3 | detecção de borda no **produtor**, via `pre` (não no consumidor) | (PH2D) |
| 4 | threshold **Schmitt** (`rise`>`fall`), memória no `pre` | Schmitt/TD/Pd |
| 5 | consumidor visível = **envelope** (strobe, decay geométrico) | TD/Unreal |
| 6 | zero RNG; futuro `random` = `hash`, não `Math.random` | HR-5 |

---

## Fontes primárias (2026-07-10)

- **Rive:** [inputs (Trigger vs Boolean vs Number)](https://rive.app/docs/editor/state-machine/inputs) ·
  [runtime web inputs (`fire()` vs `value`)](https://rive.app/docs/runtimes/web/inputs) ·
  [Unity state machines ("true for one frame")](https://help.rive.app/game-runtimes/unity/state-machines) ·
  [listeners (pointer→trigger)](https://rive.app/docs/editor/state-machine/listeners) ·
  [rive-events (payload out-bound)](https://rive.app/docs/runtimes/web/rive-events)
- **TouchDesigner:** [Trigger CHOP (ADSR, threshup/threshdown/retrigger)](https://docs.derivative.ca/Trigger_CHOP) ·
  [Count CHOP (ações por-borda, limit)](https://docs.derivative.ca/Count_CHOP)
- **Schmitt trigger:** [Wikipedia (dois limiares, histerese, memória)](https://en.wikipedia.org/wiki/Schmitt_trigger)
- **Unreal/Unity:** [Animation Notifies (Notify vs Notify State begin/tick/end)](https://dev.epicgames.com/documentation/unreal-engine/animation-notifies-in-unreal-engine) ·
  [Unity Animation Events (payload)](https://docs.unity3d.com/Manual/script-AnimationWindowEvent.html)
- **Max/Pd:** [Max `edge~` (rising/falling bang)](https://docs.cycling74.com/reference/edge~/) ·
  [Max `thresh~` (histerese)](https://docs.cycling74.com/reference/thresh~/) ·
  Pd `threshold~` help patch (`trigger,rest,debounce`) ·
  [Max `bang` (evento sem payload)](https://docs.cycling74.com/learn/articles/basicchapter02/)
- **Cavalry (contra-exemplo — sem tipo de evento):** [Comparison](https://cavalry.studio/docs/nodes/utilities/comparison/) ·
  [If Else](https://cavalry.studio/docs/nodes/utilities/if-else/) ·
  [Logic](https://cavalry.studio/docs/nodes/utilities/logic/) ·
  [Collision Events](https://cavalry.studio/docs/nodes/shapes/forge-dynamics/collision-events/)
- **MiniCavalryV2 (clean-room, comportamento):** `src/nodes/oscillator.js` (pulse zero-crossing,
  `{value,edge,t}`), `characters/stateMachine.js` (trigger `edge==='enter'`, `random`=`Math.random`),
  `utility/counter.js`, `DOCS/_NODES.md` (socket pulse = `{value, edge, t}`)
