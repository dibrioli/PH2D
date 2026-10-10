# HANDOFF DE INTEGRAÇÃO — `line/ObjectsAndComponents`, onda 5 (10/10): os abertos da onda 4 (skew do gizmo, a porta da lista que rola, o ▾ do Play Clip, a guarda do reset, o registry-init com `-p`)

> Leitor: o agente integrador (e a próxima janela desta linha). Plano e leis:
> [`01_objeto_e_componente.md`](../01_objeto_e_componente.md) §15. Ondas anteriores:
> [onda 1](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md) ·
> [onda 2](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda2_2026-10-08.md) ·
> [onda 3](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda3_2026-10-08.md) ·
> [onda 4](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda4_2026-10-09.md). **As CINCO ondas vão
> juntas na mesma branch, nenhuma está no `main`** — integre-as num só passo. As colisões das ondas
> 1–4 continuam (§4 de cada handoff); as desta onda estão no §4 abaixo.

## 1. Identidade

- Branch `line/ObjectsAndComponents` · worktree `Worktrees/line-ObjectsAndComponents` · merge-base
  `2a925859d` (= o `main`; não andou, sem rebase). Onda 5 por cima de `947b1e4a5`.
- Commits da onda 5: `2ab6de77e` (d) · `c3bd168a3` (f) · `95803feaa` (a) · `82c90cfc2` (c) ·
  `a6c4bf51f` + `00e5caaff` (smoke `=4`/`=5`) · `21c08a3e1` (b) · `71dad8a0c` (fmt) + os commits deste handoff.
- Diff da onda (`947b1e4a5..HEAD`): 81 ficheiros, `+2996 −1365`. **Shell: `+245 −47`** (a maior parte
  testes; `the_shell_only_shrinks` verde). Código de painel encolheu ~880 linhas (as 14 cópias).

## 2. O que entrou (nenhuma decisão de produto nova)

- **(a) Gizmo de sprite — a premissa caiu, o defeito real era o SKEW.** O motor compõe pai×filho
  somando rotações e multiplicando escalas (ADR-0025 emenda 1 §2.2.1): um filho rodado sob um pai de
  escala não uniforme é desenhado retângulo e a caixa de sempre já coincidia (gate
  `a_non_uniform_parent_does_not_shear_a_rotated_child`, com o controlo do produto exacto). O skew
  (editável no Inspector) é que faz o paralelogramo. `ph2d_editor_core::gizmo`: `skew_about`
  (`L = R·Sk·R⁻¹` no pivô), `sprite_box` (a conta afim→caixa saiu da shell; caixa em `T·R·S`, skew em
  `model`), `drag_skew_model` + `drag_in_skew_frame` (escalar e mover o pivô leem cursor e pen-down pela
  inversa; transladar/rodar/global são rígidos sobre o desenho e ficam de mundo),
  `skew_translation_out`, `GizmoView::world_aabb`. `ph2d_ecs::world_skew` (soma da cadeia, sem alocar,
  Floyd). Na shell: a união da global envolve os cantos PINTADOS (ignorava a rotação de cada vista) e
  já não herda o `model` da primeira; o snap do pivô e o centro do conteúdo usam a pose de MUNDO (usavam
  a local — errado sob pai rodado/escalado).
- **(b) UMA porta para a lista que rola** — `panel::paint_dropdown_list` (clamp + escrita da rolagem
  clampada, só a faixa visível no hit e como focável, pintor de linha opcional `PopoverRowPainter`,
  publica) + `visible_option_bands`. Eram **14** cópias (vector 9, skeleton 2, painter-layers, flip,
  authored) e mais 5 primas (hierarchy, timeline sources/animation, color-equalization, flip-frames;
  as 3 últimas não rolavam). Defeito confirmado RED: o polegar das listas de ícone/fonte do vector
  nunca se lia `Dragging` (`scrollbar_visual` sem dono) — `scroll_bar_panel` resolve agora a barra do
  dropdown no dropdown aberto. Os pintores da lista em `widget` passaram a `pub(crate)`. A ficha do
  tipo nas fontes do Arrange (perdida em `e34f9d17c`) voltou.
- **(c) O parâmetro do Play Clip é um ▾** com os clipes da Animação do ALVO quando o alvo é UM
  objecto (nome ou «este»); por tag / «quem bateu» fica o texto livre (o toast no Play continua).
  Recurso `ph2d_ecs::ClipNames` publicado no fim do `timeline_persist::upkeep` (só reescreve quando o
  documento muda — sem alocar no estável). `target_of` da ponte partiu-se: `alvo_nomeado(&World, …)`
  (só leitura) + as metades `stable_id_named`/`entity_with_stable_id`. Nome fora da lista ⇒ a linha
  de aviso por baixo (o idioma da secção; o chip de dropdown não tem estado de aviso no design system).
- **(d) A guarda do reset** resolve o nome do osso inteligente NA animação onde ele vive: um homónimo
  do dono que saiu já não segura a composição rançosa. `animation_of` desceu do `ph2d-app-skeleton`
  (privada) para `TimelineDoc::animation_of` — uma porta, dois chamadores.
- **(f) `cargo test -p ph2d-panel-registry-init`**: 47/77 → **124/124**. Dev-dependency da crate em si
  própria com as 4 features que só o shell liga (os censos medem o APP); o `default` não muda.

## 3. Versões e contadores (DELTA da onda 5)

- `DOC_VERSION` da timeline **sem diff** (20; contra a base `2a925859d`: 18 → 20). `PROJECT_SCHEMA` **187,
  sem diff**. Registos de componentes **sem delta** (`ClipNames` é estado de execução, não registado).
  `ObjectKind` sem diff. Contratos congelados intocados. Nenhum ADR.
- Ids novos: `INSP_ACTION_CLIP_PICK`, `INSP_ACTION_CLIP: [NodeId; 16]` (gate `== MAX_CLIPS`).
  i18n: `panel.inspector.actions.clip_not_in_target_animation`, `panel.inspector.actions.target_has_no_animation`.
- Smoke: `PH2D_ANIMACAO_SMOKE=4` e `=5` (níveis novos).

## 4. Superfície de colisão — o que um merge pode partir

| sítio | mudança |
|---|---|
| `widget::paint_dropdown_popover_scrolled` e irmãos | **`pub(crate)`** — um painel de OUTRA linha que os chame não compila; a cura é `panel::paint_dropdown_list` |
| gate `architecture_dropdown_lists_use_the_door` | reprova `DROPDOWN_SCROLLBAR_ID`, `option_rect_in(_scrolled)(`, `popover_rect_clamped(`, `set_dropdown_popover(` em código de `crates/ph2d-panel-*/src`; piso de 19 ficheiros a chamar a porta — **conta-se** se outra linha mexer em listas |
| `PaintedPopover` | campo `scroll` (literal sem `..` reprova) |
| `InspectorActionRow` | campo `arg_options: Option<Vec<String>>` |
| `ph2d_ecs::signal_actions` | `target_of` → `pub fn alvo_nomeado(&World, …)`; `stable_id.rs`: `stable_id_named`, `entity_with_stable_id` |
| `ph2d_ecs` | `ClipNames`, `world_skew` (exports em `lib.rs`) |
| `TimelineDoc::animation_of` | novo; `ph2d-app-skeleton::smart::animation_of` saiu |
| `timeline_persist::upkeep` | publica `ClipNames` no fim; `purge_the_dead` recebe `(&World, &[(Entity, String)])` |
| `interaction::…::scroll_bar_panel` | a barra do dropdown resolve-se no dropdown aberto |
| `ph2d-editor-core` `[dev-dependencies]` | `ph2d-ecs` (aresta de DEV) |
| `ph2d-panel-registry-init` `[dev-dependencies]` | ela própria, com 4 features — **`Cargo.lock` regenera, nunca à mão** |
| shell | `snapshots_gizmo.rs` (`sprite_view` → `sprite_box`; união da global), `gizmo_drag.rs`/`gizmo_drag_calculo.rs` (skew no arrasto; snap do pivô em mundo), `animacao_smoke.rs` (`=4`), `snapshots.rs` (+ mod de teste) |

- **Colisão com a `line/motion-value`** (viva): nesta onda **nenhum** ficheiro de `ph2d-app-motion`.
  `Cargo.lock` como sempre. As da onda 4 continuam.

## 5. Portão de fecho (batched sobre `947b1e4a5..HEAD`, 10/10)

| passo | | tempo · load |
|---|---|---|
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ | 40 s · 10→24 |
| `clippy --workspace --all-targets -D warnings` | ✓ | 50 s · 24→27 |
| `fmt --all --check` · `machete` · `check-standalone-optional` · `check-workflow-packages` (32/372) | ✓ | — |
| censos da árvore combinada | ✓ 12/12, 114/114 | 71 s · 20→35 |
| `BASE=947b1e4a5 nextest-impacted.sh` | 16931/16932 — o ✗ era `instructional_docs_only_cite_paths_that_exist` (o CLAUDE.md já apontava ESTE handoff, ainda por escrever); re-corrido depois dele: ver §11 | 182 s · 28→24 |
| `the_shell_only_shrinks` + tectos de LOC (panel, widget, workspace, shell) | ✓ | — |
| `cargo test -p ph2d-panel-registry-init` (sem features) | ✓ 124/124 | 94 s |

## 6. Provas de mutação

**20 mutações, todas sangraram nas leis que guardam** (cada uma restaurada com `checkout` + `touch`; controlo verde antes; agulha com 1 casamento):

- (a) + (d), janela principal (11): `skew_about` sempre identidade · colunas cruas sem a inversa ·
  `start_cursor_world` sem mapear · `skew_translation_out` devolve `local` · `Translate` no
  `drag_skew_model` · `world_aabb` sem rotação · `[0,0]` em vez de `world_skew` na `sprite_view` · a
  global a herdar o `model` · `world_skew` sem somar · `animation_of` sempre `SCENE` · a guarda do reset
  invertida. ⚠️ `animation_of ≡ SCENE` **não** sangra nos `reset_guard_tests` e é EQUIVALENTE ali: o
  reset só chega à guarda quando nenhum clip tem dono, logo toda a resolução já cai na Cena — quem a
  apanha são os `smart::list_tests` (osso da cópia, osso que vê a animação onde vive).
- (b) (4 leis, 14–18 testes cada): escrita da rolagem clampada · filtro da faixa visível · dono da
  barra (`Dragging`) · registo dos focáveis (authored/hierarchy sobrevivem a esta: o `populate` deles
  já regista as opções — esperado).
- (c) (4): `Named ⇒ Some` invertido · alvo ignorado · «só reescreve quando muda» → sempre · índice+1
  no despacho.
- (d)/(f) antes: a guarda antiga (`true`) — sangrou só `um_homonimo_de_outro_dono_nao_segura_o_reset`.
- Sem teste: a costura do arrasto de escala/pivô com skew na shell (`gizmo_drag_calculo.rs`) — §10.

## 7. Testes novos (principais)

`ph2d-editor-core` `gizmo/sprite_box_tests` (5, oráculo `GlobalTransform::from_transform`),
`tests/it/the_dropdown_list_door` (3), `architecture_dropdown_lists_use_the_door` (2) · shell
`skew_gizmo_tests` (3, pelo passe real `publish_gizmo` + `global_view`) · `ph2d-ui-testkit::prova_que_a_lista_longa_rola`
corrida por painel (painter-layers 1, flip 1, skeleton 2, vector 9, authored 1) · `ph2d-timeline-persist`
`reset_guard_tests` (3) e `clip_names_tests` (3) · `ph2d-app-components` `signal_actions_inspector_clip_tests` (5) ·
`ph2d-panel-inspector` `tests/it/action_clip_is_a_dropdown` (3, gesto real).

## 8. Premissas que caíram

1. «O filho de um pai de escala não uniforme mostra caixa sem cisalhamento» — o motor não cisalha
   filhos (§2.2.1); a caixa estava certa. O paralelogramo vem do SKEW.
2. «Cinco painéis com cópia própria» — eram 14 (+5 primas, 3 delas sem rolagem nenhuma).
3. «O Inspector não pode ter um ▾ de clipes porque não vê o documento» — vê o MUNDO; o lado da timeline
   publica um recurso.
4. «76 falhas» — 77, e nenhuma era defeito: censos do app inteiro corridos sem 4 painéis.

## 9. Smokes (binário `smoke` compilado na worktree)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-ObjectsAndComponents
PH2D_ANIMACAO_SMOKE=4 ./target/smoke/ph2d-host-desktop   # (c) ▾ do Play Clip (Herói seleccionado)
PH2D_ANIMACAO_SMOKE=5 ./target/smoke/ph2d-host-desktop   # (a) a Bandeira com skew seleccionada
PH2D_VIDA_SMOKE=1 ./target/smoke/ph2d-host-desktop       # (b) uma lista longa que rola (o ▾ «All»)
```

- `=4`: o Herói vem seleccionado; no Inspector, a secção Signal Actions tem «Play Clip» com o ▾ do
  parâmetro em «pular» — abre e mostra «andar» e «pular». Clicar na Bandeira (verde, inclinada): a
  caixa do gizmo é um paralelogramo colado ao desenho; puxar um canto mantém o canto oposto parado.
- (d) e (f) provam-se por testes; (b) nos 14 painéis pelo teste de costura partilhado.

## 10. O que fica ABERTO

- O arrasto de grupo (gizmo GLOBAL) de uma selecção com sprites com skew escala no mundo; uma escala
  não uniforme de grupo sobre um filho com skew não é exprimível em `T·R·Sk·S` (o mesmo limite que já
  força escala uniforme em selecções rodadas) — não medido como queixa; fica registado.
- A costura do arrasto de escala/pivô COM skew na shell (`gizmo_drag_calculo.rs`) prova-se pela lei
  pura (`a_corner_drag_on_a_skewed_sprite_pins_the_drawn_opposite_corner`) e pelo smoke: o `App` do
  arrasto precisa de GPU e não tem arnês headless.
- **Próxima onda: decisão do DONO**; o integrador não a abre.

## 11. Fecho

- `rm -rf target/*/incremental` feito; `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 1.ª `39.11s`, 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.26s
```

- `PH2D_ANIMACAO_SMOKE=4` e `=5` fotografados (`fotografa_cena.sh`, 1930×1040): `=4` abre com o Herói e a
  secção «Signal Actions (1)»; `=5` com a Bandeira — as oito alças nos cantos e meios das arestas do
  paralelogramo desenhado.
- O ✗ do `nextest-impacted` (`instructional_docs_only_cite_paths_that_exist`) cura-se com ESTE ficheiro;
  re-corrido depois do commit dos docs.
