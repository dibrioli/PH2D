# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, Look ao associar + traço do Look + rótulos

Leitor: o próximo agente da linha. Anterior: [ONDA8_SMOKE_E_LOOK_B](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_SMOKE_E_LOOK_B.md). O item 5 continua no [ONDA8_ITENS_2_A_4 §7](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_ITENS_2_A_4.md).

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`. Commits desta janela sobre `922fff7770`: `eb702f0a17` (8.5) · `92fb36208c` (8.6) · `80670dbced` (8.6b) · este handoff. **`PROJECT_SCHEMA` 232 → 233.** Merge-base = `bc4a6ce930`. Nada integrado, nenhum push. Binário `--profile smoke` compilado depois de `80670dbced`. **Smoke do dono: OK (10/11)** nos três.

## 2. O que mudou

- **8.5 — Look ao associar a um corpo** (dono 10/11): porta `ph2d_physics_ecs::{donos_dos_looks, look_ao_associar}` (`shape_look.rs`): retrato do dono de cada forma com `ShapeLook` antes do gesto; a que passa a OUTRO corpo veste `ShapeLook::no_padrao_do_corpo` (tudo menos o pivô). Chamada por `torna_corpo` (as três entradas; o lado-forma levado à filha re-chaveia o retrato) e `hierarchy_reparent::reparentar`. Mesmo dono não repõe. Mutação 5/5.
- **8.6 — Dash e largura do stroke** (dono 10/11): `ShapeLook { largura, tracejado, traco: [traço, vão], traco_proprio }` (px de tela; piso `TRACO_MINIMO` 0,5, sem teto). `tinta_canvas::Tom.traco` + `impl From<&ShapeLook> for Tom` (porta única do overlay) + `traco_do_corpo(col, corpo)`; a sombra segue o traço. Cartão Look: «Stroke width (px)» + checkbox «Dashed»; com tracejado «Dash and gap (px)»; mostra o traço que o canvas pinta (`ph2d-shape-handles` → dep `ph2d-panel-physics`). Edição `PhysicsFieldEdit::LookStroke { width, dashed, dash }`. Canais 4..7 no catálogo; losango na largura/traço/vão; **a amostra Color passou a ter canal** (vermelho antigo do gate `architecture_every_editable_field_is_a_channel`). Foto: `PH2D_PHYSICS_SMOKE_TRACEJADO=<corpo>` (cena 120). Dois testes da shell que não compilavam desde 8.3b curados. Mutação 9/9.
- **8.6b — rótulos de cartão** (dono 10/11, *«é regra»*, a de 06/09): `cards_rotulos` elide numa linha (a quebra da maquete F saiu) e a largura do rótulo pára na 1.ª célula com marca. Vale para todo o app. Testes `a_label_that_does_not_fit_stays_on_one_line`, `a_label_stops_before_the_key_mark_of_its_cell` (nascidos vermelhos). `fotografa_cena.sh` ganha `FOTO_LAYOUT=<ficheiro>` (`active=scene` + `[scene]` `dock_w_right=210`). Memória `feedback_a_label_that_does_not_fit_gets_an_ellipsis_never_wraps`.

Fotos: `fotos_fisica/onda8/07..11`.

## 3. ⛔ Catracas

`ph2d-app-physics` **37 793 / 37 798**; `ph2d-app-components` **80 869 / 80 869** (no tecto: cure por corte antes de crescer). Medir: `find crates/<c> -name '*.rs' | xargs cat | wc -l`; gate `the_shell_only_shrinks_and_every_app_crate_too`.

## 4. ABERTO, por ordem

1. **Item 5** do [ONDA8_ITENS_2_A_4 §7](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_ITENS_2_A_4.md): 5.1 (censo de sinais dos botões), 5.3 (`ph2d-outline-trace` caro antes do teto), 5.4 (nav e palco sem as partes), 5.5 (macro ignora o travado do contorno — confirmar `op::aplicar` antes do teste), 5.6 (painel clássico não acinzenta campos travados). Cada um: teste nascido vermelho + mutação.
2. **Medir os canais de JUNTA e de MUNDO por tique** (como na 8.3b); curar só se medir.
3. **Fecho da linha** (só por ordem do Enio): clippy/fmt das crates tocadas 1× (`PH2D_SUITE=fecho`), handoff de integração, §5 do CLAUDE.md. A suíte inteira é do integrador.
