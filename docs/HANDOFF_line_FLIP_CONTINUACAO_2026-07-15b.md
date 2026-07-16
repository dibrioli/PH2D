# Handoff — linha `line/FLIP`, continuação (2026-07-15b) · **COMECE AQUI**

> **Para o próximo agente-de-linha do Flip** (o 4º meio do PH2D: animação quadro-a-quadro,
> fork 2D clean-room do Grease Pencil — [ADR-0114](architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md)).
> **Regime:** Modo L (workstation), worktree `Worktrees/line-FLIP`, branch `line/FLIP`.
> **Você NÃO integra nem pusha** (§0.7 do CLAUDE.md) — fecha o bloco, escreve o handoff,
> e o Enio ordena a integração via agente integrador dedicado.
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 →
> [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (inteira, e
> releia a cada passo) → este arquivo → o handoff anterior
> [`HANDOFF_line_FLIP_CONTINUACAO_2026-07-15.md`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-15.md)
> (o mapa detalhado do W7.5 e do W8, que este resume) → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md).

---

## 1. O que está FECHADO (tudo commitado, nada pushado)

`git log --oneline main..HEAD` — as 3 waves desta jornada, todas **com smoke do Enio OK**:

| commit | wave | o quê | smoke |
|---|---|---|---|
| `6d437b4f` | — | docs | — |
| `5a529e1d` | **W8 fix** | o dot não-selecionado **contrasta** com a cor da linha (luminância WCAG, Y≈0.179, por ponto, recomputado por frame) | **OK** |
| `816fff9c` | **W8** | seleção no domínio **POINT** (meia-traço + máscara fina do Sculpt) | **OK** |
| `a93d29a2` | **W7.5 F2** | o **gizmo da POSE** no modo Edit (rotate/escala da instância) | **OK** |
| `df809109` | **W7.5 F1** | a pose da chave virou **AFIM** (`Pose([f32;6])`) | fundação |

**Schema atual:** `FLIP_SCHEMA_VERSION` **7**, `PROJECT_SCHEMA` **15**, pin da tripla
`(15, 7, 8)` em `shells/desktop/src/project_tests.rs`. (Se você bumpar um, bumpe os que
SOMAM — `feedback_numbers_that_sum_across_lines_count_dont_pick`.)

**Estado dos gates:** verde em todas as crates do Flip + shell + editor-core; clippy limpo;
GPU `gpu_render`/`gpu_fill_fit` `--ignored` verdes; typos limpo; release builda.

---

## 2. As regras do módulo que NÃO se re-derivam erradas (cada uma custou rodadas)

1. **O traço é a união global da polilinha** (BUGS #1). Depth first-wins ⇒ quads sobrepostos
   computam a MESMA máscara.
2. **O balde ancora no EIXO da linha** (BUGS #14) — espessura absoluta em px de TELA, fill
   assado em DOC; qualquer âncora derivada da espessura descola no zoom.
3. **A cor entra POR BAIXO da linha** (BUGS #15) — o contorno de um fill é rasterizado na cor
   do fill com a espessura da linha (dilatação).
4. **A forma pinta A SI MESMA** (BUGS #16/#17) — o preenchimento é o `fill` do PRÓPRIO traço
   (triangulação dos pontos dele), como no GP.
5. **O autokey é por FERRAMENTA** — caneta cria chave em branco; borracha e escultura DUPLICAM.
6. **Há TRÊS relógios** (BUGS #7): `drawing_at` · `source_frame` · `authoring_frame`.
7. **A escultura move as REGIÕES e os buracos delas** — senão a cor fica para trás.
8. **Seed = sample** — quem PINTA e quem ESCREVE a arte de um quadro derivam a transform da
   MESMA função (`art_to_world`/`world_to_art`, o par inverso). Já divergiu 4× (o balde
   BUGS #11/#14/#16, e o halo do W7.2). O gate `the_render_and_the_input_are_exact_inverses`
   prende o par.
9. **Arte compartilhada (instância) NUNCA deforma por arrasto** (W7.2) — mover/esculpir uma
   instância escreve a **pose da chave**, não a geometria (que é do gêmeo também). Quem quer
   divergir a arte **Unlink**a (`make_single_user`). Corolário W8: mover PONTO de uma
   instância é **recusado com toast** — selecionar pode, deformar não.
10. **O funil do MOVE é POSE-FREE** (`flip_active_world_to_object`) — o pose-aware realimenta
    e o desenho treme (smoke W7.2). Mas o DELTA desce ao espaço da arte pela **parte linear
    inversa da pose** (`flip_transform::object_delta_to_art`, W8) — senão sob pose girada a
    geometria anda na direção errada. (Translação pura = identidade byte a byte.)

---

## 3. Os padrões que a jornada deixou prontos para REUSAR (não reinvente)

Estes três são a alavanca da próxima fase. Estude-os antes de escrever qualquer gizmo/gesto.

### 3.1 O gizmo reparametrizado (`shells/desktop/src/flip_pose_gizmo.rs`, W7.5 F2)
A decisão-mãe: **não reescrever a matemática do gizmo**. A pose é reparametrizada como um
**TRS ancorado no centro da arte** (`pose(p) = t_c + R·S·(p − c_local)`), o
`ph2d_editor::compute_gizmo_transform` canônico (modifiers, snap, contador de voltas do
rotate, re-ancoragem do canto oposto) roda byte a byte, e a volta é `trs_to_pose` (inverso
exato de `pose_trs`). O drag dedicado (`FlipPoseDrag`, campo `App.flip_pose_drag`) escreve
pela porta `set_frame_pose`. Foundational tocado append-only: `GizmoTarget::FlipPose` +
scrambler de id `0x_C3A5_C85C_97CB_3127` em `keyed_handle_id` + campo
`GizmoStateGroup.pose_view` + braço keyed em `screens/hero/paint.rs` (sem interior, sem pivot
dot). **Este é o template EXATO da próxima fase** (§4.A).

### 3.2 O domínio Point (`crates/ph2d-flip/src/stroke.rs` + `flip_select_points.rs`, W8)
`FlipStroke.point_sel` privado, choke points `set_point_selected`/`broadcast_selection_to_points`/
`promote_points_to_stroke`. **Invariante-mãe:** vazio = a seleção vive no Curve; não-vazio ⇒
`selected == any(point_sel)` (o Curve é a projeção `any()` permanente — é o que mantém painel,
halo e máscara-grossa certos sem tocá-los). Helpers prontos: `selected_point_indices`,
`all_points_selected`, `translate_selected_points`, `remove_selected_points`. Conversão de
domínio explícita (broadcast desce, `any()` sobe; half-selected só existe em Point).

### 3.3 A máscara fina do Sculpt (`crates/ph2d-flip-reshape/src/lib.rs`, W8)
Por **snapshot-e-restaura** no `Session::apply`: o pincel roda LIVRE e a máscara devolve o que
ele não podia tocar (os não-selecionados + buracos). Vale pros 8 pincéis sem mudar a
assinatura de nenhum. Se você mexer no Reshape, é aqui.

---

## 4. A FILA — a próxima fase e o resto (detalhe em `HANDOFF_flip_impl.md` §Aberto)

> **Ordem recomendada:** §4.A é o passo mais natural e o de maior alavanca (o template já
> existe). §4.B é o mais auto-contido. Faça UM bloco, feche com smoke, escreva o handoff, PARE.

### §4.A — **Transformar a SELEÇÃO com o gizmo** (a próxima; `docs/Flip/08 §6` a chama de "o próximo passo natural deste modo")

Hoje o Edit Mode **move** a seleção (traço no W6.1, ponto no W8), mas não **gira/escala**. O
caminho é o gizmo de sprite agindo sobre a SELEÇÃO — e o W7.5 já resolveu 90% dele.

**O que fazer (o template é o §3.1, ponto a ponto):**
1. **Um helper de bbox da seleção** (NÃO existe ainda — confirmei). No espaço da ARTE: a
   caixa envolvente dos pontos SELECIONADOS (domínio Curve = todos os pontos dos traços
   selecionados; domínio Point = só os `point_selected`). O centro dela é o pivô (`c`).
2. **Publicar uma `GizmoView`** enquadrando a seleção posada — espelho enxuto de
   `flip_pose_gizmo::pose_view`, gated por tool Flip + modo Edit + **há seleção**. Campo novo
   `GizmoStateGroup.selection_view` (append-only, como `pose_view` foi) + braço keyed em
   `screens/hero/paint.rs` com um `GizmoTarget::FlipSelection` NOVO (próximo scrambler livre;
   anote no handoff). **Com interior** desta vez? NÃO — o interior/translate já é o arrasto de
   canvas do W6.1/W8; só rotate/scale nos handles keyed.
3. **O drag dedicado** (campo `App.flip_selection_drag`, espelho de `flip_pose_drag`): no Down
   snapshota as **posições de início** de cada ponto selecionado (`Vec<(si, ring, pi, Vec2)>`,
   como o Grab congela). Cada Move recomputa `p' = c + R·S·(p₀ − c) + Δt` do snapshot (NUNCA
   compõe por-frame — o mesmo discipline do gizmo) via `compute_gizmo_transform`, e escreve as
   posições. Translate puro pode delegar aos helpers `translate_selection`/
   `translate_selected_points` que já existem; rotate/scale precisam do bake afim novo.
4. **Bake no espaço da ARTE:** o gizmo trabalha em MUNDO, a geometria vive em ART. Use o par
   `world_to_art`/`art_to_world` (o afim `objeto ∘ pose`) para descer os pontos, aplicar o TRS
   em torno de `c`, e é só. Os **buracos** andam quando a seleção pega o traço INTEIRO
   (`all_points_selected`), como o `translate_selected_points` já decide.

**Gotchas (herdados, não re-descubra):**
- **Instância = geometria compartilhada.** Transformar a seleção numa instância deformaria o
  gêmeo — **recuse com toast**, igual ao move de ponto do W8 (regra-mãe #9). Ou só ofereça o
  gizmo em arte exclusiva (a `pose_view` do W7.5 faz o inverso: só em instância).
- **Seed = sample** (#8): a caixa/pivô saem da MESMA cadeia que o render dobra. Gate espelho de
  `the_pose_gizmo_box_lands_on_the_posed_art`.
- **Snapshot no Down, recomputa do snapshot** — senão o rotate acumula erro e a seleção deriva.
- Gizmo só publica no modo Edit + com seleção (senão come o clique da seleção de traço).

**Gates (mutação provada, DIRETIVA §3):** a caixa pousa na seleção posada · rotate gira em
torno do centro da seleção deixando o resto do desenho parado · transform de instância recusa ·
translate puro reduz ao move de sempre. **Smoke pronto** (`PH2D_FLIP_XFORM_SMOKE=1`, espelho do
`flip_edit_smoke.rs`): cena com traços, seleção parcial, gizmo de transform já visível.

### §4.B — **Segment mode** (o 3º domínio do GP; `02_referencia §11` dá a receita completa)
Corte por interseção VISUAL: raycast de cada segmento contra um BVH 2D do frame (ignorando 3
vizinhos); hit = início de segmento. **Cíclica sem corte = ZERO segmentos → fallback "1 ponto
seleciona a curva toda"**; o último segmento de cíclica enrola em DOIS ranges. É 100%
screen-space (port natural), consome a MESMA `point_sel` do W8 (seleciona um RANGE de pontos),
e o toggle vira uma 3ª pill ao lado de Stroke|Point. Auto-contido: não toca o gizmo.

### §4.C — Refinos não-bloqueantes (qualquer um serve de tarefa curta entre smokes)
Reorder de camada por drag · duplicar/agrupar camada · máscaras de camada na UI · raio dedicado
da borracha + preview · curva de pressão editável · round caps/bevel joins · write-back do
painel (espelhar o estilo da seleção no swatch) · cache de tesselação com LRU.

### §4.D — **W6 (timeline global): ADIADA** por ordem do Enio até a timeline principal fechar.
O playhead do Flip JÁ é o global. Se o Enio reabrir, leia o handoff da linha `anim` antes (ela
trouxe seletor de clips + relógio único).

---

## 5. Notas de INTEGRAÇÃO (pro agente integrador do Enio)

- **Crate de modelo (`ph2d-flip`) tocada** com **schema** (`FLIP` 5→7) e o **`PROJECT_SCHEMA`
  13→15** no shell. Reconcilie o pin `(15, 7, 8)` de `project_tests.rs` JUNTO com os contadores
  se outra linha bumpou em paralelo.
- **`ph2d-editor-core` tocada append-only** (foundational): variantes novas no `GizmoTarget`
  (`FlipPose`; a §4.A adicionará `FlipSelection`) · scramblers de id em `keyed_handle_id` · campos
  novos no `GizmoStateGroup` (`pose_view`; §4.A: `selection_view`) · braços em
  `screens/hero/paint.rs`. Colisão de mesmo-símbolo se outra linha mexeu no gizmo → resolva
  pelos ESTÁGIOS do índice, não pelos marcadores, e rode `check --workspace`.
- **ids novos** em `ids/chrome/flip.rs`: `FLIP_EDIT_DOM_STROKE`/`FLIP_EDIT_DOM_POINT` (W8).
- **Docs de planejamento** (`docs/Flip/`, `docs/architecture/decisions/0114-*`,
  `project-memory/project_flip_module_grease_pencil_2d.md`) seguem **untracked na árvore
  primária** — NÃO commitados nesta linha (senão o `merge --ff-only` quebra com "untracked
  working tree files would be overwritten"). O Enio comita ao `main` por fora.
- Rode o ship COMPLETO no fechamento (`scripts/ship.sh`) — `nextest-impacted` teve false-green
  em RAM baixa; o replay-hash muda (o postcard mudou de forma) — re-lock esperado.

---

## 6. Comandos

**Gate batched (1× no fechamento do bloco):**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && \
cargo test -p ph2d-flip -p ph2d-flip-fill -p ph2d-flip-render -p ph2d-flip-reshape \
           -p ph2d-tool-flip -p ph2d-panel-flip -p ph2d-panel-flip-frames \
           -p ph2d-ui-testkit -p ph2d-editor-core -p ph2d-host-desktop --no-fail-fast && \
cargo test -p ph2d-flip-render --test gpu_render --test gpu_fill_fit -- --ignored && \
cargo clippy -p <suas-crates> --all-targets && \
rustup run 1.95 cargo fmt -p <suas-crates> && typos && \
cargo build --release -p ph2d-host-desktop
```
(Arch-gates que VÃO te pegar: LOC 700/crate e 600/shell — **split em módulo irmão, nunca
allowlist**, e rode `fmt` ANTES de medir · `no_magic_numeric` / `arch_safe_clamp_only` (use
`safe_clamp` ou `// CLAMP-OK`/`LITERAL-PX-OK` com razão) · `architecture_panel_wiring_parity` ·
`a_schema_bump_anywhere_must_bump_the_project_schema` · `node_id_collisions`.)

**cwd:** trabalhe SEMPRE dentro do worktree — o mesmo path relativo existe na raiz do repo, e
editar `crates/...` na raiz é editar a árvore ERRADA. O `cargo test` da raiz falha com "Not a
directory" quando o cwd escorrega — o sinal de que você saiu do worktree. Mutação sempre por
caminho ABSOLUTO. Desfaça mutação com **`cp` do backup**, NUNCA `git checkout`.

**Smokes já prontos** (o Enio não monta cena — `feedback_ready_to_smoke_example`):
`PH2D_FLIP_DEMO=1` (render/composição) · `PH2D_FLIP_POSE_SMOKE=1` (gizmo da pose, W7.5) ·
`PH2D_FLIP_EDIT_SMOKE=1` (domínio Point, W8). Diagnóstico do balde: `PH2D_FLIP_FILL_DEBUG=1`;
do Edit: `PH2D_FLIP_SELECT_DEBUG=1`.

**Referência do Blender** (GPL — **comportamento, nunca código**):
`~/Downloads/blender-5.2-grease-pencil-ref/`. Docs do módulo: [`docs/Flip/`](Flip/00_README.md)
(§11 do `02_referencia` = seleção/multiframe/cíclicas — a receita do §4.A e do §4.B).

---

**Você fecha o bloco, escreve o handoff de integração, e PARA. Não integra. Não pusha.**
