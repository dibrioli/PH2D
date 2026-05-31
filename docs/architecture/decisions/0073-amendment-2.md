# ADR-0073-amendment-2 — Y-Sort self-inclusive (o próprio sprite participa, não só via ancestral)

**Status:** Accepted (sorting audit 2026-05-31) — fix implementado, golden intacto, repro test verde.
**Amends:** [ADR-0073 — Sorting canonical order](0073-sorting-canonical-order.md) §2 (estágio YSort) + [amendment-1](0073-amendment-1.md) (mantém Z-bucketiza-antes).
**Spec section refined:** `docs/Sprite_projeto/05_ordering_sorting.md` §5.2 passo-3 (semântica de cascata YSort).
**Reference:** [`crates/ph2d-ecs/src/sort_key.rs`](../../../crates/ph2d-ecs/src/sort_key.rs) (`ysort_key`), [`crates/ph2d-ecs/tests/ysort_direction_and_root_repro.rs`](../../../crates/ph2d-ecs/tests/ysort_direction_and_root_repro.rs).

---

## 1. Context — Y-Sort era inerte no caso mais comum do editor

A semântica original (espelho do Godot `y_sort_enabled`) era **só-ancestral**: `ysort_key` só
projetava um sprite por Y quando um **ancestral estrito** (subindo `ChildOf` até uma fronteira
[`TopLevel`]) carregava `YSort { enabled: true, .. }`. O nó-pai ordena os filhos; o próprio nó
nunca consulta o seu próprio `YSort`.

No editor PH2D, porém, os sprites nascem **planos** — `image_import.rs` spawna `(Transform,
Sprite, Name)` sem `ChildOf`, então cada sprite é uma **raiz**. E o toggle "Y-Sort" do Inspector
(§7 Ordering) escreve o componente **no próprio sprite selecionado**. Resultado: ligar Y-Sort
numa raiz não tinha ancestral com YSort → `ysort_key = 0` → **nenhum efeito visível**. A feature
parecia quebrada ("ysort não faz nada / parece invertido") quando na verdade estava inerte.

Diagnóstico empírico (auditoria 2026-05-31): a *direção* já estava correta — com `axis (0,-1)` num
mundo Y-up (`camera.rs`), um ancestral YSort coloca o sprite **mais baixo na tela na frente**
(confirmado por 3 lentes adversariais + teste `ysort_parent_lower_on_screen_draws_front`). O bug
era exclusivamente a inacessibilidade da feature para sprites-raiz.

## 2. Decision

**Y-Sort é self-inclusive.** `ysort_key` passa a checar o `YSort { enabled: true }` do **próprio
entity primeiro**; só na ausência dele cai na busca por ancestral (cascata Godot preservada). Um
sprite-raiz com Y-Sort ligado projeta a própria posição e participa do Y-sort entre seus irmãos
(raízes incluídas).

- A checagem-de-self vem **antes** do early-return de `TopLevel`: `TopLevel` governa só a herança
  *de cascata*, não o opt-in explícito do próprio nó.
- A cascata por ancestral (§5.2 passo-3, Godot) é mantida inalterada como fallback.
- Mudança **puramente aditiva**: o golden canônico (`sorting_pipeline_determinism.rs`) tem YSort
  num nó-pai não-sprite e os filhos não têm self-YSort → a checagem-de-self é no-op lá → ordem
  golden `[0,7,6,1,3,2,4,5,8,9]` **inalterada** (gate verde).

## 3. Consequences

- O toggle per-sprite do Inspector §7 finalmente faz o que o usuário espera: liga, o sprite ordena
  por Y entre os demais com Y-Sort ligado; mais baixo na tela = na frente.
- Sprites sem Y-Sort ficam neutros (`ysort = 0`) e bucketizam relativos aos participantes na linha
  `proj = 0` — comportamento idêntico ao de um irmão fora de um grupo YSort.
- Continua compatível com a cascata Godot (pai ordena filhos) para cenas hierárquicas.
- Edge de dados: um `YSort` serializado antes do flip de `axis` (0,+1) → (0,-1) preserva o eixo
  antigo ao religar (o handler `..cur_ysort()` mantém campos do usuário). Um componente novo já
  nasce com o default correto; migração de cenas antigas é fora de escopo deste fix.

## 4. Provenance

Auditoria completa do sistema de sorting (Enio, 2026-05-31): "depois de usar o y-sort a ordem z da
hierarquia parece quebrada e o sprite mais abaixo na tela fica mais no fundo". A investigação
isolou direção-correta + inacessibilidade-em-raiz; Enio escolheu a semântica **self-inclusive**
(vs. raiz-de-cena implícita / só-UI). Reproduções em `ysort_direction_and_root_repro.rs`.
