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
