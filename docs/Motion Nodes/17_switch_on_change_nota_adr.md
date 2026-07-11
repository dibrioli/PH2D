# 17 — Nota-ADR: Switch + On Change (fatia 5 do domínio de VALOR) — fecha o vocabulário

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia 5 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates sobre o tipo de valor `(Instances, Scalar,
Frame)` (coluna `v`) do doc 12. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Fecha os dois
últimos follow-ups nomeados (doc 16 §5): **roteamento** e **detecção de mudança** — o vocabulário-núcleo
do domínio de valor fica **completo**.

---

## 1. O problema (doc 16 §5)

Depois das fatias 1–4 o domínio de valor sabe produzir (counter/lfo/instance_field), combinar (math),
amostrar (sample_hold), comparar (compare = cruzamento de threshold), remapear (map_range) e rotear pra
canal (drive). Faltavam duas primitivas que todo grafo de valor maduro tem:

- **Roteamento / branching.** Nenhum nó ainda ESCOLHE entre campos — um grafo de valor não podia
  ramificar (rotear uma fonte diferente pro mesmo fio conforme um seletor anima). É o multiplexador.
- **Detecção de MUDANÇA (dual do compare).** O `pulse.compare` dispara no cruzamento de um NÍVEL; faltava
  o trigger que dispara quando um valor **muda** (a derivada, não o nível) — o relógio natural de um
  `pulse.counter`/`pulse.sample_hold`/`value.switch` que dá um passo.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`value.switch` (o roteador):** TouchDesigner **Switch CHOP** (índice + N inputs), Houdini **Switch VOP**
(param `input`), Nuke **Switch** (`which`), Max **`selector~`**/`gate`, Cavalry. Vereditos:

- **Um nó com N inputs + um índice**, unânime. O nome **"Switch"** é o mais universal (TD/Houdini/Nuke).
  Prefixo `value.*` correto (roteia valor, sem canal visível). Ports `select` + `in0..in3` (4 inputs
  cobrem o mux comum sem porta de aridade variável).
- **`select` é um VALOR de entrada, não um param** — a decisão de design que importa. Um param seria
  estático; um input de valor deixa `pulse.counter`/`value.lfo`/qualquer campo **animar** a seleção. É o
  que torna o switch parte viva do grafo de valor (o roteamento reage no tempo).
- **Per-element por construção + broadcast (doc 12).** Como `select` é um campo, o elemento `i` lê
  `in[round(select_i)][i]`: um `select` length-1 faz **broadcast** (a grade inteira troca junta — o caso
  comum), um `select` length-N escolhe um input possivelmente diferente por elemento (o mux per-point do
  Houdini). Todo input obedece o `1→N` hold. Índice = `clamp(round(select), 0, N-1)` (HR-5). `Pure`.

**`pulse.on_change` (o detector de mudança):** Max/Pd **`change`** (bang quando o valor muda),
TouchDesigner Trigger-on-change. Vereditos:

- **Dispara na DERIVADA, não no nível** — o complemento exato do `pulse.compare`. É o relógio de um valor
  em degraus (`counter`, `sample_hold`, `switch` flip). O Max `change` é igualdade-exata; adotei um
  **`epsilon`** (mudança mínima que conta) — a versão honesta em ponto-flutuante, que ignora dither pra um
  valor estável nunca chatterar (`epsilon = 0` recupera a igualdade-exata).
- **Sequencial:** o valor anterior anda no `pre` do porto `state` (como sample_hold/counter). **Prime na
  1ª tick** (registra o valor, NÃO dispara) pra um grafo fresco não emitir pulso espúrio. Unário sobre o
  campo (cada instância observa o seu → pulso length-N). `Effect::Pure` (tick entra pela aresta `pre`).
  Nome `pulse.on_change` (o do doc 16); prefixo `pulse.*` (produtor de pulso).

## 3. O que foi adicionado (fatia 5)

**`ph2d-node-value-switch` (drop-crate, o ROTEADOR):** `(select, in0..in3) → value`. Roteia por
`clamp(round(select_i), 0, N-1)` sob a regra de broadcast (`field_at` + `switch`, saída `max(len)`).
`select` desconectado → `0` (→ `in0`). Sem params (seleção animável via input). `Effect::Pure`. Prefixo
`value.*`. `NodeUiCategory::Utility`.

**`ph2d-node-pulse-on-change` (drop-crate, o DETECTOR):** `value → pulse`. Dispara quando
`|v − prev| > epsilon`; prime na 1ª tick (registra, não dispara). Sequencial (`oc_prev` + `oc_primed` no
`pre` do porto `state`). Unário (pulso length-N). `Effect::Pure`. Prefixo `pulse.*`.
`NodeUiCategory::Utility`.

**Cena boot PEQUENA (mantendo o pedido do Enio de simplificar)** — os dois nós num só grid
(`motion_demo_strobe.rs`, ~11 nós):

```
grid → tint → drive_size → strobe → output
       grid → instance_field(Ramp)   ─┐
       grid → instance_field(Random) ─┤
       lfo ───────────────────────────┴→ switch → size_range → drive_size.value
                                         switch → on_change ⟳ → strobe.pulse
```

- **switch** roteia o Size entre uma **Ramp ordenada** (`in0`) e um **Random scatter** (`in1`); o `select`
  é um `value.lfo` lento (amplitude 0.5, offset 0.5 → cicla `0 ↔ 1` a cada zero-crossing, ~1 s). O padrão
  de tamanho da grade **alterna entre ordem e aleatoriedade**.
- **on_change** observa o valor roteado e dispara um pulso o tick em que o padrão VIRA; o `motion.strobe`
  vira isso num **flash branco** — a grade pisca exatamente NO flip. (Strobe cor-only, `size_boost = 0`,
  pra o Size ficar o sinal puro do switch.)

Roteamento (switch) e detecção-de-mudança (on_change) lado a lado, num grid legível.

**Testes (11 unit + 3 integração):** switch (5: global broadcast, round+clamp, per-element route, fonte
length-1 held, através do cook — falsificados); on_change (6: dispara-no-degrau/quieto-quando-held,
escada = 1 pulso/degrau, epsilon ignora dither mas não o degrau, per-element, resolve — falsificados).
Integração no shell: `the_switch_routes_the_size_between_two_patterns` (**a Ramp routeada é monotônica E
difere do frame Random** — switch preso em `in0` casaria os dois; preso em `in1` nunca seria monotônico) ·
`the_on_change_flashes_the_grid_on_each_pattern_flip` (**flash + ~2 eventos** = dispara NO flip, não a cada
tick; on_change morto → vermelho preso na base). O loop-replay do doc 11
(`a_loop_range_replays_the_simulation_from_its_start`) foi ajustado (LAP 45→90 pra conter um flip; sinal =
max vermelho) — o mecanismo checkpoint/restore segue exercido (on_change + strobe sequenciais).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-value-switch`, tipo `value.switch` | nova | nome novo |
| crate `ph2d-node-pulse-on-change`, tipo `pulse.on_change` | nova | nome novo |
| `value_switch::VALUE` / `pulse_on_change::{VALUE,PULSE}` (pub const) | pub const | baixo (mirror local dos tipos) |
| `ph2d-node-registry-init` regenerado (40 crates) | codegen | **conflito provável** com outra linha que adicione nó → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita p/ o demo switch+on_change, ~11 nós) | shell | dentro do próprio módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` + `render_loop/motion_bridge_tests.rs` | shell | idem |

Colunas de stream novas `oc_prev`/`oc_primed` (locais ao stream do on_change, sem registro global).
Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de
`ph2d-nodegraph` + `ph2d-node-registry` (machete verde).

## 5. O que fica

- **`motion.delay`** (atrasa um canal N ticks — eco/time-shift puro, distinto do `motion.trail`) — o
  último utilitário do M2 (doc 01 §3) antes do **M3** (distribuições avançadas + deformers).
- **M3:** `motion.distribute-fibonacci`/`-poisson`/`-voronoi`/`-path` · `motion.lattice`/`-bend`/`-twist` ·
  `motion.morph`/`-look-at`/`-boids`/`-verlet-rope` etc. (doc 01 §3 tem a lista exaustiva).

> Com Switch + On Change o **vocabulário-núcleo do domínio de valor está completo**: produzir → combinar →
> amostrar → comparar → **detectar mudança** → remapear → **rotear** → dirigir, contínuo↔discreto, tudo
> autorado uma vez pela regra de broadcast. O próximo grande passo é o M3 (geometria/distribuições).
