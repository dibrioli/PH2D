# Síntese cruzada — ESTRUTURA & INTELIGÊNCIA (navegação · AI · percepção · spawn · prefab · tags · rede · save · pin · lifecycle)

> Domínio desta síntese: navegação (agent/obstacle/region/link, navmesh 2D) · behavior trees/utility AI/FSM de
> gameplay · line-of-sight/percepção · spawner/factory/pooling · prefabs/variants/instanciamento · tags/grupos/
> queries · rede-multiplayer (horizonte) · persistência/save de jogo · RemoteTransform/pin/anchor · lifecycle
> (on-screen enabler, auto-destroy, persistir entre cenas).
>
> Fontes: dossiês Unity/Godot/Unreal/Construct+GDevelop/GameMaker+Defold/Cocos+Phaser/Bevy + `inventario_ph2d.md`
> (2026-08-20). Decisão do dono já tomada: componentes estilo Unity (AddComponent), não herança de nodes.
> Iluminação 2D: **ADIADA** (não há item de luz neste domínio; onde um vizinho tocar luz, está marcado).
> Levantamento EXAUSTIVO — o dono decide escopo.

---

## §0 — As 7 leis transversais que os dossiês provam (valem para TODO componente abaixo)

1. **Required components (Bevy 0.15) é a UX do AddComponent resolvida.** Adicionar o conceito dirigente
   (`Spawner`) puxa as dependências (`Transform`, `Tags`…) em cascata. Sem isso, o erro "adicionei X, faltou Y,
   nada acontece" é o dia-a-dia do usuário. É pré-requisito de plataforma para o catálogo inteiro.
2. **Componente sem vocabulário é um beco (lição Construct).** Cada componente abaixo precisa publicar
   condições/eventos/ações no sistema de sinais (`ph2d-runtime`) e, futuramente, na tabela sinal→ação (R3) e no
   Luau. O painel configura; o vocabulário compõe.
3. **Marcadores são componentes** (Solid, Persist, NoSave, Replicated, Save): zero propriedades, alavancagem
   enorme, custo ~zero — o padrão mais barato do domínio (Construct/Bevy/moonshine_save convergem).
4. **Pares produtor/consumidor** (NavAgent↔NavObstacle; SightSense↔PerceptionSource; Spawner↔PrefabAsset):
   o behavior no agente CONSOME o marcador no cenário — responsabilidade nunca colide porque cada metade mora
   num objeto diferente.
5. **Referência durável é o NOME** — o PH2D já tem a resposta certa (`stable_name_id`, hash do `Name`): todo
   campo "alvo/prefab/caminho" dos componentes abaixo referencia por nome, nunca por `Entity::to_bits()`.
6. **Determinismo é lei da casa:** `BTreeMap`/`BTreeSet` sempre (HR da física); Spawner/AI com seed explícita —
   o ring GGPO e o hash 3-OS já pagam esse preço, os componentes novos não podem quebrá-lo.
7. **O custo real é a UI (passo 5 do inventário):** registro→persistência→undo são de graça; a seção artesanal
   no Inspector é o gargalo. Um derive/DSL de painel a partir do tipo (o `@export` do Godot, o `@property` do
   Cocos, o `go.property` do Defold) multiplica TODO o catálogo — investir nisso antes de escalar o cardápio.

Dependência transversal citada em quase tudo: **a tabela sinal→ação (R3)** — sinais já viajam (contato, marker
de timeline); nada autorável reage. Vários componentes abaixo são produtores (SightSense, Spawner `on_spawned`,
OnScreenNotifier) ou consumidores (Spawner `spawn_on_signal`) dessa tabela. Ela pertence ao domínio lógica/
eventos, mas é o chão deste aqui.

---

## §1 — Prefabs, variants e instanciamento

### `PrefabAsset` (+ fluxo de editor: salvar seleção → browser → instanciar)
- **Entrega:** o usuário salva qualquer subárvore da cena como template vivo, arrasta de volta N vezes, edita o
  template e TODAS as instâncias atualizam. É o multiplicador de todo o resto do editor — sem ele, cada inimigo
  é remontado à mão.
- **Equivalentes:** Unity Prefab (nested+variants, o padrão-ouro) · Godot PackedScene · Unreal Blueprint Class ·
  Construct hierarchies+`Create object with hierarchy` · GDevelop Custom Objects (prefab COM eventos próprios) ·
  GameMaker Object→Instance · Defold Collection (arquivo-protótipo, instâncias referenciam) · Cocos Prefab
  (modo de edição, nested, overrides com revert/apply/unlink) · Phaser Editor prefabs (fora do core) · Bevy
  `bevy_ecs_ldtk` `register_ldtk_entity` (entidade autorada → componentes por identificador).
- **PH2D hoje:** **parcial** — `PrefabDoc`/`SceneDoc` (`ph2d-asset` + `ph2d-ecs::scene::spawn`, postcard
  versionado, `ComponentTypeId = blake3`) e `spawn_prefab` existem; **zero UI** (sem "salvar como prefab", sem
  browser, sem instanciar por drag).
- **Prioridade:** **P0** — espinha de conteúdo; tudo do §2 depende dele.
- **Dependências:** nenhuma técnica (a base está pronta); o fluxo de editor é o trabalho.
- **Nota de desenho:** o modelo Cocos (duplo-clique abre modo de edição do prefab; instâncias marcadas; override
  por campo com revert/apply) é o mais copiável. O contrato LDtk→Bevy ("identificador → bundle de componentes")
  já é literalmente o desenho do `PrefabDoc`.

### `PrefabRef` (instância aninhada como componente)
- **Entrega:** um componente que diz "aqui dentro vive uma instância do prefab X" — inimigo = corpo + arma +
  sensores num só gesto, com a hierarquia preservada. Prefab dentro de prefab sem cópia.
- **Equivalentes:** Unreal ChildActorComponent · Unity nested prefabs · Cocos nested prefabs · Defold Collection
  Factory (spawna hierarquia inteira, devolve mapa id-local→id-runtime, properties POR objeto spawnado) · Godot
  cena instanciada dentro de cena.
- **PH2D hoje:** **parcial** — `PrefabDoc` já é "bundle de components + filhos"; falta o componente-referência e
  o override por instância.
- **Prioridade:** **P0** (par do anterior).
- **Dependências:** `PrefabAsset`.
- **Nota:** o detalhe Defold a copiar: no spawn, aceitar tabela de overrides por objeto interno
  (`props["/arma"] = {...}`).

### `PrefabVariant` (variante com herança de overrides)
- **Entrega:** "GoblinArqueiro = Goblin + estes 4 campos diferentes". Retheming e famílias de inimigos sem
  duplicar o template; mudar o pai propaga ao que a variante não sobrescreveu.
- **Equivalentes:** Unity Prefab Variants · Unreal Blueprint child class · Cocos overrides por instância ·
  Godot herança de cena · (Rule Override Tile da Unity é o mesmo padrão em tiles).
- **PH2D hoje:** **não**.
- **Prioridade:** **P1**.
- **Dependências:** `PrefabAsset` + o diff de override (o motor de diff do undo já sabe comparar `ProjectState`
  — reuso provável).

### `SpawnPoint` (marcador nomeado com gizmo)
- **Entrega:** posição nomeada visível no editor (cruz/seta), consultável por nome/tag: spawn do player, waypoint,
  boca da arma. Mata as constantes de posição hardcoded.
- **Equivalentes:** Godot Marker2D (`gizmo_extents`) · camada de objetos do Tiled (Phaser `createFromObjects` —
  o nível se popula sozinho) · LDtk entities (Bevy) · GM instância na room.
- **PH2D hoje:** **parcial** — `Name` + `Transform` + `stable_name_id` já dão a semântica; falta o marker
  dedicado (gizmo, categoria, consulta "todos os SpawnPoint com tag X").
- **Prioridade:** **P0** (custo ~zero, uso universal; par do Spawner).
- **Dependências:** `Tags` (§3) para consulta por grupo.

---

## §2 — Spawner / factory / pooling / lifecycle de instância

### `Spawner` (a fábrica como componente)
- **Entrega:** "o quê (prefab) · onde (este ponto / área / ao longo de um caminho / num SpawnPoint por tag) ·
  quando (taxa, burst, ondas com intervalo, ao receber sinal X) · quanto (limite vivo, total)". O padrão
  inimigo/moeda/projétil inteiro sem uma linha — a categoria que QUASE NENHUMA engine grande tem como
  componente (só as no-code), e por isso diferencial direto.
- **Equivalentes:** GDevelop Object spawner (extensão revisada) · Defold Factory (`Prototype`, load dinâmico,
  troca de protótipo em runtime) + Collection Factory · GameMaker: só código (`instance_create`) · Unity: SEM
  componente (só Particle System spawna por UI) · Godot: SEM (idioma `instantiate()+add_child`) · Unreal: SEM
  (SpawnActor) · Construct: SEM (`System: Create object`) · Phaser `Group.createMultiple` + timers.
- **PH2D hoje:** **não** — mas as sementes são fortes: `spawn_prefab`, `SpawnQueue` do Luau, `SignalReader`.
- **Prioridade:** **P0** — sem instanciar em runtime não há jogo dinâmico.
- **Dependências:** `PrefabAsset`/`PrefabRef` (o quê) · `SpawnPoint`/`Tags` (onde) · sinais (quando —
  `spawn_on_signal` é a metade que conecta com física/timeline hoje, antes mesmo do R3).
- **Nota de desenho:** (a) seed explícita por spawner (determinismo, lei §0.6); (b) publica vocabulário:
  `on_spawned(entity)`, `alive_count`, `exhausted`; (c) o burst "Particle-style" (rate contínua vs explosão) é
  o modelo de UI já validado 3× nos dossiês (GM particle emitter, Godot `explosiveness`, Phaser `frequency=-1`).

### `ObjectPool` (política de reciclagem)
- **Entrega:** o Spawner recicla em vez de criar/destruir: `max_size`, pré-aquecer N, o que resetar ao reusar.
  O usuário nunca escreve pooling — e o caso patológico (balas alocando todo frame) morre por default.
- **Equivalentes:** Phaser Group (`get()`/`killAndHide()`/`maxSize` — pooling como 1ª classe) · Defold interno
  (a doc PROÍBE pooling manual) · Unity `ObjectPool<T>` (API, sem UI) · Cocos NodePool (API).
- **PH2D hoje:** **não**.
- **Prioridade:** **P1**.
- **Dependências:** `Spawner` (é uma propriedade dele ou componente irmão).
- **Nota:** a decisão Defold ("a engine faz o pool; o usuário não") é a mais alinhada com §0 do CLAUDE.md —
  medir primeiro se o pool é sequer necessário com o ECS atual antes de expor knobs.

### `Lifetime` (TTL / autodestruição)
- **Entrega:** "viva N segundos e morra" (opcionalmente com fade), ou "morra quando a animação/efeito acabar".
  O ciclo de vida de projétil, popup de dano, poeira — o vazamento nº 1 de iniciante, resolvido por 1 campo.
- **Equivalentes:** Construct Fade (in→wait→out + Destroy) · Unity TrailRenderer `AutoDestruct` · Bevy
  `PlaybackSettings::Despawn` (áudio fire-and-forget) · Godot `one_shot`+sinal `finished` (idioma) · Phaser
  particle `lifespan`.
- **PH2D hoje:** **não**.
- **Prioridade:** **P0** (trivial de fazer, uso universal).
- **Dependências:** nenhuma (sinerge com o tween/fade do domínio de animação).

### `DestroyOutside` (higiene de fora-da-tela/fora-do-mundo)
- **Entrega:** destrói (ou devolve ao pool) ao sair do viewport/limites do mundo, com margem. Balas e inimigos
  não se acumulam fora da tela — 1 checkbox.
- **Equivalentes:** Construct Destroy outside · GDevelop Destroy when outside of the screen · Phaser
  `collideWorldBounds` + evento `worldbounds` · Godot VisibleOnScreenNotifier2D + `queue_free` (idioma).
- **PH2D hoje:** **não** (o primo `OnScreenEnabler` existe — §5).
- **Prioridade:** **P0**.
- **Dependências:** câmera de gameplay (domínio vizinho) para "viewport"; limites de mundo como fallback.

---

## §3 — Tags, grupos e queries

### `Tags` (multi-pertencimento + consulta + broadcast)
- **Entrega:** o objeto pertence a N grupos por checkbox ("enemies", "coletavel", "fase2"); qualquer sistema/
  componente consulta ("todos os enemies") ou difunde ("sinal para o grupo guards"). Mata as listas manuais de
  referências — o código de bookkeeping mais reescrito do gamedev.
- **Equivalentes:** Godot Groups (`get_nodes_in_group`, `call_group`) · Construct Families (tag é TRAIT:
  behavior/variável posto na família vale para todos — a versão mais poderosa) · Unreal Gameplay Tags
  (**hierárquicos**: `Damage.Fire`, comparação por prefixo) + Actor Tags · GameMaker parent-como-grupo (colisão
  contra o pai pega os filhos) · Unity Tags (1 só!) + Layers · Phaser Groups não-exclusivos · Bevy marker
  components (idioma).
- **PH2D hoje:** **não** (existem `Name`/`stable_name_id` e `VisibilityLayer`, que são outra coisa).
- **Prioridade:** **P0** — é a moeda de troca de TODO o resto (Spawner spawna "num SpawnPoint tag=X",
  SightSense vê "tag=player", colisão filtra, save filtra).
- **Dependências:** nenhuma. `BTreeSet<TagId>` (interning de string, determinístico).
- **Nota de desenho:** adotar o formato **hierárquico** do Unreal desde o dia 1 (`enemy.flying.boss` casa com
  `enemy.*`) custa quase nada e evita a migração; o `MessageBus` do Luau já faz interning de nomes — reusar.
  ⚠️ A lição Construct "família carrega traits" é a etapa 2 (família = tag + preset de componentes) — anotar,
  não construir agora.

### `Team` (afiliação: amigo/neutro/hostil)
- **Entrega:** dropdown de time por objeto + tabela times×atitude no projeto. Percepção ("detectar só Enemies"),
  dano ("fogo amigo?"), turret ("quem é alvo") filtram por ela sem código.
- **Equivalentes:** Unreal AI Perception `Detection by Affiliation` + Team sense · Construct Solid com filtro de
  tags (primo) · demais: ausente (resolvem com tag crua).
- **PH2D hoje:** **não**.
- **Prioridade:** **P1** (pequeno; par obrigatório da percepção §6).
- **Dependências:** `Tags` (pode ser uma tag especial) ou componente próprio — decidir com a percepção.

---

## §4 — Pin / anchor / remote transform

### `PinTo` (prender sem reparentar, canal a canal)
- **Entrega:** gruda este objeto noutro mantendo offset — posição e/ou ângulo e/ou escala/opacidade por
  CHECKBOX de canal — sem mudar a hierarquia (nada de bugs de reparenting no undo). Arma na mão, barra de vida
  na cabeça, escudo orbitando.
- **Equivalentes:** Construct Pin (com **Pin to image point** — segue um ponto nomeado DENTRO da animação do
  pai) e hierarchies com herança por canal · GDevelop Sticker (+destroy junto) · Unity Parent/Position/Rotation
  Constraints (multi-fonte com PESO) · Godot RemoteTransform2D (o mesmo cabo, direção inversa) · Cocos Spine
  Sockets (node preso a osso) · Defold `spine.get_go` (osso é game object).
- **PH2D hoje:** **não** (hierarquia + `stable_name_id` dão a base; `JointWorldAnchor` é o primo físico).
- **Prioridade:** **P1**.
- **Dependências:** ordem de avaliação clara no frame (a lição Unity Constraints: rodar "na ordem certa do
  pipeline" é o que o script de usuário sempre erra — aqui é um system com posição fixa no schedule).
- **Nota de desenho:** incluir o modo **push** (RemoteTransform do Godot: EU empurro meu transform para o alvo)
  como um enum `direction: Pull|Push` do MESMO componente — duas engines, um componente. "Pin to image point"
  vira "pin a um `SpawnPoint`/osso filho do alvo" — já expressável por nome.
- **Risco:** com `PinTo` + física no mesmo objeto há dois donos do transform — aplicar a regra da casa
  documentada no Construct: um dono por vez (gate).

### `FollowHistory` (seguir com atraso: trenzinho, sombra, replay/ghost)
- **Entrega:** segue outro objeto reproduzindo o HISTÓRICO dele por tempo ou distância; canais por checkbox;
  o histórico é serializável (ghost de corrida salvo em arquivo). Ring buffer + interpolação + replay prontos.
- **Equivalentes:** Construct Follow (**única** — e excelente: Follow self = replay, Load history JSON).
- **PH2D hoje:** **não** — mas o `TapeWire` da física (corrida gravada → bake) é primo direto; o mecanismo de
  gravação já existe.
- **Prioridade:** **P2** (nicho; barato dado o TapeWire).
- **Dependências:** nenhuma dura.

### `ViewportAnchor` (ancorar às bordas da tela)
- **Entrega:** HUD-no-mundo que sobrevive a resize: prende bordas do objeto às bordas do viewport (ancorar
  bordas opostas ESTICA).
- **Equivalentes:** Construct Anchor · GDevelop Anchor (window side/center/**proporcional**) · Cocos Widget
  (px/% — o mais rico) · Unity RectTransform · GM UI Layers/Flex Panels.
- **PH2D hoje:** **não** como componente de cena (o `VecLayout*`/taffy e `VecAnchors` do Vector são a
  infraestrutura irmã do lado autorado).
- **Prioridade:** **P2 neste domínio** — a decisão pertence ao domínio UI in-game (provável que a resposta
  certa seja o auto-layout taffy já existente); listado aqui só porque "anchor" cruza com pin.

---

## §5 — Lifecycle: on-screen, ativação, persistência entre cenas, mundos

### `OnScreenEnabler` — **JÁ EXISTE** (`ph2d-ecs`)
- **Entrega:** desliga processamento fora da tela, religa ao entrar ("inimigos acordam quando o player chega").
- **Equivalentes:** Godot VisibleOnScreenEnabler2D (o nome e o modelo).
- **PH2D hoje:** **sim** (`OnScreenEnabler`, registrado, com UI §visibility). Conferir contra o Godot o que
  falta: rect custom, `enable_mode`, e O QUE exatamente ele desliga (render? sistemas? física?) — a semântica
  "quais sistemas pausam" precisa estar escrita quando houver gameplay.
- **Prioridade:** — (feito; a extensão é P1 junto do Notifier).

### `OnScreenNotifier` (a metade evento do par)
- **Entrega:** sinais `screen_entered`/`screen_exited` + condição `is_on_screen` — despawn, ativar spawner,
  "só atira quando visível".
- **Equivalentes:** Godot VisibleOnScreenNotifier2D · Construct "Is on-screen" · GDevelop Is on screen ·
  Phaser `worldbounds`.
- **PH2D hoje:** **parcial** (o Enabler é o irmão mudo — falta publicar o evento no `SignalOutbox`).
- **Prioridade:** **P1** (barato: é dar voz ao que já existe).
- **Dependências:** `ph2d-runtime` (produtor de sinal).

### `ProximityActivator` (acordar/dormir por distância)
- **Entrega:** generalização do Enabler: ativa/desativa o alvo pela distância a um objeto/tag (não pela tela) —
  "a caverna só simula quando o player está a 30 m". Culling de LÓGICA dirigido por gameplay.
- **Equivalentes:** nenhuma engine entrega como componente (idioma de código em todas) — oportunidade barata.
- **PH2D hoje:** **não**.
- **Prioridade:** **P2**.
- **Dependências:** `Tags` (o "perto de quem").

### `Persistent` (sobrevive à troca de cena / a cena lembra)
- **Entrega:** dois marcadores: (a) o objeto ATRAVESSA a troca de cena sem ser recriado (player, música);
  (b) a instância LEMBRA seu estado ao revisitar a cena (baú aberto continua aberto). Persistência de mundo em
  1 clique.
- **Equivalentes:** Construct Persist (o modelo (b), zero propriedades) · GameMaker Persistent no Object (a) e
  na Room (b) · GDevelop Save state `Persisted` por objeto.
- **PH2D hoje:** **não** — e o pré-requisito (fluxo de cenas em play mode) é o **R1, adiado por decisão do
  Enio**. A infraestrutura de captura (snapshot canônico por entidade) já existe.
- **Prioridade:** **P1** (implementável junto do R1; o marcador em si é trivial).
- **Dependências:** R1 (shell de jogo / troca de cena) · `ComponentRegistry` (já dá a serialização).

### `WorldRef` / proxy de mundo carregável (com relógio próprio)
- **Entrega:** carrega/descarrega outra cena como MUNDO separado (memória e física próprias) e — o ouro do
  Defold — `time_step` por mundo: pause com o menu vivo, slow-motion, fast-forward de UM mundo sem tocar no
  resto. Streaming de fases + bullet-time por mensagem.
- **Equivalentes:** Defold Collection Proxy (+`set_time_step`) — o desenho de referência · Phaser Scenes
  paralelas (launch/sleep/wake, plugins por cena) · Unity additive scenes (API) · Godot SubViewport (parcial).
- **PH2D hoje:** **não** — mas a timeline já provou o princípio "o pai é dono do relógio" (nesting, ADR-0133),
  e a física já tem mundo/checkpoint isolados. Conceitualmente alinhado.
- **Prioridade:** **P1** (com dependência dura do R1).
- **Dependências:** R1 · decisão de escopo "1 mundo com N cenas vs N mundos".
- **Risco:** é o componente mais estrutural da lista — errar aqui contamina save, física e áudio. Merece ADR
  próprio antes de qualquer código.

---

## §6 — Percepção (sight / hearing / raycast declarativo)

### `RaySensor` (ray/shapecast persistente como componente)
- **Entrega:** a entidade CARREGA um raio (ou forma varrida) que atualiza sozinho todo frame, com seta visível
  no editor: `is_colliding`, ponto, normal, **reflexão** (laser que ricocheteia em 3 leituras). Sensor de chão,
  "tem parede à frente?", mira — sem API imperativa.
- **Equivalentes:** Godot RayCast2D/ShapeCast2D (componentes, gizmo no viewport) · avian `RayCaster`/
  `ShapeCaster` (o idioma ECS exato) · Construct LOS `Cast ray` (+ expressions Normal/Reflection) · Unity/UE/GM:
  só API.
- **PH2D hoje:** **não** (rapier tem as queries; o módulo `sense` do `ph2d-platformer` é o primo interno).
- **Prioridade:** **P1** — é a peça de baixo de TODA a percepção e de metade dos controllers.
- **Dependências:** `ph2d-physics-ecs` (camadas de colisão já existem).

### `SightSense` (visão: cone + alcance + oclusão + memória)
- **Entrega:** "eu vejo o alvo?" com raio, cone em graus, obstáculos por camada, **memória com esquecimento**
  (Max Age) e filtro por afiliação — publicando sinais `target_spotted`/`target_lost` no outbox. O coração de
  stealth/combate por dropdown; visualizador do cone no editor.
- **Equivalentes:** Unreal AI Perception Sight (SightRadius, LoseSightRadius, PeripheralVisionHalfAngle,
  memória, afiliação — o modelo completo) · Construct Line of sight (range+cone+obstáculos como behavior) ·
  Godot/Unity: lacuna declarada (compõe-se à mão) · GDevelop: não-core.
- **PH2D hoje:** **não** — mas o padrão já existe na casa: `SignalOnHit` prova "condição física → sinal
  nomeado"; SightSense é o mesmo contrato com outra condição.
- **Prioridade:** **P1** (a metade de toda IA 2D).
- **Dependências:** `RaySensor` (oclusão) · `Tags`/`Team` (quem conta como alvo) · `ph2d-runtime` (sinais).
- **Nota de desenho:** o "LoseSightRadius > SightRadius" do Unreal (histerese) e o Max Age (esquecer) são os
  dois campos que separam percepção de brinquedo de percepção de produto — incluir desde a v1.

### `HearingSense` + `NoiseEmitter` (o par barulho/ouvido)
- **Entrega:** `NoiseEmitter` publica um evento de ruído (raio, força, tag) quando mandado — passo, tiro,
  vidro quebrando; `HearingSense` ouve dentro do alcance (sem oclusão ou com atenuação) e emite
  `noise_heard(pos)`. O guarda que investiga o barulho, por UI.
- **Equivalentes:** Unreal Hearing + "Report Noise Event" + task Make Noise (único completo) · demais: ausente.
- **PH2D hoje:** **não** — o `SignalOutbox` é EXATAMENTE o substrato (ruído = sinal com origem espacial;
  `SignalOrigin` já existe).
- **Prioridade:** **P2** (depois do Sight; a infraestrutura é a mesma).
- **Dependências:** `SightSense` (compartilham o registro de percepção) · `ph2d-runtime`.

### `PerceptionSource` (marcador: o que pode ser percebido)
- **Entrega:** o alvo declara para QUAIS sentidos é detectável (visível, audível, "cheirável" custom) — o par
  produtor do §0.4. Sem ele, todo sense varre o mundo inteiro.
- **Equivalentes:** Unreal AIPerceptionStimuliSourceComponent (Auto Register + sentidos).
- **PH2D hoje:** **não**.
- **Prioridade:** **P1** (nasce junto do SightSense).
- **Dependências:** `SightSense`.

---

## §7 — Cérebros: FSM, behavior tree, utility, EQS

### `StateMachine` (FSM de GAMEPLAY como componente, com editor visual)
- **Entrega:** estados + transições com condições (sinal chegou, timer venceu, tag entrou no alcance,
  expressão) + ações on_enter/on_exit (trocar animação, emitir sinal, ligar/desligar componentes) — a tabela
  estado×gatilho×destino 100% editável por UI. Substitui o `match` gigante que todo jogo acumula — e no PH2D,
  onde o script ainda NÃO tem UI, é o cérebro mínimo autorável.
- **Equivalentes:** Bevy `seldom_state` (**estado-É-componente**: outros sistemas filtram por estado —
  `Query<&T, With<Airborne>>`; 30 triggers; o desenho de referência para ECS) · Unity Visual Scripting State
  Machine (+ Animator para animação) · Unreal PaperZD (anim) + BT · Construct Flowcharts · Godot: LACUNA
  declarada (LimboAI/Beehave addons) · GDevelop: não-revisado.
- **PH2D hoje:** **parcial-embrião** — `ph2d-ui-state::Machine` (Smart Animate: estados de cena com tween
  automático, sem relógio próprio) e o módulo `event`/`PlayerEvent` do platformer são os dois primos. Nenhum é
  FSM de gameplay autorável.
- **Prioridade:** **P0** — pelo critério estrito seria P1 (nenhuma engine o tem no core e jogos existem), mas
  no PH2D o único caminho de lógica hoje é Luau SEM UI: sem um cérebro autorável, o artista não faz gameplay
  nenhum. É o componente de maior alavancagem do domínio.
- **Dependências:** vocabulário de sinais (gatilhos = sinais/condições dos outros componentes — lei §0.2);
  ações on_enter precisam da família de ações do R3.
- **Nota de desenho:** adotar "estado-como-componente" do seldom_state: o estado ativo é um componente
  inserido/removido pela máquina ⇒ QUALQUER sistema (animação, física, spawner) reage a estado por query, sem
  acoplamento. ⚠️ Lição Godot (VisualScript removido no 4.0): NÃO construir visual scripting genérico de fluxo
  de controle — FSM declarativa + grafos de domínio (Motion Nodes) é o caminho validado.

### `BehaviorTree` + `Blackboard` (árvore de decisão visual)
- **Entrega:** patrulha→persegue→ataca→foge como árvore clicável: composites (Selector/Sequence/Parallel),
  tasks prontas (MoveTo via NavAgent, Wait, EmitSignal, PlayAnimation, sub-árvore), decorators (cooldown,
  distância, "existe caminho?", tag presente), services periódicos; `Blackboard` = memória chave-valor da IA
  que os nós leem/escrevem.
- **Equivalentes:** Unreal Behavior Trees + Blackboard (o modelo canônico: a lista de tasks/decorators/services
  embutidos do dossiê é a spec) · Godot: addons · Unity: "Unity Behavior" recente (não consolidado) · demais:
  ausente em TODAS — categoria sem dono fora da UE.
- **PH2D hoje:** **não** — mas `ph2d-nodegraph` (motor de grafo com editor completo: palette, splice, bypass,
  grupos) é uma vantagem estrutural rara: o EDITOR de árvore já está meio construído.
- **Prioridade:** **P1** (diferencial forte; a FSM cobre 70% dos casos antes).
- **Dependências:** `StateMachine` primeiro (compartilham gatilhos/ações) · `NavAgent` (a task MoveTo é o que dá
  vida à árvore) · `Blackboard` pode nascer como recurso compartilhado FSM↔BT.
- **Nota:** decisão explícita a tomar: BT sobre o `ph2d-nodegraph` existente (reuso do editor, mas o contrato
  Nodes está CONGELADO — §6 do CLAUDE.md, mexer = ADR) ou crate-folha própria com editor próprio. Cheira a ADR.

### `UtilityBrain` (utility AI: scorers + pesos + picker)
- **Entrega:** NPC que "decide o que mais vale a pena": medições (fome, distância, vida) viram scores 0..1,
  pesos em SLIDERS, o picker escolhe a ação. Comportamento emergente sem árvore de ifs.
- **Equivalentes:** Bevy big-brain (Thinker/Scorers/Pickers/Actions com máquina Requested→Executing→
  Success/Failure) — único no mercado dos dossiês.
- **PH2D hoje:** **não**.
- **Prioridade:** **P2** (depois de FSM+BT; público mais avançado).
- **Dependências:** mesma família de gatilhos/medições da FSM.

### `TacticalQuery` (EQS-lite: "ache o melhor ponto")
- **Entrega:** consultas espaciais data-driven: gera candidatos (grade, círculo, SpawnPoints por tag), filtra e
  pontua (distância, linha de visão, custo de caminho), devolve o melhor — cobertura, flanco, ponto de fuga,
  spawn justo. Com visualizador no editor.
- **Equivalentes:** Unreal EQS (Generators+Tests+Contexts, único).
- **PH2D hoje:** **não** (`ph2d-grid` dá a matemática de vizinhança).
- **Prioridade:** **P2**.
- **Dependências:** `RaySensor` (teste de LOS) · `NavSurface` (teste "existe caminho") · `Blackboard` (destino
  do resultado).

---

## §8 — Navegação 2D

### `NavSurface` (a superfície navegável: grade E navmesh)
- **Entrega:** define ONDE se anda — modo grade (células, derivada do tilemap/colisores, custo por célula) e
  modo navmesh (polígono baked dos colisores, regiões que se COSTURAM automaticamente). Rebuild automático
  quando obstáculos mudam.
- **Equivalentes:** Godot NavigationRegion2D (baking + costura automática) · Unity NavMeshSurface (**3D-only —
  lacuna 2D notória**) · Unreal NavMeshBoundsVolume · GDevelop NavMesh floor (auto-gerado dos obstáculos) e
  Pathfinding grid · Construct Pathfinding (grade derivada dos Solids, custos, async) · GameMaker mp_grid ·
  Bevy vleue_navigator (**obstáculo É o collider; mesh regenera sozinho; Polyanya**).
- **PH2D hoje:** **parcial** — `ph2d-grid`: 11 tipos de grid + **A\* determinístico pronto** (BTreeMap, HR-5),
  MAS custo uniforme e só o editor consome. A metade grade está a ~30% construída.
- **Prioridade:** **P1** (grade primeiro — o A* já existe; navmesh depois).
- **Dependências:** colisores como fonte (ph2d-physics-ecs) · tilemap (domínio vizinho) como fonte natural.
- **Nota de desenho:** a dupla do vleue_navigator (obstáculo = collider + auto-rebuild) é o estado da arte 2D e
  elimina a categoria inteira de "sincronizar colisão↔navegação". Unity não ter isso em 2D é a oportunidade
  nomeada em dois dossiês.

### `NavAgent` (o agente que se move sozinho)
- **Entrega:** `target_position` entra, próximo passo sai — A\*/funnel + repath + **desvio recíproco entre
  agentes (RVO)** automáticos: raio, velocidade máxima, distância de chegada, camadas de navegação. "Clique →
  o inimigo te acha" — meses de trabalho num componente.
- **Equivalentes:** Godot NavigationAgent2D (RVO, o modelo 2D) · Unity NavMeshAgent (3D; priority avoidance) ·
  Unreal (CharacterMovement `bUseRVOAvoidance` + DetourCrowd) · GDevelop NavMesh character (crowd avoidance:
  sight range + radius) · Construct Pathfinding behavior (acha E anda, com `Direct movement` anti-ziguezague) ·
  GameMaker mp_grid→Path (devolve um path que a instância segue).
- **PH2D hoje:** **não** (o A\* de `ph2d-grid` não tem consumidor de gameplay — lacuna 11 do inventário).
- **Prioridade:** **P1**.
- **Dependências:** `NavSurface` · quem EXECUTA o movimento é o domínio de movimento (o NavAgent produz o
  próximo ponto; a lição Construct Move To "três fontes de trajetória, um executor" vale aqui — não duplicar
  o motor de movimento dentro do agente).
- **Nota:** eventos publicados: `path_found`/`path_failed`/`arrived` (Construct) — viram sinais no outbox e
  gatilhos da FSM/BT de graça.

### `NavObstacle` (obstáculo dinâmico)
- **Entrega:** marca um objeto móvel como obstáculo: agentes desviam (avoidance local) e/ou o mesh/grade é
  re-esculpido (carve) quando ele para. A caixa empurrada bloqueia o caminho sem replan manual.
- **Equivalentes:** Godot NavigationObstacle2D · Unity NavMeshObstacle (carve) · Unreal NavModifier · GDevelop
  Pathfinding obstacle (com **Cost** — pântano custa 4×) · Bevy NavMeshObstacle/collider.
- **PH2D hoje:** **não**.
- **Prioridade:** **P1** (nasce com a superfície — o par §0.4).
- **Dependências:** `NavSurface`.

### `NavLink` (atalhos: pulo, teleporte, escada)
- **Entrega:** conecta dois pontos não-contíguos da superfície; o path passa por ali e o jogo anima a travessia
  (sinal `link_reached` para a FSM tocar o pulo).
- **Equivalentes:** Godot NavigationLink2D · Unity NavMeshLink · Unreal NavLinkProxy · GDevelop Link path
  finding (grafo de waypoints ligados — a variante "só links" é interessante para plataforma).
- **PH2D hoje:** **não**.
- **Prioridade:** **P2**.
- **Dependências:** `NavSurface` + `NavAgent`.
- **Nota:** para PLATFORMER, o grafo de links (pulos possíveis entre plataformas) é mais certo que navmesh —
  anotar como modo do NavSurface, não como sistema separado.

### `NavCostArea` (custo por região)
- **Entrega:** região que multiplica o custo (lama anda devagar, lava proibida) — o A\* passa a preferir a
  estrada. Hoje o `ph2d-grid` é custo-uniforme; este é o degrau que falta nele.
- **Equivalentes:** Unity NavMeshModifier (área/custo) · Unreal NavModifierVolume · Construct `Set move cost`/
  `Add path cost` por região · GDevelop Cost no obstáculo · Godot navigation layers.
- **PH2D hoje:** **parcial** (A\* existe; custo não).
- **Prioridade:** **P2**.
- **Dependências:** `NavSurface`.

---

## §9 — Persistência / save de JOGO (player-facing)

### `SaveGame` (slots de save de runtime)
- **Entrega:** salvar/carregar O JOGO (não o projeto): slots nomeados, async (sem travar frame), screenshot do
  slot, versão. O jogador salva; o artista não escreve serialização.
- **Equivalentes:** GDevelop Save state (**salva TUDO** — objetos, variáveis, timers, sons — em slots com 1
  action; o teto da categoria) · Unreal USaveGame (slots + async; seleção manual do que salvar) · Construct
  System save/load · Bevy moonshine_save (tese **model/view**: só o estado marcado salva; o visual se
  reconstrói) · Unity: só PlayerPrefs (lacuna) · Godot/GM/Defold/Cocos/Phaser: código.
- **PH2D hoje:** **parcial-infra** — o `ProjectState` snapshot canônico + `ComponentRegistry` (91 tipos) É uma
  máquina de serializar mundo pronta e provada (undo+save de autoria); o ring GGPO já captura estado de física
  por tick. Falta o RECORTE "runtime save" (o que entra, slots, formato estável de jogo lançado).
- **Prioridade:** **P1**.
- **Dependências:** R1 (play mode — save de jogo pressupõe jogo rodando) · `SavePolicy` abaixo.
- **Nota de desenho:** a tese moonshine (model salvável ≠ view reconstruível) espelha exatamente a decisão
  PH2D "components de CONFIG, nunca estado vivo de solver" — o save de jogo é o snapshot dos configs + o
  estado autorado marcado, e o resto se recozinha. ⚠️ Formato de save de jogo LANÇADO precisa de compat
  garantida além do `PROJECT_SCHEMA` do editor — degrau próprio.

### `SavePolicy` (o que persiste, por objeto)
- **Entrega:** dois marcadores + um perfil: `Persist` (entra no save), `NoSave` (cenário estático fora — saves
  menores), e perfis nomeados (checkpoint leve vs save completo). O opt-out é ele mesmo um componente.
- **Equivalentes:** Construct Persist/No save (marcadores puros) · GDevelop Save state configuration
  (Persisted/Do not save + **perfis nomeados**) · moonshine_save `Save` marker + filtros allow/block.
- **PH2D hoje:** **não** (o registry é o gancho natural).
- **Prioridade:** **P1** (nasce com o SaveGame).
- **Dependências:** `SaveGame`.

---

## §10 — Rede / multiplayer — **HORIZONTE** (a PH2D não tem rede; `ph2d-net` vazio, M13 por demanda)

> Listado por completude e para que as decisões de HOJE não fechem portas. O trunfo já existente: determinismo
> levado a sério (BTreeMap por lei, hash `physics_ecs_c9` na matriz 3-OS, ring GGPO de checkpoints) — é a
> fundação de ROLLBACK que nenhuma das engines pesquisadas tinha de graça.

### `Replicated` (identidade de rede + marcação por tipo)
- **Entrega:** checkbox "este objeto existe nos outros peers"; spawn/despawn replicam sozinhos; por TIPO de
  componente, marcar "replica".
- **Equivalentes:** Unreal `Replicates` (checkbox) + RepNotify · Unity NetworkObject · Godot MultiplayerSpawner
  (lista de cenas spawnáveis) · Bevy replicon (`Replicated` + `app.replicate::<C>()` — o desenho ECS) ·
  GDevelop Multiplayer object (o mais agressivo: sync automático de tudo + lobbies).
- **PH2D hoje:** **não**. **Prioridade: P2 (horizonte).**

### `NetworkTransformSync` / `NetworkStateSync`
- **Entrega:** sincroniza pose/estado com interpolação, thresholds, autoridade (server/owner), interesse por
  peer.
- **Equivalentes:** Unity NetworkTransform/NetworkAnimator/NetworkRigidbody2D · Godot MultiplayerSynchronizer
  (**UI de replicação**: lista de propriedades marcadas — o modelo de UX a copiar) · GDevelop (automático).
- **PH2D hoje:** **não**. **Prioridade: P2 (horizonte).**

### `RollbackManaged` (o caminho DIFERENCIADO do PH2D)
- **Entrega:** marca o objeto como "estado rolável": save/rollback/prediction automáticos no modelo GGPO.
- **Equivalentes:** GameMaker Rollback (flag `Managed` + 2 chamadas, sem servidor próprio — a referência de
  "multiplayer como propriedade do objeto").
- **PH2D hoje:** **semente rara** — o ring GGPO + determinismo 3-OS já são a metade difícil; o que falta é
  transporte + input delay + a marcação. Quando rede entrar no escopo, ROLLBACK (e não replicação genérica) é
  onde o PH2D já está à frente. **Prioridade: P2 (horizonte), mas a PRIMEIRA opção quando chegar a hora.**

---

## §11 — Ordem de construção sugerida (dependências medidas nos itens acima)

```
Onda A (fundações, tudo P0, sem dependência externa):
  Tags → PrefabAsset+PrefabRef+SpawnPoint (UI sobre base pronta) → Spawner + Lifetime + DestroyOutside
  [em paralelo: o derive/DSL de painel do §0.7 — multiplica tudo o que vem depois]

Onda B (o cérebro mínimo + os olhos):
  StateMachine (P0) ← gatilhos vêm dos sinais existentes (SignalOnHit, markers de timeline, OnScreenNotifier)
  RaySensor → SightSense + PerceptionSource + Team
  OnScreenNotifier (dar voz ao Enabler)

Onda C (navegação — o A* já espera consumidor):
  NavSurface (modo grade sobre ph2d-grid) + NavObstacle → NavAgent → (P2: NavLink, NavCostArea)

Onda D (conteúdo e produto):
  PrefabVariant · ObjectPool · PinTo · BehaviorTree+Blackboard (decisão nodegraph-vs-folha = ADR)
  SaveGame + SavePolicy e Persistent/WorldRef ← DEPENDEM do R1 (play mode, decisão do Enio)

Onda E (avançados/horizonte):
  UtilityBrain · TacticalQuery · FollowHistory · ProximityActivator · HearingSense · rede (Replicated/Sync/
  RollbackManaged — rollback primeiro, pelo trunfo do determinismo)
```

Gargalos externos ao domínio: **R1 (shell de jogo/play mode, adiado pelo Enio)** trava Persistent, WorldRef e
SaveGame player-facing; **R3 (tabela sinal→ação)** trava as AÇÕES autoráveis de FSM/BT/Spawner-on-signal;
**movimento** (domínio vizinho) é o executor do que o NavAgent decide.

---

## §12 — Tabela-resumo (nome · prioridade · já existe no PH2D?)

| # | Componente | Prio | PH2D hoje |
|---|---|---|---|
| 1 | PrefabAsset (fluxo de editor) | P0 | parcial (`PrefabDoc`/`SceneDoc`, sem UI) |
| 2 | PrefabRef (instância aninhada) | P0 | parcial (spawn existe, sem componente/override) |
| 3 | PrefabVariant | P1 | não |
| 4 | SpawnPoint | P0 | parcial (Name+Transform; sem marker/gizmo) |
| 5 | Spawner | P0 | não (sementes: `spawn_prefab`, `SpawnQueue`) |
| 6 | ObjectPool | P1 | não |
| 7 | Lifetime | P0 | não |
| 8 | DestroyOutside | P0 | não |
| 9 | Tags (grupos/queries/broadcast) | P0 | não |
| 10 | Team (afiliação) | P1 | não |
| 11 | OnScreenEnabler | feito | **sim** (`ph2d-ecs`) |
| 12 | OnScreenNotifier | P1 | parcial (Enabler mudo — falta o sinal) |
| 13 | ProximityActivator | P2 | não |
| 14 | Persistent (entre cenas) | P1 | não (dep. R1) |
| 15 | WorldRef (proxy de mundo + relógio próprio) | P1 | não (dep. R1; pede ADR) |
| 16 | PinTo (pull/push, por canal) | P1 | não |
| 17 | FollowHistory (delay/ghost/replay) | P2 | não (primo: `TapeWire`) |
| 18 | ViewportAnchor | P2 | não (decisão do domínio UI) |
| 19 | RaySensor | P1 | não |
| 20 | SightSense | P1 | não (padrão provado: `SignalOnHit`) |
| 21 | HearingSense + NoiseEmitter | P2 | não (substrato: `SignalOutbox`) |
| 22 | PerceptionSource | P1 | não |
| 23 | StateMachine (FSM de gameplay) | P0 | parcial-embrião (`ph2d-ui-state::Machine`) |
| 24 | BehaviorTree + Blackboard | P1 | não (trunfo: editor do `ph2d-nodegraph`) |
| 25 | UtilityBrain | P2 | não |
| 26 | TacticalQuery (EQS-lite) | P2 | não |
| 27 | NavSurface (grade + navmesh) | P1 | parcial (`ph2d-grid`: A* pronto, custo uniforme, sem consumidor) |
| 28 | NavAgent | P1 | não |
| 29 | NavObstacle | P1 | não |
| 30 | NavLink | P2 | não |
| 31 | NavCostArea | P2 | parcial (A* sem custo) |
| 32 | SaveGame (slots de runtime) | P1 | parcial-infra (`ProjectState`+registry+ring GGPO) |
| 33 | SavePolicy (Persist/NoSave/perfis) | P1 | não |
| 34 | Replicated (rede) | P2 · horizonte | não (`ph2d-net` vazio) |
| 35 | NetworkTransformSync / NetworkStateSync | P2 · horizonte | não |
| 36 | RollbackManaged | P2 · horizonte | semente rara (GGPO + determinismo 3-OS) |
