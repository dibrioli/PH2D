# ADR-0113 — Export comprimido de áudio: Ogg Vorbis via `vorbis_rs` (Opus adiado)

- **Status:** aceito (Enio, 2026-07-11 — "siga Export OGG/Opus")
- **Escopo:** módulo de áudio (`line/audio`), W6 asset-prep. Adiciona **1 dependência de codec**.

## Contexto

O editor de áudio só exporta **WAV** (PCM/float, sem perda). Para entrega de assets de
jogo, WAV é grande demais; as engines (Godot, Unity, web) aceitam **Ogg Vorbis** (`.ogg`,
o padrão legado universal) e **Opus** (moderno, melhor qualidade/bitrate). O plano
([`Audio/02_plano_implementacao_completo.md`](../../Audio/02_plano_implementacao_completo.md) §6/§W6)
pede "export OGG/Opus", e nota o risco: escolher crate **permissiva e sem `*-sys` pesado
no CI** — a lição [`imageio AVIF`](../../../project-memory/project_imageio_avif_pathc_2026_05_28.md)
foi que `dav1d`/`rav1e` puxam meson+nasm+pkg-config e doeram nos 3 SOs.

Restrição-chave: **`ph2d-audio-encode` declara `#![forbid(unsafe_code)]`** — encoder que
exija `unsafe` no crate quebra essa propriedade.

## Decisão

**Vorbis (`.ogg`) via [`vorbis_rs`](https://crates.io/crates/vorbis_rs) v0.5.5.** Encoder
= a referência libvorbis com os patchsets **aoTuV/Lancer**, com libvorbis+libogg
**vendorizados** e compilados da fonte pelos sys-crates (`aotuv_lancer_vorbis_sys`,
`ogg_next_sys`). Motivos, verificados (não assumidos):

- **API segura** → `ph2d-audio-encode` **mantém `forbid(unsafe_code)`**. (`encode_ogg` /
  `write_ogg` novos; `EncodeError::Codec(String)` mantém o erro público desacoplado do crate.)
- **Sem lib de sistema / sem toolchain pesado:** build padrão precisa só de **`cc`**
  (universal nos 3 SOs) — sem meson/nasm/pkg-config/libclang. Fonte C embutida no crate
  publicado. Confirmado: `cargo check` compilou o C no Linux de primeira.
- **Licença permissiva:** `vorbis_rs`/`aotuv_lancer_vorbis_sys`/`ogg_next_sys` = **BSD-3-Clause**
  (já em `deny.toml`); `tinyvec`/`tinyvec_macros` = Zlib/MIT/Apache. `cargo deny check` =
  **licenses/advisories/bans/sources ok**. Zero mudança em `deny.toml`.
- **Royalty-free (Xiph):** passa o critério HR-1 #6 — o MESMO do decoder Vorbis que a
  `ph2d-audio-decode` já usa (Symphonia `ogg`+`vorbis`).
- **Verificação grátis de ponta a ponta:** como o Symphonia já decoda Vorbis, o
  round-trip `encode_ogg → ph2d_audio_decode::decode` prova o encoder de verdade (não só
  "compila"). Dois testes verdes (estéreo 440 Hz + mono): stream `OggS`, mesmo
  channel-count/sample-rate, duração dentro de ~50 ms, RMS na faixa esperada.

### Opus fica para um passo próprio (mesmo pedido junto)

As duas únicas vias Rust de **encode Opus** cada uma quebra algo:

- [`unsafe-libopus`](https://lib.rs/crates/unsafe-libopus) — **puro-Rust** (libopus 1.3.1
  transpilado, BSD-3, encoding testado, **zero toolchain C**), mas expõe a **ABI C crua
  `unsafe`** → **não** pode ser chamada de um crate com `forbid(unsafe_code)`. Exigiria
  dropar o forbid de `ph2d-audio-encode` OU um crate irmão isolado.
- [`audiopus`](https://lib.rs/crates/audiopus) — API segura, mas `audiopus_sys` linka
  **libopus do sistema** no Linux (uma linha de `apt`, mas system dep no CI).

**Recomendação p/ o próximo passo:** Opus via `unsafe-libopus` num **crate irmão isolado
`ph2d-audio-opus`** (puro-Rust = zero toolchain nos 3 SOs; o `unsafe` fica **contido e
auditado** num só lugar, padrão idiomático), com uma API segura que a UI de export chama —
mantendo o `forbid(unsafe_code)` de `ph2d-audio-encode`. É uma decisão de arquitetura
(crate novo) que merece o seu próprio OK; por isso não entrou neste corte.

## Alternativas rejeitadas (Vorbis)

- **`vorbis`** (crate high-level) — sem update há ~6 anos, depende de `vorbis-sys` com
  libvorbis antigo e **CVEs conhecidas**. É por isso que `vorbis_rs` existe.
- **`vorbis-encoder` / `vorbis-sys`** — linkam libvorbis do sistema (system lib no CI).

## Consequências

- **+5 crates transitivos**, todos permissivos (acima). `cc` compila C vendorizado nos 3
  SOs — universal, mas **é build de C** (mais pesado que Rust puro; ainda muito abaixo do AVIF).
- Duplicata `thiserror` 1.x + 2.x (o sys-crate usa 1.x) — **warning** do deny, não erro
  (`bans ok`).
- **advisory-db local envelhece** ([`ship parity gaps`](../../../project-memory/feedback_ship_parity_gaps_ci_only.md)):
  um RUSTSEC novo contra libvorbis pode escapar local e só vermelhar no CI. Baixo risco
  (libvorbis maduro + aoTuV patcheado). O integrador confere no ship.

## Kill-criterion (fixado ANTES do build de CI)

- Se `vorbis_rs` **falhar o build no Windows OU macOS** do CI após **2** tentativas de
  fix (flags de `cc`), **reverter a dep** e reavaliar (`audiopus` com libopus estático em
  Win/Mac, ou adiar Vorbis). O gate testado do integrador (`foundational-integrate.sh`)
  compila só no host da integração — **o build cross-SO só é provado no CI do Enio.**
- Se o round-trip via Symphonia mostrar **corrupção/silêncio** (RMS fora da faixa,
  duração muito fora), **bloquear** — o encoder não existe nesta forma.
