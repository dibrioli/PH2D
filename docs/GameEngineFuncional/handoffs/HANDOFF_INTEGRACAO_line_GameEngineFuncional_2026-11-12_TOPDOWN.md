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

- Foto real do herói andando só para cima não existe: o roteiro `HORARIO_24` só aperta → e ↑ juntas; o 90° está provado por teste (gate do giro).
- Tecto da shell: reconte pelo integrador (§4).
- Herdados da curva do pulo ([handoff](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_CURVA_DO_PULO.md)): a curva termina nos pés e não segue o terreno; Jump Height 2,00 × ápice 2,04.
- Seguinte na fila: Projectile Motion ([plano](../23_refundacao_plano.md)).
