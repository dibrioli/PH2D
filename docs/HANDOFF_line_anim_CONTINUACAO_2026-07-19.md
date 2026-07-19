# Handoff de CONTINUAÇÃO — `line/anim-fixes` (novo agente assume)

> Para o agente que assume a linha **depois da integração do nesting** (2026-07-19).
> Este doc é o "onde paramos + para onde ir". A **mecânica de reabrir a linha** é o BLOCO de
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> — leia-o, não o resumo daqui; duas cópias da mesma regra divergem.

---

## 0. Reabrir a linha (a mecânica, curta)

Você começa na **raiz do repo, que está em `main`**. Os mesmos paths existem nas duas árvores;
editar o da raiz compila e commita sem erro e só aparece na integração. Então, **antes de ler
qualquer código:**

```
cd Worktrees/line-anim && pwd && git branch --show-current
      → tem de terminar em /Worktrees/line-anim, branch line/anim-fixes
git log --oneline -5 && git status -sb
git rebase main            # OBRIGATÓRIO no início da jornada (DIRETRIZ §1.5.2.3)
cargo check -p ph2d-timeline
```

⚠️ **O rebase vai trazer ~88 commits de outras linhas** (Painter, FLIP, GPU-nodes, audio…) que
integraram enquanto o nesting rodava. É esperado — a linha está **0 commits à frente do main e
88 atrás** (integrou limpa). O 1º build pós-rebase é frio (minutos): não investigue.
Conflito em `Cargo.lock` ou arquivo GERADO (registry-init, `chrome/mod.rs`) → **regenere, nunca
resolva na mão**. Conflito em código fora dos seus arquivos → colisão de mesmo-símbolo → **PARE
e reporte ao Enio** (DIRETRIZ §1.5.5).

⚠️ **A worktree ainda está no HEAD pré-integração** (`8b4847dd`). O `git rebase main` é o que a
alinha. Não estranhe o log velho antes de rebasear.

---

## 1. Onde paramos — o nesting FECHOU e INTEGROU

A linha entregou o **nesting da timeline**
([ADR-0133](architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md)):
um objeto que por dentro é uma cena animada inteira e por fora é um objeto, com o relógio do
**pai**. Pesquisa → ADR → plano → Fatias 0–3f, **smokado e aprovado pelo Enio**, e **integrado
ao main** (commits do nesting em `main`; handoff de integração em
[`HANDOFF_INTEGRACAO_line_anim_nesting_2026-07-19.md`](HANDOFF_INTEGRACAO_line_anim_nesting_2026-07-19.md)).

⚠️ **A entrada do CLAUDE.md §5 (Timeline) no main ainda diz "aguardando integração"** para o
nesting — está desatualizada (a integração aconteceu). Não é bug seu; se for mexer no roteador,
corrija de passagem, mas não é obrigatório.

**A ideia central, para você não reconstruir o raciocínio:** o nesting **não precisou de
mecanismo novo**. O relógio (outer-then-inner + recusa nomeada) já existia; o z-order já estava
respondido (`SortingGroup`, Unity); o custo O(bindings²) já tinha sido pago (`ClockIndex`); e o
strip já carregava o mínimo universal de override por-instância. O trabalho foi **estender uma
cadeia**, não inventar uma segunda. Onde um número não podia ser invertido (o relógio do
container), a resposta foi publicar a **relação** (`container_map`), que serve às duas direções.

---

## 2. O que já está PRONTO (não reconstrua, não re-litigue)

A timeline está madura. O que **já landou** e você pode tratar como dado:

- **Transporte, régua, lanes, zoom/pan, box-select, copy/paste, `F`=fit** (W2).
- **Curva por-faixa, handles bézier, weighted tangents, speed graph, presets de easing** (W3/W5).
- **Time remap por-objeto** (o relógio do AE) — e os 3 bugs dele já fechados.
- **Roving keys, Performing/Record, simplificação de keyframes (Schneider)** (W5).
- **Seletor de clips + composição de clips** (ADR-0115: faixas, crossfade por sobreposição,
  canais esparsos, alça de fade, read-only onde o vizinho define a janela).
- **Relógio único** (W4.T7 — `MotionTransport` morreu; o Motion cozinha no tick do `Playhead`).
- **Save de projeto cena+timeline** (W4.T6/B5 — o `TimelineDoc` viaja no `ProjectFile`).
- **Undo global** cobrindo objetos+hierarquia+canvas.
- **Nesting** (ADR-0133) — esta linha, agora integrada.

**Regra que quebrou este módulo três vezes** ([[feedback_derived_coordinate_seed_must_match_sample]]):
autoria e leitura de uma coordenada derivada usam a **mesma** transform. O nesting a aplicou
recursivamente; a próxima feature de tempo vai encostar nela de novo.

---

## 3. Os planos a seguir (ranqueados, com o gatilho de cada um)

**Nada disto está em andamento.** A linha está fechada contra o seu plano — o próximo passo é
**ordem do Enio**. Abaixo, o que está aberto, honesto e com o critério que o acorda.

### 3.1 — Cauda da Timeline (o escopo natural desta linha)

Fonte: `HANDOFF_line_anim_integracao_2026-07-11.md` §"Cauda da W4" + CLAUDE.md §5. Já filtrei o
que **já landou** (T6/T7, composição de clips, nesting):

| Item | O que é | Estado / gatilho |
|---|---|---|
| **W4.T4 — docar a timeline** no `motion_timeline_slot` quando o split do Motion está ativo | coordenação leve com Motion | **DESBLOQUEADO** (Motion integrou). Pronto para ser pego. |
| **markers → signals** | markers da régua viram eventos disparáveis | ordem do Enio |
| **bake curves → keyframes** | assar uma curva procedural em keys editáveis | ordem do Enio |
| **export** | sair da timeline para um formato (Lottie/vídeo?) — desenho ainda não feito | precisa de ADR antes de código |
| **MCP / Luau** | dirigir a timeline por script | ordem do Enio |
| **Refinamentos do fit de keyframes** | corner broken-tangents · overshoot clamp p/ opacity · rotation unwrap · low-pass | deferidos no topo de `curve_fit.rs`; gatilho = smoke reclamar |

### 3.2 — Cauda do próprio NESTING (deferido de propósito, ver ADR-0133 §6 e plano §7)

| Item | Por que NÃO foi feito | O gatilho que o acorda |
|---|---|---|
| **Relógio próprio por container** (*movie clip*) | maior fonte de confusão documentada; mata o scrub determinístico | máquina de estados / interação em runtime — e aí é **opt-in por instância**, nunca default |
| **Cache de saída do container** | sem consumidor, cachear é escolher um modo de falha antes de ter o problema | o kill-criterion ser excedido por instâncias IDÊNTICAS (desconto = *Master Pose Component* do Unreal) |
| **Teto de profundidade** | **não temos o recurso que o justifique** (§0.0 — medido: custo linear); ninguém no mercado publica um medido | alguém medir memória/recursão de pilha e escrever o número |
| **Instância que dá a VOLTA tem mapa** | sob loop/pingpong um segundo do interior está em vários da timeline; escolher um em silêncio é o palpite que o módulo recusa | desenho: "a ocorrência mais próxima do playhead" — e aí precisa de gate próprio |
| **Loop próprio do container** | pegar emprestado o loop do documento seria um loop de outro relógio embrulhando uma pilha que ele não conhece | quando o container precisar repetir por dentro sem depender do pai |
| **Vetor/pintura DENTRO do container** | o interior deste ADR é **animação** | ordem do Enio |

### 3.3 — Dívida de teste conhecida (não bloqueia, mas está medida)

- **O lado do SAVE não tem gate.** `project_save` exige `gfx` (janela + GPU): o mundo mora
  dentro do `AppGfx`, e é isso que trava o harness headless do shell. O **load** já é gateado
  (`project_tests.rs`, o `App` nasce sem janela). Se for mexer em persistência, esse é o buraco.

---

## 4. Armadilhas ESPECÍFICAS desta linha (as que ela pagou caro)

- **`stack.rs`/`stack_eval.rs`/`snapshot.rs` vivem perto do cap de LOC (700).** O nesting já fez
  3 splits (`nest.rs`, `stack_frames.rs`, `ruler_clock.rs`, `nest_map.rs`). O gate de LOC mora na
  `ph2d-editor-core` e **não roda** com `cargo test -p ph2d-timeline` — rode na árvore combinada,
  e `cargo fmt` **antes** de medir (o fmt re-expande).
- **A ponte painel→shell é um thread-local carimbado ANTES de drenar intents** (o padrão
  `keys_mode` / `edit_path`). Toda feature nova de estado-de-vista do painel copia esse padrão;
  não invente um segundo canal.
- **`DOC_VERSION` é rejeição pura, sem migração** (postcard posicional, política da casa). Bumpou
  o doc? A versão velha é recusada no load, com toast. Não escreva maquinário de migração.
- **`PROJECT_SCHEMA` não bumpa quando só o blob interno muda de versão** — a forma do
  `ProjectFile` é que manda. Dois números para uma incompatibilidade seriam duas portas.
- **Gates que CLICAM, não que compilam** ([[feedback_widget_is_done_when_a_test_clicks_it]]): o
  seam do nesting arrasta a régua e clica a trilha de verdade. Copie esse rigor — "pintado ≠
  populado", e a 1ª rodada de mutação do nesting estava MENTINDO (o filtro `--test` não rodava as
  suites do painel).

---

## 5. Ao assumir, reporte e PARE

> *"Assumi `line/anim-fixes` em `Worktrees/line-anim` (HEAD `<sha>` após rebase). O nesting
> fechou e integrou; a linha está limpa contra o plano. Aguardo a tarefa."*

Não integre, não pushe (§0.7). A tarefa vem do Enio.

**Mapa de leitura (nesta ordem, DENTRO da worktree):**
1. este doc;
2. [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md) (a mecânica);
3. [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (releia a cada passo);
4. as **REGRAS PERMANENTES A–H** do [`MODELO_ABERTURA_LINHA.md`](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md);
5. se a tarefa for de timeline: `docs/Timeline/` (00 briefing · 01 UI · 02 clips · 03 pesquisa nesting · 04 plano nesting) e os `HANDOFF_line_anim_*` por data.
