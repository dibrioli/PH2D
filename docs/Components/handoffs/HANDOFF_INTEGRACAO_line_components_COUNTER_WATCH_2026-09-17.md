# HANDOFF — `line/components` · a VIGIA DO CONTADOR · 2026-09-17

> **O que o dono ganha:** um NÚMERO passa a fazer acontecer alguma coisa. Três vidas chegam a zero
> e o jogo sabe. Dez moedas abrem a porta. Até hoje o app sabia **contar** e **mostrar** a conta, e
> mais nada.
>
> ⚠️ **Leia a §4 antes do diff:** sete coisas que uma leitura rápida entende ao contrário.

## §1 — Os contadores, como DELTA

| contador | delta | porquê |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** | o `CounterWatch`. ⛔ **Conte o delta contra o `main` em que aterra**, nunca o literal |
| registo do `ph2d-ecs` | **+1** | `ph2d::ecs::CounterWatch` |
| espelhos (`ph2d-render` · `ph2d-script`) | **+1** cada | eles contam `ecs + render` / `ecs + script` |
| `LIVE_SECTIONS` | **+1** (30 → 31) | a secção *Counter Watch* |
| `any_live_section` | **+1** (24 → 25) | ⚠️ é um `[bool; N]` — ver §3 |
| `SignalOrigin` | **+1** variante, **append-only** | `CounterWatch { source, row }` |
| `EditorAction` | **+1** variante, **append-only** | `InspectorCounterWatchEdit` |
| contrato congelado (§6) | **0** | — |
| ADR | **0** | a forma é a do `Timer`; nenhuma decisão de arquitectura nova |

## §2 — O que a wave entregou

| peça | onde |
|---|---|
| a **porta** da soma (dois leitores: o placar e a regra) | `crates/ph2d-ecs/src/counter.rs` |
| a **lei** da aresta | `crates/ph2d-ecs/src/counter_watch.rs` |
| a **ponte** (reconcilia, avalia, conta as órfãs) | `crates/ph2d-app-components/src/counter_watch_bridge.rs` |
| o **instantâneo** e o dreno | `crates/ph2d-app-components/src/counter_watch_inspector.rs` |
| a **secção** do Inspector | `crates/ph2d-panel-inspector/src/sections/counter_watch.rs` |
| a **cena** | `crates/ph2d-app-components/src/counter_watch_smoke.rs` + o prólogo na shell |

**Gates:** 8 (lei) + 1 (rebobinar) + 4 (ponte) + 5 (costura com clique **REAL**) + 2 (tecto/ordem,
na shell) + 6 (cena) + **2 (a corrente inteira, sem janela)**.
**Mutação: 25 de 25** — [`mutacao_counter_watch.sh`](../ferramentas/mutacao_counter_watch.sh).

## §3 — A superfície de colisão

Ficheiros **partilhados** que esta wave toca:

* `crates/ph2d-editor-core/src/action_bus.rs` — variante **apendada**. ⚠️ **O ficheiro fica em
  `700` de `700`**: a próxima variante **paga o corte**, e a prescrição já tem o preço medido
  (`160` sítios em `58` ficheiros — é uma wave própria);
* `crates/ph2d-editor-core/src/ids/live_sections.rs` — a 31.ª entrada, **no fim**;
* `crates/ph2d-editor-core/src/ids/inspector_camera.rs` — dois `const`, no fim de cada bloco;
* `crates/ph2d-editor-core/src/lib.rs` · `crates/ph2d-ecs/src/lib.rs` — `mod` novos;
* `crates/ph2d-ecs/src/{hud,rewind_runtime,scene/registry}.rs` — o `valor` passa pela porta, a
  vigia entra no rebobinar, o registo ganha uma linha;
* `crates/ph2d-i18n/src/inspector_game.rs` — 19 chaves, antes do `ph2d-migrar-texto:end`;
* `crates/ph2d-panel-inspector/src/{paint,paint_optional,paint_optional_top20,paint_frame,paint_frame_snapshots,popovers,event,populate,ids,state,state_components,state_popovers}.rs`;
* `crates/ph2d-runtime/src/lib.rs` + o censo exaustivo dela;
* `shells/desktop/src/render_loop/{fase_bus_inspector,fase_bus_drain_out,fase_hero_commits,fase_inspector_commits,fase_inspector_commits_top20,fase_signal_outbox,fase_signal_log,fase_snapshots_tardios,motores_do_quadro,mod,fase_app_scene_smokes}.rs`;
* `shells/desktop/src/{main,app_state,app_state_components_smokes,project_schema,project_schema_tests}.rs`.

⚠️⚠️ **`any_live_section` é um `[bool; N]`** — duas linhas a acrescentarem uma secção na mesma
rodada fundem **limpo** e a arity fica errada por um. *Conte os `is_some()` do chamador, no
`paint_frame_snapshots.rs`.*

## §4 — ⚠️ Sete coisas que uma leitura rápida do diff entende ao contrário

1. **O `Health` não foi construído, e a ausência é a wave.** Com a vigia ele é **composição**
   (`Counter{start:3}` + `AddToCounter(-1)` + `CounterWatch[≤0 → "morri"]`), e um componente
   próprio seria a segunda resposta a *«quanto vale este número?»*.
2. **A lei é a ARESTA e não um knob.** Uma comparação avaliada por quadro é um **nível**, e um
   nível dispara **por quadro** — as referências separam as duas coisas (o *Trigger once while
   true* do Construct, o *Do Once* do Unreal). O `once` é um **segundo** knob por cima da aresta.
3. **`counter::soma` não é um helper: é uma PORTA com dois leitores** — o `UiLabel` e a vigia.
   Escrita duas vezes, o placar mostraria um número e a regra reagiria a outro, e as duas contas
   concordariam em toda cena com **um** contador só.
4. **`None` e `Some(0)` são coisas diferentes, e é uma LEI herdada.** Uma vigia sobre um contador
   que não existe **nunca dispara** — sem isso, escrever `vidaas` num campo anunciaria *«morreste»*
   no arranque.
5. **A origem do sinal leva o ÍNDICE DA REGRA e não o nome do contador**, e isso foi **imposto**:
   o `SignalOrigin` é `Copy` e um `Arc<str>` tirava o `Copy` a **todas** as origens.
6. **A vigia fala com os MOTORES (antes da tabela de acções) e a latência é a mesma nas duas
   margens** — medido: antes, vê o valor de ontem e reage hoje; depois, vê o de hoje e reage
   amanhã. `1` quadro nos dois ⇒ fica junto dos irmãos, que é a margem que o censo já cobre.
7. **As TRÊS LUZES da cena são três REGRAS sobre o MESMO contador** (`≤2`, `≤1`, `≤0`), e não três
   contadores. É isso que mostra, sem um número, que uma vigia dispara **a um limiar**.

## §5 — ⛔ As quatro premissas minhas que a medição derrubou

1. *«o buraco é o `Health`»* — o buraco é **reagir a um número**; o `Health` cai de graça.
2. *«a origem do sinal pode levar o nome do contador»* — não pode: o enum é `Copy`.
3. *«o placar de texto é a maneira de mostrar a contagem»* — um `UiLabel` **troca o que um texto
   MOSTRA; ele não cria o texto**. Sem um `VecShape::Text` por baixo ele desenha um **anel vazio**.
4. *«os gates da cena cobrem a cena»* — eles perguntavam o que ela **MONTA** e nenhum perguntava
   **ONDE**: os rótulos aterraram a `−358` no mundo com os cinco **verdes**.

### §5.1 — ⛔⛔⛔ Os três achados da CORRENTE INTEIRA

O gate `render_loop::counter_watch_chain_tests` percorre
`relógio → AddToCounter → contador → vigia → sinal → Hide` **sem janela nenhuma**, e reprovou três
vezes — nenhuma das causas na lei:

1. **A IDENTIDADE.** O `resolve_signal_actions` consulta `(Entity, &SignalActions, &StableId)` ⇒
   **um mundo sem identidade não reage a sinal nenhum e não o diz**. É a família do defeito que as
   Tags pagaram em 14/09. *Um arnês que salte o passo da shell mede outro programa.*
2. **A `Visibility`.** O verbo `Hide` lê a visibilidade de ANTES para a declarar ao ledger, logo um
   alvo **sem** o componente é **inerte** — e o relatório conta-o como `inert` **sem gritar**.
3. ⭐⭐⭐ **UM SINAL É GLOBAL POR NOME.** Com os dois à escuta de `"morri"`, o controlo
   **desaparecia junto com o herói**. ⇒ ele espera por `"morri_controlo"`, que ninguém diz nesta
   cena. *É esta a forma certa de um controlo: falta-lhe quem DIGA, não quem faça.*

## §6 — O que só a árvore COMBINADA pode reprovar

* o **censo de texto** (HR-15) sobre as 19 chaves novas;
* os **tectos de LOC** por acumulação — ⚠️ o `action_bus.rs` fica **exactamente** no tecto (`700`);
* a catraca `the_shell_only_shrinks` — a wave acrescenta ~150 linhas à shell (o prólogo da cena e o
  gate da corrente, que **têm** de lá viver: o `signal_actions` e o `timer_tick` são privados ao
  `render_loop`, e alargá-los a `pub(crate)` seria a quinta agulha deste repo a nomear a
  VISIBILIDADE em vez da lei).

## §7 — Como smokar

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_COUNTERWATCH_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

A cena **confere-se a si mesma** sobre 300 quadros e imprime o veredito:
`heroi: chegou a 0, 3 luz(es) apagada(s) (tem de ser 3), SUMIU · controlo: chegou a 0, 0 luz(es)
apagada(s) (tem de ser 0), ficou · veredito: SIM`.

Diagnóstico: `PH2D_SIGNAL_LOG=1` (a linha `[signal] morri <- vigia do objecto N, regra #2`).
