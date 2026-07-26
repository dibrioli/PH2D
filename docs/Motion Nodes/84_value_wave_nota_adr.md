# Doc 84 — `value.wave`: o SHAPER de forma de onda (o dual do lfo) do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..83.

## O que é

O **shaper de forma de onda** — molda qualquer campo, lido como FASE, através de uma
forma de onda periódica. É o dual SHAPER do `value.lfo`: onde o LFO *produz* uma
oscilação do PLAYHEAD (`Temporal`), este *molda* qualquer campo de entrada numa onda
(`Pure`) — a fase vem do fio, não do relógio. Alimente um `value.instance_field`
Ramp e ele desenha uma ONDA ESTACIONÁRIA espacial pela grade (uma ondulação senoidal
de pontos); alimente um `value.time` e você **reconstruiu o LFO dos primitivos**
(`time → wave == lfo`, provado por gate). É a Wave do Cavalry / o Pattern CHOP do
TouchDesigner, feito shaper de valor.

- **input** `in` : o campo de valor (`v`), lido como FASE (em ciclos)
- **output** `out` : VALUE, `waveform(wave, in·frequency + phase)·amplitude + offset`
- **params** `wave` (Sine/Tri/Square/Saw/Spike) · `frequency` · `amplitude` · `offset` · `phase`
- **Effect** `Pure` (sem relógio — a fase é a entrada); mapa unário, comprimento preservado

## Decisões

1. **NÃO é o `value.wrap`.** O wrap DOBRA um valor num range (modo de endereçamento,
   saída em `[min,max]`); este mapeia uma fase para uma waveform BIPOLAR de oscilador
   (saída `[-1,1]·amplitude + offset`, centrada em `offset`). Um wrap-Mirror de uma
   rampa é um triângulo UNIPOLAR no range; um wave-Triangle é o triângulo `±1`
   clássico de oscilador — formas diferentes, trabalhos diferentes.

2. **O banco de ondas é uma CÓPIA leaf-local do `value.lfo`** (a convenção do
   codebase: *o vocabulário compartilhado é a FORMA, não um símbolo compartilhado* —
   o `motion.oscillator` e o `value.lfo` já a copiam; esta é a 3ª cópia). O seno é a
   aproximação parabólica + correção de 2ª ordem (Capens, ~0.09%, transcendental-free
   HR-5) — um `sin` real é não-determinístico entre plataformas.

3. **A cópia é PINADA à do lfo por gate** — `time → wave == lfo` (byte-a-byte, na CPU)
   prova que as cópias Rust concordam; combinado com o gate de paridade CPU↔GPU de
   cada nó, isso pina TRANSITIVAMENTE as cópias WGSL também (`wave.wgsl == wave.cpu
   == lfo.cpu == lfo.wgsl`). Extrair um crate compartilhado seria acoplamento contra
   a convenção; a cópia+gate é o padrão "escrito duas vezes, o gate atravessa".

## Rejeitados

- **`frequency` como `period`** (o vocabulário do lfo) — o lfo divide o TEMPO por um
  período; aqui a entrada É a fase, e multiplicá-la por uma frequência (ciclos por
  unidade de entrada) é o dual natural. `time(rate) → wave(freq)` casa o lfo com
  `rate·freq = 1/period` (o gate).
- **Um crate `ph2d-value-waveform` compartilhado** — a convenção é copiar por
  drop-crate (a forma é o vocabulário); tocar o lfo para extrair adicionaria churn e
  re-verificação num nó estável, sem ganho sobre o gate de não-drift.

## Preço / cobertura

Kernel WGSL = `vw_round(wave)` + `vw_wave(kind, in·freq + phase)·amp + offset`, com
`vw_wave` = o port byte-a-byte do `lfo_wave`. Binding `ReadWrite` na coluna `v` (lê a
fase, escreve a onda), `count_law: None` (unário). Sem `applicable` (sem fallback de
CPU). Paridade de dispositivo ε (o seno parabólico carrega FMA, o mesmo orçamento do
lfo).

**Gates:** rampa de fase desenha a waveform (Sine em `0,¼,½,¾` → `0,+A,0,−A`) ·
`frequency` fixa o nº de ciclos (freq 2 numa rampa `[0,1]` reinicia em ½) · saída na
banda `[offset−A, offset+A]`, finita · cook end-to-end (rampa `[0,¼,½,¾]` Sine →
`[0,1,0,−1]`) · os pontos-âncora de TODA forma (o banco, `wave.rs`) · registro — 6
unit tests verdes. **`time → wave == lfo`** (CPU, byte-a-byte — o gate do dual, roda
em CI). Paridade de dispositivo (`#[ignore]`, RTX, rampa → Sine freq 3 → drive; ε <
1e-4). naga valida o WGSL; contrato congelado intacto (NodeManifest=8).

## Demo — `PH2D_VALUE_WAVE_SMOKE=1`

Quatro fileiras de 24, a MESMA rampa `[0,1]` em cada, `frequency 2` (duas ondulações):
de cima **SINE** (a senoide lisa; marcada `>> EVALUATE <<`), **TRIANGLE** (o
zigue-zague), **SQUARE** (a onda quadrada), **SAW** (o dente-de-serra). Selecione a de
cima → troque **Wave** para morfar as formas, suba **Frequency** para mais ondulações,
ou mexa em **Amplitude**/**Offset**/**Phase**. Estático, sem play (a fase é a rampa,
não o relógio).
