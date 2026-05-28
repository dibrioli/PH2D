# W1.T15 — Lente ρ (rho): regression / findings-ledger closure (gate final da W1)

- **Data:** 2026-05-28
- **Auditor:** Claude Opus 4.8 (orquestração + verificação independente) com um
  sub-agente Haiku 4.5 fazendo o sweep inicial dos 12 docs de audit.
- **Escopo:** verificar que **todo finding que rodadas anteriores declararam FIXED
  está de fato fixado no código atual**, e que findings deferred são honestamente
  rastreados. Code reality > claims verbais.
- **Lente nova:** ρ ainda não usada; rotação per [[feedback-audit-lens-diversity]].
  Round único, anti-Goodhart. Ortogonal a σ (σ olha pra frente — o seam funciona?;
  ρ olha pra trás — fechamos o que prometemos?).

---

## Veredito: **PASS — 9.0 / 10**

Os 6 ciclos de audit anteriores fizeram **trabalho honesto**. Todo finding
claimed-FIXED que verifiquei está genuinamente presente no código. Os deferrals
(gradient hash-pin, sdf_font rename) são honestamente rastreados pra W1.T10.

⚠️ **Nota de método:** o sub-agente de sweep reportou 2 "blockers"
(ξ-F1 sem cobertura de teste; γ-H3 mensagem over-claiming). **Verifiquei ambos
independentemente e os DOIS são falsos-positivos** — o agente grepou
`crates/ph2d-asset-ktx2/tests/` (diretório que não existe; os testes são INLINE
em `lib.rs`) e leu um offset de linha stale pro γ-H3. Documentado em §3 como
lição de verificação ([[feedback-audit-internal-state-grep]]).

---

## §1 Catálogo consolidado dos 6 ciclos (10+ lentes α..ξ)

| Ciclo | Task(s) | Lentes | Score(s) | Veredito |
|---|---|---|---|---|
| 1 | W1.T3 (cooker first impl) | γ (impl correctness), δ (arch-as-code + cross-doc) | 9.2 / 8.5 | APPROVE |
| 2 | W1.T4 (Asset::TextureKtx2 + TierIndex + LogicalTextureMap) | ε (impl + edge), ζ (HR + cross-doc) | 9.2 / 9.2 | APPROVE |
| 3 | W1.T6 (cook_all multi-tier batch) | η (multi-tier semantics), θ (CLI UX + integration) | 9.0 / 8.5 | APPROVE |
| 4 | W1.T7 (mip pyramid gen) | λ (mip math + edge), μ (perf / memory / ctt) | 8.5 / 8.7 | APPROVE w/ 1 HIGH |
| 5 | W1.T11+T14 (7 fixtures + R8→BC4 proof-of-life) | ι (determinism), κ (integration + W3 readiness) | 7.5 / 9.2 | APPROVE w/ caveat |
| 6 | W1.T9 (kvd preservation + PremulIntent + bounds) | ν (contract preservation), ξ (bounds / DOS) | PASS / PASS_W_FINDINGS | PASS |

(α/β foram lentes de Fase 1, fora do escopo da Fase 2 W1.) Total: **14 letras
gregas, 12 docs em `docs/audits/w1-t*-lens-*.md`.**

---

## §2 Verificação dos findings-chave claimed-FIXED

Todos confirmados **presentes no código atual** (HEAD = e5fb811 + esta sessão):

| ID | Sev | Fix declarado | Estado verificado | Evidência |
|---|---|---|---|---|
| **γ-H1** | HIGH | `target_for` retorna `TargetFormat` direto (não `Option`) | ✅ CONFIRMED | `target_matrix.rs:67` assinatura `-> TargetFormat`; `cook.rs:147` sem unwrap |
| **γ-H2** | HIGH | `CookOptions::for_asset_class` deriva color_space (NormalMap→Linear) | ✅ CONFIRMED | `cook.rs:90` + teste `cook_normal_map_uses_linear_color_space_via_for_asset_class` (cook.rs:273) |
| **γ-H3** | HIGH | test renomeado + msg honesta (intra-machine only) | ✅ CONFIRMED | teste **já** se chama `cook_intra_machine_byte_identity_when_repeated` (cook.rs:241); msg disclaim cross-machine explícito (cook.rs:247-250) |
| **η-M3** | MED | comentário "lista parcial" corrigido (API não expõe parcial) | ✅ CONFIRMED | `cook.rs:179-183` |
| **ν-7** | LOW | `#[non_exhaustive]` em `Ktx2Image` + `Ktx2Error` | ✅ CONFIRMED | lib.rs:114 (`Ktx2Error`) + lib.rs:428 (`Ktx2Image`) |
| **ξ-F1** | HIGH | `build_fixture` emite KVD real + 6 testes de parse-path | ✅ CONFIRMED | tests INLINE em lib.rs: `decode_fixture_with_kvd_round_trips` (1418), `decode_kvd_premul_round_trips_end_to_end` (1440), `decode_rejects_too_many_kvd_entries` (1453), `decode_rejects_oversized_kvd_value` (1476), `decode_accepts_kvd_value_at_exact_cap` (1497), `decode_rejects_too_many_duplicate_kvd_keys` (1514), `decode_rejects_oversized_kvd_key` (1538) |
| **ξ-F2** | LOW | conta iterações (não `kvd.len()`) → fecha duplicate-key flood | ✅ CONFIRMED | `seen` counter (lib.rs ~681) + teste `decode_rejects_too_many_duplicate_kvd_keys` (1514) |
| **ξ-F3** | LOW | `MAX_KVD_KEY_BYTES=256` + `KvdKeyTooLong` | ✅ CONFIRMED | const lib.rs:85; variant lib.rs:184; check lib.rs:690; teste 1538 |
| **ν-6** | LOW | doc drift `ph2d_asset_ktx2::parse` (API inexistente) | ✅ FECHADO inline | `asset.rs:38` agora `decode_ktx2_bytes(&blob)` (esta sessão) |
| **ι-CRIT-1** | CRIT | `std::f32::sin/cos` → `libm::cosf` (determinismo) | ✅ CONFIRMED | `fixtures.rs:114` `libm::cosf`; pin `libm = "=0.2.16"` em Cargo.toml |
| **ordem count→size→alloc (ξ)** | — | bounds checados ANTES de alocar | ✅ CONFIRMED | lib.rs: count-check (684) → key-len (690) → value-size (696) → `insert` depois |

---

## §3 Deferrals — honestidade verificada

| Item | Onde | Rastreado? |
|---|---|---|
| Pin hash `gradient_64x64` desligado (assert comentado) | `fixtures.rs` / armadilha #4 do handoff | ✅ HONESTO — espera W1.T10 canonical runner estabelecer valor cross-platform. Documentado em handoff + comentário no código. |
| `sdf_font_512` semântica = radial distance (não SDF real) | `fixtures.rs:127` | ✅ HONESTO — acknowledged em ι audit como non-blocking; é fixture de teste, não asset shipável. |
| W1.T8 cooker emit kvd (ctt READ-ONLY) | handoff §6.2 + `premul_intent` doc (lib.rs:468) | ✅ HONESTO — 3 paths documentados; W1.T8.1 patcher opcional rastreado. |
| `PipelineOutput::Raw` unreachable | `cook.rs:162` | ✅ Invariante semântica documentada, não dead-code escondido. |

**Zero untracked-deferral.**

---

## §3.bis Lição de método (verificação de sub-agente)

O sweep delegado (Haiku) produziu inventário útil mas **2 falsos blockers**:

1. **ξ-F1 "test coverage absent"** — grepou `crates/ph2d-asset-ktx2/tests/` (não
   existe; o crate tem testes **inline** em `lib.rs#[cfg(test)] mod tests`, 46
   funções). Os 6 testes de parse-path existem (§2). **Falso.**
2. **γ-H3 "assertion message over-claims"** — leu um offset stale (linha ~204);
   o teste real (cook.rs:241) já tem nome + msg honestos. **Falso.**

Generaliza [[feedback-audit-internal-state-grep]] e a disciplina de verificar
claims numéricos: **claim de sub-agente sobre ausência ("não existe teste X")
exige confirmar que o grep mirou o local certo** — testes inline ≠ `tests/` dir.
Sem essa verificação, o gate teria reportado um blocker fantasma e possivelmente
re-implementado testes que já existem.

---

## §4 Conclusão

O ledger da W1 está **limpo**: todo fix declarado verificado presente, todo
deferral honestamente rastreado, zero regressão reaberta. A qualidade dos 6
ciclos anteriores se sustenta sob lente adversarial.

**ρ score: 9.0/10 — PASS.** Nenhum action item bloqueante. (O único finding novo
de toda a W1.T15 — seam sem gate — é da lente σ, não ρ; fechado lá.)
