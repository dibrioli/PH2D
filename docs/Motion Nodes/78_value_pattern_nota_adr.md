# Doc 78 — `value.pattern`: o STEP SEQUENCER (lista explícita por índice) do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..77.

## O que é

O **produtor de padrão explícito** — autore uma lista de valores que se repete
pelas instâncias por índice. Enquanto o `instance_field` (Index/Ramp/Random) e o
`value.noise` GERAM um campo por fórmula, este você **DIGITA**: `[v₀, v₁, …]`
atribuído às instâncias `0, 1, …` e ciclado. O step sequencer de uma bateria, o
*Array* do Cavalry, um detail array do Houdini — o autoramento explícito que os
procedurais nunca deram.

- **input** `in` : a geometria de instâncias (`Vec2`), lida só para a CONTAGEM (o
  mesmo port em que o `instance_field` lê o grid) — é um PRODUTOR
- **output** `out` : VALUE, `out[i] = pattern[i mod steps]`
- **params** `steps` (1..8) · `v0..v7` (os 8 slots)
- **Effect** `Pure` (sem clock, sem estado)

## Decisões

1. **Um PRODUTOR keyado na contagem** (espelho do `instance_field`): o input é lido
   só pela sua CONTAGEM (nunca passado adiante), e o kernel escreve um `v` fresco —
   binding `ColumnAccess::Write` (não lê `v`, então nenhum `in_v` é declarado para a
   naga remover; a lição do `value.reduce`).

2. **8 slots (`v0..v7`) nos PARAMS, não numa lista de texto.** Os valores SÃO o
   uniforme do kernel (`UNIFORM_BYTES = 128` = 32 slots f32; 9 params cabem folgado),
   então o nó é **device-resident pelo canal de kernel EXISTENTE** — sem canal de
   array no device, sem text param, sem LUT. `8` é o comprimento comum de sequencer;
   o cap é a **legibilidade do painel**, não o uniforme.

3. **`steps` clampado a `[1, 8]`**, então `i mod steps` nunca indexa além dos slots;
   o `switch` seleciona o valor. `steps > 8` usaria os 8; `steps = 0` colapsa em
   `v0` constante.

4. **Paridade BIT-EXATA** (`0e0`): a saída é um passthrough puro de param via
   `switch` + modulo, sem aritmética de float para divergir.

## Rejeitados

- **Uma lista de texto de comprimento arbitrário** (o *Array* completo) — seria mais
  flexível, mas o canal de LUT **LERPA** (`_sample(t)` interpola), impedância errada
  para um padrão DISCRETO (índice→valor, nearest), e forçá-lo (encoder o comprimento
  no LUT[0], indexar o buffer cru) é hacky. Uma lista arbitrária quer um **canal de
  ARRAY no device** — infra foundational (a linha GPU dedicada), deferida. Os 8 slots
  cobrem o caso comum de forma limpa.
- **Interpolar entre os valores** — isso é o `value.curve` (uma LUT suave). Este é
  DISCRETO de propósito (uma batida tem degraus, não rampas).
- **Um param de fase/offset** — deslocar o padrão é `+ offset` no índice; composição.

## Preço / cobertura

Kernel WGSL = `clamp(steps)` + `i mod steps` + um `switch` de 8 casos sobre os
params, binding `Write` na coluna `v`, `count_law: None` (a saída tem o comprimento
do input). Sem `applicable` ⇒ **sem fallback de CPU** (os valores viajam no
uniforme; **nenhum canal novo**). Paridade RTX **bit-exata**; naga valida o `switch`.

**Gates:** o padrão cicla por índice (`steps 3` sobre 7 = `[a,b,c,a,b,c,a]`) ·
`steps` clampado a `[1,8]` (0 = `v0` constante; além de 8 nunca sai dos slots) · os 8
slots são todos alcançáveis (nenhum param morto) · cook end-to-end (`steps 2, v0=0,
v1=1` sobre 5 = `[0,1,0,1,0]`) · registro · **paridade de dispositivo** (`#[ignore]`,
RTX, `steps 4` com 4 valores distintos, `max|d| = 0`).

## Demo — `PH2D_VALUE_PATTERN_SMOKE=1`

Duas fileiras de 24 instâncias: de cima `value.pattern(Steps 4)` com `[0.15, 0.6,
0.35, 1.0]` — uma **batida autorada** de 4 tempos repetida 6×; de baixo um
`instance_field(Ramp)` — uma **rampa lisa** (a referência procedural). O nó marcado
`>> EVALUATE <<` é o pattern — selecione, mude **V2** (o 3º tempo salta) ou suba
**Steps** para `8` e edite V4..V7 para uma batida mais longa.
