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
  **edição por-intervalo O(seleção)** ([ADR-0124](../architecture/decisions/0124-audio-a-range-edit-must-be-told-its-range.md)
  — gain/normalize/reverse/invert/DC/fade/silence/Apply: 22,4 ms → 0,011 ms num clipe de 3 min, e
  **não escala com o clipe**), render de preview O(seleção)
  ([ADR-0120](../architecture/decisions/0120-audio-preview-is-a-buffer-you-own-not-a-buffer-you-rebuild.md)
  — mas o **frame** do drag ainda paga a waveform inteira: §2.2, medido).

---

## 2. Fila real (o que falta, em ordem)

### 2.1 Progresso de operação longa — **FECHADO (2026-07-16)** · **do APP, não do áudio**

**O que é:** feedback visual enquanto uma operação longa trabalha. **Decisão do Enio (2026-07-15):
serve o app inteiro**, não só o AI Denoise — o áudio é apenas o **primeiro consumidor**. Isso o
torna infra compartilhada (irmã do `ToastQueue`), e é o que impede que cada módulo invente a sua
(o batch LUFS, o export, o upscale e o painter têm o mesmo problema).

**Os números que definem o escopo** (medidos, não estimados):

| | release | debug |
|---|---|---|
| clipe de 4 s | **0,16 s** (0,03× tempo-real) | 2,68 s (**16×** mais lento) |
| clipe de 16 s | 0,50 s | 8,06 s |
| carga do modelo (`DfTract::new`) | **~50 ms** | — |

Três coisas caem daí, e as três são contra-intuitivas:
1. **Cachear o `DfTract` NÃO é o ganho** que a leitura do código sugere (ele é construído por
   chamada, mas custa 50 ms).
2. **O "trava alguns segundos" do smoke era o build debug**, não o produto.
3. **A barra só importa em take longo:** 3 min de VO ≈ **5 s** de UI congelada. É esse o caso.

#### O que landou

**`ph2d-editor-core/src/progress.rs`** — o 1º padrão async do shell, e ele vira precedente:

- **`Progress`**: handle clonável e thread-safe (`Arc` + `AtomicU32` de fração em ponto-fixo ppm +
  `AtomicBool` de terminado + label). O worker escreve, o pintor lê, **é o mesmo `Arc`** — uma
  fonte de verdade, sem drift. (A alternativa óbvia — o worker manda eventos e a UI acumula a
  própria cópia — tem duas cópias do mesmo fato, e duas cópias divergem.)
- **`Job<T>`**: `spawn(label, FnOnce(&Progress) -> T)` · `try_take()` **nunca bloqueia**
  (`JoinHandle::is_finished` antes do `join`, então o `join` retorna na hora) · `is_finished()`.
  **`done` é setado por um drop guard**, não por uma linha depois do closure: `denoise_ml` tem dois
  `.expect()`, e um worker que entra em pânico sem soltar a flag deixaria uma barra eterna na tela
  — a UI congelada que a barra existe pra evitar, agora com um widget mentindo sobre isso.
- **`JobQueue`**: guarda `Progress` (não `Job<T>` — `Job` é genérico no resultado, então uma fila
  de jobs só poderia guardar um tipo de trabalho). `tick()` por frame descarta os terminados
  (`retain`, não `pop_front`: jobs terminam fora de ordem). Cap 8, drop silencioso — **derrubar a
  barra nunca derruba o trabalho**.

**Pintura:** a barra é a **`widget::ProgressBar` do design system** — que **já existia** (o brief
dizia que não; a única consumidora era a gallery). Um track desenhado à mão aqui seria uma 2ª
resposta pra "como é uma barra neste app", e as duas divergiriam. O card em volta (superfície
elevada + borda + label) é a *coluna* e é deste módulo.

**A coluna topo-centro tem 2 inquilinos e UMA régua** (`progress::column_row`): **os toasts ficam
no topo e as barras empilham embaixo**. A ordem não é arbitrária — um toast se autodestrói em 3 s,
então tem uma chance de ser lido, e a posição dele não pode depender de haver ou não um job rodando
ao fundo: mensagem que aparece em lugar diferente por motivo invisível é mensagem que não se aprende
a achar. A barra é persistente e se anuncia sozinha (é a coisa que *se move*), então é ela que cede.
A régua mora no `progress.rs` e não junto do pintor de toast porque **`paint.rs` está no teto de LOC
congelado** (884, pode encolher e nunca crescer — o gate `architecture_workspace_file_loc_cap`);
o pintor de toast agora mede contra ela (paint.rs 884 → **879**).

**A fronteira com o ML** (⚠️ o cuidado que o brief pediu): `ph2d-audio-ml` **não** depende do
editor. `denoise_ml_with_progress(data, amount, &dyn Fn(f32))` — callback puro, e o shell faz a
ponte (`&|f| p.set(f)`). O mesmo argumento de contenção que meteu o `tract` lá dentro corta pros
dois lados: nada pesado entra, e **nenhuma UI entra**. `denoise_ml` delega com um callback vazio —
**um caminho de código só**, então não há 2ª implementação pra divergir (parity gate intacto).

**Onde o progresso nasce:** o laço de hops do `enhance_48k` — é onde os segundos de fato vão, e a
única parte cujo custo restante é *conhecido* (hops são uniformes). O resample da fronteira fica de
fora: o modelo custa ~30 ms por segundo de áudio e o resample bem menos de 1, então pesá-los seria
inventar precisão (um clipe fora de 48 kHz fica em 0 % por um instante antes de a barra andar).

**Os botões durante o job:** a seção Spectral inteira fica inerte — cada controle **edita** o clipe,
e um 2º edit no meio do voo faria o resultado que chega por último descartar o primeiro em silêncio.
**A recusa está no `event.rs`**, não só no dim (dim é cosmético). Perguntada **UMA vez** pros quatro
botões: era por-arm primeiro, e o **Repair foi esquecido na hora** — o gate de seam pegou
([[feedback_a_condition_that_enumerates_its_readers_rots]]). 3 camadas: paint dima · `event.rs`
recusa · `editor_denoise_ml` recusa (onde o trabalho está). **O toggle de vista NÃO fica inerte** —
é decisão, não esquecimento (ele desenha, não edita; proibir olhar o spectrogram enquanto se espera
protegeria nada), e está **pinada num gate** pra ninguém "consertar".

**O resultado stale é descartado:** a UI fica viva enquanto o modelo roda (era o objetivo), e só a
seção Spectral está dimmed — Cut/Paste/Normalize continuam lá. Commitar um resultado calculado de um
buffer que não está mais na tela jogaria fora o que o usuário fez nesse meio tempo. O worker devolve
`MlDenoise { source, out }` e o instalador compara identidade de buffer (o idioma `BufKey`; é sólido
porque o `source` volta **vivo** — um endereço não pode ter sido reciclado por um buffer que ainda
seguramos).

#### Como VER a barra (um indicador que ninguém observa não foi verificado)

O clipe default de 4 s denoisa em ~0,16 s: **a barra passa voando**, e isso é o produto sendo rápido.
`PH2D_AUDIO_ML_SMOKE_SECS` encena o caso pro qual a barra foi construída:

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
  PH2D_AUDIO_ML_SMOKE=1 PH2D_AUDIO_ML_SMOKE_SECS=180 \
  cargo run --release -p ph2d-host-desktop --features audio-ml
```

Take de 3 min → ~5,4 s de inferência. Enquanto roda: a janela **continua redesenhando**, a barra
sobe no topo-centro, a porcentagem anda, a seção Spectral está dimmed com o motivo na status line —
e no fim o chiado sumiu. (O console imprime a estimativa do clipe encenado.)

#### Aberto

- **A `JobQueue` tem um consumidor só.** Batch LUFS, export, upscale e o painter são os próximos —
  o padrão está pronto pra ser copiado, mas copiar é trabalho deles.
- **`build_a11y` não é enxertado na árvore** — nem o do `ToastQueue` é (é API desenhada e não
  consumida, o mesmo status do resto do design system). Quando o shell enxertar toast, enxerta barra.
- **Cancelar não existe.** Nada aborta um job em voo. Para 5 s é defensável; para o primeiro job de
  minutos deixa de ser (e aí é um `AtomicBool` que o callback lê — o callback já é chamado por hop).
- **A barra some se o job entra em pânico** e não há toast de erro (o pânico vai pro stderr).

### 2.2 O preview de knob ainda RECONSTRÓI a waveform inteira por frame — **medido**

**O que é:** `PreviewScratch::step` termina em `EditClip::new(buf.clone())`, e `EditClip::new` **é**
`PeakCache::build`: **21,9 ms por frame** num clipe stereo de 3 min. Ou seja, o ganho de 62× do
[ADR-0120](../architecture/decisions/0120-audio-preview-is-a-buffer-you-own-not-a-buffer-you-rebuild.md)
(0,27 ms) **nunca chegou ao produto** — a medição do próprio ADR (`measure_preview.rs`) escreve a
região direto e nunca chama o `step`. É o MESMO bug do [ADR-0124](../architecture/decisions/0124-audio-a-range-edit-must-be-told-its-range.md),
no caminho do preview em vez do commit; achado escrevendo aquele.

**Por que não fechou junto:** o `patch` já existe (é O(seleção)), mas o scratch precisaria guardar um
`EditClip` por slot e pedir a ele "reescreva esta região **sem passo de undo**" — API nova numa
superfície que não é a desta linha, e a dança de posse do ADR-0120 (2 slots alternando com o mixer)
tem gates próprios e sutis. Enxertar meio-testado numa linha fechando é pior que nomear.

**O que acorda:** arrastar um knob num clipe longo e sentir. O fix é `[Option<EditClip>; 2]` no
scratch + `EditClip::rewrite_preview_region(r, region)`, com um gate que conte o patch como o
ADR-0120 conta o disparo.

### 2.3 Split do `fx_presets.rs` (631 LOC) — **dono: os presets, não o W7**

Herdado do rename do Gate (`a5ec9d7a`), greened com o marker sancionado `ph2d-loc-cap` para a linha
fechar. O conserto de verdade é um split por dados (a tabela de presets sai para um módulo irmão).

### 2.4 Smoke de stereo do AI Denoise

O wrapper processa por contagem de canais e o fixture é **mono** — o caminho stereo compila e nunca
foi exercido. Um clipe stereo com ruído fecharia o buraco.

### 2.5 Backlog pequeno (do `02_plano` §4)

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
