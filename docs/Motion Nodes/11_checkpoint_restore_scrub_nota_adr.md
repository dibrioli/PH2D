# 11 — Nota-ADR: `Cook::checkpoint/restore` + scrub para trás — M2.N2/N3

**Data:** 2026-07-10 · **Linha:** `line/MotionNodes` · **Status:** implementado, gates verdes.
**Escopo:** adição **aditiva** ao substrato (`cook.rs`) + o cache de scrub no pump + a fiação do
shell. **Contratos congelados intocados** (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`; o gate
`architecture_contract_surface` só conta esses três — `CookCheckpoint` e os métodos novos do
`Cook` não entram na conta). Amendment #1 do plano §4 ("`checkpoint/restore` — internals/aditivo;
caps intocadas").

---

## 1. O problema

Uma simulação (mola, integrador, partículas) é uma **recorrência sobre o tick**: o frame `T`
depende do estado do frame `T-1`, carregado nas colunas do stream pela aresta `pre` (ADR-0032).
Playback normal anda pra FRENTE, tick a tick. Mas **arrastar o playhead pra trás** (scrub, ou um
loop que volta pra `lo`) não pode simplesmente cozinhar um `t` menor: o `pre` ainda segura o estado
do tick maior, então a mola/partícula aparece no estado do **futuro**. É lixo — falsificado em
`without_restore_a_rewound_cook_reads_the_future_not_the_past` (`ph2d-nodegraph`): depois de tocar
até o frame 5, um cozimento "de volta" ao frame 3 devolve o valor do frame 6.

O jeito certo (universal na indústria): **restaurar um estado anterior e re-simular pra frente**
até o alvo.

## 2. A pesquisa do padrão-ouro (antes de codar — regra da DIRETIVA §1)

Varredura de fontes primárias (GGPO/GGRS rollback netcode, Houdini DOP cache, Blender bake, binjgb
rewind, reverse-debuggers). Achados que dirigiram o design:

- **É o modelo `save`/`load`/`advance` do GGPO** aplicado a um scrub em vez de um rollback. As duas
  metades do problema são **determinismo** (pra re-sim ser bit-exato) e **política de cache**. O
  determinismo — a metade difícil — **já estava resolvido aqui** (sem transcendentais, RNG por
  hash, `BTreeMap` ordenado — ADR-0032/HR-5). Sobra só a política de cache.
- **A regra de decisão (literal do GGRS):** vá **dense** (um checkpoint por tick) a menos que o
  custo de **copiar** o estado ≫ `K × re-cozinhar`. Estado de motion-graphics é pequeno e o cozimento
  é barato → **dense** (scrub recente = restore `O(1)`, zero re-sim; GGPO "save every frame").
- **Guardas de determinismo** que ainda mordem no mesmo binário: iteração ordenada no mapa de
  estado (usamos `BTreeMap`, não `HashMap` ✓); todo estado de nó sequencial nas colunas
  snapshotadas (sem acumulador escondido — invariante do ADR-0032, re-verificado na auditoria dos 30
  nós, doc 10 ✓); e o scrub tem que usar **o mesmo caminho de cook** do playback (um "preview
  rápido" divergente é a armadilha clássica — por isso `pump_scoped` e `scrub_to_scoped`
  compartilham `cook_sinks_into`).
- **Ajuste ao "60 ticks / 32 slots" do plano §1.4:** a referência apontou isso como "o pior dos dois
  mundos" (esparso-ish sem economizar memória, e 60 ticks cobre só 1 s). Trocado por **dense limpo +
  âncora no tick-0**: cobertura pra trás ilimitada (re-sim do seed) com memória fixa, sem o meio-termo.

## 3. O que foi adicionado

**`ph2d-nodegraph/src/cook.rs` (aditivo):**
- `pub struct CookCheckpoint { prev_outputs, tick }` — snapshot do estado de simulação (o feedback
  `pre` + o tick sequencial). **Não** inclui o memo (derivável, stale após rebobinar) nem o
  `rev_counter` (fica vivo pra que o restore leia como mudança e redesenhe).
- `Cook::checkpoint() -> CookCheckpoint` — captura o estado **antes** de um frame (= o estado que o
  `advance_tick` do frame anterior deixou).
- `Cook::restore(&CookCheckpoint)` — reinstala `prev_outputs` + `tick`, **limpa o memo** (uma
  entrada é stale pra um relógio rebobinado — o fingerprint sequencial chaveia no tick, o `Temporal`
  no playhead, e ambos pularam).

**`ph2d-eval-motion` (módulo novo `checkpoint.rs` + pump):**
- `CheckpointRing` — ring **dense** (`RECENT_CAPACITY = 300` ticks ≈ 5 s), com **fallback pro
  seed** (`CookCheckpoint::default()` = `pre` vazio, tick 0) pra qualquer alvo mais velho que a
  janela. `record`/`anchor_at_or_before`/`should_record`/`clear`.
- `MotionCookPump`: grava um checkpoint no avanço forward; `scrub_to_scoped(target, playhead_of…)`
  (restore âncora + re-sim até o alvo, mesmo caminho de cook do forward, incluindo o `advance_tick`
  final — senão o resume forward fica off-by-one); `advance_or_scrub_scoped` (**um** ponto de
  entrada que escolhe forward-vs-scrub pelo próprio tick); `mark_dirty` agora **limpa o ring**.
- LOC: `lib.rs` passou de 650→732 com as adições; **dividido** (não allowlist) — testes inline →
  `eval_tests.rs`, testes de scrub → `scrub_tests.rs`. `lib.rs` a 522.

**Shell (`motion_bridge.rs`):** os dois call-sites do pump trocaram `pump_scoped` por
`advance_or_scrub_scoped`. Nenhum gesto de scrub novo (a timeline é módulo à parte, deferida) — mas
o **loop-wrap** (`loop_range`) já produz um tick pra trás, e agora **replaya a sim do início** em
vez de mostrar o futuro. Uma futura régua que setar `transport.tick` é atendida de graça.

## 4. As decisões que valem registro

### 4.1 Sem staleness — o que torna o ring simples

Para um **grafo fixo**, a sim é função pura do tick, então `checkpoint[T]` é **invariante no
tempo**: gravado uma vez, válido pra sempre. O ring nunca envelhece durante o playback; o **único**
gatilho de invalidação é um **edit de grafo** → `mark_dirty` limpa o ring (semântica
Blender/Houdini "edit invalida o cache"; um scrub pós-edit re-sima do seed sob o grafo novo).
Provado em `an_edit_clears_the_ring_and_the_scrub_resims_from_the_seed`.

### 4.2 Dense + âncora-seed, não esparso (v1)

Recente = restore `O(1)`, zero re-sim (a janela densa). Mais velho que a janela = re-sim do seed
(tick 0), limitado e ocasional. **Follow-ups nomeados** (nenhum necessário enquanto o estado é
pequeno): coluna `Arc`/COW (barateia o clone pra partículas pesadas) e stride grosso + base+delta à
la binjgb (limita a re-sim de um scrub distante e cobre uma timeline longa em menos memória) —
ambos entram atrás da **mesma** API record/anchor (a forma sparse-saving do GGRS).

### 4.3 `advance_or_scrub` — um ponto de entrada, zero-alloc pausado preservado

O pump decide forward-vs-scrub pelo tick (`tick == last`/`last+1` = forward; senão scrub). Um frame
pausado e inalterado cai no ramo forward → `pump_scoped` retorna cedo, sem cozinhar nem clonar
checkpoint (o `record` só dispara em mudança de tick). Gate `paused_frames_allocate_nothing` segue
verde.

## 5. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| `CookCheckpoint` + `Cook::checkpoint`/`restore` | `ph2d-nodegraph` | **aditivo** (nome novo) |
| `checkpoint::{CheckpointRing, RECENT_CAPACITY}` | `ph2d-eval-motion` (módulo novo) | baixo |
| `MotionCookPump::{scrub_to_scoped, advance_or_scrub_scoped, cook_sinks_into}` | `ph2d-eval-motion` | aditivo |
| `MotionCookPump::mark_dirty` (agora limpa o ring) | `ph2d-eval-motion` | comportamento estendido |
| `lib.rs` → + `eval_tests.rs` + `scrub_tests.rs` | `ph2d-eval-motion` | **arquivo dividido** (merge textual) |
| `MotionState::playhead` **removido** (redundante) | `shells/desktop` | callers migram p/ `transport.playhead` |
| `motion_bridge.rs` 2 call-sites p/ `advance_or_scrub_scoped` | `shells/desktop` | dentro do próprio módulo |

Nenhum `NodeId`/`IconId`/token/const-global novo; nenhuma dependência nova.

## 6. O que isto destrava / o gate M2

- **Gate M2** "spring com scrub para trás correto (checkpoint)" — atendido; o loop-wrap é a
  manifestação alcançável hoje (`a_loop_range_replays_the_simulation_from_its_start`, cozido pelo
  registry real).
- Habilita a régua/timeline futura (o `advance_or_scrub` já a atende), o `motion.trail` scrub-safe,
  e é meia-peça da Zona de Simulação (O4).
- **Follow-up honesto:** o edit-while-paused continua no comportamento pré-existente do pump
  (re-cozinha no mesmo tick com `prev_outputs` correntes) — o ring já é limpo, então um scrub
  pós-edit está correto; unificar a semântica edit↔scrub num só modelo é trabalho de quando a
  timeline/régua landar.
