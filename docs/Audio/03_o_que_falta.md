# Áudio — o que falta (e o que **deliberadamente** não vai ser feito)

> **O módulo fechou: W1 → W7.** Este doc existe porque a lista de pendências anterior (a linha
> "Aberto" do CLAUDE.md) **mentia** — trazia "batch LUFS" duas vezes e o rename do Gate, e os três
> já estavam construídos. Isso não é ruído cosmético: uma lista velha é a instrução que faz a
> próxima LLM reconstruir o que existe (aconteceu comigo em 2026-07-15).
>
> Regra deste arquivo: **cada item diz o que é, por que ainda não existe, e o que o acordaria.**
> Item sem gatilho declarado vira dívida fantasma. Pesquisa: [`01`](01_editor_pesquisa_e_plano.md) ·
> plano/como: [`02`](02_plano_implementacao_completo.md) · bugs: [`BUGS_audio.md`](BUGS_audio.md).
> **Data:** 2026-07-15.

---

## 1. O que está PRONTO (para não reconstruir)

Rápido, porque o risco aqui é reconstruir, não esquecer:

- **W1-W3:** painel + overlay, transporte, waveform, edição (trim/split/fades/normalize/zero-cross),
  undo por delta, **rack de 42 efeitos + 23 presets**.
- **W4 (voz):** de-hum, de-ess, de-plosive, leveler/AGC, transient, ring mod, pitch/formant
  (granular, sem FFT), distortion, exciter, comms (telephone/radio/helmet).
- **W5 (espectral):** spectrogram, seleção T-F, **spectral repair**, **denoise** (Ephraim-Malah),
  de-clip. [ADR-0122](../architecture/decisions/0122-audio-spectral-fft-via-realfft.md).
- **W6 (asset-prep):** loop points + markers, containers de variação, import por convenção,
  force-mono, **batch LUFS** (`audio/editor/batch.rs` + `AEDIT_BATCH_LUFS`, intent consumido em
  `render_loop/mod.rs`), export **WAV/Ogg/Opus** + **variantes por plataforma** (Export Set).
- **W7 (ML):** **AI Denoise (Voice)** — DeepFilterNet via `tract`, feature `audio-ml` OFF por
  default. [ADR-0123](../architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md).
- **Motor:** streaming/residência ([ADR-0118](../architecture/decisions/0118-audio-streaming-voices-residency.md)),
  memória medida ([ADR-0117](../architecture/decisions/0117-audio-editor-memory-is-measured-not-declared.md)),
  preview O(1) ([ADR-0120](../architecture/decisions/0120-audio-preview-is-a-buffer-you-own-not-a-buffer-you-rebuild.md)).

---

## 2. Fila real (o que falta, em ordem)

### 2.1 Progresso de operação longa — **em construção (2026-07-15)** · **do APP, não do áudio**

**O que é:** feedback visual enquanto uma operação longa trabalha. **Decisão do Enio (2026-07-15):
serve o app inteiro**, não só o AI Denoise — o áudio é apenas o **primeiro consumidor**. Isso o
torna infra compartilhada (irmã do `ToastQueue`), e é o que impede que cada módulo invente a sua
(o batch LUFS, o export, o upscale e o painter têm o mesmo problema).

**Os números que definem o escopo** (medidos, não estimados — leia antes de mexer):

| | release | debug |
|---|---|---|
| clipe de 4 s | **0,16 s** (0,03× tempo-real) | 2,68 s (**16×** mais lento) |
| clipe de 16 s | 0,50 s | 8,06 s |
| carga do modelo (`DfTract::new`) | **~50 ms** | — |

Três coisas caem daí, e as três são contra-intuitivas:
1. **Cachear o `DfTract` NÃO é o ganho** que a leitura do código sugere (ele é construído por
   chamada, mas custa 50 ms).
2. **O "trava alguns segundos" do smoke era o build debug**, não o produto — daí o `--release`
   agora obrigatório na doc do smoke.
3. **A barra só importa em take longo:** 3 min de VO ≈ **5 s** de UI congelada. É esse o caso.

**Por que não é pequeno:** o shell **não tem padrão async** (zero worker thread para op de editor)
e o design system **não tem widget de progresso**; `editor_denoise_ml` **bloqueia a thread de UI**,
então *nenhuma barra pode ser pintada* antes de o trabalho sair da thread. **Barra que não anda é
pior que barra nenhuma.**

### 2.2 Split do `fx_presets.rs` (631 LOC) — **dono: os presets, não o W7**

Herdado do rename do Gate (`a5ec9d7a`), greened com o marker sancionado `ph2d-loc-cap` para a linha
fechar. O conserto de verdade é um split por dados (a tabela de presets sai para um módulo irmão).

### 2.3 Smoke de stereo do AI Denoise

O wrapper processa por contagem de canais e o fixture é **mono** — o caminho stereo compila e nunca
foi exercido. Um clipe stereo com ruído fecharia o buraco.

### 2.4 Backlog pequeno (do `02_plano` §4)

- Toggle *enabled* por-entry na UI de variação (o modelo **já** carrega o campo — é só UI).
- O manifesto de variação guarda **caminho absoluto** (relativo seria portátil).
- **Reverb por convolução** (o atual é Freeverb algorítmico; o `LoadIR` já existe).

---

## 3. Deliberadamente NÃO feito — as cercas de Chesterton

> **Não construa nada desta seção sem o consumidor.** Não é dívida: é decisão, e o motor por trás
> **já está pronto e gateado**. O que falta é a *política*, e política sem usuário é chute.

O eixo é um só: **streaming serve o JOGO, não o editor.** O editor **abre** um clipe para editá-lo
— isso é residente por definição (não se repara espectralmente o que está passando num ring).
[ADR-0118 §5](../architecture/decisions/0118-audio-streaming-voices-residency.md).

| Item | Quem seria o **consumidor real** | Por que hoje é vazio |
|---|---|---|
| **Toggle "Streamed" no Delivery** | um **jogo/cena que carrega assets** e honra a escolha por-asset | o Delivery *exporta* arquivos; nada no PH2D depois os *carrega como um jogo*. O toggle marcaria um flag que **nenhum código lê** — checkbox que mente |
| **Seek/scrub num stream** | a **timeline com uma cama de música longa** (scrub do playhead), ou um jogo saltando para um marker | o transporte do editor faz seek em voz **residente** — o clipe está na memória, seek é mover um cursor. Num stream o produtor tem de **reposicionar o decoder e descartar o ring**, e *como* depende de quem pede (amostra-exata? snap no marker?) |
| **Pitch ao vivo num stream** | um **motor de carro cuja rotação sobe** (o "blend por parâmetro — RPM/velocidade/vida" do [`01`](01_editor_pesquisa_e_plano.md) §2.7) | muda a taxa de consumo do ring: é política de **produtor**, não de mixer. Pitch em SFX **residente** já funciona |

**O gatilho:** as três acordam quando o PH2D ganhar **um caminho de runtime que carrega áudio**
(cena/jogo tocando assets, não o editor dando preview) — ou, mais perto, quando a **timeline
hospedar uma faixa longa**.

**Por que esperar em vez de "deixar pronto":** (a) sem consumidor você não sabe a **forma** — chutar
é construir a coisa errada com confiança; (b) sem consumidor não há gate que prove **em contexto**,
e sobra código unit-verde que ninguém exercita (a lição que mais se repete aqui: *passa no CI e está
morta*); (c) **botão que não faz nada é pior que botão que falta**.

---

## 4. Fora de escopo do módulo (outros donos)

- **W7 pesado — Demucs / separação de stems:** **fora do workspace por contrato**
  ([ADR-0123 §2](../architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md)).
  Se um dia for pedido: worker/serviço externo, ou `tract` — **nunca `ort`** (pré-release, baixa a
  runtime C++ da rede durante o build, 72 crates + pilha TLS).
- **TTS / voice-clone:** integração externa, não nativo ([`01`](01_editor_pesquisa_e_plano.md) §2.6).
