# 16 — Nota-ADR: Math + Compare (fatia 4 do domínio de VALOR) — follow-up dos docs 12–14

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia 4 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates sobre o tipo de valor `(Instances,
Scalar, Frame)` (coluna `v`) do doc 12. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Fecha os 2
follow-ups nomeados que sobraram (docs 12 §5, 13 §5, 14 §5): o **primeiro combinador de DOIS campos**
e a **ponte valor→pulse genuína**.

---

## 1. O problema (docs 12–14 §5)

Depois das fatias 1–3, o domínio de valor sabe **produzir** (`counter`, `lfo`, `instance_field`),
**amostrar** (`sample_hold`), **remapear** (`map_range`) e **rotear** (`drive`). Faltavam duas peças
de natureza distinta que fecham a composabilidade:

- **O primeiro combinador de DOIS campos.** Até aqui todo nó de valor é ou unário (`map_range`) ou um
  produtor; a **regra de broadcast 1→N** (doc 12) só era exercida no *consumidor* `motion.drive`, e só
  contra o stream de transform. Faltava o nó que combina **dois campos de valor** entre si — o que
  desbloqueia `instance_field × lfo → …` (um **gradiente espacial modulado no tempo**) numa aresta só.
- **A ponte valor→pulse GENUÍNA.** O `pulse.sample_hold` (doc 14) é `pulse→…→value`; o `pulse.threshold`
  (doc 06) lê um **canal de transform** (`INST_VEC2`, o input do "clock hack" que o doc 09 matou), não
  o domínio de valor. Faltava o **dual** do sample_hold: um `value → pulse` que deixa um grafo de valor
  **realimentar** o grafo de pulso, fechando o round-trip contínuo↔discreto.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`value.math` (o combinador):** varredura de TouchDesigner **Math CHOP** (Combine), Houdini VOP
add/multiply, Nuke Merge(math), Cavalry **Math**, Max/MSP `+`/`*`, vvvv. Vereditos que dirigiram o
design:

- **UM nó multi-op, não uma explosão por-op.** Os *node editors* maduros (TD Math CHOP, Cavalry Math,
  Nuke Merge) convergem num **único nó com um seletor de operação** — só os *patchers* textuais (Max/Pd)
  usam um objeto `+` minúsculo por op. A explosão por-op (6 crates add/sub/mul/div/min/max) contradiz o
  ethos "author once" do domínio de valor (doc 12) e incharia o registry. **Escolha: um `value.math`
  com param `op` enum.**
- **O conjunto de ops é o núcleo reference-convergente:** Add, Subtract, Multiply, Divide, Min, Max —
  unânime nas fontes. **Power/log ficam de fora por HR-5**; divisão é IEEE-determinística e OK, mas
  **guardada** contra divisor (quase-)zero (colapsa em `0.0`, nunca `inf`/`NaN` num campo downstream) —
  a mesma disciplina do guard `MIN_SPAN` do `map_range`.
- **O nome.** "Math" é o rótulo que TD/Cavalry usam e que um artista entende de cara (vs "Combine"/
  "Merge", que carregam bagagem de compositing). Prefixo `value.*` correto (transformador de valor, sem
  canal visível). Portos `a`/`b` (não `x`/`y`, que confundem com coordenadas).
- **A regra de broadcast é a MESMA do doc 12**, agora entre dois *campos*: saída `max(len_a, len_b)`;
  **length-1 faz hold** em todo índice; length-N element-wise; desiguais ambos >1 = mismatch
  (`debug_assert` + leitura leniente). Copiei o helper `value_at` do `motion.drive` (`field_at`, leaf).

**`pulse.compare` (a ponte valor→pulse):** varredura de Max `>~`/`edge~`, Pd `moses`/`threshold~`,
Reaktor compare, TouchDesigner Trigger CHOP, e o próprio `pulse.threshold` in-repo. Vereditos:

- **Histerese de Schmitt é o núcleo, não um extra.** Um limiar único dispara a cada wiggle sobre ele —
  ruído sozinho vira uma rajada de pulsos espúrios (Wikipedia). **Dois thresholds** (`rise` > `fall`)
  dão memória bistável: uma vez armado, o sinal precisa cair abaixo do `fall` separado pra re-armar.
  Pd `threshold~` (trigger/rest) e TD (`threshup`/`threshdown`) têm exatamente esses dois. **Portei o
  núcleo Schmitt do `pulse.threshold` verbatim** (`step_one` idêntico) — a ÚNICA diferença é o domínio
  de entrada (o campo de valor `v` vs um canal de transform). Isso confirma que os dois **coexistem sem
  duplicar**: mesma matemática, portas diferentes.
- **Direção (`edge`)** Rise/Fall/Both = os dois outlets do Max `edge~` / o seletor "Trigger On" do TD.
- **Sequencial:** o `armed` latched é uma recorrência por-instância → anda no `pre` do porto `state`,
  como counter/threshold/sample_hold. `Effect::Pure` (o tick entra pela aresta `pre`).

## 3. O que foi adicionado (fatia 4)

**`ph2d-node-value-math` (drop-crate, o COMBINADOR):** `(a, b) → value`. 6 ops (Add/Subtract/Multiply/
Divide/Min/Max) via param `op` enum; **exerce a regra de broadcast 1→N entre dois campos** (`field_at`
+ `combine`, saída `max(len)`); Divide guardado (`|b| < 1e-9 → 0`). `Effect::Pure`. Prefixo `value.*`.
`NodeUiCategory::Utility`.

**`ph2d-node-pulse-compare` (drop-crate, a PONTE valor→pulse):** `value → pulse`, Schmitt (`rise`/
`fall`/`edge`), unário sobre o campo (cada instância compara o seu `v` → pulso length-N). Estado
`cmp_armed` no `pre` do porto `state`. `Effect::Pure`. Prefixo `pulse.*`. `NodeUiCategory::Utility`.
**Não duplica `pulse.threshold`** (canal de transform) — dual do `sample_hold`, sobre o domínio de valor.

**Cena boot com a 4ª cadeia — o round-trip (Rotation)** (`motion_demo_strobe.rs`), **21 nós** (era 15):

```
grid → move → tint → drive_x → drive_y → drive_size → drive_rot → strobe → output
       instance_field × lfo_g → math ⟳ compare ⟳ counter_r → rot_range → drive_rot.value
```

- **lfo_g** (`value.lfo`, `in` DESCONECTADO → length-1 global) modula tudo.
- **math** (`value.math`, Multiply) = `instance_field(Ramp, N) × lfo_g(1)` → **broadcast 1→N**: um campo
  cuja amplitude é GRADUADA por índice (dot 0 fica em 0, o dot do topo oscila ±1 cheio).
- **compare** fira um pulso quando o `v` de um dot cruza `0.4` (Schmitt até `0.2`); **só os dots cuja
  amplitude graduada passa do threshold disparam** — a metade de cima.
- **counter_r** (`pulse.counter`, Wrap 0..8) acumula os cruzamentos; **rot_range** (`value.map_range`)
  mapeia `0..8 → 0..90°`; **drive_rot** (`motion.drive`, Rotation, Add) → os dots que disparam
  **catracam** a rotação (e dão a volta); os de baixo ficam parados. O domínio de valor contínuo
  alimentando o discreto e voltando — o round-trip do doc 16, na tela.

O `lfo_g` roda num período de 0.5 s (ritmo diferente do beat de 1.4 s), pra a catraca de rotação ler
como o próprio relógio. `instance_field` fan-out: alimenta a cadeia de Size (existente) E a `math`.

**Testes (14 unit + 1 integração):** math (7: cada op, guard de divisão, broadcast 1→N nos 2 sentidos,
element-wise N×N, input desconectado = campo zero, através do cook, resolve — falsificados); compare
(7: dispara-1×-na-borda + banda de histerese, single-threshold chatters, sustained-high = 1 pulso,
Rise/Fall/Both, banda invertida clampada, element-wise sobre o campo, resolve — falsificados). Integração
no shell — `the_value_to_pulse_round_trip_ratchets_the_rotation` (**3 falsificações:** cadeia morta →
0 dots catracam · broadcast-collapse / compare-sem-threshold → nenhum dot fica em 0 · rot_range
bypassado → estoura o bound de 90°). As 6 cenas/testes anteriores (X/Y/Size/strobe/determinismo)
seguem verdes — a cadeia nova só ADICIONA e escreve num canal (Rotation) que nenhum teste anterior lê.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-value-math`, tipo `value.math` | nova | nome novo |
| crate `ph2d-node-pulse-compare`, tipo `pulse.compare` | nova | nome novo |
| `value_math::VALUE` / `pulse_compare::{VALUE,PULSE}` (pub const) | pub const | baixo (mirror local dos tipos) |
| `ph2d-node-registry-init` regenerado (38 crates) | codegen | **conflito provável** com outra linha que adicione nó (região `<ph2d-node-sync>`) → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (4ª cadeia, 15→21 nós, +drive_rot/lfo_g/math/compare/counter_r/rot_range) | shell | dentro do próprio módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` (contagem 15→21, doc-comments, +1 teste, pre-loops 4→6) | shell | idem |

Coluna de stream nova `cmp_armed` (local ao stream do compare, sem registro global). Nenhum contrato
congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de `ph2d-nodegraph` +
`ph2d-node-registry` (machete verde).

## 5. O que fica (fan-out follow-up, mesma regra + tipo)

- **`value.switch`/`gate`** — roteia um de N campos por um seletor (o último utilitário do vocabulário
  de valor mapeado nos docs 12–14).
- **Utilitários do M2** (doc 01 §3): `motion.delay` (atrasa um canal N ticks) · `pulse.on_change`
  (dispara quando um valor muda) — os últimos do M2 antes do **M3** (distribuições avançadas +
  deformers). O doc 01 §3 tem a lista exaustiva por fase.

> Com Math + Compare o vocabulário-núcleo do domínio de valor está **completo**: produzir → combinar →
> amostrar → comparar → remapear → rotear, contínuo↔discreto, tudo autorado uma vez pela regra de
> broadcast — nunca uma variante escalar-vs-campo.
