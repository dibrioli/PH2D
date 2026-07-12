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
| **HEAD** | `c7e740b5` — **2 commits** de código (`8874a2b7` semântica de canal · `c7e740b5` raiz no gizmo) |
| **Gate** | `nextest` **WORKSPACE INTEIRO 5609/5609** · clippy `--all-targets` **0 warnings** · `fmt` (rustup 1.95) · LOC caps ok · `typos` ok |
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

## §6 — A raiz (`c7e740b5`) e o smoke

### 6.1 — RESOLVIDO: o gizmo agora acumula voltas (Enio autorizou 2026-07-12)

`Transform.rotation` era derivada de um **par** de `atan2` (`gizmo/transform.rs:280`), ambos em `(-π, π]`
— então a diferença sozinha só descreve **menos de uma volta** e **salta 2π** no corte de ramo. O unwrap
do `8874a2b7` tratava o sintoma no fit; **isto é a causa**, e ela quebrava mais que o record:

- impossível autorar **mais de ±180° num único arrasto**;
- dois keys manuais atravessando o corte interpolam pelo **caminho longo**;
- o giro gravado voltava como dente-de-serra.

**Fix:** uma função **pura** de (cursor inicial, cursor atual) **não consegue** recuperar a contagem de
voltas — ela vive no **caminho** que o cursor percorreu. Então a contagem virou **estado do arrasto**:
`GizmoDragState.turns` (i32, zero no Down), mantido por `GizmoDragState::advance_cursor` e consumido
**dentro** de `compute_gizmo_transform` — assim nenhum chamador futuro pode esquecer dele. Os caminhos de
grupo/global carregam as voltas de graça (já derivam o delta do resultado do compute).

**Mudança de UX (autorizada):** o Inspector passa a mostrar **graus acumulados** (430° em vez de 70° após
uma volta e pouco) — `snapshots.rs:610` alimenta o inspector com `t.rotation` direto. É o que Blender e AE
fazem e o que animação exige. **A view do gizmo NÃO muda** (`snapshots.rs:326` deriva o ângulo do afim por
`atan2`, invariante mod 2π).

**Símbolos novos:** `GizmoDragState.turns` (campo, apendado por último) + `GizmoDragState::advance_cursor`
(método). Os 3 construtores no shell e os 11 nos testes ganharam `turns: 0`.

**O unwrap do fit FICA** e não é órfão: ele defende dados de outras fontes (projeto carregado, rotação
derivada de matriz por `atan2` em `snapshots.rs:326`, `rotation` relativa a pai) e é a rede que faz um
giro gravado sobreviver mesmo se algo voltar a embrulhar.

### 6.2 — Smoke (o que testar no app)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop
```

1. **Girar muitas voltas na mão (a raiz).** Sem gravar nada: pegue a alça de rotação e **rode 3 voltas
   num único arrasto**, olhando o campo Rotation do Inspector.
   **Esperado:** o número **passa de 360° e continua** (~1080°) em vez de voltar pra dentro de ±180°.
   O sprite gira normal. Solte e gire de novo: continua de onde parou.
2. **Giro gravado (o fix principal).** Bind a rotação de um objeto → arme **Record** → Play → gire o
   objeto **várias voltas** com o gizmo durante a reprodução → solte.
   **Esperado:** o giro replica como giro — **não desgira**, não trava, não vira nada. Antes o giro sumia
   por completo (span reconstruído de 0.00 rad).
3. **Fade de opacidade.** Grave um fade que sobe rápido e **descansa em 1.0**. Abra o **graph editor**.
   **Esperado:** a curva **encosta** em 1.0 e não passa (antes desenhava um estufado acima do topo).
4. **Regressão — gesto suave.** Grave um movimento suave qualquer (X/Y) e olhe a curva.
   **Esperado:** exatamente como antes desta linha — poucos keys nos extremos, curva limpa, **sem dobras
   novas**. (Era isto que o detector de quina teria estragado.)
5. **Regressão — gizmo normal.** Rotação curta, escala, translação, gizmo global com vários objetos,
   Shift para snap de ângulo. **Esperado:** tudo idêntico a antes (o workspace inteiro está verde, mas
   gizmo é coisa de olho).

---

## §7 — Fila restante da linha (do handoff anterior, inalterada)

ETAPA 1 (W4.T7 relógio único ← coordena com Motion) · ETAPA 2 (W4.T4 dock no `motion_timeline_slot`) ·
ETAPA 3 (NLA / seletor de clip — 100% isolado) · ETAPA 4 (markers → signals) · ETAPA 6 (save cena+timeline).
Detalhe em [`HANDOFF_line_anim_CONTINUACAO_2026-07-12.md`](HANDOFF_line_anim_CONTINUACAO_2026-07-12.md) §2.
