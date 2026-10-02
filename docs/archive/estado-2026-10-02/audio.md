# ARQUIVO — CLAUDE.md (história, 18 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-530,549-560` do original.
>
> ⚠️ **A única alteração ao corpo:** 10 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Áudio** — rack com **42 efeitos + 23 presets** e cadeia editável (`ph2d-audio-edit` + painéis
  `audio-editor`/`audio-mixer`), espectral ([ADR-0122](../../architecture/decisions/0122-audio-spectral-fft-via-realfft.md)), export Ogg/Opus
  ([ADR-0113](../../architecture/decisions/0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md) / [0116](../../architecture/decisions/0116-audio-export-opus-isolated-unsafe-crate.md)), streaming de vozes
  ([ADR-0118](../../architecture/decisions/0118-audio-streaming-voices-residency.md)) e **AI denoise nativo** via `tract`
  ([ADR-0123](../../architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md), feature **`audio-ml` OFF por default**).
  ⚠️ **Invariante da rack:** todo efeito é **no-op byte-idêntico no ponto neutro** e o painel **se auto-popula** da
  tabela `KINDS` — efeito novo = variant + braços + row, **zero mudança de painel**.
  ⚠️ **Fronteiras duras, com gate:** nenhum **codec** e nenhum **runtime de ML** alcança o mixer RT.
  ⚠️ **HR-13 emendado** ([ADR-0117](../../architecture/decisions/0117-audio-editor-memory-is-measured-not-declared.md)): *quem declara budget possui um gate que **MEDE*** (dhat).
  ⚠️ `fx.rs` está **no teto de LOC** — o 43º efeito tem de orçar o split.
  **Aberto:** o backlog do módulo vive em [`docs/Audio/03_o_que_falta.md`](../../Audio/03_o_que_falta.md), **com o
  gatilho que acorda cada item** — é lá que se olha, não aqui. Cercas de Chesterton conhecidas: seek/scrub num stream ·
  pitch ao vivo num stream · toggle "Streamed" no Delivery (os três esperam um consumidor real).
  **Smokes:** `PH2D_AUDIO_DELIVERY_SMOKE` · `PH2D_AUDIO_ML_SMOKE` (+ `--features audio-ml`, e ⚠️ **`--release`**: o modelo
  é 16× mais lento em debug) · `PH2D_AUDIO_ML_SMOKE_SECS=180` (sem isso a barra de progresso passa voando).
  **Ler:** [`docs/Audio/`](../../Audio) · [handoffs](../../Audio/handoffs/README.md) ·
  [história](../estado-2026-08-18/audio.md)

