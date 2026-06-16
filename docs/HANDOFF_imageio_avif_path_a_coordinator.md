# HANDOFF — Image I/O W3.T4 AVIF re-ship Path A → **novo Coordenador** (2026-05-28)

**Audiência:** o **Coordenador único** que vai dirigir 3 implementadores (modelo
novo decidido pelo Enio após colisões git entre implementadores paralelos).
**Módulo deste handoff:** **Image I/O pipeline (ADR-0054) — task W3.T4 AVIF
re-ship via Path A** (`image = { features = ["avif-native"] }`).
**Autor:** sessão Coord-A+Implementador imageio que rodou a **verification protocol
completa** e encerrou ANTES de tocar código (per decisão Enio de reestruturar).

> **Sua tarefa, Coordenador:** ler isto, ler a DIRETRIZ, **destravar a decisão de
> escopo §5 com o Enio** (não tem como o implementador começar sem isso), e então
> **escrever um sub-handoff enxuto para o implementador** deste módulo
> (esqueleto pronto na §10) para ele continuar **de onde paramos** — sem refazer
> a verification protocol que já está documentada na §4.

---

## §0 TL;DR de 5 linhas

- **W3.T4 AVIF está DESHIPADO** desde `f034e9a` (audit-15 derrubou `avif-decode = "1"` por 1 RUSTSEC + 6 HIGH + 8 MEDIUM unfixable). Enio autorizou re-ship via **Path A** = `image = { features = ["avif-native"] }`.
- **Esta sessão FECHOU a verification protocol** (`cargo audit`, `cargo tree`, unsafe count, bus-factor, licenses) — números brutos na §4. **Path A passa em RUSTSEC/owning_ref/aom-decode/lodepng** ✓.
- **Único atrito real:** `dav1d-sys` precisa **libdav1d instalado no CI** (3 plataformas — Linux apt / macOS brew / Windows vcpkg). CI atual em `.github/workflows/spike.yml` **não tem** isso. Mudança em workflow = foundational → Coord decide+pergunta Enio.
- **ZERO código mudado nesta sessão.** **ZERO commit nesta sessão.** A pasta `crates/ph2d-imageio-avif/` continua exatamente como em `e905af1` (HEAD) — stub honesto com nota audit-15.
- **Decisão de escopo PENDENTE Enio (§5)**: re-ship decode-only + encode `Error::Unsupported(defer)` + 3 CI installs — implementador **não pode começar** até Enio confirmar (ou rejeitar e escolher Path B/C).

---

## §1 O que é este módulo

Pipeline **Image I/O** (ADR-0054 Accepted). W0..W3 fechado:

| Onda | Estado |
|---|---|
| W0..W2 | ✅ Real decode + encode em 11 formatos (PNG, JPEG, WebP, GIF, .ph2d-native, TIFF, ORA, APNG, PSD-decode, HDR, EXR-decode) |
| W3.T1 (JXL) | ✅ Real decode LDR; encode permanent-defer |
| W3.T2 (EXR) | ✅ Real decode; encode `Error::Unsupported(defer)` |
| W3.T3 (HDR-Radiance) | ✅ Real decode + encode (Inf/NaN guard pós audit-14) |
| **W3.T4 (AVIF)** | ❌ **DESHIPADO** em `f034e9a` (audit-15); stub honesto restaurado |
| W3.T5 (SVG) | ⚠️ Parse-only stub via usvg → `VectorDoc::default()` (espera ADR-0056 vector real body) |

15 rounds de auditoria multi-lente documentados em `docs/architecture/decisions/0054-imageio-pipeline.md` §5 (timeline §2 do HANDOFF_imageio.md). PADRÃO-OURO ✓ — todos os CRITICAL + HIGH fechados a erro-zero per [[feedback-perfection-no-deferrals]].

**Crates do módulo (16 totais):**
- Contract: `crates/ph2d-imageio/` (trait surface FROZEN: `ImageImporter`, `ImageExporter`, `DecodedImage`=5 var, `Error`=11 var, `ExportFormat`=14 var, `ColorProfile`=8 var + `limits.rs` 9 caps).
- Formats: `crates/ph2d-imageio-{png,jpeg,webp,gif,ph2d-native,tiff,ora,apng,psd,hdr-radiance,exr,jxl,avif,svg}/` (14 crates).
- Codegen-output: `crates/ph2d-imageio-registry-init/`.
- Codegen-tool: `tools/ph2d-imageio-sync/`.
- Bridge: `crates/ph2d-asset/src/loader.rs::decode_via_imageio_registry`.

Plano vivo: [`docs/plans/2026-05-imageio-waves.md`](plans/2026-05-imageio-waves.md).
HANDOFF geral (tracker): [`docs/HANDOFF_imageio.md`](HANDOFF_imageio.md) — leia §0–§9.
ADR: [`docs/architecture/decisions/0054-imageio-pipeline.md`](architecture/decisions/0054-imageio-pipeline.md) — em especial §2 (caps), §5.17 (deship AVIF).

---

## §2 Estado das tasks W3 (atualizado 2026-05-28)

| Task | Estado | Commit / Nota |
|---|---|---|
| W3.T0..T0.6 (caps + recursion + NaN + ZIP + EOF) | ✅ | 11 audits remediados |
| W3.T1 (JXL real LDR) | ✅ | wave-2 `5f9582b` |
| W3.T2 (EXR real decode; encode defer) | ✅ | wave-2 `5f9582b` |
| W3.T3 (HDR-Radiance real) | ✅ | wave-2 `5f9582b` |
| **W3.T4 (AVIF real)** | ❌ **DESHIP `f034e9a`** | **← VOCÊ está aqui. Sub-tarefa: re-ship via Path A.** |
| W3.T5 (SVG real vector body) | ⏳ **BLOQUEADO** | precisa amendment a `VectorDoc` (contrato FROZEN); Vector Module session ativa em `crates/ph2d-vector*` |
| W3.T2.1 (EXR encode) | ⏳ DEFER | espera primeiro client real (Painter HDR save demo) |
| W3.T1.6 (JXL real-fixture decode test) | ⏳ DEFER | precisa cjxl CLI offline pra checkin de fixture |

### O que ESTA sessão entregou

**Apenas a verification protocol** (§4 abaixo) + este handoff. **Nada de código** —
Enio cortou pra reestruturar pra modelo de 1 Coord + 3 Implementadores antes do
implementador AVIF começar.

Concretamente:
- `cargo audit` rodado em scratch crate com `image = { version = "0.25", default-features = false, features = ["avif-native"] }` → **EXIT 0, sem RUSTSEC**.
- `cargo tree` analisado: dep tree de 54 packages; sem `owning_ref`, sem `aom-decode`, sem `lodepng`.
- Unsafe budget contado (dav1d-sys=6 + dav1d=49 + mp4parse=0 + moxcms=344 + fallible_collections=133 = **~532**; dominante SIMD em moxcms — categoria diferente do UAF do `owning_ref`).
- Licenças verificadas contra `deny.toml` allowlist (dav1d-sys MIT ✓, mp4parse MPL-2.0 ✓, moxcms BSD-3 OR Apache-2.0 ✓).
- Bus-factor: dav1d-sys=rust-av/Luca Barbato (multi-maintainer) ✓, mp4parse=Mozilla ✓, moxcms=awxkee single ⚠, fallible_collections=vcombey single ⚠.
- Características de `dav1d-sys` build mapeadas: `system_deps` → pkg-config; fallback é git-clone+build de `code.videolan.org/videolan/dav1d.git` 1.5.0 em build.rs (network = ruim).
- CI matrix conferido em `.github/workflows/spike.yml`: ubuntu-latest + macos-latest + windows-latest; instala só `libudev-dev`. **Não tem dav1d**.

**Scratch dir de verificação (descartável):** `/tmp/avif-verify/` — `Cargo.toml` + `lib.rs` vazio só pra `cargo tree`/`cargo audit`. Pode deletar.

---

## §3 Pastas reservadas do módulo (limites anti-colisão)

**O implementador AVIF edita SÓ:**
- `crates/ph2d-imageio-avif/` — `Cargo.toml` + `src/lib.rs` + (criar) `src/decode.rs` + `tests/`
- `docs/HANDOFF_imageio.md` (atualiza §1 format matrix + §2 audit timeline + §3 defesas + §4 defers + §6 test counts + §9 traps)
- `docs/architecture/decisions/0054-imageio-pipeline.md` (acrescenta §5.18 wave-3 re-ship AVIF Path A)
- `docs/plans/2026-05-imageio-waves.md` (status W3.T4 — Path A em curso)

**SÓ COM AUTORIZAÇÃO EXPLÍCITA SUA (Coord), per §5 abaixo:**
- `.github/workflows/spike.yml` — adicionar 3 install steps (libdav1d-dev / dav1d / vcpkg). Foundational/shared. **Pergunte Enio antes**.

**NÃO tocar (outras sessões — confira SESSION_ACTIVE.md ao iniciar):**
- `crates/ph2d-tool-painter/`, `crates/ph2d-painter-*`, `crates/ph2d-panel-painter-sidebar/`, `shells/desktop/src/render_loop/painter_bridge.rs` (Painter Coord-A slot `impl-1` T1.9 ATIVO).
- `crates/ph2d-asset/`, `crates/ph2d-asset-ktx2/`, `tools/asset-cooker/`, `Cargo.lock`, `docs/plans/2026-05-texture-compression-waves.md` (KTX2 Fase 2 W1).
- `crates/ph2d-vector*` (Vector Module W1).
- `crates/ph2d-tool-bgremoval/src/tool.rs` (bgremoval session).
- `docs/Sprite_projeto/`, ADRs `0069..0074*`, `0025-amendment-1` (Sprite Inspector v2 docs-only PRONTO PARA RATIFICAÇÃO).
- Qualquer outra `crates/ph2d-imageio-*` que não seja `-avif/` — todas as outras 13 já estão prontas e não devem regredir.

---

## §4 Verification protocol — completada nesta sessão

Per HANDOFF_imageio.md §4 + §8 (passo 5-lens audit) + [[feedback-no-industrial-claims-without-verification]].

### §4.1 `cargo audit` (scratch /tmp/avif-verify/)

```
Cargo.toml:
[dependencies]
image = { version = "0.25", default-features = false, features = ["avif-native"] }

Resultado:
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1098 security advisories (from /Users/dibrioli/.cargo/advisory-db)
    Scanning Cargo.lock for vulnerabilities (54 crate dependencies)
EXIT=0  → SEM RUSTSEC ✓
```

### §4.2 `cargo tree -e normal --depth 2`

```
└── image v0.25.10
    ├── bytemuck v1.25.0
    ├── byteorder-lite v0.1.0
    ├── dav1d v0.11.1
    ├── moxcms v0.8.1
    ├── mp4parse v0.17.0
    └── num-traits v0.2.19
```

`dav1d-sys` aparece como dep transitive de `dav1d`. Tree completo (53 pkgs, sem
duplicatas):
- **Ausente:** `owning_ref`, `aom-decode`, `libaom-sys`, `lodepng`, `avif-parse` (todos os flagged-pela-audit-15).
- **Presente novo:** `dav1d`+`dav1d-sys`+`mp4parse`+`fallible_collections`+`hashbrown`(v0.13)+`ahash`+`moxcms`+`pxfm`+`av-data`+`num-derive`+`num-rational`+`num-bigint`+`num-integer`+`bitreader`+`byte-slice-cast`+`bytes`+`bitflags`(v2)+`libc`+`static_assertions`.

### §4.3 Unsafe budget (grep `^.*unsafe ` em `src/`)

| Crate | Unsafe count | Categoria |
|---|---|---|
| `dav1d-sys 0.8.3` | 6 | FFI shim (`extern "C"` bindings) |
| `dav1d 0.11.1` | 49 | Safe wrapper sobre FFI |
| `mp4parse 0.17.0` | **0** | Mozilla parser pure-safe ✓ |
| `moxcms 0.8.1` | 344 (288 `unsafe {` + 56 `unsafe fn`) | **SIMD intrinsics** (584 chamadas a `_mm_`/`_mm256_`/`vld1q_`/`vst1q_`); categoria ≠ UAF |
| `fallible_collections 0.4.9` | 133 | alloc primitives fallible |
| **Total** | **~532** | vs ~160 do deshipado (lodepng=97 + owning_ref=42 + libaom-sys=9 + aom-decode=11) |

**Interpretação:** maior em número, **menor em risk-class**. SIMD intrinsics (~344 do moxcms) são pattern bem-auditado; FFI bindings (~55 do dav1d) são bounded. O `owning_ref` 42 unsafe do deshipado eram **UAF lifetimes** com RUSTSEC — categoria muito mais grave.

### §4.4 Licenças (cruzar com `deny.toml` allowlist)

| Crate | License | Status |
|---|---|---|
| dav1d-sys | MIT | ✓ allowed |
| dav1d | MIT (assumido pelo workspace; conferir runtime) | ✓ |
| mp4parse | MPL-2.0 | ✓ allowed |
| moxcms | BSD-3-Clause OR Apache-2.0 | ✓ allowed |
| fallible_collections | (não verificado — Coord verifica antes do ship) | ⏳ |

**A confirmar:** rodar `cargo deny check licenses` real (workspace inteiro) ANTES do commit. Atualmente só checado contra a allowlist mental — não passou pelo `cargo deny` cli ainda.

### §4.5 Bus-factor

| Crate | Maintainer | Risco |
|---|---|---|
| dav1d-sys / dav1d | rust-av project (Luca Barbato + multi) | ✓ low |
| mp4parse | Mozilla | ✓ low |
| moxcms | awxkee (single-author) | ⚠ medium (compensado por OSS active + license dual) |
| fallible_collections | vcombey (single-author) | ⚠ medium |

### §4.6 CI / build matrix friction

`dav1d-sys 0.8.3` build.rs:
- `system_deps::Config::new().add_build_internal("dav1d", build::build_from_src)`.
- Default: usa `pkg-config` pra achar `libdav1d` instalado no sistema.
- Fallback `build_from_src`: clone via git `code.videolan.org/videolan/dav1d.git` tag 1.5.0 + meson+ninja+clang build (~30-60s clean × 3 platforms).
- Trigger do fallback: env `SYSTEM_DEPS_DAV1D_BUILD_INTERNAL=always` (ou `auto` se pkg-config falhar).

**Recomendação:** **install system lib em CI** (caminho A.1), NÃO fallback git-clone (caminho A.2, fragilidade de network em build.rs).

CI steps a adicionar em `.github/workflows/spike.yml`:
- **ubuntu-latest:** `sudo apt-get install -y libdav1d-dev` (~5s; Ubuntu 22.04+ tem dav1d ≥ 1.0)
- **macos-latest:** `brew install dav1d` (~30s; brew tem ≥ 1.5)
- **windows-latest:** `vcpkg install dav1d:x64-windows` + `choco install pkgconfiglite` + setar `PKG_CONFIG_PATH=$VCPKG_INSTALLATION_ROOT\installed\x64-windows\lib\pkgconfig` (~3-5min clean — caro mas viável)

Tem que aplicar em **TODOS** os jobs do spike.yml que rodam `cargo` workspace-wide (`workspace check + tests`, `c9 cross-platform replay hash`, etc.). Conta CI atual: 6+ jobs — instalar dav1d em todos.

---

## §5 ⚠️ DECISÃO DE ESCOPO PENDENTE Enio (bloqueia início do implementador)

A verification (§4) só responde "Path A é VIÁVEL?". Quem decide "vale a pena o custo?" é o Enio. Antes de delegar pro implementador, **destrave isto**:

### §5.1 Escopo proposto pra ratificar

1. **Re-ship Path A decode-only** via `image = { version = "0.25", default-features = false, features = ["avif-native"] }`.
2. **Encode = `Error::Unsupported`** com mensagem actionable (mesmo padrão de PSD/EXR/JXL). NÃO trazer `ravif`/`rav1e` ainda (adicionaria ~30+ crates incluindo `cc`/`nom`/`libfuzzer-sys`).
3. **CI install libdav1d em 3 plataformas** (§4.6). Edit em `.github/workflows/spike.yml`.
4. **Defesas pré-decode obrigatórias** (corrigem D6/D14 do audit-15 que reaparecem em qualquer wrapper):
   - `mp4parse` validate dimensões ANTES de `image::ImageReader::with_guessed_format().decode()` → reject `> MAX_RASTER_DIMENSION`.
   - `catch_unwind` wrapper no boundary (mesma defesa do PSD per audit-7 G-F2).
   - `Error::from_decoder_message` para mapear EOF/truncated (mesma defesa do audit-7 I-3).
   - `ColorProfile`: se HDR PQ/HLG detectado pelo `mp4parse` → reject `Error::Unsupported` (mesma simetria do JXL audit-14 MM); senão `ColorProfile::Srgb`.

### §5.2 Por que decode-only e não decode+encode

Padrão JXL/EXR/PSD: decode real first, encode quando aparecer client real. Reduz blast radius:
- Sem `rav1e` ≈ -30 crates, -X minutos de build CI.
- Sem `ravif` ≈ menos unsafe.
- Encode AVIF não tem cliente pedindo hoje (Painter usa PNG/HDR).

### §5.3 Alternativas se Enio rejeitar Path A

Documentadas em HANDOFF_imageio.md §4.1 + ADR-0054 §5.17:
- **Path B:** esperar `avif-decode 2.x` migrar de `owning_ref` → `safer_owning_ref` + fix `unprem()` upstream. **Re-verifiquei nesta sessão (2026-05-28): `avif-decode` ainda está em 1.0.2 no crates.io.** Path B continua bloqueado.
- **Path C:** `libavif-sys = "0.17.0+libavif.1.0.4"` direto. Wraps libavif (reference impl). Feature default `codec-dav1d` puxa `libdav1d-sys` que **vendora** dav1d C source (não precisa pkg-config). Trade: ~30s C compile vs system install. Diferente categoria de risco que avif-decode/libaom-sys (libavif é a impl canônica reference + libdav1d compacto vs libaom 26 MB).

### §5.4 Pergunte Enio exatamente isto

```
Path A verificado: cargo audit 0, sem owning_ref/aom/lodepng. Custo: install
libdav1d em ubuntu/macOS/windows runners (windows = vcpkg ~3-5min clean).
Confirma:
  (1) escopo decode-only + encode Error::Unsupported(defer) ?
  (2) edit em .github/workflows/spike.yml para 3 installs ?
Ou prefere:
  (b) Path C via libavif-sys (vendored, sem CI install, +30s C compile) ?
  (c) Postpor AVIF (manter stub) e priorizar outra task em §4 do HANDOFF
      (JXL fixture, Tier-1 fixture expansion) ?
```

**Sem essa resposta, o implementador NÃO começa.** Marcar a task como BLOCKED.

---

## §6 Known issues / armadilhas (repassar ao implementador)

1. **`use ph2d_imageio::Error` shadow risk:** `image-0.25.10` re-exporta `Error` em vários módulos. Per audit-14 wave-2 trap (HANDOFF_imageio.md §9), sempre alias: `use ph2d_imageio::Error as IoError` dentro da `fn` afetada.
2. **Pre-decode dim cap obrigatório:** o audit-15 D14 flagou `Vec::with_capacity(w*h)` antes do `MAX_RASTER_DIMENSION` check no avif-decode. **Mesma classe de bug em qualquer wrapper.** A defesa correta = pegar dimensões via `mp4parse` ou `image::ImageReader::into_dimensions()` ANTES de `decode()` e abortar `> MAX_RASTER_DIMENSION`.
3. **`catch_unwind` obrigatório no boundary:** PSD audit-7 G-F2 padrão (vide `crates/ph2d-imageio-psd/src/lib.rs`). AVIF parsers historicamente têm `assert!` em hostile input (audit-15 D11 flagou em avif-parse). Wrappear `image::ImageReader::decode()` em `std::panic::catch_unwind`.
4. **HDR PQ/HLG mislabel risk:** audit-15 D8 flagou silent quantization no avif-decode (Display-P3/BT.2020 viraram sRGB sem aviso). `mp4parse` expõe `colour` box (nclx ou icc). Defesa = detectar pq/hlg/bt2020 → `Error::Unsupported` com mensagem; só `srgb`/`unknown` → `ColorProfile::Srgb`. **Simetria com JXL audit-14 MM.**
5. **`image::ImageFormat::Avif` recognition:** `image::guess_format` reconhece AVIF (ISOBMFF `ftyp` box com `avif`/`avis` brand). Conferir que o `peek` 12 bytes basta — se não, ler 32 bytes.
6. **CI install em TODOS os jobs:** se adicionar dav1d só no job `workspace check + tests` e esquecer no job `c9 cross-platform replay hash`, o segundo quebra. Grep todos os `runs-on: ${{ matrix.os }}` em spike.yml e adicionar step antes de qualquer `cargo`.
7. **`#![forbid(unsafe_code)]` no crate:** mantenha. `image::ImageReader` é safe wrapper; toda unsafe fica nas deps transitive (consistente com `forbid` no nosso crate).
8. **Codegen `ph2d-imageio-sync`:** Se mudar de stub→real, **rodar** `cargo run -p ph2d-imageio-sync` e conferir `cargo test -p ph2d-imageio-registry-init` (staleness gate). Pra AVIF o `register_all_*` provavelmente NÃO muda (já estava registrado como stub) — só conferir.

---

## §7 Pre-existing failures cross-session (NÃO fixar — reportar ao owner)

Per [[feedback-audit-scope-discipline]] — bug em crate adjacent → handoff ao owner, não fixo. **Implementador AVIF não toca nada disso:**

1. Working tree tem **33 commits ahead de origin/main** + várias mudanças WIP de outras sessões. Vide §9 abaixo.
2. `crates/ph2d-tool-painter/src/tool.rs` (M): Painter T1.9 WIP — Coord-A slot `impl-1`.
3. `crates/ph2d-asset/*` (M): KTX2 Fase 2 W1 — sessão paralela.
4. `crates/ph2d-tool-bgremoval/src/tool.rs` (M): bgremoval session.
5. `crates/ph2d-editor-core/tests/node_id_collisions.rs` (M): origem incerta — pode ser hier companion session.
6. `docs/Sprite_projeto/`, ADRs `0069..0074*`, `0025-amendment-1` (??): Sprite Inspector v2 docs-only PRONTO PARA RATIFICAÇÃO (autorizado Enio 2026-05-27).

Se o implementador AVIF encontrar algum desses falhando em `cargo check -p <foo>` (não-AVIF), **reporta a você** com nome do crate + erro; **você** repassa pro Coord/Implementador responsável. NÃO arrumar.

---

## §8 Ship / push (decisão do Enio, via você Coordenador)

- **33 commits locais ahead de origin/main** (todas as sessões somadas, NÃO meus). Nenhum push nesta jornada minha.
- Pre-existing fmt drift do workspace é provável (várias sessões commitaram com `--no-verify` ao longo do dia) — `./scripts/ship.sh` (`cargo fmt --all --check`) vai reprovar. Per DIRETRIZ §8.1, **antes de qualquer push você Coord roda `./scripts/ship.sh` e corrige TUDO até verde**.
- **O implementador AVIF não pusha** — só reporta "commit local <sha> pronto" (CLAUDE.md).
- AVIF mexe em CI workflow (§5.1.3) — `ship.sh` localmente NÃO simula instalação de libdav1d em macOS/Windows. **A primeira verificação real do install acontece no CI run.** Esperar babysit-CI verde após push.

---

## §9 ⚠️ Estado git ATUAL do working tree (você precisa saber)

```
HEAD = e905af1 (docs imageio HANDOFF)
33 commits ahead de origin/main (várias sessões)

WIP não-meu, não-AVIF:
M Cargo.lock
M crates/ph2d-asset/Cargo.toml
M crates/ph2d-asset/src/asset.rs
M crates/ph2d-asset/src/tier.rs
M crates/ph2d-asset/tests/architecture_texture_ktx2.rs
M crates/ph2d-asset/tests/import_image.rs
M crates/ph2d-editor-core/tests/node_id_collisions.rs
M crates/ph2d-tool-bgremoval/src/tool.rs
M crates/ph2d-tool-painter/Cargo.toml
M crates/ph2d-tool-painter/src/tool.rs
M docs/Painter_projeto/*.md (vários)
M docs/SESSION_ACTIVE.md
M docs/plans/2026-05-texture-compression-waves.md
D docs/HANDOFF_bgremoval_audit_carryovers.md

Untracked não-meu (legítimos de outras sessões):
?? crates/ph2d-brush-traits/tests/_audit_dyn_send_sync.rs
?? crates/ph2d-brush-traits/tests/_audit_send_sync.rs
?? crates/ph2d-vector-traits/tests/_audit_send_sync.rs
?? docs/HANDOFF_ktx2_w1_coordinator.md
?? docs/HANDOFF_sprite_W1_to_new_coordinator.md
?? docs/Painter_projeto/14_inovacoes_extraordinarias.md
?? docs/Painter_projeto/avaliacao_e_melhorias.md
?? docs/Sprite_projeto/  (16+ arquivos)
?? docs/UI_Fonts/
?? docs/architecture/decisions/0025-amendment-1.md
?? docs/architecture/decisions/0069..0074-*.md
?? test_strip
+ outros docs Sprite_projeto/...
```

**Nada disso é meu.** Esta sessão NÃO mexeu em arquivo nenhum. O scratch `/tmp/avif-verify/` é fora do repo.

**Para o implementador AVIF:**
1. `git status -sb` antes de stage; SE algo for não-AVIF, **NÃO commit** misturado.
2. `git add -- crates/ph2d-imageio-avif/<paths>` SEMPRE explícito; nunca `-A` (per [[feedback-parallel-agent-commit-collision]] + [[feedback-scoped-commit-shared-index]]).
3. Mudanças em `.github/workflows/spike.yml` e docs (HANDOFF/ADR/plano) vão em **commit separado** do código do AVIF.

---

## §10 ESQUELETO do sub-handoff que VOCÊ (Coordenador) escreve para o implementador

Per pedido do Enio, gere um handoff curto pro implementador AVIF continuar **DEPOIS** de destravar §5 com o Enio. Use:

```
═══════════════════════════════════════════════════════════════════
HANDOFF — Implementador Image I/O AVIF · W3.T4 re-ship Path A
═══════════════════════════════════════════════════════════════════

PRE-REQUISITO: o Coord já destravou com o Enio (§5 do handoff Coord)
  ✓ Path A decode-only + encode Error::Unsupported(defer)
  ✓ Autorizada edit em .github/workflows/spike.yml (3 installs dav1d)
  ✓ Decisão de não trazer ravif/rav1e nesta task

SANITY CHECK (rode primeiro):
  git log --oneline -3            # HEAD deve conter e905af1
  git status -sb                  # 33 ahead esperado; M/?? de outras
                                  # sessões esperados (vide §9 handoff Coord) —
                                  # NÃO commit misturado
  cargo check -p ph2d-imageio-avif        # baseline (stub) compila
  cargo test  -p ph2d-imageio-avif        # 9 stub tests verdes

SUA PASTA EXCLUSIVA (edite SÓ aqui):
  crates/ph2d-imageio-avif/  (Cargo.toml + src/lib.rs + src/decode.rs novo + tests/)

SÓ COM MEU OK EXPLÍCITO (Coord) — depois que pegar OK do Enio:
  .github/workflows/spike.yml  (3 install steps libdav1d)

NÃO TOCAR: qualquer outra `crates/ph2d-imageio-*`, `crates/ph2d-asset/*`,
  Cargo.lock root, qualquer Painter/KTX2/Vector/Sprite/bgremoval crate.
  Precisou de algo fora? PARE e me reporte (sou o Coord) — não edite.

TASK: W3.T4 AVIF re-ship Path A decode-only
  1. Editar crates/ph2d-imageio-avif/Cargo.toml:
       - adicionar `image = { version = "0.25", default-features = false, features = ["avif-native"] }`
       - manter `ph2d-imageio = { path = "..." }`
       - manter `#![forbid(unsafe_code)]` no lib.rs
  2. crates/ph2d-imageio-avif/src/lib.rs:
       - manter magic recognition existente (não regredir)
       - implementar ImageImporter::decode com:
         a) catch_unwind boundary (espelho de PSD)
         b) mp4parse pre-decode: dimensões > MAX_RASTER_DIMENSION → Error::TooLarge
         c) HDR PQ/HLG/BT.2020 detect via nclx box → Error::Unsupported (simetria JXL audit-14 MM)
         d) image::ImageReader::with_guessed_format().decode() — só path "srgb/unknown"
         e) RGBA8 retornar como DecodedImage::Rgba8
         f) Error::from_decoder_message pra EOF/truncated (espelho dos outros 9)
       - ImageExporter: NÃO mudar (continua Error::Unsupported com mensagem actionable)
  3. tests/ — adicionar (espelhando crates/ph2d-imageio-jxl/tests/):
       - real_decode_smoke (8×8 fixture .avif checked in se Coord arranjar; senão
         skip + ignore tag pra rodar local com fixture privada)
       - magic_recognition (mantém atual)
       - truncated_avif → Error EOF mappable
       - too_large_dimensions → Error::TooLarge (pre-decode cap working)
       - hdr_pq_rejected → Error::Unsupported (simetria JXL)
       - encode_unsupported_returns_error_with_actionable_message
  4. Não mexer em ph2d-imageio-registry-init/ — já registrado como stub; staleness
     deve continuar verde. Rodar `cargo test -p ph2d-imageio-registry-init` pra
     confirmar.

VALIDAÇÃO INCREMENTAL (durante editing):
  cargo check  -p ph2d-imageio-avif                     # 3-15s
  cargo test   -p ph2d-imageio-avif                     # 5-30s
  cargo clippy -p ph2d-imageio-avif --all-targets -- -D warnings

VALIDAÇÃO FINAL (antes de commit):
  cargo test  -p ph2d-imageio                            # contract gates
  cargo test  -p ph2d-imageio-registry-init              # staleness + ABC order
  cargo deny check licenses                              # confirma 532 transitives ok
  cargo audit                                            # confirma 0 RUSTSEC

5-LENS AUDIT antes de declarar real (per HANDOFF_imageio.md §8 step 5):
  Lente 1 — closure correctness: catch_unwind cobre todas as branches?
            mp4parse erra antes do image::decode?
  Lente 2 — dep tree: cargo audit clean + cargo deny check licenses
            (workspace-wide, não scratch).
  Lente 3 — spec compliance: AVIF ftyp box recognition; nclx vs icc handling;
            grid/animation skip (multi-image AVIF → reject ou primeiro frame?).
  Lente 4 — HR coverage: HR-1 platform-agnostic OK só com CI install;
            HR-13 OOM cap pre-decode; HR-15 fluent_key cobre Error variants novos.
  Lente 5 — regressions: ph2d-asset loader::decode_via_imageio_registry continua
            roteando AVIF? Drag-drop .avif no shell vira DecodedImage real?

DOCS (commit separado depois do código):
  - docs/HANDOFF_imageio.md §1 format matrix: AVIF Decode ✅ Real / Encode ❌
  - docs/HANDOFF_imageio.md §2 audit timeline: + linha "Audit-16 wave-3 — W3.T4
    AVIF Path A real decode via image::avif-native"
  - docs/HANDOFF_imageio.md §3 defenses: + linha "mp4parse pre-decode dim cap
    + HDR PQ/HLG reject + catch_unwind boundary (audit-16)"
  - docs/HANDOFF_imageio.md §6 test counts: ph2d-imageio-avif de 9 → N (X stub
    removidos, Y reais)
  - docs/architecture/decisions/0054-imageio-pipeline.md §5.18: nova seção
    wave-3 re-ship Path A (espelho do §5.17 deship).
  - docs/plans/2026-05-imageio-waves.md: W3.T4 → ✅ Real (Path A).

CI WORKFLOW (commit separado — SÓ DEPOIS do código verde local):
  Editar .github/workflows/spike.yml. Em CADA job que roda cargo workspace-wide:
    - ubuntu: `sudo apt-get install -y libdav1d-dev` (antes do cargo step)
    - macos: `brew install dav1d`
    - windows: choco install pkgconfiglite + vcpkg install dav1d:x64-windows
              + setar PKG_CONFIG_PATH via $env:PKG_CONFIG_PATH
  Conta jobs com `runs-on: ${{ matrix.os }}` em spike.yml — adicionar step em
  TODOS (provavelmente 3-4 jobs). NÃO esquecer c9 cross-platform replay hash.

COMMIT (escopado, nunca -A):
  Commit 1 — código AVIF:
    git add -- crates/ph2d-imageio-avif/
    git commit -m "feat(imageio): W3.T4 wave-3 — AVIF real decode via image::avif-native (Path A)"
  Commit 2 — docs:
    git add -- docs/HANDOFF_imageio.md docs/architecture/decisions/0054-imageio-pipeline.md docs/plans/2026-05-imageio-waves.md
    git commit -m "docs(imageio): W3.T4 wave-3 — Path A re-ship documented"
  Commit 3 — CI workflow:
    git add -- .github/workflows/spike.yml
    git commit -m "ci(spike): install libdav1d on ubuntu/macos/windows for ph2d-imageio-avif"

  Eu (Coord) faço ship/push no fim. Você só reporta "3 commits locais prontos:
  <sha1> <sha2> <sha3>".

SE TRAVAR (qualquer um destes — PARE e me reporte):
  - dav1d-sys não build local mesmo com brew install dav1d (macOS) →
    pode precisar HOMEBREW_PREFIX export ou PKG_CONFIG_PATH.
  - mp4parse 0.17 API divergente do que assumimos (ex: nclx box não exposto) →
    pode precisar plugar `mp4parse-isobmff` ou parsing manual.
  - cargo deny check licenses reprova moxcms (BSD-3 OR Apache-2.0 deveria passar) →
    confirmar deny.toml allowlist.
  - cargo test -p ph2d-imageio-registry-init quebra staleness → rodar
    `cargo run -p ph2d-imageio-sync` e re-confirmar.
═══════════════════════════════════════════════════════════════════
```

---

## §11 Boundaries — o que o Coordenador decide vs PERGUNTA Enio

**Coordenador decide:**
- Quem é o implementador AVIF (qual slot).
- Ordem de commits (sugestão: código → docs → CI).
- Como dividir as 3 sessões entre AVIF / outras tasks W3 / outros módulos.
- Resolução de conflitos cross-session (per §7).

**Coordenador PERGUNTA Enio (BLOQUEIO crítico):**
- §5 escopo Path A decode-only + 3 CI installs em spike.yml. **Sem isso o
  implementador NÃO começa.**
- Se ele rejeitar Path A → escolher Path C (libavif-sys) ou postpor.
- Push para origin (33 commits ahead + commits novos de AVIF + outros).
- Qualquer amendment a contrato FROZEN (não esperado nesta task, mas se aparecer).

---

## §12 Memórias-âncora (releia antes de agir)

- [[feedback-perfection-no-deferrals]] — gaps in-scope (CI install, defesas pré-decode, license check) viram trabalho da sessão atual.
- [[feedback-no-industrial-claims-without-verification]] — toda afirmação técnica passou por `cargo audit`/`cargo tree`/`cargo info`. §4 lista os comandos brutos.
- [[feedback-audit-internal-state-grep]] — antes de afirmar "dav1d-sys vendora", grep build.rs. Não inventar internals.
- [[feedback-audit-scope-discipline]] — bug em crate adjacent (Painter/KTX2/Sprite) → handoff ao owner, não fixo.
- [[feedback-audit-lens-diversity]] — 5 lentes ortogonais na audit pós-implementação; rotacionar (não 5× mesma lente).
- [[feedback-parallel-agent-commit-collision]] + [[feedback-scoped-commit-shared-index]] — `git status` antes de stage; `git add -- <paths>` específico; nunca `-A`. Especial atenção porque o working tree tem WIP de 3-4 sessões simultâneas.
- [[feedback-destructive-git-outside-pasta]] — nunca git destrutivo fora da sua pasta.
- [[feedback-app-ui-english-only]] — labels/strings da UI em inglês; código/comentários podem ser misto. Não aplicável a AVIF puro (sem UI).

---

**Resumo de uma linha:** W3.T4 AVIF está deshipado em `f034e9a`; verification protocol pra re-ship via Path A (`image::avif-native`) **passou totalmente** nesta sessão (§4) — sem código mudado; **§5 decisão de escopo PRECISA destrancar com Enio** antes do implementador começar; sub-handoff pronto pra colar em §10.
