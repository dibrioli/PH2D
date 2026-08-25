# AUDITORIA DE FECHO — `line/seamelim` (2026-08-24)

> Template obrigatório da [`DIRETIVA_IMPLEMENTACAO.md` §3](../../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).
> ⛔ *Compilar e ver gate verde vale **zero** aqui — são contadores de símbolo.*

---

## LENTE 1 — CORREÇÃO: a lei é de facto imposta, e não só afirmada

**CLAIM:** depois da eliminação, os dois lados de uma costura **não conseguem**
discordar — não existe caminho de execução que os afaste, e o que sobra ao medir é o
erro de avaliação da substituição em `f32`.

**TRAÇO (fim a fim):**
`weld::weld` ([`weld.rs:284`](../../../crates/ph2d-gridmap/src/weld.rs)) monta a floresta
de derivação → `Weld::derive` ([`weld.rs:326`](../../../crates/ph2d-gridmap/src/weld.rs))
**escreve** a cópia pela transição, em vez de a relaxar →
`weld_solve::WeldRelaxer::relax_class` só toca **raízes** →
`weld_flat::ClosureSystem::apply` escreve as dependentes →
`corner_map` ([`corners.rs:24`](../../../crates/ph2d-gridmap/src/corners.rs)) entrega o
mapa por canto → `ph2d_quadextract::ingest::derive_transitions` **re-deriva** as
transições dali e mede `ExtractReport::shift_residual`.

**ASSERÇÃO-VERMELHA:**
`our_welded_map_closes_its_seams_at_the_floor_of_f32`
([`gate_seam_closes.rs`](../../../crates/ph2d-quadextract/tests/gate_seam_closes.rs)) —
mede **no fim da cadeia**, com a barra lida da referência pelo mesmo verificador.
**Provada por mutação, duas vezes:**

| mutação | resultado |
|---|---|
| `Weld::derive` deixa de escrever (eliminação inerte) | ⭐ VERMELHO, resíduo `1,86e10` |
| `ClosureSystem::build` não elimina nada (só os fechos) | ⭐ VERMELHO **cirúrgico**: as eliminadas ficam em `2,38e-7` e só os fechos rebentam (`7,07` / `10,20`) |

⭐ **A segunda é a que vale:** ela mostra que o gate **separa as duas metades** e não é um
detector de «alguma coisa partiu».

**NÃO-CHECADO-PELA-COMPILAÇÃO:**
- que a floresta de derivação seja um *spanning forest* (não uma árvore com ciclo
  escondido) — ⇒ `every_seam_link_is_either_eliminated_or_a_closure`, e **foi ele que
  achou** o defeito da recontagem por sentido;
- que a tabela de derivadas `∂z/∂t = R^m` descreva o que o código faz — ⇒
  `the_crossings_predict_how_a_translation_moves_a_copy` **mexe** em `t` e confere onde a
  cópia foi parar. *Uma derivada errada não dá erro: dá um solver que parece lento.*
- que a propagação incremental e a reconstrução total concordem — ⇒
  `the_incremental_bump_agrees_with_the_full_apply`, e **foi ele que achou** o termo
  repetido que o `find` engolia.

**LOC LIDAS:** `solve.rs` 689 · `round.rs` 679 · `gauge.rs` 177 · `cut.rs` (tipos, ~130)
· `corners.rs` 46 · `comb.rs` (assinaturas) · `ingest.rs` (a régua, ~60) ·
`gates_fixtures.rs` (~45) · `support/mod.rs` (~30) — mais os 2 000 escritos.

---

## LENTE 2 — COSTURA: o produto alcança o código novo, e só por onde deve

**CLAIM:** o caminho soldado é alcançável pelo botão `Quad Retopology` **dentro** de
`PH2D_RETOPO_EXTRACT=1`, e o caminho de sempre continua alcançável e intocado.

**TRAÇO:** botão → `quad_remesh_global`
([`retopo_global.rs:76`](../../../shells/desktop/src/sculpt3d_history_retopo_global.rs)) →
`extract_requested()` (**uma** chamada, com gate a contá-la) →
`quad_remesh_extract` ([`retopo_extract.rs:62`](../../../shells/desktop/src/sculpt3d_history_retopo_extract.rs))
→ `ph2d_gridmap::welded_enabled()` (linha 107) → `round_welded` (linha 110) ou
`round_to_integers` (linha 112).

**ASSERÇÃO-VERMELHA:**
`only_one_place_reads_the_welded_switch`
([`architecture_one_switch.rs`](../../../crates/ph2d-gridmap/tests/architecture_one_switch.rs)) —
varre a workspace rastreada e exige que **um** ficheiro leia a env; os outros chamam a
porta. ⚠️ Ele **reprovou na 1ª redacção** e o achado foi do próprio gate: ele media quem
*nomeia* a variável e apanhou um **doc-comment** — *um gate que confunde documentação com
acoplamento pede que se apague a documentação*. A régua passou a ser quem **lê**.

**NÃO-CHECADO-PELA-COMPILAÇÃO:**
- ⛔ **que o resultado seja BOM para o artista.** Nada nesta janela viu um pixel; o
  veredito é do Enio, pelo smoke do handoff §6.
- que o caminho penalizado continue byte-idêntico: ⚠️ **não há golden**, e o argumento é
  estrutural (as duas funções são disjuntas; o refactor do numerador de Poisson preserva a
  ordem das operações). O que o **cobre de facto** são os gates numéricos do caminho antigo
  (`as_translacoes_ficam_todas_inteiras`, `a_escada_fica_no_degrau_barato`,
  `the_two_sides_of_an_arc_agree_on_where_the_marks_fall`), **todos verdes** no gate
  batched. *É cobertura por consequência, não a afirmação directa — e fica dito.*

**LOC LIDAS:** `retopo_extract.rs` 299 · `retopo_global.rs` (a porta, ~30) ·
`chain_info.rs` 251.

---

## ⚠️ O que a auditoria NÃO fecha (e por isso está no handoff)

1. **A regressão do `>60°`** (4→10 na enrugada, 3→9 no gancho) — mecanismo nomeado
   (distorção métrica local perto de singularidade), cura publicada nomeada (*local
   stiffening*), **fora desta wave por ordem do §6 da espec**.
2. **A peça perfurada** — o gate nº8 da espec («o bordo é preservado») ainda não tem
   régua; o número está medido, o veredito não.
3. **A barra do gate nº1 atravessa duas precisões** (`f64` da referência contra `f32`
   nosso) — a pergunta está devolvida no handoff §10, com a forma que o gate usa hoje e o
   número medido.
