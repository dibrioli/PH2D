# 29 — Pesquisa: NAVEGAÇÃO (inimigos que acham o caminho sozinhos) (2026-10-01)

> Pedido do dono: *«inimigos que acham o caminho sozinhos (navegação). Comece com pesquisa: busque o estado
> da arte entre todas as game engines. Depois faça planejamento detalhado.»* Três frentes em paralelo, as três
> registadas aqui: **(A)** o que os motores dão, o que os utilizadores elogiam e de que se queixam (fontes na
> web, cada afirmação com URL — o dossiê bruto, **verbatim e em inglês**, é
> [`pesquisa/dossie_navegacao_2026-10-01.md`](pesquisa/dossie_navegacao_2026-10-01.md)); **(B)** o que a NOSSA
> composição já exprime hoje (§5.0 do roteador: *medir antes de construir*), com `ficheiro:linha`; **(C)** a
> triagem de licença e o oráculo **corrido** (§0.9), com a saída colada.
>
> ⚠️ Marca **[inferência]** o que não veio directamente de uma fonte ou de uma medição. O plano sai daqui:
> [doc 30](30_plano_navegacao.md).

---

## §1 — Resposta curta

1. **Todo motor de uso geral convergiu na MESMA arquitectura de três camadas** — Godot 4.7, Unity (AI
   Navigation 2) e Unreal 5: **(a)** uma área andável feita de polígonos convexos (*navmesh*) · **(b)** um
   planeador global (A\* sobre os polígonos ⇒ um *corredor* ⇒ o *funil* que o estica até aos cantos) ·
   **(c)** uma camada local, por quadro, de desvio entre agentes (RVO/ORCA) que **ignora a navmesh** — e é
   daí que vem a queixa nº 1 de todos: *o agente é empurrado para fora da área andável e entala-se nos cantos*
   ([Godot #60354](https://github.com/godotengine/godot/issues/60354), aberto desde 2022).
2. **Os motores «simples» (Construct, GDevelop, GameMaker, RPG Maker) escolheram todos uma GRELHA com A\*** e
   **nenhum** desvio entre agentes. O que mostram a quem não programa é pouco e é a lição: tamanho de célula,
   uma borda à volta dos obstáculos, uma caixa «diagonais», um número de suavização, velocidade/aceleração/
   rotação — e **o comportamento acha E anda**, o artista nunca toca num ponto de passagem.
3. **O estado da arte para um jogo 2D visto de cima** é: a navmesh construída **exactamente** a partir dos
   polígonos (união/diferença + encolher pelo raio do agente + partir em convexos — o que o Godot 4.3+ faz com
   o Clipper2), consultada por **Polyanya** (Cui, Harabor, Grastien, IJCAI 2017: caminho **mais curto
   possível, em qualquer ângulo, sem pré-processamento**), com o caminho mantido de forma **incremental**
   (o *corredor* do Detour) em vez de recalculado por quadro, e **ORCA** para o desvio local — sabendo que o
   ORCA se entala nos cantos ([Sunshine-Hill, Game AI Pro 3](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter19_RVO_and_ORCA_How_They_Really_Work.pdf)).
4. ⭐⭐⭐ **As duas bibliotecas de geometria que o Godot usa para construir a navmesh 2D JÁ ESTÃO NO NOSSO
   `Cargo.lock`** (§6.2): o **Clipper2** (porte em Rust puro, `clipper2-rust 1.1.0`, BSL-1.0) e uma
   **triangulação de Delaunay com restrições** (`spade 2.15.1`, MIT/Apache). ⇒ a metade de GEOMETRIA desta
   feature pode custar **zero pacotes externos novos**.
5. ⭐⭐ **O oráculo corre** (§6.1): o Godot 4.7.2 (MIT, instalado) constrói a navmesh com raio, responde a
   caminhos, faz A\* em grelha e desvio entre agentes **sem interface** — com **uma** armadilha que devolve
   caminhos **vazios** em silêncio, medida e com a cura escrita.
6. **A nossa composição NÃO exprime navegação** (§5): não há nada de navegação de gameplay no repositório.
   Há um A\* de grelha **pronto e sem consumidor** (`ph2d-grid`), e as três peças em que um agente se apoia
   já existem — o mover com deslize (`TopDownPlayer`), as consultas de raio/varrimento da física, e um corpo
   que obedece a quem lhe escrever a intenção (`default_controls = false`). Os morcegos da arena perseguem por
   *homing* em linha recta e **ricocheteiam** nas paredes.
7. **Plataformas são outro problema** (a navmesh não modela a gravidade): a resposta da indústria é um grafo
   de plataformas com saltos simulados pelos parâmetros do próprio personagem — **fora** desta primeira jornada,
   nomeado no plano.

---

## §2 — (A) O que os outros dão

### §2.1 — Os motores grandes (o que o designer recebe)

| | **Godot 4.7** (MIT) | **Unity 6** (AI Navigation 2) | **Unreal 5** |
|---|---|---|---|
| **área andável** | `NavigationRegion2D` + `NavigationPolygon` (desenhada ou *baked* dos colisores) | `NavMeshSurface` (voxel, 3D; **2D só pelo NavMeshPlus da comunidade**) | `NavMeshBoundsVolume` (Recast, por mosaicos) |
| **como se constrói** | Clipper2 (união/diferença/encolher) + polypartition (convexos) — [PR #80796](https://github.com/godotengine/godot/pull/80796) | voxel Recast-like, `voxel = raio/3` | voxel Recast; Static / Dynamic / *só modificadores*; *invokers* |
| **o raio do agente** | **na malha** (uma malha por tamanho) | **por tipo de agente** (um *bake* cada) | por configuração de agente (*bake*) |
| **consulta** | A\* + funil (`CORRIDORFUNNEL`/`EDGECENTERED`) | A\* + *string pulling* | A\* + *string pulling* (Detour) |
| **agente** | `NavigationAgent2D` (devolve o próximo ponto + velocidade segura) | `NavMeshAgent` (move o transform por omissão) | `PathFollowingComponent` |
| **desvio** | RVO2 (Apache-2.0) | RVO (fechado) | RVO **ou** DetourCrowd (exclusivos) |
| **obstáculo móvel** | `NavigationObstacle2D`: *esculpe* no bake **e/ou** desvio (estático por vértices / dinâmico por raio) | `NavMeshObstacle` com *Carve* | *Dynamic* / modificadores |
| **atalhos** | `NavigationLink2D` (o jogo anima a travessia) | `NavMeshLink` | Nav Link Proxy / Smart Link |
| **custo por área** | camadas + custos de região/ligação | *areas* + custo | NavArea + filtros |
| **depuração** | *Debug → Visible Navigation* | gizmos da cena | tecla **P** |

**Os botões do `NavigationAgent2D` (Godot 4.7, valores de fábrica — [ref](https://docs.godotengine.org/en/stable/classes/class_navigationagent2d.html)):**
`path_desired_distance 20` · `target_desired_distance 10` · `path_max_distance 100` (desvio que dispara um
recálculo) · `path_postprocessing CORRIDORFUNNEL` · `simplify_path false` · `navigation_layers 1` ·
`avoidance_enabled false` · `radius 10` (**só para o desvio, não para o caminho**) · `neighbor_distance 500` ·
`max_neighbors 10` · `time_horizon_agents 1` · `time_horizon_obstacles 0` · `max_speed 100` ·
`avoidance_layers/mask 1/1` · `avoidance_priority 1`. Uso: pôr `target_position`, chamar
`get_next_path_position()` **uma vez por quadro de física** até `is_navigation_finished()`; com desvio, mover
pela velocidade do sinal `velocity_computed`. Sinais: `path_changed`, `target_reached`, `navigation_finished`,
`waypoint_reached`.

### §2.2 — Os motores 2D «simples» (o que decidiram mostrar a quem não programa)

| motor | representação | botões expostos | desvio entre agentes |
|---|---|---|---|
| **Construct 3** | grelha A\* | tamanho e borda da célula · obstáculos = sólidos ou à escolha · velocidade, aceleração, rotação · diagonais · *regenerar região à volta do objecto* | nenhum |
| **GDevelop** (MIT) | grelha A\* (+ extensão experimental de navmesh) | diagonais · aceleração · velocidade · rodar · tamanho e deslocamento da célula · borda extra · suavização (*max cell gap*); obstáculo **impassável ou com custo** | nenhum |
| **GameMaker** | grelha (`mp_grid_*`), potencial (`mp_potential_*`), linear | tamanho de célula · diagonais · quem é obstáculo | nenhum (o potencial é um remendo local) |
| **RPG Maker** | A\* de *tiles* | nenhum na interface; **procura limitada a 12 passos** | nenhum |
| Defold · Phaser · Cocos · Stencyl · LÖVE | **nada embutido** (extensões: easystar, navmesh do mikewesthad, MicroPather, Jumper) | — | — |

⭐ **O que eles decidiram, numa frase:** uma grelha que o designer consegue imaginar; o obstáculo é o próprio
objecto, inflado por uma borda; o custo é um número no obstáculo; as diagonais são uma caixa; a suavização é
um número; **o comportamento acha E conduz** (o artista nunca vê pontos de passagem); e a mudança dinâmica é
uma **acção explícita** («regenerar região»), não automática.

### §2.3 — O que se elogia (ordenado por frequência nas fontes — **[inferência]** na ordem)

1. **«Clique → o inimigo acha-te»** — o agente que acha e anda num componente (Construct, GDevelop,
   `NavigationAgent2D`).
2. **A área andável derivada dos colisores**, sem a sincronizar à mão (Godot 4.3+, `vleue_navigator`: *o
   obstáculo É o colisor*).
3. **Caminhos lisos nos cantos** (funil / *any-angle*), em vez do ziguezague da grelha.
4. **Desvio entre agentes** (multidões que não se atropelam).
5. **Custo por área** (a lama é lenta, a lava evita-se) e **atalhos** (portas, saltos, teleportes).
6. **Ver a navmesh e os caminhos** no editor.

### §2.4 — As queixas (cada uma é um gate a escrever ANTES da lei)

| # | queixa | onde | o que o nosso desenho tem de impedir |
|---|---|---|---|
| Q1 | **o desvio empurra o agente para fora da área andável / entala-o nos cantos** | Godot [#60354](https://github.com/godotengine/godot/issues/60354) · Unreal RVO «pode sair dos limites» | o desvio respeita as fronteiras da navmesh **e** o corpo desliza na parede real |
| Q2 | **o raio do agente não muda o caminho** (só o desvio) — a causa nº 1 de «preso nos cantos» | [Godot ref](https://docs.godotengine.org/en/stable/classes/class_navigationagent2d.html) · [proposta #5977](https://github.com/godotengine/godot-proposals/issues/5977) | o raio sai do **corpo** e decide o caminho; um tamanho novo não pede trabalho ao artista |
| Q3 | **um mapa por tamanho de agente** | [Godot](https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_different_actor_types.html) · Unity *agent types* | as malhas por raio derivam-se sozinhas |
| Q4 | **caminho vazio no 1.º quadro / depois de mudar o mapa** (sincronização assíncrona) | [Godot tutorial](https://docs.godotengine.org/en/latest/tutorials/navigation/navigation_using_navigationagents.html) · [#104283](https://github.com/godotengine/godot/issues/104283) — **e medido aqui (§6.1)** | caminho no tique 1, com gate |
| Q5 | **ultrapassar / voltar atrás** quando o passo por quadro é maior que o raio de aceitação do ponto | mesmo tutorial | a aceitação depende de `velocidade × dt` |
| Q6 | **«dançar» por recalcular o caminho a cada quadro** | mesmo tutorial · GDevelop («corra o *Move to* uma vez») | política de recálculo explícita; recalcular por quadro é anti-padrão documentado |
| Q7 | **bloqueio simétrico / ORCA preso nos cantos** | Sunshine-Hill (Game AI Pro 3) | biblioteca de cenários automática (cruzamento, porta, canto, frente-a-frente) |
| Q8 | **agente e obstáculo no mesmo objecto** (ele desvia de si próprio) | [Unity](https://docs.unity3d.com/2020.1/Documentation/Manual/nav-MixingComponents.html) | o agente nunca é obstáculo de si mesmo, por construção |
| Q9 | **células grossas fecham passagens** · **obstáculo pela caixa e não pela forma** | Construct · GameMaker · [GDevelop #2689](https://github.com/4ian/GDevelop/issues/2689) | geometria exacta (navmesh), sem células |
| Q10 | **procura curta pára o agente à frente da parede, calado** | RPG Maker `searchLimit = 12` | sem limite escolhido; tecto **medido**, e se houver recusa, ela fala |
| Q11 | **lento nos cantos quando a velocidade real da física não volta ao agente** | [landmass FAQ](https://github.com/andriyDev/landmass/blob/main/FAQ.md) | o agente lê a velocidade REAL do corpo |
| Q12 | **alvo fora da malha / agente fora da malha ⇒ nenhum caminho, nenhum movimento, nenhuma mensagem** | landmass FAQ · Godot caminhos vazios | estados explícitos com voz: *sem região · alvo fora · inalcançável · preso* |
| Q13 | **reconstruir em execução custa a leitura da geometria na thread principal** | Godot docs | custo **medido** antes de escolher síncrono vs. por mosaicos |

---

## §3 — Estado da arte contra legado

**Estado da arte (2026), 2D visto de cima:**
- navmesh de **geometria exacta** por booleanas + recuo robusto + triangulação com restrições / partição convexa
  (Godot 4.3+; em Rust `i_overlay` + `i_triangle`, ou — **o que já temos** — `clipper2-rust` + `spade`). Melhor
  que voxel em 2D: arestas exactas, sem escadinha, reconstrução barata.
- **Polyanya** para o caminho óptimo em qualquer ângulo sem pré-processamento — o A\*+funil é óptimo só
  **dentro do corredor escolhido**, não globalmente ([artigo](https://www.ijcai.org/proceedings/2017/0070.pdf)).
- **Corredor** mantido ([`dtPathCorridor`](https://recastnav.com/classdtPathCorridor.html)) em vez de recálculo
  periódico.
- **ORCA** com consciência do modo de falha nos cantos e um banco de cenários automático.
- Reconstrução em segundo plano com duas iterações do mapa (Godot 4.4+).
- **Campos de fluxo** por mosaicos (Supreme Commander 2) quando centenas de agentes partilham o destino.

**Legado / limitado:** grelha A\* com suavização a posteriori (os simples) · grafos de pontos postos à mão ·
só campo potencial (mínimos locais) · VO/RVO sem amortecimento · voxel (Recast) forçado num plano 2D.

---

## §4 — As decisões que todo motor teve de tomar (e o que cada um escolheu)

| decisão | Godot | Unity | Unreal | simples 2D | **nota para nós [inferência]** |
|---|---|---|---|---|---|
| grelha ou navmesh | **as duas** | navmesh | navmesh (+faixas ZoneGraph) | grelha | geometria livre ⇒ navmesh; grelha é do projecto **Tilling** (ordem do dono) |
| *bake* ou em execução | as duas; *bake* assíncrono por regiões | as duas; *carving* | Static / Dynamic / modificadores | regenerar a pedido | a malha é **derivada** dos colisores — medir se cabe síncrona no tique |
| raio do agente | na malha (1 mapa por tamanho) | por tipo de agente | por configuração | borda da grelha | **ninguém** entrega raio por agente numa malha só — é onde se pode superar |
| desvio | RVO global, camadas | global, prioridade | RVO ou multidão | nenhum | o desvio tem de respeitar a malha (Q1) |
| recálculo | mudança de alvo + desvio > limite | corredor + recálculo automático | corredor | a pedido | nunca por quadro (Q6) |
| alisar | funil, centro da aresta, simplificar | *string pulling* | *string pulling* | suavização por número | *any-angle* óptimo dispensa alisar |
| quem move o corpo | devolve próximo ponto + velocidade segura | o agente move o transform | o componente de seguir conduz | o comportamento move | **o agente decide, o mover executa** — o `landmass` devolve só a velocidade desejada e nunca move o agente, exactamente para conviver com uma física dona da pose |
| atravessar atalhos | o jogo | auto (simples) ou manual | eventos do Smart Link | — | atalho = aresta do grafo; o movimento é do jogo |

---

## §5 — (B) O que a NOSSA composição já exprime (medido no código)

### §5.1 — Navegação de gameplay: **nenhuma**

- O único A\* é o do [`ph2d-grid/src/astar.rs`](../../crates/ph2d-grid/src/astar.rs) (`pub fn astar` :70):
  determinístico (`BTreeMap`), **custo uniforme 1 por passo**, `is_blocked` por callback, 8 testes — e **sem
  consumidor nenhum** (só o `ph2d-editor-core::grid_snap` usa a crate, e nunca o `astar`).
  ⚠️ O doc de `square.rs:16-24` diz que *«as camadas de A\* recebem um callback de custo que pode devolver
  `SQRT_2`»* — **é falso**: o `astar.rs` não tem callback de custo. *Uma promessa num doc sem gate.*
- O que **parece** navegação e não é: o `Pathfinder` do `ph2d-vec-boolean` (o painel do Illustrator), o
  `avoid.rs` do nó `motion.boids` (partículas, não entidades), Dijkstra na retopologia e na geodésica.
- `docs/Components/25_tabela_comparativa_godot.md:112` já o marcava: as sete classes de navegação do Godot
  contra **«nada»**, ⭐⭐.

### §5.2 — Os morcegos da arena perseguem por *homing* e batem nas paredes

[`vida_arena_smoke.rs:264-273`](../../crates/ph2d-app-components/src/vida_arena_smoke.rs) — `ProjectileMotion`
com `homing_accel 5`, `max_bounces 255`, ricochete. A lei
([`ph2d-projectile/src/lib.rs:140-170`](../../crates/ph2d-projectile/src/lib.rs)) aponta para onde o alvo
**está agora**, sem antecipar (recusa declarada em :160-163). ⇒ com o herói atrás de uma parede, o morcego
encosta-se nela e ricocheteia; **não a contorna**.

### §5.3 — As peças em que um agente se apoia **já existem**

| peça | onde | o que dá ao agente |
|---|---|---|
| **o executor** | `TopDownPlayer` ([`components/topdown.rs:49-74`](../../crates/ph2d-physics-ecs/src/components/topdown.rs), lei em `ph2d-topdown`) — velocidade, aceleração, rotação, **deslize na parede à velocidade cheia** | o agente **decide** a direcção; quem anda é ele (*«um executor, várias fontes»* — a lição do *Move To* do Construct, já escrita em [`00_levantamento`](00_levantamento_componentes.md) :122/:269) |
| **obediência** | com `default_controls = false`, `reads_the_keyboard` devolve `false` ([`keyboard_driven.rs`](../../crates/ph2d-physics-ecs/src/keyboard_driven.rs)) e o corpo é *«um motor PURO, obediente a quem lhe escrever a intenção»* ([`player_channel.rs:79`](../../crates/ph2d-physics-ecs/src/bridge/player_channel.rs)) | a intenção de navegação entra pelo canal que já existe |
| **o mundo bloqueia?** | `cast_ray` ([`world/cast.rs:105`](../../crates/ph2d-physics/src/world/cast.rs)), `sweep_body` (`sweep.rs:86`), `move_character_from` (`character.rs:227`), camadas `groups_for` (`layers.rs`) | linha de vista, folga, e o corpo **não entra na parede real** mesmo que a navegação erre |
| **o alvo** | por **nome** (`stable_name_id`, como o *homing* e a câmera) · por **tag** (`ph2d_ecs::tags::tagged`) | quem perseguir, sem bits de entidade |
| **os sinais** | `SignalOrigin` (append-only) + a tabela de acções (#5) + a `StateMachine` (#15) | `chegou` / `sem caminho` / `preso` viram gatilhos de graça |
| **o anel** | `player_state`/`topdown_state`/`projectile_state` com `record`/`seed`/`rebuild_from_rest` ([`bridge/tape.rs`](../../crates/ph2d-physics-ecs/src/bridge/tape.rs), [`rewind.rs:157`](../../crates/ph2d-physics-ecs/src/bridge/rewind.rs)) | o agente volta exacto num *scrub* |
| **a geometria** | colisores `Ball`/`Cuboid`/`Capsule` + corpos compostos ([`components.rs:98-116`](../../crates/ph2d-physics-ecs/src/components.rs), [`parts.rs`](../../crates/ph2d-physics-ecs/src/parts.rs)) | a fonte da área andável (⚠️ **não há colisor de polígono**, nem um `VecPath` vira colisor) |

### §5.4 — O que a composição **NÃO** exprime (e é o que se constrói)

| falta | medido |
|---|---|
| **uma área andável** derivada dos colisores | nenhum tipo, nenhuma crate (`grep -ri navmesh` só acha docs) |
| **um planeador** sobre ela | o A\* de grelha não tem consumidor nem custo; não há triangulação com furos fora do `wgpu` (o `triangulate_even_odd` do `ph2d-flip-render/src/fill_holes.rs:79`) |
| **um agente** que transforme *«quero chegar ali»* numa intenção **por tique** | nenhum dos **12** verbos da tabela move nada (`signal_actions.rs:190-203`); um script Luau não alcança nenhum A\* |
| **desvio entre agentes** | nada (o `avoid.rs` dos boids é do grafo de partículas) |
| **custo por área / atalhos** | nada |
| **ver** a navegação | nada |

⚠️ **Uma armadilha medida que molda o desenho:** a fita de entrada grava **UM** `PlayerInput` por tique e só o
distribui aos corpos lidos pelo teclado ([`tape.rs:111`](../../crates/ph2d-physics-ecs/src/bridge/tape.rs),
:346-372) ⇒ um `set_player_input` vindo de fora para um `TopDownPlayer` com `default_controls = false`
**não é gravado** e um *scrub* não o repete. ⇒ **a intenção do agente tem de ser calculada DENTRO do tique, a
partir do estado do mundo** — como o *homing* faz.

---

## §6 — (C) Triagem de licença e o oráculo corrido

### §6.1 — O Godot 4.7.2 corre sem interface, nas cinco perguntas

`/usr/bin/godot`, pacote `godot 4.7.2-1.1`, **MIT** (sem parede: porta-se, com atribuição). As bibliotecas
**compiladas dentro do binário**, lidas do próprio binário por `Engine.get_copyright_info()` (o pacote só traz
o `LICENSE` principal — *a unidade da triagem é o ARTEFACTO*, §0.9):

| componente | licença |
|---|---|
| Clipper2 | BSL-1.0 |
| PolyPartition | Expat |
| Recast | Zlib |
| RVO2 | Apache-2.0 |

Sondas versionadas em [`ferramentas/godot_nav_sonda/`](ferramentas/godot_nav_sonda/) (com a saída colada;
**sondas de TRIAGEM, não fixturas** — a W0 do plano forma o corpus com cabeçalho):

- **(a) Construir com raio funciona e é síncrono.** Rectângulo andável `(0,0)–(400,300)`, obstáculo
  `(150,100)–(250,200)`, `agent_radius = 10` ⇒ 8 vértices, 4 quadriláteros; a borda exterior encolhe para
  `(10,10)–(390,290)`, o furo cresce para `(140,90)–(260,210)` **com quinas VIVAS, não arredondadas** ⭐
  (o recuo é em esquadria, não a soma de Minkowski com um disco — §6.3).
- **(b, c) O caminho funciona, depois de a região sincronizar:**
  ```
  init after force n=4 (49.999996185, 150.000000000) (140.000000000, 210.000000000) (260.000000000, 210.000000000) (350.000000000, 150.000000000)
  init off-mesh (200,150)->(395,295) n=2 (200,210) (390,290)    ← as duas pontas presas ao ponto mais perto da malha
  init y=151 → por baixo; y=149 → por cima                      ← empate simétrico desfeito pela decomposição da malha
  ```
- **(d) A\* em grelha (`AStarGrid2D`) responde no mesmo quadro**, nos quatro modos de diagonal, com e sem saltos
  (JPS); um alvo inalcançável dá `[]`, e com `allow_partial_path` dá o caminho até ao mais perto.
- **(e) O desvio funciona nas duas formas** (API do servidor e nó `NavigationAgent2D`): o *callback* corre uma
  vez por quadro de física a partir do quadro 1; dois agentes de raio 10 frente a frente passam a `~19–20 px`.

⛔⛔ **A armadilha que dá saída plausível e errada:** as iterações assíncronas nascem LIGADAS ⇒ num mapa novo o
`map_force_update` não faz nada, e `map_get_path`/`query_path`/`map_get_closest_point` devolvem **vazio (ou
`(0,0)`) sem erro** até à iteração 2 (~quadros 4–5). A cura que funciona é
`region_set_use_async_iterations(reg, false)` **antes** de pôr o polígono. ⭐ *É a queixa Q4 medida no próprio
oráculo — e é o gate que o nosso desenho tem de passar: caminho no tique 1.* Outras: `agent_set_position` no
`_initialize` é diferido (comece a gravar depois da 1.ª sincronização) · o desvio precisa de quadros de física
(`--fixed-fps 60`) · ruído de `f32` (`49.999996185` em vez de `50`) ⇒ tolerância nos gates.

### §6.2 — ⭐⭐⭐ A geometria do Godot JÁ está no nosso `Cargo.lock`

| pacote | versão | licença | quem o traz hoje | o que dá |
|---|---|---|---|---|
| `clipper2-rust` | 1.1.0 | **BSL-1.0** (porte em Rust puro do Clipper2) | `manifold-rust` ← `ph2d-mesh-bool` | união/diferença **e recuo (*offset*)** com núcleo inteiro — *o mesmo motor do bake 2D do Godot* |
| `spade` | 2.15.1 | MIT/Apache | `parry2d` ← `rapier2d` | **triangulação de Delaunay com restrições** (`ConstrainedDelaunayTriangulation`) |

⇒ o passo de construção do Godot (Clipper2 + partição convexa) reproduz-se com dependências **já
resolvidas**; o que falta escrever é nosso (a partição em convexos sobre a triangulação, ou a consulta
directa sobre triângulos). ⚠️ **A medir na W0, não a assumir:** o `spade` usa `HashSet`/`HashMap` (no
`flood_fill_iterator.rs` e no `refinement.rs`; o `clipper2-rust` não tem nenhum fora dos testes) ⇒ a saída
tem de ser **bit-idêntica entre duas corridas em processos diferentes**, senão ela não pode entrar no tique da
física (o `physics_ecs_c9` compara os três sistemas operativos).

### §6.3 — As outras portas

- **GDevelop 5.6.282** (MIT, instalado, com arnês sem interface já no repo:
  [`ferramentas/gdevelop_health/`](ferramentas/gdevelop_health/)) traz `PathfindingBehavior` (grelha A\*) e
  `NavMeshPathfinding` (sobre `recast-navigation.wasm`) — oráculo da **grelha** e do comportamento «acha E
  anda» dos motores simples.
- **Unity 6000.7.0b1** e **Unreal 5.8.2** estão instalados e são **fechados** (só oráculos corridos, nunca
  lidos). ⚠️ A árvore do Unreal tem uma cópia do Recast com `Recast-License.txt`; o *hook* do repo bloqueou a
  leitura, e a licença exacta daquela cópia fica **por medir** (decisão do dono ou de uma janela E — e
  desnecessária: o Recast original é Zlib e o Godot já o traz).
- **Rust (MIT/Apache):** `polyanya` (Polyanya, deps `spade`/`geo`/`glam 0.32`/`rstar`) · `landmass` (agentes +
  funil + ORCA via `dodgy_2d`, núcleo sem Bevy, **devolve a velocidade desejada e nunca move o agente**) ·
  `i_overlay`/`i_triangle` · `pathfinding` (A\* genérico). ⛔ `vleue_navigator` arrasta `bevy` inteiro (*mesh*,
  câmera, *assets*), `oxidized_navigation` está **descontinuado**, `rerecast` é voxel 3D. **Não há** `rvo2`
  mantido em Rust.
- **Artigos** (A\*, JPS, Theta\*, Polyanya, HPA\*, ORCA, campos de fluxo): ideias não são protegidas;
  implementar a partir do artigo é limpo.

⭐ **O que §6.1 mostrou de graça e o plano aproveita:** o recuo do Godot é em **esquadria** (as quinas do furo
ficam vivas) ⇒ o agente circular nunca corta a quina tão perto quanto podia. A soma de Minkowski com um
**disco** é a folga exacta de um corpo circular e dá caminhos **mais curtos** nos cantos — é uma divergência
que se mede, e o Clipper2 que já temos faz as duas (*join* redondo ou em esquadria).

---

## §7 — Recomendação (o plano sai daqui, [doc 30](30_plano_navegacao.md))

**Três peças, na arquitectura de três camadas que toda a indústria escolheu, com as queixas do §2.4 escritas
como gates ANTES da lei:**

- **A área andável (`NavRegion`)** — derivada dos colisores estáticos (*o obstáculo É o colisor*, sem
  sincronizar nada à mão), recuada pelo raio do **corpo** do agente (uma malha por raio, **derivada sozinha** —
  Q2/Q3), construída com o Clipper2 e a triangulação que **já estão** no repositório, e **reconstruída quando a
  geometria muda**, com o custo medido antes de escolher síncrono ou por mosaicos (Q13).
- **O planeador** — **Polyanya** (o caminho mais curto possível em qualquer ângulo), numa crate-folha de lei
  pura (o molde da `ph2d-health`/`ph2d-topdown`), com **dois** oráculos: o Godot corrido (*nunca mais longo que
  ele*) e o **grafo de visibilidade exacto** escrito por nós (o caminho mais curto entre obstáculos poligonais é
  um teorema: ele passa pelos vértices — dá a resposta **exacta** de graça).
- **O agente (`NavAgent`)** — no **tique da física**, a calcular a intenção a partir do estado do mundo (§5.4),
  a entregá-la ao `TopDownPlayer` (que já desliza nas paredes), com estado no **anel** (o *scrub* volta
  exacto), política de recálculo explícita (Q6), aceitação por `velocidade × dt` (Q5), e **estados com voz**:
  *sem região · alvo fora da malha · inalcançável · preso* (Q12), como sinais para a tabela e para a máquina de
  estados.
- **Depois**, em waves próprias: o **desvio ORCA** que respeita as fronteiras da malha (Q1, Q7) · obstáculos
  móveis e portas · **custo por área** (e a lava da Vida e Dano evitada sozinha) e **atalhos** · ver a
  navegação no canvas · a arena dos morcegos a contornar as paredes · o tutorial em PDF.

⛔ **Fora, com motivo:** a **grelha** (é do projecto Tilling; o `ph2d-grid` fica no editor) · **plataformas
com saltos** (outro problema, plano próprio) · **campos de fluxo** antes de a medição mostrar que N consultas
não cabem no quadro · **recalcular por quadro** (anti-padrão documentado) · o agente **mover o corpo** ele
próprio (seria um quarto mover a duplicar o `TopDownPlayer`).
