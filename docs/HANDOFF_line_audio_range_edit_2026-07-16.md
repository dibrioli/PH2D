# Handoff — `line/audio`: a edição por-intervalo virou O(seleção) (ADR-0124)

> **Data:** 2026-07-16 · **Worktree:** `Worktrees/line-audio` · **Branch:** `line/audio`
> **Estado:** fechado, **não integrado, não pushado** (ordem do Enio).
> **Commits desta fatia:** `5583fffb` · `ef568856` · `2ac31ffd`
> ADR: [`0124`](architecture/decisions/0124-audio-a-range-edit-must-be-told-its-range.md) ·
> backlog do módulo: [`docs/Audio/03_o_que_falta.md`](Audio/03_o_que_falta.md)

## 1. O bug, e a medição (reproduzida antes de qualquer código)

> *"Com esse áudio grande, operações comuns (como aumentar o ganho) de faixas pequenas selecionadas
> se tornaram lentas. Aqui tudo deve ficar em tempo real."* — Enio

**A divergência de 2× do briefing está explicada, não ignorada:** o fixture relatado é **mono**
(34,5 MB = 180 s × 48 kHz × 4 B, como a própria seção "causa" do briefing diz). Medindo mono eu
reproduzo 22,37 ms contra os 21,59 ms relatados; em stereo dá 46,1 ms. Mesmo código, o dobro das
amostras, o dobro do tempo. Os gates usam o fixture mono, então antes/depois é comparável.

| pass | 180 s mono | depois |
|---|---|---|
| `ops::in_range` → `splice` (remonta o buffer) | 7,78 ms | — |
| `history::diff` (varre os DOIS buffers) | 2,12 ms | — |
| `PeakCache::build` (reconstrói a waveform) | 10,81 ms | — |
| **`apply_gain`, seleção de 100 ms** | **22,37 ms** | **0,011 ms** |

```text
ANTES  seleção FIXA (100 ms), clipe crescendo:   4s 0,76 | 30s 5,77 | 60s 12,02 | 180s 22,37 ms
       clipe FIXO (180 s), seleção 1000×:       10ms 22,4 | 100ms 22,4 | 1s 22,4 | 10s 22,4 ms
DEPOIS seleção FIXA (100 ms), clipe crescendo:   4s 0,010 | 30s 0,008 | 60s 0,008 | 180s 0,011 ms
       clipe FIXO (180 s), seleção 1000×:       10ms 0,001 | 100ms 0,008 | 1s 0,103 | 10s 1,023 ms
```

O custo agora **acompanha a seleção e mais nada** — que era o pedido.

## 2. O que mudou

`EditClip::edit_range(r, op)` é o funil: entrega a região ao `op` (que vê byte-a-byte o que via
antes) e então (1) escreve onde ela está — `SampleData::get_mut`, ADR-0120 — (2) **conta o range** ao
histórico (`History::push_rewrite`) e (3) remenda só os bins que o range toca (`PeakCache::patch`).

Rotas: gain · normalize peak/LUFS · reverse · invert · remove-DC · fade · silence · **Apply da rack**
(este por `render_effect_region`, que preserva o **warm-up** — região sem pré-roll estala na borda).

**Steps continuam nascendo num lugar só** (`step_for`): o `diff` entrega o buffer inteiro porque não
foi informado do range; o `push_rewrite` entrega o range porque foi. Um step informado é bit-a-bit o
step que o diff teria achado — é o que torna a promessa do chamador verificável.

## 3. ⚠️ A armadilha que quase virou bug mudo (leia antes de mexer aqui)

Seis caches do shell identificavam um buffer pelo **ENDEREÇO**, cada um repetindo o mesmo comentário:
*"SampleData é um Arc imutável, buffer novo = ponteiro novo, e toda edição entrega um diferente."*

**Escrever no lugar falsifica exatamente essa frase.** Spectrogram desenharia a waveform pré-edição,
o Delivery precificaria os bytes pré-edição, o Platforms o conform pré-edição, o mono view tocaria o
downmix pré-edição, o stale-check do AI Denoise chamaria de fresco um resultado velho — **e nada
pareceria quebrado**.

Fix: **`SampleData::version() -> BufferVersion`**. Pergunte a ele, **nunca** a `samples().as_ptr()`.
O `get_mut` o bumpa porque é *precisamente e unicamente* a operação que muda o conteúdo sem mover o
endereço; ele **sonda antes de bumpar** (versão que anda num write recusado invalida todo cache à
toa). O `samples_mut` era um clone byte-idêntico do `get_mut` com outro nome → agora **delega**.

## 4. Gates (todos verdes)

| gate | o que prova |
|---|---|
| `measure_range_edit.rs` | **o bug**: mesma seleção, clipe 8× maior → **0,99×** (antes ~8×). Bar é **RATIO**: `ci-test` compila em `opt-level=1`, bar de wall-clock mediria o *perfil* |
| `measure_range_edit_alloc.rs` | a mesma alegação **sem flakiness**: dhat, 0,073 MB **idêntico** nos dois clipes |
| `a_range_edit_is_the_same_edit.rs` (8 testes) | byte-identidade das 2 rotas (10 ops × 6 seleções × mono/stereo) · undo/redo vs oráculo de snapshots · no-op não custa step · `patch`==`build` · versão |
| `ph2d-audio` buffer (3 testes) | bump / recusa-não-bumpa / uma-porta-só |

**`a_sole_owner_writes_the_range_where_it_lies` se pagou na hora.** A suíte foi escrita com
`EditClip::new(data.clone())` no fixture — e `clone()` **bumpa o Arc**, então o *teste* era o 2º dono,
`get_mut` recusava, e todo "fast vs slow" era **o caminho lento contra ele mesmo: verde, sobre uma
otimização que nunca rodou**. É a armadilha que o ADR-0120 documentou, pregada no ADR que o cita.
Uma 2ª instância estava escondida no oráculo de undo (`data().clone()` vivo durante a edição).
**Se você tocar nestes testes: use `map_in_place` (cópia), nunca `clone` (bump).**

Mutação — cada uma cai no gate que nomeia a alegação:

| mutação | RED |
|---|---|
| range start errado no histórico (`lo`→`lo+ch`) | os 2 gates de undo |
| range um frame curto demais | os 2 gates de undo |
| range vazio pro peak cache | fast-vs-slow (waveform) |
| fast path nunca dispara | ratio + alloc + 3 de correção |
| peaks reconstruídos em vez de remendados | ratio + alloc |
| `get_mut` para de bumpar a versão | in-place-moves-the-version |
| versão bumpa num write **recusado** | *(sobreviveu → gate novo)* |

## 5. NÃO é O(seleção) — de propósito

- **Clipe inteiro:** irredutível (não dá pra mudar toda amostra por menos que toda amostra).
- **Edições que MOVEM áudio** (trim/delete/paste/force-mono): todo frame depois do corte muda de
  índice, então o diff e o rebuild da waveform são trabalho **honesto**. Por isso o `diff` **fica**.
- **Edição com o mixer tocando o clipe:** buffer compartilhado → `get_mut` recusa → splice. Não é
  wart, é o HR-3. E o fallback **É** o caminho antigo — não há 2ª implementação pra divergir.

## 6. Aberto

- **O frame do knob-drag ainda reconstrói a waveform inteira — medido, 21,9 ms/frame** (stereo 3 min).
  `PreviewScratch::step` termina em `EditClip::new(buf.clone())`, e `EditClip::new` **é**
  `PeakCache::build`. **O ganho de 62× do ADR-0120 nunca chegou ao produto**: a medição dele
  (`measure_preview.rs`) escreve a região direto e nunca chama o `step`. Deixado **nomeado** e não
  consertado às pressas: o fix quer `[Option<EditClip>; 2]` no scratch + uma API "reescreva esta
  região **sem passo de undo**" — decisão de superfície do ADR-0120, com gates próprios e sutis (a
  dança de posse dos 2 slots com o mixer). Detalhe: [`03_o_que_falta.md`](Audio/03_o_que_falta.md) §2.2.
- `ph2d-audio-edit` **não tem contrato congelado** (nenhum gate capeia a superfície). `version()` é
  API pública nova num tipo foundational — considerar quando essa superfície for capeada.
- Resto da fila do módulo: `03_o_que_falta.md` §2.3–2.5.

## 7. Smoke sugerido (o que só o Enio pode fazer)

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && cargo run --release -p ph2d-host-desktop
```

Carregue um clipe longo (3 min), selecione ~100 ms, e martele **Gain +/-**: deve ser instantâneo e
**não piorar** com clipe maior. Depois **Ctrl+Z/Ctrl+Shift+Z** várias vezes (o undo é o ponto mais
perigoso deste refactor). Confira também que a **waveform** e o **Delivery** acompanham a edição —
são eles que ficariam mostrando o áudio pré-edição se a `version()` estivesse errada. Com **Play**
tocando, a mesma edição roda pelo caminho lento (correto, só não rápido).
