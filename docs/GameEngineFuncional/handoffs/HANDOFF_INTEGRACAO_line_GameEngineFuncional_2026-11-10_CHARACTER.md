# HANDOFF de integração — `line/GameEngineFuncional`, 10/11 (GEF 03: o Character em cartões)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, base `bc4a6ce930` (= ponta do `main` quando a linha abriu). Esta linha tem os commits dos ABERTOS do Pivot ([handoff anterior](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_ABERTOS_DO_PIVOT.md), ainda não integrado) MAIS os 5 do GEF 03 abaixo; integre a linha inteira. Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono 23/10): são do integrador. Ficha: [`03_character.md`](../componentes/03_character.md).

## 1. Commits do item (`git log --oneline bc4a6ce930..HEAD`)

| Commit | O que |
|---|---|
| `c77af873d2` | o Character em cartões, versão B: sete `bool` em `PlatformPlayer`, `player_pronto.rs`, painel `player_cartoes.rs` + `player_grupos.rs`, catálogo 55–61, schema 232, ficha |
| `5283817d8a` | as 12 dobras abrem pelo clique: faltava `mark_collapsible_section` (os testes abriam por `set_collapsed`) |
| `4888486ba1` | a dobra de cada habilidade revela-se na largura inteira (só o título começa depois da caixa) |
| `2e6cb2b9b1` | **FUNDAÇÃO:** `widget::corte_da_dobra` — o campo «à espera» do fluxo (`cards_compacta::pede`) pinta-se dentro da dobra em que nasceu |
| `0361441e4c` | **FUNDAÇÃO:** `widget::chave_da_altura` — a altura lembrada de cada dobra é por modo de pintura |

## 2. ⚠️ Alerta: cura de FUNDAÇÃO dos cartões (vale para TODA dobra em cartões)

`ph2d-editor-core` (`widget/section_header/body.rs`, `panel/cards_compacta.rs`, `widget/mod.rs`):
- `corte_da_dobra()` — o `SectionFold` empilha a banda que recorta; a `Espera` do fluxo guarda-a e é pintada com ela; dois pedidos só se juntam se nasceram na mesma dobra. Muda a pintura de QUALQUER dobra em cartões (Inspector, Painter, Vector…): a fechar, nenhum campo passa por cima do título seguinte.
- `chave_da_altura(id)` — a mesma dobra pinta-se duas vezes por quadro (levantamento fora do cartão, 304 px, e cartão, 258 px) e partilhava `section_body_h(id)`; agora a memória é por modo. Medido com sonda em `SectionFold::begin_at`; a mola da altura foi medida e descartada.
- Uma linha que mexa em `SectionFold`, `cards_compacta` ou leia `section_body_h(id)` direto conflita ou se comporta diferente: `git grep -n 'section_body_h\|remember_section_body_h'`.

## 3. Crates e ficheiros tocados (`git diff --stat bc4a6ce930..HEAD`, só o item)

- `ph2d-physics-ecs`: `components/player.rs` (+7 campos), `components/player_pronto.rs` (novo), `tests/it/player_abilities.rs` (novo) e ~25 testes/rigs que marcam a caixa.
- `ph2d-component-desc`: `catalog/physics_fields.rs` (+7: campos 55–61, §4).
- `ph2d-i18n`: `inspector_player.rs`, `component_catalog.rs` (14 linhas de comentário para o arquivo cortado, §6).
- `ph2d-editor-core`: a fundação do §2 + `screens/hero/inspector_model_player.rs`.
- `ph2d-app-physics`: `inspector/player.rs`, `physics_smoke_out*.rs`.
- `ph2d-app-game`: `object_add.rs` (Character nasce com Crouch marcado).
- `ph2d-panel-inspector`: `sections/player.rs` (−363), `player_cartoes.rs` e `player_grupos.rs` (novos), `populate_player.rs`, `event_player.rs`, `ids/inspector_player.rs`, `sync_physics.rs`, `tests/it/seam_player.rs` (+470).
- `ph2d-physics-smoke-scenes`: ~14 cenas `physics_smoke_*` marcam a caixa onde ligavam a habilidade pelo número.
- `ph2d-panel-registry-init`: testes `o_inspector_armado`, `o_desfazer_nao_pula`.
- `shells/desktop`: `project_schema.rs` (+ `_tests`), `physics/inspector_player*_tests.rs`, `render_loop/inspector_commits.rs`, `undo_app.rs`.
- `shells/game-scenes`: `tests/it/sprite_card_edits.rs` (1 linha).

## 4. Schema e catálogo

- `PROJECT_SCHEMA` **231 → 232**: `PlatformPlayer` ganha, NO FIM, `walls_on`, `dash_on`, `crouch_on`, `swim_on`, `ledge_on`, `glide_on`, `fall_limit_on` (postcard posicional). Sem leitor do antigo (§0.11). A tripla NÃO vê este degrau; registos: DELTA 0.
- **Na integração, reconte pelo `main` do dia:** `python3 scripts/schema-recount.py` (se outra linha integrar antes e subir o schema, o degrau desta muda). Comentário do degrau em `project_schema.rs`, bloco «`231 → 232`».
- Catálogo (`ph2d-component-desc/src/catalog/physics_fields.rs` + `ph2d-i18n/src/component_catalog.rs`): **campos 55–61** (as sete caixas). Outra linha que acrescente campos ao catálogo da física no mesmo intervalo reconta.

## 5. O que um merge pode partir

- Teste/cena de outra linha que ligue Dash, Walls, Crouch, Swim, Ledge, Glide ou Fall Limit **só pelo número**: agora a caixa (`*_on`) também tem de estar marcada. ~45 sítios já curados aqui; `git grep -n 'dash_speed\|wall_slide\|ledge_grab\|glide_fall\|max_fall'` nos testes novos da outra linha.
- Literal `PlatformPlayer { .. }` completo noutra linha: faltam os 7 campos (use `..Default::default()`).
- `sections/player.rs` encolheu 363 linhas e `player_cartoes.rs` é novo: conflito textual provável se outra linha mexeu no painel do player.
- `seam_player.rs` (+470) e `component_catalog.rs`/`inspector_player.rs` (strings): conflito textual provável.
- Fundação do §2: qualquer outra linha com testes de geometria de dobra pode mudar de número.

## 6. Arquivo de comentários cortados

14 linhas de comentário do `component_catalog.rs` foram para [`COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-10.md`](../../archive/integracao-jornadas/COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-10.md) (tecto de LOC). `player_cartoes` foi partido em `player_grupos`; `populate_player_boxes` isolado.

## 7. Prova de fecho

Corrido (pela linha): fmt e clippy das crates tocadas; gates de LOC, HR-15 (strings), literais de cor e números mágicos; `seam_` **254/254**; os testes por nome da ficha §«Provas»; mutação **16/16 sangram** (M1c e M10 sobreviviam e ganharam teste); smoke do dono 10/11 OK depois das 3 curas de UI (commits 2–5).

**NÃO corrido (é do integrador):** suíte completa, `nextest-impacted`, `machete`, B2.

## 8. Smoke do dono (já feito em 10/11; para repetir)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo run -p ph2d-host-desktop --profile smoke`
2. Add ▸ Character; selecione o herói; no Inspector, cartão do Platform Player. Walk e Jump abertos; o resto dobrado com resumo.
3. Clique em Walls, Dash, Crouch…: cada dobra abre pelo título, sem campo atrasado; a fechar, nada passa por cima do título de baixo.
4. Marque/desmarque a caixa ao lado do chevron: os números ficam guardados. (o efeito no Play é coberto pelos testes de habilidade, não foi refeito neste smoke).
5. Errado: dobra que não abre, campo que aparece depois da animação, ou caixa que perde o número.

Fotos: [`fotos_03/`](../componentes/fotos_03/).

## 9. ABERTO

- O gate de geometria dos cards do player (`seam_player`) prova menos: «nenhuma row se sobrepõe» no lugar do passo da moldura por card.
- As 12 dobras sem dica de hover (isentas, como as da câmera e do HUD).
- `open_all_sections` não abre as dobras novas.
- **Próximo** (dono 10/11, memória `project-memory/project_jump_arc_on_canvas_2026_11_10.md`): a curva do pulo no CANVAS — no cartão só o botão que a liga e o seletor Floaty · Snappy · Heavy; o canvas mostra pulo inteiro, pulo curto, pulos no ar, parado e correndo; a alça dá altura (Jump Height) e distância (Speed).
