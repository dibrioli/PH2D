# Handoff — linha `line/FLIP`, continuação (2026-07-17) · **COMECE AQUI**

> **Para o próximo agente-de-linha do Flip** (o 4º meio do PH2D: animação quadro-a-quadro,
> fork 2D clean-room do Grease Pencil — [ADR-0114](architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md)).
> **Regime:** Modo L (workstation), worktree `Worktrees/line-FLIP`, branch `line/FLIP`.
> **Você NÃO integra nem pusha** (§0.7 do CLAUDE.md) — fecha o bloco, escreve o handoff,
> e o Enio ordena a integração via agente integrador dedicado.
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 →
> [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (inteira, e
> releia a cada passo) → **este arquivo** → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md) (as sagas)
> → o handoff anterior [`…2026-07-16`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-16.md) (o mapa do
> §4.A e das regras do módulo — este é o delta).
>
> **Sua tarefa: o §4.C** (§4 abaixo). São refinos independentes; escolha e feche um por vez.

---

## 1. Estado da linha — §4.B INTEGROU; **§4.C.1 (halo por-peça + hover) FECHOU, pendente smoke**

**§4.C.1 — o PEDAÇO é a unidade visual do modo Segment** (`a5738e98`, sobre a base
integrada, pendente smoke do Enio). Duas coisas, um primitivo:
- **Halo por-peça** (correção de um gap do §4.B): o overlay caía no branch de traço e
  acendia a FORMA INTEIRA ao selecionar um pedaço; agora `piece_halo_path` desenha só os
  segmentos com os dois extremos acesos — o pedaço, costura inclusa.
- **Hover** (o refino nº 1 do §4.C): `flip_segment_hover_refresh` computa o pedaço sob o
  cursor (mesma cadeia do pen-down) e o overlay o desenha em âmbar FRACO. Custo MEDIDO:
  **122 µs/frame** @2400 seg (0,7 %), só com cursor em movimento, nunca em gesto — sem cache.
- Gates: 3 no overlay + 3 de guarda (isolados via `flip_segment_hover_at`, não
  `flip_segment_hover` — sem gfx o pick é None e um gate sobre o hover ficaria verde COM a
  mutação; a armadilha [[feedback_a_green_gate_may_be_green_by_accident]] pega ao vivo).
- **Smoke:** `PH2D_FLIP_SEGMENT_SMOKE=1` (passe o mouse → âmbar fraco segue; clique → sólido,
  só o pedaço). O resto do §4.C segue aberto (§4 abaixo).

**Base:** §4.B (Segment mode) está na `main` (`segment.rs` = `8775a027`); a branch foi
fast-forwardada para a main integrada `cdc3acc1`. **§4.C.1 está À FRENTE da main** (não
integrado — fecha, handoff, PARA).

**Integrado desde a última rodada:** §4.B (Segment mode) · §4.A (gizmo da seleção) · W8
(domínio Point) · W7.5 (pose afim + gizmo da pose) · W7.4/W7.3/W7.2. Todos com smoke OK.

**Schema na base integrada:** `FLIP_SCHEMA_VERSION` **7** · `PROJECT_SCHEMA` **15** · pin
`(15, 7, 8)` em `shells/desktop/src/project_tests.rs`. Se a sua rodada bumpar um, bumpe os que
SOMAM ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]) — e conte o `PROJECT_SCHEMA`
contra o valor da main **no dia**, não contra 15.

**LOC a vigiar:** `flip_select.rs` a **568/600** (o mais apertado do módulo) — campo novo ali
→ orce o split em módulo irmão (`flip_select_pick.rs`/`flip_select_points.rs`/
`flip_select_segment.rs` já são os irmãos).

### O smoke do §4.B (já aprovado — reproduza se precisar do contexto do modo)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && \
  PH2D_FLIP_SEGMENT_SMOKE=1 ./target/release/ph2d-host-desktop
```

A cena abre no modo **Edit**, domínio **Segment** armado, com **quatro alvos**:

1. **O X** (cima-esq): clicar num braço acende SÓ aquele braço, do cruzamento à ponta.
2. **O triângulo** (cima-dir): nada o cruza ⇒ clicar em qualquer aresta acende a forma
   INTEIRA (o *fallback*; é o caso comum do balde).
3. **O quadrado** (baixo-esq): a linha vermelha que o corta vive em **OUTRA CAMADA** e mesmo
   assim corta (o corte é do QUADRO). E o pedaço da esquerda **ENROLA na costura**: clicar
   na aresta esquerda acende a quina de baixo E a de cima.
4. **A curva** (baixo-dir): densa, cortada 2× ⇒ três pedaços; o do meio acende só o meio.

Conferir ainda: arrastar um pedaço o **move**; **Shift+clique** soma; a **caixa de seleção**
acende o pedaço INTEIRO que tocou; **Point↔Segment** preserva a seleção, **Stroke**
promove/limpa.

---

## 2. As regras do módulo que NÃO se re-derivam erradas (cada uma custou rodadas)

1. **O traço é a união global da polilinha** (BUGS #1).
2. **O balde ancora no EIXO da linha** (BUGS #14) — espessura absoluta em px de TELA.
3. **A cor entra POR BAIXO da linha** (BUGS #15).
4. **A forma pinta A SI MESMA** (BUGS #16/#17) — o preenchimento é o `fill` do PRÓPRIO traço.
5. **O autokey é por FERRAMENTA** — caneta cria chave em branco; borracha e escultura DUPLICAM.
6. **Há TRÊS relógios** (BUGS #7): `drawing_at` · `source_frame` · `authoring_frame`.
7. **A escultura move as REGIÕES e os buracos delas.**
8. **Seed = sample** — quem PINTA e quem ESCREVE derivam da MESMA função. Já divergiu 4×.
9. **Arte compartilhada (instância) NUNCA deforma por arrasto** (W7.2) — escreve a POSE.
10. **O funil do MOVE é POSE-FREE**; o DELTA desce à arte pela linear inversa da pose.
11. **Uma pergunta, UMA função.** *"Quais são os segmentos deste traço?"* tinha 4 donos e 3
    erravam (BUGS #18). Hoje: `FlipStroke::segments()` ·
    `flip_selection_gizmo::grabbable_selection_box` · **🆕 `flip_select_pick::hit_at`** (o
    §4.B precisava de *onde* no traço e **não** abriu um 2º hit-test: o `stroke_at` virou
    `hit_at(..).map(si)`). **Se você precisar da resposta, CHAME a função.**
12. **Arte exclusiva PODE ter pose ≠ identidade** — não assuma identidade.
13. **🆕 As camadas só se encontram em espaço de OBJETO.** A arte é local à pose da CHAVE
    (`pose_at_cycled`, o par exato do `drawing_at_cycled`). Qualquer conta que compare
    desenhos de camadas diferentes sobe até lá — e **para**: cruzamento e fração são
    invariantes afim, então subir até a TELA (como a referência sobe, porque as camadas dela
    são 3D) só adiciona arredondamento.

---

## 3. Os padrões prontos para REUSAR (não reinvente)

### 3.1 O domínio Point (`ph2d-flip/src/stroke.rs` + `flip_select_points.rs`, W8)
`point_sel` **privado**; choke points `set_point_selected` / `promote_points_to_stroke`.
Invariante-mãe: vazio = a seleção vive no Curve; não-vazio ⇒ `selected == any(point_sel)`.

### 3.2 `FlipStroke::segments()` — a porta dos segmentos (BUGS #18)
`(i, a, b)`, `i` = ponto de PARTIDA; **fechado inclui a COSTURA**, aberto nunca. É a MESMA
convenção do BVH da referência (um elemento por ponto = o segmento que ali começa) — foi o
que deixou o §4.B indexar os cortes direto por `i`, sem tabela de tradução.

### 3.3 O gizmo da seleção (`flip_selection_gizmo.rs`, §4.A)
Porta única: `grabbable_selection_box` (recusa: instância · sem extensão) + `padded_gizmo_box`
(folga DERIVADA do `ph2d_editor::HANDLE_SIZE_PX`). Bake = `pose⁻¹ ∘ new ∘ start⁻¹ ∘ pose`.

### 3.4 O pick (`flip_select_pick.rs`) — **mudou no §4.B**
`hit_at` → `Option<(si, Where)>`, com `Where::{Ink{i,t}, Whole}`. `stroke_at` é derivado.
**A tinta é testada ANTES do fill** (o `stroke_at` não sente — é um OU; o Segment sente).
`Where::Whole` = "não há aresta onde mirar": o miolo de um preenchimento, ou traço de 1 ponto.

### 3.5 🆕 O domínio Segment (`ph2d-flip/src/segment.rs` + `flip_select_segment.rs`, §4.B)
**Motor no MODELO** (puro, sem shell): `cuts()` → `piece_of_point()` → `probe_point()`.
A saída-mãe é o **vetor de DONOS** (`dono[p]` = id do pedaço) — leia o doc do módulo antes de
tocar; ele explica por que essa forma **apaga** o fallback e o wrap como casos especiais, e
por que a verruga do `clamp_range` do Blender não nos alcança.
**Shell:** `frame_cutters(obj, frame, active)` responde *quem corta quem*.
**Porta única do pedaço:** `FrameCutters::piece_map` — o pick, o marquee e o colapso saem
todos dela.

---

## 4. ► SUA TAREFA: §4.C — refinos não-bloqueantes

Qualquer um serve de tarefa curta entre smokes:

- ✅ **realce de HOVER no Segment** — FECHOU em §4.C.1 (`a5738e98`), junto com o halo
  por-peça da seleção. Pendente smoke.
- reorder de camada por drag · duplicar/agrupar camada · máscaras de camada na UI
- raio dedicado da borracha + preview · curva de pressão editável · round caps/bevel joins
- **write-back do painel** (espelhar o estilo da seleção no swatch — `Flip/08 §6`)
- cache de tesselação com LRU

---

## 5. A fila / o que o §4.B deixou aberto (com o porquê)

- ✅ **Hover no Segment — FECHOU (§4.C.1, `a5738e98`).** A previsão de custo do §4.B
  (211 µs) foi revista na prática: o caminho INTEIRO (`frame_cutters` + `hit_at` +
  `hover_piece`) mede **122 µs/frame** @2400 seg (0,7 %) e só dispara com o cursor em
  movimento — então **não** houve cache de conteúdo, só a guarda "cursor-movido". O primitivo
  de render (`piece_halo_path`) também curou um gap do §4.B: a seleção de um pedaço acendia o
  traço INTEIRO (caía no branch de traço); agora acende só o pedaço, costura inclusa.
- **Um corte por segmento de polilinha** (limitação HERDADA da referência, gateada em
  `only_the_nearest_cut_of_a_segment_is_kept`). Dois traços cruzando o MESMO segmento
  produzem UM corte. Invisível numa polilinha densa (a caneta); visível num retângulo de 4
  pontos cruzado 2× na mesma aresta. Se doer: guardar `Vec<f32>` por segmento em vez de
  `Option<f32>` — o vetor de donos já aguenta (é só mais um corte na ordem).
- **A folga do gizmo da POSE** (aberto desde `1b090473`) — o gizmo da pose não ganhou folga.
  Se o Enio reclamar, é a MESMA `padded_gizmo_box`; mas o gate
  `the_pose_gizmo_box_lands_on_the_posed_art` afirma `half == 60/45` **exato** e vira piso+teto.
- **O handle agarrado atrasa `(ratio−1)·pad`** do cursor no scale (cosmético, documentado no
  `flip_selection_gizmo.rs`).
- **§4.D — W6 (timeline global): ADIADA** por ordem do Enio até a timeline principal fechar.

---

## 6. Notas de INTEGRAÇÃO — ✅ **CONSUMIDAS (integrado 2026-07-17)**, mantidas como registro

> Estas notas foram para o agente integrador e **já foram aplicadas**. Ficam aqui só como
> histórico do que este delta tocou (os sítios foundational append-only, os contadores). Uma
> rodada nova NÃO age sobre esta seção — a base já é a main integrada.


- **`ph2d-editor-core` tocada append-only** (foundational) — **5 sítios** (4 antigos + 1 do §4.B):
  - variantes **`GizmoTarget::FlipPose`** (W7.5) e **`GizmoTarget::FlipSelection`** (§4.A) em
    `gizmo/drag.rs` (apendadas por último);
  - scramblers de id em `keyed_handle_id` (`gizmo/paint.rs`): `0x_C3A5_C85C_97CB_3127` (pose)
    e `0x_5F1E_C7A0_2B94_D6E3` (seleção);
  - campos **`GizmoStateGroup.pose_view`** / **`.selection_view`** (`screens/hero/state.rs`)
    + os braços de pintura keyed em `screens/hero/paint.rs`;
  - **`HANDLE_SIZE_PX` virou `pub`** (`gizmo/paint.rs`) + `pub use` em `gizmo/mod.rs` e `lib.rs`;
  - **🆕 `FLIP_EDIT_DOM_SEGMENT`** em `ids/chrome/flip.rs` (`hash_node_id("flip.edit.dom.segment")`,
    apendado ao lado dos irmãos `_STROKE`/`_POINT`). Se outra linha apendou id de chrome,
    o `node_id_collisions` é quem fala.
  Colisão de mesmo-símbolo → resolva pelos **ESTÁGIOS do índice**
  ([[feedback_resolve_conflicts_from_index_stages_not_markers]]) e rode `check --workspace`
  (merge limpo pode estar semanticamente quebrado).
- **`ph2d-flip` (modelo) tocada:** módulo NOVO `segment.rs` (+`segment_tests.rs`) e os
  `pub use segment::{Cutter, cuts, piece_of_point, probe_point}` no `lib.rs`. Antes:
  `FlipStroke::segments()` (novo no §4.A) · `broadcast_selection_to_points` REMOVIDO · o par
  `selection_to_{point,stroke}_domain` renomeado para `enter_{point,stroke}_domain`.
  **Nada disso bumpa schema.**
- **`ph2d-tool-flip`:** `EditDomain::Segment` **apendado por último** (o enum não é
  serializado — é estado de tool) + `EditDomain::ALL` (novo; o seam test conta contra ele).
- **`.typos.toml`:** +2 palavras pt-BR (`acender`, `Repare`) na seção do Flip. **Chave
  duplicada mata o gate no parse** ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]])
  — se outra linha adicionou as mesmas, funda sem duplicar.
- **Shell — arquivos novos do §4.B:** `flip_select_segment.rs` (+`_tests`, 10 gates) ·
  `flip_segment_smoke.rs`. Antes: `flip_selection_gizmo.rs` (+`_tests`) ·
  `flip_selection_smoke.rs` · `flip_select_pick.rs` · `flip_pose_gizmo.rs` (+`_tests`) ·
  `flip_pose_smoke.rs` · `flip_edit_smoke.rs`.
- **Schema:** `FLIP` **7** / `PROJECT` **15**, pin `(15, 7, 8)`. As waves ANTERIORES bumparam
  (5→7 / 13→15); §4.A e §4.B não. Reconcilie o pin JUNTO com os contadores se outra linha bumpou.
- **Docs:** `docs/Flip/` e os `HANDOFF_line_FLIP_*` **são tracked na branch** e NÃO existem
  untracked na árvore primária — o `merge --ff-only` não quebra por eles.
- Rode o **ship COMPLETO** no fechamento (`scripts/ship.sh`) — `nextest-impacted` teve
  false-green em RAM baixa.

---

## 7. Comandos

**Gate batched (1× no fechamento do bloco, NUNCA por task):**
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

**Arch-gates que VÃO te pegar:** LOC **700**/crate e **600**/shell — **split em módulo irmão,
nunca allowlist**, e rode `fmt` ANTES de medir
([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]) · `no_magic_numeric` /
`arch_safe_clamp_only` (`// LITERAL-PX-OK` **com razão**; melhor: **derive** a constante) ·
`architecture_panel_wiring_parity` · `a_schema_bump_anywhere_must_bump_the_project_schema` ·
`node_id_collisions` · `file_loc_caps`.

**cwd:** trabalhe SEMPRE dentro do worktree — o mesmo path relativo existe na raiz do repo, e
editar `crates/...` na raiz é editar a árvore ERRADA. Mutação sempre por caminho **ABSOLUTO**
([[feedback_sed_relative_path_hits_primary_cwd]]). Desfaça mutação com **`cp` do backup**,
NUNCA `git checkout` ([[feedback_mutation_undo_with_cp_never_git_checkout]]).

**Smokes prontos:** `PH2D_FLIP_DEMO=1` (render/composição) · `PH2D_FLIP_POSE_SMOKE=1` (gizmo
da pose) · `PH2D_FLIP_EDIT_SMOKE=1` (domínio Point) · `PH2D_FLIP_XFORM_SMOKE=1` (gizmo da
seleção) · **`PH2D_FLIP_SEGMENT_SMOKE=1`** (§4.B — X, triângulo intacto, quadrado cortado por
outra camada, curva).
Diagnóstico: `PH2D_FLIP_FILL_DEBUG=1` (balde) · `PH2D_FLIP_SELECT_DEBUG=1` (Edit).

**Referência do Blender** (GPL — **comportamento, nunca código**):
`~/Downloads/blender-5.2-grease-pencil-ref/`. Para o §4.B os arquivos foram
`grease_pencil_select.cc` (`foreach_curve_segment`, `apply_mask_as_segment_selection`) e
`grease_pencil_geom.cc` (`find_curve_segments`, `find_curve_intersections`) — **não** o
`grease_pencil_segments_geom.cc`, que é o operador de **trim** (é dele que vêm os "paddings
load-bearing" que o handoff anterior mencionava; eles são do trim, não da seleção — o BVH da
seleção é construído com `epsilon = 0.0`).
Docs do módulo: [`docs/Flip/`](Flip/00_README.md).

---

**Você fecha o bloco, escreve o handoff de integração, e PARA. Não integra. Não pusha.**
