# Ray Sensor

**Estado: `em obra`** (15/11) — o dono escolheu a **B** e mandou: *«assim como o collision shape deixou de ser
componente e passou a ser objeto que aparece na hierarquia, também deve ser o Ray»*. A alça da C não foi pedida.
O texto «Como está» abaixo é o estado de PARTIDA.

## Como ficou (15/11, commit `0ff6b66e20`)

- **O raio é um OBJETO** (como a forma, ADR-0199): Add ▸ «Ray Sensor» (`ph2d-game-dicas` `add::RAY_SENSOR`,
  `ph2d_physics_ecs::pendura_raio`) — com seleção vira filho dela na origem dela, sem seleção fica na raiz.
  Sai do Add Component e da paleta de física (`il`, `PORTAS` 8 → 7). Dois raios = dois filhos.
- **Motor:** `RaySensor { angle (graus locais, 0 = direita, 90 = cima), reach, layer }`; a origem é a posição do
  objeto. `raio_em_mundo` sobe ao corpo dono (`owner_body`): exclui-o e nasce da pose VIVA dele mais o desvio do
  filho. `PROJECT_SCHEMA` 242 → 243, sem leitor velho (§0.11).
- **Cartão B** (`sections/ray.rs`): avisos · «Sees X, at Y» · Angle (dica de hover) · Reach | Layer por nome ·
  Rename/Open in tab · Sees · grupo **Signals** (aberto; `INSP_RAY_FOLD_SIGNALS`) com On Enter/On Exit e o
  sufixo «n listens / nobody listens» que já existia (a crítica B·3 já estava feita).
- **Aviso novo** «Nothing in the layers this ray sees. It can never see anything.» (`RayQueixa::CamadaVazia`,
  `ray_inspector::camada_vazia`): nenhuma forma de OUTRO corpo numa camada que a matriz deixa o raio ver; uma
  forma com exceção conta como visível (o aviso só fala quando é certo). A queixa «direção zero» saiu (um
  ângulo é sempre um rumo).
- **Hierarquia:** `ObjectType::RaySensor`, ícone `Probe`, cor `obj-ray-sensor`.
- **O objeto raio não mostra a face «Add Physics Body»** (`paint_frame.rs`, `raio_solto`): seria um controlo
  que transforma uma sonda em corpo.
- **Cena** `PH2D_GAME_SMOKE=26` (`ph2d-physics-scene/src/cena_ray.rs`): Hero (Character do Add) com «Wall Ray»
  (frente, 6 m) e «Ledge Ray» (pés, para baixo, 1,2 m), luzes «Wall Light» e «Edge Light», chão que acaba
  antes da parede e um poço. Fotos em `fotos_07/26_*`. O `PH2D_RAY_SMOKE=1` (ângulos 0/180) tem os dois olhos como filhos «Front Ray»/«Back Ray» do «Post» desde 15/11.

Estudos lidos:
- crítica `critica_ph2d.html#raysensor`, §1–§7, com o «Vale rever?» da Origin na C.
- O estudo não tem seção própria; a crítica usou só a doc das engines, sem foto.

Base do estudo: `main` `7b3dd983f7`.

## Como ficou: o filtro (versão C, 15/11, commits `8fa2181ffc` motor · `c6a6d7f98e` painel · `1d4bd82003` canvas)

- Grupo **Filter** no cartão (`sections/ray_filtro.rs`), entre Layer/Sees e Signals, nasce fechado com resumo
  («Ignores me», «Only Enemy, Sees triggers»…): Ignore my body | Ignore my other parts · Only see (Everything ·
  Only tag · All except tag, a linha de escolha da casa — segmentos, não lista: decisão do dono 23/09 sobre escolhas)
  · Tag (lista da árvore de tags) e Walls block só fora de Everything · Hit triggers | Starts inside · See all hits.
- **Ignore up to** (auditoria 15/11, decisão C8): campo de texto + conta-gotas que nomeia o objeto MAIS ALTO da cadeia
  que o raio ignora (escreve pelo nome). Vazio = automático. **Lei da família do raio:** a cadeia ignorada pára no
  personagem (PlatformPlayer/TopDownPlayer); sem personagem, no corpo mais alto. Ver [auditoria](07_auditoria_2026-11-15.md).
- Leitura com See all hits: uma linha por acerto («Sees Wall, at …», «and Box, at …», até 4, «and N more»).
- Canvas: com See all hits a linha vai até ao bloqueio (ou ao alcance inteiro) com um tique em cada acerto.
- **Cena 26 com o filtro** (`7f318581fa`): «Enemy» (tag «Enemy» criada na árvore do projecto pela porta
  `Portas::marca` → `tags_doc::marca`) atrás da «Crate», no alto; «Eye Ray» (filho do Hero, 30°, 7 m, Only tag Enemy,
  Walls block desligado) e «Eye Light» (acende com `enemy_seen`, apaga com `enemy_gone`). De pé no Play a luz do meio
  acende; o condutor do smoke fica de pé até ao quadro 360. Gate `jogo_ray` (game-scenes): vê através da caixa; com
  Walls block a caixa tapa. Foto `PH2D_GAME_SMOKE_FOTO=ray_filter`: o Eye Ray escolhido, com o Filter aberto.
- Fotos: [cartão](fotos_07/26_cartao.png) (Wall Ray, Filter fechado «Ignores me») · [filter](fotos_07/26_filter.png)
  (Eye Ray: Only tag, Tag Enemy, Walls block desligado) · [play](fotos_07/26_play_eye.png) (de pé, Eye Light acesa).
- Falta: o smoke do dono.

## Como ficou: as melhorias (RS, 15/11)

Mecanismo, números e gates: [handoff, sexta onda](../handoffs/HANDOFF_CONTINUACAO_2026-11-15_07_RAY_SENSOR.md). Em resumo:

- **Campos novos do raio** (todos canais, na Timeline): Enabled (desligar emite On Exit e cala) · Thickness (raio grosso = cápsula) ·
  Cone (leque, linha de vista à Construct) · Aim at (mira o corpo VIVO do alvo no mesmo tique, não o Transform atrasado) ·
  Also ignore (lista de objetos por conta-gotas, chips com ×).
- **Shape (dono 15/11):** lista Line · Thick · Cone; por baixo só o número da forma (Thickness ou Cone); o leque olha por linhas finas.
  Fotos: [Line](fotos_07/26_shape_cartao.png) · [Cone](fotos_07/26_shape_cone.png).
- **Arrumação (dono 15/11):** Reach | Shape lado a lado; grupo «Layers» com Layer, Sees e os botões juntos — [auditoria](07_auditoria_arrumacao_2026-11-15.md) · [foto](fotos_07/26_arrumado.png).
- **O acerto diz a forma tocada** (Collision Shape filha): as tags dela são o material, mais fricção/restituição — «Sees Ground (Ice), at …».
- **Script:** `ph2d.ray(id)` (lê o sensor, `seeing`) e `ph2d.raycast(from, to, filter)` (o tiro vê o mesmo que o raio com o mesmo filtro).
- **Canvas:** o feixe é desenhado como fatia (leque), cápsula (grosso) e círculo no acerto; o conta-gotas de Aim at / Also ignore clica no canvas.
- **Cena 26:** «Guard» com «Guard Eye» (cone 50°) acende «Alarm Light»; «Turret» atira por script (`turret.luau`, 0,5 m de largura) e acende «Shot Light».
- **Oráculo:** Godot 4.7.2 corrido sobre a nossa cena ([`oraculo_raio_rs/`](oraculo_raio_rs/)), gate `oraculo_godot_das_melhorias`.
- Fotos: [cartão](fotos_07/26_rs_cartao.png) ([clássico](fotos_07/26_rs_cartao_cards_off.png)) · [cone](fotos_07/26_rs_cone.png)
  ([clássico](fotos_07/26_rs_cone_cards_off.png)) · [play](fotos_07/26_rs_play.png) · [torreta](fotos_07/26_rs_play_turret.png).
- Falta: o smoke do dono.

## Como está (conferido no código)

- **O componente** `RaySensor` (`ph2d-physics-ecs/src/components/ray_sensor.rs`): `origin` (m, local), `dir`, `reach` (m), `layer: u8`.
  Padrão `origin 0,0 · dir 0,-1 · reach 1 · layer 0`. **`dir 0,-1` olha para BAIXO** (Y do mundo cresce para cima:
  `ph2d-physics/src/world.rs:310`, `ph2d-render/src/camera.rs:283`). A tela não o diz — confirma a queixa da crítica.
- **O lance** `cast_ray_sensors` (`ph2d-physics-ecs/src/bridge/ray_sensors.rs:143`) via `PhysicsWorld::cast_ray`
  (`world/cast.rs:107`): exclui o próprio corpo, não vê sensores/triggers. Grava `ray_enters`/`ray_exits`, que
  `signal_events` (`bridge/signals.rs:102`) torna sinais On Enter / On Exit.
- **No canvas** o raio JÁ se desenha no editor (foto abaixo): `probe_marks` (`ph2d-app-physics/src/overlay/probes.rs:66`),
  três estados (Idle 0,45 · Clear 0,72 · Hit 1,0 de alfa, tique no acerto). **Não há alça**: pegar na ponta não existe.
  O único gizmo de ponto é o das âncoras de junta (`overlay/point_gizmo.rs`) — molde para a alça.
- **O cartão** (`ph2d-panel-inspector/src/sections/ray.rs`, `corpo`): avisos · leitura viva «sees X at Y» ·
  relógio parado · Origin X·Y · Direction X·Y · Reach · Layer · On Enter · On Exit (texto). 8 valores, todos com
  leitor que decide (`ray_inspector.rs`, `ray_edits.rs`, `signals.rs`). Nenhum controlo morto.
- **Layer** é um índice `u8`, mas **já tem nomes**: o cartão usa `camada_por_nome::linha_da_camada` (`DO_RAIO`,
  `ids/inspector_ray.rs` `INSP_RAY_LAYER_OPTS`) sobre `LayerNames` (`ph2d-physics-ecs/src/layer_names.rs`).
- **Um sensor por objeto**: `RaySensor` é componente único por entidade. Dois raios = dois objetos (filhos).
- **Quem ouve o sinal**: a contagem existe (`ph2d-signal-census/src/lib.rs:224`, `listeners`) e o Timeline já a mostra
  (`ph2d-panel-timeline/src/signal_ouvintes.rs:436`). O cartão do Ray não a usa.
- **Smoke de hoje** `PH2D_RAY_SMOKE=1` (`ph2d-app-components/src/ray_smoke.rs`): dois olhos filhos do poste (`Front Ray`/`Back Ray`, gate
  `a_cena_do_olho_pendura_os_raios_como_filhos`), caixa, duas luzes, cartão Ray Sensor aberto. Próxima cena do Game Engine: `PH2D_GAME_SMOKE=26`.

![hoje](fotos_07/hoje_ray.png)

### Da crítica, o que não vale

- «Layer (inteiro)… se errar, nada acontece»: **Layer já escolhe por nome** hoje. O que falta é o aviso
  «nenhum objeto nesta camada» (B·3).
- «Pegar nele com o mouse: supus que não dá»: **confirmado, não dá**.
- «Não vi se um objeto aceita dois sensores»: **não aceita**. Isto pesa contra tirar Origin (C): o 2.º raio vive
  num filho, e a origem passa a ser a posição do filho — sem perda de poder.
- «Aviso de sinal solto só se o app listar quem escuta»: **lista**, a função já existe.

## O que as 9 engines fazem (crítica §2, só doc)

- Quase tudo é função de código (Unity, Unreal, GameMaker, Roblox, UEFN, Cocos). Peça pronta: Godot (RayCast2D,
  `target_position` = um vetor, com alça no editor) e Construct (Line-of-sight).
- Nenhuma mostra a leitura «vê X a Y» sem código. É o melhor do nosso.

## As três versões ([maquetes da crítica](file:///home/enio/Documentos/GameEngineSamples/critica_ph2d.html#raysensor))

| | Hoje | A · Looks em palavras | B · Angle + cartão Signals | C · alça na ponta |
|---|---|---|---|---|
| Campos à vista | 8 valores, em lista | 6 + More | 10 | 7 |
| Gestos: olhar à direita, 2 m | 4 | 2 | 2 | 1 (arrastar) |
| Direção | X·Y | Down·Up·Left·Right (ângulo em More) | Angle (graus) | alça / «90° · 100 px» |
| Origin | à vista | More | à vista | sai (objeto/filho) |
| Sinais | à vista | On Exit em More | cartão Signals | à vista |

## Recomendação

**B, com a alça da C no canvas** (como a crítica), nos cartões da casa:
- Angle (graus) no lugar de Direction X·Y; o motor continua com o vetor (`dir = (cos, sin)`), sem subir schema.
- Layer com nomes (já existe) + aviso novo «Nothing in this layer…».
- Cartão «Signals» com On Enter/On Exit e o aviso «This signal is not used by anything yet» (via `listeners`).
- Leitura «Sees X at Y» à vista.
- Alça na ponta do raio no canvas: arrastar muda Angle e Reach num passo de undo (molde `point_gizmo.rs`).
- Cena nova `PH2D_GAME_SMOKE=26` com o olho selecionado e o cartão aberto.
