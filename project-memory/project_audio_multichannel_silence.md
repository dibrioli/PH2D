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

**REINCIDÊNCIA (2026-07-15) — MESMO sintoma, causa NOVA: VOLUME do sink, não mute.**
"mobile.ogg sem áudio" no editor. O meter mexia, o WirePlumber tinha ph2d `mute:false`,
o sink não estava mutado — mas o **sink default `USB Audio Speakers` estava em 24%
(−37 dB)**, e com conteúdo a −15 dB o total (~−52 dB) era inaudível. **O navegador tocava
o mesmo arquivo audível** (pista de que era roteamento/volume, não o app). Fix: `wpctl
set-volume 57 0.8` + `wpctl set-default 57`. O DAC USB tem **3 saídas** (`57` Speakers /
`56` Front Headphones / `45` S/PDIF); apps nativos (cpal/pipewire-alsa) vão pro **default**,
e se as caixas estão noutra saída OU o default está baixo, é "sem som". **Diagnóstico que
funcionou:** `wpctl status` (árvore de roteamento + volume por sink) → tocar tom em cada
saída (`paplay --device=<sink> tom.wav`) → o Enio diz qual ouve → `set-default` nela.
**Regra:** ao investigar "sem áudio", cheque **mute E volume do sink E qual sink é o
default**, não só o mute do app. E provei o lado do código com 2 gates novos que **MEDEM
a saída** (`voice::a_24k_mono_clip_is_audible_at_48k_out`,
`engine::a_24k_mono_preview_is_audible_through_the_full_renderer`) — antes, todo gate de
reprodução comparava stream-vs-resident ou o retorno de `render_add`, e dois silêncios
comparam iguais ([[feedback_absence_gate_needs_a_presence_sibling]]).

**TERCEIRA OCORRÊNCIA (2026-09-09) — a MESMA causa de 2026-07-08, de volta.** Report do
Enio: *«os sons estão vivos no audio mixer mas não chegam até mim»* + *«o mesmo problema já
aconteceu várias vezes»*. Medido: a entrada `application.name:PipeWire ALSA
[ph2d-host-desktop]` estava outra vez com **`"mute":true`** (volume 1.0, channelMap
["FL","FR"]), enquanto as ~20 entradas irmãs `ph2d_host_desktop-<hash>` (binários de teste)
estavam todas `mute:false`. O sink default (`USB Audio Speakers`) estava a 0,89 e não
mutado ⇒ **só o app estava calado**. Cura aplicada: `systemctl --user stop wireplumber` →
desmutar SÓ a linha do ph2d (a outra `mute:true` era do `Godot Engine Editor`, deliberada) →
`start`. Depois do restart o stream vivo `#56822` apareceu `Mute: no`, 100%, roteado ao
`Speaker__sink` RUNNING.

⚠️ **O padrão é `"mute":true` VOLTAR sozinho.** Não foi identificado o que o (re)mute — um
clique no mixer do Plasma sobre o stream do app, ou um `pactl set-sink-input-mute` de
qualquer origem, é gravado pelo `module-stream-restore` e vira permanente. ⇒ *tratar como
recorrente, não como incidente.*

⭐ **O INSTRUMENTO existe desde 09/09: `bash scripts/audio-mudo.sh`** (mede as 4 causas e
imprime o comando; `--curar` aplica 1..3; a 4 — qual saída tem as colunas — fica com o
Enio). ⚠️ **Ele nasceu na `line/components` e só chega ao `main` na integração** — até lá o
comando é `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && bash
scripts/audio-mudo.sh`.

⛔⛔ **E o app NÃO pode medir isto, por construção:** o mute vive no NÓ do PipeWire, a
jusante do cpal — do lado de cá o `write_out` entrega o buffer e devolve sucesso. A linha
`[audio-2d] … pico do master L… R…` (09/09) responde só METADE: pico a mexer prova que o som
CHEGA à saída, nunca que a saída o ENTREGA. *Pico vivo + silêncio = correr o script, não
depurar o mixer.*
