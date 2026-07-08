# Editor de Áudio para Games — Plano completo de implementação

> Plano de execução do módulo **Audio Editor**. Pesquisa de estado-da-arte e catálogo de features:
> [`01_editor_pesquisa_e_plano.md`](01_editor_pesquisa_e_plano.md). Este doc é o **como**: crates,
> arquivos, tipos, tarefas, gates e critérios de aceitação por wave. Decisões travadas pelo Enio
> (2026-07-08): **(1)** roadmap completo · **(2)** UI em duas partes — painel docado **+** overlay
> flutuante e redimensionável no canvas (waveform + linha do tempo), entre Hierarchy e Inspector ·
> **(3)** fronteira ML: nativo até DeepFilterNet, resto opt-in/externo.

---

## §0 — Escopo & inegociáveis

- **Norte (ADR-0075):** drop-crates + desacoplar por ECS/eventos. Nada de plugin runtime.
- **HR-3/HR-5 só na thread de áudio.** Edição é **offline, na thread de controle** → fade, normalize,
  filtro-em-seleção, pitch/formant, FFT/spectral podem **alocar** e usar `sin/cos/exp/FFT` livremente.
  Só o **preview** passa pela `AudioEngine` (RT), que já respeita HR-3/HR-5.
- **Contratos congelados §6:** o editor **não adiciona `Tool`/`Node`** → nenhum `architecture_*_contract_surface`
  tocado. `SCHEMA_VERSION` intacto (persistir edit-metadata no save = decisão à parte, W6).
- **UI:** labels/toasts **em inglês** (HR-15); zero hex / zero f32-literal → tokens (`ColorToken`/`Spacing`/`Radius`/`TypeToken`).
- **Foundational:** só W1 toca (`AudioEngine` readback de posição) — projetado **isolado/append-only**;
  ids/consts novos anotados no handoff (lição `NodeId(832)`).
- **Fechamento:** cada wave fecha gate próprio + commit local. **Integração e ship só por ordem
  explícita do Enio**, via integrador dedicado; a linha fecha → escreve handoff (DIRETRIZ §1.5.9) → PARA.

---

## §1 — Crates novas + dependências

| Crate | Papel | Deps novas |
|---|---|---|
| **`ph2d-audio-edit`** | Ops **offline/destrutivas** sobre `SampleData` → buffer novo (trim/fade/normalize/gain/filtro/pitch/spectral). Sem RT, sem HR-3/HR-5. | (W5) `realfft` |
| **`ph2d-audio-encode`** | Writers de arquivo. WAV PCM16/24/f32 (+ `smpl`/`cue` chunks no W6); Vorbis/Opus no W6. | (W6) `vorbis-rs`/`opus` (a decidir; permissivo) |
| **`ph2d-panel-audio-editor`** | Painel **docado** (Inspector slot): transporte, seções de efeito, load/export, listas. Espelha `ph2d-panel-audio-mixer`. | — |
| **`ph2d-audio-overlay`** *(ou módulo em editor-core/shell)* | **Overlay flutuante no canvas**: waveform grande + ruler + playhead + seleção. Bridge de paint + frame móvel/redimensionável. | — |
| **`ph2d-audio-fx`** *(opcional, W3+)* | Efeitos criativos que não cabem no kit RT (convolução, chorus/flanger/phaser, vocoder, comms, cadeia criatura). Pode nascer dentro de `-edit` e extrair quando crescer. | — |
| **`ph2d-audio-ml`** *(W7, feature-gated)* | DeepFilterNet nativo; wrapper ONNX p/ Demucs (batch). **Opt-in**, nunca default. | `deep_filter` / `ort` (feature `audio-ml`) |

**Prefixo:** nenhuma na área de nós → sem colisão com `node-sync` (lição node-sync glob). Registrar no
workspace `Cargo.toml` + `ph2d-panel-registry-init` (o painel) na ordem canônica.

---

## §2 — Arquitetura da UI (duas partes)

### 2.1 Painel docado — `ph2d-panel-audio-editor`
Estrutura idêntica ao mixer (`lib.rs` ids · `populate.rs` · `paint.rs` · `event.rs` · `snapshot.rs` bridge
thread-local · `state.rs`). Ocupa o slot do Inspector (`ctx.layout.inspector`), scroll + seções colapsáveis
canônicas. Toggle por pill na topbar (`TOPBAR_AUDIO_EDITOR`, wiring em `screens/hero/topbar/`). Conteúdo:
transporte compacto, Load/Export, seções de efeito (EQ/Dynamics/Time/Voice/...), lista de markers/regiões,
lista de variação (W6). **Controles ficam no painel; a visualização grande vai pro overlay.**

### 2.2 Overlay flutuante no canvas — o preview espaçoso
**Requisito do Enio:** waveform + linha do tempo **sobre o canvas**, janela flutuante entre Hierarchy e
Inspector, **redimensionável**. Toda a máquina já existe (padrão do dock do Inspector, agnóstico a `NodeId`).

**Reuso direto (survey aterrou file:line):**
- **Regiões:** `HeroLayout.{hierarchy,inspector,canvas}` (`screens/layout.rs`). O **gap** = entre
  `hierarchy.x+hierarchy.w` e `inspector.x` (ler ambos; `mirrored` troca lados).
- **Frame móvel + 2 resize handles:** ids `INSP_DRAG_HANDLE/RESIZE_HANDLE/RESIZE_HANDLE_BL` são o padrão;
  geometria via `panel_chrome::{panel_drag_handle_rect, panel_resize_handle_rect, panel_resize_handle_rect_bl}`;
  Down arms em `interaction/dispatch/blender.rs` (`begin_blender_drag`/`begin_panel_resize{,_bl}`);
  apply em `pointer_move.rs:39-74`; clamp por `panel_chrome::clamp_panel_rect(base, off, resize, viewport)`
  — **passar o gap rect como `viewport`** clampa a janela no vão entre os docks (zero math nova).
- **Persistência:** `set_panel_rect`/`panel_rect`, `blender_picker_offset` (posição x,y), `panel_resize_delta`
  (w,h), `bump_panel_z` (click-to-front), e chave em `panel_visibility` (`default_panel_visibility()` +
  `canonical_panel_id()`, ex.: `"audio_overlay"`).
- **Paint:** bridge que desenha no `&mut VectorScene` (como `painter_bridge_overlays::draw_repeat_image`),
  chamado **depois de `paint_hero_screen`** (`render_loop/mod.rs:1582`) → z acima do canvas, sem cobrir docks
  (fica no gap). `vector_scene.inner_mut()` p/ `scene.stroke`/`fill_rect` + `paint_text`.

**Build novo:** o próprio **frame** sob um `NodeId` novo (`AUDIO_OVERLAY_PANEL`) com base rect no gap; e todo o
conteúdo — **waveform, ruler de tempo, playhead, seleção** (não existe nenhum ruler/scrubber no repo; só o
`grid_snap::render` como referência de tick+label; `motion_timeline_slot` é só um seam reservado).

**Receita do frame (clonar do Inspector, `hero/paint.rs:43-52,213`):**
```
let base = gap_rect(&layout);                                   // entre hierarchy e inspector
let off    = store.blender_picker_offset(AUDIO_OVERLAY_PANEL);
let resize = store.panel_resize_delta(AUDIO_OVERLAY_PANEL);
let (rect, off, resize) = clamp_panel_rect(base, off, resize, gap_rect_as_viewport);
store.set_panel_rect(AUDIO_OVERLAY_PANEL, rect);               // routing de clique
// registrar 3 handles (drag + resize BR/BL) no hit_index; pintar frame+waveform+ruler no vector_scene
```

**Sincronia painel↔overlay:** ambos na main thread → estado compartilhado via um `snapshot` thread-local
(mesma ponte do mixer): seleção (in/out em frames), zoom/scroll-h, playhead (frames), clipe atual.

---

## §3 — Transporte & preview (extensão foundational isolada — W1)

`AudioEngine::play` é fire-and-forget, **sem readback de posição** → transporte com playhead precisa disso.

**Design isolado/append-only na `AudioEngine`:**
- Novo conceito **preview voice**: `play_preview(SampleData, PlayParams) -> PreviewHandle`.
- **Posição observável:** átomo `AtomicU64` (frame atual) publicado pelo renderer a cada bloco — igual ao
  `AudioMeter` já faz com peak/rms (`meter.rs`). Leitura control-side: `preview_position() -> u64`.
- **Controles:** `seek_preview(frame)`, `pause_preview(bool)`, `stop_preview()`.
- **Isolamento:** átomo append-only no meter/engine, um voice dedicado (não mexe no `VoicePool` de jogo),
  método novo — **não altera assinaturas existentes**. Anotar `PreviewHandle`/átomo no handoff.

Loop de scrub: overlay lê `preview_position()` por frame → desenha playhead; arrastar playhead → `seek_preview`.

---

## §4 — Waveform + peak-cache (W1)

- **Peak-cache:** min/max por coluna de pixel a partir de `SampleData::samples()`. Multi-resolução (mip de
  peaks) p/ zoom rápido em clipes longos; recomputa só no load / na edição (buffer novo).
- **Render:** polígono de envelope via `VectorScene::fill_path` (BezPath move/line) ou `fill_rect` por coluna;
  estéreo = 2 faixas empilhadas. Cores por `ColorToken` (waveform = Accent/Text2; fundo = surface).
- **Ruler:** ticks + labels de tempo (s/ms ou beats no W6); espaçamento adaptativo ao zoom (ref
  `grid_snap::render`). Playhead = linha vertical (`scene.stroke`).
- **Interação:** seleção por drag (range in/out em frames), zoom (wheel), scroll-h. Seleção 2D-livre segue o
  padrão de dispatch em editor-core (não per-Move no painel) — `InteractiveState` dedicado do overlay.

---

## §5 — Modelo de edição offline (W2)

`SampleData` é imutável (`Arc<[f32]>`). Toda edição **produz um buffer novo** em `ph2d-audio-edit`:
```
pub struct EditClip { data: SampleData, sel: Option<Range<u64>>, /* frames */ }
// ops: trim, split, delete(ripple), silence, paste, gain, normalize_peak, normalize_lufs,
//      fade(kind,range), crossfade, dc_offset_remove, reverse, invert, snap_zero_crossing
```
- **Normalize LUFS:** reusar `LoudnessMeter` offline (rodar sobre o buffer, medir integrated, aplicar ganho).
- **Undo:** timeline de snapshots do `SampleData` (barato: `Arc` clone), padrão do Painter
  (`ModelSnapshot`) — criar/editar/efeito/normalize = uma timeline unificada.
- **Zero-crossing:** busca do zero mais próximo por canal (edições e loop points).

---

## §6 — Encode/export (W1 mínimo, W6 completo)

- **W1:** `ph2d-audio-encode::write_wav(path, &SampleData, bit_depth)` — PCM16 primeiro (base no header de
  fixture `ph2d-audio-decode/src/lib.rs:128`), depois 24-bit/f32. Necessário p/ salvar qualquer edição.
- **W6:** chunks `smpl` (loop points) + `cue`/`LIST adtl` (markers) no WAV; Vorbis/Opus; escolha de
  codec/residência por asset + readout de tamanho/RAM; variantes por plataforma; export batch.

---

## §7 — Waves (tarefas · aceitação · gate)

Gate por wave = `cargo check -p` no inner loop; no fechamento: `nextest-impacted` + clippy `--all-targets`
+ arch-gates das crates tocadas + **smoke** 1×. Ship/integração: só por ordem do Enio.

### W1 — Esqueleto (painel + overlay + transporte + waveform + WAV export)
**Tarefas:** crates `ph2d-panel-audio-editor`, `ph2d-audio-edit`, `ph2d-audio-encode`, `ph2d-audio-overlay` ·
pill topbar + registry-init + `panel_visibility` key · painel docado (transporte + Load + Export) · extensão
`AudioEngine` (preview + posição, §3) · overlay flutuante no gap (frame drag/resize clonado do Inspector, §2.2)
· peak-cache + render de waveform + ruler + playhead + seleção · `write_wav` PCM16.
**Aceitação:** carregar um WAV → waveform aparece no overlay no gap; arrastar/redimensionar a janela (clampada
ao gap); play/pause/seek com playhead correndo; exportar cópia WAV que re-decoda idêntica (round-trip).
**Foundational:** `AudioEngine` preview/posição — handoff anota `PreviewHandle` + átomo.

### W2 — Edição core (offline + undo)
**Tarefas:** seleção range · trim/crop/split/cut/copy/paste/delete(ripple)/silence · fades (linear/log/exp/
S-curve) + crossfade equal-power · gain · normalize peak · **normalize LUFS** · DC-offset · reverse · invert ·
**snap zero-crossing** · **undo history** (snapshots) · atalhos.
**Aceitação:** cada op gera buffer novo audível no preview; normalize LUFS bate alvo (−16/−23) medido;
undo/redo consistente na timeline unificada; edições sem clique (zero-crossing/fade nas emendas).
**Gate:** testes offline determinísticos por op (in→out sample-exato).

### W3 — Rack de efeitos (offline, reusa kit DSP)
**Tarefas:** EQ paramétrico (biquad) · compressor/**limiter true-peak**/gate/expander/**de-esser**/multibanda ·
reverb algorítmico (temos) + **convolução** (FFT particionado) · delay (temos) · **chorus/flanger/phaser** ·
tremolo/auto-pan · saturação/tape/**bitcrush** · **stereo width M/S** · **FX chain + presets** (grafo serializável).
**Aceitação:** cada efeito aplica offline com preview A/B; chain reordenável; presets salvam/carregam;
limiter true-peak sem overs inter-sample (oversample verificado).
**Gate:** testes de reconciliação (in conhecido → métrica esperada).

### W4 — Tratamento de voz
**Tarefas:** de-hum (comb notch 50/60 Hz+harm) · de-ess · de-click/mouth-declick/de-crackle (LPC+interp) ·
de-plosive · **leveler/AGC** · EQ-voz (HPF+presença+de-mud+air) · **comms FX** (rádio/telefone/capacete +
squelch) · **cadeia criatura** (pitch↓+formant↓+distorção+ring-mod+sub) · **pitch/formant shift** (PSOLA/
phase-vocoder próprio, clean-room — Rubber Band é GPL) · vocoder/harmonizer/robotização/granular · whisper/shout.
**Aceitação:** presets de voz aplicam e soam corretos no preview; pitch/formant shift preserva duração;
comms/criatura como presets reutilizáveis; batch de N linhas com loudness consistente.
**Gate:** testes de f0/formant em sinais sintéticos; sem transcendentais no RT (edição é offline, ok).

### W5 — Espectral (FFT)
**Tarefas:** dep `realfft`/`rustfft` · spectrogram (STFT, janela Hann/Blackman-Harris) no overlay · seleção
tempo-freq · **spectral repair/inpaint** (heal brush — interpola bins vizinhos) · **spectral denoise**
(subtração espectral / Wiener por bin, aprende profile) · de-clip.
**Aceitação:** spectrogram alterna com waveform no overlay; pintar/apagar região T-F remove artefato
pontual (cough/beep) mantendo o resto; denoise reduz ruído medido sem "musical noise" audível.
**Gate:** métricas de SNR antes/depois em fixtures.

### W6 — Asset-prep de games
**Tarefas:** **loop points** (`smpl` chunk) + snap zero-crossing por canal + **loop crossfade** + **intro→loop** +
**audição de loop** ao vivo · **containers de variação** (random/round-robin/avoid-repeat + pesos + ranges
pitch/vol) · **markers/cue** (transition/destination/loop nomeado/tempo-beat/stingers/sustain) · **force-to-mono**
p/ 3D (+warn estéreo) · **batch LUFS** não-destrutivo · **codec/residência por asset** + readout tamanho/RAM ·
export **OGG/Opus** + variantes por plataforma · **import por convenção** (`_loop/_3d/_stream`, `nome_01..NN`→grupo).
**Aceitação:** loop sample-exato sem clique em audição contínua; smpl/cue sobrevivem re-decode; set de variação
toca com no-repeat + pesos; batch normaliza biblioteca a alvo; export por plataforma com preview de tamanho.
**Decisão SCHEMA:** se persistir edit-metadata no save do projeto → ADR + bump `SCHEMA_VERSION`.

### W7 — AI/ML (opt-in, feature `audio-ml`)
**Tarefas:** **DeepFilterNet** denoise nativo (crate `deep_filter`, Rust, realtime) como efeito de voz opcional ·
**Demucs** stem/dialogue-isolate via `ort`(ONNX) **batch offline** (worker, nunca thread de áudio) ·
TTS/voice-clone via **serviço externo** (integração, não nativo).
**Aceitação:** denoise ML alterna com o DSP; stem-split roda offline num worker sem travar UI; tudo atrás de
feature-flag (build default não puxa deps pesadas).
**Gate:** feature desligada = build limpo; ligada = smoke do denoise.

---

## §8 — Sequência de execução & riscos

**Ordem:** W1 → W2 → W3 → (W4 ∥ W5, compartilham infra FFT) → W6 → W7. W1 é o alicerce (arquitetura da UI +
transporte + export). W2 destrava valor real (edição salvável).

**Riscos & mitigação:**
- **Licença de time-stretch/pitch:** Rubber Band (GPL) / élastique (comercial) → **implementar PSOLA/
  phase-vocoder próprio** (clean-room), sem vendorizar. (Lição rebecca: espec comportamental → impl fresco.)
- **Foundational (W1):** único ponto de risco de colisão — projetar isolado/append-only + anotar no handoff.
- **Codec de export (W6):** escolher crate Vorbis/Opus permissiva e sem `*-sys` pesado no CI se possível
  (lição imageio AVIF: `*-sys` exige toolchain no CI). Avaliar `ogg`/`lewton`(decode)/encoder puro-Rust.
- **Perf de waveform em clipes longos:** mip de peaks + recompute só na edição; medir antes de otimizar.
- **ML (W7):** deps pesadas atrás de feature-flag; Demucs só batch/worker.

**Fechamento de cada wave:** gate + commit local + atualização do handoff da linha. **A linha não integra nem
faz ship** — entrega o handoff e para; integração/ship é decisão exclusiva do Enio via integrador dedicado.
