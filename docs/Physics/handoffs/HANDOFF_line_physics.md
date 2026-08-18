# HANDOFF / Tracker — `line/physics` (o motor de física global)

> ⚠️ **VAI ASSUMIR ESTA LINHA? Comece por
> [`HANDOFF_REABERTURA_line_physics_2026-08-10.md`](HANDOFF_REABERTURA_line_physics_2026-08-10.md)** —
> a linha foi **REABERTA DO ZERO em 2026-08-10** (worktree nova, HEAD = `main`, **zero commits**,
> árvore limpa). Aquele doc traz o estado medido, o que **não** reconstruir, o que está **aberto com
> o preço ao lado** e os gotchas operacionais. Este tracker é o estado **por-wave**, para consulta
> pontual — não leitura linear.
>
> **Estado no `main` de 2026-08-10** (medido, não auto-relatado): `PROJECT_SCHEMA` **70**, tripla
> `(70, 13, 14)` · registro `ph2d-physics-ecs` **29** · `physics_ecs_c9` `fb27f676…`, **117 corpos**
> · gizmo ids até **973** (próximo livre **974**) · maior cena de smoke **104** (próxima livre
> **105**; ⚠️ o `=84` não existe, de propósito).
>
> ⚠️ **E o número da CENA tem duas leituras, então ele vem em duas linhas:** o **105** acima é o
> do `main`, e é onde ele está certo; **nesta LINHA o roteador já vai a `119`, e a próxima livre
> é `120`** — quem escolher um número lendo só a linha do `main` nasce colidido. ⚠️ **O
> mecanismo protege, ao contrário do que a nota do `CLAUDE.md` dizia:** o roteador da física é
> um `match` sobre a string do env (`physics_smoke.rs`), então dois braços com o mesmo número
> são `unreachable pattern` **no compilador** — a frase *"uma lista de `if level == N` e o
> primeiro vence"* é a do **Vector**, copiada para cá, e ela descreve um silêncio que aqui não
> existe. É por isso que esta família **não tem** um gate irmão do
> `no_two_smoke_scenes_claim_the_same_level`: escrevê-lo seria a segunda resposta a uma
> pergunta que o `rustc` já responde.
>
> ⚠️ **A jornada de 2026-08-15 FECHOU. A linha entrega o handoff e PARA** —
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-15.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-15.md),
> **117 commits**. Ela tem TRÊS partes: a **fila da auditoria** (sete waves, duas
> recusas medidas, smokadas) · a **AUDITORIA FINAL** de três blocos (§2c) · e a
> **CAUDA de cinco waves** (§2e: `W-WallNormal` · `W-Ceiling` · `W-Bonked` ·
> `W-HitNormal` · `W-WallMaterial`), que fechou o **§3.A** e o item das **camadas**.
> As duas últimas partes **não têm smoke, com o argumento escrito** no §2d.
>
> ⚠️ **Estado medido no TIP** (não auto-relatado): `PROJECT_SCHEMA` **82** ·
> registro `ph2d-physics-ecs` **32** · `physics_ecs_c9`
> **`2d7f9d51…`, 121 corpos**, debug ≡ release · gizmo ids **inalterados** ·
> **zero** `Cargo.toml`/`Cargo.lock`/ADR. ⚠️ **O hash MOVEU contra o `main` por
> duas causas distintas** — a CONTAGEM pela lane pareada da `W-WallMaterial`, e o
> hash dos 117 antigos pela LEI do player, que já movia antes dela.
>
> ⚠️ **A auditoria 09 saiu de NOVE ❌ para QUATRO, e nenhum dos quatro é trabalho
> pendente** (dois recusados por medição, um por arquitetura, um fora da fila sem
> pedido) — a tabela com o porquê de cada um está no §7b do MESTRE. O único buraco
> real que sobra contra o referencial é *climbing*, que o **plano 08 §4.8** nomeia.
>
> ⚠️ **Jornada de 2026-08-12 FECHADA e SMOKADA, aguardando ordem de integração** — handoff
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md).
> **Seis waves**, as três últimas de REPORT do Enio e não de plano.
>
> ⚠️ **AUDITORIA contra as ENGINES (2026-08-12) — plano, zero código:**
> [`09_auditoria_engines.md`](../09_auditoria_engines.md). O plano 08 fechou o **catálogo de
> platformer 2D** (19/20); esta olha para os **controladores de personagem** (Unity · Godot ·
> Unreal · tnua) e acha outro conjunto — **fronteiras**, não verbos. O maior achado: **o jogador
> não tem SAÍDA** (nem readout de estado, nem eventos), e o canal genérico que existe
> (`SignalOnHit`) é **estruturalmente cego** ao aterrar sob Spring, porque uma perna flutuante
> nunca toca no chão. Fila: saída → frear≠acelerar → a superfície fala → teto de queda →
> empurrão de fora.
>
> ⚠️ **`W-LedgeSensor` — o sensor da beirada ganha POSIÇÃO e EXTENSÃO** (pedido do Enio, com
> pesquisa antes: GDevelop · Corgi · Unreal · hotspots de Sonic). São **QUATRO** controles, e a
> matriz é a razão de cada um — *posição × tamanho, nos dois eixos*: `Ledge Grab` (X pos) ·
> `Grab Span` (X tam) · **`Grab Offset Y`** (Y pos) · `Grab Window` (Y tam). ⚠️ **O span é
> HORIZONTAL porque o raio aponta para BAIXO** — um raio para baixo já integra a janela vertical
> inteira num cast; a varredura vertical é o desenho do Unreal, com traços para a FRENTE, e um
> traço para a frente diz *que há parede* e ainda precisa descobrir a altura. ⚠️ **E o quarto
> controle nasceu de um erro MEU:** eu chamei o `reach_y` de *"o Y"* citando o `Grab offset` do
> GDevelop, **mas aquele é POSIÇÃO** e o que construí com o argumento dele foi o TAMANHO — a
> janela era sempre centrada no topo do corpo, então alcançar um lábio mais alto custava
> **alargar a histerese junto**. A frase do plano 08 §4.51 (*"é assim que os controles continuam
> TRÊS em vez de quatro"*) está **corrigida lá**. ⚠️ O rótulo *"Grab Height"* lia como posição ⇒
> **`Grab Window`**; e o `offset_y` é o único da família **sem `max(0.0)`** (um offset é uma
> DIREÇÃO). ⚠️ **Uma amostra DENTRO recusa o leque INTEIRO** — a rejeição de *"a parede continua
> acima da cabeça"* era grátis enquanto o sensor era um PONTO, e num leque é feita à mão.
> **`span = 0` e `offset_y = 0` reduzem LITERALMENTE ao raio único** ⇒ **`PROJECT_SCHEMA` 76 → 78
> sem mover física** (`c9` `1699123f…` intocado). `PLAYER_ROW_COUNT` **47 → 50**. 6 gates, 5
> mutações, 5 sangram — ⚠️ **a da recusa sobreviveu DUAS vezes por FIXTURE** (v1: todas as
> amostras nasciam dentro da parede, onde `return None` e `continue` empatam; v2: o corpo estava
> no CHÃO, onde `ledge_probe_wanted` nem casta).
>
> ⚠️ **Um ACORDE nunca é entrada de jogo** (report *"os players pulam e se movem sozinhos"*,
> [BUGS #8](../BUGS_physics.md)). **A física foi EXONERADA por medição**: pose bit-constante ao
> longo de 1190 tiques pelo binário do produto, com a cadência real do app. A causa era
> `player_keys.key()` a observar a tecla FÍSICA sem guarda de modificador — **`Ctrl+Z` pulava**,
> `Ctrl+A`/`Ctrl+D` andavam, `Ctrl+S` agachava. ⚠️ **E o doc do `player_input.rs` declarava isso
> impossível.** ⚠️ **A SOLTURA passa sempre**, e a assimetria é a correção: uma guarda simétrica
> trocaria um pulo espúrio por um personagem que **anda para sempre**.
>
> ⚠️ **A marca de sensor pousa onde o corpo ESTÁ** (report *"drift dos gizmos dos sensores"*,
> [BUGS #9](../BUGS_physics.md)). Medido pela porta do produto ANTES de qualquer hipótese: o leque
> fica **0,1000 m atrás em regime** — a distância exacta de um tique a 6 m/s —, e **zero parado**.
> A leitura é gravada ANTES do `step` (é ela que a lei consome) e o `readback` publica a pose
> DEPOIS. ⚠️ **A cura é a ÂNCORA, não um re-cast** (re-perguntar seria a segunda resposta a *"o
> que este sensor viu?"*): o `readback` desloca o leque pelo que o corpo andou, e `hit`/`reach`/
> `skin` ficam como foram medidos. **0,1000 → 0,0000 m.** ⚠️ **O `Sweep` não é tocado, e a
> assimetria é a prova:** ele já viaja como `(corpo, deslocamento)` e por isso **nunca** driftou.
> ⚠️ **E o corte de LOC que a wave exigiu (`player_kinmove.rs`) reprovou DOIS arch-gates sobre
> produto CORRETO** — os dois ancorados no ENDEREÇO `player.rs`; passaram a ler a **FAMÍLIA** por
> uma porta única (`tests/player_bridge_source.rs`). *Afirme a PROPRIEDADE, nunca o endereço.*
>
> ⚠️ **Jornada de 2026-08-12 (`W-MultiJump` + `W-Ledge` + `W-Glide`) FECHADA, aguardando
> ordem de integração** — handoff
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md).
> A **terceira** wave é o **PLANEIO** (`W-Glide`, cena `=112`), e é a que **a medição REFUTOU o
> plano**: o §4.6 supunha *"um multiplicador de gravidade sob botão"* e uma escala **nunca
> ASSENTA** — até 5% da gravidade do mundo a descida continua a crescer (−1,71 / −2,21 / −2,80 a
> 1/3/6 m), então *quão depressa se desce* é função de *quanto já se caiu*. Um **alvo** (a lei do
> `wall_slide`) assenta mas **inverte quem sobe** (Δv = −10 apertado a subir). Ficou o **TETO**,
> que assenta e só desacelera. ⚠️ **Ele não estava escrito em lugar nenhum porque o doc do
> `wall_slide` registra que a versão-teto DELE foi morta por medição** — *"com o atrito default o
> personagem não cai"* —, o que é verdade da PAREDE; **no ar não há atrito**, e a objeção não
> viaja. ⚠️ **E uma mutação sobreviveu a tudo; o gate que a mataria achou OUTRO defeito:** no
> tique da decolagem a `standing` já é `None` de propósito, então o planeio somava **+10,00 m/s
> por cima dos +18,26 do pulo** — sexta guarda, `!jump.takeoff`. ⚠️ **E duas mutações anteriores
> foram NO-OPS SILENCIOSOS que eu li como achados** (o `cargo fmt` colapsara a guarda e o
> `str.replace` não casou): toda mutação passou a **asserir a âncora antes de escrever**.
> **`PROJECT_SCHEMA` 75 → 76** (o teto nasce em `0` ⇒ **`c9` `1699123f…` segue INTOCADO**) ·
> **7 mutações, 7 sangram** · `PLAYER_ROW_COUNT` 47, `PLAYER_CARDS` 11. **Cena `=112`; próxima
> livre 113.**
>
> ⚠️ **As duas primeiras waves da mesma jornada (`W-MultiJump` + `W-Ledge`)** — handoff
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md).
> A **segunda** wave é a **BEIRADA** (`W-Ledge`, cena `=111`, o exemplo que o Enio deu no plano 08
> §4.5): o personagem que erra o pulo **agarra o parapeito** e sobe dali.
> ⚠️ **A bifurcação que o plano tratava como decisão DISSOLVEU** — nem o pendurar nem a subida
> escrevem POSE: os dois são **velocidade** (um `boost` mais o `gravity_hold` que o arranque já
> usava), então **a lei é a MESMA nos dois modos** e o solver continua a resolver contatos.
> ⚠️ **O sensor é UM raio para baixo, à frente**, e `distance == 0` é a recusa de *"a parede
> continua acima da minha cabeça"* **de graça** (o contrato de penetração do `cast_ray`); o **`x`
> dele É o alvo da subida** ⇒ a beirada **não depende do sensor de parede**.
> ⚠️ **Dois limiares de UM número** (agarrar exige o lábio acima da cabeça, segurar aceita a banda
> inteira) — com um só, o sucesso do servo seria a condição de largar. ⚠️ **E a subida é disparada
> por BORDA**: com disparo por nível o pendurar era **invisível**, porque se chega a uma beirada
> *a pular contra ela*, com o dedo já em baixo. **Medido:** pendurado assenta a **2,5 mm** do
> lábio, e depois da subida fica de pé em `lábio + float_height` nos **seis** pares
> `(grab, speed)`. ⚠️ **E um pulo COLADO À PAREDE alcança 0,745 m contra 1,903 do ar livre** (o
> atrito come 61%) — a primeira versão da cena usou o número do ar livre e **o corpo nunca chegava
> à janela**. **`PROJECT_SCHEMA` 74 → 75** (a capacidade nasce em `0` ⇒ **`c9` `1699123f…` segue
> INTOCADO**) · **11 mutações, 11 sangram**, e a do `gravity_hold` **sobreviveu até um gate NOVO
> nascer** (o pendurar não consegue medir esse termo — 0,1 mm; a SUBIDA consegue — 1,011× o
> autorado com ele contra 1,048× sem). **Cena `=111`; próxima livre 112.**
>
> ⚠️ **A primeira wave da mesma jornada (`W-MultiJump`)** — handoff
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md),
> que **supersede** o de 08-11 apenas como *o que integrar agora* (o detalhe de mecanismo das sete
> waves de sensores continua LÁ). O assunto é o **PULO DO AR**
> (cena `=110`, o *air actions counter* do tnua): `air_jumps` + `air_jump_height`, a carga
> recarregando no CHÃO pela porta única `on_ground` — o **terceiro** consumidor dela.
> ⚠️ **E o proxy do ARRANQUE apodreceu com o terceiro pulo:** o `lib.rs` perguntava a
> *transição para o ar*, e um pulo do AR acontece com `airborne` já verdadeiro ⇒ ele
> dizia *não* no gesto que mais se encadeia com um arranque; nasceu `JumpStep::jumped`
> e o **terceiro** gate de cancelamento. **`PROJECT_SCHEMA` 73 → 74** · **`c9`
> `1699123f…` INTOCADO** (a capacidade nasce em `0` ⇒ byte-idêntico, e o hash é a prova
> executável) · registro **29 intocado** · **zero `Cargo.toml`** · nenhum ADR. LOC:
> `jump.rs` cruzou 700 ⇒ `jump_config.rs` (*o que se AUTORA* × *o que acontece num
> TIQUE*). **Cena `=110`.**
>
> ⚠️ **Jornada de 2026-08-11 FECHADA (smoke da cena `=109` APROVADO pelo Enio em 2026-08-12)** — handoff
> [`HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md`](HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md),
> que **inclui** a de 08-10 (as duas estão nos mesmos 37 commits). O assunto são os **SENSORES**:
> eles ficaram **visíveis** (`W-Probes`, cena `=108`), ficaram **editáveis** (`W-Probes2`) e **a
> perna deixou de ser um raio** (`W-FootFan`, cena `=109`) — um raio só afundava **46% do
> `float_height`** parado sobre uma fenda de 10 cm que o corpo atravessa. Mais o catálogo do plano
> 08 (`W-Swim` · `W-ZoneForce` · `W-ShapeCast`). ⚠️ **`PROJECT_SCHEMA` 70 → 73** (três degraus,
> §4 do handoff) · **`c9` `1699123f…`** (move, e a atribuição é por **ablação**: com `samples = 1`
> ele volta exatamente ao do `main`) · registro **29 intocado** · **zero `Cargo.toml`** · nenhum
> ADR. ⚠️ **Rode a suíte com `--no-fail-fast`** — sem ele o primeiro binário vermelho esconde o
> resto, e nesta jornada a diferença foi entre *"um gate caiu"* e *"dez caíram"*.
>
> ⚠️ **A jornada de 2026-08-10 (MEDIÇÃO) está DENTRO da acima** — handoff
> [`HANDOFF_INTEGRACAO_line_physics_bobbing_2026-08-10.md`](HANDOFF_INTEGRACAO_line_physics_bobbing_2026-08-10.md).
> Ela é de **MEDIÇÃO**, e o resultado principal é um **negativo**: o *"bobeio de 1,44 m na água"*
> que a reabertura listava como aberto **não era um defeito** (com os quatro multiplicadores de
> gravidade a `1` a amplitude é o controle ao 4.º decimal; largado **submerso** o player é `1,00×`
> o controle ⇒ a trava do fluido contém, e o excesso é a modelagem do arco a agir **no AR**). Mais
> a paridade de arrasto entre modos, que era precificada por **analogia** e agora tem o número
> **desta** paridade: **`1,149%` no pico, a decair**. **3 gates, 3 mutações, todas sangram** ·
> `PROJECT_SCHEMA` **intocado** · `c9` **byte-idêntico** · zero `Cargo.toml` · nenhum ADR · o único
> toque em `src/` é **comentário**.
>
> **As jornadas que INTEGRARAM e são históricas** (o mecanismo delas continua nos respectivos
> handoffs, não foi copiado para cá):
> [kin 2026-08-09](HANDOFF_INTEGRACAO_line_physics_kin_2026-08-09.md) (o modo cinemático) ·
> [MESTRE 2026-08-08](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-08.md) (⚠️ o de 09/08 o
> supersede **apenas** como *o que integrar agora* — **o detalhe até a W23 está LÁ**) ·
> [MESTRE 2026-08-04](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-04.md) (o player de
> plataforma) · [MESTRE 2026-08-02](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-02.md) ·
> [2026-08-01](HANDOFF_INTEGRACAO_line_physics_2026-08-01.md) ·
> [world_pin 2026-07-30](HANDOFF_INTEGRACAO_line_physics_world_pin_2026-07-30.md).
>
> (Os `HANDOFF_REABERTURA_*_2026-07-22/23.md` e o `HANDOFF_CONTINUACAO_*_2026-07-19.md` estão
> **vencidos** — os planos deles foram todos executados.)
>
> **Tracker VIVO do módulo** (o `docs/HANDOFF_*` da física). Toda jornada futura **atualiza este
> arquivo**: estado por-wave, decisões, gotchas, ids/consts alocados. LLM nova lê ISTO + a
> [ADR-0131](../../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) +
> [`00_plano_waves.md`](../00_plano_waves.md) antes de tocar código.
>
> **Bugs cuja causa ENGANAVA:** [`BUGS_physics.md`](../BUGS_physics.md) — sintoma → causa-raiz →
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
> - **O ADR virou [0131](../../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md).**
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
- [`crates/ph2d-physics/src/world.rs`](../../../crates/ph2d-physics/src/world.rs) (320 LOC,
  `#![forbid(unsafe_code)]`): `PhysicsWorld::new/set_gravity/set_dt/dt/step_count/add_dynamic_circle/
  add_static_cuboid/insert_body/bodies[_mut]/colliders[_mut]/step/body_pose/body_snapshots/
  deterministic_hash`. `step()` **sempre** usa `dt` interno (HR-5). `DEFAULT_DT=1/60`,
  `DEFAULT_GRAVITY_Y=-9.81`, mundo **Y-up**. `BodySnapshot{handle_index,x,y,rotation,linvel_x,linvel_y,angvel}`
  ordenado por `handle_index`; `deterministic_hash` = blake3 sobre snapshots ordenados (`to_bits` LE).
- [`crates/ph2d-physics/Cargo.toml`](../../../crates/ph2d-physics/Cargo.toml): `rapier2d = "0.28"`,
  `default-features=false`, features `dim2`/`f32`/`enhanced-determinism` + `blake3`. **NUNCA** ligar
  `parallel`/`simd-stable`/`simd-nightly`.
- Bin [`c9.rs`](../../../crates/ph2d-physics/src/bin/c9.rs): 50 corpos + chão, 120 steps, imprime
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
- [`crates/ph2d-core/src/time.rs`](../../../crates/ph2d-core/src/time.rs): `FixedStep` — `DEFAULT_HZ=60.0`
  (f64), `DEFAULT_MAX_SUBSTEPS=8`, `advance(wall_dt)->FixedStepReport{ticks:u32,alpha:f32,dropped_secs:f64}`,
  `tick_count()->u64`, `fixed_dt()->f64`.
- [`crates/ph2d-core/src/playhead.rs`](../../../crates/ph2d-core/src/playhead.rs): `Playhead` — `time:f64` seg,
  `advance()` move só se `playing`, `advance_ticks(n)`, `seek/seek_frame` (scrub, não muda play state),
  `rewind()` (time=0, mantém rate+play), `is_playing`, loop Wrap/PingPong. Sequência bit-idêntica cross-OS
  (HR-5).
- **Precedente Motion** [`shells/desktop/src/render_loop/motion_bridge.rs`](../../../shells/desktop/src/render_loop/motion_bridge.rs):
  `ticks_owed(last_cooked, target) -> RangeInclusive<u64>` (`Some(last) if target>last => last+1..=target`;
  senão `target..=target`); caller `for tick in ticks_owed(...) { pump.advance_or_scrub_scoped(...) }`;
  `target = round(playhead.time()/fixed_dt)`. **`MotionTransport` MORREU** — um relógio.

### O checkpoint (modelo do scrub — W1.5)
- [`crates/ph2d-nodegraph/src/cook.rs`](../../../crates/ph2d-nodegraph/src/cook.rs): `CookCheckpoint`,
  `checkpoint()->CookCheckpoint`, `restore(&cp)` (reinstala estado + limpa memo/live-scope, mantém revision
  clock). GGPO save/load/advance.
- [`crates/ph2d-eval-motion/src/checkpoint.rs`](../../../crates/ph2d-eval-motion/src/checkpoint.rs):
  `RECENT_CAPACITY=300` (~5 s @60Hz), `CheckpointRing{recent:VecDeque<(u64,CookCheckpoint)>}` denso,
  `record`/`anchor_at_or_before(target)->(u64,cp)`/`should_record`/`clear` (no `mark_dirty`). Física usa
  cadência **esparsa** (estado maior).

### Registro de components (a armadilha do snapshot)
- [`crates/ph2d-ecs/src/scene/registry.rs`](../../../crates/ph2d-ecs/src/scene/registry.rs):
  `register::<T>("ph2d::ecs::Nome")`; ids = blake3(name) 8 bytes LE. `register_ecs_components(reg)` +
  tripwire `register_ecs_components_populates_registry` (`reg.len()==32`, *"este número existe para doer"*).
  **Padrão:** a crate-home possui `register_*` e o boot agrega
  ([`shells/desktop/src/init.rs`](../../../shells/desktop/src/init.rs), ao lado de `register_render_components`).
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

---

## Índice das waves — a história mora no ARQUIVO

> ⚠️ **Este tracker era de 710 KB e é agora de ~34 KB.** As 103 seções de narrativa foram
> para [`docs/archive/tracker-physics-2026-08-18/`](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md) **verbatim** — a remontagem das duas
> metades bate sha256 com o original, nenhuma linha foi editada.
>
> **Porquê:** medido em 2026-08-18, este arquivo teve **1 `Read` para 407 comandos de shell**, e
> **89% dele nunca entrou em contexto nenhum** — ninguém o lia, raspavam-no em ilhas de ~70 linhas.
> Ele guardava **667 marcadores `⚠️`/`⛔`, 558 deles além da linha 2.000**. Uma regra na linha 8.000
> não é difícil de achar: ela não é lida. (`CLAUDE.md §5.0`)
>
> ⚠️ **A tabela «Estado por-wave» que vivia aqui foi arquivada porque MENTIA:** parou em
> `W-Offset (2026-07-20)` enquanto o arquivo seguiu até 2026-08-15. Este índice é **derivado** dos
> cabeçalhos, e o arquivo está congelado — por isso ele não pode envelhecer da mesma forma.
>
> ⛔ O que estiver lá marcado **«medido e REJEITADO»** continua rejeitado.

| # | wave (link para a linha no arquivo) |
|---:|---|
| 1 | [Estado por-wave](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L17) |
| 2 | [§W1 — O alicerce LANDOU (2026-07-17, **smoke aprovado**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L46) |
| 3 | [§W1.5 — O relógio pra trás LANDOU (2026-07-18, **smoke aprovado**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L127) |
| 4 | [§W2 (metade) — A AUTORIA: a seção "Physics Body" no Inspector (2026-07-18, **smoke aprovado**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L247) |
| 5 | [✅ §W2b — o painel global de mundo LANDOU (2026-07-18) · **smoke APROVADO** (re-smoke pós-fixes)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L353) |
| 6 | [✅ §W2c — camadas de colisão (2026-07-18, **smoke aprovado**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L570) |
| 7 | [✅ §W3 — joints (2026-07-18, **smoke aprovado**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L630) |
| 8 | [✅ §W4 — bake-to-timeline (2026-07-18, **smoke aprovado**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L870) |
| 9 | [W4b — o toggle **Physics** na barra da timeline (2026-07-18, **smoke aprovado**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1210) |
| 10 | [W5 — corpos FILHOS: o collider volta para debaixo do sprite (2026-07-18, pendente smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1343) |
| 11 | [Handoff de INTEGRAÇÃO — W0 + W1 (§1.5.9)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1465) |
| 12 | [§W6 — A ESCALA ALCANÇA O COLLIDER (2026-07-19, smokada pelos gates)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1522) |
| 13 | [§W7 — SENSORES / TRIGGERS (2026-07-19, smokada pelos gates + smoke `=10`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1581) |
| 14 | [§Weld — o 5º joint (`FixedJoint`, 2026-07-19, smoke `=11`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1638) |
| 15 | [§BakeChannels — assar um subconjunto dos canais (2026-07-19)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1681) |
| 16 | [§W8 — GRAVITY SCALE por corpo (2026-07-19, smoke `=12`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1718) |
| 17 | [§Capsule — o collider de personagem (2026-07-19, smoke `=13`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1756) |
| 18 | [§W9 — VELOCIDADE INICIAL por corpo (2026-07-19, smoke `=14`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1799) |
| 19 | [§W-CCD — DETECÇÃO CONTÍNUA por corpo (2026-07-20, smoke `=15`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1841) |
| 20 | [§W-LockRot — FREEZE ROTATION por corpo (2026-07-20, smoke `=16`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1889) |
| 21 | [§W-Offset — OFFSET do collider por corpo (2026-07-20, smoke `=17`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1928) |
| 22 | [§W-LockPos — FREEZE POSITION X/Y por corpo (2026-07-20, smoke `=18`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L1967) |
| 23 | [§W-Mass — MASSA MANUAL por corpo (2026-07-20, smoke `=19`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2014) |
| 24 | [§W-Dominance — DOMINANCE / prioridade de colisão por corpo (2026-07-20, smoke `=20`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2059) |
| 25 | [§W-Material — REGRAS DE COMBINE do material (Bounce/Friction Combine, 2026-07-20, smoke `=21`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2101) |
| 26 | [§W-Damping — DRAG por corpo (Linear/Angular + modo Combine/Replace, 2026-07-20, smoke `=22`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2162) |
| 27 | [§W-OneWay — PLATAFORMA JUMP-THROUGH (one-way, 2026-07-20, smoke `=23`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2218) |
| 28 | [§W-Area — O CAMPO DE FORÇA (Area Effector, 2026-07-21, smoke `=24`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2295) |
| 29 | [§W-Contacts — QUEM TOCA QUEM, ONDE, E SOB QUE CARGA (2026-07-21, smoke `=25`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2439) |
| 30 | [§W-AreaDrag — A ÁREA RESISTE: a diferença entre VENTO e ÁGUA (2026-07-21, smoke `=26`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2540) |
| 31 | [§W-Buoyancy — ARQUIMEDES: a área sabe QUANTO do corpo está dentro dela (2026-07-21, smoke `=27`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2638) |
| 32 | [W-ContactEvents — *começou a tocar* / *parou de tocar* (2026-07-22, smoke `=29`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2844) |
| 33 | [W-ImpactForce — *quão forte foi o toque* (2026-07-22, smoke `=30`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2950) |
| 34 | [W-TickContacts — o toque RÁPIDO vira evento (2026-07-22, smoke `=31`, smoke OK 2026-07-22)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3045) |
| 35 | [W-AreaTorque — a MESA GIRATÓRIA (2026-07-22, cena `=32` smoke OK 2026-07-22; cena `=33` + fix de sync pendentes de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3124) |
| 36 | [W-AreaFrame — o FRAME da zona: girar o sensor gira o vento (2026-07-23, cena `=34`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3216) |
| 37 | [W-AreaFalloff — o empurrão desvanece do centro para a borda (2026-07-23, cena `=35`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3346) |
| 38 | [W-AreaMirror — virar o sprite vira a correia (2026-07-23, cena `=36`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3477) |
| 39 | [W-BakeRange — o início do loop é honrado (2026-07-24, cena `=37`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3560) |
| 40 | [W-BakeJoint — assar um joint puxa o grupo articulado inteiro (2026-07-25, cena `=39`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3699) |
| 41 | [W-JointAuthoring — re-pick dos corpos de um joint + smoke de autoria (2026-07-25, cena `=40`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3766) |
| 42 | [W-AnchorFollow (padrão-ouro W1) — A ÂNCORA É BODY-LOCAL E SEGUE O CORPO (2026-07-25, `6f337986c`, cena `=41`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3860) |
| 43 | [W-JointParams (P0 — correção) — TUNAR UM PARÂMETRO DE JOINT AO VIVO (2026-07-25, `line/physics`, cena `=42`, **smoke OK 2026-07-25**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3899) |
| 44 | [W-JointCreate (padrão-ouro) — ESCOLHER O TIPO NA CRIAÇÃO (2026-07-25, `ec0c944ad`, cena `=40`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4013) |
| 45 | [W-J1 (plano 02) — O JOINT DESENHA O QUE ELE É (2026-07-25, `line/physics`, cena `=43`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4044) |
| 46 | [W-J2 — A âncora tem DUAS alças, e um ímã (2026-07-25, cena `=44`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4135) |
| 47 | [W-J2b — As alças ficam MAIORES, aparecem sozinhas e ganham o pixel (2026-07-25, cena `=44`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4235) |
| 48 | [W-J3 — Pose, não digite: o limite e o comprimento no canvas (2026-07-25, cena `=45`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4345) |
| 49 | [W-J4 — Criar onde se olha (2026-07-25, cena `=46`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4462) |
| 50 | [W-J4b — a saída, e as alças fora de alcance (2026-07-25, mesma cena `=46`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4594) |
| 51 | [W-J5 — Slider (Prismatic): o 5º tipo (2026-07-26, cena `=47`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4677) |
| 52 | [W-JG — O GRUPO CARREGA O RIG (2026-07-26, cena `=51`, **smoke APROVADO** 2026-07-26)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L5426) |
| 53 | [W-Grab — A MÃO: pegar o corpo no PLAY (2026-07-26, cena `=52`, **smoke APROVADO** 2026-07-26)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L5604) |
| 54 | [W-Hand — A SEÇÃO DA FERRAMENTA: tipos de segurar, explosão e campo de atração (2026-07-26, cena `=53`, **smoke APROVADO** 2026-07-26)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L5768) |
| 55 | [W-IK — POSAR ARRASTANDO A PONTA (2026-07-27, cena `=54`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6022) |
| 56 | [W-FK + W-JointTools — A CINEMÁTICA DIRETA, e os cinco modos numa seção própria (2026-07-27, cena `=55`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6105) |
| 57 | [W-Pulley W3 — A TALHA: a roldana montada num corpo (2026-07-28, cena `=61`, **smoke OK**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6206) |
| 58 | [W-Pulley W4 — O TAMBOR DIFERENCIAL: a vantagem mecânica CONTÍNUA (2026-07-28, cena `=62`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6351) |
| 59 | [W-Pulley W5 — A COMPOSIÇÃO: o tambor e a cadernal na mesma corda (2026-07-29, cena `=63`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6436) |
| 60 | [W-Pulley W6 — AS ALÇAS que faltavam na roldana (2026-07-29, cena `=63`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6502) |
| 61 | [O PISO — uma corda não pode ser mais curta que o caminho que ela enfia (2026-07-29, cena `=63`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6746) |
| 62 | [A ROTA QUE NÃO RESOLVE — os guardas de degeneração, medidos (2026-07-29, cena `=63`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6801) |
| 63 | [O §10 FECHOU — os dois números que faltavam (2026-07-29, sem mudança de código)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6861) |
| 64 | [W-WheelMagnet — o eixo de uma roldana MONTADA tem ÍMÃ (2026-07-29, cena `=61`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6909) |
| 65 | [W-DrivenSheave — a cadernal dirigida por curva é um guincho de vantagem 2 (2026-07-29, sem cena nova)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6985) |
| 66 | [W-RopePick — a corda de uma roldana se RE-ESCOLHE (2026-07-29, cena `=61`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7054) |
| 67 | [W-Weston — a talha DIFERENCIAL (2026-07-29, cena `=64`, ordem do Enio)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7118) |
| 68 | [Estado da linha (2026-07-30) — **INTEGRADA, e a linha está REABERTA**](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7174) |
| 69 | [Estado da linha (2026-07-29) — HISTÓRICO, pré-integração](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7254) |
| 70 | [§W-Rig — o rig sai da HIERARQUIA (2026-07-31, cena `=67`, **pendente de smoke**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7323) |
| 71 | [W-SoftWeld — a solda que CEDE (2026-07-31, cena `=68`, plano 02 §12)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7419) |
| 72 | [W-Compound — um corpo, VÁRIAS formas (2026-08-01, cena `=69`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7513) |
| 73 | [W-PartFace — a PEÇA vira EDITÁVEL (2026-08-01, cena `=70`, **smoke APROVADO**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7615) |
| 74 | [W-PartSensor — o SENSOR DE PÉ (2026-08-01, cena `=71`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7753) |
| 75 | [W-CompoundZone — uma ZONA vê o corpo COMPOSTO inteiro (2026-08-01, cena `=72`, pendente de smoke)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7849) |
| 76 | [W-PartMass — o seed do `Mass: Auto → Manual` conhece as PEÇAS (2026-08-01, sem cena)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7962) |
| 77 | [W-CompoundContact — um corpo COMPOSTO toca UMA vez (2026-08-01, sem cena)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8013) |
| 78 | [W-WorldPinGlyph — a ponta que é o CENÁRIO ganha figura (2026-08-01, cena `=65`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8108) |
| 79 | [W-WorldPinLocal — a alça de ONDE NO CORPO o pino prende (2026-08-01, cena `=65`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8178) |
| 80 | [W-Signal — uma colisão FAZ alguma coisa acontecer (2026-08-01, cena `=73`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8228) |
| 81 | [W-LeadDrag — arrastar o corpo da âncora A leva o SISTEMA (2026-08-02, cena `=74`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8305) |
| 82 | [W-RopeStop — o LIMITADOR, e a força que empurrava de lado](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8422) |
| 83 | [W-SignalLeave — a porta que FECHA, e a row que era write-only (2026-08-03, cena `=76`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8526) |
| 84 | [W-PartAdopt · W-RopeSays · W-RailRope — os três abertos que tinham cura (2026-08-03)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8667) |
| 85 | [Jornada de 2026-08-03 (2ª sessão) — W-JointAnim + W-JointCustom](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8806) |
| 86 | [W-KinMove — O SEGUNDO MODO (2026-08-08, cena `=101`, **2º RE-SMOKE pendente**)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8944) |
| 87 | [W-KinWeight · W-KinPush · W-KinPure — as três caudas do modo novo (2026-08-09)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9071) |
| 88 | [W-KinCarry — a plataforma era contada DUAS vezes (2026-08-09)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9145) |
| 89 | [W-Swim — **NADAR** (2026-08-10, plano 08 §4.1, cena `=105`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9208) |
| 90 | [W-SwimLine — o nadador parado BOIA (2026-08-10, mesma cena `=105`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9332) |
| 91 | [W-ZoneForce — a CORRENTEZA leva um personagem cinemático (2026-08-10, plano 08 §4.2, cena `=106`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9411) |
| 92 | [W-ShapeCast — o wrapper varre o CORPO, não só uma linha](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9536) |
| 93 | [W-Probes — os sensores do player ficam VISÍVEIS](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9660) |
| 94 | [W-Fall — a queda tem TETO (2026-08-14, plano 10 §4, cena `=116`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9793) |
| 95 | [W-Launch — o EMPURRÃO de fora (2026-08-14, plano 10 §5, cena `=117`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9943) |
| 96 | [`W-Leave` — O QUE A PLATAFORMA DÁ AO PULO ⟨2026-08-14, o item **J** do plano 10⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10095) |
| 97 | [⬛ W-Brink — **A TRAVA DE BEIRADA** (`bCanWalkOffLedges`), item **G** da fila ⟨2026-08-15⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10240) |
| 98 | [⛔ H e I — RECUSADAS por MEDIÇÃO, e a fila da auditoria FECHOU (2026-08-15)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10383) |
| 99 | [⬛ W-WallNormal — a 2ª das três consultas de cauda, e o TETO medido como recusa ⟨2026-08-15⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10467) |
| 100 | [⬛ W-Ceiling — o teto vira um FATO ⟨2026-08-15⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10563) |
| 101 | [⬛ W-Bonked — a batida de cabeça vira EVENTO ⟨2026-08-15⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10677) |
| 102 | [⬛ W-HitNormal — o contato ganha ORIENTAÇÃO ⟨2026-08-15⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10772) |
| 103 | [⬛ W-WallMaterial — esta superfície não é parede ⟨2026-08-15⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10880) |

---

## ⛔ Recusas MEDIDAS — 31, e nenhuma volta à fila

> ⚠️ **Este doc foi cortado em 2026-08-18** e a narrativa foi **verbatim** para
> [`HANDOFF_line_physics.md`](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md) — a remontagem das duas metades confere sha256 com o original.
>
> ⛔ **Uma recusa medida é o conteúdo mais caro do repo:** ela diz *o que foi tentado, medido
> e rejeitado, com o mecanismo* — e é a única coisa que impede alguém de refazer trabalho já
> pago. Estas ficaram no arquivo; este índice existe para que continuem a existir na prática.
>
> *Antes de propor qualquer otimização ou mudança de desenho aqui, procure-a nesta tabela.*
> Linhas marcadas `§` são o próprio título da seção — as mais duras, do tipo «não refaça».

| onde | a recusa |
|---|---|
| [(topo)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10) | ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma |
| [Todo teto foi MEDIDO (`ph2d-physics/tests/measure_settings.rs`, `--rel](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L434) | ⚠️ **A hipótese óbvia do contact Hz — Nyquist em `1/(2·substep_dt)` = 120 Hz — foi REFUTADA |
| [Gates: 11 novos (5 bridge · 3 unit · 2 seam-painel · 1 seam-inspector)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L621) | mortos sob o mouse**. Agora dirige `click_at`. |
| [§](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L697) | Três números MEDIDOS, e os três primeiros palpites foram REFUTADOS |
| [⚠️ A auditoria de 2 lentes achou SEIS coisas — e as duas graves eram m](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L833) | das três listas compartilhadas — *painted, hit-registered e mortos sob o |
| [§W-LockPos — FREEZE POSITION X/Y por corpo (2026-07-20, smoke `=18`)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2004) | groups (senão os toggles pintam mas ficam mortos sob o mouse — o `architecture_panel_wiring_parity` pega). |
| [⚠️ Mesma LEI, aritmética diferente — e o gate pina o número exato](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2563) | igualdade exata e foi **rejeitada**: seria o **terceiro escritor** de um campo cujo histórico de clobber o |
| [Smoke `=26` — três meios, as mesmas três caixas](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L2604) | VÁCUO (sem zona nenhuma — uma zona vazia seria recusada pelo `zone_effect` de qualquer forma, e um retângulo |
| [§](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L3032) | A cena 30 — a DEMOLIÇÃO (o 1º corte foi recusado) |
| [(1) A winch não fazia nada, e o modelo estava certo](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L4947) | em qualquer `max_force`**, o que faz os dois knobs lerem como mortos de uma vez. |
| [O motor da Rope, lido no fonte do rapier em vez de inferido](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L5067) | gastá-lo. **Uma causa, os dois knobs mortos** — a inferência do W-J6b estava certa |
| [W-Pulley W4 — O TAMBOR DIFERENCIAL: a vantagem mecânica CONTÍNUA (2026](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6358) | s₁r₁\|`, que centros iguais nunca satisfazem ⇒ a rota inteira seria recusada e a corda **sumiria da |
| [E O SMOKE APROVOU AS ALÇAS E DERRUBOU TRÊS COISAS (2026-07-29)](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L6716) | ⚠️ **MEDIDO E REFUTADO:** *"afasta a ponta da corda dos objetos"* **não** é o |
| [As três leis da wave](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7084) | que não é polia é RECUSADO** e o pick segue armado. |
| [A investigação virou wave, e por um motivo de CUSTO](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7131) | ⚠️ **A objeção geométrica também caiu:** *"concêntricas são recusadas"* vale para |
| [§](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7160) | ⛔ NÃO há teto no peso, e a medição decidiu |
| [Os dois defeitos do smoke, que eram UM ponto](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7658) | mouse na simulação não funcionou com a shape filha"*. |
| [Os dois defeitos do smoke, que eram UM ponto](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L7661) | não está lá — o gesto era recusado **em silêncio** e o press caía adiante para o |
| [2ª rodada de smoke — a força bizarra](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8479) | ⛔ **MEDIDO E REJEITADO — não refaça:** folgar o orçamento da corda pela violação |
| [§](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8784) | ⛔ Os três que NÃO têm cura de engenharia — com a evidência |
| [W-KinMove — O SEGUNDO MODO (2026-08-08, cena `=101`, **2º RE-SMOKE pen](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L8950) | ⛔ **E o LARANJA ainda deslizava** (*"quando pousa na rampa ainda se desloca um |
| [O consumidor, e as duas notas que ele derruba](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9597) | de altura de teto recusados sobre espaço vazio. |
| [(b) EDITÁVEIS — ⚠️ a premissa desta metade estava ERRADA](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9735) | 1. ⛔ **Não existem cinco sensores, existem QUATRO.** `probe_ceiling` produz um |
| [(b) EDITÁVEIS — ⚠️ a premissa desta metade estava ERRADA](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9739) | 2. ⛔ **O alcance do headroom é DERIVADO de dois números autorados** |
| [(b) EDITÁVEIS — ⚠️ a premissa desta metade estava ERRADA](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9743) | 4. ⛔ **As CONTAGENS ficam `const`, com o custo medido ao lado** (o doc do |
| [O que a medição fez ao plano](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L9812) | ⛔ **E a prescrição de DUAS implementações está REFUTADA.** O plano escreveu |
| [O que a sonda mediu ⟨`measure_platform_leave`⟩](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10115) | REFUTADA**: a memória do `lift_momentum` (W10) funciona nos três modos, e o |
| [§](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10383) | ⛔ H e I — RECUSADAS por MEDIÇÃO, e a fila da auditoria FECHOU (2026-08-15) |
| [Superfície](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10462) | H e I recusadas com o número ao lado. O handoff de integração é o |
| [§](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10518) | ⛔ O TETO — MEDIDO e NÃO construído, com o preço nomeado |
| [⚠️ O que a tabela pedia não era o que faltava](../../archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md#L10793) | ⛔ **E é por isso que uma lista de `CharacterHit` teria sido a resposta ERRADA** — |
