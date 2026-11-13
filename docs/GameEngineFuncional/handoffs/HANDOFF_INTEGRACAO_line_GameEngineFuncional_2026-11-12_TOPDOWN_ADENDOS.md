# Adendos anteriores — `line/GameEngineFuncional`, 12/11 (movidos do [handoff Top-Down](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-12_TOPDOWN.md))

## Adendo — Rodada dos abertos (12/11) · `git log --oneline 3ee0bd34f4..HEAD`

Mecanismo no código e nas mensagens dos commits; aqui só o que o integrador precisa.

### A.1 Commits

- `c6c0848c00` — `HORARIO_24` com trechos só ←/↓/↑ (11 gestos); gate `o_horario_do_play_anda_so_para_cima_e_so_para_a_esquerda`; `fotos_05` refeitas no app.
- `9c1f51706b` — `scripts/collision-surface.sh`: a sonda dos espelhos `ph2d-render`/`ph2d-script` (`so_ecs.len() + K`) era CEGA (`val()` não lia `+`, desde `be050b8807`, 07/11, já no main). Prova `scripts/processo.prova.d/C.sh`. **O integrador beneficia:** a coluna destes espelhos passa a valer.
- `f3a92736ab` — **LEI DO PULO MUDOU (contrato partilhado):** `launch_speed` (`ph2d-platformer/src/jump_config.rs`) resolve `v0` para o ápice real = `jump_height` (forma fechada por bandas de gravidade + 3 secantes contra `discrete_apex`; 0,20 µs/chamada). `apply_feel` só escreve gravidades; `set_feel` sem argumento de mundo; `apply_player_edit_in` fundido em `apply_player_edit`; `height_for_apex` saiu. Gates novos `o_pulo_do_jogo_sobe_a_jump_height`, `a_alca_poe_o_apice_onde_foi_pedido` (`ph2d-physics-ecs/tests/it/jump_arc.rs`). Fixturas `platform_corner`/`player_ceiling`/`platform_wall` levadas ao ápice velho (`OLD_LAW_JUMP_HEIGHT` 2,1056 e 1,3146) para manterem o mesmo `v0`. Resíduo: a lei não vê os sub-passos do mundo (erro ~`v0·dt/(2·substeps)`, < 2 cm até 2,5 m). **Merge:** outra linha que meça o pulo contra Jump Height antiga reconta.
- `1979ca5c5d` — ordem das cenas: `CENAS = topdown::CENA` com `const assert topdown::CENA == input_map::ULTIMA + 1` (`ULTIMA` contada de `CODIGOS`); culpado `e0b66565dd`. Mutações: `CENAS = CENA + 1` ⇒ vermelho; 19.º código na aba ⇒ não compila.
- `ced6a4ac0a` — a curva pousa no terreno: lei simula abaixo do arranque (`MAX_FALL_M` 24, `air_time` = volta à linha); varredura sem mundo `crates/ph2d-physics/src/world/arc_cast.rs` (`sweep_path`); corte pela verdade do ECS em `crates/ph2d-physics-ecs/src/arc_terrain.rs` (fora de `bridge/`; exclui sensores, forma desligada, o player e as peças dele; matriz de camadas pela forma principal; Outline = caixa da imagem). Custo 54,3 µs/chamada (3 formas, Air Jumps 1), 2×/quadro só com a curva ligada e o relógio parado; sem cache (cachear por config+pose+versão das formas se um nível grande mostrar).
- `f591d5a3a2` — fotos da curva no terreno (`PH2D_GAME_SMOKE_FOTO=fisica_player_pulo_terreno`) e ficha 04.
- `ef83e62d9b` — pouso × choque (normal vs max slope), contorno-fantasma do corpo no choque, anéis `Spacing::Md`. **`JumpArc.touch` mudou de tipo.**

### A.2 Prova de fecho

- Verificador 1× (antes de `f591d5a3a2`/`ef83e62d9b`): build smoke ✓ · fmt ✓ · `clippy --workspace -D warnings` ✓ · censos 13/13 e 135/135 ✓ · `processo.prova.sh` ✓ · `collision-surface` sem ✗ ✓ · 25 grupos de testes por nome ✓. `ef83e62d9b`: fmt+clippy das 5 crates ✓ e testes por nome.
- `PROJECT_SCHEMA` não muda (nada gravado novo).

### A.3 Smokes

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop && PH2D_GAME_SMOKE=24 PH2D_GAME_SMOKE_DRIVE=editor ./target/smoke/ph2d-host-desktop` — Play: só ←, só ↓, só ↑ depois da diagonal. Errado: herói preso numa seta ou fora do sentido.
2. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=fisica_player_pulo_terreno ./target/smoke/ph2d-host-desktop` — a curva pousa na plataforma/chão, para na parede, corpo-fantasma onde a cabeça ou o lado batem. Errado: a curva atravessa o terreno ou termina nos pés no ar.

### A.4 ABERTO (depois da rodada)

- **Wall jump (GEF02 / [BUGS_physics.md Bug #11](../../Physics/BUGS_physics.md)):** com wall jump height ~1,9–2,0 o controle «parede sólida» de `platform_wall` é atirado ~23 m/s ao topo da parede (y≈12,9). É da ponte, não da lei do pulo; as fixturas seguram `OLD_LAW_WALL_JUMP_HEIGHT`.
- ~~`layer_override` por forma NÃO é lido pelo corte da curva~~ — FECHADO em `15429fc87f`.
- Tecto da shell: reconte pelo integrador (§4).

## Adendo — Smoke do dono 12/11 — números e sombra

Commits `adcd392fc7..HEAD` (5, só a curva do pulo, GEF 04):

- `7ac5a60521` — a curva lê-se em fundo claro: halo sob linhas, pontos e texto. Helper partilhado `crates/ph2d-editor-core/src/paint_halo.rs` (`stroke_com_halo`, `fill_com_halo`, `paint_text_centered_com_halo`); tokens `BgScrim` · `OUTLINE_SHADOW_ALPHA`, `StrokeToken::Thin`. `JumpApex` ganha braço próprio em `gizmo/point.rs`; o readout passa a `ObjPlayer`.
- `48afab6f40` — o número da alça e o do cartão são o mesmo. Mecanismo: o rapier aplica a gravidade a CADA iteração do solver (4 sub-passos × 4 iterações = 16 passos por tique), logo a queda por tique é `g·dt²·(n+1)/(2n)`. A lei e o arco passam a integrar esses passos: `JumpConfig::world_steps` (`DEFAULT_WORLD_STEPS` = 16), `ArcWorld::solver_iterations`, `ph2d_physics_ecs::world_steps`. Desenho e jogo ficam a ≤ 0,3 mm de `jump_height` (antes 1,984 / 1,996 para 2,00). Gates apertados a 1 mm (4 sensações × 0,5/1/2/2,5 m). O readout `{:.0}` cortava decimais em metros: agora `curva_do_pulo_numero::numero_da_alca`. Mutações (lei sem passos, arco sem iterações, readout `{:.0}`, fábrica 4) ⇒ vermelho.
- `e175a655da` — o readout da alça formata por `numero_da_alca`.
- `b4d52331de` — teste do texto pintado (`ouve_o_texto` público): o readout desenhado a arrastar é o texto do campo Jump Height.
- `092c7995a6` — a foto `fisica_player_pulo` rola o Inspector ao cartão Platform Player (janela de 1040 px). O nome da foto precisa do prefixo `fisica_`; nome desconhecido falha calado (ABERTO).

### ABERTO para o INTEGRADOR / GEF02

- ~~**Ponte:** `world_steps` do projeto para a lei~~ — FECHADO em `eb83c15600` (ver «Ponte autorizada»).
- ~~Menor: abaixo de ~1 m a perna do jogo apanha o corpo antes da linha de pouso~~ — FECHADO em `15429fc87f`.
- ~~Foto com nome sem prefixo `fisica_` falha calado~~ — FECHADO em `3b1060f92d`.

## Adendo — Últimos abertos (12/11, smoke do dono OK nos números e na sombra)

Commits `e20eaea286..HEAD` (3):

- `15429fc87f` — o pouso da curva usa o critério da PERNA do jogo: alcance `float_height + cling_distance`, leitura na pose do fim do tique anterior, velocidade do fim do tique ≤ 0. Prova: 48/48 casos (4 feels × 4 alturas × correndo/parado/curto) pousam no MESMO tique e a ≤ 1 cm. `ph2d_physics::Terrain` (`cast_ray`, `sweep_body`) substitui `sweep_path`; o corte filtra por `layer_override::pair_collides` (a regra dos contactos): o player pela forma principal, cada forma da cena com a sua `CollisionOverride`. Custo medido (profile smoke): 68,5 µs/chamada (era 54,3). **Para o INTEGRADOR:** `arc_terrain.rs` repete a conta do degrau caminhável e o `STEP_EPS` de `bridge/player_leg.rs` (cópia, porque a ponte é proibida a esta linha) — unificar. Limitação: o modo em que a lei move o corpo sem mola não está modelado.
- `3b1060f92d` — `PH2D_GAME_SMOKE_FOTO` e `PH2D_GAME_SMOKE_DRIVE` com nome desconhecido recusam alto (stderr com sugestão, exit 2, antes da janela). Uma fonte da verdade: cada leitor expõe NOMES, a união está em `crates/ph2d-game-smoke-fotos/src/nomes.rs`, a recusa em `ph2d-app-game` `smoke.rs`, uma linha em `shells/desktop/src/main.rs`; censo em `shells/game-scenes/tests/it/jogo_nomes_do_smoke.rs`. `ph2d-game-smoke-fotos` está a 1075 de 1077 no tecto. Docs antigos com nomes sem prefixo `fisica_` agora recusam alto.
- `536b62470b` — a foto do terreno com recuo 1,5 m (o Full agarrava a borda da Platform High a 2,1 m — verdade do jogo, medida); gate novo `a_foto_do_terreno_e_o_que_o_player_real_faz`.

### Fechados por estes commits

Pouso em pulo baixo (perna do jogo) · `layer_override` por forma no corte da curva · nome de foto calado.

### ABERTO (o que resta)

- ~~**Ponte:** `world_steps` do projeto para a lei~~ — FECHADO em `eb83c15600`: a ponte escreve `cfg.jump.world_steps` do mundo VIVO (`settings_vivos`) e `arc::simulate` os do `ArcWorld`; `launch_speed` corrige até `LAUNCH_TOL` = 1e-5 m (os 3 passos fixos deixavam 1,6 mm a 1×1). Gate `os_passos_do_projeto_sao_os_da_lei` (2×8, 8×2, 1×1; ≤ 1 mm, desenho = jogo).
- ~~Unificar a conta do degrau (`arc_terrain.rs` × `bridge/player_leg.rs`)~~ — FECHADO em `895cfb7883`: UMA régua, `WalkConfig::step_limit() → StepLimit::refuses` (`STEP_EPS` em `ph2d-platformer` `walk_brink.rs`), a cópia sai. Gates gémeos `player_foot_fan::the_leg_refuses_an_outer_foot_on_a_step_steeper_than_the_limit` e `jump_arc::a_curva_recusa_o_degrau_que_a_perna_recusa` (eb83c15600).
- **Bug #11 do wall jump** (GEF02, [BUGS_physics.md](../../Physics/BUGS_physics.md)).
- **Tecto da shell:** reconte pelo integrador (§4).
