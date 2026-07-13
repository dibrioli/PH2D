# Handoff de integração — `line/audio-w3` (DIRETRIZ §1.5.9)

> **Escopo:** a cauda do W3 do plano de áudio (`docs/Audio/02_plano_implementacao_completo.md` §7).
> **Estado:** linha fechada, gate batched verde, **1 commit**. **Pendente: smoke do Enio.**

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/audio-w3` (worktree `Worktrees/line-audio/`) |
| **Base** | `main` @ `44f89ad7` |
| **Commits** | 1 — `feat(audio): Multiband (40o efeito) …` |
| **Gate batched** | `nextest --workspace` **6590/6590** · `clippy --all-targets` exit 0 · `fmt --check` · `typos` · `machete` — todos limpos |

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

## 3. Os dois achados de DSP (o valor real desta linha)

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
| `shells/desktop/src/audio/fx_params_table.rs` | `KINDS: [FxKind; 39]` → **`; 40]`** + 1 row | ⚠️ **É um NÚMERO QUE SOMA** — ver abaixo |
| `shells/desktop/src/audio/fx_param_specs.rs` | `static MULTIBAND` novo | Nenhum |
| `shells/desktop/src/audio/fx_params/tests.rs` | lista pinada +`"Multiband"` | ⚠️ idem |

### ⚠️ `KINDS: [FxKind; 40]` é um número que SOMA entre linhas

Se outra linha também adicionar um efeito, o merge textual vai apresentar `40` de um lado e `40`
do outro — e **o valor certo é 41**, que não existe em nenhum dos dois lados do conflito.
**Conte, não escolha.** O mesmo vale para a lista pinada em `the_kind_table_is_the_rack_layout`:
as duas entradas novas precisam existir, e o teste é quem prova.
(`project-memory/feedback_numbers_that_sum_across_lines_count_dont_pick.md`)

### Arquivos NOVOS (zero risco de merge)

- `crates/ph2d-audio-edit/src/fx/multiband.rs`
- `crates/ph2d-audio-edit/tests/measure_multiband.rs`

### Contratos congelados (§6)

**Nenhum tocado.** `Effect` não é congelado (o gate `architecture_tool_contract_surface` cobre
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`, não a rack de áudio). **Zero deps novas**
→ `deny`/`audit` não têm o que ver de novo.

### LOC (medido **depois** do `fmt`)

`fx.rs` foi de 602 → **633 linhas brutas** — o gate é *comment-aware* e passa, mas o arquivo está
**perto do teto**. Quem adicionar o 41º efeito deve **orçar o split** (o candidato natural é mover
os braços de `apply` para um módulo irmão, como `warmup.rs` já fez).

---

## 6. Smoke — **já vem montado** (Enio, 2026-07-13)

`AudioSystem::new()` precisa de device de áudio e nenhum teste headless constrói um — então **o som
saindo do device é smoke-only**. Mas o clipe e a rack **não são** trabalho do Enio: a linha
sintetiza os dois (`shells/desktop/src/audio/editor/multiband_smoke.rs`).

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && PH2D_AUDIO_MULTIBAND_SMOKE=1 cargo run --release
```

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

---

## 7. Resumo (§1.5.9)

| | |
|---|---|
| **Pronto pra integrar?** | **Sim** — 1 commit, gate batched verde, sem deps novas, sem contrato tocado |
| **Ordem** | Indiferente (não depende de outra linha) |
| **Atrito se `main` andar** | Só `KINDS: [FxKind; 40]` + a lista pinada, **se** outra linha somar efeito — **conte, não escolha** |
| **Pendências** | **Smoke do Enio** (§6) · decisão do rename `"Gate"` → `"Gate / Expander"` (§2.2) |
| **Fila restante do módulo** | W4 (vocoder/robotize/granular) · W6 (export por plataforma) · débitos dos ADRs · W7 (AI/ML, exige ADR) — ver `docs/HANDOFF_line_audio_continuacao_2026-07-13.md` §3 |
