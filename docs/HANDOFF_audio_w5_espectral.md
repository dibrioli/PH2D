# HANDOFF — Áudio W5 (Espectral) · linha `line/audio`

> **Status:** W5 FECHADO. 3 commits novos (`84b1ac6a`, `75331763`, `e45b5f8d`) sobre os 16 de W4/W6.
> **Linha:** `line/audio` · **HEAD:** `e45b5f8d` · **base:** `3805f650` (main) · **19 commits** ao todo.
> **NÃO integrado, NÃO pushado.** Aguarda ordem explícita do Enio (DIRETRIZ §1.5.3–1.5.4).

---

## 1. O que entrou

**[ADR-0115](architecture/decisions/0115-audio-spectral-fft-via-realfft.md) foi ACEITO pelo Enio
(2026-07-12)** e o W5 abriu sobre o conjunto de aceitação **congelado** do §4. Os quatro itens
fecharam, mais o de-clip que o plano listava.

| item | onde | gate que prova |
|---|---|---|
| **Spectrogram** | `ph2d-audio-spectral::Spectrogram` + overlay | senoide 0 dBFS lê 0 dB no *seu* bin; cache limitado; bipe estreito sobrevive ao downsample |
| **Spectral repair** | `ph2d-audio-spectral::repair` | bipe sobre fala: **−22 dB** no bipe, **< 1 dB** na fala fora da região (o gate do ADR §4.2) |
| **Spectral denoise** | `ph2d-audio-spectral::denoise` | **+8 dB de SNR** a 0 dB de entrada, **sem musical noise** (o gate do ADR §4.3) |
| **A rack não regride** | — | os 5 gates da rack seguem verdes; nenhum efeito do W4 mudou |
| **De-Clip** (extra do plano) | `ph2d-audio-edit::fx::declip` | pico ceifado volta pra cima; **master quente e limpo fica intacto** |

## 2. Dep nova (a única)

`realfft = "3.5"` → **8 crates** no lockfile (`realfft`, `rustfmt`… ver ADR §3), todas MIT/Apache,
**zero C**, **zero `*-sys`**, RUSTSEC limpa (`transpose` resolve em 0.2.3, que é onde a
RUSTSEC-2023-0080 foi corrigida). **Nenhuma ferramenta de sistema nova no CI** — ao contrário do
`libavif-sys`, esta não compila nada.

**Confinada na crate nova `ph2d-audio-spectral`.** Não alcança `ph2d-audio` (o mixer RT) nem
`ph2d-audio-edit`: o shell depende das duas e faz a ponte. É o mesmo confinamento que já mantém
Symphonia em `-decode` e libvorbis em `-encode`.

## 3. Crates / arquivos novos

- **`crates/ph2d-audio-spectral/`** (nova, 5 arquivos): `stft` · `spectrogram` · `repair` · `denoise`.
- `crates/ph2d-audio-edit/src/fx/declip.rs` (novo) · `fx/warmup.rs` (**split** — `fx.rs` estava em
  666/700 e a variant nova não cabia) · `fx/lpc.rs` (ganhou `lsar_interpolate`, movido de `declick.rs`).
- `crates/ph2d-panel-audio-editor/src/`: `paint_spectral.rs` + `spectral_state.rs` (novos).
- `shells/desktop/src/audio/editor/spectral.rs` (novo) · `audio/wave_view.rs` (**split** — `audio.rs`
  passou 600) · `render_loop/audio_spectrogram.rs` (novo).

## 4. Ids / consts novos (para o integrador conferir colisão)

| id | valor |
|---|---|
| `AEDIT_SEC_SPECTRAL` | `hash_node_id("audio_editor_sec_spectral")` |
| `AEDIT_SPEC_VIEW` | `hash_node_id("audio_editor_spec_view")` |
| `AEDIT_SPEC_REPAIR` | `hash_node_id("audio_editor_spec_repair")` |
| `AEDIT_SPEC_LEARN` | `hash_node_id("audio_editor_spec_learn")` |
| `AEDIT_SPEC_AMOUNT` | `hash_node_id("audio_editor_spec_amount")` |
| `AEDIT_SPEC_DENOISE` | `hash_node_id("audio_editor_spec_denoise")` |

`AudioEditCmd` ganhou 3 variants **apendados** (`SpectralRepair`, `LearnNoise`, `Denoise`).
`Effect` ganhou `DeClip` **apendado por último** (índices postcard estáveis — saves antigos seguem
legíveis). `KINDS` da rack: **37 → 38**.

## 5. Foundational tocado

**Nenhum novo.** (O W4/W6 já tinha tocado `ph2d-ui-testkit::store()` e a BASELINE do HR-15 — ver
[HANDOFF_audio_w4_integracao.md](HANDOFF_audio_w4_integracao.md) §Foundational.) O W5 mexeu só em
`crates/ph2d-panel-audio-editor` (aditivo: 1 entrada no array pinado `SECTIONS`, 7 → 8) e
`crates/ph2d-editor-core/tests/` **não foi tocado**.

## 6. Contratos congelados encostados

**Nenhum.** `Tool`/`NodeOp`/`VectorOp` intactos. `Effect` **não é contrato congelado** (não há gate
de superfície na `ph2d-audio-edit`) — a variant nova é aditiva e no fim do enum.

## 7. O que só o `ship.sh` pega

Rodei por-crate: `cargo test` (spectral 18 · edit 152 · panel 25+37 · shell 7 binários) + `clippy
--all-targets` + `fmt` + os gates de LOC. **NÃO rodei** `machete`, `deny`, `audit` nem `typos` —
[a memória diz que o ship drena latentes em 2-4 iterações](../project-memory/project_integrator_ship_catches_latents_budget_iterations.md).
Pontos prováveis:

- **`cargo deny`**: a árvore do `realfft` é nova. Licenças verificadas à mão (todas permissivas) e a
  advisory-db local está limpa, mas o `deny.toml` do repo pode ter uma allowlist a atualizar.
- **`typos`**: os comentários novos têm termos de DSP (`Ephraim`, `Malah`, `Janssen`, `WOLA`, `COLA`,
  `LSAR`, `Nyquist`) e português nos commits.

## 8. O que smoke-testar (fixture pronto)

**O arquivo já está gerado em `~/ph2d_w5_smoke.wav`** (6 s, mono 48 kHz). Ele foi construído para
exercitar as quatro ferramentas de uma vez:

| trecho | o que tem | ferramenta |
|---|---|---|
| 0 – 1,5 s | só chiado (room tone) | **Learn Noise** aqui |
| 1,5 – 4 s | "fala" (harmônicos de 150 Hz) + chiado | **Denoise** |
| **2,2 – 2,8 s** | **um bipe de 5 kHz por cima da fala** | **Repair** — é a barra horizontal brilhante no spectrogram |
| 4,5 – 5,5 s | senoide estourada num teto de 0,92 (topos chatos) | **De-Clip** (na rack) |

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio/shells/desktop && cargo run --release
```

**Roteiro:**
1. Abrir o painel **Audio Editor** → **Load** → `~/ph2d_w5_smoke.wav`.
2. Seção **Spectral** → ligar **Spectrogram**. O bipe aparece como uma **linha horizontal brilhante**
   por volta de 2,2-2,8 s, na altura de 5 kHz. A fala é a faixa de harmônicos embaixo; o chiado é a
   neblina que cobre tudo.
3. **Repair:** arrastar uma **caixa** em volta do bipe (o drag agora tem eixo Y). O Repair acende.
   Clicar. O bipe some do desenho **e** do som; a fala embaixo continua.
4. **Denoise:** voltar pra waveform (ou não), selecionar o **primeiro 1,5 s** (só chiado) → **Learn
   Noise** → o Denoise acende → ajustar **Amount** → clicar. O chiado cai sem virar chuvisco.
5. **De-Clip:** selecionar 4,5-5,5 s, seção **Effects**, escolher **De-Clip**, subir o **Amount**.
   Os topos chatos voltam a ser picos (visível na waveform).

Cada op é **um passo de undo** (Ctrl+Z do painel volta).

## 9. Aberto (não é regressão — é escopo)

- **Opus** ([ADR-0113](architecture/decisions/0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md) §Opus) — decisão do Enio, ainda pendente.
- **W7 (AI/ML)** — DeepFilterNet atrás de feature-flag; o kill-criterion do ADR-0115 §4 não disparou
  (o denoise bateu o alvo), então o W7 segue opcional e não obrigatório.
- Backlog pequeno: toggle *enabled* por-entry na variação · manifesto com caminho relativo · reverb
  por convolução.
- **Débito de LOC:** `fx.rs` voltou a 601/700 e `fx/dynamics.rs` está em 662/700 — **o próximo efeito
  em `dynamics.rs` exige split antes**.
