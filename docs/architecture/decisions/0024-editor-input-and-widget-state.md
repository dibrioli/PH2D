# ADR-0024: Editor input pipeline + retained widget state

**Status:** Accepted (2026-05-10 — Enio aprovou Modelo B com plano de conformidade HR-3 detalhado)
**Data:** 2026-05-10
**Decisor:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Origem:** M13 hero screen shipou 31 widgets + tela `02-editor-main.html` em paint estático puro. Enio observou: "por que os componentes ainda não são interativos visualmente?". Resposta curta: falta input pipeline conectado aos widgets. Esta ADR fixa o modelo antes de codar.
**Implementação:** rastreada em [`docs/plans/2026-05-editor-input-pipeline.md`](../../plans/2026-05-editor-input-pipeline.md).

## Contexto

O editor M13 tem:

- 31 widgets data-only com `state: ButtonState/SliderState/...` em cada struct (vide `crates/ph2d-editor/src/widget/`).
- Cada `paint_X(widget, rect, scene, theme)` desenha o estado declarado no struct — corretamente para todos os 7 estados de cada widget, conferido por 309 testes.
- AccessKit `Node` para todo widget interativo (HR-12), com `NodeId` estável por widget.
- Shell desktop (`shells/desktop/src/main.rs`) consome `PointerEvent` via `HostHandler::on_pointer` — atualmente serve só pra (a) BrushTool desenhar sprites, (b) `dragging: Option<NodeId>` ad-hoc para arrastar o Slider único do PanelControl ativo (`main.rs:389-400` — `find().filter(matches!(PanelControl::Slider))`).

**O gap:** ninguém computa transição `Normal → Hovered → Pressed → Focused → Disabled → Loading` quando o pointer mexe; nenhum widget dispara evento "click" / "value changed" pro shell. Cada `paint_button(&Button{...})` recebe um struct novo a cada frame — não há identidade entre frames pra carregar o state que o pointer pipeline computou.

Sem decidir o modelo, a tentação é replicar a gambiarra do Slider para os outros 30 widgets — vira um `match PanelControl::*` de 30 braços que não escala pra Inspector com 4 sliders + 1 select + 4 linked-inputs nem pra ContextMenu com 8 menu-items.

Esta ADR fixa o modelo de **(a) onde mora o state interativo**, **(b) como o input pipeline o atualiza**, **(c) como widgets emitem eventos pro caller**, antes de codar input handling.

## Em prosa, sem jargão

Existem dois modelos sérios em disputa:

**Modelo "redesenha tudo a cada frame" (imediato).** A cada quadro da tela, a engine roda uma função que pinta a UI inteira. Cada componente pintado *também* pergunta "o mouse está em mim? mudou?" e devolve a resposta no mesmo passo. Não existe estado guardado entre frames — tudo é redesenhado do zero, e quem decide hover/pressed é o componente olhando o mouse no instante. Código fica linear: pinta o slider, pergunta se mudou, aplica. Bonito de ler. Custo: trabalha o tempo todo, mesmo com mouse parado, porque cada frame refaz hit-test em todos os widgets. Estado vive como variável local da função paint, então leitor de tela e LLM via MCP precisam de uma estrutura paralela para consultar "qual o valor do slider X agora?".

**Modelo "estado guardado, eventos disparam mudanças" (retido).** Existe uma caixa central que sabe o estado de cada componente: "o slider Move Speed está em 0.62, sendo arrastado, focado". Quando o mouse mexe, um sistema captura o evento, descobre qual componente está ali, e atualiza a caixa. O paint depois apenas lê a caixa e desenha — fica "burro". A regra de negócio fica num handler separado que processa a fila de eventos. Mais espalhado de ler, mas separa render de lógica. Custo: só trabalha quando há evento de mouse/teclado. Mouse parado = zero trabalho de UI. Editor canvas-first onde o usuário passa a maior parte do tempo desenhando no canvas (não tocando UI) ganha bastante. LLM via MCP consulta a caixa direto.

**O cenário "atualização em tempo real antes do mouse soltar" funciona idêntico nos dois.** Não é onde diferem.

A escolha real é entre: **simplicidade de leitura linear** (imediato) versus **separação render/lógica + acessibilidade nativa + custo zero quando ocioso** (retido). Para PH2D — editor denso + canvas-first + agentes LLM consultando estado + uma codebase só para editor e jogo — o retido (Modelo B) é a aposta certa, *desde que* a implementação evite uma armadilha técnica: alocar memória no caminho quente. A próxima seção trata disso.

## Invariantes (qualquer modelo precisa atender)

Independente do modelo escolhido:

1. **HR-12 a11y persiste.** AccessKit `NodeId` é a identidade canônica do widget; focus chain vive no AccessKit `Tree` ou em estrutura espelhada por `NodeId`.
2. **HR-3 zero allocs no hot path.** Paint pass roda 60Hz mínimo; input dispatch a cada `PointerEvent` (potencialmente 100+/s no Pencil). Nenhum modelo pode alocar `String`/`Vec` por evento.
3. **HR-7 editor é a engine.** Modelo de input precisa funcionar tanto em modo "editor" quanto em modo "jogo publicado" (cfg(feature = "editor") corta editor-only widgets, mas o resto continua interativo).
4. **WCAG 2.1.1 — keyboard accessible.** Toda interação clicável precisa ter equivalente keyboard (Tab nav + Enter/Space). Modelo precisa carregar `focused_id` global.
5. **Canvas-first (ADR-0023 §2).** Interação UI não pode capturar TODO pointer event — canvas continua recebendo Pencil quando widget não claim. "Hit-test fallthrough" obrigatório.
6. **Determinismo onde prometido (HR-2).** Input que entra na simulação (ex: clicar "play" → reset world) precisa ser reproduzível. Modelo precisa permitir log+replay determinístico de eventos.
7. **MCP-queryable.** LLM via `ph2d-mcp` precisa poder consultar `widget_state(NodeId) -> {hovered, focused, value, ...}` sem reconstruir o widget — necessário pra agente fazer assertions.

## Modelos considerados

### Modelo A — Imediato puro (egui-style)

Paint helpers ganham retorno: `paint_button(&mut Ui, label, rect) -> ButtonResponse { clicked, hovered, focused }`. Todo state interativo é re-computado a cada frame a partir de `&mut Ui { pointer, modifiers, focus_id, hot_id, active_id }`. Caller escreve:

```rust
if paint_button(&mut ui, "Save", rect).clicked {
    save_project();
}
```

Nada vive entre frames além de `Ui::focus_id`/`Ui::active_id` (3 NodeIds totais).

**Pros:**
- Código de chamada legível, próximo do mental model "imediato".
- Sem store global → sem sync paint↔state.
- Egui validou o modelo em produção (Rerun, eframe).

**Cons:**
- Quebra o pattern `data + paint helper` que estabelecemos em M13. Widgets viram funções com side-effect, não dados.
- A11y AccessKit força nodes persistentes; precisa de bridge `Ui → AccessKit Tree` que diff-a a cada frame — possível mas é trabalho extra.
- LLM via MCP perde "query state of widget X" porque widgets não persistem como dados — só como side-effects de paint.
- Migração custosa: 31 widgets atuais precisam ganhar variants `paint_X` que retornam Response, e os structs viram parâmetros de função em vez de dados.
- Foi explicitamente rejeitado em comit 4eda076 (vide MEMORY: "egui PR #29 abandonada por bug font texture + perf").

### Modelo B — Retained com WidgetStore externo

`crate::interaction::WidgetStore` é um `BTreeMap<NodeId, InteractiveState>` central onde mora hover/press/focus/drag/value de cada widget. Pre-paint, o shell roda `dispatch_pointer(&mut store, &hit_index, event)` que atualiza state. Paint helpers ficam puros: `paint_button(&Button { state: store.get(id) }, ...)`.

```rust
// shell desktop pseudocode
input.apply_pointer(event);
let hit = hit_index.find(event.pos);
store.dispatch(hit, event); // mutates state per-widget
let events = store.drain_emitted(); // [Click(NodeId), ValueChanged(NodeId, f32), ...]
for event in events { dispatch_to_app(event); }
// then paint pass — paint_button reads store.get(id) for state
```

`hit_index: BTreeMap<NodeId, Rect>` é construído pelo paint pass anterior (write-during-paint, read-on-input).

**Pros:**
- Cumpre todas as invariantes (1-7) sem ginástica.
- AccessKit `Tree` e `WidgetStore` compartilham `NodeId` — focus chain trivial.
- LLM-MCP query: `mcp_query_widget(id) -> store.get(id).snapshot()`.
- Paint helpers continuam puros (dados → pixels). Pattern M13 preservado.
- Determinismo: store é estado puro, replay de eventos = replay de state.
- Ronaldo (CCEC), AccessKit official examples, Slint usam variantes desse padrão.

**Cons:**
- Precisa hit-test index atualizado a cada frame (custo: ~31 inserts/frame em BTreeMap, trivial mas precisa lembrar).
- Caller-de-widget precisa lembrar de chamar `store.register(id, rect)` no paint — esquecido = widget invisível pro input. Mitigação: helper macro ou trait `Interactive` com método único `paint_and_register`.
- 1-frame de latência: hit-rect só vira observável depois do primeiro paint. Para um widget que aparece e é clicável no MESMO frame (ex: ContextMenu recém-aberto), precisa pré-popular o store com o rect projetado.

### Modelo C — Híbrido: state no struct + InteractionContext

Widgets continuam ownando seu state (`Button.state`). Caller chama `update_button(&mut button, &mut ictx, rect)` ANTES do paint, onde `ictx` carrega pointer, modifiers, focus_id, e devolve eventos via `Vec<WidgetEvent>` no contexto.

```rust
let mut button = Button::new(NodeId(1), "Save");
update_button(&mut button, &mut ictx, rect);
paint_button(&button, rect, scene, theme);
// later:
for event in ictx.drain_events() { ... }
```

**Pros:**
- State vive onde já vive (no struct).
- Update e paint são funções separadas, fáceis de raciocinar isoladas.
- Nenhum store global — modular por widget.

**Cons:**
- Caller precisa chamar `update_X()` antes de TODO `paint_X()` — fácil esquecer (silent bug: widget renderiza mas não responde).
- 31 widgets × 2 funções vs 31 × 1 = mais surface area pra manter.
- State persistente (ex: dropdown.open) força caller a manter widgets entre frames como dados, ou seja, materializa `WidgetStore` informalmente — só que descentralizado e sem tipo único.
- LLM-MCP query: precisa enumerar todos widgets ativos pra achar `NodeId`; sem store, sem índice.
- FloatingPanel arrastável precisa que caller mantenha a posição do panel entre frames — já fazemos isso, mas multiplicar essa pattern por 31 widgets = bagunça.

## Plano de conformidade HR-3 (zero alloc no caminho quente)

A escolha do Modelo B vem com um risco técnico real: as estruturas que esbocei (`BTreeMap<NodeId, InteractiveState>`, `Vec<WidgetEvent>`, `Vec<(NodeId, Rect)>`) **podem alocar a cada frame** se implementadas ingenuamente. `editor_layout` é hot path declarado (HR-3, linha 260 da SKILL); o bench `tests/budget/no_alloc_hot_path.rs` falha se contar > 0 alocações em 10 frames sintéticos. O modelo precisa atender HR-3 **por construção**, não por sorte.

Quatro mitigações combinadas garantem zero alloc por frame:

1. **WidgetStore pré-populado, nunca insere on-demand.** A construção da tela (ex: `HeroScreen::new`) percorre todo widget conhecido e chama `store.register(NodeId, default_state)` uma única vez. O paint pass nunca chama `store.insert` — apenas `store.get_mut(id)` que opera em entrada já alocada. Dispatch de evento idem: muta entrada existente. Substituir `BTreeMap` por `slotmap::SlotMap<NodeId, InteractiveState>` (capacidade fixa ao construir) elimina inserts em árvore B durante o frame.

2. **HitIndex usa SmallVec inline.** Em vez de `Vec<(NodeId, Rect)>` que realoca, usar `SmallVec<[(NodeId, Rect); 128]>` — 128 widgets cabem inline na pilha (cobre folgado os ~31 da hero + crescimento futuro pra ~80). Estouros caem pra heap mas são raros e visíveis no profiler. `clear()` por frame é grátis (não desaloca).

3. **Eventos via arena bumpalo resetada por frame.** A fila `emitted: Vec<WidgetEvent>` substituída por `&'frame bumpalo::Bump` — cada evento é alocado na arena, e a arena inteira é zerada (`bump.reset()`) no fim do frame. Custo de "alocar evento" vira incremento de pointer — uma instrução. Caller drena os eventos no mesmo frame antes do reset; nada vive entre frames.

4. **Strings de eventos evitadas no caminho quente.** `WidgetEvent::TextChanged(NodeId, String)` aloca `String` por keystroke. Substituir por `TextChanged(NodeId, &'frame str)` (slice na arena bumpalo) ou por evento sem payload (`TextChanged(NodeId)` + caller lê `store.text(id)`). Idem para `Click` que não precisa de payload algum.

Verificação: o bench HR-3 (`tests/budget/no_alloc_hot_path.rs`) ganha caso novo `interaction_dispatch_no_alloc` que simula 10 frames com 30 widgets recebendo PointerEvents variados; falha CI se `dhat-rs` contar qualquer alocação fora da fase de construção.

Sem essas 4 mitigações, Modelo B viola HR-3 silenciosamente (testes funcionais passam, profiler mostra jitter). Com elas, Modelo B alcança o mesmo patamar de zero-alloc do Modelo A — pagando por isso ~150 linhas de infraestrutura extra (definição do SlotMap, integração da arena, helpers de registro).

## Recomendação

**Modelo B (Retained com WidgetStore externo).**

Razões em ordem de peso:
1. **Cumpre as 7 invariantes sem hack.** A/C precisam ginástica em pelo menos 2 invariantes cada (A em a11y+MCP+determinismo; C em MCP+determinismo+canvas-fallthrough).
2. **Preserva o pattern data+paint do M13.** 309 testes validam que `paint_X(&data, rect, theme)` é a abstração certa pra render. Modelo B mantém — só adiciona `data.state` lookup numa store externa em vez de no struct local.
3. **AccessKit já é retained.** O `Tree` espelha state interativo de toda forma; ter `WidgetStore` paralelo é só compartilhar identidade, não duplicar.
4. **MCP-friendly.** `mcp_query_widget_state(NodeId)` vira 1 lookup. LLM via agentes pode escrever testes que afirmam "depois de clicar `save`, o `notification_toast` está visible" sem mockar render.
5. **Custo de implementação concreto.** Estimativa: ~12-15h pra (a) `WidgetStore` + `HitIndex` + dispatcher (~4h), (b) wire pra Button/Slider/Toggle/Checkbox primeiro (~3h), (c) wire para os 27 restantes via trait `Interactive` ou bulk PR (~5h), (d) keyboard nav + Tab focus (~3h).

Modelo A foi tentado e abandonado (egui PR #29) por outras razões (font texture bug, perf), mas mesmo se essas estivessem resolvidas, o conflito com a11y/MCP/our-pattern faria voltar a Modelo B eventualmente.

Modelo C é tentador para projetos pequenos; em editor com 31 widgets + Inspector embedando 8 sliders + ContextMenu com 12 menu-items, o ônus de "lembrar de chamar update_X antes de paint_X" vira fonte de bug crônico.

## Esboço de implementação (se Modelo B aprovado)

Novo crate ou módulo: `crates/ph2d-editor/src/interaction/` com:

```rust
// interaction/state.rs — pré-populado na construção da tela; nunca insere on-demand.
pub struct WidgetStore {
    states: SlotMap<NodeId, InteractiveState>, // capacidade fixa ao construir
    focus_id: Option<NodeId>,
    hot_id: Option<NodeId>,    // hovered
    active_id: Option<NodeId>, // pressed/dragging
}

pub enum InteractiveState {
    Button { state: ButtonState },
    Slider { state: SliderState, value: f32 },
    Toggle { state: ToggleState, on: bool },
    TextInput { state: TextInputState, text: String, caret: usize },
    // ... one variant per widget kind
}

// Eventos sem payload pesado: caller relê do store quando precisa de valor.
pub enum WidgetEvent {
    Click(NodeId),
    ValueChanged(NodeId),  // caller lê store.value(id)
    Toggled(NodeId),
    TextChanged(NodeId),
    Focus(NodeId),
    Blur(NodeId),
}

// interaction/hit.rs — SmallVec inline cobre os ~31 widgets sem heap.
pub struct HitIndex {
    /// Painted in z-order; iteration is back-to-front so first match wins.
    rects: SmallVec<[(NodeId, Rect); 128]>,
}

impl HitIndex {
    pub fn clear_for_frame(&mut self); // cheap; doesn't deallocate
    pub fn register(&mut self, id: NodeId, rect: Rect);
    pub fn hit(&self, x: f32, y: f32) -> Option<NodeId>;
}

// interaction/dispatch.rs — eventos vivem em arena resetada por frame.
pub fn dispatch_pointer<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent];
pub fn dispatch_key<'frame>(
    store: &mut WidgetStore,
    event: KeyEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent];
```

Migration: cada `paint_X(&Widget, rect, ...)` ganha um par adjacente `paint_X_into(&mut HitIndex, ...)` que registra o rect (ou um wrapper `paint_and_track`). Widgets continuam estruturas data-only; `Widget::state` vira getter convenient sobre `store.get(self.id).state`.

PRs incrementais sugeridos:
- PR-A: `interaction` module com SlotMap+SmallVec+arena bumpalo + bench HR-3 + Button/Toggle wiring (~5h).
- PR-B: Slider/RadioGroup/Checkbox + drag handling (~4h).
- PR-C: TextInput/NumberInput/Combobox + keyboard focus chain (~5h).
- PR-D: TreeView/ContextMenu/ColorPicker (~3h).

PR-A é o mais crítico: estabelece a infraestrutura zero-alloc que os PRs seguintes herdam. Se o bench HR-3 falhar nesse PR, **paramos e revisamos** antes de seguir — não vale carregar dívida de alocação para os 27 widgets restantes.

## Decisão pendente

Enio precisa escolher: **A**, **B** (recomendado), **C**, ou pedir refinamento de algum modelo / propor um quarto.

Se escolher B: implementação começa por PR-A após esta ADR ser marcada Accepted.

Se escolher A ou C: re-escrever a ADR com tradeoffs invertidos antes de codar.
