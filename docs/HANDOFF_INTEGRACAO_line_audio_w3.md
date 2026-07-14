# Handoff de integração — `line/audio-w3` (DIRETRIZ §1.5.9)

> **Escopo:** as caudas do **W3**, do **W4** e do **W6** do plano de áudio — **o plano fechou**
> (só o W7, AI/ML, fica de fora: exige ADR) — mais o **ADR-0120** (o débito de perf que o ADR-0117
> tinha registrado).
> **Estado:** gate batched verde (workspace **6620/6620**). W3, W4 e W6 **smokados e aprovados**
> (Enio, 2026-07-13).
>
> **A rack foi de 39 para 42 efeitos**, e o W6 achou um **bug de aliasing** no `conform` que ia
> shipar lixo pra dentro de um jogo (§3.4). O padrão dos três waves foi o mesmo: **metade dos
> itens da fila não eram o que o plano dizia.** Vale ler o §2 antes de acreditar num plano.

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/audio-w3` (worktree `Worktrees/line-audio/`) |
| **Base** | `main` @ `44f89ad7` |
| **Commits** | 12 (W3 · W4 · W6 · ADR-0120 · 3 smokes · handoff · memória) |
| **Gate batched** | `nextest --workspace` **6620/6620** · `clippy --all-targets` exit 0 · `fmt --check` · `typos` · `machete` — todos limpos |

---

## 2. O que fechou

### 2.1 Multiband — o 40º efeito da rack

Compressão em três bandas (low/mid/high) sobre um crossover **Linkwitz-Riley de 4ª ordem**.
Cada banda roda o **mesmo `compress()`** que o Compress da rack já é — então o envelope primado
(sem estalo na borda da seleção) e o make-up peak-preserving vêm junto, e "Multiband" significa
exatamente *"aquele compressor, por banda"*, não um segundo sutilmente diferente.

### 2.2 O Expander **não** foi construído — ele já existia

O handoff anterior dizia *"faltam `Expander` e `Multiband`"*. **Metade disso estava errado.**
`Effect::Gate` já é um **expander descendente** cujo `ratio` varre de expansão suave a gate duro
sem chave de modo — o próprio `dynamics.rs:1` já dizia *"the feed-forward compressor, the downward
**expander / gate**"*. Um 40º efeito chamando a mesma função `gate()` com um range de ratio menor
seria **uma linha falsa na rack** para fechar um checklist. Não foi feito; a doc de `Effect::Gate`
agora diz explicitamente que ele **é** o expander da rack.

**Decisão pendente do Enio (1 linha de código, não fiz por conta própria):** renomear a row
`"Gate"` → `"Gate / Expander"` deixaria a palavra descobrível no seletor — mas os presets resolvem
kind **por nome** (`kind_by_name`), então um preset que hoje diz `"Gate"` passaria a resolver
`None` e **sumiria em silêncio**. Trade real: descoberta × quebrar preset. Não renomeei.

---

### 2.3 W4 — Vocoder (41º) e Granular (42º); os outros dois itens **não eram efeitos**

O plano pedia *"Vocoder · Robotize · Granular · Whisper/Shout"*. **Dois eram o mesmo motor:**

- **Robotize** — um vocoder de **uma entrada** sintetiza o portador internamente, num pitch fixo.
  E *"vocoder com portador de pitch fixo"* **é** o robô. É este efeito com **Breath = 0**.
- **Whisper** — com portador de **ruído**, o mesmo motor **sussurra**: excitação não-vozeada
  atravessando um trato vocal é o que sussurrar fisicamente **é**. **Breath = 1**.
- **Whisper** e **Shout** já existiam como presets. **"Robot" também** (`RingMod + Distortion +
  Bitcrush` — o robô *metálico*, um som válido e **diferente**). **Não toquei em nenhum.**

Entregues: 2 efeitos + 2 presets (**"Vocoder"**, **"Vocoder Whisper"**).

**Vocoder** — banco de band-pass log-espacados; o **Q sai da própria espaçagem** (recíproco de
`√r − 1/√r`), não de um número escolhido — é o que faz o banco *ladrilhar* o espectro em vez de
deixar buracos. Portador com **whitening** (cada banda em nível unitário), senão o tilt 1/f da
serra fica assado em toda vogal. E **band-limited por síntese aditiva** numa wavetable de **um
período inteiro em amostras**: harmônicos até Nyquist e nem um a mais.

**Granular** — grão *windowed* reposto fora de ordem (scatter no tempo, pitch destoado,
overlap-add). **Não é o WSOLA de chapéu:** aquele *sincroniza* os pedaços para **esconder** a
emenda; este os espalha porque a emenda **é** o efeito. Mesmo maquinário, escalonador oposto.

> **A armadilha do Granular, e ela está gateada nos dois sentidos:** Hann a meio-hop soma
> **exatamente 1**, então com scatter 0 e pitch 0 a nuvem reconstrói o input **amostra por
> amostra**. Isso é uma boa propriedade (diz que o overlap-add é transparente) — **e é por isso
> que o Scatter default NÃO pode ser 0**: um granular totalmente wet seria um no-op e ligar o Mix
> não faria nada. O gate `turning_an_arming_knob_wakes_the_effect_up` estaria **certo** em falhar.

## 3. Os achados de DSP (o valor real desta linha)

### 3.1 O gate que o handoff anterior prescreveu é **impossível de passar**

Ele mandava, textualmente: *"a soma das bandas sem compressão tem de ser byte-idêntica ao input …
Esse é o gate que você escreve primeiro."*

**Não é, e não pode ser.** Um LR4 soma para um **allpass**: magnitude plana (±0,0000 dB) e fase
rodada uma volta inteira através da esquina. Medido: o impulso somado difere do input em **0,0713
de fundo de escala**. Nenhum crossover real satisfaz byte-identidade, e o único jeito de deixar
esse gate verde é **afrouxá-lo até ele não medir nada** — que é a armadilha do "gate que nasce
cego" (§4.1 do handoff anterior) numa roupa nova.

O gate certo é **magnitude plana**. O neutro byte-idêntico da rack fica onde fica em todo efeito:
em `is_bypass` (ratio 1 ⇒ o crossover nem roda).

### 3.2 A árvore de 3 vias ingênua tem um dip real

Split em f1, depois split da metade alta em f2 — a banda **grave nunca atravessa o 2º crossover**,
então chega carregando uma fase que as outras duas não têm, e a soma **afunda na esquina baixa**:

| f1 / f2 | dip da árvore ingênua |
|---|---|
| 200 / 2000 (o nosso) | **−0,113 dB** @ 262 Hz |
| 200 / 1000 | −0,47 dB |
| 300 / 600 | −3,57 dB |
| 400 / 500 | **−11,96 dB** |

Fix padrão: rodar a grave pelo **allpass** do 2º crossover (`LP4 + HP4` em f2), e as três bandas
voltam a carregar a mesma rotação. **Plano em todo espaçamento** (±0,0000 dB) — o que significa
que **crossovers móveis são seguros** se alguém quiser expô-los depois (hoje são fixos: a rack tem
4 sliders, não 6).

> A previsão numérica (Python, antes de escrever Rust) e a implementação batem quase exatamente:
> −0,1130 dB @ 264 Hz previsto, **−0,1135 dB @ 262 Hz** medido.

---

### 3.3 Os 7 gates do W4 nasceram vermelhos — e **nenhuma vez foi o DSP**

Vale mais que o código. **Quatro** gates do Vocoder e **um** do Granular falharam de primeira, e
em todos os casos o defeito era a **medição**:

- **Vocoder (4 de 4).** Eu media "energia harmônica" com band-pass de **Q = 4** — largura ~180 Hz.
  A 2ª harmônica da voz (180 Hz) e a fundamental do portador (160 Hz) caíam na **MESMA banda**, e
  ruído de banda larga (o portador do whisper) lia como se fosse **pitch**. A sonda media a si
  mesma. Trocada por **bin de DFT exato** (1 Hz sobre 1 s) e o fixture por um **source-filter** de
  verdade — o antigo somava senos nas duas formantes e só trocava as amplitudes (contraste de
  **1,4×** na fonte: um fixture que mal sustenta a propriedade não pode prová-la). Depois disso:
  **158×**, **3,5×**, **23×**. O DSP estava certo desde o primeiro `cargo check`.
- **Granular (1).** Media energia numa janela específica **100 ms adiante** do clique — o que só
  acontece se algum grão sortear o deslocamento certo (~19% por grão). **Cara-ou-coroa no seed**;
  perdeu. Trocado pela **largura RMS da distribuição de energia no tempo**, onde todo grão
  contribui: **0,72 ms → 64,60 ms**.

**A regra:** quando um gate novo fica vermelho, o suspeito nº 1 é a **sonda**, não o código — e a
recíproca (a mutação que não morde, §4) também. Ambas as direções custaram uma rodada nesta linha.

### 3.4 W6 — o `conform` estava com **aliasing**, e ia shipar pra dentro de um jogo

O item era *"variantes por plataforma com preview de tamanho"*. O primeiro achado veio de graça:
**o ADR-0118 já provou que o codec não economiza um byte de RAM** (o mixer decodifica pro mesmo
buffer `f32`). Então uma "variante" que só trocasse o container imprimiria **o mesmo número de RAM
nas três linhas** — a mentira exata que aquele ADR foi escrito pra matar. Um alvo é um **FORMATO**
primeiro, codec depois:

| Alvo | Formato | Codec | O que ele compra |
|---|---|---|---|
| **Mobile** | **24 kHz mono** | Ogg Vorbis | **Um QUARTO** da RAM — o único que conforma o áudio |
| **Desktop** | 48 kHz estéreo | Opus | Melhor qualidade por bit |
| **Console** | 48 kHz estéreo | WAV16 | O único que carrega **loop points e markers** |

Medido: **288.000 / 1.152.000 / 1.152.000 B**. (Opus é **48 kHz e nada mais**, então um perfil
"24 kHz Opus" precificaria a RAM em ¼ da verdade — `a_platform_cannot_lie_about_its_sample_rate`
proíbe, e é por isso que o Mobile leva Vorbis.)

**E aí o gate do smoke ficou vermelho.** `ph2d_audio_edit::conform` reamostra por **interpolação
linear sem filtro anti-aliasing**. Subir (44,1k → 48k — o caminho do *paste*, o **único caller que
existia**) é inofensivo. **Descer dobra:** tudo acima do Nyquist do destino volta pra dentro da
banda. Um shimmer de 15 kHz conformado pra 24 kHz reaparecia como um **tom de 9 kHz que nunca foi
gravado**. O Mobile é o **primeiro caller que desce** — e teria shipado isso.

Fix **portado, não inventado** (DIRETIVA §1): band-limit **antes** de decimar — FIR
**windowed-sinc** (Smith, *Digital Audio Resampling*), janela Blackman (stopband ~−74 dB), fase
linear com o group delay compensado.

| | |
|---|---|
| 15 kHz num alvo de 24 kHz | **−57,3 dB** (antes: passava inteiro e dobrava) |
| 1 kHz no mesmo alvo | **−0,00 dB** — é anti-alias, não EQ |
| Subida de taxa | **byte-idêntica** — o paste não foi tocado |
| Um clique | **não se move** — o delay é compensado |

> É a lição do ADR-0118 pela terceira vez, e a mais barata de esquecer: **uma rotina correta para o
> caller pra quem foi escrita não é, por isso, correta.** Aqui "o outro lado" não era outro
> subsistema — era **a outra direção da mesma função**.

**Bônus:** loop points e markers são **índices de frame**, e um alvo que reamostra muda o que um
frame *é*. Escrever os números do master num arquivo de 24 kHz poria o loop **no dobro do tempo** —
uma emenda no lugar errado, num asset shipado, que nada a jusante detectaria. `rescale()` + gate.
Dormente hoje (só o Console escreve WAV, e na taxa do master), mas *"por sorte"* não é desenho.

## 4. Os gates (e por que cada um morde)

Todos em `crates/ph2d-audio-edit/src/fx/multiband.rs`, exceto o último.

| Gate | O que prova |
|---|---|
| `the_crossover_sums_flat` | A soma reconstrói a **magnitude** do input: pior desvio **+0,0012 dB** |
| `without_the_phase_compensation_the_sum_dips_at_the_low_corner` | **A mutação.** Corte a compensação e o gate acima fica **vermelho** (−0,1135 dB) |
| `every_band_compresses_on_tilted_material` | O threshold segue o pico **de cada banda** |
| `it_never_raises_the_peak` | A promessa do compressor, mantida por banda |
| `measure_multiband.rs` (dhat) | Pico **3,00× o clipe** — uma banda viva por vez |

**O bar de 0,05 dB saiu da margem MEDIDA dos dois lados** (42× acima da verdade, 2,3× abaixo do
bug), não de um chute. Um bar escolhido frouxo o bastante pra passar é um bar que não mede nada.

### ⚠️ A mutação que quase me enganou (vale para quem tocar nisto)

Minha primeira mutação removeu **só a metade high-pass** do allpass — e o gate ficou **verde**, o
que parecia dizer "o gate é cego". Não era: uma década abaixo de f2 essa metade está ~−96 dB
abaixo, então tirá-la muda a soma em **0,001 dB**. O que falta à banda grave na árvore ingênua é a
**FASE** daquele estágio, não a energia dele — a mutação tem que bypassar o **estágio inteiro**.
O gate não era cego; a *mutação* é que era. Está documentado no `Chain`.

---

## 5. Superfície tocada (para o grep do integrador — §1.5.5)

### Foundational / compartilhado

| Arquivo | O que mudou | Risco de colisão |
|---|---|---|
| `crates/ph2d-audio-edit/src/fx.rs` | `mod multiband` · variante `Effect::Multiband` (**apendada depois de `Compress`**) · braço em `apply` · `is_bypass` | **Baixo** — `Effect` é enum da `-edit`, não contrato congelado |
| `crates/ph2d-audio-edit/src/fx/dynamics.rs` | `COMPRESS_MAX_MAKEUP` virou `pub(super)` (1 linha) | Nenhum |
| `crates/ph2d-audio-edit/src/fx/warmup.rs` | 1 braço novo (`Effect::Multiband`) | Nenhum |
| `shells/desktop/src/audio/fx_params_table.rs` | `KINDS: [FxKind; 39]` → **`; 42]`** + 3 rows | ⚠️ **É um NÚMERO QUE SOMA** — ver abaixo |
| `shells/desktop/src/audio/fx_param_specs.rs` | `static MULTIBAND` / `VOCODER` / `GRANULAR` | Nenhum |
| `shells/desktop/src/audio/fx_params/tests.rs` | lista pinada +3 | ⚠️ idem |
| `shells/desktop/src/audio/fx_presets.rs` | `FACTORY: [Preset; 21]` → **`; 23]`** + 2 presets | ⚠️ **outro número que soma** |
| `shells/desktop/src/main.rs` | 2 ramos de env (os smokes) | Baixo |
| `shells/desktop/src/audio/editor.rs` | 2 `mod` novos | Baixo |

### ⚠️ `KINDS: [FxKind; 42]` e `FACTORY: [Preset; 23]` são números que SOMAM entre linhas

Se outra linha também adicionar um efeito, o merge textual vai apresentar `42` de um lado e `40`
do outro — e **o valor certo é 43**, que não existe em nenhum dos dois lados do conflito.
**Conte, não escolha.** O mesmo vale para a lista pinada em `the_kind_table_is_the_rack_layout`:
as duas entradas novas precisam existir, e o teste é quem prova.
(`project-memory/feedback_numbers_that_sum_across_lines_count_dont_pick.md`)

### Arquivos NOVOS (zero risco de merge)

- `crates/ph2d-audio-edit/src/fx/multiband.rs` · `vocoder.rs` · `granular.rs`
- `crates/ph2d-audio-edit/tests/measure_multiband.rs`
- `shells/desktop/src/audio/editor/multiband_smoke.rs` · `voice_smoke.rs`

### Contratos congelados (§6)

**Nenhum tocado.** `Effect` não é congelado (o gate `architecture_tool_contract_surface` cobre
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`, não a rack de áudio). **Zero deps novas**
→ `deny`/`audit` não têm o que ver de novo.

### LOC (medido **depois** do `fmt`)

`fx.rs` foi de 602 → **689 linhas brutas** e `fx_presets.rs` está em **596**. O gate é
*comment-aware* e os dois passam, mas ambos estão **no teto**. Quem adicionar o **43º efeito** deve
**orçar o split desde já** (candidato natural: mover os braços de `apply` para um módulo irmão,
como `warmup.rs` já fez).

---

## 6. Smoke — **já vem montado** (Enio, 2026-07-13)

`AudioSystem::new()` precisa de device de áudio e nenhum teste headless constrói um — então **o som
saindo do device é smoke-only**. Mas o clipe e a rack **não são** trabalho do Enio: a linha
sintetiza os dois (`shells/desktop/src/audio/editor/multiband_smoke.rs`).

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && PH2D_AUDIO_MULTIBAND_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

> O `-p` **não é opcional**: o workspace tem 27 binários e um `cargo run` pelado morre em
> *"could not determine which binary to run"*.

Abra a pill do **Audio Editor**. Já estará carregado:

- **O clipe** — um kick de 60 Hz a cada 0,5 s (120 BPM, quase fundo de escala) sobre um pad
  (220+330 Hz) e um shimmer (6+9 kHz) **absolutamente firmes**, 25 dB abaixo. A firmeza é o truque:
  o pad e o shimmer não se mexem sozinhos, então **todo movimento que você ouvir neles é o
  compressor abaixando-os**.
- **A rack, com o A/B pronto** — dois stages no **mesmo Ratio (todo à direita, 20:1)**:
  `[Multiband: ligado] [Compress: bypassado]`.

**O teste:** inverta o `enabled` dos dois stages (é pra isso que ele existe). No **Compress**, o pad
e o shimmer **bombeiam a 120 BPM**, um mergulho por kick. No **Multiband**, ficam parados e só o
kick é domado. Mesmos números nos dois — a comparação é entre dois desenhos, não entre dois ajustes.

> **Isto está gateado, não é promessa:** `the_smoke_clip_makes_the_plain_compressor_duck_the_highs`
> mede o movimento do agudo neste clipe exato — **fonte seca 0,000 · Compress 0,532 · Multiband
> 0,002**. Se o material deixar de expor o efeito, o gate fica vermelho antes de chegar em você.

Extras que valem 30 s:
- **Neutro:** puxe o Ratio do Multiband todo pra esquerda (1:1) — tem que ficar **byte-idêntico**.
- **Borda da seleção:** aplique numa seleção no meio do clipe e escute a emenda — não pode estalar
  (o crossover tem pre-roll; os compressores se primam sozinhos).

### 6.2 W4 — Vocoder e Granular (**smokado, OK**)

```bash
PH2D_AUDIO_VOICE_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

**Não dá para vocodar um seno** — sem vogais, a afirmação do efeito (*"as vogais sobrevivem, o pitch
não"*) é intestável: você ouviria um robô e não teria como saber se as palavras foram destruídas
junto. Então o clipe é **fala sintetizada**: buzz glotal cujo pitch **plana** (100→150 Hz, a
entoação de uma pergunta — é isso que o vocoder tem que **jogar fora**) atravessando **formantes que
se movem** entre 6 vogais (é isso que ele tem que **manter**), com consoante não-vozeada entre elas.

Rack montada: `[Vocoder Breath=0: on] [Vocoder Breath=1: bypassado] [Granular: bypassado]`.

| Stage | O que ouvir |
|---|---|
| **1** | A entoação **acha** no monotom do portador; as vogais continuam marchando. **É o robô.** |
| **2** | As mesmas vogais, **sem pitch nenhum**. É um sussurro — e é o **MESMO efeito**, um knob movido. |
| **3** | A frase **borra** numa textura. |

> Gateado: vocodado, o comb do portador dá **0,0605** contra **0,00007** de pitch remanescente da
> voz (**864×**), e as vogais continuam lá.

---

## 7. Resumo (§1.5.9)

| | |
|---|---|
| **Pronto pra integrar?** | **Sim** — 1 commit, gate batched verde, sem deps novas, sem contrato tocado |
| **Ordem** | Indiferente (não depende de outra linha) |
| **Atrito se `main` andar** | Só `KINDS: [FxKind; 40]` + a lista pinada, **se** outra linha somar efeito — **conte, não escolha** |
| **Pendências** | **Smoke do Enio** (§6) · decisão do rename `"Gate"` → `"Gate / Expander"` (§2.2) |
| **Fila restante do módulo** | W4 (vocoder/robotize/granular) · W6 (export por plataforma) · débitos dos ADRs · W7 (AI/ML, exige ADR) — ver `docs/HANDOFF_line_audio_continuacao_2026-07-13.md` §3 |

### 6.3 W6 — variantes por plataforma (**pendente**)

```bash
PH2D_AUDIO_DELIVERY_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

O clipe tem um **shimmer de 15 kHz** — que um alvo de 24 kHz **não pode representar** (Nyquist é
12 kHz) — mais uma imagem estéreo de verdade, um loop e dois markers.

1. Na seção **Delivery**: três linhas, e **as RAMs DIFEREM** (Mobile é um quarto). Uma troca de
   codec imprimiria o mesmo número três vezes.
2. **Export Set** → escolha uma pasta → **três arquivos** (`delivery-smoke.mobile.ogg`,
   `.desktop.opus`, `.console.wav`).
3. **Carregue o `.mobile.ogg` de volta:** o shimmer **sumiu** — e sumiu *limpo*, não virou um tom
   de 9 kHz (que é o que acontecia antes do fix de aliasing).
4. Só o `.console.wav` volta com o **loop e os markers**.

---

## 8. ADR-0120 — o preview de knob (o débito de perf do ADR-0117 §5)

Arrastar um knob num clipe de 3 min com seleção de 1 s custava **16,86 ms por frame** — o orçamento
inteiro de 60 fps — e **74% disso era memcpy de áudio que não mudou**. O DSP que o frame de fato
precisava: **0,17 ms**.

**A descoberta:** o ADR-0117 supôs que O(seleção) exigiria **mudar o contrato de preview/playback**.
**Não exige.** `SampleData` é `Arc<[f32]>` imutável de propósito (a thread RT o segura) — mas
`Arc::get_mut` devolve o slice se você for o **único dono**, e **recusa sozinho** se houver clone. E
dá pra ser o único dono, porque a máquina já existia: o hot-swap faz o mixer **devolver o buffer
antigo pela return ring**, e a thread de controle o solta (HR-3). **O buffer que você mandou dois
frames atrás é seu de novo.**

Dois scratches alternando ⇒ **16,86 ms → 0,27 ms (62×)**, sem mexer no contrato nem no HR-3.

### ⚠️ Para quem tocar nisto

- **Scratch velho = áudio silenciosamente ERRADO** (pior que áudio lento). Ele é chaveado em
  (buffer do head, seleção) e jogado fora se qualquer um se mover. O gate é **byte-identidade**
  contra o render completo, não *"soa igual"*.
- **Todo bail-out cai no render completo.** O caminho rápido é otimização, **nunca 2ª fonte de
  verdade** — 2 estágios audíveis, efeito de cauda, ou o frame em que o mixer ainda não devolveu.
- **O bug que os gates pegaram:** inicializar o scratch com `head.data().clone()`. Um `clone()` de
  `SampleData` **bumpa o `Arc`, não copia os dados** — o `get_mut` recusaria **para sempre**, o
  caminho rápido seria **código morto**, e **todos os outros gates continuariam verdes**. O único
  sintoma seria que os knobs estavam tão lentos quanto antes. Daí o gate
  `the_fast_path_fires_every_frame_while_the_mixer_is_holding_a_buffer` (8/8, dirigindo a
  alternância real). → memória `feedback_an_optimization_needs_a_gate_that_proves_it_fires`.
- **Não ponha barra de wall-clock na suíte do CI.** O `ci-test` compila em `opt-level = 1`: o DSP
  fica 30× mais lento e a memcpy não muda, então a barra mede o **perfil**. Só razões.

## 9. Débitos que eu NÃO fechei — e por quê (leia antes de "terminar" eles)

Três dos "débitos abertos" do handoff anterior são **cercas de Chesterton**: decisões ratificadas,
com motivo escrito, que **ainda valem**. Construí-los seria feature órfã.

| Débito | Por que a cerca fica de pé |
|---|---|
| **Seek/scrub num stream** | ADR-0118 §5: *"Fica para quando houver um consumidor real"* — o editor é **residente por construção** (o clipe está aberto). Ainda não há consumidor. |
| **Pitch ao vivo num stream** | ADR-0118 §5: *"é uma política de produtor, não de mixer"*. |
| **Toggle "Streamed" no Delivery** | ADR-0118 §5, textual: ***"Botão que não faz nada é pior que botão que falta"*** — a razão pela qual ele não foi feito. |
| **Múltiplas regiões de loop** | ADR-0119 §5: `smpl` aceita várias, **jogos usam uma**. |

**Abertos de verdade:** o rename `"Gate"` → `"Gate / Expander"` (decisão do Enio: descoberta ×
quebrar preset, que resolve por nome) · o **split de `fx.rs`** (689 linhas, no teto — o 43º efeito
tem que orçar) · **W7** (AI/ML — ADR + autorização explícita antes de qualquer linha).
