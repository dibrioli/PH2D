# Doc 70 — `value.mix`: o crossfader do domínio de VALOR (nota-ADR)

**Data:** 2026-07-25 · **Linha:** `line/motion-value` (reaberta pós-integração) · **Modo:** L

## O que é

`value.mix` — a **mistura suave** entre dois campos de valor: `mix = a + t·(b − a)`.
Completa a trilogia de COMBINE que o vocabulário de valor convergiu:

| Nó | Faz |
|---|---|
| `value.math` | aritmética (`+ − × ÷ min max`) |
| `value.switch` | seleção DURA (pega `a` OU `b`) |
| **`value.mix`** | a mistura SUAVE entre eles (o crossfade) |

Um nó de VALOR (`(Instances, Scalar, Frame)` no `v`), `Pure`, HR-5. É o crossfader
que todo grafo maduro tem: **Mix (Float)** do Blender, **Merge(mix)** do Nuke,
**Cross CHOP** do TouchDesigner — o "combine two behaviours" fundamental.

## Pesquisa (regra-ouro — porto por SEMÂNTICA, não por código)

Todos convergem em `lerp(a, b, t)` com um fator que pode ser socket OU inline, e um
clamp opcional:

| App | Nó |
|---|---|
| **Blender** | **Mix** (Float) — Factor (socket com default), A, B, **Clamp Factor** |
| **Nuke** | **Merge** (mix) / `dissolve` |
| **TouchDesigner** | **Cross CHOP** (o cross-fade por índice/valor) |
| **Max/MSP** | `mix~` |

## As decisões

1. **O fator é um VALOR, não (só) um param.** A filosofia que o `value.switch`
   fixou (*"controles são valores"* — um `pulse.counter`/`value.lfo` pode dirigir a
   seleção): o `t` é uma **PORTA** drivável. Mas um nó pelado ainda quer um knob ⇒
   `factor` é um **PARAM de fallback**: **`t` conectado sobrepõe `factor`; `t` solto
   lê `factor`** — exatamente o socket Factor do Blender, cujo default um fio
   sobrepõe.

2. **A escolha porta-vs-param é a MESMA nos dois lados.** No CPU: `t` conectado ==
   campo não-vazio na porta 2. No GPU: o kernel lê o **`HAS_t_v`** — a const de
   presença que o codegen gera por binding — e faz `select(factor, port, HAS_t_v)`.
   É o mecanismo canônico do substrato para *"esta porta está ligada?"* (o mesmo que
   o `value.math` usa via `read_a_v`/`read_b_v`), então nenhum canal novo, nenhum
   toque no contrato (`NodeManifest=8` intacto).

3. **`clamp` (o Clamp Factor do Blender, default On).** Segura `t` em `[0,1]` para a
   mistura ficar entre `a` e `b`; **Off** deixa as pontas **overshoot** (`t > 1`
   passa de `b`) / **undershoot** (`t < 0` antes de `a`) — um extremo AUTORADO, não
   bug. É o mesmo enum Off/On do `clamp` do `value.map_range`.

4. **A regra de broadcast é a UMA** (doc 12): um campo length-1 é HELD em todo
   índice; a saída é o `max` dos comprimentos conectados (a lei que `value.math`/
   `value.switch` usam). Assim `mix(constante, constante, t_length_N)` cruza dois
   valores por um fator per-element de um fio.

5. **100% GPU-resident, sem fallback.** O kernel WGSL é o porto do mesmo blend; sem
   gate `applicable` (o norte "maximize GPU").

## Alternativas rejeitadas

- **Só um `factor` param (sem porta `t`):** perderia o diferencial — o crossfade
  DIRIGIDO por outro valor (um LFO cruzando entre ruído e onda). A porta é a razão
  do nó existir sobre um `value.math` de 3 nós.
- **Só a porta `t` (sem `factor` param):** um nó pelado precisaria de um source
  constante para um crossfade fixo, e o domínio não tem um slider-constante. O
  fallback param é o socket-com-default do Blender, e é grátis via `HAS_t_v`.
- **Identity da porta `t` = 0.5 (o default do Blender) sem param:** um blend fixo
  ≠ 0.5 exigiria um source; e a identity é um const, não editável — knob morto.
- **Um canal de metadados novo para "porta conectada?":** desnecessário — o
  `HAS_<col>` já existe no codegen e o `value.math` já o usa.

## O preço (medido)

- Paridade CPU↔GPU no **dispositivo (RTX)** dentro de ε: `max |Δ|` = **4,11e-6** no
  canal P de `grid → {lfo→a, noise→b, ramp→t} → mix → drive(Y)`, com `factor = 0,9`
  de isca e o `t` conectado vencendo nos dois lados (o gate
  `value_mix_kernel_matches_the_cpu_on_the_device`, `#[ignore]`). O blend é `+ − ×`;
  o `select`/`clamp` são exatos.
- O `generated_wgsl_validates` (naga, presença exaustiva) valida o `HAS_t_v` nas
  variantes t-conectado / t-solto.

## Demo

`PH2D_VALUE_MIX_SMOKE=1` — duas fileiras de 24, o MESMO par (onda, ruído) em `a`/`b`,
só o fator difere. De cima **DRIVEN** (`t` = LFO triangular lento em `[0,1]`): a
fileira transita entre a onda limpa e o ruído e volta, ao vivo. De baixo **FACTOR**
(`t` solto, `factor = 0.5`): uma onda permanentemente meio-ruidosa. Selecione o mix
de baixo → arraste o **Factor** de 0 (só a onda) a 1 (só o ruído). Cozinha na GPU.
