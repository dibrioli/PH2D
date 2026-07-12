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
| **HEAD** | `a762aebf` — **4 commits** de código (`8874a2b7` semântica de canal · `c7e740b5` raiz no gizmo · `8ef30a82` W4.T7 relógio único · `a762aebf` **ETAPA 3 seletor de clip**) |
| **Gate** | `nextest` **WORKSPACE INTEIRO 5624/5624** · clippy `--all-targets` **0 warnings** · `fmt` (rustup 1.95) · LOC caps ok · `typos` ok |
| **⚠️ Linha Motion VIVA** | **Leia §7 antes de integrar** — superfície de colisão em `motion_bridge.rs` mapeada por função |
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

## §7 — W4.T7: o editor tem UM relógio (`8ef30a82`) — **e a linha Motion está viva**

### 7.1 — O que mudou

Eram **três** cópias do tempo: o `MotionTransport` do shell, o `Playhead`, e o `last_cooked_tick` do pump.
O transporte do Motion foi **REMOVIDO** (`MotionState.transport`); o tick que o cook renderiza é **derivado**
do `Playhead` (`motion_tick = round(time / fixed_dt)`), e o `last_cooked_tick` do pump é o único registro de
onde a simulação está. **Não há o que divergir — por construção, não por disciplina.**

O pump já estava pronto: a doc de `advance_or_scrub_scoped` diz literalmente *"a future timeline ruler that
sets `transport.tick` is handled for free"*. Um tick pra frente = pump barato; um salto (scrub, seek, wrap de
loop) = restore do checkpoint + re-sim **bit-exato** (M2.N2).

**A garantia de determinismo está isolada em `ticks_owed()`** e testada: um frame lento (ou `rate 2`) deve
simular **todos** os ticks que pulou — a trajetória de um nó sequencial (`integrate`/`spring`/`verlet`) é a
**soma** dos passos. Pra trás é o oposto: **uma** chamada (restore + re-sim), não um passo por tick.

**Semântica nova (é o objetivo):** Space no grafo pausa a **timeline** e vice-versa · a régua da timeline
move o grafo · abrir o Motion toca o relógio do editor.

### 7.2 — ⚠️ SUPERFÍCIE DE COLISÃO (o integrador LEIA ISTO)

A linha `motion-value` está **aberta**. Ela **não** disputa o relógio — o briefing dela põe o W4.T4/T7 em
"⛔ FORA DESTA LINHA: coordena com a linha `anim`". Mas nós dois podemos escrever no **mesmo arquivo**:

| `shells/desktop/src/render_loop/motion_bridge.rs` | dono | o que fiz |
|---|---|---|
| cabeçalho + assinatura de `dispatch` | **anim** | `frame_ticks: u32` → `playhead: &mut ph2d_core::Playhead` |
| bloco 2 (auto-play na entrada) | **anim** | `motion.transport.play()` → `playhead.play()` |
| bloco 3 (cook por-frame) | **anim** | reescrito: `motion_tick` + `ticks_owed` (fns novas) |
| **`apply_graph_intents`** | **⚠️ AMBOS** | ganhou o param `playhead`; `GraphIntent::TogglePlay` → `playhead.toggle_play()` |
| `apply_connect` / `apply_disconnect` / params | **motion-value** | **não toquei** |

**O ponto quente é `apply_graph_intents`** — a linha Motion mexe nela para intents novos do painel (a
ETAPA D dela). São **símbolos diferentes dentro da mesma função**: o Mergiraf funde, mas confira.

**Outros arquivos:** `motion_state.rs` (campo removido) · `mod.rs` (call site) · `motion_bridge_tests.rs` ·
`crates/ph2d-eval-motion/src/lib.rs` (**um getter novo**, `MotionCookPump::last_cooked_tick()` — append-only).

### 7.3 — Uma limpeza que é DELES, não minha

`ph2d_motion_doc::MotionTransport` ficou **sem nenhum uso**. Não o removi: mora na crate da linha Motion, e
apagar um tipo `pub` na crate alheia enquanto ela está viva é exatamente a colisão que se deve evitar
([[feedback_audit_scope_discipline]]). **Nada no shell o usa mais** — a remoção é um one-liner para eles.

### 7.4 — Smoke do T7 (some aos itens de §6.2)

6. **Um relógio.** Abra o Motion (grafo com um nó temporal — `motion.emitter`, um `spring`). **Space** no
   grafo deve pausar/tocar **a timeline junto**. Mova a **régua da timeline**: o grafo deve seguir, inclusive
   **para trás** (o spring re-simula, não mostra o futuro).
7. **Custo conhecido (não é bug).** Se você tocar a timeline até, digamos, 50 s e **só então** abrir o Motion,
   ele re-simula até lá numa tacada (o ring de checkpoints está frio). É o preço correto de um relógio único
   — o grafo é avaliado no tempo em que a **cena** está. Scrubs seguintes são O(1). Se travar de forma
   inaceitável, me fale: dá pra semear o ring.

---

## §8 — Fila restante da linha (do handoff anterior, inalterada)

~~ETAPA 1 (W4.T7 relógio único)~~ **FEITA** (§7) · ~~ETAPA 3 (seletor de clip)~~ **FEITA** (§9) ·
**ETAPA 2** (W4.T4: docar a timeline no `motion_timeline_slot` — destravada pelo T7, mas **ESPERE a linha
Motion fechar**: ela cai em `motion_bridge.rs` blocos 1-2 + dentro de `apply_graph_intents`, a região viva
dela, e nada depende dela) · ETAPA 4 (markers → signals — isolada, pequena) · **composição de clips**
(empilhar; o seletor é o passo 1 — **desenhada, não construída**: §10) · ETAPA 6 (save cena+timeline).
Detalhe em [`HANDOFF_line_anim_CONTINUACAO_2026-07-12.md`](HANDOFF_line_anim_CONTINUACAO_2026-07-12.md) §2.

---

## §9 — ETAPA 3: seletor de clip (`a762aebf`)

O dado já existia (`TimelineDoc.clips` + `active_clip`) e só o clip ativo era exposto — faltava **toda** a
autoria. Agora dá para **ver, trocar, criar, renomear e apagar** clips.

**O modelo (é o que torna o recurso barato):** os **bindings são do DOCUMENTO**, não do clip. Todo clip
anima os mesmos objetos e só as **keys** mudam — um segundo clip custa um nome e nada mais ("walk" e "run"
sobre um rig só). É o precomp do AE e o clip da Unity.

**UI:** `[ Main ▾ ] [+] [✎] [🗑]` à esquerda da barra de transporte (onde a Unity põe). Dropdown real; o
popover é **diferido** para o fim do paint, senão a lista sai desenhada **debaixo** da régua e das rows.

**Símbolos novos** (nenhum id numérico — todos são hash de slug, colisão impossível):

| símbolo | onde |
|---|---|
| `MAX_CLIPS = 16` | `ph2d-timeline::doc` |
| `TimelineDoc::{add_clip, rename_clip, remove_clip, fresh_clip_name}` | idem |
| `TimelineIntent::{SetActiveClip, AddClip, RenameClip, DeleteClip}` | `ph2d-timeline::intent` (**apendados** ao fim do enum) |
| `TimelineViewSnapshot::{clips, active_clip}` | `ph2d-timeline::snapshot` (campos novos) |
| `TIMELINE_CLIP_DD` · `TIMELINE_CLIP_OPT[16]` · `TIMELINE_ADD_CLIP` · `TIMELINE_RENAME_CLIP` · `TIMELINE_DELETE_CLIP` · `TIMELINE_CLIP_RENAME_INPUT` | `ph2d-editor-core::ids::chrome::timeline` |
| `clip_rename.rs` (módulo irmão) · `paint_overlays` | `ph2d-panel-timeline` |

**O teto NÃO é um chute:** os ids de opção de um dropdown são um **array fixo** de `NodeId` (o chrome não
cria hit id em runtime), então o número de clips que o **doc** aceita tem de ser o número de ids que o
**painel** endereça. O doc **recusa** o 17º, e um gate amarra os dois
(`MAX_CLIPS == TIMELINE_CLIP_OPT.len()`) — sem ele, um clip a mais pinta uma opção que nada clica.

**`DOC_VERSION` NÃO mudou:** `clips` já era um `Vec` serializado; nenhuma forma nova foi para o disco.

**ZERO mudança no shell:** os intents viajam o canal direto (`drain_intents`), que o shell já aplica
genericamente (`mod.rs:805`).

### Smoke (some aos itens de §6.2 e §7.4)

8. **Clips.** Crie um objeto, bind uma track, ponha keys. Clique **+** → nasce "Clip 2", **vazio**, e a
   **row continua lá** (o binding é do documento). Volte para "Main" pelo dropdown: **as keys estão onde
   você deixou**. Renomeie pelo lápis (Enter confirma, Esc cancela). Apague pela lixeira.
   **Esperado:** com **um** clip só, a lixeira **não aparece** (o documento sempre tem um clip para editar).
   Um **Ctrl+Z** desfaz cada operação inteira.

---

## §10 — Composição de clips: DESENHADA, não construída (só docs neste commit)

**Nada de código.** O Enio pediu o próximo passo ("empilhar clips") e mandou **pesquisar o padrão-ouro
antes de portar** — *"Blender nem sempre é o melhor"*. A pesquisa (5 frentes) **inverteu o desenho** e o
resultado é [**ADR-0115**](architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md)
+ [plano](Timeline/02_plano_composicao_clips.md). Ambos **propostos** — aguardam ratificação.

**O que a pesquisa matou (portar o strip-stack do Blender):**
- O **próprio Blender** está movendo blend/influence do strip pra **camada** (projeto Baklava: Slotted
  Actions shipou no 4.4; Layered Animation em WIP; strips-na-Action 2027+), cortou 5 modos pra 2 e quer
  **eliminar o tweak mode**. Veredito deles sobre o NLA: *"not a pleasure to work with"*.
- Os **sequenciadores** (Unity/Unreal/Maya/MotionBuilder) convergiram no gesto **sobrepôs-cruzou** — que
  falta ao Blender (strips na mesma faixa **não podem** se sobrepor lá).
- No **2D**, "empilhar e blendar" **não é o idioma**: Animate/Harmony/AE não têm blend de animação nenhum;
  a Cavalry (única 2D com blend real) escolheu **camadas por-atributo**; o Moho resolve overlap por **canais
  disjuntos**. O idioma 2D é **nesting** — e nós temos **zero** (nomeado como o ADR seguinte).

**Três coisas que a pesquisa expôs no NOSSO código, e que o integrador deve conhecer:**
1. `remapped_time` lê `doc.active_clip()` ([apply.rs:98](../crates/ph2d-timeline/src/apply.rs)) — sob uma
   pilha, *qual clip dá o relógio da entidade?* é **indefinido**. O ADR §2/R6 fixa: o strip mapeia
   timeline→clip, o `TimeRemap` **daquele clip** mapeia clip→fonte (modelo precomp do AE).
2. O apply **já é O(bindings²)** (o remap re-varre a lista por binding). Empilhar em cima disso vira
   **cúbico** — o hoist é **pré-requisito medido** do kill-criterion, não bônus.
3. `TranslationX` é posição **absoluta**: "blend-to-default" (a regra do Blender/Godot) **jogaria o sprite
   na origem do pai**. Eles não sofrem disso porque osso é rest-relative. Daí o `rest` **capturado** por
   binding (ADR §2/R5) — o Capture Base State do Rive e a Base Pose do Unreal, que os dois precisaram.

**Nenhum contrato congelado é tocado**; quando construído, `DOC_VERSION` vai 3→4.
