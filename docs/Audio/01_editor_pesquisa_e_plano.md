# Editor de Áudio para Games — Pesquisa de estado-da-arte + plano faseado

> Pesquisa completa (edição · efeitos especiais · tratamento de voz · mixagem · asset-prep de
> games) cruzada com o que a linha `line/audio` já provê, para desenhar um editor **perfeito para
> games**, não só básico. Fontes: Audacity, Adobe Audition, REAPER, Sound Forge, WaveLab, iZotope
> RX 11/12, Waves Clarity Vx, Accentize, Supertone, ElevenLabs, DeepFilterNet/Demucs; middleware
> Wwise, FMOD, Unreal MetaSounds, Unity, Godot. (2024–2026.)

## §0 — Tese

Um editor **geral** trata o arquivo como *um som pronto para ouvir*. Um asset de **game** é
*matéria-prima para um runtime* que vai **loopar pra sempre, randomizar, espacializar, streamar e
re-encodar por plataforma**. O trabalho do editor é assar a metadata e as restrições que o runtime
consome. Logo o editor do PH2D precisa da base de um Audition/RX **mais** a camada game-específica
(loop sample-exato, variação, formato/codec, markers interativos, mono-3D, batch).

Chave arquitetural que destrava tudo: **edição é offline, na thread de controle** — HR-3
(no-alloc) e HR-5 (sem transcendentais) valem só na **thread de áudio**. Portanto todo o DSP de
edição (fade, normalize, filtro em seleção, pitch/formant, FFT/spectral) pode **alocar e usar
`sin/cos/exp/FFT` à vontade**. Só o *preview* toca pela `AudioEngine` (RT).

## §1 — O que a linha JÁ tem (reusar) vs. lacunas (construir)

### Reusar direto (inventário `line/audio`)
| Bloco | Onde | Uso no editor |
|---|---|---|
| `SampleData` (f32 interleaved, `Arc<[f32]>`, mono/estéreo) | `ph2d-audio/src/buffer.rs` | O clipe que o editor opera |
| `decode(&[u8]) -> SampleData` (WAV/AIFF/FLAC/Ogg/MP3, Symphonia) | `ph2d-audio-decode` | Importar |
| Kit DSP: `Biquad` (LP/HP/BP/peak/lo-hi-shelf), `SmoothGain`, `Adsr`, `Compressor`, `Delay`, `Reverb` (Freeverb), `LoudnessMeter` (BS.1770), `equal_power_pan` | `ph2d-audio/src/dsp/` | Efeitos + normalize LUFS (reusáveis offline num `Vec<f32>`) |
| `AudioEngine::play` + metering (`levels/rms/momentary_lufs`) | `ph2d-audio/src/engine.rs` | Preview |
| Padrão de painel docado (lib/populate/paint/event/snapshot + scroll + seções colapsáveis + widgets da gallery) | `ph2d-panel-audio-mixer` | Esqueleto do painel do editor |
| `LevelMeter` + `VectorScene::fill_path`/`push_clip` | editor-core / `ph2d-vector` | Waveform + medidores |

### Construir novo (lacunas confirmadas)
| Lacuna | Nota |
|---|---|
| **Render de waveform / peak-cache** | Não existe nenhum. Base: `SampleData::samples()` → min/max por coluna de pixel → `fill_path` |
| **Transporte com playhead** (seek/pause/posição) | `AudioEngine::play` é fire-and-forget, sem readback de posição. Extensão foundational (isolada: átomo de posição append-only + `play_preview`) |
| **Edição offline destrutiva** de `SampleData` | Buffer é imutável — trim/splice/fade/normalize/gain-em-seleção geram buffer novo. Crate nova `ph2d-audio-edit` |
| **Encode/export** | **NÃO existe** WAV/OGG writer (só um header PCM16 de fixture de teste). Crate nova `ph2d-audio-encode` — necessária já no W1/W2 pra salvar edições |
| **FFT** (spectral, denoise, pitch phase-vocoder) | Dep nova (`rustfft` / `realfft`) — só offline |

## §2 — Catálogo de features (taxonomia completa)

Tags: **[DSP]** = barato, Rust nativo offline · **[ML-leve]** = modelo pequeno nativo (RNNoise/DeepFilterNet) · **[ML-pesado]** = ONNX/serviço, batch/opt-in, nunca na thread de áudio.

### 2.1 Edição não-destrutiva / core  — **[DSP]**
Trim/crop · split/clip · cut/copy/paste/delete/silence (ripple) · fades linear/log/exp/**S-curve raised-cosine** · **crossfade equal-power (−3 dB)** · gain/amplify · **normalize peak** · **normalize LUFS/RMS** (usa `LoudnessMeter`) · **remoção de DC-offset** · reverse · invert (polaridade) · **markers/labels/cues** · **snap a zero-crossing** · zoom H+V / zoom-to-selection · seleção por range/canal/(spectral) · heal/consolidate/glue.

### 2.2 Tempo & pitch  — **[DSP]**
Time-stretch (WSOLA time-domain barato; phase-vocoder p/ transparência) · pitch-shift · **formant-preserving** (LPC/cepstral envelope) · vari-speed (resample) · pitch-envelope · detecção de tempo/beat (spectral-flux + autocorrelação) · detecção de transientes.
> Licença: Rubber Band = GPL/comercial; élastique = comercial. Implementar **PSOLA/phase-vocoder próprio** (clean-room) ou crate permissiva.

### 2.3 Efeitos — canal/insert/offline  — **[DSP]**
- **EQ/filtros:** paramétrico multibanda (biquad RBJ, já temos) · gráfico · FFT/linear-phase · LP/HP/BP/comb/allpass · **notch/de-hum** (comb 50/60 Hz + harmônicos).
- **Dinâmica:** compressor (temos) · **limiter true-peak** (oversample) · gate/expander · **multibanda** (crossover Linkwitz-Riley) · **de-esser** · transient shaper · maximizer LUFS.
- **Tempo/modulação:** reverb algorítmico (temos) + **convolução** (IR, FFT particionado) · delay (temos) · **chorus/flanger/phaser** (delays modulados/allpass) · tremolo/auto-pan · vibrato · ring-mod.
- **Distorção/cor:** saturação/tape/tube (waveshaper tanh, oversampled) · overdrive/fuzz · **bitcrush** · exciter · **stereo width M/S** (`M=L+R,S=L−R`) · **vocoder** (filterbank).

### 2.4 Espectral (frequência)  — **[DSP]** (precisa FFT)
Spectrogram (STFT, janela Hann/Blackman-Harris) · seleção tempo-freq · **spectral repair / inpaint** (heal brush — interpola bins vizinhos) · de-click/de-crackle (LPC + interpolação) · de-clip (reconstrói picos) · **de-noise broadband** (subtração espectral / Wiener por bin) · de-hum comb · analisador de espectro.

### 2.5 Geradores & análise/metering  — **[DSP]**
Tone (sine/square/saw/sweep) · noise (white/pink/brown) · silence · **metering LUFS BS.1770** (temos) + **true-peak** · analisador de espectro (RTA) · **correlação/goniômetro** (mono-compat) · VU/PPM · estatísticas (peak/DC/clip-count).

### 2.6 Tratamento de VOZ (jogo: diálogo, NPCs, criaturas)
Restauração — **[DSP]**: de-hum · **de-ess** · de-click/mouth-declick · de-crackle · de-plosive · **spectral-gate denoise** · de-reverb DSP (modesto) · hiss.
Diálogo — **[DSP]**: **leveler/AGC** · breath control · **EQ de inteligibilidade** (HPF+presença 2–5k+de-mud) · presença/air (hi-shelf) · **normalize a spec** (−16/−23 LUFS).
Design de personagem — **[DSP]**: pitch-correction (YIN/pYIN + PSOLA) · **formant shift** (gênero/idade/espécie) · vocoder/harmonizer/ring-mod/granular/robotização · **cadeia criatura** (pitch↓ + formant↓ + distorção multibanda + ring-mod + sub-harmônico) · **comms FX** (telefone 300–3400 Hz / rádio / capacete + distorção + squelch) · whisper/shout variants.
AI/ML: **DeepFilterNet** denoise (Rust, permissivo, RTF~0.19, realtime) **[ML-leve]** · **Demucs** stem/dialogue-isolate (ONNX, batch offline) **[ML-pesado]** · voice-conversion (RVC/Supertone) e **TTS/voice-clone** (ElevenLabs/Piper/XTTS) = integração externa/serviço **[ML-pesado]**.

### 2.7 Delta GAME-específico (o que editor geral NÃO tem)
1. **Loop points sample-exatos persistidos** (WAV `smpl`/`cue`) + **snap a zero-crossing por canal**.
2. **Loop crossfade** como metadata de emenda (conteúdo que não zero-cruza limpo).
3. **Intro→loop** (cabeça one-shot → região de loop repetida).
4. **Codec + residência por asset** (streaming vs in-memory) com **readout de tamanho/RAM**.
5. **Variantes por plataforma** de um master (Vorbis/Opus/ADPCM/PCM).
6. **Force-to-mono** p/ fontes 3D posicionais (+ warn em estéreo).
7. **Containers de variação** (random/round-robin/avoid-repeat + pesos).
8. **Ranges de pitch/volume randomizáveis** por asset (não detune assado).
9. **Normalize LUFS/RMS em batch** não-destrutivo p/ consistência da biblioteca.
10. **Markers interativos** (transition/destination, loop nomeado, grade de tempo/beat, stingers, sustain).
11. **Blend por parâmetro** (camadas cross-fade por RPM/velocidade/vida).
12. **Import batch por convenção de nome** (`footstep_dirt_01..12` → auto-grupo; `_3d/_loop/_stream` → preset).
+ Bônus espacial: validar **ambisonics B-format** (ordem ACN/SN3D) p/ ambiências rotacionáveis.

## §3 — Plano faseado (valor cedo, empilha)

Painel **docado** `ph2d-panel-audio-editor` (espelha o mixer; toggle na topbar — não é tool, áudio não é canvas). Cada W fecha gate + commit local; **integração/ship só por ordem do Enio**.

| Wave | Entrega | Crates novas / deps | Toca foundational? |
|---|---|---|---|
| **W1 Esqueleto** | Painel + Load + **waveform** (peak-cache) + **transporte** (play/stop/pause/seek/playhead) + zoom/scroll + **WAV export** mínimo | `ph2d-panel-audio-editor`, `ph2d-audio-edit`, `ph2d-audio-encode` | Sim: readback de posição na `AudioEngine` (isolado, append-only) |
| **W2 Edição core** | Seleção · trim/split/cut/paste/silence · fades+crossfade · gain/**normalize peak+LUFS** · DC-offset · reverse/invert · **snap zero-crossing** · **undo history** | — | Não |
| **W3 Rack de efeitos** | EQ paramétrico · comp/limiter true-peak/gate/de-esser · reverb(+convolução)/delay/chorus/flanger/phaser · saturação/bitcrush · width M/S · **FX chain + presets** | (reusa DSP) | Não |
| **W4 Voz** | de-hum/de-ess/de-click/de-plosive · leveler/AGC · EQ-voz · **comms FX** · **cadeia criatura** · **pitch/formant shift** (PSOLA) | — | Não |
| **W5 Espectral** | Spectrogram (STFT) · seleção T-F · **spectral repair** · spectral denoise | `realfft`/`rustfft` | Não |
| **W6 Asset-prep game** | **Loop points** (`smpl`) + crossfade + intro→loop · **variação** (random/RR/pesos + ranges pitch/vol) · markers · force-mono · **batch LUFS** · codec/residência + **OGG/Opus export** · import por convenção | `ph2d-audio-encode`(+Vorbis) | Não |
| **W7 AI/ML (opt-in)** | **DeepFilterNet** denoise nativo · **Demucs** stem/dialogue-isolate (ONNX, batch) · TTS/voice-clone via serviço | feature-gated (ONNX/deps pesadas) | Não |

Time-stretch transparente (phase-vocoder) entra no W4/W5 conforme a infra de FFT.

## §4 — Encaixe arquitetural (inegociáveis)

- **Drop-crates** (norte §0.1): painel + `ph2d-audio-edit` (ops offline) + `ph2d-audio-encode` (writers). Nada de plugin runtime.
- **HR-3/HR-5**: edição é **offline/control-thread** → livre de no-alloc e de transcendentais. Só o preview toca via RT (respeita HR-3/HR-5 já garantidos).
- **Contratos congelados §6**: painel não adiciona `Tool`/`Node` → sem `architecture_*_contract_surface`. `SCHEMA_VERSION` intacto no MVP (se persistir metadata de edição no save, é decisão à parte).
- **Foundational (W1)**: readback de posição do preview na `AudioEngine`. Projetar **isolado** (átomo de posição + `play_preview` append-only), anotar ids/consts novos no handoff (lição `NodeId(832)`).
- **UI**: labels/toasts **em inglês** (HR-15); zero hex/f32-literal → tokens.
- **Export encoder**: base no header PCM16 de `ph2d-audio-decode/src/lib.rs:128`; WAV primeiro, Vorbis/Opus no W6.

## §5 — Decisões em aberto (Enio)

1. **Ambição/ritmo:** construir rumo ao roadmap completo em fases (recomendado: começar W1 já) vs. um MVP menor primeiro.
2. **Fronteira ML:** nativo só até **[DSP]** + **[ML-leve]** (DeepFilterNet); **[ML-pesado]** (Demucs/TTS) fica opt-in/serviço externo — confirmar.
3. **Painel docado** (recomendado) vs. tool com pill.
