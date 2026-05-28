# HANDOFF — KTX2 Texture Compression (Fase 2 — retomar)

**Status:** ABERTO. Fase 1 entregue 2026-05-26 (codec puro). Fase 2 (integração com `Asset::*` + `ph2d-render` + cooker offline) ABORTADA pós-auditoria 4-lente em 2026-05-26. Aguarda Coord-A futura retomar com pesquisa fresca seguindo este handoff.

**Decisor:** Enio (ratificação do path Opção E em 2026-05-26).
**Audiência:** Coord-A LLM que vier retomar texture compression.
**Última auditoria:** limpeza pós-aborto confirmou repo LIMPO (zero contaminação residual em ADRs 0050-0054, planos, HANDOFFs, SKILL §11.10, `tools/asset-cooker`, `crates/ph2d-color`, Cargo.lock).

---

## §0 Sanity check obrigatório antes de começar

Independente do que esteja escrito aqui, **sempre** rode primeiro:

```bash
# 1. Estado canônico
git log --oneline -5
git status -sb
cargo check --workspace 2>&1 | tail -5

# 2. RE-RODAR audit de limpeza (recomendado pelo auditor 2026-05-26)
#    Confirma que durante o intervalo entre esta sessão e a retomada,
#    nenhum agente paralelo introduziu artefatos do stack abortado.
grep -rn -E "ph2d-asset-basisu|ph2d-color-pipeline|basis-universal|UASTC|BasisLZ|AcescgLinear" \
  --include="*.md" --include="*.rs" --include="*.toml" \
  --exclude-dir=target --exclude-dir=.git \
  /Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/ 2>/dev/null

# 3. Memórias-âncora (auto-loaded mas reler explicitamente):
#    - [[ktx2-phase1-done-phase2-aborted-2026-05-26]] — tabela completa de NÃO-fazer
#    - [[no-industrial-claims-without-verification]] — checklist pre-flight
#    - [[feedback-perfection-no-deferrals]] — padrão-ouro régua de qualidade
#    - [[project-painter-w0-ratified-2026-05-26]] — Painter cascade ratificada hoje
```

Se algo divergir do esperado, **pare e reporte ao Enio** antes de qualquer ação.

---

## §1 TL;DR

PH2D precisa de cooked texture pipeline GPU-comprimido (BC7/ASTC/ETC2/BC6H) para shipar 4 platforms (Desktop, iPad/iOS, Android, Web). Fase 1 (codec KTX2 puro) entregue como crate isolado. Fase 2 (integração com renderer + Asset DB + cooker) foi tentada via stack KTX2+Basis Universal+BC6H+ACEScg e **abortada** após auditoria 4-lente paralela revelar 12 CRITICAL findings (score 5.67/10 vs Painter ratificado mesmo dia 9.0/10). Path canônico recomendado: **Opção E — cooking offline nativo per-platform, sem Basis runtime, reusando `ph2d-color` ADR-0051 sem amendments**.

---

## §2 Fase 1 — ENTREGUE (não tocar)

Crate `crates/ph2d-asset-ktx2/` está completo, isolado, e testado. **NÃO mexer** (codec é alicerce; ampliar via Fase 2 wiring, não reescrever).

| Item | Estado |
|---|---|
| Commits locais (não pushados) | `f30e225` codec base + `db96f28` reject non-2D + `7806369` synth fixture + `b276cef` exhaustive coverage |
| LOC | 1207 lib.rs + 22 Cargo.toml = 1229 total |
| Tests | 24 unit + 2 doctests = **26 verdes** |
| Cobertura | parse container KTX2 · 25 `Ktx2Format` variants (RGBA8/16/32 + BC1/3/4/5/6H/7 + ASTC 4×4..8×8 + ETC2 RGB/RGBA) · limits MAX_DIMENSION=8192 / MAX_TOTAL_BYTES=512MiB / MAX_LEVELS=16 · reject 3D/cubemap/array · 8 error variants exercised (exceto `TotalBytesExceeded` documentado como inviável a custo razoável) |
| Quality gates | `cargo clippy --all-targets -- -D warnings` clean · `cargo fmt --check` clean |
| Deps | `ktx2 = "0.5"` (pure-Rust read-only parser) + `thiserror = "2"` |
| Safety | `#![forbid(unsafe_code)]` — precedente para outros `ph2d-asset-*` |
| Cliente atual | **Zero.** Crate está "ilhado" — design intencional. Fase 2 conecta ao Asset DB + renderer. |

---

## §3 Fase 2 ABORTADA — anti-patterns a NÃO repetir

Detalhe completo do raciocínio em [[ktx2-phase1-done-phase2-aborted-2026-05-26]]. Resumo operacional dos **11 anti-patterns** que afundaram o ADR-0055 deletado:

1. **NÃO criar `ph2d-asset-basisu`.** Runtime transcoder FFI C++ é overengineering para 2D engine + dupla compressão lossy destrói qualidade pro tool. Renderer lê KTX2 → `queue.write_texture` direto.

2. **NÃO criar `ph2d-color-pipeline`.** ADR-0051 já tem mandato de expandir `ph2d-color` (cap LOC ≤ 2500; atual 1003 LOC = 60% margem). Adicionar matemática ACES dentro de `ph2d-color` se necessário.

3. **NÃO afirmar `basis-universal-rs >= 0.4`.** Versão real publicada é **0.3.1 (nov/2023, dormente ~2 anos)**. Maintainer individual `aclysma`, não "gfx-rs ecosystem". Verificar com `cargo search basis-universal` antes de qualquer afirmação.

4. **NÃO assumir BC6H universal.** Apple Metal-iOS **não expõe BC formats** (iPhone/iPad rodando iOS/iPadOS — confirmar em Apple Metal Feature Set Tables). Apenas macOS expõe BC em Apple Silicon. HDR mobile = ASTC HDR (Vulkan 1.3+ Android, Apple Silicon iPadOS).

5. **NÃO escrever `-50% VRAM` (BC7 vs RGBA8).** Conta correta: `BC7=8 bpp ÷ RGBA8=32 bpp = 0.25` → **-75% saving**. ASTC 6×6 (3.56 bpp) = -89%. Mostrar a conta sempre no texto do ADR.

6. **NÃO amendar `ColorProfile` cap.** **Existem DOIS `ColorProfile` distintos** — ADR-0051 Painter (8 FROZEN: Srgb/LinearSrgb/DisplayP3/LinearDisplayP3/ProPhoto/Rec2100Pq/AdobeRgb + headroom) com gate `color_profile_variant_count_is_exact_8`, e ADR-0054 ImageIO (8: Srgb/DisplayP3/AdobeRgb/ProPhoto/LinearRec709/LinearRec2020/Custom/Unknown). Amendar um sem o outro derrota FREEZE silenciosamente. Solução: NÃO amendar — usar como estão.

7. **NÃO override "pure-Rust only" sem critério objetivo.** ADR-0054 §1.1 rejeitou HEIC porque libheif é C++ FFI. Aceitar libbasisu C++ FFI no mesmo dia abre slippery slope. Se for necessário override no futuro, listar critério objetivo aplicável a ADRs futuras (e.g., "FFI C++ permitido SE: reference impl única + presentation-only + binary < 1 MB + crate Rust wrapper maintained < 12 meses + não patent-encumbered").

8. **NÃO citar ADR-0009 como existente.** `ls docs/architecture/decisions/0009-*.md` retorna no matches. SKILL §16 lista ADR-0009 como slot reservado "esperado" (Holographic Radiance Cascades — não escrita ainda desde initial commit). Referenciar como dependência só depois de existir.

9. **NÃO afirmar adoção industrial sem WebFetch oficial.** ADR-0055 falsamente afirmou "Unity 2022.3+ KTX2+Basis default" / "Unreal 5.3+ KTX2 import suportado" / "Houdini 19+ KTX2 native". Verificação real: Unity 6 docs listam BC7/ASTC/ETC2 nativos; Unreal canonical é Oodle Texture; Houdini SideFX docs não listam KTX2 nativo. KTX2+Basis é padrão **emergente em web 3D / glTF transmission**, NÃO default em game engines mainstream.

10. **NÃO confundir ACES tonemap operator com ACEScg working space.** Unity HDRP usa ACES **tonemap** (output) sobre Linear sRGB **working space**. Unreal idem. **ACEScg working space em 2D games shippados: zero conhecidos.** Para PH2D: usar Linear sRGB working space + ACES tonemap shader no output, sem gamut transforms ACEScg.

11. **NÃO modelar HDR sprite pipeline sem ecossistema de criação.** Procreate/PSD/Krita não exportam HDR mainstream. Radiance Cascades (quando ADR-0009 for escrita) trabalha com SDR sprite × float emissive scalar (Unity 2D Light pattern), NÃO precisa HDR texture source. Wave HDR só faz sentido quando Painter tiver export HDR real.

---

## §4 Caminho canônico recomendado — Opção E

**Princípio:** "melhor para 2D pro tool" ≠ "stack mais complexo de 3D AAA". O melhor é zero CPU spike em load + pixel-perfect + WASM portable + builds puro-Rust estáveis.

### Stack final

| Camada | Decisão | Onde implementa |
|---|---|---|
| **Container** | KTX2 (Fase 1 codec já entregue) | `crates/ph2d-asset-ktx2/` (existe) |
| **Compressão SDR** | BC7 desktop · ASTC LDR mobile · ETC2 Android fallback · BC1 low-end — **cookado offline per-platform** | `tools/asset-cooker` extension (W1) |
| **Compressão HDR** (se priorizado em algum Wave) | BC6H desktop + ASTC HDR mobile — cookado offline, sem Basis layer | `tools/asset-cooker` extension (W2 ou W3) |
| **Apple iOS/iPadOS** | **ASTC apenas** (BC não exposto pelo Metal-iOS). Verificar runtime via wgpu feature query. | `ph2d-render` wgpu caps query |
| **Runtime transcoder** | **NÃO criar.** Renderer lê KTX2 → `wgpu::queue::write_texture` direto. | `ph2d-render` (W2) |
| **Color pipeline** | Usar `ph2d-color` (ADR-0051) expandido. **NÃO criar crate paralelo.** | `crates/ph2d-color/` (existente, 1003/2500 LOC) |
| **Color management** | Linear sRGB working space + ACES **tonemap shader** no output. **NÃO ACEScg gamut.** | shader em `ph2d-render` (W2) ou `ph2d-color` (W0) |
| **`ColorProfile`** | Reusar 8 variants FROZEN da ADR-0051 + 8 variants da ADR-0054 como estão. **NÃO amendar.** Conversor `imageio_profile_to_painter_profile` se necessário. | `crates/ph2d-color/` (W0) |
| **Cooker** | `tools/asset-cooker texture` sub-command chama CLIs externas oficiais (`toktx`/Compressonator) — **NÃO FFI in-process libbasisu**. Determinismo cross-platform via canonical runner (Linux x86_64). | `tools/asset-cooker/src/texture/` (W1) |
| **Painter wins prioritários** | Brush atlases R8 → BC4 (**-50% saving real**, R8=8bpp ÷ BC4=4bpp = 0.5 — corrigido 2026-05-27 pós-ADR Round 1 audit; HANDOFF anterior dizia "4× saving" mas isso só vale se source fosse RGBA8). UI assets/templates → ASTC LDR. Texture compression é Painter-critical na mobile platform. | W3 |

### Por que Opção E vence

| Aspecto | Opção E (recomendada) | Stack abandonado (Basis Universal) |
|---|---|---|
| Load latency | Zero CPU spike (direct upload) | 1-5 ms/texture transcode runtime |
| Qualidade | Pixel-perfect (compressão única offline) | Dupla compressão lossy (UASTC → BC7) |
| Web wasm | -500 KB (sem transcoder.wasm) | +500 KB (transcoder.wasm) |
| Supply chain | Pure-Rust mantido (HR-1 letra E espírito) | C++ FFI dormente (basis-universal-rs 0.3.1) |
| CI | Determinístico (calls binários oficiais selados) | Risco cross-platform CMake (std::sort divergence) |
| iOS App Store | Sem PrivacyManifest libbasisu | Manifesto FFI C++ exige audit |
| ColorProfile | 2 FROZEN preservados | Quebra silenciosa do gate ADR-0051 |

---

## §5 Pre-flight obrigatório antes de escrever ADR novo

Derivado da auditoria 4-lente que afundou o ADR-0055 anterior. Checklist verificável (memória [[no-industrial-claims-without-verification]] tem versão expandida):

```bash
# 1. Estado canônico (também §0)
git log --oneline -5
git status -sb

# 2. Qual número ADR está livre?
ls docs/architecture/decisions/ | sort | tail -5
# (em 2026-05-26 noite, 0054 é último; 0055 está LIVRE — ADR anterior deletado)

# 3. ADRs ratificadas nas últimas 48-72h (podem conflitar)
git log --since="72 hours ago" --oneline docs/architecture/decisions/

# 4. Para cada tipo/crate/módulo que vai mencionar:
grep -rn "<nome>" --include="*.md" --include="*.rs" --include="*.toml" \
  --exclude-dir=target --exclude-dir=.git .

# 5. Para cada dep externa proposta:
cargo search <nome>       # versão real publicada?
cargo info <nome>         # último release date?
# + WebFetch repo upstream — open issues, último commit, maintainer ativo?

# 6. Para cada afirmação de adoção industrial:
# WebFetch ≥ 2 fontes oficiais (não blog posts). Quote da página.

# 7. Para cada redução percentual no texto:
# Escrever a conta EXPLÍCITA no parágrafo: "A bpp ÷ B bpp = ratio → saving %"

# 8. Caps FROZEN em ADRs alvo:
grep -rn "is_exact\|_cap\|FROZEN" \
  crates/<crate>/tests/ docs/architecture/decisions/

# 9. ADRs cited devem existir:
ls docs/architecture/decisions/<numero>-*.md
```

**Vermelhinho-triggers** (PARE e verifique antes de continuar):
- "Eu acho que lembro da versão de X" → `cargo search` agora.
- "X engine usa Y como default" → WebFetch antes.
- "Cap-bump em ADR-NNNN" → grep `*_count_is_exact` cross-repo (especialmente nomes comuns: `ColorProfile`, `AssetId`, `Format`, `Variant`).
- "Override de HR-N / decisão anterior" → critério objetivo aplicável a futuras ADRs ou NÃO escreva.
- "Ganho percentual X%" → conta no parágrafo.

---

## §6 Pesquisa fresca obrigatória antes de propor ADR-0055-Revised

Estas perguntas estão **sem resposta confiável** no momento do handoff. Investigar com WebFetch + `cargo search` + `cat ~/.cargo/registry/src/.../*` antes de escrever ADR.

### Supply chain

1. **`basis-universal-rs`** ainda 0.3.1 dormente? Ou alguém publicou 0.4+ wrapping libbasisu v2.x? — Mesmo se publicou, **Opção E não usa** este crate, mas confirmar pra fechar pergunta histórica.
2. **`JakubValtar/basisu_rs`** (pure-Rust transcoder port) publicado como crate? Maturity? — Importante caso a Coord-A futura escolha trilha híbrida (Basis no cooker offline + pure-Rust transcoder no runtime).
3. **`toktx` (KhronosGroup/KTX-Software)** CLI determinismo cross-platform real? Linux vs macOS vs Windows produzem byte-exact identical `.ktx2` para mesmo input?
4. **`Compressonator` (AMD)** CLI cross-platform binário disponível? Performance vs toktx?
5. **`intel-tex-rs-2`** (ISPC FFI) versão atual, BC7 encoder qualidade vs UASTC? — Opção fallback Rust-friendly se quiser cooker FFI ao invés de CLI externa.

### Platforms reais

6. **Apple Metal iOS 19/20** — algum BC format exposto em hardware Apple Silicon iPad Pro? Confirmar com Apple Metal Feature Set Tables atualizadas.
7. **Android Vulkan 1.3 ASTC HDR coverage** — porcentagem de devices em 2026 com `VK_EXT_texture_compression_astc_hdr` (era ~30% em 2025).
8. **WebGPU compressed-texture extension** — Chrome 130+ / Safari 19+ / Firefox 145+ status real. ASTC em Safari mobile cobre?
9. **iOS PrivacyManifest** — se cooker offline NÃO usa libbasisu in-process, ainda precisa de manifest específico? Provavelmente não — confirmar.

### Painter integration (cliente óbvio)

10. **Brush atlas size** atual em `crates/ph2d-painter-brush/` — confirmado 2026-05-27: Shape atlas 64×256² R8 = **4 MB**; Grain atlas 64×1024² R8 = **32-64 MB** (T1.3 all-bitmap até W5+ Procedural). BC4 saving = **-50%** sobre R8 (não 4×).
11. **Canvas dinâmico** — Painter canvas 8K+ multi-layer usa RGBA8 ou Rgba16Float? Compression offline não ajuda canvas DINÂMICO; foca em atlases/UI/static art.
12. **Painter "Save for Game" UX** — workflow real Procreate-refugee. Target picker? Quality preset? Progress bar?

---

## §7 Wave structure proposta

Estimativas em LOC + sessões de Coord-A. Não-prescritivo — Coord-A futura ajusta após pesquisa §6.

### Wave 0 — ADR + scaffold foundational (Coord-A only)

| Task | Estimate | Bloqueador |
|---|---|---|
| W0.T0 — Investigar §6 + responder perguntas com cites | ~2h pesquisa | nenhum |
| W0.T1 — Escrever ADR-0055-Revised seguindo §5 pre-flight | ~3-4h | W0.T0 |
| W0.T2 — Auditoria N-lente paralela (≥3 lentes) ANTES de marcar Accepted | ~30min/agente × 3 + remediação | W0.T1 |
| W0.T3 — SKILL §11.10 update reconciliando (não "superseded" — "implementado por") | ~30 LOC | W0.T1 |
| W0.T4 — SKILL §12.1 memory budget — provisional numbers + W2 audit gate | ~20 LOC | W0.T1 |
| W0.T5 — `docs/plans/2026-05-texture-compression-waves.md` plano vivo | ~150 LOC | W0.T1 |

### Wave 1 — Cooker offline

| Task | Estimate |
|---|---|
| W1.T1 — `tools/asset-cooker/src/texture/` sub-module | ~300 LOC |
| W1.T2 — CLI wrapper para `toktx` ou `Compressonator` (binário externo) | ~200 LOC |
| W1.T3 — Mip pyramid generation (box / Lanczos / point) | ~150 LOC |
| W1.T4 — `Asset::TextureKtx2` variant em `ph2d-asset` (foundational, MENOR amendment ADR-0054) | ~50 LOC |
| W1.T5 — CLI sub-commands + reproducibility test (HR-6 cross-OS gate) | ~200 LOC |
| W1.T6 — Auditoria 5-lente paralela | gate Coord-A |

### Wave 2 — Runtime path

| Task | Estimate |
|---|---|
| W2.T1 — `wgpu` feature query (BC vs ASTC vs ETC2 vs RGBA8 fallback) | ~150 LOC |
| W2.T2 — `SpriteSource::CookedTexture { asset_id }` variant (amend ADR-0026, conferir cap) | ~100 LOC |
| W2.T3 — Renderer pipeline-per-format (BC7/BC6H/ASTC/ETC2/RGBA8) — wgpu bind group differs por format | ~400 LOC |
| W2.T4 — End-to-end smoke (cook → ship → load → upload → sample) | ~200 LOC |
| W2.T5 — Memory budget HR-13 audit com benchmark real | ~100 LOC + gate |

### Wave 3 — Painter integration (alta prioridade — VRAM critical em mobile)

| Task | Estimate |
|---|---|
| W3.T1 — Brush atlas cooking (R8 → BC4) | ~150 LOC |
| W3.T2 — UI assets cooking (UI textures → ASTC LDR) | ~100 LOC |
| W3.T3 — Painter "Export to Cooked Texture" dialog (target picker, presets, async progress) | ~500-800 LOC |

### Wave 4+ — HDR (deferred até ecossistema de criação existir)

Defer até: Painter ter export HDR real (EXR) + ADR-0009 (Radiance Cascades) shippada e demonstrar uso.

---

## §8 Triagem DIRETRIZ §1.4 (pré-classificada)

```
TRIAGEM
- Tarefa: implementar Fase 2 KTX2 (cooked texture compression pipeline)
- Balde: §3.6 foundational (toca ph2d-asset, ph2d-render, tools/asset-cooker, SKILL)
- Toca contrato congelado? Possivelmente SpriteSource (ADR-0026) — verificar cap antes
- COMO PROCEDER:
    W0 → (C) Coord-only + novo ADR (ratificação + auditoria N-lente)
    W1 → (C) Coord-only — extends cooker foundational
    W2 → (C) Coord-only — renderer foundational + Asset variant
    W3 → (B) Coord scaffold + Implementador para Painter integration painter-specific
- Razão: texture compression pipeline é foundational cross-cutting; toca múltiplos crates ph2d-asset/ph2d-render/tools/asset-cooker/SKILL/ADRs vizinhas
```

---

## §9 Dependências cross-ADR / SKILL

ADRs e docs que a Coord-A futura PRECISA ler **antes** de escrever ADR-0055-Revised:

| Doc | Por quê ler |
|---|---|
| ADR-0021 (sim/present boundary) | confirmar texture cache fica em PresentWorld |
| ADR-0026 (sprite source strategies) | verificar cap de variants antes de adicionar `CookedTexture` |
| ADR-0040 (tool isolation) | confirmar tools/asset-cooker é o local correto (FREEZE 2026-05-22) |
| ADR-0042 (Wave 10 closure) | confirmar mandato de expansão `ph2d-color` (cap LOC 2500) |
| ADR-0051 (Painter ColorProfile) | **8 variants FROZEN** — gate `color_profile_variant_count_is_exact_8` |
| ADR-0053 (cross-platform tier) | usar `DeviceTier` enum existente (5 variants FROZEN), NÃO criar `CookTarget` paralelo |
| ADR-0054 (ImageIO Painter) | consome `DecodedImage::Flat`/`FlatHdr` como source; **8 variants ColorProfile FROZEN no lado ImageIO** |
| SKILL §11.10 | texto canônico atual (BC7+ASTC+ETC2 multi-target) — Coord-A reconcilia, não substitui |
| SKILL §12.1 | memory budget table atual — atualizar com provisional + gate W2.T5 medição real |
| SKILL §HR-1, HR-3, HR-5, HR-6, HR-13, HR-17 | cumprir letra **e** espírito |

---

## §10 Gates de qualidade

Padrão-ouro **9.0/10** (vide Painter cascade 0050-0053 ratificada 2026-05-26).

- **Auditoria N-lente paralela** ANTES de marcar Proposed (mínimo 3 lentes ortogonais — vide [[feedback-audit-lens-diversity]]).
- **Rotação canônica de lentes**: técnico/supply-chain · HR-ADR compliance · industrial reality · WGSL/ABI (se aplicável) · cross-GPU · test-coverage vs verbal claims.
- **Gates executáveis > claims verbais** — toda afirmação técnica vira test ou comentário com cite/quote.
- **`feedback-perfection-no-deferrals` ativo** — gaps conhecidos viram trabalho na sessão, não diferidos.

---

## §11 SESSION_ACTIVE.md

Coord-A futura: **ATIVAR** seção Coord-A no [`docs/SESSION_ACTIVE.md`](SESSION_ACTIVE.md) ao iniciar W0. Pastas reservadas:

- `tools/asset-cooker/` (W1 extension)
- `crates/ph2d-asset/` (W1.T4 variant)
- `crates/ph2d-render/` (W2 pipeline-per-format)
- `crates/ph2d-color/` (W0 reuso, possíveis adições dentro do cap)
- `docs/architecture/decisions/0055-*.md` (ADR novo — número 0055 livre)
- `SKILL_Stack_PH2D_Definitiva.md` §11.10 + §12.1
- `docs/HANDOFF_ktx2_phase2.md` (este arquivo — pode editar status no final da sessão)

---

## §12 Status canônico no fim desta sessão (2026-05-27 noite — v4 Accepted, W0 fechada)

**Diagnóstico que destravou:** 2ª opinião de 3 LLMs externas (consulta 2026-05-27 noite) convergiu em (a) metodologia quebrada para esta classe de ADR (auditoria adversarial sem oráculo = Goodhart's Law), (b) recomendação Opção 4 (ADR enxuto strategic-only + plano vivo canônico). v3 660 LOC com snippets de código gerava vapor verificável e ciclo R1→R4 sem convergência.

- **ADR-0055-cooked-texture-compression-pipeline.md** — `Accepted` v4 enxuta (101 LOC strategic-only, sem snippets de código). v3 660 LOC arquivada em [`docs/archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md`](../archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md) como histórico do raciocínio.
- **`docs/plans/2026-05-texture-compression-waves.md`** — plano vivo é agora **specification canônica**. Inclui §Symbol Registry (22 símbolos verificados migrados da tabela canon do v3), §Anti-Patterns (11 originais + 3 novos sobre over-specification), §Memory Budget Math (contas explícitas), §Open Issues E1..E13 (13 vapor dependencies adjacentes com owner identificado).
- **HANDOFF §4 + §6.10** — patched (BC4 saving correto -50%, não 4×).
- **SKILL §11.10 / §12.1 / HR-1 §2.7.1** — atualizadas em Round 1-3.
- **SESSION_ACTIVE.md** — Coord-A ATIVO (esta sessão); pastas tocadas documentadas (5 docs); NÃO-TOCAR list dos agentes paralelos respeitada (commit escopado).
- **Memória refinada** — `[[feedback-perfection-no-deferrals]]` ganhou escopo decisão-atual vs decisões-adjacentes (anti-inversão da própria regra). `[[feedback-audit-internal-state-grep]]` criada em Round 3.
- **W1.T0 ✅ + W1.T1 ✅ + W1.T2 ✅ fechadas mesma sessão.** Commits locais: `971e237` (v4 + plano + HANDOFF), `db6971c` (T0 cargo add + sweep-grep), próximo commit (T1 cargo check passed + T2 audit consolidado).
- **W1.T2 audit deliverable**: [`docs/audits/ctt-source-audit-2026-05-27-CONSOLIDATED.md`](audits/ctt-source-audit-2026-05-27-CONSOLIDATED.md). Veredito **APPROVE_WITH_CAVEATS** (8/8 sub-crates PASS HR-1 §2.7.1; 0 CRITICAL; 4 HIGH mitigáveis PH2D-side). 5 disciplinas operacionais → 5 sub-tasks novas no plano vivo (T2.1..T2.5).
- **Próxima sessão Coord-A**: ler ADR-0055-v4 + audit consolidado + plano vivo T2.1..T2.5. Implementar D1 (features pinadas), D3 (wrapper guard AMD+BC7+UltraFast), D4 (snapshot test 64 bytes) e D5 (deny.toml ban) **ANTES** de W1.T3 (`tools/asset-cooker/src/texture/mod.rs` sub-command). Estimate ~1 sessão para T2.1+T2.2+T2.3+T2.4; T2.5 (PR upstream) ⏳ defer.

---

## §13 Quando este handoff fica obsoleto

Atualizar este HANDOFF ou substituí-lo quando:

1. ADR-0055-Revised for Accepted (incluir SHA da ratificação).
2. Cada Wave fechar (incluir SHAs dos commits + status testes).
3. SESSION_ACTIVE.md trocar status (Coord-A ATIVO → INATIVO).
4. Arquivar em `docs/archive/handoffs-completed/` quando Wave 3 fechar.

Coord-A futura: edite a §12 ao iniciar e ao final da sua sessão. Não delete este arquivo — atualize.

---

**Boa retomada. A engine está madura, Fase 1 é alicerce sólido, o caminho é claro. Não pule §5 pre-flight nem §0 sanity check.**
