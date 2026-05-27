# ADR-0055 — Cooked Texture Compression Pipeline

**Status:** Accepted (v4 enxuta, 2026-05-27)
**Data:** 2026-05-27
**Decisor:** Enio (decisão) + Claude (arquiteto)
**Substitui:** v1 (deletada 2026-05-26, KTX2+Basis+ACEScg, 12 CRITICAL findings); v3 Round 3+4 (arquivada em [`docs/archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md`](../../archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md), 660 LOC com snippets de código e 13 vapor dependencies, nunca Accepted).
**Estende:** ADR-0021 (sim/present boundary), ADR-0025/0028 (gameobject/codegen), ADR-0026 (sprite source strategies), ADR-0040 (tool isolation — cooker em `tools/asset-cooker`), ADR-0042 §2.3 (`ph2d-color` cap 2500 LOC), ADR-0050 (DeviceCapability/GpuId), ADR-0053 (DeviceTier = 5 FROZEN), ADR-0054 (`ph2d-imageio::ColorProfile` = 8 FROZEN — única materializada).
**Plano vivo (specification canônica):** [`docs/plans/2026-05-texture-compression-waves.md`](../../plans/2026-05-texture-compression-waves.md). Detalhes táticos (símbolos exatos, gates executáveis, fixtures, vapor dependencies E1..E13, ordering de waves) vivem lá — este ADR registra apenas a **decisão estratégica**.

---

## 1. Contexto

PH2D precisa shipar texture compression pipeline GPU-comprimido em 4 plataformas (Desktop, iPad/iOS, Android, Web). Restrições duras:

- **Zero CPU spike em load** (engine 2D pro tool com canvas dinâmico não pode pagar 1-5ms/texture transcoding runtime).
- **Pixel-perfect** (sem dupla compressão lossy).
- **Supply chain Cargo-estável** (HR-1 espírito; sem download manual de binários C++ separados).
- **iOS App Store sem fricção** (PrivacyManifest para FFI C++ in-process é audit overhead).
- **Determinismo content-addressed** (HR-6: `AssetId = blake3(bytes)`).
- **Painter critical na mobile** (brush atlas R8 → BC4 = -50% VRAM; UI assets → ASTC LDR).

Fase 1 entregou codec puro `crates/ph2d-asset-ktx2/` (parser read-only KTX2, 26 tests, `#![forbid(unsafe_code)]`, 1207 LOC). Sem clientes ainda — design intencional. Esta Fase 2 decide como cookar source assets → KTX2 e como o runtime consome.

### 1.1 Por que este ADR é enxuto

Tentativas anteriores (v1 abortada, v3 Round 3+4 archived) escalaram em densidade técnica até 660 LOC com snippets de código `pub fn` e 13 vapor dependencies (E1..E13). Auditoria adversarial 4-rounds (scores 4.5 → 6.5 → 6.2, padrão-ouro 9.0/10 nunca atingido) trocou classe de drift por round (externos → internos → cross-doc → gates-vapor) sem convergir. Diagnóstico convergente de 3 LLMs externas (consulta 2026-05-27 noite): over-specification em domínio sem oráculo (Goodhart's Law), perfeccionismo deslocado de código para documento. Solução: ADR retorna ao papel original (registro de decisão estratégica ~150 LOC); detalhes táticos migram para plano vivo onde tem oráculo (código Rust executável via `cargo check`). Regra `[[feedback-perfection-no-deferrals]]` refinada com escopo decisão-atual vs decisões-adjacentes.

## 2. Decisão

PH2D adota **cook offline + direct upload runtime** como pipeline canônico para texture compression:

- **Container:** KTX2 (parser puro Rust read-only já entregue em Fase 1).
- **Cooker:** Rust crate `ctt v0.4.0` (Connor Fitzgerald — wgpu core contributor) integrada em `tools/asset-cooker/` como lib API. Multi-encoder unificado vendored: bc7e ISPC + astcenc + etcpak + Compressonator + Intel ISPC. Encoders C/C++ aceitáveis sob critério HR-1 §2.7.1 (offline-only, ref impl única, vendored Cargo, license MIT/Apache/Zlib, maintainer ativo, não patent-encumbered).
- **Compressão por tier:** BC7 desktop · ASTC 6×6 LDR mobile · ETC2 Android fallback · BC4 single-channel atlases (brush/grain/mask). HDR (BC6H · ASTC HDR) deferido para Wave 4+ até Painter exportar HDR real.
- **Identidade multi-tier:** cooker emite N artefatos por source (1 per platform tier). N AssetIds blake3 distintos preservam HR-6 content-addressed. Mapping externo `LogicalTextureId → tier → AssetId` resolve no renderer sem amendar AssetDb.
- **Determinismo:** cook em runner canônico único (Linux x86_64 GitHub Actions); KTX2 outputs versionados via Git LFS; cross-OS bit-exactness NÃO assumida (ISPC SIMD encoders conhecidamente não bit-exact cross-arch). Replay-hash gate vira "Linux ≡ Linux mesma SHA do `ctt-cli`" + 5 cooks consecutivos consistency check.
- **Runtime:** renderer lê KTX2 → `wgpu::queue::write_texture` direto. **Zero transcoding runtime**, zero CPU spike no load path, zero `transcoder.wasm` no bundle Web.
- **Color:** Linear sRGB working space + ACES tonemap shader no output. **Sem ACEScg gamut.** `ph2d-imageio::ColorProfile` (ADR-0054, 8 FROZEN) é single source-of-truth. Este ADR **NÃO** materializa `ph2d-color::ColorProfile` (vapor em ADR-0051), **NÃO** cria `ph2d-color-pipeline` paralelo, **NÃO** amenda nenhum cap FROZEN.
- **Platforms reais (target matrix):** iPad Apple7+/M1+ obtém BC opcional (runtime feature query source-of-truth); iPhone só ASTC (Metal-iOS não expõe BC); macOS (Intel + Apple Silicon) obtém BC; Web depende de adapter (BC → ASTC → ETC2 → RGBA8 ladder).

## 3. Alternativas rejeitadas

- **Basis Universal runtime transcoder** (v1 ADR-0055 abortada): 1-5ms/texture CPU spike + dupla compressão lossy (UASTC → BC7/ASTC) + 500KB transcoder.wasm + maintainer dormente (basis-universal-rs 0.3.1, Nov/2023, individual `aclysma`) + iOS PrivacyManifest FFI C++ in-process. Trade-offs derrotam vantagem de "1 source format universal".
- **toktx (Khronos C++ CLI) ou Compressonator (AMD CLI) via download binário**: HR-1 espírito violado (não Cargo-managed); CI setup frágil. Disponível como fallback B do plano vivo se `ctt` falhar W1.T2 audit.
- **Single-tier (1 KTX2 universal por source)**: força lowest-common-denominator (ETC2 ou RGBA8 uncompressed); desperdiça VRAM saving em desktop/iPad recent.
- **In-process FFI C++ libbasisu em release runtime**: HR-1 letra violada + iOS App Store manifest + spike na supply chain. Rejeitado.
- **ACEScg working space + criar `ph2d-color-pipeline`**: nenhum 2D game shippado em ACEScg working encontrado em pesquisa 2026-05-27; ACES tonemap (output) vs ACEScg gamut (working space) eram confundidos. Unity HDRP / Unreal default = Linear sRGB working + ACES tonemap. PH2D segue convenção.

## 4. Consequências

- **`Asset` enum (`crates/ph2d-asset/src/asset.rs`)** ganha variant `TextureKtx2` (`#[non_exhaustive]` já preserva downstream).
- **`SpriteSource` enum (`crates/ph2d-render/src/sprite.rs`)** ganha variant `CookedTexture` (breaking — exige `#[non_exhaustive]` + bump `Sprite::VERSION 3→4` + mirror chain sync em editor-core).
- **`tools/asset-cooker/`** ganha sub-command `texture cook` + lib API (consumível pelo Painter Export Cooked Texture).
- **Renderer (`crates/ph2d-render/`)** introduz mapping `Ktx2Format → wgpu::TextureFormat + Features` + pipeline-per-format selection. `wgpu::queue::write_texture` direto, sem transcoding.
- **HR-1 pure-Rust core preservado**: `ctt` é offline-only (developer machine ou CI), nunca embarcado em release-game bundle. Critério §2.7.1 codificado em SKILL.
- **HR-6 content-addressed preservado**: multi-tier identity via mapping externo `LogicalTextureId`, sem amendar AssetDb.
- **HR-13 budget**: renderer declara orçamento de cache comprimido via API real `MemoryBudget` existente (Plugin trait permanece vapor — não materializado neste ADR; entry E3 do plano vivo §Open Issues).
- **`ph2d-color` cap 2500 LOC preservado**: 1003 atual + ADR-0051 expansões previstas (~1200) + helper ACES opcional (~200) = ~2400 ≤ 2500. Monitorado pré-W3.
- **Painter integration W3**: brush shape/grain atlas R8 → BC4 (-50% VRAM, conta `R8 8 bpp ÷ BC4 4 bpp = 0.5`); UI assets → ASTC LDR; Export Cooked Texture dialog (HR-7 editor-feature-gated, HR-15 Fluent strings, HR-17 Luau example).
- **13 vapor dependencies (E1..E13)** catalogadas em §Open Issues do plano vivo, cada uma com owner identificado (ADR slot reservado, wave de resolução, ou wontfix). NÃO bloqueiam W1.T0. NÃO contam como vapor desta ADR — são dependências adjacentes legitimamente diferidas conforme regra refinada [[feedback-perfection-no-deferrals]].

## 5. Gates de arquitetura (Architecture-as-Code, não declarativos)

Cada cap/contrato/invariante deste ADR deve virar **teste Rust executável** em `crates/*/tests/architecture_*.rs` antes da wave correspondente fechar. Plano vivo §Architecture Gates rastreia estado. Gates declarativos sem enforcement real são banidos — se um cap não puder virar teste, é flagged em §Open Issues até virar.

Estado-alvo (lista informativa; especificação fica no plano vivo):

- W1: cooker determinism (5 cooks consecutivos mesmo input → blake3 igual em mesmo runner).
- W1: `Ktx2Format → wgpu::TextureFormat` mapping exaustivo (match sem `_ =>` wildcard).
- W2: renderer cache budget aggregator soma cooked delta + tools + core baseline contra `Platform::max_total_mb()`.
- W3: Painter Export dialog HR-7 release-game gate (ignored hoje, un-ignore quando feature materializar — E4 do plano vivo).
- Cross-cutting: `ctt-cli` SHA + flags pinned em manifest; canonical-runner hash registry em `assets/cooked-hashes.lock`.

## 6. Política de amendment

- **"Accepted"** desta ADR significa aceitação da **decisão estratégica** (cook offline + KTX2 + ctt + multi-tier identity + direct upload + Linear sRGB working + ACES tonemap). NÃO significa que toda symbol/API/cap mencionado está implementado — implementação roda em W1+ no plano vivo.
- **Amendments à decisão estratégica** (ex: trocar `ctt` por toktx fallback B) exigem nova ADR (ADR-0055.1 ou ADR-NNNN).
- **Detalhes táticos** (símbolos exatos, fixtures, ordering, vapor dependencies resolution) mudam livremente no plano vivo sem amendment formal.
- **HDR Wave 4+** ratifica via amendment ADR-0055.1 quando Painter exportar HDR real + ecossistema de criação destrancar.

---

## 7. Histórico (auditoria pré-v4)

ADR-0055 passou por 4 rounds de auditoria adversarial antes de chegar à v4 (2026-05-27):

- **v1 (deletada 2026-05-26)**: stack KTX2 + Basis Universal runtime + BC6H + ACEScg + criar `ph2d-color-pipeline`. Score 5.67/10 (12 CRITICAL findings — alucinações externas verificáveis em segundos).
- **v3 Round 1 (manhã 2026-05-27)**: Opção E remediada. Scores 4.5/4.5/5.0 — 13 CRITICAL convergentes (`ColorProfile` vapor, MSRV errado, Asset/SpriteSource shape falso, ADR-0009 fantasma).
- **v3 Round 2 (manhã 2026-05-27)**: remediou os 13. Scores 6.5/6.5/5.2 — 6 NOVOS CRITICAL **internos** (`Ktx2Blob` vs real `Ktx2Image`, `DeviceTier` vapor, `MemoryBudget::Render` triplo-vapor, `AssetDb::resolve_for_tier` inexistente, parser kvd claim falso). Padrão "trocar classe de drift por round" identificado.
- **v3 Round 3 (tarde 2026-05-27)**: sweep-grep preventivo Coord-A + 12 fixes inline + tabela canon de 22 símbolos verificados. Scores 6.5/6.2 — Lente D (drift cross-doc) + Lente E (gates declarativos vapor).
- **v3 Round 4 mecânico**: drift Lente D fixado; Lente E findings deferidos para §Open Issues. ADR permanece Proposed, NÃO Accepted.
- **v4 (noite 2026-05-27, este doc)**: 2ª opinião de 3 LLMs externas convergiu em diagnóstico (Goodhart's Law + over-specification sem oráculo) e recomendação (Opção 4: ADR enxuto + plano vivo canônico). Reescrita strategic-only ~200 LOC; tabela canon migrada para plano vivo §Symbol Registry; v3 arquivada como histórico do raciocínio.

**Memórias relacionadas:**
- [[ktx2-phase1-done-phase2-aborted-2026-05-26]] — v1 abortada
- [[project-ktx2-phase2-round3-proposed-2026-05-27]] — v3 estado pre-v4
- [[no-industrial-claims-without-verification]] — verificações externas (R1 falha-classe)
- [[feedback-audit-internal-state-grep]] — verificações internas (R2 falha-classe)
- [[feedback-audit-lens-diversity]] — rotação de lentes
- [[feedback-perfection-no-deferrals]] — refinada 2026-05-27 com escopo decisão-atual vs decisões-adjacentes
