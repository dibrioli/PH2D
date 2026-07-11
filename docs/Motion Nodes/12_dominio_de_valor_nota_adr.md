# 12 — Nota-ADR: o domínio de VALOR (pulse.counter puro + motion.drive) — P2 do doc 09

**Data:** 2026-07-10 · **Linha:** `line/MotionNodes` · **Status:** fatia 1 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates sobre um tipo de porta que **já
existe**. **Contratos congelados intocados** (o gate `architecture_contract_surface` conta só
métodos de `NodeOp`=2/`OpResolver`=1 e campos de `NodeManifest`=8; nenhum muda). Fecha o P2 que o
doc 09 §4.3 reservou ("decida com o Enio" — o Enio mandou prosseguir).

---

## 1. O problema (doc 09 §4.3)

Hoje cada behaviour **embute** seu próprio cálculo de valor e aplica num canal (o `motion.step`
conta E empurra X num nó só). Falta um **valor** que flua num socket próprio, separado dos canais
de transform, pra compor `fonte → reduz → mapeia → dirige canal` em vez de tudo-num-nó. É o que
libera `pulse.counter` como REDUTOR puro (o nome que o doc 09 §4.2 deixou livre) e habilita o combo
canônico `LFO → Counter → SampleHold → drive`.

## 2. A pesquisa do padrão-ouro (antes de codar — regra da DIRETIVA §1)

Varredura de fontes primárias (Cavalry, TouchDesigner CHOP, Houdini detail↔point, Max/MSP
control-vs-signal, vvvv spreads, Faust). Convergência que dirigiu o design:

- **O "domínio de valor" NÃO é um substrato de segunda classe** — é o **mesmo stream carregando um
  escalar por elemento**. E o tipo **já existe** no PH2D: `PortType(Instances, Scalar, Frame)`, a
  coluna `v` que o `debug.const`/`debug.wave` já usa; é o **dual contínuo** do pulse (`…, Event`).
  Logo, isto é fan-out drop-crate, não substrato novo.
- **A única decisão real é a cardinalidade (como um 1 combina com um N).** Veredito unânime: um
  valor é **SEMPRE um campo por-instância (N escalares)**; um valor "global" é só um **campo de
  comprimento 1**. Nada de tipo "escalar global" separado — isso **dobraria o conjunto de nós**
  (escalar-vs-campo de cada nó), a explosão que TD/Houdini/vvvv/Faust todos evitaram. Lição do
  Faust: *uma constante é o campo degenerado*.
- **A regra de broadcast (uma só, load-bearing):** um combinador/consumidor opera sobre
  `max(count_A, count_B)`; **comprimento-1 faz hold (broadcast) pra todos os N**; comprimentos
  iguais combinam element-wise; **desiguais ambos >1 = mismatch**. É o "held constant" do TD / o
  "detail→point" do Houdini, **restrito a só 1→N** — preserva o estrito do substrato (um 3 nunca
  encaixa silenciosamente num 7) e habilita o único caso ergonômico do combo (um LFO/count global
  dirige muitos). Deliberadamente **NÃO** adotei o modulo-cíclico do vvvv pra mismatches
  arbitrários (esconde bug).

## 3. O que foi adicionado (fatia 1 — a menor que prova o domínio end-to-end)

**`ph2d-node-pulse-counter` (drop-crate, o REDUTOR PURO):** `pulse → value`. O núcleo de contagem
do `motion.step` **verbatim** (tick monotônico no `pre`, +1 só na borda de subida; displayed =
tick+modo: Wrap/Clamp/Zigzag; módulo euclidiano, HR-5), **menos** o `channel`/`step` e a escrita no
canal — emite a coluna `v`. Prefixo `pulse.*` correto (plumbing abstrato de pulso/valor, doc 09
§4.2). `NodeUiCategory::Utility`.

**`ph2d-node-motion-drive` (drop-crate, o CONSUMIDOR):** `(instance stream, value) → canal`, com
`scale` (o ex-`step`) e `mode` (Add/Set/Multiply), falloff-masked. **É onde vive a regra de
broadcast** (`channel::value_at`: len-1 → hold, len-N → element-wise, `debug_assert` no mismatch —
zero no-op silencioso). Prefixo `motion.*` correto (escreve canal = behaviour visível).
`NodeUiCategory::Transform`.

**Cena boot reconstruída** (`motion_demo_strobe.rs`): `beat → pulse.counter → motion.drive(X)`
substitui `beat → motion.step`. **Visual idêntico** (a grade varre X no beat), mas agora
COMPOSTO — e o teste `one_value_fans_out_to_two_channels` prova o ganho que o bundle não faz (um
valor → X e Rotação). `motion.step` **fica registrado** (o atalho reduce+apply). 8 nós (era 7).

**Testes:** counter puro (edge-safe + 3 modos + emite valor sem canal, falsificados); drive
(broadcast 1→N + element-wise + Set/Multiply + falloff + valor desconectado = no-op, falsificados);
integração headless com o registry real (`the_value_domain_sweeps_the_grid_in_discrete_notches`) +
composabilidade (1 valor → 2 canais).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-pulse-counter`, tipo `pulse.counter` | nova | nome novo (o antigo `pulse.counter` virou `motion.step` no doc 09 — agora o nome está reutilizado pro redutor PURO, como o §4.2 planejou) |
| crate `ph2d-node-motion-drive`, tipo `motion.drive` | nova | nome novo |
| `pulse_counter::VALUE` / `motion_drive` (VALUE local) | pub const | baixo (nome novo; o tipo é `Instances/Scalar/Frame`, não um símbolo compartilhado) |
| `ph2d-node-registry-init` regenerado (32 crates) | codegen | **conflito provável** com outra linha que adicione nó |
| cena boot `motion_demo_strobe.rs` (step → counter+drive) | shell | dentro do próprio módulo Motion |
| `motion_state_tests.rs` contagem 7→8 + nome do teste | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo.

## 5. O que fica (fan-out follow-up, cada um drop-crate sobre o mesmo tipo + regra)

O resto da mini-Utility que a pesquisa mapeou, agora barato:
- **`value.lfo`** — oscilador emitindo `value` (o produtor contínuo; hoje o oscillator só aplica em canal).
- **`value.map_range`** — `fit`/`scale`: `(in_lo,in_hi,out_lo,out_hi,clamp)`, guarda `in_lo==in_hi`, faixas invertidas.
- **`pulse.sample_hold`** — trava o valor na borda do pulse, segura entre (o `sah~`; a ponte discreto→contínuo).
- **`pulse.compare`** — `value vs threshold → pulse` (a ponte contínuo→discreto, dual do sample_hold; histerese = 2 thresholds).
- **`value.instance_field`** — o ÚNICO nó que MINTA um campo len-N da identidade da instância (index/ramp/random) — o análogo Cavalry-Falloff / vvvv-index; a forma sancionada de nascer variação por-elemento.
- **`value.switch`/`gate`** — roteia um de N por seletor.

**Nota de arquitetura:** todos são field→field, **autorados uma vez** graças à regra de broadcast
— nunca uma variante escalar-vs-campo. Variação por-elemento entra SÓ por `value.instance_field`,
nunca por distinção de tipo.
