# HANDOFF DE CONTINUAÇÃO — linha `line/audio`

> **Para o próximo agente-de-linha.** A jornada anterior **foi integrada ao `main`**
> (2026-07-12, 37 commits). O worktree já foi **rebaseado e está limpo, 0 commits à
> frente do main**. Você continua daqui.
>
> Leia **este doc inteiro** + os obrigatórios do §1 antes de escrever a primeira linha
> de código.

---

## 1. VOCÊ ESTÁ EM MODO L — as regras (DIRETRIZ §1.5 · [MODELO_ABERTURA_LINHA](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md))

**Sua branch:** `line/audio` · **Sua worktree:** `Worktrees/line-audio/`
Esta linha é **reaberta** (o worktree já existe — não recrie).

### Setup (rode já, sem pedir confirmação; reporte cada ✗)
```bash
bash scripts/hw-profile.sh          # DEVE dizer `workstation`. Disse `constrained`? PARE.
cd Worktrees/line-audio
git branch --show-current           # DEVE imprimir line/audio
git rebase main                     # rota "linha reaberta" — no início de CADA jornada
cargo check -p ph2d-audio-edit      # warm-up
```

### Leia INTEIRAS (dentro da worktree), antes de codar
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` → **§0, §1.5, §2, §6**
- `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` → **tudo**, e **RELEIA a cada
  passo** (é o antídoto das 4 causas da semana perdida no Painter).

### REGRAS PERMANENTES (valem até o fim, sem exceção)

| | |
|---|---|
| **A** | **TODO** read/edit/git/cargo acontece **DENTRO da worktree** (`Worktrees/line-audio/`). A raiz do repo é o checkout primário compartilhado — **o mesmo path relativo existe nas duas árvores**. Editar `crates/...` na raiz = editar a árvore ERRADA. **Na dúvida, `pwd` antes de editar.** (Lição cara: `sed -i` com path relativo escreve no repo errado — **mutação sempre por caminho absoluto**.) |
| **B** | Edite a pasta do seu módulo à vontade. **Foundational você PODE e DEVE tocar** (com cuidado, ADR-0107). **PARE e reporte ao Enio SÓ se:** (a) for **contrato congelado** (CLAUDE.md §6 — exige ADR), ou (b) o rebase conflitar em código **FORA** dos seus arquivos (colisão de mesmo-símbolo). **Nunca negocie com outra linha.** |
| **B'** | Ao **CRIAR** foundational novo, projete pra **ISOLAMENTO**: prefira **módulo/arquivo IRMÃO novo** a engordar arquivo compartilhado; ponto de extensão **append-only**. Todo id/const/variant novo → **anote no handoff de integração** (regra H) pro integrador detectar colisão. |
| **C** | Commits locais frequentes: `git commit --no-verify`. **NUNCA `push`. NUNCA `--force`. NUNCA `git add -A`.** |
| **D** | `git rebase main` no início de cada jornada. Conflito em `Cargo.lock` ou arquivo GERADO: **NUNCA** resolva na mão — refaça por geração. |
| **E** | Fechamento = **gate batched** (§7 abaixo). Depois **PARE**. **Você NÃO integra e NÃO roda `foundational-integrate.sh`.** Quem funde é um **agente integrador dedicado**, só por **ordem explícita do Enio**. |
| **F** | **Ship** (`ship.sh` + push + babysit CI): **NUNCA** por conta própria. Ordem explícita do Enio, feita pelo integrador. **Integrar ou pushar sem ordem = violação do protocolo.** |
| **G** | **UI canônica:** zero hex, zero `f32` literal de UI, zero string hardcoded — tudo por tokens/i18n. **UI do app em INGLÊS** (labels/toasts). |
| **H** | **HANDOFF DE INTEGRAÇÃO é entregável obrigatório ao fechar** (DIRETRIZ §1.5.9): branch/HEAD/base · foundational tocado + por quê · **ids/consts novos com valores** · contratos congelados encostados (deve dar nenhum) · **deps novas** · o que só o `ship.sh` pega · o que smoke-testar. Reporte "linha pronta + handoff" e **ESPERE**. |

---

## 2. Onde a linha parou (estado real, verificado)

- **Worktree:** rebaseado no main (`3805f650`), **0 commits à frente**, **árvore limpa**.
- **Gate na base nova:** `ph2d-audio-edit` **122** testes · painel **20 lib + 29 seam** ·
  shell **7 suites** — **todos verdes**.
- **A jornada passada entregou:** rack **14 → 34 efeitos**, presets **7 → 15**, containers
  de variação, import por convenção, export **Ogg Vorbis** (ADR-0113), e 3 bugfixes de
  auditoria (undo/redo/invert intermitentes).

### ⭐ O PADRÃO DA RACK — leia antes de adicionar qualquer efeito

**Adicionar um efeito NÃO toca o painel.** O painel se auto-popula da tabela `KINDS`
(`set_fx_kind_names` / `set_fx_kind_defaults`, empurrados todo frame pelo shell).
São **5 pontos**, sempre os mesmos:

1. **DSP** num módulo de `crates/ph2d-audio-edit/src/fx/` (`tone.rs`, `dynamics.rs`,
   `modulation.rs`, `space.rs`, `pitch.rs`, `comb.rs`, `autowah.rs`, `transient.rs`…).
2. **Variant** em `Effect` (ou `TailEffect`, se estende a duração) — `fx.rs` / `fx/tail.rs`.
3. **Braço `apply`** com o **guard do ponto neutro** + **cláusula `is_bypass`** + (se tiver
   estado) **braço `warmup_frames`**.
4. **Row** em `shells/desktop/src/audio/fx_params_table.rs` + **specs** em
   `fx_param_specs.rs` (arquivos irmãos, por causa do teto de LOC).
5. **Teste de layout** em `fx_params.rs` (a lista de nomes, pinada).

**A INVARIANTE INEGOCIÁVEL:** todo efeito é **no-op byte-idêntico no seu ponto neutro**.
Não é "quase" — um filtro no topo da faixa ainda desloca fase; um compressor 1:1 ainda
arredonda. Por isso o neutro é um **bypass explícito** (`is_bypass` → `apply` devolve
`data.clone()`), nunca emergente.

**Os 5 gates da rack provam isso por-efeito, automaticamente** (em `fx_params.rs`):
neutro é no-op · o *arm* acorda o efeito · os **outros** knobs são inertes enquanto
neutro · o layout está pinado · nenhum slider mostra um "0" falso. **Se seu efeito passa
nos 5, ele está costurado.**

### Gotchas que já custaram tempo (não repita)

- **`fx/dynamics.rs` está em 662/700** — o **próximo** efeito de dinâmica **exige split**
  (módulo irmão, como fiz com `deplosive.rs`/`transient.rs`).
- **Tetos de LOC:** `crates/**` ≤ **700** · `shells/desktop/src` ≤ **600** · painel ≤ 600
  arquivo / 200 fn. **Split, nunca allowlist.** Meça **DEPOIS** do `fmt` (ele re-expande).
- **⚠️ O gate de LOC do shell e o `typos` NÃO rodam em `cargo test --bins`.** Eu descobri
  os dois **só no fechamento**. **Rode no loop:** `cargo test -p <crate> --tests` **e**
  `typos` **e** `cargo fmt --all -- --check` — não só `--bins`.
- **`fmt`:** use `rustup run 1.95 rustfmt --edition 2024` (o `cargo fmt` puro dá skew).
- **`gen` é palavra reservada** na edition 2024 (me pegou no Exciter).
- **HR-3/HR-5 (no-alloc/no-transcendentais) valem SÓ pra thread de áudio RT.**
  `ph2d-audio-edit` é **control-thread** — pode alocar e usar `sin`/`exp`/`tanh` à vontade.
- **O probe dos gates da rack é estéreo, 2400 frames = 50 ms.** Efeito rítmico lento não
  "acorda" dentro dele (o Trance Gate me pegou — precisei de default ≥ 11 Hz).
- **Commit message com parêntese/backtick quebra o fish** → use `git commit -F <arquivo>`.

---

## 3. ETAPAS PLANEJADAS — continue daqui

Fonte: [`docs/Audio/02_plano_implementacao_completo.md`](Audio/02_plano_implementacao_completo.md) §7.
Ordem do plano: W1 → W2 → W3 → (W4 ∥ W5) → W6 → W7.

### ✅ Fechado
**W1** (esqueleto/transporte/waveform/WAV) · **W2** (edição offline + undo) ·
**W3** (rack + cadeia editável).

### 🔵 ETAPA 1 (recomendada) — **W4 restante, tudo SEM dep nova**
O W4 está majoritariamente fechado. Falta (verifiquei por grep — **0 arquivos** hoje):

| Item | Abordagem sugerida | Dep? |
|---|---|---|
| **De-click / mouth-declick / de-crackle** | **LPC + interpolação** (o plano já manda isso). Detecta o clique pelo erro de predição linear, substitui a amostra pela predição dos vizinhos. **É tempo-domínio → NÃO precisa de FFT.** | **Não** |
| **Formant shift** (separado do pitch) | **PSOLA** (tempo-domínio, clean-room — o plano diz "PSOLA/phase-vocoder próprio; **Rubber Band é GPL**, não copie"). Reaproveita a linha de delay/grãos que o `fx/pitch.rs` já tem. | **Não** |
| **Harmonizer** | Barato: **N cópias do `pitch_shift` existente** nos graus (3ª/5ª/oitava), misturadas. O motor já está lá e testado. | **Não** |
| **EQ-voz** (HPF + presença + de-mud + air) | **Preset de fábrica** (puro data em `fx_presets.rs`) — os 5 filtros já existem. Barato, alto valor. | **Não** |
| **Whisper / shout** | Presets combinando os efeitos existentes (exciter + compress + saturate + EQ). | **Não** |

**Por que começar aqui:** zero fricção (sem dep, sem ADR), o motor já existe, e fecha a
wave. O **de-click (LPC)** é o de maior valor real — é o único item de *reparo* que sobrou.

### 🟡 ETAPA 2 — **W5 Espectral (FFT)** — ⚠️ **PRECISA DO OK DO ENIO**
A wave grande que sobrou. **Bloqueada por decisão do Enio:**
- **Exige dep nova:** `realfft` (ou `rustfft`) → **ADR obrigatório** + **autorização
  explícita do Enio** antes de adicionar.
- **Escopo:** spectrogram (STFT, Hann/Blackman-Harris) alternando com a waveform no
  overlay · seleção tempo-frequência · **spectral repair/inpaint** (heal brush — interpola
  bins vizinhos, remove tosse/bipe pontual) · **spectral denoise** (subtração espectral /
  Wiener por bin, aprende profile) · de-clip.
- **Gate do plano:** métricas de **SNR antes/depois** em fixtures.
- **Nota minha:** eu **deliberadamente evitei FFT** no pitch shift (usei granular, que é a
  ferramenta *certa* pra voz de personagem). Mas o W5 **realmente precisa** de FFT — não dá
  pra fingir. **Escreva o ADR primeiro, mostre ao Enio, espere o OK.**

### 🟢 ETAPA 3 — **W6 restante**
Loop points, markers, variação, import e OGG **já landaram**. Falta:
- **Opus** — ADR-0113 §Opus já analisou: recomendação = **crate irmão isolado
  `ph2d-audio-opus`** (puro-Rust, `unsafe` contido). **Decisão do Enio.**
- **Codec/residência por-asset** + readout de tamanho/RAM.
- **Force-to-mono** p/ 3D (+ warn se estéreo).
- **Batch LUFS** não-destrutivo (o `normalize` LUFS já existe — falta o batch).

### ⚪ ETAPA 4 — **W7 AI/ML** (opt-in, feature `audio-ml`)
DeepFilterNet (denoise) · Demucs via ONNX. **Tudo atrás de feature-flag** (build default
não puxa deps pesadas). Longe; só depois do W5.

---

## 4. Backlog / dívida conhecida (não bloqueia)

- **`fx/dynamics.rs` em 662/700** — próximo efeito de dinâmica = **split obrigatório**.
- Variação: toggle *enabled* por-entry na UI (o modelo e o manifesto **já** carregam o
  campo — é só UI). O manifesto guarda **caminho absoluto** (relativo seria mais portátil).
- Reverb por **convolução** (o atual é Freeverb algorítmico).

## 5. 🚩 Deferidos a OUTROS DONOS (não são seus — não conserte)

Da auditoria de intermitências da jornada passada:
1. **Undo global grava passos espúrios** (dono: `undo.rs` / sim) — os sprites da cena
   default têm `Velocity` e bouncam **todo frame** (sim não-gated em play/pause), então
   `post_frame_undo` registra diff toda hora. **Fix real:** gatear a sim, ou o
   `post_frame_undo` ignorar diffs só-de-sim.
2. **Timeline/motion preemptam o Ctrl+Z do áudio** — os blocos deles rodam **antes** do de
   áudio em `input_dispatch/keyboard.rs`. **Recomendação:** centralizar a prioridade de undo
   num ponto só.
3. **Gap de harness de teclado** — não existe harness headless que dirija `handle_editor_key`
   num `App` completo. Por isso o fix do Ctrl+Z do áudio **não tem asserção-vermelha**.

---

## 6. Smoke — o arquivo de teste

O Enio testa com um **WAV estéreo sintético** (transientes espaçados + melodia sustentada
L≠R + zumbido 60 Hz + clicks brilhantes) que exercita a rack inteira. Se precisar
regerar, o script está no scratchpad da sessão passada — ou refaça: 48 kHz, estéreo,
16-bit, ~6 s.

**Linha de comando pra rodar (sempre com o `cd` junto):**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && cargo run -p ph2d-host-desktop --features panel-audio-editor
```

## 7. Gate de fechamento (rode 1× no fim, sobre o diff acumulado)

```bash
cargo test -p ph2d-audio-edit -p ph2d-panel-audio-editor          # model + painel + seam
cargo test -p ph2d-host-desktop --tests                            # ⚠️ --tests, NÃO --bins (LOC gate mora aqui)
cargo test -p ph2d-editor-core --tests                             # 32 suites arch-gate
cargo clippy --all-targets -p ph2d-audio-edit -p ph2d-host-desktop
rustup run 1.95 cargo fmt --all -- --check                         # fmt canônico
typos                                                              # ⚠️ não roda em cargo test
cargo deny check && cargo machete                                  # só se mexeu em deps
```
Depois: **escreva o handoff de integração (regra H) e PARE.**
