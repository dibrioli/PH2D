# Doc 80 — `value.time`: o RELÓGIO animado como produtor do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..79.

## O que é

O **produtor do relógio cru** — o playhead animado levado ao grafo de valor como um
número simples. Todos os outros produtores do domínio são espaciais ou aleatórios
(`value.instance_field` Index/Ramp/Random, `value.noise`, `value.pattern`) e nenhum
é o **relógio**. O `value.lfo`/`value.noise` trazem o tempo mas embutem uma FORMA DE
ONDA em volta dele; este emite o relógio **sem forma**, para o grafo construir a
própria função do tempo. É o `@Time`/`$T` do Houdini, o Timer/Beat CHOP do
TouchDesigner, o nó `time` de todo editor de nós.

- **input** `in` : geometria (`Vec2`), lida só para a CONTAGEM (opcional) — é um PRODUTOR
- **output** `out` : VALUE, `out[i] = playhead · rate + offset + i · stagger`
- **params** `rate` (velocidade) · `offset` (início) · `stagger` (deslocamento por-instância)
- **Effect** `Temporal` (lê o playhead, sem estado)

## Decisões

1. **MONOTÔNICO, não periódico — e é o ponto inteiro.** O `value.lfo(Saw)` dobra o
   relógio a cada período (`phase − floor(phase)`); este SOBE pra sempre
   (`t · rate + offset`). Essa distinção é a razão de existir — rotação infinita, um
   offset que rola sem fim, um drift que acumula — e é o que o torna o par natural
   do **`value.wrap`**: `time → wrap(Repeat)` é um relógio dente-de-serra
   controlável, `time → wrap(Mirror)` um triângulo, cada um dobrado exatamente onde
   você quer, enquanto o `time` cru segue subindo para o que o quiser sem dobra.

2. **Cardinalidade segue a geometria** (o padrão do `value.lfo`): o `in` opcional é
   lido só pela CONTAGEM — conectado → campo length-N com `stagger` por-instância
   (uma rampa que viaja pela grade), **desconectado → relógio length-1 global** (o
   caso comum: *o* relógio), difundido por toda parte pela regra `1→N` do
   `motion.drive`. A `count_law` devolve `max(1)`: um port vazio é `0`, um stágio de
   contagem-zero é PULADO, e o nó ficaria inalcançável no device no instante em que
   algo consumisse o relógio global (a lição do `value.lfo`).

3. **O playhead é `params.playhead`** — o MESMO uniform que o `value.lfo` lê. O
   kernel é uma multiplicação-soma pura, então o resultado do device é
   bit-comparável ao CPU (a única divergência é um FMA que o driver pode fundir, ε
   abaixo do orçamento). `Write` binding (cunha um `v` fresco, não carrega o input;
   port-0 in-type ≠ out-type ⇒ o sequenciador sabe). Sem `applicable` ⇒ **sem
   fallback de CPU**.

## Rejeitados

- **Um `value.time` length-1 puro sem input** — o padrão `in`-opcional do `value.lfo`
  já dá o length-1 global (desconectado) E a rampa length-N (conectado + stagger) da
  MESMA porta; um produtor sem-input seria um count-law especial sem ganho.
- **Embutir uma forma de onda** — isso é o `value.lfo`. Este é o relógio CRU de
  propósito; quem quer uma onda a compõe (`time` → `lfo`? não: o `lfo` já lê o
  playhead) ou dobra com `value.wrap`/`value.quantize`.
- **`value.time` NÃO é o `motion.time_remap`** (que existe): aquele reescala o TEMPO
  DE COOK de uma sub-árvore (escopos de relógio); este emite o playhead como um
  VALOR. Sistemas diferentes.

## Preço / cobertura

Kernel WGSL = uma linha: `params.playhead * params.rate + params.offset +
f32(i) * params.stagger`. Sem `wgsl_lib` (nenhum helper), sem round (nenhum branch),
sem transcendental. `count_law` = `max(1)` sobre o input. Binding `Write` na coluna
`v`. Paridade de dispositivo bit-comparável (multiply-add).

**Gates:** relógio global desconectado (t → o playhead; rate escala, offset desloca)
· **MONOTÔNICO** (sobe ~10 em 10 "períodos", não dobra — falsificável contra um Saw)
· rampa que viaja conectado + stagger (rampa espacial em t=0, sobe com t) · rate
negativo anda pra trás · registro · **paridade de dispositivo** (`#[ignore]`, RTX,
`rate 1.3`/`offset 0.4`/`stagger 0.13` — o stagger nonzero faz o campo VARIAR por
instância, senão um relógio chato passaria o `column_is_nonzero`; `max|d| < 1e-4`).

## Demo — `PH2D_VALUE_TIME_SMOKE=1`

⚠️ **Aperte PLAY** — value.time é temporal (parado em t=0 mostra só a rampa do
stagger). Duas fileiras de 24: de cima **RAW** (`time → drive(Y)`, marcada
`>> EVALUATE <<`) — tocando, os pontos SOBEM juntos numa diagonal e saem de quadro,
o relógio nunca volta; de baixo **WRAPPED** (`time → wrap(Repeat, [0,1]) → drive`) —
o MESMO relógio dobrado num dente-de-serra que salta de volta e fica na tela pra
sempre. É `time → wrap`: o relógio cru domado num loop. Selecione a de cima → mude
**Rate** (negativo anda pra trás), **Stagger** (a diagonal).
