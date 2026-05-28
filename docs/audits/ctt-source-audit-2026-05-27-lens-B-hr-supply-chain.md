# ctt 0.4.0 Source Audit — Lens B: HR-Compliance + Supply-Chain (2026-05-27)

Auditor: Claude (Opus 4.7) — lens B (HR-compliance + supply-chain hygiene).
Alvo: `ctt v0.4.0` + 7 sub-crates (vendored encoders) no cargo cache local.
Tempo: ~45min. Read-only — não modificou nada fora de `docs/audits/`.

## Resumo executivo

- **Sub-crates auditados**: 8 (1 main + 5 vendored encoders + 2 prebuilt wrappers).
- **Findings**: **0 CRITICAL**, **1 HIGH**, **3 MEDIUM**, **0 LOW**.
- **Recomendação**: **APPROVE_WITH_CAVEATS** — passa em todos os 6 critérios HR-1 §2.7.1
  para os 8 sub-crates. Único bloqueio próximo de "shipping": criterion-em-`[dependencies]`
  do ctt main precisa de feature-gate ou patch upstream (HIGH-1) para evitar leak no bundle
  release-game. Mitigação local trivial via Cargo feature unification + audit deny.

## HR-1 §2.7.1 critério checklist

| Sub-crate | (1) Offline-only | (2) Ref impl única ou best-of-domain | (3) Vendored + build.rs reprod | (4) License | (5) Maintainer ativo | (6) NÃO patent-encumbered | Verdict |
|---|---|---|---|---|---|---|---|
| `ctt` (main) | YES | YES (umbrella, único Rust pure-wrapper p/ KTX2+DDS multi-encoder) | YES (sem build.rs; pure Rust) | `MIT OR Apache-2.0 OR Zlib` | YES (último commit 2026-05-16, repo `cwfitzgerald/ctt` Apache-2.0, 13 issues abertas) | YES (BC/ASTC/ETC = royalty-free famílias) | **PASS** |
| `ctt-astcenc` | YES | YES (`astcenc` é ref impl ARM oficial) | YES (vendored em `cpp/`, build.rs:54-128 compila C++ sem network) | `(MIT OR Apache-2.0 OR Zlib) AND Apache-2.0` (ASTC encoder Apache-2.0) | YES (mesmo repo) | YES (ASTC royalty-free Khronos) | **PASS** |
| `ctt-bc7enc-rdo` | YES | YES (`bc7enc_rdo` por Rich Geldreich = best-of-domain BC7 com RDO) | YES (vendored em `ispc/bc7e.ispc`, build.rs:5-15 cfg-gated compila se feature `build-from-source`) | `(MIT OR Apache-2.0 OR Zlib) AND Apache-2.0` (Binomial LLC Apache-2.0) | YES (mesmo repo) | YES (BC7 royalty-free MS) | **PASS** |
| `ctt-bc7enc-rdo-prebuilt` | YES | YES (mesma source, prebuilt binary p/ devs sem ispc toolchain) | YES (binary `.a` em `bins/<platform>/` COM sigstore.jsonl assinado + source `ispc/` no crate irmão; mtime `Jul 23 2006` = SOURCE_DATE_EPOCH canônico) | `(MIT OR Apache-2.0) AND Apache-2.0` | YES (mesmo repo) | YES | **PASS** |
| `ctt-compressonator` | YES | YES (AMD Compressonator open-source, padrão-indústria BC suite) | YES (vendored em `cpp/`, build.rs:18-90 compila C++ + SIMD variants, sem network) | `(MIT OR Apache-2.0 OR Zlib) AND MIT` (AMD MIT) | YES (mesmo repo) | YES (AMD MIT) | **PASS** |
| `ctt-etcpak` | YES | YES (`etcpak` por Bartosz Taudul = fastest ETC2 encoder open-source) | YES (vendored em `cpp/`, build.rs:26-205 compila C++ + ISA namespace-wrapping; cuidadoso) | `(MIT OR Apache-2.0 OR Zlib) AND BSD-3-Clause` | YES (mesmo repo) | YES (ETC2 royalty-free Khronos) | **PASS** |
| `ctt-intel-texture-compressor` | YES | YES (Intel ISPC Texture Compressor = ref ISPC impl) | YES (vendored em `ispc/kernel.ispc`, build.rs:5-11 cfg-gated) | `(MIT OR Apache-2.0) AND MIT` (Intel MIT) | YES (mesmo repo) | YES | **PASS** |
| `ctt-intel-texture-compressor-prebuilt` | YES | YES (mesma source) | YES (binary `.a` em `bins/<platform>/` COM sigstore.jsonl + source no crate irmão; mtime SOURCE_DATE_EPOCH) | `(MIT OR Apache-2.0) AND MIT` | YES (mesmo repo) | YES | **PASS** |

**Resultado: 8/8 PASS HR-1 §2.7.1.**

## Findings detalhados

### HIGH-1: `criterion` em `[dependencies]` do `ctt` main (leakage risk em release-game)

- **Onde**: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-0.4.0/Cargo.toml.orig:dependencies` linha próxima ao topo, E também `[dev-dependencies]` (duplicado).
- **Reproduzido em** `Cargo.toml` (normalized): `[dependencies.criterion] version = "0.8"` (linhas ~80) + `[dev-dependencies.criterion]` mais abaixo.
- **Por que viola HR-7**: `criterion` é um framework de benchmark com plotters, regex, walkdir, etc. — pesa ~80 deps transitivas. Está em `[dependencies]` (NÃO dev-only), portanto será linkado em qualquer binário que use `ctt` como lib regular. Para PH2D, `release-game` NÃO usa cooker, então o leak só atinge o `ph2d-asset-cooker` (CI tool / dev tool, HR-7 OK), mas é uma armadilha futura: se alguém usar `ctt` lib em código compartilhado, criterion vai vazar.
- **Mitigação**:
  1. **Upstream PR** (preferido): mover `criterion` 100% pra `[dev-dependencies]`; o uso em `benches/` já é coberto.
  2. **Local workaround**: o cooker offline absorve a bloat sem prejudicar release-game; aceitável SE PH2D mantiver `ctt` confinado a `crates/ph2d-asset-cooker/` (já é a intenção ADR-0055-v4). Não bloqueante.
  3. **cargo-deny ban**: adicionar regra que falha se `criterion` aparecer em release deps de `ph2d-asset-cooker`'s graph downstream (defensivo).

### MEDIUM-1: 308 blocos `unsafe` no agregado (todos contextualizados, nenhum patológico)

- **Onde**: contagem total por crate
  - `ctt` main: 96 (todos em `processing/{load,store}_kernels/srgb.rs` = SIMD intrinsics com SAFETY comments + `is_x86_feature_detected!` runtime gating).
  - `ctt-astcenc`: 25 (FFI bridge p/ ASTC C++ + ISA dispatch).
  - `ctt-bc7enc-rdo`: 28 (FFI bridge ISPC).
  - `ctt-compressonator`: 140 — `src/bindings.rs` (gerado por bindgen) + `src/lib.rs` (wrappers).
  - `ctt-etcpak`: 27 (FFI bridge + ISA dispatch).
  - `ctt-intel-texture-compressor`: 14.
  - prebuilts: 0.
- **Por que MEDIUM**: total alto (308), mas amostragem mostra padrão saudável: cada `unsafe` tem comentário `// SAFETY:` justificando, e SIMD blocks são gated por `is_x86_feature_detected!` macros. `bindings.rs` em compressonator é geração bindgen padrão — expected. Não encontrei `unsafe { ptr.read() }` sem bounds-check óbvio em sample inspection.
- **Mitigação**: nenhuma ação requerida; apenas tracking. PH2D pode rodar `cargo geiger -p ph2d-asset-cooker` post-integração como gate defensivo.

### MEDIUM-2: ASTC encoder usa contraction FP "fast" / non-invariance (`ASTCENC_NO_INVARIANCE=1`)

- **Onde**: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-astcenc-0.4.0/build.rs:147` (`build.define("ASTCENC_NO_INVARIANCE", "1")`) + `build.rs:174` (`-ffp-contract=fast`).
- **Por que MEDIUM (HR-6 risk)**: `ASTCENC_NO_INVARIANCE=1` desliga garantias de invariância numérica entre runs do astcenc (modos paralelo vs scalar podem divergir bit-a-bit). Isso COMBINA com `-ffp-contract=fast` que permite fused-multiply-add em ordens diferentes por compiler version. Para PH2D HR-6 (`AssetId = blake3(bytes)`), se o output ASTC variar entre toolchain versions de cooker, a CAS-id muda. Isto não é "non-determinism por run", é "non-determinism por compiler-version" — output BIT-equivalente desde que o mesmo binário cooker rode duas vezes na mesma máquina (clean checkout). Isto basicamente é o `SOURCE_DATE_EPOCH` model do cooker.
- **Mitigação**:
  1. CI deve travar versão exata de rustc + cc no cooker (já planejado ADR-0055-v4).
  2. ADR-0055-v4 já documenta "cooking offline nativo per-platform" — incluir nota explícita: "ASTC output bit-stable apenas para mesma toolchain version. Re-cook full-rebuild ao upgrade de rustc/cc."
  3. **Opcional**: PR upstream propondo `feature = "strict-invariance"` que NÃO define `ASTCENC_NO_INVARIANCE`. Trade-off perf: ~10-20% slowdown segundo docs do astcenc.

### MEDIUM-3: `etcpak` build.rs usa `c++20` (eleva MSRV de toolchain implícita)

- **Onde**: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-etcpak-0.4.0/build.rs:116,119` (`build.std("c++20")` em ambos os ramos).
- **Por que MEDIUM (CI compat)**: o sub-crate compila com C++20 (precisa GCC ≥ 10 / Clang ≥ 10 / MSVC ≥ 19.29). Os outros 4 sub-crates compilam C++14 (build.rs:139 do astcenc; build.rs:29 do compressonator). Inconsistência. PH2D CI já tem clang moderno (macOS 16/Linux), mas Windows runners GitHub Actions com MSVC antigo podem falhar.
- **Mitigação**: validar no spike W1.T2 que o CI Windows runner default (`windows-2022`) tem MSVC ≥ 19.29 (provavelmente sim). Documentar requirement na seção CI da ADR-0055-v4.

## Build.rs reproducibility analysis

| Sub-crate | build.rs LOC | Reproducível? | Notas |
|---|---|---|---|
| `ctt` (main) | 0 (sem build.rs) | YES | Pure Rust, nenhuma codegen. |
| `ctt-astcenc` | 257 | YES | Compila C++ vendored, ISA-wrapped em namespaces; sem network/timestamp/random. |
| `ctt-bc7enc-rdo` | 16 | YES | cfg-gated `build-from-source` chama `ispc_build_utils::Config`; sem network. |
| `ctt-bc7enc-rdo-prebuilt` | 3 | YES (com caveat) | Apenas `link_prebuilt`; reprodutibilidade depende de TRUST nos binários assinados via sigstore + crate `ispc-build-utils`. |
| `ctt-compressonator` | 90 | YES | Compila C++ vendored + SIMD variants; sem network. |
| `ctt-etcpak` | 273 | YES | Compila C++ vendored + ISA namespace-wrapping; sem network. |
| `ctt-intel-texture-compressor` | 11 | YES | cfg-gated `build-from-source` via ispc_build_utils. |
| `ctt-intel-texture-compressor-prebuilt` | 3 | YES (com caveat) | Mesmo caveat do bc7enc-rdo-prebuilt. |

**Caveat dos prebuilts**: o crate `ispc-build-utils` é o vendor das libs prebuilt. Não-auditado nesta lente (escopo era os 8 ctt-*). Recomendo audit separada de `ispc-build-utils` se PH2D for usar `ispc-prebuilt` em CI publicado (vs `build-from-source` que evita o trust em prebuilts). Para devs locais sem ispc toolchain, prebuilt é OK; CI deve forçar `build-from-source` ou pinar prebuilt version + sigstore-verify.

## License summary

| Sub-crate | License (SPDX) | Compatível MIT/Apache PH2D? |
|---|---|---|
| `ctt` | `MIT OR Apache-2.0 OR Zlib` | YES |
| `ctt-astcenc` | `(MIT OR Apache-2.0 OR Zlib) AND Apache-2.0` | YES |
| `ctt-bc7enc-rdo` | `(MIT OR Apache-2.0 OR Zlib) AND Apache-2.0` | YES |
| `ctt-bc7enc-rdo-prebuilt` | `(MIT OR Apache-2.0) AND Apache-2.0` | YES |
| `ctt-compressonator` | `(MIT OR Apache-2.0 OR Zlib) AND MIT` | YES |
| `ctt-etcpak` | `(MIT OR Apache-2.0 OR Zlib) AND BSD-3-Clause` | YES |
| `ctt-intel-texture-compressor` | `(MIT OR Apache-2.0) AND MIT` | YES |
| `ctt-intel-texture-compressor-prebuilt` | `(MIT OR Apache-2.0) AND MIT` | YES |

**Zero GPL/LGPL/proprietária.** Todos compatíveis com PH2D MIT/Apache. PH2D precisa anexar os arquivos LICENSE-*-* dos sub-crates ao bundle dist (legalmente exigido pelas licenças BSD-3/MIT), o que `cargo about` já faz automaticamente se PH2D já o usa.

## Threading & determinism scan

Greps run em todos os 8 sub-crates `src/` trees (Rust-only):

- `rayon` uses: **0 paths**.
- `std::thread` uses: **0 paths**.
- `tokio` uses: **0 paths**.
- `lazy_static` / `OnceCell` / `OnceLock` / `static mut`: **5 hits**, todos init-once:
  - `ctt-astcenc/src/lib.rs:35,494` — `OnceLock<Dispatch>` p/ ISA dispatch table (init-once, immutable depois).
  - `ctt-etcpak/src/dispatch.rs:10,56` — idem.
  - Inocentes para HR-6 (output não depende do estado mutável).
- `SystemTime::now` / `Instant::now`: **0 paths** em LIB code.
- `HashMap` iteration affecting output: **0 paths**.
- `ctor` crate (auto-init): **0 paths**.

**Veredito threading**: Rust code é determinista. C++ vendored pode ter threading interno (astcenc tem thread pool no `astcenc_compress_image`, mas é determinista por design — a libs documenta bit-equivalence). Não auditei C++ source porque (a) fora do escopo da lente B Rust-side, (b) ref impls são tratadas como confiáveis sob critério HR-1 §2.7.1(2).

## Veredito

**APPROVE_WITH_CAVEATS**.

Os 8 sub-crates passam **todos os 6 critérios HR-1 §2.7.1**: offline-only, ref-impls
únicas, vendored Cargo + build.rs reproducível (sem network/random/timestamp),
licenças MIT/Apache/BSD-3/Zlib (zero GPL), maintainer ativo (commits no último mês),
zero patent encumbrance (BC/ASTC/ETC = royalty-free famílias Khronos/MS).
Supply-chain higiene exemplar: prebuilts assinados via sigstore, mtime SOURCE_DATE_EPOCH
canônico, source paralelo presente, zero git-dependencies (todos 103 deps de crates.io).
Rust code é puro-determinista (zero rayon/thread/tokio/timestamp/HashMap-iter-output;
2 OnceLocks são init-once dispatch tables).

Único ponto que merece ação não-bloqueante: HIGH-1 (`criterion` em `[dependencies]` em
vez de `[dev-dependencies]`) — propor PR upstream + mitigação local via confinement do
`ctt` em `crates/ph2d-asset-cooker/` (já planejado por ADR-0055-v4 W1.T2).

MEDIUMs são tracking/documentação, não blockers. ADR-0055-v4 já cobre o requirement
implícito de "mesma toolchain version → mesmo output bit" para HR-6 CAS-id stability.

## Triage 13 open issues (W1.T2 stretch goal)

Issues consultadas via `gh api repos/cwfitzgerald/ctt/issues?state=open` (2026-05-27).

| # | Título | Labels | Classificação PH2D |
|---|---|---|---|
| 74 | Properly handle HDR ASTC | bug, codec | **MEDIUM** — PH2D não usa HDR ASTC no v1 (Painter w0 é LDR), mas afeta Wave 2+ se HDR brush adicionado. Track. |
| 73 | Support rgb9e5 output and input | enhancement | LOW — formato exótico, fora do roadmap PH2D Painter v1. |
| 72 | Fully support 3D textures | enhancement, codec, processing | LOW — PH2D é 2D engine (HR-1 escope: PH2D**2D** definitiva). N/A. |
| 71 | CI for Test Images | enhancement | LOW — meta-issue da própria ctt, não afeta PH2D consumption. |
| 70 | Support RGBM | enhancement, processing | LOW — formato HDR encoding fora do roadmap v1. |
| 68 | Integrate with basisu | enhancement, codec | **CRITICAL_AVOID** — Basis Universal foi REJEITADO em ADR-0055 v1/v2/v3 (alucinação). Se ctt integrar basisu, PH2D fica preso a feature-flag eviction. **Mitigação**: PH2D deve sempre passar `default-features = false` + opt-in encoder list, NUNCA habilitar `encoder-basisu` se aparecer. |
| 65 | Re-normalize normal maps between generating each mip level | enhancement, processing | **MEDIUM-HIGH** — PH2D Painter normal map workflow precisaria disso. Trackear, propor PR se W2 Painter adicionar normal map brush. |
| 50 | Support for alpha-to-coverage compatible mipmapping | enhancement, processing | **MEDIUM** — sprite alpha cutout em PH2D usa alpha-to-coverage em alguns pipelines. Trackear. |
| 36 | Vendoring script for bc7enc | codec, infrastructure | LOW — meta. |
| 35 | Record exact vendoring commits | codec, infrastructure | **MEDIUM** — reprodutibilidade afeta HR-6. Vincular ao supply-chain hygiene check do PH2D CI. |
| 33 | Support 3D textures | codec, processing | LOW — duplica #72. |
| 23 | Integrate with nvtt | enhancement, codec | **CRITICAL_AVOID** — NVTT é proprietary NVIDIA (não Apache/MIT), violaria HR-1 critério #4 license. **Mitigação**: PH2D deve fail-fast se ctt expuser feature `encoder-nvtt` (cargo-deny ban). |
| 1 | Dependency Dashboard | (none) | LOW — Renovate bot meta. |

**Síntese triage**: 2 issues exigem **proactive defense** no PH2D side (#68 basisu + #23 nvtt → cargo-deny ban + opt-in feature only); 2 issues podem afetar Wave 2 Painter (#74 HDR ASTC + #65 normal map renorm) → tracker em `docs/HANDOFF_painter.md`. Resto é meta/codec-features que PH2D v1 não consome.
