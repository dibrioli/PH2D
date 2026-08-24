---
name: feedback-a-silent-output-channel-can-be-muted-outside-the-process
description: Saida muda com TODOS os elos internos verdes = o elo partido esta' fora do processo (mute por-aplicacao do PipeWire, gravado em disco pelo nome da app)
metadata:
  type: feedback
---

**Uma saída muda cujos elos INTERNOS estão todos verdes tem o elo partido FORA do processo** — e
nenhum gate deste repo o alcança.

Caso medido (2026-08-23, primeiro smoke do som de UI / D1): silêncio total. O
`PH2D_UI_SOUND_DIAG=1` interrogava os quatro elos que conhecia — preferência `true`, dispositivo
`true`, voz alocada, bus SFX — e os quatro estavam verdes. O controlo positivo `PH2D_AUDIO_SMOKE=1`
(440 Hz, 600 ms, ganho 0,4 — 17× mais longo e o dobro do ganho do blip) era **igualmente mudo**, o
que já excluía a hipótese confortável de *"o meu som é curto/baixo demais"*.

O elo partido era o **mixer por-aplicação do servidor de som**:

```text
Sink Input #29923 · application.name = "PipeWire ALSA [ph2d-host-desktop]"
Mute: yes
module-stream-restore.id = "sink-input-by-application-name:PipeWire ALSA [ph2d-host-desktop]"
```

com `"mute":true` **gravado em disco** em `~/.local/state/wireplumber/stream-properties`, indexado
pelo **NOME da aplicação**. Um mute dado uma vez sobrevive a todo `cargo run` seguinte, a builds
novos e a árvores novas — a chave é o nome, não o binário. (As entradas irmãs
`ph2d_host_desktop-<hash>`, dos binários transitórios de teste, estavam todas `mute:false`: só a
canónica estava muda, que é exactamente a que um smoke usa.)

**Why:** o sintoma de um canal mudo fora do processo é **indistinguível** do sintoma de "a feature
não funciona", e a suíte inteira deste repo cobre apenas o nosso lado do canal — *um gate verde
sobre um canal mudo continua verde*. Pior: o instrumento que eu tinha construído para separar as
causas (`PH2D_UI_SOUND_DIAG`) **enumerava a cadeia** e omitia este elo, o que transformou o
diagnóstico numa confirmação repetida de quatro verdades irrelevantes. `cpal` não expõe
volume/mute por-stream — é limite de fronteira, não lacuna de implementação.

**How to apply:**
1. Antes de suspeitar do código, **conte os elos do canal até ao hardware** e pergunte quantos deles
   o processo consegue medir. Os que ele não medir entram no diagnóstico como **ponteiro impresso**,
   nunca como silêncio (é o que o `ui_sound.rs` faz hoje, uma vez por processo).
2. Áudio nesta máquina: `pactl list sink-inputs | grep -E 'application.name|Mute:'` **com o app
   aberto**; cura `pactl set-sink-input-mute <id> 0` (o WirePlumber grava sozinho).
3. Um controlo positivo que use o **mesmo canal** e seja muito mais forte (600 ms @ 0,4 contra 35 ms
   @ 0,18) separa "cadeia partida" de "o meu sinal é fraco" numa corrida só — e foi ele que apontou
   para fora. Irmã de [[feedback-a-new-features-gate-can-expose-a-pre-existing-bug-check-the-control-first]].
4. ⚠️ O mesmo desenho vale para qualquer saída atravessando processo: notificação, clipboard,
   impressão, GPU offscreen. Ver também
   [[feedback-a-rule-only-exists-if-it-is-on-the-path-of-who-executes-it]].
