# ADR-0125 — Precificar um shipping target é trabalho de EXPORT, não de edição

- **Status:** aceito (2026-07-16)
- **Contexto:** linha `line/audio`; sucede a família ADR-0117 / ADR-0120 / ADR-0124
- **Gatilho:** Enio, medindo o produto: *"1 seg e meio para mudar ganho"*

---

## 1. O fato

Clipe de 3 minutos (`PH2D_AUDIO_ML_SMOKE=1 PH2D_AUDIO_ML_SMOKE_SECS=180`), release, mono
48 kHz, 8.640.000 frames. Um clique de **Gain**:

```
clique de Gain              = 1562 ms
  editor_publish_platforms    1549 ms   (99,2%)   <- o culpado
    Desktop -> cost(Opus) do clipe INTEIRO   ~985 ms
    Mobile  -> conform (FIR 127 taps)        ~460 ms
    Desktop/Console -> conform (up-mix)      ~136 ms
    Console -> cost(Wav16)                    ~33 ms
  editor_publish_delivery       13 ms
  apply_gain (sem selecao, O(clipe))          25 ms   (1,5%)
```

**O trabalho que o usuário pediu era 1,5% do clique.** Os outros 98,5% eram três `conform`s e
três **encodes reais do clipe inteiro**, na thread de UI, 18 linhas depois do `editor_apply`,
para desenhar um readout de três linhas — **de uma seção que nasce FECHADA**.

## 2. Por que o cache não defendia nada

`PlatformCache` era keyed em `SampleData::version`, e **uma edição move a versão por
definição** — é isso que uma edição *é*. Então o cache acertava em todo frame **menos** o único
que importava: o clique.

O docstring dizia isso em voz alta e ninguém ouviu:

> *"A cache hit on all but the frame after the buffer actually changed."*

O frame em que o buffer muda **é** o clique. **Um cache não é um orçamento.**

## 3. A premissa invertida (ADR-0116, cobrada)

`delivery.rs` tinha teto para o Vorbis (`MEASURE_SECS = 10.0`) e **isentava o Opus**:

> *"Opus is bitrate-driven and fast, so the honest number is also the cheap one."*

Medido head-to-head, mesmo buffer, sem teto em nenhum dos dois:

| clipe | Vorbis | Opus | |
|---|---|---|---|
| 2 s | 5,2 ms | 6,7 ms | **1,3×** |
| 10 s | 17,3 ms | 33,9 ms | **2,0×** |

**O Opus é o LENTO dos dois — e era o que estava sem teto.** O mecanismo está no próprio
ADR-0116: o `unsafe-libopus` é o libopus **transpilado por c2rust, sem o SIMD do C**. *"libopus é
rápido em C"* não sobrevive à viagem; a isenção raciocinava sobre uma codebase que não linkamos.

E pelo **próprio critério do comentário** ele se condena: o Opus é linear (~5,5 ms por segundo de
áudio), então o *"five-minute stem"* que o teto do Vorbis existe pra evitar custa **~1,65 s de
Opus**.

## 4. A decisão

**Precificar é trabalho de EXPORT.** O conserto não é deixar isso mais rápido — é **tirar do
frame de edição**:

1. **Gate de visibilidade.** Não precifique o que ninguém está olhando. As rows moram numa seção
   colapsável que **ships folded** (`populate_sections`), então o caso comum passou um ano
   pagando 1,5 s para computar três strings fora da tela. A pergunta é feita ao `HeroScreen` do
   shell, que **já é dono das duas metades** (`is_panel_visible("audio_editor")` +
   `!is_collapsed(AEDIT_SEC_DELIVERY)`) — nenhuma das duas basta sozinha: o painel fica aberto
   com a seção fechada, que é o *default*, e é o caso em que o bug foi reportado.
2. **Fora da thread de UI.** `audio/editor/pricing.rs::OffThread<K,V>` — **uma** máquina de
   estados, **dois** consumidores (`platforms`, `delivery`). Duas cópias seriam duas respostas
   para *"este número é atual?"*, e no dia em que divergissem o painel imprimiria um readout
   honesto e um mentiroso, sem nada na tela que os distinguisse.
3. **Teto para todo codec com perda**, pela **mesma função** (`measure_capped`). "Como um clipe
   longo é dimensionado?" é **uma** pergunta; eram duas, e elas discordavam.

### 4.1 Debounce, e por que ele não é enfeite

Um drag de knob entrega um buffer novo **por frame** (a audition da rack). Sem `SETTLE`
(250 ms) isso é **uma thread por frame**, cada uma precificando um estado intermediário que
ninguém vai shipar. O usuário está *no meio do gesto*; não existe número que ele queira ainda.

### 4.2 Por que NÃO tem barra de progresso

Este é o **2º consumidor** de `ph2d_editor_core::progress`, e ele toma metade do padrão de
propósito: **`Job`** (o worker + o caminho de volta) e **nunca `JobQueue`** (a barra).

A doc da própria `JobQueue` diz para quem ela é — *"user-initiated, seconds-long operations"* — e
precificar não é nenhum dos dois: é **automático** e é **sub-segundo**. Uma barra piscaria na
coluna de toasts a cada nudge de knob, disputando com mensagens reais um lugar que o usuário
deveria conseguir aprender ([`progress::column_row`](../../../crates/ph2d-editor-core/src/progress.rs)).

**O indicador honesto de um readout é o readout.** Ele diz `…` enquanto trabalha, no lugar onde o
usuário já está olhando.

### 4.3 A regra que importa mais que a velocidade

Enquanto o worker roda, o readout **nunca** mostra o número velho. Depois de uma edição, o preço
que temos é do áudio de **antes** dela — publicá-lo seria **um número errado apresentado como
certo**, e ao contrário de um readout lento, ninguém jamais perceberia. `OffThread::current`
devolve `Some` **apenas para a chave exata**; o resto é `…`.

Corolário na mesma direção: `disk_exact`. O figure capeado é uma **estimativa** e imprime `~`.
Rápido **e errado** não era o trade.

### 4.4 RAM não entra nesse trade

RAM é `size_of_val` num slice: custa nada e é sempre exata. Só o *disco* precisa de um encoder,
então só o disco vira `…` — a metade do readout que pode ser honesta de graça continua honesta
de graça, no frame da edição.

## 5. Resultado

| | antes | depois |
|---|---|---|
| **clique de Gain** (thread de UI) | **1758 ms** | **24,3 ms** (`apply_gain`, e nada mais) |
| `cost(Opus)` de 180 s | 941 ms | **55 ms** (`~`, 0,05% do real) |
| `editor_publish_delivery` (Opus) | 561 ms | **33 ms** |
| precificar as 3 plataformas | 1721 ms na UI | 779 ms **num worker**, e só com a seção aberta |

O clique agora é **o `apply_gain` e nada mais** — que é O(clipe) e irredutível: não dá pra mudar
toda amostra por menos que toda amostra (ADR-0124 §"NÃO é O(seleção), de propósito").

## 6. A linhagem (e o que ela diz da próxima vez)

- **ADR-0117:** *uma edição é um INTERVALO* — no eixo da **memória**.
- **ADR-0124:** a mesma frase no eixo do **tempo**: quem está a jusante tem de ser **informado**
  do intervalo, nunca obrigado a redescobri-lo.
- **ADR-0125 (este):** a terceira face. Os consumidores aqui **não têm como ser informados** — um
  re-encode é um re-encode. Então sobram as duas respostas restantes: **não faça o que ninguém
  está olhando**, e **quando alguém estiver, não faça na thread dele**.

A pergunta que os três compartilham, e que vale a próxima vez: **quando o clipe muda, quem
responde "então refaço tudo"?** Hoje: waveform (a §2.2 do backlog), spectrogram, delivery,
platforms. Dois deles ainda respondem assim.

## 7. Consequências

- `ph2d-audio-encode::cost` ficou **O(MEASURE_SECS)** para os dois codecs com perda. Um clipe
  longo em Opus agora imprime `~` — **é uma mudança visível**, e é a correta: sempre foi uma
  estimativa que se dizia medição.
- `DeliveryCache` deixou de guardar `DeliveryCost` inteiro e guarda só `(disk_bytes, disk_exact)`;
  RAM saiu do cache porque nunca precisou estar lá.
- `editor_publish_platforms` ganhou um parâmetro (`visible: bool`). O gate
  `the_edit_frame_only_prices_when_the_delivery_section_is_open` recusa um `true` literal ali —
  sem ele, todo gate de unidade continuaria verde com o bug de volta.
- **Não feito, de propósito:** o FIR do `conform` (§2.x do backlog). Ver ali o achado que importa —
  *"só compute as amostras que sobrevivem à dizimação"* **não é byte-preserving**, e a versão que
  é, é um redesenho do resampler.
