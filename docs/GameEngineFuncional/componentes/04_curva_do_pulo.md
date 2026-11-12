# Curva do pulo no canvas (Platform Player, versão C no canvas)

**Estado: `em obra`.** Dono 10/11: **«B, mas com linhas coloridas»** — pulo inteiro forte; Short, Standing e
Air jump finos e tracejados, cada um na SUA cor (tokens), nome ao passar o mouse. O preset conserva a altura
(recomendação da ficha; acabamento = estado da arte). Pedido do dono 10/11
(`project-memory/project_jump_arc_on_canvas_2026_11_10.md`): a ideia C da crítica
(`critica_ph2d.html#character`, «Puxe o pulo») entra, mas **no canvas**. No cartão: só o botão que liga a curva
e o seletor **Floaty · Snappy · Heavy**. Base `bc4a6ce930`, sobre o Character versão B ([03](03_character.md)).
Maquete: [`f4_maquete.html`](f4_maquete.html) · [foto](f4_maquete.png).

## Como está (conferido no código)

- **A lei do pulo** — `ph2d-platformer/src/jump.rs:278` `jump_step(...)`. O pulo é parametrizado por ALTURA:
  `v₀ = √(2·|g|·h)` uma vez, no salto (`jump.rs:1-10`, `:437`). Depois a gravidade é `(escala − 1)·g` extra, por
  fase (`jump.rs:567-583`): subindo acima de `peak_speed` → `takeoff_gravity` (acima de `takeoff_speed`) ou `1`;
  com o botão solto na subida → `cut_gravity` (`:550`, não desarma); `|v| ≤ peak_speed` → `peak_gravity`;
  descendo → `fall_gravity`. O passo do jogador inteiro é `lib.rs:180` `player_motor(...)`; o andar no ar é
  `walk.rs` (`air_acceleration`, padrão 20).
- **Consequência medida (antes de 12/11; hoje o ápice É a Jump Height):** a altura REAL do ápice ≠ `jump_height` (o doc do módulo avisa: «altura de gravidade
  neutra»). Com os padrões, 2,00 m autorados sobem 2,04 m. **Não há** função que derive o arco (ápice, tempo,
  alcance); a única medição é por simulação num teste ignorado (`ph2d-physics-ecs/tests/it/platform_jump.rs:281`).
- **Parâmetros** — `JumpConfig` (`jump_config.rs:89`, padrões `:234`: H 2 · Air Jumps 0 · Air H 2 · Takeoff G 1
  acima de 0 · Peak G 0,5 janela 1,5 · Fall G 2 · Cut G 4). O componente gravado é `PlatformPlayer`
  (`ph2d-physics-ecs/src/components/player.rs:63`, com `speed`, `acceleration`, `air_acceleration`). **Nenhum
  preset** existe hoje.
- **Overlays da física** — `ph2d-app-physics/src/overlay/outline.rs:205` `outline::draw(...)`, chamado da shell por
  `shells/desktop/src/render_loop/fase_physics_overlay.rs:108`. Espaço do MUNDO projetado pela câmara
  (`Camera2d`), `BezPath` com `Tinta`, tracejado `joints.rs:388`. Cores em `const` RGBA marcadas
  `LITERAL-COLOR-OK`, tamanhos `*_PX` marcados `LITERAL-PX-OK`. Os marcadores do player (`probes.rs:66`) já vivem
  aqui.
- **Alças no canvas** — o molde é o da âncora de junta: `overlay/point_gizmo.rs` publica as alças
  (`joint_param_handles :115`) no `hero.gizmo.point_hit_map`; `resolve_anchor_hit :334`; o Down entra em
  `shells/desktop/src/input_dispatch/despacho_clique_gizmo.rs:~300-330`; o arrasto é
  `ph2d-app-physics/src/joint_anchor_drag.rs:256`, e o undo é UM diff no soltar (`post_frame_undo` suprime com o
  botão preso, `joint_anchor_drag.rs:25`).
- **Cartão** — `ph2d-panel-inspector/src/sections/player_cartoes.rs` (592 LOC, perto do tecto; `player_grupos.rs`
  já é o corte). Cena do smoke: `ph2d-app-game/src/smoke.rs:300` (`PH2D_GAME_SMOKE=4`); foto `player_cartoes` em
  `ph2d-physics-scene/src/fotos_dos_cartoes.rs:176`.

## O desenho

### Cartão Platform Player (dentro do Jump, versão B)

- **`Show jump arc`** — CHECKBOX (padrão da casa, nunca pílula). Liga a curva no canvas para o player selecionado.
  Estado do EDITOR (como mostrar colisores), não do jogo: não entra no `PlatformPlayer`, não sobe o schema.
- **`Feel` Floaty · Snappy · Heavy** — seletor de três. **Derivado, não gravado**: aceso o preset cujos números
  batem com os do player; se o artista mexer à mão num número de gravidade em «More Jump», nenhum acende
  (o seletor nunca mente). Escolher um preset escreve as cinco gravidades **e re-resolve `jump_height` para o
  ápice ficar onde estava** (a sensação muda a forma e o tempo, nunca a altura que o artista puxou). Sem isso,
  Heavy baixaria um pulo de 2 m para 1,49 m (medido, tabela abaixo). Um diff de undo.

### Canvas (com o player selecionado e a caixa marcada)

Origem: os pés do player, para o lado em que ele olha. Cada curva termina onde toca o terreno da cena (12/11,
abaixo).

| Curva | O que simula (a LEI, 60 Hz, gravidade do mundo) |
|---|---|
| **Full** (com a alça no topo) | botão preso, correndo a `speed` |
| **Short** | botão solto no 1.º tique ⇒ `cut_gravity` |
| **Standing** | parte parado, direção presa no salto ⇒ `air_acceleration` até `speed` |
| **Air jump** ×N | um por `air_jumps`, cada um disparado no ápice do anterior, com `air_jump_height` |

Só aparece o que existe: sem Air Jumps, sem curva de ar.

**A alça do topo** fica EXATAMENTE no ápice simulado (não no `jump_height`). Arrastar:
- **para cima/baixo** → `jump_height` resolvido por bisseção para o ápice cair no cursor (o ápice é monótono em `h`);
- **para o lado** → `speed = x_ápice / t_ápice` (o tempo até ao topo só depende da vertical, então a conta é exata e
  o pulo chega lá).
Ao lado, enquanto arrasta: `200 px · 600 px/s` (unidade do projeto, porta `unidade_mundo`). Um undo no soltar. O
cartão segue (os números mudam ao vivo).

**Maquete** ([html](f4_maquete.html) · [foto](f4_maquete.png)) — a única escolha que é do dono é **como se lê**:
- **A** — as quatro curvas fortes, cada uma com cor e nome.
- **B** — só o pulo inteiro forte; as outras finas e tracejadas, nome ao passar o mouse.
Recomendo **B**: a alça e o pulo que se puxa são o assunto; as outras são referência. As cores vão por tokens.

### Onde mora (drop-crate, shell só compõe)

- `ph2d-platformer/src/arc.rs` (novo, puro): `jump_arcs(cfg, walk, gravity, dt, facing) -> JumpArcs` e
  `solve_height_for_apex`. Chama o MESMO `jump_step` + o passo do ar do `walk` num laço (sem cópia da lei).
- `ph2d-platformer/src/feel.rs` (novo): os três presets + `feel_of(cfg) -> Option<Feel>`.
- `ph2d-app-physics/src/overlay/jump_arc.rs` (novo): desenho + publicação da alça; `jump_arc_drag.rs` (novo): o
  arrasto pelo molde `joint_anchor_drag`.
- `ph2d-panel-inspector/src/sections/player_pulo.rs` (novo): a caixa e o seletor (o `player_cartoes.rs` não cresce).
- Shell: uma linha no `fase_physics_overlay.rs` e no despacho do clique, compensada por corte (catraca
  `the_shell_only_shrinks`).

## Valores Floaty · Snappy · Heavy (decisão técnica; vão ao smoke)

Referências: Celeste (topo alongado, queda rápida, corte forte), Hollow Knight (pesado, pouco topo), Mario (Floaty).
Simulado com a lei (Jump Height resolvido para ápice 2 m, Speed 6 m/s, 60 Hz):

| Feel | Takeoff G (acima de) | Peak G (janela) | Fall G | Cut G | No ar | Alcance correndo | Pulo curto |
|---|---|---|---|---|---|---|---|
| Floaty | 1,0 (0) | 0,45 (2,0) | 1,4 | 2,5 | 1,67 s | 9,9 m | 1,01 m |
| Snappy | 1,6 (4) | 0,6 (1,2) | 2,6 | 5,0 | 1,27 s | 7,5 m | 0,54 m |
| Heavy | 1,3 (0) | 1,0 (0,8) | 3,2 | 4,0 | 1,02 s | 6,0 m | 0,63 m |
| *(padrão de hoje)* | 1,0 (0) | 0,5 (1,5) | 2,0 | 4,0 | 1,45 s | 8,7 m | 0,54 m |

O padrão de hoje não é nenhum dos três (o seletor nasce apagado). Ponto de dúvida para o smoke: o Heavy tem pulo
curto mais alto que o Snappy (Cut G menor); se o dono sentir, sobe o Cut G do Heavy.

## Provas (por nome, cada cura com mutação)

1. **A curva é a da lei:** `o_arco_do_desenho_e_o_do_jogo` — o `jump_arcs` contra o player REAL na ponte
   (rapier, mola, cápsula) num chão plano: ápice, tempo no ar e alcance dentro de 1 tique/1 cm. Mutações: trocar
   `cut_gravity` por `fall_gravity` no desenho; `dt` 1/30; esquecer a aceleração no ar.
2. **A alça cai no ápice:** depois de arrastar para (x, y), o ápice simulado está em (x, y) ± 1 px
   (mutação: escrever `jump_height = y` direto).
3. **O preset conserva a altura** e `feel_of` devolve o preset escolhido; mexer num número apaga o seletor.
4. **Clique REAL:** Down na alça pelo despacho do canvas → arrasto → Up = um diff de undo; Ctrl+Z volta aos dois
   números. A caixa e o seletor pelo clique, não por `set_*`.
5. **Animação abrir/fechar** da dobra Jump com as linhas novas (a fundação `corte_da_dobra`).
6. **Custo:** simular as curvas por quadro (~4 × 100 tiques) medido no smoke; se passar de 0,1 ms, só re-simula
   quando um número muda.

## Aberto (para o dono decidir ou depois)

- **A ou B** na maquete (recomendo B).
- ~~O arco termina na linha dos pés~~ e ~~Jump Height 2,00 × 2,04~~: fechados em 12/11 (abaixo).

## O que foi construído (10/11, aguarda o smoke do dono)

- **Lei:** `ph2d-platformer/src/arc.rs` (`jump_arcs`, `full_arc`, `height_for_apex`, `speed_for_apex`) corre o
  `player_motor` com a integração da ponte (`v += accel·dt + boost`) e os **4 sub-passos** do mundo (a 1.ª versão sem
  sub-passos ficava 6 cm abaixo do jogo — medido). `feel.rs`: Floaty · Snappy · Heavy + `feel_of` exacto.
- **Componente:** `PlatformPlayer.show_jump_arc` (schema 233); `components/player_arc.rs` (`set_feel`, `pull_apex`,
  `arc_world` das `PhysicsSettings`).
- **Cartão:** `ph2d-panel-inspector/src/sections/player_pulo.rs` — caixa «Show Jump Arc» e seletor Feel no cartão Jump.
- **Canvas:** `ph2d-app-physics/src/jump_arc.rs` — desenho a partir do chão sob o player (centro − `float_height`),
  alça `PointHandleKind::JumpApex` (diâmetro padrão de gizmo), arrasto (undo = um diff no soltar), nome da curva no
  hover a 9 px, readout `200 px high · 600 px/s` a arrastar. Só com o relógio PARADO.
- **Cores (tokens):** Full `ObjPlayer` (forte) · Short `Warn` · Standing `Info` · Air `ObjMacro` (finas, tracejadas).
  O `Accent` foi recusado: no tema escuro sai azul e confunde-se com o `Info` (foto).
- **Medido:** gate `o_arco_do_desenho_e_o_do_jogo` (12 casos: padrão + 3 sensações × correndo, parado, curto) —
  altura ≤ 1,3 cm, pouso no MESMO tique, alcance < 1 mm, lado do ápice ≤ 1 tique de corrida (topo achatado empata
  dois tiques). Custo: **11,8 µs** por `jump_arcs` (release, Air Jumps 1), 2 por quadro.
- **Fotos:** [`fotos_04/curva_no_canvas.png`](fotos_04/curva_no_canvas.png) ·
  [`fotos_04/curva_zoom.png`](fotos_04/curva_zoom.png) · [`fotos_04/curva_e_cartao.png`](fotos_04/curva_e_cartao.png)
  (`PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_FOTO=fisica_player_pulo`: caixa marcada e 1 pulo do ar).

## 12/11 — o terreno e a altura real

- **A curva pousa no terreno** (`ced6a4ac0a`): a lei desce abaixo do arranque (`MAX_FALL_M` 24) e cada curva é
  cortada no primeiro toque com as formas sólidas da cena (`ph2d-physics-ecs/src/arc_terrain.rs`, varredura
  `ph2d-physics/src/world/arc_cast.rs`): sem sensores, sem formas desligadas, sem o player e as peças dele, pela
  matriz de camadas, mão única só por cima. O painel pinta um **anel de pouso** na cor de cada curva.
- **O ápice real é a Jump Height** (`f3a92736ab`): `launch_speed` resolve v₀ pelas bandas de gravidade; o cartão
  diz 2,00 e a alça fica a 2,00.
- **Fotos** (`PH2D_GAME_SMOKE=4`): [`curva_terreno.png`](fotos_04/curva_terreno.png) ·
  [`curva_terreno_zoom.png`](fotos_04/curva_terreno_zoom.png) com `PH2D_GAME_SMOKE_FOTO=fisica_player_pulo_terreno`
  (o Player sobre a «Platform Low», 2,1 m antes da borda: o Full desce ao chão da direita, o Standing pousa na
  «Platform High», o Short pára na parede dela) · [`curva_e_cartao.png`](fotos_04/curva_e_cartao.png) refeita
  com `fisica_player_pulo`. Gate da pose: `a_foto_do_terreno_desce_ao_chao_e_pousa_na_plataforma_alta`
  (`ph2d-physics-scene`; mutações recuo 0,5 m e pose não movida ⇒ vermelho).
- **Custo medido:** `launch_speed` 0,20 µs por chamada; lei + corte **54,3 µs** por chamada com 3 formas e Air
  Jumps 1 — 2 chamadas por quadro, só com a curva ligada e o relógio parado. Escala com o número de formas: se
  um nível grande o mostrar, cache por (config, pose, versão das formas).
- **Visto nas fotos (para o smoke):** o anel do Short fica no ar, ~0,2 m antes e abaixo do canto da «Platform High»:
  o anel marca o PÉ, e quem bate é o topo da cápsula (pé + 0,8 m) contra o canto de baixo — leitura certa da lei,
  mas pode ler-se como «parou do nada». O pulo do ar sai pela direita da vista (pousa além dos 13 m, atrás do
  Inspector) e a vista é larga (16 m: o smoke centra no Player selecionado).
