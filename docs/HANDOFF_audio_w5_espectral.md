# HANDOFF — Áudio W5 (Espectral) · linha `line/audio`

> **Status:** W5 FECHADO + **AUDITADO** (4 lentes paralelas — §8b). Smoke do Enio: OK.
> **Linha:** `line/audio` · **base:** `3805f650` (main) · **25 commits** ao todo (16 de W4/W6 + 9 do W5).
> **NÃO integrado, NÃO pushado.** Aguarda ordem explícita do Enio (DIRETRIZ §1.5.3–1.5.4).

---

## 1. O que entrou

**[ADR-0122](architecture/decisions/0122-audio-spectral-fft-via-realfft.md) foi ACEITO pelo Enio
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

## 7. O que só o `ship.sh` pega — **rodado, e verde**

Rodei tudo, inclusive o que o handoff original deixava para o integrador:

| gate | resultado |
|---|---|
| `cargo test` (spectral **24** · edit **154** · panel 25+37 · shell 7 binários) | verde |
| `clippy --all-targets` (4 crates + shell) | 0 warnings |
| `cargo deny check` (a árvore nova do `realfft`) | **advisories ok · bans ok · licenses ok · sources ok** |
| `typos` | limpo |
| `cargo machete` | nenhuma dep não-usada |
| `cargo fmt --all --check` (toolchain pinado 1.95) | limpo |
| gates de LOC (HR-18) | verde |

Ou seja: os dois pontos que eu previa que o ship pegaria (`deny` pela árvore nova, `typos` pelos
termos de DSP) **não pegaram nada**. O integrador deve orçar menos iterações do que o normal.

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

## 8b. AUDITORIA (2026-07-12, 4 lentes paralelas) — o que ela achou e o que foi corrigido

O smoke passou; a auditoria **não**. Quatro lentes independentes (DSP/numérica · costura ·
performance · qualidade dos gates) sobre o diff do W5. Achados **verificados executando o
código**, não por leitura. Tudo abaixo está **corrigido e gateado** (commits `0afd1c9b`,
`06f6a5ad`, `8c71dbb4`).

### Os dois críticos (entregáveis ao usuário no primeiro gesto natural)

1. **`repair` APAGAVA o áudio** numa banda encostada em **DC** ou **Nyquist** — e o fundo do
   spectrogram é onde mora o rumble/hum, ou seja, o gesto que a feature existe para servir.
   Sinal real não tem fase nesses bins; eu escrevia uma; o `realfft` **rejeita a coluna
   inteira** (`FftError::InputValues`); o `is_err() { return }` a descartava; com 75 % de
   overlap as 4 colunas que cobrem cada amostra erram juntas → `wsum = 0` → o WOLA grava
   **zero**. Medido: **3350/4000 amostras da seleção viravam silêncio digital**, sem log e sem
   pânico. Fix estrutural na `Stft` (projeta `im = 0` antes da inversa — protege qualquer
   caller futuro) + semântico no `repair` (DC/Nyquist têm **sinal**, não fase).
2. **`De-Clip` REESCREVIA crista de áudio limpo** abaixo de ~2 kHz. O teste de planura era
   delta *por-amostra*, e a crista de um seno de 220 Hz é genuinamente plana entre amostras
   vizinhas (passo 4e-4) — **nenhuma constante** separa os dois assim. Dano medido num master
   quente-limpo: **0.0220**, 2,2× acima da barra do próprio gate. **E o gate mentia:** o
   fixture era um seno *puro*, que o modelo AR reconstrói **exatamente**, então o detector
   disparava e a saída voltava igual. O oráculo media o **interpolador**, não o detector.
   Fix: planura = **excursão da corrida inteira** (~0 no clipping, ~pico−threshold numa crista
   limpa — **independente da frequência**), a −86 dB, + `MIN_RUN` 3→6.
   *Preço honesto:* um platô **smeared** por codec lossy sai do alcance. Perder um caso mole
   bate destruir um limpo.

### Costura (o seam painel→intent estava sólido; os furos eram todos no shell)

3. O **spectrogram desenhava o clipe commitado** enquanto régua/playhead/hit-test usam o
   **soando** (a audition da rack — com reverb, mais **longa**). Um bipe no pixel *x* mapeava
   para outro instante: o Repair apagaria o lugar errado, sem crashar.
4. **`SpectralState` sobrevivia a um Load.** Perfil de ruído aprendido no clipe A + Load do
   clipe B = Denoise aceso, subtraindo de B o espectro de A. Os *caches* eram chaveados por
   ponteiro (seguros); o estado **aprendido** não era chaveado por nada.
5. A **banda era gravada em qualquer arrasto**, inclusive na waveform (onde não há eixo de
   frequência): um arraste horizontal sobrescrevia a caixa desenhada por uma degenerada.
6. Repair/Denoise que viravam no-op **gastavam um passo de undo**.
7. Um **Learn** com seleção curta demais **apagava em silêncio** um perfil bom.

### Gates que não provavam nada (todos consertados e mutation-tested)

- O **eixo Y da imagem** não era gateado (só o `freq_at_y` do shell). Os dois formam um **laço**:
  figura invertida → o usuário vê o bipe embaixo, arrasta embaixo, o `freq_at_y` diz "grave" →
  o repair apaga o grave. As 22 provas da crate ficavam verdes. *(E a 1ª versão do gate novo
  também passava — por **overflow de `u8`**: `250 + 100 = 94` em release. Certo na física,
  errado na aritmética.)*
- **Calibração unilateral:** `quantise_db` clampa em 0 dB, então uma referência **quente**
  satura e um teste em full-scale não a vê. Uma imagem **6 dB clara demais em toda parte**
  passava. Âncora agora em −20 dBFS.
- **Repair: fase e rota-do-tempo não eram provadas por nada** (fase zero, fase hash e
  só-frequência passavam **todas**). Oráculo novo = **coerência entre colunas** (CoV):
  coerente 0.33 · burst 1.04–1.43.
- **A sobre-subtração do denoise SAIU.** Meu comentário afirmava que ela compra margem contra
  musical noise. **Medido:** compra 4,8 dB de supressão e **custa 6,7 dB de fidelidade**
  (SNR 8,2 → 14,9 sem ela); o CoV não muda. Distorcia mais do que removia. Gate novo pina a
  **atenuação real do chiado** — porque SNR sozinho é gamificável *fazendo menos*.

### Performance (dois graves)

- **As crates de áudio não estavam em `[profile.dev.package]`** → rodavam em **opt-0** no
  `cargo run`. Tudo 15–25× mais lento: um Repair vira **3,1 s**, o resize do overlay **40
  ms/frame**. **Um smoke reportaria o W5 como quebrado com o release fazendo 3 ms.** É a mesma
  armadilha que o próprio `Cargo.toml` documenta ter causado o "FPS drop fantasma" do Painter.
- **Slider Amount do De-Clip: 94 ms/frame** (10 fps) — recalculava a reconstrução inteira, que
  **não depende dele** (Amount é um blend linear). Memoizada por (buffer, threshold):
  **94 → 0,9 ms/frame**.

### Limites CONHECIDOS que ficam (medidos, nomeados, não corrigidos)

| limite | número | o caminho |
|---|---|---|
| arrastar o **Threshold** do De-Clip num clipe longo muito estourado | até ~2 s/frame | o solver LSAR é O(m³) e a matriz é **Toeplitz** → O(m²) por Levinson (~200× em m=192) |
| `repair` faz a STFT do **clipe inteiro** (2× por canal) mesmo para uma banda de 5 % | 67 ms (60 s st.) · 230 ms (3 min) | rodar a STFT só na janela `[c0−1 .. c1+1]`; o splice já garante bit-identidade fora |
| resize do overlay re-renderiza o RGBA por frame | 3,3 ms (cabe nos 16 ms) | quantizar w/h, ou esticar durante o gesto |
| **HR-13 (30 MB) já não descreve o Audio Editor** | +55 MB transientes por render; +198 MB num clipe de 3 min | pré-existente ao W5 (toda op da rack copia o buffer inteiro); merece item próprio |

## 9. Aberto (não é regressão — é escopo)

- **Opus** ([ADR-0113](architecture/decisions/0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md) §Opus) — decisão do Enio, ainda pendente.
- **W7 (AI/ML)** — DeepFilterNet atrás de feature-flag; o kill-criterion do ADR-0122 §4 não disparou
  (o denoise bateu o alvo), então o W7 segue opcional e não obrigatório.
- Backlog pequeno: toggle *enabled* por-entry na variação · manifesto com caminho relativo · reverb
  por convolução.
- **Débito de LOC:** `fx.rs` voltou a 601/700 e `fx/dynamics.rs` está em 662/700 — **o próximo efeito
  em `dynamics.rs` exige split antes**.
