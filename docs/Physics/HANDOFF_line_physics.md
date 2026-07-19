# HANDOFF / Tracker — `line/physics` (o motor de física global)

> ⚠️ **VAI ASSUMIR ESTA LINHA? Comece por
> [`HANDOFF_CONTINUACAO_line_physics_2026-07-19.md`](HANDOFF_CONTINUACAO_line_physics_2026-07-19.md)** —
> a linha **INTEGROU** ao `main` (as 8 waves + W4b + W5, smoke aprovado) e o **plano original acabou**.
> Aquele doc te diz como REABRIR a linha, onde paramos e os planos a seguir. Este tracker é o estado
> por-wave; aquele é o ponto de partida.
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
