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
5. ⚠️ **Um AVISO tem prazo de validade igual ao do bug.** O `BlindPass` foi certo por
   uma wave e virou mentira na seguinte. ⛔ Curar um bug sem apagar o aviso dele deixa
   uma armadilha que ensina o artista a não usar o que passou a funcionar — e o gate
   que fixa a ausência é o que impede a ressurreição por `git revert` distraído.
