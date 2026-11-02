# HANDOFF de continuação — line/UIUX, F47 (02/11): «Above Effects»

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `4a6d492e7e` (F47). Linha reaberta em 02/11
sobre o main `3df5894489`. Commits F47: `a2f83dfd59` (feat) · `0a9d739fa1` (clippy + catraca) · `4a6d492e7e` (fix do smoke
do dono). **Nada integrado, nada enviado.** Antes de ler: `cd` + `pwd` + `git branch --show-current`.
Formato anterior: [F46](HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F46.md).

## 1 — Pedido curado (era o ABERTO 1 do F46)

Pedido do dono 02/11: no Stroke do Vector, uma opção para o traço ficar POR CIMA dos Filtros. Escolhas do dono
(perguntadas antes de construir): **por TRAÇO** (base e cada camada Stroke da pilha); nome **«Above Effects»**
(checkbox no ⚙ do traço; dica «Draw this stroke on top, untouched by the shape's effects.»).

⚠️ A 1.ª escolha («a sombra nasce do traço também»: a imagem de FX continuava a levar o traço) foi **REVERTIDA** pelo
smoke do dono: «o stroke amarelo, se checado above effects aparece duplicado, um sob efeito do blur e outro por cima».
Agora o traço marcado **SAI da imagem de FX e desenha-se UMA vez, por cima**; sombra/brilho nascem do resto da forma
(dito ao dono). Curado em `4a6d492e7e`.

## 2 — Mecanismo

- `StrokeSpec::above_effects` (FIM do struct). **VEC_SCENE 27→28:** `stroke_wire.rs` `StrokeSpecV23` congelado para
  23..=27 (`reading_stroke_v27`), `StrokeSpecV28` espelho vivo; `from_bytes` braço V25..=V27. Fixture
  `crates/ph2d-vec-scene/tests/it/fixtures/vec_scene_v27_traco.bin` gravada pelo código v27 (HEAD `d95d9a1bea`, sha256
  `50d3b34f67ef…`), gate `vec_v27_fixture` (3 testes, com controlo).
- `VecPath::is_above_effects` / `has_stroke_above_effects` / `under_effects` (`paint_stack.rs`, vec-scene).
- **vec-render:** `dispatch::draw_one` com imagem de FX chama `draw_vector(.., above_only=true)` →
  `stack_draw::draw_above_effects` (só os traços marcados, na ordem da pilha). `standalone.rs`: `draw_path_for_effects`
  (produtor de FX, rasteriza `under_effects`) × `draw_path_isolated` (assado do Motion, forma inteira); corpo comum
  `draw_isolated_with`.
- **app-vec:** `fx_live.rs` usa `draw_path_for_effects`; `fx_silhouette.rs` resolve a silhueta com `under_effects`
  (Bevel/Inner Shadow lêem o resto).
- **Auto Key:** canal `canal_vetor::TRACO_SOBRE_EFEITOS = 117` (`Especie::Ligado`), base i=0 e camada pelo id;
  `canal_vetor_lista` (`Lista::Tinta`) e `marcas::na_aberta` alargadas até 117; i18n «Stroke Above Effects»;
  `canais_campos` lê/escreve/publica.
- **Painel:** id `ph2d_panel_vector::ids::VECTOR_STROKE_ABOVE_EFFECTS`; `checkbox_row` no `stroke_settings_body` (base e
  camada aberta); `state::set_stroke_above_effects(Option<bool>)` publicado por `vector_bridge_publish` via
  `paint_stack::shown_above_effects` (linha aberta); `event_paint_stack::is_button`; tooltip em `populate_appearance`.
  Clique → `join_code_for_id` → `ABOVE_CODE = ALIGN_CODE+3` → shell `fase_transform_ops` chama
  `paint_stack::set_chip(scene, sel, open_layer_index(), code)` (base sem camada aberta; a camada aberta com ela; toggle =
  contrário da 1.ª forma). `marcas.rs` CAMPOS liga o id ao canal 117. `paint_side::open_stroke_settings` (novo, exportado).

## 3 — Catraca do `ph2d-app-vec` (cura por CORTE)

Tecto 93 637: estava +464 (F46+F47) e falhava. Cabeçalhos `//!` de 19 módulos sem leitores de fonte nem `include_str`
(symmetry_live, widget_icon, fx_live_memo, svg_export, shape_build, pattern_seam_probe, smoke_fade, contour_live,
bool_live, widget_drive, hier_group, bucket_claim, smoke_bone_par, fx_silhouette, align_live, vec_frame_edit,
paint_dilate, group_gizmo_view, frame_spans) movidos verbatim para
[`docs/Vector Module/50_cabecalhos_do_app_vec.md`](../../Vector%20Module/50_cabecalhos_do_app_vec.md) (fica o 1.º
parágrafo + link). Medido: 93 564 / 93 637.

## 4 — Cena nova `PH2D_OBJECT_MODE_SMOKE=49`

`vector_mode_fill_stroke.rs` `por_cima_dos_filtros` (`qual == 4`): rectângulo degradê, Blur + Drop Shadow, traço escuro
de base com «Above Effects», contorno amarelo sem, ⚙ do traço aberto. Tabela `vector_mode_scenes.rs` 9..=49 / 43..=49 /
handles 31..=35; a rota passa `sim`. **Próxima cena: 50.** Fotos em `img_F47/`: `cena49.png` · `cena49_controlo_sem_above.png`
· `cena49_lado_a_lado_com_e_sem.png` · `cena49_escuro_e_amarelo_por_cima.png`.

## 5 — Formato e contadores (DELTA)

- `PROJECT_SCHEMA` 213→214 (`PROJECT_SCHEMA_SEM_TRACO_SOBRE_FILTROS = 213`, `fio_do_projeto` diz 27 até 213); tripla
  `(214, 13, 28)`; degrau 166→167 arquivado em `project_schema_history_v160.rs` (agora v160..v167); escada viva 593/600.
  `project_geometria_v194_tests.rs` varint `0xD6` (214).
- `VEC_SCENE` 27→28. `DOC_VERSION` intocado. Sem ADR.
- Cenas `PH2D_OBJECT_MODE_SMOKE`: `1|2|4|6..49`.
- **Para o integrador:** CLAUDE.md §5.1 UI/UX passa os smokes a `6..49` e o link do handoff; `VEC_SCENE` 28 e
  `PROJECT_SCHEMA` 214 **somam entre linhas** (recontar, `python3 scripts/schema-recount.py`); a catraca do app-vec também
  soma. Os 3 ficheiros do main por formatar (`crates/ph2d-panel-pixel-preview/src/paint.rs`,
  `crates/ph2d-panel-registry-init/tests/it/the_vector_control_bar.rs`, `crates/ph2d-vec-entities/src/connector_handles.rs`)
  continuam fora desta linha.
- Superfície de colisão: vec-scene (`stroke_wire`, `paint_stack`, fixture), vec-render (`dispatch`, `stack_draw`,
  `standalone`), `ph2d-app-vec`, `ph2d-ecs` (canal 117), `ph2d-i18n`, `ph2d-panel-vector`, vec-edit, vec-svg, shell
  (`fase_transform_ops`, schema), `docs/Vector Module/50_*`, `img_F47/`.

## 5b — Prova de fecho (02/11)

- fmt e `clippy -D warnings` limpos nas 9 crates tocadas (ph2d-app-vec, -ecs, -i18n, -panel-vector, -vec-edit,
  -vec-render, -vec-scene, -vec-svg, host-desktop).
- Portões verdes: `shell_only_shrinks` (2), panel/widget/workspace loc caps, claude_md, `hr15_no_hardcoded_ui_strings`,
  `no_literal_color`, i18n english/key, vec-scene inteira (616, formato), panel seam 228, f44 11, f45 5, f47 3,
  app-vec `canais_campos` 13 · `marcas` 13 · `paint_stack` 25 · `fx_silhouette` 11 · `fx_live` 34, vec-render
  `stack_draw`/`above_effects`/`standalone`, ecs `canal` 56, shell `project_schema`/`each_project_version`/
  `the_same_bytes_under_todays_header`.
- **Mutação 7/7 sangrou:** dispatch sem o redesenho · filtro do `draw_above_effects` aberto · leitor do canal fixo ·
  âmbito v27 desligado · `set_chip` None→false · checkbox removido · marca removida. Nota: `above_effects_changes_nothing`
  sozinho NÃO apanha a mutação 2 (o irmão apanha).
- **Suíte completa NÃO corrida** (é da integração). Binário `smoke` compilado em 02/11.

## 6 — Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=49 ./target/smoke/ph2d-host-desktop`
2. Selecione o rectângulo; no Inspector, secção Stroke, abra o ⚙ do traço escuro e ligue «Above Effects» (a cena já vem com ele ligado).
3. Deve acontecer: o traço escuro aparece UMA vez, nítido, por cima do Blur; desligando, ele borra com a forma.
4. Deu errado se: o traço aparecer duplicado (borrado por baixo e nítido por cima), ou o amarelo mudar.

**Estado:** a 1.ª rodada achou o duplicado (curado em `4a6d492e7e`); a **2.ª rodada, depois da cura, NÃO foi confirmada
pelo dono** — ele passou ao pedido seguinte.

## 7 — ABERTO (por ordem)

1. ⭐ **NOVO pedido do dono, 02/11** (foto da secção «Expand» do Inspector: Offset 25, Side Outer/Inner/Both, Corner
   Miter/Round/Bevel, botões «Apply Offset» e «Outline Stroke», Profile): «Aparentemente Expand e suas funções não funcionam
   corretamente com nosso novo modo de camadas stroke/fill. Auditoria completa de expand e demais tools do painel para que
   tudo fique funcional». Medir cada controlo da Expand (e das outras secções/tools do painel Vector) contra a pilha F45
   (base com posição `base_order`, camadas Fill/Stroke, olho `fill_hidden`/`stroke_hidden`, F47 `above_effects`): quem lê só
   `path.fill`/`stroke` em vez de `paint_stack()`/`draw_order`. Reproduzir cada defeito com gate VERMELHO antes de curar;
   família num bloco (CLAUDE.md §0.10).
2. Confirmar com o dono a 2.ª rodada do smoke 49.
3. Esc não cancela o arrasto de linha de painel (família `panel_row_drag`, pré-existente).
4. Perfil de largura não é por camada de traço.
5. Ids de Position fora do censo `seam`.
6. `migra_pilha_v23` sem chamadores.
7. Com nada seleccionado, o ⚙ do traço de base fica activo e o do Fill apagado (F44; confirmar com o dono).
8. ⛔ Janela baixa: a régua de ferramentas FLUTUANTE regista botões FORA da janela (430 px de altura, botões até `y≈774`:
   inalcançáveis). Falta recorte/rolagem pela altura da região. Sem gate ainda: medir no ECRÃ, não no mundo, e reprovar
   antes da cura. Detalhe no §8 do handoff F40–F45.

## 8 — Observação de outra linha (não mexida)

`PH2D_CONSTRAINT_SMOKE=7` («Caixa com tampa») imprime «Edit + Regras», mas na foto fica no modo Object, sem a tool Regras
(linha Constraints; ver F46 §7).

## 9 — Prompt de continuação

```
/pd-linha-assumir line/UIUX — Worktrees/line-UIUX
Próximo item: ABERTO 1 — auditoria completa da secção Expand (Offset, Side, Corner, Apply Offset, Outline Stroke, Profile) e das demais tools do painel Vector contra a pilha de tintas F45/F47 (camadas Fill/Stroke, ordem livre, olho, Above Effects).
Handoff: docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F47.md
Meça cada controlo antes de curar (gate vermelho primeiro), família num bloco. Só tokens e i18n, app só em inglês, Auto Key em todo campo. Cena fotografada antes do smoke (próxima = 50). PARE no meu smoke. Sem integrar nem push.
```
