---
name: ""
metadata: 
  node_type: memory
  originSessionId: 6d3039ad-668d-4133-a295-f69680a93752
---

Bug recorrente do módulo de áudio: **meters/playhead mexem mas não sai som**.
Diagnóstico decisivo (HANDOFF_audio_module §4): se o meter do **Master** mexe, o
sinal chega ao `write_out` → o problema é **depois** dele (device/sistema), NÃO no
código. Auditoria multiagêntica (2026-07-08) confirmou: **nenhum bug de código** —
o meter e o `write_out` leem o mesmo buffer, o scatter cpal está correto.

**CAUSA REAL (2026-07-08):** o **WirePlumber salva mute/volume/rota por-app**
(`module-stream-restore`) em `~/.local/state/wireplumber/stream-properties`, e
re-aplica **toda vez que o app abre**, independente do volume. A entrada
`application.name:PipeWire ALSA [ph2d-host-desktop]` estava com **`"mute":true`**
(volume 1.0) → som mudo com meter vivo. **Fix aplicado:** parar wireplumber
(`systemctl --user stop wireplumber`), `sed -i 's/"mute":true/"mute":false/'` no
arquivo (o ph2d era a ÚNICA entrada mute:true), reiniciar. Alternativa p/ o Enio:
app rodando → `pactl set-sink-input-mute <idx> 0` (persiste sozinho), ou pavucontrol
→ Playback → desmutar.

**Multicanal era pista FALSA:** o `channelMap` salvo do app já era `["FL","FR"]` —
o pipewire-alsa bridge entrega estéreo e o PipeWire mapeia pro par frontal do 7.1.
(O device tem saída 7.1/8ch, mas isso NÃO causava o mudo.) A preferência por stream
2ch em `AudioSystem::new` (commits a32db2fe/82b99fcd/119cfee1) fica como robustez —
inofensiva, recomendada pela pesquisa cpal, mas não era o fix.

**Inspecionar rápido (sem o app rodando):** `grep -c '"mute":true'
~/.local/state/wireplumber/stream-properties` + `grep ph2d` no mesmo. Com o app
rodando: `pactl list sink-inputs`. O app aparece como "PipeWire ALSA [ph2d-host-desktop]".
Ver [[feedback-run-command-include-cd]].
