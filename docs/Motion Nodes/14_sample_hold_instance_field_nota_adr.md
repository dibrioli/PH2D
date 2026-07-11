# 14 — Nota-ADR: Sample & Hold + Instance Field (fatia 3 do domínio de VALOR)

**Data:** 2026-07-11 · **Linha:** `line/MotionNodes` · **Status:** fatia 3 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates sobre o tipo de valor `(Instances,
Scalar, Frame)` (coluna `v`) do doc 12. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Fecha mais 2
follow-ups do doc 13 §5 e **completa o combo canônico do doc 09** `LFO → SampleHold → drive`.

---

## 1. O problema (doc 13 §5)

Depois das fatias 1–2 (counter, drive, LFO, map_range) faltavam duas peças de natureza distinta:

- **A ponte contínuo→discreto AMOSTRADO.** `pulse.threshold` já existe, mas lê um **canal de
  transform** (`INST_VEC2`, X/Y/Rot/Size) — foi o input do "clock hack" que o doc 09 matou — não o
  domínio de valor; e é um *gerador de pulso*, não um *sampler*. Faltava travar um **valor contínuo**
  na borda de um pulse e segurá-lo (o `sah~`). Um 2º Schmitt sobre valor seria ~80% duplicata do
  threshold e o combo do doc 09 não pede isso — pede **sample-and-hold**.
- **A fonte de variação POR-ELEMENTO.** Todo valor até aqui é ou um global len-1 (LFO, count) feito
  broadcast, ou um campo cuja variação por-elemento veio contrabandeada no `phase_stagger` de um
  behaviour. Faltava o nó que **MINTA** um campo len-N da *identidade* da instância — a forma
  sancionada de nascer variação espacial como valor de 1ª classe.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

- **Sample & Hold:** Buchla/modular S&H, Max `sah~`, Reaktor, TouchDesigner Hold CHOP. Semântica
  unânime: na borda de subida do gatilho, amostra a entrada; entre gatilhos, segura a última amostra.
  É **sequencial** (estado no `pre`, como counter/strobe/threshold). Edge-safe (um pulse mantido alto
  amostra UMA vez, não por-tick — TD "Off to On"). **Priming na 1ª tick** (amostra imediata) pra a
  saída nascer viva, não um 0 morto — a mesma disciplina do `pulse.beat`.
- **Instance Field:** Houdini `@ptnum`/`@id`, Cavalry Index/Falloff, vvvv spread-index, TouchDesigner
  Pattern CHOP (ramp/index). Convergência: os modos essenciais são **Index** (ordinal cru), **Ramp**
  (0..1 normalizado) e **Random** (hash por-índice). O Random é **stateless** (Jarzynski/Olano 2020):
  função pura de `(seed, index)` → reproduz bit-a-bit sob scrub e num lowering GPU. Reusei o
  `hash3`/`rand01` do `motion.emitter` (splitmix 32-bit → [0,1), HR-5), copiado por-crate (leaf).

## 3. O que foi adicionado (fatia 3)

**`ph2d-node-pulse-sample-hold` (drop-crate, o SAMPLER):** `(value, pulse) → value`. Na borda de
subida do pulse amostra o `v` da entrada e o segura entre; prime na 1ª tick. Broadcast do pulse
(`pulse_at`: len-1 dispara todas juntas, len-N cada uma na sua borda). Estado (valor segurado +
`sh_prev` + `sh_primed`) no `pre` do porto `state`. `Effect::Pure` (o tick entra pela aresta `pre`).
Sem params — um sampler edge-triggered puro. Prefixo `pulse.*` (é disparado por pulse).
`NodeUiCategory::Utility`.

**`ph2d-node-value-instance-field` (drop-crate, o MINTADOR):** `in?(instances) → value`. Minta o
campo len-N da identidade: **Index** (`0..N-1`), **Ramp** (`i/(N-1)` em [0,1]), **Random**
(hash → [0,1)). `in` opcional lido só pela contagem (como o LFO); desconectado → len-1 degenerado.
`Effect::Pure`, emite `v`. Prefixo `value.*` (produtor de valor). `NodeUiCategory::Utility`.

**Cena boot com TRÊS cadeias de valor** (`motion_demo_strobe.rs`), 15 nós (era 11):

```
grid → move → tint → drive_x → drive_y → drive_size → strobe → output
       grid → beat ⟲ → { counter.pulse, strobe.pulse, sample_hold.pulse }
              counter ⟲ → drive_x.value           (X: discreto, BROADCAST)
       grid → lfo → sample_hold ⟲ → map_range → drive_y.value   (Y: sample-and-HOLD)
       grid → instance_field → size_range → drive_size.value    (Size: por-elemento)
```

- **X** (fatia 1): a grade desliza em notches no beat (broadcast).
- **Y** (NOVO): o LFO contínuo é **amostrado no beat e segurado** → a onda vira **escada** (cada dot
  pula pra um novo nível no beat e segura entre). Fecha o combo `LFO → SampleHold → drive` do doc 09.
- **Size** (NOVO): `instance_field(Ramp)` dá a cada dot um tamanho por índice → **gradiente espacial**
  (pequeno→grande), que o strobe multiplica no flash.

**Testes (16 novos):** sample_hold (6: segura/amostra na borda, held-high amostra 1×, nova borda
re-amostra, broadcast len-1→N, escada de rampa, resolve — todos falsificados); instance_field
(hash 3 + campo: Index/Ramp/Random/unconnected/cook/resolve, 7); integração no shell —
`the_sample_and_hold_chain_staircases_the_grid_in_y` (3 falsificações: **segura entre beats** /
**degrau no beat** / element-wise via >3 Y distintos ≠ 3 linhas-base) e
`the_instance_field_chain_gives_the_grid_a_size_gradient` (spread>0.1 + limitado por size_range em
repouso). O strobe segue verde (base virou o gradiente ~0.3..0.55; flash ×3.2 = ~1.76 > 1.5).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-pulse-sample-hold`, tipo `pulse.sample_hold` | nova | nome novo |
| crate `ph2d-node-value-instance-field`, tipo `value.instance_field` | nova | nome novo |
| `pulse_sample_hold::VALUE` / `value_instance_field::VALUE` (pub const) | pub const | baixo (mirror local do tipo) |
| `ph2d-node-registry-init` regenerado (36 crates) | codegen | **conflito provável** com outra linha que adicione nó (região `<ph2d-node-sync>`) |
| cena boot `motion_demo_strobe.rs` (3 cadeias, 11→15 nós, +drive_size/sample_hold/instance_field/size_range) | shell | dentro do próprio módulo Motion |
| `motion_state_tests.rs` contagem 11→15 + 2 testes novos + Y test reescrito | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de
`ph2d-nodegraph` + `ph2d-node-registry` (machete verde).

## 5. O que fica (fan-out follow-up — doc 13 §5 restante)

- **`pulse.compare`** — `value vs threshold → pulse` (a ponte contínuo→discreto GENUÍNA sobre o
  domínio de valor, dual do sample_hold; o `pulse.threshold` só lê canal de transform, não `v`).
  Histerese = 2 thresholds (portar o núcleo Schmitt do threshold).
- **`value.switch`/`gate`** — roteia um de N por seletor.
- **`value.math`** (2-entradas: add/sub/mul/min/max) — o 1º combinador que exerce a regra de
  broadcast entre DOIS campos de valor (hoje só o `motion.drive` a exerce, e só 1→N contra o stream).
  Com ele o `instance_field × lfo → drive` (gradiente espacial modulado no tempo) fica trivial.
