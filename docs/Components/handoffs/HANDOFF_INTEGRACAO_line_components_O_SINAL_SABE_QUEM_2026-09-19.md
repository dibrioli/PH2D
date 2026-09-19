# HANDOFF — `line/components` · suplente #24 (2.ª metade) · **O SINAL SABE QUEM** · 2026-09-19

> **O que o dono ganha:** um tiro num inimigo mata **aquele** inimigo. Até aqui a tabela de acções
> ouvia um NOME e não sabia de quem ele tinha vindo — dez inimigos iguais com a mesma linha davam
> **dez efeitos para um tiro** (medido). A linha passa a responder a **três** perguntas em vez de
> duas — *quando · **de quem** · a quem · o que faz* — e ganha o verbo que **tira da cena**.
>
> ⚠️ **Leia a §5 antes do diff:** sete coisas que uma leitura rápida entende ao contrário.

## §1 — Os contadores, como DELTA

| contador | delta | porquê |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (`148 → 149`) | o campo `from` na `SignalAction`; o postcard é **posicional**, logo sem o degrau um ficheiro velho seria lido errado **em silêncio**. ⛔ **Conte o delta contra o `main` em que aterra**, nunca o literal — a linha inteira carrega `144 → 149` |
| registo do `ph2d-ecs` e os dois espelhos | **0** | ⭐ **nenhum componente novo**: `SignalFrom` é um CAMPO de um componente que já existia, e o `Disparo` é um tipo de chamada que nunca entra num mundo |
| `LIVE_SECTIONS` · `any_live_section` | **0** | a secção *Signal Actions* existe desde o `#5` |
| `SignalVerb::ALL` | **+1** (`8 → 9`) | `Destroy`, **apendado** — ⚠️ a POSIÇÃO NO ARRAY É A TAG do segmentado |
| `SignalTarget` | **+1** variante | `Other`, apendada depois de `Tagged` |
| `DeathCause` | **+1** variante | `Killed`, apendada |
| `ActionFieldEdit` (`EditorAction`) | **+2** variantes | `TargetMode(u8,u8)` · `From(u8,u8)`, **append-only** |
| contrato congelado (§6) | **0** | — |
| ADR | **0** | — |

## §2 — O que cada wave entregou

| wave | o quê | onde |
|---|---|---|
| **sonda** | a medição do §5.0, **antes da 1.ª linha**: `10` efeitos para um tiro · nenhum alvo diz *«quem»* · `8` verbos e nenhum tira da cena · dois contadores com o mesmo nome somam ao MESMO sítio | [`mede_o_que_a_composicao_ja_da_ao_golpe.rs`](../../../crates/ph2d-ecs/tests/it/mede_o_que_a_composicao_ja_da_ao_golpe.rs) |
| **W1** | `SignalFrom { Anyone, Myself }` + o `Disparo { nome, quem, outro }` + o `resolve` a receber **disparos** em vez de nomes | [`signal_actions.rs`](../../../crates/ph2d-ecs/src/signal_actions.rs) |
| **W2** | `SignalOrigin::quem()` / `::outro()` — a porta ÚNICA de *«quem falou?»*, **exaustiva** | [`ph2d-runtime/src/lib.rs`](../../../crates/ph2d-runtime/src/lib.rs) |
| **W3** | o verbo `Destroy` (que **anuncia** e nunca apaga) + o `lido()` da ponte | [`signal_actions_bridge.rs`](../../../crates/ph2d-app-components/src/signal_actions_bridge.rs) |
| **W4** | a coluna **«de quem?»** no painel + o 3.º modo de alvo + a ponte a descer para a família | [`actions_editor.rs`](../../../crates/ph2d-panel-inspector/src/sections/actions_editor.rs) |
| **W5** | a **cena** (`PH2D_DANO_SMOKE=1`) e a cura do instrumento que a fotografa | [`dano_smoke.rs`](../../../crates/ph2d-app-components/src/dano_smoke.rs) |

**Gates:** 8 (a cerca e o outro lado) + 4 (a porta da origem) + 2 (o `Destroy` na ponte) + 2 (a
ida-e-volta do painel) + 1 (a ordem da fase) + 6 (a cena) = **23**.
**Mutação: 18 de 18 sangram** — [`mutacao_golpe_2026-09-19.sh`](../ferramentas/mutacao_golpe_2026-09-19.sh).

## §3 — A superfície de colisão

Ficheiros **partilhados** que esta wave toca e onde um merge textual pode colidir:

* `shells/desktop/src/project_schema.rs` **e** `project_schema_tests.rs` — ⚠️ **a escada e a tripla
  são ficheiros IRMÃOS**, e um degrau escrito no errado funde **limpo** e evapora;
* `crates/ph2d-ecs/src/signal_actions.rs` — o `SignalAction` ganha um campo **no fim** da struct;
* `crates/ph2d-runtime/src/lib.rs` — dois métodos novos, **nenhuma variante**;
* `crates/ph2d-editor-core/src/screens/hero/inspector_model_action.rs` — duas variantes apendadas
  ao `ActionFieldEdit` e dois campos novos na `InspectorActionRow`;
* `crates/ph2d-panel-inspector/src/{event_action,populate_action,ids/inspector_action,sections/actions,sections/actions_editor}.rs`;
* `crates/ph2d-i18n/src/inspector_game.rs` — 7 chaves, antes do `ph2d-migrar-texto:end`;
* `shells/desktop/src/render_loop/{fase_signal_outbox,fase_tabela_de_accoes,mod}.rs`;
* `shells/desktop/src/{components_scenes,app_state_components_smokes}.rs`.

⚠️⚠️ **O `INSP_ACTION_VERB` é um `[NodeId; 9]` e a POSIÇÃO é a tag.** Duas linhas a acrescentarem um
verbo na mesma rodada fundem **limpo** e um clique passa a escrever o verbo do vizinho. *Conte os
braços do `SignalVerb::ALL`, nunca a arity do array.*

⭐ **A ponte MUDOU DE CASA e é isso que o diff mostra como «apagado + criado»:**
`shells/desktop/src/render_loop/signal_actions.rs` →
`crates/ph2d-app-components/src/signal_actions_bridge.rs`, com os dois ficheiros de gates dela.
A causa foi a catraca `the_shell_only_shrinks` (`197 340` contra `196 990`) e a cura é **MOVER**.

## §4 — O que só a árvore COMBINADA pode reprovar

1. **A catraca da shell** — ela mede a SOMA: outra linha a crescer `400` linhas põe-na vermelha com
   esta já dentro do orçamento. A cura é **corte por responsabilidade**, nunca subir o número.
2. **O `PROJECT_SCHEMA`** — se outra linha da rodada também subir, o degrau desta **renumera-se**.
   ⛔ A coluna `base:` do `collision-surface.sh` é o **merge-base**, não o `main` de agora:
   `python3 scripts/schema-recount.py`.
3. **Os 30 censos de texto do HR-15** — eles só correm no `ship.sh` e sobre a árvore combinada.
4. **`architecture_no_dependency_climbs_a_layer`** — ver a §5.6.

## §5 — ⚠️ Sete coisas que uma leitura rápida do diff entende ao contrário

1. **A cerca é do REACTOR, não do alvo — e o `SignalTarget::Speaker` SAIU antes de chegar ao
   painel.** Com `From Myself`, quem reage **é** quem falou, logo o alvo vazio já o exprime; um
   `Speaker` faria os dez reactores aplicarem o verbo ao MESMO sujeito (`−10` numa vida só). *Um
   alvo sem consumidor é um controlo morto com cara de feature.*
2. **Um sinal SEM sujeito nunca passa o `Myself`, e isso é a lei.** `None` não é curinga: lê-lo como
   *«qualquer um»* faria uma cerca **fechada** deixar passar tudo, e o modo de falha é **MUDO**.
   Há mutação dedicada só a essa leitura.
3. **A linha reage a um DISPARO, não a um NOME — e é uma mudança OBSERVÁVEL declarada.** Dois
   eventos com o mesmo nome no mesmo quadro davam **1** efeito e passam a dar **2** (é a cura de
   *«duas moedas num quadro contavam uma»*). ⛔ Ela **não** é o colapso `fires`/`cycles`/`rows`, que
   é dentro de UM produtor e continua igual.
4. **O `Destroy` ANUNCIA e nunca apaga.** Ele produz uma `Death { DeathCause::Killed }` e quem
   remove é o dreno da `fase_fabrica_e_morte`, por ÚLTIMO no quadro — *«quando é que isto sai da
   cena?»* é uma pergunta só. Um moribundo continua visível a toda consulta até ao fim do quadro.
5. **A fronteira do `Destroy` é FORÇADA, não escolhida:** só sai quem nasceu numa corrida
   (`is_transient`). Apagar um objecto de DOCUMENTO tira-o do documento e o `Ctrl+Z` **herda** a
   remoção. ⚠️ **E a recusa NÃO virou aviso de linha**, com mecanismo: um `Destroy` sobre um objecto
   de documento **está certo** — ele é a receita, e as cópias que a `Factory` faz dele são
   transitórias. *O mesmo alvo muda de resposta entre o original e a cópia.*
6. **O SOM entra por INJECÇÃO, e não é estilo.** Com a ponte na família, `ph2d-app-components →
   ph2d-app-audio` é Família → Família e o `architecture_no_dependency_climbs_a_layer` **recusa** a
   aresta; a cura que ele prescreve por escrito é *«uma tabela para ser injectada pela composição»*
   ⇒ o `apply` pede *«toca o som deste objecto»* por um fecho e a SHELL responde. A tabela de acções
   deixou de saber o que é um mixer.
7. **O prefixo `fase_` da fase-filha é LOAD-BEARING.** O texto emendado do quadro colhe **só** as
   `fn fase_*` do `render_loop/`, e uma fase-filha com outro nome desaparece do oráculo de **toda**
   lei de ordem desta shell, em silêncio. O corte foi imposto pelo tecto de FUNÇÃO (`232/200`).

## §6 — ⛔ As premissas que a medição derrubou

1. **«O teto de LOC da shell aguenta»** — não aguentava: a catraca reprovou e a ponte do
   `SignalActions` era a **única das oito** desta família ainda na shell (as sete irmãs já viviam na
   crate). *A catraca apontou a coisa certa.*
2. **«O `Destroy` só precisa de um braço no `apply`»** — precisa de um DESPACHANTE, senão a remoção
   compete com os dois drenos de morte que já existem.
3. **«A foto confirma a cura»** — ⛔⛔⛔ **ela confirmou o defeito JÁ CURADO, duas vezes**: o
   `fotografa_cena.sh` **não constrói**, e as duas corridas mediram o binário de antes da cura — a
   segunda até reproduziu o suicídio no log, à letra. *Uma sonda que mede outro programa é pior do
   que nenhuma: ela não fica em silêncio, ela CONFIRMA.* ⇒ o roteiro passa a **RECUSAR** quando há
   `.rs` mais novo que o binário, com o nome do ficheiro na mensagem (controlo positivo **e**
   negativo corridos).
4. **«Um `SpawnAt::Area` espalha os alvos»** — espalha, e a semente pôs dois a **tocarem-se**. Cada
   alvo é um sensor que grita o golpe dele, cada um ouve o seu, e a cerca `Myself` fez
   **exactamente o que promete**: mataram-se no primeiro quadro (`0 entities`). ⇒ pontos MARCADOS
   (`Pick::Cycle`) **e** um `SignalTagFilter` da bala — *as duas metades, as duas com gate*.
5. **«O roteiro só tem de descrever a lei»** — ele tem de ser **produzível**: com o herói em `y = 0`
   o tiro recto passava **entre** as duas fileiras. Hoje ele nasce à ALTURA da fileira de cima e o
   passo 2 é **uma tecla**.

## §7 — O que fica ABERTO (e de quem é)

| item | de quem | mecanismo |
|---|---|---|
| **morte de pré-visualização** (matar um objecto de DOCUMENTO e o rebobinar devolvê-lo) | **dono** | dois leitores novos (desenho · física) + reposição; hoje a recusa é em voz, com o nome do objecto |
| **uma vida POR inimigo** | **dono** | o `Counter` soma por NOME em todo o mundo; vida por-objecto pede que a porta do contador saiba de quem é |
| `SignalFrom::Tagged` (*«só se quem falou pertence à tag X»*) | — | ⛔ **sem consumidor** — um modo sem sink é um controlo morto com cara de feature |
| o **gesto** do smoke (carregar no `Q`, acertar num alvo) | **dono** | os eventos sintéticos não chegam à Xwayland virtual e o `ydotool` move o rato REAL: os gates cobrem a LEI e os DADOS, e o gesto é dele |

## §8 — Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_DANO_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

O roteiro sai impresso em **seis passos** (`[dano-smoke]`). Diagnóstico: `PH2D_SIGNAL_LOG=1` diz a
origem de cada sinal, e com ele a `fase_fabrica_e_morte` imprime `[fabrica] N copia(s) nascida(s)`.
