# HANDOFF DE INTEGRAÇÃO — linha `line/audio` (W4 fechado)

> **DIRETRIZ §1.5.9.** A linha **fechou, comitou local e PAROU**. Não integra, não faz ship,
> não pusha (§0.7 — Enio-only, via agente integrador dedicado).
> Tracker do módulo: [`HANDOFF_audio_module.md`](HANDOFF_audio_module.md) ·
> Continuação: [`HANDOFF_audio_line_continuation.md`](HANDOFF_audio_line_continuation.md).

## ⚠️ O QUE O INTEGRADOR PRECISA SABER ANTES DE TUDO

1. **ZERO dep nova.** Nenhum `Cargo.toml`/`Cargo.lock` tocado. Nada de `machete`/`deny` novo.
2. **ZERO foundational tocado.** Todo o diff vive em `crates/ph2d-audio-edit/` +
   `shells/desktop/src/audio/` — a pasta do módulo. Sem `editor-core`, sem `tokens`, sem `ph2d-core`.
3. **ZERO contrato congelado encostado** (CLAUDE.md §6): `NodeOp`/`OpResolver`/`NodeManifest`,
   `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`, `VectorOp` — nenhum foi lido nem escrito.
4. **Um arquivo foi DELETADO:** `crates/ph2d-audio-edit/src/fx/pitch.rs` (motor granular,
   substituído por `fx/wsola.rs`). É o único delete. Nenhuma outra linha o consumia (era
   `pub(super)` dentro de `fx`).

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/audio` |
| **merge-base com `main`** | `3805f650` (HEAD do main quando a jornada abriu) |
| **HEAD da linha** | o topo de `line/audio` (= o commit de docs deste handoff) |
| **Commits à frente do main** | **2**: `64fcf4d7` (todo o codigo do W4) + o commit de docs |
| **Fast-forward puro?** | **SIM** — o main não avançou desde o merge-base |
| **Árvore** | limpa |
| **Diff** | 12 arquivos · +1255 / −161 |

Se o main tiver avançado quando você rodar: **rebase e re-rode o gate (§5)**. O risco de
conflito é baixíssimo — o diff não sai da pasta do módulo.

---

## 2. O que landou

Fecha o **W4** (o que restava dele) **sem dep nova**. Rack **34 → 37 efeitos**,
presets **15 → 21**.

### 2.1 O bug que a jornada achou — o pitch shifter estava DESAFINADO

Não estava no plano. Apareceu quando o Harmonizer (que toca **acordes**) expôs o que uma
voz de monstro escondia.

O motor granular de 2 taps re-emendava a onda a cada wrap de grão. Como o grão tem tamanho
**fixo**, o erro de fase na emenda era **o mesmo toda vez** — e um degrau de fase constante
por período de grão não é borrão, é **desvio de frequência**:

```
Δf = frac(f_in · GRAIN / SR) · |rate| · SR / GRAIN
```

| shift | alvo | previsto pela fórmula | medido |
|---|---|---|---|
| +4 st | 378.0 Hz | −4.87 Hz | **−5.0** (−23 cents) |
| +7 st | 449.5 Hz | −9.34 Hz | **−9.5** (−37 cents) |
| +12 st | 600.0 Hz | −18.75 Hz | **−18.5** (−54 cents) |
| −5 st | 224.7 Hz | +4.70 Hz | **+5.0** (+38 cents) |

Quatro de quatro. A nota saía **baixa**, e pior quanto maior o shift.

**Fix:** `fx/wsola.rs` (Verhelst & Roelands, ICASSP 1993) — overlap-add que, para cada grão,
**procura o trecho que melhor continua o grão já assentado**. A emenda cai em ondas casadas,
não numa fase arbitrária → sem degrau, sem viés. Pitch shift = reamostrar (move pitch **e**
formantes) + esticar de volta por WSOLA (não mexe em nenhum dos dois), o que **preserva o
caráter documentado** do `PitchShift` (formantes viajam junto = chipmunk/monstro — a cerca de
Chesterton fica de pé). Ganhou de brinde um **filtro anti-alias** no shift pra cima, que o
granular não tinha.

**Por que o teste antigo não pegou:** ele media taxa de cruzamento por zero com folga
(`up > dry * 1.6` para uma oitava, que deveria dar 2.0) — 1.94× passava. A asserção nova
(`the_shifted_note_lands_in_tune`) mede em **cents** e exige < 10 em 6 intervalos.

### 2.2 Efeitos novos (3)

Núcleo compartilhado **`fx/lpc.rs`** (autocorrelação + Levinson–Durbin, f64): o MESMO ajuste
AR que separa um clique do sinal separa o trato vocal do pitch.

| Efeito | Arquivo | Referência |
|---|---|---|
| **De-Click** (a única ferramenta de **reparo** da rack) | `fx/declick.rs` | Godsill & Rayner, *Digital Audio Restoration* cap. 5; interpolação LSAR de Janssen/Veldhuis/Vries (1986) |
| **Formant Shift** (move o trato vocal, **não** o pitch) | `fx/formant.rs` | Modelo fonte-filtro (Fant 1960); warp do envelope via reamostragem da resposta impulsiva. Sem FFT |
| **Harmonizer** (2 vozes afinadas) | `fx/harmonize.rs` | 2× WSOLA + blend convexo |

### 2.3 Presets (6 novos)

`Voice EQ` · `Whisper` · `Shout` (os do plano W4) + `Restore` · `Giant` · `Choir` — estes três
são **demos de 1 clique** dos efeitos novos, pro smoke não depender de montar cadeia na mão.

### 2.4 A probe dos gates da rack agora carrega um CLIQUE

`shells/desktop/src/audio/fx_params.rs::probe()`. Uma ferramenta de **reparo** só deixa marca
em **dano** — áudio íntegro não consegue testar uma. Sem isso o gate
`turning_an_arming_knob_wakes_the_effect_up` pediria pro de-clicker se provar num sinal sem
defeito, e o único jeito de passar seria **borrar áudio que nunca esteve quebrado** — o bug que
o próprio `the_undamaged_audio_survives_untouched` proíbe.

---

## 3. Símbolos novos (o que grepar por colisão)

Nenhum é foundational; todos vivem dentro do módulo de áudio. Listados porque a regra manda.

- **`Effect` (enum, `ph2d-audio-edit`)** — 3 variantes **apendadas**: `FormantShift`,
  `Harmonizer`, `DeClick`. Enum interno da crate, **não serializado**, sem cap/gate.
- **Módulos novos** em `crates/ph2d-audio-edit/src/fx/`: `lpc`, `declick`, `formant`,
  `harmonize`, `wsola`. **Removido:** `pitch`.
- **`KINDS`** (`shells/desktop/src/audio/fx_params_table.rs`): **34 → 37**. `De-Click` entrou no
  **meio** (índice 10, junto do De-Esser/De-Plosive); `Formant Shift` e `Harmonizer` no fim.
  Inserir no meio é seguro **por construção**: presets e arquivos de usuário são chaveados por
  **nome**, nunca por índice (o módulo `fx_presets` documenta isso).
- **`FACTORY`** (`fx_presets.rs`): **15 → 21**.
- **Specs novas** em `fx_param_specs.rs`: `FORMANT_SHIFT`, `HARMONIZER`, `DE_CLICK`.

---

## 4. O que só o `ship.sh` pega (e o que eu já rodei)

Rodei localmente, **verde**: `fmt --check` (pin 1.95) · `clippy --all-targets` (**0 warnings**) ·
`typos` · testes das 4 crates.

**Não rodei** (não mexi em deps, então não deve mudar nada): `cargo deny`, `cargo machete`,
`nextest-impacted`. Ver [[project_integrator_ship_catches_latents_budget_iterations]] — o gate
per-linha não é o ship; orce 2-4 iterações se ele vier vermelho por dívida pré-existente.

---

## 5. Gate (reproduza no tip rebaseado)

```bash
cargo test -p ph2d-audio-edit -p ph2d-panel-audio-editor   # 145 + 20 + 29
cargo test -p ph2d-host-desktop --tests                     # 284 (+ suites); ⚠️ --tests, não --bins
cargo test -p ph2d-editor-core --tests                      # 32 arch-gates
cargo clippy --all-targets -p ph2d-audio-edit -p ph2d-host-desktop
rustup run 1.95 cargo fmt --all -- --check
typos
```

---

## 6. O que smoke-testar (nada disso foi smokado pelo Enio ainda)

**Arquivo pronto:** `/home/enio/ph2d_audio_smoke.wav` — 6 s, 48 kHz, estéreo. Voz sintética
(trem glotal + 3 formantes, melodia 165→196→220→196→165 Hz, L≠R), zumbido de 60 Hz, 5
transientes espaçados e **6 cliques reais**.

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && cargo run --release -p ph2d-host-desktop --features panel-audio-editor
```

Carregue o WAV e rode os presets, nesta ordem:

1. **Restore** → os 6 cliques (0.83 · 1.71 · 2.94 · 3.62 · 4.35 · 5.11 s) somem, **e a voz
   sob eles fica intacta**. É o par que interessa: reparo sem borrão.
2. **Giant** → a mesma melodia, **no mesmo tom**, saindo de uma cabeça muito maior. É o efeito
   que nenhum pitch shifter consegue fingir — se o tom mudar, o Formant Shift está errado.
3. **Choir** → um acorde maior. **Ouça se afina** — é o que o fix do WSOLA comprou.
4. **Pitch Shift** em +12 st → uma oitava que soa como oitava (antes saía 54 cents baixa).
5. **Whisper** / **Shout** / **Voice EQ** → caráter de voz.

**Custo de Apply** (medido, release, clipe de 6 s estéreo): De-Click 14 ms · Pitch Shift 90 ms ·
Formant Shift 136 ms · Harmonizer 177 ms. Nada perceptível.

---

## 7. Dívida que eu deixo anotada

- **`fx.rs` está em 666/700** — o **próximo** efeito que ganhar variante ali **exige split**
  (o `warmup_frames` é o candidato natural: ~70 linhas, sai limpo pra um `fx/warmup.rs` irmão).
  `fx/dynamics.rs` segue em 662/700 (dívida herdada da jornada passada).
- `fx_presets.rs` em 567/600.
