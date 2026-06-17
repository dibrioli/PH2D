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

## 3 — Auditar ≠ compilar (a regra que o Enio cobrou)
- [ ] `cargo check -p` e os gates `architecture_*_contract_surface` = forma-de-ABI / velocidade. No audit
      valem **ZERO**: são contadores de símbolo; um bug lógico que mantém a contagem passa todos eles.
- [ ] Um audit **só conta** se produzir, por lente:
      **(a)** ≥1 caminho traçado **fim-a-fim com file:line**;
      **(b)** a lista de comportamentos que a compilação **não** checa;
      **(c)** para cada "verde", a **asserção executável que ficaria VERMELHA** se a propriedade quebrasse;
      **(d)** quantas LOC você de fato **leu**.
- [ ] 0 file:line + veredito "nenhum bug" = **não-audit**, rejeitado.

## 4 — Feature perceptual (marca / pintura) não fecha sem OLHAR
- [ ] Rode o **golden-image harness**: `begin_stroke → queue_pointer (arco determinístico) → end_stroke
      → assert pixels` — opacidade da linha central; azul-sobre-amarelo → verde no pigmento; taper;
      ripple/scallop por FFT ou max−min na espinha do traço.
- [ ] Testes GPU / `#[ignore]`: rode com `--include-ignored` e **registre** o resultado.
      Surface sem teste comportamental = **um achado**, não um pass.

## 5 — Antes de marcar FECHADO
- [ ] Grep dos ids nos 8 sites. Grep `pub struct <Tipo>` — **um dono só** (não recrie modelo paralelo).
- [ ] **Não entregue incremento que o Enio precise QA na mão pra descobrir que está morto.**
      DEFER **nomeia** a capacidade exata faltante + abre handoff + **não conta** como fechamento.
      Veredito é **condicional** ("APPROVE pending smoke S1") até o smoke manual voltar.
- [ ] Alvo irrefutável ("paridade", "o melhor") **não é** done-definition. ADR ambicioso declara
      conjunto de aceitação **concreto/congelado** + **kill-criterion ANTES do build**
      ("se perf > X ms @4K após a tentativa 2, a feature não existe nesta forma").
      Bateu na **2ª reconstrução de topologia** → **PARE e prove o modelo** antes da 3ª (regra two-strikes).
