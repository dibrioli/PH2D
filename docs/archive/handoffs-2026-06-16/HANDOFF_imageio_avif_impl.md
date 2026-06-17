═══════════════════════════════════════════════════════════════════
HANDOFF — Implementador Image I/O AVIF · W3.T4 re-ship
Autor: Coordenador (sessão 2026-05-28) · você é o 2º dos 5 implementadores
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ ✅ GO — escopo destravado pelo Enio (2026-05-28).                   ║
║ Diretriz do projeto: "o melhor possível, SEM pensar em custos".     ║
║ → Isso INVERTE as 3 economias do handoff Coord original. Escopo     ║
║   agora é Path C + decode E encode + HDR/wide-gamut REAL. Detalhe   ║
║   abaixo. Custo de build/CI/footprint NÃO é razão pra escolher menos.║
╚═══════════════════════════════════════════════════════════════════╝

CONTEXTO: W3.T4 AVIF foi DESHIPADO em `f034e9a` (audit-15 derrubou
`avif-decode 1.0` por 1 RUSTSEC + HIGHs unfixable, todos em `owning_ref` UAF).
Re-ship agora visando padrão-ouro de AVIF, não o mínimo.

───────────────────────────────────────────────────────────────────
⚠️ MUDANÇA DE ESCOPO vs handoff Coord de origem (leia com atenção)
───────────────────────────────────────────────────────────────────
O handoff de origem propunha Path A (decode-only, reject-HDR) PRA ECONOMIZAR
CI install + crates. O Enio decidiu "melhor possível sem custo". Resultado:

| Handoff origem economizava | AGORA (padrão-ouro) |
|---|---|
| Path A: `image` feat `avif-native` (dav1d) | **Path C: `libavif-sys`** — implementação de REFERÊNCIA do AOM |
| decode-only; encode `Error::Unsupported(defer)` | **decode E encode reais** (ImageExporter completo) |
| rejeitar HDR PQ/HLG/BT.2020 | **suportar** wide-gamut/HDR via `ColorProfile` (ADR-0054 tem 8 variants) |
| CI install libdav1d em 3 OS (apt/brew/vcpkg) | libavif-sys **vendora** libdav1d+aom via build.rs → **sem CI install** |

Por que Path C é o "melhor": libavif é a impl canônica de referência do AV1
Media (grid images, alpha, animação, ICC + nclx, lossless, todas as transfer
functions) — cobre o spec inteiro que o `image::avif-native` decode-only não
cobre. Vendored = build reproduzível (HR-6), sem fragilidade de system-lib no CI.

───────────────────────────────────────────────────────────────────
PASSO 0 OBRIGATÓRIO — Verification protocol do Path C (ANTES de codar)
───────────────────────────────────────────────────────────────────
A verification que já existe (handoff Coord §4) foi pra Path A. Path C é deps
DIFERENTES → refaça do zero (per [[no-industrial-claims-without-verification]]).
Num scratch /tmp/avif-c-verify/ com:
  libavif-image = "..."  (ou libavif-sys = "0.17.0+libavif.1.0.4" — confira a
  versão atual com `cargo search libavif`; NÃO assuma número de cor)
  features: codec-dav1d (decode) + codec-aom (encode)
Rode e me REPORTE os números brutos antes de tocar no crate:
  cargo audit            → 0 RUSTSEC esperado; se houver, PARE e me reporte
  cargo tree -e normal   → confirme libaom-sys/libdav1d-sys vendored; sem owning_ref
  cargo deny check licenses  → libavif BSD-2 / dav1d BSD-2 / aom BSD-2 + patent
                               clause (AV1 royalty-free AOM); confirme allowlist
  unsafe budget + bus-factor (espelho §4.3/§4.5 do handoff Coord)
  HR-1 FFI vendored (SKILL §HR-1 critério 6 pontos): libavif = ref impl única ✓,
    build.rs vendored reproduzível ✓, BSD-2 ✓, AOM ativo ✓, AV1 royalty-free ✓,
    #1 "offline-only" — encode é editor/cooking (offline ok); decode runtime é
    consistente com dav1d já aceito no módulo. CONFIRME e documente no relatório.
Se algum item reprovar de forma estrutural → PARE, me reporte; reavaliamos
(fallback documentado: Path A decode-only só se Path C for inviável tecnicamente,
NÃO por custo).

───────────────────────────────────────────────────────────────────
SANITY CHECK (rode primeiro — baseline já validado por mim)
───────────────────────────────────────────────────────────────────
  git log --oneline | grep f034e9a            # deship na história
  git status -sb -- crates/ph2d-imageio-avif/ # esperado: NADA pendente (limpo)
  cargo check -p ph2d-imageio-avif            # stub compila
  cargo test  -p ph2d-imageio-avif            # 9 stub tests verdes

  ⚠️ Working tree TEM WIP de 3-4 outras sessões. HEAD = e5fb811, 83 ahead.
  NADA disso é seu. `git status` antes de stage; nunca comite misturado.

───────────────────────────────────────────────────────────────────
SUA PASTA EXCLUSIVA (edite SÓ aqui)
───────────────────────────────────────────────────────────────────
  crates/ph2d-imageio-avif/  (Cargo.toml + src/lib.rs + src/decode.rs + src/encode.rs + tests/)

NÃO TOCAR: qualquer outra crates/ph2d-imageio-* (13 prontas — não regrida),
  crates/ph2d-asset/*, Cargo.lock root, Painter/KTX2/Vector/Sprite/bgremoval.
  Path C provavelmente NÃO precisa editar .github/workflows/spike.yml (vendored),
  o que é uma vantagem — MAS se a verification §0 revelar que precisa (ex: build
  dep de nasm/meson pro aom), PARE e me reporte: workflow edit é só com meu OK.
  Precisou de algo fora? PARE e me reporte (sou o Coord) — não edite.

───────────────────────────────────────────────────────────────────
TASK — W3.T4 AVIF re-ship Path C (decode + encode + HDR), padrão-ouro
───────────────────────────────────────────────────────────────────
  1. Cargo.toml: + libavif-sys (ou wrapper libavif-image) com codec-dav1d +
     codec-aom. Manter ph2d-imageio path dep. Manter #![forbid(unsafe_code)] no
     NOSSO lib.rs (FFI vive nas deps; nosso crate fica safe).
  2. src/decode.rs — ImageImporter::decode:
       a) catch_unwind no boundary (espelho crates/ph2d-imageio-psd/, audit-7 G-F2).
       b) pre-decode dim cap: dimensões > MAX_RASTER_DIMENSION → Error::TooLarge
          ANTES de qualquer alloc grande (audit-15 D14 — a classe de bug a evitar).
       c) HDR/wide-gamut: ler nclx/icc box → mapear pra ColorProfile correto
          (PQ/HLG/BT.2020/Display-P3 → o ColorProfile certo, NÃO reject; só faça
          Error::Unsupported pra combinação genuinamente não-representável).
       d) RGBA8 (e RGBA16/float se HDR) → DecodedImage variant apropriado.
       e) grid/animation multi-image: decida e documente (1º frame? full sequence?
          — padrão-ouro = não perder dados silenciosamente; me reporte a escolha).
       f) Error::from_decoder_message pra EOF/truncated (espelho dos outros crates).
  3. src/encode.rs — ImageExporter::encode (NOVO — não é mais defer):
       encode AVIF via libavif/aom; quality/speed params sensatos; preserve
       ColorProfile no nclx box de saída; round-trip test (decode(encode(x)) ~= x).
  4. tests/ (espelhe crates/ph2d-imageio-jxl/ + -hdr-radiance/):
       real_decode_smoke, magic_recognition, truncated_avif, too_large_dimensions,
       hdr_wide_gamut_roundtrip (decode P3/BT.2020 → ColorProfile certo),
       encode_decode_roundtrip, grid_image_handling.
  5. NÃO mexer em ph2d-imageio-registry-init/ (já registrado stub). Rodar
     `cargo test -p ph2d-imageio-registry-init` pra confirmar staleness verde.

───────────────────────────────────────────────────────────────────
ARMADILHAS (decoradas — já queimaram no módulo)
───────────────────────────────────────────────────────────────────
  1. `Error` shadow: alias `use ph2d_imageio::Error as IoError` na fn (audit-14).
  2. Pre-decode dim cap OBRIGATÓRIO antes de alloc (audit-15 D14).
  3. catch_unwind OBRIGATÓRIO — AVIF parsers têm assert! em hostile input (D11).
  4. magic: libavif/ftyp box (avif/avis brand). Conferir peek de bytes basta.
  5. #![forbid(unsafe_code)] no nosso crate fica — unsafe só nas deps transitive.

───────────────────────────────────────────────────────────────────
VALIDAÇÃO + 5-LENS AUDIT — 1× NO FECHAMENTO do módulo (DIRETRIZ §6.6)
───────────────────────────────────────────────────────────────────
  INNER LOOP por task = SÓ `cargo check -p ph2d-imageio-avif` (ou cargo-check-narrow.sh).
  O bloco abaixo roda UMA vez, ao declarar o AVIF pronto — NÃO por commit:
  cargo nextest + clippy -p ph2d-imageio-avif --all-targets -- -D warnings
  cargo test -p ph2d-imageio              # contract gates
  cargo test -p ph2d-imageio-registry-init # staleness + ABC order
  cargo deny check licenses               # vendored aom/dav1d ok (workspace-wide)
  cargo audit                             # 0 RUSTSEC
  L1 closure correctness (catch_unwind cobre branches? dim cap antes do decode?)
  L2 dep tree + license (workspace-wide, não scratch)
  L3 spec compliance (ftyp; nclx vs icc; grid/animation; HDR transfer functions)
  L4 HR coverage (HR-1 FFI vendored 6-critério; HR-6 build reproduzível;
     HR-13 OOM pre-decode cap; HR-15 fluent_key dos Error novos)
  L5 regressions (asset loader::decode_via_imageio_registry roteia AVIF?
     drag-drop .avif → DecodedImage real? encode .avif salva e relê?)

───────────────────────────────────────────────────────────────────
COMMIT (escopado, nunca -A) / REPORT
───────────────────────────────────────────────────────────────────
  Commit 1 (código): git add -- crates/ph2d-imageio-avif/
    "feat(imageio): W3.T4 wave-3 — AVIF real decode+encode via libavif (Path C)"
  Commit 2 (docs):   git add -- docs/HANDOFF_imageio.md \
                       docs/architecture/decisions/0054-imageio-pipeline.md \
                       docs/plans/2026-05-imageio-waves.md
    → §1 format matrix (AVIF Decode ✅ / Encode ✅), §2 +Audit-16, §3 +defesas,
      §6 test counts (9 → N); ADR §5.18 nova seção (re-ship Path C, decode+encode,
      HDR real — registre que substitui a proposta Path A do handoff por decisão
      Enio "melhor sem custo"); plano W3.T4 → ✅ Real.
  Fast-mode de dia OK (`--no-verify`). Você NÃO pusha — eu rodo ship.sh + push +
  babysit no fim. Ao terminar reporte: "commits locais <sha…> prontos. Path C
  verification: <números>. avif+imageio+registry-init+deny+audit verdes.
  5-lens: <N findings, todos fechados>."

───────────────────────────────────────────────────────────────────
SE TRAVAR (PARE e me reporte — sou o Coord)
───────────────────────────────────────────────────────────────────
  - libavif-sys build precisa nasm/meson/cmake não presente → me reporte (pode
    exigir workflow edit = meu OK + Enio).
  - cargo deny reprova patent clause do aom → confere deny.toml comigo (AV1 é
    royalty-free AOM; clause é BSD-2 + AOM patent license).
  - QUALQUER cargo check de crate não-AVIF falhando → pre-existing de outra sessão
    (Painter PanelEvent / panel_loc_cap); reporta o nome, NÃO fixe.
═══════════════════════════════════════════════════════════════════
