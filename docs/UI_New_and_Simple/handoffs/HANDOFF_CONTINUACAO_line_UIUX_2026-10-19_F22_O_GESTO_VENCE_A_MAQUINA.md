# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 19/10 — F22 (o gesto do artista vence a máquina dos estados; o olho diz «misto») feita

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (já atual, sem rebase). Anterior:
  [F21](HANDOFF_CONTINUACAO_line_UIUX_2026-10-19_F21_O_MISTO_COMPLETO.md).
- Fecha **F21 ABERTO 4** (olho misto), **F19 ABERTO 2** (isenção das abas sem gate), **F19 ABERTO 5** (edição durante o Show sobrescrita), **F19 ABERTO 6** (Preview × Hierarquia: medido, **não é defeito**) e **F18 ABERTO 6** (os sinais chamam `request`: medido, **não é defeito** — só agem no Preview, onde o hover usa a mesma porta; doc em `ui_state_bridge::request`).
- Plano: [`33_plano_F22_o_gesto_vence_a_maquina.md`](../vetor_nova_ui/33_plano_F22_o_gesto_vence_a_maquina.md) (§3 = desenho, §6 = gates, §9 = feito).

## 1 — Commits

| commit | o quê |
|---|---|
| `3320239de0` | docs: plano 33 |
| `d3b578cfdb` | feat: todo o código (§2) + cenas 35 e 36 |
| (este commit) | docs: handoff, plano §9, fotos |

## 2 — Superfície de colisão e decisões que um merge pode partir

**A máquina dos estados (`ph2d-ui-state`)**
- `Machine`: campos privados `written: Option<Vec<ObjectPose>>` e `adrift: bool`; métodos `note_written(read)`, `yield_to(read, motor) -> bool`, `is_adrift()`. `arrive`/`retarget` limpam `written`.

**A ponte (`ph2d-app-vec` `ui_state_bridge`)**
- `dispatch` mudou de assinatura: `(machines, states, world: (&mut SimWorld, &mut VecScene, &VecEntityMap), dt, out: &mut Cooked, motor: &dyn Fn(VecPathId) -> bool)`. Todos os chamadores atualizados (`bool_live_bridge_tests`, shell `morph_set_*_tests`, `ui_preview_tests`, `fase_ui_state_preview`).
- Novo `ui_state_bridge::motor(map, &PreviewDrive)`. `Cooked.pousou: bool` (campo novo; `Cooked` é `Default`). `live_role` devolve `None` quando `is_adrift && !is_animating`.
- `PH2D_UI_STATE_LOG=1` regista cada pouso e cada id conduzido por motor.

**Shell**
- `fase_ui_state_preview.rs`: `self.any_input_this_frame |= self.ui_cooked.pousou;` logo após o `dispatch` (gate `the_gesture_that_stops_a_show_counts_as_input` em `shells/desktop/tests/it/the_ui_state_machines_run_and_undo_waits.rs`).

**Ferramenta (`ph2d-tool-vector` `tool_adopt.rs`)**
- `adopt_fill`/`adopt_stroke`/`adopt_markers` mantêm qualquer campo ARMADO (`self.authored`): RGB, A, WIDTH, CAP, JOIN, ALIGN, cada número do tracejado, marcadores. Achado pela cena 35: durante um Show a ponte readotava o estilo «por mão alheia» depois de o seletor armar a cor, e o passo 4 escrevia a cor adotada.

**Painel / olho**
- `panel-vector` `RowMixed.enabled` (campo novo; construtor literal quebra) + `paint_stack_rows::eye_icon` (`IconId::Minus` quando misto); `app-vec` `paint_stack_mixed::row_mixed` calcula-o.

**Cenas e roteador**
- Novos `vector_mode_gesto.rs` (cena 35, estágios 22..=24; começa pela cena 25 via `interface::end` para `h == 21`) e `vector_mode_olho.rs` (cena 36, 33..=36). `route::FAIXAS` passa a 11 faixas; `vector_mode_scenes.rs` agora 9..=36, trata 21 → `interface::FIRST`, 22 → `olho::FIRST`.

**O que um merge pode partir**
- Qualquer chamador de `dispatch` na forma antiga de 7 argumentos; construtor literal de `RowMixed`.
- Qualquer escritor que reescreva os canais de pose de um hospedeiro de estados A CADA QUADRO sem se declarar no `PreviewDrive`: passa a parar os Shows (medido: nenhum na cena 29).
- Estágio de cena fora de `route::FAIXAS`.
- `PROJECT_SCHEMA` intocado. Contratos congelados: `git diff 3ee22f984b..HEAD --stat -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-editor-core/tests/it/architecture_tool_contract_surface.rs crates/ph2d-vector-doc/src/primitives.rs` → **saída vazia**.

## 3 — Smoke (por fazer pelo dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=35 ./target/smoke/ph2d-host-desktop`
   — o botão azul vira componente com Default azul e Hover laranja (a cena 25), a Duration vai a 2 s, o Show do Hover começa e, a meio, o seletor do Fill escolhe verde ⇒ o botão fica VERDE (#28C85A). Feche o seletor (×) e veja na aba Interface: nenhum papel aceso. Errado = o botão acabar laranja.
2. Idem `=36` — duas formas vermelhas, cada uma com contorno amarelo extra: visível na esquerda, escondido na direita. Em Appearance a linha Stroke tem «—» no lugar do olho. Clique nele ⇒ as duas ficam iguais e o olho volta. Errado = olho aberto/fechado.
3. Manual: na 25, Duration 2 s, Show Hover e arraste o botão no canvas a meio ⇒ ele fica onde o soltou; Ctrl+Z desfaz tudo de uma vez (volta a antes do Show).
4. Manual: Preview ligado, apague um botão com estados na Hierarquia, saia do Preview ⇒ ele não volta.

Fotos (`docs/UI_New_and_Simple/vetor_nova_ui/fotos/`): `F22_cena35.png`, `F22_cena36.png` (1930×1012, zoom 98 %). Os `.log` irmãos ficam não rastreados. Tempo de quadro: **não medido**.

## 4 — Verificação

- **Vermelho primeiro OBSERVADO:** `gesto_tests` (3 vermelhos antes da cura: `uma_edicao…`, `um_arrasto…`, `depois_do_gesto…`), `o_olho_diz_misto`, `adopting_never_overwrites_a_field_armed_this_frame`, `uma_cor_escolhida_enquanto_a_forma_muda_por_mao_alheia_chega` (vermelho com a cura do adopt revertida).
- Testes novos: `ph2d-app-vec` `ui_state_bridge_gesto_tests.rs` (5); `machine_tests` `yield_to_*` (4); `tool_armed_tests` (1); `paint_stack_rows` (1); `misto_cor_tests` (2); shell `ui_preview_tests` `a_host_deleted_during_the_preview_stays_deleted_after_leaving`; `tests/it/a_aba_do_encaixe_passa_no_preview.rs`; `the_gesture_that_stops_a_show_counts_as_input`; `vector_mode_scenes_tests` `as_cenas_35_e_36…`; teste da fixture de `vector_mode_olho`.
- Prova de mutação **12/12 sangrou**: M1 `yield_to` sempre false · M2 comparar com a pose viva (f32) · M3 ignorar o motor · M4 sem `note_written` · M5 sem `pousou` · M6 sem `adrift` em `live_role` · M7 `row_mixed.enabled` · M8 `eye_icon` · M9 `adopt_fill` · M10 `adopt` da width · M11 isenção da aba de slot · M12 input da shell.
- Bateria por nome (verificador), verde: ui-state 22+4; app-vec `ui_state` 42 / `gesto` 5 / `misto` 21 / `pilha` 7 / `vector_mode` 33 / `bool_live` 32 / `vector_bridge` 45 / `paint_stack` 10; tool-vector `armed` 4; panel-vector `seam_paint_stack` 14 / `seam_ui_states` 13; shell `ui_preview` 18 / `morph_set` 33 / `ui_states` 10 / `ui_spring` 8; `architecture_workspace_file_loc_cap`, `panel_wiring_parity`, `tool_contract_surface`, `the_shell_only_shrinks`.
- clippy `fecho` `-D warnings --all-targets` em 5 crates: verde. `fmt` limpo.
- Sonda `PH2D_UI_STATE_LOG=1`: cena 35 → 1 pouso, 0 motor; cena 29 (Show sem toque, controlo) → 0 pousos, 0 motor.
- `nextest-impacted` **não corrido** (onda, não fecho de linha).

## 5 — ABERTO

1. Smoke do dono (§3).
2. **Fechados aqui:** F21 ABERTO 4; F19 ABERTO 2, 5, 6; F18 ABERTO 6 (os dois últimos por medição, não por código).
3. A opacidade do objeto NÃO é canal de estado (`capture`/`install` não a escrevem; `ObjectPose.opacity` é o fade de entra/sai): pergunta de produto — um Hover que mude a opacidade não anima.
4. Opacidade conduzida pela timeline, no painel: o painel lê `VecPath.opacity`, a tela `VecDrivenStyle.alpha`, e nenhum campo tem marcador de «animado» (só o diamante da dope-sheet). Família timeline × painel.
5. Tracejado de vários pares na placa: `ph2d-shape-gpu/src/eixo.rs:129` só aceita 1 par.
6. F20 ABERTO 6 (`bindings::token_world`) fica.
7. Herdados: F18 3 (aparência checkbox/toggle — o BUGS #26 citado é uma linha de índice em `docs/Vector Module/BUGS_vector.md:567` sobre altura de widgets do painel; a extrapolação ao canvas não foi verificada), F18 4 (Preview com rato real não fotografado), F16–17 (profiler das sondas do Motion, grupos visuais).
8. F21 ABERTO 7 (flake `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`) não tratado.
9. `CLAUDE.md` §5.1 UI/UX: **não** alterado; ao fechar a linha, trocar o link do último handoff e o intervalo do smoke (`…26` → `…36`).
10. Suíte inteira (`nextest-impacted`) não corrida nesta onda.

Binário do smoke: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` → 2.ª saída: «Finished smoke profile [optimized] target(s) in 0.42s» (zero `Compiling`)

> ✅ **Smoke do dono: OK (19/10)** — cenas 35 e 36 e os passos manuais. Pedido seguinte do dono (19/10): *«passe a animar a opacidade. faça aqui e agora»* → F22b.
