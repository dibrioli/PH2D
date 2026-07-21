# ADR-0133 — Nesting: uma instância de container é um STRIP, e o relógio é do PAI

**Status:** proposto · **Data:** 2026-07-18 · **Linha:** `line/anim-fixes` (worktree `line-anim`)
**Pesquisa:** [`docs/Timeline/03_pesquisa_nesting.md`](../../Timeline/03_pesquisa_nesting.md) (4 frentes, fontes primárias)
**Precede:** [ADR-0115](0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md) (composição de clips) · [ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md) (hierarquia única) · [ADR-0121](0121-vector-live-corners-authored-source-cooked-geometry.md) (fonte ≠ cozido)

---

## Contexto

O ADR-0115 nomeou o nesting como **fora de escopo e próximo ADR**, textualmente: *"é o idioma 2D
de reuso e nós temos zero"*. A composição de clips cobre *"transição entre dois estados"*; o
nesting cobre *"esta peça de animação é uma coisa, e eu a uso onde quiser"*.

**Três premissas do briefing desta linha estavam desatualizadas, e as três encolhem o trabalho.**
Verificadas no código, não herdadas:

1. **"O relógio sob pilha é indefinido" — foi FECHADO.** Existe hoje a composição
   *outer-then-inner* (R6): `timeline → ClipStrip::source_time → clip → Time Remap da entidade`
   (`stack_eval.rs::strip_source_time`). E quando o mapa **não é função**, o sistema **recusa e
   diz o motivo** — `KeyRefusal::{NotPlaying, PlaysTwice}` via `sole_strip_of`, com a lei escrita:
   *"se o clip que você edita está tocando duas vezes neste instante, 'aqui' tem duas respostas"*.
2. **O `O(bindings²)` que o 0115 §4 listava como pré-requisito também foi pago** — `clock.rs`
   hoistou o relógio para uma resolução por ENTIDADE (`ClockIndex`), citando explicitamente que
   *"um laço de strip dentro de um apply quadrático seria cúbico"*.
3. **A tensão "aba vs breadcrumb" era falsa** — os dois eixos são ortogonais (§3 abaixo).

Ou seja: **o nesting não precisa inventar a lei do tempo. Precisa decidir se é mais um elo da
mesma cadeia.** Este ADR diz que sim.

---

## Decisão

### 1. O relógio é do PAI, e a cadeia ganha UM elo do MESMO tipo

**Quatro dos cinco produtos pesquisados dão o relógio ao pai** (AE, Rive, Animate-graphic,
Harmony, Cavalry). O único filho com relógio próprio — o *movie clip* do Animate — é justamente o
que produz o maior corpus de confusão documentado, e sempre pelo mesmo motivo: **o relógio próprio
torna o filho invisível em autoria** (ele não anima no palco do editor; só toca em runtime).

Para uma engine cujo gesto central é **scrub determinístico**, isso decide sozinho.

A cadeia passa de três elos para quatro, todos da mesma família:

```
timeline t → strip.source_time → [container: sua própria pilha] → clip t → Time Remap da entidade → t_fonte
```

**A lei da recusa se estende sem código novo de política:** `sole_strip_of` já responde *"este
clip está tocando exatamente uma vez agora?"*, e sob nesting a pergunta é a mesma feita
recursivamente. Um container que toca duas vezes torna "aqui" ambíguo exatamente como um strip
duplicado torna — e a resposta certa continua sendo **recusar e nomear**, nunca escolher em
silêncio.

⚠️ **A armadilha que a pesquisa nomeou e que este desenho precisa evitar por construção:** o AE
tem um **bug aberto** em que *Essential Properties não se aplicam depois do frame 0 quando Time
Remapping está ativo no precomp* — um canal do PAI morre quando o FILHO é remapeado. A defesa é
que o remap do filho seja um elo **dentro** da cadeia de tempo e não um interceptador do canal de
valor: o tempo compõe, o valor não passa por ele.

### 2. Uma instância de container é um `ClipStrip` — o campo que muda é a FONTE

O `ClipStrip` **já carrega exatamente o conjunto que a pesquisa achou ser o mínimo universal de
override por instância**, e isso não é coincidência — é a mesma pergunta respondida duas vezes:

| O que todo produto oferece por instância | O que o `ClipStrip` já tem |
|---|---|
| qual tempo/animação toca dentro | `src_in` / `src_out` / `speed` / `loop_mode` |
| onde ela toca no pai | `t_start` / `t_end` |
| como entra e sai | `ease_in` / `ease_out` / `lead_in` |

`speed` é o `speed()` do Rive; `src_in` é o `First` do graphic symbol do Animate; `loop_mode` é o
`{Loop, Play Once}` dele; `t_start/t_end` é o in-point + time-stretch do AE.

**Decisão:** `ClipStrip.clip: u16` vira `ClipStrip.source: StripSource`, onde

```rust
pub enum StripSource { Clip(u16), Container(u16) }
```

Nada mais no strip muda. Uma faixa pode misturar strips de clip e de container, o crossfade por
sobreposição continua valendo, e os canais esparsos continuam valendo — porque **o que blenda é a
saída, e a saída de um container tem a mesma forma que a de um clip**.

> **Postcard é posicional:** isto **SUBSTITUI** um campo em vez de apendar um, então os bytes de
> um v7 não estão "faltando" — significam outra coisa dali em diante. **`DOC_VERSION` 7 → 8, e um
> v7 é REJEITADO.**
>
> ⚠️ **Correção (2026-07-18, Fatia 1):** a 1ª versão deste ADR dizia *"o load de v7 migra
> `clip: u16` para `Source::Clip(u16)`, migração total sem perda"* — e, na frase seguinte, *"o load
> rejeita versão desconhecida"*. As duas não podem ser verdade, e a segunda é a política da casa:
> `from_bytes` recusa QUALQUER versão ≠ atual, e o gate que já existia diz textualmente *"an older
> blob must be REJECTED by the version gate, not misread field-for-field"*. Não há maquinário de
> migração neste documento, e inventar um dentro de uma fatia de dados seria contrabando.

### 3. O container é um ASSET REFERENCIADO no DOCUMENTO — não um tipo novo de entidade

**Ninguém escolheu árvore-de-objetos pura.** Unity, Godot e Blender *parecem* árvore, mas os três
promovem a árvore a **ID endereçável** e a instância a **referência + diff**. É o híbrido, e é a
maioria esmagadora. E quem chegou mais perto de árvore-primeiro — o Blender — é quem tem a dor
documentada: ciclos acontecem mesmo assim, e o código carrega um conserto automático no load que
**apaga o `instance_collection` do usuário em silêncio**.

**E o repo já tem a metade organizacional pronta, sem saber que ela era metade.** O Harmony separa
de propósito duas coisas que se parecem:

| Harmony | O que é | Nós |
|---|---|---|
| **grupo** | organização; **expande em linha**; mesmo relógio | **`GroupedChildren`** — marcador de tamanho zero, só semântica de seleção/lock, **zero tempo** |
| **símbolo** | nesting temporal; **entra**; timeline independente | **não existe — é o que este ADR cria** |

Nossa hierarquia ECS é o *grupo* do Harmony, exatamente. O container é o *símbolo*. **Eles não
competem: são as duas metades, e nós já temos uma.**

⚠️ **Não contradiz o [ADR-0129](0129-vector-envelope-warp-one-spine-cage-as-container-entity.md)
("a gaiola é uma entidade-container").** A gaiola é um container **geométrico** — ela deforma
espaço e por isso tem de viver onde a pose vive. Este é um container **temporal** — ele agenda
tempo e por isso vive onde os clips vivem (o documento). Eixos diferentes; a palavra é que é a
mesma. Se um dia um objeto for os dois, ele carrega os dois, e nenhuma das duas definições precisa
ceder.

### 4. Ciclo: DFS na CRIAÇÃO do link, e re-checagem no LOAD — duas camadas, gate por camada

O padrão é unânime e **nenhuma camada é em runtime**. Godot e Blender pagam duas (e o Blender,
três) pelo motivo que o Godot escreve no fonte: a árvore vira cíclica por caminhos que **não
passam pelo "add"**.

1. **Na criação:** DFS ancestral antes de aceitar o link; a recusa é **visível e nomeada**
   (`KeyRefusal` tem irmão: `NestRefusal::WouldCycle`). O AE e o Animate recusam em **silêncio** e
   isso é o pior dos dois mundos — a pesquisa não achou sequer a mensagem de erro deles.
2. **No load:** re-checagem, e **rejeição do documento**, nunca conserto silencioso. O
   auto-reparo do Blender apaga trabalho do usuário sem avisar; um documento cíclico é um bug
   nosso a ser corrigido, não um arquivo a ser mutilado na abertura.

Isto é defesa em camadas ⇒ **cada camada tem gate próprio**
([[feedback_layered_defenses_need_per_layer_gates]]): a mutação de uma só não pode ficar verde
porque a outra segurou.

### 5. UI: breadcrumb E aba — são eixos ortogonais, e o AE já provou que duas réguas funcionam

A tensão que este ADR nasceu para resolver se dissolve quando se separa o que cada controle
responde:

- **A breadcrumb diz ONDE VOCÊ ESTÁ** (que container, em que profundidade). É a linhagem 2D —
  Animate e Harmony —, e a linhagem de composição (AE, aba nova) tem o sintoma documentado
  repetidamente: perde-se o contexto do pai, e a comunidade **reconstrói manualmente** o
  edit-in-place travando viewers.
- **A aba diz QUAL METADE VOCÊ OLHA** (`Tab::{Keys, Arrange}`), e essa doutrina — *"uma régua mede
  um relógio"* — **não muda**: ela passa a valer no nível em que a breadcrumb te pôs.

⚠️ **Correção ao briefing:** o **Figma não tem breadcrumb** (é deep-select + salto para a fonte),
então ele não é o precedente que o §2.3 do briefing sugeria. Os precedentes são Animate e Harmony.

**E o relógio do container fica VISÍVEL.** A pesquisa achou um só mecanismo implementado de
fato: quando há remap, o AE mostra **duas réguas** — a de baixo é o tempo do pai, a de cima o da
fonte, com um marcador ligando as duas.

⚠️ **EMENDA (2026-07-18, na Fatia 3e): não copiamos as duas réguas, e a razão é que a premissa
delas não vale aqui.** A régua de cima do AE existe porque o Layer panel **não tem outro lugar
onde o tempo da comp apareça**. O nosso tem: o chip de segundos da barra de transporte mostra
`snap.time_seconds` — o tempo da CENA — e ele nunca deixou de estar na tela dentro de um
container. Uma segunda régua gastaria uma linha permanente de um painel curto para re-exibir um
número que já está lá. (A nota que dizia o contrário foi escrita sem conferir o `transport.rs`;
ela é o erro, não o desenho.)

O que de fato falta com dois relógios na tela é a **RELAÇÃO** entre eles: ver `8.00` no chip e a
marca no `4` da régua é uma contradição até você saber onde a instância começa. Isso é uma linha
de texto, não uma régua — o readout ao lado da trilha diz `plays 4.00 - 12.00`, e diz **`not
playing here`** quando o container não toca no segundo atual, que é *por que* a marca some em vez
de a régua estar quebrada.

E a régua **é** rotulada, de um jeito: a trilha ao lado dela nomeia o container. A pesquisa
observou que nenhum produto rotula a sua — e é exatamente por isso que *"que relógio é este?"* é
confusão de pé em todos eles.

### 6. O que NÃO fazemos, e por quê

- **Relógio próprio por container (o *movie clip*).** É a fonte documentada de mais confusão de
  usuário em toda a pesquisa, e mata o scrub determinístico. Se algum dia houver máquina de
  estados/interação, ele volta como **opt-in explícito por instância** — nunca como default.
- **Cache de saída do container.** O Rive não cacheia nada e sobrevive; o AE cacheia e paga em
  invalidação. Sem um consumidor que doa, cachear é escolher um modo de falha
  (*luz velha que ninguém vê que é velha*) antes de ter o problema. **Gatilho que o acorda:** o
  kill-criterion do §Kill ser excedido por instâncias IDÊNTICAS — aí o desconto certo já tem nome
  (o *Master Pose Component* do Unreal: uma avaliação, N cópias do resultado).
- **Teto de profundidade.** Ver §Kill: **não temos o recurso que o justifique**, e o §0.0 proíbe
  escrever um número sem a medição que o nomeia. Nenhum produto pesquisado publica um limite
  medido — nem o Rive, que também não tem detecção de ciclo.
- **Nesting de vetor/pintura dentro do container** (o interior conter formas do módulo Vector ou
  documentos do Painter): fora do 1º corte, e nomeado. O interior deste ADR é **animação**.

---

## ⚠️ A bifurcação que o Enio decide, não eu

Há duas coisas que a palavra "nesting" pode significar aqui, e elas custam ordens de grandeza
diferentes. Este ADR recomenda **(B)** e a decisão acima está escrita para (B) — mas a escolha é
de produto, não técnica, e é por isso que a linha **para aqui**.

**(A) Container temporal, instância única.** Um container agrupa clips no tempo e é usado **uma
vez**. Barato: cai quase inteiro do maquinário existente. **Mas não é o multiplicador** — é um
grupo com relógio, e grupo nós já temos (`GroupedChildren`). O reuso, que é a razão de existir do
nesting, não chega.

**(B) Container instanciável N vezes.** *"Esta peça é uma coisa, e eu a uso onde quiser"* — o
briefing pediu isto, e é o que todo produto que shipou nesting entrega. Custa: instâncias
precisam de identidade própria, o apply roda por instância, e a interação com **ordem de desenho**
é real.

⚠️ **O aviso mais forte que a pesquisa trouxe, e ele é contra (B):** o **Spine** — o motor 2D
esqueletal mais maduro do mercado — **nunca implementou nesting em 10 anos** (enhancement aberto
desde 2016). A razão estrutural está na doc deles: cada skeleton é desenhado inteiro antes do
próximo, e não dá para intercalar draw order entre skeletons — *"if that is needed, it is easiest
to use a single skeleton"*. **Nesting e ordem de desenho global brigam.**

Nós temos a mesma tensão viva: o z-order é projeção da árvore única
([ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md)), e um container que instancia
sua sub-árvore N vezes precisa dizer onde cada cópia entra nessa pilha. **Não é intransponível — é
o item que precisa de resposta antes da 1ª linha de código de (B)**, e por isso é a Fatia 0 do
plano.

---

## Conjunto de aceitação (concreto e CONGELADO — DIRETIVA §5)

Cada item é executável e nasce VERMELHO.

1. `a_container_instance_is_a_strip_and_reads_the_parents_clock` — um container instanciado a
   `t_start=3, speed=2` amostra o interior no tempo composto, e o resultado é **idêntico** ao do
   mesmo conteúdo achatado num clip com o mesmo mapa. *Mutação:* usar o playhead cru em vez do
   composto ⇒ RED.
2. `the_clock_composes_outer_then_inner_at_every_depth` — a 3 níveis, `key_home` e a amostragem
   dão o MESMO instante (a lição `feedback_derived_coordinate_seed_must_match_sample`, agora
   recursiva). *Mutação:* inverter a ordem de composição num nível ⇒ RED.
3. `a_container_playing_twice_refuses_the_key_and_names_why` — `NestRefusal`/`KeyRefusal`
   propagam pela recursão; a recusa é **visível**, nunca uma escolha silenciosa.
4. `linking_a_container_into_itself_is_refused_at_the_gesture` — DFS ancestral; mensagem nomeada.
5. `a_cyclic_document_is_rejected_at_load_not_repaired` — o oposto do auto-reparo do Blender.
   ⚠️ **Gate por camada**: 4 e 5 são independentes; neutralizar um não pode deixar o outro verde.
6. `the_schema_is_eight_and_a_v7_blob_is_refused` — v7 recusado, não mal-lido (ver a correção
   acima; o gate desta linha originalmente prometia migração).
7. `the_ruler_shows_the_parents_clock_and_the_sources_together` — seam que CLICA: entrar num
   container publica a breadcrumb e troca o relógio da régua; sair restaura. (Emenda da 3e: **as
   duas réguas viraram um readout**, ver §5. E o gate que faltava era outro — a régua LIA no
   relógio do container e ESCREVIA no da timeline: `scrubbing_inside_a_container_seeks_the_
   timeline_second_not_the_local_one`, Fatia 3d.)
8. **Perf**: o gate do §Kill.

---

## Kill-criterion (declarado ANTES do build, e o baseline é MEDIDO)

**Baseline medido hoje** (2026-07-18, workstation, release, na árvore DESTA linha,
`cargo test -p ph2d-timeline --release --all-features --test apply_perf -- --ignored`), **4 execuções**:

```text
  bindings   us/apply (4 runs)        us/binding
       350   7.8 · 8.5 · 10.5 · 10.6   0.022 – 0.030
      1400  50.1 · 50.2 · 63.3 · 61.1   0.036 – 0.045
  4x dados -> custo por binding x1.48 – x1.62   (antes do hoist do ClockIndex: x2.84 – x2.91)
```

⚠️ **A métrica que vale é a FORMA, não o wall-clock** — e isso não é preferência minha, é o que o
próprio harness manda: *"total wall-clock ratios are a poor test… the cost per binding does not
have that problem"*. O wall-clock varia ~25% entre execuções (a primeira de cada sessão é sempre a
lenta: cache frio e turbo). **O crescimento por-binding, que é o que revela a lei, fica em
1,48–1,62 — linearítmico, e longe do ~4× que a forma quadrática daria.** O hoist do `ClockIndex`
segue de pé.

Reproduz a medição de 2026-07-12 (51,84 µs @ 1400). **1400 bindings custam ~0,3% de um frame de
60 Hz.**

**O kill, como declarado:** um documento de **8 containers × profundidade 3** (a escala de um rig
2D real) deve aplicar em **< 2× o custo do mesmo número de bindings achatado**.

### ⚠️ A barra declarada FALHOU, e a substituta é medida (2026-07-18, Fatia 2)

Medido (release, 300 bindings, 8 instâncias sobrepostas): **~2,1–2,9× a profundidade 3**. A barra
falhou como escrita. Mas medir por PROFUNDIDADE mostrou o porquê, e o porquê isenta o desenho:

```text
  depth 1: 1,51x   depth 2: 1,89x   depth 3: 2,11x   depth 5: 2,59x   depth 8: 3,40x
  ratio/(depth+1): 0,754 → 0,630 → 0,528 → 0,432 → 0,377   (CAI)
```

O custo é **linear na profundidade, inclinação ~0,27/nível**, e a coluna normalizada **cai**: cada
nível a mais custa menos que o primeiro. **Um "2×" é uma CONSTANTE, e avaliar N níveis é
honestamente trabalho de N níveis** — a barra media a grandeza errada, e eu a escrevi antes de ter
qualquer medição, que é exatamente o que o §0.0 adverte.

**A barra nova pina a LEI, não o relógio:** dobrar a profundidade não pode mais que dobrar o
sobrecusto (`the_cost_of_depth_is_linear_not_explosive`, teto 2,9× para o salto 3→8, onde linear
seria 2,67×). É a mesma doutrina do `apply_perf` deste módulo. Fosso medido dos dois lados: são
**2,22–2,27** (±2%), mutante (fatia por-frame → varredura completa) **3,42–3,50**.

⚠️ **E o wall-clock absoluto depende da ORDEM em que se mede** — o mesmo depth 3 deu 2,11 e 2,94
conforme o aquecimento —, o que é a segunda razão para gatear um ratio-de-ratios e não um limiar.

**E é por isso que NÃO há teto de profundidade** (§0.0 — um limite legítimo diz de que recurso ele
é): a ~0,04 µs/binding, mesmo a profundidade 10 sobre 1400 bindings custaria ~600 µs = **3,6% do
frame**. **Tempo de avaliação não justifica um teto raso**, e nenhum outro recurso foi medido
ainda. Se alguém quiser um limite, tem de medir **memória** ou **profundidade de recursão** e
escrever o número que a medição deu. *"Por segurança" é um palpite esperando um smoke.*

⚠️ **O custo que este ADR NÃO mediu, e que pode ser o verdadeiro:** o de **compor a IMAGEM** de N
containers (a armadilha (a) da pesquisa — cada nível não-colapsado materializa um raster
intermediário). É medição da Fatia 0, não deste documento, e é a única razão pela qual a
bifurcação (B) pode custar mais do que o número acima sugere.

---

## Plano por fatias (depois do aceite, não antes)

- **Fatia 0 — a pergunta do z.** Onde a sub-árvore de cada instância entra na pilha única de
  z-order, e quanto custa compor a imagem de N instâncias. Medição + decisão; **é o pré-requisito
  do resto** (o aviso do Spine).
- **Fatia 1 — dados headless.** `StripSource`, migração `DOC_VERSION` 7→8, DFS de ciclo nas duas
  camadas. Gates 1-6. Zero UI.
- **Fatia 2 — o relógio recursivo.** Composição e recusa em profundidade; gates 2-3 e o do §Kill.
- **Fatia 3 — UI.** Breadcrumb, entrar/sair, o relógio da régua (3c), o **mapa da instância**
  nas duas direções (3d) e o readout da janela (3e — que substitui as duas réguas, §5). Gate 7.

---

## Consequências

- **`ClipStrip.clip` deixa de existir.** É quebra de compilação, não silêncio — o que se quer.
- **`DOC_VERSION` 7 → 8**, com migração total de v7.
- A hierarquia ECS **não muda** — `GroupedChildren` continua sendo organização pura, e passa a ter
  um irmão explícito em vez de uma ambiguidade.
- O `Tab::{Keys, Arrange}` **não muda de significado**; ganha um nível onde valer.
- **O contrato congelado não é tocado** (§6 do CLAUDE.md): nada de `NodeOp`/`NodeManifest`/`Tool`.

## Alternativas consideradas

- **Container com relógio próprio (movie clip).** Rejeitado no §6: invisível em autoria.
- **Container como entidade ECS com filhos.** Rejeitado no §3: nenhum produto escolheu árvore
  pura, o Blender documenta a dor, e nossa árvore já é a metade *organizacional*.
- **Instância única (opção A).** Não rejeitada — **devolvida ao Enio** na bifurcação acima. É
  barata e honesta, mas não é o multiplicador que o briefing pediu.
- **Achatar o container no load (bake).** Rejeitado: mata a edição não-destrutiva, que é a razão
  de o container existir. Continua disponível como *export*, que é como o AE o oferece
  (pre-render + proxy).

---

## Emenda (2026-07-20) — o mapa é da ENTRADA, e o transporte segue o animador

O smoke do exemplo completo (`PH2D_NEST_SMOKE=2`, 3 clips / 2 lanes / 3 instâncias) derrubou
quatro coisas, e duas eram de DESENHO deste ADR:

1. **O mapa da instância (§5/Fatia 3d) deixou de vir do scratch e passou a vir do CAMINHO DE
   ENTRADA** (`entry_map` sobre `EnterStep { container, lane, strip }`). A versão aceita aqui
   derivava o mapa de *"o único strip tocando agora"*, e as duas recusas que ela fazia eram
   fabricadas: nos VÃOS entre instâncias o container toca zero vezes (o mapa — e o arrasto da
   régua com ele — piscava), e com o container instanciado 2+ vezes recusava sempre. Mas entrar
   é clicar num STRIP: a instância que o animador quer dizer está nomeada pela própria
   caminhada, então não há ambiguidade a recusar. O que segue recusando, e deve: instância que
   DÁ A VOLTA (sem inverso) e caminhada obsoleta (strip deletado/retargetado). O par
   scratch-based (`container_playhead`/`container_map`) morreu sem consumidores.
   É a lição de [[feedback_an_impossible_inverse_is_a_reason_for_a_second_clock_not_a_readonly_control]]
   aplicada de novo: a resposta a um inverso impossível era dar ESTADO à vista, não travar o
   controle.
2. **Entrar num container ajusta o loop do transporte à janela da instância** (leads
   incluídos, `entry_reach`); sair reinstala o loop do DOCUMENTO (`sync_transport_loop`) —
   navegação não é edição, nada escreve no doc. Edge-triggered: o gesto do artista sobre o
   Loop, dentro, continua ganhando.
3. **O "fim" da pilha inclui o `lead_out`** (`stack_end_seconds` → `lead_end()`): um loop
   recém-armado que parava em `t_end` cortava do ciclo a própria fade que o artista autorou.
4. **A aba Keys dentro de um container entrava em pânico** — o rebuild do snapshot pulava o
   prime em `keys_mode` e o braço do host lia o scratch mesmo assim. Os readouts do host agora
   são Arrange-only (e puros — sem scratch).

Gates: `nesting_map.rs` (reescrito para `entry_map`/`entry_reach`), `nesting_view.rs` (novo,
o lado do snapshot), `timeline_bridge_tests` (o loop de navegação), `lead_in.rs` (o extent).
Mutações: 6/6 sangram no gate certo + o extent nasceu vermelho.
