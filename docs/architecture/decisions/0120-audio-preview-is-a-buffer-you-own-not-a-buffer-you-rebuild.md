# ADR-0120 — O preview é um buffer que você POSSUI, não um que você reconstrói

- **Status:** Proposto (aguarda ratificação do Enio) — implementado e gateado
- **Data:** 2026-07-13
- **Fecha:** ADR-0117 §5 ("preview de render verdadeiramente O(seleção)")
- **Não toca:** o contrato de preview/playback, nem o HR-3. É a descoberta central deste ADR.

---

## 1. Contexto — o número que o ADR-0117 registrou em vez de esconder

O ADR-0117 mediu a memória do editor e deixou uma pendência explícita:

> *"Mesmo depois do D2, arrastar um knob num clipe de 3 min re-renderiza um buffer de preview
> **inteiro** por frame — porque o mixer toca esse buffer, e tocar exige um `SampleData` contíguo.
> (…) Se o Enio quiser knobs fluidos em clipes longos, é o próximo ADR, e ele tem um alvo medido
> para mirar."*

Medi o alvo (`crates/ph2d-audio-edit/tests/measure_preview.rs`), num clipe de 3 min (65,9 MB) com
uma seleção de **1 segundo** — o gesto real: você seleciona um trecho e mexe num knob.

| | |
|---|---|
| Cópia do clipe inteiro (o "imposto de contiguidade") | **12,52 ms** — 74% do frame |
| **Um frame de drag de knob** (render completo) | **16,86 ms** — o orçamento **inteiro** de 60 fps |
| O DSP que esse frame de fato precisava (1 s de áudio) | **0,17 ms** |

**Três quartos de cada frame eram memcpy de áudio que não mudou**, e o trabalho útil era 1% dele.
Isto é o que a memória do projeto chama de *"difícil de ajustar" = bug de DESIGN* — pare de
calibrar, questione o modelo.

## 2. Por que não bastava "mutar o buffer"

`SampleData` é um `Arc<[f32]>` **imutável de propósito**: a thread RT o segura, e um buffer que
mudasse debaixo do mixer **rasgaria** o áudio. Não existe caminho in-place.

**A menos que você seja o único dono.** É exatamente o que `Arc::get_mut` pergunta — e ele
simplesmente **recusa** quando existe um clone. Seguro por construção, sem `unsafe`.

E dá pra **ser** o único dono, porque a máquina já estava lá: um hot-swap de preview faz o mixer
**devolver o buffer antigo pela return ring**, e a thread de controle o solta
(`AudioSystem::poll`) — precisamente porque um `free()` na thread de áudio é uma alocação rodando
ao contrário (**HR-3**). O buffer que você mandou dois frames atrás **é seu de novo**.

## 3. Decisão

**Dois buffers de scratch, alternando.** Manda A; enquanto o mixer toca A, reescreve **B** — que o
mixer devolveu no frame passado. Por frame de drag, escreve-se **só a região da seleção**.

- `SampleData::get_mut()` — `Arc::get_mut`, o único escape da imutabilidade, e ele recusa sozinho.
- `ops::in_range_warm_region` / `EditClip::render_effect_region` — tudo o que o render completo faz
  **menos o splice no clipe inteiro**: devolve a região processada, e nada mais.
- `PreviewScratch::step` no shell — a alternância, com o scratch **chaveado em (buffer do head,
  seleção)** e jogado fora no instante em que qualquer um dos dois se move.

**Nem o contrato de preview/playback nem o HR-3 se mexem um milímetro.** Essa é a parte que
justifica o ADR: o ADR-0117 supôs que O(seleção) exigiria mudar o contrato, e **não exige** — a
return ring que o ADR-0118 construiu por outro motivo já resolvia o problema.

### O resultado

| | antes | depois |
|---|---|---|
| Um frame de drag (clipe de 3 min, seleção de 1 s) | **16,86 ms** | **0,27 ms** |

**62×.** O frame deixou de ser dominado pela cópia e passou a custar o que o trabalho custa.

## 4. O que pode dar errado, e o gate de cada coisa

Um scratch só é correto se o áudio **fora da região** ainda for o do head. Mova a seleção, ou mude
um estágio a montante, e ele está **velho** — e áudio velho é áudio **silenciosamente errado**, que
é pior que áudio lento.

| Gate | O que ele impede |
|---|---|
| `the_incremental_preview_is_byte_identical_to_a_full_render` | O caminho rápido divergir do lento. **Byte a byte**, sob uma sequência de drags, movimentos de seleção e re-tunes — não *"soa igual"* |
| `the_fast_path_fires_every_frame_while_the_mixer_is_holding_a_buffer` | A otimização ser **código morto**. Dirige a alternância real, com o mixer segurando um buffer e devolvendo pela ring: **8 de 8 frames** |
| `a_buffer_the_mixer_still_holds_cannot_be_mutated` | O argumento de segurança inteiro: `get_mut` recusa enquanto qualquer clone existir |
| `it_rewrites_the_region_and_nothing_else` | O gate acima ser vacuamente verde (um scratch que nunca escreve nada passa, se o efeito for no-op) |
| `the_region_render_matches_the_region_of_the_full_render` | O pre-roll ser descartado na amostra errada |
| `measure_preview.rs` | A **premissa** apodrecer: se o DSP um dia dominar a cópia, este ADR perdeu a razão de existir e alguém descobre aqui |

**Todo bail-out cai no render completo, que sempre funciona.** O caminho rápido é uma otimização,
**nunca uma segunda fonte de verdade**.

### O bug que os gates pegaram no meio do caminho

A primeira versão inicializava o scratch com `hd.clone()`. Um `clone()` de `SampleData` **bumpa o
`Arc`, não copia os dados** — então o head continuava segurando o buffer, `get_mut` recusaria
**para sempre**, e o caminho rápido seria código morto que **nunca rodava**. Todos os outros gates
continuariam verdes e o único sintoma seria que os knobs estavam exatamente tão lentos quanto
antes. É `SampleData::map_in_place` (uma cópia de verdade, uma alocação).

## 5. Escopo — o que o caminho rápido NÃO faz (e cai no lento)

- **Mais de um estágio audível** depois do editado: o segundo age sobre a saída do primeiro, então
  cada um precisa da sua própria base.
- **Efeitos de cauda** (reverb/delay): mudam o **comprimento** do buffer; não há região a reescrever.
- **O frame em que o mixer ainda não devolveu**: `get_mut` recusa e o frame re-renderiza inteiro.

Nenhum é um caso estreito de propósito — o drag de um knob é, quase sempre, **um** estágio Plain.

## 6. Alternativas descartadas

- **Throttle do re-render** (renderizar a 20 Hz em vez de 60). Esconde o sintoma e mantém o
  desperdício; e "difícil de ajustar" é bug de design, não de calibração.
- **Processar a rack ao vivo na thread RT** (como um DAW). **Fechado por construção:** os efeitos
  do editor são explicitamente control-thread (alocam, usam `tanh`/`exp` — HR-3/HR-5 não valem lá).
- **Auditar só a seleção** (o preview toca um buffer curto). Muda a UX (você deixa de ouvir o clipe
  correndo) e quebra o contrato "o que soa é o que é mostrado e exportado".
