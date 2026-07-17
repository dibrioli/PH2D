# Handoff — linha `line/FLIP`, continuação (2026-07-16) · **COMECE AQUI**

> **Para o próximo agente-de-linha do Flip** (o 4º meio do PH2D: animação quadro-a-quadro,
> fork 2D clean-room do Grease Pencil — [ADR-0114](architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md)).
> **Regime:** Modo L (workstation), worktree `Worktrees/line-FLIP`, branch `line/FLIP`.
> **Você NÃO integra nem pusha** (§0.7 do CLAUDE.md) — fecha o bloco, escreve o handoff,
> e o Enio ordena a integração via agente integrador dedicado.
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 →
> [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (inteira, e
> releia a cada passo) → **este arquivo** → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md) (as sagas;
> #18 é da sessão passada) → o handoff anterior
> [`HANDOFF_line_FLIP_CONTINUACAO_2026-07-15c.md`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-15c.md)
> (o detalhe do §4.A e dos 6 achados do smoke — este resume) e o
> [`…15b`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-15b.md) (o mapa do W7.5/W8).
>
> **Sua tarefa: o §4.B — Segment mode** (§4 abaixo tem o mapa completo). É auto-contido, não
> toca o gizmo, e consome a `point_sel` que o W8 já deixou pronta.

---

## 1. Estado da linha — **o §4.A FECHOU com smoke OK** (32 commits, nada pushado)

**Enio, 2026-07-16: *"perfeito!"*** — o gizmo da seleção passou o smoke depois de 6 achados
consecutivos. Nada da linha está no `main`.

| commit | o quê | smoke |
|---|---|---|
| `1b090473` | a caixa do gizmo tem **FOLGA** (handles fora das âncoras, nunca sobrepostos) | **OK** |
| `33d7784d` | **revert:** 2+ pontos **precisam** de gizmo (regressão do agente anterior) | **OK** |
| `994ce21c` | `Select: Point` começa **DESSELECIONADO** (o broadcast do §11 saiu) | **OK** |
| `017b8f00` | a **ÁREA** do gizmo agarra a seleção · ponto único não abre gizmo | **OK** |
| `b793b47c` | a **COSTURA** do traço fechado é clicável (BUGS #18) | **OK** |
| `1b51f59b` | **§4.A**: o gizmo da SELEÇÃO no modo Edit (rotate/escala assado nos pontos) | **OK** |

Abaixo disso, tudo já tinha smoke OK (ver 15b): **W8** (domínio Point), **W7.5** (gizmo da
pose), **W7.5-F1** (pose afim), W7.4/W7.3/W7.2.

**Schema:** `FLIP_SCHEMA_VERSION` **7** · `PROJECT_SCHEMA` **15** · pin da tripla `(15, 7, 8)`
em `shells/desktop/src/project_tests.rs`. **O §4.A inteiro não bumpou nada** (o
`FlipSelectionDrag` é runtime; `GizmoStateGroup.selection_view` também). Se você bumpar um,
bumpe os que SOMAM ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

**Gates no fechamento:** 64 suítes verdes (`ph2d-flip{,-fill,-render,-reshape}` ·
`ph2d-tool-flip` · `ph2d-panel-flip{,-frames}` · `ph2d-ui-testkit` · `ph2d-editor-core` ·
`ph2d-host-desktop`) · clippy `--all-targets` limpo · GPU `gpu_render`(8)/`gpu_fill_fit`(17)
`--ignored` verdes · typos · `no_magic_numeric` · LOC caps · release builda.

---

## 2. As regras do módulo que NÃO se re-derivam erradas (cada uma custou rodadas)

1. **O traço é a união global da polilinha** (BUGS #1). Depth first-wins ⇒ quads sobrepostos
   computam a MESMA máscara.
2. **O balde ancora no EIXO da linha** (BUGS #14) — espessura absoluta em px de TELA, fill
   assado em DOC.
3. **A cor entra POR BAIXO da linha** (BUGS #15) — o contorno de um fill é rasterizado na cor
   do fill com a espessura da linha (dilatação).
4. **A forma pinta A SI MESMA** (BUGS #16/#17) — o preenchimento é o `fill` do PRÓPRIO traço.
5. **O autokey é por FERRAMENTA** — caneta cria chave em branco; borracha e escultura DUPLICAM.
6. **Há TRÊS relógios** (BUGS #7): `drawing_at` · `source_frame` · `authoring_frame`.
7. **A escultura move as REGIÕES e os buracos delas** — senão a cor fica para trás.
8. **Seed = sample** — quem PINTA e quem ESCREVE a arte derivam da MESMA função
   (`art_to_world`/`world_to_art`, o par inverso). Já divergiu 4×. Gate:
   `the_render_and_the_input_are_exact_inverses`.
9. **Arte compartilhada (instância) NUNCA deforma por arrasto** (W7.2) — mover/esculpir
   escreve a **pose da chave**. Quem quer divergir a arte **Unlink**a. Corolário: o gizmo da
   POSE só existe em instância; o da SELEÇÃO, só em arte exclusiva.
10. **O funil do MOVE é POSE-FREE** (`flip_active_world_to_object`) — o pose-aware realimenta
    e treme. Mas o DELTA desce à arte pela **parte linear inversa da pose**
    (`flip_transform::object_delta_to_art`) — senão sob pose girada a geometria anda torto.
11. **🆕 Uma pergunta, UMA função.** Foi a lição das 3 sessões: *"quais são os segmentos deste
    traço?"* tinha 4 donos e 3 erravam (BUGS #18); *"onde a seleção é agarrável?"* tinha 2 e
    divergiam. Hoje são `FlipStroke::segments()` e
    `flip_selection_gizmo::grabbable_selection_box`/`padded_gizmo_box`. **Se você precisar da
    resposta, CHAME a função — não re-derive.**
12. **🆕 Arte exclusiva PODE ter pose ≠ identidade** — o `make_single_user`/Unlink preserva a
    pose. Não assuma identidade em arte exclusiva.

---

## 3. Os padrões prontos para REUSAR (não reinvente)

### 3.1 O domínio Point (`crates/ph2d-flip/src/stroke.rs` + `flip_select_points.rs`, W8)
`FlipStroke.point_sel` **privado**; choke points: `set_point_selected` /
`promote_points_to_stroke`. **Invariante-mãe:** vazio = a seleção vive no Curve; não-vazio ⇒
`selected == any(point_sel)` (o Curve é a projeção `any()` permanente — é o que mantém painel,
halo e máscara-grossa certos sem tocá-los). Helpers: `selected_point_indices`,
`all_points_selected`, `translate_selected_points`, `remove_selected_points`.
**A troca de domínio:** `enter_point_domain` (**limpa** — diverge do GP de propósito) /
`enter_stroke_domain` (promove por `any()`).

### 3.2 `FlipStroke::segments()` — a porta dos segmentos (BUGS #18)
`(i, a, b)` com `i` = índice do ponto de PARTIDA; **fechado inclui a COSTURA** (último→
primeiro), aberto nunca. Convenção espelhada do render (`ph2d_flip_render::pack::stroke_segments`).
**O §4.B vive disto** — é a lista de segmentos que o raycast vai varrer.

### 3.3 O gizmo da seleção (`shells/desktop/src/flip_selection_gizmo.rs`, §4.A)
Espelho do gizmo da POSE. A **porta única** é a `grabbable_selection_box` (recusa: instância ·
sem extensão) + `padded_gizmo_box` (a folga, DERIVADA do `ph2d_editor::HANDLE_SIZE_PX`). A
view **desenha** e o down do Edit **testa o interior** pela mesma função. O bake é
`pose⁻¹ ∘ new ∘ start⁻¹ ∘ pose` (`art_bake_xform`), snapshot no Down e recomputa do snapshot.

### 3.4 O pick (`flip_select_pick.rs` traço · `flip_select_points.rs` ponto)
`stroke_at` (re-exportado por `flip_select` — uma porta) e `point_at`. Os dois convertem px de
TELA → arte por `px_to_world * w2l.mean_scale()` — **é a convenção do módulo para tudo que é
chrome/folga** (o raio de pick, a folga do gizmo).

---

## 4. ► SUA TAREFA: §4.B — **Segment mode** (o 3º domínio do GP)

**Auto-contido: não toca o gizmo nem a pose.** A receita completa está em
[`Flip/02_referencia_algoritmos_blender_5.2.md` §11](Flip/02_referencia_algoritmos_blender_5.2.md)
(procure "Segment mode"). O resumo dela:

> **Segment mode = corte por interseção VISUAL:** raycast de cada segmento contra um **BVH 2D
> do frame** (ignorando **3 vizinhos**); **hit = início de segmento**; **cíclica sem corte tem
> ZERO segmentos → fallback "1 ponto seleciona a curva toda"**; o último segmento de uma
> cíclica **enrola em DOIS ranges**.

**O que o §11 já decidiu por você:** o modo é **Point + pós-processo** (o domínio do dado
continua sendo `point_sel` — o segment mode seleciona um **RANGE de pontos**, não inventa um
3º vetor). E: *"o segment mode é 100% screen-space — port natural"*.

**O caminho no nosso código:**
1. **A 3ª pill** ao lado de Stroke|Point: `EditDomain::Segment` (variante nova em
   `ph2d-tool-flip::params::EditDomain`, apendada) + o id `FLIP_EDIT_DOM_SEGMENT` em
   `ph2d-editor-core/src/ids/chrome/flip.rs` (append-only, ao lado de
   `FLIP_EDIT_DOM_STROKE`/`FLIP_EDIT_DOM_POINT`) + o arm no painel. **Fie as 7 pontas juntas**
   (DIRETIVA §2) — o pill pintado e não-armado é o bug nº 1 do projeto.
2. **Os cortes** — onde o `segments()` (§3.2) entra: para cada segmento do frame, ache as
   interseções contra os outros. O `02 §11` manda **BVH 2D**; o `segments_geom.cc` do Blender
   admite O(N²) como TODO. **Comece medindo** ([[feedback_measure_perf_symptom_scale]]): um
   frame de line-art tem centenas de segmentos, e O(N²) sobre 500 = 250k testes/clique — pode
   ser aceitável para um CLIQUE (não é por-frame). **Se for, diga isso no doc** em vez de
   construir um BVH que ninguém pediu.
3. **A conversão hit → range de pontos** e o `set_point_selected` sobre o range. O domínio de
   dado é o do W8; o toggle Segment é uma **política de pick**, não um vetor novo.
4. **A troca de domínio** (`flip_edit_domain_refresh`): decida o que Segment→Stroke/Point faz e
   **escreva o porquê** — a assimetria de hoje (entrar no Point limpa, voltar ao Stroke
   promove) é deliberada e documentada; a sua tem de ser também.

**As armadilhas que o §11 nomeia (não as re-descubra):**
- **Cíclica sem corte = ZERO segmentos.** Sem o fallback, clicar numa forma fechada intacta
  não seleciona NADA. É o caso comum do balde.
- **O último segmento de uma cíclica enrola em DOIS ranges** (`[i..n)` + `[0..j)`).
- **Ignorar 3 vizinhos** no raycast — senão todo segmento intersecta o próprio vizinho.
- **Paddings load-bearing** no Blender (bbox +2px, ±1px na aresta, snap de fator com eps 1e-4):
  *"a religação depende de igualdade float EXATA que só funciona por causa do snap"*. Se você
  portar o trim, porte os paddings **ou** prove que o nosso caminho não precisa deles.

**Gates (mutação provada, DIRETIVA §3) — o mínimo:**
- um clique num segmento entre 2 cortes acende **só aquele range** (mutação: ignorar os cortes
  → acende o traço todo);
- **cíclica sem corte** → o fallback acende a curva inteira (mutação: tirar o fallback → não
  acende nada — o caso comum do balde);
- o segmento que **enrola** na cíclica acende os DOIS ranges;
- o raycast **ignora os 3 vizinhos** (mutação: não ignorar → todo segmento vira um corte).
- **Fixture com CURVA, não só polígono** — o BUGS #18 mostrou que fixture de polígono esconde
  bug de curva ([[reference_topic_fixture_discipline]]).

**Smoke:** entregue uma cena pronta (`PH2D_FLIP_SEGMENT_SMOKE=1`, espelho do
`flip_selection_smoke.rs`) — o Enio **não monta cena**
([[feedback_ready_to_smoke_example]]). Ela precisa de: duas linhas que se **cruzam** (os
cortes), uma **cíclica intacta** (o fallback) e uma cíclica **cortada** (o wrap).

---

## 5. A fila DEPOIS do §4.B

- **§4.C — refinos não-bloqueantes** (qualquer um serve de tarefa curta entre smokes):
  reorder de camada por drag · duplicar/agrupar camada · máscaras de camada na UI · raio
  dedicado da borracha + preview · curva de pressão editável · round caps/bevel joins ·
  **write-back do painel** (espelhar o estilo da seleção no swatch — `Flip/08 §6`) · cache de
  tesselação com LRU.
- **A folga do gizmo da POSE** (aberto, `1b090473`): o gizmo da pose **não** ganhou folga — ele
  enquadra o desenho INTEIRO, que na prática sempre tem extensão. Se o Enio reclamar de handle
  em cima da arte lá, é a MESMA `padded_gizmo_box`; mas o gate
  `the_pose_gizmo_box_lands_on_the_posed_art` afirma `half == 60/45` **exato** e vira
  piso+teto, como o da seleção virou.
- **O handle agarrado atrasa `(ratio−1)·pad`** do cursor no scale (`1b090473`, cosmético e
  conhecido). Conserto, se incomodar: passar a meia-extensão PADDED ao drag — e aí a ARTE é
  que deriva. O trade está documentado no `flip_selection_gizmo.rs`.
- **§4.D — W6 (timeline global): ADIADA** por ordem do Enio até a timeline principal fechar. O
  playhead do Flip JÁ é o global. Se ele reabrir, leia o handoff da linha `anim` antes.

---

## 6. Notas de INTEGRAÇÃO (pro agente integrador do Enio) — **acumuladas, nada foi integrado**

- **`ph2d-editor-core` tocada append-only** (foundational) — **4 sítios**:
  - variantes **`GizmoTarget::FlipPose`** (W7.5) e **`GizmoTarget::FlipSelection`** (§4.A) em
    `gizmo/drag.rs` (apendadas por último);
  - scramblers de id em `keyed_handle_id` (`gizmo/paint.rs`): `0x_C3A5_C85C_97CB_3127` (pose)
    e `0x_5F1E_C7A0_2B94_D6E3` (seleção);
  - campos **`GizmoStateGroup.pose_view`** e **`.selection_view`** (`screens/hero/state.rs`) +
    os braços de pintura keyed em `screens/hero/paint.rs`;
  - **`HANDLE_SIZE_PX` virou `pub`** (`gizmo/paint.rs`) + entrou nos `pub use` de
    `gizmo/mod.rs` e `lib.rs`.
  Colisão de mesmo-símbolo se outra linha mexeu no gizmo → resolva pelos **ESTÁGIOS do índice**
  ([[feedback_resolve_conflicts_from_index_stages_not_markers]]) e rode `check --workspace`
  (merge limpo pode estar semanticamente quebrado).
- **`ph2d-flip` (modelo) tocada:** `FlipStroke::segments()` (novo) · `broadcast_selection_to_points`
  **REMOVIDO** (ficou órfão quando `enter_point_domain` passou a limpar) · o par
  `selection_to_{point,stroke}_domain` **renomeado** para `enter_{point,stroke}_domain`.
  **Nada disso bumpa schema** (métodos, não layout).
- **Schema:** `FLIP` **7** / `PROJECT` **15**, pin `(15, 7, 8)`. As waves ANTERIORES bumparam
  (5→7 / 13→15); o §4.A não. Reconcilie o pin JUNTO com os contadores se outra linha bumpou.
- **Shell — arquivos novos:** `flip_selection_gizmo.rs` (+`_tests`, 10 gates) ·
  `flip_selection_smoke.rs` · `flip_select_pick.rs` (split do cap de LOC) ·
  `flip_pose_gizmo.rs` (+`_tests`) · `flip_pose_smoke.rs` · `flip_edit_smoke.rs`.
- **ids novos** em `ids/chrome/flip.rs`: `FLIP_EDIT_DOM_STROKE`/`FLIP_EDIT_DOM_POINT` (W8).
- **Docs:** `docs/Flip/` e os `HANDOFF_line_FLIP_*` **são tracked na branch** e NÃO existem
  untracked na árvore primária — o `merge --ff-only` não quebra por eles.
- Rode o **ship COMPLETO** no fechamento (`scripts/ship.sh`) — `nextest-impacted` teve
  false-green em RAM baixa; o replay-hash muda (o postcard mudou de forma nas waves
  anteriores) — re-lock esperado.

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
nunca allowlist**, e rode `fmt` ANTES de medir ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]];
o `flip_select.rs` já estourou uma vez e virou `flip_select_pick.rs`) · `no_magic_numeric` /
`arch_safe_clamp_only` (use `safe_clamp` ou `// CLAMP-OK`/`// LITERAL-PX-OK` **com razão**;
melhor ainda: **derive** a constante, como a folga do gizmo saiu do `HANDLE_SIZE_PX`) ·
`architecture_panel_wiring_parity` · `a_schema_bump_anywhere_must_bump_the_project_schema` ·
`node_id_collisions` · `file_loc_caps`.

**cwd:** trabalhe SEMPRE dentro do worktree — o mesmo path relativo existe na raiz do repo, e
editar `crates/...` na raiz é editar a árvore ERRADA. Mutação sempre por caminho **ABSOLUTO**
([[feedback_sed_relative_path_hits_primary_cwd]]). Desfaça mutação com **`cp` do backup**,
NUNCA `git checkout` ([[feedback_mutation_undo_with_cp_never_git_checkout]]).

**Smokes prontos:** `PH2D_FLIP_DEMO=1` (render/composição) · `PH2D_FLIP_POSE_SMOKE=1` (gizmo da
pose) · `PH2D_FLIP_EDIT_SMOKE=1` (domínio Point) · `PH2D_FLIP_XFORM_SMOKE=1` (gizmo da seleção
— retângulo VAZADO + triângulo; cobre folga, costura, área, 1-vs-2 pontos).
Diagnóstico: `PH2D_FLIP_FILL_DEBUG=1` (balde) · `PH2D_FLIP_SELECT_DEBUG=1` (Edit).

**Referência do Blender** (GPL — **comportamento, nunca código**):
`~/Downloads/blender-5.2-grease-pencil-ref/`. Docs do módulo: [`docs/Flip/`](Flip/00_README.md)
— **`02_referencia §11` é a receita do §4.B**.

---

**Você fecha o bloco, escreve o handoff de integração, e PARA. Não integra. Não pusha.**
