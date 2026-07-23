# HANDOFF / Tracker — `line/physics` (o motor de física global)

> ⚠️ **VAI ASSUMIR ESTA LINHA? Comece por
> [`HANDOFF_REABERTURA_line_physics_2026-07-22.md`](HANDOFF_REABERTURA_line_physics_2026-07-22.md)** —
> a jornada de 21/07 (mais 21 waves, W6 → W-FormDrag) **INTEGROU** ao `main`, com todos os smokes
> aprovados. Aquele doc te diz como REABRIR a worktree, o que já existe (para não reconstruir) e o
> plano. Este tracker é o estado **por-wave**, para consulta pontual — não leitura linear.
>
> (O `HANDOFF_CONTINUACAO_line_physics_2026-07-19.md` era o equivalente da jornada anterior e está
> **vencido**: o plano dele foi todo executado.)
>
> **Tracker VIVO do módulo** (o `docs/HANDOFF_*` da física). Toda jornada futura **atualiza este
> arquivo**: estado por-wave, decisões, gotchas, ids/consts alocados. LLM nova lê ISTO + a
> [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) +
> [`00_plano_waves.md`](00_plano_waves.md) antes de tocar código.
>
> **Bugs cuja causa ENGANAVA:** [`BUGS_physics.md`](BUGS_physics.md) — sintoma → causa-raiz →
> tentativas que falharam → lições. Leia antes de re-diagnosticar qualquer coisa deste módulo.
>
> **Norte (não re-litigar):** runtime-truth + bake opcional; rígido primeiro; solver = `rapier2d 0.28`
> (M10, já determinístico) — esta linha escreve **integração e autoria**, não solver.

---

> **✅ INTEGRADA NA `main` (2026-07-18).** W0 · W1 · W1.5 · metade do W2, com todo o smoke aprovado.
> Handoff usado: [`HANDOFF_INTEGRACAO_line_physics_2026-07-18.md`](HANDOFF_INTEGRACAO_line_physics_2026-07-18.md)
> (histórico — os números de identidade dele são do dia da entrega, não do baseline atual).
>
> **⚠️ Dois números MUDARAM na integração, e é assim que tinha de ser:**
> - **`PROJECT_SCHEMA` = 18, não o 17 que a linha entregou.** Recontado: o 17 desta linha + o bump da
>   `line/FLIP` na mesma janela. É a regra *"o valor se CONTA, não se escolhe"* funcionando —
>   escolher um dos lados faria os saves do outro passarem na checagem de versão e serem lidos com o
>   layout errado. A tripla-pin é **`(18, 8, 8)`**. **W3 bumpa 18 → 19.**
> - **O ADR virou [0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md).**
>   O 0130 tinha dois donos (a `line/gpu-nodes` também o reivindicou) — 2ª vez que isso acontece no
>   repo. **Um número de ADR escolhido numa linha paralela é PROVISÓRIO até integrar**, e todas as
>   referências dele em doc-comment viram custo de rename. Já não há referência órfã ao 0130 nesta
>   linha (verificado por grep + o gate `architecture_adr_numbers_are_unique`).
>
> **Verificado na árvore integrada (não presumido):** `cargo check --workspace --all-targets` limpo ·
> as 4 suítes do módulo verdes · seam do Inspector 7/7 · **os dois hashes C9 BYTE-IDÊNTICOS** aos da
> entrega (`2f7e2d58…` / `54fea296…`) ⇒ a física atravessou o merge sem mover um bit.

> **▶️ O W2b FECHOU (2026-07-18).** O painel global existe, está fiado nos 5 sites + os 4 do scroll,
> persiste no arquivo de projeto e tem cena de smoke própria (`PH2D_PHYSICS_SMOKE=4`). A **tarefa
> zero também fechou**: o `CLAUDE.md` §5 e o roteador §1 agora apontam para `docs/Physics/`.
> Detalhe em **§W2b** abaixo; o handoff de integração é
> [`HANDOFF_INTEGRACAO_line_physics_W2b_2026-07-18.md`](HANDOFF_INTEGRACAO_line_physics_W2b_2026-07-18.md).
> A próxima wave é **W2c** (camadas de colisão) ou **W3** (joints) — ordem do Enio.

## Estado por-wave

| Wave | Estado | Commit | Nota |
|---|---|---|---|
| **W0 — Arquitetura** | ✅ **INTEGRADO** | `456e8b99` | ADR-0131 + plano de waves + tracker + visão. **Zero código.** |
| **W1 — Ponte ECS + tick + hash** | ✅ **INTEGRADO** (smoke aprovado) | `44e08cf5`→`9f5fee05` | o alicerce — ver §W1 abaixo |
| **W1.5 — Scrub (checkpoint ring)** | ✅ **INTEGRADO** (smoke aprovado) | ver §W1.5 | kill-check passou de primeira; stride MEDIDO |
| **W2a — Inspector body** | ✅ **INTEGRADO** (smoke aprovado) | ver §W2 | a autoria |
| **W2b — Painel global de mundo** | ✅ **INTEGRÁVEL** — smoke **APROVADO** (2026-07-18) | ver §W2b | gravidade/solver/ar/damping/sono + persistência |
| **W2c — Camadas de colisão** | ✅ smoke aprovado | ver §W2c | matriz no painel + camada no Inspector |
| **W3 — Joints** | ⏭️ **A PRÓXIMA** | — | pêndulo/corrente/ragdoll; bumpa o schema **21 → 22** |
| **W4 — Bake-to-timeline** | ✅ smoke aprovado | — | acopla `ph2d-anim` (outra linha) |
| **W6 — A escala alcança o collider** | ✅ **INTEGRÁVEL** — smokada pelos gates (2026-07-19) | ver §W6 | a única CORREÇÃO do cardápio; `ShapeDesc::Ellipse`; **zero bump de schema** |
| **W7 — Sensores / triggers** | ✅ **INTEGRÁVEL** — smoke `=10` (2026-07-19) | ver §W7 | o primitivo de trigger (item B); `Collider.is_sensor`; `PROJECT_SCHEMA 26→27`; o **sinal de gameplay** fica pro Enio |
| **Weld — `FixedJoint`** | ✅ **INTEGRÁVEL** — smoke `=11` (2026-07-19) | ver §Weld | o 5º joint (polimento C do W3); trava rígido; `PROJECT_SCHEMA 27→28` |
| **Bake — seleção de canais** | ✅ **INTEGRÁVEL** — gate + seam (2026-07-19) | ver §BakeChannels | seletor All/Position/Rotation no §11 (polimento D); layering; transiente (não salvo) |
| **W8 — Gravity Scale por corpo** | ✅ smoke `=12` (2026-07-19) | ver §W8 | multiplicador de gravidade por corpo; componente opcional; sem bump |
| **Capsule — collider de personagem** | ✅ smoke `=13` (2026-07-19) | ver §Capsule | 3ª forma `ColliderShape::Capsule`; uniforme exata / não-uniforme = Stadium |
| **W9 — Velocidade inicial por corpo** | ✅ smoke `=14` (2026-07-19) | ver §W9 | lançamento linear+angular no spawn; seta amarela; sem bump |
| **W-CCD — Detecção contínua por corpo** | ✅ **INTEGRÁVEL** — smoke `=15` (2026-07-20) | ver §W-CCD | toggle Discrete/Continuous; corpo rápido não tunela parede fina; marcador `Ccd`; sem bump |
| **W-LockRot — Freeze Rotation por corpo** | ✅ **INTEGRÁVEL** — smoke `=16` (2026-07-20) | ver §W-LockRot | toggle Rotation Free/Locked; personagem não tomba; marcador `LockRotation`; sem bump |
| **W-Offset — Offset do collider por corpo** | ✅ **INTEGRÁVEL** — smoke `=17` (2026-07-20) | ver §W-Offset | Offset X/Y; collider nos pés, não no centro; campo no `Collider`; **bump 28→29** |

**W0 entregou:** [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) ·
[`00_plano_waves.md`](00_plano_waves.md) · [`01_visao.md`](01_visao.md) · este tracker. Nenhuma linha de
código, nenhum contrato tocado, nenhum foundational tocado.

---

## §W1 — O alicerce LANDOU (2026-07-17, **smoke aprovado**)

Um sprite com `RigidBody{Dynamic}` + `Collider` **cai e assenta** sobre um `Collider{Static}` no ECS real,
ao dar play, e o mundo é determinístico. A ponte promoveu o wrapper M10 de dormente a **wired e global**.

**Crate-ponte nova `ph2d-physics-ecs`** (glob `crates/*` a pega — zero edit central): components
`RigidBody{kind}` + `Collider{shape,density}` (**config only** — nunca estado vivo de solver, senão o
`canonicalize` do undo diffaria um passo espúrio por tick); `PhysicsBridge` (owns `PhysicsWorld` +
`BTreeMap<Entity, handle>` + `last_stepped`); `register_physics_components`; `deterministic_hash` sobre os
`Transform` do readback; bin `physics_ecs_c9`.

**A ponte (`bridge.rs`), o coração:** `dispatch(sim, playing, target)` — **play** = `reconcile_structure`
(spawn/remove em ordem entity-sorted, HR-5) + `step()×(target−last_stepped)` sequencial + `readback`
(pose→`Transform`, só corpos Dynamic); **paused** = `settle` (corpos seguem o `Transform` autorado,
read-only no Transform ⇒ frame parado não gera passo de undo). `QueryState` cacheado (zero-alloc, idiom do
`propagate_transforms`). O `BTreeMap` (não `HashMap`) é a **espinha do determinismo**: itera por `Entity`,
ordem estável per-run e cross-OS; a lint disallowed-`HashMap` é o guarda estrutural.

**`ph2d-physics` estendido (append-only, meu módulo):** `spawn_body(BodyDesc)`/`set_body_pose`/`remove_body`
+ `BodyDesc`/`ShapeDesc` — cobre os 4 combos body×shape. **Os helpers existentes + `step` + o hash c9
ficaram byte-idênticos** ⇒ o gate M10 (`physics-c9`) segue verde (`2114f483…`).

**Escala (D4 CORRIGIDO medindo):** o `Transform` já é METROS (Y-up, radianos CCW); rapier é metros ⇒
**fronteira 1:1, sem conversão**. A única px→m já existe: `ProjectSettings.pixels_per_meter` (default 100)
no import, do PROJETO. **NÃO** criei um 2º `PIXELS_PER_METER` (seria a 2ª porta que diverge).

**Shell wired:** `AppGfx.physics: PhysicsBridge` (ao lado de `sim`/`motion`); `render_loop/physics_bridge::
dispatch` chamado **antes de `sim_extract`** (mod.rs, corpo renderiza same-frame; `target =
round(playhead.time()/dt)`, `playing = is_playing`); `register_physics_components` no boot (`init.rs`);
smoke `physics_smoke.rs`. **Persistência:** `PROJECT_SCHEMA` **15→16** + a **tripla-pin** de
`project_tests` para `(16,7,8)` (o gate disparou no bump — o valor se CONTA); `physics.rebuild()` no reset
de load (mundo é derivado — D2; reconcile self-heal é o backstop).

**Gates (6, todos mutation-verified RED-first):** e2e falls-and-settles (kill readback→RED) · determinismo
repeatability (guarda estrutural = BTreeMap + lint + CI cross-OS) · zero-alloc capacity (kill `seen.clear()`
→RED) · registry count=2 (kill um register→RED) · round-trip de snapshot (kill um register→RED) · self-heal
no respawn (kill remoção de stale→RED). **CI:** `physics-ecs-c9` na matriz do `spike.yml` + compare cross-OS
(`sort -u | wc -l`) — mirror do `ph2d_physics_c9`.

**Batched gate verde:** fmt · clippy `--all-targets` · `cargo check --workspace` · `nextest-impacted` (723
passed, 5 skipped).

**⚠️ SMOKE (Enio):** `cd Worktrees/line-physics && env PH2D_PHYSICS_SMOKE=1 cargo run -p ph2d-host-desktop`

> ⚠️ **O `env` não é enfeite.** O shell do Enio é **fish**, onde `VAR=valor comando` **não** define a
> variável — ela é ignorada em silêncio, a cena nunca monta, e o app abre normal como se nada tivesse
> sido pedido. Um comando de smoke que falha calado é o pior tipo. `env` funciona em fish e em bash. —
uma bola laranja (dynamic) deve **cair e assentar** sobre a barra cinza (static floor). Ponte morta = bola
pendurada no ar.

**Transport Play/Pause/Reset (2026-07-18, aprovado o smoke da física):** os 3 chips da TopBar estavam
**pintados e inertes** (o clique só imprimia o nome). Agora dirigem o **Playhead** (física/motion/timeline/
flip andam juntos). `EditorAction::Transport(TransportCmd{Play,Pause,Reset})` (editor-core, append,
non_exhaustive) + `chrome/transport.rs` (handler z=300, regen do `dispatch_all` pelo `ph2d-chrome-sync`) →
dreno no shell chama a **porta única** `shells/desktop/src/transport.rs::apply(cmd, &mut Playhead)` (Reset =
`rewind` + `pause`, porque `rewind` sozinho mantém o play state). 2 gates mutation-verified (o clique via
`dispatch_all` levanta o comando certo; o mapeamento muda o Playhead).

**⚠️ E a FÍSICA não obedecia (Enio 2026-07-18: *"funcionou para timeline mas não para a física"*) — 2
defeitos reais, corrigidos:** (a) o `dispatch` só andava pra FRENTE, então relógio pra trás era ignorado —
Reset deixava a bola no chão e o transport parecia morto. Agora o `dispatch` é **função do TICK**
(`target < last` = replay · `>` = step · `==` = hold): rapier não rebobina, então **cada corpo carrega o
`BodyDesc` do spawn** (`BodyRef.rest`, a pose em tick 0) e `rewind_to` reconstrói um mundo novo a partir
deles e re-simula `target` passos. **Reset (target 0) custa zero passos; scrub-back passou a funcionar**, a
O(target) — o ring que torna isso O(1) amortizado segue **W1.5**. (b) o `settle` teleportava em TODO frame
pausado, e `set_body_pose` **zera a velocidade** ⇒ Pause→Play recomeçava a queda parada; agora só teleporta
quando o `Transform` autorado de fato **difere** do corpo (o gesto do gizmo — o caso que ele existia pra
servir). Gates: `resetting_the_clock_returns_the_body_to_its_rest_pose` (matar o ramo de trás → bola fica
em y=0,35 → RED, o bug exato reportado) · `pausing_mid_fall_does_not_change_the_trajectory` (teleporte
incondicional → a corrida pausada cai menos → RED). **A cena do smoke é só a SIMULAÇÃO** —
`PH2D_PHYSICS_SMOKE` pula as 8 entidades demo do boot (`init.rs`), então a Hierarchy mostra só o chão + a
bola.

**Deferido (por design, não esquecido):** scrub-back re-sim = **W1.5** (o `settle` seta `last_stepped=target`
no paused; scrub não rebobina o corpo ainda — o ring é a próxima wave, com o **kill-check de serialização
do rapier ANTES do build**). restituição/atrito/damping/Kinematic/camadas = **W2** (append + wire no painel).
`readback` só trata corpo root (Transform local = mundo); corpo filho = W2 (via `parent_world_transform`).
`reconcile` stale é O(N²) (trivial nos counts de W1).

---

## §W1.5 — O relógio pra trás LANDOU (2026-07-18, **smoke aprovado**)

Arrastar o playhead pra trás re-simula **bit-exato** sem custo O(t). rapier não anda pra trás (**nenhum**
motor anda — resolução de contato não é invertível), então é GGPO save/load/advance, o mesmo desenho do
`Cook::checkpoint`/`CheckpointRing` do Motion.

**O kill-check passou de primeira, e a 2ª metade dele decidiu o desenho.** Os 8 tipos cross-frame do
rapier são `Clone` ⇒ **sem `serde-serialize`, sem bincode**. O `PhysicsPipeline` — o único campo que o
`step()` muta e que **não** é `Clone` — é *workspace* (buffers de manifold/constraint + counters), e é por
isso que os snapshots do próprio rapier serializam os SETS e reconstroem o pipeline. Isso não foi
acreditado: o gate de bit-exatidão ficaria vermelho em todo tick de âncora se houvesse estado real ali.

**O stride é MEDIDO, não chutado** (`tests/measure_checkpoint.rs`, dhat + timing):

| | 50 corpos | 200 corpos |
|---|---|---|
| checkpoint | **59,4 KB** · 11,2 µs | 229,6 KB · 40,0 µs |
| um `step()` | 7,3 µs | 46,3 µs |

⚠️ **Um checkpoint custa ~UM step.** A regra do GGRS (*denso a menos que a cópia domine `K × re-sim`*) leva
o Motion a **denso** — estado pequeno, cook barato — e leva a física ao **oposto**: denso **dobraria o custo
do play** (contra os 1,5 ms de HR-4) e gastaria **17,4 MB dos 20 MB** de HR-13 em 5 s de janela.
**`STRIDE = 10`**: play +10%, janela 1,74 MB, pior caso do scrub = 10 steps (~0,07 ms, abaixo da percepção
— a única coisa que um scrub deve a alguém).

**O cap é em BYTES, não em contagem** (`DEFAULT_BUDGET_BYTES = 8 MB`) — a lição do ADR-0117: contagem é
**multiplicador**, não teto (uma cena de 5000 corpos estouraria um ring de 30 checkpoints com o número
parecendo tranquilo). Cena pesada ganha janela mais CURTA, não conta maior. Medido: 10 min de sim →
595 checkpoints, **7,99 MB**.

**O fallback É o produto, não uma 2ª implementação:** miss devolve `None` e o chamador cai no
`rebuild_from_rest` — o caminho que já shipou no W1 e já tinha gate. **Apague o ring e o produto ainda
scrubba, só mais devagar.** Nada pra divergir (mesma forma do fallback de splice do ADR-0124).

**Invalidação (cada camada com gate PRÓPRIO):** spawn/remove de corpo (`reconcile_structure`) · `set_gravity`
· `rebuild` (load/undo) · `rebuild_from_rest` (handles novos). Restaurar um checkpoint de um body-set
diferente devolveria handles que não endereçam mais as entidades que a ponte segura — e a pose publicada
seria **stale em silêncio**, o pior tipo de errado.

### 2 bugs de autoria fechados junto (achados construindo os gates)

1. **`rest` era a pose do SPAWN, congelada** ⇒ mover um objeto e apertar **Reset** jogava fora o
   posicionamento do artista e pulava de volta pro lugar original. **A regra que fecha: a pose de repouso é
   a pose AUTORADA no tick 0** — lida todo frame, não lembrada (cobre de graça shape/densidade editados no
   Inspector, W2: re-descrever o corpo é UMA regra em vez de uma lista crescente de campos a vigiar). Tem
   gate irmão provando que a regra **não** dispara com o relógio andando (senão o `Transform`, que ali é a
   SAÍDA da sim, seria realimentado e o corpo renasceria a cada frame, perdendo a velocidade).
2. **Uma linha defensiva que eu quase shipei e REMOVI:** um `ring.clear()` no `settle` quando o artista
   arrasta um corpo pausado. Construindo o gate, não achei o caso em que ela muda o resultado: com o ring
   sujo o scrub restaura um checkpoint pré-arrasto; com o ring limpo o fallback re-simula do repouso — **os
   dois descartam o arrasto igualmente**. Defesa que não se observa é comentário que mente. No lugar dela,
   a semântica está DOCUMENTADA: **a sim é função de `(tick, repouso autorado)`, então um empurrão no meio
   é transiente e qualquer rewind o descarta** (Unity/Godot descartam edições de play-mode pelo mesmo
   motivo; fazer uma pose do meio GRUDAR é autorar keyframe = o bake do W4).

### O oráculo que quase passou (a lição desta wave)

O gate 1 nasceu comparando **o endpoint** e ficou **VERDE** sob uma mutação real (`restore` sem o
narrow_phase). Motivo: uma pilha assentando é um sistema **amortecido** — ele **esquece** a perturbação e
re-converge pro mesmo repouso, então o tick 137 concordava e os ticks do meio não. **O scrub que o artista
assiste é o CAMINHO, não o destino** ⇒ o oráculo virou a trajetória inteira, e aí a mutação sangra.
Corolário: tirar o `broad_phase` do restore **sobreviveu** a 2 fixtures independentes (pilha + cena de
espalhamento a 9 m/s, onde um índice espacial obsoleto daria pares errados) ⇒ o BVH é **derivado**, não
autoritativo. Fica no checkpoint (um snapshot deve ser completo, e a memória já está orçada), mas isso
agora está **medido**, e ninguém precisa re-litigar por prosa.

**Gates (11 novos):** `ph2d-physics/tests/checkpoint.rs` (6) + `measure_checkpoint.rs` (1, dhat) ·
`ph2d-physics-ecs/tests/scrub.rs` (5) + `authoring.rs` (2). **8 mutações, 8 sangram no gate certo**
(a 9ª — `broad_phase` — é nula e está documentada acima). O gate de O(K) **CONTA steps**, não cronometra
(`PhysicsBridge::steps_taken`): a alegação é sobre quanta simulação um scrub re-roda, e step é exatamente
essa grandeza — sem skew do perfil `ci-test`, sem flake.

**Smoke: `PH2D_PHYSICS_SMOKE=2`** — 12 corpos caem numa pilha (⚠️ a cena é uma PILHA de propósito: é onde um
scrub errado é *visível* — no meio da queda os corpos estão espalhados no ar, assentados são um monte). Abre
o painel de timeline sozinho. Deixe assentar e **arraste a régua pra trás**.

### O CONTORNO DO COLLIDER (2026-07-18, smoke do Enio: *"os colliders parecem redondos mas os desenhos são box"*)

Parecia bug do demo; **é o caso NORMAL**. Um sprite é um QUAD texturizado e um collider é **invisível**,
então uma bola sob um sprite quadrado é indistinguível de uma caixa sob o mesmo sprite — até rolar. Num
projeto real a arte é o que o artista desenhou e o collider é a forma que ele escolheu; os dois só se
relacionam por intenção. Deixar o *sprite* redondo consertaria só o demo (e nem dá: o renderer desenha
quads, não há círculo no atlas).

**A resposta é a que todo editor de física dá** — Unity, Godot e o debug draw do próprio Box2D pintam o
collider como wireframe sobre a arte: `render_loop/physics_overlay.rs`. Contorno por corpo, **verde =
estático / ciano = dinâmico** (a 1ª pergunta que se faz a uma cena de física é *"quem aqui se move?"*, e sem
cor ela não tem resposta na tela). Bola ganha **raio-guia** — o contorno é simétrico por rotação, então sem
ele um círculo rolando é idêntico a um parado, e rolar é justamente o que o collider existe pra produzir
(o debug draw do Box2D carrega o mesmo raio, pelo mesmo motivo). Toggle **`B`** (tecla livre desde que o
W4.T5 da timeline aposentou a demo de `SpriteAnimation`), **default ON** como os gizmos do Unity: uma coisa
invisível que você está autorando não pode ser julgada. **Cena sem corpos não desenha nada e não custa
nada**, então usuário de painter/vector nunca vê chrome de física.

⚠️ **Geometria em px de TELA, sob `Affine::IDENTITY`** — os PONTOS sobem pela câmera, a espessura não. No
Vello o transform do `stroke` **multiplica a largura**: passar o afim mundo→tela transformaria 1,5 px em
`1,5 × px_por_unidade_de_mundo`. Isso é cicatriz, não hipótese — foi o que virou o realce do Flip num borrão
que cobria o desenho (smoke, 2026-07-13); o `flip_cursor` sempre desenhou assim por isso.

**A decisão `outlines()` é PURA** (padrão `hit_plan`): o toggle e o *"há física aqui?"* são respondidos e
devolvidos como dado, não resolvidos dentro do laço de pintura — recusa que mora num laço não se testa, e
overlay que desenha depois de desligado é o que ninguém nota até estar num screenshot.

**As cenas de smoke pararam de mentir:** todo collider casa com o quad do seu sprite (só cuboides). A cena 2
usa dois tamanhos de caixa, então a pilha ainda empilha torto e tomba.

**Gates: 8** (redondo-não-é-caixa · 4 cantos · roda com o corpo · segue a pose · px de tela sob zoom 4× ·
off não desenha · cena sem corpos não desenha · estático ≠ dinâmico na cor). **5 mutações, 5 sangram** — a
primeira delas é o bug reportado LITERAL (desenhar a bola como o quad do sprite). ⚠️ Tolerância do gate
redondo é **0,01 px, com motivo**: mundo é `f32`, então a borda carrega ~1e-4 px de arredondamento de trig;
o erro que o gate existe pra pegar é uma CAIXA, cujos cantos ficam 41 px mais longe — a barra é ~4000× mais
apertada que o fenômeno.

**Append-only em foundational:** `ph2d-vector` re-exporta `PathEl` (o gateway do kurbo; **não** é a
superfície congelada — o gate `architecture_vector_contract_surface` escaneia só `-doc` e `-traits`,
verificado). Campo novo `App.show_colliders`; **W2 põe o checkbox "Show Colliders" no painel lendo ESTE
flag** — duas portas pra mesma pergunta divergem.

---

## §W2 (metade) — A AUTORIA: a seção "Physics Body" no Inspector (2026-07-18, **smoke aprovado**)

**O que mudou de verdade:** antes disto, um `RigidBody` só podia vir de uma cena de smoke — **não
existia gesto nenhum no editor que tornasse um sprite físico**. Agora: selecione qualquer sprite → seção
**Physics Body** → **Add Physics Body** → Play, e ele cai.

**A seção tem DUAS faces, e a vazia é a importante.** Toda outra seção do Inspector descreve algo que a
entidade já tem; esta também precisa oferecer o que ela ainda **não** tem, senão a física é alcançável só
onde já há física — ou seja, em lugar nenhum. Por isso `build_physics_info` devolve `Some` para qualquer
entidade com `Transform`, com `has_body: false`.

**⚠️ O collider nasce da CAIXA DO SPRITE** (`apply_physics_edit`, ramo `Add`) — a única forma inicial que
**não pode** discordar do que está desenhado. É a lição do smoke de 2026-07-18 virada regra: uma bola
default sob um sprite 2×1 desenharia retângulo e rolaria como círculo **desde o primeiro clique**. Unity e
Godot ajustam a box ao renderer pelo mesmo motivo. Gate: `the_added_collider_is_boxed_to_the_sprite`.

**Trocar de forma PRESERVA a pegada** (box → a bola que CABE nela, e volta): o objeto não pode pular de
tamanho quando o artista só está escolhendo entre caixa e bola.

**`restitution`/`friction` foram APENDADOS ao `Collider`** — e na ordem honesta: campo → `BodyDesc` →
rapier, **no mesmo commit** (campo sem consumidor é órfão, DIRETIVA §2). Os defaults são **os do próprio
rapier** (0.0 / 0.5), e isso está **MEDIDO**, não suposto: `the_new_collider_defaults_are_the_ones_rapier_already_used`
roda 240 steps comparando `spawn_body` nos defaults contra `add_dynamic_circle`, que nunca setou nenhum
dos dois. Mutação (0.3) sangra. ⚠️ **`PROJECT_SCHEMA` 16→17** — postcard é POSICIONAL, então apendar campo
muda o layout do arquivo e **nenhum gate podia ver isso** (nenhuma constante de esquema mudou).

**Os dois tetos de LOC brigaram, e o repo diz split, nunca allowlist.** `paint_inspector` estava congelado
em 431 e uma seção custa ~18 linhas ⇒ extraí o **frame da seção** (o corpo do macro `live_section!`, que
conta LOC por estar definido DENTRO da função) + a **fase B** (placeholder/publish/scrollbar) para
`paint_frame.rs`; `paint_inspector` 455 → **424**, e a allowance foi **catracada 431 → 424** (elas só
encolhem). Idem no lado do evento (`event_physics.rs`) e do shell (`inspector_physics.rs`, porque
`inspector_ordering.rs` bateu 730/600). ⚠️ **O rustfmt re-expande chamadas compactas** — tentei ganhar
linhas comprimindo argumentos e o fmt as devolveu MAIORES; a extração tem de ser estrutural.

**Gates (14 novos, 2 famílias que se cobrem):**
- **`ph2d-panel-inspector/tests/seam_physics.rs` (7)** — o SWEEP: **todo** controle é clicado/comitado e a
  ação exata afirmada (não "o card mais cheio" — essa premissa já apodreceu duas vezes aqui). Inclui a
  **recusa no event.rs**: Add numa entidade que JÁ tem corpo, e Remove numa que não tem, não podem chegar
  ao bus (dim/não-pintado não é recusa). 4 mutações, 4 sangram no gate certo.
- **`shells/desktop/src/render_loop/inspector_physics_tests.rs` (7)** — a OUTRA metade, a que o repo já
  pagou caro: seam verde ≠ produto vivo. O oráculo não é "os componentes existem", é **o sprite está
  deitado no chão um segundo depois**. Mais: Remove tira os DOIS componentes · editar um campo não zera os
  outros · Static para de cair · o snapshot reflete o que foi escrito.

**Smoke: `PH2D_PHYSICS_SMOKE=3`** — chão + 3 sprites de proporções DIFERENTES (num quadrado, um collider
que ignora a arte é invisível), **relógio PAUSADO** (cena que já está rodando não se monta). Selecione,
Add, Play. Com **B** ligado o contorno deve traçar cada sprite exatamente.

**⏳ A OUTRA METADE do W2 segue pendente:** o **painel global** (`ph2d-panel-physics`, categoria MUNDO —
gravidade, substeps, damping, sleep, camadas de colisão). É a metade menos urgente: os defaults já são
bons, enquanto sem o Inspector a física era inalcançável. Terreno mapeado (5 sites de registro, o gate do
z-order que faz um painel registrado+visível **nunca ser pintado**, `PHYSICS_SCROLLBAR_ID = NodeId(836)`
livre).

---

### A INTERPENETRAÇÃO COM O CHÃO (2026-07-18, smoke do Enio) — MEDIDA antes de mexer

*"Observa-se alguma interpenetração dos objetos dinâmicos com o chão"*. A medição separou **duas coisas
que se parecem e não são**:

| | profundidade | duração |
|---|---|---|
| **em repouso** | **1,3 mm** | permanente |
| **no impacto** (queda de y=4, 9,4 m/s) | **83 mm** | **9 frames (0,15 s)** |

O repouso é o `normalized_allowed_linear_error` do rapier — **1 mm por projeto**. A ~100 px/m isso é
**0,13 px**: não é o que ninguém viu, e não vale perseguir. O que se vê é o impacto: ~8 px na tela por um
sexto de segundo.

⚠️ **E a PROFUNDIDADE não é falha do solver.** Medi damping de contato, teto de velocidade corretiva,
iterações extras do solver e **CCD** — cada um deixou o número em **exatamente 83,2 mm**. É `v × dt`: a
9,4 m/s o corpo anda 157 mm num tick de 60 Hz, então **no tick em que encosta ele já está dentro**, e
nenhum solver desfaz isso *depois*. (O CCD não faz nada aqui porque nada **tunela** — 83 mm de
sobreposição num corpo de 560 mm não é colisão perdida.) Subir o **damping** — o conselho usual do rapier
para "parecer mais rígido" — vai para o lado **errado**: 5,0 já é super-amortecido e 20 esticou a
recuperação de 9 para **30 frames**.

**Duas alavancas ortogonais, uma para cada metade:**
- **`DEFAULT_SUBSTEPS = 4`** ataca a PROFUNDIDADE (1→83 mm · 2→73 · 4→31 · 8→8,8). É o joelho da curva, e
  o **Box2D v3 ships o mesmo default pelo mesmo motivo**.
- **`DEFAULT_CONTACT_HZ = 120`** (rapier: 30) ataca a DURAÇÃO — o doc do próprio rapier diz que a
  frequência natural é o que *"corrige penetrações mais rápido"*. 30 Hz → 9 frames · 120 Hz → 1.

**Resultado: 83 mm/9 frames → 23 mm/1 frame.** Custo medido: 264 µs para **500 corpos** (18% do 1,5 ms de
HR-4); em cena de smoke é 15 µs. E o trade que eu temia **não se materializou**: a pilha assentada fica em
**0,00000 mm/tick** antes e depois (gate próprio — trocar penetração por tremor seria artefato pior).

⚠️ **`dt()` mudou de significado e eu quase deixei passar:** com substeps, o `dt` do integrador ≠ o do
tick. O teste `dt_default_is_60hz` pegou. `dt()` agora é o **TICK** (o que casa com o `FixedStep`/Playhead
— um `dt()` que virasse o sub-passo em silêncio discordaria do relógio); o do integrador é `substep_dt()`.

⚠️ **Os dois hashes C9 MUDARAM** (`physics-c9` → `2f7e2d58…`, `physics-ecs-c9` → `54fea296…`): parâmetros
de integração entram no solver. **Nenhum é pinado em literal** — o CI compara os 3 OSes entre si
(`sort -u | wc -l`), então o gate segue válido e continua provando o que sempre provou.

**Gates (3, em `ph2d-physics/tests/penetration.rs`):** o corpo nunca fica visivelmente dentro do chão por
mais de 1 frame (nas 4 alturas de queda que as cenas de smoke usam; a barra é **1 px do ARTISTA**, não um
número que lisonjeia o solver) · a pilha assentada é imóvel · o custo do substepping é RATIO, não
cronômetro. **2 mutações, 2 sangram** — uma por metade, porque as duas constantes consertam metades
diferentes do mesmo artefato.

---

---

## ✅ §W2b — o painel global de mundo LANDOU (2026-07-18) · **smoke APROVADO** (re-smoke pós-fixes)

### O terreno que a wave usou (medido pós-integração, mantido como registro)

⚠️ **Os números abaixo foram medidos DEPOIS da integração, não copiados do plano.** A `main` recebeu
Painter, FLIP e GPU na mesma janela, e um "próximo id livre" anotado antes do merge é exatamente o tipo
de fato que envelhece em silêncio.

| Fato | Valor verificado hoje |
|---|---|
| `PHYSICS_SCROLLBAR_ID` | **`NodeId(836)` ainda LIVRE** (o topo ocupado é 835, `FLIP_SCROLLBAR_ID`) |
| Próximo `z` de chrome livre | **310** (240 · 270 · 271 · 280 · 290 · **300 = o transport desta linha**) |
| Painéis registrados hoje | **19** (`EXPECTED_TYPED` é à mão e **não** é regenerado pelo `panel-sync` — some 1) |
| Ponto de inserção no z-order | logo após `ids::TIMELINE_PANEL` (`hero/paint.rs:341`), **antes** da cauda flutuante `INSP_BLENDER_PICKER`/`GAL_PANEL` — o que vem depois pinta por cima |
| `PROJECT_SCHEMA` atual | **18** — o painel global **não** persiste nada novo por si só (gravidade e afins são settings de mundo; decidir ONDE moram é parte do W2b) |

**O que o W2b entrega** (ADR-0131 D8): crate `ph2d-panel-physics` docada na categoria MUNDO — gravidade
(vetor), substeps/iterações do solver, damping global, sleep thresholds, matriz de camadas de colisão. A
escala do mundo é `ProjectSettings.pixels_per_meter` (setting do PROJETO) — **o painel exibe, não duplica**.

**Os 5 sites de registro** (precedente canônico: `ph2d-panel-vector`) e ⚠️ **a armadilha que não falha
alto**: sem a entrada na lista de fallback de z-order, o painel fica registrado, visível, e **NUNCA é
pintado** — nada quebra, nada avisa.

**Já existe e o W2b só liga:** os knobs `set_gravity` (na ponte, que já limpa o ring) ·
`set_substeps`/`set_contact_frequency`/`set_contact_response`/`set_solver_iterations` (no `PhysicsWorld`)
· e o flag `App.show_colliders`, que o checkbox "Show Colliders" deve LER — **duas portas para a mesma
pergunta divergem**, então o checkbox e a tecla `B` compartilham o flag, não cada um o seu.

---

### O que a wave entregou

**Crate nova `ph2d-panel-physics`**, docada, categoria MUNDO — a metade do mundo da autoria
(a metade do CORPO é a seção "Physics Body" do Inspector, W2a). Gravidade (X/Y) · sub-passos ·
iterações do solver · frequência de contato · arrasto linear/angular · sono (velocidade, giro,
atraso) · Show Colliders · Reset to Defaults · readouts de escala e nº de corpos.

**Abridor: tecla `W`** (de World), espelho do `L` da timeline. Um painel de mundo não é
tool-gated, então sem abridor próprio ele é feature que ninguém alcança.

### As decisões que decidem tudo

- **UMA TABELA, QUATRO CONSUMIDORES** (`rows.rs::SECTIONS`). Um knob é pintado, registrado,
  virado em valor no drag e varrido pelo seam — quatro listas à mão driftam, e o drift é MUDO
  (row pintada e não registrada = clique dropado em silêncio). `paint`/`populate`/`event`/
  `tests/seam.rs` iteram a MESMA lista, então um knob novo nasce pintado, registrado, vivo e
  varrido. É também a resposta estrutural ao *"o card mais cheio apodrece"*.
- **Não há tool, então não há `ToolPanelEvent`:** o painel emite INTENTS que a ponte do shell
  drena (padrão `motion-graph`/timeline). Inventar uma tool pro cano existente encaixar seria
  uma tool que não é tool.
- **O ARTISTA é dono da visibilidade:** a ponte nunca a escreve — sem edge-trigger
  `LAST_ACTIVE`, sem tomada do slot do Inspector. Não há aresta de ativação em que disparar, e
  roubar o Inspector de um painel que o artista abriu de propósito tiraria o que ele estava
  olhando.
- **Duas coisas são EXIBIDAS, nunca possuídas:** a escala do mundo é
  `ProjectSettings.pixels_per_meter` (D4 — já tem dono no menu Settings) e o contorno é o
  `App.show_colliders` do shell, o MESMO flag da tecla `B`. O toggle devolve um PEDIDO.
- **Sem camadas de colisão, e por um motivo nomeável** — ver W2c no plano: a matriz é metade de
  uma feature, e a outra metade (a camada por-corpo) é component + Inspector.

### O que já existia e a wave só ligou — mais o que ela teve de construir

`set_gravity` e os `set_substeps`/`set_contact_frequency`/`set_solver_iterations` já estavam lá.
O que **não** estava: **damping e sono globais não existem no rapier** — os dois são POR CORPO
(medido: o `IntegrationParameters` não tem nenhum dos dois). Expô-los como setting de mundo é o
idioma que todo motor 2D shipa (Godot: `default_linear_damp`, `sleep_threshold_linear`,
`time_before_sleep`; Unity: sleep tolerances), então nasceu o `BodyDefaults` em
`ph2d-physics/src/world/defaults.rs`, com **uma porta só** por número.
⚠️ **Um override por-corpo, se um dia existir, TEM de chegar com modo de combinação** (o
`damp_mode` do Godot) — um 2º campo que ganha em silêncio é a divergência clássica.

### Todo teto foi MEDIDO (`ph2d-physics/tests/measure_settings.rs`, `--release`, `#[ignore]`)

| knob | teto | de que recurso |
|---|---|---|
| sub-passos | **12** | CPU: 500 corpos acordados = **101,9% do HR-4** (4=34,1% · 8=67,8% · 16=135%) |
| iterações | **16** | CPU: 85,7% do HR-4 (24 = 120,5%, estoura) |
| contact Hz | **480** | estabilidade: deriva EXATAMENTE 0,0000 mm até 960 Hz; a 1920 Hz aparece (0,011 px) |
| arrasto | **10** | significado: velocidade terminal ≈ g/d ⇒ 10 = 0,98 m/s (corpo que DERIVA); além disso só sombras de "parado" |

⚠️ **A hipótese óbvia do contact Hz — Nyquist em `1/(2·substep_dt)` = 120 Hz — foi REFUTADA
pela medição.** As soft constraints do rapier são estáveis muito além. O teto shipado é o
medido, não o derivado.

⚠️ **E a 1ª rodada do harness mediu NADA:** uma pilha assentada DORME, e corpo dormindo não é
integrado — a sonda de jitter leu 0,0000 mm em todas as frequências (inclusive 1920) e a tabela
de custo cronometrou uma pilha que tinha parado de ser simulada. Os dois zeros eram
**garantidos, não observados**. O harness agora proíbe o sono (que é também o pior caso honesto
pra um orçamento).

### Persistência: `PROJECT_SCHEMA` **18 → 19**, tripla-pin `(19, 8, 8)`

`ProjectFile.physics` (6º campo), FORA do `ProjectState` — o `ProjectState` é a unidade do undo
GLOBAL e um Ctrl+Z do canvas não deve rebobinar a gravidade da cena (mesmo motivo do `motion` e
da `timeline`). ⚠️ **A ORDEM no load: `rebuild()` primeiro, `set_settings` depois** — o rebuild
constrói um mundo novo nos defaults do motor, então instalar antes seria escrever no que ele
joga fora, e a cena carregaria com a gravidade do documento ANTERIOR, em silêncio. Pinado por
arch-gate sobre o fonte (o fato é uma ORDEM; nenhum teste de unidade a alcança porque `gfx` é
`None` sem janela).

### Gates: 30 novos, 26 mutações, 25 sangram (1 sobrevive por projeto — ver acima)

`ph2d-physics/tests/body_defaults.rs` (6 + 1 unit) · `ph2d-physics-ecs/tests/settings.rs` (6) ·
`ph2d-panel-physics/tests/seam.rs` (9) · `project_tests` (2). Mutações: 7 + 6 + 8 + 1.

**Três gates nasceram VERDES sobre o bug que existiam pra pegar. Vale mais que os 21:**

1. *"os defaults são os do rapier"* comparava `BodyDefaults::rapier()` **contra ele mesmo** (os
   dois mundos liam a MESMA função) e ficou verde com `linear_damping` mutado pra 0.05. O
   oráculo tem de ser o RAPIER — um corpo que ele construiu e ninguém configurou — e por isso
   mora como unit test, onde o rapier é alcançável.
2. *"cada row muda só o campo dela"* computava a expectativa **com `row.set`**, então ligar a
   row de `gravity_y` no setter de `gravity_x` mexia nos dois lados. O gate novo não usa
   aritmética da tabela: round-trip (`get ∘ set` == identidade) + disjunção.
3. *"as settings sobrevivem ao scrub"* acertava o **RING**, e um checkpoint restaurado carrega o
   damping dentro do body set ⇒ o `rebuild_from_rest` nunca rodava e o código pré-W2b passava. O
   MISS é o **Reset** (tick 0 nunca é gravado), e o ring vazio virou pré-condição do fixture —
   é a assinatura observável de que aquela pista rodou.

Padrão comum: **um oráculo que usa a função sob teste para computar o que espera é sempre
verde.** Vale a pena procurar por essa forma antes de confiar num gate que passou de primeira.

E duas metades do sono ficaram verdes **uma sem a outra**: uma bola ASSENTADA está abaixo de
qualquer threshold são e parada por qualquer timer são, então o knob sobrevivente decidia
sozinho. Agora o threshold é provado por **queda livre** (a bola dorme NO AR — que é também o
bug que o artista reportaria) e o timer por oráculo **diferencial** (dois timers, mesma cena).

### ⚠️ O SMOKE DO W2b REPROVOU DUAS COISAS — e as duas já fecharam

**1. *"não vejo o painel, não abre com w"* — ele não existia no build.**
O shell declara `ph2d-panel-registry-init = { default-features = false }` e
re-enumera os painéis na **própria** lista `default`. Eu liguei `panel-physics`
na lista `default` da crate de registry, **que não alcança ninguém**. O painel
nunca foi compilado no registro, e tudo a jusante funcionou perfeitamente sobre
um painel que não existe: a tecla vira `panel_visibility["physics"]`, o walk de
z-order pergunta o id ao registro, recebe `None`, não pinta nada. Sem erro, sem
warning, sem símbolo faltando. E o `EXPECTED_TYPED` ficou **verde o tempo todo**,
porque roda dentro da crate de registry com as features DELA — nada olhava o
build do shell. Gate novo, escrito **onde o shell é compilado**:
`every_panel_the_shell_drives_is_in_its_registry` (duas asserções: a feature está
no `default` do shell · o registro que o grafo produz de fato contém o id, porque
o push é codegen). Memória: [[feedback_a_default_feature_list_does_not_reach_a_consumer_that_disables_defaults]].

**2. *"Air Drag… todos os objetos grandes e pequenos caem na mesma velocidade"* —
verdade, e o erro era o RÓTULO.** Medido: com `linear_damping = 2.0`, quatro
caixas cobrindo **25× de massa** caíram a **4,8925 m/s**, idênticas até a 4ª
decimal. O `linear_damping` do rapier é um decaimento **uniforme** — massa e
tamanho não podem entrar nele — e isso é o comportamento **correto** daquele
knob (é o que Godot e Unity shipam). Só não é ar. Portei a equação publicada
(`F = ½ρCdA|v|v` ⇒ para corpo 2D de densidade uniforme, `a ∝ v²/s`) em
`ph2d-physics/src/world/drag.rs`, e os **dois modelos coexistem, separados por
SEÇÃO** — é a seção que os mantém distinguíveis:

| seção | knob | o que faz |
|---|---|---|
| **Air Drag** | Density | escala com a secção transversal, resistido pela massa ⇒ **o grande cai mais rápido** |
| **Damping** | Linear · Angular | decaimento uniforme ⇒ **tudo desacelera igual** |

Memória: [[feedback_a_label_must_promise_what_the_model_delivers]].

⚠️ **`add_force` do rapier é força CONSTANTE até `reset_forces`, e o pipeline
nunca a limpa** — aplicar por substep acumulou ~720× pela terceira segunda, e as
velocidades terminais saíram **não-monotônicas** (0,05 / 0,51 / 0,52 / 0,01 m/s),
que foi o que me mandou olhar. O primitivo certo para *"esta força, por esta
fatia de tempo"* é o **impulso** (`F·dt`): não carrega estado e deixa o canal de
força do usuário livre.

**Teto MEDIDO `MAX_AIR_DRAG = 10`, e o recurso é o LIMIAR DE SONO:** terminal é
`√(mg/(k·L))`, então a `k=20` o corpo de 0,28 m cai abaixo do threshold e
**dorme no ar** (leu 0,00). Parece bug, não ar grosso.

**`PROJECT_SCHEMA` 19 → 20** (o `air_drag` é campo apendado ao `PhysicsSettings`,
que entra no layout do `ProjectFile.physics`).

**Gates novos: 6** (4 de drag + 2 de registro), **4 mutações de drag**:
- o oráculo do terminal é a **forma fechada publicada**, não um número que este
  código produziu (barra de 2%);
- ⚠️ *"o maior cai mais rápido"* **sozinho não basta**: com `length = 1.0` o
  terminal ainda cresce com o tamanho (a massa ainda varia) — quem pega a
  regressão é a equação;
- *"zero é byte-idêntico"* (trajetória, não endpoint) protege os hashes C9;
- e um gate afirma que o **damping continua UNIFORME**: se um refactor fundir os
  dois modelos, o knob que DEVE ignorar tamanho para de ignorar em silêncio, e a
  rotulagem honesta do painel volta a ser mentira;
- ⚠️ a mutação que remove o early-out de `k<=0` **SOBREVIVE, por projeto**: a
  força seria o vetor zero e o impulso um no-op, então o contrato é honrado
  **duas vezes** (pelo ramo e pela aritmética) — mesma forma do early-out de
  tinta plana na luz GPU do impasto. O comentário dizia mais do que o ramo faz;
  agora diz que é só caminho rápido.

---

### Fiação (o mapa do handoff de continuação, agora percorrido)

5 sites de painel + 4 do scroll: `ids/chrome/physics.rs` (29 ids, todos na tabela de colisão
elemento a elemento) · `mod`/`pub use` · **a lista de fallback de z-order** · `panel-sync` +
`EXPECTED_TYPED` 18→19 + a lista `default` (as duas à mão) · `PHYSICS_SCROLLBAR_ID = NodeId(836)`
+ auto-checagem + `scrollbar_panel_for_id` + **`|| inside(PHYSICS_PANEL)`** no
`cursor_over_hero_panel` (o 4º, o que não falha alto: sem ele a roda ZOOMA a câmera por baixo).
i18n `panel.physics.*`.

⚠️ **As seções SÃO colapsáveis por necessidade, não por estilo:** o `paint_section_header` pinta
o chevron SEMPRE, então um header sem id vivo desenharia um "clique pra dobrar" que não dobra.
⚠️ **"Show Colliders" é um Button, não um Checkbox:** `Checkbox` emite `Toggled`, que este
`event.rs` não encaminha — ficaria registrado e morto (a mesma cicatriz do painter-layers).

### Aberto no W2b

- **Nenhum gate mede a perf do painel** — ele é 10 rows de slider, e o custo real do W2b está no
  solver, que já é gateado por RATIO. Se um knob novo trouxer trabalho por-frame, gateie.
- **O `body_count` do readout conta corpos, não "corpos dormindo"** — a pergunta *"por que nada
  se move?"* teria resposta melhor com os dois números. Barato; não foi feito porque ninguém
  ainda a fez.

---

## ✅ §W2c — camadas de colisão (2026-07-18, **smoke aprovado**)

**O modelo é o da Unity, e a escolha muda tudo.** Godot/Box2D dão a cada corpo
um `layer` E um `mask`: flexível, sem estado global — e a regra *"bala não
acerta quem atirou"* é re-digitada em cada bala. A Unity tem UMA matriz global e
cada corpo nomeia uma camada: a regra é autorada **uma vez, no mundo**. rapier é
nativamente o primeiro (`InteractionGroups{memberships, filter}`), então
`world/layers.rs` é o segundo em cima dele: `memberships` = o bit da camada,
`filter` = a **linha** daquela camada na matriz.

⚠️ **A matriz TEM de ser simétrica, e aqui o assimétrico é INEXPRIMÍVEL.** A
regra do rapier é `(A.mem ∩ B.filter) ≠ ∅ **AND** (B.mem ∩ A.filter) ≠ ∅` — as
duas direções. Uma matriz meio-escrita não significa *"i vê j mas não o
contrário"*: o AND faz significar **colisão nenhuma**, uma regra que ninguém
escreveu. `LayerMatrix::set` escreve as **duas** metades e `from_rows` (a porta
de leitura de arquivo) **simetriza** — um arquivo editado à mão não instala um
estado que o tipo diz não existir. Por isso o painel desenha só o **triângulo
inferior**: a célula espelho seria um segundo controle pro mesmo checkbox.

`groups_for(layer, matrix)` é a **porta única** — spawn e re-filtragem produzem
os grupos pela mesma função. E o collider já carrega a própria camada (ela **é**
o `memberships`, um bit), então `set_layer_matrix` re-filtra os vivos sem
ninguém guardar a camada duas vezes.

**8 camadas, com o limite NOMEADO:** a representação permite 32 (o `Group` do
rapier é `u32`) e não é isso que aperta — é o painel. Matriz triangular de N tem
`N(N+1)/2` células: 8 → **36**, 16 → 136, 32 → **528**. A Unity shipa 32 e a
matriz dela é o exemplo padrão de tela ilegível. Crescer é mudança de UI +
schema, não de física.

**As duas metades, e é isso que fez a wave existir separada do W2b:** a matriz é
metade de uma feature — a outra é a **camada por-corpo**, que é campo de
component (`Collider.layer`) e UI do **Inspector**. Matriz sem ela é 1×1.

**`PROJECT_SCHEMA` 20 → 21** (duas quebras de layout no mesmo bump: `Collider.layer`
apendado ao component **e** `layer_matrix` apendado às settings).

### Gates: 11 novos (5 bridge · 3 unit · 2 seam-painel · 1 seam-inspector), 12 mutações, 12 sangram

⚠️ **Dois nasceram VERDES sobre o bug que existiam pra pegar** — a mesma família
das três do W2b:
- o gate da **simetria** envolvia o valor guardado em `from_rows`, a própria
  função sob teste, então os dois lados normalizavam. Agora lê as **linhas
  cruas**. E o valor guardado importa independente do solver: `apply_to`
  simetriza na entrada do rapier, então a SIMULAÇÃO está segura de qualquer
  jeito; quem não está é o **painel**, que pinta checkbox dessas linhas, e o
  **arquivo**, que as salva.
- o gate das 36 células mandava um `WidgetEvent` **sintético**, que chega direto
  no `apply_event`. Um clique REAL primeiro tem de achar a célula no hit-index
  **e** achá-la FOCÁVEL no store — então tirar as células do `populate` deixava
  o gate verde sobre 36 widgets pintados, hit-registrados, com arm ligado, e
  **mortos sob o mouse**. Agora dirige `click_at`.

⚠️ **As células são registradas num LAÇO, que o `architecture_panel_wiring_parity`
não enxerga** — e os ids são um **array const**, não hasheados em runtime, pra
que o `node_id_collisions` ao menos os cubra. O seam que clica as 36 não é
redundante com os arch-gates: é a única coisa cobrindo aquele widget.

---

## ✅ §W3 — joints (2026-07-18, **smoke aprovado**)

### Um joint é uma ENTIDADE, e isso decide o resto

O norte do repo já respondeu o que todo objeto é (**ADR-0110**: no vetor, um
path *é* uma entidade, árvore única). Um joint-entidade herda de graça a
Hierarquia, a seleção, o nome, o delete, o undo e o save — apagar um joint é
apagar um objeto, e não há "remover joint" para inventar.

E tira o teto: um joint guardado **no corpo** só pode ser um por corpo (bevy tem
um componente de cada tipo por entidade), o que proíbe laço fechado e impede a
pelve de um ragdoll de receber três.

⚠️ **A âncora sai de graça, e isso não foi sorte:** o snapshot do Inspector diz,
no próprio código (`snapshots.rs`), que Position *"lands on every entity that
has a `Transform`, not just sprites"*. A entidade-joint carrega um `Transform`,
então o pivô é autorável em números no dia um, com **zero widget novo**. Um
gizmo de **PONTO** no canvas é outra coisa — os três publicadores de `GizmoView`
são CAIXAS com alças de escala — e **não** é esta wave (§Aberto).

### Os dois corpos são NOMEADOS, nunca apontados

`PhysicsJoint.body_a/body_b` guardam **`ph2d_ecs::stable_name_id`** (hash FNV-1a
do `Name`), nunca `Entity::to_bits()`. Bits são id de **ALOCAÇÃO**: o undo
respawna toda entidade com bits novos.

⚠️ E é pior que "a referência solta": bits dentro dos **bytes de um componente**
envenenam o próprio undo. O `canonicalize` ordena as linhas pelos bytes dos
componentes e remapeia **só** o campo estrutural `parent` — então dois estados
logicamente iguais comparariam diferente, que é exatamente o passo espúrio que
ele existe para matar.

A timeline já tinha essa resposta desde o W4.T6. Duas cópias seriam **duas
respostas** a *"qual é o id durável deste objeto?"*, então a função subiu para
`ph2d-ecs` (ao lado do `Name` de onde deriva) e o `timeline_persist` **delega**.
⚠️ O hash é um **FORMATO DE ARQUIVO** — todo projeto em disco já carrega esses
números — e o gate o pina contra valores computados **FORA** deste codebase
(chamar a própria função é o oráculo sempre-verde).

**O preço, pinado num gate:** renomear um corpo **DESACOPLA** os joints dele — e
os reacopla se o nome voltar. É a mesma exposição de toda binding da timeline;
o dia em que não servir, o conserto é um `StableId` de verdade migrando os
**dois** consumidores, não um segundo esquema de identidade para um deles.

### A política de âncora, numa frase

**O `Transform` do joint é a âncora em A. Em B é o MESMO ponto para um pino —
dois corpos num lugar só *é* o que um pino é — e o CENTRO de B para mola/corda,
cujas pontas são para estar separadas.**

O motor (`ph2d-physics`) toma **duas** âncoras e não tem opinião sobre quais
pontos são; a política mora numa função só (`bridge/joints.rs::joint_desc`).
⚠️ Colapsar o par foi tentado e **medido**: uma corda de 2 m pendurava a bola a
**2,5 m** sempre que o ponto autorado não era o centro dela — um número que o
artista digitou, calado, não significando o que diz. Box2D e Unity tomam o par
pelo mesmo motivo.

### Joints reconciliam DEPOIS dos corpos, e são re-descritos só em repouso

As duas coisas saem do mesmo fato: as âncoras **locais** derivam de onde os
corpos estão, então têm de derivar de onde o artista os **pôs**. A metade dos
corpos já tinha essa regra (`at_rest && b.rest != desc`) e os joints a montam em
vez de inventar uma segunda. ⚠️ E voltam no **MESMO chamado** do
`rebuild_from_rest`: o rewind replaya os ticks devidos na hora, e um replay sem
os joints é outra simulação — a corrente cairia aos pedaços e se remontaria um
frame depois.

### Três números MEDIDOS, e os três primeiros palpites foram REFUTADOS

| o quê | palpite | medido | por quê o palpite errava |
|---|---|---|---|
| contato entre corpos unidos | (o default do rapier) | **desligado** | rapier tem `contacts_enabled: **true**` — o OPOSTO de Box2D (`collideConnected`) e Unity (`enableCollision`). O caso canônico é um elo de corrente, que **sobrepõe o vizinho por construção**: ligado, o solver briga com uma interpenetração permanente e o motor mandado girar a 4 rad/s foi medido em **−80** |
| `MOTOR_TRACKING` | — | **100** | a 10 o motor não ergue o próprio braço (1,39 rad de 20); a 300+ só há retorno decrescente. Em TODO valor um motor capado em 0,1 N·m ainda trava — o teto continua mordendo |
| mola default | 100 / 5 | **30 / 0,5** | a 100 uma bola de 0,2 kg afundava **1,9 cm** num descanso de 1 m (2% — lê como VARA); a 5 de damping o repique era 2 mm |

⚠️ **Dois fixtures foram trocados porque MEDIAM MAL, não porque o código
errava.** (a) O pêndulo é **PERIÓDICO**: o gate pedia *"termina embaixo"* e
falhava sobre um pêndulo perfeito, pego no alto do balanço — toda afirmação
agora é sobre a **TRAJETÓRIA**, e a rigidez, sendo invariante, é checada a cada
passo. É a mesma lição que o W1.5 pagou. (b) Uma prancha presa pela **PONTA** é
um pêndulo **FORÇADO**, caótico (a mesma entrada caindo em −8,5, +9,1 e +2,0
rad/s): o motor se mede numa **RODA**, presa pelo centro, onde a gravidade não
faz torque.

### Persistência: `PROJECT_SCHEMA` **NÃO** bumpa (e o plano pedia 21 → 22)

O blob de um componente é chaveado por `stable_type_id = blake3(nome)[..8]` —
derivado do **NOME**, não de uma posição no registry. Registrar
`ph2d::physics::PhysicsJoint` cunha um id novo e **não move nada**: o oposto do
W2c, que apendou `layer` DENTRO do `Collider`, onde postcard é posicional.

E bumpar não é neutro: um schema divergente **recusa o arquivo inteiro**, então
jogaria fora todo projeto já salvo — para melhorar a mensagem de erro na única
direção que não funciona de qualquer jeito. **`PROJECT_SCHEMA` segue em 21**, e o
raciocínio está falsificável (`tests/joint_persistence.rs`): se algo mover o
layout, o 1º gate fica vermelho e o bump passa a ser devido.

### O gesto de criar mora na §11, não na §12

Um joint **não existe ainda** quando você quer fazer um, então o botão tem de
estar onde você já está — olhando os dois corpos que selecionou. **"Join
Selected Bodies"** aparece na seção *Physics Body* quando a seleção é
exatamente DOIS corpos, um fato que só a shell enxerga (o painel recebe uma
entidade por vez), perguntado **uma vez** e lido pelos dois lados: o pintor
decide se **oferece**, o arm decide se **honra**.

⚠️ **Join NÃO pode fan-out.** Todo outro edit da §11 é por-entidade e a
`render_loop` o espalha pela seleção; Join é um clique sobre um **PAR**, e
espalhado criaria **dois** joints entre os mesmos dois objetos no clique que
deveria criar um. Interceptado antes do fan-out, com **arch-gate sobre o fonte**
(`tests/join_is_one_gesture_not_a_fan_out.rs`) — nenhum unit test alcança aquela
função.

⚠️ **Corpos sem `Name` são NOMEADOS na criação.** Não é efeito colateral a
pedir desculpa: um corpo sem nome é um que um joint não consegue apontar, e as
bindings da timeline têm o mesmo requisito.

### A §12 e o anti-knob-morto

Só os parâmetros do tipo escolhido são pintados — um campo de *stiffness* numa
corda é um controle que não pode fazer nada, o que é pior que um faltando
porque parece que deveria funcionar. Quem responde *"este tipo tem motor?"* é
**`JointKind::is_hinge`**, e a **PONTE pergunta a MESMA função** antes de
entregar o motor ao solver ⇒ um knob pintado que o solver ignora não existe.
Ângulos em **GRAUS** na fronteira, **radianos** no componente (a cerca do
`rotation_rad`). Deletar um joint é deletar um **objeto**: despawn pelo caminho
de sempre, com o passo de undo de graça.

### O joint na tela

Um collider é invisível; um joint é **menos** que isso — é uma relação, sem
geometria nenhuma. Segmento entre as duas âncoras + um anel em cada ponta, em
**âmbar** (terceira coisa, ao lado do verde estático e do ciano dinâmico).

⚠️ **As âncoras vêm do SOLVER, não do `Transform` da entidade-joint.** O
Transform é a âncora **autorada** e nada o reescreve durante o play, então
desenhar dele pregaria o marcador onde o artista o largou enquanto a corrente
que ele segura balança embora — errado exatamente na situação que o artista
está olhando. O par vivo ainda conta a verdade sobre **tensão**.

⚠️ E é por **coincidirem** que existem os anéis: um pino em repouso tem as duas
âncoras no MESMO ponto, o segmento tem comprimento zero e **não pinta nada** — o
joint mais comum do editor seria invisível. Gate próprio.

### Gates: 56 novos, 39 mutações, 39 sangram

| onde | quantos | o que cobrem |
|---|---|---|
| `ph2d-physics/tests/joints.rs` | 10 | pino/limites/motor/corda/mola no solver; cada afirmação de restrição **pareada** com a afirmação do movimento que ela deve permitir |
| `ph2d-physics-ecs/tests/joints.rs` | 15 | undo (respawn REAL via `world_to_snapshot`), Reset, scrub, rename, meio-autorado, corpo re-spawnado, ordem de arquétipo |
| `ph2d-physics-ecs/tests/joint_persistence.rs` | 3 | o arquivo antigo ainda abre; nenhum id se moveu; round-trip com parâmetros |
| `ph2d-panel-inspector/tests/seam_joint.rs` | 7 | sweep que **CLICA de verdade** (`click_at`) |
| `seam_physics.rs` (+1) | 1 | Join oferecido **e** honrado só com dois corpos |
| shell `inspector_joint_tests.rs` | 8 | bus → ECS → simulação |
| shell `physics_overlay_joints.rs` | 4 | o desenho, inclusive âncoras coincidentes |
| shell `join_is_one_gesture_not_a_fan_out.rs` | 2 | arch-gate do fan-out |
| unit em `ph2d-physics-ecs/src/joint.rs` | 4 | discriminantes postcard pinados em ordem; joint meio-autorado; qual tipo é dobradiça |
| unit em `ph2d-ecs/src/name.rs` | 2 | o hash pinado contra valores de fora; nunca zero |

⚠️ **A 1ª rodada de mutação da ponte teve UMA sobrevivente:** a ordem
determinista de inserção — que com **um** joint não pode importar. O gate que
faltava monta a MESMA corrente com os arquétipos caindo diferente (um joint com
`Name`, outro sem) e exige o mesmo hash.

### ⚠️ A auditoria de 2 lentes achou SEIS coisas — e as duas graves eram minhas

**(1) Qualquer joint dormente destruía o cache de scrub, todo frame.**
`joints_to_remove` era populado para todo joint que não dava para construir
(meio-autorado, nomeando um corpo apagado ou renomeado) **independente de ele
ter SIDO construído**. A lista nunca esvaziava ⇒ `ring.clear()` a cada frame ⇒
o W1.5 morria calado: scrub para o tick 150 replayava **150** passos em vez de
0, com ring de 1. Alcançável pelos gestos mais banais. ⚠️ **A metade dos CORPOS
nunca teve esse bug** — ela só enfileira quem está em `self.bodies`; a dos
joints tinha divergido da própria irmã.

**(2) A âncora ANDAVA pelo corpo.** `JointRef.rest` guardava as âncoras em
MUNDO e o spawn re-derivava as locais contra a pose **VIVA**, então o spawn ao
vivo e o `rebuild_from_rest` respondiam *"onde no corpo isto está preso?"*
diferente: medido, um pino feito no meio do balanço prendia a **1,611 m** e
replayava a **0,642 m** depois de um Reset — 0,969 m de caminhada sem ninguém
tocar em nada. E o doc do módulo afirmava uma regra de *"só em repouso"* que
**não existia**: nada gateava o primeiro spawn.

O fix não foi gatear o gesto: **a âncora virou função do estado AUTORADO**. A
conversão mundo→local acontece UMA vez, contra a pose de **REPOUSO** dos corpos
(`PhysicsWorld::local_anchor_at_pose`), e o par **LOCAL** é o que se guarda e
replaya. Agora não importa quando um joint é criado — o que torna a frase do
cabeçalho verdadeira em vez de aspiracional. ⚠️ **O gate desta correção nasceu
vermelho com 1,771 m e SOBREVIVEU ao primeiro fix** (guardar locais, mas ainda
convertendo contra a pose viva); só a pose de repouso fecha.

**As outras quatro:** `PhysicsJoint::clamped()` na porta de carga (um componente
é serde e vem do arquivo — `NaN` em `stiffness` levava a pose para `(NaN, NaN)`
em 120 passos e o `readback` escrevia isso no `Transform` **e no hash de
determinismo**) · **limites invertidos SOLDAVAM a dobradiça** (`min > max`
entregue ao rapier congela a prancha; ordenados no `clamped`) · `create_joint`
devolvia `Some` para dois corpos de mesmo **NOME** (o guard comparava
entidades; um joint guarda hashes de nome) · e um comentário afirmando que o
factor do motor estava atado ao `max_force`, o que nunca foi verdade e o doc da
própria constante contradizia.

**E a lente de seams:** as **duas** seções desta linha (§11 do W2a e §12 do W3)
pintavam um dot de cor e um chevron de collapse que não estavam em **nenhuma**
das três listas compartilhadas — *painted, hit-registered e mortos sob o
mouse*. Ligadas agora. ⚠️ **O braço do picker ENUMERA seus leitores e já
apodreceu**: `NAME` e `VISIBILITY` armam e não abrem nada, e
ORDERING/SAMPLING/BLEND não estão em lugar nenhum. As minhas duas entraram; o
resto está **NOMEADO aqui em vez de contrabandeado** — são waves de outros
donos, e o conserto de verdade é UMA tabela `(seção, cor)` que o `pre_populate`
e o braço leiam (a forma que o `SECTIONS` do painel de física já usa).

**Achados LOW aceitos e não corrigidos** (registrados para não serem
re-descobertos): `Entity::to_bits()` é *generation-major*, então a ordenação é
estável quanto a arquétipo mas não sobrevive a um respawn que bumpa gerações —
herdado da metade dos corpos, não introduzido aqui · o Inspector resolve um
nome varrendo TODAS as entidades nomeadas e a ponte varre só os corpos, então
nomes duplicados podem divergir (mitigado pelo `unique_name`) · o `centre_b` de
um corpo sem `Transform` cai em `[0,0]` em vez de recusar (inalcançável hoje —
todo corpo em `self.bodies` veio de uma query que exige `Transform`).

### Smoke: `PH2D_PHYSICS_SMOKE=6`

Pêndulo · corrente · ragdoll, lado a lado — **três** porque cada uma responde
uma pergunta diferente. O pêndulo diz que a âncora é onde o artista a pôs (está
preso na **PONTA** da prancha, então uma versão que usasse centros a penduraria
pelo meio). A corrente diz que elos **sobrepostos** nos pinos não brigam. O
ragdoll diz que os **limites** seguram — joelhos dobram para um lado só.

### Aberto no W3

- **Gizmo de âncora no canvas** — um handle de **PONTO**, e os três publicadores
  de `GizmoView` são caixas com alças de escala. A âncora É autorável hoje (os
  campos Position da §12), então isto é refinamento, não buraco.
- **Re-escolher os corpos de um joint existente** — precisa de um *picker* de
  entidade, que o Inspector não tem. Hoje: apague o joint e faça outro.
- **Weld (`FixedJoint`)** — ~4 linhas no motor e um chip, deliberadamente FORA:
  nada no plano nem no smoke o exercita, e um 4º chip que a wave não fuma é um
  chip shipado às cegas.
- **Motor em mola/corda** — rapier expõe; nenhum consumidor pediu.

## ✅ §W4 — bake-to-timeline (2026-07-18, **smoke aprovado**)

### Assar não é simular de novo — é ANOTAR

A sim já é função pura de `(tick, estado de repouso autorado)` (D2/D7) e o W1.5
provou que qualquer tick é alcançável e **bit-exato**. Então o bake não é uma
simulação nova: é a **MESMA**, corrida sobre um alcance, com a pose anotada em
**cada** tick em vez de só no que o playhead calha de estar.

`ph2d-physics-ecs::bake` (`bake_trajectories`) lê a trajetória — headless,
determinística, sem saber o que é uma curva. O shell (`render_loop/physics_bake.rs`)
escreve as chaves. A divisão é deliberada: *"o que é uma chave"* já tem resposta
no editor, e uma segunda resposta morando dentro da física é como uma curva
assada passaria a diferir de uma gravada.

**O bake DEVOLVE o relógio** onde o artista o deixou. Gate red-first: clicar Bake
olhando o tick 40 teleportava a cena para o fim do alcance. A restauração não é
best-effort — a sim é função do tick, então re-despachar o tick original o
reproduz exatamente (a propriedade que o `scrub.rs` já gateia, emprestada em vez
de re-provada).

### O ajuste é o do RECORD, extraído e não reescrito

`render_loop/record_fit.rs` (novo): duas coisas neste editor produzem um valor
por frame e precisam virar keyframes — o **record** da timeline (gizmo arrastado
com o relógio correndo) e o **bake**. É o mesmo problema (centenas de chaves
densas que têm de colapsar para um punhado sem se mover) e agora é a mesma
resposta, senão uma curva assada e uma gravada sairiam com cara de ferramentas
diferentes. Foi extraído **do** record: a calibração é a dele, chegada pelos
smokes do Enio (§17 da timeline), e re-derivá-la para o bake seria um segundo
jogo de números para manter de acordo com o primeiro.

### O passa-baixa é um número da ENTRADA, nunca do ajuste (MEDIDO)

As 8 passadas do record existem porque um gesto de mouse carrega **tremor**. Um
solver não carrega nenhum: é determinístico, amostrado exatamente no tick que
avançou, e cada oscilação do sinal é uma oscilação que o corpo de fato teve.
Pior: uma trajetória de física é feita de **IMPACTOS**, um quique é uma
**cúspide**, e o kernel binomial arredonda o ápice — que *é* o quique.

Medido no cenário do próprio gate (pior erro contra a pose simulada, como fração
da amplitude do movimento):

| passadas | pior erro |
|---|---|
| **0** | **2,13%** |
| 1 | 2,02% |
| 2 | 2,48% |
| 4 | 3,31% |
| 8 (a do record) | 5,70% |

Monotônico depois de uma passada: **o número que a suavização existe para
melhorar era o que ela estava piorando.** O bake passa `0`. (O `1` mede um fio
melhor e **não** foi escolhido — a diferença está dentro do ruído de uma fixture
só, e *"o solver não tem tremor a remover"* é uma razão, enquanto *"uma passada
pontuou 0,1% melhor num cenário"* é uma coincidência.)

### O bake ENTREGA a pose — `BodyKind::Kinematic`, e isso É o bake

A **ordem do frame** decide, não a preferência: o apply da timeline escreve o
`Transform` (`mod.rs:1097`) e o readback da física escreve **depois**
(`mod.rs:1129`). Um corpo dinâmico recém-assado é sobrescrito pelo solver todo
frame — o artista clica em Bake, não vê nada mudar, e conclui que o botão está
quebrado. **Dois autores de um fato, e o de trás vence em silêncio.**

Então o bake entrega a pose. `Kinematic` é precisamente o estado *"a cena dirige
isto, e o solver é avisado"* — o corpo continua no mundo, continua empurrando o
que atravessa, mas o movimento agora vem da curva. É isso que *runtime-truth
vira animação* quer dizer, e é por isso que o kind aterrissou nesta wave. O flip
vai pela **MESMA porta** do chip da §11 (`apply_physics_edit`).

⚠️ **Duas filas de undo, e o artista vê dois passos.** As chaves são da timeline
(um bracket); o kind é edição de objeto e cai na fila global. É a forma que o
editor já tem (Ctrl+Z no Audio Editor também não desfaz um move de sprite), mas
significa que **um** Ctrl+Z depois do bake devolve o corpo a Dynamic com as
curvas de pé. O toast diz o que aconteceu e o chip da §11 mostra Kinematic
selecionado, então o estado é visível em vez de inferido.

O playhead **rebobina** depois do bake — que é onde a animação recém-feita
começa, e é também o que faz o flip chegar ao rapier (`reconcile_structure`
re-descreve um corpo **em repouso**; sem a rebobinada o corpo diria Kinematic no
Inspector e se comportaria como Dynamic até o próximo Reset — uma mentira que o
artista consegue ver).

### `BodyKind::Kinematic` e o BUG que a lei já escrita pegou

O componente **já prometia** o variant (*"`Kinematic` lands with W2/W3"*,
`components.rs:17`) e ele não tinha chegado. O bake é o primeiro consumidor
honesto: assar é exatamente o instante em que uma pose deixa de ser **saída** do
solver e vira **entrada** dele.

Três kinds, três donos da pose. `readback` pergunta `solver_owns_pose()` — porta
única, senão um corpo reclamado pelos dois lados teria a pose escrita duas vezes
por tick e a segunda venceria calada.

**O bug:** `step()` roda `substeps = 4` passes por tick, e
`set_next_kinematic_position` é consumido pelo **PRIMEIRO** deles — a plataforma
atravessava o tick inteiro em um quarto de tick, a 4× a velocidade real, e ficava
parada nos outros três. O atrito não tinha o que integrar: **a carga andava
0,009 m dos 1,000 m da plataforma.** E o comentário do `drag::apply`,
imediatamente acima do laço, já dizia a regra: *"Per SUBSTEP, not per tick: a
force applied once per tick would be wrong by the substep count"*. Fix: a mira é
para o fim do **TICK** e o `step` a fatia entre os sub-steps. **0,009 → 0,975 m.**
(Sono foi **medido** e não é fator: idêntico com 60 ticks de assentamento, então
não há `wake` devido.)

A **última fatia é o TARGET**, não um caminho aritmético até ele: `a0 + (a1 - a0)`
não é sempre exatamente `a1` em f32, e a pose é lida de volta para o `Transform`
— um ulp na última fatia seria uma pose que o artista não autorou, chegando todo
tick, sempre na mesma direção.

`BodyKind::tag`/`from_tag`: o mapeamento tag↔variant estava escrito **duas
vezes** (um `match` produzindo, um `if tag == 1 { Static } else { Dynamic }`
consumindo) e o consumidor dobrava **todo** tag desconhecido em `Dynamic`. Com
dois variants era só redundante; com o terceiro seria um chip que o artista clica
e que seleciona outra coisa. Tag desconhecido agora é **descartado**, não
adivinhado.

### Canal que nunca se move NÃO vira track

E isso protege trabalho, não arrumação: escrever uma track é **tomar posse** do
canal, e uma caixa que cai reto tem X constante — assar um X plano por cima de
quem o artista já animou de lado apagaria a animação dele e pareceria bug do
bake. Constância **bit-exata** (o rapier escreve o `f32` idêntico num canal que
não toca), que é o limiar morando onde o domínio é vazio.

### O alcance é mostrado NO BOTÃO

`Bake 5.0s to Timeline`. Ordem: **loop armado** → **extensão do documento** →
`DEFAULT_BAKE_SECONDS`. O loop primeiro porque **já é** o controle de *"esta
parte da timeline"* — um campo de range ao lado do botão seria um segundo jeito
de dizer a mesma coisa. O default é medido: queda de 4 m toca em ~0,9 s e para de
quicar em ~2,5 s; corrente de 6 elos assenta em ~3,5 s.

### Bake NÃO faz fan-out, e errar custa mais que no Join

Uma corrida da sim serve **todos** os corpos selecionados. Espalhado, ele
re-simularia a cena inteira uma vez por corpo — **mesmos números**, N vezes o
trabalho — e deixaria N passos de undo, então desfazer *"o bake"* custaria tantos
Ctrl+Z quantos objetos. **Nada pareceria errado.** O arch-gate do Join virou
`shells/desktop/tests/selection_gestures_are_not_fanned_out.rs` e pergunta pelos
dois.

### Gates: 36 novos, 30 mutações, 28 sangram

| arquivo | n |
|---|---|
| `ph2d-physics/tests/kinematic_substeps.rs` | 6 |
| `ph2d-physics-ecs/tests/kinematic.rs` | 8 |
| `ph2d-physics-ecs/tests/bake.rs` | 5 |
| `ph2d-physics-ecs/src/bake.rs` (unit) | 2 |
| `render_loop/physics_bake_tests.rs` | 11 |
| `ph2d-panel-inspector/tests/seam_physics.rs` (+2) | 2 |
| `shells/desktop/tests/selection_gestures_are_not_fanned_out.rs` (+2) | 2 |

**Dois sobreviventes FICAM, e os dois estão documentados onde alguém os
reencontraria:**

- **o early-out do bake vazio é sobre CUSTO.** O `commit_if_changed` compara o
  documento e recusa empurrar passo, então um bake vazio pode cair até o fim sem
  sujar a pilha. Qual camada faz qual promessa está escrito no gate
  ([[feedback_layered_defenses_need_per_layer_gates]]).
- **`COLUMN_MERGE_S` é INERTE num bake, e isso foi MEDIDO, não suposto:** colunas
  `[0.0, 0.583, 1.5]` com a fusão e `[0.0, 0.583, 1.5]` sem ela. Uma mão cruza o
  extremo de cada eixo alguns milissegundos separada — é para isso que a fusão
  existe — mas um bake amostra todo canal na **MESMA grade de ticks**, então os
  extremos caem no mesmo tempo exato. Gate ali seria gate que não pode falhar; a
  constante é do record e é do record prová-la.

**Três sobreviventes da primeira rodada viraram gate, e o primeiro era grave:**
apagar o ajuste deixa **uma chave por tick**, o que reproduz a sim
**exatamente** (o gate de fidelidade fica mais verde que nunca), custa um passo
de undo (aquele gate passa) e é completamente inútil — ninguém edita noventa
keyframes por segundo. **A entrega da wave não estava gateada por nada.**

**Duas fixtures nasceram vermelhas por FIXTURE, não por código** — e as duas
ensinam a mesma coisa: o de fidelidade batia na cúspide (foi o que expôs o
passa-baixa) e o do alcance default media uma bola rolando para **fora** de um
chão inclinado — ela nunca para, então *"o movimento terminou"* ali media a
rampa, não o default ([[feedback_moving_the_law_is_half_the_fix_the_fixture_must_contain_it]]).

E três mutações da primeira rodada do Kinematic sobreviveram por motivos
diferentes, todos reais: o unwrap de arco curto só era exercitado numa
**direção** (o `d` da fixture era sempre negativo, então só encontrava o ramo que
continuava de pé); a guarda do `readback` é **invisível** num corpo kinematic (a
mira põe o corpo exatamente onde o `Transform` diz, então reescrever escreve o
mesmo número — o caso observável é um corpo **Static** movido durante o play); e
a guarda de tipo da mira **não protege a pose** (medido: o rapier ignora
next-position em corpo dinâmico), ela protege o **CUSTO** — então o gate lê a
**lista**, não a pose.

### ⚠️ A auditoria de 2 lentes achou DEZ coisas — e as três piores eram UMA

`Kinematic` faz a sim depender de um **fluxo de entrada por-tick**, e três
lugares continuavam supondo que ela depende só de `(tick, repouso autorado)`: o
laço de ticks devidos, o **replay** do rewind, e o **bake**. Um chapéu, três
cabeças.

**O desenho que fechou: `SceneAtTick`.** A ponte **PERGUNTA** onde a cena põe
seus corpos dirigidos, num tick dado; quem responde é a metade que possui as
curvas (o shell, via `apply_from_doc`), e a física não aprende o que é uma
timeline. O invariante volta a ser verdadeiro: *o mundo é função do tick, dado o
repouso autorado **E as curvas autoradas*** — as duas reproduzíveis. `dispatch`
mantém a assinatura (99 chamadores intactos) e delega para
`dispatch_with_scene`: **uma** implementação, um atalho. Responder `false`
significa *"não tenho nada a dizer sobre esse tick"* e a ponte volta a espalhar
o movimento do frame pelos ticks devidos — a reconstrução honesta para um corpo
sendo arrastado à mão, cujas poses intermediárias nunca foram gravadas.

Os três, medidos:

- **PLAY — e este eu criei consertando os sub-steps.** O `step()` limpa a lista
  de mira todo passo, e eu mirava **FORA** do laço de ticks devidos: um frame
  devendo N ticks mirava uma vez, e o corpo cruzava o vão inteiro num tick a N×
  a velocidade. Arrastar a régua até o tick 60 **ARREMESSAVA** a carga
  (`x = 1,049` tick-a-tick vs **`-0,520`** num salto). E mirar o **mesmo alvo**
  N vezes não resolve: a mira é **absoluta**, então o corpo chega no primeiro
  passo e fica com velocidade zero pelo resto. O vão é **fatiado** entre os
  ticks devidos, pela mesma lei dos sub-steps.
- **SCRUB — a resposta dependia do CACHE.** O replay rodava `world.step()` puro,
  então um corpo kinematic ficava congelado no repouso a replay inteira:
  ~**3,4 cm** de divergência num replay parcial, e *"a caixa nunca viajou"* num
  miss do ring. Mesmo gesto, resultado diferente em dias diferentes.
- **BAKE — simulava outra cena.** Nada avançava a timeline durante o bake, então
  todo corpo que ela dirige — ou seja, **todo corpo assado antes deste** — ficava
  parado. Uma caixa sobre uma plataforma assada chega a `x ≈ 1,05` tocando, e o
  bake reportava **X constante**: nenhuma track horizontal escrita, curva só de Y
  para um objeto que anda um metro.

**Mais quatro, do produto:**

- **O bake dava `RigidBody{Kinematic}` a TODA entidade selecionada** — um sprite
  comum ganhava um órfão sem `Collider`, **invisível na §11** (que exige os dois
  para oferecer Remove) e **salvo no arquivo de projeto**; um chão estático pego
  num marquee perdia o kind autorado em silêncio. Achado pelas **duas** lentes,
  independentemente. Agora só os corpos cuja curva foi de fato escrita — e a 2ª
  porta fechou junto: o braço `Kind` do `apply_physics_edit` **ANEXAVA** o
  componente sem conferir que havia corpo, e `Kind`/`Shape` eram os únicos chips
  da §11 sem recusa por `has_body`.
- **Assar TOCANDO nunca entregava a pose:** `Playhead::rewind` preserva o play
  state **de propósito** (está no CLAUDE.md desde o load de projeto) e o
  `advance_ticks` roda **antes** do dispatch, então o relógio já passou de 0
  quando a ponte olha, `at_rest` nunca mais é verdade, e o corpo segue caindo
  como Dynamic. O bake **pausa**.
- **As chaves iam em segundos CRUS de playhead**, enquanto todo outro caminho de
  autoria do shell escreve no relógio da **ENTIDADE**. Sob um Time Remap de meia
  velocidade a curva errava a pose simulada por **1,618 m** numa queda de ~2 m —
  [[feedback_derived_coordinate_seed_must_match_sample]] ao pé da letra. Agora
  pela mesma porta (`key_time`), e um instante sem resposta única (clipe tocando
  zero ou duas vezes sob uma pilha) **RECUSA o bake inteiro**: curva com buraco
  parece simulação errada.
- **Bake dentro de bracket alheio** (recusa) e o toast que dizia *"nada se
  moveu"* para um corpo que o artista vê andando — ele já fora assado, e
  `readback` pula kinematic, então os números são idênticos aos de "parado" e a
  frase honesta é outra.

**E DOIS defeitos de GATE, os dois meus:**

- **`a_kinematic_body_follows_the_transform_it_is_given` era VAZIO.** Afirmava um
  valor que ele mesmo tinha escrito, e que o `readback` **não pode tocar** (pula
  kinematic) — ficava **VERDE com o estágio de drive inteiro deletado**, sendo um
  duplicado do gate do readback sob um nome que prometia medir o drive. O oráculo
  agora pergunta ao **SOLVER** onde o corpo está.
- **`architecture_panel_wiring_parity` só lia arquivos cujo NOME continha
  "paint"** — e o Inspector pinta quase tudo de `src/sections/*.rs`. Todo id
  registrado lá era **invisível** ao gate: pintado, hit-registrado, e livre para
  faltar no `populate.rs` sem nada notar, **no painel com mais widgets do app**.
  ⚠️ A lente diagnosticou o *fechamento* do meu refactor de LOC como causa; era o
  **nome do arquivo**, e o buraco **precede a wave**. Alargar surfou **ZERO**
  ofensor (nenhuma seção estava errada — o que faltava era algo conferindo que
  continuem certas), e agora tirar dois botões do `populate` deixa o gate
  **VERMELHO**, o que antes não deixava. Escopado a `paint*` + `sections/`: ler
  **tudo** também acusa 4 ids da Timeline/Painter, que são de outras waves e
  parecem os widgets dinâmicos que a allowlist já documenta — decidir isso é dos
  **donos deles**, então estão **nomeados aqui** em vez de allowlistados por quem
  não os escreveu (`TIMELINE_LANES`, `TIMELINE_SCROLLBAR`,
  `TIMELINE_CLIP_RENAME_INPUT`, `PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP`).

**TRÊS fixtures nasceram fracas, e as três pela mesma razão — não continham o
fenômeno** ([[feedback_moving_the_law_is_half_the_fix_the_fixture_must_contain_it]]):
o gate do scrub media o **ENDPOINT** depois de um replay para frente que lavava o
erro (um sistema amortecido re-converge — a lição do W1.5 **dentro** de um gate
escrito para honrá-la); o alvo do scrub caía **exatamente na grade do `STRIDE`**,
então o ring sedia ali e replayava **zero** passos; e a plataforma andava devagar
demais para os 7 ticks replayados aparecerem acima da tolerância.

**Duas afirmações de doc que o código não sustentava** foram corrigidas em vez de
apagadas: *"entidades sem `Transform` são puladas"* (não eram — voltavam com zero
samples e eram contadas, viradas e toastadas) e a proteção do canal constante,
que **só** protege o canal INTOCADO — um canal que o solver moveu é um canal cujo
span o bake **toma**, e keys autorados à mão lá dentro são substituídos. Isso é o
que assar SIGNIFICA, mas o parágrafo lido rápido prometia mais.

### LOC: TRÊS tetos, todos por SPLIT (nunca allowlist)

- `paint_physics_section` 211/200 → **`paint_body_actions`** (os três botões são
  uma **lista**, não um formulário: cada um é um botão, oferecido ou não, e
  nenhum lê um campo acima).
- `ph2d-physics/src/world.rs` 779/700 → **`world/kinematic.rs`**. A mira não é um
  setter de uma linha — é um alvo para o fim do **TICK** que o `step` tem de
  espalhar pelos sub-steps, e essa regra mais a interpolação de arco curto que
  ela exige são uma responsabilidade só.
- `ph2d-physics-ecs/src/bridge.rs` 728/700 → **`bridge/kinematic.rs`** (depois da
  auditoria): tudo que responde *"onde a cena diz que este corpo está"* —
  `SceneAtTick`, `FrozenScene`, a captura e o drive.

### Persistência: `PROJECT_SCHEMA` **NÃO** bumpa (de novo, e pela mesma contagem)

`BodyKind::Kinematic` é variant **apendado** — postcard codifica o discriminante
posicionalmente, então um variant no FIM mantém todo save legível, exatamente
como o `JointKind` do W3. Nenhum campo novo entrou em componente nenhum.
`DOC_VERSION` da timeline idem: o bake só **acrescenta chaves**.

### Smoke: `PH2D_PHYSICS_SMOKE=7`

Rampa + bola que **rola** + duas caixas que quicam, relógio **PAUSADO** e
timeline aberta. A rampa existe porque **rotação** é o terceiro canal assado e o
mais fácil de errar em silêncio; as duas caixas existem porque assar **dois**
corpos num clique é exatamente o caso onde um fan-out apareceria como três passos
de undo. Pausada de propósito: assar começa no tick 0 de qualquer jeito, então
uma cena já correndo faria a imagem e a curva discordarem sobre onde o movimento
começou — o tipo confuso de correto.

### Aberto no W4

- **Assar um JOINT** — hoje o bake lê a pose de **corpos**. Uma corrente assada
  vira N corpos kinematic com curvas próprias, o que reproduz o movimento mas
  descarta a articulação. Assar *a restrição* (ou recusar assar corpos unidos) é
  decisão de design, não mecânica.
- **Escolher os canais** — o bake escreve X/Y/Rotation sempre que se movem. Não
  há como pedir "só a rotação".
- **Alcance com INÍCIO** — o bake sempre parte do tick 0 (a sim é função do
  tick). Assar `[2s, 5s]` significaria assar de 0 e descartar o começo; nada pede
  isso ainda.
- **Um Ctrl+Z para as duas metades** — as chaves e o kind vivem em filas
  diferentes (acima). Unificá-las é mudança na arquitetura de undo do editor,
  não no bake.

---

# W4b — o toggle **Physics** na barra da timeline (2026-07-18, **smoke aprovado**)

> **Uma frase:** o transporte é UM relógio com DOIS consumidores — as curvas e o mundo
> rapier — e agora o artista escolhe qual deles o Play alcança. **Desmarcado por padrão.**

### O relato do Enio (o conflito era real)

> *"os controles de simulação e de animação parecer ser os mesmos e na timeline o play ativa a
> simulação física. Sendo assim temos um conflito: a simulação roda junto com a animação."*

Estava certo, e o conflito não é cosmético: revisar uma animação com scrub/play também
**derrubava mais um pouco** todo corpo dinâmico, então a cena que o artista julga nunca é
a cena que ele autorou. O pedido: checkbox na timeline, marcado = animação **e** física
(como estava), desmarcado = só animação. **Default desmarcado.**

### ⚠️ Isto CORRIGE uma nota minha do W4 que passou do ponto

O `00_plano_waves.md` §W4 dizia *"não existe interruptor … e o desligamento manual seria o
desenho errado de qualquer jeito"*. A segunda metade estava **errada por generalizar**: ela
respondia *"o Bake deve desligar a física no corpo assado?"* (não — ele **entrega a pose** via
`Kinematic`, porque o readback escreve DEPOIS do apply da timeline) e enunciou isso como
verdade sobre qualquer interruptor. São perguntas diferentes, e não se tocam:

| pergunta | quem responde | resposta |
|---|---|---|
| o solver **roda** neste take? | o TRANSPORTE (`simulate_physics`) | o artista escolhe, off por padrão |
| quem **escreve a pose** quando ele roda? | o CORPO (`BodyKind`) | `Kinematic` depois do bake |

O plano foi corrigido no lugar, com a correção datada — nota velha que contradiz o código
faz a próxima LLM propor desfazer o que existe.

### Onde o flag mora

`TimelineFlags::simulate_physics` — ao lado de `auto_key` / `frame_snap` / `performing`,
**não serializado** (`DOC_VERSION` intacto, `PROJECT_SCHEMA` intacto). É a mesma classe:
um *arm* de barra de transporte, por sessão. Isso dá o default pedido de graça, e dá a
propriedade que importa mais: **a resposta a "o que o Play faz?" é a mesma no frame
seguinte a um load que era antes do save.** Uma simulação que se arma sozinha é uma cena
que já mudou quando você olha pra ela.

⚠️ O flag vive numa crate de **outra linha** (`ph2d-timeline`). É um campo apendado a um
struct não-serializado + um variant apendado ao `TimelineIntent` — o padrão append-only do
ADR-0107. Sete seams, todos espelhados do **Record** (o toggle mais recente): flag ·
intent · `intent_apply` · snapshot (campo + `rebuild`) · id + re-export · i18n · painel
(`Item`/`ITEMS 13→14`/width/paint/`is_toggle`/`populate`) · `intent_for_transport`.

### "Off" **não** é "pule o dispatch" — é `PhysicsBridge::hold`

Esta é a decisão de projeto da wave, e as quatro coisas abaixo são quatro bugs distintos:

1. **reconcilia** — corpo autorado com o toggle off existe, guarda o repouso onde o artista
   o pôs, e **desenha o contorno**. Sem isso, física autorada com o toggle off é invisível
   até ser armada, e o mundo teria de ser construído no instante em que se quer ver
   movimento.
2. **assenta** (`settle`) — o corpo rapier acompanha o `Transform` autorado, venha ele da
   mão do artista ou de uma curva assada ⇒ **armar retoma do que está na tela**, a única
   retomada que um artista consegue prever.
3. **`last_stepped` segue o alvo** — a armadilha. Toque 10 s desarmado, arme, e a ponte
   deveria 600 ticks: um frame simula todos e a cena chega onde ninguém pediu. Gate:
   `arming_mid_take_resumes_it_does_not_replay_what_was_skipped` (mede **1** passo).
4. **o ring é DESCARTADO** — todo checkpoint descreve uma corrida que acabou. Semear um
   scrub posterior com um deles responderia com um estado de antes de o artista desarmar e
   mexer na cena à mão — e só para os ticks que por acaso estavam em cache, então o mesmo
   scrub discordaria de si mesmo conforme onde caísse.

⚠️ **Nada em `hold` escreve `Transform`.** É exatamente o que o toggle desligado promete
(física não contribui movimento nenhum), e é o que mantém a regra de ponto-fixo do frame
pausado: `settle` lê a cena e escreve o mundo rapier; o `readback`, que vai no sentido
contrário, só é alcançável pelos caminhos que dão passo.

**Preço documentado:** scrubbar PARA TRÁS sobre um trecho que nunca foi simulado o replaya
como se tivesse sido. Não há resposta melhor a dar — a trajetória daqueles ticks não existe,
porque os ticks não rodaram.

### O smoke ficaria FROZEN, e isso era o risco real do default

Com o default off, **todas** as 7 cenas `PH2D_PHYSICS_SMOKE` abririam paradas e leriam como
"a física quebrou". O prólogo do `physics_smoke.rs` arma o flag — é cena de demo de física.
E a **cena 7 pede ao artista que o DESARME**, que é a demonstração inteira do Bake: assar
converte simulação em **animação**, e animação é precisamente o que toca com o solver off.

### Gates: 12 novos, 13 mutações, **13 sangram**

| gate | arquivo | o que morre sem ele |
|---|---|---|
| `a_held_world_never_steps_however_far_the_clock_runs` | `ph2d-physics-ecs/tests/hold.rs` | o toggle não faz nada |
| `arming_after_a_held_stretch_owes_one_tick_not_the_whole_span` | idem | a avalanche de catch-up |
| `a_body_authored_while_held_is_reconciled_not_deferred` | idem | corpo invisível enquanto desarmado |
| `the_held_world_tracks_the_pose_the_scene_authored` | idem | armar teleporta pra pose velha |
| `holding_drops_the_checkpoints_of_a_run_that_is_over` | idem | scrub responde do cache morto |
| `the_transport_toggle_decides_whether_play_steps_the_solver` | `render_loop/physics_bridge_tests.rs` | **o `hold` que ninguém chama** |
| `arming_mid_take_resumes_it_does_not_replay_what_was_skipped` | idem | idem, no laço do produto |
| `the_simulation_is_disarmed_by_default` | idem | o default que o Enio pediu |
| `a_baked_take_plays_with_the_simulation_disarmed` | `physics_bake_curve_tests.rs` | a composição bake × toggle |
| `the_physics_toggle_is_painted_and_clicks_through_to_the_shell` | `ph2d-panel-timeline/tests/transport_physics_seam.rs` | pintado / registrado / roteado |
| `the_painted_switch_shows_what_the_transport_is_driving` | idem | switch que mente |
| `arming_physics_reaches_the_snapshot_the_panel_paints` | `ph2d-timeline/tests/intents.rs` | intent → flag → snapshot |

⚠️ **DOIS achados do processo de mutação, e nenhum foi "o código está certo":**

- **Um "sobrevivente" era o MEU HARNESS.** O filtro `cargo test --bins timeline_bridge_tests`
  casa com **zero** testes (o módulo é `render_loop::timeline_bridge::tests`), então o verde
  significava *nada rodou* e eu quase o registrei como gate cego. **Busca negativa exige
  controle positivo** ([[feedback_a_negative_search_needs_a_positive_control]]): conferido,
  são 8 testes, e com o filtro certo a mutação sangra.
- **Um sobrevivente era REAL, e é a armadilha de fixture de sempre.** Apagar
  `self.simulate_physics = state.flags.simulate_physics` do `rebuild` deixava tudo verde,
  porque o gate do painel **constrói o `TimelineViewSnapshot` à mão** e nunca chama o
  `rebuild`. O fixture não continha o fenômeno. Gate novo (`arming_physics_reaches_the_
  snapshot_the_panel_paints`) percorre a corrente inteira dentro da `ph2d-timeline`.

### Aberto no W4b

- **O flag não viaja no arquivo**, de propósito (ver acima). Se algum dia um projeto quiser
  abrir *já* simulando, isso é `PROJECT_SCHEMA`, não `TimelineFlags`.
- **Um corpo dinâmico não "congela" ao desarmar: ele para de ser integrado** e fica onde
  está. Não há pose de repouso automática — Reset continua sendo o gesto para isso.
- **Sem atalho de teclado** (o painel tem `L`, o de mundo tem `W`; o toggle não tem).

### Smoke: `PH2D_PHYSICS_SMOKE=7` (a cena 7 ganhou passos novos)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics && env PH2D_PHYSICS_SMOKE=7 cargo run -p ph2d-host-desktop
```

Faça o bake como antes; depois **desmarque Physics na barra** e dê Play: o movimento assado
**continua tocando** (virou animação) e a caixa **não** assada para de cair. Em qualquer
outra cena (`=1`..`=6`) o toggle nasce **marcado**, porque são demos de física.

---

---

# W5 — corpos FILHOS: o collider volta para debaixo do sprite (2026-07-18, pendente smoke)

> **Uma frase:** o solver fala MUNDO e o `Transform` guarda LOCAL; para um corpo-raiz
> os dois coincidem, e é por isso que isto atravessou quatro waves e 190 gates verdes.

### O bug, medido antes de tocar em nada

Parentear um objeto físico na Hierarquia — gesto que o app suporta inteiro — fazia o
corpo **simular num lugar e desenhar noutro**, em silêncio:

```
bola em LOCAL (0, 4) sob um pai em (5, 0)  ⇒  desenhada em (5, 4)
  solver simula em   x = 0
  renderizado em     x = 5
```

O collider não estava onde o sprite estava. Detalhe completo, com as cinco lições:
[`BUGS_physics.md`](BUGS_physics.md) **#2**.

⚠️ **O `readback` prometia isto desde o W1** — *"child bodies land in W2"*. O W2 shipou
em três pedaços e nenhum tocou nisto. Nota que promete wave futura apodrece pior que nota
errada: lida de passagem, ela tranquiliza.

### Cinco sítios, não um

Consertar só o `readback` (onde o número errado aparece) deixaria o sistema **estável e
errado**. Entrada: `body_desc` (nasce/repousa), `settle` (compara pose local com corpo em
mundo ⇒ todo filho parece "movido à mão" todo frame pausado), `drive_kinematic` (a
plataforma anda por um caminho que ninguém autorou e leva a carga), `reconcile_joints` (a
âncora é um ponto do mundo). Saída: `readback`.

### A álgebra do repo é INVERTÍVEL, e isso apagou o compromisso

Eu ia propor *"só pai rígido; escala não suportada"*. Não precisou: `compose` soma
rotações, multiplica escalas e cisalha com `[[1, tan sx], [tan sy, 1]]` ⇒ existe inverso
exato. `Transform::inverse_compose` mora **ao lado** da lei que inverte, e o round-trip é
a especificação inteira — erro medido **6,08e-6** sobre rotações, escala não-uniforme,
escala negativa e skew nos dois eixos.

⚠️ **A guarda de degeneração não é um limiar, e a v1 estava errada.** `det == 0.0` nasceu
vermelha: em `f32` um shear construído para ser singular dá `det ≈ 1e-8`. Virou **"todo
campo do resultado é finito?"** — a pergunta que o chamador tem (*posso guardar isto?*),
sem número mágico, e que ainda recusa `NaN` vindo da **entrada**. Pai mal-condicionado
passa de propósito (local enorme, mas `compose` o leva de volta à pose certa).

### ⚠️ Eram SEIS sítios, não cinco — o overlay é da shell

O `render_loop/physics_overlay.rs` lia o `Transform` cru e desenhava cada
contorno na pose LOCAL, enquanto o sprite vem da cadeia composta: os contornos de
uma cena parenteada empilhavam no meio, longe das artes. Foi o que o Enio viu no
smoke (*"os colliders estão deslocados de suas sprites"*), e é a leitura correta
do primeiro relato dele — o que parecia "rigs sobrepostos no centro" eram os
CONTORNOS. Porta única definitiva: **`ph2d_ecs::world_transform{,_into}`**, com o
`bridge::space` delegando a ela. Os 12 gates do overlay eram todos corpo-raiz e
ficaram verdes; o gate novo tem um pai.

### Onde mora o quê

| Camada | O quê |
|---|---|
| `ph2d-ecs::transform_inverse` (módulo NOVO) | `Transform::inverse_compose` · `Transform::is_finite` · `parent_world_transform` + `parent_world_transform_into` (sem alocar; a versão antiga **delega**) |
| `ph2d-physics-ecs::bridge::space` (módulo NOVO) | as **duas portas**: `world_transform` (entrada) e `write_world_pose` (saída, recusa pai não-inversível) |
| `PhysicsBridge.chain` | buffer persistente do caminhador ⇒ zero alocação por frame |

### Gates: 12 novos, 10 mutações, **10 sangram**

Um por sítio religado (uma asserção só "o filho acaba no lugar certo" fica verde sobre
quase todos), mais a guarda, o `clear()` do scratch e a subtração de rotação do inverso.
⚠️ O `hot_path_no_alloc` **não tinha hierarquia na fixture**, então a promessa de
zero-alloc do caminho novo era afirmada sobre código que ela nunca entrava; agora tem um
corpo parenteado a dois níveis, e `scratch_capacity` **soma** os buffers (vigiar um
enquanto o outro dobra ao lado é o mesmo furo).

Regressão pinada: **corpo-raiz é byte-idêntico**. `compose(IDENTITY, local)` e
`inverse_compose(IDENTITY, mundo)` são exatos, então rotear a raiz pela conversão nova
não pode mudar nada — sem esse gate a wave poderia pagar o suporte a filhos com um drift
silencioso no caso que é 99% das cenas.

### LOC: dois tetos, os dois por SPLIT

`transform.rs` estava **exatamente** na sua linha congelada (784) e meu acréscimo levou a
907 ⇒ o irmão `transform_inverse.rs`. Como levei junto a `parent_world_transform`
pré-existente, a catraca **BAIXOU para 768** (a única direção que aquela tabela deve
andar). `physics_smoke.rs` foi a 704/600 ⇒ irmão `physics_smoke_rigs.rs` com as cenas
6/7/8 — a costura é real: tudo ali precisa de uma **segunda** coisa para significar algo
(um joint precisa de dois corpos, um bake precisa da timeline, um filho precisa de um
ancestral).

### Aberto no W5

- **A escala não alcança o collider** — um sprite escalado 2× tem collider do tamanho
  autorado. **Pré-existente e igual para corpo-raiz** (`body_desc` lê `col.shape`
  verbatim), portanto ortogonal a esta wave; consertar aqui misturaria duas correções.
  É wave própria, e vale para os dois casos.
- **`GlobalTransform` não é consultado** — a ponte compõe a cadeia ela mesma, porque
  aquele componente é `PresentComponent` (vive no mundo de apresentação, reconstruído no
  extract) e a física roda sobre o `SimWorld`. Se algum dia a propagação publicar no sim,
  esta é a segunda porta a fechar.
- **O overlay de contorno** desenha do solver, que agora está certo em qualquer
  profundidade — mas nada gateia o *desenho*; o smoke é o oráculo.

### Smoke: `PH2D_PHYSICS_SMOKE=8`

```
env PH2D_PHYSICS_SMOKE=8 cargo run -p ph2d-host-desktop
```

Três rigs — um nível, dois níveis, e um **rotacionado** — cada um com uma bola física
parenteada, cada um sobre um pedestal **estreito**. Cada rig é um **quadradinho azul**
visível: sem `Sprite` uma entidade não publica `GizmoView`, então o primeiro corte tinha
rigs invisíveis e mandava o artista arrastá-los (⚠️ ver `BUGS_physics.md` #2, seção da
fixture invertida — a bola do rig rotacionado também errava o próprio pedestal, e a cena
**premiava** a implementação bugada). A regressão é inconfundível por
construção: um corpo que volte a ler a pose local como mundo cai pela linha `x = 0`, erra
o pedestal sobre o qual foi desenhado, e some de quadro. Tecla `B`: cada contorno tem de
sentar exatamente no seu sprite, em toda profundidade. Depois **arraste um rig** no
viewport: a bola acompanha, mantém o collider e continua colidindo.

---

---

## Decisões (ADR-0131, condensadas — o *porquê* está lá)

- **D1** runtime-truth + bake opcional (Enio). **D2** `PhysicsWorld` transiente shell-side (precedente
  `MotionCookPump`), dirigido por components; NÃO persistido (é rebuild). **D3** contrato
  `RigidBody`/`Collider` append-only, registrado pela crate-ponte, destinado a congelar. **D4** escala
  **D4 corrigido no W1: sem porta de escala** — `Transform` já é metros = rapier metros (1:1); a única px→m
  é `ProjectSettings.pixels_per_meter` no import (do projeto). **D5** relógio no `Playhead`
  (`ticks_owed`); scrub por **checkpoint ring esparso** (modelo `CheckpointRing`/`Cook`). **D6** fronteira
  tríplice (rapier / Zona-de-nós / XPBD). **D7** hash do mundo-ECS estende o gate c9 cross-OS. **D8**
  painel global (categoria nova) + seção "Physics Body" no Inspector. **D9** rígido apenas; 0063 fora.
  **D10** budgets 1,5 ms / 20 MB / zero-alloc. **D11** bake via `fit_fcurve`/Schneider.

---

## Terreno verificado on-disk (2026-07-17 — NÃO re-derive; cite daqui)

### O que herda pronto — `ph2d-physics` (M10)
- [`crates/ph2d-physics/src/world.rs`](../../crates/ph2d-physics/src/world.rs) (320 LOC,
  `#![forbid(unsafe_code)]`): `PhysicsWorld::new/set_gravity/set_dt/dt/step_count/add_dynamic_circle/
  add_static_cuboid/insert_body/bodies[_mut]/colliders[_mut]/step/body_pose/body_snapshots/
  deterministic_hash`. `step()` **sempre** usa `dt` interno (HR-5). `DEFAULT_DT=1/60`,
  `DEFAULT_GRAVITY_Y=-9.81`, mundo **Y-up**. `BodySnapshot{handle_index,x,y,rotation,linvel_x,linvel_y,angvel}`
  ordenado por `handle_index`; `deterministic_hash` = blake3 sobre snapshots ordenados (`to_bits` LE).
- [`crates/ph2d-physics/Cargo.toml`](../../crates/ph2d-physics/Cargo.toml): `rapier2d = "0.28"`,
  `default-features=false`, features `dim2`/`f32`/`enhanced-determinism` + `blake3`. **NUNCA** ligar
  `parallel`/`simd-stable`/`simd-nightly`.
- Bin [`c9.rs`](../../crates/ph2d-physics/src/bin/c9.rs): 50 corpos + chão, 120 steps, imprime
  `physics-c9 hash: <hex64>`.

### O gate cross-OS REAL (o path da SKILL não existe)
- **`.github/workflows/spike.yml`**: job `determinism` (matriz `[ubuntu-latest, macos-latest,
  windows-latest]`, `fail-fast:false`) roda `cargo run --release --locked --bin ph2d_physics_c9
  -p ph2d-physics`, parseia `grep -E '^physics-c9 hash: ' | awk '{print $3}'`, sobe artifact
  `physics-c9-hash-${os}`. Job `determinism-compare` (needs `determinism`) baixa os 3 e exige
  `sort -u | wc -l == 1`.
- ⚠️ **`tests/determinism/replay_cross_platform.rs` NÃO existe on-disk** (a SKILL mente). A verdade é o
  `spike.yml` + os bins `c9.rs` (physics) e `tests/spike/src/bin/c9_replay.rs` (ECS). **W1 adiciona
  `physics-ecs-c9`** (novo bin/harness + etapa de matriz + artifact + comparação).

### O relógio
- [`crates/ph2d-core/src/time.rs`](../../crates/ph2d-core/src/time.rs): `FixedStep` — `DEFAULT_HZ=60.0`
  (f64), `DEFAULT_MAX_SUBSTEPS=8`, `advance(wall_dt)->FixedStepReport{ticks:u32,alpha:f32,dropped_secs:f64}`,
  `tick_count()->u64`, `fixed_dt()->f64`.
- [`crates/ph2d-core/src/playhead.rs`](../../crates/ph2d-core/src/playhead.rs): `Playhead` — `time:f64` seg,
  `advance()` move só se `playing`, `advance_ticks(n)`, `seek/seek_frame` (scrub, não muda play state),
  `rewind()` (time=0, mantém rate+play), `is_playing`, loop Wrap/PingPong. Sequência bit-idêntica cross-OS
  (HR-5).
- **Precedente Motion** [`shells/desktop/src/render_loop/motion_bridge.rs`](../../shells/desktop/src/render_loop/motion_bridge.rs):
  `ticks_owed(last_cooked, target) -> RangeInclusive<u64>` (`Some(last) if target>last => last+1..=target`;
  senão `target..=target`); caller `for tick in ticks_owed(...) { pump.advance_or_scrub_scoped(...) }`;
  `target = round(playhead.time()/fixed_dt)`. **`MotionTransport` MORREU** — um relógio.

### O checkpoint (modelo do scrub — W1.5)
- [`crates/ph2d-nodegraph/src/cook.rs`](../../crates/ph2d-nodegraph/src/cook.rs): `CookCheckpoint`,
  `checkpoint()->CookCheckpoint`, `restore(&cp)` (reinstala estado + limpa memo/live-scope, mantém revision
  clock). GGPO save/load/advance.
- [`crates/ph2d-eval-motion/src/checkpoint.rs`](../../crates/ph2d-eval-motion/src/checkpoint.rs):
  `RECENT_CAPACITY=300` (~5 s @60Hz), `CheckpointRing{recent:VecDeque<(u64,CookCheckpoint)>}` denso,
  `record`/`anchor_at_or_before(target)->(u64,cp)`/`should_record`/`clear` (no `mark_dirty`). Física usa
  cadência **esparsa** (estado maior).

### Registro de components (a armadilha do snapshot)
- [`crates/ph2d-ecs/src/scene/registry.rs`](../../crates/ph2d-ecs/src/scene/registry.rs):
  `register::<T>("ph2d::ecs::Nome")`; ids = blake3(name) 8 bytes LE. `register_ecs_components(reg)` +
  tripwire `register_ecs_components_populates_registry` (`reg.len()==32`, *"este número existe para doer"*).
  **Padrão:** a crate-home possui `register_*` e o boot agrega
  ([`shells/desktop/src/init.rs`](../../shells/desktop/src/init.rs), ao lado de `register_render_components`).
  Physics segue isso → `register_physics_components` na crate-ponte, contagem-32 de `ph2d-ecs` **intocada**.

### Painel docado — 5 sites (canônico: `ph2d-panel-vector`)
1. `impl Panel` (`ID`/`NODE_ID`/`DEFAULT_VISIBLE`/`populate`/`paint`/`apply_event`).
2. push no `ph2d-panel-registry-init` (GERADO por `ph2d-panel-sync`) + const `EXPECTED_TYPED` à mão.
3. feature Cargo `panel-<x>`.
4. **lista de fallback de z-order em `hero/paint.rs`** (sem ela = registrado+visível mas NUNCA pintado).
5. visibilidade dirigida pela ponte (`hero.panel_visibility.insert("<x>", ...)` no `render_loop`).

### Fora de escopo (Chesterton)
- **ADR-0063** (collider-gen vetorial + fratura dinâmica): amarrada ao `ph2d-vector-runtime` que a
  **ADR-0108** aposentou. Motor app-level **não reabre a 0108 nem herda os mecanismos da 0063**.
- **XPBD soft** (`ph2d-physics-soft`, M13+) e **FLIP/PIC** (`ph2d-fluids`, M13+): linhas próprias.

---

## Ids / consts / variants — ALOCADOS e A ALOCAR (regra §1.5.9.3)

**Alocados e CRIADOS no W1:**
- Crate-ponte **`ph2d-physics-ecs`** (glob `crates/*` — zero edit central). Components `RigidBody`/`Collider`;
  enums `BodyKind{Dynamic,Static}` / `ColliderShape{Ball,Cuboid}` (append-only, variants novos no FIM).
  Nomes canônicos de registro: **`ph2d::physics::RigidBody`** / **`ph2d::physics::Collider`**.
  `register_physics_components`; `PhysicsBridge`; bin **`physics_ecs_c9`**.
- `ph2d-physics` (aditivo): `BodyDesc`/`ShapeDesc`/`spawn_body`/`set_body_pose`/`remove_body`.
- Shell: campo **`AppGfx.physics`**; módulo **`render_loop/physics_bridge`**; **`mod physics_smoke`**;
  **`App.physics_smoke_done`**; feature de Cargo `ph2d-physics-ecs` (dep de path no shell).
- Env de smoke: **`PH2D_PHYSICS_SMOKE`** (=1 usado; 2=painel/3=joint/4=bake **reservados**).
- CI: **`physics-ecs-c9`** + artifact **`physics-ecs-c9-hash-${os}`** (spike.yml).
- **`PROJECT_SCHEMA` = 16** (era 15) + tripla-pin `(16,7,8)` em `project_tests`.
- ADR **0131** (era 0130 — renumerado na integração de 2026-07-18: a `line/gpu-nodes` reclamou o 0130 no mesmo dia).
- ~~`PIXELS_PER_METER`~~ **NÃO existe** — D4 corrigido; reusa `ProjectSettings.pixels_per_meter`.

**Alocados e CRIADOS no W4b (o toggle Physics do transporte):**
- `ph2d-timeline` (append-only, **zero bump de schema** — nada disto é serializado):
  campo **`TimelineFlags::simulate_physics`** (default `false`) · variant
  **`TimelineIntent::SetSimulatePhysics(bool)`** (apendado ao fim do bloco de flags) ·
  campo **`TimelineViewSnapshot::simulate_physics`** (+ preenchimento em `rebuild`).
- `ph2d-editor-core`: NodeId **`TIMELINE_PHYSICS`** = `hash_node_id("timeline.physics")`
  (bloco *Transport bar*, apendado) — re-exportado em `ph2d-panel-timeline/src/ids.rs`.
- `ph2d-i18n`: chave **`panel.timeline.physics`** → `"Physics"`.
- `ph2d-panel-timeline`: variant **`Item::Physics`** + **`ITEMS: [Item; 13] → [Item; 14]`**.
- `ph2d-physics-ecs`: método público **`PhysicsBridge::hold`** (módulo novo
  **`src/bridge/hold.rs`**, split de LOC) + método privado **`prepare`** (prólogo
  compartilhado com o `dispatch_with_scene` — porta única).
- Shell: parâmetro **`simulate: bool`** em `physics_bridge::dispatch` (⚠️ **assinatura
  MUDOU** — 1 chamador, o `render_loop/mod.rs`); módulo de teste
  **`render_loop/physics_bridge_tests`**.
- Testes novos: `ph2d-physics-ecs/tests/hold.rs` ·
  `ph2d-panel-timeline/tests/transport_physics_seam.rs`.

**Alocados e CRIADOS no W5 (corpos filhos):**
- `ph2d-ecs`: módulo NOVO **`transform_inverse.rs`** (`pub mod`) — `Transform::inverse_compose`,
  `Transform::is_finite`, e as duas `parent_world_transform{,_into}` **movidas** para lá
  (re-exportadas do `lib.rs`, então os chamadores não mudam).
- `ph2d-physics-ecs`: módulo NOVO **`src/bridge/space.rs`** (privado) + campo
  **`PhysicsBridge.chain`**. ⚠️ `scratch_capacity()` passou a **somar** os buffers.
- `shells/desktop`: módulo NOVO **`physics_smoke_rigs.rs`** (cenas 6/7/8 movidas; `spawn_floor`
  virou `pub(crate)`); env **`PH2D_PHYSICS_SMOKE=8`**.
- Testes novos: `ph2d-ecs/tests/transform_inverse.rs` · `ph2d-physics-ecs/tests/child_bodies.rs`.
- ⚠️ Catraca de LOC **BAIXADA**: `ph2d-ecs/src/transform.rs` 784 → **768**.

**Alocados e CRIADOS no W3:**
- `ph2d-ecs` (aditivo): **`stable_name_id`** em `name.rs` (+ re-export no `lib.rs`). O
  `shells/desktop/src/timeline_persist.rs::wire_id_for_name` passou a **delegar** — mesma FNV-1a,
  byte a byte, pinada contra valores externos.
- `ph2d-physics` (aditivo): módulo **`world/joints.rs`** — `JointDesc`/`JointKind{Pin,Spring,Rope}`/
  `MotorDesc`, `spawn_joint`/`remove_joint`/`joint_count`/`joint_anchors`; re-export de
  **`ImpulseJointHandle`**. Const privada `MOTOR_TRACKING`.
- `ph2d-physics-ecs`: component **`PhysicsJoint`** + enum **`JointKind`** (`src/joint.rs`), nome
  canônico **`ph2d::physics::PhysicsJoint`** (registrado; a contagem do registry foi 2 → **3**).
  Módulo **`src/bridge/joints.rs`**. Dev-dep **`postcard`**.
- `ph2d-editor-core`: **`InspectorJointInfo`** + **`JointFieldEdit`** (`inspector_model.rs`), campo
  **`InspectorPhysicsInfo.can_join`**, variant **`PhysicsFieldEdit::Join`**, variant de ação
  **`EditorAction::InspectorJointEdit`**. Ids §12 (23 novos, todos na tabela do
  `node_id_collisions`): `INSP_LIVE_JOINT_SECTION/_COLOR`, `INSP_JOINT_{KIND,LIMITS,MOTOR}_GROUP`,
  `INSP_JOINT_KIND[3]`, `INSP_JOINT_{LIMITS,MOTOR}[2]`, `INSP_JOINT_LIMIT_{MIN,MAX}`,
  `INSP_JOINT_MOTOR_{SPEED,FORCE}`, `INSP_JOINT_{REST_LENGTH,STIFFNESS,DAMPING,MAX_LENGTH,REMOVE}`,
  **`INSP_PHYS_JOIN`**.
  ⚠️ `any_live_section` foi `[bool; 8]` → **`[bool; 9]`** e o array de slots de nota 10 → **11**
  (os dois são rígidos DE PROPÓSITO — *"a signature that changes when you forget"*).
  ⚠️ Allowance de LOC de `paint_inspector` **permanece 424**: a §12 custou ~22 e pagou movendo a
  família de física inteira para `paint_frame::paint_physics_sections`. **Está na linha.**
- `ph2d-panel-inspector`: `sections/joint.rs`, `sections/rows.rs` (helpers compartilhados,
  extraídos de `physics.rs`), `event_joint.rs`, `tests/seam_joint.rs`.
- Shell: `render_loop/inspector_joint.rs`, `render_loop/inspector_joint_tests.rs`,
  `render_loop/physics_overlay_joints.rs`, `tests/join_is_one_gesture_not_a_fan_out.rs`;
  `physics_smoke_joints` (**`PH2D_PHYSICS_SMOKE=6`** — o 6 estava reservado no W1 como "bake",
  que agora é o **7**).
- **`PROJECT_SCHEMA` INTOCADO em 21** — ver §W3 (a contagem deu zero).

**Alocados e CRIADOS no W4:**
- `ph2d-physics` (aditivo): módulo **`world/kinematic.rs`** — `set_next_kinematic_pose`,
  `kinematic_slice` (`pub(super)`), acessor `#[doc(hidden)] kinematic_aim_count`; campo
  **`PhysicsWorld.kinematic_targets`**.
- `ph2d-physics-ecs` (aditivo): módulo **`src/bake.rs`** — `BakedTrajectory`/`PoseChannel`/
  `bake_trajectories`; variant **`BodyKind::Kinematic`** (APENDADO, tag `2`) +
  `BodyKind::{solver_owns_pose,tag,from_tag}`; estágio privado `PhysicsBridge::drive_kinematic`.
- `ph2d-editor-core`: id **`INSP_PHYS_BAKE`** (na tabela do `node_id_collisions`) ·
  **`INSP_PHYS_KIND` foi `[NodeId; 2]` → `[NodeId; 3]`** (o 3º entrou na tabela — ela é escrita
  à MÃO por índice e parava no `[1]`, então o chip novo não era conferido) · variant
  **`PhysicsFieldEdit::Bake`** · campo **`InspectorPhysicsInfo.bake_seconds`**.
- `ph2d-panel-inspector`: `sections/physics.rs::paint_body_actions` (split do teto de 200 LOC);
  `KIND_LABELS` foi `[&str; 2]` → `[&str; 3]`.
- Shell: `render_loop/physics_bake.rs` + `render_loop/physics_bake_tests.rs` ·
  **`render_loop/record_fit.rs`** (extraído do `autokey_pass.rs`: `RecSpan`, `simplify_recorded`
  — que ganhou o parâmetro `smooth_passes` —, `value_tol`, as 4 consts do record) ·
  `physics_smoke_bake` (**`PH2D_PHYSICS_SMOKE=7`**) · `KINEMATIC_RGBA` no `physics_overlay.rs` ·
  `build_physics_info` e `snapshots::publish` ganharam o parâmetro `bake_seconds`.
- **RENOMEADO:** `shells/desktop/tests/join_is_one_gesture_not_a_fan_out.rs` →
  **`selection_gestures_are_not_fanned_out.rs`** (agora cobre Join **e** Bake).
- **`PROJECT_SCHEMA` INTOCADO em 21** e **`DOC_VERSION` intocado** — variant apendado não move
  layout, e o bake só acrescenta chaves.

**Alocados e CRIADOS no W2b:**
- Crate **`ph2d-panel-physics`** (glob `crates/*`), `Panel::ID = "physics"`, struct
  **`PhysicsPanel`** (o nome é load-bearing: o `ph2d-panel-sync` faz parse de `pub struct <N>Panel`).
- Feature **`panel-physics`** (gerada) + a entrada na lista `default` e o `EXPECTED_TYPED`
  **18 → 19** (as duas **à mão** — o sync não as regenera).
- **29 ids** `PHYSICS_*` em `ids/chrome/physics.rs` (slug family `physics.*`, distinta do
  `INSP_PHYS_*` do Inspector) — todos na tabela de `node_id_collisions`.
- **`PHYSICS_SCROLLBAR_ID = NodeId(836)`** (o próximo livre agora é **837**).
- `ph2d-physics::BodyDefaults` + `world/defaults.rs`; `ph2d-physics-ecs::PhysicsSettings` +
  `settings.rs` + as consts de range (`MAX_SUBSTEPS`/`MAX_SOLVER_ITERATIONS`/`MIN_CONTACT_HZ`/
  `MAX_CONTACT_HZ`/`MAX_DAMPING`/`MAX_SLEEP_THRESHOLD`/`MAX_TIME_UNTIL_SLEEP`/`GRAVITY_LIMIT`/
  `DEFAULT_SOLVER_ITERATIONS`).
- Shell: `render_loop/physics_panel_bridge.rs` (**nome distinto do `physics_bridge`**, que é a
  simulação — duas pontes, duas fases), **tecla `W`**, cena de smoke **`PH2D_PHYSICS_SMOKE=4`**.
- **`PROJECT_SCHEMA` = 19** + tripla-pin `(19, 8, 8)`; `ProjectFile.physics` (6º campo).
- i18n: 21 chaves `panel.physics.*`.

**A alocar na wave que os cria (próximo LIVRE):**
- W2c: `Collider.layer` (append) + `PhysicsSettings.layer_matrix` + ids da matriz (**dinâmicos** —
  precisam do gate irmão de colisão, o `architecture_panel_wiring_parity` NÃO vê registro em laço)
  + `PROJECT_SCHEMA` **19 → 20**.
- W3: `PROJECT_SCHEMA` **19 → 20** (ou 20 → 21 se o W2c vier antes — o valor se **CONTA**) + a
  tripla-pin; components de joint.

---

## Handoff de INTEGRAÇÃO — W0 + W1 (§1.5.9)

> Reportar ao Enio e **PARAR** (regra E/H). NÃO integrar, NÃO pushar.

1. **Identidade:** branch `line/physics`; base (merge-base com main) = `cdc3acc1`; HEAD + nº de commits =
   `git log --oneline cdc3acc1..HEAD` no momento da integração (W0: docs · W1: `44e08cf5` core,
   `018b00e9` wiring, `9f5fee05` gate, + docs de correção por cima).
2. **Foundational/compartilhado tocado:**
   - `crates/ph2d-physics/` — **meu módulo** (regra B), **aditivo**: `spawn_body`/`set_body_pose`/
     `remove_body` + `BodyDesc`/`ShapeDesc`. Helpers existentes + `step` + c9 **byte-idênticos** (hash
     `physics-c9` intacto = `2114f483…`).
   - `shells/desktop/` (o consumidor É parte do work item): `Cargo.toml` (+dep), `app_state.rs` (+campo
     `physics` + `physics_smoke_done`), `init.rs` (+construtor + registro), `main.rs` (+`mod physics_smoke`
     + init do latch), `project.rs` (schema 15→16 + `rebuild()` no load), `project_tests.rs` (tripla-pin),
     `render_loop/mod.rs` (+`mod physics_bridge` + `dispatch` antes do `sim_extract`), **novos**
     `physics_smoke.rs` + `render_loop/physics_bridge.rs`.
   - `.github/workflows/spike.yml` (+step/artifact/compare `physics-ecs-c9`). `Cargo.lock`.
   - **`ph2d-ecs` NÃO foi tocado** (só lido; o registro mora na minha crate).
   - **`ph2d-editor-core` (transport, foundational-shared):** `action_bus.rs` (+`EditorAction::Transport`
     variant + `TransportCmd` enum, aditivo), `screens/hero/chrome/transport.rs` (**novo** handler z=300),
     `screens/hero/chrome/mod.rs` (**bloco GERADO** re-sincronizado por `ph2d-chrome-sync`),
     `screens/hero/topbar/mod.rs` (tooltips). Shell: `transport.rs` (**novo**, a porta única), `main.rs`
     (`mod transport`), `render_loop/mod.rs` (arm do dreno).
3. **Símbolos que podem COLIDIR (grep na integração):**
   - **ADR `0130` → RESOLVIDO como `0131`** (a `line/gpu-nodes` também o reclamou; gate `architecture_adr_numbers_are_unique`). Renomeio
     escopado a `git diff --name-only`, **nunca** `git grep` de árvore ([[feedback_a_token_rewrite_scopes_to_changed_files_not_the_whole_tree]]).
   - **`PROJECT_SCHEMA` = 16 + a tripla-pin `(16,7,8)`** — ⚠️ **se OUTRA linha também bumpar o schema, o
     valor se CONTA, não se escolhe** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]): some os
     dois deltas (ex.: se outra linha subiu p/ 16 por outro motivo, o combinado é 17) e atualize a tripla.
     O gate `a_schema_bump_anywhere_must_bump_the_project_schema` fica **vermelho** até baterem.
   - Listas append-only que o Mergiraf funde mas o integrador confere: `mod physics_smoke;`/`mod transport;`
     (main.rs), `mod physics_bridge;`(render_loop/mod.rs), o campo `AppGfx.physics` + seu destructure, o bloco
     `component_registry` de `init.rs`, os `mod`/prólogo do frame, o `match` de `EditorAction` no dreno.
   - **`EditorAction::Transport` + `TransportCmd`** (append em `action_bus.rs`) — se outra linha também
     apendar variant no `EditorAction`, Mergiraf funde (variants distintos), mas confira. **`chrome/mod.rs`
     é GERADO** (bloco `<ph2d-chrome-sync:...>`): conflito ali = **re-rode `cargo run -p ph2d-chrome-sync`**,
     NUNCA resolva na mão (DIRETRIZ §1.5.5); o gate `architecture_chrome_dispatch_in_sync` confirma. Marcador
     `z=300` no `chrome/transport.rs` (próximo livre; os outros vão até 290).
   - Nomes de código (únicos, improváveis de colidir): `ph2d::physics::{RigidBody,Collider}`,
     `physics-ecs-c9-hash-*`, `PH2D_PHYSICS_SMOKE`.
4. **Contratos congelados encostados:** **NENHUM**. O contrato de física é novo e não-congelado.
5. **O que só o `ship.sh`/CI pega:** `typos` (pt-BR + comentários) · `machete` (deps novas: `bevy_ecs`+`blake3`
   na ponte, `ph2d-physics-ecs` no shell — todas USADAS) · `deny`/`audit` (sem crate externa nova além de
   `bevy_ecs`, já na árvore) · a **matriz cross-OS do `physics-ecs-c9`** (o verdadeiro gate HR-5 — só roda no
   push; localmente só provei repeatability + os guardas estruturais). O `spike.yml` **não** é validável por
   yamllint local (indisponível) — os blocos são mirror exato dos existentes.
6. **O que smoke-testar (Enio):** `cd Worktrees/line-physics && env PH2D_PHYSICS_SMOKE=1 cargo run -p
   ph2d-host-desktop` → a bola cai e assenta. **E confirme que o app normal (sem a env) segue igual** — o
   `physics_bridge::dispatch` roda todo frame, mas é no-op sem entidades de física (query vazia).

**Resumo:** *Linha `physics` (W0+W1) pronta — HEAD `9f5fee05`, 5 commits. Foundational tocado: `ph2d-physics`
(meu módulo, aditivo, c9 intacto) + shell (consumidor). Contratos congelados: nenhum. Colisões a grepar: ADR
0130 · `PROJECT_SCHEMA=16`+tripla-pin (CONTAR se outra linha bumpar). 6 gates mutation-verified; batched gate
verde. Smoke pendente: `PH2D_PHYSICS_SMOKE=1`. Aguardo ordem de integração / W1.5 / W2.*

---

## §W6 — A ESCALA ALCANÇA O COLLIDER (2026-07-19, smokada pelos gates)

**Reaberta a linha pós-integração** (o plano original acabou; ver
[`HANDOFF_CONTINUACAO_line_physics_2026-07-19.md`](HANDOFF_CONTINUACAO_line_physics_2026-07-19.md)).
O Enio escolheu do cardápio o item **(A)** — *a escala não alcança o collider*, a única
CORREÇÃO da lista. A causa e as lições estão em [`BUGS_physics.md`](BUGS_physics.md) **#3**;
o resumo de produto está na entrada de física do `CLAUDE.md` §5. Aqui, só o essencial de
integração.

**O quê:** um sprite escalado 2× desenhava 2× e o collider **não** — `body_desc` lia
`col.shape` verbatim (translation + rotation, nunca scale). Agora a **escala de MUNDO**
(`t.scale`, o `t` já é world desde o W5) alcança o collider, pela porta única
**`ph2d_physics_ecs::scaled_shape(ColliderShape, scale) -> ShapeDesc`** — lida pela ponte
(→ rapier) E pelo overlay (→ o wireframe), senão o contorno mentiria o tamanho.

**A decisão de produto (do Enio):** Cuboid toma escala per-eixo nativamente; Ball uniforme
fica círculo; **Ball não-uniforme vira ELIPSE** (não colapsa num círculo como a Unity) —
variant novo **`ShapeDesc::Ellipse{rx,ry}`** (polígono convexo via `ellipse_vertices` +
`convex_polyline`; tesselação `libm` p/ determinismo). O collider casa com o sprite.

**⚠️ ZERO bump de schema:** o `ColliderShape` **autorado** não mudou (ainda Ball/Cuboid); a
elipse é derivada só na plain-data `ShapeDesc`; a escala já vive no `Transform` persistido.
`PROJECT_SCHEMA` intacto. Nada a CONTAR.

**Arquivos (todos aditivos, isolados no módulo):**
- `crates/ph2d-physics/src/world/shape.rs` (**novo** — `ShapeDesc` + `ELLIPSE_SEGS` +
  `ellipse_vertices`, split de `world.rs` que bateu o teto de 700 LOC) · `world.rs` (arm
  `Ellipse` em `spawn_body`, `pub use shape::…`) · `lib.rs` (re-export) · `Cargo.toml`
  (`libm = "=0.2.16"`, o pin do workspace — machete/deny OK).
- `crates/ph2d-physics-ecs/src/scale.rs` (**novo** — `scaled_shape`) · `bridge.rs`
  (`body_desc` chama `scaled_shape`) · `lib.rs` (re-export `scaled_shape`/`ShapeDesc`/
  `ellipse_vertices`/`ELLIPSE_SEGS`) · `bin/physics_ecs_c9.rs` (+1 bola escalada ⇒ **52
  corpos** — o hash MUDA, e é comparado só entre OSes, nunca a um literal nem ao raw c9).
- `shells/desktop/src/render_loop/physics_overlay.rs` (`collider_outline` recebe `ShapeDesc`,
  arm `Ellipse`; `outlines` resolve por `scaled_shape`).

**Gates (8+2 novos, 7 mutações, todas sangram):**
`crates/ph2d-physics-ecs/tests/scale_reaches_the_collider.rs` · `crates/ph2d-physics/tests/ellipse_collider.rs`
· `render_loop::physics_overlay::tests` (2 novos). Verde local: ambas as crates de física
(31 bins), overlay (11/11), LOC-cap, fmt, clippy `--all-targets`, machete, typos (diff).

**Contratos congelados:** nenhum tocado. **Foundational tocado:** `ph2d-physics` (meu módulo
de física, aditivo — `ShapeDesc` é append-only, c9 intacto exceto o +1 corpo). **Colisões a
grepar na integração:** nenhuma constante de schema mudou; o único número móvel é o
`body_count` do c9 (51→52, só cosmético no log). **Aberto (herdado, não desta wave):** escala
não-uniforme + **rotação** compõe um cisalhamento que a decomposição do `world_transform` põe
em `scale`+`skew`; o **skew é ignorado** (rapier não cisalha collider — a mesma limitação
honesta que o Cuboid sempre teve).

**Smoke visual:** `PH2D_PHYSICS_SMOKE=9` (`shells/desktop/src/physics_smoke.rs::physics_smoke_scale`)
— 4 bolas caem, cada uma um `Ball` escalado diferente: **círculo** de referência · **2×
uniforme** (círculo maior, repousa mais alto) · **não-uniforme** (ELIPSE, cai deitada e
balança) · **parenteada** sob um rig 2× (o collider herda a escala do pai). O oráculo é o
contorno (tecla `B`, default ON): desenha a forma RESOLVIDA, então um scale→collider morto
traçaria o raio autorado dentro de cada sprite escalado. Os gates behavioral já cobrem a
física; a cena é para o olho.

---

## §W7 — SENSORES / TRIGGERS (2026-07-19, smokada pelos gates + smoke `=10`)

**Item (B) do cardápio** ([`HANDOFF_CONTINUACAO_line_physics_2026-07-19.md`](HANDOFF_CONTINUACAO_line_physics_2026-07-19.md)),
pedido pelo Enio. O `is_sensor` era só um esboço no ADR-0131 (*"waits for a consumer of its
own"*); esta wave constrói o **primitivo de trigger**.

**O quê:** um `Collider.is_sensor` que **atravessa** (sem forças de contato) mas o solver
**reporta o que o sobrepõe**. A metade da DETECÇÃO é limpa e contida na física; o **consumidor
de GAMEPLAY** (colisão→sinal via `ph2d-script`/`Marker` da timeline) é **outra camada, cross-line,
decisão do Enio** — este primitivo é a fundação necessária, não trabalho jogado fora.

**O consumidor VISÍVEL desta wave** (pra o sensor não ser flag morto):
- **Overlay acende** o sensor: magenta apagado (`SENSOR_IDLE`) → brilhante (`SENSOR_ACTIVE`)
  quando algo está dentro. O overlay resolve `triggered_sensors()` do bridge.
- **Estado consultável:** `PhysicsBridge::{is_triggered, bodies_inside, triggered_sensors}`
  (`BTreeMap` determinístico) — a API que o sinal de gameplay vai ler.
- **Autoria:** toggle **"Solid | Sensor"** no Inspector §11, um `seg_row` como o Kind/Layer
  (populate/event/seam de graça; `PhysicsFieldEdit::Sensor(bool)` + `INSP_PHYS_SENSOR`).

**Decisões que decidem tudo:**
- ⚠️ **A detecção só corre no PLAY** — o `hold` (física desarmada, W4b) **limpa** o estado: sem
  sim não há overlap, e um trigger aceso sem nada dentro seria mentira.
- ⚠️ **Um sensor respeita as CAMADAS** (W2c) — só detecta o que está configurado pra colidir.
  Sai de graça: o `groups_for` já filtra o collider do sensor.
- `world/sensors.rs::intersecting_body_pairs` é **ordenado + dedup** (o hash C9 **não** inclui
  o trigger — só poses — mas o readout tem de ser reproduzível frame a frame).
- `bridge/triggers.rs::rebuild_triggers` mapeia handle→entity **uma vez** por frame, e **só
  quando há overlap** (a query vem vazia sem sensores ⇒ retorna antes de alocar).

**Persistência:** `Collider.is_sensor` APENDADO ⇒ `PROJECT_SCHEMA` **26 → 27** (+ a tripla-pin
`(27, 8, 13)` do `project_tests`). Mesmo padrão do v21 (`layer`).

**LOC (splits):** `world.rs` → `world/desc.rs` (BodyDesc/BodySnapshot, saíram pra abrir espaço)
+ `world/sensors.rs` (o `intersecting_body_pairs`) · `bridge.rs` → `bridge/triggers.rs` (o
estado de trigger) + `body_desc` **mudou-se pra `scale.rs`** (ao lado do `scaled_shape` que usa).

**Gates (10 novos, 3 mutações provadas):**
- `ph2d-physics-ecs/tests/sensors.rs` (4 behavioral): detecta+atravessa · sólido bloqueia+nunca
  dispara · sem-sensores-sem-triggers · hold-limpa.
- `ph2d-physics/tests/sensors.rs` (2): `intersecting_body_pairs` reporta um overlap de sensor ·
  um par sólido **não** reporta (contato, não interseção).
- `physics_overlay` (2): magenta idle vs active · um sensor não usa a cor do kind.
- `seam_physics` (o sweep clica o toggle Solid/Sensor, cada lado seu bool) · `persistence`
  (is_sensor sobrevive ao snapshot round-trip).
- **Mutações:** spawn ignora `.sensor()` → mata detecção · `rebuild_triggers` no-op → mata o
  trigger state · overlay idle==active → mata a cor.

**Smoke: `PH2D_PHYSICS_SMOKE=10`** (`physics_smoke.rs::physics_smoke_sensor`) — 2 pistas: bola
bloqueada por uma plataforma sólida × bola atravessa um sensor idêntico que **acende** (tecla
`B`). Um sensor morto bloquearia a bola como o sólido, ou nunca acenderia.

**Aberto (a próxima camada, decisão do Enio):** o **sinal de gameplay** — colisão→ação (um
`Marker` da timeline / um callback do `ph2d-script`), cross-line, precisa do desenho do
consumidor. O primitivo (detecção + estado consultável + viz) é o pré-requisito, e está pronto.

---

## §Weld — o 5º joint (`FixedJoint`, 2026-07-19, smoke `=11`)

Polimento **C** do cardápio (o W3 deixou o Weld fora — *"~4 linhas e um chip,
deliberadamente FORA: nada no smoke o exercita, e um 4º chip que a wave não fuma
é chip shipado às cegas"*). Agora ele é fumado (`=11`).

**O quê:** trava dois corpos **rigidamente** no ponto de âncora — sem translação
NEM rotação relativa (rapier `FixedJoint`). Um Pin com a rotação congelada.

**Decisões:**
- Variant `Weld` apendado nos DOIS `JointKind` (ecs component + physics plain-data),
  discriminante **3** (append-only; o gate `the_kind_discriminants_are_pinned_in_order`
  o pina). `PROJECT_SCHEMA` **27 → 28** (tripla-pin `(28, 8, 13)`) — apender variant
  não move os índices anteriores, o bump é pro caminho INVERSO (um save com Weld
  aberto por um binário v27 morre como erro de VERSÃO, não como postcard perdido no
  discriminante 3; mesmo raciocínio do v24 do vetor).
- ⚠️ **A âncora deixou de key-ar em `is_hinge()`.** Um Weld **compartilha um ponto**
  como o Pin, mas **não é hinge** — então `has_length()` não podia mais ser
  `!is_hinge()` (senão o Weld ganhava uma length que não tem), e o anchor policy
  não podia mais ler `is_hinge()` (senão o Weld anchorava no centro de B como um
  Spring). Nasceu **`JointKind::shares_a_point()`** (Pin | Weld) — a pergunta que o
  anchor policy de fato tem.
- **Sem params:** o §12 recusa toda linha de param (o `else` de Rope virou
  `else if KIND_ROPE`, senão o Weld herdava "Max Length"). Chip "Weld" no seletor
  (`INSP_JOINT_KIND` [3]→[4] + label; populate/event/seam de graça).

**Gates:** `ph2d-physics-ecs/tests/weld.rs::a_weld_holds_the_body_rigid_where_a_pin_would_swing`
(behavioral, com o Pin como CONTROLE — prova que a cena não está congelada) ·
discriminante pinado + `has_length`/`shares_a_point` (unit em `joint.rs`) ·
`seam_joint::each_kind_paints_only_the_rows_it_uses` estendido a `0..4` (o Weld
pinta 0 linhas de param) + `the_kind_chips_each_pick_their_own_kind` (auto). **Mutação:**
Weld→Revolute em `spawn_joint` deixa a prancha balançar (o gate sangra).

**Smoke `=11`** (`physics_smoke_rigs.rs::physics_smoke_weld`): duas barras, cada uma
junta a um hook estático pela ponta esquerda — a **soldada** fica horizontal (rígida),
a **pinada** balança como pêndulo, lado a lado.

**Aberto (fora de propósito, C do cardápio):** motor em mola/corda · re-escolher os
corpos de um joint (picker de entidade) · break force no Weld (um Weld que quebra sob
carga — nada pede ainda).

---

## §BakeChannels — assar um subconjunto dos canais (2026-07-19)

Polimento **D** do cardápio ("escolher os canais do bake — hoje escreve X/Y/Rotation
sempre que se movem; não há 'só a rotação'").

**O quê:** um seletor **"Bake: All | Position | Rotation"** no §11 (acima do botão
Bake, só p/ corpo Dynamic). Assa um SUBCONJUNTO dos canais de pose — o caso de
**layering**: manter a posição animada à mão e assar só o giro da física, ou o inverso.

⚠️ **Mais estreito que "canal que não se moveu é pulado"** (que o bake já faz — protege
um canal que a sim não tocou): isto **descarta** um canal que a sim MOVEU, porque o
artista quer possuí-lo. Um canal não assado, num corpo `Kinematic`, segue o que a cena
diz (a animação do artista, ou o `Transform` estático).

**Desenho:** `BakeChannels` (enum em `physics_bake.rs`: All/Position/Rotation +
`channels()`/`tag`/`from_tag`) é **transiente** — não salvo, porque é como o botão se
comporta, não o documento (classe do AutoKey/Record). Vive em `App.bake_channels`; a
edição `PhysicsFieldEdit::BakeChannels(tag)` é **global** (roteada no render loop como o
Bake/Join, não um edit de Collider); `bake_selection` itera `channels.channels()` em vez
de `PoseChannel::ALL`. Seletor = `seg_row` (reusa o padrão do Kind/Layer).

**Gates:** `physics_bake_tests::baking_a_channel_subset_writes_only_those_tracks` (All
assa os 3 — o piso TOMBADO move todos, é o controle; Rotation-only assa só rotação, nem X
nem Y; Position-only o inverso) + `seam_physics` (o sweep clica os 3 chips). **Mutação:**
iterar `PoseChannel::ALL` escreve X/Y sob um bake de rotação-só (sangra).

⚠️ **`inspector_model.rs` está a 699/700 LOC.** Esta wave o tocou (o campo
`bake_channels_tag` + a variant `BakeChannels`), e as docs dos meus campos foram
comprimidas p/ caber. **A próxima adição de tipo de inspector obriga o split por domínio**
(extrair `InspectorPhysicsInfo`/`PhysicsFieldEdit`/`InspectorJointInfo`/`JointFieldEdit`
p/ um irmão `inspector_model_physics.rs`, re-exportado) — não feito aqui porque é um
arquivo foundational compartilhado e o split estrutural arrisca colisão de merge com
outras linhas que tocam os tipos de inspector.

**Sem smoke dedicado** (é UI + comportamento gateado; o `=7` do bake exercita o caminho,
e o seletor é visível no §11 de qualquer corpo Dynamic). Sem bump de schema (transiente).

## §W8 — GRAVITY SCALE por corpo (2026-07-19, smoke `=12`)

**Multiplicador de gravidade por corpo.** `GravityScale(f32)`, default 1.0. `0.0` sem peso · `<0`
flutua · `>1` pesado. Row "Gravity Scale" no §11, **Dynamic-only** (rapier só aplica gravidade a
Dynamic). Preenche um vão real: até aqui todo corpo só podia começar sob gravidade cheia.

**Componente opcional, NÃO campo do `RigidBody`.** O plano (nota do W1 em `components.rs`) previa
apendar ao `RigidBody`. Medido: `RigidBody` é literal `{ kind }` em ~80 sítios de fixtures ⇒ apendar
campo obrigatório é churn grande e recorrente (damping/ccd/can-sleep viriam depois). Escolhido o
idioma de presence-override do resto do Inspector (`ZIndexOverride`/`BlendMode`): ausente = default,
presente = override. **Zero churn nos 80 sítios · zero bump de schema** (blob-key próprio, precedente
`PhysicsJoint`) · `RigidBody` intocado (o `Kind` do apply preserva a gravidade de graça). Nota do
plano corrigida.

**`BodyDesc.gravity_scale`** (recipe de spawn) É necessário — o `rewind_to` reconstrói do descriptor,
então sem isso o scrub perderia a gravidade. ~19 fixtures em `ph2d-physics/tests` ganharam o campo
neutro. O `body_desc(rb, col, t, gravity_scale)` recebe o valor; a ponte (`reconcile_structure`) lê
`world.get::<GravityScale>(e).map_or(NEUTRAL, |g| g.0)`.

**Detach no 1.0:** o apply desanexa o componente no valor neutro (arquivo fica sem no-op `1.0`s), como
os campos de ordering. `build_physics_info` lê `map_or(NEUTRAL)`.

**inspector_model SPLITADO** (resolve o 699/700 do §BakeChannels): §11+§12 saíram p/
`inspector_model_physics.rs` (irmão re-exportado, paths intactos). A churn de física agora mora num
arquivo desta linha.

**Dívida de LOC consertada de carona:** `physics_overlay.rs` (776, latente-vermelho no `file_loc_caps`
desde o W7) → `mod tests` movido p/ `#[path]` irmão → 258/519. `physics_smoke.rs` (603 com a cena 12)
→ cena 12 p/ `physics_smoke_rigs.rs`.

**Gates:** `gravity_scale_multiplies_the_bodys_fall` (trajetória 0/1/2/-1, mut RED) ·
`a_gravity_scale_component_is_folded_into_the_sim` (ponte lê o componente, mut RED) ·
`gravity_scale_is_offered_and_committed_only_for_a_dynamic_body` (seam offer+honour, mut RED) ·
persistência round-trip estendida · `registers_every_physics_component` 3→4 · c9 53 corpos.

**Ids/consts alocados:** `INSP_PHYS_GRAVITY_SCALE` · componente `ph2d::physics::GravityScale` ·
`GravityScale::NEUTRAL = 1.0`. Smoke `=12`.

## §Capsule — o collider de personagem (2026-07-19, smoke `=13`)

**Terceira forma: `ColliderShape::Capsule { half_height, radius }`** (Y-alinhada). Existia só Ball e
Cuboid; o próprio `components.rs` marcava o vão. Uma caixa engancha em emenda de tile e quina de
rampa, uma cápsula desliza — é por isso que personagem 2D é cápsula (Unity/Godot shipam a mesma).

**Y-alinhada de propósito:** é o default do `CapsuleCollider2D`/`CapsuleShape2D`, e cápsula deitada é
cápsula em corpo rotacionado (o `Transform` já roda o collider). Um flag de eixo seria 2ª maneira de
dizer a mesma coisa.

**Duas formas, pelo precedente do W6:** escala uniforme → `ShapeDesc::Capsule` EXATA (rapier tem
nativa); não-uniforme → `ShapeDesc::Stadium` (polígono convexo via `capsule_vertices`), porque as
tampas viram elípticas e nenhum solver representa isso — a MESMA regra Ball→Ellipse, pelo mesmo
motivo (o collider tem de casar com o sprite desenhado).

**`capsule_vertices(half_height, rx, ry)`** é porta ÚNICA: o build do collider E o overlay traçam
dela, então o wireframe não pode descrever borda que o solver não colide. Com `rx == ry` ela é o
contorno da cápsula exata — é assim que o overlay desenha a `Capsule`. `libm::sincosf` (HR-5).

**Sem bump de schema:** variante APENDADA em `ColliderShape` mantém os discriminantes de
Ball/Cuboid (precedente do `BodyKind::Kinematic`). `PROJECT_SCHEMA` fica **28**.

⚠️ **`Radius` não pode FORÇAR bola:** o raio da tampa é a mesma grandeza com o mesmo nome, então num
capsule o edit muda as tampas e mantém a cápsula. Forçar `Ball` ali apagaria a cápsula do artista no
primeiro toque no raio. E `Shape(_)` desconhecido agora é DESCARTADO (com 2 formas o catch-all era
redundante; com 3 seria um chip que seleciona outra coisa) — a disciplina que o `Kind` já segue.

⚠️ **Id próprio p/ o `half_height` da cápsula** (`INSP_PHYS_CAP_HALF_H`): "half height" é o
half-extent numa caixa e o SEGMENTO reto numa cápsula (as tampas somam `radius` por cima). Um
controle com dois significados é o bug que esta seção não para de gatear. O `Radius` É reusado —
essa sim é a mesma grandeza.

**Gates:** extents da cápsula e do stadium NO SIM (lidos do collider vivo) · **`a_capsule_climbs_a_step_that_stops_a_box`**
— o gate que justifica a wave; mutação (cápsula vira caixa de mesmos extents) trava em **x = −0,13**,
antes do degrau · `scaled_shape` uniforme/não-uniforme · seam: `each_shape_paints_only_its_own_dimension_rows`
(presença E ausência por forma) + o sweep dos 3 chips + o commit do row novo · c9 **54 corpos** com
um Stadium (põe a tessellation da cápsula no hash 3-OS).

**LOC:** `paint_physics_section` estava no teto de 200 ⇒ as rows de dimensão viraram
`paint_shape_dims` (uma pergunta, uma resposta). E as cenas 12+13 saíram p/ o arquivo novo
**`physics_smoke_props.rs`** ("uma propriedade de UM corpo"), o que devolve ao `physics_smoke_rigs.rs`
a costura limpa dele (corpos RELACIONADOS) em vez de virar despejo de overflow.

## §W9 — VELOCIDADE INICIAL por corpo (2026-07-19, smoke `=14`)

**Velocidade inicial autorada (linear + angular), aplicada no SPAWN.** Até aqui todo corpo começava
parado — o vão que fez as cenas de smoke inclinarem a gravidade pra fingir um empurrão (2× nesta
jornada). Um projétil, uma bola chutada em t=0, uma roda girando. `linvel` (m/s, eixos de MUNDO) +
`angvel` (rad/s).

**Componente opcional `InitialVelocity`, o padrão do W8** (`GravityScale`): ausente = repouso,
presence-override, **zero churn no `RigidBody`, sem bump de schema** (blob-key próprio; registro
4→5). **`BodyDesc.linvel/angvel` (recipe de spawn) É preciso:** aplicada no BUILD e depois é do
solver (nunca re-aplicada por tick — isso re-lançaria), mas rida no `BodyDesc` pra o **rewind
re-armar o lançamento** (o gate prova: scrub a t=0 + replay reproduz a trajetória). ~21 fixtures de
`ph2d-physics/tests` ganharam os 2 campos neutros.

**Eixos de MUNDO, não do pai:** o corpo nasce na pose de mundo (a ponte compõe a cadeia), então o
lançamento é um vetor de mundo — a convenção do `Rigidbody2D.linearVelocity`/`linear_velocity`. Detach
no repouso.

**Angular em deg/s na UI, rad/s no componente** (como `rotation_rad`): a conversão mora na fronteira
do painel (sync deg←rad, event deg→rad). Gate `initial_velocity_is_offered_..._dynamic_body` prova a
conversão (90 deg/s → `Angvel(π/2)`) — esquecê-la giraria o corpo 57× rápido demais.

**A SETA de velocidade no overlay** (amarela): um lançamento armado não se vê num corpo parado. Só
enquanto `last_stepped() == 0` (bodies na pose autorada) — depois que a sim anda, a velocidade viva
não é mais o lançamento autorado, então some. Construída em ESPAÇO DE TELA (a regra do módulo): haste
= vetor projetado (escala com zoom/velocidade), cabeça = ornamento de tamanho fixo. Só linear (spin
não tem direção pra apontar).

**Rows Dynamic-only** (Init Vel X/Y + Init Spin): velocidade é inerte em Static (não move sob força) e
Kinematic (dirigido por curvas) — a mesma regra da gravidade. `paint_dynamics_rows` extraído (a fn no
teto de 200). Cena 14 nasceu em `physics_smoke_props.rs`, **pausada** (pra ver as setas antes do Play).

**Gates:** `initial_velocity_launches_and_spins` (trajetória, mut RED) ·
`the_bridge_launches_..._and_a_rewind_re_arms_it` (fold + re-arm no scrub, mut RED) ·
`initial_velocity_is_offered_and_committed_only_for_a_dynamic_body` (seam presença/ausência +
conversão) · persistência round-trip estendida · registro 4→5 · c9 **55 corpos** com um lançado.

**Ids/consts:** `INSP_PHYS_LINVEL_X/Y`, `INSP_PHYS_ANGVEL` · `ph2d::physics::InitialVelocity` ·
`InitialVelocity::REST`. `PROJECT_SCHEMA` fica **28**. Smoke `=14`.

---

## §W-CCD — DETECÇÃO CONTÍNUA por corpo (2026-07-20, smoke `=15`)

**O sequel direto do W9.** A detecção *discreta* (o default do rapier) só testa um corpo na pose de
FIM de cada (sub-)passo, então um corpo pequeno e rápido passa **limpo através** de geometria fina
entre dois passos — a bala que atravessa a parede. É a colisão que o jogo **PERDE**, distinta da
sobreposição de POUSO profundo que o sub-stepping (`DEFAULT_SUBSTEPS=4`) ataca (o corpo já está dentro
do chão no frame em que toca — `v×dt`, que nenhum solver desfaz). **CCD varre o movimento e para no
1º impacto.** Toggle **Discrete / Continuous** no §11 (o vocabulário do Unity).

**Marcador `Ccd` — a PRESENÇA é o booleano** (idioma do `ph2d_ecs::Locked`), não o `f32`-valued do
W8/W9: um booleano não tem valor pra carregar. Registro **5→6**, blob-key próprio, **sem bump de
schema** (fica **28**), zero churn no `RigidBody`. Attach no Continuous, detach no Discrete (arquivo
nunca carrega off-flag). **`BodyDesc.ccd` (recipe de spawn) É preciso:** `.ccd_enabled()` no build, e
rida no `BodyDesc` pra o **rewind re-armar** — o gate prova (scrub a t=0 + replay, o corpo marcado
segue parado). ~22 fixtures de `ph2d-physics/tests` ganharam o campo neutro `ccd: false`.

**Dynamic-only** (só a família que o solver move rápido tunela; Static não move, Kinematic é dirigido
por pose): row `seg_row("Collision", …)` no bloco Dynamic, ao lado de Gravity Scale / Init Vel — a
mesma regra da gravidade. Reusa o path Sensor/Bake-channel (seg de 2 opções).

**O tunelamento é ALINHAMENTO-sensível (medido, não presumido):** a 80 m/s os 4 sub-passos amostram
uma pose EXATA dentro da parede e o corpo discreto **não** tunela; 100..=600 m/s tunelam com margem
larga (varredura no gate). O gate usa **200 m/s** (discreto termina em x≈99, contínuo em x≈−0,07); o
smoke usa **160 m/s** (geometria de bola maior/visível, verificada por varredura). ⚠️ Não escolha um
número "bonito" sem varrer — a ressonância dos sub-passos morde.

**Determinismo:** `enhanced-determinism` está ON, então o solver de CCD (conservative advancement /
time-of-impact) entra na garantia cross-OS. O c9 ganhou **uma bola CCD rápida contra uma parede fina**
(o solver de CCD DE FATO roda seu sweep e o `f32` dele entra no hash) → **57 corpos** (55 + parede +
bola), hash re-derivado (não pinado — o CI compara os 3 OSes).

**Gates:** `a_discrete_body_tunnels_..._and_a_continuous_one_does_not` (wrapper, comportamental; mut
`.ccd_enabled(desc.ccd)`→`false` faz a bola CCD tunelar a x=99, RED) · `the_bridge_makes_a_marked_body_
continuous_and_a_rewind_re_arms_it` (ecs; marcado para, controle sem marcador tunela, scrub re-arma;
mut ignora marcador → RED) · `ccd_is_offered_and_committed_only_for_a_dynamic_body` (seam
presença/ausência) · CCD no sweep de `every_segmented_option_reaches_the_bus` · persistência round-trip
estendida (marcador sobrevive) · registro 5→6 · c9 57 corpos.

**Ids/consts:** `INSP_LIVE_PHYSICS_CCD` (group) + `INSP_PHYS_CCD: [NodeId;2]` · `ph2d::physics::Ccd`.
`PROJECT_SCHEMA` fica **28**. Smoke `=15` (pausado — B mostra as 2 setas idênticas; Play: a bola verde
Continuous para na parede, a laranja Discrete tunela e some).

**Aberto (deliberado):** CCD em corpo **Kinematic** (o rapier suporta, mas o caso reportado/canônico é
o projétil Dynamic; nada no smoke exercita kinematic — cerca de Chesterton, como o Weld ficou fora do
W3).

---

## §W-LockRot — FREEZE ROTATION por corpo (2026-07-20, smoke `=16`)

**O bloco que todo personagem 2D precisa.** Uma caixa livre numa rampa **tomba** ao deslizar, e um
personagem cai. Travar o DOF angular a mantém em pé — ela ainda translada e colide, só nunca gira.
É o `lock_rotation` do Godot / "Freeze Rotation" do Unity. Toggle **Rotation: Free / Locked** no §11.

**Marcador `LockRotation` — a PRESENÇA é o booleano** (idioma do `Ccd`/`Locked`): um booleano não tem
valor a carregar. Registro **6→7**, blob-key, **sem bump** (fica **28**). Attach no Locked, detach no
Free. `BodyDesc.lock_rotation` → `.locked_axes(LockedAxes::ROTATION_LOCKED)` no build, rida no recipe
pra o **rewind re-armar** (gate prova: scrub a t=0 + replay, o corpo travado segue em pé). Um corpo
travado **ignora `angvel`** (não há DOF pra girar) — foi assim que o gate ficou afiado (spawn com spin,
livre gira, travado fica em 0). ~22 fixtures de `ph2d-physics/tests` ganharam `lock_rotation: false`.

**Dynamic-only** (só a família que o solver gira sob forças tem rotação a travar): row `seg_row("Rotation",
["Free","Locked"])` no bloco Dynamic, ao lado de CCD. Reusa o path Sensor/CCD (seg de 2 opções).

**Gates:** `a_free_body_spins_and_a_rotation_locked_one_does_not` (wrapper; spin 5 rad/s → livre 2,5 rad,
travado 0; mut `.locked_axes(...)`→`empty()` faz o travado girar, RED) · `the_bridge_pins_a_marked_body_
and_a_rewind_re_arms_it` (ecs; marcado não gira, controle gira, scrub re-arma; mut ignora marcador → RED)
· `lock_rotation_is_offered_and_committed_only_for_a_dynamic_body` (seam presença/ausência) · no sweep de
`every_segmented_option_reaches_the_bus` · persistência round-trip · registro 6→7 · c9 **58 corpos** (uma
caixa travada girando).

**LOC:** o wrapper e a ponte bateram o teto de 700 (707 cada, +7 desta wave) → split em irmão: `world.rs`
636 (tests → `world/tests.rs`), `bridge.rs` 675 (`deterministic_hash`/`scratch_capacity` → `bridge/
diagnostics.rs`). `body_desc` chegou a 8 args → `#[allow(clippy::too_many_arguments)]` (cada flag é um
componente opcional independente; empacotar só moveria os mesmos campos atrás de um nome).

**Ids/consts:** `INSP_LIVE_PHYSICS_LOCKROT` (group) + `INSP_PHYS_LOCKROT: [NodeId;2]` ·
`ph2d::physics::LockRotation`. `PROJECT_SCHEMA` fica **28**. Smoke `=16` (pausado; Play: a caixa laranja
Free tomba descendo a rampa esquerda, a verde Locked desce em pé pela direita — rampas espelhadas, os
corpos divergem).

**~~Aberto: Freeze Position X/Y~~ → FEITO** (§W-LockPos, smoke `=18`): o resto do `RigidbodyConstraints2D`
(Freeze Position X + Y ao lado do Freeze Rotation — o trio de constraints do Unity/Godot), mesmo
maquinário de `LockedAxes`. Ver a entrada §W-LockPos abaixo.

---

## §W-Offset — OFFSET do collider por corpo (2026-07-20, smoke `=17`)

**O collider quase nunca fica centrado no sprite.** A hitbox dos pés fica abaixo, a de um projétil na
ponta. Até aqui o collider nascia colado ao centro do sprite. Agora **Offset X/Y** no §11 (o
`Collider2D.offset` do Unity) o desloca.

**Campo no `Collider`, NÃO componente opcional** — offset é propriedade intrínseca do collider (como
restituição/atrito/layer/sensor foram apendados). ⚠️ **Isso BUMPA o schema (28→29)**: postcard é
posicional, apendar campo ao `Collider` muda o layout. Tripla-pin `(29, 8, 13)`. É o oposto dos 4
marcadores anteriores (componente próprio → sem bump); a escolha é semântica, não de conveniência.

**`BodyDesc.offset` → `ColliderBuilder::translation`** (posição do collider relativa ao corpo; rapier
rotaciona COM o corpo — a foot-box de um personagem girado gira junto). Rida no recipe pro rewind
preservar. ⚠️ **Escala SINCADA (signed), não `abs`:** diferente das half-extents (tamanho, sempre +), o
offset é uma POSIÇÃO, então um flip (`scale.x<0`) o **espelha** pro outro lado (`scale.rs`:
`offset[0]*t.scale.x`). ~25 fixtures de `ph2d-physics/tests` ganharam `offset: [0.0, 0.0]`.

**O overlay É a única forma de VER o offset** (a lição da seta de velocidade). O contorno é desenhado no
offset (escala sincada + rotação do corpo, os MESMOS que a ponte manda pro solver — `physics_overlay.rs`
`outlines`), senão o wireframe descreveria o collider onde ele NÃO está. **Não é Dynamic-only** (qualquer
collider — static/kinematic — pode ter offset): rows `num_row` "Offset X/Y (m)" com as dims de forma.

**Gates:** `a_collider_offset_moves_where_the_body_rests` (wrapper; collider +2m acima → corpo repousa 2m
abaixo; mut `.translation`→zero, RED) · `the_bridge_folds_the_collider_offset_and_a_rewind_preserves_it`
(ecs; mut scale ignora offset, RED) · `scale::tests` **the_collider_offset_scales_with_the_body** +
**a_flip_mirrors_the_collider_offset** (unit direto no `body_desc` — o mirror é a parte fácil de errar) ·
`an_offset_collider_outline_sits_where_the_collider_is` (overlay: desloca +100px E rotaciona 90°; mut
tira o offset, RED — bbox-center, não point-mean, por causa do raio do spoke) · Offset X/Y no sweep de
`every_dimension_field_reaches_the_bus` · persistência round-trip · c9 **59 corpos** (um offset).

**Ids/consts:** `INSP_PHYS_OFFSET_X/Y` · `Collider.offset` · `BodyDesc.offset`. `PROJECT_SCHEMA` **29**.
Smoke `=17` (pausado; Play: o personagem verde com collider nos PÉS fica de pé no chão, o laranja com
collider CENTRADO afunda até a cintura — o collider centrado é o que repousa no chão).

**Aberto (deliberado):** **rotação do collider** relativa ao corpo (o `Collider2D` não tem, mas rapier
aceita) e **múltiplos colliders por corpo** (composite — feature maior). Nada pede ainda.

---

## §W-LockPos — FREEZE POSITION X/Y por corpo (2026-07-20, smoke `=18`)

**O resto do trio de constraints.** Ao lado do Freeze Rotation (§W-LockRot), o Unity/Godot shipam **Freeze
Position X** e **Freeze Position Y**. Travar X prende o corpo a um trilho (elevador, ator numa lane);
travar Y o faz flutuar (plataforma que gravidade não puxa). Dois toggles **Freeze X / Freeze Y** no §11,
ao lado do Rotation.

**Dois marcadores `LockPositionX`/`LockPositionY` — a PRESENÇA é o booleano** (idioma do `LockRotation`/
`Ccd`/`Locked`): dois DOFs independentes = dois marcadores. Registro **7→9**, blob-key, **sem bump** (fica
**29** — o oposto do W-Offset, que apendou campo ao `Collider`; marcador registrado é aditivo). Attach no
Locked, detach no Free. `BodyDesc.lock_x/lock_y` ORam no MESMO `LockedAxes` do `lock_rotation`
(`TRANSLATION_LOCKED_X`/`_Y`), rida no recipe pro **rewind re-armar cada eixo**.

⚠️ **rapier NÃO zera a velocidade inicial de um eixo de translação travado** (o gate red-first pegou): o
`LockedAxes` zera a inversa-massa do eixo (força/gravidade não acelera — por isso o Y-locked não cai), mas
`RigidBodyVelocity::integrate` avança o corpo pela `linvel` CRUA sem projetar os eixos travados. Medido:
um corpo X-locked lançado a 3 m/s deslizou os 1,5 m inteiros. rapier só trata rotação assim. Então
`spawn_body` **zera a componente travada da velocidade** (`if desc.lock_x { 0.0 }`) — um eixo congelado
não carrega velocidade, que é o que "Freeze Position" significa no Unity/Godot, e torna o lock autoritário.
~27 fixtures de `ph2d-physics/tests` ganharam `lock_x: false, lock_y: false`.

**Dynamic-only** (só a família que o solver MOVE tem posição a travar): dois `seg_row("Freeze X"/"Freeze Y",
["Free","Locked"])` no bloco Dynamic, entre Rotation e Bake. Reusa o path Sensor/CCD/LockRot.

**Gates (todos red-first + mutação):** `an_x_locked_body_ignores_a_sideways_force` (o BIT de X: gravidade
lateral, mut tira `TRANSLATION_LOCKED_X` → RED) · `a_y_locked_body_ignores_gravity` (o BIT de Y) ·
`a_frozen_axis_drops_an_authored_launch_and_the_axes_are_independent` (o DROP de velocidade — mut tira o
`if desc.lock_x{0.0}` → RED — E a independência via FORÇA: mut ORa `TRANSLATION_LOCKED` inteiro → RED; a
independência TEM de empurrar por FORÇA, não velocidade, senão a velocidade crua atravessa o bit largo).
⚠️ Duas camadas por eixo (bit + drop) = um gate por camada ([[feedback_layered_defenses_need_per_layer_gates]]).
ECS: `the_bridge_pins_a_marked_body_on_x_and_a_rewind_re_arms_it` + `the_two_position_locks_are_independent`
(mut ignora marcador → RED). Seam: `freeze_position_is_offered_and_committed_only_for_a_dynamic_body`
(presença/ausência, X e Y varridos independentes) + no sweep de `every_segmented_option_reaches_the_bus`.
Persistência round-trip (ambos marcadores) · registro **7→9** · c9 **60 corpos** (um X-locked lançado).

**Ids/consts:** `INSP_LIVE_PHYSICS_LOCKX`/`LOCKY` (groups) + `INSP_PHYS_LOCKX`/`LOCKY: [NodeId;2]` ·
`ph2d::physics::LockPositionX`/`LockPositionY` · `BodyDesc.lock_x/lock_y`. `populate.rs` registra os 2
groups (senão os toggles pintam mas ficam mortos sob o mouse — o `architecture_panel_wiring_parity` pega).
`PROJECT_SCHEMA` fica **29**. Smoke `=18` (pausado; Play: 3 bolas lançadas de lado — a verde Free ARCA
pra baixo-direita, a ciano Freeze-X cai RETO (o lançamento é dropado), a laranja Freeze-Y PLANA de lado à
altura constante porque gravidade não a puxa).

**Aberto (deliberado):** freeze de **rotação + posição combinados** já funciona (os 3 marcadores ORam no
mesmo bitmask); nada mais pede. `lock_z`/2.5D fora de escopo (motor 2D).

---

## §W-Mass — MASSA MANUAL por corpo (2026-07-20, smoke `=19`)

**O artista pensa em MASSA, não em densidade.** Até aqui a massa era `densidade × área` — quem quer "esta
caixa pesa 10 kg" tinha de calcular a densidade. Agora um toggle **Mass: Auto | Manual** no §11 (o
`useAutoMass` do Unity): Auto mostra a row **Density**, Manual mostra **Mass (kg)**.

⚠️ **Densidade e massa são a MESMA grandeza por dois caminhos** (`massa = densidade × área`), então **só uma
row é viva por vez** — mostrar as duas seria o bug "duas portas pra uma grandeza". O toggle escolhe qual.
Static/Kinematic (massa infinita, rapier ignora as duas) mantêm a row Density simples, sem toggle (Dynamic-only).

**Componente VALUADO opcional `MassOverride(f32)`** (idioma do `GravityScale`, não marcador — carrega um
valor): ausente = Auto, presente = Manual. Registro **9→10**, blob-key, **sem bump** (fica **29**). `BodyDesc.
mass_override: Option<f32>` → `world.rs` ramifica `ColliderBuilder::mass(m)` (kg, ignora densidade; inércia
angular ainda derivada da forma) vs `.density(d)` (auto, byte-idêntico ao de antes). Rida no recipe pro
rewind re-armar. **Auto→Manual semeia com a massa auto** (`shape_mass` = densidade × área da forma autorada,
`inspector_physics.rs`) pra não SALTAR ao trocar — exato pra corpo sem escala (o comum); um corpo escalado
tem a massa real também escalada, mas é só o valor inicial que o artista ajusta (ler a massa exata seria
re-derivar a conta do rapier num 2º lugar). ~27 fixtures de `ph2d-physics/tests` ganharam `mass_override: None`.

**Gates (red-first + mutação):** wrapper `a_manual_mass_overrides_the_density_derived_mass` (lê `RigidBody::
mass()` cru: auto ≈ 0,785 = π·0,25, manual == 10,0; mut world sempre `.density` → RED) +
`a_heavier_body_dominates_a_head_on_collision` (massa muda COMPORTAMENTO, não só um readout — colisão de
momento, o pesado atravessa o leve; gravidade esconderia, todos caem a `g`). ECS `the_bridge_folds_a_mass_
override_and_a_rewind_preserves_it` (bola pesada `MassOverride(20)` ara a fila de leves e passa de x=1,5; mut
bridge ignora o componente → vira leve, para em ~1,25 = RED; + controle auto-massa + rewind). Seam
`mass_source_toggle_swaps_density_for_mass_and_is_dynamic_only` (Auto pinta Density e NÃO Mass; Manual pinta
Mass e NÃO Density — presença E ausência; Static = Density sem toggle) + no sweep de `every_segmented_option`.
Persistência round-trip (`MassOverride(7.5)`) · registro **9→10** · c9 **61 corpos** (uma bola pesada).

**LOC:** `physics.rs` (painel) bateu **609 > 600** (o cap do painel, que o `architecture_workspace_file_loc_cap`
NÃO cobre — `ph2d-panel-*` é do `architecture_panel_loc_cap`) → split: a helper das rows de massa foi pro
irmão **`sections/physics_rows.rs`** (recebe `is_dynamic`/`mass_manual` resolvidos, então não compartilha
const privada). `physics.rs` 543.

**Ids/consts:** `INSP_LIVE_PHYSICS_MASSMODE` (group) + `INSP_PHYS_MASSMODE: [NodeId;2]` (Auto/Manual) +
`INSP_PHYS_MASS` (NumberInput) · `ph2d::physics::MassOverride` · `BodyDesc.mass_override`. `populate.rs`
registra o group + a range de MASS (min 0.001 — massa deve ser positiva). `PROJECT_SCHEMA` fica **29**. Smoke
`=19` (2 lanes: bola CYAN pesada — Manual 30 kg — ara a fila de 5 pinos e segue; a ORANGE do mesmo tamanho
mas auto-massa PARA no 1º pino; flipe a pesada pra Auto e ela para igual).

**Aberto (deliberado):** damping/drag por-corpo (precisa de modo de combinação com o global — `damp_mode` do
Godot).

---

## §W-Dominance — DOMINANCE / prioridade de colisão por corpo (2026-07-20, smoke `=20`)

**O contraponto da massa.** Dominance é uma PRIORIDADE de colisão (rapier `dominance_group`, Box2D): o corpo
de dominância ESTRITAMENTE maior é tratado como massa infinita pelo menor — **atropela e nunca é empurrado
de volta**, enquanto ainda cai sob gravidade e colide normalmente com pares iguais/maiores. ⚠️ **É ortogonal
à massa:** um corpo LEVE com dominância alta empurra um PESADO neutro — o que a massa sozinha NÃO faz. Expressa
um meio-termo que nenhum KIND consegue: diferente de Static/Kinematic (que também empurram tudo) um Dynamic
de dominância alta ainda cai e reage aos pares. Static/Kinematic ficam no máximo (`i8::MAX+1`), então um
dinâmico nunca os empurra — consistente com massa infinita. Row **Dominance** no §11, Dynamic-only.

**Componente VALUADO opcional `Dominance(i8)`** (idioma do `GravityScale`): ausente = neutro `0`, presente =
prioridade. Registro **10→11**, blob-key, **sem bump** (fica **29**). `BodyDesc.dominance: i8` →
`RigidBodyBuilder::dominance_group`, rida no recipe pro rewind. Detach em 0 (arquivo livre de no-op). O painel
converte o float do widget → i8 (round+clamp) na fronteira do event. ~30 fixtures de `ph2d-physics/tests`
ganharam `dominance: 0`.

**Gates (red-first + mutação):** wrapper `a_light_high_dominance_body_plows_through_a_heavy_one` (o teste que
a MASSA não reproduz: mover leve Dominância 5 atravessa alvo pesado 20 kg; neutro quica; mut tira
`.dominance_group` → RED) + `the_dominance_group_reaches_the_body` (readback direto `RigidBody::
dominance_group()`). ECS `the_bridge_folds_dominance_and_a_rewind_preserves_it` (mover leve `Dominance(5)` ara
o pesado e passa de x=1,5; mut bridge ignora → quica pra x=0,02 = RED; + controle neutro + rewind). Seam
`dominance_is_offered_and_committed_only_for_a_dynamic_body` (presença/ausência + conversão 5.0→`Dominance(5)`).
Persistência round-trip · registro **10→11** · c9 **63 corpos** (mover leve dominante + alvo pesado — a
dominância só percorre o solver de contato se COLIDIR).

**⚠️ LOC — DOIS splits, e a lição:** rodei o gate errado nas 2 waves anteriores. O `architecture_workspace_
file_loc_cap` varre `crates/*/src`, NÃO `shells/`; a shell tem gate PRÓPRIO **`shells/desktop/tests/
file_loc_caps.rs`** (HR-18, cap 600). A wave da Massa deixou `physics_smoke_props.rs` em 643 > 600 sem eu ver
(rodei só o gate de crates). Fix: (1) `physics_smoke_props.rs` 717 → split, cenas 19 (Mass) + 20 (Dominance)
foram pro irmão **`physics_smoke_collision.rs`** (a costura é real — as duas lançam um mover numa fila e variam
o que faz atravessar: PESO vs PRIORIDADE); 561 agora. (2) `inspector_physics.rs` 607 → o helper `shape_mass`
virou **`Collider::auto_mass()`** em `ph2d-physics-ecs` (o lar natural — a massa auto é propriedade do collider);
582 agora. **A partir daqui o `file_loc_caps` da shell entra no gate de fechamento.**

**Ids/consts:** `INSP_PHYS_DOMINANCE` (NumberInput) · `ph2d::physics::Dominance` · `BodyDesc.dominance` ·
`Collider::auto_mass`. `populate.rs` registra a range (-10..10, step 1; o componente/desc tomam qualquer i8).
`PROJECT_SCHEMA` fica **29**. Smoke `=20` (2 lanes: mover LEVE Dominância 5 ara a fila de pesados e segue; o
neutro do mesmo tamanho QUICA no 1º).

**Aberto (deliberado):** damping/drag por-corpo (precisa de modo de combinação com o global — `damp_mode` do
Godot); é a única propriedade "padrão" (Unity/Godot) que falta. Nada mais pede.

## §W-Material — REGRAS DE COMBINE do material (Bounce/Friction Combine, 2026-07-20, smoke `=21`)

**Completa o material físico.** `Bounce` e `Friction` já eram autorados (valores por-collider), mas sempre
combinavam por **`Average`** — o default do rapier. Preço invisível: o artista põe **Bounce = 1.0** numa
superball, larga num chão comum (**Bounce = 0.0**), e ela mal quica (as duas médias dão 0.5). Não havia como
autorar *"esta bola quica em QUALQUER coisa"*. Duas seções segmentadas novas — **Bounce Combine** e **Friction
Combine** (Average/Min/Multiply/Max) — logo abaixo de Bounce/Friction. ⚠️ **NÃO Dynamic-only** (é propriedade
de MATERIAL do collider): o combine de um chão ESTÁTICO importa. rapier resolve um contato com
`rule1.max(rule2)` (ordem `Average<Min<Multiply<Max`), então **a regra de MAIOR prioridade dos dois vence** —
uma bola `Max` quica em qualquer piso, independente da regra do piso. Isso torna a feature robusta e o smoke
determinístico (só a bola carrega a regra).

**Por que ESTA wave e não o damping:** o damping por-corpo está gateado por [`defaults.rs`](../../crates/ph2d-physics/src/world/defaults.rs)
§18-22 — exige um **modo Combine/Replace** (a doença "duas portas pro mesmo número"), é a wave mais espinhosa.
O combine de material é o oposto: material puro de collider, **zero interação** com os defaults globais, e
*fecha* a história de Bounce/Friction (o `PhysicMaterial` da Unity = friction + bounce + os dois combines).
⚠️ **Não confundir os dois "combine":** o combine do material (esta wave) é como DUAS SUPERFÍCIES se combinam
num contato; o `damp_mode` do damping é como o override por-corpo combina com o DEFAULT global. Perguntas
diferentes.

**Bundle no wrapper (`CombineRules`), componente único no ECS (`MaterialCombine`).** O `BodyDesc` ganhou UM
campo `material: CombineRules { restitution, friction }` (carrega o `CoefficientCombineRule` do rapier — o
BodyDesc já não é rapier-free, carrega `RigidBodyType`) em vez de dois campos flat, pelo mesmo motivo do rapier
(`ColliderMaterial` agrupa os dois) **e** porque as ~33 fixtures de `BodyDesc {` ganham UMA linha
`material: Default::default()` **sem import** em vez de duas linhas + import de `CoefficientCombineRule` por
arquivo. No ECS: enum serde **`CombineRule { Average, Min, Multiply, Max }`** (discriminantes casam com o
rapier de propósito — a seção usa o tag como índice, sem remap) + componente **`MaterialCombine { restitution,
friction }`**. `scale::body_desc` mapeia `CombineRule → CoefficientCombineRule` (a porta única, padrão
`BodyKind → RigidBodyType`); `combine_to_rapier` é `match` explícito (reordenar qualquer enum não compila em
vez de remapear em silêncio). Registro **11→12**, blob-key, **sem bump** (fica **29**). Detach quando **ambos**
voltam a Average (`is_neutral()` — arquivo livre do no-op `{Average, Average}`).

**Gates (red-first + mutação):** wrapper `max_combine_bounces_off_a_dead_floor_and_min_stays_dead` (superball
`Max` num piso morto quica de volta perto da altura de queda; `Average` volta a ~¼; `Min` fica morta; mut tira
`.restitution_combine_rule` → `Max` colapsa em `Average`, max==average==3.535676 = RED) + `the_combine_rule_
reaches_the_collider` (readback direto `Collider::restitution_combine_rule()`/`friction_combine_rule()`). ECS
`the_bridge_folds_the_combine_rule_and_a_rewind_preserves_it` (apex de rebote alto pro `Max`, baixo pro neutro,
e o rewind re-arma; mut bridge ignora o componente → apex cai = RED). Seam `combine_rules_are_offered_for_
every_kind_and_each_option_reaches_the_bus` (⚠️ a propriedade que separa das vizinhas: pintado pra TODO kind, não
gateado em `kind_tag==0`; cada uma das 4 opções despacha o próprio tag; refusado sem corpo). Shell `combine_rule_
read_modify_writes_and_detaches_at_neutral` (o read-modify-write preserva a OUTRA regra; ambos Average →
detach). c9 **63→64 corpos** (superball `Max` bate no chão — o combine só percorre o solver de contato se COLIDIR).

**⚠️ LOC — split do apply + do fn do painel, e UM red LATENTE herdado:** (1) `inspector_physics.rs` bateu **634 >
600** (HR-18) com o braço do material → o `apply_physics_edit` inteiro foi pro irmão **`inspector_physics_apply.rs`**,
re-exportado por `inspector_physics` (caller paths intactos); build lê, o irmão escreve — as duas metades da seção.
(2) `paint_physics_section` bateu **230 > 200** (cap de FN do painel) → o bloco de material (Bounce/Friction + os 2
combines) virou **`physics_rows::paint_material_rows`**, ao lado do `paint_mass_source`. (3) ⚠️ **O gate
`arch_safe_clamp_only` estava VERMELHO desde a wave da Dominance** — o clamp dela (`v.round().clamp(f32::from(i8::MIN),
…)`) usa bounds NÃO-literais sem `safe_clamp`, e a wave da Dominance **não rodou esse gate** (mesma classe do miss do
`file_loc_caps` que a própria Dominance documentou). Corrigido pra `ph2d_editor_core::math::safe_clamp` (NaN-aware).
**A partir daqui o `arch_safe_clamp_only` entra no gate de fechamento.**

**Ids/consts:** `INSP_LIVE_PHYSICS_REST_COMBINE`/`INSP_PHYS_REST_COMBINE[4]` + `..._FRIC_COMBINE[4]` (segmentadas)
· `ph2d::physics::MaterialCombine` · `CombineRule` · `BodyDesc.material` · `CombineRules`. `PhysicsFieldEdit::
{RestitutionCombine,FrictionCombine}(u8)` · `InspectorPhysicsInfo::{restitution,friction}_combine_tag`. Smoke `=21`
(2 superballs Bounce 1.0 no MESMO chão morto: a de `Max` quica alto repetidamente, a de `Average` morre em 2 saltos).

**Aberto (deliberado):** damping/drag por-corpo — segue sendo a única propriedade "padrão" que falta, e continua
esperando a decisão de `damp_mode` (Combine/Replace) que a `defaults.rs` exige. Nada mais pede.

## §W-Damping — DRAG por corpo (Linear/Angular + modo Combine/Replace, 2026-07-20, smoke `=22`)

**Fecha o conjunto padrão** (Unity/Godot). Era a única propriedade que faltava, e a `defaults.rs` §18-22 já a
tinha PROJETADO exigindo um **modo Combine/Replace** (o `damp_mode` do Godot) — *"um segundo campo que ganha em
silêncio do global é a falha de duas-portas que este repo paga"*. Três controles no §11 (Dynamic-only): **Linear
Damping** + **Angular Damping** (num) + **Damp Mode** (Combine|Replace). `Combine` SOMA ao drag global do mundo
(`BodyDefaults`); `Replace` IGNORA o global e usa o override direto. ⚠️ **Com o drag global default (0) os dois
modos COINCIDEM** — o mode só diverge quando o artista autora um drag de mundo (painel W2b). Default `Combine`
(o do Godot). ⚠️ **Não confundir os dois "combine":** o do material (W-Material) combina 2 SUPERFÍCIES num
contato; o `damp_mode` combina o override por-corpo com o DEFAULT global.

**O clobber, e a porta única.** O drag por-corpo é o PRIMEIRO override que colide com o global: `stamp_defaults`
(spawn) e `apply_to_all` (mudança de settings) **escrevem drag global em TODO corpo**, então mudar o drag global
com o Play rodando **clobbaria** um corpo com override. Fix em duas peças que compartilham UMA porta
(`PhysicsWorld::apply_damping_override`, resolve `Replace`/`Combine` vs `body_defaults` — usada por spawn E pelo
bridge; duas cópias resolveriam diferente): (1) spawn aplica o override **DEPOIS** do `stamp_defaults`, então
ganha do global; (2) o bridge tem um **passe de re-stamp por dispatch** (`restamp_damping`, em `prepare` após
reconcile) que re-aplica o override de cada corpo. ⚠️ **O passe lê o `rest.damping` (config no desc), NÃO o
componente vivo** — de propósito: a sim é função de `(tick, rest state)`, então uma EDIÇÃO do override mid-play
é transiente (vale no próximo re-describe em repouso, como todo config por-corpo), enquanto uma mudança do drag
GLOBAL é pega ao vivo pro modo `Combine` (o que o `apply_to_all` pretende). Ler o vivo faria os dois discordarem.

**Bundle no wrapper, componente único no ECS.** `BodyDesc.damping: Option<DampingDesc { linear, angular, replace:
bool }>` (~32 fixtures ganham `damping: None`). ECS: enum serde **`DampMode { Combine, Replace }`** (default
Combine; tag = índice da seção) + componente **`DampingOverride { linear, angular, mode }`**. `scale::body_desc`
mapeia `DampMode::Replace → replace:true`. Registro **12→13**, blob-key, **sem bump** (fica **29**). ⚠️ Detach é
**MODO-AWARE**: neutro = `linear==0 && angular==0 && mode==Combine` (Combine+0 = o global = sem efeito); `Replace(0,0)`
NÃO é neutro (força drag zero, ignorando um drag de mundo — escolha deliberada), então FICA.

**Gates (red-first + mutação):** wrapper `linear_damping_slows_a_slide_and_angular_damping_slows_a_spin` (bola
freada desliza <60% da livre; spin freado <50%; mut tira o apply do spawn → damped==undamped = RED) +
`replace_ignores_the_world_drag_while_combine_adds_to_it` (sob drag global 3, `Replace(0)` desliza 2× o `Combine(0)`;
mut ignora `replace` → iguais 1.5969117 = RED) + readback direto. ECS `the_bridge_folds_damping_and_a_rewind_
preserves_it` + **`a_global_drag_change_mid_play_does_not_clobber_a_replace_override`** (o teste que PINA o passe:
sobe o drag global mid-play, o `Replace` segue deslizando; mut tira `restamp_damping` → clobbado, replace==plain==
3.14 = RED). Seam `damping_rows_are_dynamic_only_and_each_reaches_the_bus`. Shell `damping_read_modify_writes_and_
detaches_only_at_the_mode_aware_neutral` (RMW preserva os outros; `Replace(0,0)` FICA, `Combine(0,0)` detacha). c9
**64→65 corpos** (bola lançada+girando com override — o drag percorre o integrador se INTEGRAR).

**⚠️ LOC — TRÊS splits por RESPONSABILIDADE (os dois primeiros são crates-core, cap 700):** (1) `world.rs` 721 →
`apply_damping_override` foi pro irmão **`world/damping.rs`**; 694. (2) `bridge.rs` 728 → `restamp_damping` pro
irmão **`bridge/damping.rs`** + o `settle` (pré-existente) foi pro **`bridge/hold.rs`** (é a behavior de pausa/hold
— `settle` já era chamado por `hold`, então é move por-responsabilidade, não só por-LOC); 662. Fmt reexpande, então
medi DEPOIS do fmt. **Um tofu pego no fechamento:** um `→` (U+2192) no eprintln do smoke 22 (o gate `no_tofu_glyphs`
varre string literals do shell) → trocado por ASCII.

**Ids/consts:** `INSP_PHYS_LINEAR_DAMPING`/`INSP_PHYS_ANGULAR_DAMPING` (num) + `INSP_LIVE_PHYSICS_DAMPMODE` +
`INSP_PHYS_DAMPMODE[2]` (seg) · `ph2d::physics::DampingOverride` · `DampMode` · `DampingDesc` · `BodyDesc.damping`.
`PhysicsFieldEdit::{LinearDamping,AngularDamping}(f32)`/`DampMode(u8)` · `InspectorPhysicsInfo::{linear_damping,
angular_damping,damp_mode_tag}`. `PhysicsWorld::apply_damping_override` (porta única) · `PhysicsBridge::restamp_damping`.
Smoke `=22` (2 demos: bola VERDE de Linear Damping 4 flutua/desce como pena vs ORANGE que cai rápido; caixa VERDE de
Angular Damping 4 para de girar vs ORANGE que gira pra sempre, ambas hover com `GravityScale 0`).

**Aberto (deliberado):** o conjunto de propriedades "padrão" por-corpo (Unity/Godot) está COMPLETO. Sobram só as
avançadas (soft-body, contact events, per-axis damp_mode). ~~one-way platforms~~ → FEITO, §W-OneWay abaixo.

## §W-OneWay — PLATAFORMA JUMP-THROUGH (one-way, 2026-07-20, smoke `=23`)

**O collider icônico do platformer 2D**, e a primeira wave desta linha que adiciona uma CAPACIDADE em vez de mais
um knob por-corpo (o conjunto padrão fechou no W-Damping). Um collider marcado one-way é sólido **só pelo lado
+Y LOCAL dele**: um corpo que chega por BAIXO atravessa limpo e depois POUSA em cima na volta. É o
`one_way_collision` do Godot.

**rapier já traz a primitiva** (`ContactModificationContext::update_as_oneway_platform`) **e a HISTERESE**
(allowed/forbidden por manifold, que é o que impede o corpo de "pipocar" enquanto atravessa a superfície) — então
esta wave é INTEGRAÇÃO, não solver: qual collider é plataforma, e para que lado ela é sólida. `world/oneway.rs`.

**Como a flag chega ao hook.** Um `PhysicsHooks` é `&self` e só enxerga o contexto do contato, então o bit
"este collider é plataforma" viaja no **`user_data` do próprio collider** (`ONE_WAY_BIT` — que é exatamente para
isso que o rapier oferece `user_data`, e nada mais no repo o usava), junto de **`ActiveHooks::MODIFY_SOLVER_CONTACTS`**,
sem o qual o hook nunca é chamado para aquele par. O `OneWayHooks` é **stateless** (⇒ `Send + Sync` de graça, e
sem espelho da cena para envelhecer), e instalá-lo é **byte-neutro** para toda cena sem plataforma: o rapier só
chama `modify_solver_contacts` em pares onde algum collider pediu a flag.

**⚠️ A NORMAL PERMITIDA VIVE NO FRAME DO COLLIDER1 — e a plataforma pode ser o collider2.** O helper testa
`manifold.local_n1`, a normal no espaço local do **collider1**, apontando para o exterior dele; o rapier NÃO
ordena o par pra gente. Um `+Y` constante só está certo quando a plataforma é o collider1; passar `-Y` no outro
caso (como o demo do rapier faz, e funciona lá porque a fixture dele é toda axis-aligned) assume em silêncio que
os dois colliders compartilham orientação — o que uma plataforma ROTACIONADA ou um corpo girando quebram. Então a
direção é DERIVADA, não assumida:

```text
allowed_local_n1 = R1⁻¹ · (s · platform_world_up),   s = +1 se a plataforma é o collider1
                                                      s = −1 se é o collider2
```

Quando a plataforma É o collider1 isso reduz exatamente ao `+Y` local dela ⇒ **UMA fórmula, sem caso especial**, e
correta para plataforma em qualquer ângulo encontrando corpo em qualquer ângulo.

**⚠️ A MUTAÇÃO QUE SOBREVIVEU — a fixture não continha o fenômeno.** As 3 primeiras versões dos gates passaram de
primeira, e apagar o flip de sinal (`let signed = world_up;`) **passou em todas**: a fixture spawnava a plataforma
SEMPRE primeiro, então ela era sempre o collider1 e o ramo do collider2 nunca rodava. Fix: a **ordem de spawn virou
PARÂMETRO** (`platform_first`) e os 3 testes varrem os dois. Com isso a mesma mutação mata **os três**, e
especificamente em `platform_first=false` — a bola cai até y=−41 onde deveria pousar, e é BLOQUEADA onde deveria
atravessar (o caso collider2 estava completamente invertido). O sinal estava certo; o que faltava era a prova.

**Marker, não campo do `Collider`.** `OneWayPlatform` é marker (presença = boolean, idioma do `Ccd`/`LockRotation`):
apendar em `Collider` seria bump de `PROJECT_SCHEMA` (postcard é POSICIONAL — foi o que `layer`/`is_sensor`/`offset`
custaram cada um), enquanto um componente novo é keyed pelo hash do type-name e é puramente **aditivo**. Registro
**13→14**, **sem bump** (fica **29**). ⚠️ **NÃO é Dynamic-only** — é propriedade de COLLIDER e uma plataforma é quase
sempre **Static**, então gatear em Dynamic (copiando as vizinhas) deletaria o controle exatamente do corpo para o
qual a feature existe. O toggle é oferecido para TODO kind, e o gate de seam pina isso.

**Gates (red-first + mutação):** wrapper `a_body_from_below_passes_through_and_then_lands_on_top` (sobe atravessando,
apex > 1, pousa em y≈0.35; controle SÓLIDO é barrado embaixo) + `a_dropped_body_lands_on_a_one_way_platform`
(one-way não é "não-sólido") + **`the_solid_side_follows_the_platforms_own_rotation`** (plataforma de CABEÇA PARA
BAIXO, π: sólida por baixo ⇒ o corpo largado de cima ATRAVESSA — é o teste que a matemática de direção ganha), os
três nos DOIS spawn orders. Mutações: hooks `()` (2 morrem) · world-up hardcoded (a rotação morre) · sem o flip de
sinal (os 3 morrem). ECS `the_bridge_folds_one_way_and_a_rewind_preserves_it` + seam
`one_way_is_offered_for_every_kind_and_each_option_reaches_the_bus`. c9 **65→67** (plataforma + bola: o hook roda
DENTRO da narrow phase e limpa solver contacts, então atravessar e pousar são folds do caminho determinístico).

**⚠️ LOC — DOIS splits por responsabilidade:** (1) `paint_physics_section` bateu **211 > 200** (cap de FN do painel)
→ Layer + Trigger + One-Way viraram **`physics_rows::paint_collision_rows`** (as três são "como este collider
participa de uma colisão": em que camada, sólido ou trigger, e de que lado). (2) `world.rs` bateu **711 > 700** → a
metade-collider do `spawn_body` virou **`world/collider_build.rs::build_collider`** (`spawn_body` são duas perguntas
— que CORPO é este, e que COLLIDER pendura nele; agora todo campo de `BodyDesc` que descreve a forma e a superfície
tem um lar óbvio); 634 agora, e o split é comprovadamente behaviour-neutro (a suíte inteira segue verde).

**Ids/consts:** `INSP_LIVE_PHYSICS_ONEWAY` + `INSP_PHYS_ONEWAY[2]` · `ph2d::physics::OneWayPlatform` ·
`BodyDesc.one_way` · `oneway::{OneWayHooks, ONE_WAY_BIT, ALLOWED_ANGLE}` · `PhysicsFieldEdit::OneWay(bool)` ·
`InspectorPhysicsInfo::one_way`. **`ALLOWED_ANGLE = FRAC_PI_4`** e é escolha explicada: a distinção que o hook faz é
CIMA vs BAIXO (180° de distância), então o cone só precisa separar isso, e ser generoso é o que impede um pouso
levemente inclinado (ou um contato perto da borda, onde a normal abre em leque) de virar "proibido". O demo do
rapier usa 0.1 rad (5,7°), afinado para caixa chata em plataforma chata. Smoke `=23` (2 lanes: bola sobe ATRAVÉS
da plataforma VERDE e pousa em cima; a LARANJA sólida é idêntica e a bola bate embaixo).

**Aberto (deliberado):** contact events (quem bateu em quem) — precisa de um consumidor de gameplay, e a precedência
do W7 diz que a resposta é torná-lo VISÍVEL primeiro · soft-body/fluidos/fratura seguem fora de escopo (D9) ·
per-axis `damp_mode`.

---

## §W-Area — O CAMPO DE FORÇA (Area Effector, 2026-07-21, smoke `=24`)

**Uma área que EMPURRA o que está dentro dela** — vento, corrente ascendente, esteira, correnteza. É a
segunda wave de CAPACIDADE desta linha (a primeira foi o one-way): um collider **sensor** carrega um vetor de
força em newtons, aplicado a cada sub-passo a todo corpo **dinâmico** que o sobrepõe. `AreaEffector2D` da Unity,
os overrides de `Area2D` do Godot.

**FORÇA, nunca aceleração — e é isso que decide o modelo.** O impulso `F·dt` é **resistido pela massa**, então
uma folha é levada por um vento que um caixote mal sente. Essa assimetria É a feature; e uma zona de
*aceleração* seria a **segunda resposta** para o que o `GravityScale` (W8) já diz sobre um corpo. A metade que
não dá para autorar por-corpo hoje é justamente a força.

**⚠️ A ZONA E A PLATAFORMA ONE-WAY FICARAM MUTUAMENTE EXCLUSIVAS, e isso é física, não layout.** Uma
plataforma one-way é realizada modificando **CONTATOS** do solver, e um sensor não gera nenhum; uma zona de
força é lida do grafo de **INTERSEÇÃO** da narrow phase, que só registra um par quando um dos lados é sensor.
Cada controle é **morto** no modo do outro ⇒ cada um é oferecido só no seu: sólido pergunta *de que lado*,
sensor pergunta *com que força*. São os primeiros controles da §11 gateados em **outro CONTROLE** e não no
`kind_tag` — e o One-Way, que era oferecido para todo kind, agora é oferecido para todo kind **sólido**
(o gate dele ganhou a metade nova; a mutação que a remove sangra).

**⚠️ O impulso ACORDA o corpo** (o `drag` passa `false`; aqui é `true`). Uma zona que não consegue iniciar um
corpo que já assentou e dormiu está quebrada exatamente onde um artista a usaria — a esteira sob o caixote, a
corrente sob a caixa. O preço é que um corpo dentro de uma zona ativa não dorme, o que é honesto: ele está
sendo empurrado.

### ⚠️ TRÊS lições de FIXTURE, e as três são a mesma doença

1. **Os dois CONTROLES foram atropelados pelo próprio experimento.** No wrapper, a bola "que não deve se
   mexer" apareceu a **12,9 m** da origem: a bola de dentro da zona foi lançada e a acertou. Na ponte ECS, o
   controle "que deve cair reto" terminou 2,7 m de lado, pelo mesmo motivo. Os dois foram para **fora do
   caminho** — um para CIMA da coluna, o outro para **contra o vento** (a jusante não bastava; ali o caminho é
   uma *direção*). O produto estava certo nas duas vezes.
2. **A fixture do sono não continha o fenômeno, e DUAS mutações passaram por isso.** A primeira versão
   spawnava a zona junto com a bola ⇒ a bola era empurrada desde o tick 1 e **nunca dormia** ⇒ tanto "o
   impulso não acorda" quanto "força zero também registra" ficaram VERDES. A fixture agora deixa a bola
   **assentar e dormir** (com uma asserção da própria premissa) e só então spawna a zona. Aí `wake_up: false`
   mata o gate: o rapier **não integra corpo dormindo**, então um impulso que muda a velocidade sem acordar
   move zero, para sempre.
3. **O filtro `intersecting` sobreviveu a 5 gates porque toda fixture usava uma caixa.** A narrow phase
   reporta o par assim que os **volumes limitantes** se tocam e diz à parte se as FORMAS se tocam — e para uma
   caixa alinhada aos eixos os dois coincidem. O gate que faltava usa uma zona **REDONDA**: um corpo parado na
   quina da AABB do círculo está 0,34 r fora do círculo, e um vento que soprasse nele estaria soprando fora da
   própria coluna.

### ⚠️ DUAS defesas em camada, medidas — e nenhuma é load-bearing para a simulação

O `zone_force` recusa força zero e recusa collider sólido. **Deletar qualquer uma das duas linhas deixa todos
os gates do wrapper verdes, e isso é esperado:** o `apply_impulse` do rapier abre com `if !impulse.is_zero()`
(lido no fonte, não suposto), então uma força zero não acordaria nada mesmo se registrada; e o grafo de
interseção só existe para sensor, então uma zona sólida não veria ninguém. O que elas COMPRAM: uma zona inerte
nunca entra em `effectors`, então o passeio por sub-passo é pulado inteiro; e a metade do sensor é a **porta
única** que as rows da §11 espelham — é lá, no seam, que a regra é observável. Mesma forma do early-out do
`drag` e do ramo de tinta plana da luz do impasto ([[feedback_layered_defenses_need_per_layer_gates]]).

**E um guard foi REMOVIDO por ser inalcançável:** eu tinha escrito "a zona não empurra a si mesma", mas
`spawn_body` insere **um** collider por corpo, então "o outro collider do par" é sempre outro CORPO — nenhum
gate conseguia alcançar aquela linha. Código que nenhum gate alcança é uma afirmação de que o modelo é mais
frouxo do que é; o invariante ficou escrito onde o código se apoia nele.

### O que atravessa cada camada

`BodyDesc.effector: Option<[f32;2]>` (apendado) → `world/effector.rs` (`zone_force` + `apply`, chamado no laço
de sub-passos ao lado do `drag`) → tabela `PhysicsWorld.effectors` **ordenada por handle** (um corpo em duas
zonas sobrepostas soma os impulsos, e a ORDEM de uma soma `f32` é exatamente o que faz um hash cross-OS
derivar — HR-5); é **config**, nada no laço a escreve, e por isso um restore de checkpoint (que troca as arenas
de corpo/collider, não ela) a deixa válida. Componente `AreaEffector { force }`, registro **14→15**, **sem bump**
(fica **29** — componente novo é keyed pelo hash do type-name; apendar em `Collider` seria posicional).
`InspectorPhysicsInfo::force` + `PhysicsFieldEdit::ForceX/ForceY` + `INSP_PHYS_FORCE_X/_Y`. c9 **67→69** (a zona
+ a bola: o impulso é lido do grafo de interseção, um caminho que nenhum outro corpo do harness percorre).

**A SETA no overlay** — *para que lado isto sopra?* Um sensor que empurra e um que só nota são idênticos na
tela sem ela. Laranja (nenhum collider, joint ou lançamento usa esse tom), e desenhada **mesmo com o relógio
rodando**: uma força é propriedade da ÁREA e não deixa de ser verdade quando a simulação começa (a seta de
lançamento é escondida no play justamente porque deixa). Ela reusa a função da seta de velocidade convertendo
os newtons na velocidade que dariam a **1 kg** em `ARROW_SECONDS` — uma força não é um comprimento, então
qualquer seta para ela é uma afirmação sobre ALGUM corpo, e 1 kg é a referência honesta (a 1 kg os newtons SÃO
a aceleração, então a seta se lê direto do número da row). Uma segunda escala de comprimento seria uma segunda
resposta para *"quão longa é uma forte"*.

### ⚠️ LOC — TRÊS splits, todos por RESPONSABILIDADE

1. **`inspector_physics_apply.rs` 597/600** → os cinco braços cuja PRESENÇA é o valor (Ccd, Freeze Rotation,
   Freeze Position X/Y, One-Way) viraram **`inspector_physics_markers.rs`**. Juntos, a duplicação ficou visível
   e sumiu: um gate, um branch, cinco linhas (`set_or_clear`). Um sexto marker agora é uma linha, não mais
   dezoito. **Eles não tinham gate nenhum no nível da shell** — só no seam do painel, que para no bus — então o
   split trouxe o gate que faltava (`every_presence_marker_attaches_detaches_and_is_refused_without_a_body`);
   refactor sem gate é uma afirmação. 516 + 114.
2. **`components.rs` 702/700** → os **overrides opcionais** (gravity scale, velocidade inicial, os markers de
   constraint, massa, dominance, material combine, damping, one-way, a zona) foram para
   **`components/overrides.rs`**; o pai fica com *o que faz uma entidade ser um corpo* (`RigidBody` +
   `Collider`, obrigatórios). 247 + 495.
3. **`physics_overlay_tests.rs` 670/600** → a **geometria pura** ("este círculo é redondo, em pixels de tela,
   nesta câmera") fica; o **passeio de CENA** (`outlines` sobre um `SimWorld`: cores, sensores, parentesco,
   toggles, setas) vira **`physics_overlay_scene_tests.rs`**. Os helpers `camera()`/`window()`/`points()` ficam
   num lugar só — duas metades não podem começar a discordar sobre o que é um pixel. 212 + 473.

### Gates e mutações

Wrapper (`tests/effector.rs`, 6): dentro é empurrado / fora não · **massa resiste** (o gate que diz que é
força e não aceleração) · a zona **inicia um corpo que já tinha adormecido** · força zero é inerte E
**byte-idêntica** a não ter zona · sai da zona ⇒ para de ser empurrado · **empurra o que sobrepõe a FORMA, não
a bounding box**. Todos varrem `zone_first` nos dois valores — a lição do sinal do one-way, aplicada desde o
começo. ECS (2): a ponte dobra e o **rewind re-arma** · uma zona **sólida** não empurra. Seam do painel (+1,
23 no total) e shell (+2). **Mutações: 12 rodadas, 11 sangram**; a que sobrevive é a do guard de força zero,
documentada acima com o mecanismo (o rapier honra o contrato duas vezes).

### Smoke `=24` — e os números foram MEDIDOS, não escolhidos

ESQUERDA uma **corrente ascendente** (sensor azul, `Force Y = +3,5 N`) com três caixas do mesmo material e três
TAMANHOS: uma força, três respostas — a pequena (0,16 kg) sobe como foguete, a média (0,36 kg) quase **paira**
(o peso dela é 3,53 N) e a grande (0,81 kg) afunda a meia velocidade. DIREITA uma **esteira** (sensor verde,
`Force X = +2 N`) leva um caixote parado e o deixa desacelerar até parar em x ≈ 3,85. ⚠️ A primeira versão
usava **6 N** e **jogava o caixote para fora do mundo em menos de um segundo** — demonstrar a feature tornando-a
impossível de olhar. Selecione qualquer zona: a §11 mostra **Trigger = Sensor** + as duas rows de **Force**; vire
Trigger para **Solid** e as rows **somem** e One-Way toma o lugar delas — a regra da wave, visível. **B** para os
contornos.

### ⚠️ "Como configuro a RESPOSTA de um objeto à área?" — a resposta já existe, e não é script

Pergunta do Enio no smoke. **Não há script em lugar nenhum do PH2D** — a autoria é o Inspector. Mas o
controle por-objeto já está no produto, em duas camadas, e nenhuma delas é nova:

- **QUEM sente:** a **matriz de camadas de colisão** (W2c). Uma zona é lida da narrow phase, e a matriz já
  filtra a narrow phase ⇒ um corpo numa camada que não colide com a da zona é **invisível** para ela. É o
  `colliderMask` do effector da Unity, só que dito **uma vez** na matriz que o artista já autorou no painel de
  MUNDO, em vez de re-digitado em cada área. **Medido** (gate `the_collision_layer_matrix_decides_who_the_zone_can_touch`):
  bloqueado o corpo não anda **um float**, e o controle de mesma camada na MESMA corrida é levado 8,5 m — é ele
  que prova que a zona estava viva.
- **QUANTO sente:** a **massa** (Auto/Manual, W-Mass). Não é configuração da resposta, é a própria lei: `a = F/m`.

**O que NÃO existe:** um *multiplicador por-corpo* ("este objeto sente 50% de qualquer vento"). Seria um
`AreaResponse(f32)` — mesmo idioma dos outros overrides, ~1 wave — e a pergunta de projeto que ele traz é se um
multiplicador global por corpo é a granularidade certa ou se o artista quer *por-área* (Unity resolve com
máscara, não com peso). Não construir sem pedido: hoje a dupla camada+massa cobre "não sente" e "sente menos".

### Aberto no W-Area

Falloff (hoje a força é uniforme dentro da área — Unity tem um gradiente e o Godot não) · torque de área (a
zona empurra o centro de massa, então não faz nada girar) · arrasto de área (a diferença entre "vento" e
"água"; o `DampingOverride` responde por-CORPO, não por-REGIÃO, então é pergunta diferente e não uma segunda
porta) · a força é sempre em eixos de MUNDO, então rotacionar a zona não roda o vento.

---

## §W-Contacts — QUEM TOCA QUEM, ONDE, E SOB QUE CARGA (2026-07-21, smoke `=25`)

**O canal sólido, irmão do sensor do W7.** Um sensor responde *"quem está dentro de mim"* para um collider que
deixa passar; este responde *"em quem estou encostado"* para um que não deixa — e, ao contrário de uma
sobreposição, um contato tem **lugar** e **carga**. Era o item que o handoff nomeava como próximo desde o
W-OneWay, com a precedência do W7 dizendo a resposta: **torne-o VISÍVEL primeiro**.

**⚠️ Read-only, e esse é o contrato inteiro.** Nada dentro do `step` chama isto. O `c9` **não ganhou corpo
nenhum** e o hash saiu **byte-idêntico** ao da wave anterior (`c01d4c6a…`, 69 corpos) — não há nada de novo no
caminho determinístico para provar. Há gate pedindo o hash antes e depois de uma leitura completa, sobre um
mundo em plena colisão.

**Um relatório por PAR, não por ponto de contato.** Uma caixa deitada no chão tem **dois** pontos (as duas
quinas) e um polígono tem mais; relatar cada um responderia *"quantas quinas estão encostando"*, que é fato
sobre tesselação, não sobre a cena. Dois objetos se tocando é UM evento. O relatório leva o ponto **mais
profundo** (onde a colisão mais é) e o impulso **somado**.

### ⚠️ O que o impulso É — e a primeira versão do gate estava perguntando a coisa errada

O gate nasceu como *"cair de 6 m empurra mais forte que estar parado"*. **NÃO empurra:** medido, os dois batem
em sete dígitos (**0,010032237 vs 0,010032236**). O `step` retorna depois de o solver já ter parado o corpo, então
o **pico do impacto vive ENTRE os sub-passos** e sumiu antes de qualquer um poder ler. Um "impact strength" lido
daqui seria um número que nunca fica grande.

O que o número É, e é exato e útil: a **CARGA que aquele par está carregando agora**. Numa pilha de quatro caixas
idênticas os impulsos saem **4 : 3 : 2 : 1** de baixo para cima, porque o contato de baixo segura quatro caixas e
o de cima segura uma. **É fato sobre a CENA** (o mesmo em qualquer timestep), e é por isso que virou o oráculo —
e é a leitura que o tamanho da marca no overlay significa.

### ⚠️ A banda do quase-toque existe, foi medida, e o gate dela precisou de DUAS mutações para se provar

O grafo de contato mantém o par vivo enquanto os **volumes limitantes** se tocam: dois círculos a **0,566** de
distância (raios 0,25) são **1 par no grafo com 0 contatos ativos**, enquanto os mesmos círculos a 0,003 num eixo
**nem estão no grafo**. Relatar o grafo cru chamaria o primeiro par de colisão.

**Duas camadas honram isso e cada uma sozinha basta hoje** — a flag `has_any_active_contact` e o `?` do
`find_deepest_contact` (sem ponto de manifold, sem relatório). Mutar **qualquer uma** deixa os 6 gates verdes;
mutar **as DUAS** deixa o gate do quase-toque vermelho, e foi assim que ele se provou não-vazio
([[feedback_layered_defenses_need_per_layer_gates]]). A flag fica como predicado primário porque é a afirmação
do **próprio rapier**; o lookup apenas *por acaso* a implica, e deixaria de implicar no dia em que pontos
especulativos forem mantidos. ⚠️ E a fixture usa corpos **REDONDOS** de propósito: para uma caixa, forma e
volume limitante são o mesmo retângulo — é exatamente assim que o filtro de sobreposição do W-Area sobreviveu a
cinco gates.

### A metade visível: a CRUZ branca

Uma cruz no ponto mais profundo, **branca** (o único valor que nenhum collider, joint ou campo usa — um toque é
um evento *entre* duas coisas, então não pertence à cor de nenhuma), com os braços crescendo com a carga:
**3 px** solto → **9 px** carregado, régua `LOAD_FULL_NS = 0.05` **medida** (uma caixa de 0,5 m densidade 1
parada reporta ~0,0128; uma pilha de quatro reporta ~0,0511 embaixo). Cruz e não ponto: dois corpos assentados
produzem contatos a milímetros um do outro, e um disco desse tamanho é uma mancha enquanto duas linhas cruzando
ainda leem como *aqui, e aqui*. Desenhada **por cima de tudo** (é a menor marca da tela e fica exatamente SOBRE
os contornos que descreve). Satura na régua de propósito — passado um ponto, *"muito carregado"* é a leitura útil.

### Onde mora o quê

`world/contacts.rs` (`ContactReport` + `PhysicsWorld::contact_reports`, ordenado por handle) →
`bridge/contacts.rs` (`BodyContact` + `rebuild_contacts` no fim do `dispatch`, ao lado do `rebuild_triggers`) →
`physics_overlay_contacts.rs` (`contact_marks` puro + `CONTACT_RGBA`). ⚠️ **Lista plana, não mapa** — ao
contrário do `triggers` (`BTreeMap<sensor, dentro>`, porque um trigger é perguntado sobre UMA entidade), um
contato **não tem dono**: é uma relação simétrica por construção, então a forma honesta é a lista de relações;
`contact_count` varre. **Nada de componente novo, nada de `PROJECT_SCHEMA`** (um contato é estado vivo do solver,
o oposto de config — o `canonicalize` do undo ordena por bytes de componente, e guardar isto ali faria cada frame
virar um passo de undo).

**⚠️ Um comentário mentiroso, encontrado e corrigido de passagem:** o `is_triggered` do W7 dizia *"o Inspector lê
isto para o readout de N inside"*. **Nunca leu** — a §11 não tem row de readout, e grepar por consumidor não achou
nenhum. Foi achado porque a wave nova enfrentou a MESMA pergunta e deu a mesma resposta (a metade visível é o
OVERLAY). Comentário que nomeia consumidor inexistente é pior que nenhum: lê como cobertura.

### Gates e mutações

Wrapper 6 (par único no ponto certo · queda livre não toca nada · quase-toque · **4:3:2:1** · a leitura não move
o mundo · sobreposição de sensor **não** é contato, com o par de interseção como controle). ECS 3 (a ponte publica
entidades · **a lista descreve ESTE frame**, não a história — a armadilha que uma lista de "eventos" convida · a
pilha, com o oráculo nomeando ENTIDADES e não índices: o 1º rascunho afirmava `loads[0] > loads[1] > loads[2]` e
ficou vermelho sobre uma lista que era só a outra ordem, o que não diz nada sobre física). Overlay 3 (carga maior
= marca maior, **e satura** · a marca senta no ponto em **pixels de tela** · o toggle desliga).
**9 mutações, 9 sangram** (as duas camadas contam como uma, provada em conjunto).

### Smoke `=25` — e o V estava de cabeça para baixo

ESQUERDA uma **pilha de quatro caixas**: as cruzes crescem para baixo, 4:3:2:1, medido em cena
(0,0511 / 0,0383 / 0,0256 / 0,0128). DIREITA uma **bola descansando num V** de duas rampas: duas marcas, uma em
cada face inclinada, em (2,35, −0,38) e (2,65, −0,38), **0,00872 cada** — metade do peso da bola em cada rampa, e
nenhuma delas no centro de ninguém (a bola está em 2,5; as rampas em 1,75 e 3,25). ⚠️ A primeira versão girava as
rampas **+0,45 / −0,45** e montava um **Λ**, não um V: a bola rolava do pico e ia parar do outro lado da cena, e
a sonda headless mostrou **um** contato onde deviam ser dois. Roda TOCANDO (o estado interessante é o assentado).
Arquivo próprio (`physics_smoke_contacts.rs`) porque o `physics_smoke_collision` declara ser das cenas que
*autoram um resultado de colisão* — esta não varia nada, ela **observa**.

### Aberto no W-Contacts

Eventos de INÍCIO/FIM (*"eles se tocaram agora"* vs *"estão se tocando"*) — é outra estrutura (precisa de
memória entre frames) e o consumidor honesto dela é gameplay, não overlay · a força de impacto de verdade
(exigiria acumular o pico DENTRO do laço de sub-passos: custo em toda cena para uma leitura de debug) · readout
"Contacts: N" na §11 (a seção não tem row de readout; seria widget novo, e o W7 estabeleceu que a metade visível
é o overlay).

---

## §W-AreaDrag — A ÁREA RESISTE: a diferença entre VENTO e ÁGUA (2026-07-21, smoke `=26`)

**A outra metade do campo de força**, e estava nomeada como aberta desde o W-Area (*"arrasto de área — a
diferença entre vento e água"*). Uma zona com força e sem resistência é um **vácuo que sopra**: a caixa pequena
é arremessada e nunca desacelera. Com `drag`, ela entra, **afunda devagar** e a coisa lê como líquido.

**A MESMA lei em todo lugar.** `v /= 1 + d·dt` — o `apply_damping` do próprio rapier, o mesmo que o default de
mundo e o `DampingOverride` por-corpo usam. A palavra "drag" significa **uma** coisa neste app: decaimento
uniforme, independente de massa. E damping **linear E angular** com um knob só, porque um meio resiste a giro
também (o Godot expõe os dois separados num `Area2D`; aqui uma região é uma *substância*, e o caso assimétrico
já tem dono no override por-corpo).

### ⚠️ Mesma LEI, aritmética diferente — e o gate pina o número exato

O gate nasceu pedindo igualdade e ficou vermelho por **1,25%** (mundo 1,3512776 · zona 1,3681686). A causa não
é aproximação: o rapier aplica damping **dentro do velocity solver**, logo antes de integrar posições
(`velocity_solver.rs:240`), enquanto uma zona aplica no topo do sub-passo — e só alcança corpos que o sub-passo
**anterior** reportou como sobrepostos (o lag de um sub-passo que este módulo documenta desde o W-Area). A zona
faz portanto **exatamente um decaimento a menos**, e a razão medida é `1 + d·dt_substep` = **1,0125**.

O gate afirma **esse número** (`the_zone_drag_is_the_world_drag_law_off_by_exactly_one_substep`), não uma
tolerância: assim, mudar ONDE o arrasto é aplicado aparece como uma quantidade nomeada em vez de 1% misterioso.
⚠️ A alternativa — escrever no `linear_damping` do próprio rapier para os corpos dentro da zona — daria
igualdade exata e foi **rejeitada**: seria o **terceiro escritor** de um campo cujo histórico de clobber o
W-Damping já pagou (o `apply_to_all` global), e exigiria restaurar o valor autorado ao sair.

### ⚠️ DOIS componentes, e a razão é o custo de um bump

O wrapper **junta** os dois num `AreaEffect { force, drag }` (aquele lado não é serializado, e juntar custa a
cada fixture um `effector: None` em vez de dois). Do lado ECS eles ficam **separados** — `AreaEffector` e
**`AreaDrag`** — e isso não é desleixo: o blob de um componente é postcard, que é **POSICIONAL**, então apendar
um campo no `AreaEffector` seria bump de `PROJECT_SCHEMA`, e **um bump recusa TODO projeto já salvo no número
antigo**. Jogar fora trabalho real para evitar um segundo componente é o trade errado. Registro **15→16**,
`PROJECT_SCHEMA` fica em **29**. E os dois são independentemente significativos de qualquer forma: um vento que
não te freia (`drag: 0`), uma poça de xarope que não empurra (`force: [0,0]`).

⚠️ **A mudança de `Option<[f32;2]>` para `Option<AreaEffect>` custou DOIS sítios de fixture, não quarenta** —
todo `effector: None` continua compilando. Trocar o TIPO dentro de um `Option` é barato exatamente onde apendar
um campo é caro.

### Onde mora o quê

`AreaEffect` em `desc.rs` · `effector::zone_force` → **`zone_effect`** (a porta única agora responde *"o que
esta área faz?"*, e recusa a zona **inerte** — sem força E sem arrasto — pelo mesmo motivo de antes: registrá-la
acordaria corpos) · o damping no mesmo laço do impulso · `AreaDrag(f32)` em `components/overrides.rs` ·
`INSP_PHYS_AREA_DRAG` na row **Drag** do bloco de sensor · `PhysicsFieldEdit::AreaDrag`. c9 **69→71** (a poça
+ a bola: o decaimento roda FORA do ponto onde o rapier aplica o dele, então é um fold de `f32` que nenhum outro
corpo do harness percorre).

**Sem visual novo, e é decisão:** uma força precisa de seta porque *para que lado sopra* não é inferível de nada
na tela; um arrasto **se vê nos corpos desacelerando**. Uma cena pausada mostra só o contorno de sensor, e isso
é honesto — não há direção para desenhar.

### Gates e mutações

Wrapper 4 novos (arrasto freia quem cai · **é a lei do mundo menos um sub-passo** · resiste a giro, com controle
fora da poça · zona inerte é byte-idêntica **pelos dois caminhos**, força zero ou arrasto zero). ECS 1 (a ponte
dobra e o rewind re-arma; a fixture usa um corpo com **só** `AreaDrag` e nenhum `AreaEffector`, o que prova o
bundle). Seam +1 na varredura (as 3 rows agora, com a nota de que enumerar as rows conhecidas é a premissa que
apodrece) e shell +1 (**componente próprio: mexer num não mexe no outro**, e o negativo detacha). **9 mutações,
9 sangram.**

### Smoke `=26` — três meios, as mesmas três caixas

VÁCUO (sem zona nenhuma — uma zona vazia seria recusada pelo `zone_effect` de qualquer forma, e um retângulo
pintado que não faz nada é o controle dimmed que este repo apaga) · VENTO (`Force Y = +3,5`) · ÁGUA (a mesma
força **+ `Drag = 4`**). Medido em t = 5 s: no vento a caixa média já estava no fundo em **1 s**; na água ela
ainda está **atravessando a poça** (y = 0,87 e caindo), a pequena **flutua na superfície** (2,23) e a pesada
chega ao chão. É a descida lenta que lê como líquido.

### ⚠️ "Como eu criaria uma zona de água só com a UI?" — o gesto composto virou gate

Pergunta do Enio no smoke do `=26`, e ela expôs um buraco real: **cada edit tinha gate, o GESTO não tinha
nenhum**. Um controle pode estar vivo e a sequência ainda não levar a lugar nenhum — uma row que só aparece
depois de outra, um default que atrapalha, um passo que exige digitar um número que o artista não tem como saber.

O caminho, dirigido pela costura real (`build_physics_info` → `apply_physics_edit`) e agora pinado em
`a_water_zone_is_authorable_with_ui_gestures_alone`:

1. o artista posiciona um retângulo de 2 × 3 m (um sprite — que também dá à piscina a cor translúcida);
2. **Add Physics Body** → o collider nasce **1,00 × 1,50**, a caixa do sprite: *nenhuma dimensão digitada*;
3. **Kind = Static** (uma piscina não cai);
4. **Trigger = Sensor** (dá para entrar nela) — é este passo que faz as rows **Force/Drag** serem oferecidas;
5. **Force Y = 3,5** e **Drag = 4**.

**Cinco gestos, zero código.** O oráculo não é "os componentes existem": é **a caixa que caiu na piscina estar
visivelmente mais alta que a idêntica que caiu ao lado** depois de 3 s (medido **0,44 vs −1,70**). A mutação que
sangra é o `Add` voltar ao fallback de 0,5 m — porque aí o passo 2 passa a exigir que o artista descubra e digite
as dimensões, e o gesto deixa de ser um gesto.

### Aberto no W-AreaDrag

Empuxo de verdade (hoje "flutuar" é uma força constante para cima vencendo o peso — um empuxo real dependeria
da fração SUBMERSA, e a zona não sabe quanto do corpo está dentro dela) · a força continua em eixos de MUNDO,
então girar a zona não gira o vento · falloff dentro da área · torque de área.

---

## §W-Buoyancy — ARQUIMEDES: a área sabe QUANTO do corpo está dentro dela (2026-07-21, smoke `=27`)

**A lacuna que o W-AreaDrag deixou nomeada**, e que eu tinha dito ao Enio ser a parte desonesta: até aqui
"flutuar" era uma `Force Y` constante vencendo o peso. Três defeitos que um artista sente na hora:

1. **não se auto-nivela** — a força não sabe onde a superfície está, então o corpo leve é *arremessado para
   fora da piscina* em vez de parar na linha d'água;
2. **é por MASSA, não por densidade** — o número certo tem de ser re-descoberto por objeto (caixa 4× mais
   pesada, 4× a força), quando a intuição real é *madeira boia, pedra afunda*, propriedade do **material**;
3. **não endireita nada** — barco tombado fica tombado.

`ρ_fluido · |g| · A_submersa`, para cima, no **centroide da parte submersa**, resolve os três com **um número
só**. O corpo sobe até a área submersa gerar o próprio peso ⇒ a linha d'água cai de graça; mais denso que o
fluido nunca chega lá; e o centroide se desloca quando o corpo inclina ⇒ o braço de alavanca **endireita o
barco**, sem uma linha extra.

**A superfície é perpendicular à GRAVIDADE, não ao eixo Y.** Água tem superfície horizontal mesmo numa poça
torta — e o mesmo raciocínio diz que com gravidade lateral a superfície é vertical. Sai de graça e apaga dois
casos especiais; ⚠️ com gravidade **zero** não há empuxo, o que é fisicamente certo e não um degenerado a
tratar. Gate irmão pina cada metade.

**O polígono vem do collider VIVO do rapier**, nunca do `BodyDesc`: é ele que a escala do `Transform` já
alcançou (W6), então uma poça ou um barco escalados boiam com o tamanho que estão **desenhados**. Recorte
Sutherland–Hodgman contra um semi-plano + shoelace para área e centroide. ⚠️ **O ponto de interseção entra na
lista** — sem ele a área saltaria de vértice em vértice e o corpo tremeria na linha d'água (mutação sangra).

### ⚠️ O viés de 0,64%, medido em vez de acreditado

O rapier representa bola e cápsula **exatamente** (sem vértices), então o empuxo as tessela pelas MESMAS portas
que constroem o collider de elipse e que o overlay desenha. Um N-gono regular inscrito tem `(N/2π)·sin(2π/N)` da
área do círculo = **99,36%** em `ELLIPSE_SEGS = 32`. O gate afirma **esse número**, não uma tolerância.

⚠️ **E a 1ª medição dele deu 0,745 — exatamente 3/4** — o que denunciou o mecanismo em vez de esconder: no
PRIMEIRO tick só 3 dos 4 sub-passos aplicam, porque a zona só alcança quem o sub-passo **anterior** reportou
como sobreposto (o lag de um sub-passo que o módulo documenta desde o W-Area). O gate mede em regime.

### ⚠️ QUATRO fixtures nasceram erradas, e as quatro pelo mesmo tipo de erro

1. **Um retângulo não tem quilha.** O gate do barco exigia ângulo pequeno e ficou vermelho sobre um barco
   perfeitamente nivelado que por acaso girou 180° (3,141 rad). O oráculo virou `|sin(ângulo)|`: 0 e π são a
   MESMA pose flutuante.
2. **O gate de "corpo acima da superfície" zerou a gravidade do MUNDO** — e assim desligou o próprio empuxo que
   ele media: verde pelo motivo errado (e o gate irmão já cobria esse caso). Agora a gravidade é normal e o
   controle é quem tem `gravity_scale: 0`.
3. **O controle foi atropelado pelo experimento — TERCEIRA vez nesta linha** (o W-Area teve duas). O corpo de
   baixo, arremessado por uma poça de densidade 100, atingiu o controle a **21,8 m**. Controle vai para outra
   COLUNA, e a densidade da fixture desceu para 8.
4. **A linha d'água foi medida cedo demais.** O sistema é amortecido; medir a deriva a 400 passos reprova um
   corpo que ainda está assentando, que é o produto funcionando. Agora mede sobre os últimos 300 de 1200.

### ⚠️ O smoke afirmava "fica a meia-água" e a medição desmentiu

A caixa de densidade **igual** à do fluido (4) **vai ao FUNDO** (−2,46 medido). Está certo: empuxo neutro não
empurra de volta, ele só deixa de puxar — a velocidade com que ela chega só é removida pelo arrasto. A física
estava certa e **a frase mentia**; a caixa do meio virou densidade **3** e agora flutua *quase toda submersa*
(−0,11), que é o caso intermediário de verdade.

### Onde mora o quê

`AreaEffect.density` · `world/buoyancy.rs` (`local_polygon` — a porta única de *"que forma é esta?"*, lendo o
collider do solver — `clip_below`, `area_centroid`, `buoyant_force`, `apply`) · `zone_effect` conta densidade
como não-inerte · `AreaBuoyancy(f32)` em `components/overrides.rs` (**registro 16→17**, `PROJECT_SCHEMA` fica em
**29** pela **terceira vez pela mesma razão**: componente novo é aditivo, campo novo seria bump, e um bump
**recusa todo projeto já salvo**) · row **Fluid Density** no bloco de sensor · `PhysicsFieldEdit::AreaDensity`.
c9 **71→73** (a poça + a caixa que entra INCLINADA, então o momento restaurador entra no hash).

⚠️ **`apply_impulse_at_point`, não `apply_impulse`** — é o único gate que distingue as duas, e é o que
endireita o barco. O empuxo sai do laço de empréstimos do `effector::apply` porque precisa ler os DOIS
colliders enquanto escreve no corpo; a lista é reusada por zona, então cena sem empuxo não aloca.

### Gates e mutações

Wrapper 7 (linha d'água é EQUILÍBRIO · densidade decide, não massa · **o barco se endireita** · corpo fora da
poça intocado · gravidade zero = sem empuxo · **superfície ⊥ gravidade** · o viés do polígono). ECS 1 (dobra +
rewind, fixture com **só** `AreaBuoyancy`). Seam +1 (as **quatro** rows da área) e shell +2 (**terceiro
componente, e mexer num não mexe nos outros** · **a poça com empuxo é autorável só com gestos da UI**, cujo
oráculo é a cortiça **PARAR** perto da superfície — exatamente o que separa Arquimedes da força constante).
**6 mutações no kernel, 6 sangram.**

### Smoke `=27`

Poça (`Fluid Density = 4`, `Drag = 1.5`) e cinco corpos, medidos: cortiça (d=1) para em **y ≈ 0,14** · madeira
(d=3) flutua quase submersa em **−0,11** · pedra (d=12) vai ao fundo em **−2,55** · bola (d=1,5) boia em
**0,06** · e o barco entra tombado a 1 rad (**sin 0,84**) e se endireita para **sin 0,01**.

### ⚠️ "Como criar o tronco boiando pela UI?" — e o passo que NÃO se dá

Pergunta do Enio depois do smoke. O gesto é **curto** — o sprite comprido, **Add**, e
**Density = 1** — e a lição está no passo que eu quase recomendei: converter para
**Capsule**.

Uma cápsula é **Y-alinhada por design** (`ShapeDesc::Capsule` documenta o porquê: um eixo
configurável seria uma segunda forma de dizer o que o `Transform` já diz), e um tronco
DEITADO é largo. A conversão toma `radius = min(hx, hy)` — a regra honesta *"a cápsula
nunca fica mais larga que a caixa"* — e numa caixa larga isso dá `half_height = 0`:
**o círculo inscrito na altura**. Medido: 2,4 × 0,5 vira uma bola de raio 0,25, e o tronco
**deixa de endireitar**, porque um círculo não tem orientação. Não é bug da conversão, é
geometria — e o contorno (**B**) mostra a bola, então o artista VÊ.

O caminho certo é **deixar a caixa que o `Add` já casou com o sprite**. Medido, largado a
1 rad: y → 0,13 (linha d'água) e ângulo **1,000 → −0,001 rad**. Pinado em
`a_floating_log_is_authorable_with_ui_gestures_alone`, cuja segunda metade **converte para
cápsula e afirma o círculo** — sem ela, a recomendação *"não converta"* seria prosa sem
prova. Duas mutações sangram: o `Add` ignorar a caixa do sprite, e o empuxo aplicar no
centro de massa em vez do centroide submerso.

### ⚠️ A LINHA D'ÁGUA — a pergunta do Enio pegou o buraco certo

*"Como a água faz boiar o tronco? Não temos UI para isso?"* — a UI existia (a row **Fluid
Density**), mas o **overlay não desenhava NADA**: um campo de força ganha seta, um arrasto
não tem direção para desenhar, e o empuxo tem um **lugar** — a superfície — que era o
único número do modelo invisível na tela. O artista posicionava o tronco no olho.

Agora a poça desenha sua linha d'água (**ciano claro**, o único traço do overlay que
descreve um LUGAR em vez de um corpo; sobra de 6 px em cada ponta para ler como
superfície e não como aresta do retângulo), **antes** dos corpos — ela é o cenário, e o
que se lê por cima são as coisas que boiam.

⚠️ **Sai da MESMA `surface_level` do empuxo** (`buoyancy::waterline` →
`PhysicsWorld::waterlines` → ponte → overlay), nunca de uma re-derivação: duas respostas
para *"onde está a água?"* divergiriam numa poça rotacionada ou sob gravidade lateral, que
é precisamente onde ninguém confere. Só zona com **densidade** tem linha (vento e xarope
não têm superfície nenhuma para mostrar), e sem gravidade não há linha.

⚠️ **Uma mutação SOBREVIVEU e nomeou o buraco da fixture:** trocar `up = -g/|g|` por
`(0,1)` deixou os 9 gates verdes — porque **com gravidade padrão os dois são a mesma
coisa**. A metade que faltava é gravidade **lateral**, onde a superfície é VERTICAL; com
ela o gate sangra. É a mesma lição de sempre, e desta vez a mutação a encontrou antes do
smoke.

**LOC:** a linha levou `world.rs` a 703/700 ⇒ split em **`world/queries.rs`** (as
consultas de LEITURA — irmão de `sensors`/`contacts`, e o corte que o arquivo já vinha
fazendo: *o que MOVE* e *o que se OLHA* não são a mesma responsabilidade). 4 gates novos
(2 wrapper, 2 overlay), 4 mutações, **4 sangram** após a correção da fixture.

### ⚠️ §W-FormDrag — o arrasto que sabe para onde o corpo aponta (2026-07-21)

O `Drag` da área é **viscosidade**: uniforme, igual em toda direção — o modelo certo para
*xarope*, e o que rapier/Unity/Godot chamam de damping. Mas ele não sabe nada sobre a
FORMA, então um tronco atravessado desce exatamente como um tronco de proa.

**Shape Drag** é a outra metade: cada aresta virada para o escoamento é empurrada ao
longo da **própria normal**, no ponto dela. Dá duas coisas que o uniforme não pode dar —
**resistência por SECÇÃO** (o mesmo tronco sofre **4×** mais de través que de proa) e
**freio de rotação pela FORMA** (um tronco comprido resiste a girar muito mais que uma
bola de mesma área). Componente `AreaFormDrag`, registro **17→18**, schema **fica em 29**
(4ª vez), c9 **73→75**.

**⚠️ TRÊS coisas que eu esperava e a medição negou:**

1. **O cata-vento não existe num corpo simétrico.** Construí isto prevendo *"o tronco vira
   para a correnteza"* e medi **zero torque em toda inclinação**. Não é bug: num corpo
   simétrico o centro de pressão é o centroide, que é o centro de massa, e `r × F` some
   aresta por aresta. Uma flecha só se alinha porque as penas ficam **atrás** do centro de
   massa. Pinado num gate para ninguém "consertar" com um torque inventado.
2. **A força tem de seguir a NORMAL, não a velocidade.** A 1ª versão empurrava ao longo de
   `v` (arrasto se opõe ao movimento, parece natural) e o torque saía **exatamente zero**
   por **identidade** — forças todas paralelas sobre um polígono fechado se cancelam.
   Pressão de fluido age perpendicular à superfície, e a não-paralelidade é o que
   produz **sustentação** (a placa inclinada plana de lado enquanto cai).
3. **Uma amostra por aresta mede freio de rotação ZERO.** No meio da aresta de um corpo
   simétrico a velocidade de rotação é exatamente tangencial ⇒ `v·n = 0`. O efeito inteiro
   está no GRADIENTE ao longo da aresta — `EDGE_SAMPLES = 2` o captura.

**Smoke `=28`, e a fixture nasceu CONTAMINADA duas vezes** (a sonda pegou antes do Enio): com os troncos caindo de
FORA, o de proa — 2,0 de altura — entrava na zona antes e já descia diferente; e pousados, os dois descansavam em
alturas diferentes por geometria. A comparação só é sobre DIREÇÃO se todo o resto for igual, então eles nascem
**já dentro** de uma zona funda. Medido: viscosidade **−5,04 / −5,02** (juntos) · forma **−0,17 / −2,63** (2,5 m
de diferença) · e o mesmo tronco girando dá **2,74 rad** na viscosidade contra **1,21** na forma.

⚠️ **E DUAS mutações sobreviveram à primeira rodada, ambas nomeando fixture faltante:**
trocar a normal pela velocidade (só divergem na **sustentação**, que nenhum gate media) e
a ponte descartar o componente (não havia gate de ECS). Com os dois gates novos, **8
mutações, 8 sangram**.

### ⚠️ "Tudo isso está exposto na UI?" — conferido, e virou GATE

Pergunta do Enio depois do smoke das oito waves. A resposta é **sim**, conferida
componente a componente: os **18** componentes de física registrados têm todos um caminho
de escrita a partir do Inspector (§11 · §12 · o botão Join), e o `Transform` — posição,
**rotação**, escala, skew — é autorável na seção de Transform, que é o que permite largar
um tronco inclinado ou montar um V de rampas.

Mas **prosa envelhece**, e a nona wave será escrita por alguém que não estava nesta
conversa. Um componente que chega ao motor sem chegar à §11 é o **órfão** que a DIRETIVA
§2 proíbe: funciona em toda cena de smoke (que constrói com código) e é inalcançável no
produto — o modo de falha exato do painel de MUNDO no W2b, onde tudo a jusante funcionava
sobre um painel que não existia no build.

`shells/desktop/tests/every_physics_component_is_authorable.rs` fecha isso: para cada nome
registrado em `register_physics_components`, algum dos três escritores de UI
(`inspector_physics_apply` · `inspector_physics_markers` · `inspector_joint`) tem de
nomeá-lo. Estrutural, sobre o fonte. Ele **não** prova que o controle está pintado — isso
é o `architecture_panel_wiring_parity` e os seams que CLICAM — prova que ele **existe**, e
era a metade que faltava. Duas mutações: registrar um componente sem escritor (falha
nomeando-o) e o parse ler zero (falha dizendo que um gate que não lê nada passa sempre).

### Aberto no W-Buoyancy

Arrasto de forma (o arrasto da área é uniforme; um casco de barco deveria resistir mais de lado que de proa —
depende da secção projetada, que este módulo já sabe calcular) · a superfície é PLANA (ondas seriam outra
coisa) · o `local_polygon` não conhece `Compound`/`TriMesh`, e devolve `None` — nenhum empuxo é melhor que um
empuxo sobre silhueta inventada.

---

## W-ContactEvents — *começou a tocar* / *parou de tocar* (2026-07-22, smoke `=29`)

A frente **A** do handoff de reabertura, e a que ele chamava de *"o mais valioso, e o mais
desenhado"*. O W-Contacts entrega **quem está tocando agora**; isto entrega a **transição**
— o que um consumidor de gameplay de fato consome (som de impacto, dano, gatilho).

`PhysicsBridge::contact_events() -> &[ContactEvent]`, com `ContactPhase::{Began, Ended}`,
o LUGAR do toque e a carga. ⚠️ **O consumidor de gameplay continua NÃO construído** — é
cross-line e é decisão do Enio (a mesma fronteira que o W7 traçou para os sensores). O que
esta wave deve é o primitivo **mais uma leitura VISÍVEL dele**, para o canal não nascer
flag morto.

### A armadilha que a wave existe para evitar

O conjunto permanente é **recomputado do zero todo dispatch**, então um diff ingênuo
transforma **todo movimento descontínuo do relógio numa tempestade de colisões**: arraste a
régua para trás sobre uma pilha assentada e cada par que o replay encontra parece novo em
folha. Mas nada começou — o artista mexeu no relógio.

A lei: **um evento descreve uma transição que a simulação de fato ATRAVESSOU.** Toda
descontinuidade derruba `contacts_continuous`, e o rebuild seguinte adota o conjunto **em
SILÊNCIO** (`discard_contact_history`, com **dois** chamadores — `rewind_to` e `hold`).

⚠️ **A baseline nasce VAZIA e contínua**, então o primeiro frame simulado reporta o que
achar, inclusive uma pilha autorada já encostada. É a leitura da **Unity**
(`OnCollisionEnter` dispara no 1º `FixedUpdate` para corpos pré-tocando) e a única
defensável: a narrow phase nunca tinha rodado, logo não existe verdade anterior.

### ⚠️ O bug VIVO que a busca por uma baseline encontrou

`hold` (o toggle **Physics** do transporte desmarcado) limpava `triggers` — com um
comentário explicando que um overlap velho acenderia um sensor com nada dentro — e **não
limpava `contacts`**. Os contatos saem da narrow phase, e só `step` a atualiza ⇒ **desligar
a física deixava as cruzes na tela**, descrevendo toques num mundo que o artista podia
então desmontar com a mão. A metade sólida da frase que a metade sensor já dizia, faltando
desde o W-Contacts. Gate red-first + mutação M2.

### A metade visível: o flash é um `×`, não um `+` maior

O comprimento do braço da cruz **já significa CARGA**. Um flash que também crescesse os
braços colocaria dois significados num canal só — um toque novo e leve ficaria idêntico a
um velho e pesado. O flash entra a **45°**: por 6 ticks (~100 ms, régua de display, não
knob) um `×` abre e some; juntos leem como faísca, e depois o `+` volta a dizer só o que
sempre disse. `BodyContact.age_ticks: Option<u64>` — e o `None` (*"já tocava quando o
relógio pulou"*) é exatamente o que impede um scrub de acender a cena inteira.

⚠️ **Um gate achou um defeito de desenho real:** o flash nascia com 4 px, **menor** que os
9 px da cruz de carga máxima, então o pouso no contato mais carregado da cena seria
anunciado por uma marca escondida dentro da que ela anunciava. `FLASH_MIN_PX` passou a ser
**derivado** de `MARK_MAX_PX`.

### ⚠️ O LIMITE, medido — um impacto RÁPIDO não produz evento nenhum

A amostra é tomada depois que `step` retorna, e o solver resolve um pouso duro **e já
separa o corpo dentro do mesmo passo**: medido numa bola de raio 0,3 largada de 3,9 m, ela
desce até `y = −0,478` contra profundidade de toque `−0,5` e **já está subindo** quando o
tick acaba. Nos dois instantes em que o canal amostra, o par não está encostado — e o
pouso que uma pessoa vê claramente é invisível aqui.

Fronteira **varrida**: de 1,2 m (~5,8 m/s) **todo** quique é reportado; de 2,0 m (~7,0 m/s)
o primeiro **não**. A cena `=29` larga dentro da faixa reportada **de propósito**, e diz
isso na mensagem — uma cena que largasse de 3,4 ensinaria que a feature falha.

**É o MESMO mecanismo do pico de impulso que falta** (`ContactReport::impulse`), e tem a
mesma cura: amostrar **DENTRO do laço de sub-passos**. Quem construir *"força de impacto
real"* (frente **B**) ganha este de graça — **são uma wave, não duas**.

### Gates

**9 no kernel** (`crates/ph2d-physics-ecs/tests/contact_events.rs`) + **4 no overlay**.
**8 mutações, 8 sangram** (M1 rewind mantém a história · M2 `hold` deixa as cruzes — o bug
vivo · M3 flag de continuidade ignorada · M4 par re-baselinado carimbado como começando
agora · M5 overlay pisca par sem começo · M6 flash não expande · M7 flash não morre · M8
flash nasce menor que a cruz de carga máxima).

⚠️ **`reading_the_transitions_does_not_move_the_world`:** esta wave adiciona **memória**
(um mapa, um flag, uma fila), que é exatamente o tipo de adição que começa a escrever de
volta no dia em que alguém decidir que um evento deve acordar um corpo. **Provado com
número, não com argumento:** `physics_ecs_c9` roda **75 corpos, 120 passos, hash
`7d55a4abb03fb4654c1a3e62492b7741de7d5a79e36817668983df43ab081177`** — **byte-idêntico** ao
do `main` (rodado nas duas árvores). Nenhum bump de `PROJECT_SCHEMA` (fica **29**), nenhum
componente novo (registro fica **18**): uma transição é estado vivo do solver, o oposto de
config.

⚠️ **Um gate nasceu vermelho por FIXTURE, não por produto:** o do scrub mirava o tick 20,
onde as caixas ainda estão CAINDO — nada tocava. Corrigido cobrindo os **dois** scrubs
perigosos: para um tick onde tudo também toca (o diff ficaria quieto por SORTE, então
aquela metade afirma a IDADE) e para dentro da queda livre, onde quatro pares somem de uma
vez e um diff ingênuo anunciaria quatro partidas que nunca houve.

### LOC

`bridge.rs` bateu **722/700** ⇒ split por RESPONSABILIDADE: `rewind_to` +
`rebuild_from_rest` viraram `bridge/rewind.rs` (642), o irmão exato do `bridge/hold.rs` —
*o que a ponte faz quando o relógio PARA* e *o que ela faz quando ele volta* são os dois
comportamentos de uma timeline que não avança, e nenhum pertence ao meio do caminho que dá
passo.

### Aberto no W-ContactEvents

O **consumidor de gameplay** (marker de timeline / callback de script) segue cross-line e
decisão do Enio · o impacto rápido invisível **e** o pico de impulso, que são **um** item
com **uma** cura (amostrar dentro do laço de sub-passos, e o preço é pago por toda cena —
medir antes) · eventos **por-tick** em vez de por-dispatch (um frame que deve vários ticks
reporta a diferença entre as duas pontas) · readout de contatos na §11.

## W-ImpactForce — *quão forte foi o toque* (2026-07-22, smoke `=30`)

A frente **B** que a `W-ContactEvents` nomeou, e a cura que ela e a `W-Contacts` prometiam:
o `impulse` de um contato é a **CARGA** que o par segura *agora* — não o **PICO** do
impacto. Medido (`tests/measure_impact.rs`): a mesma bola cai de 0,6 m ou de 10 m e a carga
de repouso é **plana em ~0,014 N·s**, enquanto o pico do impacto cresce **0,58 / 1,13 / 1,59
/ 2,18 / 2,98 / 3,89** de 0,6 a 10 m. O pico vive **entre** os sub-passos e some quando
`step` retorna (o solver já parou o corpo). É o número que um som de impacto quer.

### O kernel

`PhysicsWorld` ganhou um campo **`contact_peaks: BTreeMap<PeakKey, f32>`** (par de corpos →
pico do impulso sobre os sub-passos do tick). `step()` **limpa** no início e chama
`contacts::accumulate_peaks` (um `max` por par) **depois de cada** `physics_pipeline.step()`.
`ContactReport` ganhou `impact` (o pico, lido do mapa; `>= impulse` sempre), e a lógica
`collider→corpo→ordem` virou **uma porta** `active_pair` que `contact_reports` e
`accumulate_peaks` compartilham (duas cópias divergiriam sobre *quais* pares tocam).

⚠️ **READOUT, não estado de solver** — nada no solver lê `contact_peaks`, então é invisível
ao hash de determinismo (que é a pose dos corpos): c9 saiu **byte-idêntico** ao `main`
(`7d55a4ab…`, 75 corpos). `BTreeMap` e não `HashMap` para o readout ser reproduzível
cross-OS (a mesma lei que o resto do módulo).

### Sempre-ligado, porque MEDIDO primeiro (CLAUDE.md §0.0)

O custo do `max` por-sub-passo em toda cena foi medido **antes** de decidir: no pior caso
(pilha de 500 pares) é limitado por cima por `substeps × contact_reports()` = **≤ 0,036
ms/tick = ≤ 2,4% do orçamento HR-4** (e o custo real é menor — `contact_reports` faz
alloc+sort+map que a captura pula). Barato ⇒ **incondicional**: um flag que nunca se desliga
seria flag morto. O `measure_impact.rs` mede o custo sem escrever a captura (limite superior
por APIs que já existem).

### A metade visível

O flash `×` do overlay agora **escala com o `impact`** (`FLASH_IMPACT_BOOST_PX`, régua
`IMPACT_FULL_NS = 2,0` medida) — um slam forte pisca maior que um toque leve. ⚠️ O piso
`FLASH_MIN_PX` (derivado de `MARK_MAX_PX`) fica intacto, então o gate do front A (*o flash
mais leve ainda supera a maior cruz de carga*) segue verde. O `×` (impacto) e o `+` (carga)
são **canais separados** de propósito.

### ⚠️ A armadilha que quase passou — fixture, não código

No nível do mundo `impulse` (tick-end) É o pico quando o par ainda toca no fim do tick — o
corpo é pego **mais forte NA fronteira**, então num tick de **POUSO** `impact == impulse`, e
um gate que lê o endpoint de um tick de pouso **não distingue** o `max`-sobre-sub-passos do
último sub-passo. Medido: no caminho `spawn_body(desc)` (o que a ponte usa) o tick de pouso
dá `load == impact == 3,00` — a coincidência. O gap só aparece num tick de **RASPÃO**, onde o
corpo já saiu antes do último sub-passo (endpoint ~0, pico não): scene-29 tick 63, `load 0,0
/ impact 0,85`. Por isso os gates usam **bola que QUICA + chão FINO** (onde os raspões caem
dentro do `contact_reports`) e perguntam *"existe um tick cujo pico supera claramente o
próprio endpoint?"* — verdade só para o `max` real; as 3 mutações que colapsam `impact` em
`impulse` deixam nenhum tick assim → RED. (Detalhe reproduzível em
`debug_the_peak_shows_only_on_a_grazing_tick`.)

### Gates: 3, e 5 mutações que sangram

- **Wrapper** `the_impact_peak_is_the_hit_the_load_meter_misses` (bola quica, chão fino):
  existe um tick com pico >> endpoint. Mata M1 (`contact_reports` lê `impulse`) · M2
  (`accumulate_peaks` usa `=` em vez de `max`) · M3 (remove a chamada em `step`).
- **Ponte** `a_began_event_carries_the_impact_of_the_landing` (bola quica, ponte): algum
  `Began` carrega pico >> endpoint. Mata M1/M2/M3 (via kernel) **e** M4 (a ponte enfia
  `r.impulse` no campo `impact`).
- **Overlay** `a_harder_impact_flashes_bigger`: impacto duplo → flash maior, mesmo age.
  Mata M5 (o flash ignora o `impact`). O piso de A segue pinado no mesmo gate.

### Aberto no W-ImpactForce — FECHADO pela W-TickContacts (ver abaixo)

O **impacto RÁPIDO invisível** (um toque que começa e termina dentro de UM tick) segue sem
evento — o pico É capturado em `contact_peaks`, mas o diff de eventos do front A é sobre o
conjunto **permanente** (`contact_reports`, vivo no fim do tick), que não vê o toque rápido.
Um evento para o toque rápido reestrutura esse diff (conjunto permanente → conjunto tocado
**por tick**, com a união dos ticks de um dispatch de vários ticks) e é a **próxima wave** —
esta captura é o pré-requisito dela. **Isso é a W-TickContacts, ver a seção final.**

### LOC

`world.rs` bateu **706 > 700** (o campo `contact_peaks` + a captura) ⇒ split
`world/convenience.rs` (os dois construtores `add_dynamic_circle`/`add_static_cuboid`, ~40
linhas, `impl PhysicsWorld` num módulo irmão) → **661**. `physics_smoke_events.rs` 172→290 (a
cena 30) e `physics_overlay_contacts.rs` 221→251 (o flash por impacto) seguem sob o cap 600
do shell.

### A cena 30 — a DEMOLIÇÃO (o 1º corte foi recusado)

O 1º corte da cena 30 era uma **escada** de bolas caindo num chão imóvel, e o `×` crescia com
a queda. O Enio recusou: *"bater no chão imóvel não mostra o efeito"* — e tinha razão, o chão
absorve tudo e sobra só o `×` abstrato. A cena virou uma **DEMOLIÇÃO**: duas raias iguais
(torre de caixas leves + bola pesada lançada), só a VELOCIDADE muda — lenta (5 m/s) a torre
balança e o `×` é pequeno; rápida (16 m/s) a torre EXPLODE e o `×` é enorme. A força do
impacto ganha uma **consequência visível** (as caixas voam), e o `×` a quantifica. Medido
(probe `probe_scene_30`, bola pesada num alvo leve): impacto **0,70 / 1,41 / 2,80 / 4,53** a
3 / 6 / 10 / 16 m/s. **Lição:** um readout de debug (o `×`) só é legível quando amarrado a algo
que o olho já lê como "forte" — e um chão estático não reage. A bola rápida ganha CCD (não
atravessar a caixa fina entre dois passos).

## W-TickContacts — o toque RÁPIDO vira evento (2026-07-22, smoke `=31`, smoke OK 2026-07-22)

A **próxima wave** que a W-ContactEvents e a W-ImpactForce nomearam, e para a qual a captura do
pico (front B) era o pré-requisito. O diff de contatos rodava por **DISPATCH** sobre
`contact_reports` (o estado vivo no FIM do passo), então dois toques ficavam invisíveis:

- **entre dois endpoints de um dispatch multi-tick** — um par que começa e termina dentro do
  span (catch-up / scrub-forward) não aparece em nenhum dos dois endpoints;
- **dentro de UM tick** — o solver resolve um pouso duro e já separa o corpo antes do fim do
  passo, então nos dois instantes em que o canal olhava o par não estava encostado.

**Medido (probe `probe_fast_bounces`, 1 tick/dispatch):** o 1º pouso de uma queda de **3 m** não
gerava evento (o 1º Began reportado era um quique lento no tick **91**, não no pouso ~45); uma
queda de **8 m** não gerava evento **nenhum** nos primeiros 100 ticks. Depois do fix: 3 m → 1º
pouso no tick 45 (impacto 2,83); 8 m → tick 76 (impacto 4,80); 10 m → tick 85 (5,38).

### O kernel — o diff roda por TICK sobre a UNIÃO dos sub-passos

O `contact_peaks` que o front B acumulava para o pico **já é a união dos sub-passos** (um par
tocado em qualquer sub-passo está lá, mesmo que suma antes do último) — só faltava consumi-lo
para eventos. `PhysicsWorld::tick_contacts` o expõe; o `PeakSample` ganhou `point` + `impulse`
(o `impact` é o `max`, `point`/`impulse` são o ÚLTIMO sub-passo ativo — o evento de um toque que
já saiu precisa de um lugar e uma carga). A ponte diffa esse conjunto **depois de cada
`world.step()`** no laço forward, não uma vez no fim (`rebuild_contacts` virou
`accumulate_contact_events` por tick + `rebuild_standing_contacts` por dispatch). O único toque
que ainda escapa é o que começa **e** termina no MESMO sub-passo — que o solver discreto nem
produz (seria túnel, trabalho do CCD).

### O flash virou canal próprio, event-sourced

O `×` do overlay cavalgava `BodyContact.age_ticks`, então vivia só enquanto o par **TOCAVA** —
um pouso curto **sub-piscava** e um rápido **nunca piscava** (ele nunca entra na lista
permanente). Agora é `ContactFlash` (a/b, point, impact, age), semeado dos `Began` e **decaído
em ticks pela ponte**, `CONTACT_FLASH_TICKS` mora lá. Um começo pisca sua vida inteira, encoste
ou não. `age_ticks`/`began` **saíram** (só serviam ao flash antigo); a supressão de re-baseline
que o `None` fazia foi **subsumida** — um re-baseline (scrub/disarm) simplesmente não cria flash.
`discard_contact_history` também apaga os flashes vivos (uma descontinuidade não é hora de
acender).

### Custo — em play normal, ZERO regressão

Play normal é 1 tick/dispatch, então o diff roda **exatamente** as vezes que o front A rodava.
Só um catch-up/scrub multi-tick paga K diffs, e cada diff é um `BTreeMap`-sobre-contatos (µs
contra os **ms** do `step`, que roda por tick de qualquer jeito — medido **57 ms/tick** a 500
contatos numa pilha assentada, `measure_impact.rs`). O enriquecimento do `PeakSample` (2 escritas
a mais por par por sub-passo) ficou **≤ 2,2%** do `step`.

### Gates — 12 ecs + 10 overlay, 6 mutações que sangram

Red-first (os 2 fast-touch): `a_fast_landing_fires_during_normal_one_tick_play` (8 m,
1 tick/dispatch — o caso comum) e `a_touch_between_two_dispatch_endpoints_still_fires` (queda de
3 m, joga tick-a-tick até 40 e depois UM dispatch até 55 que abrange o pouso; a bola está no ar
nos dois endpoints, provando que é invisível ao diff de endpoints). Mais:
`the_begin_flash_decays_over_a_fixed_span_even_while_the_pair_keeps_touching` (o flash é um
começo, não uma duração) e `a_discontinuity_puts_out_a_live_flash` (disarm com o flash ainda
aceso). As 6 mutações: **união→fim-de-passo** (`out.clear()` por sub-passo) mata os 2 fast-touch
+ o gate de impacto do front B · **once-per-dispatch** (accumulate fora do laço) mata SÓ o
multi-tick (o single-tick é 1 tick = 1 diff, imune) · **light_flash no-op** mata 3 de presença ·
**nunca-descartar** (retain sempre true) mata o decay · **discard-não-limpa** mata o live-flash ·
**re-baseline-off** (`if false`) mata SÓ o re-arm-silencioso (um scrub é sempre backward, nunca
roda o accumulate forward — por isso não sangra ali). **c9 byte-idêntico** (`7d55a4ab…`, readout
puro). `PROJECT_SCHEMA` **fica 29**, registro **fica 18**.

### A cena 31 — a bola de 8 m cujos pousos eram escuros

Duas bolas que quicam, mesma restituição, só a ALTURA muda: a BAIXA (1,2 m) tem pousos lentos
que sempre acenderam (controle); a ALTA (8 m) tem pousos rápidos que eram **invisíveis** — ela
quicava alto e não acendia `×` nenhum, e quanto mais forte o pouso, mais invisível. Agora todo
pouso acende, e o `×` da alta é MAIOR (impacto maior). É a mesma máquina do pico da cena 30: a
30 mostrou a FORÇA de um toque que já era reportado; a 31 mostra um toque que **não era
reportado de jeito nenhum** passando a existir.

### Aberto no W-TickContacts

O **consumidor de gameplay** (colisão → som/dano/marker/callback) segue cross-line e decisão do
Enio, a fronteira que o W7 desenhou — este canal é o primitivo + a leitura visível, não o
consumidor. E o toque começa-e-termina no MESMO sub-passo (túnel sem CCD) fica sem evento por
construção.

## W-AreaTorque — a MESA GIRATÓRIA (2026-07-22, cena `=32`, pendente de smoke)

A frente da **família das zonas** que a reabertura nomeou. O `AreaEffector` (W-Area) empurra
pelo CENTRO DE MASSA e não gira nada — este é o análogo ROTACIONAL: uma área que aplica um
**torque** a cada corpo dinâmico dentro dela (um redemoinho, uma mesa giratória, uma esteira que
gira). Fecha o item "torque de área" aberto desde o W-Area.

### O kernel — o torque pendura na porta única do efetor

`AreaEffect` (o bundle do `desc`, **não serializado**) ganhou `torque: f32`, e `effector::apply`
aplica `b.apply_torque_impulse(effect.torque * dt, true)` no MESMO laço que já aplica a força.
⚠️ **`apply_torque_impulse`, NÃO `set_angular_velocity`** — o impulso é resistido pelo **MOMENTO
DE INÉRCIA** do corpo, o espelho EXATO de a força ser resistida pela massa: um tronco comprido
gira mais devagar que uma bola de mesma área (medido: 8,03× para uma barra 4×0,25 contra uma
compacta 1×1). Uma zona de *aceleração angular* seria independente da forma e uma segunda porta
para o que a inércia já responde. ⚠️ **O SINAL é o sentido** (`> 0` anti-horário, `< 0` horário),
então o neutro é `== 0.0` — diferente dos irmãos de arrasto (`<= 0.0`): um torque negativo é uma
direção, não um valor inválido. O `zone_effect` ganhou `&& e.torque == 0.0` no `inert` (uma zona
só-torque É uma zona).

### Componente novo, ZERO bump

`AreaTorque(f32)` — o **quinto** componente da mesma área, pela quinta vez pela mesma razão: um
blob é postcard POSICIONAL, então apendar campo no `AreaEffector` seria bump de `PROJECT_SCHEMA` e
um bump **recusa todo projeto salvo**; um componente novo cunha blob-key próprio e é aditivo.
Registro **18→19**, `PROJECT_SCHEMA` **fica 29**. A ponte dobra `AreaTorque` no MESMO `AreaEffect`
(mais um `zone_torque` no `any`), então ele rida o `BodyDesc` e um rewind o re-arma de graça.

### A metade visível — o GLIFO de giro

`AreaEffector` desenha uma SETA (para que lado sopra?); o torque desenha um **arco de 270° com
ponta de flecha**, VIOLETA (cor que nenhum collider/joint/lançamento/força usa), no centro da
zona, em pixels de tela constantes (um giro não tem "longe"). ⚠️ O arco é construído numa BASE de
tela derivada da câmera (`û` = onde o mundo +x cai na tela, `ŵ` = +y), não em ângulos de tela
cravados, então um torque +τ desenha o sentido que um corpo VISIVELMENTE giraria sob esta câmera,
qualquer que seja o y-flip. Desenhado **mesmo no play** (o giro é propriedade da ÁREA, como a seta
da força). Sem esse glifo uma zona de giro seria uma caixa magenta indistinguível de um sensor
comum.

### ⚠️ A armadilha que a medição pegou: `rotation` WRAPA, `angvel` não

Um torque forte gira o corpo VÁRIAS revoluções, e o readback escreve `Transform.rotation`, que dá
a volta em ±π — então a rotação lida vira ruído (medido: compacta 2,688, barra −1,254 com torque
6, sem sentido como taxa). Os gates de MUNDO leem o `angvel` cru (não-wrapado) e podem apertar; os
gates de ECS/gesto/smoke leem `rotation` e por isso a fixture mantém o giro **sub-revolução**
(torque 0,5–1,0 num corpo 1×1 → ~86–171° por segundo, sem wrap). É a mesma classe de
[[reference_topic_oracle_discipline]]: uma coordenada que WRAPA é um oráculo ruim quando a
grandeza pode passar do período do wrap — meça a TAXA, não o ângulo acumulado.

### Gates — 4 mundo + 2 ecs + count + seam + gesto + overlay-scene, 5 mutações que sangram

Mundo (`ph2d-physics/tests/effector.rs`): spins-inside+outside-still · sign-sets-direction ·
moment-of-inertia-resists (compacta > barra × 4, o pin "torque não é aceleração") · solid-zone-
spins-nothing. ECS (`ph2d-physics-ecs/tests/area_torque.rs`): fold+rewind · solid-coupling. Mais:
count 18→19 · seam sensor-only + commit NEGATIVO (o sinal atravessa o event layer sem clamp) ·
gesto "mesa giratória autorável só com UI" · overlay-scene glyph presente/ausente/direção. As **5
mutações**: neutralizar `apply_torque_impulse` (3 gates de mundo RED) · tirar torque do `inert`
(zona só-torque não registra, spin RED) · `torque.abs()` (sign RED) · glifo ignora o sinal (scene-
direction RED) · ponte nunca dobra `AreaTorque` (ECS fold RED). **c9 77 corpos** (mesa +
spinner), hash `27f3c1aa…` **determinístico entre debug/release** — ⚠️ MUDA em relação ao main
(`7d55a4ab…`), e isso é CORRETO: o torque muda a sim, ao contrário dos readouts A/B/C.

### A cena 32 — quatro caixas flutuantes

Compacta (+1, rápida) · barra (+1, 8× mais lenta = a inércia) · compacta (−1, gira ao contrário =
o sinal) · controle sem zona (parada). Flutuam por `GravityScale(0)` (o giro vem só do torque).
Números medidos no 1º segundo: 171° · 21° (razão 8,03) · −171°. `B` liga o glifo.

### Aberto no W-AreaTorque

Falloff dentro da área (o giro é uniforme; um redemoinho real cai com o raio) · o torque é
constante (não decai com a distância ao centro) · a família das zonas ainda tem **o frame da
zona** (a força/torque em eixos de MUNDO — girar a zona não gira o vento) por fazer.
