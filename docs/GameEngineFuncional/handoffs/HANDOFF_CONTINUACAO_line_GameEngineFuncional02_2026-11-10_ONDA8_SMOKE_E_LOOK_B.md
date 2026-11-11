# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, smoke da onda 8 + Look B + pedidos novos do dono

Leitor: o próximo agente da linha. Anterior: [ONDA8_ITENS_2_A_4](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_ITENS_2_A_4.md) (a §7 dele é o item 5, ainda ABERTO).

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`. Commits desta janela sobre `64ffc7c8b4`: `be48b89680` (8.3b + cena 122 + cena 48 movida) · `4e266f5004` (Look sem resets, grade do Pivot) · este handoff. **`PROJECT_SCHEMA` 232, inalterado.** Merge-base = `bc4a6ce930` (ponta do main a 10/11). Nada integrado, nenhum push. Binário `--profile smoke` compilado depois de `4e266f5004`.

## 2. Decisões do dono (10/11)

- Smoke do Look: «ótimo» depois de (a) **retirar os dois resets** (Color, Opacity) e (b) **grade do Pivot da altura dos campos X e Y juntos** — feitos (`4e266f5004`).
- **NOVO, por fazer (§5):** «Ao associar a um corpo, a cor e a linha têm de resetar para o padrão do corpo. Se o usuário vier a mudar, ela assume a nova aparência.» e «Coloque também em Look a capacidade de **Dash** e a **largura do stroke**.»

## 3. O que mudou

- **8.3b — o livro do quadro recebia a pré-visualização como autorado** (achado ao montar a cena 122, sonda no app): os tiques que correm ANTES do dreno do quadro (arranque do Play, cena montada a meio do quadro) aplicavam a curva FISICA com livro descartável; o undo/save fotografava Height 1,13 m em vez de 1 m em todo quadro. Cura: `TimelineScene.livro` (`bake.rs`), `dispatch` recebe `(PreviewDrive, ConducaoFisica)` (shell `fase_physics_step.rs`, 0 linhas), `aplicar_do_tique` passa `conduziu` e a foto do contorno ao livro do quadro. Testes `os_tiques_antes_do_quadro_nao_trocam_o_autorado_da_forma` / `_o_contorno_autorado` (`canais_por_tique_tests.rs`, pela entrada `dispatch`), nascidos vermelhos (1,6667 ≠ 1); mutação 3/3.
- **Cena `PH2D_PHYSICS_SMOKE=122`** (`ph2d-physics-smoke-scenes/src/physics_smoke_grow.rs`): três caixotes, Height 1↔2 m por segundo, 60 s; o trecho da Timeline é posto a 60 s (os 4 s de omissão congelavam a curva — `clip_cut`). ⚠️ `SceneSetup.playhead_loop` NÃO segura o relógio com a física armada (medido: t passou de 6 s); não usado.
- **Cena 48 movida** para `ph2d-physics-smoke-scenes` (287 linhas, com o tecto: `ph2d-app-physics` 38_085 → **37_798**, smoke-scenes 18_572 → 18_859). Corte de comentário histórico em `smoke.rs` e `bridge/dispatch.rs`.
- **Look B:** fora `INSP_SHAPE_{TINT,OPACITY}_RESET`, `PhysicsFieldEdit::LookReset*`, as duas dicas i18n; `grade_de_lado` em `cards_sprite_pivot.rs` (Sprite igual), Look usa lado `2·FIELD_H + FIELD_ROW_GAP`, topo `fim_rotulo − FIELD_H` (a linha do cartão acaba no fundo do campo, sem gap). Teste `os_campos_do_pivo_estao_sempre_a_vista_ao_lado_da_grade` com as duas pontas; mutação sangra.

## 4. ⛔ Catraca

`ph2d-app-physics` está **EXATAMENTE no tecto (37_798 / 37_798)**, testes incluídos. Qualquer linha nova ali precisa de corte/mudança antes (cenas `physics_smoke_*` sem `crate::` interno mudam para a smoke-scenes levando o tecto; o líquido cura-se por corte). Medir: `find crates/ph2d-app-physics -name '*.rs' | xargs cat | wc -l`; gate `the_shell_only_shrinks_and_every_app_crate_too` (`ph2d-editor-core --test it`).

## 5. ABERTO, por ordem

1. **Look ao associar a um corpo** (dono 10/11): quando uma forma solta passa a ser filha de um corpo (arrastar na Hierarchy, «Make Independent Body» ao contrário, `torna_corpo`/`pendura`), o `ShapeLook` volta ao padrão do corpo (cor = camada, sem `cor_propria`, e a «linha» = traço padrão); depois disso, o que o artista muda fica. Medir primeiro onde a associação acontece (`ph2d-physics-ecs` `migra_formas.rs`, a shell da Hierarchy) e se hoje o Look da forma solta sobrevive. Teste nascido vermelho.
2. **Dash e largura do stroke no Look** (dono 10/11): dois controlos novos no cartão (`cards_forma_look.rs`, `sections/look.rs`), campos em `ShapeLook` (`ph2d-shape-handles` / `ph2d-physics-ecs`), edição `PhysicsFieldEdit::Look*`, pintura do contorno (`ph2d-app-physics/src/overlay/outline.rs`); formato gravado ⇒ **subir `PROJECT_SCHEMA`** (`python3 scripts/schema-recount.py`); canal da Timeline se o resto do Look o tiver; checkbox para liga/desliga (padrão da casa, nunca toggle); UI só em inglês. Foto do cartão ao lado da Sprite antes do smoke.
3. **Item 5** do [ONDA8_ITENS_2_A_4 §7](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_ITENS_2_A_4.md): 5.1, 5.3, 5.4, 5.5, 5.6 + medir canais de JUNTA e MUNDO por tique.

Smoke das cenas 122 e 78 (fotos `fotos_fisica/onda8/04_*`, `05_*`): **aprovado pelo dono 10/11** («demais smoke parece OK»).

O fecho da linha e a integração continuam só por ordem do Enio.
