# ADR-0115 — FFT para o módulo espectral de áudio (W5): `realfft`, em crate isolada

- **Status:** **ACEITO** — Enio autorizou a dep em 2026-07-12. O conjunto de aceitação do §4 está
  **congelado** e o W5 abriu sobre ele.
- **Data:** 2026-07-12
- **Contexto:** [`docs/Audio/02_plano_implementacao_completo.md`](../../Audio/02_plano_implementacao_completo.md) §W5
- **Relacionado:** [ADR-0113](0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md) (o precedente de dep de áudio)

---

## 1. O que está sendo pedido, e por que só agora

O **W5 (Espectral)** é a última wave grande do módulo: **spectrogram** (STFT) alternando com
a waveform, **seleção tempo-frequência**, **spectral repair/inpaint** (o "heal brush" — apagar
uma tosse ou um bipe pontual interpolando os bins vizinhos, mantendo o resto), **spectral
denoise** (subtração espectral / Wiener por bin, aprendendo um profile de ruído) e **de-clip**.

**Nada disso dá pra fingir no tempo.** Trabalho *por bin* exige os bins.

Vale registrar que isso **não** é o pedido reflexo de quem quer FFT pra tudo. O W4 inteiro
foi entregue **sem FFT**, e de propósito:

| feature | ferramenta usada | por que NÃO foi FFT |
|---|---|---|
| Pitch Shift | **WSOLA** (tempo) | a emenda por similaridade de onda é o que mantém a nota afinada; um phase vocoder não era necessário |
| Formant Shift | **LPC** + warp da resposta impulsiva | o modelo fonte-filtro *é* a ferramenta certa; escalar a IR no tempo escala o espectro em frequência de graça |
| De-Click | **LPC + LSAR** (tempo) | um clique é o que o modelo AR não prevê — detectar e reparar são ambos tempo-domínio |

Ou seja: a dep está sendo pedida **exatamente onde ela é irredutível**, depois de três
features que poderiam ter apelado pra ela e não apelaram.

---

## 2. Decisão proposta

**Adicionar `realfft = "3.5"`**, e **confinar a FFT numa crate nova `ph2d-audio-spectral`**.

O confinamento é o ponto arquitetural, não um detalhe: é o mesmo padrão que já isola
`ph2d-audio-decode` (Symphonia) e `ph2d-audio-encode` (libvorbis) — **nenhuma dep de codec/DSP
pesado alcança o mixer RT** (`ph2d-audio`). A crate espectral é control-thread, como
`ph2d-audio-edit`.

`realfft` é uma casca fina sobre `rustfft` para sinais **reais** (áudio é real): metade do
trabalho e metade da memória de uma FFT complexa. É a escolha certa para STFT.

---

## 3. Os fatos (verificados agora, não de memória)

Árvore resolvida com o toolchain do repo (1.95):

```
realfft v3.5.0
└── rustfft v6.4.1
    ├── num-complex v0.4.6 → num-traits v0.2.19
    ├── num-integer v0.1.46
    ├── primal-check v0.3.4
    ├── strength_reduce v0.2.4
    └── transpose v0.2.3
```

**8 crates ao todo** (+ `autocfg` como build-dep).

| critério | resultado | como foi verificado |
|---|---|---|
| **Licenças** | `realfft` MIT · todas as outras **MIT OR Apache-2.0** — permissivas, sem copyleft | `license` do `Cargo.toml` de cada crate no registry |
| **C / `*-sys`** | **NENHUM.** Rust puro, sem `cc`, sem `links`, sem lib de sistema | grep por `[build-dependencies]` / `cc =` / `links =` — zero |
| **RUSTSEC** | **limpo.** A árvore tem UMA advisory (`RUSTSEC-2023-0080`, buffer overflow em `transpose`), **corrigida em ≥ 0.2.3** — e a árvore resolve **exatamente 0.2.3** | advisory-db local (atualizada há 30 h) |
| **`unsafe`** | `rustfft` usa `unsafe` internamente nos kernels SIMD (37 arquivos). **Nossas crates seguem `#![forbid(unsafe_code)]`** — o `unsafe` fica na dep, não em nós | grep no source do registry |

**Comparação honesta com o precedente:** o `vorbis_rs` que já está no repo (ADR-0113)
**compila C**. Essa aqui não compila nada — é estritamente mais leve pro CI (nada de meson /
nasm / cmake, que é o que o `libavif-sys` custa hoje).

### Alternativas consideradas

- **`rustfft` direto** (sem o `realfft`): funciona, mas paga FFT complexa num sinal real —
  2× o trabalho e a memória, à toa. `realfft` é a mesma árvore + uma casca.
- **FFT própria (zero dep):** rejeitada. Uma FFT *correta e rápida* (radix-mista, SIMD,
  numericamente estável) não é coisa pra escrever à mão: o risco seria nosso e o ganho, zero.
  Isto é justamente o que a DIRETIVA §1 manda (existe algoritmo/impl de referência → porte,
  não reinvente).
- **`kofft` / `ndrustfft`:** resolvem outros problemas (no_std/MCU, N-dimensional). Nenhum
  ganho aqui.

---

## 4. Conjunto de aceitação **concreto** e **kill-criterion** (DIRETIVA §5)

Declarados **antes** de construir, para que "espectral" não vire alvo irrefutável.

**Aceitação (todas as quatro, ou a wave não fechou):**

1. **Spectrogram:** alterna com a waveform no overlay e repinta em tempo interativo num clipe
   de **60 s** (STFT calculada uma vez e cacheada; o *zoom* não recalcula).
2. **Spectral repair:** num fixture com um **bipe tonal** por cima de fala, apagar a região
   T-F remove o bipe com **≥ 20 dB** de queda na energia daquele bin e **≤ 1 dB** de mudança
   na energia da fala fora da região.
3. **Spectral denoise:** num fixture fala+ruído a **0 dB SNR**, o denoise entrega **≥ 8 dB**
   de melhora de SNR **sem** musical noise audível (medido: a variância dos bins residuais
   não pode subir contra o baseline do gate de ruído tempo-domínio que já temos).
4. **A rack não regride:** os 5 gates da rack seguem verdes; nenhum efeito do W4 muda.

**Kill-criterion:** se o **denoise** não bater o de-noise tempo-domínio que já existe (o `Gate`
+ `De-Hum`) no fixture de SNR **após a 2ª tentativa**, a feature **não existe nesta forma** —
mantemos spectrogram + repair (que não têm substituto) e o denoise vira W7 (DeepFilterNet, ML,
atrás de feature-flag), em vez de ficar empurrando um Wiener medíocre.

---

## 5. Consequências

- **+8 crates** no lockfile, todas permissivas, sem C, sem RUSTSEC aberta.
- **CI:** nenhuma ferramenta de sistema nova (ao contrário do AVIF).
- **`cargo machete`/`deny`:** a dep é usada de fato pela crate nova; sem exceções a criar.
- **RT mixer intocado:** a FFT não alcança `ph2d-audio`.
- Se a decisão for **não**, o W5 fica em aberto e o módulo de áudio está **fechado no que dá
  pra fechar sem FFT** — o que já é uma rack de 37 efeitos, reparo, variação, loop/markers e
  entrega por codec.

---

## 6. O que eu preciso do Enio

**Uma palavra: sim ou não à dep.** Se sim, a linha abre o W5 na próxima jornada com o conjunto
de aceitação do §4 congelado.
