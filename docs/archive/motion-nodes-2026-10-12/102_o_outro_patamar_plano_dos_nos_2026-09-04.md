# ARQUIVO — 102_o_outro_patamar_plano_dos_nos_2026-09-04.md (história, 211 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`102_o_outro_patamar_plano_dos_nos_2026-09-04.md`](../../Motion%20Nodes/102_o_outro_patamar_plano_dos_nos_2026-09-04.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-225,369-379,448-523` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### W2 — FÓRMULAS EM QUALQUER CAMPO + PICK-WHIP (capacidades 1 e 2 — um mecanismo)
**Objectivo.** `amplitude = @n12.rows * 4` num campo; arrastar de um param para outro escreve
essa fórmula; renomear um nó **não parte nada**.
- **Gramática** (`ph2d-expr-parse`, append): `@n<id>.<param>` (o param EFECTIVO do outro nó —
  resolvido pela mesma escada) · `@n<id>.<porta>` (o **elemento 0** do `v` daquela saída — o que
  o `d` lê) · `@self.<param>` · `@t` (playhead, s) · `@frame`. ⛔ `Attr` (coluna por-elemento) é
  **recusado num campo** com a cura: *«a field is one number per node; per element use
  `value.attribute` → `motion.drive`»* — um param é um **uniform** (§2, W1).
- **Identidade = id, nunca o nome** (Houdini guarda *«a hidden pointer to the node's ID»*; o AE
  guarda o nome e parte). O editor aceita um **rótulo** (`Graph::label`) e reescreve para `@n<id>`
  ao confirmar; o display mostra o rótulo quando existe; rótulo ambíguo ⇒ recusa com a lista.
- **Formato:** registo **`f <id> <param> <expr...>`** (texto livre após o 3.º espaço, como `x`),
  acima de `[layout]`, header **`v6` iff existe um `f`**. `Graph`: `param_exprs: BTreeMap<NodeId,
  BTreeMap<String, String>>` + `set_param_expr / clear_param_expr / param_expr`. **Invariante com
  gate: um param tem no máximo UMA fonte** — `set_param_expr` chama `undrive_param`, `drive_param`
  apaga o `f`; a leitura é **uma porta**: `Graph::param_source(node, name) ->
  Option<ParamSource<'_>>` com `ParamSource::Wire(NodeId, u16) | Expr(&str)`.
- **Dependências:** `Graph::expr_refs(node, name) -> Vec<NodeId>` (parse cacheado por hash do
  texto no `Cook`: `BTreeMap<u64, Arc<Expr>>`); `would_cycle` anda pelos `f`; `remove_node` limpa
  os dois lados — uma referência a um nó apagado **mantém o texto**, cai para override/default e
  acende o chip de erro (a lei do `d`: nunca `0.0`).
- **Avaliação (cook, passo 1b′):** cozinhar as referências (revisões em `input_revs`), avaliar
  com `ExprBindings { cook, graph, node, in_key }` (impl de `ph2d_expr::Bindings`: `param` resolve
  `@n.p` pela escada efectiva, `attr` é inalcançável por construção) e **inserir no mesmo
  `driven` que o `d` preenche** — `EvalCtx::param` **não muda** (uma porta).
- **Device:** o escalar avaliado na CPU entra no `drv` de W1 como *fallback* (zero cópias) — um
  campo custa nanossegundos e **não precisa de `to_wgsl`**; `@n.<porta>` usa a lane (a) tal qual.
- **Fingerprint:** `param_source::fingerprint` ganha o hash do texto; as revisões das referências
  já entram por `input_revs`.
- **Determinismo:** a política de `Func` do `ph2d-expr` (o agente lê `expr.rs` e escreve no doc
  do gate quais funções existem); um campo corre na CPU como o `motion.expression` corre hoje.
- **UI:** o campo numérico aceita o prefixo `=` (idioma Excel/Cavalry) →
  `MotionParamIntent::SetParamExpr { node, param, text }` (variante nova, append); chip `[≈]`
  com o texto no balão; erro = a mesma tríade do L-System (a queixa **nasce · chega à row ·
  chega a PIXEL** — 3 gates). No cartão (W4): `GraphHitKind::ParamSocketOut { node, param }` — o
  **pick-whip**: soltar num `ParamSocketIn` escreve `= @n<from>.<param>`; soltar num pino `VALUE`
  oferece **«Number from param»** (insere `value.number` com a fórmula — ⛔ sem nós escondidos).
  Um `f` que referencia outros nós desenha **fios tracejados de VISTA** (derivados dos `refs`,
  não são arestas do documento): `GraphEdgeView` ganha `kind: Wire | Ref` (append).
- **Gates:** `an_expression_round_trips_byte_for_byte` (`v6` só quando existe) ·
  `a_param_has_at_most_one_source` (mutação: saltar o `undrive`) ·
  `an_expression_reference_is_cooked_first` · `an_expression_cycle_is_refused_like_an_edge` ·
  `renaming_a_node_does_not_break_an_expression` · `a_deleted_reference_falls_back_and_says_so` ·
  `a_column_in_a_field_is_refused_with_the_cure` ·
  `the_expression_value_reaches_the_device_as_a_uniform` (paridade) · `the_error_reaches_pixel`.
- **Smoke:** `PH2D_EXPR_FIELD_SMOKE=1` — grid → oscillator com `amplitude = @grid.rows * 4`;
  arrastar `rows` e ver a amplitude seguir; pick-whip de `rows` para `phase_stagger`.
- **Orçamento:** um `f` = um parse cacheado + N `f32` ops por quadro; **zero passes**.
- **Contrato:** `NodeManifest` intocado; `Graph`/`format` não são congelados (v6 pela política).

### W3 — KEYFRAME EM QUALQUER PARAM, NO CARTÃO (capacidade 3)
**Objectivo.** O losango ao lado de cada número; a timeline anima params de nós; o undo e o
ficheiro nunca veem a pré-visualização.
- **Timeline:** `PropKind::Param = 6` (append, `repr` numérico — o bump do `DOC_VERSION` do
  `TimelineDoc` segue a regra dele); `AnimTarget` já é **opaco** e documenta *«a node param»* como
  uso: `motion_param_target(node, param) = AnimTarget::new(fnv64("motion-param\0<id>\0<param>"))`
  no shell (a timeline continua genérica; o `NodeId` é estável no documento).
- **Aplicar por quadro:** o shell lê o valor amostrado de cada binding motion → **`MotionPreview`**
  (irmão de `PreviewDrive`, chave `(NodeId, String)`, guarda o **autorado**) → `graph.set_param`
  antes do cook; `substitute_authored` antes de toda captura (undo/save) e `restore_live` depois —
  a lei do `preview_drive.rs`: *«o documento é o valor AUTORADO»*. Entra na **assinatura** de
  `ProjectState::capture` como o ledger existente (uma função-irmã «com ledger» seria a segunda
  porta pela qual o defeito voltava).
- **Precedência — UMA lei, um gate:** `d`/`f` (exclusivos) **>** keyframe **>** override **>**
  default. Um param dirigido e keyframado mostra o chip `[◆]` e a lane cinzenta — nunca um
  controlo que não faz nada sem o dizer (doc 90).
- **UI:** `ParamRow` ganha `keyed: KeyState { None, Keyed, OnKey }` (derivado do `TimelineDoc`);
  losango → `MotionParamIntent::InsertKey { node, param }` / `RemoveKey`; **auto-record** com a
  lei do Mini Cavalry (Phase 26): *Play desliga o auto-record, Stop restaura, toggle manual
  invalida* — sobre o `MotionTransport` que já existe. Keyframes amostram **em tempo de
  timeline**, nunca no `time` do eval (Phase 25).
- **Unidades:** `PropKind::Param` é sem unidade — o param leva a sua (`ParamUnit`); a divergência
  rad/graus da memória de 2026-07-09 **não se coloca**.
- **Memo:** o ledger escreve overrides ⇒ o `params_fingerprint` invalida o memo sozinho (memória
  de 2026-07-09, achado 4).
- **Gates:** `a_keyframed_param_never_reaches_the_undo_queue` · `…_never_reaches_the_file` ·
  `the_precedence_is_one_law` (tabela de combinações) · `insert_key_from_the_card_lands_in_the_doc`
  · `play_disarms_auto_record_and_stop_restores_it` · `a_bound_motion_param_survives_save_and_load`.
- **Smoke:** `PH2D_MOTION_KEYS_SMOKE=1`. **Orçamento:** amostragem = `O(bindings)`
  (`doc.bindings()` nomeia quem anima). **Contrato:** nenhum; o **W4.T4** (dock da timeline no
  Motion) é pré-requisito de UI e está *«aguardando re-smoke»* (§5 do CLAUDE.md).

### W4 — O CARTÃO 2.0 (doc 101 W1–W6 + doc 63 I1–I8 + a spec da §1)
- **Renderer:** crate-folha **`ph2d-param-rows`** (o modelo `ParamRow…`, `paint_rows`,
  `snapshot_ids`, com `RowHost::{Panel, Card { hot }}`); o painel fica **byte-idêntico** (golden
  de cena do `ph2d-ui-testkit` antes/depois).
- **Formato:** promoção **`q <id> <param>`** (**v7**; ⚠️ `p` já é o override — o doc 101
  escreveu `p` por engano, corrigido); sob `[layout]`: `c <id> <0|1>` (expandido) e `w <id> <px>`
  (largura) — **não-semânticos**, como `l`.
- **Rows no cartão:** `card_rows(n, promoted, expanded)` em `geom.rs` (o hit-test e o pintor
  leem a mesma); conteúdo = dirigidos (`d`∪`f`) ∪ promovidos ∪ (expandido ⇒ todos, em secções
  `ParamGroup`). **Só o cartão quente regista widgets**; os frios pintam glifo+texto — gate:
  `the_card_registers_widgets_only_when_hot` (nº de registos = f(quente), nunca f(N)).
- **Sockets de param:** `GraphHitKind::ParamSocketIn/Out { node, param }` (append); 3.ª forma de
  glifo; soltar um fio de valor no `In` = `GraphIntent::DriveParam` (existe); do `Out` = W2.
- **LOD:** `z₁`/`z₂` medidos em W0 (legibilidade = `TITLE_SIZE × zoom ≥ n px` pelo `TextSystem`);
  abaixo de `z₁` rows nem pintadas nem registadas (gate mede as duas metades).
- **Compacto:** `value.*` com ≤ 1 param e 1 saída = pílula com o glifo.
- **Decoração:** a tabela da §1.1 — largura 220, cabeçalho em gradiente + ícone + modo, silhueta
  no contorno, sombra (token novo), fios por espécie (`GraphEdgeView.ty: PortType`, append),
  chips de proveniência `[·] [≈] [◆] [◇]`.
- **Gestos baratos de doc 63:** *value ladder* (arrasto com modificador em degraus
  0,001…100) · **«Drive by…»** no menu do param (LFO / Noise / Random / Expression — insere e liga
  pela porta da paleta + `DriveParam`) · pips **bypass** (existe, `y`) e **viewer** (liga a moldura
  daquele nó); ⏸️ *solo* fica nomeado, sem mecanismo de sink temporário definido.
- **Gates:** os 8 do doc 101 §8 + `the_panel_paint_is_byte_identical_after_the_extraction` +
  `every_card_of_the_registry_fits_the_tablet_when_lean` (≤ 25 % do mini) +
  `every_widget_file_wires_a11y` (linha nova).
- **Smoke:** `PH2D_CARD_SMOKE=1` (a cena do `PH2D_DRIVEN_ROW_SMOKE=1` — já tem dois params
  dirigidos e um por promover). **Orçamento:** paint de 20 cartões ricos medido em W0; **zero
  alocação por cartão frio por quadro**. **Tablet:** cada sub-wave imprime o custo nos 3 alvos.

### W5 — ALÇAS NO CANVAS, DERIVADAS DA TABELA (capacidade 5)
- **Derivação** (`shells/desktop/src/param_handles.rs`, tabela pura sobre
  `ParamUiHint`+`ParamUnit`): `(<b>_x, <b>_y)` ambos `Length` ⇒ `Handle::Point` (**20 pares
  hoje**) · `radius`/`<b>_radius` `Length` ⇒ `Handle::Ring` · `angle`/`rotation`/`<b>_angle`
  `Angle` ⇒ `Handle::Dial` · `width`+`height` ⇒ `Handle::Box`. **127 params** com unidade espacial.
- **Os dois à mão viram os primeiros consumidores:** gate
  `the_derived_handle_for_field_box_equals_the_hand_written_one` (números iguais ao bit), depois o
  código à mão de `field_gizmo.rs:209` **é apagado** (sem segunda fonte).
- **Sink:** `MotionParamIntent::SetParam` (undo por diff); com auto-record ligado ⇒ chave (W3).
  Só para o nó **seleccionado**; hit ≥ 44 pt no toque; espaço de mundo.
- **Bidireccional por construção** (a alça lê o param que escreve) — o Blender 4.3 exige ligar
  *Gizmo nodes* à mão ([fonte](https://projects.blender.org/blender/blender/pulls/112677)).
- **Gates:** `every_spatial_pair_in_the_registry_grows_a_handle` (censo derivado — um nó novo com
  `center_x/center_y` ganha alça de graça) · `a_handle_drag_is_one_undo_step` ·
  `a_handle_never_paints_for_an_unselected_node`.
- **Smoke:** `PH2D_HANDLES_SMOKE=1`. **Orçamento:** 1 overlay por nó seleccionado. **Tablet:**
  0 px permanentes.

### W6 — CUSTO E ROTA NO CARTÃO (capacidade 7 — a metade visível de W0)
- **Badge** sob o título (overlay ligável, como o *Timings* do Blender): `⚡ 0,12 ms` (device,
  `last_stage_ns`) · `CPU 3,4 ms` (`cost_of`) · `CPU · sem kernel` / `CPU · param dirigido` / …
  (a `Refusal` de W0, **por nó**); o balão diz a razão inteira e **a cura** quando há uma (o
  `converter_from` já deriva conversores; a lane de W1 apaga duas razões).
- **HUD por cena** (canto do canvas, overlay): rota · passes · ms CPU/GPU — a linha do
  `PH2D_MOTION_ROUTE_LOG` no ecrã.
- **Gates:** `every_refusal_has_a_badge_text` (tabela `Refusal → &'static str`, censo
  exaustivo — `match` sem `_`) · `the_badge_never_paints_below_the_lod` ·
  `a_badge_with_no_clock_says_so` (sem `TIMESTAMP_QUERY` ⇒ *«GPU · sem relógio»*).
- **Smoke:** ligar o overlay (`O`? — decidir a tecla no design system) numa cena `=108`.
- **Orçamento:** texto por cartão só com o overlay ligado. **Tablet:** 0 px sem o overlay.

### W8 — NÓ PRÓPRIO: subgrafo com INTERFACE + BIBLIOTECA (capacidade 6 — o W-J do doc 63)
- **Interface = promoção + fórmula (a lei do Houdini):** *«When you promote a parameter, Houdini
  creates a copy … and replaces the original … with an expression that references the value of
  the parameter on the asset»*. Aqui: a interface de um subgrafo é uma lista de
  `(nome, ParamUiHint, valor)` gravada no documento do subgrafo (`ph2d-motion-doc`, registo
  próprio — o agente lê o formato desse crate antes de escolher a letra), e cada param interno
  promovido recebe **`f = @g<subgraph_id>.<nome>`** — W2 resolve `@g` como resolve `@n`. **Zero
  mecanismo novo de cook.** O cook continua PLANO (doc 57).
- **Cartão do subgrafo:** as rows da interface, pelo mesmo renderer (W4); portas = o que cruza a
  fronteira (já derivado).
- **Ficheiro:** `.ph2dnode` = o texto do sub-grafo (formato `format.rs`, ids re-mapeados a partir
  de 1) + a interface + nome + versão + categoria; biblioteca em `~/.ph2d/nodes/` e
  `<projecto>/nodes/`; a paleta lista-os numa secção **«Yours»** com a mesma busca.
- **Inserir = COPIAR** com `Graph::insert_raw(id, type_name)` e o re-mapeamento de ids (fase A);
  **ligado** (hash de conteúdo + *«Update from library»*) é a fase B — o modelo Components
  (ADR-0164, mestre/instância por `StableId`) é o precedente da casa, ⛔ não reinventar.
- **Gates:** `a_promoted_inner_param_is_an_expression_to_the_interface` ·
  `a_saved_node_reinstantiates_with_fresh_ids_and_the_same_bits` ·
  `the_library_palette_lists_only_readable_files_and_says_why_not`.
- **Smoke:** «DominoWave» salvo, re-instanciado e dirigido só pela interface (doc 63 W-J).
- **Orçamento:** zero por quadro (é documento). **Contrato:** `NodeManifest` intocado — um nó
  próprio **não é um tipo do registry**; é um subgrafo com nome.

### W9 — FUSÃO DE KERNELS: uma cadeia por-elemento = UM passe (capacidade 8)
- **Censo (2026-09-04):** 70 kernels; **56** são mapas puros (`count_law: None`, sem `StreamOp`);
  12 mudam a contagem; `StreamOp` em 17 (`SourceRows` 7 · `Concat` 4 · `Compact` 4 · `Project` 2).
  A tee custa **3 passes**, o oscilador 2; o `motion.drive` que lê um `value.lfo` de comprimento 1
  já vira uniform em W1.
- **Regra de fusão** (em `plan.rs`, depois de `plan()`): estágios **consecutivos** na ordem
  topológica, cada um `count_law: None` e sem `StreamOp`, sem `pre`, sem `applicable` que varie,
  com **uma** aresta entre eles (o consumidor lê só o produtor) e a mesma contagem ⇒ um
  `FusedStage { members: Vec<GpuStage> }`.
- **Codegen** (`codegen.rs`): concatenar os corpos na ordem; os acessores `read_<col>(i)` /
  `write_<col>(i, v)` viram **variáveis locais** (`var c_<col>: <dim>`) carregadas uma vez do
  buffer de entrada e escritas uma vez no de saída no fim; `params.<name>` de cada membro
  prefixado (`p<k>_<name>`) num único uniform struct; `wgsl_lib` deduplicado por hash; presença e
  broadcast (`presence_signature`, `broadcast_mask`) calculados sobre a **união** das bindings.
  ⛔ Um membro cuja binding é `Consume` ou que lê uma coluna que o anterior **remove** quebra o
  grupo (a fusão é por leitura/escrita de colunas — a `ColumnAccess` é a régua).
- **Limites (medir em W0):** membros por grupo (tempo de compilação WGSL contra passes
  poupados) · `max_storage_buffers_per_shader_stage` (união das bindings) · tamanho do uniform.
- **`to_wgsl` ganha o primeiro consumidor:** o `motion.expression` (hoje `LoweringKind::Cpu`)
  passa a gerar o corpo com `ph2d_expr::to_wgsl(expr)` sobre os acessores — é o *compiled
  block* do Houdini e o *context* do VFX Graph, com o que nenhum dos dois tem: **paridade
  bit-a-bit com a CPU provada por gate**.
- **Gates:** `a_fused_chain_is_bit_identical_to_the_unfused_one` (o arnês de paridade, sobre as
  109 cenas — mutação: trocar a ordem dos membros) · `a_fusion_never_crosses_a_count_change` ·
  `the_tee_runs_in_one_pass` · `fusion_is_off_by_env_for_bisection` (`PH2D_GPU_FUSE=0`).
- **Smoke:** `PH2D_MOTION_ROUTE_LOG=1` numa cena de 10 modificadores: passes antes/depois.
- **Orçamento:** o objectivo é **passes ↓**, ms ↓ medidos pela sonda de W0; a régua honesta é a
  tabela das 109 cenas antes/depois, com `/proc/loadavg` ao lado.

### W10 — SPREADSHEET · SONDAS · INSPECÇÃO (capacidade 10 — doc 100 §7.4, doc 63 I7)
- **Balão do socket com as colunas** (`stream_at` existe; `tip_for_hot` é só do pino quente):
  `Out · a stream · P size rot falloff · 400 rows` — a espécie que DESCREVE (a que ACUSA foi
  recusada, handoff 03/09 §6).
- **Painel `ph2d-panel-motion-spreadsheet`:** instâncias × colunas do nó seleccionado, sobre
  `Cook::peek` (e o `gpu_tap` para a porta 0 no device); filtros; **escolher uma instância no
  canvas** (hit-test nas instâncias compostas) selecciona a linha — Houdini/Blender não ligam o
  clique no viewport à linha.
- **Sondas:** um nó de valor mostra o **sparkline** dos últimos 120 quadros no readout (doc 63
  «probe+sparkline ✓» — conferir se o sparkline existe; o readout existe).
- **Gates:** `the_socket_tip_lists_the_columns_the_stream_carries` ·
  `the_spreadsheet_shows_what_peek_holds` · `picking_an_instance_selects_its_row`.
- **Orçamento:** só com o painel aberto; **tablet:** painel dobrável, custo impresso nos 3 alvos.

---

