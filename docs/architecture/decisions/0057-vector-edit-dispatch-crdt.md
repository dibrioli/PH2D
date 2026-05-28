# ADR-0057 — Vector edit dispatch + CRDT data model

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0056 — Vector Network data model](0056-vector-network-data-model.md), [ADR-0021 — SimWorld/PresentWorld](0021-simulation-presentation-boundary.md), [ADR-0022 — No HashMap in simulation](0022-no-hashmap-in-simulation.md), [ADR-0040 — Tool isolation](0040-tool-as-isolated-feature-crate.md).
**Spec normativa:** [`docs/Vector Module/01_data_model.md §1.4 + §1.5`](../../Vector%20Module/01_data_model.md).
**Tags:** vector, wave-0, contract, crdt, multi-agent

---

## 1. Contexto

Mutações ao `VectorNetwork` precisam de mecanismo canônico **rastreável**, **replayável** e **convergente em multi-agente local** (LLM agent + designer humano editando paralelo). Três decisões interrelacionadas:

1. **Como dispatchar mutações?** Variant novo em `EditorAction` (cap-bump ADR-0040) OU usar `ToolPanelEvent` existente?
2. **Como gravar history?** Event-sourced `EditLog` (immutable ops sequence) é canônico (vide ADR-0056 §2.8).
3. **Como resolver concurrent edits?** CRDT data model — Proposta 5 Antigravity 1ª iteração absorvida integral.

### 1.1 Por que CRDT em local-only?

PH2D ainda **não** suporta web-collab cross-internet (OUT v1.0 vide §4 README). Por que CRDT então?
- **Multi-agente local** = LLM assistant (via MCP) + designer humano editando mesmo canvas em paralelo. Sem CRDT, agente sobrescreve trabalho humano OR vice-versa.
- **Multiplayer rollback netcode-ready** = se PH2D V2.0+ adicionar networking, CRDT já está pronto. Custo upfront ~30% memory overhead em sessões multi-agent; zero overhead single-user (CRDT runtime opt-in).
- **Replay determinístico cross-platform** = CRDT log replays bit-identical (LWW timestamps + RGA order). Multiplica valor de `tests/determinism/vector_replay.rs`.

---

## 2. Decisão

### 2.1 `EditorAction::VectorOp(VectorOp)` — variant novo (cap-bump ADR-0040 §7)

Payload size de paths complexos não cabe em `PanelEvent::SetValue|Click(id, value)` (key-value simple). Solução: variant novo no `EditorAction` enum.

```rust
// Em ph2d-editor-core::action_bus::EditorAction
#[non_exhaustive]
pub enum EditorAction {
    ActivateTool(ToolId),
    OneShotImageOp(ImageOpRequest),
    ToolPanelEvent(PanelEvent),
    CancelActiveTool,
    VectorOp(VectorOp),         // NEW W0 ratified 2026-05-29
}
```

**Cap-bump arch-gate** `architecture_tool_contract_surface` (ADR-0040 §7): `EditorAction = 5` (era 4). Amendment de ADR-0040 obrigatório (`0040-amendment-1.md` pattern per §10 README Vector).

### 2.2 CRDT hybrid: LWW + RGA + custom per-component

3 categorias de state em VectorNetwork com semantics distintas exigem CRDTs diferentes:

| Categoria | Exemplo | CRDT escolhido | Razão |
|-----------|---------|----------------|-------|
| **Set membership** | which vertices em region | **LWW-Element-Set** (Shapiro 2011) | Add/remove vertex; conflict simples last-writer-wins por timestamp |
| **Ordered sequences** | segments dentro de region (winding direction matters) | **RGA** (Replicated Growable Array) | Preserve intent ordering across concurrent inserts |
| **Continuous values** | `vertex.pos`, tangent handles | **Per-component LWW** | Two tangents moving concurrent colide; LWW por axis (não whole vector) preserve fine-grained semantics |

```rust
pub struct CrdtReplay {
    pub site_id: u64,                  // estável per session
    pub seq: u64,                      // sequence local counter
    pub region_members: BTreeMap<RegionId, LwwSet<(SegmentId, bool)>>,
    pub segment_order: BTreeMap<RegionId, Rga<SegmentId>>,
    pub continuous: BTreeMap<(VertexOrSegmentRef, ComponentAxis), LwwRegister<f32>>,
}
```

`BTreeMap` em vez de `HashMap` (HR-5 + ADR-0022).

### 2.3 Timestamp validation window (Antigravity 3ª iter L4F3 security)

Adversarial agent pode forjar timestamps far-future ("my edits always win LWW") OR far-past ("rollback your edits"). Clamp contra SimWorld clock:

```rust
impl CrdtReplay {
    pub fn apply(&mut self, op: VectorOp, claimed_ts: Timestamp) -> Result<()> {
        const MAX_DRIFT: Duration = Duration::from_secs(30);
        let local = self.sim_world_clock_now();

        if claimed_ts > local + MAX_DRIFT {
            return Err(Error::TimestampFromFuture { claimed: claimed_ts, local });
        }
        if claimed_ts < local - MAX_DRIFT {
            return Err(Error::TimestampTooOld { claimed: claimed_ts, local });
        }
        let safe_ts = claimed_ts.clamp(local - MAX_DRIFT, local + MAX_DRIFT);
        self.apply_with_validated_ts(op, safe_ts)
    }
}
```

### 2.4 Periodic integrity check (Antigravity 3ª iter L7F3 silent divergence recovery)

Silent CRDT divergence em multi-agent local é rare mas real (concurrent edits + clock drift); mitigation:
- Cada **30s** (configurable), todos sites computam `blake3(state.serialize())` e exchange hash via shared channel.
- Discrepância → trigger rollback to **last consensual snapshot** (LCS) + replay logs from there.
- LCS = `EditLog::snapshots[snapshot_idx]` periódico (cada 100 ops, vide ADR-0056 §2.8).

Gate `tests/determinism/vector_crdt_silent_divergence_recovery.rs` simula divergence + valida recovery converge.

### 2.5 Property-based testing obrigatório (Antigravity 2ª iter L1F6)

Crate `proptest` em `tests/determinism/vector_crdt_proptest.rs`:

```rust
proptest! {
    #[test]
    fn crdt_converges_under_random_concurrent_edits(
        site_a_ops in prop::collection::vec(arb_vector_op(), 1..200),
        site_b_ops in prop::collection::vec(arb_vector_op(), 1..200),
    ) {
        let (mut a, mut b) = (CrdtReplay::new(SITE_A), CrdtReplay::new(SITE_B));
        for op in &site_a_ops { a.apply(op.clone(), arb_timestamp()).unwrap(); }
        for op in &site_b_ops { b.apply(op.clone(), arb_timestamp()).unwrap(); }

        a.merge(&b.export_log()).unwrap();
        b.merge(&a.export_log()).unwrap();

        prop_assert_eq!(hash(&a.state()), hash(&b.state()));
        prop_assert!(a.state().has_no_orphan_segments());
        prop_assert!(a.state().has_no_self_intersecting_regions());
        prop_assert!(a.state().region_cycles_close_properly());
    }
}
```

Config: 256 cases default; CI nightly `PROPTEST_CASES=10000`. Gate `vector_crdt_proptest_convergence`.

### 2.6 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| `EditorAction` variants | **5** (cap-bumped de 4) | Amendment ADR-0040 |
| `MAX_DRIFT` timestamp window | **30 seconds** | Aceita relógio dessincronizado moderado; rejeita forge |
| Periodic integrity check interval | **30 seconds** | Balance overhead vs divergence detection latency |
| Snapshot interval | **a cada 100 ops** | Balance memory vs rollback granularity |
| Proptest default cases | **256** (CI nightly **10_000**) | Padrão proptest crate |

---

## 3. Consequências

### 3.1 Positivas

- **Multi-agente local destrancado desde W1** — LLM via MCP edita network em paralelo com designer humano sem perda de trabalho.
- **Replay determinístico cross-OS bit-identical** via LWW+RGA+custom — gate `vector_crdt_proptest_convergence` valida 10k random.
- **Security hardening L4F3** — timestamp forge attacks rejected.
- **Silent divergence recovery L7F3** — periodic integrity check + LCS rollback evita state corruption.
- **Future multiplayer co-edit-ready** — CRDT data model é a barreira #1 de Figma multiplayer; PH2D já preparada.

### 3.2 Negativas

- **CRDT bookkeeping overhead ~30% memory** em sessões multi-agent. Opt-in via `CrdtReplay` instance; single-user mode zero overhead.
- **Hybrid CRDT** (LWW + RGA + custom) é mais complexo que LWW puro. Debugging exige understanding de 3 semantics. Mitigação: docstrings exhaustive + property tests.
- **`EditorAction` cap-bump 4→5** quebra contrato congelado ADR-0040 — exige amendment doc + retest todo o tooling existing.

### 3.3 Neutras

- Periodic integrity check (30s interval) adds ~0.1% CPU overhead per session.

---

## 4. Alternativas consideradas

### 4.1 LWW-Element-Set puro (rejeitada — perde ordering)

Proposta 5 Antigravity 1ª iter sugeriu LWW. **Por que rejeitada parcialmente**: LWW perde segment ordering em regions; winding direction matters; resulting state pode renderizar boolean errado. Hybrid LWW+RGA preserve.

### 4.2 RGA puro (rejeitada — over-engineering para single-user)

RGA é overkill para single-user (>90% dos casos). Hybrid usa RGA apenas em segment ordering; LWW para resto. Pragmatic balance.

### 4.3 Snapshot-based undo (rejeitada — memory heavy)

`Vec<NetworkSnapshot>` por op. ~200KB per snapshot × 10k ops = 2GB memory. Inviable. Event-sourced log = leve (~50 bytes per op average).

### 4.4 `EditorAction::ToolPanelEvent` para vector ops (rejeitada — payload size)

Reusar slot existente sem cap-bump. Mas `VectorOp::AddRegion { segments: SmallVec<[(SegmentId, bool); 16]> }` não cabe em `PanelEvent` key-value model. Cap-bump justificado.

---

## 5. Implementação (Wave 1)

- **T1.2** — `ph2d-vector-doc::edit_log` + `crdt` modules.
- **T1.6** — CRDT data model (LWW + RGA + custom).
- **T2.5** — Undo via CRDT edit_log no chrome handler.
- **T1.8** — Audit + proptest convergence + integrity check tests.

Amendment ADR-0040 (`0040-amendment-1.md`) registra cap-bump `EditorAction = 5`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/01_data_model.md §1.4 + §1.5`](../../Vector%20Module/01_data_model.md).
- Shapiro et al. 2011 (CRDTs theory): <https://hal.inria.fr/inria-00609399v1>
- RGA paper (Roh et al. 2011): <https://csl.skku.edu/papers/jpdc11.pdf>
- Proptest crate: <https://crates.io/crates/proptest>
- 3 iterações Antigravity (L1F6 proptest, L4F3 timestamp window, L7F3 integrity check, P5 CRDT hybrid) absorvidas.
