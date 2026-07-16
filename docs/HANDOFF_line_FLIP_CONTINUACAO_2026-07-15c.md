# Handoff — linha `line/FLIP`, continuação (2026-07-15c) · **COMECE AQUI**

> **Para o próximo agente-de-linha do Flip** (o 4º meio do PH2D: animação quadro-a-quadro,
> fork 2D clean-room do Grease Pencil — [ADR-0114](architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md)).
> **Regime:** Modo L (workstation), worktree `Worktrees/line-FLIP`, branch `line/FLIP`.
> **Você NÃO integra nem pusha** (§0.7 do CLAUDE.md) — fecha o bloco, escreve o handoff,
> e o Enio ordena a integração via agente integrador dedicado.
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 →
> [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (inteira, e
> releia a cada passo) → este arquivo → o handoff anterior
> [`HANDOFF_line_FLIP_CONTINUACAO_2026-07-15b.md`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-15b.md)
> (o mapa detalhado do W7.5/W8 e da fila, que este estende) → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md).

---

## 1. O que FECHOU nesta sessão (commitado, nada pushado)

| commit | wave | o quê | smoke |
|---|---|---|---|
| `994ce21c` | **§4.A fix** | **`Select: Point` começa DESSELECIONADO** (o broadcast do §11 saiu — diverge do GP de propósito) | **PENDENTE (re-rode o `XFORM_SMOKE`)** |
| `08ba6358` | **§4.A fix** | **`Select: Point` some com o gizmo NA HORA** — o gizmo é do domínio **Stroke** (ADR-0112 parity) | **PENDENTE (re-rode o `XFORM_SMOKE`)** |
| `017b8f00` | **§4.A fix** | a **ÁREA** do gizmo agarra a seleção · **ponto único não abre gizmo** — 2 achados do smoke | **PENDENTE (re-rode o `XFORM_SMOKE`)** |
| `b793b47c` | **BUGS #18** | a **COSTURA** do traço fechado agora é clicável (pick/marquee/hover) — achado do smoke do §4.A | **PENDENTE (re-rode o `XFORM_SMOKE`)** |
| `1b51f59b` | **§4.A** | o **gizmo da SELEÇÃO** no modo Edit (rotate/escala assado nos pontos de arte exclusiva) | **PENDENTE — rode `PH2D_FLIP_XFORM_SMOKE=1`** |

Tudo abaixo do `1b51f59b` já tinha smoke OK (ver 15b): W8 (domínio Point), W7.5 (gizmo
da pose), W7.5-F1 (pose afim). **`git log --oneline main..HEAD`** = 27 commits.

**O achado do 1º smoke do §4.A (Enio):** *"uma linha do triângulo e uma linha do quadrado
não são sensíveis à seleção"* — a **aresta de fechamento**. `positions().windows(2)` não
tem como produzir a costura, e o pick/marquee/hover a perdiam **enquanto o render e o halo
a desenhavam**: dava pra VER e REALÇAR uma linha que não dava pra APONTAR. Fechado em
`b793b47c` com uma porta única no modelo (`FlipStroke::segments()`, convenção espelhada do
render) + 3 gates / 2 mutações provadas. Saga completa: [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md) **#18**.
**Por que só o §4.A expôs:** o `hits` testa fill OU tinta, e o fill pega o interior inteiro
— toda forma fechada dos fixtures anteriores era **preenchida**, e a cena do §4.A é a
primeira com forma fechada **sem fill**.

**Os outros 2 achados do mesmo smoke (`017b8f00`)** — e os dois são a MESMA pergunta mal
respondida, *"onde a seleção é agarrável?"*:
1. *"qualquer clique na área do gizmo"* — errar a tinta **dentro** da caixa abria marquee
   (que ainda LIMPAVA a seleção). Agora é um `Move` do grupo. O interior **não** entra no
   hit-index (tornaria `on_canvas` falso sobre a seleção inteira e mataria a re-seleção ali
   dentro): quem responde é o **down do canvas do Edit**, que já roda lá. **Tinta primeiro**
   (clicar noutro traço dentro da caixa ainda o seleciona); **Shift** preserva o marquee
   aditivo — a saída de dentro da caixa.
2. *"um ponto único: os handles ficam sobre o ponto e não dá pra movê-lo"* — a caixa de um
   ponto tem meia-extensão `(0,0)`. **Sem EXTENSÃO, sem gizmo** (não se rotaciona nem se
   escalona um ponto): só o realce do W8, e o arrasto dele é o gesto de sempre. O limiar é o
   **zero exato**, não um épsilon.

**O 3º achado (`08ba6358`):** *"se eu seleciono no painel `Select: Point`, o gizmo do stroke
deve sumir imediatamente"*. Não sumia porque a troca de domínio faz **broadcast**
(`selection_to_point_domain`, W8): a MESMA seleção continua lá ponto a ponto, a caixa segue
com extensão, e só a regra do **domínio** some com ele. E é a regra certa, não só a
preferência: no Point o alvo do clique são as **âncoras**, e os handles pousariam em cima
delas (a bbox de um retângulo tem as âncoras NAS quinas). **O projeto já tinha tomado essa
decisão no Vector** — [ADR-0112](architecture/decisions/0112-vector-select-node-pen-are-three-tools.md):
*"o gizmo da forma só publica `GizmoView` no modo Select — em Node ele comeria o clique do
nó"*. **Stroke/Point aqui é o análogo exato de Select/Node lá** (e o idioma do Illustrator:
seta preta = caixa, seta branca = âncoras sem caixa).

A porta única é a **`grabbable_selection_box`** (`flip_selection_gizmo.rs`) — hoje com **3
recusas: domínio Point · arte instanciada · sem extensão**. A `selection_view` a **desenha**
e o `plan_down`/`plan_down_points` a tornam **arrastável**, então os dois somem juntos (sem
gizmo não há área). Duas funções divergiriam — e o artista veria uma caixa que não pega.
**Se você mexer no gizmo da seleção, é essa função que decide tudo.** O domínio vem de
`App::flip_edit_domain_now` (uma porta) e é **içado antes do empréstimo de `gfx`** no
`render_loop` (método em `&self` colide com `self.gfx.as_mut()`).

**O 4º achado (`994ce21c`):** *"quando `Select: Point` os pontos ficam todos selecionados.
faça com que comece com pontos desselecionados"*. A troca de domínio fazia **broadcast** (o
`02_referencia §11` do GP: traço aceso ⇒ todos os pontos dele acesos) — e o 1º gesto do
artista no Point é quase sempre *"quero estas duas âncoras"*, ou seja, ele começava
**desmarcando**. **Diverge do GP de propósito.** A volta ao Stroke continua **promovendo**
por `any()`; a assimetria é deliberada (entrar no Point = *"vou escolher âncoras"*; voltar
ao Stroke = *"as âncoras que toquei são deste traço"*). Par renomeado para o que de fato
acontece: **`enter_point_domain`/`enter_stroke_domain`** (`selection_to_point_domain`
mentiria — não converte mais, limpa). O **`broadcast_selection_to_points` ficou órfão e
saiu** (só o próprio teste o chamava; `select_all_points` usa `set_point_selected`), junto
com a doc que o listava como choke point do `point_sel` — **hoje são 2 choke points**.

**Esta regra e a do domínio são COMPLEMENTARES, não redundantes** — e o gate prova: com
`enter_point_domain` mutado para no-op, o gate do gizmo continua **verde** (ele arma a
seleção pelo CLIQUE, não pela troca de domínio) e só o gate novo cai. Entrar limpa; a regra
do domínio impede o gizmo **depois** que o artista acende âncoras.

**Gate partido em dois, de propósito:** o do ponto único passaria **pelo motivo errado**
depois da regra do domínio (o caso do ponto único só existe NO domínio Point). Então virou
`the_point_domain_never_opens_the_gizmo` (faz o broadcast REAL e exige a recusa) +
`a_selection_without_extent_never_opens_the_gizmo` (o caso alcançável no Stroke: um traço de
um ponto só). **Cada camada sangra sozinha** — `feedback_layered_defenses_need_per_layer_gates`.

**Split pelo cap de LOC** (HR-18, nunca allowlist): o **pick de TRAÇO** saiu para o módulo
irmão **`flip_select_pick.rs`** (`MIN_PICK_PX`/`stroke_at`/`hits`/`seg_dist2` — o gêmeo do
`flip_select_points`, que faz o pick de PONTO), com o `flip_select` **re-exportando** o
`stroke_at` (uma porta só). `flip_select.rs` 611→534; os 2 gates de área foram para
`flip_selection_gizmo_tests.rs`, e `flip_select_tests.rs` 648→571.

**Schema:** INTACTO — `FLIP_SCHEMA_VERSION` **7**, `PROJECT_SCHEMA` **15**, pin `(15, 7, 8)`.
O §4.A **não bumpou nada** (o `FlipSelectionDrag` é estado de runtime no `App`, não é
serializado; o campo `GizmoStateGroup.selection_view` também não).

**Gates no fechamento (1× sobre o diff):** verde em `ph2d-flip{,-fill,-render,-reshape}` +
`ph2d-tool-flip` + `ph2d-panel-flip{,-frames}` + `ph2d-ui-testkit` + `ph2d-editor-core` +
`ph2d-host-desktop` (64 suítes ok, 0 falhas) · clippy `--all-targets` limpo · GPU
`gpu_render`(8)/`gpu_fill_fit`(17) `--ignored` verdes · typos limpo · `no_magic_numeric` +
`architecture_panel_loc_cap` verdes · release builda · LOC dos 3 arquivos novos ≤ 429 (cap 600).

**O SMOKE do §4.A (rode isto):**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && PH2D_FLIP_XFORM_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
Abre com 1 objeto Flip de arte EXCLUSIVA — um **retângulo SELECIONADO** (roxo) + um
triângulo (azul) —, modo **Edit**, e o gizmo da SELEÇÃO já enquadrando o retângulo.
Roteiro: (1) quina = rotate (anel de hover) / escala em torno do centro do retângulo;
(2) borda = escala 1 eixo; (3) arrastar a arte (fora dos handles) = **move** (o gesto do
W6.1); (4) o **triângulo NÃO se mexe**; (5) **Ctrl+Z** desfaz o gesto inteiro (1 passo);
(6) clicar no vazio desmarca → o gizmo **some**; (7) trocar pro domínio **Point** e
selecionar meia geometria → o gizmo passa a enquadrar SÓ os pontos selecionados.

---

## 2. Como o §4.A foi construído (não re-derive)

**A espinha:** é o **espelho do gizmo da POSE** (W7.5, `flip_pose_gizmo.rs`) — o template
que o 15b §3.1 anunciou. Uma única diferença de fundo:

- **Pose gizmo** (instância): escreve a **pose da chave** (bloco rígido; arte compartilhada
  não deforma).
- **Selection gizmo** (arte exclusiva): assa o delta na **GEOMETRIA** dos pontos selecionados.

Os dois são **mutuamente exclusivos por `is_instanced`** — a `pose_view` só publica em
instância, a `selection_view` só em arte exclusiva **com seleção**. Nenhum toast: a que não
se aplica não publica handles (a alternativa "recusa com toast" do 15b §4.A ficou
desnecessária — o gate da view já separa os casos).

**Arquivos novos (shell):** `flip_selection_gizmo.rs` (+ `_tests`, 6 gates) + a cena
`flip_selection_smoke.rs`. Estado do arrasto: `App.flip_selection_drag`
(`FlipSelectionDrag` — não-`Copy`, carrega o `Vec<SelPoint>` do snapshot; o `move` faz
`take`-e-restaura, ≠ o `FlipPoseDrag` que é `Copy`).

**Reparametrização = a MESMA da pose** (reusa `pose_trs`/`trs_to_pose`): a seleção é um
"sprite" cujo pivô é o **centro da bbox dos pontos selecionados** (`c_art`, em coords da
ARTE), `parent_world` = afim do OBJETO, `start_transform` = TRS da pose ancorado em `c_art`.
`compute_gizmo_transform` roda byte a byte (modifiers/snap/contador de voltas).

**O bake (o que difere da pose) — `art_bake_xform`:** o gizmo devolve um TRS novo em
espaço de OBJETO; a geometria vive em ART. O delta afim ART→ART é
`pose⁻¹ ∘ new_aff ∘ start_aff⁻¹ ∘ pose` — desce o ponto pela pose, aplica a mudança do
frame do gizmo, sobe de volta. Sob pose girada a geometria anda na direção certa (regra-mãe
#10); numa pose de translação pura (o caso comum) as pontas se cancelam e sobra
`new_aff ∘ start_aff⁻¹` = rotação/escala/translação em torno de `c_art`. **Arte exclusiva
PODE ter pose ≠ identidade** (o `make_single_user`/Unlink preserva a pose da instância) —
por isso a pose entra na conta, não se assume identidade.

**Snapshot no Down, recomputa do snapshot:** as posições de partida de cada ponto entram no
`FlipSelectionDrag.points` no Down; cada Move recomputa `p' = M_art(p₀)` do snapshot — nunca
compõe por-frame (deltas compostos driftariam). Os **buracos** entram no snapshot só para os
traços INTEIROS selecionados (`all_points_selected`, o mesmo critério do
`translate_selected_points`).

**Sem interior (Translate):** o arrasto de canvas do Edit (W6.1/W8) já translada a seleção;
um interior no hit-index roubaria o clique de re-seleção. Os handles keyed
(`GizmoTarget::FlipSelection`) registram só rotate/scale.

**6 gates (4 mutações provadas vermelhas, `flip_selection_gizmo_tests.rs`):**
- `the_selection_gizmo_box_lands_on_the_posed_selection` (seed=sample; mut: `selection_center_half` varrer TODOS os pontos)
- `the_snapshot_is_only_the_selected_points` (mut: `snapshot_selected_points` ignorar `point_selected`)
- `a_rotate_drag_spins_the_selection_about_its_center` (mut: `art_bake_xform` dropar `start_inv`)
- `an_instanced_drawing_never_opens_the_selection_gizmo` (mut: remover o `is_instanced`)
- `an_empty_selection_never_opens_the_gizmo` (coberto pela 1ª mutação também)
- `a_pure_translate_bake_is_a_rigid_shift_matching_the_move_funnel` (casa com `object_delta_to_art`)

O par render/input inverso e o funil pose-free do move (em `flip_transform`/`flip_pose_gizmo`)
**continuam verdes** — esta fase não os tocou.

---

## 3. Notas de INTEGRAÇÃO (pro agente integrador do Enio)

- **`ph2d-editor-core` tocada append-only** (foundational), 3 sítios NOVOS a somar aos do
  W7.5 (anote a colisão de mesmo-símbolo se outra linha mexeu no gizmo):
  - variante **`GizmoTarget::FlipSelection`** (`gizmo/drag.rs`, apendada por último, depois de `FlipPose`)
  - scrambler de id **`0x_5F1E_C7A0_2B94_D6E3`** em `keyed_handle_id` (`gizmo/paint.rs`) —
    distinto dos existentes (Primary/Extra/Global/FlipPose)
  - campo **`GizmoStateGroup.selection_view`** (`screens/hero/state.rs`, apendado) + braço de
    pintura keyed em `screens/hero/paint.rs` (logo após o de `pose_view`).
- **Nenhum contrato congelado tocado.** O `GizmoTarget` NÃO é gateado por
  `architecture_*_contract_surface` (é do editor-core, não da superfície de nodes/tools/vector).
- **Sem bump de schema** — o pin `(15, 7, 8)` de `project_tests.rs` fica. Se outra linha
  bumpou `PROJECT_SCHEMA`/`FLIP_SCHEMA` em paralelo, reconcilie os que SOMAM
  (`feedback_numbers_that_sum_across_lines_count_dont_pick`) — mas o §4.A não contribui número.
- **`ph2d-flip` (modelo) ganhou `FlipStroke::segments()`** (`b793b47c`, método novo — nada
  serializado muda, sem bump). É a **porta única** de "quais são os segmentos deste traço?".
  Se outra linha tiver adicionado um consumidor que itere `positions().windows(2)` sobre um
  traço que pode ser `closed`, ele tem o bug **#18** e deve passar por `segments()`.
- **Shell:** `flip_selection_gizmo.rs`/`_tests`/`_smoke.rs` + **`flip_select_pick.rs`** (split do cap de LOC) novos; `main.rs` (2 `mod` + 1 init
  de campo), `app_state.rs` (campo `flip_selection_drag`), `input_dispatch.rs` (down/move/up,
  espelho dos 3 sítios do pose gizmo), `render_loop/mod.rs` (publica `selection_view`; o gate
  Flip+Edit foi extraído p/ `flip_edit_mode` e reusado pelas duas views).
- **Docs de planejamento** (`docs/Flip/` etc.) seguem como no 15b. Este handoff É committado
  na branch (como os anteriores); NÃO existe untracked na árvore primária, então o
  `merge --ff-only` não quebra.
- Rode o **ship completo** no fechamento da jornada (`scripts/ship.sh`) — `nextest-impacted`
  teve false-green em RAM baixa; o replay-hash muda se o postcard mudou (aqui não mudou).

---

## 4. A FILA restante (o §4.A saiu; o resto herda do 15b §4)

> **Ordem recomendada:** §4.B é agora o mais auto-contido e o próximo natural (não toca o
> gizmo). §4.C são tarefas curtas entre smokes.

### §4.B — **Segment mode** (o 3º domínio do GP; `02_referencia §11` dá a receita)
Corte por interseção VISUAL: raycast de cada segmento contra um BVH 2D do frame (ignorando 3
vizinhos); hit = início de segmento. **Cíclica sem corte = ZERO segmentos → fallback "1 ponto
seleciona a curva toda"**; o último segmento de cíclica enrola em DOIS ranges. É 100%
screen-space (port natural), consome a MESMA `point_sel` do W8 (seleciona um RANGE de
pontos), e o toggle vira uma 3ª pill ao lado de Stroke|Point. **Auto-contido: não toca o
gizmo** (nem o de seleção nem o de pose).

### §4.C — Refinos não-bloqueantes (tarefa curta entre smokes)
Reorder de camada por drag · duplicar/agrupar camada · máscaras de camada na UI · raio
dedicado da borracha + preview · curva de pressão editável · round caps/bevel joins ·
**write-back do painel** (espelhar o estilo da seleção no swatch — `08 §6`) · cache de
tesselação com LRU.

### §4.D — **W6 (timeline global): ADIADA** por ordem do Enio até a timeline principal fechar.
O playhead do Flip JÁ é o global. Se o Enio reabrir, leia o handoff da linha `anim` antes.

---

## 5. As regras do módulo que NÃO se re-derivam erradas (do 15b §2, ainda valem)

1. O traço é a união global da polilinha (BUGS #1). 2. O balde ancora no EIXO da linha
(BUGS #14). 3. A cor entra POR BAIXO da linha (BUGS #15). 4. A forma pinta A SI MESMA
(BUGS #16/#17). 5. O autokey é por FERRAMENTA. 6. Há TRÊS relógios (BUGS #7). 7. A escultura
move as REGIÕES e os buracos delas. 8. **Seed = sample** — quem PINTA e quem ESCREVE derivam
da MESMA função (`art_to_world`/`world_to_art`); o gate
`the_render_and_the_input_are_exact_inverses` prende o par. 9. Arte compartilhada (instância)
NUNCA deforma por arrasto — mover/esculpir escreve a POSE; deformar exige Unlink. **O §4.A é
o corolário disto:** o gizmo de seleção só existe em arte EXCLUSIVA; na instância é o gizmo
de POSE. 10. O funil do MOVE é POSE-FREE (`flip_active_world_to_object`), mas o DELTA desce à
arte pela parte linear inversa da pose (`object_delta_to_art`) — e o §4.A usa a MESMA descida
no bake (`art_bake_xform`, a conjugação `pose⁻¹ ∘ … ∘ pose`).

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
(Arch-gates: LOC 700/crate e 600/shell — **split em módulo irmão, nunca allowlist**, `fmt`
ANTES de medir · `no_magic_numeric` / `arch_safe_clamp_only` (`safe_clamp` ou
`// CLAMP-OK`/`LITERAL-PX-OK` com razão) · `architecture_panel_wiring_parity` ·
`a_schema_bump_anywhere_must_bump_the_project_schema` · `node_id_collisions`.)

**cwd:** trabalhe SEMPRE dentro do worktree. Mutação sempre por caminho ABSOLUTO. Desfaça
mutação com **`cp` do backup**, NUNCA `git checkout`.

**Smokes prontos:** `PH2D_FLIP_DEMO=1` · `PH2D_FLIP_POSE_SMOKE=1` (gizmo da pose) ·
`PH2D_FLIP_EDIT_SMOKE=1` (domínio Point) · **`PH2D_FLIP_XFORM_SMOKE=1` (gizmo da seleção, §4.A)**.
Diagnóstico: `PH2D_FLIP_FILL_DEBUG=1` (balde) · `PH2D_FLIP_SELECT_DEBUG=1` (Edit).

**Referência do Blender** (GPL — comportamento, nunca código):
`~/Downloads/blender-5.2-grease-pencil-ref/`. Docs do módulo: [`docs/Flip/`](Flip/00_README.md)
(`02_referencia §11` = a receita do §4.B).

---

**Você fecha o bloco, escreve o handoff de integração, e PARA. Não integra. Não pusha.**
