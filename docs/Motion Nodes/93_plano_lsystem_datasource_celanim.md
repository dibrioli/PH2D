# 93 — Plano: L-System · Data Source · Cel Animation

> **Pedido do Enio (2026-08-28):** os três primeiros itens do [doc 92](92_o_que_o_mini_cavalry_tem_e_nos_nao.md),
> com **estudo e pesquisa intensos ANTES de planejar, e planejamento antes de implementar**.
>
> Este doc é o **planejamento**. A pesquisa que o sustenta foi feita em três frentes paralelas e
> cada afirmação abaixo tem endereço no código medido.

---

## §0 — A triagem que dispensou o clean-room

O Enio apontou a [`SKILL_Cleanroom_Reimplementacao.md`](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)
*"se necessário"*. A §2 dela manda **ler a licença real do alvo** antes de tudo, e a leitura fecha o
assunto no primeiro degrau:

| Evidência | Onde |
|---|---|
| `"private": true`, sem `LICENSE` no repositório | `MiniCavalryV2/package.json` |
| *"De: agente do protótipo MiniCavalryV2"* · *"o dono definiu que a PH2D usará o MiniCavalry como base/referência de design de si mesma"* | `MiniCavalryV2/PROMPT_AVALIACAO_ENGINE_PH2D.md` |

⇒ **Não é código de terceiros — é o protótipo do próprio dono.** Não há parede, não há
especificador isolado, não há revisor de similaridade. E os três algoritmos também não têm nada
restrito atrás deles: L-System é Lindenmayer 1968 (o livro canônico é publicado **de graça** pelos
autores), CSV/JSON são especificações abertas, e "alternar entre entradas ao longo do tempo" não é
propriedade de ninguém.

⚠️ **Uma restrição FICA, e ela é do dono, não da lei:** o mesmo documento pede implementação
**nativa, do zero**, com o protótipo como *referência canônica, não código a traduzir*. A pesquisa
leu o protótipo para saber **o que ele resolve**; o plano abaixo não porta uma linha.

---

## §1 — O que a pesquisa mudou (leia isto antes de qualquer estimativa)

**As três features saíram da pesquisa com uma forma diferente da que o doc 92 supunha.**

| | O doc 92 dizia | O que a medição diz |
|---|---|---|
| **Cel Animation** | "um nó novo" | ⛔ **Não é nó novo.** É um **MODO do `motion.sub_uv`** — que já tem grelha, FPS (`speed`), defasagem por elemento (`stagger`) e **rota de GPU completa**. Falta-lhe ping-pong, duração desigual e *once* |
| **Data Source** | "um nó novo" | ⚠️ **~70% é o molde do `audio.bands`** copiado. E ⛔⛔ **o campo de colar CSV não pode existir** (§4) |
| **L-System** | "um nó novo" | ✅ **É mesmo um nó novo** — e o achado é que ele deve emitir uma **ÁRVORE**, não uma nuvem |

---

## §2 — L-System

### O achado que decide o desenho

**A coluna `parent` já existe, e ela é grátis hoje.** O contrato do esqueleto declara
`parent · len · rot · P · wrot` como **colunas ordinárias**
([`crates/ph2d-node-rig-skeleton/src/fk.rs:40-43`](../../crates/ph2d-node-rig-skeleton/src/fk.rs)),
e a nota dele registra a alternativa recusada: *"O plano floated a `Domain::Rig` for this — which
would have meant **unfreezing the node contract**. It is not needed: an element IS a joint, and four
ordinary columns describe the chain."*

⇒ **Uma árvore de L-System é exatamente essa tabela.** A referência emite pontos soltos
(`lsystem.js:73-78`) e a linha entre eles não existe em lugar nenhum — o desenho aparece porque um
duplicador carimba uma forma em cada vértice.

> ⭐ **Emitir `parent` custa uma linha** (guardar o índice do último ponto junto de `(x,y,dir)` no
> `push` da pilha) **e é irrecuperável depois**: uma nuvem sem parentesco já não sabe distinguir
> tronco de folha, e nenhum consumidor a jusante pode reconstruí-lo.

E o consumo já existe: `rig.fk` resolve `(parent, len, rot) → (P, wrot)` sem uma linha nova.

### As decisões, com o motivo

| Decisão | Motivo medido |
|---|---|
| **`Effect::Pure`**, nunca `Temporal` | `Temporal` põe o playhead no fingerprint (`cook.rs:566`) e **mata o memo** — recozinharia a reescrita exponencial a 60 fps. O crescimento animado vem de **fora**, animando o param |
| **CPU-only, com a cerca NOMEADA** | A rota de device exige uma `count_law` que devolve a contagem **antes** do kernel, só dos params (`motion-grid/lib.rs:174-190`). Um L-System **não tem forma fechada** para a contagem de pontos. ⚠️ Escrever isso ao lado do `lowerings` é o que impede a próxima linha de tentar e refazer a medição |
| **Axioma e regras como TEXT PARAM** | `ParamSpec` é `f32`-only (contrato congelado, ADR-0039). Precedentes: a fórmula do `motion.expression`, a curva do `motion.time_remap`, o `branches` do `rig.skeleton` |
| **`BTreeMap`/`Vec` para as regras, nunca `HashMap`** | HR-5 — é a espinha do determinismo. Na referência é um objeto JS, onde a ordem não importa; aqui importaria |
| **`trig.rs` copiada, não `f32::sin`** | HR-5 (sem transcendental instável). A folha já está copiada em **30 crates** — a convenção de drop-crate é *o algoritmo é partilhado, o símbolo não* |
| **Estocástica com `ParamWidget::Seed`** | O `hash3(seed, key, lane)` do `motion.scatter` já existe e é determinístico. E `Seed` é widget, não slider — há gate |
| ⛔ **NÃO construir o desenhador de segmentos** | É irmão do `skeletonRender` (item 8 do doc 92) e serve **os cinco `rig.*` e o L-System de uma vez**. Decisão do Enio se vem junto |

### A ordem das extensões (a régua é o Houdini L-System SOP)

**Primeiro:** estocástica · `Random Scale` · tropismo · decaimento por profundidade (`"`/`!`).
**Depois:** paramétrica (símbolo carrega variáveis).
**Por último:** sensível a contexto — a de **menor retorno visual por preço**.
⛔ **Nunca:** ramificação 3D — em 2D `+`/`-` esgotam o grupo de rotação.

### Os tetos — TODOS por medir

⛔ **Os literais da referência (`MAX_LEN = 30000`, `MAX_POINTS = 4000`) não podem ser copiados.**
Não foram medidos lá, e a §0.0 diz que o caminho lento não define o teto do rápido — a referência é
JavaScript.

| Teto | De que recurso | Como medir |
|---|---|---|
| `iterations` | **tempo de cook** (crescimento exponencial na taxa de expansão da regra) | Sonda no molde de `measure_instance_ceiling.rs`, com as 3 gramáticas canônicas. ⚠️ O teto **depende da regra** ⇒ a saturação real tem de ser pelo **comprimento da string** |
| comprimento da string derivada | **memória** | ⚠️ E ela tem de **dizer que saturou** — corte silencioso é o que o doc 91 chama pelo nome |
| contagem de pontos | tempo de cook + custo por linha a jusante | Comparar com os dois números que já existem: `MEASURED_CEILING = 262 144` (~10–28 ns/linha) e o `motion.fibonacci` a 1 M em 4,622 ms |
| profundidade da pilha `[`/`]` | memória | Precedente de forma: `MAX_JOINTS = 64` no `rig.skeleton` |
| comprimento de `axiom`/`rule` | ⚠️ **Não existe teto de text param em lugar nenhum do repo** — este nó é o primeiro a ter de responder. A resposta certa é provavelmente **saturar na string derivada** e deixar a entrada livre |

---

## §3 — Cel Animation

> ⛔⛔ **CORRIGIDO EM 2026-08-28 — DUAS DAS TRÊS LEIS QUE ESTA SECÇÃO DIZIA FALTAR JÁ EXISTEM.**
> A secção abaixo dizia que ao `motion.sub_uv` faltavam *ping-pong*, *duração desigual* e
> *tocar uma vez*. A medição — [`cel_animation_laws_the_graph_already_has.rs`](../../crates/ph2d-node-registry-init/tests/cel_animation_laws_the_graph_already_has.rs),
> quatro gates que COZEM o grafo e lêem a célula que o artista vê — refuta duas:
>
> | lei | o plano dizia | a medição diz |
> |---|---|---|
> | **inverso** | falta | ⛔ o `speed` do `sub_uv` **já vai a negativo** (`min: -MAX_CELL_SPEED`). **Zero** nós: `[0,5,4,3,2,1,0]` |
> | **ping-pong** | falta | ⛔ o `value.wrap` **já tem `Mirror`** (`MirroredRepeat`, período `2w`). **Um** nó, e sem repetir as pontas: `[0,1,2,3,4,5,4,3,2,1,0]` |
> | **tocar uma vez** | falta | ⛔ o `value.wrap` **já tem `Clamp`**, e segura na ÚLTIMA célula: `[0,1,2,3,4,5,5,5,5]` |
> | **duração desigual** | falta | ⏳ **essa falta mesmo** — a rota que existe é uniforme *por construção*, e há gate a medi-lo |
>
> ⇒ Construir `direction` e `play` como params seria pôr no painel botões que o app já tem
> (`CLAUDE.md` §5.0: *"antes de construir um item de lista aberta, MEÇA se a composição já o
> exprime"*). **O item encolheu de três leis para uma.**
>
> ⚠️ **Como o erro entrou:** a pesquisa leu o que o `sub_uv` *escreve* e o que a `AnimationTag`
> *tem*, e não leu o `min` do slider dele nem a lista de modos do vizinho. *Uma ausência
> afirmada sem olhar a API é um palpite com cara de medição.*

### O achado que decide o desenho

**Não é um nó, e não é um modo do `value.switch`.**

- O `value.switch` roteia **um escalar na coluna `v`** (`value-switch/lib.rs:36-39`), com `N_INPUTS = 4`.
  Um quadro de cel animation é um **stream**. ⇒ ele não alcança.
- **Nenhum nó do catálogo roteia streams** — medido sobre os 131 crates `ph2d-node-*`.
- ⭐ **Mas o `motion.sub_uv` já É o cel animation desta casa**, e o doc dele diz-se assim: *"É o que
  faz um flipbook — uma explosão, uma faísca a piscar, **uma folha de personagem percorrida por
  partícula**"*. Ele tem grelha `cols × rows`, **`speed`** em células/s (o FPS), **`stagger`** por
  elemento (a defasagem que a referência **não tem**), embrulho por `rem_euclid`, e **rota de GPU
  completa**.
- E a cadeia de valor **já é exprimível hoje, sem uma linha**:
  `value.time → value.wrap(Repeat) → value.quantize(Floor) → value.switch.select`, com
  `wrap(Mirror)` dando o ping-pong.

⇒ **O que falta ao `sub_uv` é a LEI DO TEMPO, e ela também já existe** — na `AnimationTag` do Sprite
(`crates/ph2d-ecs/src/sprite_anim.rs`): `per_frame_ms` (**duração por quadro**, e a recusa dela foi
reaberta e fechada com medição), `direction` (4 modos), `repeat`, `hold_ms`, `repeat_delay_ms`,
`signal_on_finish`/`on_loop`, e o **nome** do quadro.

### A forma

> **Um MODO do `motion.sub_uv`, com a lei da `AnimationTag` reescrita como função PURA
> `(tag, t) → célula`, viajando por text param.**

| Decisão | Motivo medido |
|---|---|
| **Reusar a LEI, não o componente** | `SpriteAnimator` é `SimComponent` **per-entidade** com acumulador (`elapsed_ticks`) e relógio de **parede em passo fixo**. Um nó é `Temporal` e **derivado do playhead** — pôr um acumulador nele o tornaria `Stateful` e o quebraria sob scrub. ⚠️ E metade já é grátis: `frame_ticks_at` e `resolve` **já são puras**; o que acumula é só `advance`/`step_ticks` |
| **Índice derivado de `t`, nunca acumulado** | É o que torna o scrub para trás correto **por construção**. Na referência, o `celAnimation` não tem guarda de regressão de tempo e os dois vizinhos (`loopSequencer`, `autoAnimate`) **têm** — porque acumulam |
| **Alternar CÉLULA, não streams** | Alternar streams colide com a lei de contagem (`the_output_count_is_decided_by_branches_nobody_chose`); alternar geometria reordena o desenho todo quadro (a ordem reagrupa por `texture_id`) |
| **Um `mode` novo vai no FIM do enum** | Um documento salvo guarda o **número**, não o nome |
| ⛔ **Nada aqui se chama `substeps`** | A palavra tem dono (só `sim.zone` e `motion.integrate`), com gate. Precedente do preço: a `motion.verlet_rope` usava a mesma chave e a corda caía **4,8× menos** que os gates mediam |

### A recusa medida que já se aplica

⛔ **Se ganhar um knob de duração, ele NÃO leva `set_number_range`.** Recusa R1 da auditoria da §11:
o alcance torna o arrasto **proporcional ao intervalo** — um `frame_ms` de `[1, 60000]` num slider dá
~600 ms de salto por pixel. O `sub_uv` já pagou a mesma lição: `SOFT_CELLS_PER_AXIS = 16` contra
`MAX_CELLS_PER_AXIS = 256`, *"o número não foi escolhido: foi imposto por um gate que já existia"*.

### Os números

⭐ **A maior parte já está medida**, e é o argumento a favor desta forma: `MAX_CELLS_PER_AXIS = 256`
(porque `8192 px / 256 = 32 px` por célula) e `MAX_CELL_SPEED = 120` (porque acima de 60/s a 60 fps a
célula seguinte nunca chega a ser desenhada). ⏳ **Falta medir** o custo de `t → célula` com duração
desigual (busca numa tabela de prefixos) e se `FRAME_MS_MIN` e `MAX_CELL_SPEED` concordam.

---

## §4 — Data Source (CSV/JSON)

### ⛔⛔ O achado que MATA o desenho da referência

**Um `\n` num text param CORROMPE o documento.** O formato do grafo é **linha-orientado**:

- escrita: `writeln!(out, "x {} {} {}", id.0, name, value)` (`nodegraph/src/format.rs:114`);
- leitura: `splitn(4, ' ')`, o valor é o campo final **da linha** (`format.rs:185-196`);
- uma segunda linha do CSV cai no `_ => return Err(ParseError::BadLine)` (`format.rs:281`)
  ⇒ **`MotionDoc::from_text` falha inteiro**, e o `.ph2dproj` (que guarda o doc como `String`)
  **não abre mais**.

⚠️ **E isto é alcançável hoje:** `set_label` recusa newlines **de propósito** (`graph.rs:439-448`) e
`set_text_param` **não** (`graph.rs:335-345`); o commit não sanitiza; e o `TextInput` da casa
**suporta texto com `\n`**.

> ⇒ **Um campo de colar CSV não é uma decisão de UX; é uma mudança de formato.** Ou o dado entra
> **só por CAMINHO** (como o `audio.bands`), ou o record `x` ganha *escaping* — e isso é mexer num
> formato que 5 versões já atravessaram.

**Recomendação: só por caminho.** É o precedente, é barato, e o preço (*missing footage*) é o que
todo DCC sabe nomear.

### O que é cópia e o que é novo

**~70% é o molde do `audio.bands`**, peça a peça: crate-folha com deps mínimas · **arch-gate por
lista-branca** sobre o próprio `Cargo.toml` (com controle positivo) · caminho como text param ·
`Spec::key(file)` por bits com o caminho por último · `eval` = ler o external, ausência ⇒ identidade
silenciosa · membrana irmã chamada de `publish_all` · cache de **dois níveis** (bytes por caminho ·
tabela cozida por chave de conteúdo).

**O que é genuinamente novo:**

1. **O parse.** ⚠️ Medido: `serde_json` **já está na árvore** (7 crates + 2 tools) e acrescentá-lo ao
   shell **não traz uma única crate nova**. ⛔ `csv` **não está** — só o `csv-core` seria
   genuinamente novo, e as licenças (`Unlicense OR MIT`) já estão na allowlist do `deny.toml`.
2. **A tipagem por COLUNA.** Uma `Column` só tem `Scalar/Vec2/Vec3/Vec4` — **uma coluna de texto não
   cabe num `Stream`**. ⇒ colunas não-numéricas são descartadas **com nota**, ou viram índice de
   categoria, ou cor. **Isto é desenho de produto e não tem precedente na casa.**
3. **A política de NOME de coluna.** ⚠️ Um cabeçalho chamado `v`, `falloff` ou `id` **sequestra uma
   coluna reservada em silêncio**. Precisa de decisão: prefixo? recusa? renome com nota?
4. **O mapeamento para `P` é AUTORADO, nunca inferido por nome.** A referência infere de `x`/`y`; uma
   tabela com uma coluna `x` de **anos** seria posicionada por acidente.

### A forma: um nó, os dois lados

> **`data.table` com uma porta `in` OPCIONAL.** Solta ⇒ `n = nº de linhas` (a tabela vira
> instâncias, como o `source.text` faz por glifo). Ligada ⇒ `n` vem da geometria e a linha `i`
> decora o elemento `i`.

⇒ Instâncias **e** colunas com **um** nó, e o precedente é literal (o `audio.bands` tem exatamente
essa porta opcional). Depois de virar coluna, `field.remap` / `value.map_range` / `value.normalize` /
`motion.drive` fazem normalização e faixa **sem um nó novo** — construí-las aqui seria a segunda
resposta à mesma pergunta.

### As armadilhas

| ⚠️ | O quê |
|---|---|
| **`resolved_params`** | A lei de 2026-08-28: a chave é derivada dos params **antes** do cook, e um param **conduzido por fio** só tem valor **durante** o cook. A membrana chama `motion_externals::resolved_params`, **nunca** `node_param_overrides` à mão. Duas das quatro membranas nunca herdaram isto e o sintoma é *a arte desaparece com o nó certo selecionado e nada vermelho em lado nenhum* |
| **`Effect::Pure`, não `Temporal`** | Uma tabela é função do arquivo, não do playhead. `Temporal` faria o cook recomputar a jusante todo quadro **de graça** |
| **Hot-reload é um GESTO, nunca um watcher** | Um watcher faria o cook mudar de resposta **sem nenhuma edição do documento** — o oposto do scrub exato que o módulo promete. Um botão *Reload* que troca a chave |
| **Nada de I/O no cook** | A cerca do `audio.bands` é **estrutural, não disciplinar** (arch-gate por lista-branca). O nó `data.*` não pode ter `std::fs` nem parser: quem lê o disco é o shell |
| **O diálogo passa por `crate::modal::pick_file`** | Um `rfd::FileDialog` aberto à mão **volta a congelar sem declarar** — há gate |
| ⛔ **Não existe `ParamWidget::File`** | O `audio.bands` faz o artista **digitar o caminho à mão**. Criá-lo **paga-se uma vez e cura os dois nós** |
| **A quinta pergunta sobre um nome** | O canal externo já tem quatro chaves por nome (aparência · posição · curva · aparência deslocada). Uma tabela é a **quinta**, e a chave tem de ser prefixada e única, com gate |

---

## §5 — A ordem recomendada, e por quê

| Ordem | O quê | Por que aqui |
|---|---|---|
| **1º** | **Cel Animation** (modo do `motion.sub_uv`) | ⭐ **Menor risco e maior parte já medida.** Não cria crate, não cria dependência, não toca formato, e reusa uma lei já auditada. É a wave que ensina o resto |
| **2º** | **L-System** | Nó novo mas **isolado**: crate-folha, CPU-only, zero dependência externa, zero mudança de formato. O único trabalho de desenho é `parent` (que é uma linha) e os tetos (que são medição) |
| **3º** | **Data Source** | ⚠️ **O de maior superfície**: crate nova + dependência nova (`csv-core`) + membrana nova + provavelmente um `ParamWidget::File` novo + quatro decisões de produto sem precedente (§4). Vale, e é o último porque é o que mais precisa das suas decisões |

⚠️ **Uma decisão sua antes do 3º:** o `ParamWidget::File`. Ele não existe; construí-lo cura o
`audio.bands` (que hoje pede o caminho digitado à mão) e o `data.table` de uma vez. Sem ele, o Data
Source shipa com um campo de texto onde o artista digita `/home/…/vendas.csv`.

---

## ⛔ Recusas MEDIDAS (desta pesquisa)

| Item | Motivo |
|---|---|
| Cel animation como **nó novo** | O `motion.sub_uv` já é o flipbook da casa, **com GPU**; e a cadeia de valor já é exprimível em 3 nós hoje |
| `direction` (inverso / ping-pong) como param novo | ⛔ **MEDIDO 2026-08-28**: o inverso é o `speed` negativo (zero nós) e o ping-pong é o `value.wrap(Mirror)` (um nó). Gate: `cel_animation_laws_the_graph_already_has` |
| `play` (tocar uma vez) como param novo | ⛔ **MEDIDO**: é o `value.wrap(Clamp)`, e ele já segura na ÚLTIMA célula, que é a lei da §11 do Sprite |
| Cel animation como modo do **`value.switch`** | Ele roteia um **escalar**, não um stream — `N_INPUTS = 4`, coluna `v` |
| Reusar o **componente** `SpriteAnimator` | `SimComponent` per-entidade, com acumulador e relógio de parede. Um nó é derivado do playhead; o acumulador o quebraria sob scrub |
| Campo de **colar CSV** | Um `\n` num text param **corrompe o `.ph2dproj`** — é mudança de formato, não de UX |
| **Watcher** de arquivo | Faria o cook mudar de resposta sem edição do documento |
| Copiar `MAX_LEN`/`MAX_POINTS` da referência | Não foram medidos lá; e a referência é JavaScript (§0.0) |
| L-System com `Effect::Temporal` | Mata o memo e recozinha reescrita exponencial a 60 fps |
| L-System no **device** | A `count_law` precisa da contagem antes do kernel; um L-System não tem forma fechada |
| Ramificação **3D** no L-System | Em 2D, `+`/`-` esgotam o grupo de rotação |
| Normalização/faixa **dentro** do Data Source | `field.remap`, `value.map_range`, `value.normalize` já existem |
| Inferir `P` de colunas chamadas `x`/`y` | Uma coluna `x` de anos seria posicionada por acidente |
