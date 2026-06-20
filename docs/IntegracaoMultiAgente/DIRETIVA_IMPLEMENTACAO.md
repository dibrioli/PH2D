# DIRETIVA DE IMPLEMENTAÇÃO — leia a CADA passo (curta de propósito)

> **Por que existe:** uma semana de "compila, passa, e nada funciona". A investigação forense
> (2026-06-16) achou 4 causas estruturais — costura nunca testada, "audit" virou "compilar",
> isolamento que fabrica fios órfãos, alvo irrefutável. Esta diretiva é o antídoto **operacional**.
> Não é opcional. Verde-de-compilação é sinal de VELOCIDADE; no fechamento/audit vale **ZERO**.

## 1 — Antes de codar
- [ ] Leia o **tracker único** do módulo (1 por módulo; o resto é histórico arquivado).
- [ ] Sua mudança cruza foundational / shell / outra crate? **O consumidor faz parte DESTE work item.**
      Proibido armar flag/evento órfão e "fiar depois" — é a causa nº 1 de feature morta (eyedropper, pills).
- [ ] Existe algoritmo de **referência publicado** (Krita Dulling, Curtis g/d, Mixbox, param do Procreate)?
      **Porte-o** antes de escrever a sua versão. Constante inventada (`D_MAX`, `COVER_K`, …) = **PARE**.

## 2 — Codando UI interativa (slider / botão / pincel): fie as pontas JUNTAS
A mesma feature atravessa, no mínimo, **8 sites**:

`id (editor-core/chrome.rs)` → `variante BrushParam (params.rs)` → `braço set_brush_param (lifecycle.rs)`
→ `campo do snapshot (params.rs)` → `register em populate.rs` → `paint em sections.rs`
→ `allowlist em event.rs` → `dispatch em trait_impls.rs`.

- [ ] Faltar **uma** ponta = clique dropado **em silêncio**, não erro de compilação. Fie todas no mesmo passo.
      Se uma tabela/macro (tipo `tool-sync`) consegue emitir as 8, **use-a** — é a correção definitiva.
- [ ] **Zero no-op silencioso.** Fora de escopo (ex.: `SelectBrush` sem library) = `debug_assert!` /
      `tracing::warn!` + UI mostra "desabilitado". Nunca um corpo vazio que "passa".
- [ ] **Gates de costura (lêem o FONTE, não compilam — falham no CI se faltar ponta):**
      `architecture_panel_wiring_parity` (todo id hit-indexado no paint está registrado em `populate.rs` → focável; a costura de DISPATCH do evento é provada pelo teste comportamental de seam, `ph2d-ui-testkit`).
      Âncora = **o site de pintura** (`cycler_row`/`pct_row` em sections.rs): **pintou ⟹ wirado**.
      Widget novo de tipo **sem gate** (ex.: um row interativo inédito) = **escreva o gate junto** —
      checklist em prosa NÃO morde (o bug do "Filter: não é clicável" tinha doc completa e mesmo assim passou).
- [ ] **Cycler/dropdown** = 2 registros que o compilador não cobra, ALÉM do slider-flow:
      `button(store, ids::X)` em **populate.rs** (hit-test) **E** `|| id == ids::X` em **`is_studio_button`/event.rs**
      (emitir o Click). Faltar qualquer um = botão pintado mas inerte. O gate de cycler prova os dois + o dispatch.

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

## 4 — Feature perceptual (marca / pintura) não fecha sem OLHAR
- [ ] Rode o **golden-image harness**: `begin_stroke → queue_pointer (arco determinístico) → end_stroke
      → assert pixels` — opacidade da linha central; azul-sobre-amarelo → verde no pigmento; taper;
      ripple/scallop por FFT ou max−min na espinha do traço.
- [ ] Testes GPU / `#[ignore]`: rode com `--include-ignored` e **registre** o resultado.
      Surface sem teste comportamental = **um achado**, não um pass.

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
