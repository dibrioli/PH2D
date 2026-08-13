# ESTUDO — A UI VIVA: o que falta para encantar

> **Pedido do Enio (2026-08-12):** *"temos o básico para construir UI. Agora vamos buscar algo mais
> avançado, mais moderno, algo que pode transformar a experiência do usuário em algo realmente
> encantador. Exemplo: no Pixelmator, um tool cujo gizmo era dinâmico, com uma CORDA animada por
> física ligando o gizmo ao botão da tool. Pesquisa profunda sobre aspectos de UI e UX que ainda
> não temos e que levam a UI a outro nível de agradabilidade, dinâmica, eficiência e decoração."*

---

## §0 — A resposta em cinco linhas

1. **O app paga um laço de quadros CONTÍNUO (`ControlFlow::Poll`) e gasta-o inteiro a desenhar uma
   função ESCADA.** Toda mudança de estado do chrome é instantânea: não existe um `t` em lugar
   nenhum da camada de widgets. *Animação já está paga; falta quem a consuma.*
2. **A mola existe, com velocidade contínua sob interrupção — e serve só a UI que o ARTISTA
   autora.** O chrome do próprio app não tem acesso a ela. É a peça certa, na sala errada.
3. **Os ingredientes de encanto estão todos no prédio, noutros quartos:** partículas na GPU, fluido,
   corpos rígidos, 42 efeitos de áudio, um resolvedor de molas, um paleta de comandos. **Zero** deles
   alcança a interface do app.
4. **A corda do Pixelmator não é uma corda: é um TETHER** — geometria simulada cujas pontas são um
   *controlo* e o seu *efeito*. Custa ~80 linhas próprias, e ⛔ **não** deve tocar o `rapier`.
5. O maior ganho de **eficiência** não é animado: é o **scrub numérico** (arrastar sobre o número
   para o mudar), universal em todo DCC, **que não temos**.

---

## §1 — O que foi MEDIDO, e como reproduzir

Nada abaixo é lido de doc. Cada linha tem o comando ao lado.

| fato | medida | comando |
|---|---|---|
| o laço redesenha **sempre** | `event_loop.set_control_flow(ControlFlow::Poll)` | `grep -n "ControlFlow::" shells/desktop/src/main.rs` |
| **`wall_dt` já existe** e é calculado todo quadro | `render_loop/mod.rs:1425` | `grep -n "wall_dt" shells/desktop/src/render_loop/mod.rs` |
| o chrome **não** recebe tempo nenhum | zero ocorrências de `dt`/`Instant` na `paint` do editor-core | `grep -rn "Instant" crates/ph2d-editor-core/src/hero/` |
| os estados de widget são **discretos** | `Normal · Hovered · Pressed · Focused · Disabled` | `crates/ph2d-editor-core/src/widget/button.rs:24` |
| **nenhum widget interpola** entre eles | zero `hover_t` / `transition` / `lerp` em `widget/*.rs` | `grep -rn "lerp\|transition" crates/ph2d-editor-core/src/widget/*.rs` |
| a mola é **keyed por `VecPathId`** | `sets.rs:150 pub fn spring(&self, host: VecPathId)` | `crates/ph2d-ui-state/src/spring.rs` |
| **49** arquivos de widget, **~1.600** ids de chrome | — | `ls crates/ph2d-editor-core/src/widget/ \| wc -l` |
| a rolagem **não tem inércia** | zero `velocity`/`momentum` no scrollbar | `grep -rn "velocity" crates/ph2d-editor-core/src/widget/scrollbar.rs` |
| **sem scrub numérico** | zero `Drag` no `number_input.rs` | `grep -n "drag" crates/ph2d-editor-core/src/widget/number_input.rs` |
| **sem menu radial**, **sem reduced-motion**, **sem som de UI** | zero ocorrências | `grep -rn "radial\|reduced_motion" crates/ shells/` |

### ⚠️ E a medição achou um defeito VIVO, pequeno e exato

`ToastQueue` é **o único relógio do chrome** — e ele conta **QUADROS**:

```rust
pub ttl_frames: u32,   // "default ~3 s @ 60 Hz"
pub age: u32,          // "per frame; toast removes itself when age >= ttl_frames"
```

A 30 fps aquele toast dura **6 segundos**; a 120, **1,5**. E o mesmo repositório **já aprendeu esta
lição**, um arquivo adiante, com o motivo escrito no comentário (`render_loop/mod.rs:1496-1499`):

> *"…which made the sprites race + jitter. `wall_dt` makes the motion frame-rate-independent."*

⇒ **o conhecimento existe no prédio e não atravessou a porta.** É a mesma classe de doença que este
repo curou quatro vezes no relevo do Painter — *a lei é função do relógio de PAREDE, nunca de quão
depressa a máquina conseguiu desenhar*. Custa uma linha, e é o degrau zero de tudo o que segue.

---

## §2 — O achado central

> **O app corre um laço contínuo, mede `wall_dt` todo quadro, tem um resolvedor de molas com
> continuidade de velocidade sob interrupção — e a sua própria interface é uma função escada.**

Isto não é uma falha de gosto, é uma **peça em falta**: não há onde guardar estado contínuo por
widget, e não há `t` a chegar ao pintor. Sem esses dois, *nenhuma* das ideias das secções seguintes
é exprimível; com eles, quase todas custam poucas dezenas de linhas cada.

⚠️ **E há uma ironia que vale como argumento:** a `line/Vector` acabou de construir, para a UI que o
artista autora, **poses nomeadas, transições com catálogo de curvas, Smart Animate e uma MOLA** — e
o painel que mostra esses controlos não usa nenhum deles. *Estamos a entregar ao artista uma
interface viva a partir de uma interface morta.*

---

## §3 — Os ingredientes que já estão no prédio (noutras salas)

| ingrediente | onde vive | alcança o chrome? |
|---|---|---|
| **mola** com velocidade contínua | `ph2d-ui-state::spring` | ❌ — só UI autorada |
| catálogo de **curvas** de easing | `ph2d-anim::Easing` | ❌ |
| **partículas na GPU** (119 nós, milhões de instâncias) | `ph2d-gpu-cook` + Motion | ❌ |
| **fluido**, **corpos rígidos**, **cordas com roldanas** | wet-paint · `ph2d-physics-ecs` | ❌ (e **deve** continuar) |
| **42 efeitos de áudio** + mixer + vozes por streaming | `ph2d-audio*` | ❌ |
| **paleta de comandos** de tela cheia | `editor-core::widget::command_palette` | ⚠️ só o editor de nós |
| **readout que segue a mão** (o rótulo de distância) | `LengthDisplay`, W6.6 | ✅ **regra** (`crate::readout`) — ver a C3 na §6 |
| **cursor que veste a ferramenta viva** (o anel do pincel) | `the_brush_ring_wears_the_live_dab_rotor` | ⚠️ **uma** instância |
| **snap que mostra o PORQUÊ** (4 espécies, 4 marcas) | W6.1 | ✅ — e é estado-da-arte |

⇒ As duas linhas com ⚠️ são o padrão a **generalizar**, não features a inventar. Este app já sabe
fazer as duas coisas mais difíceis desta lista; fá-las **uma vez cada**.

---

## §4 — O estado da arte, triado por MECANISMO

A literatura e os produtos misturam tudo sob *"delight"*. Separado por mecanismo, há **três eixos com
custos e retornos diferentes** — e um quarto que é imposto.

### Eixo 1 — CONTINUIDADE (movimento como tecido conjuntivo)

*Nada teletransporta.* Não é decoração: é o que permite ao olho manter a identidade dos objectos, de
modo que o utilizador não tenha de **re-encontrar** as coisas depois de cada mudança.

| ideia | o mecanismo | referência viva |
|---|---|---|
| **Mola em vez de duração** | a interrupção herda a VELOCIDADE, não recomeça de zero | SwiftUI, Framer Motion; **o nosso `spring.rs` já argumenta isto por medição** |
| **INTERRUPTIBILIDADE** | toda animação é agarrável a meio; a posição/velocidade actuais são a condição inicial nova | é *o* diferenciador — uma animação que não se agarra é **pior** que nenhuma |
| **Elemento partilhado (*magic move*)** | o que existe dos dois lados **MOVE-SE**; não desaparece e reaparece | Figma Smart Animate (que já temos… para o artista) |
| **Cascata (*stagger*)** | `delay = índice × ε` faz N itens lerem-se como UM gesto | custa uma multiplicação |
| **Coerência direccional** | o painel que veio da direita volta para a direita | o movimento codifica o modelo espacial |
| **Antecipação / *overshoot*** | o recuo antes do salto; squash & stretch aplicado a UI | ⚠️ dose: charmoso uma vez, ruído às 400 |

### Eixo 2 — CAUSALIDADE (a UI mostra *porquê*)

É aqui que vive a corda do Enio, e é o eixo em que este app já é forte.

| ideia | o mecanismo | temos? |
|---|---|---|
| **TETHER físico** | geometria simulada entre um controlo e o seu efeito | ❌ — **§5** |
| **Razão do encaixe visível** | 4 espécies de ímã, 4 marcas distintas | ✅ estado-da-arte |
| **Readout que segue a mão** | o número aparece onde os olhos já estão | ✅ **regra** (C3) |
| **Realce de proveniência** | passar sobre um valor **acende** o que ele controla — e o inverso | ❌ — é o que torna navegável um inspector de 400 widgets |
| **Fio de dependência** | no grafo, o *hover* num socket **apaga** o que não é a jusante | ❌ |
| **Pré-visualização viva** | `LiveGeometry`, onion, ghost | ✅ farto |

### Eixo 3 — AGÊNCIA (a mão fica mais rápida)

Mede-se em **segundos por operação**, não em encanto.

| ideia | o mecanismo | temos? |
|---|---|---|
| ⭐ **SCRUB numérico** | arrastar SOBRE o número muda-o; modificador dá precisão | ❌ — **universal** em Blender/Figma/AE/Houdini |
| **Menu RADIAL** | direcção em vez de posição ⇒ tempo **constante**, memória muscular | ❌ — e é o idioma da caneta |
| **Quasimodo mola-carregado** (Raskin) | segurar entra no modo, largar sai; a mão **lembra-se porque está a segurar** | ⚠️ parcial |
| **Inércia** no pan e nas listas | o gesto tem massa; o conteúdo continua | ❌ |
| **Paleta de comandos GLOBAL** | um atalho, todo o verbo do app | ⚠️ só no grafo |
| **Repetir última acção** | o multiplicador silencioso de todo DCC | ❌ (⚠️ `KeyD` já é Subtract) |
| **A CANETA: pressão, inclinação, rotação de barril** | fidelidade de entrada | ❌ — **nem pressão temos** |

### Eixo 4 — RESPEITO (o imposto, não a feature)

- **Reduced motion** — um interruptor que corta *tudo* do eixo 1. ❌ não temos, e **tem de nascer com
  a primeira animação**, não depois.
- **Nenhuma animação bloqueia entrada.** Um clique durante uma transição é sempre aceite.
- **O tempo é de PAREDE.** (§1: o toast ainda conta quadros.)

---

## §5 — O arquétipo do Enio: a corda, dissecada

**O que é.** Não é um enfeite com forma de corda: é um **TETHER** — geometria *simulada* cujas duas
pontas são um **controlo** e o seu **efeito**. Faz três coisas ao mesmo tempo, e é por isso que se
lembra dela anos depois:

1. **Torna visível uma relação invisível** — *este gizmo pertence àquele botão*. Uma linha reta faria
   isto e seria esquecida.
2. **Tem INÉRCIA**, portanto devolve ao olho a *dinâmica* do gesto, não só o resultado. A UI
   reconhece **como** você moveu a mão, não apenas para onde.
3. **É inequivocamente viva** — e uma coisa viva lê-se como um **objecto físico**, não como um
   desenho. É o que transforma "software" em "instrumento".

**O que custa AQUI.** ~80 linhas: 8-16 pontos de massa, uma restrição de distância (Verlet ou PBD),
gravidade, 2-3 iterações por quadro, em **espaço de chrome** (píxeis de tela). Não precisa de
colisão, nem de determinismo, nem de save.

⛔ **E não deve tocar o `rapier`, nem o `verlet_rope` dos nós, nem o motor de cordas/roldanas da
física** — decisão preventiva, com mecanismo: aqueles são simuladores de **MUNDO**, com contrato de
determinismo (`physics_ecs_c9`, hash comparado em 3 SOs) e com schema. Enfeite de UI a entrar ali
significa que uma decoração passa a poder **mover um hash de determinismo**. *Um tether de chrome é
descartável por construção; um corpo do mundo nunca é.*

**A família que ele abre** (o mesmo primitivo, quatro usos):

| uso | o que liga |
|---|---|
| a corda do gizmo | o punho na tela ↔ o botão que o armou |
| o fio elástico | o socket de saída ↔ o cursor, enquanto se arrasta uma ligação |
| a barriga do divisor | um divisor de painel longo **cede** ao ser arrastado, e assenta |
| o pêndulo do popover | a lista que cai de um chip **balança** ao aparecer e ao seguir a janela |

---

## §6 — A lista, com preço e pré-requisito

⚠️ **F0 é pré-requisito de quase tudo.** Sem ele, nada do eixo 1 é exprimível.

| # | item | eixo | pré-req | tam. |
|---|---|---|---|---|
| **F0** | **`wall_dt` chega ao chrome + estado contínuo por widget** (`UiMotion`: um mapa `id → (valor, velocidade)`) — e o toast passa a contar **segundos** | 1 | — | **M** |
| F1 | a **mola** sai da UI autorada e passa a servir o chrome (mesma crate, `pub`) | 1 | F0 | **P** |
| ⭐ **F2** | **hover/press/focus interpolam** — ⚠️ **MEDIDA em 2026-08-13, e ela NÃO está feita: o `.hover_t()` é passado em DOIS sítios do app inteiro contra 161 que pintam um Button/Toggle/Checkbox/IconButton.** O `hover_targets` publica alvos para todos eles e o `tick` integra as molas; a **pintura deita o resultado fora** (o default `hover_t = 1.0` cai no estado duro). Ver a §6.2 | 1 | F0+F1 | ~~P~~ **G** |
| F3 | **interruptibilidade** como lei: todo alvo novo herda a velocidade actual | 1 | F1 | **P** |
| F4 | **secções e painéis** abrem/fecham com movimento e **direcção coerente** | 1 | F1 | **M** |
| ~~F5~~ ✅ | **cascata** — **FEITA na PALETA** (`ε = 0,020 s`, MEDIDO; ver a §6.3). ⚠️ E ela é o **primeiro consumidor de `Role::Travel` do produto** — até aqui o eixo que o *reduced motion* existe para matar não era usado por ninguém. Hierarquia e rows do inspector ficam para quando a F2 abrir a porta | 1 | F0 | **P** |
| ⭐ **E1** | **SCRUB numérico** em todo campo de número (o maior ganho de eficiência do estudo) | 3 | — | **M** |
| ~~E2~~ ✅ | **rolagem SUAVE** nas listas — **FEITA**, e a forma é o que a fez caber: `panel_scroll` passa a devolver o **VIVO** e ganha o irmão `panel_scroll_target`, então os **~130 leitores** e os **36 escritores** herdaram sem uma linha. ⚠️ A roda acumula no ALVO — no vivo, cinco voltas de 100 px somam **230,56** em vez de 500. O **pan de canvas** fica de fora (outro gesto, outro dono) | 3 | F0 | **P** |
| ~~E3~~ ✅ | **paleta de comandos GLOBAL** (o widget já existe) — **FEITA** (`Ctrl+K`, **62 comandos**: 10 do rail + 19 painéis + **33 rows de menu**). ⚠️ Ela é uma **projecção** das listas que o app já mantém, nunca uma tabela. ⚠️ **E a 1ª conclusão desta linha era LARGA DEMAIS** — ela mediu que o **PILL** não é servível (abre um menu ancorado a um rectângulo, e uma paleta não tem rectângulo) e escreveu *"a barra de topo fica de fora"*; a **ROW de dentro dele** é tipo 1, e entrou na wave seguinte (ver a §6.1 abaixo) | 3 | — | **M** |
| E4 | **menu radial** sob a caneta / botão do meio | 3 | — | **M** |
| C1 | **TETHER** (§5) + as três irmãs da família | 2 | F0 | **M** |
| C2 | **realce de proveniência** nos dois sentidos (valor ↔ objecto) | 2 | — | **M** |
| ~~C3~~ ✅ | o **readout que segue a mão** vira REGRA — **FEITA**. ⚠️ E a medição corrigiu a linha: eram **três** superfícies (o rótulo do smart guide · a carga de um joint · as dimensões do Line), cada uma com o próprio corpo e caixa, e **nenhuma segue a mão** — as três ancoram em GEOMETRIA. O buraco real era o gesto mais usado do app (arrastar o gizmo), que **não tinha número nenhum** sobre a tela | 2 | — | **M** |
| R1 | **reduced motion** — um interruptor, e nasce **com** a F2 | 4 | F2 | **P** |
| ~~⭐~~ **X1** | **a pressão da caneta chega à shell** (afecta Flip **e** Painter) — ⚠️ o ⭐ e o **P** foram **REFUTADOS por medição**: winit 0.30.13 crava `force: None` nos três backends de desktop, então não há função a escrever. Ver a §8 | 3 | **winit** | **M/G** |
| D1 | **som de UI** opt-in, do motor que já temos | 4 | F0 | **M** |
| D2 | **partículas de feedback** do motor que já temos (dissolver em vez de sumir) | 4 | F0 | **G** |

⭐ = **melhor razão ganho/custo do quadro**, e nenhum dos dois é animado.

### 6.1 — A cauda da E3: as ROWS dos menus da barra (2026-08-13)

O commit da paleta global fechou nomeando esta wave, e o **enquadramento dele estava largo demais**.
Ele mediu que **o PILL** não é servível e escreveu *«a barra de topo fica de fora»*.

**Os verbos deste app são de DOIS tipos, e só um é servível por uma paleta:**

1. **o clique É o verbo** — endereçável por id, porque `chrome::dispatch_all` o resolve **sem
   geometria nenhuma** (os chips do rail · a visibilidade de um painel · **uma row de menu**);
2. **o clique ABRE um menu onde o verbo está** — posicional, ancorado a um `hit_rect`, e uma paleta
   **não tem rectângulo** (o chip pode nem estar visível).

O pill `Save` é tipo 2; a row `Save · Cmd+S` **de dentro dele** é tipo 1. São dois gestos diferentes
no mesmo widget — *abrir* e *escolher* — e só um é posicional.

**Verificado por LEITURA de cada handler, não por grep sobre um nome de função:** theme · radius ·
rail_size · view_toggles · io_menu · settings_* tocam o contexto de menu **apenas** por
`close_context_menu()`, que é um **no-op sem nada aberto**. As únicas que leem posição são as rows de
**CASCATA** do `SettingsMenu` (`cascade_anchor(hero, id)`) — e elas estão **FORA**: servidas pela
paleta seriam *consumidas* (`apply_event` devolve `true`) e abririam um submenu ancorado a uma row
que ninguém pintou. ⚠️ **Consumir não é FAZER**, e é por isso que o gate que carrega a wave tem por
oráculo o **EFEITO** (o tema muda, o `pixels_per_meter` muda) medido num `HeroScreen` que **nunca
abriu menu**, e não o `true` do despacho.

**Censo: 33 verbos, 9 menus** (File 2 · Open 2 · Look 12 · e os 5 submenus de Settings) — **um**
grupo com um *sub* por menu, não nove grupos: cada grupo é um CARD no masonry, e nove cards de duas
linhas seriam ruído.

⚠️ **O rótulo perde UMA coisa: o travessão de recuo.** Metade das rows do Look abre com `"— "` porque
o pintor de rows **não tem afordância de indentação**, então o menu codifica o recuo como
**caractere** — e numa lista plana de pills não há nada contra o que recuar. O atalho FICA
(`Save · Cmd+S`): mostrar a tecla ao lado do verbo é o que todo *command palette* faz.

E a tabela de rows saiu do `context_menu_overlay` para o **`menu_rows`** (218 linhas de
`match req.kind`) — porta única: ela estava certa inline enquanto o pintor era o único consumidor, e
deixou de estar no instante em que a paleta quis os mesmos verbos.

### 6.2 — ⚠️ A F2 está MEDIDA e não está feita: 161 sítios pintam, 2 leem

O substrato corre. O `hover_targets` publica um alvo para **todo** `Button`/`Radio`/`Toggle`/
`Checkbox` do store, o `tick` integra as molas, e depois:

| quem | número |
|---|---|
| sítios que **pintam** um Button | **103** |
| … um IconButton | 24 |
| … um Checkbox | 22 |
| … um Toggle | 12 |
| sítios que passam **`.hover_t()`** | **2** |

Os dois são o card de Fill. Fora deles a rota do `t` é o rail e a barra de topo, que leem por outra
porta (`hover_t(id)` do painter do rail). Todo o resto cai no **default `hover_t = 1.0`**, que faz
o `bg_color` saltar o ramo do blend e devolver o estado DURO. ⇒ *o botão de um painel salta ao lado
de um chip do rail que amacia*, e a mola daquele botão foi integrada na mesma.

⚠️ **A cerca do `hover_targets` está a ser violada ao contrário do que o autor dela temia.** O
doc-comment dele diz *«um tipo que não aparece aqui não ganha entrada nenhuma — é o que mantém o
mapa do tamanho do que se move»*; medido, o mapa é **maior** do que o que se move.

**E há um segundo buraco, mais barato:** quatro tipos **pintam** uma diferença de hover e nem estão
na lista — `ListItem` (`Bg2` no fundo — é a **Hierarquia**), `TextInput` e `Dropdown`
(`BorderEmph`), `Tag`. Esses não têm sequer o alvo.

**A wave que isto pede: 124 chamadas de `button_state` em 64 arquivos**, e ela tem uma **bifurcação
de desenho** que não é mecânica — ou cada sítio passa a pedir (`.hover_t(…)`, e o sítio 125 nasce
sem), ou a leitura passa a ser **estrutural** (o `button_state` devolve o par, e é o compilador que
enumera os sítios). ⇒ **não é «por gosto», e não se começa pela metade.**

### 6.3 — A CASCATA (F5), e o `ε` que foi reprovado uma vez

A paleta aparecia instantânea. Agora os cartões chegam escalonados — mas **o primeiro valor foi
REPROVADO no smoke**, e o que se aprendeu vale mais que o número.

**Rodada 1 (`ε = 0,020`) — reprovada.** *«os cartões têm um discreto movimento de subida
SIMULTÂNEO e não encadeado»*. Medido: os vizinhos viravam o alvo a **um quadro** de distância
(16,7 ms) — distinguível por um cronómetro, simultâneo para um olho.

⚠️ **O erro foi de CRITÉRIO, não de número.** Eu tinha escrito duas cercas — *acima de um quadro*
(senão dois cartões viram o alvo no mesmo tique) e *`(n−1)·ε < assentamento`* (para se ler como um
gesto) — e depois **escolhi o piso da primeira como valor**. Estar acima de um quadro é condição
**NECESSÁRIA**, e usei-a como suficiente.

⛔ **E a segunda cerca era invenção minha.** O smoke mostrou que uma cascata pode durar mais que o
assentamento e continuar a ler-se bem; ela foi **retirada** — com o gate que a encarnava — em vez
de afrouxada, porque estava errada e não apertada demais.

**Rodada 2 (`ε = 0,050`) — aprovada.** As duas perguntas que sobraram são as que o produto faz, e a
régua da primeira é o **QUADRO**:

| ε | quadros entre vizinhos | entrada n=3 | entrada n=7 |
|---|---|---|---|
| 0,020 | **1** ⛔ *(o reprovado)* | 0,22 s | 0,30 s |
| 0,035 | 2 | 0,25 s | 0,38 s |
| **0,050** | **3** | **0,28 s** | **0,48 s** |
| 0,065 | 3 | 0,30 s | 0,57 s |
| 0,080 | 4 | 0,33 s | 0,65 s |

`0,050` é o primeiro com três quadros de separação, e 0,065 compra os mesmos três por mais 0,09 s
de espera. O número está declarado como o que é — **número de APARÊNCIA, que sai do smoke e não de
um teste** —, o idioma que o `HOVER_LIFT_PX` já usava no mesmo módulo.

**Mais três coisas que a medição corrigiu antes disso:**

1. escrevi a tabela por **aritmética** e pu-la no doc antes de a medir — somando um assentamento de
   0,467 s que é do **Expressivo**, quando o default é o Discreto (0,22 s);
2. **`n = 17` era invenção minha** — a paleta de nós faz um grupo por categoria e há **sete**
   tokens `NodeCat*`;
3. a sonda mediu `n = 1` em **0,02 s — um quadro**: o `tick` somava `dt` **antes** de alvejar,
   então o cartão 0 nascia alvejado em `1.0` e, pela lei do substrato (*a primeira vista CHEGA ao
   alvo*), aparecia assente. **A wave era meio no-op e ia shipar assim.**

⚠️ Essa lei mordeu **três vezes** na wave — no produto, na sonda, e na fixture do meu próprio gate,
que media um «assentamento» de 0,017 s e comparava contra nada.

**A lei que fica:** *o desenho anda, o alvo não.* O cartão desenha-se 12 px abaixo durante a
entrada e o `hit_index` regista na posição **assente** — a mesma lei do `hover_lift`, que aqui
morde mais forte.

---

## §7 — O descartado, com o motivo (cercas plantadas antes)

- ⛔ **Confetti / explosões de celebração.** Um DCC é usado 8 h/dia. Uma recompensa animada é
  encantadora **uma** vez e uma interrupção nas outras quatrocentas.
- ⛔ **Animar o CONTEÚDO do canvas.** A obra é a verdade. O chrome pode estar vivo; a arte do
  artista, não — ela move-se apenas quando ele a move.
- ⛔ **`rapier` para decoração** (§5).
- ⛔ **Mola em NÚMEROS.** Uma posição pode balançar; um **valor lido** que balança está **errado
  durante 200 ms**. Números encaixam; posições molejam.
- ⛔ **Som ligado por omissão.**
- ⛔ **Qualquer animação que atrase a aceitação de um clique.**
- ⛔ **Skeuomorfismo por si.** O tether é físico porque a *relação* é física (uma ponta puxa a
  outra); um botão com textura de couro não descreve relação nenhuma.

---

## §8 — A ordem recomendada

**F0** (o degrau que desbloqueia o eixo inteiro, e corrige o toast) → **F1 · F2 · R1 juntos** (a mola
chega ao chrome, os 49 widgets ganham vida, e o interruptor que a desliga nasce no mesmo commit) →
**E1 scrub numérico** (o ganho de eficiência, independente de tudo) → **C1 o TETHER** (o pedido, e
com a F0 ele é barato) → o resto por gosto.

⚠️ ~~**X1 (a pressão da caneta) não depende de nada e está parada há waves — custa *uma função*.**~~
**MEDIDO E REFUTADO (2026-08-12): não custa uma função, custa uma DEPENDÊNCIA.** Ela é a única da
lista que melhora a **fidelidade do traço** e não a da interface, e por isso vale ter o preço certo:

- os **dois** sítios que constroem um `PointerEvent` (`input_dispatch.rs`) cravam `pressure: 1.0` e
  `source: PointerSource::Mouse` — isso é o que a frase *«custa uma função»* via, e está certo;
- mas **winit 0.30.13 não tem caminho de caneta em desktop nenhum.** O único evento que carrega
  pressão é o `WindowEvent::Touch { force: Option<Force> }`, e nos **três backends de desktop** o
  `force` é literalmente uma constante: `x11/event_processor.rs` escreve `force: None, // TODO`,
  o `wayland/seat/touch/mod.rs` escreve `force: None`, e o `windows/event_loop.rs` escreve
  `force: None, // WM_TOUCH doesn't support pressure information`. **Só `android`, `ios` e `web`
  o preenchem.**

⇒ **Ligar o `Touch` hoje seria uma função que mede `None` na máquina onde o smoke corre** — uma
feature que ninguém consegue julgar, que é a forma de dívida que este repo nomeia. O que move o
número é o **winit** (a API unificada de ponteiro, que esta medição **não** verificou — não há
versão mais recente na cache local para ler) ou um caminho por plataforma. É **decisão do Enio**, e
o tamanho honesto é **M/G com bump de dependência da shell**, não **P**.

⚠️ **E a consequência que já shipa: TRÊS controlos vivem a pretender que a pressão chega.** O
`WidthSource::Pressure` do lápis do Vector e os dois knobs `pressure_min_width` / `pressure_response`
do Flip são **provavelmente correctos e provadamente inertes** — os motores funcionam (há gates), a
entrada é que é constante. Não se removem (funcionam no dia em que a pressão existir); o que faltava
era o número estar escrito ao lado da promessa. E `PointerSource::Pencil` é um **variant que ninguém
constrói** (medido por grep) — ele nasce vivo no mesmo dia.

---

## §9 — O que este estudo NÃO diz

- **Não mediu o custo por quadro** de nenhum item. A F0 é `O(widgets vivos)` e o tether é
  `O(16 pontos)`, mas *o número sai da sonda, não daqui*.
- ~~**Não decide o carácter**~~ — ⚠️ **DECIDIDO pelo Enio no mesmo dia: as DUAS, e quem escolhe é o
  utilizador, no pill Settings.** Ver **§10**, que é agora a restrição mais forte do estudo: nenhum
  efeito entra sem resposta definida nos dois caracteres.
- **Não afirma o detalhe do produto do exemplo.** A corda descrita é a do Enio; o estudo trata o
  **padrão** (o tether) como arquétipo, e não depende de qual app a shipou.
- **Não abre wave nenhuma.** Nada aqui começa sem ordem explícita.

---

## §10 — A DECISÃO do carácter (Enio, 2026-08-12)

> *"Discreta e rápida **ou** expressiva e física, com escolha para o utilizador no pill Settings."*

Isto fecha a pergunta aberta da §9 e **passa a ser a restrição mais forte deste estudo**: nenhum
efeito da §11 entra sem uma resposta definida **nos dois caracteres**.

### 10.1 A lei: uma PORTA, perguntada uma vez

⚠️ **Discreto NÃO é Expressivo com os números baixos.** Se fosse, seria um multiplicador global e
qualquer um o escreveria em dez minutos — e o resultado seria uma UI expressiva a mexer-se depressa
demais, que é a pior das três. São **duas respostas diferentes à mesma pergunta**:

| | Expressivo | Discreto |
|---|---|---|
| o que governa o movimento | **mola** (rigidez/amortecimento) | **duração curta + curva** |
| o que o movimento comunica | *o objecto é físico* | *a mudança aconteceu, e onde* |
| percurso | pode ultrapassar e voltar | nunca ultrapassa |
| tempo típico | 200-450 ms, assentando | 80-140 ms, chegando |
| decoração (§11 D·F·G) | ligada | **ausente**, não atenuada |

E a lei estrutural, no molde que este repo já usa em toda parte: **a pergunta *"qual carácter?"* é
feita UMA vez** (`UiCharacter::of()`), e quem pinta consulta **a mesma porta** que quem despacha.
Duas cópias divergem no dia em que um efeito ganha um caso especial — é a cicatriz do
`TimelineInterpScope::menu_table()` e a do `stroke_cover_wanted`.

### 10.2 ⚠️ Discreto ≠ Reduced Motion — e colapsá-los seria shipar acessibilidade disfarçada de gosto

São **dois eixos independentes**, não três pontos de um:

- **Carácter** é *gosto*. Um utilizador em Discreto ainda quer os seus 100 ms de transição; ele só
  não quer que a interface tenha peso.
- **Reduced motion** é uma *garantia*: mata os gatilhos **vestibulares** — percurso de área grande,
  paralaxe, rotação, zoom — **independentemente do carácter escolhido**.

⇒ *Expressivo + reduced motion* é uma combinação legítima e tem de funcionar: alguém que gosta do
material, do som e da mola, mas a quem a paralaxe faz mal. Um único seletor de três posições
tornaria essa pessoa incapaz de pedir o que precisa sem desistir do que gosta.

### 10.3 ⚠️ E a decisão expôs uma peça em falta, medida

**Não existe preferência de APP neste repo.** Medido: `grep -rln "prefs\|preferences\|config_dir"
shells/desktop/src/` devolve **vazio**, e as `SavedSettings` (v69 — escala do mundo, unidade, snaps,
filtragem) **viajam dentro do `ProjectFile`**.

Pôr o carácter ali seria dizer que **o gosto viaja com o documento**: abrir o ficheiro de um colega
mudaria como o *seu* app se mexe. Isso está errado do mesmo modo que uma binding de timeline apontar
para bits de entidade.

⇒ o carácter (e o reduced motion, e o volume do som da §11 G) pedem um **armazém de preferências de
utilizador** — ficheiro pequeno no config dir, fora do `PROJECT_SCHEMA`. Não existe, é barato, e o
carácter seria o **primeiro inquilino**. *Nomeado, não contrabandeado dentro da F0.*

### 10.4 O que isto faz à lista da §6

Cada linha ganha uma coluna implícita — *o que ela faz em Discreto* — e a **R1** deixa de ser um
item: ela vira **parte da F2**, porque a porta de carácter e a garantia de reduced motion são o
mesmo sítio no código, e um efeito que nasce sem as duas nasce dívida.

---

## §11 — O CATÁLOGO: os efeitos, além da corda

Quarenta e um, por **mecanismo** — não por app que os shipou. A coluna **Discreto** é o que a §10
obriga: onde diz *"ausente"*, o efeito simplesmente **não existe** naquele carácter, e isso é uma
resposta, não uma falha.

### A — MASSA E MOLA (o objecto tem inércia)

| # | efeito | o mecanismo | onde cairia aqui | em Discreto |
|---|---|---|---|---|
| A1 | **Overshoot / settle** | o alvo é ultrapassado e a mola devolve | painéis, chips, o card de onion | ausente |
| A2 | **Squash & stretch** | o corpo deforma na direcção do movimento | thumbnails a voar, o chip arrastado | ausente |
| A3 | **Rubber-band de fim de curso** | arrastar além do limite **resiste** e devolve | fim de lista, limite de slider, borda do canvas | ausente (encosta seco) |
| A4 | **Inércia / fling** | o conteúdo continua depois de o dedo sair | pan de canvas, listas roláveis, a tira do Flip | ausente |
| A5 | **Sag / catenária** | algo longo cede sob o próprio peso | divisor de painel arrastado, a própria corda | ausente |
| A6 | **Pêndulo** | o que está pendurado balança ao aparecer/mover | dropdown, popover, painel flutuante | ausente |
| A7 | **Massa diferencial** | elementos distintos têm massas distintas; o pesado atrasa | hierarquia de leitura **feita com movimento**, não com cor | ausente |
| A8 | **Recoil** | o botão empurra de volta e recupera | todo botão, todo chip | *press* instantâneo |
| A9 | **Aproximação magnética** | o ímã **puxa visivelmente** antes de agarrar | o snap (que hoje só marca **depois**) | só a marca |
| A10 | **Detent** | o valor "estala" nos pontos notáveis, com resistência | sliders de ângulo, opacidade, zoom | ⚠️ **fica** — é função, não enfeite |

### B — MATERIAL E DEFORMAÇÃO (a superfície tem propriedades)

| # | efeito | o mecanismo | onde cairia aqui | em Discreto |
|---|---|---|---|---|
| B1 | **Gooey / metaball** | dois corpos que se aproximam fundem-se por um istmo (soma de campos) | o indicador de aba a mudar; chips a agrupar | ausente |
| B2 | **Ripple do ponto de toque** | a onda nasce **onde o dedo tocou**, não no centro | botões grandes, o canvas ao confirmar | ausente |
| B3 | **Vidro / refracção** | o painel refracta o que está por trás | painéis flutuantes sobre a arte | ausente (opaco) |
| B4 | **Tilt paralaxe** | o card inclina para o cursor; camadas a taxas diferentes | cards da paleta de comandos, thumbnails | ausente ⚠️ **e é gatilho vestibular** |
| B5 | **Specular sweep** | um brilho atravessa a superfície quando ela fica disponível | um botão que acabou de ficar activo | ausente |
| B6 | **Membrana elástica** | a borda do contentor cede quando um filho é arrastado contra ela | auto layout, moldura, a tira | ausente |
| B7 | **Sombra que responde à altura** | a sombra conta *quão acima* o objecto está, e muda ao levantar | arrastar um objecto na hierarquia | sombra fixa |

### C — CONEXÃO (a relação é geometria)

| # | efeito | o mecanismo | onde cairia aqui | em Discreto |
|---|---|---|---|---|
| C1 | ⭐ **TETHER / corda** | pontos de massa + restrição de distância, entre controlo e efeito | o gizmo ↔ o botão que o armou (o pedido) | **linha reta** |
| C2 | **Fio elástico** | o fio estica-se e assenta ao ligar dois sockets | grafo de nós, o picker de caminho-guia | linha reta |
| C3 | **Magic move / elemento partilhado** | o que existe dos dois lados **move-se**; não desaparece | painel → modal, thumbnail → canvas | *cross-fade* curto |
| C4 | **Morph de FORMA** | o botão **vira** o painel; a silhueta transita | ⚠️ temos motor de blend de formas (`ph2d-vec-blend`) | ausente |
| C5 | **Voo / portal** | o item voa da origem para o destino, com arco | shape nova → linha da hierarquia; cor → swatch | ausente |
| C6 | **Fio de dependência** | *hover* num socket **apaga** o que não é a jusante | grafo de Motion, pilha de LPE | ⚠️ **fica** — é legibilidade |
| C7 | **Realce de proveniência** | passar sobre o valor acende o objecto — **e o inverso** | inspector de ~1.600 ids | ⚠️ **fica** |

### D — PARTÍCULAS (o gesto deixa rasto)

⚠️ Família inteira **ausente em Discreto**. E temos o motor: 119 nós, milhões de instâncias na GPU.

| # | efeito | o mecanismo | onde cairia aqui |
|---|---|---|---|
| D1 | **Dissolver em vez de sumir** | o apagado **emite as próprias partículas** | DELETE — torna legível *o quê* e *onde* |
| D2 | **Poeira de impacto** | faísca no ponto EXACTO do encaixe | snap, join de nós, o pouso de um strip |
| D3 | **Rasto do cursor** | trail que segue o ponteiro | ⚠️ dose mínima; num app de pintura pode ser **pigmento** |
| D4 | **Sopro do pincel** | pressão forte solta um puff | temos quatro meios de pintura a justificá-lo |
| D5 | **Confirmação por emissão** | o commit emite uma vez, do ponto de acção | Apply, bake, export |

### E — TEMPO E CADÊNCIA (o ritmo comunica)

| # | efeito | o mecanismo | onde cairia aqui | em Discreto |
|---|---|---|---|---|
| E1 | **Cascata / stagger** | `atraso = índice × ε` faz N itens lerem-se como **um** gesto | paleta, hierarquia, rows do inspector | ⚠️ **fica**, com ε menor |
| E2 | **Antecipação** | o recuo antes do avanço | abrir um painel grande | ausente |
| E3 | **Follow-through** | o corpo chega e as **partes** continuam | o rótulo assenta depois do card | ausente |
| E4 | **Ease assimétrico** | entrar e sair **não são a mesma curva** | tudo | ⚠️ **fica** — é regra, não dose |
| E5 | **Máscara temporal** | a animação preenche **exactamente** o trabalho real | load, bake, cook — em vez de spinner | ⚠️ **fica** |
| E6 | **Interrupção com herança de velocidade** | o alvo novo parte da velocidade actual | **todos** | ⚠️ **fica** — é a lei, não um efeito |

### F — VIDA OCIOSA (a UI respira)

| # | efeito | o mecanismo | onde cairia aqui | em Discreto |
|---|---|---|---|---|
| F1 | **Breathing** | pulso lento no que espera acção | o botão primário de um estado vazio | ausente |
| F2 | **Shimmer de atenção** | o que mudou **por baixo** do utilizador brilha **uma** vez | render acabou, ficheiro recarregou, colega editou | ⚠️ **fica** |
| F3 | **Cursor com peso** | o anel segue com atraso mínimo | ⚠️ **já temos o anel** (`the_brush_ring_wears_the_live_dab_rotor`) | sem atraso |
| F4 | **Consciência de proximidade** | reage à **distância** do cursor, não só ao contacto | o rail de ferramentas (o dock do macOS) | ausente |
| F5 | **Progresso SEM spinner** | o elemento **é** a barra: a borda enche | o botão que dispara o trabalho | ⚠️ **fica** |

### G — SOM (o motor existe e não toca a UI)

⚠️ **Opt-in sempre; nunca por omissão.** Temos 42 efeitos, mixer e vozes por streaming.

| # | efeito | mecanismo | em Discreto |
|---|---|---|---|
| G1 | **Click / detent** | o valor notável tem um som distinto do valor comum | ⚠️ independente do carácter — é o **seu** interruptor |
| G2 | **Whoosh de transição** | curto, ligado ao percurso; **cala-se** sem percurso | ausente |
| G3 | **Confirmação / recusa** | dois timbres; a recusa nunca é um "erro" agressivo | fica |
| G4 | **Textura da ferramenta** | a caneta tem um som; a faca tem outro | ⚠️ decisão de produto, dose alta |

### H — ESPAÇO E FOCO (o que se vê muda com a intenção)

| # | efeito | mecanismo | onde cairia aqui | em Discreto |
|---|---|---|---|---|
| H1 | **Zoom semântico** | o conteúdo muda de **representação**, não de tamanho | tira do Flip, timeline, hierarquia funda | ⚠️ **fica** |
| H2 | **Focus + context / fisheye** | a vizinhança do cursor expande | listas longas, a tira, o dope-sheet | ausente |
| H3 | **Peek mola-carregado** | segurar mostra, largar **desfaz** | pré-visualizar uma camada, um clip, um preset | ⚠️ **fica** — é agência |
| H4 | **Dim do irrelevante** | a acção corrente **apaga** o que não participa | modo de corte, pick de caminho, box-select | ⚠️ **fica** |

### ⚠️ O que o catálogo revela quando se lê a coluna Discreto

**Dezassete dos 41 ficam em Discreto** — e não são os decorativos: são o **detent**, o **ease
assimétrico**, a **interrupção**, o **fio de dependência**, a **proveniência**, o **zoom semântico**,
o **peek**, o **dim**, a **máscara temporal**. Ou seja:

> a maior parte do que faz uma UI parecer *inteligente* não é do eixo expressivo — é **legibilidade
> e agência**, e sobrevive intacta no carácter discreto. O que Discreto remove é **peso, percurso e
> rasto**; o que ele **não** pode remover é a UI dizer *porquê*.

⇒ E isso reordena a §6: **C6 · C7 · H3 · H4 · E4 · A10** valem em **ambos** os caracteres, logo o seu
retorno é o dobro do de qualquer efeito da família D ou F.
