# `ctt v0.4.0` Source Audit — CONSOLIDATED (2026-05-27)

**W1.T2 deliverable** do plano vivo [`docs/plans/2026-05-texture-compression-waves.md`](../plans/2026-05-texture-compression-waves.md). Consolida 2 lentes paralelas adversariais:

- [Lente A — Data-Integrity](ctt-source-audit-2026-05-27-lens-A-data-integrity.md) (~5.8k LOC auditadas: `processing/` + `encoders/` + format/convert + KTX2 output + 3 wrappers críticos)
- [Lente B — HR-Compliance + Supply-Chain](ctt-source-audit-2026-05-27-lens-B-hr-supply-chain.md) (8 sub-crates Cargo.toml + build.rs + license + threading scan + 13 upstream issues)

---

## Veredito consolidado: **APPROVE_WITH_CAVEATS** ✅

`ctt v0.4.0` é **adotável** como cooker offline para PH2D Fase 2 (decisão ADR-0055-v4). **Fallback B NÃO necessário** (NÃO precisa migrar para `toktx` + `Compressonator` CLIs em canonical runner). Adoção requer **5 disciplinas operacionais** documentadas + 1 PR upstream opcional.

### Scores

| Lente | Score | Findings | Veredito |
|---|---|---|---|
| A · Data-Integrity | 7.5/10 | 0 CRITICAL · 3 HIGH · 5 MEDIUM · 4 LOW | APPROVE_WITH_CAVEATS |
| B · HR-Compliance + Supply-Chain | 9.0/10 | 0 CRITICAL · 1 HIGH · 3 MEDIUM · 0 LOW | APPROVE_WITH_CAVEATS |

### HR-1 §2.7.1 checklist (8/8 PASS)

Todos os 8 sub-crates (ctt main + ctt-astcenc + ctt-bc7enc-rdo + ctt-bc7enc-rdo-prebuilt + ctt-compressonator + ctt-etcpak + ctt-intel-texture-compressor + ctt-intel-texture-compressor-prebuilt) **PASS 6/6** dos critérios FFI C/C++ aceitáveis:

1. ✅ Offline-only (cooker dev/CI, never release-game)
2. ✅ Ref impl única OR best-of-domain (bc7e/astcenc/etcpak/Compressonator todos canônicos do domínio)
3. ✅ Vendored Cargo + build.rs reproducível (zero network/random/timestamp; 103/103 deps de crates.io)
4. ✅ Licenses MIT/Apache-2.0/Zlib/BSD-3 (zero GPL/LGPL/proprietário)
5. ✅ Maintainer ativo (último commit upstream 2026-05-16)
6. ✅ Não patent-encumbered (zero claims HEVC/H.265/DTS/AAC family)

---

## Disciplinas operacionais obrigatórias (W1.T3+ implementação)

### D1 — `ctt` deps pinadas com `default-features = false` + features explícitas
Razão: Lente A HIGH#3 — auto-dispatch order do `ctt` é feature-gated; ativar/desativar features muda silenciosamente qual encoder backend é escolhido para um dado formato, divergindo bytes de output. Implementação:

- `tools/asset-cooker/Cargo.toml`: trocar `ctt = "0.4.0"` por `ctt = { version = "0.4.0", default-features = false, features = ["encoder-bc7enc", "encoder-astcenc", "encoder-etcpak", "format-ktx2"] }` (lista exata em W1.T3 após inventory).
- Cargo.lock commitado.
- Arch-gate `architecture_ctt_features_pinned` (`crates/ph2d-asset-cooker/tests/`): grep Cargo.toml + assert features exatas.

### D2 — Canonical runner CPU class pinada (ADR-0055-v4 já decide)
Razão: Lente A HIGH#1 — ISA dispatch runtime (astcenc + sRGB SIMD em ctt) usa CPU features detectadas em runtime; cross-CPU class (AVX2 vs AVX512 vs NEON) diverge output bytes. PH2D HR-6 (`AssetId = blake3(bytes)`) quebra fora do runner canonical.

- ADR-0055-v4 §2.3 já decidiu: cook em GitHub Actions `ubuntu-latest` Linux x86_64 único.
- W1.T10 do plano vivo: `runs-on: ubuntu-latest` + `if: matrix.os == 'ubuntu-latest'` no step de cook.
- W1.T5 cooked-hashes.lock detecta drift no canonical runner via 5 cooks consecutivos + assert blake3 igual.

### D3 — Banir `(encoder-amd + UltraFast + BC7)` combo no wrapper PH2D
Razão: Lente A HIGH#2 — Compressonator BC7+UltraFast retorna `R=0` silencioso em Linux/macOS (acknowledged em comentário do próprio teste do ctt em `compressonator.rs:296-300`). Bug encoder upstream conhecido.

- Opção segura: NÃO ativar feature `encoder-amd` (cobrir BC7 via bc7enc-rdo apenas) — preferido se bc7enc-rdo cobre todos os casos.
- Se precisar `encoder-amd` para outro formato (BC1-BC5): adicionar guard no wrapper PH2D que `panic!` se config = (Compressonator backend + UltraFast quality + BC7 format).
- Arch-gate `architecture_no_compressonator_bc7_ultrafast` (`crates/ph2d-asset-cooker/tests/`).

### D4 — Snapshot integration test: primeiros 64 bytes per (format, encoder, quality)
Razão: Lente A HIGH#1+#3 — garantir que (a) version bump do `ctt` não muda output silenciosamente (b) feature drift não muda encoder.

- W1.T11.5 (NOVO sub-task): `tools/asset-cooker/tests/snapshot_output_first_64_bytes.rs`. Para cada combinação de (format, encoder, quality) suportada, fixture 64×64 deterministic gradient → cook → assert primeiros 64 bytes match snapshot file.
- Snapshot files versionados em `tools/asset-cooker/tests/snapshots/` (NÃO via Git LFS, são pequenos).
- Falha no test = upgrade do `ctt` ou flag mudou → human review.

### D5 — `cargo-deny ban` defensivo: `basis-universal-sys` + `nvtt`
Razão: Lente B triage 13 upstream issues — issues #68 (basisu integration) e #23 (NVTT proprietary) representam direções FUTURAS do `ctt` upstream que PH2D NÃO quer consumir transitivamente.

- `deny.toml`: adicionar `[bans] deny = [{ name = "basis-universal-sys" }, { name = "nvtt" }]` (verificar nomes exatos das crates upstream).
- Falha CI se transitive dep aparecer pós upgrade do `ctt`.
- ADR §2.7 fallback continua válido (toktx + Compressonator CLIs separadas, não via libbasisu in-process).

### D6 (opcional) — PR upstream: mover `criterion` para `dev-dependencies`
Razão: Lente B HIGH#1 — `criterion` está em `[dependencies]` (não `[dev-dependencies]`) do ctt main; isso o linkaria em release builds se ctt for usado fora de cooker context.

- Mitigação direta: ADR-0055-v4 confina `ctt` em `tools/asset-cooker` (binário tool, nunca release-game) — risco já neutralizado para PH2D.
- PR upstream opcional para health do ecossistema. Não bloqueia W1.T3.

---

## Achados detalhados — síntese cross-lente

### CRITICAL (0 encontrados, 0 bloqueadores)
Nenhum finding CRITICAL identificado em nenhuma das 2 lentes. ctt é estruturalmente sólido: pipeline single-threaded, sem rayon/HashMap iteration affecting output/RNG sem seed; alpha simétrico load↔store; edge-clamping correto em `tile_to_blocks`; KTX2 emite buffer pre-sized; FFI surface bem-contextualizada com `SAFETY:` comments + bounds checks.

### HIGH (4 total — 3 data-integrity + 1 supply-chain, todos mitigáveis)

**HIGH-A1** · ISA dispatch runtime em astcenc + sRGB SIMD → cross-CPU divergence. Mitigado por D2 (canonical runner pinada — ADR já decide).
**HIGH-A2** · Compressonator BC7+UltraFast = R=0 silent em Linux/macOS. Mitigado por D3 (ban combo no wrapper).
**HIGH-A3** · Auto-dispatch order feature-gated → drift silencioso. Mitigado por D1 (`default-features=false` + features explícitas).
**HIGH-B1** · `criterion` em `[dependencies]` do ctt main. Mitigado por confinamento ctt em `tools/asset-cooker` (ADR já decide).

### MEDIUM (8 total — 5 data + 3 supply)
Detalhes nos arquivos de lente. Resumo: 308 `unsafe` blocks total (todos contextualizados), `ASTCENC_NO_INVARIANCE=1` + `-ffp-contract=fast` no astcenc (HR-6 OK com toolchain pin), etcpak C++20 (validar MSVC ≥19.29 no CI Windows — mas nosso canonical runner é Linux só, então não-blocker), 2 `OnceLock` init-once dispatch tables (HR-6 OK).

### LOW (4 total)
Tracking only. TODO/FIXME em LIB code minoritários.

---

## Encoder backend dispatch — sumário

| Format | Default backend escolhido pelo ctt | Determinismo da escolha | PH2D action |
|---|---|---|---|
| BC7 | bc7enc-rdo (se feature ativa) > Intel ISPC > Compressonator | feature-gated (D1 pina features) | feature `encoder-bc7enc` ON, banir AMD+BC7+UltraFast (D3) |
| BC1-BC5 | Intel ISPC > Compressonator > etcpak (alguns) | feature-gated | feature `encoder-intel` ON |
| ASTC LDR | astcenc (ARM ref impl única) | ISA dispatch runtime (D2 canonical runner) | feature `encoder-astcenc` ON |
| ETC2 | etcpak (fast/quality presets) | single backend | feature `encoder-etcpak` ON |
| KTX2 container | ctt own emit via ktx2 = 0.5 (mesma crate da Fase 1 PH2D) | deterministic | OK |

---

## Triage 13 issues upstream (síntese Lente B)

- **2 issues a evitar transitividade** (D5): #68 basisu integration, #23 NVTT proprietary
- **2 issues a tracker para W2**: #74 HDR ASTC (relevante W4+ HDR wave), #65 normal map renormalize (relevante quando PH2D shipar normal maps)
- **9 outras**: meta/features que PH2D v1 não consome (3D textures, layered images, RGBM encoding, etc.)

Nenhum issue ABERTO classificado como CRITICAL data-loss/security/non-determinism que afetaria PH2D production usage hoje.

---

## Plano vivo updates resultantes desta audit

W1.T2 fechada com APPROVE_WITH_CAVEATS. Adicionar 5 sub-tasks ao W1 (depois de T2):

- **W1.T2.1** (NOVO) — D1: `tools/asset-cooker/Cargo.toml` features pinadas + arch-gate
- **W1.T2.2** (NOVO) — D3: wrapper guard ban (Compressonator + UltraFast + BC7) + arch-gate
- **W1.T2.3** (NOVO) — D4: snapshot integration test 64 bytes per (format, encoder, quality) + fixture
- **W1.T2.4** (NOVO) — D5: `deny.toml` ban basis-universal-sys + nvtt
- **W1.T2.5** (opcional) — D6: PR upstream `criterion` → dev-deps (não bloqueia W1.T3)

D2 já está em W1.T10 do plano vigente.

---

**Auditores:** Claude Opus 4.7 (1M context) × 2 sub-agentes paralelos. Tempo: ~10min wall-clock (~50min cumulative). Coord-A consolidou.

**Próximo desbloqueio:** W1.T3 — implementar `tools/asset-cooker/src/texture/mod.rs` + sub-command CLI. Pré-requisitos: D1+D3+D4+D5 implementados primeiro (~1 sessão adicional).
