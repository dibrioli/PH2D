# ADR-0116 — Export Opus: `unsafe-libopus` + `ogg`, numa crate irmã isolada

- **Status:** **ACEITO** — Enio autorizou em 2026-07-12 ("Opus, depois memória").
- **Escopo:** módulo de áudio (`line/audio`), W6 asset-prep. Adiciona **4 deps** (2 diretas).
- **Supersede:** a seção "Opus fica para um passo próprio" do
  [ADR-0113](0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md), que **recomendou
  exatamente este desenho** e adiou a execução. Este ADR executa a recomendação.

---

## 1. O problema, e por que ele não era trivial

O editor exporta WAV (sem perda) e Ogg Vorbis (ADR-0113). Falta **Opus** — o codec moderno,
melhor qualidade por bitrate, e o padrão para voz em jogos.

O ADR-0113 já tinha feito a pesquisa e encontrado a tensão que atrasou a decisão:

> **`ph2d-audio-encode` declara `#![forbid(unsafe_code)]`** — e as duas vias Rust de encode
> Opus quebram algo. `unsafe-libopus` é puro-Rust mas expõe a **ABI C crua `unsafe`**;
> `audiopus` tem API segura mas **linka libopus do sistema** no Linux.

Ou seja: ou a gente abandona uma propriedade que a crate garante, ou a gente aceita uma dep de
sistema no CI dos três SOs.

**Nenhum dos dois.** É um falso dilema, e a saída é a que o próprio ADR-0113 recomendou:
**isolar o `unsafe` numa crate irmã**.

## 2. Decisão

**Crate nova `ph2d-audio-opus`**, com **duas deps diretas**:

- **[`unsafe-libopus`](https://crates.io/crates/unsafe-libopus) 0.2** — libopus 1.3.1
  transpilada para Rust por `c2rust`. É o **encoder de referência**, sem toolchain C.
- **[`ogg`](https://crates.io/crates/ogg) 0.9** (RustAudio) — o **contêiner**. Um `.opus` é
  Opus *encapsulado em Ogg*: páginas, CRC32, granule positions, e os headers `OpusHead` /
  `OpusTags`. O `opus_encode` só devolve pacotes crus; sem o contêiner não existe arquivo.

### O `unsafe` fica contido, e é onde ele *deve* ficar

`ph2d-audio-encode` **mantém o `#![forbid(unsafe_code)]`**. A crate nova **não** o declara —
ela não pode, porque a ABI transpilada é `unsafe` por construção — e em troca:

- o `unsafe` vive num **único módulo**, atrás de uma API segura (`encode_opus(&SampleData, …)
  -> Result<Vec<u8>>`);
- **nenhum ponteiro cru cruza a fronteira** da crate;
- é a divisão idiomática do ecossistema Rust (o que `audiopus` faz sobre `audiopus_sys`, o que
  toda `*-sys` wrapper faz), e a razão pela qual `forbid(unsafe_code)` numa crate *vizinha*
  continua significando alguma coisa.

## 3. Os fatos (verificados AGORA, não de memória)

Árvore resolvida com o toolchain do repo (1.95):

```
unsafe-libopus v0.2.0        ogg v0.9.2
├── arrayref v0.3.9          └── byteorder v1.5.0
├── num-complex v0.4.6  (já no lockfile — veio com o realfft)
└── num-traits v0.2.19  (idem; autocfg como build-dep, idem)
```

| critério | resultado | como foi verificado |
|---|---|---|
| **Crates NOVAS no lockfile** | **4** (`unsafe-libopus`, `ogg`, `arrayref`, `byteorder`) — as outras 3 já entraram com o `realfft` | `cargo tree` numa crate-sonda + grep no `Cargo.lock` |
| **Licenças** | `unsafe-libopus` **BSD-3** · `ogg` **BSD-3** · `arrayref` **BSD-2** · `byteorder` **Unlicense OR MIT** — todas permissivas | `cargo info` de cada uma |
| **Toolchain C** | **NENHUMA tem `build.rs`.** Zero `cc`, zero `links`, zero lib de sistema | `test -f build.rs` + grep por `cc::`/`links` nos 4 diretórios do registry |
| **RUSTSEC** | **limpo** — nenhuma das 4 tem sequer diretório na advisory-db | advisory-db local |
| **`unsafe`** | contido em `ph2d-audio-opus` (a ABI transpilada). `ph2d-audio-encode` **mantém `forbid(unsafe_code)`** | por construção; gate abaixo |

**Comparação honesta com o que já está no repo:** o `vorbis_rs` (ADR-0113) **compila C**
(libvorbis + libogg vendorizados, via sys-crates). Esta árvore **não compila nada** — é
estritamente mais leve para o CI do que a dep de codec que já aceitamos.

### Alternativas consideradas

- **`audiopus`** — API segura, mas `audiopus_sys` **linka libopus do sistema** no Linux. Uma
  linha de `apt` no CI, mais o mesmo problema no macOS e no Windows. É exatamente a dor que o
  `libavif-sys` custou ([memória](../../../project-memory/project_imageio_avif_pathc_2026_05_28.md)),
  e ela é evitável aqui. **Rejeitado.**
- **`opus` (0.3)** — bindings seguros, mas também sobre libopus do sistema. Mesmo problema.
- **Dropar o `forbid(unsafe_code)` de `ph2d-audio-encode`** para chamar a ABI direto: paga com
  uma propriedade que vale para *toda* a crate de encoding, para economizar uma crate.
  **Rejeitado** — é trocar uma garantia por conveniência.
- **Escrever o muxer Ogg à mão** (evitando a dep `ogg`): páginas, lacing, CRC32 com polinômio
  próprio, granule positions. ~150 linhas de formato binário onde um bit errado = arquivo
  corrompido, e o CRC não perdoa. A DIRETIVA §1 manda **portar, não reinventar**, quando
  existe implementação de referência — e existe, BSD-3, da RustAudio (a mesma org do `lewton`).
  **Rejeitado.**

## 4. Conjunto de aceitação **concreto** (declarado ANTES de construir)

1. **O arquivo é um `.opus` de verdade:** o que sai é **decodificável pelo nosso próprio
   decoder** (Symphonia, já no repo) de volta a PCM, com a duração e a contagem de canais
   corretas. Round-trip real, não "o encoder não retornou erro".
2. **É de fato lossy-mas-fiel:** num clipe de fala, o SNR do decodificado contra o original
   fica **≥ 10 dB** a 96 kbps. (Opus é transform-coding: a forma de onda não é preservada
   amostra a amostra, mas o sinal é.)
3. **Comprime:** o `.opus` é **< 25 %** do WAV PCM16 equivalente, na qualidade default.
4. **Delivery sabe o preço:** o codec entra em `Codec::ALL`, o painel mostra o tamanho **real**
   (medido pelo encoder, como os outros — nunca estimado por bitrate), e o aviso de
   `carries_metadata = false` aparece (Opus não carrega os `smpl`/`cue` do WAV).
5. **O `forbid(unsafe_code)` de `ph2d-audio-encode` sobrevive** — gate executável.

**Kill-criterion:** se o encoder não produzir um arquivo que o Symphonia leia de volta **na 2ª
tentativa**, a feature não existe nesta forma — o Vorbis continua sendo o caminho lossy e o
Opus vira `audiopus` + dep de sistema (com o custo de CI assumido explicitamente), ou nada.

## 5. Consequências

- **+4 crates** no lockfile, todas permissivas, **zero C**, RUSTSEC limpa.
- **CI:** nenhuma ferramenta de sistema nova (ao contrário do AVIF, e mais leve que o Vorbis).
- **`ph2d-audio-encode` mantém `forbid(unsafe_code)`**; o `unsafe` fica auditado num só lugar.
- **Opus só aceita 48/24/16/12/8 kHz.** Um clipe em 44,1 kHz é **reamostrado para 48 kHz** no
  encode — e isso é dito no código, porque um resample silencioso é a classe de bug que a
  auditoria do W5 pegou duas vezes.
- O RT mixer (`ph2d-audio`) segue intocado: encoding é control-thread, offline.
