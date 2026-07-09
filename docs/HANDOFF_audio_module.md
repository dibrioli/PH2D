# HANDOFF — Módulo de Áudio (`line/audio`)

> Para o próximo agente. Contém: (1) proposta/plano do módulo, (2) arquitetura,
> (3) a linha `line/audio` e a integração ao `main` (Modo L), (4) o histórico do
> BUG "meter vivo, sem som" (**RESOLVIDO**). Leia o §0 + o §1 antes de mexer.

---

## 0. TL;DR do estado atual (2026-07-09)

**A linha está LIMPA, verde e sincronizada com o `main`** (integração concluída;
o integrador rebaseou os commits — SHAs novos — e ainda acrescentou `split
audio.rs` sob HR-18, fix de `fmt` drift `style_edition 2024` e AVIF no CI).
Nada pendente de integrar. Ponto de partida pronto para feature nova.

**O bug "meter vivo, sem som" está RESOLVIDO** e **não era código**: o
WirePlumber guardava `mute:true` por-app em `stream-properties`. Detalhe +
receita em [`docs/Audio/BUGS_audio.md`](Audio/BUGS_audio.md) (Bug #1). O §4
abaixo é histórico — não re-investigue.

**O que já landou:**
- **W1** — `ph2d-audio-encode` (WAV writer) + `ph2d-audio-edit` (`PeakCache`,
  `EditClip`) + painel docado `ph2d-panel-audio-editor` + **overlay flutuante do
  waveform** (drag/resize) + **preview transport** na `AudioEngine` (voz
  dedicada `PREVIEW_VOICE_ID`, fora do pool do jogo).
- **W2** — ops offline (gain · normalize peak/LUFS · reverse · DC · invert ·
  trim · cut · silence · fade · zero-crossing) + **undo/redo timeline** +
  **seleção por arrasto** na waveform + **edição ao vivo** (hot-swap do buffer no
  preview sem parar o play, com loop).
- **W3 Bloco 1** — **rack de efeitos offline** (`fx.rs`): `Effect::{LowPass,
  HighPass, Compress, Saturate, Bitcrush, StereoWidth}`, reusando o kit
  `ph2d_audio::dsp` (Biquad, Compressor).
- **W3 Bloco 2** — **efeitos com cauda** (`TailEffect::{Reverb, Delay}`, reusando
  Freeverb + Delay do kit). Splice novo `ops::in_range_tail`: dentro do alvo o
  efeito **substitui**; a cauda **soma** por cima do que vem depois (áudio
  original se a seleção é no meio) e o **clipe cresce** se o alvo toca o fim.
  ⚠️ `tail_secs` precisa vencer a latência do próprio efeito — o comb mais curto
  do Freeverb é ~25 ms, então cauda curta rende **silêncio** (preset usa 2,5 s;
  há um teste fixando esse modo de falha).
- **W3 Bloco 3a** — **rack paramétrico**: um seletor (`◀ nome ▶`) + até 4 sliders
  + **Apply**, no lugar dos 8 botões de preset fixo (os defaults *são* os presets).
  **Divisão de responsabilidade:** o painel continua **UI-only** — guarda só um
  índice de efeito e sliders **normalizados 0..1**; o shell
  (`shells/desktop/src/audio/fx_params.rs`) dona as faixas reais, o mapeamento
  (**log** p/ Hz e tempo), a formatação (`3.0 kHz`, `250 ms`) e a construção do
  `Effect`/`TailEffect`. Compressor ganhou **auto-makeup**. O `paint` re-semeia os
  sliders com o preset **só quando o efeito muda** (mesmo guard da caixa de nome),
  pra não brigar com o arraste do usuário.
- **W3 Bloco 3a-bis — audição ao vivo (não-destrutiva)**: mexer no seletor ou num
  slider marca o rack como *dirty*; o shell renderiza o efeito sobre o clipe
  **pristino** e faz hot-swap no preview voice → **você ouve enquanto ajusta**, e
  a waveform do overlay mostra o resultado (cauda inclusa). **Apply** commita
  **exatamente o buffer que soou** (1 passo de undo); **Cancel** descarta. O clipe
  só muda no Apply. Invariantes:
  - `EditClip::render_effect`/`render_tail_effect` (puros) + `commit_rendered`
    garantem *what-you-hear-is-what-you-commit* (teste
    `rendered_audition_is_what_gets_committed`).
  - `fx_dirty` é setado **só por input real** (`cycle_fx_kind`, `set_fx_norm`) —
    o re-seed dos defaults usa `seed_fx_norm`, senão abrir o painel já começaria a
    auditar um efeito que ninguém pediu (teste no seam).
  - O change-gate `FxSig` inclui **a seleção**: mover a seleção re-alveja o efeito
    e re-renderiza. Qualquer OUTRA edição (gain/trim/undo…) cancela a audição
    antes, senão o clipe se moveria debaixo dela.
  - **Custo:** um render completo do target range por *mudança de parâmetro*
    (change-gated ⇒ no máximo 1 por frame). Selecionar um trecho escopa o render —
    é o que mantém clipes longos responsivos. Se um dia pesar (reverb com cauda
    longa em música de 3 min), medir antes de mexer.
- **Atalhos:** `Ctrl+Z` undo · `Ctrl+Shift+Z` / `Ctrl+Y` redo (roteados ao
  `EditClip` quando o painel WAVE está aberto com clipe carregado).
- **Fix:** `cpal::Stream` é dropado no `on_close_request`, não no drop-cascade do
  `App` (segfault 139 de teardown).

**Invariante-chave do rack (duas famílias):** tudo age no **target range** (a
seleção, quando existe; senão o clipe inteiro), e a família decide o splice:
- **length-preserving** (`Effect`) → `apply_effect` → `ops::in_range`;
- **tail-extending** (`TailEffect`) → `apply_tail_effect` → `ops::in_range_tail`.

Efeito novo: escolha a família **antes** de escrever o DSP. Um efeito com cauda
enfiado no `in_range` tem a cauda **truncada** silenciosamente.

**Adicionar um efeito novo** (o caminho é curto agora): (1) a variante em
`fx.rs` (escolha a família!), (2) a tabela de params + o arm do `build` em
`audio/fx_params.rs`, (3) o nome em `FX_KINDS`. O painel **não muda** — ele lê
tudo do snapshot. `MAX_FX_PARAMS = 4`: um efeito com 5 params exige subir a
constante nos dois lados (painel + shell) e criar mais um id de slider.

**Próximo (plano vivo: [`docs/Audio/02_plano_implementacao_completo.md`](Audio/02_plano_implementacao_completo.md)):**
1. **W3 Bloco 3b** — **FX chain reordenável + presets** (salvar/carregar) +
   preview A/B, fechando o critério de aceitação do W3.
2. Resto do DSP do W3: limiter true-peak, gate/expander, de-esser, multibanda,
   convolução (FFT particionado), chorus/flanger/phaser, tremolo/auto-pan.
3. Depois: W4 (voz) ∥ W5 (espectral/FFT) → W6 (asset-prep) → W7 (ML).

**Protocolo (Modo L):** trabalhe e comite **nesta linha** (`git commit
--no-verify`), **sem push**. Você **não integra nem faz ship** — fecha, escreve o
handoff de integração (DIRETRIZ §1.5.9) e **para**. Integração/ship são decisão
exclusiva do Enio, via agente integrador dedicado.

---

## 1. Proposta & escopo do módulo

Objetivo: um **subsistema de áudio em tempo real** para o engine PH2D
(`ph2d-audio`), platform-agnostic (HR-1), com uma **UI de mixer** estilo mesa de
som docada no Inspector.

Escopo escolhido: **subsistema** (`ph2d-audio`), não um domínio de nós. O domínio
de nós `Sound` (autoria via `ph2d-eval-sound`) é fase futura e **exige ADR**
(`Domain::Sound` é contrato congelado — Coord/Enio-only, DIRETRIZ §4). A API
Luau/MCP (jogo dispara sons) está **bloqueada**: o loop de script gameplay na
shell é placeholder (`shells/desktop/.../integration.rs`), não há tick por-frame
pra dirigir uma fila de comandos de áudio.

### O que JÁ está construído (fundamentos + features)

**Fundamentos (já integrados ao `main` numa jornada anterior):**
- Duas metades pela fronteira de thread: `AudioEngine` (control, aloca, devolve
  handles) ↔ `AudioRenderer` (audio thread, `Send`, no-alloc/no-free HR-3) via
  **2 ring buffers** lock-free bounded (`crossbeam-queue::ArrayQueue`).
- `VoiceId` opaco (HR-8), pool de 64 vozes com **voice stealing** (oldest-quietest).
- Playback: pitch/resample (interp linear, cursor f64), pan (equal-power), gain
  (`SmoothGain` linear-ramp), ADSR. Decode via **symphonia** (`ph2d-audio-decode`).
- `cpal` confinado na shell (`shells/desktop/src/audio.rs`).
- Meter (`AudioMeter`), `MemoryBudget` (HR-13), gate no-alloc por capacidade.

**Features (os 10 commits desta linha — §3):** sub-buses (Master/Music/SFX/UI/
Voice), faders dB-tapered, pan por strip, filtro (Tone) low-pass por strip, meter
VU (RMS + peak-hold + clip latching), **limiter** (soft-clip + envelope de
redução de ganho), **reverb** (Freeverb no master), **ducking** (Music/SFX
abaixam sob o bus Voice), **Play Test** (oscilador embutido pra testar sem
arquivo).

---

## 2. Arquitetura (o que você PRECISA saber pro bug)

### Crates
- `crates/ph2d-audio` — o motor (platform-agnostic). Sem `cpal`.
- `crates/ph2d-audio-decode` — decode symphonia → `SampleData`.
- `crates/ph2d-panel-audio-mixer` — o painel do mixer (UI-only, sem dep de
  `ph2d-audio`; fala com a shell por thread-locals de snapshot).
- `shells/desktop/src/audio.rs` — `AudioSystem` (cpal + engine).
- `shells/desktop/src/render_loop/mod.rs` — a **ponte** por-frame painel↔engine.

### Fluxo de sinal (control → audio → device)

```
Painel (thread-locals snapshot)  ──ponte por-frame──►  AudioSystem (shell)
   fader/pan/tone/mute/solo/                              set_master_*/set_bus_*
   limiter/reverb/ducking                                 (change-gated) → comandos
                                                                 │ command ring
                                                                 ▼
   AudioRenderer::render(out, frames)   ← cpal callback (audio thread) chama isto
     1. drena comandos → mixer.apply
     2. scratch.reset (master + bus_scratch)
     3. mixer.render(master, bus_scratch, &mut bus_peaks, &mut bus_rms, ...)
          · por sub-bus: render voices → filtro → fader → (peak/rms do BUS) → fold em master c/ pan
          · voices master-direct somam em master
          · LOOP MASTER: gain → filtro → reverb(se on) → pan → limiter(se on)  [in-place em master]
     4. peak/rms do MASTER a partir de `master` (pós-tudo) → meter.store
     5. write_out(out, master, ...)   ← copia master → out (clamp ±1)
                                                                 │
   cpal callback: scatter `out`(nossa fmt) → `data`(device, T::from_sample) ──► DEVICE
```

### ⚠️ Fato-chave pro diagnóstico (LEIA)

- O **meter do MASTER** (RMS/peak da UI, via `engine.levels()/rms()`) é calculado
  no **passo 4**, do buffer `master` **DEPOIS** de todo o processamento master
  (gain/filtro/reverb/pan/limiter). É **o MESMO buffer** que o `write_out` (passo
  5) manda pro device. **Logo: se o meter do MASTER se mexe, o `master` tem sinal
  e o device recebe sinal.**
- Os **meters dos SUB-BUSES** (Music/SFX/UI/Voice) leem `bus_peaks/bus_rms`,
  calculados **no fold, ANTES** do processamento master. **Podem se mexer mesmo
  com o master zerado.**

→ **Isso é o bisect central do bug (§4).**

Arquivos exatos:
- Loop master + meter master + `write_out`: `crates/ph2d-audio/src/engine.rs`
  (`AudioRenderer::render` ~linha 210-275; `write_out` ~linha 310).
- Loop dos sub-buses + processamento master in-place:
  `crates/ph2d-audio/src/mixer.rs` (`Mixer::render`).
- Scatter → device: `shells/desktop/src/audio.rs` (`build_stream`, o closure do
  callback com `T::from_sample`).

---

## 3. A linha `line/audio` e a integração futura ao `main`

**Modo L** (workstation, ADR-0106/0107): a linha tem **worktree + índice
próprios** em `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio`, branch
`line/audio`. Sem colisão de git; valem só conflitos de merge.

- **Comitar:** `git commit --no-verify -m "..."` (fast mode). **Sem `git push`.
  Sem integrar.** A integração ao `main` é **1× no fim**, quando o Enio mandar.
- **Fim de mensagem de commit:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>`. (Msgs com `()`/backticks quebram no fish — use `git
  commit -F <arquivo>`.)
- **10 commits locais não-integrados** (`git log --oneline main..line/audio`):
  `9eae9fb6` dB-taper+solo+UI/Voice sub-buses · `a14e579b` VU meter · `1b881e61`
  Play Test · `ff245ee9` clip indicator · `3cac1cc6` limiter soft-clip ·
  `62df1af7` limiter audível (envelope) · `41d1c4e3` Tone por canal · `060f4b84`
  reverb Freeverb · `6d3011fa` ducking · `b5328b40` fix Play Test clipado.
  (Uma leva ANTERIOR de features de áudio JÁ foi integrada ao `main` numa jornada
  passada — o integrador achou até um 4º spot de `libasound2-dev` no CI. Estas 10
  são as novas desde então.)
- **Integração (quando o Enio pedir):** Modo L integra ao `main` por
  `scripts/foundational-integrate.sh` (gate da árvore combinada) + `--ff-only`;
  Mergiraf funde resíduo textual. Pontos de atenção do handoff anterior que
  seguem valendo (arquivos foundational que outras linhas também tocam):
  - `crates/ph2d-panel-registry-init/` (registro de painéis — re-rodar
    **panel-sync** na árvore combinada; blocos gerados não fundem textualmente).
  - `crates/ph2d-editor-core/src/widget/` + `showcase/` (widget `LevelMeter` +
    re-rodar **widget-sync**).
  - `.github/workflows/spike.yml` (`libasound2-dev` p/ ALSA — precisa sobreviver).
  - `ids/` (inspector/topbar), `screens/hero/` (paint z-order, fixture),
    `topbar/mod.rs` (extração `chip_name.rs` p/ ficar sob o LOC cap).
  - `deny.toml` já permite MPL-2.0 (symphonia). Workspace é glob (`crates/*`).
- **Contrato congelado NÃO tocado** (nada de ADR necessário pela linha): áudio não
  adiciona Tool/Node gateado. `SCHEMA_VERSION` intocado (não mexe em save).

---

## 4. O BUG — "grafos vivos, sem som audível" (RESOLVIDO 2026-07-08)

### ✅ Causa-raiz + fix (2026-07-08)
Confirmado o **Diagnóstico #1**: o meter do **Master mexe** → o sinal chega ao
`write_out` → o problema é **depois** dele (device/sistema). Inspeção do PipeWire
(`pactl`) revelou a máquina do Enio com saída ativa **7.1 de 8 canais**
(`alsa_output...HiFi_7_1__Speaker__sink`, s32le 8ch). O app abria o device no
**config nativo (8 canais)** e o scatter (`build_stream`) escrevia a mix estéreo só
em **FL/FR (canais 0,1)**, silêncio nos outros 6 — **bypassando o roteamento/upmix
estéreo→surround do PipeWire**. Todo app audível abre um stream **estéreo** (2ch) e
deixa o PipeWire mapear; o nosso não.
**Fix (`audio.rs::AudioSystem::new`):** pedir um stream **estéreo** quando o device
tem >2 canais **e** oferece uma config de 2ch (`supported_output_configs`), com
fallback pro nativo. Agora o PipeWire trata como qualquer app estéreo.
**Fator secundário (verificar se persistir):** há uma entrada **`module-stream-restore`
salva pro `ph2d-host-desktop`** — o PipeWire lembra rota/volume/**mute** do app entre
sessões (independente do painel). Se ainda mudo após o fix, no **pavucontrol** →
Playback → `ph2d-host-desktop` → **desmutar/rotear** (fica lembrado), ou limpar a
entrada de stream-restore.

### Sintoma (relato do Enio)
Toca o Play Test → **os meters/grafos do mixer se mexem** (algo está acontecendo)
mas **não há som audível** no device. (Antes disso houve um sintoma DIFERENTE já
resolvido: o botão Play Test tinha sido **empurrado pra fora da área visível** do
Inspector pelo crescimento do painel — corrigido em `b5328b40` movendo Play pro
topo + encolhendo os strips. Agora o Play é clicável, os meters mexem, mas sem som.)

### Timeline (pista forte de bisect)
- No commit do **reverb** (`060f4b84`) o Enio confirmou **"smoke ok"** → **tinha
  som**.
- Depois vieram só: **ducking** (`6d3011fa`, shell+painel) e **fix do Play
  clipado** (`b5328b40`, só paint do painel).
- **Ducking DESLIGADO (default) é comprovadamente no-op**: `update_ducking(false,
  _)` retorna `1.0`; a ponte faz `g = sub_gain[i] * 1.0`. O `fix do Play` só mexe
  em layout de paint. → **Nenhum dos dois deveria quebrar o áudio no caso
  default.** Isso é suspeito: ou (a) tem efeito ligado, ou (b) algo mais sutil.

### 🔍 Diagnóstico #1 (FAÇA PRIMEIRO — bisecta em 1 observação)
**O meter do strip MASTER se mexe, ou SÓ os de Music/SFX?**
- Pelo §2 (fato-chave): o meter do **Master** lê `master` pós-tudo, o mesmo que
  vai pro device. Se o **Master mexe** → o device recebe sinal → o problema é
  **depois do write_out** (scatter/cpal/device/sistema) ou conversão de amostra.
- Se **só Music/SFX mexem** e o **Master fica parado** → bug de **estágio
  master** (algo entre o fold e o write_out zera/NaN-a o `master`).

### Suspeitos (com localização)
Todos os defaults DEVERIAM ser passthrough, então instrumente pra achar o valor
real. Ordem sugerida:

1. **Efeitos ligados pelo usuário.** Peça pra testar num **launch limpo, TODOS os
   efeitos OFF, só Play Test**. Se com tudo OFF **tem som** → o culpado é um
   efeito (reverb instável/limiter/ducking). Se **sem som mesmo tudo OFF** →
   estágio master default ou device.

2. **Default do Tone/Cutoff NÃO é bypass real (imperfeição confirmada por
   leitura).** `engine.lowpass_coeffs` (`engine.rs`): `if cutoff_hz >= sr*0.5*0.9
   { identity } else { lowpass(...) }`. A 48 kHz, `sr*0.5*0.9 = 21600`. O default
   "aberto" do Tone é **20000 Hz** (`sub_tone_target()`/`master_cutoff_target()`),
   e 20000 < 21600 → aplica um **low-pass real de 20 kHz** em TODO sub-bus **e no
   master**, por default. Isso **não silencia** (só corta HF acima de 20 kHz), mas
   é um bug de default a corrigir (o "aberto" deveria mapear pra identity). **Cheque
   se não há um caminho onde o cutoff efetivo cai muito** (ex.: algum lugar
   mandando um Hz baixo). Confirme os Hz reais enviados (instrumento #2).

3. **Estágio master (`mixer.rs`, loop master).** Reverb/limiter só quando ON. Pan
   default `balance_gains(0.0) = [1,1]`. Gain default 1.0. **Cheque NaN/Inf**: se o
   reverb (feedback) ou o limiter produzir NaN, propaga pro `master`;
   `NaN.clamp(-1,1)` em `write_out` pode virar NaN → device muda pra silêncio/
   estouro. (Reverb OFF por default, mas confirme.)

4. **Caminho device (inalterado, mas verifique).** `write_out` (`engine.rs`) +
   scatter `T::from_sample` (`audio.rs`). Se o meter do Master mexe mas `out[]`
   sai zero/NaN, o bug está aqui.

5. **Ponte da shell mandando valor ruim.** `render_loop/mod.rs`: a cada frame
   manda `set_master_gain/pan/cutoff/limiter/reverb` + `set_bus_gain/pan/cutoff`.
   Todos os defaults parecem passthrough — **instrumente os valores reais**.

### Instrumentação sugerida (temporária, remova depois)
1. Em `AudioRenderer::render` (`engine.rs`), a cada ~100 blocos:
   `eprintln!` do `peak_l` do master **e** de `out[0..4]` **depois** do
   `write_out`. Prova se (a) o master tem sinal e (b) o buffer do device recebe.
2. Na ponte (`render_loop/mod.rs`), ~1×/s: `eprintln!` dos valores mandados
   (`master_gain_target()`, `master_cutoff_target()`, `master_pan_target()`,
   `limiter()`, `reverb_on/size/mix`, `ducking()`, e o `duck` calculado, `sub_tone`).
3. **Bisect por env-smoke (ótimo!):** o path de env NÃO passa pelo Play Test:
   - `PH2D_AUDIO_SMOKE=1` → tom 440 Hz no **bus SFX**.
   - `PH2D_AUDIO_FILE=<wav>` → arquivo no **bus Music**.
   Se o env-smoke **tem som** mas o Play Test não → o problema está nos geradores
   do Play Test (`pluck_loop`/`swell_loop`/`blip_loop`) ou no roteamento. Se o
   env-smoke **também é mudo** → estágio master/device (independe do Play Test).
4. **`git bisect`** entre `060f4b84` (reverb, tinha som) e `HEAD` (mudo). Só 2
   commits no meio (ducking, fix-clip) — build + smoke manual em cada.

### Hipóteses que JÁ foram descartadas por leitura (não gaste tempo)
- Ducking OFF atenuar Music/SFX: **não** (`update_ducking(false,_) == 1.0`).
- Play-clip-fix mexer em áudio: **não** (só paint do painel).
- Change-gate floodar/dropar comando de Play: **não** (ring 1024, drenado por bloco).

---

## 5. Como rodar / gates / convenções

- **Rodar (SEMPRE com o `cd` junto — preferência do Enio):**
  ```
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && cargo run -p ph2d-host-desktop
  ```
  Abre a janela → TopBar **MIX** abre o mixer → **Play Test** (topo da seção master).
  Env-smokes: prefixe `PH2D_AUDIO_SMOKE=1` ou `PH2D_AUDIO_FILE=/som.wav`.
- **Inner loop:** `cargo check -p <crate>`. **Não** `--workspace`.
- **Gate no fechamento** (rode do worktree):
  - `cargo test -p ph2d-audio` (34 unit + `tests/bus_routing.rs` prova roteamento,
    RMS≤peak, limiter, reverb tail, filtro).
  - `cargo test -p ph2d-panel-audio-mixer` (14 seam tests, `tests/seam.rs`).
  - `cargo test -p ph2d-editor-core --test no_magic_numeric --test no_literal_color
    --test hr15_no_hardcoded_ui_strings --test architecture_panel_loc_cap
    --test architecture_workspace_file_loc_cap` (UI/LOC gates — o painel toca-os).
  - `cargo clippy -p ph2d-audio -p ph2d-panel-audio-mixer -p ph2d-host-desktop --all-targets`.
  - `cargo build -p ph2d-host-desktop` (link completo — cpal/ALSA precisa
    `libasound2-dev`).
- **UI canônica (HR-15):** zero hex, zero `f32` literal de UI, zero string
  hardcoded. Constantes de áudio (Hz, dB, ratios) marcam `// LITERAL-PX-OK: <razão>`.
  Widgets pela **Widget Gallery** (o painel usa `Slider`/`LevelMeter` canônicos,
  não chrome improvisado). Labels em inglês.
- **LOC cap:** fn ≤200, arquivo ≤700. Se estourar, **extraia módulo/fn-irmã** (já
  foi feito: `paint()` → `paint_master_section`/`paint_labeled_slider`;
  `topbar/mod.rs` → `chip_name.rs`).

---

## 6. Mapa de arquivos-chave

**Core (`crates/ph2d-audio/src/`):**
- `engine.rs` — `AudioEngine`/`AudioRenderer`; **`render` (meter + write_out)** ⭐.
- `mixer.rs` — `Mixer::render` (**loop sub-bus + loop master**) ⭐; `soft_clip`,
  `balance_gains`, `set_reverb/limiter/bus_filter`.
- `bus.rs` — `BusId` (Master/Music/SFX/UI/Voice), `SUB_BUS_COUNT=4`, `SUB_BUSES`.
- `meter.rs` — `AudioMeter` (peak+RMS master + por-bus).
- `dsp/` — `biquad.rs`, `gain.rs` (`SmoothGain`), `envelope.rs`, `pan.rs`,
  **`reverb.rs`** (Freeverb).
- `voice.rs`/`pool.rs`/`buffer.rs`/`command.rs`/`format.rs`.
- `tests/bus_routing.rs` — testes de integração de roteamento/efeitos.

Também no core, adicionados pelo editor (append-only, isolados):
- `command.rs` — 6 variantes `AudioCommand::{Play,Seek,SetData,SetLooping,Pause,Stop}Preview`.
- `engine.rs` — `play_preview`/`seek_preview`/`set_preview_data`/`set_preview_looping`/
  `pause_preview`/`stop_preview`/`preview_frame`/`preview_playing`; const
  **`PREVIEW_VOICE_ID = VoiceId(u64::MAX)`** (voz de preview FORA do pool do jogo).
- `voice.rs` — `replace_data` (**hot-swap** do buffer mantendo o cursor → edição ao vivo).
- `meter.rs` — atomics `preview_frame: AtomicU64` / `preview_active: AtomicU32`.
- `dsp/loudness.rs` — `integrated_lufs()` (BS.1770 offline, usado pelo normalize LUFS).

**Edição offline (`crates/ph2d-audio-edit/src/`)** — control-thread only, HR-3/HR-5
NÃO se aplicam (aloca e usa `tanh`/`exp` livremente):
- `lib.rs` — `EditClip` (clipe + peak cache + seleção + **timeline de undo**,
  snapshots `Arc` baratos, cap 64); `apply_*` + **`apply_effect`**; `target()` =
  seleção ou clipe inteiro.
- `ops.rs` — gain/normalize peak+LUFS/reverse/invert/DC/trim/silence/delete/fade/
  `snap_to_zero_crossing` + **`in_range`** ⭐ (aplica op length-preserving só na
  região e reencaixa; é o que torna tudo selection-aware).
- `fx.rs` — **`Effect`** ⭐ (LowPass/HighPass/Compress/Saturate/Bitcrush/StereoWidth).
- `peaks.rs` — `PeakCache`/`column_peaks` (envelope min/max p/ a waveform).

**Encode (`crates/ph2d-audio-encode/src/`):** `write_wav`/`encode_wav`, `BitDepth`.

**Painéis (`crates/ph2d-panel-audio-{mixer,editor}/src/`):** ambos UI-only.
- mixer: `lib.rs` (ids `AMIX_*`/`SUB_*` + `snapshot`), `paint.rs`, `event.rs`,
  `populate.rs`, `fader.rs`, `tests/seam.rs`.
- editor: `lib.rs` (ids `AEDIT_*` + **`AudioEditCmd`**, o enum one-shot painel→shell),
  `paint.rs` (transporte + seções Edit/Range/Effects), `event.rs`, `populate.rs`,
  `snapshot.rs` (thread-locals), `tests/seam.rs`.

**Shell (`shells/desktop/src/`)** — `audio.rs` foi **dividido** (HR-18 shell cap):
- `audio.rs` — `AudioSystem` (cpal, `build_stream`/scatter ⭐, `update_ducking`,
  `set_master_*`/`set_bus_*` change-gated). **Prefere stream 2ch em device 7.1.**
- `audio/editor.rs` — runtime do editor: `editor_load/toggle_play/apply/…`,
  `WaveView` (rect+frames publicados p/ o hit-test da seleção).
- `audio/signals.rs` — geradores `sine_tone`/`pluck`/`swell`/`blip_loop`.
- `render_loop/mod.rs` — **a ponte por-frame** ⭐ (drena intents dos painéis → engine).
- `render_loop/audio_overlay.rs` — overlay flutuante do waveform (ruler, playhead,
  banda de seleção, handles de drag/resize).
- `input_dispatch.rs` — **seleção por arrasto** na waveform (SHELL-only) + **teardown
  do `cpal::Stream` no `on_close_request`** ⭐ (segfault 139).
- `input_handlers.rs` — **Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y** → undo/redo do `EditClip`.
- `main.rs` — env-smokes `PH2D_AUDIO_SMOKE`/`PH2D_AUDIO_FILE`.

**⚠️ Gotcha de wiring:** nenhum seam test cobre **atalho de teclado** — eles vivem
só em `input_handlers.rs`. Botão verde no seam ≠ atalho ligado (foi exatamente o
"undo não funciona" de 2026-07-09: os botões funcionavam, o Ctrl+Z não existia).

**Processo:** `CLAUDE.md` (roteador), `docs/IntegracaoMultiAgente/DIRETRIZ.md` +
`DIRETIVA_IMPLEMENTACAO.md`, `project-memory/MEMORY.md`.
