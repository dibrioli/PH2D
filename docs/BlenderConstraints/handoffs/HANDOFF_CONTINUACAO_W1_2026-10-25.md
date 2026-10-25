# Continuação — `line/BlenderConstraints`, onda W1 a meio (25/10/2026)

> Troca de janela por custo (≈ 357 mil tokens por passo). **Ainda não há smoke**: nada visível ao dono
> até o cartão existir. Plano: [`01_plano.md`](../01_plano.md) · pesquisa: [`reports/`](../reports/) ·
> oráculo: [`fixtures/blender_5_2_3_constraints_2d.json`](../fixtures/blender_5_2_3_constraints_2d.json).

## Identidade

- Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, base (merge-base)
  `c3662cd8e4`. HEAD = o commit deste handoff. Árvore limpa.
- Decisão do dono pendente (só pesa na W2): **IK na posição da pilha** (diferente do Blender, que a corre
  sempre por último — medido). Proposta feita, sem resposta; perguntar de novo ao abrir a W2.

## Feito (verde)

| Peça | Onde | Prova |
|---|---|---|
| Lei pura: `Pose2`, `blend` (arco curto via `ph2d_skeleton::blend_angle`), `ChildOf`, `CopyTransform` (canais, Replace/Add, World/Local), `LookAt`, `Limit` (faixas + arco via `clamp_to_limit`), `Distance` (Inside/Outside/Surface), `Kind::keep_current` (porta única do desvio) | `crates/ph2d-constraints/` | `cargo-test-narrow ph2d-constraints` 11/11 (6 contra o oráculo Blender) |
| ECS: `Constraints { items, next_id }`, `Item {id, kind, target: Option<StableId>, influence, enabled}`, `ordem()` topológica + ciclo, `evaluate()` (parte do AUTORADO pelo ledger, corpo dinâmico recusado, devolve ao apagar/desligar, sem deriva de f32), `Estados` (resource) | `crates/ph2d-constraints-ecs/` | 9/9 (meia força não escorrega, ciclo, nunca grava, apagar devolve, dinâmico, Child Of sem salto numa hierarquia, ordem, alvo morto, snapshot) |
| `Driver::ConstraintPose` / `Driven::ConstraintPose(Transform)` (append) | `crates/ph2d-preview-drive/src/{lib,facto}.rs` | check do workspace da shell |
| Registo no boot + fase | `shells/desktop/src/init.rs` (registo), `render_loop/fase_skeleton_drives.rs` (`evaluate` ANTES do `skeleton_smart`) | shell **+6 −7** `.rs` (tecto `189 041` não sobe) |
| Descritor `authored`, `O::ANY`, `C::Transform`, sem campos + `component.constraints.{name,keywords}` | `ph2d-component-desc/src/catalog/constraints.rs`, `ph2d-i18n/src/component_{catalog,keywords}.rs` | `every_registered_component_is_described` 6/6 |
| Porta das macros `Recusado(SEM_REMAP)` (o `StableId` do alvo não é remapeado) | `ph2d-app-macros/src/mundo/portas.rs` | idem |

## Falta na W1 (por esta ordem)

1. **Família `ph2d-app-constraints`** (molde `ph2d-app-light2d`: `FAMILY`, `smoke::ROUTERS` com
   `PH2D_CONSTRAINT_SMOKE`, `encenar(world, n)`), ligada em `shells/desktop/Cargo.toml` + bloco de
   `ph2d-app-registry-init` (`Cargo.toml` feature `app-constraints` + `src/lib.rs:60-75`); gate
   `the_shell_links_exactly_the_registered_families`.
2. **Cartão** — o caminho exacto do `path_follow` (mapa levantado nesta janela):
   - modelo: `ph2d-app-components/src/path_follow_inspector.rs:24` (`build_*_info(world, bits, …)`) →
     shell `render_loop/snapshots_inspector.rs:426-433` (`late`) → `:201`
     `ph2d_panel_inspector::set_current_inspector_*` → `ph2d-panel-inspector/src/state_components.rs:54/58/238`.
     O tipo do modelo vive em `ph2d-editor-core` (`path_follow_edits.rs:27`).
   - ids de lista: arrays `const` de tamanho fixo (`ids/inspector_timer.rs:55-71`, `INSP_TIMER_ROW: [NodeId; 16]`),
     registo `populate.rs:122` (`register_button_ids`), pintura por `zip` (`sections/timers.rs:110-117`),
     tecto `:196`, evento por `position` (`event_timer.rs:34`).
   - slider: `populate_anim.rs:~36` (`InteractiveState::Slider`), `sections/anim.rs:246-259`
     (`hit_index.register` + `widget::paint_slider_track`), evento `WidgetEvent::ValueChanged(id)` lido com
     `host.store().slider(id)` (`event_anim.rs:156-164`); não sobrescrever o espelho durante `Dragging`
     (`sync_sections.rs:307-316`). Checkbox: `property_row.rs:472/535`. Botão: `widget/button.rs:235`.
   - secção: `paint_optional_movers.rs:138-188` (begin/finish_section) + `paint_familias.rs:189` (`plano.push`)
     + ordem em `paint_optional.rs:315-331`.
   - edição: `EditorAction::InspectorComponentEdit { entity_bits, edit: ComponentEdit::<X>(…) }`
     (`ph2d-editor-core/src/action_bus_component.rs:59`; empacotar como `event_path_follow.rs:128-133`) →
     shell `fase_bus_inspector.rs:229` → `fase_inspector_commits_top20.rs:57-76` → `apply_all` da família.
     ⚠️ undo: confirmar onde `any_input_this_frame` liga numa edição de Inspector (`undo_app.rs:430`).
   - pick no palco: o único fluxo de produção é o das juntas — armar em `fase_bus_inspector.rs:465-468`
     (`joint_body_pick` em `app_state.rs:684`), resolver em `input_dispatch/despacho_clique_select.rs:89-95` →
     `despacho_metodos_picks_e_arrastos.rs:64-96` (`pick_sprites_at_world`). Generalizar este, não criar outro.
   - seam test no molde `ph2d-panel-inspector/tests/it/seam_path_follow.rs` (gate
     `architecture_interactive_crate_has_behavioral_test`; `BEHAVIORAL_TEST_DEBT` é vazia e não aceita entrada).
   - ⚠️ **Cada linha nova na shell paga-se a mover/cortar** (tecto exacto). Preferir que a família exponha
     uma função por ponto de contacto e a shell só a chame.
3. **Canal animável da força**: família `CONSTRAINT = 8` (append em `ph2d-ecs/src/canal.rs:212`, ficheiro
   irmão `canal_constraint.rs`; `campo` = id do parâmetro, `i:j` = `Item::id`). Publicar/aplicar no molde
   `canais_componente.rs` (chamadas em `fase_autokey.rs:57` e `fase_timeline_drain.rs:93`).
4. **Gizmos** (linha ao alvo, arco/caixa/círculo do limite) no molde `ph2d-app-skeleton/src/bone_limit.rs:107`.
5. **Bake** para chaves (porta do Auto Key) + força 0 no intervalo.
6. **`look_at` do Motion chama `ph2d_constraints`** (a lei hoje é privada em
   `ph2d-node-motion-look-at/src/lib.rs:390`) — uma lei, dois consumidores; sem mexer no `NodeOp`.
7. **ADR-0192** (próximo livre; `docs/architecture/decisions/`), `PROJECT_SCHEMA` `193 → 194` com o parágrafo
   na escada (`shells/desktop/src/project_schema.rs:494`; componente NOVO registado; espelhos `ph2d-render`/
   `ph2d-script` **não** mexem — registo próprio).
8. **Smoke 1–3** do plano §6, com a sonda headless e foto ANTES da mensagem ao dono.
9. Fecho da onda: provas de mutação (`blend` arco curto, `ordem`, ciclo, `base` autorada), `/pd-onda-fechar`.

## Armadilhas medidas nesta janela

- **Partir do autorado é obrigatório**: misturar a partir do `Transform` vivo faz uma força de 50 % escorregar
  até ao alvo quadro a quadro (a âncora de IK mistura do vivo — não copiar esse padrão). Gate
  `a_half_influence_does_not_creep_frame_after_frame`.
- **Sem mudança devolve o autorado ao bit**: `inverse_compose(compose(p, t))` em f32 mexe o último dígito.
- O Blender do pacman só arranca com `LD_LIBRARY_PATH` para uma `libcudart.so.13` (comando no cabeçalho de
  `ferramentas/oraculo_blender.py`); o hook da placa exige `PH2D_GPU=1 bash scripts/ph2d-run.sh …`.
- Ids/variantes novos (colisão na integração): `Driver::ConstraintPose`, `Driven::ConstraintPose`,
  `"ph2d::constraints::Constraints"`, descritor/chaves `component.constraints.*`, a linha de `PORTAS`.
