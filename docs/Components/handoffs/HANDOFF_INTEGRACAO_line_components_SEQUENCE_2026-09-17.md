# HANDOFF — `line/components` · TOP-20 #19 `SequencePlayer` (W1+W2+W3) · 2026-09-17

> **O que o dono ganha:** a Timeline vira **cutscene de jogo**. Um objecto aponta a uma sequência
> pelo nome, e ela corre **no relógio dele** — não no da cena. Um sinal arranca-a, e o que ela
> escreve **não entra no `Ctrl+Z`**.
>
> ⚠️ **Leia a §4 antes do diff:** sete coisas que uma leitura rápida entende ao contrário.

## §1 — Os contadores, como DELTA

| contador | delta | porquê |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** | o componente novo (W1). ⛔ **Conte o delta contra o `main` em que aterra**, nunca o literal |
| registo do `ph2d-ecs` | **+1** | `SequencePlayer` |
| espelhos (`ph2d-render` · `ph2d-script`) | **+1** cada | eles contam `ecs + render` / `ecs + script` |
| `LIVE_SECTIONS` | **+1** (29 → 30) | a secção *Sequence* |
| `any_live_section` | **+1** (23 → 24) | a mesma secção |
| `EditorAction` | **+1** variante | `InspectorSequenceEdit`, **append-only** |
| contrato congelado (§6) | **0** | — |
| ADR | **0** | — |

## §2 — O que cada wave entregou

| wave | o quê | onde |
|---|---|---|
| **W1** | o componente + a lei do NOME (`resolve`) + o descritor que **EXIGE** os `Timers` | [`sequence.rs`](../../../crates/ph2d-ecs/src/sequence.rs) |
| **W2** | *quais correm e em que instante* (`em_corrida`) + a FASE que as aplica pelo ledger | [`fase_sequences.rs`](../../../shells/desktop/src/render_loop/fase_sequences.rs) |
| **W3** | a **secção do Inspector**, a **cena** e as duas portas da vista | [`sections/sequence.rs`](../../../crates/ph2d-panel-inspector/src/sections/sequence.rs) · [`sequence_smoke.rs`](../../../shells/desktop/src/sequence_smoke.rs) |

**Gates:** 11 (lei) + 7 (fase, 3 deles da porta da vista) + 13 (instantâneo/dreno) + 5 (costura com
**clique real**) + 1 (tecto). **Mutação: 5 de 5 (W2) e 12 de 12 (W3)** —
[`mutacao_seq_w2.sh`](../ferramentas/mutacao_seq_w2.sh) ·
[`mutacao_seq_w3.sh`](../ferramentas/mutacao_seq_w3.sh).

## §3 — A superfície de colisão

Ficheiros **partilhados** que esta wave toca e onde um merge textual pode colidir:

* `crates/ph2d-editor-core/src/action_bus.rs` — variante **apendada**;
* `crates/ph2d-editor-core/src/ids/live_sections.rs` — a 30.ª entrada, **no fim** da tabela;
* `crates/ph2d-editor-core/src/ids/inspector_camera.rs` — dois `const` novos, no fim de cada bloco;
* `crates/ph2d-i18n/src/inspector_game.rs` — 15 chaves, antes do `ph2d-migrar-texto:end`;
* `crates/ph2d-panel-inspector/src/{paint,paint_optional,paint_frame,paint_frame_snapshots,popovers,event,populate,ids,state,state_components,state_popovers}.rs`
  — uma a duas linhas cada;
* `crates/ph2d-panel-timeline/src/{state,state_requests,paint}.rs` — a porta `request_arrange_tab`,
  **append-only** (nenhuma assinatura existente mexida);
* `shells/desktop/src/render_loop/{fase_bus_inspector,fase_bus_drain_out,fase_hero_commits,fase_inspector_commits,fase_inspector_commits_top20,fase_snapshots_publish,timeline_bridge}.rs`;
* `shells/desktop/src/{main,app_state,app_state_components_smokes}.rs`.

⚠️⚠️ **`any_live_section` é um `[bool; N]`** — duas linhas a acrescentarem uma secção na mesma
rodada fundem **limpo** e a arity fica errada por um. *Conte os `is_some()` do chamador, no
`paint_frame_snapshots.rs`.*

## §4 — ⚠️ Sete coisas que uma leitura rápida do diff entende ao contrário

1. **O `SequencePlayer` não tem relógio, e a ausência é a wave.** O relógio de corrida é o `Timer`
   que já existia (duração · repetir · `autostart` · sinal por disparo · `rewind_runtime`). Um
   `SequenceRuntime` seria um **segundo** relógio, e um verbo `PlaySequence` uma segunda maneira de
   dizer *«começa»*.
2. **A `ph2d-app-components` continua a NÃO ver a timeline.** O instantâneo recebe os nomes e as
   durações **prontos**; quem os colhe do documento é a fase-filha da shell, que vê as duas coisas.
3. **O tecto do selector é um literal `16` no painel, e isso é o padrão da casa** (`PROPS_MAX`,
   `STATES_MAX`): o gate que o ata ao `ph2d_timeline::MAX_CONTAINERS` vive na **shell**, a única
   crate que vê as duas. ⛔ Importar a timeline para dentro do painel poria o documento da animação
   na closure de compilação de um painel que, por desenho, não o conhece.
4. **`a_vista_deixa_correr` não é uma função solta: é uma PORTA com dois leitores** — o
   `timeline_bridge` e o publicador do instantâneo. Há **dois censos** a afirmá-lo, e a razão é o
   modo de falha: escrita duas vezes, o painel prometeria uma coisa e o motor faria outra.
5. **`request_arrange_tab` vence um `request_keys_tab` pendente**, e a ordem no `publish_view` é o
   que o exprime — sem um campo de prioridade. Uma cena que escolhe um objecto **e** pede Arrange
   quer a segunda coisa.
6. **A cena NÃO põe nada na faixa da timeline, e isso é load-bearing.** O *Arrange* com a pilha
   vazia **não toca nada** (ordem do dono, 2026-07-27), logo tudo o que se vê a mexer veio do
   `SequencePlayer`. *Com uma instância na faixa, a cena ensinaria o contrário do que diz.*
7. **A cena CONFERE-SE a si mesma**: mede a excursão das duas portas durante 330 quadros e imprime
   o veredito. ⚠️ Não é um `eprintln` decorativo — **a foto não decide esta pergunta** (ela mostra
   um instante, e *«mexeu-se»* é uma propriedade de um intervalo), e sem ele o defeito da §5.1
   teria ido ao dono.

## §5 — ⛔ As quatro premissas minhas que a medição derrubou

1. *«a foto é o oráculo do lado pintado»* — é, **para o que é estático**. Ela mostrou a cena certa
   três vezes enquanto a cutscene **não corria**.
2. *«deixar de ABRIR a timeline chega»* — não chega: ela vem aberta do `~/.ph2d/layout.txt` do dono.
3. *«esconder a timeline é a cura»* — é **uma** cura, e a errada: tira a régua do tempo ao artista
   exactamente quando ele a quer ver. A cura é **pedir a aba**.
4. *«o gate do relógio curto cobre a folga»* — cobria a DIRECÇÃO e não a GRANDEZA: um empate exacto
   não distingue `<` de `+1e-3 <`, e a mutação que apagava a folga **sobreviveu**.

### §5.1 — ⛔⛔⛔ O achado: a aba de OMISSÃO parava TODAS as cutscenes

`Tab::Keys` é `#[default]` **e** é a que escolher um objecto **PEDE**
(`selection_jumps_to_keys`) — e ali o `keys_mode` é `true`, logo a condição da W2 não deixa a fase
correr. ⇒ duas portas paradas com todos os dados certos, e o artista a concluir que o componente
está partido. ⚠️ **A fronteira que fica é de PRODUTO e está documentada:** na aba *Keys* as
cutscenes pausam (ali sola-se a clip que se edita), e **o painel di-lo** em vez de a deixar muda.

## §6 — O que só a árvore COMBINADA pode reprovar

* o **censo de texto** (HR-15) sobre as 15 chaves novas;
* os **tectos de LOC** por acumulação — esta wave deixa o `app_state.rs` **exactamente** no tecto
  (`976`), que foi onde a wave do HUD o pôs;
* a catraca `the_shell_only_shrinks`. ⚠️ **A cena de smoke e a fase-filha somam ~390 linhas à
  shell**, e a razão de elas ficarem lá é a mesma do `nest_smoke`: a cena **autora o documento da
  timeline**, que vive na `App`.

## §7 — Como smokar

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_SEQUENCE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

Diagnóstico: `PH2D_SEQUENCE_LOG=1` (a linha da fase + os três elos da auto-conferência).
