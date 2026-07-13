# HANDOFF DE CONTINUAÇÃO — `line/anim` (2026-07-13)

> **Para:** o **próximo agente** da linha `line/anim`.
> **De:** o agente que fechou a composição de clips (ADR-0115) — **integrada no main**.
> **Estado:** a linha **integrou**. O worktree está **desatualizado** (§1). Nada em voo, nada quebrado.
>
> Leia **§0 inteiro** (o modo de trabalhar) e **§2** (a bomba-relógio que eu deixei) antes de tocar em
> qualquer coisa. A fila está em **§4**.

---

## §0 — Como se trabalha aqui (Modo L) — **isto não é opcional**

Você é **uma linha autônoma** numa jornada multi-agente
([GUIA_JORNADA_MODO_L.md](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) ·
[DIRETRIZ §1.5](IntegracaoMultiAgente/DIRETRIZ.md) · [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md) ·
[ADR-0107](architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)).

### O que isso significa na prática

| | |
|---|---|
| **Seu worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim/`, branch `line/anim`. **Índice e git próprios** — colisão de commit não existe. |
| **Você commita** | `git commit --no-verify -m "..."` (fast mode). Local, à vontade, em blocos. |
| **Você NÃO** | **integra** · **pusha** · roda **`ship.sh`**. Nunca. Por conta própria é **violação de protocolo** (CLAUDE.md §0.7). |
| **Foundational** | Você **PODE e DEVE** tocar (ADR-0107). Ao **criar** foundational novo, **projete para isolamento** (módulo irmão, extensão append-only). |
| **PARE e reporte ao Enio** | Só em **2 casos**: (a) **contrato congelado** (CLAUDE.md §6 — exige ADR); (b) **rebase conflitando fora dos seus arquivos** (colisão de mesmo-símbolo). |
| **Nunca** | negocie direto com outro agente/linha. **O Enio é o único canal.** |
| **Você fecha** | escreve o **handoff de integração** (DIRETRIZ §1.5.9) e **PARA**. O Enio dispara um **agente integrador** dedicado. |

### ⚠️ A regra mecânica que custou caro (leia mesmo achando óbvio)

**O `cwd` do shell deriva NO MEIO do turno.** Eu contaminei o repo primário assim, e hoje mesmo — escrevendo
este handoff — quase reportei um alarme falso ao Enio porque um `ls` relativo leu o worktree velho em vez do
`main`. As duas vezes, a mesma causa.

> **Todo comando que MUTA começa com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim &&`.**
> **Todo caminho dentro de script é ABSOLUTO, com o segmento `/Worktrees/line-anim/`.**
> Para **ler** o estado do `main`, use **refs** (`git show main:arquivo`, `git grep ... main --`), **nunca**
> o filesystem — o filesystem do seu worktree **não é** o main.

Memórias: [[feedback_sed_relative_path_hits_primary_cwd]] · [[feedback_perl_utf8_mojibake_use_edit_tool]]
(texto acentuado **só** via ferramenta Edit — `perl`/`sed` corrompem o arquivo inteiro) ·
[[feedback_backticks_in_commit_message_are_command_substitution]] (crase na msg de commit é **executada** e a
palavra some em silêncio — use `git commit -F <arquivo>` e **releia o log**).

### Ritmo

- **Inner loop:** **só** `cargo check -p <crate>`. Nada de test/clippy/auditoria por task.
- **Gate batched, 1× no fechamento** sobre o diff acumulado: `scripts/nextest-impacted.sh <base>` +
  `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` +
  `rustup run 1.95 cargo fmt --all -- --check` + `typos` + **auditoria de ≥2 lentes**.
- **A auditoria não é ritual.** No fechamento da minha jornada as 2 lentes acharam **7 defeitos, 2
  CRÍTICOS** — um deles fazia o sprite **afundar até a pose de repouso no meio de um clip que não se move**,
  e todos os meus testes estavam **verdes**. Uma lente montou uma crate-sonda contra o código real e voltou
  com repro numérico. **Faça isso.** ([[feedback_audit_lens_diversity]])
- **Smoke 1× no fim**, com o comando pronto incluindo o `cd` ([[feedback_ready_to_smoke_example]]).

---

## §1 — Primeira coisa que você faz: **sincronizar o worktree**

A linha **integrou** (o conteúdo está no `main`; a integração **rebasou**, então os hashes mudaram). O
worktree está **140 commits atrás** e ainda aponta para o `HEAD` velho.

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && \
  git fetch origin && git status --short          # tem que estar LIMPO
```

Se estiver limpo, **jogue a branch em cima do main** — não há nada a preservar, tudo já está lá:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && \
  git reset --hard main && git log --oneline -1
```

**Confira que a integração não perdeu nada** (eu conferi; refaça mesmo assim — leva 5 s):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && \
  ls crates/ph2d-timeline/src/{stack,stack_eval,stack_edit,clock,refusal}.rs && \
  cargo check -p ph2d-timeline -p ph2d-panel-timeline
```

> A integração mexeu em **exatamente 2 linhas** do que eu escrevi (dois `use` redundantes no `seam.rs`).
> Nada mais.

---

## §2 — ⚠️ A BOMBA-RELÓGIO QUE EU DEIXEI: **dois ADRs numerados 0115**

**Achei isto escrevendo este handoff.** É meu, é real, e é a **tarefa 0** da sua fila.

O `main` tem **dois** ADRs com o número **0115**:

| arquivo | assunto | entrou no main |
|---|---|---|
| `0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md` | **meu** (composição de clips) | 2026-07-12 **22:42** |
| `0115-audio-spectral-fft-via-realfft.md` | da linha de áudio (FFT espectral) | 2026-07-12 **22:53** |

**Nomes de arquivo diferentes ⇒ o git nunca conflitou ⇒ os dois entraram em silêncio.** É exatamente
[[feedback_numbers_that_sum_across_lines_count_dont_pick]]: *duas linhas escolheram "o próximo número
livre" contra um main que ainda não tinha o ADR da outra.*

**A consequência é pior que cosmética:** hoje `grep "ADR-0115"` devolve **52 arquivos** falando de **dois
assuntos sem relação** — as crates de áudio dizem "ADR-0115" para o spectral, as de timeline dizem
"ADR-0115" para a pilha de clips. **Um `sed` global aqui destrói metade das referências.** O fix tem que
ser **escopado por módulo**.

### Recomendação (é do Enio a decisão; a evidência é esta)

**Quem renumera é o ÁUDIO**, `0115-audio-spectral…` → **`0120-…`**:

- **chegada:** o meu landou **11 min antes** (convenção de quem chega primeiro fica com o número);
- **custo:** o lado do áudio cita `ADR-0115` em **9 arquivos**; o meu, em **36**.

Os dois critérios apontam para o mesmo lado — é raro e é conveniente.

> ⚠️ **Não renumere o ADR do áudio por conta própria.** É arquivo de outro módulo. **Reporte ao Enio**
> (CLAUDE.md §0.2) e execute só com ordem dele. Se ele mandar você mexer, ótimo — o `main` já integrou o
> áudio, então não é WIP alheio.

### E o fix DURÁVEL (este é seu, sem pedir nada a ninguém)

**Não existe gate contra número de ADR duplicado.** Escreva um. É pequeno e mata a classe inteira:

> um teste em `crates/ph2d-editor-core/tests/` que lista `docs/architecture/decisions/*.md`, extrai o
> prefixo `NNNN` e **falha se dois arquivos dividem o número**.

A lição da memória é literal: *"o valor certo não existe em nenhum lado do conflito — **conte, não
escolha**. Prove com o teste."* Duas linhas paralelas **vão** colidir de novo no próximo número; o gate é
o que faz isso vermelhar no fechamento em vez de entrar mudo.

---

## §3 — Onde a composição de clips parou (o que existe HOJE no main)

**ADR-0115 (o meu) + [`docs/Timeline/02_plano_composicao_clips.md`](Timeline/02_plano_composicao_clips.md).**
Fatia A (dados+avaliação) e fatia B (UI) **fechadas**, exceto **B4** (§4).

O modelo, em uma frase: **a sobreposição É o crossfade.** Faixas (`ClipLane`) de instâncias de clip
(`ClipStrip`); sobrepor dois strips **cria** o crossfade (ninguém digita duração); canais **esparsos** (o que
o clip keya É a máscara); blend/peso vivem na **FAIXA**, não no strip. Portar o strip-stack do Blender foi
**descartado pela pesquisa** — o próprio Blender está abandonando o dele (projeto Baklava), e no **2D** o
idioma não é "empilhar e blendar", é **nesting** (§4, tarefa 5).

### As 5 armadilhas que a auditoria expôs — **sobrevivem como conhecimento, não como fix**

Se você for mexer no avaliador ou na pintura das faixas, estas são as minas:

1. **A faixa NÃO é uma escada.** `blend_out(i)` perguntava a `strips[i+1]` — o vizinho na *ordem do vetor*.
   É o strip certo **só** se os strips formarem uma escada, e **nada os obriga a isso**. Solte um strip
   curto **dentro** de um longo e a cobertura desaba: **o sprite rasteja de volta ao rest no meio de um
   clip que não se move** (500 → 104, medido). Hoje a janela vem de **todo** strip vivo naquela borda.
2. **Uma key pode mover a REFERÊNCIA contra a qual ela é medida.** Numa faixa `Additive`, o delta é medido
   contra o valor do próprio clip em `src_in`. Keye no **primeiro frame do strip** — onde o animador começa
   a posar — e a key que você escreve **É** a referência. A sonda modela a **escrita**
   (`Probe{clip, value, t_key}`), não só o valor.
3. **`prime_stack(t)` antes de QUALQUER pergunta à pilha.** O `scratch` (strips vivos + relógios) é
   reconstruído **dentro do apply**. Quem pergunta "onde a key cai?" pede *agora* e era respondido *quando o
   apply rodou por último* — em produção coincidem, **e é isso que torna o acoplamento invisível**. `key_home`
   tem `debug_assert` conferindo. É a **4ª** vez que esta classe de bug morde este módulo
   ([[feedback_derived_coordinate_seed_must_match_sample]]).
4. **A inversão VERIFICA a afinidade; não acredita nela.** Dois pontos passam uma reta por **quaisquer** duas
   amostras — não dizem o que houve **entre** elas. Uma 3ª sonda confere e **recusa** o que não for afim (o
   mesmo clip numa faixa `Override` **e** numa `Ratio` é **quadrático** em `v`).
5. **A ordem de registro de hit é load-bearing.** O hit index resolve para o **último** id registrado, e os
   strips **podem se sobrepor** — isso **É** o crossfade. Registrar cada strip inteiro punha o **corpo** do
   strip vizinho **em cima da borda** que você usa para ajustar o crossfade recém-criado. **Dois passes:
   todos os corpos, depois todas as bordas.** A ordem é função pura (`hit_plan`) e tem gate.

### Gotchas do módulo que já existiam

- **`TimelineHitKind::Lane` ≠ `LaneHeader`.** `Lane` é a linha **vazia** do dope-sheet (onde nasce o
  marquee); `LaneHeader` é o rótulo da faixa da pilha. Duas coisas chamadas "lane" é obra da própria
  timeline. **Se um merge tentar unificá-las, está errado.**
- **Menu de contexto:** quando o `Click` chega, o menu **já fechou**. Leia
  `context_menu().or_else(last_context_menu())` — ler só o aberto **entrega um menu que não faz nada**.
- **Botão dimmed ainda despacha** ([[feedback_disabled_button_still_dispatches]]) — recuse no `event.rs`,
  não só na pintura.
- **`DOC_VERSION` = 4.** Postcard é **posicional**: campo novo **só apendado**, e bump obrigatório.

---

## §4 — A FILA (ordem sugerida; a prioridade é do Enio)

### 0. ⚠️ ADR-0115 duplicado — **§2**
Reporte ao Enio + escreva o **gate anti-duplicata**. É a única tarefa que eu classificaria como *dívida
minha*, e é rápida.

### 1. `CLAUDE.md` §5 está **mentindo** sobre a timeline
A entrada ainda diz *"composição de clips (ADR-0115, **aguarda ratificação**)"* e lista **W4.T4** como
*"espere a linha Motion fechar"*. **As duas coisas mudaram:** a composição **landou e integrou**, e a linha
Motion **integrou** (o `MotionTransport` foi extinto — confirmei: o tipo não existe mais em
`ph2d-motion-doc`).

Atualize a entrada **Timeline** do §5. Comentário velho e estado velho **mentem**
([[feedback_stale_comment_and_dead_code_lie]]) — e um agente novo lê o CLAUDE.md **antes** de agir.

### 2. **B4 — o ease handle do strip** (o buraco real da fatia B)
**É o único item da fatia B que eu não construí**, e ele deixa um gesto **impossível**, não só inconveniente:

Os campos `ClipStrip.ease_in` / `ease_out` **existem, são serializados e o avaliador os usa** — mas **só
quando o strip NÃO tem vizinho** (com vizinho, a sobreposição vence, que é a regra do Unity). E **não há UI
nenhuma** para autorá-los (`grep ease_in crates/ph2d-panel-timeline/` = **zero**).

> **Consequência:** um strip **sozinho** na faixa **não pode fazer fade-in/out**. Ele entra e sai **duro**.

**O desenho** (ADR-0115 §B4): alça na **quina** do strip (padrão Unreal — alça direta, não campo de
Inspector), que vira **read-only** quando um vizinho define a janela (padrão Unity — ease e blend são a
**MESMA** curva, e é isso que impede dois números de discordarem).

**A cunha já é desenhada** a partir de `blend_in`/`blend_out` **perguntados à faixa** — então a alça deve
**escrever `ease_*` e ler a cunha**, nunca recalcular. E o `StripView` já carrega `blend_in`/`blend_out`.

### 3. Ajustes de UX da pilha (**pergunte ao Enio o que ele viu**)
Ele aprovou o smoke com *"faremos ajustes depois"*. **Não adivinhe** — peça a lista. (Se ele disser
*"difícil de ajustar"* sobre algo, isso é um **bug de DESIGN**, não de calibração:
[[feedback_ergonomics_verdict_is_a_design_bug]] — questione o modelo, não afine o número.)

### 4. **W4.T4 — dock da timeline no `motion_timeline_slot`** ✅ **DESBLOQUEADO**
Estava travado esperando a linha Motion; **ela integrou**. O slot **existe e está vazio**
(`screens/layout.rs:270`); a timeline hoje é painel docado bottom próprio.

### 5. **Nesting** — o idioma 2D de verdade (**o próximo ADR**)
A pesquisa da composição de clips **nomeou isto explicitamente** e não varreu para debaixo do tapete: no 2D,
"empilhar e blendar" **não é** o idioma (Animate/Harmony/AE têm **zero** blend de animação); o idioma é
**nesting** (Precomp do AE, Symbol do Animate) — **e nós temos zero**.

**É um ADR antes de ser código.** Pesquise o padrão-ouro **antes** de propor (foi exatamente assim que a
composição de clips virou o ADR certo em vez de um port ruim do Blender — [[feedback_no_industrial_claims_without_verification]]).

### 6. **W4.T6/B5 — persistir a timeline no projeto**
Confirmei: **`shells/desktop/src/project.rs` não salva a timeline** (`grep timeline` = zero). O
`ProjectState = {WorldSnapshot + VecScene}` salva objetos, hierarquia e canvas — **a animação inteira se
perde ao fechar o app.** O `TimelineDoc` já é `Serialize`/`Deserialize` com `DOC_VERSION` 4; falta **anexá-lo
à captura** e cuidar da versão.

### 7. Markers → signals · export
Backlog da timeline (`docs/Timeline/01_plano_timeline_ui.md`). Sem bloqueio conhecido.

---

## §5 — Mapa do código (onde as coisas moram)

| o quê | onde |
|---|---|
| **Dados** da pilha | `crates/ph2d-timeline/src/stack.rs` (`ClipLane`/`ClipStrip`/`weight_at`/`blend_in`/`blend_out`) |
| **Avaliador** + inversão | `stack_eval.rs` (`sample_stack` · `invert_stack` · `Probe` · `StackScratch` · `AFFINE_TOL`) |
| **Autoria** (doc) | `stack_edit.rs` (`add_lane`/`add_strip`/`duplicate_strip`/`MAX_LANES`) |
| **Intents** | `intent.rs` (o vocabulário) · `intent_apply.rs` (o roteador — `trim_strip`/`stretch_strip`/`settle`) |
| **Relógio** | `clock.rs` (`ClockIndex`) · `apply.rs` (`remapped_time` · `key_time` · **`key_home`**) |
| **Recusa** | `refusal.rs` (`KeyRefusal` — as 3 causas + a mensagem) |
| **UI da pilha** | `crates/ph2d-panel-timeline/src/stack_lane_paint.rs` (pintura + **`hit_plan`**) · `strip_drag.rs` (arrasto/trim/stretch) |
| **Eventos** | `event.rs` (`stack_event` é a **porta única** da pilha) |
| **Toast da recusa** | `shells/desktop/src/render_loop/autokey_pass.rs` (latch `AutokeyState.refusal`) |
| **Gates de costura** | `crates/ph2d-panel-timeline/tests/seam.rs` (anti-item-morto: **cada linha de menu levanta o intent que o NOME dela promete**) |

**Smoke:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop` ·
**`L`** abre a timeline · o app já sobe com 8 objetos nomeados.

---

## §6 — Ao fechar

1. Gate batched (§0) + **auditoria de ≥2 lentes** sobre o diff acumulado.
2. **Smoke** e reporte ao Enio (comando pronto, com o `cd`).
3. **Handoff de integração** (DIRETRIZ §1.5.9) — mapeie a **superfície de colisão** com as linhas vivas
   (`git worktree list`), e **teste o merge de verdade**: `git merge-tree` passa **verde** em merge que
   **não compila** (foi o que aconteceu com a linha Motion — o conflito era **semântico**, não textual).
4. **PARE.** Não integre, não pushe, não faça ship.
