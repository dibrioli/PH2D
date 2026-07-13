# Handoff — continuação da `line/audio` (para o próximo agente)

> **Escrito por:** o agente que fechou a linha `audio` (W2 · cortes/peças · ADR-0117 memória ·
> ADR-0118 streaming · ADR-0119 regiões de loop). **Integração já feita** — `main` contém tudo.
> **Este documento é o seu briefing completo.** Leia inteiro antes de codar.

---

## 0. Estado — leia primeiro

- **`main` já contém a linha inteira** (o integrador rebaseou; os SHAs mudaram).
- A branch `line/audio` existe e está **idêntica a `main`** (`main..line/audio` = vazio).
- **`main` tem um commit do integrador que você precisa conhecer** (`83e596b7`):
  `shells/desktop/src/input_handlers.rs` estourou o cap de 600 LOC **só na árvore combinada** —
  a minha linha somou +41 linhas ali e outra linha somou as dela; **nenhuma das duas estourava
  sozinha**. Ele extraiu `shells/desktop/src/input_drop.rs`. **Lição operacional:** o cap de LOC
  do shell é um recurso **compartilhado entre linhas**. Se você mexer em `input_handlers.rs`,
  `input_dispatch.rs` ou `render_loop/mod.rs`, **orce o split desde já**.
- Último smoke aprovado pelo Enio: **regiões de loop (ADR-0119) — OK**.

---

## 1. Como trabalhar (Modo L) — [`GUIA_JORNADA_MODO_L.md`](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md)

Você é **uma linha autônoma**. Não há Coordenador. Resumo do que **te obriga**:

### 1.1 Setup (você mesmo faz, na 1ª mensagem)

```bash
cd /home/enio/Documentos/Projetos/PH2D
git pull --ff-only origin main
git worktree add -b line/audio-<sufixo> Worktrees/line-audio-<sufixo> main
cd Worktrees/line-audio-<sufixo>      # TODO o trabalho a partir daqui
```

> A worktree `Worktrees/line-audio/` **já existe** (é a minha, agora idêntica a `main`). Você pode
> reusá-la (`git checkout main && git pull --ff-only && git checkout -b line/audio-w3`) **ou** abrir
> uma nova. Reusar é mais barato: o `target/` já está quente (build frio = minutos).

**Depois do setup, NENHUM path da raiz é seu.** O mesmo caminho relativo existe nas duas árvores —
editar `crates/…` na raiz é editar o **checkout primário compartilhado**. Todo read/edit/git/cargo
acontece **dentro da worktree**, e **toda mutação por caminho absoluto** (um `sed -i` relativo
escreve na árvore errada).

### 1.2 As regras inegociáveis

| | |
|---|---|
| **NUNCA** | `git push` · integrar · rodar `ship.sh` |
| **NUNCA** | `git add -A` fora dos seus paths · `git stash` · `--force` |
| **Sempre** | `git commit --no-verify` (fast mode: instantâneo, zero CI) |
| **Contrato congelado** (CLAUDE.md §6) | **PARE e reporte ao Enio** — exige ADR |
| **Rebase conflita fora dos seus arquivos** | **PARE e reporte** — é design, não merge |
| **Foundational** (`ph2d-core`, `ph2d-audio`, `shells/*`) | **Você PODE e DEVE tocar** (ADR-0107), com cuidado. Ao **criar** foundational novo, projete p/ isolamento: **módulo irmão**, não engordar arquivo compartilhado |
| **UI** | **inglês**, sempre. Zero hex, zero `f32` literal de UI, zero string hardcoded (HR-15) |

### 1.3 Velocidade

- **Inner loop = SÓ `cargo check -p <crate>`.** Nada de test/clippy por task.
- **Gate batched 1× no fechamento** (não por task): `cargo nextest run --workspace` + clippy
  `--all-targets` + `typos` + `machete` + `fmt --check`.
- ⚠️ **RODE `nextest --workspace`, NÃO `-p`.** Os arch-gates (LOC cap, magic-numeric, contract
  surface) moram em `ph2d-editor-core` — `cargo test -p <suas crates>` **nunca os roda**. Foi assim
  que eu descobri 4 gates vermelhos **escrevendo o handoff**, não implementando.

### 1.4 Como você termina

Fecha o módulo → roda o gate batched → **escreve o handoff de integração
([DIRETRIZ §1.5.9](IntegracaoMultiAgente/DIRETRIZ.md))** → reporta **"linha pronta + handoff"** →
**PARA**. O Enio junta os handoffs e abre um **agente integrador dedicado**. Integrar ou pushar por
conta própria = **violação do protocolo**.

Modelo do meu handoff anterior, para copiar a estrutura:
[`docs/HANDOFF_INTEGRACAO_line_audio.md`](HANDOFF_INTEGRACAO_line_audio.md).

---

## 2. O que JÁ existe (não reimplemente)

### 2.1 As crates

| Crate | O que é |
|---|---|
| `ph2d-audio` | **O mixer** (RT). Vozes, buses, meters, **streaming** (`stream.rs`), **`LoopRegion`**. HR-3: **zero alloc/free/decode/lock na thread de áudio**. |
| `ph2d-audio-edit` | **O documento do editor** (control thread). `EditClip` = buffer + peaks + seleção + **`Structure`** (cuts/markers/loop) + histórico por delta. **39 efeitos.** |
| `ph2d-audio-encode` | WAV (+`smpl`/`cue`) · Ogg Vorbis · Opus. |
| `ph2d-audio-decode` | Symphonia + `Reader` incremental (streaming). |
| `ph2d-audio-opus` | Isola o `unsafe` do libopus. Encoda **e decoda** (Symphonia não lê Opus). |
| `ph2d-audio-spectral` | STFT/FFT (`realfft` confinada aqui — **não alcança o mixer**). |
| `ph2d-audio-stream` | Thread produtora: decodifica ⇒ ring ⇒ voz. |
| `ph2d-panel-audio-editor` | O painel (seções colapsáveis). |

### 2.2 A rack: **39 efeitos**

**`Effect` (35, length-preserving):** AutoPan · AutoWah · Bitcrush · Chorus · Comb · Compress ·
DeClick · DeClip · DeEss · DeHum · DePlosive · Distortion · Doubler · Exciter · Flanger ·
FormantShift · Gate · Haas · Harmonizer · HighPass · HighShelf · Leveler · Limiter · LowPass ·
LowShelf · Peak · Phaser · PitchShift · RingMod · Saturate · StereoWidth · TranceGate · Transient ·
Tremolo · Vibrato.

**`TailEffect` (4, cauda estende o buffer):** Reverb · Delay · PingPong · **Convolution**.

> **A INVARIANTE DA RACK — leia antes de adicionar efeito:**
> todo efeito é **no-op byte-idêntico no seu ponto neutro** (`is_bypass` espelha os defaults), e o
> **painel se auto-popula da tabela `KINDS`**. Adicionar um efeito = variant `Effect` + braço
> `apply`/`is_bypass`/`warmup` + row na tabela do shell. **ZERO mudança de painel.**
> **5 gates provam isso por-efeito**, em
> [`shells/desktop/src/audio/fx_params/tests.rs`](../shells/desktop/src/audio/fx_params/tests.rs):
> `every_effect_is_a_no_op_at_its_defaults` (neutro) · `turning_an_arming_knob_wakes_the_effect_up`
> (o arm acorda) · `no_slider_reads_a_false_zero` · `the_kind_table_is_the_rack_layout` ·
> `every_kind_has_a_spec_and_builds`. A tabela é `KINDS: [FxKind; 39]` em
> [`fx_params_table.rs`](../shells/desktop/src/audio/fx_params_table.rs) — **é ela que o painel lê**
> (`set_fx_kind_names`/`set_fx_kind_defaults`, no `render_loop`).
> Se o seu efeito novo não passa nesses 5, o erro é seu, não dos gates.

### 2.3 O que fechou nesta linha (com ADR)

- **ADR-0117 — memória do editor.** Histórico por **delta capeado por BYTES** (4351 MB → 156 MB).
  `SampleData::{from_fn, build, map_in_place}` constroem o buffer **uma vez** (`Arc::from(Vec)`
  SEMPRE realoca e copia). **HR-13 emendado: quem declara budget possui um gate que MEDE** (dhat,
  `tests/measure_*.rs`).
- **ADR-0118 — streaming.** Uma faixa de 3 min residente = **65,9 MB**; streamed = **0,06 MB**.
  Streaming é **bit-idêntico** ao residente (6 gates).
- **ADR-0119 — regiões de loop.** `PlayParams.loop_region`. Intro→loop no runtime. O metadado
  (`smpl`/`cue`) **volta no Load**. O Crossfade virou **bake destrutivo** (um loop de runtime *pula*).
- **Cortes/peças.** O clipe é **um buffer + uma lista de cortes**. Move (permutação byte-reversível)
  e Scale (WSOLA, pitch-preserving). Cuts+markers+loop viajam **juntos** no passo de undo.

---

## 3. A FILA — o que falta (do plano `docs/Audio/02_plano_implementacao_completo.md`)

Ordenada por **valor / risco**. Escolha com o Enio; **não comece sem a tarefa dele**.

### 🥇 W3 — cauda: **Multibanda + Expander** (o menor risco, valor real)

**O que falta, literalmente:** o plano W3 pede `compressor/limiter/gate/expander/de-esser/multibanda`.
Existem **Compress**, **Limiter**, **Gate**, **DeEss**. **Faltam `Expander` e `Multiband`.**

- **Expander** — o inverso do compressor (abaixo do threshold, *aumenta* a faixa dinâmica). É quase
  o `Gate` com ratio suave; o envelope já existe (`Compressor::prime` — sem ele a borda da seleção
  estala).
- **Multiband** — o compressor aplicado em N bandas independentes. **A decisão de projeto que
  importa:** o crossover. Um Linkwitz-Riley de 4ª ordem soma **plano** (fase alinhada); dois biquads
  em cascata **não** — e a soma das bandas sem compressão **tem de ser byte-idêntica ao input**, que
  é a invariante da rack. **Esse é o gate que você escreve primeiro.**

**Por que é o melhor primeiro passo:** custo baixo (a rack se auto-popula), risco baixo, e te ensina
a invariante da casa. **Só não caia na armadilha:** a soma no ponto neutro tem de ser
*byte-idêntica*, não *aproximadamente igual*.

### 🥈 W4 — cauda: **Vocoder / Robotize / Granular / Whisper-Shout**

O W4 é o mais completo (voz: de-hum, de-ess, de-plosive, leveler, transient, ring-mod, pitch/formant
shift, comms, cadeia criatura). **Falta a família do vocoder:**

- **Vocoder** — banco de filtros: o envelope da voz (modulador) modula um portador (serra/ruído).
  Clássico, e o `ph2d-audio-spectral` já te dá a análise se quiser fazer por STFT.
- **Robotize** — vocoder com portador de pitch fixo (ou zeragem de fase no STFT).
- **Granular** — o motor WSOLA já existe (`fx/wsola.rs`), mas exposto só como pitch/stretch.
- **Whisper / Shout** — presets, não motores novos.

### 🥉 W6 — cauda: **variantes de export por plataforma**

Existe: loop points · markers · containers de variação · import por convenção · export
WAV/Ogg/Opus · Delivery com preço de disco+RAM · Batch LUFS · **Export Pieces**.
**Falta:** *"export por plataforma com preview de tamanho"* — um perfil (mobile/desktop/console) que
escolhe codec+qualidade+rate e exporta o set. **Encanamento**, valor moderado.

### ⚠️ W7 — AI/ML (jornada própria, exige ADR ANTES)

`DeepFilterNet` (denoise nativo, crate `deep_filter`) · `Demucs` via `ort`/ONNX (stem-split offline).
**Tudo atrás de `feature = "audio-ml"`; build default não puxa deps pesadas.**
**Deps novas = ADR + autorização explícita do Enio.** Não comece sem isso.

### 🔧 Débitos abertos, dos ADRs (pequenos, bem definidos)

| Item | Onde | O que é |
|---|---|---|
| **Preview O(seleção)** | ADR-0117 §5 | Arrastar um knob num clipe grande ainda re-renderiza mais do que a seleção. O número está **registrado, não escondido**. |
| **Seek/scrub num stream** | ADR-0118 §5 | Uma stream não faz seek (o produtor teria de reposicionar o decoder e jogar o ring fora). Hoje é **no-op honesto**. |
| **Pitch ao vivo num stream** | ADR-0118 §5 | `advance` fracionário funciona; mudar pitch *durante* a stream, não. |
| **Toggle "Streamed" no Delivery** | ADR-0118 §5 | Só faz sentido quando o **jogo** carrega assets — hoje o editor **abre** um clipe, e um clipe aberto é residente por definição. |
| **Múltiplas regiões de loop** | ADR-0119 §5 | `smpl` aceita várias; jogos usam uma. O reader pega a primeira. |
| **`deny.toml` com ignore obsoleto** | — | `RUSTSEC-2023-0089` — "no crate matched advisory criteria". **Pré-existente, não é da minha linha.** Vale limpar num ship. |

---

## 4. As armadilhas que ME custaram tempo — não caia nelas

### 4.1 O gate que nasce cego

**Aconteceu 3× nesta linha.** Um gate que passa de primeira **não prova nada** até você **mutar o
código** e exigir que ele fique **vermelho**.

- **Sinal de teste errado.** O gate da imagem estéreo usava L e R **correlacionados** (um atraso do
  outro) — as duas implementações (certa e errada) davam **a mesma resposta**, porque a
  auto-similaridade é idêntica. O gate media **nada**. Só a mutação mostrou.
- **Taxa 1:1 esconde frame segurado.** Num gate de **emenda** (loop wrap, costura de stream, splice
  de grão), taxa da fonte = taxa de saída ⇒ `frac` é sempre 0 ⇒ **o segundo frame da interpolação
  nunca é lido** ⇒ um "frame segurado" é invisível. **Qualquer coisa sobre emenda tem de ser medida
  com avanço fracionário** (fonte a `OUT_RATE/2`, ou 44,1k contra 48k).
  → memória: `feedback_seam_gates_need_fractional_advance`
- **O fixture sem outro.** O gate do produtor não distinguia "nunca vira no fim da região" de "vira
  no EOF" — porque a região **terminava onde o arquivo terminava**. Um **outro alto que o loop nunca
  pode alcançar** transformou dois bugs em **um número**.

### 4.2 O master grampeia em unidade

Um "stamp" por frame (`valor = índice`) é a melhor forma de saber **qual frame de origem** saiu — mas
**escale para abaixo de 1.0**. Acima disso o master clipa e **todo stamp volta como √2**. Perdi uma
rodada nisso.

### 4.3 `str.replace()` que não casa é um no-op **silencioso**

Ao editar com python, **sempre `assert old in s`**. Um replace que não acha o padrão **não falha** —
escreve o arquivo inalterado e sai com 0. E `cargo fmt` **reflowa o texto entre uma edição e a
próxima** (colapsou um `match` multi-linha e o meu padrão deixou de existir): a correção "aplicada"
nunca entrou, o gate ficou verde sob mutação, e eu quase concluí que o gate é que estava cego.
**Prefira a Edit tool** (ela **erra** quando não casa).
→ memória: `feedback_python_replace_silent_noop_after_fmt`

### 4.4 O `populate.rs` é invisível para o seam test

Um botão pode estar **pintado, mapeado em `event.rs`, coberto por seam test — e MORTO no app**, se
faltar no `populate.rs`. O seam injeta `Click(id)` direto; no app o clique chega como **posição**, e
é o `WidgetStore` (que o `populate` enche) que traduz. **Gate:
`ph2d-panel-audio-editor/tests/no_dead_buttons.rs`** — a lista `CLICKABLE` é o checklist. **Botão
novo = entrada nova ali**, ou ele não está testado.

### 4.5 O cap de LOC do shell é **compartilhado entre linhas**

Ver §0. Se mexer em `input_handlers.rs` / `input_dispatch.rs` / `render_loop/mod.rs`, **orce o
split**. E lembre: **LOC cap = split, não allowlist** (extraia módulo irmão). Meça **depois** do
`fmt` — ele re-expande.

### 4.6 O que NÃO é gateável headless

`AudioSystem::new()` precisa de **device de áudio**, e **nenhum teste em `shells/desktop/tests/`
constrói um**. Então: o gesto de peça (press→grab→drag→release), o som saindo do device e o
transporte real são **smoke-only**. O **modelo** (`ph2d-audio-edit`, `ph2d-audio`) é onde os gates
vivem — **empurre a lógica para lá** e o shell fica fino o bastante para ser óbvio.

---

## 5. Onde ler (só o que sua tarefa exigir)

| Sua tarefa | Leia |
|---|---|
| **Qualquer implementação** | [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) — regra-mãe: *verde-de-compilação é velocidade; no audit vale ZERO* |
| **Modo L / como fechar** | [`GUIA_JORNADA_MODO_L.md`](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) + [`DIRETRIZ.md §1.5`](IntegracaoMultiAgente/DIRETRIZ.md) |
| **Efeito novo** | `crates/ph2d-audio-edit/src/fx.rs` (a tabela `KINDS`) + `shells/desktop/src/audio/fx_params_table.rs` |
| **Plano do módulo** | [`docs/Audio/02_plano_implementacao_completo.md`](Audio/02_plano_implementacao_completo.md) §7 |
| **O que já landou** | [`HANDOFF_audio_line_continuation.md`](HANDOFF_audio_line_continuation.md) — as duas últimas seções são cortes/peças e regiões de loop |
| **Bugs conhecidos** | [`docs/Audio/BUGS_audio.md`](Audio/BUGS_audio.md) |
| **Memória / lições** | [`project-memory/MEMORY.md`](../project-memory/MEMORY.md) — **leia o índice antes de agir** |
| **Hard Rules** | `SKILL_Stack_PH2D_Definitiva.md` §HR-1..18 (cite por ID) |

---

## 6. Sua primeira mensagem ao Enio

Faça a **triagem** (DIRETRIZ §2) e **pergunte qual item da fila** — não escolha sozinho. Sugestão de
recomendação, se ele deixar você escolher:

> **Comece pelo W3 (Multibanda + Expander).** É o menor risco, valor real, e a *soma das bandas no
> ponto neutro tem de ser byte-idêntica ao input* — o que te obriga a aprender a invariante da rack
> **escrevendo um gate**, que é exatamente o jeito certo de entrar neste módulo.

E antes de qualquer coisa: **`bash scripts/hw-profile.sh`** (a estratégia é função do hardware) e
**leia `project-memory/MEMORY.md`**.
