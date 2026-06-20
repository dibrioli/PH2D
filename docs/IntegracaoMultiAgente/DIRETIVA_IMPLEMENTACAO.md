# DIRETIVA DE IMPLEMENTAÇÃO — leia a CADA passo (curta de propósito)

> **Por que existe:** uma semana de "compila, passa, e nada funciona". A investigação forense
> (2026-06-16) achou 4 causas estruturais — costura nunca testada, "audit" virou "compilar",
> isolamento que fabrica fios órfãos, alvo irrefutável. Esta diretiva é o antídoto **operacional**.
> Não é opcional. Verde-de-compilação é sinal de VELOCIDADE; no fechamento/audit vale **ZERO**.

## 1 — Antes de codar
- [ ] Leia o **tracker único** do módulo (1 por módulo; o resto é histórico arquivado).
- [ ] Sua mudança cruza foundational / shell / outra crate? **O consumidor faz parte DESTE work item.**
      Proibido armar flag/evento órfão e "fiar depois" — é a causa nº 1 de feature morta (eyedropper, pills).
- [ ] Existe algoritmo de **referência publicado** (transfer sRGB/OKLab, math canônica de blend-mode,
      geometria kurbo/vello, K–M/Mixbox onde houver pigmento)? **Porte-o** antes de escrever a sua versão.
      Constante de magia inventada (`*_MAX`, `*_K`, fator solto) = **PARE** e ache a fonte.

## 2 — Codando UI interativa (slider / botão / cycler): fie as pontas JUNTAS
Um controle interativo atravessa o **seam painel↔tool**, no mínimo **7 sites**:

`id (ph2d-editor-core/src/ids/…)` → `register em populate.rs/seam.rs (vira focável)`
→ `paint + hit-index em paint*.rs` → `emite o evento em event.rs/seam.rs`
→ `EditorAction::ToolPanelEvent (bus)` → `tool.handle_panel_event` → `apply_ui_edit (muda o spec/estado)`.

- [ ] Faltar **uma** ponta = clique dropado **em silêncio**, não erro de compilação. Fie todas no mesmo passo.
      Se uma macro (`panel_seam!`, abaixo) consegue emitir as pontas do lado do painel, **use-a** — é a correção definitiva.
- [ ] **Painel forwarder novo (slider+chip / botão → tool): use `ph2d_editor_core::panel_seam!`**
      (Fase 2). Ele gera `populate` + `apply_event` de UMA declaração — registrar e forwardear
      saem JUNTOS, então um widget registrado não pode ficar sem arm. Referência:
      [`ph2d-panel-padding/src/seam.rs`](../../crates/ph2d-panel-padding/src/seam.rs). **Escopo:** só
      a forma forwarder (slider+chip, forward-button, cancel-button). Painel com dropdown / toggle /
      picker / dispatch indireto (grid-snap, inspector, vector, painter-layers) fica com `event.rs`
      explícito — não dobre o macro pra encaixar; as gates `architecture_panel_wiring_parity` +
      o seam test guardam os dois estilos. **A parte do TOOL** (`UiEdit`/`apply_ui_edit`/`handle_panel_event`/
      snapshot) continua escrita à mão (lógica bespoke por-tool, não boilerplate).
- [ ] **Zero no-op silencioso.** Caminho fora de escopo / pré-condição ausente = `debug_assert!` /
      `tracing::warn!` + UI mostra "desabilitado". Nunca um corpo vazio que "passa".
- [ ] **Gates de costura (lêem o FONTE, não compilam — falham no CI se faltar ponta):**
      `architecture_panel_wiring_parity` (todo id hit-indexado no paint está registrado em `populate.rs`/`seam.rs` → focável; a costura de DISPATCH do evento é provada pelo teste comportamental de seam, `ph2d-ui-testkit`).
      Âncora = **o site de pintura** (onde o widget é hit-indexado): **pintou ⟹ wirado**.
      Widget novo de tipo **sem gate** (ex.: um row interativo inédito) = **escreva o gate junto** —
      checklist em prosa NÃO morde (o bug do "Filter: não é clicável" tinha doc completa e mesmo assim passou).
- [ ] **Cycler/dropdown** = registros que o compilador não cobra, ALÉM do slider-flow: hit-index +
      `store.register` em **populate.rs** **E** o arm que emite o Click em **event.rs**, com a tabela
      id→opção resolvida no paint (padrão vivo: `kind_option_ids_in_order` em
      [`ph2d-panel-grid-snap/src/paint_helpers.rs`](../../crates/ph2d-panel-grid-snap/src/paint_helpers.rs) +
      dispatch em [`event.rs`](../../crates/ph2d-panel-grid-snap/src/event.rs)). Faltar qualquer um =
      botão pintado mas inerte; o seam test (`ph2d-ui-testkit`) que dirige o Click prova o dispatch.

## 3 — Auditar ≠ compilar (a regra que o Enio cobrou)
- [ ] `cargo check -p` e os gates `architecture_*` (inclusive `*_contract_surface`) = forma-de-ABI /
      velocidade. No audit valem **ZERO**: são contadores de símbolo; um bug lógico que mantém a
      contagem passa todos eles. **Auditoria que rodou cargo/gates e concluiu "compila, OK" é
      REJEITADA por definição** — é o exato erro que custou semanas (vector pills CI-verdes-e-mortas).
- [ ] **TEMPLATE OBRIGATÓRIO — preencha por lente/claim. Sem isto, não é audit:**
      ```
      LENTE:  <correção | wiring | perf | determinismo | …>
      CLAIM:  <a propriedade que afirmo estar correta>
      TRAÇO:  <caminho fim-a-fim com file:line — entrada → … → efeito observável>
      ASSERÇÃO-VERMELHA: <o teste executável que ficaria VERMELHO se a propriedade quebrasse>
              (seam de UI: um teste `ph2d-ui-testkit` que DIRIGE o evento real e afirma o efeito
               observável; se não existe, ESCREVA-O — o teste é o entregável, não o veredito)
      NÃO-CHECADO-PELA-COMPILAÇÃO: <comportamentos que o build/gates não pegam>
      LOC LIDAS: <nº de linhas que de fato li>
      ```
- [ ] 0 file:line + veredito "nenhum bug" = **não-audit**, rejeitado. Claim verde sem
      ASSERÇÃO-VERMELHA correspondente = **não-audit**, rejeitado.

## 4 — Efeito perceptual (ajuste / blend / filtro / compositor) não fecha sem OLHAR
- [ ] Prove **paridade numérica** contra a referência canônica, não "parece certo": o caminho GPU bate
      o CPU **bit-a-bit** (ex.: `shader_adjustment_coefficients_bit_identical_with_rust` em
      [`ph2d-render/src/layer_compositor/tests.rs`](../../crates/ph2d-render/src/layer_compositor/tests.rs);
      blend-modes em [`ph2d-painter-effects/src/blend.rs`](../../crates/ph2d-painter-effects/src/blend.rs)).
      Efeito espacial (bloom / shadows-highlights) reconcilia o kernel contra a fn CPU canônica.
- [ ] Testes GPU `#[ignore]` (headless Metal — rodam no sandbox): rode com `-- --ignored` e **registre**
      o resultado. Kernel/efeito **sem** teste de paridade = **um achado**, não um pass.

## 5 — Antes de marcar FECHADO
- [ ] **DoD (definição de pronto): teste comportamental de seam VERDE (`ph2d-ui-testkit`:
      evento real → efeito observável) + smoke do Enio.** Compile-verde e gate-verde **NÃO** são
      "pronto" — são velocidade (§3). Painel interativo sem seam test é barrado pela gate
      `architecture_interactive_crate_has_behavioral_test` (e a dívida vive em
      `BEHAVIORAL_TEST_DEBT`, drive-to-zero — não é exceção permanente).
- [ ] Grep dos ids nos 8 sites. Grep `pub struct <Tipo>` — **um dono só** (não recrie modelo paralelo).
- [ ] **Não entregue incremento que o Enio precise QA na mão pra descobrir que está morto.**
      DEFER **nomeia** a capacidade exata faltante + abre handoff + **não conta** como fechamento.
      Veredito é **condicional** ("APPROVE pending smoke S1") até o smoke manual voltar.
- [ ] Alvo irrefutável ("paridade", "o melhor") **não é** done-definition. ADR ambicioso declara
      conjunto de aceitação **concreto/congelado** + **kill-criterion ANTES do build**
      ("se perf > X ms @4K após a tentativa 2, a feature não existe nesta forma").
      Bateu na **2ª reconstrução de topologia** → **PARE e prove o modelo** antes da 3ª (regra two-strikes).
