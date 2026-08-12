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
| **readout que segue a mão** (o rótulo de distância) | `LengthDisplay`, W6.6 | ⚠️ **uma** instância |
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
| **Readout que segue a mão** | o número aparece onde os olhos já estão | ⚠️ 1 instância |
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
| F2 | **hover/press/focus interpolam** — os 49 widgets herdam de graça pela porta de pintura | 1 | F0+F1 | **P** |
| F3 | **interruptibilidade** como lei: todo alvo novo herda a velocidade actual | 1 | F1 | **P** |
| F4 | **secções e painéis** abrem/fecham com movimento e **direcção coerente** | 1 | F1 | **M** |
| F5 | **cascata** na lista da paleta / hierarquia / rows do inspector | 1 | F0 | **P** |
| ⭐ **E1** | **SCRUB numérico** em todo campo de número (o maior ganho de eficiência do estudo) | 3 | — | **M** |
| E2 | **inércia** no pan de canvas e nas listas roláveis | 3 | F0 | **P** |
| E3 | **paleta de comandos GLOBAL** (o widget já existe) | 3 | — | **M** |
| E4 | **menu radial** sob a caneta / botão do meio | 3 | — | **M** |
| C1 | **TETHER** (§5) + as três irmãs da família | 2 | F0 | **M** |
| C2 | **realce de proveniência** nos dois sentidos (valor ↔ objecto) | 2 | — | **M** |
| C3 | o **readout que segue a mão** vira REGRA (hoje é 1 instância) | 2 | — | **M** |
| R1 | **reduced motion** — um interruptor, e nasce **com** a F2 | 4 | F2 | **P** |
| ⭐ **X1** | **a pressão da caneta chega à shell** (custa *uma função*; afecta Flip **e** Painter) | 3 | — | **P** |
| D1 | **som de UI** opt-in, do motor que já temos | 4 | F0 | **M** |
| D2 | **partículas de feedback** do motor que já temos (dissolver em vez de sumir) | 4 | F0 | **G** |

⭐ = **melhor razão ganho/custo do quadro**, e nenhum dos dois é animado.

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

⚠️ **X1 (a pressão da caneta) não depende de nada e está parada há waves** — é a única da lista que
melhora a **fidelidade do traço**, não a da interface.

---

## §9 — O que este estudo NÃO diz

- **Não mediu o custo por quadro** de nenhum item. A F0 é `O(widgets vivos)` e o tether é
  `O(16 pontos)`, mas *o número sai da sonda, não daqui*.
- **Não decide o carácter** — se a UI deste app deve ser discreta e rápida ou expressiva e física. As
  duas são coerentes; a escolha é do Enio, e ela decide as doses do eixo 1.
- **Não afirma o detalhe do produto do exemplo.** A corda descrita é a do Enio; o estudo trata o
  **padrão** (o tether) como arquétipo, e não depende de qual app a shipou.
- **Não abre wave nenhuma.** Nada aqui começa sem ordem explícita.
