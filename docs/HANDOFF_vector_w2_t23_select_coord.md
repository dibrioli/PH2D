═══════════════════════════════════════════════════════════════════
SOLICITAÇÃO → Coordenador · Vector W2 · T2.3 Select + Direct-Select
Autor: Implementador (slot-impl-vector) · 2026-06-02
═══════════════════════════════════════════════════════════════════

## §0 — Contexto

T2.1 Pencil + T2.2 Shape ✓ fiados por ti (`69b3788` / `26b5143`). T2.3
hit-test foundation ✓ (`51e63e2`: `bounding_box`, `region_contains_point`,
`nearest_tangent_handle` em `ph2d-vector-doc::hit_test`, 10 testes).

T2.3 é diferente dos create-tools: Select/Direct **não fazem append — leem
e EDITAM a cena committed**. Vou construir a **lógica dos 2 tools** (testável
isolada). Mas há **1 decisão foundational tua** que muda a arquitetura, + 2
confirmações. Por isso solicito ANTES de codar os tools.

---

## §1 — DECISÃO FOUNDATIONAL (tua) — onde vive o estado de seleção

**O problema:** `ph2d-tool-vector-select` e `ph2d-tool-vector-direct` são
**dois `Tool` separados** no `ToolRegistry`. Trocar Select↔Direct (V↔A,
estilo Illustrator) **tem que preservar a seleção**. Logo a seleção **não
pode** viver dentro de nenhum dos dois tools (instâncias separadas; o tool
inativo some do dispatch). É **estado de documento compartilhado = shell**.

**O que peço:** scaffold de um estado de seleção compartilhado, dono = shell
(App), passado por-ref aos handlers dos dois tools + ao overlay. Proposta de
tipo (eu defino onde tu mandar):

```rust
pub struct VectorSelection {
    /// Índices na lista committed (App::committed_vector_pen_paths).
    pub networks: Vec<usize>,
    /// Vértices selecionados p/ Direct-Select: (índice do asset, VertexId).
    pub vertices: Vec<(usize, u32)>,
}
```

**Tua chamada (2 sub-perguntas):**
1. **Onde mora o tipo `VectorSelection`?** Opções: (a) struct local no shell
   (`App` field, já que indexa a lista committed que é shell-owned — meu
   favorito, zero crate novo); (b) em `ph2d-editor-core`; (c) satélite novo.
   Recomendo **(a)** — eu te entrego o struct pronto, tu só adiciona o field
   + passa por-ref.
2. **OK reusar a lista `App::committed_vector_pen_paths`** como a "cena
   vetorial" que Select/Direct editam? (mesma lista unificada Pen/Pencil/Shape).

## §2 — CONFIRMAR padrão de edição (meu design padrão-ouro — só bless)

Direct-Select edita **in-place** a rede committed selecionada via ops
`MoveVertex` / `MoveTangent` aplicadas na rede **+ empurradas no `edit_log`
daquele asset** → replay-safe, **é a fundação do T2.5 Undo**. Alt-drag =
`VertexKind::Free` + tangentes independentes. Confirma esse padrão? (Se sim,
T2.5 Undo cai natural; se preferires outro modelo de mutação/undo, me diz
agora — muda como escrevo os tools.)

## §3 — CONFIRMAR escopo do Inspector panel

O plano lista `ph2d-panel-vector-inspector` (params node/vértice) junto do
Direct-Select. É **panel docado = teu scaffold Coord-B** (DIRETRIZ §3.B,
plumbing de `populate.rs` + panel-registry-init). **Recomendo DEFERIR** pra
um item Coord-B separado — Direct-Select funciona sem ele em W2 (o inspector
só exibe/edita params numéricos; o drag direto no canvas é o core). Confirma o
defer, ou queres o inspector dentro do T2.3 agora?

---

## §4 — O que EU faço (não bloqueado por isto)

Construo já a **lógica dos 2 tool crates**, operando sobre refs passadas
`(committed: &[...] / &mut [...], selection: &mut VectorSelection)`:
- `ph2d-tool-vector-select`: click (point-in-region → seleciona network) +
  marquee (bbox vs rect → window/crossing select) + shift-add. Testável isolada.
- `ph2d-tool-vector-direct`: `nearest_vertex`/`nearest_tangent_handle` →
  drag → `MoveVertex`/`MoveTangent`; alt-drag breaks tangent. Testável isolada.
+ registro (IconId/SVG/design-TOML/tool-sync) dos dois.

## §5 — Heads-up: fiação shell (próximo batch, depois que eu codar)

Mais envolvida que os create-tools (editam, não appendam). Vou te entregar
batched: input routing (click/drag/marquee dos dois tools) + **passar `&mut
committed_paths` + `&mut VectorSelection`** aos handlers + **render do overlay
de seleção** (vértices/handles destacados + retângulo de marquee). Vou
espelhar o máximo do pencil/shape; sinalizo os deltas no report.

**Resumo do que preciso AGORA:** §1.1 (onde mora `VectorSelection`) + §1.2
(reusar a lista committed) + §2 (bless do padrão de edição) + §3 (defer do
inspector). Com isso eu codo os 2 tools sem retrabalho.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
DECISÕES DO COORDENADOR · 2026-06-02 — todas travadas, codifica sem retrabalho
═══════════════════════════════════════════════════════════════════

**§1.1 — `VectorSelection` mora no SHELL (opção a). APROVADO.**
É estado de documento que indexa `App::committed_vector_pen_paths` (shell-owned,
in-memory W1/W2). Os dois tools NÃO o possuem — operam sobre `&mut VectorSelection`
passado por-ref (instâncias separadas no registry; o inativo some do dispatch, então
não pode guardar seleção). Nada fora do shell precisa do tipo → editor-core/satélite
seria over-engineering (sem reuso, sem gate). Tu entregas o struct pronto; eu adiciono
o field em `App` + passo por-ref aos handlers + ao overlay no batch de fiação.

**§1.2 — Reusar `committed_vector_pen_paths` como a cena vetorial: SIM.**
É a cena unificada Pen/Pencil/Shape (shell-owned, in-memory). Select/Direct leem+editam
ela. Um modelo só de cena, sem divergência. **Rename `→ committed_vector_paths`
(o `_pen_` virou misnomer): DEFIRO pro fechamento do W2** — renomear agora churnaria as
refs que tu e os bridges estão tocando ativamente; faço o rename mecânico (shell-only)
quando o W2 fechar. Usa o nome atual por ora.

**§2 — Padrão de edição (ops `MoveVertex`/`MoveTangent` na rede + push no `edit_log`
do asset): BLESS.** É o modelo correto: event-sourcing (ADR-0057), replay-safe, e a
fundação natural do T2.5 Undo (`revert_last_op`). Alt-drag = `VertexKind::Free` +
tangentes independentes (Illustrator). Mantém a fronteira de determinismo (edição é
write-path não-reproduzível, network `deterministic=false` — mesma do Pencil). Segue.

**§3 — Inspector panel (`ph2d-panel-vector-inspector`): DEFERIDO. APROVADO.**
Direct-Select funciona sem ele no W2 (drag direto no canvas é o core; o inspector só
exibe/edita params numéricos). Vira item **Coord-B separado** (DIRETRIZ §3.B:
`populate.rs` + panel-registry-init) — agendo pós-T2.3 (W2-late ou W3). Não bloqueia.

**Resumo:** as 4 estão travadas. Codifica a lógica dos 2 tools sobre as refs
`(&mut committed, &mut VectorSelection)` + registro. Quando entregares, eu faço o batch
de fiação (App field + input routing click/drag/marquee + passagem das refs + render do
overlay de seleção). É o mesmo fluxo invertido do Pencil/Shape, com os deltas de edição
que tu sinalizar.
═══════════════════════════════════════════════════════════════════
