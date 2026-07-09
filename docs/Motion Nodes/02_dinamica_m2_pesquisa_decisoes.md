# M2-Dynamics — pesquisa e decisões (forças + integrador + spring, SEM timeline)

**Data:** 2026-07-09 · **Linha:** `line/MotionNodes` · **Contexto:** keyframes/timeline do Motion
foram **adiados** (Enio, 2026-07-09 — a timeline nasce em outra linha; o Motion integra no fim).
O próximo bloco do plano ([`01_plano_modulo_motion_nodes.md`](01_plano_modulo_motion_nodes.md) §M2)
que não depende de timeline é o **motor de dinâmica**: forças + integrador + spring.
`cook_scoped`/`checkpoint` (M2.N1-N4) ficam pro bloco seguinte — checkpoint serve scrub,
e scrub é UI da timeline.

## Norma §2 — comparação em três colunas

### 1. Arquitetura de forças

| O que eu pensei sozinho | O que o MiniCavalryV2 faz | Padrão-ouro da indústria |
|---|---|---|
| Forças acumulam numa coluna `accel` do stream; UM nó sequencial integra (o que o plano §1.6 já decidiu). | Cada força é um "forceModule" com **estado oculto por-nó** (`node._state`, keyed por pid): integra a própria contribuição (`s.v += f/m·dt; s.d += s.v·dt`) e emite `x = in.x + s.dx` (`helpers.js:1156` `_forceProcess`). dt vem de wall-clock clampado a 0.05; regressão de tempo reseta o estado. | Houdini POPs: microsolvers de força **acumulam** no atributo `force`; o [POP Solver](https://www.sidefx.com/docs/houdini/nodes/dop/popsolver.html) "updates particles for one timestep" integrando uma vez ([POP Force](https://www.sidefx.com/docs/houdini/nodes/dop/popforce.html)). Semi-implícito (symplectic) Euler é o integrador padrão de partículas — preserva o Hamiltoniano sem dissipação ([TinyDEM, arXiv 2507.12610](https://arxiv.org/pdf/2507.12610)). |

**Decisão:** accumulate-then-integrate (plano §1.6 = Houdini), **não** o modelo do MVP. Estado
oculto por-força quebra replay (HR-5), duplica integradores e não loweriza pra GPU. A força vira
nó `Pure` que soma em `accel × falloff`; `motion.integrate` é o único sequencial.

### 2. Semântica de composição (o que o integrador emite)

| Eu | MVP | Padrão-ouro |
|---|---|---|
| 1ª ideia: o loop possui `P` (estado = P+vel); upstream só semeia. Problema detectado: animação upstream (oscillator/tint) congela após o seed. | `x = inst.x + s.dx` — o deslocamento acumulado **compõe com o upstream vivo** (`helpers.js:1238`): wind empurra uma grade que continua oscilando. | Motion design (Cavalry/AE): efeitos compõem sobre o rig vivo; sims puras (Houdini) possuem o estado. Para um módulo *motion*, compor é o comportamento útil. |

**Decisão:** `motion.integrate` carrega **deslocamento** `sim_d` (não P absoluto):
`out.P = rest.P(vivo) + sim_d`; colunas não-simuladas (tint/size/uv/falloff) fluem **do rest vivo**
a cada tick. Upstream continua animando; a física compõe por cima — igual MVP, sem estado oculto.
Count mudou → re-seed (sim_d=0, vel=0), documentado.

### 3. Topologia do loop no substrato (pre de 1 tick)

| Eu | MVP | Padrão-ouro |
|---|---|---|
| `rest --fwd--> integrate.rest`; `integrate.out --pre--> força₁.in`; `forçaₙ.out --fwd--> integrate.state`. Sem forças: self-loop `integrate.out --pre--> integrate.state`. Tick 0: pre=Empty → seed do rest. | Não tem `pre` — o estado oculto É o loop. | Lustre/`pre` (ADR-0032): feedback = delayed edge; TouchDesigner: feedback explícito. |
| Editor: usuário não consegue criar edge `pre` (Connect é sempre forward e ciclo é recusado). | n/a | n/a |

**Decisões:**
- **Auto-pre no ciclo:** `Connect` que falha com `WouldCycle` é re-tentado como `delayed: true`
  + toast informativo. Um ciclo só tem UM significado legal no substrato (feedback `pre`) — o
  editor cria o significado em vez de recusar. Vira o jeito genérico de fechar qualquer loop.
- **Template de nó sequencial** (plano §M2 editor F2): `AddNode` de um tipo cujo manifest tem
  input chamado `state` com o mesmo PortType do output 0 → self-loop `pre` já ligado.
  Convenção append-only: qualquer nó sequencial futuro só precisa nomear a porta `state`.

### 4. dt determinístico (a correção mais importante do bloco)

| Eu | MVP | Padrão-ouro |
|---|---|---|
| Detectado no bridge: `advance(frame_ticks)` + **um** pump por frame → frame com 2 ticks fixos avança o `pre` UMA vez com playhead pulando 2·dt. Trajetória depende do frame rate = não-determinístico. | dt = wall-clock delta clampado (0.033/0.05) — explicitamente a fraqueza que o plano corrige ("wall-clock default → tick fixo"). | Fixed timestep canônico (Gaffer On Games; todo engine de física): 1 step de sim por tick fixo, dt constante. |

**Decisões:**
- **Bridge cozinha por tick fixo:** `for _ in 0..frame_ticks { advance(1); pump(...) }` + um
  pump final (edição com transport pausado continua cozinhando). Custo igual no caso normal
  (frame_ticks=1); frame lento re-sima N passos (determinismo > custo, e N é pequeno).
- **dt dentro do nó = `playhead − sim_t[0]`** clampado a `[0, 0.1]`: `sim_t` é coluna de estado
  emitida pelo próprio nó. Zero constante mágica cross-crate (nada de `1/60` hardcoded), robusto
  a mudança do fixed_dt do shell, e `EvalCtx` (substrato) fica intocado.

### 5. As forças (semântica portada por comportamento, clean-room)

| Força | MVP (`src/nodes/forces/*.js`) | Porte PH2D (Y-up, unidades de mundo ~10 u de altura) |
|---|---|---|
| Attractor | radial, `strength·(1−d/R)^power`, d<1 ou >R → 0, repel inverte, mass-multiplied | igual, mas `pow` livre é transcendental (HR-5) → **curva Enum Linear/Quad/Smooth/Smoother** (mesmo vocabulário do `motion.falloff`); sem coluna `mass` no v1 (≡ mass=1) |
| Vortex | tangencial `(−dy,dx)`, falloff linear, clockwise; +Y-down → sign+1 = horário | mundo é **Y-up** (`camera.rs:3`) → horário visual = `(dy,−dx)`; teste ancora a direção |
| Wind | direcional + rajadas `perlin2(t·gust, seed_i·0.5)`, NÃO mass-multiplied | direcional (Angle em graus, 0°=+X CCW) + rajadas com o value-noise do wiggle (hash+polinômio, HR-5); cobre Gravity (gust=0, angle 270°) — **crate gravity não existe**, documentado no wind |
| Drag | `a = −k·v_combinada` (lê `inst.vx + s.vx` pra frear upstream) | `a = −k·vel` — com integrador único a `vel` do estado JÁ é a velocidade total (a soma do MVP era workaround do estado fragmentado) |
| (todas) | falloff só no spring (`FalloffStrength`) | **toda força multiplica pela coluna `falloff`** (plano §1.6) — o campo de foco gateia a física |

### 6. Spring

| Eu | MVP (`spring.js`) | Padrão-ouro |
|---|---|---|
| Portar literal o sub-step adaptativo (o plano §1.7 manda) | canal X/Y/Rot/Scale/Size; estado `{value, velocity}` por chave; **semi-implícito Euler com sub-step**: `idealSubDt = sqrt(0.05/tension)`, `steps = ceil(dt/subDt)`; `accel = −friction·v − tension·(value−target)`; falloff lerpa saída; NaN guard reseta; regressão de tempo limpa | sub-step por limite de estabilidade (`subDt²·k < ε`) é a técnica padrão para molas stiff com Euler explícito/semi |

**Decisão:** porte literal do algoritmo (sqrt é IEEE, permitido), estado como colunas
(`spring_value`, `spring_vel`, `sim_t`) no self-loop `pre`; canal Enum X/Y/Rot/Size
(identidade do size = `[1,1]`, lição da linha); alvo = canal do input **vivo** (spring persegue
o que anima — a nota do MVP "só age sobre alvos que MUDAM" vira o demo: spring atrás do oscillator).

## Vocabulário novo (anotar no handoff — risco de colisão entre linhas)

- **Crates:** `ph2d-node-motion-integrate` · `ph2d-node-motion-spring` ·
  `ph2d-node-force-attractor` · `ph2d-node-force-vortex` · `ph2d-node-force-wind` ·
  `ph2d-node-force-drag`
- **Type names:** `motion.integrate` · `motion.spring` · `force.attractor` · `force.vortex` ·
  `force.wind` · `force.drag`
- **Colunas de convenção:** `vel` (Vec2, §1.2 do plano — agora tem produtor) · `accel` (Vec2,
  transiente por-tick: forças somam, integrate consome e DROPA) · `sim_d` (Vec2, deslocamento
  acumulado do integrate) · `sim_t` (Scalar, playhead do último eval — deriva dt) ·
  `spring_value`/`spring_vel` (Scalar, estado do spring)
- **Convenção de porta:** input `state` (mesmo tipo do output 0) = porta de feedback; ganha
  self-loop `pre` automático no AddNode
- **Bridge:** cook por tick fixo; `WouldCycle` → retry `delayed` + toast
- **Contratos congelados:** intocados (NodeOp/OpResolver/NodeManifest/Tool/…); `EvalCtx`/`Cook`
  intocados; categorias/ícones/tokens: nenhum novo (categoria Transform, precedente wiggle)

## Fora deste bloco (próximos)

`motion-delay`/`-trail` (History ring) · particle-emitter (spawn determinístico) · pulse family ·
`motion-noise` Perlin · time-remap (`cook_scoped`, M2.N1) · checkpoints p/ scrub (M2.N2, junto
com a timeline) · coluna `mass` · alvo dinâmico por porta point (attractor `input` mode do MVP).
