# ADR-0123 — A fronteira ML do áudio (W7): denoise nativo via `tract` (opt-in), **`ort`/ONNX rejeitado**

> **Status:** **ACEITO (direção)** — o Enio delegou a escolha ("padrão-ouro, custo à parte") e o
> **experimento de aceitação do §3.5 decidiu**: o DeepFilterNet3 bate o nosso denoise W5 por
> **+12,07 dB** de SI-SDR (o dobro da barra de +6 dB), então o kill-criterion **não disparou** e a
> direção é **construir o denoise ML nativo via `tract`**, `ort`/ONNX **rejeitado**. O build-out abre
> a crate `ph2d-audio-ml` (opt-in) numa próxima etapa. Nenhuma dep foi adicionada à linha ainda —
> o experimento rodou num crate descartável no scratchpad.
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

## 2. Decisão (recomendação primeiro)

**Construir o denoise ML nativo via `tract`. Recusar `ort`/ONNX no workspace. W7 opt-in, com
fronteira dura:**

- **Denoise (DeepFilterNet):** caminho nativo via **`tract`** (Rust, sem lib de sistema),
  numa **crate nova isolada `ph2d-audio-ml`** atrás da feature `audio-ml` (default OFF), com o
  `libDF` 0.5.6 **vendorizado** (§3.6) e o modelo DFN3 (7,6 MB) embutido — licença **verificada
  redistribuível** (§3.4). **A FFT não vaza pro mixer RT** — mesmo confinamento do
  `ph2d-audio-spectral`/`-decode`/`-encode`. **O experimento do §3.5 justificou:** +12 dB sobre o
  W5, o dobro da barra. Não é melhora marginal.

- **Demucs / separação de stems:** **fora do workspace.** Ou um **worker/serviço externo** offline
  (o app chama um binário, não linka a runtime), ou — se tiver de ser em-processo um dia — **também
  via `tract`** (que carrega o mesmo `.onnx`), **nunca `ort`**.

- **`ort` (ONNX Runtime): rejeitado como dep do workspace** pelos fatos do §3 (pré-release; baixa
  um binário C++ da rede *durante o build*; 72 crates + pilha TLS/HTTP). Isso quebra CI hermético e
  repete, amplificada, a dor de `*-sys` do AVIF.

- **Ritmo:** o Enio pediu o padrão-ouro sem olhar custo, e o §3.5 tirou a dúvida — **construir**.
  Padrão-ouro aqui não foi "empilhar a IA", foi **provar que ela ganha antes de pagar**: o número
  (+12 dB) é o que autoriza os ~130 crates + o modelo. Se o experimento tivesse empatado, a
  resposta padrão-ouro seria a oposta (o W5 já basta). A evidência é que decide, não a vontade.

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

### 3.4 O modelo — verificado, **redistribuível**

`DeepFilterNet3_onnx.tar.gz` = **7,6 MB** (a variante `_ll` low-latency = 34,7 MB; usamos a
padrão). Licença: os arquivos `LICENSE-MIT`/`LICENSE-APACHE` do repo são **repo-wide** (MIT ©
2021 Hendrik Schröter), e — a prova mais forte que a redação do README — **o próprio autor
empacota o modelo dentro do crate MIT/Apache dele**: o `libDF` tem a feature `default-model` que
faz `include_bytes!` do `.tar.gz` e o embute no binário, e esse binário (a CLI `deep-filter`, o
plugin LADSPA) é **distribuído em releases e empacotado por distros / PipeWire**. Distribuir o
peso sob MIT/Apache já é a prática do mantenedor. **Redistribuição resolvida.**

### 3.5 O experimento de aceitação (§4) — **rodado, decisivo**

Antes de pagar pela complexidade, rodei o kill-criterion num crate descartável (scratchpad, zero
toque na linha), sobre o **próprio fixture do autor**: `noisy_snr0.wav` (voz real a 0 dB SNR) com
a referência limpa pareada `clean_freesound_33711.wav` — 48 kHz mono, 10,6 s. Métrica: **SI-SDR**
(scale-invariant SDR, o padrão de avaliação de denoise), com alinhamento de atraso por correlação
cruzada. DFN pela CLI oficial `deep-filter` v0.5.6 (`-D` compensa o atraso); o nosso W5 pelo
`ph2d_audio_spectral::denoise` com o profile aprendido do trecho mais silencioso do clipe (o uso
real: o usuário seleciona um vão), varrendo `amount` 0,6/0,8/1,0.

| sinal | SI-SDR vs limpo | ganho sobre a entrada |
|---|---|---|
| entrada com ruído | 6,05 dB | — |
| **nosso denoise W5** (melhor, amount 0,6) | 7,94 dB | **+1,89 dB** |
| **DeepFilterNet3** | 20,01 dB | **+13,96 dB** |

**Vantagem do DFN sobre o W5: +12,07 dB** — o dobro da barra de +6 dB do §4. O resultado é robusto
a folga de medição: mesmo que o harness do W5 esteja subestimando 2–3 dB, o DFN a +14 limpa a
barra com sobra. Bate exatamente onde importa pra jogo: ruído **não-estacionário** sobre voz, o
caso em que a subtração espectral sofre. **Kill-criterion NÃO disparou → construir.**

### 3.6 Nota de build-out (achada no experimento)

O `deep_filter` que denoiza (`libDF` **0.5.7-pre**) **não está no crates.io** e o **git HEAD não
compila** contra o `tract` 0.21.17 atual (API drift: `symbol_table` sumiu do `InferenceModel`, 17
erros de tipo). Além disso `kstring 2.0.3` (transitivo via `liquid`/`tract-nnef`) exige **rustc
1.96** > nossa pin 1.95 (pinável em 2.0.2). **Portanto o build-out fixa o tag `v0.5.6`** (o release
que gerou a CLI usada no §3.5, cujo `libDF` casa com o `tract` que ele fixa) **ou vendoriza o
`libDF` 0.5.6** (MIT/Apache, cópia com atribuição) — **nunca o HEAD**. Vendorizar é o mais limpo
para um produto (sem git-dep sobre `-pre`), e é o padrão que a linha já usa para dep sensível
(`ph2d-audio-opus`).

---

## 4. Conjunto de aceitação e kill-criterion — declarados antes, e o §2 já cumpriu o crux

Declarados antes, para o "denoise ML" não virar alvo irrefutável (DIRETIVA §5):

1. **Feature OFF = build byte-idêntico ao de hoje.** `cargo build` default não resolve `tract`,
   não compila C novo, não muda o lockfile do caminho quente. Gate: a feature `audio-ml` desligada
   não aparece em `cargo tree` do `ph2d-host-desktop`.
2. **Feature ON = denoise que ganha do W5.** **CUMPRIDO no §3.5:** +12,07 dB de SI-SDR sobre o W5
   no fixture do autor a 0 dB SNR (barra = +6 dB). Este gate reaparece no build-out como teste
   sobre o efeito integrado (o §3.5 rodou a CLI oficial; o build-out mede o **nosso** wrapper de
   `tract` no mesmo fixture, provando paridade com a CLI). **Se o wrapper não reproduzir o ganho, a
   feature não existe.**
3. **A runtime ML nunca toca a thread de áudio.** O denoise é offline/control-thread (como toda a
   `-edit`); o mixer RT (`ph2d-audio`) segue sem alcançar `tract`. Gate irmão do
   `no_codec_reaches_the_mixer`: `no_ml_runtime_reaches_the_mixer`.
4. **A rack não regride.** Os gates da rack seguem verdes; nenhum efeito muda.

**Demucs/stems** não têm conjunto de aceitação aqui **de propósito** — a decisão é que ele **não
entra no workspace**. Se for pedido, abre ADR próprio para o caminho worker/serviço.

---

## 5. Consequências

- **Construindo (a direção):** +~130 crates atrás de `audio-ml` (**OFF por default** — build de hoje
  intocado), `cc` no build (CI já tem, via vorbis/AVIF), `libDF` 0.5.6 vendorizado + modelo DFN3 de
  7,6 MB no repo. Ganho medido: **+12 dB** de SI-SDR no diálogo ruidoso.
- **`ort` fica barrado por contrato:** ninguém lê "Demucs via `ort`" no `02_plano`, faz
  `cargo add ort`, e faz o próximo CI limpo baixar a runtime C++ do onnxruntime pela rede. O gate
  `no_ml_runtime_reaches_the_mixer` + a ausência de `ort` no lockfile são a cerca.
- **Reversível:** se o build-out esbarrar em algo intransponível (o `tract` vendorizado brigar com
  a MSRV do CI, p.ex.), o custo afundado é uma crate opt-in que ninguém liga por default — some sem
  tocar o resto do áudio.

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

## 7. Estado da decisão

O Enio delegou a escolha ("padrão-ouro, custo à parte") e o experimento do §3.5 decidiu. Portanto:

1. **`ort`/ONNX no workspace: NÃO** — barrado por contrato (§3.3).
2. **Demucs/stems: fora do workspace** (worker/serviço externo; ADR próprio se um dia).
3. **Denoise ML nativo (`tract`, opt-in): CONSTRUIR** — o build-out abre `ph2d-audio-ml`,
   vendoriza o `libDF` 0.5.6, embute o DFN3, e o gate #2 do §4 vira teste do nosso wrapper.

**Ponto de veto do dono:** as únicas coisas que só o Enio decide são (a) aceitar os ~130 crates +
7,6 MB de modelo no repo do produto e (b) a leitura de licença do §3.4 (o peso é redistribuível
porque o autor já o distribui no crate MIT/Apache dele). Se qualquer um dos dois for "não", o
build-out para e o áudio fecha no que já tem — sem prejuízo, porque nada foi adicionado à linha
ainda.

Se a resposta for "deferir", o módulo de áudio está **fechado** e este ADR é o contrato que impede
a regressão de dep. Se for "construir", a próxima jornada abre `ph2d-audio-ml` com o §4 congelado.
