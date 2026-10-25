# BlenderConstraints — plano do módulo (25/10/2026)

> Linha `line/BlenderConstraints`, base `c3662cd8e4`. Pesquisa: [relatório](reports/Constraints%20para%20engine%202D.md)
> (+ notas em `research_notes/`). Oráculo medido: [`fixtures/blender_5_2_3_constraints_2d.json`](fixtures/blender_5_2_3_constraints_2d.json),
> gerado por [`ferramentas/oraculo_blender.py`](ferramentas/oraculo_blender.py).
> Estado: **PLANO — nenhuma linha de produto escrita.** Espera o «sim» do dono.

## 0. O que o artista vai conseguir fazer

1. Selecionar um objecto (ou um osso, em Pose), abrir o cartão **Constraints** no Inspector, carregar em
   **+** e escolher *Child Of*, *Copy Transform*, *Look At*, *Limit*, *IK*, *Follow Path*, *Spring*…
2. Escolher o alvo **clicando nele no palco** — sem campo de texto, sem *Set Inverse*.
3. Ver a ligação no palco (linha até ao alvo, arco/caixa/círculo do limite) e **arrastar** os limites.
4. Animar a **força** (0–100 %) de cada constraint com o Auto Key — é assim que se «pega e larga» um objecto.
5. Reordenar a lista arrastando a pega; a ordem é **exactamente** a ordem do cálculo (sem excepções escondidas).
6. **Cozer** um intervalo em chaves (*Bake*) e desligar a constraint ali.
7. Ver em cor quando uma constraint **não faz nada** (sem alvo, ciclo, força 0, corpo de física dinâmico) e porquê.
8. (Ondas 4–5) desenhar no Vetor com regras de CAD — paralelo, perpendicular, tangente, medida — e animar as medidas.

## 1. Estado da arte — o que eles fazem, porquê, e o que ABANDONARAM

Resumo do [relatório](reports/Constraints%20para%20engine%202D.md); números nossos medidos no oráculo.

| App | O que acertou | O que tentou e largou / a dor que ficou |
|---|---|---|
| **Blender 5.2** | pilha com influência animável, espaços, *bake* visual; 28 tipos | **largou** Python constraint e Rigid Body Joint (4.5); trocou o Limit Rotation (4.2, «Legacy Behavior»); **Set Inverse** foi a dor nº 1 — **MEDIDO: no 5.2 o Child Of já guarda o offset ao nascer** (`child_of.sem_inverse.ao_ligar` = pose de antes, ao 1e-6). Dores vivas: **IK corre sempre por último** — MEDIDO: um Copy Rotation 90° ABAIXO do IK some sem aviso (`ik_com_copy_rotation_abaixo.b2.cauda` = alvo `(1.2, 0.9)`); ciclos só na consola; o resultado não aparece na UI. Planeia *Constraint Nodes* e *Declarative Constraints* — **não entregou** |
| **Spine 4.2/4.3** | IK de 1–2 ossos (recusa 3+ **de propósito**), *Softness*, bit de dobra, Physics por canal com modo **Deterministic** (re-simula do quadro 0), Slider (osso dirige animação), *bake* | Mix entre 0–100 % pode **inverter a rotação** de repente; escala negativa no alvo gira 180° |
| **Rive** | 8 constraints, Strength animável, ordem por arrasto, «a de baixo vence a 100 %» | separou Layout das constraints (são problemas diferentes) |
| **Unity** | botão **Activate** (grava o offset), pesos por fonte, Animation Rigging empilhado | **Godot** deixou a IK 2D (`SkeletonModification2D`) **experimental** e fez a nova só em 3D |
| **Unreal** | criar com conta-gotas no viewport; **chaves de compensação** ao ligar/desligar; *bake* deixa a constraint keyada inactiva | — |
| **After Effects / Duik / Cavalry / Apple Motion** | *behaviours* de 2–4 parâmetros com nome físico, somados à animação | expressões são poderosas e **ilegíveis** ao animador; Lottie não leva `wiggle()` |
| **CAD (SolveSpace, Onshape, FreeCAD)** | inferência ao desenhar, dimensão que sobre-restringe nasce **REF**, cores livre/definido/conflito, lista «apague uma destas» | saltos de ramo (ângulo suplementar, lado da tangência) ao arrastar |
| **Illustrator / Graphite / Figma** | Figma: pins por lado + Auto Layout | **nenhum** editor vectorial de animação tem constraints geométricas — é o nosso diferencial |

**Leis medidas no oráculo** (Blender 5.2.3, plano XY) — viram fixture dos gates:

| Pergunta | Resposta medida | Lei do PH2D |
|---|---|---|
| Copy Rotation a 50 % entre 170° e −170° | **180°** (0,25 → 175°, 0,75 → −175°) | mistura pelo **arco curto** |
| Track To / Locked Track / Damped Track, alvo em (3,4) | **53,1301°** nos três (Δ ≤ 1e-5) | **um** Look At |
| Limit Rotation [−45°, 45°] | 60/120/170 → **45**; −100/−170 → **−45** | borda **mais próxima no círculo** |
| Limit Distance *inside* 2, dono a 5 | **(2, 0)** | projecção radial |
| Copy Location(8,1) + Limit x≤3, nas duas ordens | **3** vs **8** | a ordem da pilha **é** a lei |
| IK + Copy Rotation abaixo | o Copy Rotation **some** | ⛔ **recusamos a lei do oráculo aqui**: no PH2D a IK é um item como os outros, na posição em que está (a dor da pesquisa; decisão nossa, gate próprio) |
| Child Of ao ligar | **sem salto** | offset automático, sem botão |

## 2. O desenho — e a porta ÚNICA de cada pergunta

**Bones são entidades com `Transform`** (`ph2d-skeleton-ecs/src/lib.rs:85`): a MESMA pilha serve objecto e osso.

### 2.1 Crates (drop-crates; nada de família na shell)

| Crate | O que é | Depende de |
|---|---|---|
| `ph2d-constraints` (NOVA, pura, sem ECS) | os tipos (`Constraint`, `ConstraintKind`, `Space`, `Mix`), a **pose 2D** `Pose2 {x, y, θ, sx, sy}` e as LEIS de cada tipo | `ph2d-skeleton` (`reach`, `clamp_to_limit`, `bend`), `ph2d-arc-length` (`lut`, `at`), `ph2d-node-motion-spring::law::passo` |
| `ph2d-constraints-ecs` (NOVA) | componente `Constraints` (a pilha), alvos por `StableId`, ordenação topológica, ciclos, a fase de avaliação, a família de canais, escrita via `PreviewDrive` | `ph2d-ecs`, `ph2d-preview-drive`, `ph2d-skeleton-ecs` |
| `ph2d-app-constraints` (NOVA, família) | cartão do Inspector (lado app), pick no palco, gizmos, *bake*, roteador `PH2D_CONSTRAINT_SMOKE` | as duas acima + `ph2d-app-host` |
| `ph2d-panel-inspector` (existente) | `ids/inspector_constraints.rs`, `populate_constraints.rs`, `event_constraints.rs`, `sections/constraints.rs` — ficheiros **irmãos novos** | — |

### 2.2 As portas únicas

| Pergunta | Porta ÚNICA | Quem NÃO pode responder |
|---|---|---|
| «qual é a lei do tipo X?» | `ph2d_constraints::<tipo>::resolve(&Pose2 dono, &Alvos, &Params) -> Pose2` | o nó Motion `look_at` passa a **chamar** `ph2d_constraints::look_at` (hoje a lei é privada em `ph2d-node-motion-look-at/src/lib.rs:390`) — uma lei, dois consumidores |
| «IK de 2 ossos / cadeia» | `ph2d_skeleton::reach` (`reach.rs:411`) | ⛔ a cópia privada em `ph2d-node-rig-ik-2bone/src/lib.rs:96` passa a chamar `reach` (hoje são **duas** leis dos cossenos — a terceira seria a nossa) |
| «misturar duas poses com força t» | `ph2d_constraints::blend(a, b, t)` — translação lerp, **θ pelo arco curto**, escala log-lerp | nenhum tipo mistura à mão |
| «onde está a pose MUNDIAL de E agora?» | `ph2d_ecs::transform_inverse::world_transform` (`:124`) + `Transform::inverse_compose` (`:251`) | a fase não lê `GlobalTransform` (é do `PresentWorld`, de um quadro antes) |
| «quem escreve o `Transform` restringido?» | `PreviewDrive::driven(e, before, after)` com `Driver::ConstraintPose` (variante NOVA, append em `ph2d-preview-drive/src/lib.rs:76`) | ninguém escreve `Transform` directo; *preview ≠ documento*: gravar a meio do play grava o **autorado** |
| «em que ordem se avalia?» | `ph2d_constraints_ecs::ordem(world) -> Vec<Entity>`: topológica sobre (hierarquia ∪ alvos); dentro da entidade, a ordem da pilha, **IK incluída** | nenhum `if kind == Ik { … por último }` |
| «isto faz alguma coisa?» | `ph2d_constraints_ecs::estado(item) -> Estado {Activa, SemAlvo, Ciclo, Inerte(força 0), CorpoDinamico, Invalida(motivo)}` — o cartão, o gizmo e o gate leem esta | o cartão não recalcula |
| «qual o canal animável deste campo?» | família **`CONSTRAINT = 8`** (append em `ph2d-ecs/src/canal.rs:212`, ficheiro irmão `canal_constraint.rs`): `campo` = id do parâmetro, `i:j` = **id estável do item** (`u32`, monotónico, nunca reusado) | a família `COMPONENTE` (6) **não serve**: endereça por tipo de componente, não por item da lista (`canal_componente.rs:4`) — reordenar partiria as chaves |
| «onde está o offset?» | `Constraint.offset: Pose2`, gravado ao **ligar** (força 0→>0 por gesto, criação, troca de alvo); botão *Reset offset* | sem botão *Set Inverse* |
| «um intervalo cozido» | `ph2d_app_constraints::bake(range)` escreve chaves pela porta do Auto Key e uma chave de força 0 no intervalo | — |

### 2.3 Onde corre no quadro (`shells/desktop/src/render_loop/fase_frame_open.rs:21`)

```
fase_timeline_drain → fase_paralaxe → fase_physics_step → … → fase_open_recipe
  → ★ fase_constraints (objectos)  → fase_skeleton_drives (★ pilha dos ossos dentro, depois do smart, junto do goal)
  → fase_macros_do_painter → fase_sim_extract (propaga transforms)
```

- **Depois da física:** prender um sprite a um corpo que cai é o caso de jogo nº 1. Um corpo **dinâmico** restringido
  é recusado com `Estado::CorpoDinamico` («use uma junta»); um **cinemático** segue a pose restringida no tique seguinte.
- **Antes do esqueleto:** a pilha de um osso pode ter por alvo um objecto já restringido.
- **Depois da timeline:** a constraint opera sobre a pose animada (como em todas as apps).
- ⚠️ **A verificar na abertura da W1:** onde correm `fase_tweens`, scripts e o cook do Motion (não estão em
  `fase_frame_simulation`); e se o `skeleton_goal` (`fase_skeleton_drives.rs:33`) É a IK — se for, a IK da pilha
  **é a mesma porta** (o goal vira o item IK), não uma segunda.

### 2.4 Determinismo ao arrastar a timeline

Todos os tipos menos **Spring** são função pura das poses do quadro ⇒ *scrub* dá o mesmo bit. **Spring** tem
estado: re-simula do quadro 0 com anel de checkpoints — o padrão de `ph2d-physics-ecs/src/bridge/rewind.rs:25`
(`STRIDE`), copiado como **lei** e não como código partilhado se o tipo de estado divergir.

## 3. Contrato congelado e schema

**Não encosta em contrato congelado.** Nenhum ficheiro do contrato é tocado: `Tool`/`PanelEvent`
(`crates/ph2d-editor-core/src/tool.rs:26,63`), `NodeOp`/`OpResolver` (`crates/ph2d-nodegraph/src/node.rs:232`,
`cook.rs:52`), Vector antigo (`crates/ph2d-vector-doc/`). O pick usa `take_command_pick_if`
(`ph2d-editor-core/src/interaction/state/chrome_ops.rs:501`), não um `PanelEvent` novo; o nó `look_at` muda o
**corpo** do `eval`, não a superfície do `NodeOp`. **Prova no fecho de cada onda** (tem de imprimir vazio):

```bash
git diff --name-only $(git merge-base main HEAD) | grep -E 'ph2d-editor-core/src/tool\.rs|ph2d-nodegraph/src/(node|cook)\.rs|ph2d-vector-doc/'
```

**Schema (o número CONTA-SE na integração, nunca se escolhe):**
- `Constraints` é componente novo gravado ⇒ **+1 degrau** em `PROJECT_SCHEMA` (hoje `193`,
  `shells/desktop/src/project_schema.rs:494`; tripla `(193, 13, 23)` em `project_schema_tests.rs:318`), salvo se o
  padrão «componente com `VERSION` próprio, zero bump» (`ph2d-ecs/src/vec_anchors.rs:47`) se aplicar — decide-o o
  teste red-first de ida-e-volta, na W1.
- **Onda 4 (vetor)**: o `VecVertex` **não tem id estável** (`ph2d-vec-scene/src/lib.rs:401`, endereçado por índice).
  Uma regra de CAD presa a índices parte quando se apaga um ponto ⇒ id por vértice ⇒ **+1 em
  `VEC_SCENE_SCHEMA_VERSION`** (hoje `23`, `ph2d-vec-scene/src/schema.rs:90`), com migração.
- Ids/variantes novos a anotar no handoff (colisão): `Driver::ConstraintPose`, `familia::CONSTRAINT = 8`, ADR
  **0192**, `PH2D_CONSTRAINT_SMOKE`, a feature `app-constraints`, o descritor `catalog/constraints.rs`.

## 4. A UI — as 4 condições, uma a uma

| Peça | EXISTE | PINTADA + REGISTADA | O CLIQUE CHEGA AO BARRAMENTO | A SEQUÊNCIA LEVA A ALGUM LUGAR |
|---|---|---|---|---|
| Entrada «Constraints» no **Add component** | descritor em `ph2d-component-desc/src/catalog/constraints.rs` | paleta `component_palette.rs:189` | `InspectorAddComponentRequested` (`event.rs:199`) → `attach_one` (`component_attach.rs:234`) | o cartão aparece, vazio, com **+** |
| **+** (menu de tipos) | id em `ids/inspector_constraints.rs` | `populate_constraints.rs` + `sections/constraints.rs` | `event_constraints.rs` → `EditorAction` | item nasce com offset = pose actual (**zero salto**, gate) |
| **Escolher alvo** (botão + clique no palco) | padrão «armar pick» das juntas (`ph2d-app-physics/src/joint.rs:263,365`) | idem | `take_command_pick_if` (`chrome_ops.rs:501`) | o `StableId` do alvo grava-se; recusa a si mesmo e os ciclos com mensagem |
| **Força** (slider) | idem | idem | `event_constraints.rs` | com Auto Key ligado grava chave na família 8 |
| **Pega** (reordenar) | o padrão da pega de secção (`ids/inspector.rs:43-50`, `plano.rs:73`) | idem | idem | a ordem gravada é a ordem do cálculo (gate da fixture `ordem_da_pilha`) |
| **Liga/desliga · apagar · Reset offset · Bake** | idem | idem | idem | undo = **um passo** por gesto |
| **Estado** (cor + motivo) | lê `estado(item)` | idem | — | o texto diz o que fazer («use uma junta») |
| **Gizmos** (linha ao alvo, arco/caixa/círculo do limite, raio da mola) | padrão `bone_limit::arc` + `drag_edge` (`ph2d-app-skeleton/src/bone_limit.rs:107`, `bone_pose.rs:65`) | overlay da família | arrastar a borda edita o parâmetro | o limite actua **também no gesto** do artista |
| **Ossos** | a pilha do osso seleccionado em Pose (`bone_gesture.rs:227`) | mesmo cartão | idem | idem |

Textos só por `ph2d-i18n` (`app_components.rs`, as duas línguas), zero hex, zero `f32` de UI.

## 5. Gates red-first — e a fixture que contém o fenómeno

Cada gate é escrito **antes** do código e visto **vermelho**; a fixture é a do oráculo ou uma cena nossa que tem o fenómeno.

| Gate | Fenómeno na fixture | Onda |
|---|---|---|
| `the_blend_takes_the_short_arc` | 170°/−170° a 0, ¼, ½, ¾, 1 → valores do oráculo (`mistura_rotacao_copy_rotation`) | 1 |
| `child_of_never_jumps_when_switched_on_or_off` | avô rodado −20° e escalado 1,2, alvo 37°/1,5 — `child_of.*` do oráculo; desligar a meio também não salta | 1 |
| `one_look_at_is_the_three_blender_tracks` | `apontar_3_tipos` (53,1301°) + pai espelhado (sx = −1): o ângulo inverte, não gira 180° | 1 |
| `limit_rotation_goes_to_the_nearest_edge_on_the_circle` | os 6 ângulos de `limit_rotation_z_-45_45` | 1 |
| `limit_distance_projects_radially` | `limit_distance_inside_2` | 1 |
| `the_stack_order_is_the_evaluation_order` | `ordem_da_pilha` (3 vs 8) | 1 |
| `a_cycle_is_reported_and_never_evaluated` | A copia B, B copia A: os dois `Estado::Ciclo`, poses = autorado, zero NaN | 1 |
| `a_constraint_never_writes_the_document` | gravar a meio do play devolve o autorado (padrão `substitute/restore`) | 1 |
| `the_bake_equals_the_live_stack_at_every_frame` | 48 quadros, ao bit | 1 |
| seam (`ph2d-ui-testkit`): `add_pick_and_key_a_constraint` | clique real em **Add → Look At → Escolher alvo → clique no sprite → força 50 %** ⇒ alvo gravado + chave na família 8 | 1 |
| `ik_is_an_ordinary_item_in_the_stack` | a cena do oráculo `ik_com_copy_rotation_abaixo` com o resultado **contrário** (recusa documentada) | 2 |
| `the_two_bone_law_has_one_owner` | o nó `rig-ik-2bone`, o gesto e a constraint dão o mesmo bit | 2 |
| `bend_side_flips_with_the_mirror` | rig espelhado: a dobra troca de lado, o joelho não salta | 2 |
| `a_spring_rewinds_to_the_same_bit` | scrub 0→40→12→40: igual ao play contínuo | 3 |
| `a_dynamic_body_refuses_the_constraint` | corpo dinâmico com Copy Location ⇒ `Estado::CorpoDinamico`, a física manda | 1 |
| prova de mutação (`/pd-mutacao`) sobre `blend`, `ordem` e `estado` | apagar o arco curto / a ordem / o ciclo ⇒ o gate certo sangra | fecho |

Os números do oráculo da W2–W5 (IK com *softness*, Stretch To com volume, Follow Path por %, Spring) entram em
`oraculo_blender.py` **na abertura de cada onda**, antes do código — e o FreeCAD 1.1.4 (`freecadcmd`, LGPL,
instalado) é o oráculo do sketch geométrico da onda 4.

## 6. Cenas de smoke (números)

`PH2D_CONSTRAINT_SMOKE=<n>` (roteador em `ph2d-app-constraints`). **Medido hoje:** só as leis, no oráculo (tabela
do §1). **Ainda não medido:** a cena no PH2D — não existe código. A W1 corre a sonda headless e fotografa cada cena
(`docs/Components/ferramentas/fotografa_cena.sh`) **antes** de escrever a mensagem ao dono, com os números
medidos lado a lado com o oráculo.

| n | Cena | O que o dono vê | Número a medir antes da mensagem |
|---|---|---|---|
| 1 | Mão pega a espada (Child Of animado) | quadro 0–24 espada no chão; 24 a mão pega; 48 larga | salto ao ligar/desligar = **0 px** |
| 2 | Olho que segue o rato (Look At) + pupila dentro do olho (Limit Distance) | a pupila nunca sai do círculo | distância máxima ≤ raio |
| 3 | Ponteiro com limite de ângulo e mistura a 50 % | arco desenhado no palco, arrastável | 170° → 45°, mistura = 180° |
| 4 | Braço com IK (2 ossos) e Copy Rotation **abaixo** | a mão roda — ao contrário do Blender | ângulo da mão = alvo |
| 5 | Antena com Spring + scrub | arrastar a timeline não muda o balanço | diferença play × scrub = 0 |

## 7. Ondas (uma janela cada; o dono faz o smoke no fim de cada uma)

| Onda | Entrega | Peças |
|---|---|---|
| **W1 — a pilha** | cartão Constraints, + menu, alvo por clique, força com Auto Key, pega, estado, gizmos, Bake, undo, gravar; em **objectos** | Child Of · Copy Transform (por canal, Replace/Add) · Look At · Limit (caixa · ângulo · escala · distância) · ADR-0192 · `look_at` do Motion passa a chamar a lei única |
| **W2 — ossos** | a mesma pilha em Pose | IK (2 ossos analítica + cadeia FABRIK, bit de dobra, *Softness*, Stretch) unificada com o `skeleton_goal` · Follow Path (% e distância, orientar pela tangente) · Stretch To com volume · unificação da lei dos 2 ossos |
| **W3 — poderosas** | movimento secundário e controlo | Spring por canal (determinístico) · Transformation (mapear canal → canal) · Action/Slider (osso dirige clip; Flip troca desenho por ângulo) · Multi-parent ponderado · Pivot · Floor/colar ao contorno · Spline IK · prender a ponto de forma/simulação |
| **W4 — CAD no vetor** | regras de desenho técnico no editor vetorial, animáveis | id estável por vértice (+1 `VEC_SCENE_SCHEMA_VERSION`) · medir **ezpz** (`kcl-ezpz` 0.2.18, MIT, crates.io) × **fiksi** (`endoli/fiksi`, Apache-2.0, só git) no mesmo processo · coincidente/H/V/paralelo/perpendicular/tangente/igual/simétrico/medida · inferência ao desenhar · cores de graus de liberdade · REF · «apague uma destas» |
| **W5 — diferenciais** | o que nenhum app junta | «A e B colados» (bidirecional, sobre o solver da W4) · física de parâmetro→parâmetro (**medir antes** se `drive`+`spring` já o exprime) · *behaviours* um-clique como presets dos nós que já existem (`wiggle`, `oscillator`, `orbit`, `falloff`, `force-attractor`) · layout de HUD (`kasuari` 0.4.13, MIT/Apache) |

**Kill-criterion (DIRETIVA §5):** se a avaliação da pilha passar de **0,5 ms** por quadro com **1 000** itens
activos em `--profile smoke` depois da 2.ª tentativa, a fase muda de forma (cache por dependência) antes de seguir.
Na W4: se nenhum dos dois solvers resolver um sketch de **200** pontos em < **2 ms** no arrasto, o sketch fica
restrito a peças rígidas (*blocks*) e reabre-se a escolha.

## 8. Riscos medidos e por medir

- **A shell está no tecto exacto** (`189 041` = `TETO_LOC`, `architecture_the_shell_only_shrinks.rs:112`): a linha
  que chama `fase_constraints` paga-se **movendo** o mesmo número de linhas para fora — nunca subindo o tecto.
- **Família de canais nº 8 e `Driver::ConstraintPose`** são append em enum partilhado: outra linha pode pegar o mesmo
  número ⇒ anotar no handoff.
- O oráculo Blender só arranca com `LD_LIBRARY_PATH` para uma `libcudart.so.13` (o pacote do sistema perdeu-a):
  o comando está no cabeçalho do script.
