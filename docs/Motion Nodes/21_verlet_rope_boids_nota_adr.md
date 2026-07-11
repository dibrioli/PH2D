# 21 — Nota-ADR: Verlet-Rope + Boids (M4.1 — abre a família de SIMULAÇÃO)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia M4.1 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Primeira fatia da
**família de SIMULAÇÃO** (doc 01 §3, doc 20 §5): a próxima classe de capacidade — **dinâmica sequencial**
sobre a porta de feedback `state` (o `pre` self-loop), depois das distribuições (M3.1–3.2) e deformers
(M3.1–3.3), todos `Pure`/playhead. Dois registros da mesma capacidade nova: **restrição determinística**
(rope) e **emergência** (boids).

---

## 1. O problema (doc 01 §3 — a família de simulação)

M3 deu o vocabulário geométrico (distribuições + deformers), mas tudo era **função pura do playhead** —
nada acumulava estado quadro-a-quadro. A família que faltava é a de **dinâmica sequencial**: agentes/pontos
cujo estado (posição, velocidade) evolui por integração passo-a-passo e cujo comportamento **emerge** do
laço, não de uma fórmula fechada. Os nós stateful que já existiam (`motion.integrate`, `motion.spring`)
eram plumbing do sistema de forças; faltavam as **simulações-primeira-classe** que todo pacote de motion
tem: uma **corda/chicote** física e um **enxame** (flock). Ambos exercitam o mesmo substrato — o `pre`
self-loop com `sim_t`/`dt` — que `motion.integrate`/`spring` estabeleceram.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.verlet_rope` (a corda):** o padrão-ouro de corda/pano é a **integração de Verlet posicional de
Jakobsen — *Advanced Character Physics* (2001)**. Veredito:

- **Verlet guarda posição atual + anterior** (a velocidade é implícita no par), passo
  `x' = x + (x − x_prev)·(1−damp) + a·dt²`, seguido de **passes de relaxação** que puxam cada segmento de
  volta ao comprimento de repouso. É **incondicionalmente estável** (redes de molas explícitas explodem
  quando rígidas) e simples. É exatamente por isso que sims de pano/corda usam Verlet, não molas de força.
- **Restrição por posição** (não por força): cada iteração corrige a distância entre pontos vizinhos; um
  ponto **fixado** (a cabeça; e a cauda com `pin_tail`) segura, o livre leva a correção inteira. Mais
  iterações = mais rígido.
- **A âncora é input de valor** (`anchor_x`/`anchor_y`, animável; desconectado → origem): um `value.lfo`
  desliza o pino e a corda **chicoteia** com follow-through real. HR-5: aritmética + **um `sqrt`** por
  segmento (o comprimento), como `motion.spring`. `Temporal` (lê o playhead p/ `dt`), replay bit-a-bit.

**`motion.boids` (o enxame):** o padrão-ouro absoluto é o **boids de Craig Reynolds (SIGGRAPH 1987)** —
três regras locais que produzem flocking emergente. Veredito:

- **Separação** (afasta de vizinhos próximos, repulsão inverso-do-quadrado), **Alinhamento** (vira p/ o
  heading médio), **Coesão** (vira p/ o centroide) — cada agente olha só vizinhos dentro de um `radius` de
  percepção. Mais um **seek** p/ um alvo (inputs de valor; desconectado → origem): como o seek é uma **mola
  linear**, ele **também é a coleira** — cresce com a distância e mantém o enxame limitado em torno da casa
  (sem fly-away, sem hack de fronteira). Um `value.lfo` no alvo **conduz o bando inteiro**.
- **Vizinhança O(N²)** (all-pairs) — exata e simples para as contagens que um nó produz. **Spatial hash** é
  o truque-padrão de escala e fica como **follow-up puro-de-perf** (§5): não mudaria uma posição emitida.
- HR-5: steering é add/multiply + **um `sqrt`** por normalização (heading/clamp de velocidade); o seed é o
  hash splitmix (Jarzynski/Olano, estável). `Temporal`, replay bit-a-bit.

## 3. O que foi adicionado (fatia M4.1)

**`ph2d-node-motion-verlet-rope` (drop-crate, a CORDA):** `(anchor_x?, anchor_y?, state) → out`. `count`
pontos ligados por segmentos de comprimento fixo, Verlet + relaxação; cabeça fixada na âncora animável,
gravidade puxa numa catenária. Params: `count`/`length`/`gravity`/`iterations`(rigidez)/`damping`/`pin_tail`.
Sequencial (`pre` self-loop), `Temporal`. `NodeUiCategory::Source`. Display "Verlet Rope".

**`ph2d-node-motion-boids` (drop-crate, o ENXAME):** `(target_x?, target_y?, state) → out`. `count` agentes,
flocking de Reynolds (sep/align/cohesion) + seek p/ o alvo (que também limita). Params:
`count`/`seed`/`radius`/`separation`/`alignment`/`cohesion`/`seek`/`max_speed`. Sequencial (`pre` self-loop),
`Temporal`. `NodeUiCategory::Source`. Display "Boids".

**Cena boot — DUAS cenas pequenas lado a lado** (`motion_demo_strobe.rs`, 8 nós, 2 sinks):

```
ESQUERDA (chicote):  rope  → tint(âmbar) → output       lfo_anchor → rope.anchor_x
                     rope --pre--> rope.state
DIREITA  (enxame):   boids → tint(ciano) → output       lfo_target → boids.target_x
                     boids --pre--> boids.state
```

- **rope** (x≈−6): a gravidade pendura a corda, o `anchor_x` (`value.lfo` ±1.5) desliza o pino → **chicoteia**.
- **boids** (x≈+6): o bando cai em torno da casa, o `target_x` (`value.lfo` ±3) desliza o alvo → o enxame
  inteiro **roda pra persegui-lo**.

Duas simulações sequenciais (uma restrita/determinística, uma emergente), cada uma conduzida pelo domínio
de valor, em regiões separadas do canvas. Mantida **pequena e legível** (duas cenas de ~4 nós, isolando os 2
nós novos — regra do doc 12 / feedback "simplifique o exemplo"). Os `motion.output` compõem num só draw
(vários sinks; o bridge já suporta).

**Testes (18 unit + 3 integração):** rope (9: seed reto no tick 0, **restrição segura o comprimento** [180
ticks sob gravidade; sem relaxação estica sem limite], cauda cai, âncora móvel fixa a cabeça + arrasta a
corda, `pin_tail` sagueia entre as pontas, replay determinístico, sem-loop = reto/parado, NaN recupera, cook
com pre-loop — falsificados); boids (9, inclui 2 do hash: **coesão junta** [vs inércia que espalha],
**alinhamento alinha headings** [coerência → alta vs baixa], **separação impede colapso** [distância mínima ≫
sem separação], **seek persegue o alvo** [centroide migra p/ o alvo], **mola seek limita** [600 ticks
finitos < raio], replay + seed re-rola, cook com pre-loop — falsificados). Integração no shell:
`the_verlet_rope_whips_from_the_moving_anchor` (a cauda cai < −1 e varre lateralmente; rope morto a prende) ·
`the_boids_flock_seeks_the_moving_target` (centroide homed à direita + rastreia o alvo deslizante) ·
`the_default_document_replays_deterministically` (drena o pump sequencial completo — prova que as sims
avançam reprodutivelmente, não só os nós puros).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-verlet-rope`, tipo `motion.verlet_rope` | nova | nome novo |
| crate `ph2d-node-motion-boids`, tipo `motion.boids` | nova | nome novo |
| `hash.rs` (copiado de `motion.scatter`) em boids | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (48 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita p/ rope+boids, 2 cenas, 8 nós) | shell | módulo Motion |
| `motion_state.rs` (`build`→`Vec` de 2 sinks) + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de
`ph2d-nodegraph` + `ph2d-node-registry` (machete verde). **Nota p/ o integrador:** o `build` da cena passou
a devolver **dois** sinks (antes um) — a mudança está contida no shell do Motion.

## 5. O que fica (M4 + M3, doc 01 §3)

- **Simulação (continuação):** `motion.soft_body` (XPBD — malha 2D deformável) · `motion.pin_constraint`
  (fixa instâncias arbitrárias de um stream simulado) · **spatial hash** no boids (só-perf, O(N²)→O(N)).
- **Distribuições restantes (M3):** `motion.distribute-voronoi` (Lloyd) · `-path` (integra vector.*) ·
  `motion.lattice`.
- **Deformers restantes (M3):** `motion.four-point-warp` · `-slit-scan`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

> M4.1 abre a **dinâmica sequencial de primeira classe**: uma corda restrita (Verlet/Jakobsen) e um enxame
> emergente (boids/Reynolds), ambos no substrato `pre`/`sim_t` que `motion.integrate`/`spring` firmaram. O
> que resta na simulação é o corpo mole (XPBD), constraints avulsas e a otimização de vizinhança; e as
> distribuições/deformers avançados do M3.
