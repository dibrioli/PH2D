# HANDOFF / Tracker — `line/physics` (o motor de física global)

> **Tracker VIVO do módulo** (o `docs/HANDOFF_*` da física). Toda jornada futura **atualiza este
> arquivo**: estado por-wave, decisões, gotchas, ids/consts alocados. LLM nova lê ISTO + a
> [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) +
> [`00_plano_waves.md`](00_plano_waves.md) antes de tocar código.
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
| **W2c — Camadas de colisão** | ✅ **LANDOU** (pendente smoke) | ver §W2c | matriz no painel + camada no Inspector |
| **W3 — Joints** | ⏭️ **A PRÓXIMA** | — | pêndulo/corrente/ragdoll; bumpa o schema **21 → 22** |
| **W4 — Bake-to-timeline** | ⏳ pendente | — | acopla `ph2d-anim` (outra linha) |

**W0 entregou:** [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) ·
[`00_plano_waves.md`](00_plano_waves.md) · [`01_visao.md`](01_visao.md) · este tracker. Nenhuma linha de
código, nenhum contrato tocado, nenhum foundational tocado.

---

## §W1 — O alicerce LANDOU (2026-07-17, pendente smoke)

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

**⚠️ SMOKE (Enio):** `cd Worktrees/line-physics && PH2D_PHYSICS_SMOKE=1 cargo run -p ph2d-host-desktop` —
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

## §W1.5 — O relógio pra trás LANDOU (2026-07-18, pendente smoke)

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

## §W2 (metade) — A AUTORIA: a seção "Physics Body" no Inspector (2026-07-18, pendente smoke)

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

## ✅ §W2c — camadas de colisão (2026-07-18, pendente smoke)

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
6. **O que smoke-testar (Enio):** `cd Worktrees/line-physics && PH2D_PHYSICS_SMOKE=1 cargo run -p
   ph2d-host-desktop` → a bola cai e assenta. **E confirme que o app normal (sem a env) segue igual** — o
   `physics_bridge::dispatch` roda todo frame, mas é no-op sem entidades de física (query vazia).

**Resumo:** *Linha `physics` (W0+W1) pronta — HEAD `9f5fee05`, 5 commits. Foundational tocado: `ph2d-physics`
(meu módulo, aditivo, c9 intacto) + shell (consumidor). Contratos congelados: nenhum. Colisões a grepar: ADR
0130 · `PROJECT_SCHEMA=16`+tripla-pin (CONTAR se outra linha bumpar). 6 gates mutation-verified; batched gate
verde. Smoke pendente: `PH2D_PHYSICS_SMOKE=1`. Aguardo ordem de integração / W1.5 / W2.*
