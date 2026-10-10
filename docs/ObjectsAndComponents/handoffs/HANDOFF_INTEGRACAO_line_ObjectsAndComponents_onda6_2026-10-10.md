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
85/85, shell 71/71 · `the_shell_only_shrinks` ✓ · `file_loc_caps` ✓ · `nextest-impacted` sobre `76ef418e6..HEAD`: ver §8.

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

FECHO6
