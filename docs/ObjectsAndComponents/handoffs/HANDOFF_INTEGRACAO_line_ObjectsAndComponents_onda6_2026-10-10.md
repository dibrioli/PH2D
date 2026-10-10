# HANDOFF DE INTEGRAÇÃO — `line/ObjectsAndComponents`, onda 6 (10/10): esticar em grupo uma imagem inclinada

> Leitor: o agente integrador (e a próxima janela desta linha). Plano e lei:
> [`01_objeto_e_componente.md`](../01_objeto_e_componente.md) §16. **As SEIS ondas vão juntas na mesma
> branch, nenhuma está no `main`** — integre-as num só passo; a superfície de colisão das ondas 1–5 está
> no §4 de cada handoff ([onda 5](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda5_2026-10-10.md) é o
> anterior). Ondas 1–5 aprovadas no smoke pelo dono.

## 1. Identidade

- Branch `line/ObjectsAndComponents` · worktree `Worktrees/line-ObjectsAndComponents` · merge-base `2a925859d`
  (= o `main`, sem rebase). Onda 6 por cima de `76ef418e6`.
- Commit: `a8c78b98b` (lei + shell + smoke `=6`) + os commits deste handoff.
- Diff (`76ef418e6..HEAD`, sem docs): 6 ficheiros, `+334 −57`. **Shell `+58 −57`** (a conta do membro saiu
  para a fundação).

## 2. O que entrou

- `ph2d_editor_core::gizmo::group` — `group_member_pose(start_local, start_parent, live_parent, parent_skew,
  (live_local, live_skew), GroupMotion) -> MemberPose` e `skew_under_world_scale`. Sem skew, a conta que
  vivia em `ramo_gizmo_escrita_grupo`, bit a bit (gate).
- Shell `gizmo_drag_escrita.rs`: o laço chama a lei e escreve `skew_x/skew_y`; `gizmo_drag_calculo.rs`: a
  regra da escala uniforme lê `parent_world.rotation + start_transform.rotation`.
- Smoke `PH2D_ANIMACAO_SMOKE=6`.

## 3. Versões e contadores

Nada: `DOC_VERSION` 20, `PROJECT_SCHEMA` 187, registos, ids e i18n sem diff. Nenhum ADR.

## 4. Superfície de colisão

| sítio | mudança |
|---|---|
| `ramo_gizmo_escrita_grupo` (shell) | o corpo do laço passou a `gizmo::group_member_pose`; escreve o skew |
| regra «rodada ⇒ uniforme» (shell `gizmo_drag_calculo.rs`) | rotação de MUNDO |
| `ph2d_editor_core::gizmo` | módulo `group` novo (exports `GroupMotion`, `MemberPose`, `group_member_pose`, `skew_under_world_scale`) |

## 5. Portão de fecho

`clippy -p ph2d-editor-core -p ph2d-host-desktop --all-targets -D warnings` ✓ · `fmt` ✓ · gizmo: fundação
85/85, shell 71/71 · `the_shell_only_shrinks` ✓ · `file_loc_caps` ✓ · `nextest-impacted` 16094/16094 (§8).

## 6. Provas de mutação — 5/5 sangraram

skew intacto · `fₓ/f_y` trocados · o `global` ignorado (o skew muda também no pivô local) · o desconto do
skew do PAI (sobreviveu à 1.ª corrida: nenhum teste tinha pai inclinado — o 6.º teste,
`under_a_skewed_parent_the_world_skew_is_the_one_the_law_stretches`, nasceu dela e a faz sangrar) · a
escala de mundo sem o pai. A costura da shell (o App do arrasto precisa de GPU) prova-se pela lei e pelo smoke.

## 7. Smoke

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-ObjectsAndComponents
PH2D_ANIMACAO_SMOKE=6 ./target/smoke/ph2d-host-desktop
```

A Bandeira (inclinada, sem rotação) e a Caixa vêm seleccionadas, com a caixa do grupo à volta das duas
(fotografado). Puxar a alça do meio à DIREITA da caixa do grupo: as duas esticam na horizontal e a
inclinação da Bandeira fica mais deitada — como o desenho esticado. Errado: a Bandeira esticar mantendo o
ângulo, ou deformar-se para fora da caixa do grupo.

## 8. Fecho

- `BASE=76ef418e6 nextest-impacted.sh`: **16094/16094** ✓ (114 s, load 12→35).
- `rm -rf target/*/incremental` feito; build `smoke` 1.ª `27.51s`, 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.28s
```

- `PH2D_ANIMACAO_SMOKE=6` fotografado (1930×1040): as duas seleccionadas e a caixa do grupo a envolver os
  cantos DESENHADOS. O arrasto não se fotografa (a tela virtual não aceita eventos) — é o passo do dono.
- **O que fica aberto:** nada desta linha. Próxima onda ou integração: decisão do DONO.

## 9. Integração no `main` (10/10, ordem do dono: «vc mesmo pode integrar ao main. Faça»)

- **Uma linha só, ordem trivial.** merge-base == `main` (`2a925859d`): rebase vazio, **`--ff-only`**, zero
  conflitos, zero marcadores. Os passos do `foundational-integrate.sh` corridos à mão: o script recusa
  porque o primário tem ficheiros por registar FORA da `project-memory/` que NÃO são desta linha
  (`crates/ph2d-tool-painter/tests/sonda_tracos_por_meio.rs`, `docs/Painter/NewFeaturesWetPaint/` — de
  outra janela; nenhum caminho no diff da linha, não tocados).
- Codegen (`tool`/`node`/`app`/`panel-sync`): nada a regenerar. Staleness dos registos ✓.
- Árvore combinada (== a ponta da linha): `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ ·
  `clippy -D warnings` ✓ · `fmt` ✓ · `machete` ✓ · `check-standalone-optional` ✓ · `check-workflow-packages` ✓ ·
  censos 12/12, 114/114 ✓ · `BASE=main nextest-impacted` **19030**, 3 ✗ de razão de relógio do
  `ph2d-tool-painter` (`mask_gate_tests::…follows_the_footprint…`, `mask_tests::the_mask_stroke_cost…`,
  `wetpaint::tests::a_wet_move_costs…`) a load ~24 — os três na tabela de `FLAKES_DE_CARGA.md`, 3/3 verdes
  sozinhos, e a linha não toca máscara nem wet paint · `the_shell_only_shrinks`, tectos de LOC,
  `ph2d-panel-registry-init` 124/124 ✓.
- Números que somam: `PROJECT_SCHEMA` **187** (sem degrau nas seis ondas; tripla `(187, 13, 22)`), registos de
  componentes da onda 3 (ecs 106 / render 107 / script 107), `DOC_VERSION` da timeline 18 → 20, nenhum ADR.
- **Para as linhas vivas** (motion-value, UIUX, PainterWatercolor, components, MiroClone, Luz2D): o §4 de cada
  handoff das ondas 1–6 é a lista do que o rebase delas encontra — em especial a porta única da lista que rola
  (`widget::paint_dropdown_popover_scrolled` agora `pub(crate)` + gate `architecture_dropdown_lists_use_the_door`)
  e `motion_state_demo_router.rs` (687/700, tecto SOMADO) com a motion-value.
- Push: **não** — só por ordem do dono.
