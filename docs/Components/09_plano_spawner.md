# Plano — a FÁBRICA e o CICLO DE VIDA (TOP-20 #11 e #12)

> **Estado:** PLANO — nenhuma linha de produto escrita. `line/components`, 2026-09-14.
> Fila: [levantamento §7](00_levantamento_componentes.md) #11 e #12 ·
> [síntese estrutura_ai §2](pesquisa/sintese_estrutura_ai.md) · [plano 05 §11.2](05_plano_de_implementacao.md).
> Molde das waves: o `Timer` (§12 do plano 05) e as `Tags` ([08](08_plano_tags.md)).
>
> ⚠️ **Tudo o que aqui tem número foi CORRIDO nesta máquina antes de ser escrito** — um oráculo sem
> interface e uma sonda Rust sobre o ECS, os dois versionados:
> [`ferramentas/godot_spawn_lifetime_probe.gd`](ferramentas/godot_spawn_lifetime_probe.gd) ·
> [`crates/ph2d-ecs/tests/it/measure_spawn_and_death.rs`](../../crates/ph2d-ecs/tests/it/measure_spawn_and_death.rs).
>
> ⭐ **Os dois itens são UM bloco por declaração do levantamento**, não por conveniência:
> *«`Lifetime` + `DestroyOutside` — higiene de ciclo de vida em 2 marcadores triviais; **sem eles o
> 11 e o 14 vazam**»*. Uma fábrica sem higiene é um vazamento com botão.

---

## §0 — O que o artista passa a conseguir fazer

1. **Uma receita nasce sozinha na cena** — um objecto com a *Factory* põe cópias de um mestre a
   correr, no sítio que ele escolher: aqui, espalhadas numa área, ou **num ponto marcado com uma
   tag** (as tags de ontem).
2. **Um sinal faz nascer** — *«quando o `alarme` tocar, nasce um inimigo»* — porque a fábrica é um
   consumidor da tabela `SignalActions`, que já existe.
3. **O que nasce MORRE sozinho** — por tempo (*Lifetime*) ou por sair do ecrã do jogo
   (*Destroy Outside*), que são as duas higienes sem as quais uma bala enche a memória.
4. **A fábrica tem limites que o artista lê**: quantos vivos ao mesmo tempo, quantos ao todo, e um
   sinal quando ela se esgota.
5. ⭐⭐ **E nada disto suja o projecto**: o que nasce numa corrida **não** entra no ficheiro, **não**
   entra na pilha de `Ctrl+Z`, e **desaparece quando você rebobina** — ver §2.1, que é a lei nova
   desta wave e a metade que decide tudo o resto.

⛔ *Um componente sem consumidor lê-se, numa varredura, exactamente como uma feature pronta* (plano
05 §11.2). ⇒ a fábrica nasce **com** o que a alimenta (o relógio e o sinal, que já existem) e **com**
o que a limpa (as duas mortes), nunca antes deles.

---

## §1 — Estado da arte, e o que cada um TENTOU e ABANDONOU

### §1.1 — Medido nesta máquina (oráculo corrido sem interface; só a SAÍDA é usada)

**Godot 4.7.2 (MIT)** — `godot --headless --script docs/Components/ferramentas/godot_spawn_lifetime_probe.gd`
(duas corridas, `load 33` e `load 11`; os relógios são da segunda):

| pergunta | saída medida |
|---|---|
| logo depois de `queue_free()` | o nó está **válido**, **na árvore**, **filho do pai** e **na consulta de grupo** |
| no quadro seguinte | inválido, fora do grupo — ⇒ **a morte é ADIADA para o fim do quadro** |
| `queue_free()` duas vezes | sem erro (`is_queued_for_deletion` continua `true`) |
| `free()` | **imediato**: a contagem de filhos cai na mesma instrução |
| um nó nascido no tique `3` | tem `1` tique no tique `4` ⇒ **o recém-nascido NÃO tica no tique em que nasce** |
| relógio de `0,105 s` a 60 Hz (= `6,3` tiques) | períodos `[6,6,7,6,6,7,6,6,7,6,6]`, média **`6,273`** ⇒ **o resto é CARREGADO**, não re-zerado |
| `VisibleOnScreenNotifier2D` sem janela | `is_on_screen() = false` sempre e **zero sinais**, mesmo na origem com câmera ⇒ **não mede sem servidor de render** |
| o rectângulo que ele testa | é o **`rect` do próprio notificador** (`-10,-10,20,20` de fábrica), **não** o da sprite |
| instanciar de uma `PackedScene` de 2 nós | `4,205` µs (N=1 000) · `4,090` (N=10 000) · `4,219` (N=100 000) ⇒ **plano: o preço NÃO cresce com a cena** |
| pedir a morte de N | `0,17` µs por nó |

⭐⭐ **As três leituras que mudam o nosso desenho:**
1. **a morte é adiada e o moribundo continua visível a toda consulta durante o quadro** — é isso que
   torna um laço de iteração seguro sem regra nenhuma escrita;
2. **o recém-nascido não tica no tique em que nasce** — uma lei de determinismo de graça;
3. **o preço de nascer não é função da cena** — e o nosso É (§6), o que é o achado desta pesquisa.

⛔ **E a pergunta que o oráculo NÃO respondeu:** o fora-do-ecrã do Godot vive no servidor de render,
logo não corre sem interface. ⇒ o nosso `DestroyOutside` **não pode** ser um notificador de render —
tem de ser um teste geométrico puro contra a câmera do jogo. *É a forma mais fraca de o oráculo
falar, e ainda assim ela decidiu um desenho* (§2.6).

### §1.2 — A categoria que quase nenhuma engine grande tem

Dos dossiês (cada um com a linha declarada):

| engine | fábrica como componente | pooling | ciclo de vida |
|---|---|---|---|
| **Godot** | ⛔ **não existe** — o idioma é `instantiate()+add_child()`, *sempre código*; o `MultiplayerSpawner` é só replicação de rede | ⛔ não | `queue_free()` à mão |
| **Unity** | ⛔ **não** — só o Particle System spawna por Inspector; instanciar é `Instantiate(prefab)` | API `ObjectPool<T>`, **sem UI** | `Destroy(go, t)` em código |
| **Unreal** | ⛔ **não** (`SpawnActor`) | ⛔ **pooling genérico AUSENTE, declarado** | `LifeSpan` (é um campo de Actor) |
| **Bevy** | ⛔ ausente | manual | manual |
| **Construct / GDevelop** | ✅ *System: Create object* / *Object spawner* — os **no-code** são os únicos que a têm | — | ✅ *Destroy outside*, *Fade→Destroy* |
| **Defold** | ✅ **Factory** (protótipo, load dinâmico, troca em runtime) | ✅ interno — **a doc PROÍBE pooling manual** | à mão |
| **Phaser** | `Group.createMultiple` + timers | ✅ `Group` (`get`/`killAndHide`/`maxSize`) | `lifespan` de partícula |

⇒ **o diferencial é directo**: a categoria existe nas engines *no-code* e falta nas grandes, e a
razão delas é sempre a mesma — *instanciar é código*. É exactamente o buraco que este produto existe
para fechar.

⭐ **A decisão da Defold é a que se adopta** (*«a engine faz o pool; o utilizador não»*) e ela alinha
com o §0.0 do `CLAUDE.md`: **medir primeiro se o pool é sequer necessário**. A medição está em §6 e a
resposta é *«não é o pool que custa; é a identidade»* — ver §2.8.

---

## §2 — O desenho: uma porta por pergunta

### §2.1 — ⭐⭐⭐ A LEI NOVA: **o que nasce numa corrida não é documento**

É o espelho exacto da lei que o `ph2d-preview-drive` já paga, um nível acima: ali o que o motor
escreve num componente é **pré-visualização**; aqui o que o motor **põe no mundo** é
pré-visualização.

> **O documento é o que o artista autorou. Uma cópia que uma fábrica pôs na cena vê-se, e não se
> guarda, não se desfaz e não sobrevive a rebobinar.**

⛔ **Sem esta lei nada do resto pode existir,** e o mecanismo está medido no código de hoje:

- `world_to_snapshot` percorre **toda** raiz com `Transform` e desce por `Children` — não há filtro
  nenhum de transiência (lido em [`save.rs`](../../crates/ph2d-ecs/src/scene/save.rs)). Uma fábrica a
  60 Hz poria uma entidade nova no **ficheiro** e na **pilha de undo** por tique.
- **Não existe modo de jogo neste app**: os relógios do passo fixo correm sempre
  ([`fase_fixed_step_clocks`](../../shells/desktop/src/render_loop/fase_fixed_step_clocks.rs)), e o
  que decide se o mundo anda é `playhead.is_playing() && flags.simulate_physics`. ⇒ uma fábrica sem
  esta lei encheria a cena **enquanto o artista edita**.

**A forma:** um componente marcador **NÃO registado**

```rust
/// Quem pôs esta entidade no mundo — o `StableId` da fábrica. ⛔ NÃO registado, de propósito.
pub struct Spawned { pub by: u64, pub born_tick: u64 }
```

⚠️ **A ausência de registo é a lei, e ela é travada pelo TIPO** — é o precedente exacto do
`TimerRuntime` (§12.2 do plano 05): registá-lo exigiria `Serialize`, e sem ele **não compila**. *A
porta fecha-se sozinha.*

⚠️ **E ela não basta:** a captura visita a árvore, e uma entidade sem `Serialize` continua a ser
visitada. ⇒ a segunda metade é **uma poda na travessia** (`world_to_snapshot` não desce por uma raiz
com `Spawned`), com gate nos dois sentidos — *o que tem a marca não entra; o que não a tem entra*.

⚠️ **O `Spawned.by` não é decoração:** é ele que responde *«quantos vivos esta fábrica tem?»* numa
varredura só, sem um segundo índice a manter coerente com o mundo depois de todo restore de undo —
a mesma razão pela qual o `stable_id.rs` recusa por escrito um mapa `nome → entidade`.

### §2.2 — **O QUÊ**: a receita é um MESTRE, nunca um ficheiro

```rust
pub struct Factory { pub master: u64 /* StableId do mestre */, … }
```

⭐ A porta é `ph2d_app_components::instantiate::instantiate_master`, **a mesma do botão
*Instantiate***, e com ela vêm de graça três coisas que um caminho próprio teria de reescrever:
editar a receita muda as cópias vivas no mesmo quadro (F4.3), as **variantes** funcionam (F5), e os
documentos possuídos são clonados em vez de partilhados (F4.6a).

⛔ **Não é o `PrefabRef`/`PrefabDoc`** (o caminho de asset cozido): ele existe, mas o fluxo que o
artista tem no produto é *Make Component*, e uma segunda resposta a *«o que é uma receita?»*
divergiria em silêncio.

⚠️ **Um `master` que não é mestre é SILÊNCIO**, nunca um erro de motor — a lei do alvo que não
existe, do `SignalActions`. O painel é que o diz (§4).

### §2.3 — **ONDE**, e porque o `SpawnPoint` **NÃO** nasce

```rust
pub enum SpawnAt { Here, Area { w: f32, h: f32 }, Tagged { tag: u64, pick: Pick } }
pub enum Pick { Cycle, Random }
```

⭐⭐⭐ **O `SpawnPoint` do TOP-20 #11 é uma RECUSA MEDIDA, e a composição que o dissolve shipou
ontem.** O levantamento pedia *«marcador nomeado com gizmo, consultável por nome/tag»* e
descrevia-lhe as dependências como *«`Tags` para consulta por grupo»*. Com as tags no produto:

- um ponto de nascimento **é** um objecto vazio marcado com uma tag — e um objecto vazio já se pega
  no canvas com um anel próprio desde a F4;
- *«todos os SpawnPoint com a tag X»* é `ph2d_ecs::tags::tagged`, que devolve **pela ordem da
  identidade** (determinista, medido em `0,0076 ms` para 1 000 acertos numa cena de 10 000);
- o que sobrava era o **gizmo**, e um componente novo para desenhar uma cruz é o item de lista que a
  §5.0 do `CLAUDE.md` manda medir antes de construir.

⇒ **não se constrói.** O que se constrói é a linha do painel que diz *«escolhe um objecto com esta
tag»*, e a documentação da recusa vive aqui.

### §2.4 — **QUANDO**, e porque a fábrica **NÃO** tem relógio próprio

```rust
pub struct Factory { …, pub on_signal: String, pub burst: u32, … }
```

⭐⭐ **A fábrica só nasce ao SINAL**, e o ritmo vem do `Timer` que já existe (TOP-20 #2). Medida a
composição antes de construir (lei §1.8 do levantamento — *«Sine/Orbit/Wiggle já existem como nodes:
o componente é o atalho de 1 clique, nunca um segundo motor para a mesma lei»*):

| o que o artista quer | com um relógio próprio na fábrica | com o que já existe |
|---|---|---|
| *nasce um a cada segundo* | `rate = 1` | `Timer{repeat, signal="spawn"}` + `Factory{on_signal="spawn"}` |
| *nasce ao apanhar a moeda* | `on_signal` (na mesma) | `on_signal` |
| *nasce 5 de uma vez a cada 3 s* | `rate` + `burst` | `Timer{3s, repeat}` + `burst = 5` |

⇒ o relógio próprio **não compra nenhuma capacidade nova**; compra cliques. E custa a coisa que este
repo mede como defeito: **dois motores para a mesma lei**, com dois sítios onde *«que horas são»* pode
divergir.

⭐ **Os cliques compram-se com o mecanismo que já existe e que a fila declara como lei** — os
**componentes requeridos** (levantamento §1.4, a F0/F3 shipou-os: `ComponentDesc::requires`):
acrescentar *Factory* puxa um `Timers` com uma linha chamada `spawn` e o sinal já ligado. *O artista
faz um clique; a casa não ganha um segundo relógio.*

⚠️ **Um `on_signal` vazio = nunca nasce** — a lei da casa (*«um produtor sem nome não fala, em vez de
falar com um nome vazio»*), e o painel di-lo em vez de o esconder.

### §2.5 — **QUANTO**: dois limites, e o sinal de esgotada

```rust
pub struct Factory { …, pub alive_max: u32, pub total_max: u32,
                     pub on_spawned: String, pub on_exhausted: String, pub seed: u64 }
```

- `alive_max = 0` ⇒ sem limite de vivos; senão a fábrica não nasce acima dele (e **não** mata
  ninguém para caber — matar é do §2.6, e uma fábrica que mata o próprio filho mais velho é uma
  decisão que ninguém autorou).
- `total_max = 0` ⇒ sem limite; ao chegar lá publica `on_exhausted` **uma vez** (o estado *esgotada*
  é vivo, não autorado — o precedente do `TimerState::running`).
- ⭐ **`seed` explícita** — lei §1.6 do levantamento (determinismo é lei da casa: o anel GGPO e o
  hash `physics_ecs_c9` já pagam esse preço). O gerador vive no estado **vivo**, e a semente é o que
  o artista autora.

⚠️ **E um TECTO POR TIQUE**, que o §0.0 proíbe escrever antes de medir: ver §6.3 — a medição de hoje
diz que **o número não pode ser cravado ainda**, porque ele descreveria o caminho lento.

### §2.6 — **A MORTE**: duas leis, um só efeito

```rust
pub struct Lifetime { pub duration_us: u64, pub on_death: String }
pub struct DestroyOutside { pub margin: f32 }
```

⭐⭐⭐ **A morte só alcança o que NASCEU numa corrida** — só quem tem `Spawned`.

*Porquê:* uma corrida nunca pode apagar documento. A física move objectos autorados e a captura repõe
a pose autorada; **apagar não tem valor a repor** — o ledger troca valores, não existências. Um
`Lifetime` que apagasse um objecto desenhado destruiria trabalho com um `Ctrl+Z` que não o traz de
volta.

⚠️ **E isso não faz dele um controlo morto**, que é a armadilha óbvia (a caça de 2026-08-30 mediu 34
controlos mortos): **os dois componentes vivem na RECEITA e correm nas CÓPIAS**. Um mestre está
escondido por construção (*Make Component*), então o sítio natural deles é exactamente onde eles
fazem efeito. Num objecto solto, a linha do painel diz a metade honesta — *«nothing is born from this
object»* —, que é a forma que o `Timer` já usa (*«This timer never starts»*).

⚠️ **A morte é ADIADA, como no oráculo:** ela produz um **facto** (`Death { entity, why }`), e a
remoção acontece num dreno único no fim do quadro. ⇒ dentro do tique, quem morreu ainda existe para
toda consulta — que é o que torna o laço seguro sem regra escrita, e o que faz o sinal `on_death`
poder nomear quem morreu.

**`DestroyOutside` — contra QUE ecrã:**

⭐ Contra a **`GameCamera` activa** (TOP-20 #7, no produto desde 10/09), crescida por `margin`.
⛔ **Nunca contra a vista do editor**: isso faria a corrida depender de onde o artista rolou o ecrã —
o mesmo defeito que a memória `a_probe_that_arms_a_module_by_env_var_measures_another_program`
descreve. ⇒ **sem `GameCamera` na cena o componente não mede nada, e o painel di-lo.** *O fora-do-ecrã
precisa de um ecrã, e o ecrã de um jogo é a câmera dele.*

⚠️ **E ele é geométrico puro**, não um notificador de render — a decisão que o oráculo forçou ao não
conseguir responder sem janela (§1.1). Ganha-se com isso uma coisa que o Godot não tem: **ele é
testável sem GPU**.

### §2.7 — A ordem no quadro (é lei, e tem gate)

```
   os relógios do passo fixo (§11 · timers · LIFETIME)      ← fase_fixed_step_clocks
   → o mundo anda (física)                                  ← fase_physics_step
   → o quadro de sinais vira · os produtores publicam
   → o DRENO: toast · SignalActions · A FÁBRICA             ← fase_signal_outbox
   → o DRENO DA MORTE (o que morreu neste quadro sai)       ← ⭐ novo, no fim da mesma fase
   → post_frame_undo (a fotografia)
```

⚠️ **A fábrica lê o sinal no MESMO sítio que a tabela de acções**, e pela mesma razão escrita ali:
depois do dreno (senão os sinais deste quadro só chegavam ao próximo) e antes do `post_frame_undo`
(senão a escrita não é fotografada). ⛔ **E o dreno da morte tem de vir DEPOIS do da fábrica** —
senão uma cópia com `Lifetime = 0` morreria no quadro seguinte ao que devia, e o gate que mede a
ordem lê a POSIÇÃO das chamadas no ficheiro (o precedente do
`the_authored_intent_queue_has_a_drain_and_it_runs_before_the_signal_drain`).

### §2.8 — O que fica de FORA, com o motivo

- ⛔ **`ObjectPool` (reciclar em vez de criar/destruir).** A medição (§6) diz que o preço de nascer
  **não é a alocação** — é a varredura de identidade, que um pool paga na mesma ao dar identidade
  nova a um reciclado. ⇒ o pool cura o sintoma errado. A cura medida é a porta em lote (§6.3).
  *A decisão da Defold — «a engine faz o pool» — sobrevive: o utilizador nunca o escreve, e por
  enquanto a engine também não precisa.*
- ⛔ **Um verbo `Destroy` no `SignalActions`.** Ele precisaria que a morte de um objecto **autorado**
  fosse exprimível na captura, e hoje não é (§2.6). O que existe e serve é `Hide`.
- ⏳ **Herdar a velocidade de quem nasce** (o *Inherit* do emissor do Motion). É a mesma família e
  precisa de um campo a mais; fica nomeado, não construído.
- ⏳ **Nascer ao longo de um caminho** (*spawner-along-path*): o vector tem a caneta e o
  `motion.path`, mas o sujeito aqui é um objecto de cena. Fica nomeado.

---

## §3 — Contratos congelados e schema (com a prova)

| pergunta | resposta | prova |
|---|---|---|
| toca contrato **Nodes** (§6)? | **não** | nada aqui é `NodeOp`/`OpResolver`/`NodeManifest` |
| toca contrato **Tools** (§6)? | **não** | nenhuma ferramenta nova; `Tool` fica em 12 |
| toca o contrato do **doc vectorial**? | **não** | a fábrica copia por `deep_copy_subtree`, que já é a porta de hoje |
| `PROJECT_SCHEMA` | **+1** (um degrau para os três componentes registados) | regra do degrau `127 → 128`: um `ComponentBlob` de `type_id` desconhecido **recusa o load inteiro** |
| registo do `ph2d-ecs` + os 2 espelhos | **+3** cada (`Factory`, `Lifetime`, `DestroyOutside`) | ⛔ o `Spawned` **não** entra — §2.1 |
| `VEC_SCENE_SCHEMA` · `FIELD_DOC_VERSION` · `FLIP_SCHEMA` | **não se mexem** | — |

⚠️ **O delta conta-se contra a árvore em que vai aterrar**, nunca o literal (`CLAUDE.md` §5.0) — esta
linha já pagou essa lição no degrau `127 → 128`.

⚠️ **Migração:** COM degrau de leitura, como o `128 → 129`: desde 10/09 há projectos gravados com
tabelas de acções, e recusá-los apagaria autoria que existe.

---

## §4 — A UI: as quatro condições, uma a uma

O item #1 do TOP-20 (Inspector derivado do tipo + paleta *Add Component* + requeridos) está **feito**
por esta linha, então os três componentes nascem com secção sem uma linha artesanal. O que a wave
deve provar, condição a condição:

1. **EXISTE** — três entradas no catálogo `ph2d-component-desc` (família `Logic`, que já os nomeia
   por escrito no doc-comment de [`catalog/logic.rs`](../../crates/ph2d-component-desc/src/catalog/logic.rs)).
2. **É PINTADO E REGISTADO** — secção no `ph2d-panel-inspector` com os ids no `HitIndex`; ⚠️ os chips
   guiados por TABELA têm de entrar no gate irmão (`table_driven_chips_are_registered_too`), que
   existe porque o gate de registo era cego a eles.
3. **O CLIQUE CHEGA AO BARRAMENTO** — `FactoryEdit` no `action_bus`, drenado na
   `fase_inspector_commits` (⚠️ ela está a **219/200** antes desta wave — o corte já foi feito em
   `fase_tag_tree_commits`; esta entra por fase-filha própria).
4. **A SEQUÊNCIA LEVA A ALGUM LADO** — o smoke (§7) é a prova, e há `seam_*` com ponteiro REAL sobre
   o campo do mestre.

⚠️ **O picker do mestre** é o mesmo da troca de mestre da F5.8 (ele já filtra por `MasterRoot`) — ⛔
não se escreve um segundo.

---

## §5 — Os gates, red-first

**W1 — as leis puras** (`ph2d-ecs`, sem shell):
1. uma fábrica sem sinal nunca nasce · 2. um `master` que não é mestre é silêncio · 3. `burst`
produz exactamente `burst` · 4. `alive_max` conta **os desta fábrica** e não os do mundo · 5.
`total_max` publica `on_exhausted` **uma vez** · 6. a `seed` dá a MESMA sequência em duas corridas ·
7. duas fábricas com a mesma semente **não** produzem a mesma sequência (a semente entra com a
identidade) · 8. o `Lifetime` mata ao tique **exacto** e carrega o resto (a lei do oráculo §1.1) ·
9. um `Lifetime` sem `Spawned` **não mata** · 10. o `DestroyOutside` sem câmera **não mata** ·
11. o `DestroyOutside` mata fora do rectângulo **crescido pela margem** e não dentro ·
12. o recém-nascido **não** é contado pelo relógio no tique em que nasce (a lei D do oráculo).

**W2 — a ponte e a transiência:**
13. uma cópia nascida **não aparece** no `WorldSnapshot` · 14. …e uma autorada aparece (o controlo,
senão a poda podia estar a podar tudo) · 15. uma corrida com 100 nascimentos deixa a pilha de undo
**do mesmo tamanho** · 16. rebobinar **varre** o que nasceu · 17. o que nasce entra na física e sai
dela ao morrer · 18. a ordem do quadro (a posição das duas chamadas no ficheiro).

**W3 — o painel:** 19. a secção pinta o que o componente tem · 20. os ids estão registados ·
21. o campo do mestre recusa quem não é mestre · 22. a linha honesta aparece num objecto que não é
receita.

**W4 — as cenas:** 23/24. as duas cenas montam o que dizem que montam.

⚠️ **Fixturas:** a cena de medição vive numa função só (`fixture::cena(n)`), e o **mundo em que a
fábrica trabalha** é parte da fixtura — a medição do §6 mostra que uma fixtura de 8 objectos mediria
outro programa.

---

## §6 — As MEDIÇÕES (corridas antes de escrever este plano)

`cargo test -p ph2d-ecs --release --test it measure_spawn_and_death -- --include-ignored --nocapture --test-threads=1`
· `load 13,27` · mínimo de 9 corridas, mediana ao lado.

### §6.1 — O preço de NASCER é função da CENA, não da receita

| cena (objectos) | cópias no tique | só a identidade (ms) | copiar, min (ms) | **por cópia (µs)** |
|---:|---:|---:|---:|---:|
| 100 | 100 | 0,0013 | 0,4260 | **4,26** |
| 1 000 | 100 | 0,0013 | 0,5521 | **5,52** |
| 10 000 | 100 | 0,0075 | 1,8091 | **18,09** |
| 10 000 | 256 | 0,0075 | 4,6803 | **18,28** |
| 100 000 | 100 | 0,0698 | 14,3206 | **143,21** |
| 100 000 | 256 | 0,0698 | 36,7854 | **143,69** |

E a receita quase não conta:

| cena | peças na receita | por cópia (µs) | por PEÇA (µs) |
|---:|---:|---:|---:|
| 10 000 | 3 | 18,48 | 6,16 |
| 10 000 | 10 | 24,20 | 2,42 |
| 10 000 | 30 | 40,88 | 1,36 |

⇒ **`10×` as peças custa `2,2×`; `10×` a cena custa `8×`.** A parte fixa domina.

⭐⭐⭐ **A causa está lida no código e o número confirma-a:** `deep_copy_subtree` chama
`assign_missing_stable_ids` **a cada cópia**, e essa porta faz **duas varreduras do mundo inteiro**
(o máximo dos ids, e quem não tem id). A coluna *«só a identidade»* mede uma delas: `0,0698 ms` numa
cena de 100 000, que é **49 %** do preço de uma cópia. ⇒ **a fábrica de hoje é quadrática na cena**
(`N` nascimentos × `O(mundo)`), e isso é **invisível numa cena pequena** — a 100 objectos ela custa
`4,26 µs` e parece plana.

⚠️ **Contra o oráculo:** o Godot instancia em **`4,2 µs` independentemente da cena** (1 000, 10 000 e
100 000 dão o mesmo número). Não é uma comparação de iguais — a nossa porta serializa blobs pela
vtable do registo — mas a **forma** é comparável, e é a forma que está errada do nosso lado.

### §6.2 — O preço de MORRER é plano e desprezável

| cena | mortes no tique | apagar, min (ms) | por morte (µs) |
|---:|---:|---:|---:|
| 1 000 | 1 000 | 0,1673 | 0,17 |
| 10 000 | 1 000 | 0,1637 | 0,16 |

⇒ **nascer custa ~100× morrer**, e morrer não vê a cena. *A higiene é grátis; a fábrica é que não.*

### §6.3 — ⭐⭐⭐ A PORTA EM LOTE, medida na W1 — e o tecto que ela permite escrever

O §0.0 manda medir antes de limitar — e manda mais: **nunca deixar o fallback definir o produto**.
Cravar `SPAWN_MAX_PER_TICK` a partir da tabela §6.1 teria escrito o tecto do **caminho lento**.

⇒ a W1 construiu [`deep_copy_subtree_many`](../../crates/ph2d-ecs/src/instantiate.rs) — copiar `n`
vezes pagando a identidade **uma** — e mediu-a ao lado da de série (`release`, mínimo de 9,
`load 16,02`):

| cena | cópias | em SÉRIE (ms) | µs/cópia | em LOTE (ms) | µs/cópia | ganho |
|---:|---:|---:|---:|---:|---:|---:|
| 100 | 256 | 1,1857 | 4,63 | 0,6860 | 2,68 | 1,7× |
| 1 000 | 256 | 1,4987 | 5,85 | 0,6744 | 2,63 | 2,2× |
| 10 000 | 256 | 4,6589 | 18,20 | 0,6865 | 2,68 | **6,8×** |
| 10 000 | 1 024 | 20,3492 | 19,87 | 2,6383 | 2,58 | 7,7× |
| 10 000 | 4 096 | 104,4247 | 25,49 | 10,6372 | 2,60 | 9,8× |
| 100 000 | 256 | 37,0477 | 144,72 | 0,8297 | 3,24 | **44,7×** |
| 100 000 | 1 024 | 150,5285 | 147,00 | 2,8334 | 2,77 | 53,1× |
| 100 000 | 4 096 | 625,1080 | 152,61 | 10,8490 | 2,65 | **57,6×** |

⭐⭐⭐ **O preço por cópia passou a ser PLANO na cena** — `2,58`–`3,25 µs` em toda a tabela, que é a
**forma** do oráculo (Godot: `4,2 µs`, plano) e **mais barato** que ele. A quadratura desapareceu
porque desapareceu a causa, não porque alguém a escondeu atrás de um tecto.

⇒ **`BURST_MAX = 1024`**, e o recurso dele é **o QUADRO**: 1 024 cópias custam `2,64 ms` numa cena de
10 000 e `2,83 ms` numa de 100 000 — **17 % de um quadro de 16,7 ms** —, e o degrau seguinte (4 096)
custa `10,6 ms`, que é **64 %**. *O número é o do caminho rápido, e a escada está ao lado dele.*

⚠️⚠️ **ESTES NÚMEROS SÃO DA PORTA DE CÓPIA E NÃO DO PRODUTO — leia a §6.3-bis antes de os citar.**
O tecto continua em `1024`; o preço dele no caminho do artista é **`5,35 ms` (32 % de um quadro)**,
não os `2,6 ms` desta tabela.

⚠️ **E o tecto é uma CONTAGEM, nunca um orçamento de relógio** — um tecto em milissegundos faria o
passo fixo produzir um número diferente de nascimentos em cada máquina, e o replay (`physics_ecs_c9`,
a matriz de 3 OS) divergiria. *Uma lei de simulação não pode perguntar as horas.*

### §6.3-bis — ⭐⭐⭐ E a porta do PRODUTO, re-medida na W2 (é ELA que o tecto descreve)

O caminho que o artista percorre é `ph2d_app_components::instantiate::instantiate_master_many`, que
paga, **por cópia**, o que a cópia não paga: o clone dos documentos possuídos, o remap de
referências e o elo por peça — mais, antes do lote existir, o `unique_name` (`O(n² × nomes)`
sozinho) e as três `assign_*`. Medido (`release`, mínimo de 5, `load 7`, cena de 10 000):

| cópias | em SÉRIE (ms) | µs/cópia | em LOTE (ms) | µs/cópia | ganho |
|---:|---:|---:|---:|---:|---:|
| 16 | 7,33 | 458,0 | 0,53 | 33,1 | 13,8× |
| 64 | 29,67 | 463,5 | 0,77 | 12,1 | 38,3× |
| 256 | 126,95 | 495,9 | **1,72** | 6,7 | **73,8×** |
| 1 024 | 646,84 | 631,7 | **5,35** | 5,2 | **120,8×** |

⇒ **`BURST_MAX = 1024` custa `5,35 ms`, que é `32 %` de um quadro** — e não os `16–17 %` que a
tabela da porta de cópia sugeria. *O tecto fica, e o número ao lado dele passa a ser o do produto.*
⚠️ **Em série a rajada máxima era inalcançável:** `647 ms` são **39 quadros**.

⛔ **A varredura NÃO foi apagada** — ela é a rede contra dois objectos com o mesmo id, e esta wave não
é sobre a identidade. O que mudou é **quantas vezes** ela corre. ⏳ **A cura de fundo fica nomeada:**
o `StableIdCounter` já é um recurso, e a varredura do *máximo* existe só para nunca ficar atrás do
mundo; torná-lo autoritativo tiraria `O(mundo)` de **todo** caminho de identidade — é outra wave, com
o risco a viver na única coisa que não se pode enganar em silêncio.

### §6.4 — A varredura por tique é grátis

O relógio de vidas e o teste de fora-do-ecrã são queries sobre quem tem o componente. A sonda irmã
das tags mede essa forma: `0,0076 ms` para 1 000 acertos numa cena de 10 000, `0,0753 ms` para 10 000
numa de 100 000. ⇒ **não há aqui decisão de desenho a tomar**, e é por isso que esta wave não
constrói índice nenhum.

---

## §7 — As waves, e o que cada uma entrega ao dono

| wave | o que entra | o que o dono vê |
|---|---|---|
| **W1** | as três leis puras + o `Spawned` + a porta em LOTE **medida** + 12 gates | nada ainda (é motor) |
| **W2** | a ponte na shell, a poda da captura, o dreno da morte, a varredura ao rebobinar + 6 gates | — |
| **W3** | **DUAS** secções do Inspector (ver abaixo), o catálogo, os requeridos + 4 gates | os controlos |
| **W4** | duas cenas de smoke + a prova de mutação | **o smoke** |

⚠️ **A W3 entregou DUAS secções e não três, e a razão é o SUJEITO:** a `Factory` vive em quem
fabrica e a `Lifetime`/`DestroyOutside` vivem na **receita**, que é outro objecto — uma secção só
chamada *Factory* a mostrar apenas uma vida seria um título a mentir. ⇒ *Factory* e *Lifecycle*,
com **um** snapshot e **uma** enum de edição, porque a plumbing é a mesma pergunta.

**As duas cenas (§0.8 — passos numerados, o que se vê, e como saber que deu errado):**

⚠️ **Elas são `PH2D_FACTORY_SMOKE=1|2`** — um roteador próprio da família, e não níveis novos de
um que já existia (a lei do `CLAUDE.md` §5.1 sobre o `PH2D_MOTION_NODE_PATH_SMOKE`).

- **`=1` A chuva que não cresce** — uma nuvem com um relógio de `0,7 s` e uma fábrica ligada a ele;
  a receita é uma moeda que cai e vive `2 s`. **Medido a correr:** nasce uma a cada `0,7 s`, morre
  uma a cada `0,7 s` a partir da terceira, e a população estabiliza. ⚠️ *A prova não é o ecrã — é a
  contagem parar de subir*; uma fábrica sem higiene enche a memória com o ecrã igual.
- **`=2` Nascer NUM PONTO marcado, com limite e sem lixo** — três marcas com a tag `SpawnPoint`, a
  fábrica em roda-viva, `Max Alive = 6`, e um `DestroyOutside` a colher quem sai do ecrã da
  **câmera do jogo** (a cena **toma** a vista dela, senão as cópias somem no meio do ecrã do editor
  e a cena ensina o contrário). ⭐ Ela compõe **quatro waves anteriores desta linha**: tags, sinais,
  câmera de jogo e instâncias.

---

## §8 — As decisões que são do DONO (o resto é técnico e está decidido acima)

1. **O que nasce numa corrida some quando você rebobina** (§2.1) — é o que Godot, Unity e Unreal
   fazem, e é a única forma de a fábrica não sujar o ficheiro.
2. **A fábrica não tem relógio próprio: ela usa o `Timer` que já existe**, puxado automaticamente
   quando você a acrescenta (§2.4). Um clique, e não dois relógios.
3. **O jogo nunca apaga um objecto que você desenhou** (§2.6) — a morte alcança só as cópias que
   nasceram na corrida; para esconder um objecto autorado existe o *Hide*.

⛔ **E uma coisa que o plano NÃO constrói de propósito:** o `SpawnPoint` como componente — as tags de
ontem já o exprimem (§2.3).
