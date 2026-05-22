# ADR-0030 — Multi-domain node engine: substrato unificado + avaliadores plurais + membrana como tipo

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto), com review de 4 rodadas de opinião independente (paisagem competitiva · UX de artista · teoria/first-principles dataflow · resolução grafo-único-vs-contextos).
**Habilita:** ADR-0031..0038. É a decisão-mãe da virada node-centric.
**Spec completa:** [`docs/Migracao/2026-05-node-centric-architecture.md`](../../Migracao/2026-05-node-centric-architecture.md).

> **Nota de numeração:** o ADR-0029 §closeout/§12 fez forward-reference a um "ADR-0030" para o carve-out do `PanelHost` público + deleção do shim `ph2d-editor` (~6 meses pós-merge). Aquele trabalho **renumera para ADR-0039+** quando acontecer — 0030..0038 ficam alocados à arquitetura de nós por decisão do Enio em 2026-05-21.

## 1. Contexto

O Enio decidiu que a PH2D passa a ser construída em torno de um **sistema de nós multi-domínio** (shaders, motion, programming/gameplay, sound), no modelo Houdini/Unreal/Blender, mais ferramentas imperativas fora do grafo. A questão arquitetural central: **um grafo único heterogêneo** (posição teórica: tipos de porta algébricos, contextos = regiões) **vs editores/contextos tipados separados** (posição de mercado: Houdini SOP/COP/CHOP/DOP)?

A investigação mostrou que a tensão é **majoritariamente falsa**: falsa no nível de **dados**, real no nível de **avaliação**. Os três sistemas de referência (Houdini, TouchDesigner, Blender) já são uma reconciliação — substrato compartilhado por baixo, famílias tipadas por cima — e Blender é o corte mais nítido: framework unificado (socket-geometria único, fields), mas shader-tree / geometry-tree / compositor **separados porque o alvo de avaliação difere**.

## 2. Decisão

**Substrato unificado + avaliadores plurais.** A linha de corte:

> **DADOS + CONTRATO = unificado. AVALIAÇÃO (relógio + alvo de compilação + classe de efeito) = plural e tipado.**

- **UNIFICADO** (`ph2d-nodegraph`, ADR-0032): modelo de atributos, tipos de porta algébricos que carregam **domínio + dimensionalidade + relógio**, sistema de efeitos `Pure/Temporal/Stateful`, delay `pre` + aciclicidade-por-construção, formato textual diffável, registry de portas; e o compute compartilhado (`ph2d-expr`, ADR-0033).
- **PLURAL** (ADR-0034): avaliadores por modelo-de-avaliação (shader `pull→WGSL`, áudio `synchronous-dataflow` relógio-fixo, motion `pull-no-playhead`, gameplay `push→Luau`) + views/editores por domínio, com **contexto escondido do artista** (ADR-0038).

**Membrana mão-única CHECADA POR TIPO** (ADR-0021 elevado a regra de grafo): só o domínio gameplay escreve o `SimWorld` via porta de export designada → **só ele exige HR-5 determinismo**; motion/shader/sound só leem o `PresentWorld` (efeito `Pure`/`Temporal`) → **isentos de HR-5**, como Radiance Cascades. Enforcement = arch-gate que recusa nó `Stateful` referenciado no lado pull.

**Regra de decisão porta-vs-domínio:** "dois nós deste tipo, ligados, cozinham no mesmo agendador **sem mudar relógio nem alvo de compilação**?" Sim → porta tipada na mesma região. Não → domínios separados, conexão = travessia de membrana tipada (com `pre`/conversão de clock), nunca canvas comum nem import-node ad-hoc.

## 3. Consequências

**Aceitas:**
- Um motor de cook genérico reusado por todos os domínios (não 4 engines divergentes).
- A composabilidade cross-domínio (força da posição teórica) sobrevive **sem** o canvas-sopa (defeito dela), porque a ponte é um tipo de porta, não um nó especial.
- A navegabilidade tipada (força da posição de mercado) sobrevive **sem** o conversor O(n²) do TouchDesigner.
- A membrana vira invariante de compilação + arch-gate (encaixa na cultura de pre-commit existente).

**Riscos:**
- Substrato `ph2d-nodegraph` instável vira god-crate e mata o paralelismo → capar superfície; mudança de substrato é evento raro Coordenador-only.
- Over-abstração dos avaliadores → cada avaliador é um crate fino sobre backend existente (WGSL/Vello/audio/Luau).

## 4. Alternativas consideradas

- **Posição 1 pura (grafo único, um canvas):** rejeitada. Mata navegabilidade (tab-menu-explosion comprovado no Houdini) e torna a membrana invisível até o type-error.
- **Posição 2 pura (contextos/motores totalmente separados):** rejeitada. Mata composabilidade cross-domínio (conversor O(n²)), fragmenta o compute reusável em N cópias divergentes (pior modo de falha multi-agente) e nega formato/save único.
- **Copiar Houdini integralmente:** rejeitada. Herdaria curva de aprendizado + modelo offline sem ganhar real-time nem segurança de gameplay. Houdini é referência de poder, não de design final.
