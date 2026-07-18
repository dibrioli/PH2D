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

## 1. Estado da linha — §4.B INTEGROU; **§4.C em andamento na branch** (à frente da main)

**Fechados nesta rodada (sobre a base integrada, à frente da main — não integrados):**
- **§4.C.1** (`a5738e98`, **smoke OK**) — o pedaço é a unidade visual do Segment (detalhe abaixo).
- **§4.C.2 — Duplicate Layer** (`47fd348c`, **smoke OK 2026-07-17**): capacidade nova (reorder já
  existia via ↑↓; duplicar não). `FlipObject::duplicate_layer` = cópia INDEPENDENTE acima da
  original — desenhos próprios (deep-copy; editar a cópia não toca o original), a instância
  DENTRO da camada preservada (mapa por-desenho ⇒ ciclo continua ciclo), refcount por-quadro
  (delete independente). Botão "Duplicate" na toolbar (Add | Duplicate | Delete; a toolbar
  virou loop, largura de `.len()` — o `no_magic_numeric` pega `3.0` solto). Gates: 4 modelo +
  2 seam (forward do painel + apply do shell) + 2 shell, mutações provadas.
  **Smoke:** `PH2D_FLIP_DEMO=1` → tool Flip → Layers → selecione FG → Duplicate.
- **§4.C.3 — Rename Layer** (`a4609669`, **pendente smoke**): companheiro natural do
  Add/Duplicate/Delete (o animador duplica "FG copy" → renomeia "FG shadow"; não existia).
  **DOUBLE-click** no nome abre um `TextInput` inline SOBRE a faixa do nome (espelha o
  `marker_rename` do timeline), semeado+focado; Enter/Blur commitam, Esc cancela; clique
  simples segue selecionando. Seam: o commit viaja pela **Row id** via `SelectOption(row_id,
  name)` — o shell (`flip_layers`) já decodifica Row id → camada (mesmo canal do blend chip,
  edição de DOCUMENTO). `FlipObject::rename_layer` (troca só o nome, id/frames/arte intactos).
  Estado: `FlipPanelState.layer_rename` (a struct deixou de ser unit → os literais `= FlipPanelState;`
  viraram `::default()`). Guarda de teclado: o Delete/Backspace do Edit Mode agora cede a
  campo de texto focado (`!vector_text_field_focused()`) — bug latente até o Flip ganhar um
  campo. Gates: 2 modelo + 3 seam painel (abre/commit/ownership) + 2 shell, mutações provadas;
  `FLIP_LAYER_RENAME_INPUT` no allowlist de wiring-parity (campo dinâmico, como o marker).
  **A guarda de teclado NÃO é gateada de propósito:** `vector_text_field_focused()` lê o
  `hero_screen.store` (só com `gfx`=janela); headless nasce `gfx=None` ⇒ o helper é sempre
  `false` ⇒ um teste headless ficaria verde COM ou SEM a guarda (o "verde por acidente" do §4.C.1).
  É mirror de uma linha do bloco vetorial irmão, já testado, sobre o mesmo helper.
  **Smoke:** `PH2D_FLIP_DEMO=1` → tool Flip → Layers → **double-click no NOME** de uma camada
  → digite → Enter (Esc cancela; Backspace edita o texto, não apaga traços).

- **§4.C.4 — raio/força PRÓPRIOS da borracha atrás de um LINK** (`27144941`, **pendente
  smoke**): ordem do Enio — *"como no blender: a opção de linkar ou não as propriedades dos
  pincéis de pintura e borracha através de um botão de link na linha da propriedade"* (o
  **Unified Paint Settings**, um toggle POR PROPRIEDADE, na linha dela).
  **⚠️ Leia isto antes de mexer:** eu havia proposto *"dar raio próprio à borracha"* e me
  RECUSEI a fazer sozinho, porque **reverteria o gate**
  `size_is_shared_by_brush_eraser_and_sculpt_and_absent_elsewhere`. O desenho do Enio é
  melhor e dissolve o conflito: com o toggle, o **default LINKADO preserva o comportamento
  histórico** (a borracha usa o `FLIP_SIZE` do pincel) ⇒ **o gate continua VERDE e continua
  verdadeiro**; deslinkar virou opt-in. A cerca de Chesterton foi honrada **emendando o que
  ela afirma**, não derrubando-a — o padrão a copiar quando um pedido esbarrar num gate.
  **Escopo: pintura↔borracha.** O **Sculpt segue compartilhando** (decisão dele, no `params.rs`,
  intocada). **Porta única:** `FlipTool::eraser_size_px()`/`eraser_strength()` resolvem o link
  UMA vez; o snapshot publica os EFETIVOS (`erase_px`/`erase_strength`) e o anel do cursor +
  o apply da borracha leem só esses campos. **Dois widgets por propriedade** (um slot de
  slider guarda UM valor); os próprios nascem nos defaults do pincel (1º deslink não pula) e
  sobrevivem ao re-link. Gates: 4 tool + 4 seam + 1 anel + **1 arch-gate**
  (`the_eraser_uses_the_erasers_own_numbers`: `flip_erase_apply` precisa de `gfx`, é
  inalcançável headless ⇒ o gate lê o arquivo do produto e proíbe `width_px`/`opacity`).
  LOC: `paint_sections.rs` estourou (681/600) ⇒ primitivas de linha foram pro irmão
  **`paint_rows.rs`**. **Smoke:** modo **Erase** → ícone de corrente à direita das linhas
  Size/Strength; aceso = segue o pincel, apagado = a borracha tem o seu (mude o Size e veja
  o ANEL mudar, com o pincel de desenho intacto).

- **§4.C.5 — a borracha macia é fato do CAMINHO + cena de boot VAZIA** (`d760c745`,
  **pendente smoke**). Dois achados do smoke do §4.C.4:
  - 🔴 *"qualquer nível de strength apaga completamente a linha, nunca deixa
    semitransparente"* — era **acumulação sequencial**: `ops[i] -= strength·falloff` **por
    DAB**, e a borracha carimba um dab por EVENTO DE PONTEIRO ⇒ `0,1 × 12 dabs` zera a
    linha. O resultado era função de **quão fino o motor amostrou o caminho**. É a MESMA
    doença que o Painter curou 2× (cápsula do depósito · mordida telescópica `2e1806fb`) e
    a lei vale igual: *o apagado é propriedade do pincel e do CAMINHO, nunca do
    espaçamento*. **Cura: a mordida tem PISO** (`soft_erased`, porta única) — o Soft leva a
    opacidade até `1 − strength·falloff` e PARA ⇒ **idempotente ⇒ independente de
    amostragem por construção**, sem estado de sessão. **Strength É a translucidez que
    sobra.** O `.min(current)` impede a borracha de DESAPAGAR (empurrar pra cima um ponto já
    mais claro que o piso). ⚠️ Trade aceito: passadas repetidas não desbotam mais — pra
    apagar mais, suba a Strength (acumular entre gestos reintroduz o bug dentro do gesto).
    3 gates, **cada camada com a sua mutação**. ⚠️ O gate antigo
    `soft_mode_reduces_opacity_then_cleanup_removes_faded` **tinha o bug escrito como
    premissa** (ficava parado e contava com a acumulação pra zerar as PONTAS, que parado só
    veem `falloff ≈ 0,8`) — agora VARRE a linha, que é o que o artista faz.
  - **Cena de boot vazia:** `populate_sim_live` (8 entidades falsas: `group_01/02` +
    `sprite_001..008`) **REMOVIDO** — existia (M14.4a) só pra Hierarquia ter linhas quando o
    app não tinha conteúdo real; hoje tem, e o andaime virou ruído na árvore do artista.
    Removido, **não gateado** (código morto mente). O `populate_sim` (Vogel 1000,
    `PH2D_M5_DEMO=1`) **FICA** — cerca de Chesterton documentada (frame-budget do HR-4).
  - LOC: `flip_erase.rs` passou de 600 ⇒ testes pro irmão `flip_erase_tests.rs`.

- **§4.C.6 — o Size mede o MUNDO + Strength é Soft-only** (`9b149bd8`, **pendente smoke**):
  - 🔴 *"a largura do traço está relativa ao zoom do canvas e não é fixa no mundo"* —
    **REVERTE a decisão de 2026-07-11** (pincel ABSOLUTO em px de tela), que estava
    documentada em 4 arquivos. O dono da cerca foi quem a tirou, e a nova é a certa: traço
    é ARTE, e arte não muda de espessura porque a câmera aproximou. **A cura já estava
    desenhada no renderer** (`thickness_px = raio_mundo · px_per_world`); o `camera_raw`
    passava `1.0` justamente para FORÇAR a leitura em tela. **Porta única
    `ph2d_tool_flip::size_to_world`** (`SIZE_PX_PER_WORLD = 100`) — todo sítio que AUTORA
    passa por ela (traço · Edit ×2 · borracha · sculpt); o anel faz o caminho de volta e
    **acompanha o zoom** (ele promete o que vai acontecer). ⚠️ **Não dava para virar só o
    traço:** o Size é compartilhado com borracha/sculpt e, desde o §4.C.4, é literalmente o
    MESMO número quando linkado — mundo pra desenhar e tela pra apagar seria um número com
    dois significados. **De quebra curou o documento:** posições eram mundo e larguras eram
    tela, e essa mistura já custara um bug ao balde (*"uma linha de 3 unidades de mundo
    (≈324 px!)"*) — a conversão do `flip_fill::boundaries` **SUMIU**. ⚠️ O `px_to_local` do
    Reshape FICA, mas só para o DELTA do arrasto (gesto de tela por definição).
  - 🔴 *"borracha hard não obedece a strength"* — não obedecia mesmo, e nunca obedeceu:
    Hard corta o ponto, Stroke apaga o traço, as duas são binárias e o `erase_at` sempre
    documentou *"(Soft only)"*. O slider ficava pintado e INERTE = o controle morto que a
    doutrina modal do painel proíbe. Agora a linha Strength (e o link dela) só existe no
    **Soft**; o Size fica nos três.
  - 4 gates novos, mutações provadas. Testes que codificavam a lei ANTIGA foram
    **corrigidos, não silenciados** (os 5 do anel ganharam um zoom de REFERÊNCIA explícito
    onde as duas leis coincidem; o do Edit tinha fixture em números da era-px, o que fazia
    o ida-e-volta ser só proporcional — em mundo é EXATO, que é o que ele diz testar).

**§4.C.1 — o PEDAÇO é a unidade visual do modo Segment** (`a5738e98`, **smoke OK 2026-07-17**).
Duas coisas, um primitivo:
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

- ✅ **realce de HOVER no Segment** — FECHOU em §4.C.1 (`a5738e98`, smoke OK).
- ✅ **duplicar camada** — FECHOU em §4.C.2 (`47fd348c`, smoke OK).
- ✅ **renomear camada** — FECHOU em §4.C.3 (`a4609669`, pendente smoke).
- reorder de camada por DRAG (o reorder já existe via ↑↓; isto é a affordance de arrastar) ·
  AGRUPAR camada (precisa de conceito de grupo no modelo do Flip — não existe hoje).
- ⚠️ **máscaras de camada na UI — NÃO é "só a UI":** o modelo tem `FlipLayer.masks`
  (`Vec<LayerMask{source, invert}>`) mas **NENHUM consumidor** — o `flip_pass.rs` compõe as
  camadas pelo `LayerCompositor` (blend/opacity) e **nunca aplica máscara**. Expor a UI sem o
  render seria um controle morto (o bug nº 1). É feature GRANDE (render de clip/mask + UI),
  não refino — precisa de plano/ordem, não é um "próximo".
- ✅ **raio/força próprios da borracha** — FECHOU em §4.C.4 (`27144941`, pendente smoke), como
  **toggle de LINK** por propriedade (Blender): o default linkado preservou o gate
  `size_is_shared_...`, então não houve reversão. Ver o §1.
- curva de pressão editável · round caps/bevel joins (caps já existem no `pack.rs`; **joins**
  = trabalho de shader/render, mais fundo).
- **write-back do painel** (espelhar o estilo da seleção no swatch — `Flip/08 §6`; ⚠️ é
  design-ambíguo: o painel hoje é "aplique este valor", não espelho)
- cache de tesselação com LRU (perf — só com problema MEDIDO)

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
