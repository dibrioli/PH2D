# Handoff — linha `line/FLIP`, continuação (2026-07-15)

> **Para o próximo implementador que pega a linha E o rotate/escala da pose (W7.5).**
> Modo L: worktree `Worktrees/line-FLIP`, branch `line/FLIP`. **Você NÃO integra nem pusha**
> (§0.7 do CLAUDE.md) — fecha, escreve o handoff e o Enio ordena a integração via agente
> integrador. Este documento é o estado + o mapa preciso da Fase 2.

---

## 1. Estado da linha (o que está commitado, nada pushado)

`git log --oneline main..HEAD` (do mais novo pro mais velho):

| commit | o quê | smoke |
|---|---|---|
| `df809109` | **W7.5 Fase 1**: a pose da chave vira AFIM (`Pose`) | fundação, sem UI nova |
| `78754294` | falloff SIMÉTRICO 50%/quadro + número da célula não é mais lavado | **OK** (Enio) |
| `93b8bac7` | W7.4: falloff no FUNDO do quadro (cor de acento que clareia) | reestilizado por `78754294` |
| `67538a58` | régua de scrub ganha banda própria (frames não encolhem) | **OK** |
| `7d74b96f` | **W7.3**: régua de scrub (move o playhead sem desmontar a multisseleção) | **OK** |
| `3a95954b` | falloff visível na tira (barra=peso — depois virou cor em W7.4) | substituído |
| `3c94e28d` | criar quadro/instância no MEIO da tira + Unlink encerra multiframe | **OK** |
| `fce56856` | mover instância tremia + realce descolava (2 bugs do smoke W7.2) | **OK** |
| `bc21c20b` | docs (handoff de continuação anterior) | — |
| `ace54f41` | test: 1º teto do `pack_perf` também é por PERFIL | — |

**Todos os smokes de UI passaram** (régua de scrub, banda própria, falloff cor/simétrico/número,
criar no meio, unlink, mover/realce da instância). A Fase 1 do W7.5 é fundação (translate
byte-idêntico) e não muda a tela — não há smoke a fazer nela isoladamente.

**Gate suites verdes na Fase 1:** 76 testes `ph2d-flip` + 116 flip do shell + painel (`ph2d-panel-flip-frames`).

---

## 2. W7.5 Fase 1 — a pose da chave virou AFIM (FEITO, `df809109`)

**Por quê:** a pose (`FlipFrame`) era translação-só (`offset: Vec2`), e o Enio quer **girar/escalar**
uma instância. A pose agora é um afim `Pose([f32;6])`, no MESMO layout do `Xform` do shell.

- **`crates/ph2d-flip/src/pose.rs`** (NOVO): `Pose([f32;6])` — `IDENTITY`, `from_translation`,
  `coeffs`/`from_coeffs`, `is_identity`, `translation`, `translate` (pós-translada `[4]/[5]`),
  `apply` (afim·ponto, MESMA convenção do `Xform::apply` = `a·x + c·y + tx`), `lerp` (tween).
  ph2d-flip **só armazena** os coeficientes; a composição de rot/escala (multiplicação de afim +
  pivô) mora no SHELL, que tem `Xform`.
- **Superfície tocada** (mecânico): `frame.rs` (`offset: Vec2` → `pose: Pose`), `layer.rs`
  (`frame_pose`/`set_frame_pose`), `layer_time.rs` (`pose_at_cycled`), `object.rs`
  (`frame_pose`/`set_frame_pose`/`posed_drawings`→`(Pose,&FlipDrawing)`/`posed_bbox` usa
  `pose.apply` em vez de `p+off`/`translate_frame` usa `Pose::translate`/`duplicate_frame` carrega
  `src_pose`), `autokey.rs`/`tween.rs` (carregam a Pose).
- **Shell**: `flip_transform.rs` (`key_xform`/`art_to_world`/`world_to_art` recebem `Pose`),
  `flip_draw.rs` (`flip_active_pose` → `Pose`), `flip_gizmo_view.rs` (hit-test desce pelo
  **inverso** do afim — `key_xform(pose).inverse().apply(local)`; translação pura = `local - off`),
  `flip_pass.rs`/`flip_selection_overlay.rs` (render/realce usam `frame_pose`/`pose_at_cycled`),
  `project.rs` (**`PROJECT_SCHEMA` 13→14**).
- **Schema**: `FLIP_SCHEMA_VERSION` **5→6**, `PROJECT_SCHEMA` **13→14** (postcard posicional,
  8B→24B por chave posada).
- **Invariante preservada** (o par render/input inverso, o halo carrega a pose, seed=sample):
  gates `the_render_and_the_input_are_exact_inverses`, `the_move_funnel_must_be_pose_free_or_the_drag_trembles`,
  `the_halo_carries_the_key_pose` + 3 novos em `pose.rs`.

---

## 3. W7.5 Fase 2 — o GIZMO da pose no modo Edit (TODO — o que falta)

**Decisão de UX do Enio (2026-07-15):** *"gizmo no modo Edit"*. Quando o quadro ativo é uma
**instância**, aparece um gizmo (handles de sprite) enquadrando a arte posada dela; arrastar corpo
= mover, alças = girar/escalar — tudo escrevendo a **pose da chave**. O gizmo do OBJETO (modo
Select) fica **intocado**. (As outras opções — retarget do Select, teclas modificadoras — foram
descartadas.)

### 3.1 O que a exploração já estabeleceu (não re-descubra)

- O gizmo do objeto é publicado como `GizmoView` em `render_loop/snapshots.rs` (a closure
  `build_gizmo_view`, ~linha 265; chama `crate::flip_gizmo_view::view(...)` p/ objeto Flip, gated
  por `flip_gizmo_on`). O `flip_gizmo_view::view` (já existe) monta bbox/pivô/rotação a partir do
  `Transform` do objeto.
- O DRAG do gizmo é aplicado em **`shells/desktop/src/input_dispatch/gizmo_drag.rs`**
  (`advance_gizmo_drag`, ~600 linhas): ele escreve o `Transform` da entidade em VÁRIOS pontos
  (`get_mut::<Transform>` em ~146/539/553/578, por drag-kind e por membro de grupo).
- **A matemática do delta NÃO é um helper reusável** — está embutida no `advance_gizmo_drag`. Não
  existe um `GizmoDragState::result() -> TransformSnapshot` pronto. Peças puras que EXISTEM em
  `ph2d-editor-core/src/gizmo/`: `GizmoView` (paint.rs), `paint_sprite_gizmo` (desenha + registra
  handles no hit index), `gizmo_kind_for_id`/`is_gizmo_handle_id` (hit.rs), `GizmoDragState` +
  `advance_cursor` (drag.rs), `TransformSnapshot` + `compose_snapshot`/`world_delta_to_local`
  (transform.rs), `GizmoCamera` (camera.rs).
- O frame-pose MOVE de hoje mora no **modo Edit**, arrasto de canvas
  (`flip_edit_gesture.rs::flip_edit_canvas_move` → `EditGesture::Move` → `move_drawing` →
  `translate_frame`), com **funil POSE-FREE** (`flip_active_world_to_object`) — porque mover a
  instância escreve a pose e o funil pose-aware realimentaria (tremor, gate
  `the_move_funnel_must_be_pose_free`). O rotate/escala tem de morar no MESMO modo (Edit).

### 3.2 Os DOIS pontos de integração

**(A) Publicar a `GizmoView` da POSE no modo Edit.** Uma nova função em `flip_gizmo_view.rs` (ex.
`fn pose_view(...)`) que enquadra a **arte posada da chave ativa** (a bbox local dos traços da
chave passada pela `Pose`), com **pivô no centro dessa bbox** e **rotação extraída do afim da pose**
(`atan2(pose.coeffs()[1], pose.coeffs()[0])`). Pintá-la (via o mesmo caminho de `paint_sprite_gizmo`
que o objeto usa) **só** quando: tool Flip ativa · modo **Edit** · o quadro ativo é **instância**
(`FlipDrawing::is_instanced`). Isso registra os handles no hit index.

- **Cuidado (seed=sample):** a caixa e o pivô têm de sair da MESMA pose que o render dobra
  (`pose_at_cycled` no quadro atual), senão o gizmo pousa longe da arte — é o mesmo bug do halo
  (`the_halo_carries_the_key_pose`, smoke W7.2).
- A rotação/escala da bbox p/ o `GizmoView`: espelhe `flip_gizmo_view::view` (que já faz
  `centro = pivot + R·(anchor⊙scale)`), mas com a **pose da chave** no lugar do `Transform`.

**(B) Retargetar o apply para escrever a POSE, não o `Transform`.** Duas sub-opções — escolha a de
menor risco depois de ler `advance_gizmo_drag`:

- **Opção B1 (recomendada p/ isolamento): um drag DEDICADO.** Um `GizmoDragState` próprio guardado
  no `App` (ex. `flip_pose_gizmo_drag: Option<GizmoDragState>`), iniciado no pointer-down do Edit
  quando um handle foi hit (`gizmo_kind_for_id`), e um `advance` próprio (espelho enxuto do
  `advance_gizmo_drag`, SÓ os kinds translate/rotate/scale-corner/scale-edge, **sem** grupos/snap/
  pivô-tool) que compõe o delta na pose via `object.set_frame_pose(lid, key, delta ∘ pose_atual)`.
  A conta do delta você reusa de `advance_gizmo_drag` (translate = delta de mundo → local; rotate =
  giro em torno do pivô; scale = razão de distâncias ao pivô), montando um `Xform` de delta e
  compondo com `key_xform(pose_atual)`. **Não toca o gizmo do objeto** → zero risco de regressão em
  sprite/vetor. Custo: replicar a math de 3-4 kinds (não os 600 linhas — só o núcleo).
- **Opção B2: um ramo no `advance_gizmo_drag`.** Quando o alvo é objeto Flip + Edit + instância,
  compor o delta na pose em vez de escrever o `Transform`. Mais DRY, mas mexe no dispatch que
  sprites/vetores usam — **teste os dois (sprite e vetor) não regridem** se for por aqui.

### 3.3 Peças de modelo que a Fase 2 vai querer (ph2d-flip)

- `object.set_frame_pose(lid, key, pose)` — **JÁ EXISTE** (Fase 1). É o choke point de escrita.
- Um `Pose::compose` (afim·afim) ou faça a composição no shell via `Xform::then` e
  `Pose::from_coeffs(xform.0.map(|c| c as f32))`. **Recomendo compor no shell** (o `Xform` já tem
  `then`/`inverse`), mantendo ph2d-flip sem álgebra de matriz.
- O **pivô** do gizmo (centro da bbox posada) em espaço de MUNDO: `art_to_world(objeto, pose)`
  aplicado ao centro da bbox local. O delta de rotate/scale é em torno DESSE ponto.

### 3.4 Gates que a Fase 2 precisa (mutação provada, seed=sample)

- **A caixa do gizmo pousa na arte posada** (pura, espelho do `the_halo_carries_the_key_pose`):
  a `pose_view` com uma pose girada/escalada põe o pivô no centro da arte COMO ELA APARECE.
  Mutação: usar a bbox de geometria crua (sem pose) → pivô longe.
- **O drag compõe na pose, não no Transform** (o objeto não se move; só a chave ativa): rotacionar
  a instância gira o afim da chave e deixa as OUTRAS chaves (e o `Transform`) intactas.
- **Rotate/scale só na instância** (exclusiva não abre o gizmo de pose): numa chave de arte própria
  o Edit segue no arrasto-geometria de hoje (`translate_selection`).
- **Pose neutra = caminho de sempre**: sem instância movida, nada de gizmo de pose no Edit.
- O **par render/input inverso** (`the_render_and_the_input_are_exact_inverses`) e o **funil
  pose-free do move** (`the_move_funnel_must_be_pose_free`) **continuam verdes** — a Fase 2 não
  pode quebrá-los.

### 3.5 Exemplo pronto pro smoke (feedback do Enio: sempre entregue um)

Uma cena `PH2D_FLIP_POSE_SMOKE=1` que: cria 1 objeto Flip, desenha 1 traço na chave 0, faz
**Instance** (chave 1 compartilha a arte), move a instância um tanto (pra ela ter pose ≠ identidade),
seleciona o quadro da instância e entra no modo **Edit** — já com o gizmo da pose visível. O Enio
gira/escala e confere que a OUTRA instância e o objeto não se mexem.

---

## 4. Fila aberta (herdada — detalhe em `HANDOFF_flip_impl.md` §Aberto)

- **W7.5 Fase 2** (acima) — a próxima.
- **Seleção no domínio POINT** (hoje por TRAÇO): mover uma âncora só, máscara fina pro Sculpt.
- **W6 (timeline global): ADIADA** (a timeline principal ainda está em dev; o playhead do Flip já é
  o global, então a integração não terá relógio a reconciliar).
- **Refinos não-bloqueantes**: reorder de camada por drag, duplicar/agrupar camada, raio dedicado
  da borracha + preview, curva de pressão editável, round caps/joins, cache de tesselação com LRU.
- **`pack_perf.rs` — defeito latente** (flag da linha/Vector): o 2º assert (`ms < 30.0`) é uma mina
  no CI; o 1º teto já foi tornado por-perfil em `ace54f41`, o 2º ainda não.

---

## 5. Notas de INTEGRAÇÃO (pro agente integrador do Enio)

- **A linha tocou `ph2d-flip`** (crate de modelo — foundational-ish) com **mudança de schema**
  (`FLIP_SCHEMA` 5→6) e o **`PROJECT_SCHEMA` 13→14** no shell. Se outra linha bumpou o
  `PROJECT_SCHEMA` em paralelo, os números que **SOMAM** têm de ser reconciliados (não escolha um
  lado — `feedback_numbers_that_sum_across_lines_count_dont_pick`).
- **Colisão de mesmo-símbolo provável** em `flip_transform.rs`/`flip_gizmo_view.rs`/`object.rs` se
  outra linha mexeu na pose — resolva pelos ESTÁGIOS do índice, não pelos marcadores
  (`feedback_resolve_conflicts_from_index_stages_not_markers`), e rode `check --workspace` (merge
  limpo pode estar semanticamente quebrado).
- **Docs de planejamento** (`docs/Flip/`, `docs/architecture/decisions/0114-*.md`,
  `project-memory/project_flip_module_grease_pencil_2d.md`) seguem **untracked na árvore primária** —
  NÃO commitados nesta linha (senão o `merge --ff-only` quebra com "untracked working tree files
  would be overwritten"). O Enio comita ao `main` por fora.
- Rode o ship completo no fechamento (`scripts/ship.sh`) — `nextest-impacted` teve false-green em
  RAM baixa; o replay-hash muda porque o postcard mudou de forma (re-lock esperado).

---

## 6. Regras-mãe desta linha (não quebre)

- **Seed = sample**: quem PINTA e quem ESCREVE a arte de um quadro derivam a transform da MESMA
  função (`art_to_world`/`world_to_art`, o par inverso). Já divergiu 3× (o balde, BUGS #11/#14/#16).
- **O funil do MOVE é pose-free** (`flip_active_world_to_object`) — o pose-aware realimenta e treme.
- **`verde-de-compilação é velocidade; no audit vale ZERO`** — todo gate com mutação provada; desfaz
  mutação com `cp` do backup, NUNCA `git checkout`.
- **Exemplo pronto pro smoke** em toda feature nova (Enio).
