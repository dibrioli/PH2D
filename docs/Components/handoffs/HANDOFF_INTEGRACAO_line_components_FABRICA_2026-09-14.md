# HANDOFF DE INTEGRAÇÃO — `line/components`, a FÁBRICA e o CICLO DE VIDA (TOP-20 #11 e #12)

> **2026-09-14**, a seguir ao [handoff das TAGS](HANDOFF_INTEGRACAO_line_components_TAGS_2026-09-14.md)
> do mesmo dia. Plano: [`09_plano_spawner.md`](../09_plano_spawner.md).
> ⛔ **Não integrado e não enviado** (§0.7) — a linha fecha, entrega isto e espera.

---

## §1 — O que o artista passa a conseguir fazer

1. **Uma receita nasce sozinha na cena** — um objecto com *Factory* põe cópias de um mestre a
   correr: aqui, espalhadas numa área, ou **num ponto marcado com uma tag**.
2. **Um sinal faz nascer** — a fábrica escuta o mesmo barramento que a tabela *Signal Actions*, logo
   qualquer produtor (um relógio, uma colisão, um marcador da timeline) a acorda.
3. **O que nasce MORRE sozinho** — por tempo (*Lifetime*) ou por sair do ecrã do jogo
   (*Destroy Outside*).
4. **Limites que ele lê**: quantas vivas ao mesmo tempo, quantas ao todo, e um sinal quando a
   fábrica se esgota — mais o número de **vivas agora**, no painel.
5. ⭐⭐⭐ **E nada disto suja o projecto**: o que nasce numa corrida **não entra no ficheiro**, **não
   entra no `Ctrl+Z`** e **desaparece quando o relógio volta ao princípio**.

---

## §2 — Os contadores, em DELTA (⛔ nunca o literal — `CLAUDE.md` §5.0)

| contador | base (o commit das TAGS) | a linha | delta |
|---|---:|---:|---:|
| `PROJECT_SCHEMA` | 129 | 130 | **+1** |
| a tripla de `project_schema_tests` | `(129, 13, 22)` | `(130, 13, 22)` | **+1 / 0 / 0** |
| registo do `ph2d-ecs` (`registry_tests`) | 86 | 89 | **+3** |
| espelho do `ph2d-render` | 87 | 90 | **+3** |
| espelho do `ph2d-script` | 87 | 90 | **+3** |

⚠️ **O degrau é `+1` e os componentes são TRÊS**, e a diferença é a lei da wave: o `Spawned` — o
quarto tipo — **não é registado**, porque ele marca o que uma corrida pôs na cena e o
`world_to_snapshot` **poda** essas subárvores. *O número mede o que o FICHEIRO passa a conter.*

⛔ **Catracas que DESCERAM neste fecho:**

| catraca | antes | agora |
|---|---:|---:|
| `the_app_only_sheds_fields::TETO_CAMPOS` | 183 | **180** |
| `fn_loc_caps` `main.rs::new` | 233 | **226** |
| `file_loc_caps` `app_state.rs` | 1019 | **998** |

⚠️ **E duas que ficaram onde estavam, cada uma com a sua cura:**

- `architecture_the_foundation_modules_form_a_dag`, aresta `action_bus → screens`, tecto **24** — o
  vocabulário desceu para `ph2d_editor_core::factory_edits` (o **segundo** degrau da migração que a
  wave das tags começou no mesmo dia; sem ele a aresta ia a **25**).
- `hr15_no_hardcoded_ui_strings` — os quatro literais da secção nova foram para a **tabela**
  (`ph2d-i18n` ganhou o módulo `factory`), nunca para a lista de dívida.

---

## §3 — Crates novas

**Nenhuma.** Tudo entra em crates que já existiam:

| onde | o quê |
|---|---|
| `ph2d-ecs` | `factory.rs` (a lei da fábrica) · `lifetime.rs` (o ciclo de vida + `is_transient`) · a porta em lote no `instantiate.rs` · a poda no `save.rs` e no `snapshot.rs` |
| `ph2d-runtime` | duas origens de sinal (`Spawned` · `Death`) — **append-only** |
| `ph2d-unique-name` | `unique_names` (a porta em lote dos nomes) |
| `ph2d-component-desc` | três descritores na família `Logic`, com `requires` |
| `ph2d-editor-core` | `factory_edits` (o vocabulário, abaixo do `action_bus`) + dois ids de secção |
| `ph2d-i18n` | o módulo `factory` |
| `ph2d-panel-inspector` | as duas secções, os ids, o registo, o despacho e as molduras |
| `ph2d-app-components` | `factory_bridge` (a ponte) · `factory_smoke` (as duas cenas) |
| `shells/desktop` | a fase-filha `fase_fabrica_e_morte`, o instantâneo, o dreno e o prólogo |

---

## §4 — Os achados, cada um apanhado por um instrumento diferente

### §4.1 — ⭐⭐⭐ O preço de NASCER era função da CENA, e ninguém o via

A sonda (`measure_spawn_and_death.rs`) mediu a porta de cópia por tamanho de cena: `4,26 µs` a 100
objectos, `18,09` a 10 000 e **`143,21` a 100 000**. A causa está lida no código: `deep_copy_subtree`
chamava `assign_missing_stable_ids` **duas vezes por cópia**, e essa porta varre o mundo inteiro
(`2 × 69,8 = 139,6` de `143,2` — o preço de uma cópia **É** as duas varreduras).

⇒ `deep_copy_subtree_many` paga a identidade **uma** vez: o preço passa a ser **plano** (`2,58`–
`3,25 µs` em toda a tabela), que é a forma do oráculo e **mais barato** que ele.

⚠️ **E a medição da porta de CÓPIA não descrevia o produto:** o caminho do artista é
`instantiate_master_many`, que fazia mais quatro passagens `O(mundo)` por cópia (`unique_name` —
`O(n² × nomes)` sozinho — e as três `assign_*`). *Uma sonda que mede um sucedâneo para sempre mede
outro programa*, e a armadilha apareceu **no mesmo dia** em que o doc da sonda irmã a nomeia.

### §4.2 — ⭐⭐ A lei do transiente tem DOIS leitores que não se conhecem

O documento (`world_to_snapshot`) e a lista (`build_hierarchy_snapshot`). Uma porta
(`ph2d_ecs::is_transient`), e gates com **controlo** nos dois: *o que tem a marca não entra; o que
não a tem entra*. ⛔ Sem o controlo, uma poda que saltasse **tudo** ficaria verde e o produto
gravaria ficheiros vazios.

### §4.3 — ⚠️ A fase-filha tinha de se chamar `fase_*`, e a falha sem isso é MUDA

O texto emendado do quadro (`frame_text::render_frame`) — o oráculo de **toda** lei de ordem desta
shell — colhe **só** as `fn fase_*` do `render_loop/`. Uma fase-filha com outro nome desaparece dali,
e as leis que a atravessam deixam de ser medidas **sem um único teste ficar vermelho**.

### §4.4 — ⛔ Uma mutação que apaga um TECTO faz o teste alocar o que o tecto impedia

A prova do `BURST_MAX` tinha a fixtura em `u32::MAX`: com o tecto apagado, o binário chegou a
**27 GB de RSS** antes de ser morto à mão. *A fixtura de um tecto põe-se **logo acima** dele.*

### §4.5 — ⚠️ Uma fixtura que não contém o fenómeno deixa a lei sem gate

`100 000 µs` não distingue `>=` de `>` (o relógio chega a `100 002` e os dois matam no mesmo tique).
A duração passou a ser um **múltiplo exacto** do tique.

### §4.6 — ⚠️ A lei «só morre quem nasceu» tem DOIS guardas, e nenhum é observável sozinho

O `With<Spawned>` do reconcile e o `&Spawned` da query. Mutar um devolve *«SOBREVIVEU»* sobre um
produto correcto. Os dois ficam, com a nota ao lado da lei, e o arnês ganhou uma variante de **duas
agulhas**. *Segunda vez que esta linha paga esta forma em dois dias.*

### §4.7 — ⭐ A equivalência das duas portas achou uma divergência que é da CASA

A `assign_missing_root_order` numera as raízes sem ordem pela ordem de `Entity::to_bits()`, que
**não é a ordem de criação** neste `bevy_ecs`: num lote as `n` cópias estão todas sem ordem ao mesmo
tempo e saem `(3), (2), (1)`. A porta de série nunca exercitou o desempate porque nunca teve duas
raízes sem ordem ao mesmo tempo. ⛔ Não é observável no produto (uma cópia é transiente e não entra
na Hierarquia; o *Instantiate* do artista é `n = 1`), e o gate afirma o que é verdadeiro: ordens
**distintas e contíguas**.

### §4.8 — ⚠️ E um `git` mordeu: crase em mensagem de commit é substituição de comando

A primeira mensagem da W2 perdeu duas palavras — o `zsh` executou o conteúdo entre crases. ⇒ mensagem
por ficheiro (`-F`), nunca por `-m` com crases.

---

## §5 — Coisas que uma leitura rápida do diff entende ao contrário

1. **`Spawned` não é um componente esquecido no registo** — a ausência **é** a lei, e ela é travada
   pelo TIPO (sem `Serialize` o registo não compila), como o `TimerRuntime`.
2. **A fábrica não tem `rate`, e isso não é uma feature em falta** — é uma recusa medida: o ritmo vem
   do `Timers`, que o `requires` puxa. Um relógio próprio seriam **dois motores para a mesma lei**.
3. **Não há componente `SpawnPoint`** — as tags do dia anterior já o exprimem, e `tagged` devolve os
   pontos pela ordem da identidade.
4. **A `Lifetime` num objecto comum não está partida** — ela é **inerte por lei**, e o painel di-lo.
   O sítio dela é a **receita**.
5. **A secção diz «the clock is stopped» e isso não é um aviso de erro** — a corrida É o relógio a
   andar, e sem essa cerca a fábrica encheria a cena enquanto o artista edita.
6. **O `BURST_MAX = 1024` não é um número escolhido** — ele é o degrau em que 1 024 cópias custam
   `17 %` de um quadro e o seguinte custa `64 %`. E é uma **contagem**, nunca um orçamento de
   relógio: uma lei de passo fixo não pode perguntar as horas sem o replay divergir.
7. **A `=2` toma a vista da câmera do jogo de propósito** — sem isso ela ensinaria o contrário (as
   cópias sumiriam no meio do ecrã do editor).

---

## §6 — As premissas que a medição derrubou

1. *«O tecto por tique mede-se na porta de cópia»* — não: a porta do produto tem mais quatro
   passagens `O(mundo)`.
2. *«Pôr as três `assign_*` fora do laço é só optimização»* — não: uma delas muda a ORDEM DE RAIZ, e
   foi um gate que o disse.
3. *«Mutar o `With<Spawned>` prova a lei»* — não: há dois guardas, e o outro tapa o buraco.
4. *«A fixtura da fronteira pode ser qualquer duração»* — não: só um múltiplo exacto do tique
   distingue `>=` de `>`.
5. *«A `line/components` toca só o painel e a shell»* — não: as catracas que a julgam vivem no
   `ph2d-editor-core`, e é a **quinta** vez que esta cegueira morde.

---

## §7 — O smoke, CORRIDO no binário antes deste handoff

```
[factory-smoke] =1 uma moeda a cada 0,7 s, cada uma vive 2 s — a cena NAO cresce
[fabrica] 1 copia(s) nascida(s), 0 recusada(s)
[signal] drop <- timer do objecto …, 1 periodo(s)
…
[fabrica] 1 copia(s) sairam da cena
[signal] coin_gone <- morreu a copia …
```

```
[factory-smoke] =2 tres marcas `SpawnPoint` em roda-viva, Max Alive 6, e o fora-do-ecra a colher
[fabrica] 1 copia(s) nascida(s), 0 recusada(s)
[signal] born <- fabrica …, 1 copia(s) nasceram
[fabrica] 1 copia(s) sairam da cena
```

⇒ nas duas, **nascimentos e mortes alternam** e a população estabiliza. *A prova não é o ecrã — é a
contagem parar de subir.*

---

## §8 — O que fica ABERTO

- ⏳ **O tecto de fundo da identidade.** O `StableIdCounter` já é um recurso, e a varredura do
  *máximo* existe só para nunca ficar atrás do mundo; torná-lo autoritativo tiraria `O(mundo)` de
  **todo** caminho de identidade. É outra wave, e o risco vive na única coisa que não se pode
  enganar em silêncio.
- ⏳ **Uma cópia com arte VECTORIAL deixa o documento dela para trás quando morre** — o
  `clone_owned_documents` cria o `VecPath` da cópia e o dreno da morte só despawna a entidade.
  Nomeado, não medido.
- ⏳ **Não há árvore REMOTA** — as cópias vivas não aparecem em lista nenhuma (só no canvas e no
  contador da secção). O Godot resolve-o com uma segunda árvore; aqui é uma decisão de produto.
- ⏳ **Um verbo `Destroy` no `SignalActions`** precisa que a morte de um objecto **autorado** seja
  exprimível na captura, e hoje não é.
- ⏳ **O `ObjectPool`** é recusa medida **por agora**: o preço de nascer não é a alocação. Se a
  medição mudar, a nota muda com ela.

---

## §8-bis — O portão de fecho (batched, 1× sobre o diff acumulado)

`cargo fmt --all --check` limpo · `doc-index.sh --check` → *«19 índices em dia»* ·
`scripts/nextest-impacted.sh` → **14 068 testes, 14 066 verdes**, com **dois** vermelhos:

1. `hr12_widgets_a11y::every_widget_file_wires_a11y` — **real e curado**: a moldura nova é um
   ficheiro de pintura pura, e entra no `PANEL_A11Y_DELEGATE_OK` com a justificação **medida**
   (zero ocorrências de `NodeId`, `hit_index.` e `register(`). É a **quarta** vez que este ficheiro
   regista a mesma forma, e a segunda no mesmo dia.
2. `ph2d-app-flip … a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` — **flake de
   carga conhecida**, da família que o `CLAUDE.md` §5.0 já nomeia (`flip_smooth::
   resample_measurement::precisao::orcamento`). Assinatura confirmada: **3 de 3 verde sozinha** a
   `load 45,65`, e o diff desta linha tem **zero linhas** naquela crate.

⚠️ **E o portão correu `-p ph2d-editor-core` de propósito** — é a lição que a wave anterior pagou
(as catracas que julgam um painel vivem na fundação). Ele apanhou **três**: o HR-15, o DAG e o tecto
de ficheiro de painel, todos curados por **corte** ou pela **tabela**, nunca por isenção nova.

---

## §9 — Onde ler

- O plano, com as tabelas e as recusas: [`09_plano_spawner.md`](../09_plano_spawner.md).
- O oráculo, corrido: [`ferramentas/godot_spawn_lifetime_probe.gd`](../ferramentas/godot_spawn_lifetime_probe.gd).
- As sondas: [`measure_spawn_and_death.rs`](../../../crates/ph2d-ecs/tests/it/measure_spawn_and_death.rs)
  e o `measure_instantiate_door` do `ph2d-app-components`.
- As provas de mutação: [`mutacao_spawner_w1.sh`](../ferramentas/mutacao_spawner_w1.sh) ·
  [`mutacao_spawner_w2.sh`](../ferramentas/mutacao_spawner_w2.sh).
