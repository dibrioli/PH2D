# HANDOFF — `line/audio` · Precificação fora do frame de edição (ADR-0125)

- **Data:** 2026-07-16 · **Linha:** `line/audio` (Modo L)
- **Estado:** **FECHADA**. Não integrada, não pushada, `ship.sh` não rodado (ordem do Enio).
- **ADR:** [0125](architecture/decisions/0125-audio-pricing-a-shipping-target-is-export-work-not-edit-work.md)
- **Backlog:** [`docs/Audio/03_o_que_falta.md`](Audio/03_o_que_falta.md) §2.3 (FIR, deferido c/ gatilho) e §2.4 (`apply_gain`, verificado como irredutível)

---

## 1. O bug, e o número

Enio, num clipe de 3 min: *"1 seg e meio para mudar ganho"*. **Reproduzido e confirmado** — a
auditoria de 8 lentes estava certa em cada item:

| | medido (release, fixture = ruído+tom, mono 48 kHz, 8.640.000 frames) |
|---|---|
| `editor_publish_platforms` | **1721 ms** (auditoria dizia 1549) |
| ↳ Desktop `cost(Opus)` do clipe inteiro | **941 ms** (auditoria: ~985) |
| ↳ Mobile `conform` (FIR 127 taps) | **553 ms** (auditoria: ~460) |
| ↳ Desktop+Console `conform` (up-mix) | **106 ms** (auditoria: ~136) |
| ↳ Console `cost(Wav16)` | **31 ms** (auditoria: ~33) |
| `editor_publish_delivery` (Wav16) | **12,7 ms** |
| `editor_publish_delivery` (Opus — 1 clique na seta Prev) | **561 ms** (auditoria: ~590) |
| `apply_gain` (sem seleção, O(clipe)) | **24,3 ms** — o trabalho que o usuário pediu |

**Clique = 1758 ms, e 98,5% dele era precificar plataformas que ninguém estava olhando.**

E a premissa que isentava o Opus do teto **estava invertida** — medido head-to-head, mesmo buffer,
sem teto em nenhum: **Opus é 1,3× o Vorbis a 2 s e 2,0× a 10 s**. O comentário dizia o contrário
(*"Opus is bitrate-driven and fast, so the honest number is also the cheap one"*), e era ele que
autorizava o encode sem teto. Mecanismo já estava escrito no ADR-0116: o `unsafe-libopus` é libopus
**transpilado por c2rust, sem o SIMD do C**.

## 2. O que foi feito

1. **Gate de visibilidade** (`render_loop/mod.rs`). A pergunta vai ao `HeroScreen` do shell, que
   **já é dono das duas metades**: `is_panel_visible("audio_editor")` **e**
   `!store.is_collapsed(AEDIT_SEC_DELIVERY)`. Nenhuma basta sozinha — o painel fica aberto com a
   seção **fechada**, que é o *default* (`populate_sections`) e é o caso do bug reportado. Preferido
   a um thread-local publicado pelo painel: isso custaria um frame de lag e uma 2ª cópia dum fato
   que já tem dono.
2. **`shells/desktop/src/audio/editor/pricing.rs`** — `OffThread<K,V>`: **uma** máquina de estados,
   **dois** consumidores (`platforms`, `delivery`). Debounce `SETTLE = 250 ms`, ≤1 worker em voo,
   poison de worker morto. Duas cópias seriam duas respostas p/ *"este número é atual?"*.
3. **Teto p/ todo codec com perda**, pela **mesma** função (`measure_capped`), + o comentário
   mentiroso substituído pelo número medido.
4. **RAM saiu do caminho lento**: é `size_of_val`, custa nada, é sempre exata → só o **disco** vira
   `…`. A metade do readout que pode ser honesta de graça continua honesta de graça.

### 2.1 A decisão que se afasta do briefing (e por quê)

O briefing pedia *"o 2º consumidor da `JobQueue`"*. **Tomei metade do padrão de propósito:** `Job`
(o worker + o caminho de volta), **nunca `JobQueue`** (a barra).

A doc da própria `JobQueue` diz p/ quem ela é — *"user-initiated, seconds-long operations"* — e
precificar não é nenhum dos dois: é **automático** e **sub-segundo**. Uma barra piscaria na coluna
topo-centro a cada nudge de knob, disputando com toasts reais um lugar que o usuário deveria
conseguir aprender (é a regra que o próprio `progress::column_row` documenta). **O indicador honesto
de um readout é o readout:** `…`, no lugar onde o usuário já está olhando. Continua sendo o 2º
consumidor do módulo `progress` — só não da barra.

## 3. Resultado

| | antes | depois |
|---|---|---|
| **clique de Gain** (thread de UI) | **1758 ms** | **24,3 ms** |
| `cost(Opus)` de 180 s | 941 ms | **55 ms** (`~`, erro de **0,05%**) |
| `editor_publish_delivery` (Opus) | 561 ms | **33 ms** |
| precificar as 3 plataformas | 1721 ms **na UI** | 779 ms **num worker**, só com a seção aberta |

O clique agora é o `apply_gain` **e nada mais** — O(clipe) e irredutível (§2.4 do backlog: foi
verificado, não presumido).

## 4. Gates

**Novos** (todos com irmão de PRESENÇA — "não vaza" fica verde num readout morto):

- `shells/desktop/tests/audio_pricing_is_export_work_not_edit_work.rs`
  - `the_edit_frame_only_prices_when_the_delivery_section_is_open` — **arch-gate sobre o arquivo do
    produto** (idioma do `the_z_projection_reads_the_tree_after_the_sync`). Recusa um `true` literal
    no call site e exige as duas metades. **Sem ele, todo gate de unidade fica verde com o bug de
    volta.**
  - `the_priced_rows_still_say_something_true_about_each_target` — presença.
  - 2 medições `#[ignore]` (`measure_what_a_gain_click_used_to_cost`,
    `measure_opus_against_vorbis_head_to_head`) — o número que diz se funcionou, re-rodável.
- `audio::editor::pricing::tests` (8) — **medidos por contador**, não por leitura de fonte:
  o frame que vê chave nova não trabalha · a chave parada é precificada e volta · valor stale nunca
  é publicado · drag não spawna nada · soltar o knob precifica 1× · ≤1 worker · worker morto não
  ressuscita · clear esquece.
- `ph2d-audio-encode::delivery::tests` (+3) — cap por **RATIO** (`ci-test` compila em `opt-level=1`;
  bar de wall-clock mediria o perfil, não o algoritmo) + a estimativa é honesta + short ainda é exato.

**Mutação (4 rodadas):**

| mutação | resultado |
|---|---|
| re-isentar o Opus do teto (**o bug original**) | 2 gates novos **RED**; **os 10 velhos ficam VERDES** — é exatamente por isso que viveu um ano |
| `editor_publish_platforms(true)` | arch-gate **RED** |
| gate só com `is_panel_visible` (sem a metade da SEÇÃO) | arch-gate **RED** |
| `current()` computa **inline** na thread de UI | **7/8 RED** |

O sobrevivente da 4ª é `a_key_that_holds_still_is_eventually_priced_and_the_value_comes_back` — um
gate de **presença**, e um cômputo inline *de fato* entrega o valor certo. Os gates de ausência é que
pegam. Divisão de trabalho desenhada, não gate faltando.

**Verde no fechamento:** `ph2d-audio-encode` + `ph2d-audio-edit` (198 + 8 + …) · `ph2d-host-desktop
--tests` (10 suítes) · `--features audio-ml` (check + clippy) · clippy `--all-targets` limpo ·
`rustup run 1.95 cargo fmt --all -- --check` limpo · `typos` limpo · `file_loc_caps` ok.

## 5. Aberto (com gatilho, em `03_o_que_falta.md`)

- **§2.3 — o FIR do `conform`** (Mobile, ~430-600 ms). **Deferido com razão, e o achado importa:**
  *"só compute as amostras que sobrevivem à dizimação"* **não é byte-preserving aqui**. 48k→24k é
  razão exatamente 0,5 → `frac == 0.0` exato → o vizinho ímpar é lido e multiplicado por zero
  (metade morta). Mas **em razão não-inteira os dois vizinhos contribuem de verdade** — não há
  amostra morta. "Metade do trabalho é desperdício" vale **só no Mobile, por acidente aritmético**.
  A versão que rende (sinc polyphase fundido) **muda os bytes de todo asset Mobile já shipado**: é
  redesenho de resampler, com aceitação e listening test próprios, não contrabando dentro de "deixe
  o Gain rápido". O ADR-0125 já o tirou do frame de edição. **Meio-caminho byte-preserving se
  alguém quiser só a perf:** o laço interno tem um `if` **por tap** (1,1 G branches) — partir em
  head/body/tail é byte-idêntico.
- **§2.4 — `apply_gain` sem seleção (24,3 ms):** **verificado, não presumido.** O remendo de bins já
  existente **não cobre porque não há o que cobrir**: `patch(0..frames)` percorre `b0=0..b1=bins`
  rodando o *mesmo* `bin_minmax` do `build`. É o build menos uma alocação. Irredutível.
- **Herdado, intacto:** §2.2 (o frame do knob-drag ainda reconstrói a waveform inteira — 21,9 ms,
  medido, do ADR-0120/0124).

## 6. Smoke (o que o Enio deve olhar)

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
  PH2D_AUDIO_ML_SMOKE=1 PH2D_AUDIO_ML_SMOKE_SECS=180 cargo run --release -p ph2d-host-desktop
```

1. **O bug:** abra o Audio Editor → selecione tudo (ou nada) → **Gain**. Era ~1,5 s; deve ser
   **instantâneo**. Repita algumas vezes.
2. **A honestidade (o que importa mais que a velocidade):** abra a seção **Delivery**. Os 3 targets
   aparecem como `…` por um instante e **preenchem sozinhos** (~0,8 s). Os números têm de estar
   **certos** — Mobile ~1 MB / Desktop ~3,2 MB / Console ~34,6 MB, e **Mobile ~¼ da RAM dos outros**
   (é o único que conforma o áudio). Um readout que mostrasse bytes errados em silêncio seria pior
   que a lentidão que consertamos.
3. **O `~`:** com a seção aberta, cicle o codec até **Opus**. O tamanho vem com `~` — **é mudança
   visível e é a correta**: sempre foi uma estimativa; agora ela admite.
4. **O drag:** arraste um knob da rack com o Delivery aberto. O readout fica em `…` **enquanto você
   arrasta** e resolve quando você solta (debounce). Nada trava.
5. **Com o Delivery FECHADO** (o default): Gain instantâneo e **zero** trabalho de precificação.

## 7. Commits

Ver `git log --oneline main..HEAD` — todos com o trailer `Co-Authored-By`.
