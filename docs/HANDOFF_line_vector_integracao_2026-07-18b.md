# HANDOFF de INTEGRAÇÃO — `line/Vector`, sessão de 2026-07-18

**Para:** o agente integrador (DIRETRIZ §1.5.3–1.5.4), quando o Enio mandar.
**Estado:** ✅ linha **fechada e verde**, **27 commits** sobre a `main`. **NÃO integrei e NÃO
pushei** — a linha fecha, entrega o handoff e para (CLAUDE.md §0.7).

---

## §0 — Leia isto primeiro (o resto é detalhe)

**O que a linha entrega:** os **Live Path Effects** — uma pilha por-caminho avaliada dentro do
`cooked()` ([ADR-0132](architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)),
com **4 efeitos** (Trim Path · Zig Zag/Roughen · Repeater · Pucker & Bloat), a **seção Effects**
no painel (cards com ordenar/olho/apagar) e a **sonda visual** que passou a ser o método.

**Tudo smokado pelo Enio** (2026-07-18), incluindo o smoke final do Repeater enriquecido.

### As três coisas que decidem a ORDEM de integração

| ⚠️ | Facto | Onde |
|---|---|---|
| **Toca FOUNDATIONAL** | `ph2d-editor-core` — o scrub das caixas numéricas inteiras. Passa por `scripts/foundational-integrate.sh` (ADR-0107). | §12 |
| **Bumpa 2 schemas** | `VEC_SCENE_SCHEMA_VERSION` 8→**13** · `PROJECT_SCHEMA` 18→**23** · tripla do gate `(23, 8, 13)` | §6 |
| **Contrato congelado: NÃO tocado** | O `NodeOp`/`OpResolver`/`NodeManifest` (§6 do CLAUDE.md) foi **medido** e não bloqueia — a pilha não é um grafo. | §2, §3 |

⚠️ **Se outra linha bumpar o `PROJECT_SCHEMA` na mesma jornada, o valor certo não está em nenhum
dos dois lados do conflito: ele se CONTA**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

### O que fica ABERTO (nenhum é bloqueante)

1. **undo/redo dos efeitos** — o Enio reportou-o três vezes e eu **não consegui reproduzi-lo**.
   Varri o caminho inteiro quatro vezes, com dois agentes, e **não há assimetria nenhuma** entre
   os efeitos e o resto do documento. Corrigi o único sítio do frame onde eles se comportavam
   de forma diferente (o pivô — §15.4). Protocolo de diagnóstico em §11.
2. **Offset Path não pode ser um efeito da pilha** — achado arquitetural, §15.5.
3. **O Twist foi CORTADO** e a razão está no §15.3. Não é dívida: é uma decisão.

---

---

## §1 — O que entra

Em quatro blocos, na ordem em que aconteceram.

**A espinha (ADR-0132) + o 1º efeito**

| SHA | O quê |
|---|---|
| `02382568` `19383f48` | **ADR-0132** — a decisão: LPE é uma PILHA por-caminho, não um grafo de nós |
| `e5e40aa6` | **fix**: a alça de raio pergunta se a geometria é DERIVADA (bug vivo, 4 de 5 objetos) |
| `db50c236` | **feat**: a PILHA + o motor de arco (`arclen.rs`) + o **Trim Path** |
| `6f599cf1` `b6e66db5` | **feat**: a cena de smoke `PH2D_BUILD_SMOKE=13` |

**O painel, e a promessa do ADR posta à prova**

| SHA | O quê |
|---|---|
| `e5992c4b` | **feat**: a **seção Effects** — o artista alcança a pilha |
| `130cde9e` `ea46a9b9` | **feat**: **Zig Zag / Roughen**, o 2º efeito |
| `662e1f48` | **refactor**: a seção é **DIRIGIDA PELA TABELA** — o próximo efeito custa zero painel |
| `c0d69ab8` | **feat**: cada efeito num **CARD** (nome · ordenar · olho · apagar) e o `Size` **relativo** |

**Os quatro achados do 1º smoke**

| SHA | O quê |
|---|---|
| `e38134a1` | **fix**: o 2º ZigZag ondula **sobre** o 1º (era aliasing: apagava-o) · o chip mostra o número do **documento** |
| `7fa5f969` `4daf72cc` | **fix**: o log do undo explica o silêncio (andaime, retirado depois) |
| `64a95f4b` | **fix** ⚠️ **foundational**: a caixinha alterna · o arrasto vertical de uma CONTAGEM anda |
| `c76b1f80` | **fix**: a caixinha ganha **id próprio** — um id não pode ter dois tipos de widget |

**Os efeitos, e a correção do MÉTODO**

| SHA | O quê |
|---|---|
| `2518ea39` | **feat**: Repeater, Twist e Pucker & Bloat — a promessa do ADR **medida** (zero painel) |
| `a7ea910c` | **fix**: os três defeitos que o smoke apanhou — e nenhum precisava de smoke |
| `d63b225a` | **feat**: a **sonda de RENDERIZAR-E-OLHAR** — e o **Twist é CORTADO** |
| `3d803ebf` | **feat**: o Repeater ganha o **2º eixo** (grelha) e a **2ª rotação** — o Array do Blender a sério |

(Os `docs(vector)` intercalados são atualizações deste handoff.)

**Base:** `02382568` (a linha estava a 2 commits de docs sobre a `main` `389676f9`).

---

## §2 — A decisão, em três frases

A fila pedia *"Live Path Effects como NÓS"*. **O contrato congelado foi medido e não bloqueia**
(`CookValue::Opaque` + `Domain::Vector` + `input_any`/`emit_any` já carregam geometria em aresta —
padrão Houdini/USD; param não-`f32` tem o canal de TEXT PARAM e a convenção de discriminante `f32`),
então **não houve PARE nem ADR de contrato** — e é justamente por a escolha ser livre que ela teve de
se defender pelo desenho.

**O desenho é PILHA, não grafo**: LPE é uma *lista no objeto selecionado* em toda ferramenta shipada;
não existe grafo POR objeto (o Motion Nodes tem UM para a cena inteira); o `Cow::Borrowed` que
sustenta o `cooked()` é trivial numa lista e deixaria de ser um `if` sob chave de memo; e é o desenho
que já funcionou 4× nesta linha. **O caminho do nó fica aberto de graça** — cada efeito é função pura
numa crate/módulo, então um nó o *embrulha* em vez de o reimplementar (ADR-0132 §4).

---

## §3 — Onde a pilha mora, e por que não podia morar noutro sítio

`VecPath.effects` é dado de documento; o `cooked()` roda a pilha **logo depois do estágio da quina**.
Nenhum consumidor mudou — o funil já era o `cooked()`.

⚠️ **Duas restrições de camada decidiram isto, e quem for mexer precisa de as saber:**

1. **O `cooked()` é chamado de DENTRO do próprio `ph2d-vec-scene`** (`inside.rs`, `boundary.rs`,
   `path_ops.rs`, `space.rs`). Avaliar a pilha noutro crate deixaria o **hit-test e a bbox** vendo
   geometria sem efeito — *"o que se vê"* divergindo de *"o que se aponta"*.
2. **`ph2d-vec-scene` é sem-kurbo por decisão declarada no `Cargo.toml`.** Por isso o motor de arco
   (`arclen.rs`) nasceu aqui em vez de vir do `kurbo::inv_arclen`: arrastar a stack Linebender para
   dentro do modelo de documento por 40 linhas de quadratura seria pagar caro por uma cerca decidida.

**A quina é o estágio ZERO e não entra na pilha** (ADR-0132 §3): o raio mora no vértice **autorado** e
arredondar **divide** um vértice; todo efeito a jusante **resampleia**, então a contagem de vértices é
*saída* dele. Os 4 sítios que escrevem `corner_radius: 0.0` em `ph2d-vec-envelope`/`ph2d-vec-blend`
**estão certos** — não os "conserte".

---

## §4 — Os invariantes que não podem morrer (e os gates que os seguram)

| Invariante | Por quê | Gate |
|---|---|---|
| Pilha vazia = **mesmo ponteiro** | foi o que permitiu ligar o `cooked()` em todo consumidor sem mudar comportamento | `an_empty_stack_still_borrows_the_source` |
| Pilha **neutra** também empresta | abrir a seção Effects e não configurar nada não pode custar uma alocação/frame | `a_stack_of_neutral_effects_still_borrows` |
| Cozinhar 2× == 1× | a saída sai com a pilha **vazia**, espelhando o `corner_radius: 0.0` do `corner_live`; sem isso a forma encolhe a cada passagem, **sem erro nenhum** | `cooking_the_cooked_path_changes_nothing` |
| A **ordem** importa | é o que faz "reordenar por arrastar" ser feature | `the_order_of_the_stack_changes_the_geometry` |
| Trim mede por **ARCO** | a versão ingênua (fatiar por `t`) *parece certa numa reta* | `asking_for_a_fraction_returns_that_fraction_of_the_length` |

**20 gates novos** (6 arco · 6 pilha · 8 trim) + **7** da alça de raio + **2** arch-gates.
Os do arco usam **oráculo externo** (reta em forma fechada · amostragem densa de 200k cordas), nunca
a mesma quadratura a concordar consigo mesma.

**Mutações:** 3 na pilha/trim (3 gates distintos, um cada — inclusive a implementação ingênua da
pesquisa) · 3 no guard da alça (**conjuntos distintos**: esquecer conector+morph derruba esses 2; não
subir a cadeia derruba o envelope; recusar demais derruba os 2 controles de presença) · 1 no
arch-gate.

---

## §5 — SMOKE (o que o Enio tem de ver)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  PH2D_BUILD_SMOKE=13 cargo run -p ph2d-host-desktop --release
```

- **A elipse desenha-se sozinha** em ~3 s, segura cheia ~1 s, e **RECOMEÇA**. ⚠️ **O que importa não
  é que ela apareça — é que a ponta ande a velocidade CONSTANTE.** Se ela acelerar e frear conforme a
  curvatura, a medida voltou a ser por `t` e o gate da fração ficou para trás.

  > ⚠️ **A 1ª versão desta cena foi REPROVADA no smoke** (*"na elipse não vejo nada acontecendo"*) e o
  > motor estava certo: a rampa era **one-shot** e acabava antes de o Enio olhar para a janela — o que
  > restava era uma elipse inteira e **parada**. O gate headless
  > (`the_smoke_ellipse_reveals_progressively`, sobre a `shape()` do PRODUTO) inocentou o Trim, e o
  > gate novo mora na **política de tempo** (`draw_on_phase`), que era o que estava errado. Um
  > exemplo que exige apanhar uma janela de 3 s no arranque não está pronto para smoke.
- **A estrela**: a janela de ¼ do caminho **gira** à volta da forma e atravessa a emenda sem
  tropeçar.

**A seção Effects** smoka-se à mão, e é o teste mais direto do produto. Com a tool **Vector**,
desenhe uma forma, selecione-a (**um** caminho — a seção é por-caminho) e abra **Effects**:

1. **Add Trim Path** — a forma **não pode mudar** no clique (todo efeito nasce neutro). Arraste
   **End** para baixo: ela encurta a partir do fim, **medindo por arco**.
2. **Add Zig Zag** — suba o **Size**. Ligue **Smooth: Off → On** (ondas em vez de serrote) e
   **Rough** (o mesmo motor, deslocamento pseudo-aleatório e determinístico).
3. **Reordene com Up/Down** e olhe: ondular-depois-cortar **não** é cortar-depois-ondular. Se as
   duas ordens desenharem igual, a pilha não está a compor.
4. **O olho** desarma sem apagar: os parâmetros do efeito desarmado continuam lá e editáveis.
5. **Add Repeater** — `Copies X` faz a fileira e `Move X = 100` **encaixa sem folga**. Ligue
   `Copies Y` e sai uma **grelha**. `Spin` gira cada cópia sobre si mesma (a fileira continua
   fileira); `Orbit` **leva** a cópia — é o arranjo radial. Se as duas rotações desenharem igual,
   uma delas é um botão morto.
6. **Add Pucker & Bloat** — negativo dá a **estrela de pontas**, positivo dá a **flor**, e o meio
   é a forma intacta. Se ele só aumentar e diminuir a forma, voltou a ser uma escala.

⚠️ **Tudo o acima foi smokado e aprovado pelo Enio** (2026-07-18). O que segue por confirmar é só
o comportamento do **undo** sobre a pilha (§11).

**Também vale conferir o fix da alça de raio** (`e5e40aa6`), que é independente: num **filho de
envelope** (`PH2D_BUILD_SMOKE=11`), no modo **Node**, as alças de raio **não devem mais aparecer**.
Antes apareciam, funcionavam, e o raio sumia no frame seguinte.

---

## §6 — Gates rodados nesta árvore

- `cargo check --workspace --all-targets` ✅ (o campo novo em `VecPath` muda o layout postcard de tudo)
- `cargo test` das 4 crates tocadas (`ph2d-vec-scene` · `ph2d-panel-vector` · `ph2d-host-desktop` ·
  `ph2d-editor-core`) ✅ **1964**, 0 falhas
- `cargo clippy --workspace --all-targets` ✅ **0 warnings** · `cargo fmt` **antes** de medir LOC
- ⚠️ Os arch-gates de arquivo (`no_magic_numeric`, LOC cap) moram na **`ph2d-editor-core`** e **não**
  rodam com `cargo test -p` de outro crate. A linha pagou esse pedágio **três** vezes — a última no
  commit final da 1ª leva, e o gate apanhou um `100.0` cru a caminho do commit.

**Schema:** `VEC_SCENE_SCHEMA_VERSION` 8→**13** · `PROJECT_SCHEMA` **18→23**, tripla do gate de
acoplamento em `(23, 8, 13)`. A escada, para quem tiver de a fundir com outra linha:

| v | o que mudou de forma |
|---|---|
| 9 | `VecPath` ganhou `effects` |
| 10 | a entrada da pilha virou `FxEntry` (o efeito **+ se está LIGADO**) |
| 11 | variants `Repeat`/`Twist`/`Bloat` |
| 12 | o `Twist` saiu e os índices fecharam-se atrás dele (a v11 nunca existiu num save) |
| 13 | o `RepeatSpec` ganhou o 2º eixo e a 2ª rotação |

De passagem, a narrativa do gate ganhou o **v18** que ninguém tinha acrescentado (a UNIDADE do
`width` do Flip, `cb42c9a2`).

---

## §7bis — A promessa do ADR foi TESTADA, e valia metade (`130cde9e` → `662e1f48`)

O ADR-0132 alegou que a espinha faria cada efeito futuro custar pouco. **Medi, com o 2º efeito:**

| | Custo real |
|---|---|
| **Motor** do Zig Zag | ~170 linhas num módulo irmão + 1 variant + 2 braços de match. ✅ |
| **UI** do Zig Zag | outra rodada completa dos 8 sites de costura. ❌ |

O gargalo tinha deixado de ser a geometria — e o 3º efeito pagaria o mesmo pedágio. **O remédio já
existia neste repo**: a rack de áudio, onde *"o painel se auto-popula da tabela `KINDS`"*.

**Agora o motor DESCREVE e o painel RENDERIZA.** `PathEffect` ganhou
`KINDS`/`from_kind`/`params`/`get`/`set`; um efeito declara os próprios parâmetros (nome, faixa,
é-caixinha) e a seção desenha linhas genéricas. **Nem `paint_effects.rs` nem `fx_bridge.rs` nomeiam
um efeito.** O 3º tipo custa: um variant, um braço de `apply`, uma linha em `KINDS`.

⚠️ **Três coisas que quem mexer precisa de saber:**

1. **A fronteira de unidades mudou de lugar.** O painel manda o track **normalizado `0..1`** e não
   conhece a faixa; quem converte é a **ponte**, que a lê do efeito. Se a conversão vivesse no painel
   haveria duas cópias da faixa, e elas divergiriam no 1º efeito com faixa diferente — que é
   exatamente o que o Zig Zag trouxe (`Size` a 100, `Ridges` a 64, contra as frações do Trim).
2. **Um parâmetro de CAIXINHA partilha o id do slider** (o painel pinta um *ou* outro) e chega como
   `Click`, não `ValueChanged`. O dispatch consulta a **cena** antes de alternar (`is_toggle`) —
   sem isso um clique perdido num slider viraria escrita silenciosa. Há gate.
3. **A pilha agora é pilha de verdade** (N efeitos, teto 4, com Up/Down). Nas bordas os botões **não
   são oferecidos**: subir a primeira linha não faz nada, e botão inerte ensina o artista a
   desconfiar dos que funcionam.

**`ph2d_editor_core::ids` passou a re-exportar `NodeId`** — o módulo distribui `NodeId`s e as
fábricas indexadas devolvem um, mas ninguém de fora conseguia **nomear** o tipo.

**SEM bump de schema no 2º efeito**, e a razão importa: postcard indexa o variant e o novo foi
**apendado** — um save v9 só com Trim continua a ser lido certo. A regra do gate de acoplamento é
*"bumpe quando um arquivo antigo passar a ser lido ERRADO"*, não a cada mudança de forma.

---

## §7 — A seção *Effects* — **FEITA** (`e5992c4b`, refatorada em `662e1f48`)

A pilha deixou de ser alcançável só por código: há uma seção no painel vetorial com o **toggle do
Trim** + **Start/End/Offset**, costurada nos 8 sites.

**A UI expõe UM Trim por caminho, e isso é decisão, não limite do motor.** Só existe **um tipo** de
efeito: empilhar dois Trims idênticos é curiosidade, e reordenar dois iguais não significa nada.
Quando o 2º tipo chegar, *"em que ordem?"* vira pergunta real e a lista com reordenação nasce com
ela. Por isso o botão é um **TOGGLE** — "Add" sobre um caminho que já tem Trim criaria o segundo,
que a tela não sabe mostrar.

Três decisões que quem mexer precisa de conhecer:

- **Pôr o Trim não muda o desenho**: ele nasce no ponto **neutro** (no-op byte-idêntico), senão a
  forma saltaria no instante do clique. Há gate.
- **Tirar remove TODOS os Trims**, não só o primeiro — a UI expõe um, mas um documento vindo de
  código pode ter mais, e órfãos invisíveis seriam a pior saída.
- **A MESMA `sole_path`** responde ao painel (o que PINTAR) e ao dispatch (onde ESCREVER). Duas
  portas divergiriam e a seção ofereceria controle para um caminho que o clique não alcança.

Os três sliders **não são pintados** sem Trim — nem *dimmed*: um controle apagado que ainda despacha
mente, e um que não despacha é um botão morto.

⚠️ **Dívida herdada que paguei de passagem** (não é minha, mas o commit a toca): `apply_event`
passou dos 200 LOC e virou `is_trim_param`/`forward_trim`; `state.rs` estourou 600 e o estado dos
Effects saiu para `state_effects.rs`. E **o gate de tofu pegou uma seta `U+2192` num `assert!` do
commit anterior** — ele mora na `ph2d-editor-core` e eu só tinha rodado a suíte do shell: rodar
`cargo test -p ph2d-host-desktop` **não** é o gate completo.

**Resto da fila** (com §4.2 já corrigido no handoff de continuação — o **morph vivo JÁ ESTÁ FEITO**,
`244e546e`): chamfer (quase de graça) · texto em caminho (agora barato — o `arclen` existe) ·
repeater · largura variável · blend em cadeia.

---

## §8 — Uma coisa que encontrei e não corrigi

**`CLAUDE.md` §5 diz que o `PROJECT_SCHEMA` é 13.** Ele estava em **18** quando comecei (agora 19).
Não o mexi porque o `CLAUDE.md` é território partilhado por todas as linhas e uma edição minha ali
colide com todas elas na integração — mas alguém tem de o corrigir, e o integrador é quem está em
posição de o fazer sem colisão.

---

## §9 — O smoke do Enio (2026-07-18) e os quatro achados

Enio smokou a seção e reportou quatro coisas. Três eram bugs meus; a quarta foi uma ordem.

### §9.1 — O layout: cada efeito num CARD (`c0d69ab8`)

> *"Funciona mas muito mal organizado o layout. Pode ser assim: EFEITO, setinha para cima
> (ordenar), setinha para baixo, olho de ocultar, x de fechar(apagar). Abaixo os parâmetros. Cada
> efeito dentro de um card. Veja cards no painel painter"*

Feito, com o **mesmo `paint_card`** do painel do Painter — não podia haver duas respostas a *"como
é um card neste app"*. Os ícones são contados **da direita**, então o ✕ nunca muda de sítio; subir
na 1ª linha e descer na última **não são desenhados** (ícone inerte ensina a desconfiar dos que
funcionam).

O **olho** exigiu motor: `FxEntry { effect, enabled }`. O "ligado" mora na ENTRADA, não no efeito —
zerar a amplitude para "desligar" e depois querer o valor de volta obriga o artista a lembrar-se de
números. `VEC_SCENE_SCHEMA_VERSION` 9→10.

### §9.2 — O `Size` era absoluto, e o slider era inútil (`c0d69ab8`)

> *"os valores de zigzag size estão de escala gigantesca. Melhor colocar relativa ao tamanho da
> forma. Algo do tipo: 100 = média entre as dimensões de x e y da própria forma"*

O doc do código **argumentava que unidades de mundo eram o certo**. Era o inverso: as formas da cena
têm ~2-3 unidades, então o slider destruía a forma no primeiro centésimo do curso. Nasceu o
`FxCtx { ref_size }`, tirado **uma vez do caminho AUTORADO** — não do que chega a cada efeito. Se
viesse da entrada, pôr um Trim antes do ZigZag encolheria a onda, e o mesmo `Size` significaria
coisas diferentes conforme a **ordem** da pilha.

### §9.3 — O 2º ZigZag APAGAVA o 1º (`e38134a1`) ⬛ o achado grande

> *"um zigzag funciona bem! mas se eu acrescentar um segundo zigzag ele não atua bem sobre o
> primeiro zigzag"*

Não atuava mal: **apagava**. Medido, círculo de raio 60 — 16 cristas dão 718 de comprimento, e um 2º
zigzag de 4 cristas deixava **7 âncoras e 310**, mais curto do que o próprio círculo (339).

**Causa: aliasing.** O efeito descartava as âncoras de entrada e reamostrava em `2·ridges` posições.
Sobre um caminho liso isso é exato; sobre um caminho que **já tem uma onda**, é amostrar mais grosso
do que o sinal que lá está. O Illustrator e o AE não sofrem disto porque contam cristas **por
segmento** — e é *por isso* que empilhar lá produz o fractal auto-semelhante que se espera. Nós
contamos por arco de propósito (para o efeito não descrever a autoria), e o preço dessa escolha era
este.

**Cura: a UNIÃO** — amostra-se na grade das cristas **e** nas posições de arco das âncoras que
chegaram. Um pico do 1º zigzag *é* uma âncora, logo sobrevive exatamente. É a mesma regra que o
`ph2d-vec-blend` usa para parear duas formas sem arredondar as quinas de nenhuma.

Três consequências que o integrador deve conhecer:

- A alternância passou a ser **função da posição de arco** (`ridge_wave`) — `k % 2` deixou de
  significar alguma coisa numa lista não-uniforme.
- O braço da alça suave é o **vão até o vizinho**; na grade uniforme é o mesmo `step/3`, byte-idêntico.
- Uma **cúspide entra sem deslocamento** em vez de ser descartada. Descartar tirava uma crista da
  conta em silêncio — era a origem dos "7 verts".
- `walk_to` virou **busca binária** sobre o prefixo somado. Empilhar cruzava `amostras × segmentos`,
  e os dois crescem juntos: o empilhado 128+128 ficou **mais rápido** (0,857 → 0,654 ms).

**O gate `subdividing_the_path_does_not_change_the_wave` mudou de forma**, e a propriedade nova é
mais forte: *cada vértice do caminho magro existe, no mesmo sítio, no picado*. Subdividir só
ACRESCENTA amostras entre as que já havia.

### §9.4 — O chip mostrava `0..1` durante o arrasto (`e38134a1`)

> *"O número que aparece ao arrastar a caixa numérica é de 0 a 1, sendo que os números reais (quando
> solta o mouse) são outros"*

O slider guarda um track `0..1` e o chip está LIGADO a ele. Registados na **identidade**, o chip
repetia o track o gesto inteiro. O canal já existia (`link_slider_number_mapped`); o que faltava era
saber a faixa, e ela só se sabe no frame — daí `seed_effect_ranges`, que corre no `paint`, como o
`set_number_range` dos eixos de fonte já fazia.

⚠️ Converter no **painel** seria a solução errada: ele passaria a guardar a faixa, e haveria **duas
cópias dela**. Divergiriam no primeiro efeito com faixa diferente — que é exatamente o que o Zig Zag
trouxe (`Size` a 100, `Ridges` a 128).

Junto veio `FxParam.integer`: `Ridges` é uma CONTAGEM, então o motor arredonda no `set` (porta única
de escrita — o **documento** guarda o inteiro) e o painel mostra-o sem casas.

### §9.5 — Teto de Ridges 64 → 128, medido

Ordem do Enio. O recurso é o tempo de `cooked()` e o custo é **linear**:

| cristas | 8 | 32 | 64 | 128 | 256 |
|---|---|---|---|---|---|
| ms/cook | 0,019 | 0,104 | 0,219 | 0,475 | 0,902 |

128 custa 0,41 ms. **Não há parede física antes de ~2000** (o `MAX_SAMPLES` de guarda); quem quiser
subir só tem de re-medir (CLAUDE.md §0).

---

## §10 — Um bug ADJACENTE que a pilha expôs (`e38134a1`)

`path_curve_bbox` semeava o min/max com a âncora **CRUA** e depois varria pontos **COZIDOS** — duas
fontes na mesma caixa. Com raio de quina vivo o erro era o canto cortado (pequeno, ninguém viu); com
a pilha, a âncora crua pode cair **fora da forma inteira**, e um caminho aparado ganhava gizmo maior
do que a arte. Agora só o cozido semeia, e **um caminho sem geometria devolve `None`** — a resposta
que o docstring sempre prometeu para "vazio", e que `settle_origins` já sabia consumir.

E o comentário em `space.rs::curve_bbox` que dizia *"aqui é sempre a identidade hoje: o catálogo gera
formas sem raio de quina vivo"* **era verdade até esta feature**. Retirado — um comentário que afirma
o contrário do que o código faz é pior do que nenhum.

⚠️ **O gate deste nasceu vermelho pelo ORÁCULO ERRADO.** Aparar `[0, 0.25]` de um quadrado dá a
aresta de baixo, cuja largura ainda é 40 — e ela **começa na própria âncora crua**, então a semente
errada coincidia com a geometria certa. A fixture não continha o fenómeno
([[feedback_moving_the_law_is_half_the_fix_the_fixture_must_contain_it]]). Corrigido para
`[0.5, 0.75]`, a aresta de cima, a 40 da âncora crua.

---

## §11 — undo/redo dos efeitos: NÃO encontrei wiring em falta ⚠️ ABERTO

Enio reportou *"undo/redo ainda não implementado para efeitos"*. Varri o caminho inteiro — input →
bus → drain → mutação → snapshot → diff → restore — e **não há um "falta chamar X"**:

- `effects` está no `PartialEq` do diff (`undo.rs:345`); o `canonicalize()` só toca o `WorldSnapshot`,
  e o postcard só é usado no **save**, não no undo.
- O restore reinstala a `VecScene` **inteira** (`undo.rs:105,249`).
- A mutação (`fx_bridge_dispatch::apply`, `render_loop/mod.rs:2603`) corre **antes** do
  `post_frame_undo` (`main.rs:502`), no **mesmo frame** em que `any_input_this_frame` ficou `true`.
- Um clique dentro de um painel docado **não** é filtrado (`input_dispatch.rs:2182` é incondicional);
  `handle_editor_key` corre sempre (`keyboard.rs:548`).
- Os únicos escritores de `effects` são a ponte do painel e a cena de smoke. `recook_into` copia
  campo a campo e não lhe toca.

Acrescentei o gate que a classe exige — **`a_path_with_effects_is_a_fixed_point_of_settling`**
(`vec_transform.rs`): assentar origens duas vezes com efeitos na pilha tem de convergir, senão cada
frame produz um passo de undo que ninguém pediu (foi assim que o bug do z-order se manifestou). Ele
está **VERDE**.

**Não afirmo que está consertado.** Só o app vivo separa as duas hipóteses restantes, e elas pedem
correções opostas:

```
PH2D_UNDO_LOG=1 cargo run -p ph2d-host-desktop
```

- Ao clicar **Add Zig Zag**, se **não sair linha nenhuma** → o passo não é registado, e o problema é
  a montante (o `Click` não chega ao drain, ou `sole_path` devolve `None` porque a seleção não é de
  exatamente **um** path). Nesse caso o efeito também não devia ter sido aplicado — vale confirmar
  se ele muda o desenho.
- Se sair **`vec=true`** e o Ctrl+Z mesmo assim não desfizer → há um **passo espúrio a mais** logo a
  seguir, e o 1º Ctrl+Z gasta-se a desfazer o lixo. É a classe que
  `vec_zorder_fixpoint_tests.rs` já apanhou uma vez.

---

## §12 — ⚠️ ESTA LINHA TOCA FOUNDATIONAL (`ph2d-editor-core`)

Até `7fa5f969` a linha vivia inteira em `ph2d-vec-scene` / `ph2d-panel-vector` / `shells/desktop`.
O commit `64a95f4b` **entra na `ph2d-editor-core`**, e o integrador precisa de saber porquê antes de
decidir a ordem (ADR-0107 · DIRETRIZ §1.5.3 — `scripts/foundational-integrate.sh`).

**O que mudou lá:** `NumberInputDragState` ganhou o campo `accum` (apendado no FIM da struct),
`WidgetStore` ganhou `set_number_input_drag_accum`, e o `pointer_move` escolhe a base do Move.

**Porquê:** o scrub lê o valor da caixa de volta como base do Move seguinte, e uma caixa registada
por `link_slider_number_mapped_integer` **arredonda em toda escrita** — o arredondamento estava
DENTRO do laço e o resíduo morria a cada Move (`round(round(v) + d) == round(v)` para `d < 0.5`).
Com `DRAG_RANGE_PX_V = 2500`, uma contagem de `1..128` corre a 0,05 unidades/px e um Move de 3 px
carrega 0,15: **o eixo vertical estava aritmeticamente morto**.

**⚠️ Não é um bug desta linha, e o alcance é o app inteiro.** Toda caixa inteira sofria: Sculpt
radius (faixa 15 ⇒ 0,006/px), BgRemoval Min Px, Color-Eq Tile Grid, Posterize Dither Grain. Chegou
aqui porque as cristas do Zig Zag foram a primeira que o Enio arrastou na vertical.

**Projetado para ISOLAMENTO** (a exigência do CLAUDE.md §0.2 ao criar/tocar foundational):

- O campo é **apendado**, e `NumberInputDragState` **não é serializado** — não há schema a bumpar.
- **Só as caixas que arredondam leem o acumulador.** As contínuas continuam a ler o valor de volta,
  byte por byte como antes: o caminho comum é intocado, e não há um 2º modelo de scrub a divergir.
- O ponto de extensão é uma pergunta ao store (`linked_slider_snap_integer`), que já existia.

**Se outra linha tocar o mesmo arquivo** (`interaction/dispatch/pointer_move.rs` é popular), o
conflito é textual e local — mas confira que o `base` continua a ramificar, porque a mutação que o
neutraliza **não sangrava em nenhum gate** antes deste commit (foi o único sobrevivente da sessão, e
o gate `an_integer_chip_still_travels_on_the_precise_vertical_axis` nasceu dessa sobrevivência).

---

## §13 — O outro achado do mesmo smoke: a caixinha não alternava (`64a95f4b`)

> *"Smooth on/off Rough não consigo ter resultado imediato se apertar seguidamente"*

Este é **da linha**. Uma caixinha é pintada como BOTÃO mas **partilha o id do slider** — e o
`populate` regista sempre um slider ali, porque regista o **teto**, antes de saber que efeito cai na
linha. Então um press na caixinha emite **também** `ValueChanged` do slider, com o track = a posição
**horizontal** do cursor dentro do botão; e essa escrita corria **depois** do flip do Click.

Quem decidia o estado era o *onde* do clique, não o flip — clicar repetidamente no mesmo sítio dava
sempre o mesmo resultado. A pergunta certa (`is_toggle`) já estava no arm do Click; faltava-lhe o
**complemento** no arm do parâmetro. Duas escritas para um fato, e a do track tinha de se recusar.

O gate clica **quatro vezes no mesmo track** (`0.9`, o canto direito) e exige quatro alternâncias.
E tem irmão obrigatório — *um parâmetro contínuo continua a receber o track* — senão a recusa podia
ser escrita larga demais e matar todos os sliders em silêncio
([[feedback_absence_gate_needs_a_presence_sibling]]).

---

## §14 — Estado do undo — ⚠️ AINDA ABERTO

O `PH2D_UNDO_LOG` ficou mais falante durante a caça (`7fa5f969`) e o andaime foi retirado a pedido
do Enio quando o smoke passou (`4daf72cc`) — **cedo demais**: a ronda seguinte precisou dele. Fica
a lição, e o protocolo em §11 continua a ser a forma de o decidir numa corrida.

O que MUDOU desde então: o pivô deixou de reagir aos efeitos (§15.4). Era o único sítio do frame
onde os efeitos e o resto do documento não se comportavam igual perante o registo de passos, e
agora comportam-se. **Não afirmo que isso fecha o relato** — afirmo que fecha a única assimetria
que dois agentes e quatro varreduras conseguiram encontrar.

---

## §15 — A leva de efeitos, e o que ela custou aprender (`2518ea39` → `3d803ebf`)

A pilha existia com dois efeitos. Dois provam um mecanismo, não uma plataforma — e o ADR-0132 foi
construído para que o terceiro custasse **zero painel**. Isso estava por cobrar.

### §15.1 — A promessa foi cobrada, e o número é ZERO

Três efeitos entraram inteiramente dentro de `ph2d-vec-scene`. **Nem uma linha de painel, nem de
shell.** Fora dela só mudou o `PROJECT_SCHEMA` e a tripla do gate, que são consequência do formato
e não da UI. Acrescentar um efeito é hoje: um variant + os braços + uma linha em `KINDS`.

### §15.2 — ⚠️ E três dos que entraram estavam MAUS

O Enio: *"três implementações paupérrimas"*. Estavam, e a causa era **uma só**:

> 238 gates verdes, mutações a sangrar, e **eu nunca tinha renderizado nenhuma delas.** Todos os
> oráculos perguntavam *"o buffer diz o que eu disse que dizia"*; nenhum perguntava *"isto parece
> a ferramenta cujo nome tem"*.

Os três defeitos, e nenhum precisava de olho clínico:

| efeito | defeito | causa |
|---|---|---|
| **Twist** | torcia um "lowpoly" | campo **não-afim** amostrado só nas âncoras |
| **Pucker & Bloat** | *"só aumenta e reduz a escala"* | implementei o meu **palpite** do efeito em vez de ler o que ele é. A Adobe define-o como um PAR (âncoras para dentro **enquanto** os segmentos curvam para fora); com o mesmo fator nos dois é uma escala, que é o gizmo |
| **Repeater** | as combinações não encaixavam | os dois eixos dividiam pela **média** de largura e altura, então `100` nunca encaixa numa forma não-quadrada |

**O gate do Bloat afirmava o comportamento errado** — verde por cima da escala. É a lição do
oráculo, na sua forma mais crua.

### §15.3 — O Twist foi CORTADO, e isso é uma decisão

O sintoma era real e a causa clara. Construí a subdivisão adaptativa que o resolve, e **ela
funcionava** — há gate a provar que partir não move a curva.

O que não funcionava era o **efeito**. Quatro tentativas, cada uma verificada na folha de contacto:
força a crescer com o raio, a decrescer, raio de referência pela média, raio pelo máximo,
subdivisão seis vezes mais fina. **Todas rasgavam** — sobre uma forma com quinas, qualquer queda
radial cria um diferencial enorme ao longo de UMA aresta e o canto chicoteia à volta do corpo.

Isso deixou de ser defeito de código e passou a ser defeito do meu **modelo** do efeito, e não
tenho referência que consiga verificar. **Um item de menu que produz geometria rasgada é pior do
que um item que falta.** A subdivisão e o gate dela ficam no histórico (`fx_warp.rs` antes de
`d63b225a`), para quem o souber especificar.

### §15.4 — O pivô deixou de reagir aos efeitos

`settle_origins` media a bbox **COZIDA** — correto para o gizmo, que abraça o que se vê; errado
para o **pivô**, que é propriedade da identidade do objeto e não da aparência de hoje. Com o
cozido, acrescentar um Trim ou um Repeater deslocava a caixa e fazia esse sistema **escrever no
documento num frame que o utilizador não provocou** — um escritor por-frame que reage a efeitos,
que é a forma de um passo de undo espúrio. Agora mede o autorado (`path_bbox`).

É o único sítio do frame onde os efeitos e o resto do documento não se comportavam igual.

### §15.5 — ⚠️ ACHADO ARQUITETURAL: o Offset Path **não pode** ser um efeito da pilha

`ph2d-vec-scene` é deliberadamente `serde` + `postcard` e mais nada (foi o que obrigou o
`arclen.rs` a existir à mão). Como o `cooked()` é avaliado **dentro** dela, **todo efeito da pilha
herda essa cerca**.

Um Offset correto exige tratamento de quinas e remoção de auto-interseções — isto é, o motor
booleano. Ele tem de ser um **comando de edição**, como as booleanas que já existem, e não uma
entrada na pilha. Melhor saber isto antes de alguém gastar um dia a construí-lo no sítio errado.

### §15.6 — O Repeater, contra o Array do Blender

O modificador do Blender tem **três deslocamentos independentes e acumuláveis**: Relative (fração
da caixa, POR EIXO), Constant, e **Object Offset** (a transformação cumulativa, de onde saem os
arranjos radiais e as espirais). Faltavam-me duas dessas ideias.

O Repeater tem agora **6 parâmetros**:

- `Copies X` + `Move X`, `Copies Y` + `Move Y` — dois eixos ⇒ a **grelha sai de um só efeito**
  (no Blender ela exige empilhar dois Array). A **contagem é o interruptor** do eixo: `1`
  desliga-o, e um toggle separado seria uma 2ª resposta a *"este eixo está ligado?"*.
- `Spin` — cada cópia gira sobre o centro **dela**. A fileira continua fileira e ganha um leque.
- `Orbit` — cada cópia gira em torno do centro do **original**. Sobre uma cópia deslocada isto
  **leva-a** para outro sítio: é o arranjo radial e a espiral, e é o *Object Offset*.

⚠️ **As duas rotações existem porque fazem coisas diferentes.** A 2ª versão substituiu uma pela
outra, e **substituir era o erro** (Enio). Há gate que mede a diferença pelo sítio onde a cópia
acaba — sem ele, uma delas seria um botão morto.

`MAX_FX_PARAMS` 4→**6**, e o `MAX_FX_ROW_PARAMS` do painel com ele. ⚠️ O doc do painel afirmava
*"há gate a exigir que os dois lados concordem"* e **não havia**; agora há
(`the_engine_and_the_panel_agree_on_the_parameter_ceiling`).

Tetos **medidos** (CLAUDE.md §0): 128 por eixo, **1024 no produto** — o teto de um eixo não é o
teto de uma grelha. `32×32` custa 0,66 ms de cozimento. O recurso por medir continua a ser o
**render** dos contornos, não o cozimento.

---

## §16 — ⬛ A MUDANÇA DE MÉTODO, e é o que mais vale levar daqui

`crates/ph2d-vec-scene/tests/fx_look.rs` + `tests/look/mod.rs` — uma **folha de contacto**: uma
linha por capacidade, uma coluna por valor do parâmetro, num PNG.

```
PH2D_FX_LOOK_DIR=/tmp/look cargo test -p ph2d-vec-scene --test fx_look --release -- --ignored --nocapture
```

Preenchimento = a forma que o artista vê · linha fina = a geometria · **cruzes = as âncoras** (a
diferença entre *"a curva está errada"* e *"as âncoras estão no sítio errado"*).

**Sem dependência nenhuma**, de propósito: a crate é deliberadamente `serde`+`postcard`, e um PNG
com blocos deflate **não comprimidos** troca ~40 linhas por zero deps. É o irmão do
`push_look_probe` do Painter.

Ela pagou-se ao primeiro olhar e **duas vezes depois disso**:

1. O Repeater **orbitava** um centro fixo em vez de ladrilhar — invisível em qualquer gate de
   buffer, óbvio em meio segundo de imagem.
2. **A própria sonda estava errada**: preenchia em even-odd, e uma forma auto-intersectada aparecia
   rasgada onde o produto desenha tinta cheia. Quase fui perseguir um defeito do desenhador.
3. Um gate meu acusou a subdivisão de mover a curva 0,44 — mas media distância ao **ponto**
   amostrado mais próximo, com espaçamento 1,25. O oráculo é que estava errado.

⚠️ **A recomendação para quem herdar isto:** um efeito novo entra na folha **antes** de entrar na
tabela `KINDS`. Foi a ausência disto que deixou passar três efeitos maus numa leva só.

---

## §17 — Fila, para depois da integração

- **Chamfer** (tipo de quina — reta em vez de arco; quase de graça sobre o `corner_live`)
- **Texto em caminho** — ficou muito mais barato: o `arclen.rs` que o Trim trouxe é o pré-requisito
- **Offset Path** — ⚠️ como COMANDO, não como efeito (§15.5)
- **Largura variável** · **mais primitivas** · **blend em cadeia** (>2 formas) · **morph vivo**
- **Twist**, quando alguém o souber especificar contra uma referência verificável (§15.3)
