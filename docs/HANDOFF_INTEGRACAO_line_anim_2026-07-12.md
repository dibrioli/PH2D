# HANDOFF DE INTEGRAÇÃO — `line/anim` (2026-07-12)

> **Para:** o **agente integrador** (e o Enio, para as 2 decisões abertas em §6).
> **De:** o agente da linha `line/anim`. **Etapa:** ETAPA 5 da fila (refinamentos do fit do record).
> **Estado:** linha **PRONTA**. Não integrei e não fiz ship (CLAUDE.md §0.7).

---

## §1 — Cabeçalho

| | |
|---|---|
| **Branch** | `line/anim` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim/` |
| **Base** | `3805f650` (main de 2026-07-12) — rebasada, sem dívida de merge |
| **HEAD** | `8874a2b7` — **1 commit** de código (+ `39beaf8c`, só o handoff de continuação) |
| **Gate** | `nextest` **639/639** (`ph2d-anim` · `ph2d-timeline` · `ph2d-panel-timeline` · `ph2d-host-desktop`) · clippy `--all-targets` **0 warnings** · `fmt` (rustup 1.95) · LOC caps ok · `typos` ok |
| **Contratos congelados** | **NENHUM tocado** (`Tool`/`NodeOp`/`PanelEvent`/vector-doc intactos) |
| **`DOC_VERSION` / `SCHEMA_VERSION`** | **NÃO mudaram** (nada novo é serializado) |

---

## §2 — O que a linha entrega (2 de 3 itens da ETAPA 5)

O fit do record recebia `(tempo, valor)` e **nada mais**. Dois canais carregam estrutura que os números
sozinhos não mostram — e ignorá-la produz curva errada que **tolerância nenhuma pega**.

### 2.1 — Unwrap de rotação (era um bug CATASTRÓFICO, não um refinamento)

O handoff anterior supunha que rotação *poderia* embrulhar. **Embrulha, e eu achei a fonte:**

> [`crates/ph2d-editor-core/src/gizmo/transform.rs:276-280`](../crates/ph2d-editor-core/src/gizmo/transform.rs#L276-L280)
> ```rust
> let start_angle = (…).atan2(…);   // (-π, π]
> let now_angle   = (…).atan2(…);   // (-π, π]
> let mut rotation = drag.start_transform.rotation + (now_angle - start_angle);
> ```
> Os dois `atan2` vivem em `(-π, π]`, então `Transform.rotation` **salta 2π num frame** quando o cursor
> cruza o corte de ramo. Na tela é invisível (rotação é mod 2π) — por isso ninguém viu.

**Medido no caminho de produção, ANTES do fix:** um giro de 2 voltas (4π = **12.57 rad**) reconstruía
com span de **0.00 rad** — o giro simplesmente **sumia**, virava 11 keys de dente-de-serra.

`ph2d_anim::unwrap_angles` recompõe o giro contínuo **exatamente**: o salto é exatamente 2π e a mão nunca
gira meia volta entre dois frames (a 60fps isso seria 30 rev/s).

### 2.2 — Clamp de limite (opacidade)

Opacidade é `[0, 1]`. A cúbica de mínimos quadrados por um fade que assenta **NO** limite estourava para
**1.0028** e **−0.0040**. O runtime clampa o display, mas o **graph editor desenha a curva**.

O limite viaja com o canal e o fit clampa os **4** pontos de controle do segmento → por casco convexo a
curva inteira obedece, **exatamente**. **Os keys também são clampados** — gravação tem tremor, então um
fade que descansa em 1.0 tem amostras em 1.004 (foi o que me pegou: "os endpoints são amostras, já dentro
do limite" é **falso**).

### 2.3 — Onde a semântica mora

Módulo **irmão novo** `crates/ph2d-anim/src/curve_prep.rs` (isolamento, DIRETIVA §1 — não engordei
`curve_fit.rs`). `PropKind::fit_channel()` (ph2d-timeline) faz o mapeamento; o **fit segue rotina numérica
pura** que não sabe o que é um sprite.

---

## §3 — Símbolos NOVOS (para o integrador detectar colisão)

Nenhum id numérico, nenhum discriminante, nenhum variant de enum. **Zero risco de colisão de valor.**

| símbolo | onde | nota |
|---|---|---|
| `curve_prep` (módulo) | `ph2d-anim/src/` | módulo irmão novo |
| `FitChannel { angular: bool, bounds: Option<(f64,f64)> }` | `ph2d-anim::curve_prep` | **estende por CAMPO** (append-only; `default()` = não faz nada, então um campo novo mantém todo caller byte-idêntico) |
| `FitChannel::{LINEAR, ANGLE, bounded}` | idem | consts |
| `unwrap_angles`, `prepare` | idem | `pub` |
| `PropKind::fit_channel()` | `ph2d-timeline/src/prop.rs` | método **inerente novo**, discriminantes intactos |

**Assinaturas MUDADAS** (o integrador vê isto se outra linha tocou os mesmos arquivos):
- `fit_fcurve(samples, tol)` → `fit_fcurve(samples, tol, bounds)`
- `fit_fcurve_at(samples, times)` → `fit_fcurve_at(samples, times, bounds)`
- `Track::simplify_range(_at)(…)` ganham `channel: FitChannel`
- `Track::range_samples(…)` ganha `channel`; `RangeSamples` deixa de ser alias de tupla e vira **struct** `{ids, samples}`
- `autokey_pass::value_tol` passa a receber as amostras **PREPARADAS**, não o `RecSpan` cru — a extensão
  crua de um canal angular é **uma volta embrulhada (~2π)** por mais voltas que tenha dado de verdade, e
  uma tolerância derivada dela seria absurdamente apertada para a curva desembrulhada.

---

## §4 — O 3º item foi DEFERIDO, com dados (não por preguiça)

**O pré-passe de quina não entrou.** Construí **quatro** detectores; **todos os quatro fabricam quinas no
meio de gestos SUAVES** assim que a entrada tem tremor de mão realista.

**Medição sobre 200 seeds de ruído** (o melhor detector, tremor de 2% do range — normal para mouse):

| gravação | falso-positivo |
|---|---|
| Senoide lenta + tremor 2% | **100% dos seeds** — 2467 quinas fantasmas |
| Senoide rápida + tremor 2% | **100%** — 1368 |
| Ease exponencial + tremor 2% | **100%** — 450 |
| Reta + tremor 10% | 0% ✓ |

**Por que não é problema de ajuste:** na escala da amostra, um gesto suave rápido e um cusp diferem só de
um jeito que o tremor mascara — e a estimativa de ruído que os separaria é ela mesma inflada por
movimento rápido. Os 4 modelos e por que cada um morreu estão no doc do módulo
[`curve_prep.rs`](../crates/ph2d-anim/src/curve_prep.rs).

**A assimetria que decidiu:** uma quina fantasma **fixa um key e quebra uma tangente dentro de uma curva
suave** (regressão visível). O arredondamento que ela evitaria — o ápice de um quique reconstrói **2,8% do
range** abaixo — está **DENTRO** do envelope de ±1–3% que o fit já declara e que o Enio **aprovou**
("ficou bom", §17.2). Enviar trocaria uma aproximação aceita por uma regressão. É a **regra two-strikes**
da DIRETIVA §5 (eu estava na 4ª reconstrução do modelo).

**Pin executável:** `a_recorded_bounce_still_loses_its_apex_the_corner_pass_is_deferred` afirma que o
ápice perde 2–4%. Se um pré-passe de quina landar, ele fica **VERMELHO** — o adiamento não pode ser
esquecido em silêncio.

**Se for retomar, os 2 caminhos que valem:** (a) restringir a busca de quina a **reversões** (extremos que
o detector de picos já acha) — mata a classe inteira de falso-positivo em rampa, ao custo do "joelho" sem
reversão; (b) decomposição multi-escala de verdade (wavelet/scale-space), que é pesquisa, não tarefa de
sessão.

---

## §5 — O que só o `ship.sh` pega

Rodei fmt (rustup 1.95), clippy `--all-targets`, nextest, typos, LOC caps. **Não** rodei: `machete`,
`deny`, `audit`, nextest com `--cargo-profile ci-test`. Não adicionei dependência nenhuma, então `machete`/
`deny` devem passar limpos — mas o gate per-linha **não** os roda
([[project_integrator_ship_catches_latents_budget_iterations]]: orce 2–4 iterações no ship).

---

## §6 — DUAS DECISÕES SUAS (Enio)

### 6.1 — O gizmo escreve rotação embrulhada: consertar na RAIZ?

O unwrap conserta o **record**. Mas a raiz — `gizmo/transform.rs:280` — segue lá, e ela quebra mais coisa:

- **Você não consegue autorar mais de ±180° num arrasto.** O valor embrulha; visualmente o sprite gira,
  mas o `Transform.rotation` nunca sai de uma janela de 2π. Autorar um giro de 3 voltas na mão é
  impossível hoje.
- **Dois keys manuais atravessando o corte interpolam pelo caminho LONGO** (3.0 rad → −3.0 rad = −6 rad em
  vez de +0.28).
- **Resíduo de splice no record:** keys fitted (desembrulhadas, ex. 12.46 rad) coladas em keys vizinhas
  fora do span (embrulhadas, ex. 0.0) → o giro desaba na fronteira. É a mesma ambiguidade que o Blender
  expõe com o operador "Discontinuity (Euler) Filter".

**Fix proposto (pequeno):** desembrulhar o resultado contra a rotação **viva** — as voltas acumuladas
passam a morar no próprio `Transform.rotation`, sem estado novo:
```rust
t.rotation = cur + wrap_to_pi(new_rotation - cur);   // cur = valor do frame anterior
```
**Não fiz porque o raio de impacto é seu para decidir:** `compute_gizmo_transform` é **compartilhado**
(sprite, vetor por ADR-0111, drags de grupo) e a mudança é **visível na UX** — o Inspector passaria a
mostrar **430°** em vez de 70° depois de uma volta e um pouco. Isso é o que Blender/AE fazem (e é o
correto para animação), mas é mudança de comportamento que você não pediu e que eu não consigo
smoke-testar sozinho.

### 6.2 — Smoke (o que testar no app)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop
```

1. **Giro gravado (o fix principal).** Bind a rotação de um objeto → arme **Record** → Play → gire o
   objeto **várias voltas** com o gizmo durante a reprodução → solte.
   **Esperado:** o giro replica como giro — **não desgira**, não trava, não vira nada. Antes deste commit
   o giro sumia por completo. *(Nota: por causa do §6.1, autorar >180° **num único arrasto** ainda é
   limitado — gire com vários arrastos, ou solte o §6.1 primeiro.)*
2. **Fade de opacidade.** Grave um fade que sobe rápido e **descansa em 1.0**. Abra o **graph editor**.
   **Esperado:** a curva **encosta** em 1.0 e não passa (antes desenhava um estufado acima do topo).
3. **Regressão — gesto suave.** Grave um movimento suave qualquer (X/Y) e olhe a curva.
   **Esperado:** exatamente como antes desta linha — poucos keys nos extremos, curva limpa, **sem dobras
   novas**. (Era isto que o detector de quina teria estragado.)

---

## §7 — Fila restante da linha (do handoff anterior, inalterada)

ETAPA 1 (W4.T7 relógio único ← coordena com Motion) · ETAPA 2 (W4.T4 dock no `motion_timeline_slot`) ·
ETAPA 3 (NLA / seletor de clip — 100% isolado) · ETAPA 4 (markers → signals) · ETAPA 6 (save cena+timeline).
Detalhe em [`HANDOFF_line_anim_CONTINUACAO_2026-07-12.md`](HANDOFF_line_anim_CONTINUACAO_2026-07-12.md) §2.
