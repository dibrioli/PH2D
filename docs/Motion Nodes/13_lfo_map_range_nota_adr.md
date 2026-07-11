# 13 — Nota-ADR: LFO + Map Range (fatia 2 do domínio de VALOR) — follow-up do doc 12

**Data:** 2026-07-10 · **Linha:** `line/MotionNodes` · **Status:** fatia 2 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates sobre o tipo de valor que o doc 12
introduziu (`PortType(Instances, Scalar, Frame)`, coluna `v`). **Contratos congelados intocados**
(gate `architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Fecha os
2 primeiros follow-ups que o doc 12 §5 mapeou (`value.lfo`, `value.map_range`).

---

## 1. O problema (doc 12 §5)

A fatia 1 (doc 12) provou o domínio end-to-end com o par mínimo `pulse.counter → motion.drive`: um
**produtor discreto** (conta pulsos) e o **consumidor** (dirige canal). Faltavam (a) o **produtor
CONTÍNUO** — hoje o `motion.oscillator` embute a onda E escreve num canal, sem forma de a onda fluir
como valor pra compor — e (b) a **cola universal** que todo grafo de valor precisa: remapear uma
faixa (o `[-1,1]` de um LFO → os graus/pixels que o canal quer). Sem esses dois, o domínio só sabia
contar e broadcast.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

Fontes primárias: TouchDesigner (LFO CHOP, Math CHOP Range), Houdini VEX (`fit`/`efit`/`fit01`/
`fit11`), Cavalry (oscilador + Remap), Nuke, Max/MSP (`scale`), vvvv. Vereditos que dirigiram o design:

- **LFO = o oscillator na forma PRODUTORA.** A onda é a mesma; a diferença é só *emitir valor* vs
  *escrever canal*. Reusei o **wave core transcendental-free do `motion.oscillator`** (aproximação
  parabólica de seno com correção Capens/devmaster de 2ª ordem, ~0.09% de erro, só multiply+abs) —
  a referência in-repo já limpa de HR-5. Copiado por-crate (`wave.rs`), convenção de leaf drop-crate
  (o compartilhado é a *forma*, não um símbolo).
- **Cardinalidade segue a geometria.** O `in` do LFO é **opcional**, lido só pela **contagem**
  (como o oscillator lê N do stream): conectado → campo length-N com `phase_stagger` por-instância
  (uma **onda viajante**); **desconectado → length-1** (uma oscilação global, que o `motion.drive`
  faz broadcast). É a regra do doc 12 (*valor é sempre campo; global = campo de comprimento 1*)
  aplicada ao produtor.
- **`period` (segundos), não `frequency`.** Adotei o vocabulário da família pulse (`pulse.beat` usa
  `period`), mais intuitivo pra animação ("uma onda de 2 s"), com guard `MIN_PERIOD` (nunca divide
  por zero — espelha o `pulse.beat`).
- **`map_range` = `fit` linear, com clamp NO PARÂMETRO NORMALIZADO.** `t = (v−in_lo)/(in_hi−in_lo)`,
  clamp em `t∈[0,1]`, depois `out_lo + t·(out_hi−out_lo)`. Clampar o `t` (não a saída crua) mantém
  **faixas de saída invertidas** honestas (`10..0` fica em `[0,10]`, não `[-10,10]`). **Clamp ON por
  default** = a convenção canônica do Houdini `fit()` (o `efit` extrapolador é clamp OFF) — o default
  mais defensável: um valor mapeado dirigindo size/opacity não deve estourar a faixa. Span degenerado
  (`in_lo==in_hi`) **colapsa em `out_lo`** (guard `MIN_SPAN`), nunca `NaN`.
- **`map_range` é UNÁRIO** → preserva o comprimento do campo exatamente. Nenhuma decisão de broadcast
  aqui (essa regra vive no *consumidor* `motion.drive`, doc 12). Só `+ − × ÷` + `clamp` (HR-5;
  divisão é IEEE-determinística).

## 3. O que foi adicionado (fatia 2)

**`ph2d-node-value-lfo` (drop-crate, o PRODUTOR contínuo):** `in?(instances) → value`. Emite
`value_i = waveform(wave, t/period + phase + i·phase_stagger)·amplitude + offset` na coluna `v`.
5 ondas (Sine/Tri/Square/Saw/Spike), `Effect::Temporal` (lê o playhead, sem estado). Prefixo
`value.*` (produtor abstrato de valor). `NodeUiCategory::Utility`.

**`ph2d-node-value-map-range` (drop-crate, a COLA):** `value → value`, unário, `[in_lo,in_hi] →
[out_lo,out_hi]` com `clamp` (default ON = Houdini `fit`). `Effect::Pure`. `NodeUiCategory::Utility`.

**Cena boot com DUAS cadeias de valor** (`motion_demo_strobe.rs`): ao lado da cadeia discreta
existente (`beat → counter → drive_x` em X, **broadcast**), uma cadeia contínua nova
(`grid → lfo → map_range → drive_y` em Y, **element-wise**):

```
grid → move → tint → drive_x → drive_y → strobe → output
       grid → beat ⟲ → { counter.pulse, strobe.pulse } ; counter → drive_x.value
       grid → lfo → map_range → drive_y.value
```

O ganho visível: a grade **desliza de lado em notches discretos ENQUANTO ondula pra cima/baixo
continuamente**, e pisca a cada beat — um notch broadcast e uma onda viajante element-wise, duas
cadeias por uma grade. As duas metades do domínio (produzir→remapear→dirigir, composável) na tela.
11 nós (era 8). O `drive_y` é o **mesmo tipo** `motion.drive` do `drive_x`, só outro canal — nenhum
nó novo de consumo, a prova de que a regra de broadcast do doc 12 escala.

**Testes (17 novos):** LFO (7: unconnected=global, connected=onda viajante element-wise,
offset/phase, guard period→0, wave bank in-range+periódico, resolve); map_range (7: ranges canônicos,
clamp on-pina/off-extrapola, faixa de saída invertida, span degenerado≠NaN, unário preserva N pelo
cook, resolve); integração no shell (o teste `the_continuous_lfo_chain_ripples_the_grid_in_y_element_wise`
com **3 falsificações**: cadeia morta→Y plano · map_range bypassado→bob estoura os bounds · valor
broadcast→dots em lock-step, spread=0 num instante). Todos falsificados dos 2 lados.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-value-lfo`, tipo `value.lfo` | nova | nome novo |
| crate `ph2d-node-value-map-range`, tipo `value.map_range` | nova | nome novo |
| `value_lfo::VALUE` / `value_map_range::VALUE` (pub const) | pub const | baixo (mirror local do tipo `Instances/Scalar/Frame`; não é símbolo compartilhado) |
| `ph2d-node-registry-init` regenerado (34 crates) | codegen | **conflito provável** com outra linha que adicione nó (região `<ph2d-node-sync>`) |
| cena boot `motion_demo_strobe.rs` (2ª cadeia, 8→11 nós, `drive`→`drive_x`+`drive_y`) | shell | dentro do próprio módulo Motion |
| `motion_state_tests.rs` contagem 8→11 + teste novo Y | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de
`ph2d-nodegraph` + `ph2d-node-registry` (machete verde).

## 5. O que fica (fan-out follow-up, mesma regra + tipo — doc 12 §5)

- **`pulse.sample_hold`** — trava o valor na borda do pulse, segura entre (o `sah~`; ponte
  discreto→contínuo). *Sequencial* (tem estado → `pre`, como o counter).
- **`pulse.compare`** — `value vs threshold → pulse` (ponte contínuo→discreto, dual do sample_hold;
  histerese = 2 thresholds). Fecha o combo canônico `LFO → Counter → SampleHold → drive` do doc 09.
- **`value.instance_field`** — o ÚNICO nó que MINTA um campo len-N da identidade da instância
  (index/ramp/random) — a forma sancionada de nascer variação por-elemento sem depender de um LFO
  staggered. O análogo Cavalry-Falloff / vvvv-index.
- **`value.switch`/`gate`** — roteia um de N por seletor.
- **`value.math`** (2-entradas: add/sub/mul/min/max) — o primeiro combinador que exerce a regra de
  broadcast entre DOIS campos de valor (hoje só o `motion.drive` a exerce, e só 1→N contra o stream).
