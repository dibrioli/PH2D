# 08 — `pulse.counter`: o redutor de pulsos (a ponte Evento→Valor)

> **RENOMEADO (2026-07-10, handoff [09](09_handoff_pulse_signal_source_and_naming.md) §4.2):**
> o nó deste doc chama-se hoje **`motion.step`** (crate `ph2d-node-motion-step`, display
> "Step"). Motivo: o counter do MiniCavalry é redutor PURO (pulso→valor, não toca canal);
> o nosso **empurra um canal por batida** → é um behaviour visível (`motion.*`). O nome
> `pulse.counter` ficou LIVRE pro redutor puro, quando o domínio de valor existir (09 §4.3).
> A matemática, os modos e as decisões abaixo continuam válidos por inteiro.

> **Onda M2, família pulse (3/n).** Sequência: `pulse.threshold` (produtor) →
> `motion.strobe` (consumidor momentâneo) → **`pulse.counter` (redutor persistente)**.
> Estudo do padrão-ouro ANTES de implementar (método do Enio). Fontes primárias
> citadas; nenhuma inventada. Companheiro doc: [`06_pulse_gatilho_primeira_classe.md`](06_pulse_gatilho_primeira_classe.md).

## 0. A pergunta

Já temos um **produtor** de pulsos (`pulse.threshold`, Schmitt: um sinal cruza um
nível → dispara um gatilho de 1 tick) e um **consumidor** (`motion.strobe`: cada
pulso acende um flash que decai). Mas o pulso ainda é um **beco sem saída**: a
única coisa que ele faz é um flash momentâneo. Nada ainda:

1. **acumula** eventos num valor persistente e dirigível (contar batidas);
2. **guarda estado** entre batidas (liga/desliga, alterna);
3. **captura** um sinal na batida (sample-and-hold).

Qual é o próximo nó de **maior poder pelo menor custo** — o que transforma o pulso
de "efeito momentâneo" em "lógica de eventos componível"?

## 1. O que a indústria faz (matriz de referência)

Pesquisa em seis ecossistemas (Rive, Cavalry, Houdini CHOPs, TouchDesigner, Max/MSP,
vvvv, + Niagara). O achado estrutural: **o contador é o nó-mãe.** Toggle, sequência
e "quantas vezes aconteceu" são todos *vistas* de um contador (`& 1`, `mod N`, cru).
Por isso toda ferramenta madura entrega um contador de 1ª classe e deriva o resto
com módulo / `sel` / `route`.

| Conceito | Cavalry | Houdini | TouchDesigner | Max/MSP | vvvv |
|---|---|---|---|---|---|
| **contador** | Timeline Counter (Accumulate/Trigger) | **Count CHOP** | **Count CHOP** | **counter** (up/down/updown, carry) | accumulator |
| sample-and-hold | (valor mantido) | **Hold CHOP** | **Hold CHOP** | `sah~`, `zl reg` | **S+H** / HoldLatest |
| gate / switch | If Else | Switch / Select | Switch / Select | **gate** (1→N) / **switch** (N→1) | Switch |
| toggle | Boolean | Logic (Toggle) | **Logic → Toggle** | `toggle` / `counter … 2` | Toggle |
| on-change / edge | Comparison+change | Logic (crossing) | **Logic → Rising/Falling Edge** | `change`, `togedge` | Changed / TogEdge |
| compare | Comparison → Logic | Logic (thresholded) | Logic (convert modes) | `>` `<` `==`, `sel` | comparison |
| sequência | **Sequence** / Timeline Counter | Count+Loop → índice | Count+Loop → índice | `counter` + `sel` | ForEach |

**O Count CHOP (TouchDesigner/Houdini) é a referência canônica** e o template que
vamos seguir: conta quando um canal **cruza um limiar** (de ≤ para >), com um
**limiar de release** separado e um **Re-Trigger Delay** (histerese anti-duplo-conta);
a cada evento pode `±1`, `±tempo` ou **resetar**; entradas `{sinal, Reset, Increment}`.
O vocabulário de **limite/wrap** dele é provado e copiável: `Off`, `Loop Min/Max`
(wrap), `Clamp Min/Max`, `Zigzag Min/Max`. ([TD Count CHOP](https://docs.derivative.ca/Count_CHOP),
[Houdini Count](https://www.sidefx.com/docs/houdini/nodes/chop/count.html))

O **Max `counter`** acrescenta a saída de **carry/overflow** (emite um bang no limite)
— exatamente como se encadeiam contadores (segundos→minutos) ou se re-dispara algo
no wrap. ([Max counter](https://docs.cycling74.com/max8/refpages/counter))
O **Cavalry Timeline Counter** mostra os dois modos: **Accumulate** (+1 ao passar um
marcador) vs **Trigger** (1 enquanto no marcador). Rive **não tem** contador nativo —
incrementa um Number input via transições/data-binding.

## 2. A decisão: `pulse.counter` primeiro

**Ranking por poder/custo como PRÓXIMO nó** (relatório de pesquisa completo):

1. **`pulse.counter`** — o redutor universal de pulsos. É *auto-contido* (só precisa
   do pulso que já produzimos), é **inteiro/sem transcendentais** (HR-5 de graça:
   soma + módulo euclidiano), tem **um único inteiro de estado** no `pre`-loop, e
   **subsome três dos candidatos**: toggle = `count & 1`, sequência = `count mod N`,
   "quantas vezes" = o próprio count. Todo tool da indústria o entrega de 1ª classe.
   **É o único nó que destrava a maior parte da família de uma vez.**
2. **sample-and-hold / latch** — a ponte *complementar* (evento → *valor congelado de
   outro sinal*); par natural do threshold. Fica em #2 só porque seu payoff depende de
   haver um sinal que valha capturar, enquanto o contador é útil no instante em que
   existe um pulso. **Próximo follow-up nomeado.**
3. **gate / switch** — roteamento/ramificação; naturalmente #3, idealmente *depois* do
   contador (`counter mod N → switch` = sequência real).
4-7. **toggle** (=`counter & 1`), **sequência** (=`counter mod N → switch`), **compare**
   (utilitário puro), **on_change** (duplicata mais fraca do threshold que já temos).

### 2.1 Precisão: o contador NÃO é a "primeira ponte"

Cuidado com a alegação fácil. `motion.strobe` **já cruza Evento→Frame** — ele come um
`PULSE` (clock `Event`) e emite um stream `Frame`. Então o contador não é a *primeira*
travessia de membrana. Ele é o **primeiro REDUTOR**: a resposta do strobe é
*momentânea* (decai a cada tick, a envelope é o estado), a do contador é *persistente*
e *acumulada* (o inteiro sobrevive e cresce). Essa é a distinção que a demo mostra
lado a lado: mesmo pulso, um vira um **passo que fica**, o outro um **flash que some**.

## 3. Contrato do nó (o que vai ser construído)

Espelha `pulse.threshold`/`motion.strobe` (mesma família drop-crate isolada):

```
pulse.counter  (Effect::Pure, Clock::Frame)
  in    : (Instances, Vec2, Frame)   — o stream que passa e cujo canal é modulado
  pulse : (Instances, Scalar, Event) — o PULSE que conta (a membrana)
  state : (Instances, Vec2, Frame)   — pre self-loop: carrega `count_tick` + `count_prev`
  out   : (Instances, Vec2, Frame)   — `in` passado adiante + delta no canal + coluna `count`
params:
  channel   0 X · 1 Y · 2 Rotation · 3 Size     (default 0 = X)
  step      unidades somadas ao canal por contagem (default 0.5)
  count_max N contagens distintas do ciclo, ≥1    (default 6)
  mode      0 Wrap · 1 Clamp · 2 Zigzag           (default 0 = Wrap)
```

**Estado = um tick monotônico**, não a contagem dobrada. `count_tick` (f32, +1 por
borda de subida do pulso) e `count_prev` (o valor do pulso do tick anterior, p/
detecção de borda) viajam no `state`. A **contagem exibida** é derivada do tick +
modo a cada tick — assim os três modos caem de um só estado:

- **Wrap**: `tick mod N` (euclidiano) — a escada zera. (TD Loop Min/Max.)
- **Clamp**: `min(tick, N-1)` — a escada estaciona no topo. (TD Clamp.)
- **Zigzag**: triângulo de período `2(N-1)` — sobe e desce. (TD Zigzag.)

O deslocamento visível = `count_exibida · step`, somado ao canal escolhido (mascarado
por `falloff` se presente — 1.0 na ausência), via o `apply_channel_delta` compartilhado.
A coluna `count` (Scalar) sai no output como a **saída-redutor crua** que consumidores
futuros leem (índice, matiz, passo).

### 3.1 Correção (as armadilhas que a pesquisa apontou)

- **Duplo-conta em pulso mantido:** conta **só na borda de subida** (`pulse>0.5 &&
  prev<=0.5`), nunca "enquanto alto". O `pulse.threshold` já garante pulso de 1 tick,
  mas o contador é edge-safe *independente* do produtor (robustez p/ pulsos futuros
  estilo Cavalry 0/1). Essa é a distinção TD `Off to On` vs `While On`.
- **Wrap por módulo inteiro** (euclidiano, `rem_euclid`), nunca float — exato e sem
  transcendentais.
- **`count_max` degenerado:** clampado a ≥1 no eval (N=1 → contagem sempre 0, sem
  divisão por zero nem período de zigzag zero).
- **Determinismo:** estado inteiro puro no `pre`-loop → replay-hash estável.
- **Reset como INPUT** (2º pulso, TD Count) fica **deferido**: exige tolerar porta
  opcional desconectada (validate rejeita input faltante). Em v1 o **wrap** É o reset
  cíclico; reset-por-evento é follow-up nomeado.
- **Carry-out pulse** (Max, re-dispara no overflow / encadeia contadores) fica
  **deferido**: exige 2ª porta de saída `Event`. Follow-up nomeado.

## 4. A demo (ready-to-smoke)

Estende a **cena pulse-loop** (a 3ª cena do documento default, `motion_demo_strobe.rs`).
O MESMO pulso do threshold alimenta AGORA dois consumidores em leque:

```text
grid → move → tint → counter → strobe → output
                 clock(osc rot) → threshold ⟲ → { counter.pulse, strobe.pulse }
                 counter.out --pre--> counter.state
                 strobe.out  --pre--> strobe.state
```

- **counter**: canal X, `step 0.5`, `count_max 6`, modo **Zigzag** → a cada batida a
  grade **desliza um passo** na horizontal e, após 6 passos, volta — um *sweep de
  sequenciador*. O passo **fica** entre as batidas.
- **strobe**: no mesmo pulso, acende o flash que **decai**.

Resultado: a grade varre em passos discretos, **piscando em cada passo** — a diferença
entre estado persistente (contador) e envelope momentâneo (strobe), dirigidos pelo
mesmo gatilho. É o menor laço fechado que prova o **redutor** de pulsos.

## 5. Follow-ups nomeados

- `motion.latch` (sample-and-hold, rank #2): `{value, pulse}` → congela o valor na
  borda; Hold CHOP semantics (Off-to-On default, `hold-last` + `initial`).
- `pulse.gate`/`switch` (rank #3): rotear stream por índice pulso-avançado
  (`counter mod N → switch` = sequência).
- Contador: **reset input** (2º pulso) · **carry-out pulse** (encadeia/re-dispara) ·
  **direção down** (hoje `step` negativo já dá o visual descendente).

## Fontes

- TouchDesigner — [Count CHOP](https://docs.derivative.ca/Count_CHOP) · [Hold CHOP](https://docs.derivative.ca/Hold_CHOP) · [Logic CHOP](https://docs.derivative.ca/Logic_CHOP)
- Houdini — [Count CHOP](https://www.sidefx.com/docs/houdini/nodes/chop/count.html) · [Hold CHOP](https://www.sidefx.com/docs/houdini/nodes/chop/hold.html)
- Max/MSP — [counter](https://docs.cycling74.com/max8/refpages/counter) · [gate](https://docs.cycling74.com/max8/refpages/gate) · [sah~](https://docs.cycling74.com/reference/sah~/)
- Cavalry — [Timeline Counter](https://cavalry.studio/docs/nodes/utilities/timeline-counter/) · [Sequence](https://docs.cavalry.scenegroup.co/nodes/utilities/sequence/) · [Logic](https://cavalry.studio/docs/nodes/utilities/logic/)
- vvvv — [Data Flow (S+H, Toggle, Changed, TogEdge)](https://thegraybook.vvvv.org/introduction/lo_2_dataflow.html)
- Rive — [State Machine inputs (trigger/boolean/number)](https://help.rive.app/editor/state-machine/inputs) · [Data Binding](https://rive.app/docs/editor/data-binding/overview)

**Status:** IMPLEMENTADO nesta onda (crate `ph2d-node-pulse-counter`).
