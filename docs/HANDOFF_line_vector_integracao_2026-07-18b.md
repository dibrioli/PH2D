# HANDOFF de INTEGRAÇÃO — `line/Vector`, sessão de 2026-07-18 (a PILHA de efeitos)

**Para:** o agente integrador (DIRETRIZ §1.5.3–1.5.4), quando o Enio mandar.
**Estado:** ✅ linha **fechada e verde**, 15 commits sobre a `main`. **NÃO integrei e NÃO pushei** —
a linha fecha, entrega o handoff e para (CLAUDE.md §0.7).

> ⚠️ **Pendente de SMOKE do Enio** — `PH2D_BUILD_SMOKE=13`. Ver §5.
>
> ⚠️ **O smoke de 2026-07-18 já correu e derrubou quatro coisas** — o layout em card, a escala do
> `Size`, o empilhamento de dois ZigZags e o readout do chip. Tudo corrigido; ver **§9**. O item
> `undo/redo` **fica ABERTO** e o §11 diz porquê e como o decidir numa corrida.

---

## §1 — O que entra

| SHA | O quê |
|---|---|
| `19383f48` | **ADR-0132** — a decisão de arquitetura do LPE |
| `e5e40aa6` | **fix**: a alça de raio pergunta se a geometria é DERIVADA (bug vivo, 3 objetos) |
| `db50c236` | **feat**: a PILHA + o motor de arco + o **Trim Path** |
| `6f599cf1` | **feat**: a cena de smoke `PH2D_BUILD_SMOKE=13` |
| `1d85fddb` | **docs**: este handoff + a fila do handoff de continuação parou de mentir |
| `b6e66db5` | **fix**: o draw-on da cena RECOMEÇA (o smoke reprovou o TEMPO, não o Trim) |
| `e5992c4b` | **feat**: a **seção Effects** no painel — o artista alcança a pilha |
| `130cde9e` `ea46a9b9` | **feat**: **Zig Zag / Roughen**, o 2º efeito — e a prova da promessa |
| `662e1f48` | **refactor**: a seção é **dirigida pela TABELA** — o próximo efeito custa zero painel |
| `ae4bfd69` | **docs**: o handoff conta o que a promessa do ADR custou de facto |
| `c0d69ab8` | **feat**: cada efeito num **CARD** (nome + ordenar + olho + apagar) e o `Size` **relativo** |
| `e38134a1` | **fix**: o 2º ZigZag ondula **sobre** o 1º; o chip mostra o número do **documento** |

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

**Também vale conferir o fix da alça de raio** (`e5e40aa6`), que é independente: num **filho de
envelope** (`PH2D_BUILD_SMOKE=11`), no modo **Node**, as alças de raio **não devem mais aparecer**.
Antes apareciam, funcionavam, e o raio sumia no frame seguinte.

---

## §6 — Gates rodados nesta árvore

- `cargo check --workspace --all-targets` ✅ (o campo novo em `VecPath` muda o layout postcard de tudo)
- `cargo test` das 4 crates tocadas (`ph2d-vec-scene` · `ph2d-panel-vector` · `ph2d-host-desktop` ·
  `ph2d-editor-core`) ✅ **1943**, 0 falhas
- `cargo clippy --workspace --all-targets` ✅ **0 warnings** · `cargo fmt` **antes** de medir LOC
- ⚠️ Os arch-gates de arquivo (`no_magic_numeric`, LOC cap) moram na **`ph2d-editor-core`** e **não**
  rodam com `cargo test -p` de outro crate. A linha pagou esse pedágio **três** vezes — a última
  neste commit final, e o gate apanhou um `100.0` cru a caminho do commit.

**Schema:** `VEC_SCENE_SCHEMA_VERSION` 8→**10** · `PROJECT_SCHEMA` **18→20**, com a tripla do gate de
acoplamento atualizada para `(20, 8, 10)`. ⚠️ **Se outra linha bumpar o `PROJECT_SCHEMA` na mesma
jornada, o valor certo não está em nenhum dos dois lados do conflito: ele se CONTA**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). De passagem, a narrativa do gate ganhou
o **v18** que ninguém tinha acrescentado (a UNIDADE do `width` do Flip, `cb42c9a2`).

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
