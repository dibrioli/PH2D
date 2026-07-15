# ADR-0123 — A fronteira ML do áudio (W7): denoise nativo via `tract` (opt-in), **`ort`/ONNX rejeitado**

> **Status:** **PROPOSTO** — precisa da palavra do Enio (§7). Nenhuma dep foi adicionada; este
> documento é a decisão *antes* do código, no molde do [ADR-0122](0122-audio-spectral-fft-via-realfft.md).
> **Data:** 2026-07-15
> **Contexto:** [`docs/Audio/02_plano_implementacao_completo.md`](../../Audio/02_plano_implementacao_completo.md) §W7 · [`docs/Audio/01_editor_pesquisa_e_plano.md`](../../Audio/01_editor_pesquisa_e_plano.md) §2.6
> **Relacionado:** [ADR-0113](0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md) · [ADR-0116](0116-audio-export-opus-isolated-unsafe-crate.md) · [ADR-0122](0122-audio-spectral-fft-via-realfft.md) (o precedente: a dep entra onde é irredutível, confinada, com aceitação declarada antes)

---

## 1. O pedido, e por que ele é frágil *agora*

O W7 é a última etapa planejada do módulo de áudio: **denoise por ML** (DeepFilterNet) como
efeito de voz opcional, e **separação de stems / isolamento de diálogo** (Demucs) em batch
offline. O plano (`02_plano` §W7) descreve isso como *"crate `deep_filter`, Rust, realtime"* +
*"Demucs via `ort` (ONNX), batch"*, tudo atrás de uma feature `audio-ml`.

**Duas coisas mudaram desde que esse plano foi escrito, e as duas enfraquecem o pedido:**

1. **O denoise já existe e já passou.** O kill-criterion do [ADR-0122 §4](0122-audio-spectral-fft-via-realfft.md#4-conjunto-de-aceitação-concreto-e-kill-criterion-diretiva-5)
   dizia: *"se o denoise espectral não bater o de-noise tempo-domínio na 2ª tentativa, o denoise
   vira W7 (DeepFilterNet)."* Ele **não disparou** — o spectral denoise (Ephraim-Malah
   decision-directed, W5) bateu o alvo. Ou seja: a feature-título do W7 **não preenche um buraco**;
   ela seria uma melhora marginal sobre algo que já funciona.

2. **A premissa "é uma crate" é falsa.** Verifiquei (não de memória — §3): o `deep_filter`
   publicado no crates.io é **só o front-end DSP** (STFT/ERB/normalização de banda). O runner
   neural — o que carrega o modelo e de fato limpa o ruído — **não está publicado como
   biblioteca**. Ele mora no repo do DeepFilterNet, sobre o **`tract`** (inferência Rust da Sonos),
   e teria de ser **portado/vendorizado** junto com um **arquivo de modelo**. Não é `cargo add`.

Este ADR existe para **decidir a fronteira antes** de alguém, mais tarde, reflexivamente fazer
`cargo add ort` e arrastar pra dentro do workspace exatamente o tipo de peso de CI que a lição do
AVIF (`*-sys` + toolchain) nos ensinou a recusar.

---

## 2. Decisão proposta (recomendação primeiro)

**Recusar `ort`/ONNX no workspace. Manter o W7 como opt-in, e — se e quando construído —
com uma fronteira dura:**

- **Denoise (DeepFilterNet):** caminho nativo via **`tract`** (Rust, sem lib de sistema),
  numa **crate nova isolada `ph2d-audio-ml`** atrás da feature `audio-ml` (default OFF), com o
  modelo vendorizado sob licença verificada. **A FFT não vaza pro mixer RT** — mesmo confinamento
  do `ph2d-audio-spectral`/`-decode`/`-encode`. **Só se** o Enio quiser a melhora marginal sobre o
  denoise do W5 que **já** passou.

- **Demucs / separação de stems:** **fora do workspace.** Ou um **worker/serviço externo** offline
  (o app chama um binário, não linka a runtime), ou — se tiver de ser em-processo um dia — **também
  via `tract`** (que carrega o mesmo `.onnx`), **nunca `ort`**.

- **`ort` (ONNX Runtime): rejeitado como dep do workspace** pelos fatos do §3 (pré-release; baixa
  um binário C++ da rede *durante o build*; 72 crates + pilha TLS/HTTP). Isso quebra CI hermético e
  repete, amplificada, a dor de `*-sys` do AVIF.

- **Recomendação de ritmo:** dado que o denoise do W5 passou, o movimento de padrão-ouro é
  **deferir o W7** e ratificar esta fronteira como o *contrato* do que ele pode ou não puxar. O
  módulo de áudio está fechado no que dá pra fechar bem: rack de 39 efeitos, espectral (repair +
  denoise), reparo, variação, loop/markers, entrega por 3 codecs, streaming/residência. **Um
  denoiser marginalmente melhor não paga 130 crates + um modelo.**

---

## 3. Os fatos (resolvidos agora com o toolchain do repo, 1.95 — não de memória)

Tudo abaixo veio de resolver as árvores num crate descartável, sem tocar o `Cargo.toml` da linha.

### 3.1 `deep_filter` (crates.io) **não denoiza**

- Última versão publicada: **`0.2.5`** (o plano dizia `0.5` — inexistente). Licença **MIT/Apache-2.0**.
- Árvore default: **17 crates**, e ela **reusa o `realfft`/`rustfft` que a linha já tem** (ADR-0122).
- **A API pública é DSP puro:** `DFState`, `erb_fb`, `analysis`/`synthesis` (STFT), `band_*norm`,
  `compute_band_corr`, `band_compr`. **Zero** `tract`, zero `onnx`, zero `model`, zero inferência
  (`grep -riE "tract|onnx|DfTract|infer|model"` no source = **nada**). O `Cargo.toml` dela não tem
  sequer uma feature `tract`.
- **Conclusão:** o "efeito de denoise" não vem dessa crate. Vem do runner `DfTract` do repo, que
  **não é publicado** e teria de ser portado (o repo é MIT/Apache, então **pode**).

### 3.2 O caminho nativo real: **`tract`** (o que eu recomendaria)

`tract-onnx` + `tract-pulse` **`0.21.17`** (versão que o DeepFilterNet usa; MIT/Apache):

- **130 crates** na árvore — pesado, mas **Rust**, **sem lib de sistema**, **sem download de rede
  no build**.
- **Uma ressalva honesta:** `tract-linalg` tem **`cc` como build-dependency** (kernels SIMD/asm) —
  ou seja, precisa de um **compilador C no build**. Isso o CI **já tem** (`vorbis_rs`/AVIF), e é
  fundamentalmente mais leve que o `ort`: **não baixa binário nenhum, não linka lib de sistema.**
- Puxa `liquid`/`pest`/`time` (o NNEF da tract usa templating) — daí o volume de crates.

### 3.3 `ort` (ONNX Runtime) — o que estamos **rejeitando**, e por quê

`ort` **`2.0.0-rc.12`**:

| fato | valor | por que importa |
|---|---|---|
| **Maturidade** | ainda **pré-release** (a série 2.x nunca saiu de `-rc`) | dep de produção sobre um `-rc` |
| **Tamanho** | **72 crates** | vs. 8 do `realfft` |
| **`ort-sys`** | baixa a **runtime C++ do onnxruntime pela REDE durante `cargo build`** (`download-binaries`) **ou** exige onnxruntime instalado no sistema | **quebra CI hermético/offline**; blob binário não auditável entrando no build |
| **Cauda** | `openssl-sys`, `native-tls`, `ureq`, `vcpkg`, `webpki-root-certs`, `lzma-rust2` | uma pilha TLS+HTTP+descompressão **só pra baixar** a runtime |

Isto é a lição do AVIF (`*-sys` = toolchain no CI) **amplificada**: o AVIF ao menos compila do
source com ferramentas fixas; o `ort` busca um **binário pré-compilado pela rede** a cada build
limpo. É exatamente o oposto do que o repo escolheu ser.

### 3.4 O modelo (item a verificar **antes** de vendorizar)

Os modelos vêm como `DeepFilterNet3_onnx.tar.gz` (+ variante `_ll` low-latency) — tract lê. **Não
consegui confirmar o tamanho em MB nem a licença do artefato do modelo separada do código** pela
página do repo. **Isto fica como gate de entrada:** vendorizar o modelo exige confirmar (a) o
tamanho (entra no repo ou é baixado no boot?) e (b) que o *peso* é MIT/Apache como o código.
Nenhum byte de modelo entra sem isso resolvido. (Lição: [[feedback_no_industrial_claims_without_verification]].)

---

## 4. Conjunto de aceitação e kill-criterion — **para SE/quando o W7 for construído**

Declarados antes, para o "denoise ML" não virar alvo irrefutável (DIRETIVA §5):

1. **Feature OFF = build byte-idêntico ao de hoje.** `cargo build` default não resolve `tract`,
   não compila C novo, não muda o lockfile do caminho quente. Gate: a feature `audio-ml` desligada
   não aparece em `cargo tree` do `ph2d-host-desktop`.
2. **Feature ON = denoise que ganha do W5.** Num fixture fala+ruído a 0 dB SNR, o denoise ML
   entrega **≥ 6 dB de melhora de SNR acima** do que o spectral denoise (W5) já entrega no MESMO
   fixture — medido, não afirmado. **Se não ganhar, a feature não existe** (é o kill-criterion: um
   ML que empata com o DSP que já temos não paga 130 crates + modelo).
3. **A runtime ML nunca toca a thread de áudio.** O denoise é offline/control-thread (como toda a
   `-edit`); o mixer RT (`ph2d-audio`) segue sem alcançar `tract`. Gate irmão do
   `no_codec_reaches_the_mixer`: `no_ml_runtime_reaches_the_mixer`.
4. **A rack não regride.** Os gates da rack seguem verdes; nenhum efeito muda.

**Demucs/stems** não têm conjunto de aceitação aqui **de propósito** — a decisão é que ele **não
entra no workspace**. Se for pedido, abre ADR próprio para o caminho worker/serviço.

---

## 5. Consequências

- **Se SIM à fronteira (recomendado):** o W7 fica deferido com contrato escrito. Ninguém adiciona
  `ort` por reflexo; se o denoise ML for construído um dia, já se sabe que é `tract`, isolado,
  opt-in, sobre o kill-criterion do §4. **Custo hoje: zero dep, zero código.**
- **Se SIM ao denoise ML já:** +~130 crates atrás de `audio-ml` (OFF por default), `cc` no build
  (CI já tem), modelo vendorizado sob licença verificada. Ganho: um denoiser possivelmente melhor
  que um que **já** passou.
- **Custo de NÃO decidir:** o risco real — alguém lê "Demucs via `ort`" no `02_plano`, faz
  `cargo add ort`, e o próximo CI limpo passa a baixar a runtime C++ do onnxruntime pela rede. Este
  ADR existe para tornar isso uma decisão consciente, não um acidente.

---

## 6. Alternativas consideradas

- **`ort` para tudo (o plano original):** rejeitado — §3.3.
- **Vendorizar o DeepFilterNet inteiro agora:** rejeitado por ritmo — o denoise do W5 passou; é
  esforço grande por ganho marginal. Fica disponível via §4 quando o Enio quiser.
- **Serviço externo de denoise (ElevenLabs/etc.):** fora de escopo do editor offline; um jogo não
  quer denoise por chamada de rede. Serve só para stem-split batch, se algum dia.
- **RNNoise (mais leve que DeepFilterNet):** possível, mas qualidade inferior ao DFN e ao nosso W5
  espectral — não move o ponteiro.

---

## 7. O que eu preciso do Enio

**Ratificar a fronteira do §2**, que é uma decisão em três partes:

1. **`ort`/ONNX no workspace: NÃO.** (recomendo firmemente — §3.3)
2. **Demucs/stems: fora do workspace** (worker/serviço externo, ADR próprio se um dia). 
3. **Denoise ML nativo (`tract`, opt-in):** **deferir** (recomendado — o W5 já denoiza) **ou**
   **construir agora** sobre o kill-criterion do §4.

Se a resposta for "deferir", o módulo de áudio está **fechado** e este ADR é o contrato que impede
a regressão de dep. Se for "construir", a próxima jornada abre `ph2d-audio-ml` com o §4 congelado.
