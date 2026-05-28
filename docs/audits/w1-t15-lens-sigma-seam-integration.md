# W1.T15 — Lente σ (sigma): seam / system integration (gate final da W1)

- **Data:** 2026-05-28
- **Auditor:** Claude Opus 4.8 (adversarial single-pass, lente σ)
- **Escopo:** o **seam** entre os dois lados do pipeline de texture compression —
  `ph2d-asset-cooker` (EMITE KTX2 via ctt) ↔ `ph2d-asset-ktx2` (LÊ KTX2 via parser).
  Mais o caminho de batch `cook_all` (W1.T6) → decode.
- **Método:** análise estática do grafo de dependências + leitura do mapeamento
  `from_vk_format` vs formatos emitidos por `target_matrix` + **gate executável novo**
  (`tools/asset-cooker/tests/seam_cook_decode_ktx2.rs`).
- **Lente nova:** σ ainda não usada nas 6 rodadas anteriores (γ/δ/ε/ζ/η/θ/ι/κ/λ/μ/ν/ξ);
  rotação per [[feedback-audit-lens-diversity]]. Round único, anti-Goodhart.

---

## Veredito: **PASS_WITH_FINDINGS — 9.0 / 10**

O seam **fecha corretamente**: todos os VkFormats que o encoder ISPC grava são
reconhecidos pelo parser. Porém a verificação disso era **afirmada e não travada** —
nenhum teste do workspace exercitava o seam. Findings σ-1 (gate ausente) fechado
inline nesta sessão com gate executável.

---

## §1 A descoberta central (σ-1, HIGH → FECHADO inline)

As 6 rodadas de audit anteriores cobriram cada task **isoladamente**:

| Rodada | Task | Lentes | Lado do seam |
|---|---|---|---|
| 1 | W1.T3 (cooker first impl) | γ, δ | EMIT (cook) |
| 2 | W1.T4 (Asset::TextureKtx2) | ε, ζ | (asset variant) |
| 3 | W1.T6 (cook_all multi-tier) | η, θ | EMIT (batch) |
| 4 | W1.T7 (mip gen) | λ, μ | EMIT (mip) |
| 5 | W1.T11+T14 (fixtures + proof-of-life) | ι, κ | EMIT (magic só) |
| 6 | W1.T9 (kvd preservation) | ν, ξ | READ (parser) |

**Nenhuma cruzava o seam.** O grafo de dependências confirma por quê:

- `tools/asset-cooker/Cargo.toml` → **não** depende de `ph2d-asset-ktx2`.
- `crates/ph2d-asset/Cargo.toml` → **não** depende de `ph2d-asset-ktx2`
  (decisão pragmática W1.T4, documentada em `ph2d-asset/src/asset.rs:35-41`).
- Testes do cooker (`cook_64x64_*`, `sample_cook_brush_atlas`) checam só os
  **12 bytes de magic** `«KTX 20»\r\n\x1A\n` — nada do conteúdo do header.
- Testes do parser (`decode_fixture_*`) usam `build_fixture` **hand-construído** —
  nunca bytes reais do encoder ctt.

Logo a hipótese central da Fase 2 — **"o VkFormat que o encoder ISPC grava no
container bate com o `ktx2::Format` que o decoder lê"** — vivia só em prosa. Se o
ctt 0.4.0 gravasse, digamos, um VkFormat de bloco ASTC fora do subset que
`from_vk_format` (lib.rs:305-396) conhece, o W2 renderer receberia
`Ktx2Format::Unsupported(raw)` **silenciosamente** — sem nenhum teste falhando.

### Fix inline (σ-1)

Novo gate executável: `tools/asset-cooker/tests/seam_cook_decode_ktx2.rs`
(7 testes). Cozinha o fixture canônico `gradient_64x64` (W1.T11) para cada
`(Tier, AssetClass)` e **decodifica o resultado pelo parser real**, assertando:

1. dimensões round-trip (64×64 sobrevive cook → decode);
2. `base_level()` presente + payload não-vazio;
3. **formato decodificado é o ESPERADO**, nunca `Unsupported`:
   - Desktop/SpriteColor → BC7 (`is_compressed()`)
   - Desktop/SingleChannel → BC4
   - Desktop/NormalMap → BC5
   - Mobile/SpriteColor → ASTC 6×6
   - LowEnd/SpriteColor → ETC2 RGBA8
   - Constrained/SpriteColor → RGBA8 uncompressed (`!is_compressed()`)
4. `cook_all` (W1.T6): os 5 tiers decodificam todos para formato **conhecido**
   (gate de batch — o "final integration check" pedido pelo W1.T15).

Dependência adicionada como **dev-only** em `asset-cooker/Cargo.toml`
(`ph2d-asset-ktx2` é parser puro: `ktx2` + `thiserror`, sem ISPC, sem ciclo —
não depende de asset-cooker). **A decisão pragmática W1.T4 de NÃO adicionar dep
de produção é preservada** — o seam é provado sem acoplar os crates em release.

Tolerância UNORM-vs-SRGB intencional: o ctt escolhe a variante de VkFormat em
função do `color_space`; o gate aceita ambas as variantes da família e crava a
família (BC7/BC4/BC5/ASTC/ETC2/RGBA8) — robusto a essa decisão interna sem
travar pixel-perfect (que precisa GPU transcode, W2).

---

## §2 Mapeamento de formato — verificação estática (PASS)

`target_matrix::target_for` emite: `BC7_UNORM_BLOCK`, `BC4_UNORM_BLOCK`,
`BC5_UNORM_BLOCK`, `ASTC_6x6_UNORM_BLOCK`, `ASTC_4x4_UNORM_BLOCK`,
`ETC2_R8G8B8A8_UNORM_BLOCK`, `R8G8B8A8_UNORM`.

`from_vk_format` (lib.rs:305-396) cobre **todos** esses (+ variantes SRGB +
BC1/BC3/BC6H/ASTC 5×5/8×8/ETC2_RGB). Zero gap de cobertura. O uso das constantes
`ktx2::Format::*` (não inteiros hard-coded) é defesa correta contra typo de
VkFormat ID — confirmado.

---

## §3 Findings

| ID | Sev | Descrição | Estado |
|---|---|---|---|
| σ-1 | HIGH | Seam cook→decode sem gate executável; hipótese central não travada | **FECHADO inline** (`seam_cook_decode_ktx2.rs`, 7 testes) |
| σ-2 | LOW | `cook_all` não testava decode dos artefatos de batch (só `.len()==5` + magic) | **FECHADO** (incluído no gate σ-1, teste `cook_all_every_tier_decodes_to_a_known_format`) |
| σ-3 | NIT (adjacent — W2) | O seam para `ph2d-asset::Asset::TextureKtx2` ainda é manual: nada testa que um blob colocado no variant decodifica. Mas o variant carrega `Arc<Vec<u8>>` opaco e o decode é responsabilidade do renderer W2 (asset.rs:37). Honesto-deferred. | Reportado — owner = W2 renderer |
| σ-4 | HIGH (adjacent — **NÃO minha pasta de origem**) | `tools/asset-cooker/tests/cooker_determinism.rs:72 prefab_cook_hash_is_locked` FALHA em HEAD: hash atual `6feb338498d6d0b6dbe4a018187c4dfb3725db624bea71301231d380dcc8afab` ≠ pinado `905a9b77...`. **Causa raiz confirmada:** commit `4591f7e` (Sprite Inspector v2 session) expandiu `ph2d::render::Sprite` v3→v4 (20 campos); `simple_sprite.json5` contém um Sprite → postcard agora serializa os campos default novos → bytes cookados mudam → blake3 muda. **Verificado independente do meu dev-dep** (reproduz com a dep removida — prefab cook não usa ktx2). O fix exige atualizar o hash pinado **+** o prefab id referenciado em `tests/fixtures/scene/two_sprites.json5` — ambos dependem do schema Sprite v4. | **Reportado ao Coordenador — owner = Sprite Inspector v2 / commit 4591f7e.** NÃO fixado ([[feedback-audit-scope-discipline]]). |

---

## §4 Conclusão

O pipeline `cook → asset → ktx2` é **íntegro**: o seam fecha e agora está
**travado por gate executável** (gates > claims verbais, [[feedback-audit-lens-diversity]]).
A única lacuna restante (σ-3) é genuinamente W2 (o renderer ainda não existe; o
`TextureKtx2` variant é opaco por design W1.T4). Nada bloqueia a abertura da W2.

**σ score: 9.0/10 — PASS_WITH_FINDINGS** (1 HIGH fechado inline + 1 LOW fechado +
1 NIT adjacent reportado).
