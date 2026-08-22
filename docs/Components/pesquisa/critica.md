# Crítica de completude e verificação — levantamento de componentes PH2D

> Crítico de completude, 2026-08-20. Insumos: 7 dossiês + `inventario_ph2d.md` + 4 sínteses
> (visual_camera, movimento_fisica, interacao_fluxo, estrutura_ai), todos lidos por inteiro.
> 5 verificações por WebSearch/WebFetch contra doc oficial. Iluminação 2D: ADIADA (respeitado —
> nada aqui a prioriza).

---

## 1. O que caiu entre as cadeiras — presente em dossiê, AUSENTE das 4 sínteses

Ordenado por gravidade (impacto × frequência de uso num jogo 2D):

### Graves (mereciam slot P0/P1 e nenhuma síntese os lista)

1. **`Health` (vida/escudo/armadura/regen/i-frames/dodge)** — GDevelop `Health` (extensão
   revisada: max health, over-heal, shield com regen, armadura plana e %, damage cooldown,
   conditions `Is dead`/`Is just damaged`) + UE GAS Attributes. É o "RPG-combat-core" que o
   próprio dossiê Construct/GDevelop chama de matador nº 10 — e nenhum dos 4 domínios o reivindicou
   (movimento parou em corpos; estrutura parou em Team; interação parou em sinais). **O par
   dano→vida→morte é o loop de gameplay nº 1 e hoje não tem dono no cardápio.**
2. **`GameplayEffect` (buff/debuff/DoT/stack como ASSET)** — UE GAS Gameplay Effects (Instant/
   Duration/Infinite, modifiers, periodic=veneno, stacking, tags que concedem/exigem). Dossiê Unreal
   §14 o marca "média-alta para 2D, UI pura, motor pequeno". Sumiu inteiro. (Depende de Health;
   entra como wave 2 do mesmo dono.)
3. **`WeaponFire` (a ARMA do player: cooldown/ammo/reload/overheat/spread)** — GDevelop
   `Fire bullets` (matador nº 10 do dossiê). A síntese de movimento cobre `TurretAim` (torreta
   AUTÔNOMA) e `ProjectileMotion` (a bala) — mas o disparador dirigido por input com munição e
   recarga não está em síntese nenhuma. Trio completo seria WeaponFire + ProjectileMotion + Spawner.
4. **Controles de formulário de UI: `UiTextInput` (EditBox), `UiToggle`, `UiSlider`, `UiDropdown`,
   `UiPageView`** — Cocos (EditBox com teclado nativo, Toggle+ToggleContainer, Slider, PageView+
   indicador) e Unity uGUI (Dropdown, Input Field). A síntese de interação lista Button, Label,
   ProgressBar, ScrollView, LayoutGroup, FocusNav — e para aí. **Sem slider e toggle não existe
   tela de settings; sem text input não existe "digite seu nome".** Buraco sistemático: a suíte de
   UI da síntese cobre HUD, não formulários.
5. **Variáveis de jogo observáveis (`GameVariables` / Data Manager com eventos)** — Phaser Data
   Manager (key-value com eventos `changedata-…` — "vida mudou → HUD reage sem acoplamento"),
   variáveis de objeto/cena do GDevelop/Construct, Blackboard global. A interação menciona
   "incrementar contador/variável" como AÇÃO da tabela R3 e o `UiLabel` menciona binding — mas o
   armazém de variáveis em si (por objeto + global, com evento de mudança) não é item de nenhuma
   síntese. É o chão do placar, da quest flag e do HUD reativo.

### Médios (cauda que faz "tem um componente pra isso" ser verdade)

6. **`RichText` + `TypewriterText`** — Cocos RichText (marcação inline, imagens embutidas, links
   clicáveis) e GDevelop Typewriter — o texto de DIÁLOGO. Ausentes.
7. **Diálogo/conversa (`Flowchart controller`)** — Construct Flowcharts é citado no dossiê
   (project primitive p/ "diálogos, FSMs de alto nível") e nenhuma síntese o herdou: a FSM da
   estrutura é de gameplay, não de conversa. Diálogo com escolhas é gênero inteiro (visual novel,
   RPG) sem dono no cardápio.
8. **`VideoPlayer`** — Cocos VideoPlayer, Phaser Video (vídeo como textura), GDevelop Video.
   Cutscene em vídeo/tela de logo. Nenhuma síntese o lista.
9. **Rotação contínua (`RotatingMotion`)** — Construct `Rotate` (behavior com preview) e UE
   `RotatingMovementComponent` ("a serra giratória — todo platformer tem"). A síntese de movimento
   tem Orbit, Sine/Oscillator, mas o rotor puro não. Mitigante: Motion Nodes provavelmente o
   exprimem (lei "meça a composição antes") — mas a nota precisava existir.
10. **Instancing em massa (`MultiMesh`/static batch)** — Godot MultiMeshInstance2D, Phaser
    Blitter/SpriteGPULayer (milhões de sprites, 1 draw). Vegetação/props em massa. Mitigante:
    `motion.clone` GPU-resident já cobre a simulação; falta o recorte "prop estático barato".
11. **`BackBufferCopy` (screen-texture regional p/ shader)** — Godot; habilita distorção/refração/
    heat-haze localizados. `ObjectFx` (K.1) cobre a pilha de efeitos, não a CAPTURA de tela que
    esses efeitos leem.
12. **Desenho procedural de runtime (`ShapePainter`/Gizmos de jogo)** — Cocos Graphics, GDevelop
    Shape Painter, Phaser Graphics, Bevy Gizmos. Mira dinâmica, indicador de alcance, telegraph de
    ataque, debug draw. A síntese visual menciona Gizmos num item de tabela do dossiê Bevy e não
    gera componente.
13. **Gestos touch (swipe/pinch/rotate)** — GM Gesture events (tap/drag/flick/pinch/rotate com
    event_data), GDevelop Swipe/Pinch. `InputTriggers` (interação 1.3) cobre tap/hold/combo/chord —
    parsing temporal de BOTÃO — mas não gestos de toque em tela.
14. **Estados de APP (Menu/Playing/Paused — Bevy `States`, `OnEnter`/`OnExit`)** — o fluxo global
    do jogo. Tocado de raspão por WorldRef/TimeChannel, nunca listado como item próprio; o dossiê
    Bevy o marca "ALTA". É R1-adjacente — mas a nota devia existir para o R1 herdá-la.
15. **`LookAt`/Aim constraint puro** — Unity Aim/LookAt Constraint, GDevelop Face forward. TurretAim
    é um superset (aquisição+cadência); o "vire para o alvo" isolado (torre decorativa, olhos que
    seguem) não está em síntese nenhuma. Barato; candidato a preset de Motion Node.
16. **Formation/batch actions** (Phaser Actions: GridAlign, PlaceOnCircle, ShiftPosition-cobra) —
    arranjo de N objetos em runtime (menu circular, formação de inimigos). Menor; anotar.
17. **`UISkew`** (Cocos) — cisalhamento no transform. Trivial; só registrar que o Transform não tem.

### Notas de defesa das sínteses
As 4 sínteses cobriram ~95% dos dossiês e adicionaram critério (P0/P1/P2, "já tem", dependências).
Os buracos acima têm padrão claro: **(a) combate/RPG (Health/Effects/Weapon) não pertencia
naturalmente a nenhum dos 4 domínios; (b) a suíte de UI foi cortada em "HUD" e perdeu
"formulários"; (c) juice/utilitário de cauda longa (rotate, gestos, shape painter) caiu no vão
entre "movimento" e "interação".** Se houver uma 5ª síntese, ela se chama "combate & formulários".

---

## 2. Categorias que NENHUM dossiê cobriu bem

Contra o checklist dado (visual, esqueleto/deform, partículas, câmera, física, character
controllers, paths, animação, timeline, áudio, input, UI, navegação/AI, spawn/pooling, timers,
rede, save, parallax, utilitários, extensibilidade/scripting): **todas as 19 têm pelo menos um
dossiê forte** — o fan-out de engines foi bem escolhido. As duas mais rasas do conjunto:

- **Rede/multiplayer** — quase todo dossiê "declara" (5 linhas de resumo); só GDevelop Multiplayer
  object e GM Rollback têm detalhe de produto. Nenhum dossiê cobriu transporte, input delay,
  relay/lobby, interest management com profundidade. Aceitável (ph2d-net vazio, M13 por demanda) —
  mas quando rede entrar, o levantamento atual NÃO basta para desenhar.
- **Save de jogo** — a maioria dos dossiês é "declarado ausente"; só GDevelop (Save state) e UE
  (USaveGame) têm substância. Falta o problema DIFÍCIL em todos: migração de versão de save de
  jogo LANÇADO (a estrutura §9 o nota de raspão como "degrau próprio").

Categorias que o checklist NÃO pedia e NENHUM dossiê tocou (blind spots do levantamento inteiro):

1. **Diálogo/narrativa** (árvore de conversa, escolhas, balões) — só Construct Flowcharts de
   raspão. Para uma engine "para artistas", é possivelmente a ausência mais cara do levantamento.
2. **Localização de CONTEÚDO do jogo** (strings do jogo por língua, fonte por script, assets por
   locale) — zero menções em 7 dossiês. O PH2D tem i18n do EDITOR (HR-15); o do JOGO é outra
   categoria e nasce barata se desenhada junto com UiLabel/binding.
3. **Acessibilidade de jogo** (subtítulos, daltonismo, reduced-motion de gameplay) — zero. Irônico:
   o PH2D já tem `reduced_motion` em `~/.ph2d/prefs.txt` — a semente existe na casa.
4. **Inventário/itens/loot** — zero em todos (território de asset store nas engines grandes).
   Candidato a componente-diferencial da mesma família de Health.
5. **Debug/tuning para o USUÁRIO da engine** (overlay de FPS, debug draw de física por toggle,
   console de comandos) — pontas soltas (GM Debug Overlay, GDevelop FPS displayer, flag de debug do
   Arcade) sem tratamento em dossiê nem síntese.
6. **Aleatoriedade autorável com seed** (random como serviço determinístico exposto na UI) — só a
   seed de partículas do Godot aparece. Numa engine com determinismo 3-OS e GGPO, o "random do
   usuário" TEM de passar pelo serviço com seed — regra que ninguém escreveu.
7. **Tutorial/onboarding in-game e achievements/serviços de plataforma** — zero (defensável fora
   de escopo agora; registrar).

---

## 3. Verificação de 5 afirmações (WebSearch/WebFetch contra doc oficial)

| # | Afirmação (síntese) | Veredito |
|---|---|---|
| 1 | `bevy_trauma_shake`: modelo trauma-com-decay, `add_trauma(0.3)`, `ShakeSettings {amplitude, trauma_power, decay_per_second, frequency, octaves}` (visual H.4) | **CONFIRMADO** — crate real (johanhelsing), trauma 0–1, API e campos exatamente como descritos. github.com/johanhelsing/bevy_trauma_shake · docs.rs/bevy_trauma_shake |
| 2 | Enhanced Input da UE tem trigger built-in **"Combo"** (sequência de ações numa janela — "fighting game por asset") (interação 1.3) | **CONFIRMADO** — `UInputTriggerCombo` oficial: array ordenado de Input Actions com estados de conclusão e tempo entre passos. dev.epicgames.com (Python API InputTriggerCombo, Enhanced Input 5.8) |
| 3 | Cocos RigidBody2D tem tipo **"Animated"** que deriva velocidade da animação (plataforma por keyframe sem teleporte) (movimento §A, base do KinematicPlatform) | **CONFIRMADO** — doc oficial 3.8: Animated deriva de Kinematic e "calcula a velocidade necessária a partir da pose alvo e a atribui", existindo para evitar penetração ao animar corpos. docs.cocos.com/creator/3.8/.../physics-2d-rigid-body.html |
| 4 | `vleue_navigator`: pathfinding **Polyanya**, navmesh **auto-atualizável**, obstáculo direto **dos colliders avian/rapier** (estrutura §8, base do NavSurface) | **CONFIRMADO COM RESSALVA** — Polyanya sim; `NavMeshUpdaterPlugin` com auto-update (Direct/debounced) sim; `avian2d`/`avian3d` são dependências OPCIONAIS oficiais (obstáculo-do-collider avian confirmado). A metade "**rapier** como fonte" NÃO foi confirmada na doc — tratar como avian-only ao citar o desenho de referência. github.com/vleue/vleue_navigator · docs.rs/vleue_navigator |
| 5 | Sound Attenuation da UE tem **"Non-Spatialized Radius"** — perto da fonte o som interpola para 2D, matando o "pan pulando" (interação 3.1, spec do AudioSource2D P0) | **CONFIRMADO** — `FSoundAttenuationSettings.NonSpatializedRadius` (+ `NonSpatializedRadiusStart/End/Mode` no 5.x): abaixo do raio os canais fazem bleed interpolado até 100% na origem. dev.epicgames.com/.../sound-attenuation-in-unreal-engine |

**Leitura:** 5/5 sustentadas (uma com ressalva pontual). As sínteses estão factualmente confiáveis;
a única correção a propagar é "avian, não necessariamente rapier" na nota do NavSurface.

---

## 4. TOP-20 — se o PH2D implementasse os primeiros 20 amanhã, nesta ordem

Critério: (custo dado o que JÁ existe) × (código que elimina) × (o que destrava os seguintes).
As 4 sínteses convergem em 3 fatos que a ordem respeita: o custo real é a UI (passo 5 do
inventário); a tabela sinal→ação é o item ⭐ único; e o transporte de sinais + Timeline +
partículas + platformer JÁ estão prontos e só pedem fachada.

| # | Item | Por quê (1 linha) |
|---|---|---|
| 1 | **Infra: Inspector derivado do tipo + UI "Add Component" + required components** | As 4 sínteses o declaram pré-requisito: sem ele cada um dos 19 abaixo paga uma seção artesanal de Inspector — é o divisor do custo do catálogo inteiro. |
| 2 | **`Timer`** | Melhor razão custo/benefício do levantamento (veredito unânime): trivial de fazer, uso universal, e é o primeiro produtor de `Signal` barato. |
| 3 | **`SensorZone`** | Fechar `is_sensor`+`SignalOnHit/Leave` em modo overlap: todo trigger de gameplay (moeda, dano, porta, checkpoint) vira shape+evento — o rapier já reporta, falta a costura. |
| 4 | **`AudioSource2D` + `AudioListener2D`** | O jogo ganha a primeira reação AUDÍVEL e o rack de 42 efeitos (hoje 100% editor-side) ganha seu consumidor de cena; copiar `max_polyphony` (Godot) e non-spatialized radius (UE, verificado). |
| 5 | **`SignalActions` (a tabela R3)** ⭐ | O item mais importante de todas as sínteses: transporte pronto e gateado, NADA autorável reage — com som(4), timer(2) e enable/spawn no dropdown, sinais viram gameplay sem código. |
| 6 | **`ActionMap` + `ActionState`** | Input semântico (a maior lacuna bruta: `ph2d-input` é gamepad cru, sem teclado de gameplay) — pré-requisito do padrão "default controls" de TODO controller e da tela de rebinding de graça. |
| 7 | **`GameCamera` (componente) + `CameraFollow` + `CameraLimits`** | A maior lacuna do PH2D e de metade da indústria (Cocos/Bevy zero, Godot sem shake/confiner): promover `Camera2d` a entidade + follow com deadzone + clamp de fase = o rig que todo jogo 2D reescreve. |
| 8 | **`SpriteFrames` + `AnimatedSprite`** | "O componente nº 1 de qualquer engine 2D" (Godot) que a Unity notoriamente não tem leve — e com sinergia única: tocar quadros de um documento Flip autorado DENTRO do app. |
| 9 | **`Tags`** | A moeda de troca de tudo que segue (spawner "onde", percepção "quem", broadcast, save-filter); adotar hierarquia estilo Unreal (`enemy.flying`) desde o dia 1 custa quase nada. |
| 10 | **`PrefabAsset` (fluxo de editor) + `PrefabRef`** | `PrefabDoc`+`spawn_prefab` prontos com ZERO UI: salvar-seleção/instanciar/override é só editor — e é o multiplicador de conteúdo de todo o resto. |
| 11 | **`Spawner` + `SpawnPoint`** | A fábrica como componente (categoria que Unity/Godot/UE NÃO têm) sobre sementes fortes (`SpawnQueue`, `SignalReader`) — `spawn_on_signal` conecta imediatamente com a tabela R3. |
| 12 | **`Lifetime` + `DestroyOutside`** | Higiene de ciclo de vida em 2 marcadores triviais — sem eles o Spawner e o projétil vazam; o par obrigatório do item 11 e 14. |
| 13 | **`TopDownPlayer`** | O 2º controller canônico (o platformer JÁ existe e está à frente do mercado): destrava RPG/twin-stick/isométrico no dia 1, reusando o desenho lei-pura+ponte validado. |
| 14 | **`ProjectileMotion`** | O mover genérico arcade (avança+gravidade+ricochete+alcance+homing, modelo UE ProjectileMovement): sem ele todo usuário escreve integração e reflexão de bala à mão. |
| 15 | **`StateMachine` (FSM de gameplay)** | Enquanto o script não tem UI, é o único cérebro autorável — estado-como-componente (seldom_state) deixa animação/física/spawner reagirem a estado por query; P0 da síntese de estrutura. |
| 16 | **`ScriptProperties` + attach visual do `LuauScript`** | Host, persistência e determinismo prontos, ZERO UI: anexar script pela UI + campos por instância (modelo `go.property`) é o mecanismo nº 1 de "script parametrizável" em toda engine madura. |
| 17 | **`TilemapLayer` + `TileSet` + `TilemapCollider`** | A linha longa que precisa começar cedo: nível pintado com colisão fundida (composite anti-ghost), sobre `ph2d-grid` (11 grids + A* prontos) e física pronta; autotile fica para a wave 2. |
| 18 | **`ParticleEmitter` (fachada de componente)** | A melhor simulação da classe JÁ existe (Motion Nodes GPU, 4,19M @ 3,6 ms): o trabalho é só o painel de módulos no objeto + a escada "abrir como grafo" — cuidado com o cap (caso §0.0). |
| 19 | **`SequencePlayer`** | A Timeline já vence Godot/GM/Bevy e o difícil (religar bindings por `WireId`=hash do Name) está SHIPADO: o componente que a toca na cena + "play on signal" transforma o módulo inteiro em cutscenes de jogo. |
| 20 | **HUD mínimo: `UiCanvas` + `UiAnchor` + `UiLabel` + `UiButton`** | Placar, vida e menu fecham o loop de demo completo — sobre a família Vec* (widgets+taffy+Smart Animate) que já é melhor fundação que a de Godot; o botão publica `Signal` para a tabela do item 5. |

**Primeiros suplentes (21–25), para quando um jogo-guia puxar:** `RaySensor` (P0 na síntese de
movimento — sobe se IA/turret entrarem antes), `Tween`+presets Fade/Flash (motor pronto, só
empacotamento), `PathFollow` (a vitrine "desenhe a patrulha com a caneta"), `Health`+`WeaponFire`
(o buraco nº 1 da seção 1 — dono novo: domínio combate), `CameraShake`+`ShakeEmitter` (o juice que
casa 1:1 com o idioma Signal).

**Coerência da ordem:** 1 destrava o custo de todos; 2–5 fazem sinais virarem jogo (com som);
6 destrava controllers; 7–8 dão o olhar e o corpo visível; 9–12 dão conteúdo dinâmico; 13–15 dão
gameplay sem código; 16 abre a válvula de escape; 17–20 são as três fachadas sobre módulos já
vencedores + o HUD que fecha a demo.
