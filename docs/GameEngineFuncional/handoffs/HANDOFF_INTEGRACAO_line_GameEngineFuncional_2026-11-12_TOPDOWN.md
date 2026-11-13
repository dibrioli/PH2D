# HANDOFF de integração — `line/GameEngineFuncional`, 12/11 (GEF 05: Top-Down Player)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, base `4f46a3573c` (= `main` quando a linha foi reaberta, 12/11; o tip antigo está na tag `backup/GEF-antes-reabrir-2026-11-12`). 8 commits, 70 arquivos. Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono 23/10): são do integrador. Ficha: [`05_topdown.md`](../componentes/05_topdown.md). Dono: versão A escolhida e smoke OK em 12/11.

## 1. Commits (`git log --oneline main..HEAD`)

| Commit | O que |
|---|---|
| `3a83b2bac6` | ficha: lei conferida (12 campos, nenhum morto), versões A/B/C |
| `2929ec4948` | cartão versão A + dobra partilhada `grupo_dobravel::dobra` (Player e Top-Down) |
| `e0b66565dd` | cena `PH2D_GAME_SMOKE=24` + gate do giro (Transform e base do renderer a 45/90/0°) + `fotos_05` |
| `1188f2196e` | palavras do cartão para `ph2d-i18n/src/topdown.rs` (tecto 700); isenções POR_FUNCAO seguem |
| `486d1da055` | a cena 24 abre com o cartão à vista: `request_inspector_section` atendido pela lista de cartões; `PH2D_GAME_SMOKE_FOTO=topdown_more\|topdown_off` |
| `b6f0300a6c` | crate-folha **nova** `ph2d-game-smoke-fotos` (`foto`, `mao`, `janela`) sai de `ph2d-app-game` e de `janela_do_jogo_smoke.rs` da shell (214 → 77) |
| `515c5498e4` | herói visto de cima: `ph2d-render/src/atlas/fabrica/boneco_de_cima.svg` (6×2: idle 4, walk 6, olha +X), gerado por `ferramentas/arte_svg.py boneco_de_cima()`; `hero_topdown.rs`; regras «Key held: move_* → walk» |
| `8571861b96` | acabamento: `grupo_dobravel::botao` partilhado, aviso «Default Controls is off: …», nome inteiro de Directions/Facing vem do catálogo, dicas nas 12 dobras do Player |

## 2. Superfície de colisão (`collision-surface.sh`, caminho absoluto)

Colado da saída (merge-base `4f46a3573c`, 8 commits, 70 arquivos):

- **Schemas, DELTA 0:** `PROJECT_SCHEMA` 237 (base 237), tripla `(237, 13, 29)`, `VEC_SCENE_SCHEMA` 29, `FLIP_SCHEMA` 13, `DOC_VERSION` 25. Nenhum campo gravado novo. Reconte pelo `main` do dia: `python3 scripts/schema-recount.py`.
- **Registo de componentes, DELTA 0:** `ph2d-ecs` 121 (base 121). O script marca `✗ SONDA CEGA` em `ph2d-render` e `ph2d-script` (espelho): defeito do script (a const mudou de ficheiro), não da linha; esta linha não mexe em componentes.
- **Contratos §6:** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` intocados. ADR: nenhum criado (próximo livre 0201).
- **Cargo.lock:** único `+name` novo é `ph2d-game-smoke-fotos` (interna). `ph2d-physics-scene` passa a depender de `ph2d-input` (só dep).
- **Conflito:** nenhum marcador. Tectos de LOC: nenhum arquivo da linha passa.
- **Não-colisão com a linha InputMap:** `git diff --name-only main..HEAD | grep -E 'bridge/|physics_apply|ph2d-input/|input_map_ops'` vazio.
- **Exceção a nomear:** UMA linha acrescentada em `crates/ph2d-i18n/src/asset_browser.rs` (chave do nome da arte nova `asset.factory.hero_top_down`, «Hero (top-down)»).

## 3. O que um merge pode partir

- **Rótulos do catálogo renomeados app-wide** (`component_catalog.rs`): Acceleration → Speed up, Deceleration → Slow down, Turn Speed → Turn, Push Recovery → Hit recovery. Nomes de script inalterados. Teste de outra linha que compare os rótulos antigos reprova.
- `grupo_dobravel::{dobra, botao}` (`ph2d-panel-inspector`): usados por Top-Down, Player (Fit×2) e Instance. Outra linha que mexa em `player_cartoes.rs`, `instance_cartao.rs` ou `topdown.rs` (639 LOC reescritas) conflita textualmente.
- `ph2d-editor-core`: `panel/cards_rolagem.rs` (+70) e `cards_lista.rs` (+2, `request_inspector_section`), `topdown_edits.rs`, `hier_rows_ops.rs` (+19). Outra linha que toque a rolagem/pedido de secção do Inspector reconta.
- `ph2d-render`: `FACTORY_ART` 9 → 10; chave `u32::MAX-11` em `hero_topdown.rs`. Outra linha que crie arte de fábrica no mesmo número da chave ou no mesmo contador reconta (conte, não escolha).
- `smoke.rs`: `CENAS = 24` (6..23 são do Input Map). Outra linha com cena 24 reconta.
- Shell: `janela_do_jogo_smoke.rs` 214 → 77 e `janela_do_jogo.rs` (+17). Conflito provável com a linha InputMap, se mexeu na janela do jogo.

## 4. Tectos (`architecture_the_shell_only_shrinks.rs`)

- `ph2d-app-game` 2728 → 2093 (medido 1993). Nova crate `ph2d-game-smoke-fotos` com tecto 1077 em `TETOS_DAS_CRATES_DA_FAMILIA`.
- **Tecto da SHELL não mexido: é do integrador.** A shell só encolhe nesta linha (janela do jogo -137); reconte com o `main` do dia.

## 5. Prova de fecho (verificador, 1×)

- `cargo build --profile smoke -p ph2d-host-desktop` ✓ · `cargo fmt --all --check` ✓ · `cargo clippy --workspace --all-targets -- -D warnings` ✓ · `cargo metadata --locked` ✓.
- `scripts/censos-da-arvore-combinada.sh`: 135/135 e 13/13 ✓.
- Por nome: `ph2d-editor-core` `architecture_*` (5); `ph2d-panel-inspector` `seam_topdown`, `seam_player`, `a_seccao_topdown`, `every_label`, `every_word`, `dica`, `nenhum_botao`; `game-scenes` `jogo_topdown`, `a_arte_de_fabrica`, `sprite_image_choice_gate`; `ph2d-app-game` `o_nivel_le_se_pela_porta`; `ph2d-render` fabrica; `ph2d-i18n` (3); `ph2d-app-components` `toda_opcao_do_catalogo` — todos verdes.
- **Mutações vistas vermelhas:** cartão 10/10 · cena 2/2 · foco do cartão 5/5 · arte 4/4 · acabamento 6/6.

## 6. Smokes (comando inteiro)

1. Cena do Top-Down (abre com o cartão à vista, herói selecionado): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop && PH2D_GAME_SMOKE=24 ./target/smoke/ph2d-host-desktop` — Play, segurar → e ↑: o herói anda na diagonal, gira para onde anda, desliza na parede. Errado: herói de lado/plataforma, ou parado com as setas.
2. Fotos (só com a variável): o mesmo comando com `PH2D_GAME_SMOKE_FOTO=topdown_more` (dobra More aberta) ou `topdown_off` (Default Controls desligado). Saída em [`fotos_05/`](../componentes/fotos_05/).
3. Cena de gate: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && bash scripts/cargo-test-narrow.sh ph2d-host-game-scenes jogo_topdown`.

## 7. ABERTO

- Tecto da shell: reconte pelo integrador (§4).
- Herdados da curva do pulo ([handoff](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_CURVA_DO_PULO.md)): ~~a curva termina nos pés e não segue o terreno~~ e ~~Jump Height 2,00 × ápice 2,04~~ — FECHADOS na rodada de 12/11 (adendo abaixo).
- Defeito do wall jump (GEF02, adendo §A.3) · ~~`layer_override` por forma não lido pelo corte da curva (§A.2)~~ — FECHADO (adendo «Últimos abertos»).
- Seguinte na fila: Projectile Motion ([plano](../23_refundacao_plano.md)).

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

- **Ponte (proibida a esta linha):** em `bridge/player.rs`, depois de `let mut cfg = cfg.config();`, pôr `cfg.jump.world_steps = crate::world_steps(&self.settings());` — e o mesmo em `components/player_arc.rs` (`jump_arcs`). Sem isso, um projeto com sub-passos/iterações fora da fábrica volta a errar a Jump Height (desenho e jogo seguem iguais entre si, ambos com 16).
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

- **Ponte:** `world_steps` do projeto para a lei (linha pronta no adendo «Smoke do dono 12/11») e unificar a conta do degrau (`arc_terrain.rs` × `bridge/player_leg.rs`).
- **Bug #11 do wall jump** (GEF02, [BUGS_physics.md](../../Physics/BUGS_physics.md)).
- **Tecto da shell:** reconte pelo integrador (§4).
