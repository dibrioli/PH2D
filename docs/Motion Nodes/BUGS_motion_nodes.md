# Bugs do módulo Motion Nodes — registro + soluções

> Log vivo dos bugs **não-triviais** do sistema de nós (sintoma → causa-raiz → hipóteses que
> caíram → medição → solução → lições). O objetivo não é listar todo fix (o git já faz isso),
> mas registrar os bugs cuja **causa enganava** — aqueles em que a aparência, ou o nó que o
> report acusou, levou o diagnóstico para a pista errada. Cada entrada termina em **lições
> generalizáveis**, para o próximo agente não repetir o erro.
>
> Estado por-wave: os handoffs em [`handoffs/`](handoffs/). O catálogo por família:
> [`89_conferencia/`](89_conferencia/).

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--o-nó-acusado-estava-inocente-o-campo-gateia-o-pulso-e-não-gateia-a-memória) | **"Box inconsistente"** — marcar Invert e desmarcar não devolve o quadro inicial | `field.box` (acusado, **inocente**) + `pulse.counter`/`pulse.sample_hold` (a memória) | ✅ **Fechado — smoke aprovado** (porta `reset`) | 2026-08-10 |
| [2](#bug-2--o-glow-e-a-forma-vivem-nos-dois-lados-do-tonemap-e-o-lod-troca-o-lado) | **"Glow não funciona com shape"** | `fx.glow` (acusado, **inocente**) + a partição de render sprite ⁄ vetor | ✅ **CURADO** (aguarda smoke) — o passe passou a ler a CAMADA | 2026-08-20 |
| [3](#bug-3--o-diagnoser-sabia-e-ninguem-perguntava) | **"Todas as peças paradas"** (cena `=71`, banda 6) | uma cena com um fio a menos — e o INSTRUMENTO que ninguém invocava | ✅ **CURADO** (aguarda smoke) — mais um falso positivo pré-existente do diagnoser | 2026-08-20 |
| [4](#bug-4--o-multiply-não-desobedecia-à-alfa-ele-a-invertia-e-o-gate-media-o-único-ponto-em-que-os-modos-concordam) | **"Shadow multiply não obedece o alpha"** (cena `=84`) | `fx.drop_shadow` (acusado, **inocente**) + o par de fatores do `Multiply` em `ph2d-render` | ✅ **FECHADO — smoke aprovado** (cena `=84`, linha ALFA) — resposta invertida, num gate verde há anos | 2026-08-23 |
| [5](#bug-5--o-editor-de-curva-era-oferecido-numa-onda-que-não-o-lê--e-o-censo-que-o-teria-apanhado-não-podia-vê-lo) | **"Wave curve dos osciladores não está funcionando"** (com foto) | `motion.oscillator` (o MOTOR, **inocente**) + a tabela de gates dele — e mais DOIS nós pelo mesmo mecanismo | ✅ **CURADO** (aguarda smoke, cena `=94`) — o editor aparecia em toda onda e só era lido na `Custom` | 2026-08-24 |

---

## Bug #1 — O nó acusado estava INOCENTE: o campo gateia o pulso, e não gateia a MEMÓRIA

**Estado:** ✅ **FECHADO** em 2026-08-10 — mecanismo medido, cura construída (a porta `reset`),
gate `a_round_trip_of_the_field_leaves_the_scene_where_it_found_it`, e **smoke aprovado pelo
Enio** na cena `PH2D_GPU_COOK_DEMO=23`.

### Sintoma

Report do Enio na cena `PH2D_GPU_COOK_DEMO=23` (O PORTÃO ESPACIAL):

> *"Nó Box inconsistente! Ao checar Invert e depois desmarcar, o resultado é diferente do
> inicial."*

Na tela: o losango pisca; marcar **Invert** e desmarcar devolve a máscara, mas a arte fica
**invertida** — a área de FORA acesa e parada, o miolo apagado. Lido do assento do artista, o
`field.box` "esquece" o estado dele.

### O que a medição disse (e ela derrubou o próprio report)

Sonda `probe_invert_round_trip`, no caminho REAL (o `set_param` que o checkbox emite + o `Cook`
que a cena usa), 262.144 linhas:

| medição | resultado |
|---|---|
| **M1 — o NÓ**: `invert` 0 → 1 → 0 | `invert` mudou **262.144** linhas · o ida-e-volta difere em **0** |
| **M2 — a CENA** no tique 120 | difere em **262.144 (100%)** |
| **M3 — o QUE difere** (sem toggle) | DENTRO **33.540/33.540** grandes · FORA **0**/228.604 |
| **M3 — o QUE difere** (pós ida-e-volta) | DENTRO **0**/33.540 · FORA **228.604/228.604** grandes |

⚠️ **O `field.box` é uma função PURA dos params** — o ida-e-volta re-deriva a máscara **byte a
byte**, em todas as linhas. E só existem dois tamanhos na cena (`0.0180` = repouso, `0.0480` =
crescido): é **paridade limpa**, não corrupção. O retrato pós-ida-e-volta é o **inverso EXATO**
do inicial.

### A causa-raiz

O que muda de lugar não é a máscara: é o **`count_tick` do `pulse.counter`**, que vive no `pre`
self-loop. Enquanto o campo está invertido, quem está **FORA** recebe as batidas do metrônomo e
avança a paridade; quem está **DENTRO** congela. Desmarcar devolve a máscara e **não** devolve a
memória.

A aritmética prevê a medição sem folga: a janela do toggle contém **uma** batida (`period` 0,5 s),
então cada lado fica exatamente **um** pulso fora de fase — e um contador `count_max = 2, Wrap` é
paridade, logo um pulso de diferença é a inversão completa.

⚠️ **E a informação que faltaria para consertar é DESTRUÍDA antes do contador:** o portão é um
`value.math(Multiply)`, e ele colapsa *"não há pulso agora"* e *"esta linha saiu do campo"* no
**mesmo zero**. O `pulse.counter` não tem como distinguir os dois — e não tem porta de **RESET**
(`inputs` = `pulse` + `state`, o self-loop). *O campo consegue gatear um EVENTO; ele não tem como
gatear ESTADO.*

### Por que nenhum gate viu (a pergunta 4 do protocolo)

`the_scene_blinks_only_inside_the_box` cozinha de `t = 0` com os params **FIXOS**, e o irmão
`the_gate_is_the_pulse_not_the_drives_own_mask` cozinha **um** quadro. ⚠️ **Nenhum gate deste
repositório cozinha uma cena ATRAVÉS de uma edição de param** — a classe inteira *"autorar sobre
um grafo vivo com estado"* estava sem cobertura. Os gates provam o comportamento **inicial** e
são estruturalmente cegos ao **gesto**.

### O que foi corrigido agora

1. **O doc do `field.box` MENTIA em duas frases** (e um doc que mente é parte do bug): ele abria
   com *"an **axis-aligned** rectangle"* e a lista de params **omitia o `rotation`** — num nó que
   tem o param, tem gate de rotação, e cuja rotação de 45° é o que faz o **losango** desta cena.
2. **A prosa da cena dizia *"Fora dela nada acontece, NUNCA"***. O "nunca" é falso depois de uma
   edição de campo — corrigido para nomear a condição.
3. **O gate executável** (nasceu como `the_field_gates_the_pulse_but_not_the_counters_memory`
   e foi reescrito para a lei nova quando a cura landou, que é o que o doc dele mandava fazer;
   o retrato invertido sobrevive nele como **CONTROLE**, com o fio do `reset` desligado por
   ablação), com as duas
   metades: a que **inocenta** o nó (pura, e `invert` morde todas as linhas — sem essa segunda
   metade o gate ficaria verde sobre um memo que ignorasse o param) e a que **mede** a inversão.
   Prova de mutação: um `field.box` impuro (o flip guardado em vez de derivado) faz a metade da
   pureza sangrar em **262.144** linhas.

### A CURA (construída em 2026-08-10, ordem do Enio: *"cubra todos os nós similares"*)

**A porta `reset`** — a que torna o estado ALCANÇÁVEL. Nível (o `Reset` do TD Count CHOP),
`reset_to` como destino, o reset ganha o tique, e **desconectada é o mundo anterior byte a
byte** (stream vazio ⇒ zeros ⇒ nada passa de `0.5`; nenhum caso especial).

⚠️ **E o "similar" foi MEDIDO pelo mecanismo, não presumido.** Dos 18 nós com porta de
feedback, o estado que **não se auto-cura** são três: `compare`/`threshold` reescrevem o
`armed` todo tique, `on_change` guarda o valor DESTE tique e `beat` deriva do TEMPO. Sobram:

| nó | estado | o que ganhou |
|---|---|---|
| `pulse.counter` | `count_tick` acumula | porta `reset` + param `reset_to` |
| `pulse.sample_hold` | o valor segurado sobrevive sem trigger | porta `reset` (⚠️ **RE-PRIMA**, não zera — o nó recusa *"a dead 0"* no boot, e a porta `value` já é o caminho de injetar número) |
| `motion.step` | o mesmo `count_tick` | ⚠️ **JÁ TINHA** — ver abaixo |

⚠️ **O ancestral já shipava a porta, e a conferência não tinha visto.** O `motion.step` — de
onde o contador saiu (*"the count math is `motion.step`'s, verbatim"*) — tem `reset` +
`reset_to` com a **lei idêntica**, derivada de forma independente nesta wave antes de eu a
encontrar. ⇒ o item da folha 12 não era *omissão do catálogo*, era a **mesma classe da linha
42 dela**: *"o redutor perdeu a capacidade do ancestral"*. Os nomes foram alinhados
(`reset_to`, não `reset_value`) e cada cópia da lei nomeia a outra — elas têm de se mover
juntas, e são cópias porque cada nó é uma leaf drop-crate.

### O que o reset NÃO promete (medido, não suposto)

Ele devolve o **REPOUSO**, não a **história**. E o número que quase virou uma lei falsa:

| janela do ida-e-volta | batidas perdidas | quadro final |
|---|---|---|
| `(20, 50)` | 1 | **byte-idêntico** ao controle (0 de 262.144 diferem) |
| `(20, 80)` | 2 | fora em repouso ✅, mas o DENTRO em fase oposta |

⚠️ **Com UMA janela eu quase escrevi *"volta ao lugar exato"* como a lei** — era coincidência
de paridade (3 e 5 contagens são ambas ímpares). O gate roda **duas** janelas de propósito, e
afirma o que é verdade nas duas: *fora do losango, repouso; dentro, todos na mesma fase*.

### E a cura precisou de um segundo achado: UM número, DUAS perguntas

A primeira fiação levou o resíduo de 228.604 para **2.640** — e os 2.640 tinham endereço: a
**banda macia** do losango. O portão arma em `peso ≥ 0.5` e o reset disparava no `fall`
default (`0.3`) ⇒ as linhas cujo peso vivia entre 0,3 e 0,5 **contavam e nunca resetavam**.
*Dois números para uma pergunta é a banda; um número é zero* — o limiar virou o
`PARTICIPATES_AT` da cena, lido pelos dois nós. **2.640 → 0.**

### O que segue ABERTO (fora do escopo, nomeado)

O `pulse.adsr` (o outro P1 da folha 12) **não** foi construído: um envelope que volta ao
repouso sozinho é uma capacidade legítima, mas é **nó novo**, não a cura deste defeito — e o
reset é o que torna o estado alcançável para toda a família, não só para esta cena. A fila
original dizia:

- **`pulse.counter` ganha entrada de RESET.** O artista liga o complemento do campo ali, e sair
  do campo limpa o contador. Compõe, é explícito, e custa uma porta nova (default desconectado =
  `Empty` = o mundo de hoje). ⚠️ A cerca que a deferia tem **premissa falsa** (o `Graph::validate`
  itera ARESTAS e não recusa input faltante — dois nós que shipam já dependem disso).
- **`pulse.adsr`.** Um envelope **volta ao repouso sozinho**: uma linha que sai do campo decai e
  para, sem o artista fiar nada. É a cura auto-curável, e provavelmente a certa *para esta cena* —
  o `pulse.counter(2, Wrap)` foi escolhido por legibilidade (o toggle segura meio período), e o
  preço dessa escolha é justamente um estado que nunca volta.

⚠️ **O gate novo pina um defeito ABERTO, de propósito** — ele é o número dele. Quando a cura
landar, a metade da inversão fica vermelha: **reescreva-a para a lei nova, não a afrouxe.**

### Lições generalizáveis

1. **O nó que o report acusa é uma hipótese, não um diagnóstico.** A primeira medição tem de ser
   a que pode **inocentá-lo** — aqui, cozinhar o nó sozinho no ida-e-volta (0 de 262.144).
2. **Um gate de pureza precisa da metade irmã.** *"O ida-e-volta restaura"* é satisfeito por um
   nó que **não faz nada**; sem *"e o param MORDE todas as linhas"*, o gate passaria sobre um memo
   que ignorasse a edição.
3. **Compor um campo com um acumulador é compor duas coisas de tempos de vida diferentes.** A
   máscara é função do agora; o contador é função da história. Toda vez que um multiplicador
   gateia um evento a montante de um estado, *sair do gate* vira um evento que ninguém observa.
4. **Gate que cozinha de `t = 0` não testa AUTORIA.** Uma cena com estado tem dois
   comportamentos — o do boot e o do gesto — e o segundo custa um gate próprio.


---

## Bug #2 — O glow e a forma vivem nos DOIS LADOS do tonemap, e o LOD troca o lado

**Sintoma (Enio, 2026-08-20):** *"Glow não funciona com shape"*.

**O nó acusado está INOCENTE.** O `fx.glow` faz exactamente o que promete; ele nunca
recebe a forma.

### A causa-raiz, com os sítios

O Motion tem **duas metades de render**, e depois do cook cada elemento cai numa delas:

| metade | onde vive | quem a desenha | quando |
|---|---|---|---|
| sprites | `pump.instances` | `SpriteRenderer` → `game_rt` | **antes** do tonemap (HDR) |
| vetor vivo | `pump.vector_instances` | `motion_shape_gen::encode` → cena Vello | **depois** do tonemap (LDR) |

O passe do glow (`present.rs`, Pass 1c) chama
`renderer.render_instances_only(motion_fx.rt_view(), …, &motion.pump.instances, …)` —
**só a primeira metade**. Um `source.shape` emite uma instância com `geometry_id`
(o doc-header do nó di-lo: *"it emits one instance carrying a `geometry_id`"*), que
vai para a segunda. O bright-pass lê um RT em que a forma nunca foi desenhada.

⚠️ **E há um segundo modo de falha, mais grosso:** o bloco do glow é guardado por
`!motion.pump.instances.is_empty()`. Num grafo **só de formas** essa lista está vazia,
então o passe nem corre.

### O que torna este bug cruel: ele é INTERMITENTE POR CONTAGEM

A partição de LOD (`apply_object_lod`, `LOD_COUNT = 16_000`) **move** para `instances`
toda geometria carimbada acima do limiar que tenha tile assado — e essas passam a ser
exactamente o que o glow lê.

> **A mesma forma não brilha com 16 000 cópias e brilha com 16 001.**

Fixado pelo gate `the_lod_threshold_is_where_a_shape_starts_being_visible_to_the_glow`
(`motion_bridge_objects_tests.rs`) — ele não cura nada; impede o degrau de se mover em
silêncio.

### Hipóteses que caíram

- ⛔ *"o `threshold` do glow está alto e a forma é LDR"* — seria verdade **também**
  depois de a forma chegar ao passe, mas não é a causa: ela não chega. Testável em
  segundos baixando o threshold a `0` — o halo continua ausente.
- ⛔ *"o `stretch`/`clamp` novos quebraram"* — o neutro dos três é literal e tem gate;
  e o bug é anterior a eles (o glow nunca desenhou formas).

### A cura (ordem do Enio: *"tudo deve brilhar"*)

**O passe deixou de ler uma LISTA e passou a ler uma CAMADA.**
`render_loop::motion_glow_layer::layer_instances` deriva o que o bright-pass desenha:
os sprites **mais** toda geometria vetorial viva, convertida em quad a partir do seu
**tile assado**.

⚠️ **A rota "óbvia" está MEDIDA e recusada:** rasterizar a metade vetorial em alta
fidelidade num alvo HDR é impossível hoje — o `render_to_texture` do Vello 0.8 escreve
numa storage texture `Rgba8Unorm` (`vello_pass.rs`, textual), e o RT do glow é
`Rgba16Float`; passar por um intermediário LDR perderia o HDR, que é onde o bloom vive.

⚠️ **O que torna o tile suficiente — e não um remendo:** a primeira coisa que o passe
faz com esse RT é um bright-pass em meia resolução seguido de **seis** reduções de mip.
*Um halo nunca precisou de nitidez de tela.* O caminho crispo continua a desenhar o
quadro visível; o que muda é só de onde o bloom tira a silhueta.

⚠️ **E o HDR sobrevive**, que é o que a ponte por Vello não conseguiria: o tile é a arte
da forma (sem o tint da cópia), e o shader de sprite multiplica os dois — um `tint` de
`40` chega ao bright-pass como `40`.

| origem | tile de onde | estado |
|---|---|---|
| `source.object` Sprite | desenhado direto | ✅ já brilhava |
| `source.object` Vector / Flip | `motion_object_bake` | ✅ e **em qualquer contagem** |
| `source.shape` (paramétrico) | `motion_shape_bake` (novo) | ✅ |
| 3D | — | ⏸️ quando existir: assine `unreachable_geometries` |

⚠️ **A ÂNCORA é o que separa isto de um halo torto.** Um `source.shape` emite `size`
unitário (a dimensão vive na geometria), e o bbox dele não é necessariamente centrado na
origem local. `motion_shape_bake::tile_quad` mede a forma, escala o quad pelo tile ×
`vi.size`, e desloca o centro pelo bbox **através da base** da instância. Os três passos
têm gate; o terceiro só falha quando alguém liga um `motion.rotate`.

⚠️ **O aviso da wave anterior foi RETIRADO.** O `Deficit::BlindPass` existia enquanto o
bug existia; mantê-lo seria ensinar que uma composição válida está errada. Há um gate a
medir a AUSÊNCIA dele (com controle positivo no `Shadowed`), para ninguém o ressuscitar
de um commit antigo sem reler isto.

### A SEGUNDA causa, achada por instrumento depois de um *"não funcionou"*

A cura acima estava certa e **não funcionou**, e a diferença entre as duas frases é o
que este parágrafo regista.

O `PH2D_GLOW_DIAG=1` (posto exactamente para isto) devolveu:

```
[glow-diag] assador de formas: pedidas=1 com_tile_agora=1
[glow-diag] sprites=119 vetor_vivo=1 (tile_objeto=0 tile_forma=1 SEM_TILE=0) camada=120 glow=intensidade 1.6
```

Tudo do lado da CPU estava certo — o tile existia, a camada tinha os 120, o nó estava lá
com intensidade `1,6`. Então a falha era **dentro do tile**: ele saía **inteiramente
transparente**.

⚠️ **Um `source.shape` nu é um PRIMITIVO** — sem `fill` e sem `stroke` autorados —, e as
duas portas de desenho da `ph2d-vec-render` **discordam** sobre ele:

| porta | um primitivo nu | quem a usa |
|---|---|---|
| `draw_path` (DOCUMENTO) | encoda **0** caminhos | o `dispatch` da cena vetorial |
| `draw_shape_instance_tessellated` (INSTÂNCIA) | preenche a **silhueta** | o caminho crispo de `source.shape` |

O bake chamava a do documento. Ela está certa **para ela** — um caminho do documento
sempre tem paint —, e o ramo que salva o primitivo mora na outra, que é justamente a que
o caminho crispo desta mesma forma já usava. O `draw_path_standalone` passou a chamar a
porta de INSTÂNCIA, com o tint **branco** (o da cópia multiplica no shader).

Gate `a_bare_primitive_encodes_its_silhouette`, com o irmão de CONTROLE
`the_document_route_draws_nothing_for_the_same_bare_primitive` — sem o segundo, o
primeiro ficaria verde num mundo em que as duas portas fossem a mesma, e não diria por
que a escolha da porta importa.

### Lições generalizáveis

1. ⚠️ **Um efeito de PASSE só alcança a metade do stream que o passe re-renderiza.**
   Ao ligar um FX de tela, a primeira pergunta não é *"o efeito está certo?"* e sim
   ***"que lista o passe lê, e todo elemento está nela?"***
2. ⚠️ **Uma partição por CONTAGEM transforma um bug determinístico num intermitente.**
   O LOD existe por performance e não sabe que muda a aparência de um efeito; quem
   escreve uma partição dessas tem de perguntar o que MAIS lê a lista que ela move.
3. ⚠️ **O nó que o report acusa é quase sempre o que o artista TOCOU, não o culpado** —
   é a mesma lição do Bug #1, aqui a um nível acima: lá o acusado era o nó vizinho, aqui
   é o próprio efeito, e o culpado é a fronteira entre dois passes de render.
4. ⚠️ **Quando a rota de alta fidelidade está bloqueada, pergunte de que FIDELIDADE o
   consumidor precisa.** O bloqueio era real (o Vello não escreve em `Rgba16Float`) e
   levou a diagnosticar isto como decisão de renderer. O que destravou não foi vencer o
   bloqueio: foi medir que o consumidor — um bright-pass seguido de seis reduções de
   mip — **não precisa** do que o bloqueio negava.
5. ⚠️ **DUAS PORTAS para «desenhar um caminho» discordam no caso NU, e o desacordo é
   mudo.** Escolher a porta pelo que a coisa **É** (uma forma de instância) e não pelo
   que ela **PARECE** (um caminho) é a regra. O sintoma foi um tile transparente, e o
   renderer **pula um run sem bind group em silêncio** — nenhum erro em lado nenhum.
6. ⚠️ **Quando a cura certa «não funciona», INSTRUMENTE em vez de tentar a próxima
   hipótese.** *"O halo não aparece"* tinha cinco causas indistinguíveis a olho; o
   `PH2D_GLOW_DIAG` custou uma corrida e eliminou quatro numa linha. ⛔ *Chutar entre
   cinco consertos é como se perde uma tarde — e como se conserta o que não estava
   partido.*
7. ⚠️ **Um AVISO tem prazo de validade igual ao do bug.** O `BlindPass` foi certo por
   uma wave e virou mentira na seguinte. ⛔ Curar um bug sem apagar o aviso dele deixa
   uma armadilha que ensina o artista a não usar o que passou a funcionar — e o gate
   que fixa a ausência é o que impede a ressurreição por `git revert` distraído.


---

## Bug #3 — O diagnoser SABIA, e ninguém perguntava

**Sintoma (Enio, 2026-08-20):** *"6. EMPUXO com a coluna `density`. Todas as peças
paradas"* — a banda 6 da cena `=71`.

### A causa imediata: um fio a menos na CENA

O `value.instance_field` que alimentava a densidade estava **solto**. O doc-comment
dele já dizia o que isso faz — *"Cardinality follows the geometry; unconnected → one
degenerate value"* —, então ele devolvia **um** valor em vez de oito, o `motion.drive`
transmitia-o a toda a fileira, e ele era **zero**. Densidade zero é empuxo nenhum: a
fileira ficava parada.

⚠️ **E o `map_range` que entrou junto não é enfeite:** um índice normalizado começa em
`0`, então mesmo com o fio ligado a PRIMEIRA peça teria densidade zero e ficaria parada
sozinha — o mesmo defeito, um oitavo dele. A rampa agora vai de `0,35` a `2,2`.

### A causa de fundo: o instrumento existia e nenhum passo o invocava

O `ph2d_motion_diagnose` reporta `MissingSource`/`MissingInput` **exactamente** para um
nó sem nada ligado. Ele existia desde o ADR-0155 e **nenhuma cena da conferência o
consultava** — não havia sequer uma porta por onde um teste pudesse montar uma cena
(o roteador lia a env var direto). Foi preciso extrair `demo_router::build_level` para
que a pergunta fosse possível.

### O que a sonda achou quando finalmente perguntou

**Seis** cenas marcadas (`=3`, `=31`, `=38`, `=57`, `=61`, `=71`) — e cinco delas
estavam **certas**. O falso positivo era do diagnoser:

> A isenção de *«não é source-less»* exigia que a aresta atrasada viesse do **PRÓPRIO
> nó** (`seeds_own_state`, escrita para as sims que se semeiam). Mas o laço canónico de
> uma força é `integrate ⟿pre⟿ força ⟿fwd⟿ integrate.forces` — a aresta vem de OUTRO
> nó. **Toda cadeia de força correctamente montada exibia um badge ⚠** a dizer *"este
> nó não tem nada ligado"*, inclusive a que o próprio AUTO-HEAL constrói.

Curado: qualquer aresta atrasada conta como stream. Gate + controle (uma força com
aresta NENHUMA continua reportada), e a fixture usa `force.vortex` e não `force.wind` —
o vento é uma força GLOBAL, não lê `P`, e o gate passaria por vácuo.

### E o nó que ninguém conseguia diagnosticar

Com o diagnoser curado, a mutação que devolvia o bug do Enio **ainda sobrevivia**: o
`value.instance_field` não declarava precisar de nada. O `P` chega a ele pela CONTAGEM e
pelo `id`, não por uma binding que o leia — nem o kernel nem uma `Coupling` o
denunciavam. Ele passou a declarar `Requires("P")`, e agora **dois** gates pegam o
defeito.

### O portão que sobrou

`no_conference_scene_ships_a_setup_hole` (no roteador, que é de quem ele é): monta as
**71** cenas e exige zero déficits, com controle de contagem para não passar por vácuo.
⛔ Se uma cena futura encenar um defeito de propósito, ela não entra numa allowlist
muda — ou o defeito não é de SETUP, ou o gate ganha o nível **nomeado** com o motivo.

### Lições generalizáveis

1. ⚠️ **Um instrumento que nenhum passo INVOCA não protege nada.** O diagnoser sabia
   apontar este buraco desde o ADR-0155; o que faltava era uma porta por onde perguntar.
   *A cura não foi escrever análise nova — foi tornar a existente alcançável.*
2. ⚠️ **MEÇA antes de a sonda virar barra.** A primeira corrida marcou seis cenas, e
   cinco eram falso positivo. Um portão posto no lugar da sonda teria sido «consertar»
   cinco cenas correctas para calar um bug do diagnoser.
3. ⚠️ **Uma isenção escrita para UM caso costuma ser estreita demais para a FAMÍLIA.**
   `seeds_own_state` estava certa para as sims que se semeiam e cega para o laço de
   força — a pergunta real era *«tem stream?»*, e uma aresta atrasada é um stream venha
   ela de onde vier.
4. ⚠️ **Um nó cuja necessidade não é derivável TEM de a declarar.** O `P` do
   `value.instance_field` chega pela contagem, não por uma binding — invisível ao
   diagnoser, e por isso o `Coupling::Requires` existe. *Se o doc-comment avisa e o
   registry não, o aviso não protege ninguém.*
5. ⚠️ **Uma fixture de força tem de LER `P`.** `force.wind` é global e nunca é
   reportado: um gate sobre ele fica verde nos dois mundos.

---

## Bug #4 — O `Multiply` não desobedecia à alfa: ele a INVERTIA, e o gate media o único ponto em que os modos concordam

**Sintoma (Enio, 2026-08-23):** *"shadow multiply parece não obedecer o alpha da cor"*.

**O nó acusado está INOCENTE.** O `fx.drop_shadow` calcula a alfa do fantasma como
`color[3] × base[i][3] × falloff` e escreve-a no `tint` — correcto, e com gate. O que
estava errado era o **par de fatores de mistura** com que o renderer compõe o `Multiply`,
duas crates abaixo, num caminho partilhado por **toda a sprite do app**.

### A medição, antes de tocar em nada

Sonda `measure_alpha_response_of_every_mode`
([`blend_mode_regression.rs`](../../crates/ph2d-render/tests/blend_mode_regression.rs)) —
fundo opaco cinza (byte 55 no alvo linear), frente cinza 128, byte do centro:

| modo | α=0,00 | α=0,25 | α=0,50 | α=0,75 | α=1,00 |
|---|---|---|---|---|---|
| `Mix` | 55 | 55 | 56 | 55 | 55 |
| `Add` | **55** | 69 | 83 | 96 | 110 |
| `Subtract` | **55** | 41 | 27 | 14 | 0 |
| **`Multiply` (antes)** | **0** | 3 | 6 | 9 | 12 |
| `Screen` | **55** | 66 | 77 | 87 | 98 |
| **`Multiply` (depois)** | **55** | 44 | 34 | 23 | 12 |

⚠️ Não era *"não obedece"*: era **invertido**. Alfa `0` pintava **preto** onde estava o
fundo, e **subir** a alfa **clareava**. Não havia valor nenhum em que a sombra sumisse — o
cursor tinha deixado de dizer *"quanta sombra"* e dizia *"quão escuro"*, ao contrário.

### A causa-raiz

O `sprite.wgsl` emite `vec4(rgb·α, α)`: uma fonte **pré-multiplicada**. Ela codifica
*"não contribui"* como **zero** — e é por isso que **todo modo cujo elemento neutro é `0`
ganha a resposta à alfa de graça**: `Add` (`dst + 0`), `Subtract` (`dst − 0`), `Screen`
(`0·(1−dst) + dst`), o `over`. Todos eles aparecem correctos na tabela sem nunca terem
sido pensados para isso.

O elemento neutro do `Multiply` é **`1`**. Com `dst_factor: Zero` o resultado era
`src_premult · dst = dst·rgb·α`, que a pré-multiplicação leva para **preto** em vez de
para *nada*.

**Cura** — `src: Dst`, `dst: OneMinusSrcAlpha` ⇒
`src_premult·dst + dst·(1−α)` = `dst·(α·src + 1 − α)`: a lei de opacidade de camada do
Photoshop. ⚠️ **As duas colunas coincidem em `α = 1`** (12 = 12), que é o que garante que
nada opaco no app mudou.

### Por que ele viveu anos dentro de um gate VERDE

O `blend_modes_composite_as_advertised` é um gate **honesto e rico**: tabela por modo,
barra por modo, e a ordenação `Multiply < Mix < Screen < Add` como prova de que cada
pipeline compõe como anuncia. E media **tudo a `alpha = 1`** — o **único ponto em que os
seis modos concordam sobre o que a alfa quer dizer**.

⚠️ *Uma suíte que varia só o MODO não tem eixo nenhum para expor uma lei sobre a ALFA.*
A dimensão que faltava não era um caso a mais dentro do eixo existente: era um **segundo
eixo**.

### A fronteira que fica (registada, não escondida)

Um par de fatores fixos **não exprime** o `Cs' = (1−αb)·Cs + αb·B(Cb,Cs)` da W3C, que
precisa da alfa do **destino** como termo. A família inteira de FX aqui (`keep_dst_alpha`)
pressupõe um fundo **opaco**. Onde o fundo é translúcido de propósito — a pilha de camadas
do Painter — a fórmula completa já existe e **está correcta**
([`layer_composite.wgsl`](../../crates/ph2d-render/src/shaders/layer_composite.wgsl):
em `αs = 0` ela devolve `cb`). Como a alfa do destino nem se move aqui, sobre um pixel
`αb = 0` o resultado é invisível de qualquer maneira; a divergência vive só na faixa
parcial. *Quem tentar "consertar" isso com outro par de fatores não vai conseguir — o
caminho é o passe programável.*

### Lições generalizáveis

1. ⚠️ **Ao escrever um modo de mistura em função fixa, pergunte qual é o ELEMENTO NEUTRO
   dele.** Se for `1` (multiply, e os primos color-burn/darken), a pré-multiplicação
   trabalha contra si e o par de fatores tem de trazer o `OneMinusSrcAlpha` de volta.
   Se for `0`, a alfa sai de graça — e é essa gratuidade que faz o modo excepcional passar
   despercebido no meio dos irmãos correctos.
2. **A régua de um modo é «α = 0 devolve o destino, exactamente»**, e a barra é o fundo
   **medido no mesmo passe**, nunca um número escrito à mão (o alvo é linear e o byte
   depende do sRGB do atlas). Uma linha por modo, e um controle positivo de que a alfa
   cheia move o pixel — senão o zero passa por vacuidade.
3. ⚠️ **Um gate rico numa dimensão pode ser cego noutra, e a riqueza disfarça.** Antes de
   confiar numa suíte, liste os parâmetros que o produto oferece e pergunte quais são
   **eixos** dela. O que não é eixo não está medido.
4. **O report do Enio é sobre APARÊNCIA e chega antes de qualquer instrumento.** *"Parece
   não obedecer"* estava certo no sítio e errado na direcção — o defeito era pior do que
   o relato. Meça a resposta INTEIRA do curso antes de aceitar a descrição do sintoma.
5. ⚠️ **O sintoma apareceu num nó e a causa estava numa crate que o nó não conhece.** O
   `fx.drop_shadow` só escreve um número numa coluna; quem escolhe o pipeline é o lowering,
   e quem compõe é o `wgpu`. *Três camadas entre a queixa e o defeito.*

---

## Bug #5 — O editor de curva era oferecido numa ONDA QUE NÃO O LÊ — e o censo que o teria apanhado não podia vê-lo

**Estado:** ✅ **CURADO** em 2026-08-24 — mecanismo medido, três nós curados, e o **censo da
espécie** construído. Aguarda smoke (cena `PH2D_GPU_COOK_DEMO=94`, fileira de cima).

### Sintoma

Report do Enio, com foto do painel do `motion.oscillator`: uma curva de três pontos desenhada
no editor **Custom Wave**, e a legenda *«Wave curve dos osciladores não está funcionando»*.

### ⚠️ O motor estava CERTO — medido antes de tocar em nada

Uma sonda que coze `motion.grid → motion.oscillator` com o texto da curva posto à mão:

| `wave` | a saída em 9 instantes |
|---|---|
| `0` (Sine) | `0,000 0,708 1,000 0,708 0,000 −0,708 −1,000 −0,708 0,000` |
| `5` (**Custom**) | `1,000 0,750 0,500 0,250 0,000 0,250 0,500 0,750 1,000` |

O V autorado sai **ao valor**. A curva chega, é parseada, e conduz. ⇒ *o defeito não estava
onde o report o punha*, e a primeira hora foi gasta a provar isso em vez de a mexer no motor.

### O mecanismo

O `wave` nasce em **`Sine`**, e o editor `Custom Wave` era desenhado **em toda onda** — sem
`ParamGate`. A `waveform` só lê a curva no braço `Custom`. Logo: o artista abre o nó, vê um
editor, desenha, e não acontece nada.

⚠️ **É a doença que a própria tabela deste nó já curava quatro vezes** — `frequency`/`bpm` e
`amplitude`/`min`/`max` são gateados exactamente por isto. A curva ficou de fora porque ela
**não é um `ParamSpec`**: uma curva vive no canal de TEXTO, por decisão registada (*«uma curva
não é um número»*).

### ⭐⭐ E o report expôs uma ESPÉCIE, não um caso

A caça aos knobs mortos ([doc 90](90_caca_aos_knobs_mortos.md)) varre **660 params
declarados** — e ela lê o `MANIFEST`. **Nenhum param de forma foi alguma vez perguntado se
alguém o lê**: são dezassete, em dezasseis nós. É o **nono ponto cego** daquela sonda, e o
único que é uma família inteira em vez de um caso.

O censo novo — `every_shape_param_is_either_always_read_or_gated_to_the_mode_that_reads_it`
(`shells/desktop/src/render_loop/motion_bridge_shape_reach_tests.rs`) — pergunta: *ou é lido
em todo modo, ou é gateado ao modo que o lê*. Ele acusou **três**:

| nó | param | lido só quando |
|---|---|---|
| `motion.oscillator` | `curve` | `wave = Custom` |
| `motion.stagger` | `curve` | `ease_curve = Custom` |
| `field.remap` | `curve` | `contour = Curve` |

⇒ **O Enio reportou um e o instrumento achou três.**

### ⛔ E uma CERCA DECLARADA dissolveu — pelo mesmo tipo de report que a criou

O `field.remap` tinha escrito, verbatim: *«O `curve` NÃO é gateado, de propósito … gateá-lo por
`contour == 4` esconderia-o no modo default e reprovaria o gate
`selected_field_remap_yields_an_interactive_curve_row`. Se um dia a decisão for escondê-lo, é
aquele gate que se reconcilia primeiro»*.

⚠️ **A cerca nasceu de um report do Enio de 21/08** (*«Curve offset e outros parâmetros não têm
efeito»*) e curou os três knobs numéricos vizinhos, deixando o editor de fora para ele não
sumir no modo default. **O preço era um editor vivo e inerte** — e três dias depois o irmão
produziu a mesma frase. *A cerca trocava um report por outro.* O gate que ela nomeava é uma
afirmação sobre o PAINEL, e reconciliou-se escolhendo o contorno na fixture.

### ⚠️ E uma mutação mostrou que «gateado» não é «gateado ao modo certo»

Trocar o `values` da curva do oscilador de `[Custom]` para `[0,1,2,3,4,5]` **passava no censo**
— ele perguntava *«existe gate?»*. O segundo censo (`no_gate_hides_nothing`) deriva a escada do
param observado e exige que a lista do gate seja um subconjunto **estrito** dela.

### ⚠️ E o controle de um nó tinha o mesmo ponto cego, um nível abaixo

O `every_contour_number_is_gated_to_the_contour_that_reads_it` do `field.remap` confere que
todo nome gateado existe — perguntando **só ao `MANIFEST`**. No dia em que a curva entrou na
tabela ele acusou *«`curve` não é param deste nó»*, sobre produto correcto. *Um censo que só
conhece um dos dois canais não vê metade dos controles.*


---

## Bug #6 — O mar não parecia mar: DOIS defeitos com a mesma assinatura, e as réguas da cena partilhavam-na com eles

**Estado:** ✅ **CURADO** em 2026-08-24 — as duas causas medidas, quatro gates novos, quatro
mutações. Aguarda smoke (cena `PH2D_GPU_COOK_DEMO=95`, fileira de baixo).

### Sintoma

Report do Enio sobre a cena `=95`: *«os exemplos do mar não parecem mar mas sim partículas ao
vento»*. Nenhum gate da cena estava vermelho.

### ⚠️ Duas causas independentes, e leem-se IGUAL no ecrã

A assinatura partilhada é: **as peças atravessam a banda sem nunca subirem nem descerem.**

| | causa | excursão vertical | deriva horizontal em 5 s |
|---|---|---|---|
| 1.ª | **não havia gravidade no grafo** | — | a média de `y` subia `0,58` por 25 tiques, **sem abrandar** (`−5,73 → −2,72`), e a banda ia de `3,16` a `14,01` de largura |
| 2.ª | **armadilha de cava** | `0,0056` | `4,92` — exactamente `0,98` da velocidade da onda |
| curado | — | `0,377` (`0,81` da altura da vaga) | `0,26`, e **líquida `0,067`** |

**A primeira** é a que a cena `=4` já documentava e ninguém releu: o `force.buoyancy` **não
tem param de gravidade de propósito** (nem existe nó `force.gravity`) porque uma força
direcional constante já existe — o `force.wind` a 270° **É** a gravidade. Sem ela o empuxo
lança tudo, e a nota da `=4` di-lo por palavras: *«buoyancy alone would launch everything
upward»*. ⇒ *Uma cena que demonstra um nó tem de reproduzir as condições em que a resposta
dele quer dizer alguma coisa.*

**A segunda** é mais interessante, e **o nó não está errado**: o doc-comment dele declara-a
(*«a boia deriva para a cava e cavalga a vaga, em vez de subir e descer no mesmo sítio»*). O
que estava errado era a cena escolher números **dentro do regime onde esse comportamento come
tudo o resto**. A boia escorrega para a cava até o empurrão em declive igualar o arrasto, logo
ela **ENCAIXA** se existir declive que o faça à velocidade da onda:

```
densidade · declive_máximo · inv_len  ≥  arrasto · velocidade
```

⚠️ **O espectro multiplica o declive pelo número de camadas** — cada oitava tem metade da
amplitude e metade do comprimento, logo o **mesmo** declive —, então a fileira de 4 ondas é a
exigente e é ela que manda no número. A varredura confirma que a transição cai **onde a lei a
põe**:

| densidade | ondas | limiar previsto | arrasto 6 | arrasto 11 |
|---|---|---|---|---|
| 12 | 1 | `6,38` | preso (deriva `4,92`) | livre |
| 12 | 4 | `11,15` | preso (deriva `4,92`) | livre (deriva `1,72`) |
| 6 | 4 | `5,57` | livre | livre (deriva **`0,025`**) |

⚠️ **A margem CUSTA vida:** mais arrasto afasta da armadilha e ao mesmo tempo abafa a boia (a
`20` a excursão vertical cai para `0,29` da altura da vaga). A cena shipa `2×` o limiar.

### ⛔ Porque nenhuma régua o via — e uma delas era MINHA e estava a favor do defeito

1. **Dispersão e distância entre as duas bandas são grandezas que um mar e uma nuvem lançada
   PARTILHAM.** O gate que existia media `|y| < 40` e passava com as peças a sair do ecrã.
2. **Uma régua só de `y` não vê a segunda causa**, que é horizontal.
3. ⚠️ **A excursão horizontal sozinha não distingue ORBITAR de PARTIR** — uma boia que vai e
   vem meia onda mede o mesmo que uma que anda meia onda e nunca volta. É preciso a deriva
   **líquida** da banda.
4. ⛔⛔ **E o gate «bonito» que eu escrevi no meio da cura passava PELA RAZÃO ERRADA.** Ele
   afirmava que a mediana da submersão bate o equilíbrio estático `(gravidade/densidade) ·
   calado`, e batia — a **`0,8%`**. Só que um corpo **encaixado** na cava não se mexe, logo
   assenta no estático: *o gate só era verde enquanto o defeito estava lá*. Assim que as boias
   passaram a cavalgar a vaga ele passou a medir `0,29` contra `0,167` e teve de cair. **Um
   gate que só passa quando a cena está morta é um gate a favor da cena morta.**
5. ⚠️ **E o CONTROLO do gate da deriva teve de nascer por mutação sobreviva:** com a gravidade
   a zero e as boias a nascerem acima da água elas ficam secas ⇒ **força nenhuma** ⇒ paradas
   para sempre, e um campo congelado tem deriva zero. *Um balde que ninguém enche lê-se como
   perfeito.*

### Mais duas coisas que a medição corrigiu na cena (não eram o report, apareceram a caminho)

- ⚠️ **A vaga estava `2,7×` para lá do ponto em que uma onda de água QUEBRA.** O limite físico
  é `H/λ = 1/7`, ou seja `a/λ = 1/14 ≈ 0,071`; a cena estava a `0,19`. Uma forma que a água
  não faz é uma forma que o olho não lê como água. O número que fica é o que a `=4` já shipa
  (`2,0 / 20,0` ⇒ **`0,1`**) — *uma segunda cena de mar não é onde essa decisão se re-abre*.
- ⚠️ **A amostragem estava abaixo de Nyquist.** A onda mais fina do espectro é `λ/8`, e as 48
  boias davam **1,96 por período** — abaixo de dois pontos um seno não é sub-amostrado, é
  **irreconhecível**. A fileira prometia «cristas de tamanhos diferentes» e teria mostrado
  pontinhos a tremer. `128` boias dão `5,3`.

### O que ficou construído

- Duas funções públicas no `force.buoyancy` — `surface_at` e `finest_wavelength`. ⚠️ Elas
  existem porque **uma lei que só o autor consegue avaliar não é verificável por quem a
  mostra**: sem a superfície, o gate de uma cena só mede dispersões e derivas, que é
  precisamente o conjunto de grandezas que o defeito partilhava com o produto correcto.
- Quatro gates na cena: `the_floats_ride_the_wave_instead_of_being_carried_by_it` (com o
  controlo de que elas estão a boiar), `the_drag_clears_the_trapping_threshold` (a LEI, não o
  sintoma — dispara em quem baixar o arrasto, subir a esbelteza, subir a densidade ou
  acrescentar camadas, e três dessas não parecem ter nada a ver com ela),
  `the_floats_resolve_the_finest_wave` e `the_floats_are_in_the_water`.
- ⚠️ **O `force.buoyancy` não foi tocado no comportamento** — a cena `=4` é byte-idêntica.
