# BUGS — Módulo de Áudio (`line/audio`)

> Registro de bugs não-óbvios do módulo de áudio e suas soluções, no espírito de
> [`docs/Painter/BUGS_painter.md`](../Painter/BUGS_painter.md). Um bug por seção,
> com sintoma · investigação · causa-raiz · fix · lições. Cross-ref:
> [`docs/HANDOFF_audio_module.md`](../HANDOFF_audio_module.md) §4.

---

## Bug #1 — "meters/playhead vivos, sem som audível" (FECHADO 2026-07-08)

### Sintoma
Toca o Play Test do mixer **ou** o preview do editor: os **medidores e o playhead
se mexem** (o master mostra sinal vivo), mas **não sai som** no device. Persistiu
por várias sessões; o Enio lembrava "o mudo estava ligado em algum lugar", mas o
botão de mute do painel **não** estava ligado.

### Investigação (auditoria multiagêntica, 3 frentes)
1. **Caminho de código** — veredito **NÃO há bug**. Em `AudioRenderer::render`
   (`ph2d-audio/src/engine.rs`) o medidor do master (passo 4) e o `write_out`
   (passo 5) leem o **mesmo buffer `master`**, com o preview somado **antes** de
   ambos (passo 3b). `write_out` (`ph2d-audio/src/output.rs`) copia master→out
   corretamente; o scatter cpal em `build_stream` (`shells/desktop/src/audio.rs`)
   está correto pra estéreo e 8ch. **Se o meter mexe, o sinal chega ao device.**
   (Diagnóstico #1 do HANDOFF §4 confirmado.)
2. **Seleção de device** — o app abre `default_output_device()` **cego** (sem
   enumerar/escolher por nome). Achou 2 formas de "meter vivo, sem som" fora do
   mix: (a) o build 2ch rejeitado → fallback 8ch nativo → só FL/FR → inaudível em
   alguns 7.1 DACs; (b) o default sink errado.
3. **Pesquisa cpal/PipeWire** — o app aparece como "PipeWire ALSA
   [ph2d-host-desktop]" e **os meters mexem** → o PipeWire **já recebe** o áudio;
   a falha é **downstream** (rota/mute/volume), não no cpal. **Suspeito primário:
   a entrada salva do `module-stream-restore`** — o PipeWire/WirePlumber lembra
   **mute/volume/rota por-app** e re-aplica a cada launch, independente do volume
   interno; e **um app ALSA não consegue desfazer o próprio mute** (é server-side).

### Causa-raiz (confirmada por inspeção do estado do sistema)
`~/.local/state/wireplumber/stream-properties` tinha a entrada do app com
**`"mute":true`** (volume 1.0), re-aplicada a cada abertura:
```
Output/Audio:application.name:PipeWire ALSA [ph2d-host-desktop]
  = {"channelMap":["FL","FR"], "volume":1.0, "mute":true}
```
Era **exatamente** o "mudo salvo em algum lugar". O `channelMap` já era `["FL","FR"]`
→ o app **já abria estéreo**, então **o 7.1/multicanal era pista FALSA**.

### Fix
Desmutar a entrada salva (o `ph2d` era a **única** com `mute:true` no arquivo):
```
systemctl --user stop wireplumber                 # flush do estado em memória
sed -i 's/"mute":true/"mute":false/' ~/.local/state/wireplumber/stream-properties
systemctl --user start wireplumber                # recarrega mute:false
```
Alternativa (com o app rodando, persiste sozinho):
```
pactl list sink-inputs short                      # pega o <id> do ph2d-host-desktop
pactl set-sink-input-mute <id> 0
```
Ou **pavucontrol** → aba Playback → desmutar `ph2d-host-desktop`. Reset nuclear de
todo estado por-app salvo: apagar `~/.local/state/wireplumber/` + relogar.

### Robustez de código (não era o fix, mas ficou)
`AudioSystem::new` agora prefere um stream **estéreo** em device >2ch (tenta 2ch de
fato, com fallback pro nativo) — recomendação da pesquisa cpal, inofensiva. Commits
`a32db2fe`/`82b99fcd`/`119cfee1`.

### Lições
- **"Meter vivo, sem som" no Linux/PipeWire ≈ mute/rota salvos por-app no
  WirePlumber**, não bug de código. O medidor do master mexer **prova** que o
  sinal chega ao `write_out`/device → olhar o servidor de som, não o mix.
- Um app cpal/ALSA **não desfaz o próprio mute server-side** — é ambiente
  (pavucontrol/pactl), mas dá pra editar `stream-properties` + reiniciar o
  wireplumber.
- **Inspecione o estado real antes de teorizar:** `pactl list sinks short`
  (device 8ch?) + `grep -c '"mute":true' ~/.local/state/wireplumber/stream-properties`
  cravaram em minutos o que teoria não cravava.
- Memória: [[project-audio-multichannel-silence]].

---

## Bug #2 — "1 seg e meio para mudar o ganho" num clipe longo (FECHADO 2026-07-16)

> Decisão em [ADR-0125](../architecture/decisions/0125-audio-pricing-a-shipping-target-is-export-work-not-edit-work.md).
> Aqui fica a **saga** — as armadilhas que enganaram o diagnóstico, que é o que um ADR
> não guarda. Encadeia com a família [ADR-0117](../architecture/decisions/0117-audio-editor-memory-is-measured-not-declared.md)
> /[0120](../architecture/decisions/0120-audio-preview-is-a-buffer-you-own-not-a-buffer-you-rebuild.md)/[0124](../architecture/decisions/0124-audio-edit-is-a-range-in-time-not-a-rebuilt-clip.md).

### Sintoma
Com um clipe de ~3 min carregado, **cada clique de Gain (e toda edição comum) trava a UI
por ~1,5 s**. Faixas pequenas selecionadas não ajudavam — a lentidão não dependia do
tamanho da seleção.

### Investigação (3 camadas, cada uma refutando a anterior)
1. **A medição isolada MENTIU por omissão.** `EditClip::apply_gain` cronometrado num
   bench dava **0,008 ms** — sugeria abismo de 200.000× e "o bug não está aqui". **Dois
   enganos embutidos:** (a) o bench chamava `set_selection` e o `ml_smoke` **nunca
   seleciona** → o produto roda o caminho O(clipe), o bench rodava o O(seleção); (b) o
   bench media o motor, não **o frame do app**. Lição já catalogada:
   [[feedback_harness_reproduces_mechanism_not_context]].
2. **Primeiro alvo (real, mas menor).** Toda edição por-intervalo era O(clipe): `splice`
   remonta o buffer inteiro, `history::diff` **varre os dois buffers pra redescobrir o
   intervalo que o chamador já sabia**, e `install` reconstrói o `PeakCache` inteiro.
   Consertado ([ADR-0124](../architecture/decisions/0124-audio-edit-is-a-range-in-time-not-a-rebuilt-clip.md),
   22 ms → 0,01 ms com seleção) — mas o Enio **remediu e ainda era 1,5 s**. O motor não era
   o culpado principal.
3. **Auditoria de 8 lentes + refutação adversarial** (26 achados, 26 sobreviventes),
   medindo **o app real** headless. A conta fechou num lugar só.

### Causa-raiz
**Um clique de edição re-precificava os 3 alvos de shipping.**
`editor_publish_platforms` (`shells/desktop/src/audio/editor/platforms.rs:63`) roda no
**mesmo frame** do comando (`render_loop/mod.rs:312`, 18 linhas após `editor_apply`), e
cada `price()` faz um **`conform` real + um `cost()` que é um ENCODE real** do clipe
inteiro, na thread de UI, **pra desenhar um readout de 3 linhas** — sem gate de
visibilidade (pagava com a seção fechada). Repartição medida (1758 ms):

| item | ms | % |
|---|---:|---:|
| `cost(Opus)` do clipe inteiro, **sem teto** | ~985 | 60% |
| `conform` do Mobile (FIR 127 taps na taxa da fonte) | ~460 | 28% |
| conforms up-mix + `cost(Wav16)` | ~170 | 10% |
| o motor da edição (o "abismo" da camada 1) | ~25 | 1,5% |

**A armadilha central — uma premissa documentada e INVERTIDA:** o Vorbis tinha teto
(`MEASURE_SECS=10`), o Opus era isento, com o comentário *"Opus is bitrate-driven and
fast, so the honest number is also the cheap one"*. Medido head-to-head: **o Opus é 1,6–2,7×
MAIS LENTO que o Vorbis** — o `unsafe-libopus` é libopus **transpilado por c2rust, sem o
SIMD do C** ([ADR-0116](../architecture/decisions/0116-audio-export-opus-isolated-unsafe-crate.md)).
*"libopus é rápido em C"* não transfere. **O codec que capearam era o rápido; o isento por
ser "rápido" era o que travava.**

### Fix (ADR-0125)
1. **Tirar a precificação do frame de edição.** Gate de visibilidade (não precifica com a
   seção fechada) + quando visível, roda **fora da thread de UI** pelo padrão async
   `ph2d_editor_core::progress` (2º consumidor dele — mas **`Job`, não a barra**:
   precificar é automático e sub-segundo; o indicador honesto de um readout é o próprio
   readout mostrando `…`).
2. **O teto vale para TODO codec com perda:** os dois passam pela **mesma** `measure_capped`
   — *"so the two cannot drift on what a long clip costs to size. They did."* Opus 941 → 55 ms
   (17×), erro 0,05%; o número longo agora vem com `~` (sempre foi estimativa; agora admite).
3. **FIR do Mobile: deferido COM gatilho** (`03_o_que_falta.md` §2.3). O item 1 já o tirou
   do frame de edição, e o conserto "só compute o que sobrevive à dizimação" **muda os bytes
   de todo asset Mobile já shipado** (só há amostra morta na razão exata 0,5; em razão
   não-inteira os dois vizinhos contribuem) — é redesenho de resampler, não item de fix de perf.

**Resultado:** clique de Gain **1758 ms → 24 ms** (só o motor, e nada mais).

### Lições
- **O motor consertado dispara o custo que o esconde.** O ADR-0124 fez a edição O(seleção),
  e o ganho foi **engolido por um O(clipe) que a própria edição dispara** a jusante. Consertar
  o produtor não basta se os **consumidores** reagem a *"o clipe mudou"* com *"então refaço
  tudo"* — a causa-raiz nomeada: [[feedback_a_condition_that_enumerates_its_readers_rots]].
- **A mutação explica a longevidade:** re-isentar o Opus deixa **2 gates novos vermelhos e
  os 10 velhos verdes.** Um bug que todos os testes existentes aprovam vive um ano. O gate
  que vale é o que **falha** com o bug presente ([[reference_topic_mutation_proofs]]).
- **Comentário que afirma performance sem medir é dívida:** *"fast, so cheap"* sobre uma
  transpilação c2rust era fé, e a fé estava ao contrário. [[feedback_measure_perf_symptom_scale]].
- **Precificar é EXPORT, não edição.** A pergunta de design que resolve: *"quem está olhando,
  e isso é trabalho do frame ou do usuário pedir?"* — não *"como deixo isto mais rápido?"*.
- **Nunca um número errado como certo:** durante o cálculo o readout mostra `…` ou o valor
  velho, jamais uma estimativa vestida de exata ([[feedback_absence_gate_needs_a_presence_sibling]]:
  o gate "não precifica na edição" tem o irmão "ainda precifica certo quando aberto").
