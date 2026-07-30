# Plano — as ferramentas de DESENHO que faltam ao módulo vetorial

> `line/Vector`, 2026-07-29. Nasce de uma **auditoria de cinco agentes** pedida pelo Enio:
> *"No que se refere a desenho vetorial, nossa ferramenta ainda não tem alguma feature importante
> que deve ter para ser considerada um app de fronteira? … Por exemplo: percebo que não temos a
> ferramenta Lápis, nem faca, etc. … Podemos ter um pincel como em flip que aumenta a largura do
> stroke de forma suave onde pintamos?"*
>
> Escopo que ele deu: **ferramentas de desenho, manipulação de curvas, Boolean, Build**. Cada
> afirmação abaixo foi conferida no CÓDIGO (`arquivo:linha`) — os docs deste módulo têm notas
> obsoletas comprovadas, três delas escritas por mim nesta mesma semana.

## §1 — O veredito da auditoria em uma frase

O módulo é **excepcional em MODIFICAR** geometria não-destrutivamente (≈48 Live Shapes · 21 Live Path
Effects · 12 FX raster · Blend · Envelope com Coons+MLS · Contour · Pattern along path) e **quase não
tem como PRODUZIR** geometria à mão. **Todo caminho deste app nasce de cliques discretos** (a caneta)
**ou de um arrasto dimensionado** (as shape tools). Não há lápis, não há pressão, não há faca, não há
tesoura, não há borracha de caminho — e o ponteiro **nem carrega a pressão** até o código do vetor,
embora a shell já a tenha.

E o eixo de PRECISÃO nunca foi pesquisado: o `20_pesquisa_ferramentas_de_artista.md`, que dirigiu as
últimas seis waves, tem **zero linhas** sobre snap, guias, réguas, simetria, estabilizador, medição ou
autotrace.

## §2 — As decisões do Enio (2026-07-29), e o que cada uma fecha

| decisão | consequência |
|---|---|
| **"não haverá boolean recomputado por-frame"** — a D4 do ADR-0108 **MANTIDA** | ⛔ **O boolean VIVO sai do plano.** Nada roda o sweep por frame. O pathfinder da §8 é **destrutivo, edit-time**, e não toca a D4 |
| **Linha de runtime — agora não** | o `[lib]` da shell, os memos em espaço LOCAL e o dirty-tracking ficam **nomeados na §10, fechados a esta rodada** |
| **Width Tool completo — criar** | §5, **com ADR na frente** (é o único item onde a pesquisa do repo diz que não há caminho de prateleira) |
| **W6 (precisão) — implementar** | §9 entra: snap a caminho/interseções, guias e réguas, Mirror vivo |

⚠️ **A rota que a D4 não proíbe, registada para não ser re-descoberta:** união, XOR e
buraco-dentro-da-forma são **exatos e de graça HOJE** pela regra de preenchimento — concatenar
contornos num `BezPath` (o `build_path` já o faz) e escolher `NonZero`/`EvenOdd`, com o winding a
resolver antes do antialiasing. É a receita que a doc do **Rive** dá ao artista, e **nenhum boolean é
computado**. Fica **não construído** porque a pergunta que ela responde ("boolean vivo") foi fechada;
se um dia reabrir, esta é a porta, e a **interseção** não passa por ela (exige clip, e paga o
*conflation artifact* do `vello#49`, cujo caso de falha **é** o caso normal de uma diferença).

## §3 — O que a auditoria mediu, e que decide o TAMANHO de cada wave

**A maior parte do trabalho que falta é fiar o que já existe.** Estes são os ativos enterrados,
todos verificados:

| ativo | onde | estado |
|---|---|---|
| **`fit_hobby_open`** — solver de Hobby, spline interpolante, tridiagonal | `ph2d-vector-doc/src/hobby.rs:99` | **533 LOC, testado, ZERO chamadores.** O doc dele nomeia o consumidor que nunca chegou: *"the Pencil tool smooths a recorded freehand stroke"*, e a decisão de design ao lado (**não** é o least-squares do Schneider, *"the Inkscape default the plan flags as the anti-pattern"*) |
| **`pressure` / `tilt`** | `ph2d-editor-core/src/tool.rs:219-225` | Existem no `CanvasPointer` (*"Apple Pencil is first-class"*); o Painter e o Flip consomem. **Zero ocorrências de `pressure` nas quatro crates `ph2d-vec-*`** — o vetor descarta na fronteira |
| **`pressure_width_factor(pr, min, response)`** | `ph2d-tool-flip/src/params.rs:288` | A curva do Flip, **já calibrada em smoke e aprovada pelo Enio** (defaults 0,05 / 0,5) |
| **`lazy_mouse_step`** | `ph2d-painter-brush/src/stroke/stabilize.rs:83` | Estabilizador: função **livre, pública, `#[must_use]`, pura**. O doc já diz *"Shared by the Space-method stabilizer and the Free Hand capture"* — dois consumidores, **nenhum no vetor** |
| **`merged_segment_fit(prev, mid, next)`** | `ph2d-vec-scene/src/reshape.rs:242` | *A cúbica que passa pelas mesmas tangentes.* O **Simplify chama-a**; o **Delete ignora-a**. É o motor de **duas** ausências (§6) |
| **`nearest_point_on_path`** + **`split_segment`** | `ph2d-vec-scene/src/geometry.rs:277,239` | A tesoura são **estas duas chamadas** |
| **`subtract_all`** | `shells/desktop/src/shape_build.rs:326` | *fonte − união do que está acima* — o **Trim** do pathfinder, que o Build já usa |
| **`Arrangement`** | `ph2d-vec-boolean/src/arrangement.rs:70` | Faces planares de N formas; **já computa toda interseção** (o alvo de snap da §9) |
| **`fx_repeat`** com `spin`/`orbit` | `ph2d-vec-scene/src/effect_params.rs:234` | Multiplica contornos **vivo**; o **Mirror** da §9 é um irmão dele |
| **`VecLabel`** | `shells/desktop/src/label_live.rs` | Guarda uma **relação** e re-deriva a pose por frame, com undo e save de graça. Uma **cota** é exatamente isso |
| **`ph2d-flip-colorize`** | `crates/ph2d-flip-colorize/` | Já faz **raster → região → curva** tolerante a vão. O autotrace tem ~70% do motor no repo, **espalhado** ⇒ segue **G**, fora desta rodada |

⚠️ **E um ativo que NÃO é reuso:** o `WidthProfile` de 4 números (`ph2d-vec-scene/src/width_profile.rs`)
tem **dois consumidores, os dois destrutivos** (`Expand::PowerStroke` e a fiação dele). O próprio
header confessa: *"As alças na linha são um GESTO de canvas… outra wave"*. Ele é **semente**, não
solução — ver §5.

## §4 — W1: A MÃO (o pedido do Enio)

**O que o artista ganha:** desenhar à mão livre, com a largura a responder à pressão suavemente, e com
a mão estabilizada — o gesto que hoje existe em **raster** neste app e não em vetor.

Três peças, e **nenhuma pede matemática nova**:

1. **O lápis** — o gesto grava amostras, decima, e ajusta por **`fit_hobby_open`**. Um 11º `DrawMode`.
   ⚠️ O fitter fala `glam::Vec2` (f32) e a engine nova é `[f64;2]`: **portar o solver para f64 num
   módulo leaf** é mais limpo que criar a aresta de dependência para a crate congelada. O gate do §6
   conta **variantes de enum e campos de struct** (`VectorOp` ≤ 16 · `Vertex` ≤ 5 · `Segment` ≤ 6 ·
   `Region` ≤ 5 · `VertexKind` = 4) ⇒ **nada disto o toca**, mas o port evita a discussão inteira.
2. **A pressão atravessa a fronteira** — o `CanvasPointer` já a traz; o press/drag do vetor passa a
   carregá-la. **Mouse reporta `1.0`**, então nada muda sem tablet.
3. **O estabilizador** — `lazy_mouse_step` fiada no laço de captura + um slider. **Porta única:** a
   MESMA função que o Painter usa; uma segunda suavização divergiria do que o artista já aprendeu.
   ⚠️ **Não confundir com o RDP + Catmull-Rom do Flip**, que é suavização **pós-hoc** e não substitui.

**A largura viva é o que faz disto um pincel** e é a espinha compartilhada com a §5: a pressão escreve
num **perfil de largura VIVO** (não no bake). Ver §5 para onde ele mora.

### ✅ W1c — ENTREGUE: a largura viva (ADR-0145)

A espinha está de pé, e **pela metade que serve às DUAS waves**: crate-folha `ph2d-stroke-width`
(`WidthStops` — a lista de paradas — com o `WidthProfile` de quatro números como FACE dela, e a
redução preset→lista provada **bit a bit**) · componente `VecStrokeProfile` (**zero bump**: registo
38→39, `PROJECT_SCHEMA` fica em **38**) · `power_stroke` passou a consumir paradas · `profile_live`
coze por frame com memo e desenha no z da forma · e **os quatro sliders `W *` AUTORAM**: arrastá-los
arma o perfil na seleção e a fita aparece na hora; o botão virou **Apply Power Stroke** e
materializa, o par exato do Offset. A porta é **uma** (`vec_expand::power_stroke_layers`): o preview
desenha o que ela devolve e o Apply insere o que ela devolve, com gate byte a byte.

Isto **fecha a consequência que o ADR-0145 nomeava** (*"os sliders passam a escrever no perfil do
caminho e o Apply assa o que está lá"*) — antes o preview mostrava uma espessura e o Apply assava
outra. Smoke: **`PH2D_BUILD_SMOKE=41`**.

### ⚠️ W1b.2 — A PRESSÃO: bloqueada por um fato do SHELL, e é decisão de produto

O item 2 acima diz *"o `CanvasPointer` já a traz … mouse reporta `1.0`, então nada muda sem tablet"*.
**Metade disso está errada, e foi MEDIDO:** esta shell **constrói o `PointerEvent` com `pressure: 1.0`
literal** nos seus dois únicos sítios (`input_dispatch.rs`, `source: PointerSource::Mouse`), e o laço
de eventos **não casa `WindowEvent::Touch`** — o único evento do winit que carrega `force`. O
`CursorMoved`, que é o que a shell escuta, **não tem pressão nenhuma no protocolo**. Logo: hoje
nenhum dispositivo entrega pressão a este app, tablet incluído. Fiar a pressão no perfil produziria
um **fio morto** — a feature ficaria correta e invisível, e ninguém saberia se está quebrada.

**As duas saídas foram apresentadas e o Enio escolheu a (1)** (2026-07-30):

1. ✅ **Uma FONTE de largura** (`Uniform | Speed | Pen`) — o modelo do Krita/GP. *Speed* é o único
   que um mouse de facto dirige, então o artista vê a largura viva **hoje**, e a rota de pressão
   fica construída e gateada para o dia do tablet. **ENTREGUE, ver W1d abaixo.**
2. **O caminho do tablet** — casar `WindowEvent::Touch` e levar o `force` até o `PointerEvent`. É
   trabalho de INPUT da shell (não do vetor), afeta o **Flip do mesmo jeito** (o `flip_draw.rs`
   também recebe um `1.0` literal), e não pode ser verificado sem hardware. **Segue aberto**, e
   agora custa **UMA função** (`App::pointer_dynamics`) em vez de uma varredura.

### ✅ W1d — ENTREGUE: a fonte da largura

Fileira **Width** na seção Pencil, e o lápis passou a gravar a **dinâmica** de cada amostra (a
pressão do dispositivo + o carimbo de tempo do evento). Motor em `ph2d-vec-edit::pencil_width`;
o perfil que ele produz é a `WidthStops` do W1c, pendurada **ao vivo** a cada frame em que o traço
está aberto — o artista vê a espessura enquanto desenha, que é a promessa do lápis desde o W1a.

**Três decisões, todas MEDIDAS:**

- **Rápido AFINA** (a convenção de todo DCC), e a velocidade é normalizada **no próprio traço** —
  velocidade absoluta depende do zoom e do rato, e calibrá-la seria um knob que ninguém acerta.
  Consequência de graça: um gesto de velocidade CONSTANTE não pendura perfil nenhum.
- **O filtro e o reamostrador eram a MESMA pergunta.** A 1ª versão suavizava e depois amostrava em
  N pontos — aliasing: o perfil saía não-monotônico num gesto que só acelera, e o extremo era
  perdido entre paradas (faixa efetiva 2,05× de 4,14×). Um filtro **casado** (a média sobre uma
  fatia igual de amostras) responde as duas, e a const do suavizador desapareceu.
- **`STOP_BUDGET = 8`, e a intuição estava invertida:** eu esperava que mais paradas descrevessem
  melhor o gesto, e a medição mostra o erro **crescendo** com o orçamento (cada parada a mais é uma
  fatia com menos amostras, logo menos média). 8 é o maior orçamento com **zero reversões
  espúrias** — a coluna absoluta, contável, num gesto onde a mão só acelera.

⚠️ **`Pen` está construída, gateada, e hoje não muda nada** — o rótulo o diz. Quando o caminho do
tablet existir, é uma linha que muda (`App::pointer_dynamics`, a porta única).

⚠️ **Uma mutação SOBREVIVEU e produziu o gate que faltava:** voltar ao ponto-a-ponto deixava os
onze gates verdes, porque todos afirmam EXTREMOS (*o rápido é mais fino*) e o aliasing põe degraus
no MEIO. O gate novo é *um gesto monotônico dá um perfil monotônico*, com jitter de relógio na
fixture — sem o jitter o ponto-a-ponto acerta, e a fixture não conteria o fenômeno.

**Tamanho: M.** Smoke: desenhar um S com pressão variável e ver a largura acompanhar; com estabilizador
a 0 e a 1, a mesma mão dá traços diferentes.

## §5 — W2: O WIDTH TOOL (aprovado pelo Enio — **ADR na frente**)

**O que o artista ganha:** o Width Tool do Illustrator — **pontos de largura arbitrários** na curva,
adicionáveis, movíveis, duplicáveis e apagáveis, com **perfis salvos**; e o traço fica **VIVO**, não
assado.

**Por que exige ADR:** é o único item da auditoria onde a pesquisa do próprio repo
(`20_pesquisa_ferramentas_de_artista.md:96-118`) diz que **não há de prateleira** — nem kurbo nem Skia
têm largura variável; existe o caminho do Levien (*stroke expansion*, arXiv 2405.00127) e mais nada. E
há uma decisão de representação a registar.

### ⚠️ A decisão que o ADR tem de tomar: ONDE o perfil vivo mora

| rota | preço | veredito |
|---|---|---|
| campo novo no **`StrokeSpec`** | ele é `Serialize` e a `VecScene` vai **EMBUTIDA** no `ProjectState` (o `project.rs` o diz na entrada v10) ⇒ bumpa **`VEC_SCENE_SCHEMA_VERSION` 13→14 E `PROJECT_SCHEMA` 38→39**, e **recusa todo projeto já salvo** | ⛔ |
| **componente ECS `VecStrokeProfile`** | cunha `stable_type_id` próprio (`blake3(NOME)[..8]`), **não move layout posicional de nada** ⇒ **zero bump** | ✅ **recomendado** |

O componente é o padrão que este repo já usou **sete vezes** (`VecOffset` · `VecTextPath` ·
`VecEnvelope` · `VecBlend` · e a família de área da física), e a lei que o justifica está escrita sete
vezes também: **um bump recusa todo projeto já salvo**, e jogar fora trabalho real para evitar um
componente é o trade errado. Semanticamente também casa: um perfil de largura é atributo **por-path**,
que é exatamente o que o `VecOffset` é.

**Conteúdo:** lista `(posição, largura)` arbitrária (o Power Stroke do Inkscape) em vez dos 4 números ·
alças arrastáveis na curva (o `InteractiveState::CurvePoint` é o dispatch 2D compartilhado — o
precedente do repo) · perfis salvos · e o render a variar a largura **sem assar**. O `Expand::PowerStroke`
de hoje **fica** como a porta para quem quer a forma preenchida.

### ⚠️ O que a W1c JÁ entregou desta wave (não reconstrua)

A **representação inteira** e o **motor vivo** já estão de pé (ver §4, W1c): a lista de paradas
arbitrárias existe (`WidthStops`), o componente existe e é salvo/desfeito de graça, o `power_stroke`
já a consome, o cozimento vivo já roda por frame no z da forma, e o par *arrasta-e-vê* / *Apply* já é
o do Offset. **Zero bump** — a decisão da tabela acima foi tomada e executada.

**O que sobra para a W2, e só isto:** as **alças no canvas** (adicionar/mover/apagar uma parada
apontando a curva) e os **perfis salvos** (a lista por nome). O que NÃO sobra é decidir onde o
perfil mora nem escrever um segundo motor: quem o fizer estará a construir a segunda porta que o
ADR-0145 §3 proíbe.

### ✅ W2a — ENTREGUE: as alças no canvas

Pill **Width** (o 12º modo), e uma alça por parada — **fora** da curva, à distância que a fita de
facto tem ali. Arrastar para fora ENGROSSA, para dentro AFINA, ao longo MOVE a parada; arrastar a
partir de um ponto da curva CRIA um ponto de largura; o botão direito sobre uma alça a APAGA (e
abaixo de duas paradas o perfil inteiro sai, o neutro-é-ausência das outras rotas).

É um modo, e não uma alça no Node, pela razão que o Fillet/Chamfer já pagou: estas alças pousam a
milímetros da curva num multiplicador pequeno, ou seja **em cima das âncoras** que o Node existe
para editar. O Illustrator também a faz uma ferramenta (Shift+W).

⚠️ **Um número medido decidiu o gesto.** Inserir uma parada numa lista interpolada por
`smoothstep` **não preserva a forma entre as vizinhas** — o desvio máximo é **13,1% da faixa do
perfil**, e é ESTRUTURAL (o mesmo em todo perfil: é o máximo entre um smoothstep e dois
meio-smoothsteps). Trocar por interpolação LINEAR tornaria a inserção exata e poria um vinco em
cada parada, que é o que o `WidthProfile` recusa desde o 1º dia. **A cura não é a interpolação, é
o gesto:** cria-se um ponto de largura **arrastando** (é o que o Illustrator faz), e um clique que
não moveu nada é desfeito no release — quem arrasta nunca vê os 13,1%, porque a espessura já está
a mudar sob o dedo.

⚠️ E a alça **não escala com a pose**, porque a fita também não: o `bake_xform` transforma pontos e
comprimentos de path e deixa `stroke.width` como está. As duas têm de concordar, e há gate.

#### ⚠️ Correção pós-smoke (2026-07-30) — a ficha mudou-se para a CURVA

Report do Enio: *"linhas muito próximas ou cruzadas, cria-se duas alças (1 em cada segmento
próximo). Mas deveria criar apenas uma alça na linha mais próxima do mouse."*

**A escolha da linha nunca esteve errada** — o `closest_arc` já devolvia o braço mais próximo, e um
press cria **uma** parada (o despacho é único, conferido). Errado estava o **DESENHO**: a ficha ficava
na borda da fita, a `meia-largura × multiplicador` da curva. MEDIDO num grampo de braços a `0,30`
com traço `0,16`: um arrasto que produziu multiplicador `3,75` pôs a ficha em **`y = 0,300`** — o
braço vizinho, ao milésimo. O artista clicava numa linha, a alça nascia na de ao lado; clicava outra
vez no sítio certo, e o hit-test não achava a alça (ela estava na outra linha) ⇒ **segunda parada**.
Duas alças, uma em cada segmento — exatamente o report.

Agora a **ficha fica SOBRE a curva** e uma **haste** sai dela até a borda da fita. *"De que linha é
esta alça?"* deixa de ter resposta errada possível. É o *Width Tool* do Illustrator (o ponto de
largura senta no traçado) e os nós do *Power Stroke* do Inkscape. A largura continua diretamente
manipulável: puxar para longe da curva cresce a haste e engrossa a fita.

⚠️ **A haste ainda pode atravessar a linha vizinha, e isso é honesto** — a fita de facto chega lá.

⚠️ **E a sonda achou um SEGUNDO defeito, independente do reportado:** na forma VIRGEM o 1º gesto
**sequestrava a parada do FIM** em vez de acrescentar uma. A parada criada nasce com o multiplicador
que o perfil já tem ali (para o desenho não saltar), então sobre o neutro a lista continua uniforme
— e o `arm` **remove** um perfil uniforme (o neutro-é-ausência). O `press` devolvia um índice para
uma lista que nunca foi guardada, e o `drag` relia o neutro (duas paradas) editando a de índice 1: a
ponta do traço. MEDIDO: `[(0,1),(1,1)]` virava `[(0,1),(0.241,5)]` — puxar no meio movia o fim do
traço e engrossava toda a metade final. É o **primeiro gesto que qualquer artista faz**. Cura: o
`Grab` carrega a **posição** da parada (não só o índice) e o `drag` reconstrói a lista pela mesma
porta que o `press` usou.

#### ⚠️ 2ª rodada (mesmo dia) — a proximidade é medida à LINHA

*"Ainda não gostei: próximo da linha de cima não consigo clicar na linha cruzada abaixo. O melhor
critério para escolher que segmento atuar é a proximidade do mouse em relação à linha."* (Enio)

Pôr a ficha na curva tirou-a da linha errada, mas **não mudou a pergunta**: o press ainda procurava
no **plano** (*existe alguma ficha a menos de 12 px do rato?*), e isso é **indecidível** entre linhas
mais juntas que o raio — junto a um cruzamento a distância entre elas tende a zero, então a alça de
uma engole sempre o clique dirigido à outra. **Nenhum ajuste do raio salva.**

Agora há **uma** pergunta de proximidade (`closest_arc`, que já escolhe o ramo mais próximo) e a
segunda — *já há parada aqui?* — corre em **ARCO**, sobre o ramo que a primeira escolheu. Duas
linhas que se cruzam estão a milímetros no plano e a **meio traço** uma da outra ao longo do
percurso: `0,509` de fração contra um alcance de `0,031`. É isso que torna a escolha decidível, e é
a mesma grandeza em que a parada vive. Porta única `landing` — o press agarra-ou-cria por ela e o
botão direito apaga por ela.

⚠️ **A 1ª fixture do gate não continha o fenômeno e a mutação SOBREVIVEU:** o X que eu montei tinha
o arrasto a saltar de perna (o alvo estava mais perto da *outra*), então a alça nunca ficava perto
do clique e a busca no plano acertava por acidente. O vão da fixture — **`0,15` contra um raio de
`0,25`** — é o que faz o gate ser um gate; com o vão de `0,30` do irmão a mutação passa. O gate
**afirma a premissa** antes de medir.

⚠️ E duas mutações sobreviventes nomearam dois buracos reais, agora gateados: **o botão direito**
tinha a MESMA ambiguidade (apagava a alça da linha vizinha) e **com duas paradas dentro do alcance**
agarrava-se a primeira da lista em vez da mais próxima.

7 gates novos, 7 mutações, 7 sangram (a que repõe a ficha na borda reproduz o report ao milésimo; a
que repõe a busca no plano reproduz o segundo). LOC: `width_handles_tests.rs` bateu 625 ⇒ a família
das linhas próximas saiu para o filho `width_handles_near_lines_tests.rs`.

Smoke: **`PH2D_BUILD_SMOKE=42`**, passos 11-13.

Smoke: **`PH2D_BUILD_SMOKE=42`**.

### ✅ W2b — ENTREGUE: os perfis salvos

A fileira **Profile**, acima dos quatro sliders: **Uniform · Taper · Both · Bulge**. Escolhe-se a
forma pelo nome e, se for o caso, refina-se nos knobs — a ordem do *Width Profile* do Illustrator
(lá a lista é de miniaturas; um nome só serve se descrever a curva).

**A tabela é o produto inteiro** (`ph2d_stroke_width::PRESETS`): um perfil novo é **uma linha lá** e
zero mudança de UI — o idioma dos presets de gaiola do envelope e o da rack de áudio que se popula
de `KINDS`. Os números foram **medidos** antes de escritos (`measure_width_presets`, o multiplicador
em cinco pontos do arco):

| perfil  | t=0   | 0.25  | 0.50  | 0.75  | 1.00  | fita (8 px × 100) |
|---------|-------|-------|-------|-------|-------|-------------------|
| Uniform | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | — |
| Taper   | 1.000 | 0.775 | 0.550 | 0.275 | 0.000 | 420.0 |
| Both    | 0.000 | 0.500 | 1.000 | 0.500 | 0.000 | 400.0 |
| Bulge   | 1.000 | 1.400 | 1.800 | 1.400 | 1.000 | 1120.0 |

⚠️ **A ponta de largura ZERO era o único valor a tocar a borda do domínio, e foi medida**: a fita
fecha em ponta com 400-420 de área e 252-253 âncoras, todas finitas — a mesma ordem do Bulge. Não há
caso degenerado a tratar.

⚠️ **`Uniform` é a REMOÇÃO, não uma forma**: o `power_stroke` devolve vazio para um perfil uniforme
(ali o comando é o *Outline Stroke*), então esta entrada é a única porta de volta ao traço de
largura única — e há gate exigindo que ela seja a **primeira** e a **única** uniforme.

⚠️ **A linha ACESA é DERIVADA** (`params::active_preset`), nunca guardada: não existe campo *"preset
corrente"* em lugar nenhum, e é isso que faz a fileira apagar sozinha quando o artista arrasta um
slider ou uma alça — aí a forma não é mais nenhuma delas, e dizer que é seria o painel mentindo.

⚠️ **E a comparação é em TRILHO, não em multiplicador** — o ida-e-volta `f32` devolve
`1.0000000298…` para `1.0`, então uma fileira que comparasse multiplicadores ficaria
**permanentemente apagada**: pintada, clicável, e incapaz de mostrar o que o artista acabou de
escolher. `params::preset_tracks` é a porta única, e o `write_preset_to_store` da shell passa por
ela (há arch-gate: o gate de unidade continuaria verde se ele voltasse a converter sozinho).

⚠️ **`segmented3` passou a DELEGAR ao `segmented`** — a aritmética de largura dos dois era a mesma
expressão escrita duas vezes (`(inner_w − gap·(n−1))/n` colapsa em `(inner_w − gap·2)/3`), e duas
respostas para *"onde cada botão senta?"* divergem no dia em que uma ganhar wrap.

⚠️ **E um arch-gate da W1c quebrou sobre produto CORRETO:** ele ancorava na *primeira* `arm(` do
arquivo, e o catálogo acrescentou um armamento acima — *"a primeira ocorrência"* é uma distância
disfarçada, exatamente o que o cabeçalho daquele arquivo manda evitar. Re-ancorado no **ramo** do
arrasto.

LOC: `params.rs` bateu 745 ⇒ o vocabulário do perfil de largura saiu para o irmão
**`params_width.rs`** (a faixa, o mapa do slider, o default, o catálogo), pelo mesmo corte de
`params_pencil`/`params_text`.

Smoke: **`PH2D_BUILD_SMOKE=41`**, passos 10-14.

**Tamanho: G.** Smoke: um traço com 5 pontos de largura, arrastados; o mesmo traço sob Expand tem de
dar a MESMA silhueta (a paridade vivo↔assado é o gate que impede duas respostas).

## §6 — W3: O ALCANCE DO NÓ

Duas ausências que **compartilham um motor que já está no repo** (`merged_segment_fit`):

1. **Apagar um nó preservando a forma.** Hoje `delete_selected_vertex` é literalmente
   `verts.remove(local)` + limpeza de contorno degenerado (`selection.rs:306-316`) — **nenhum handle
   vizinho é tocado**, e a curva morre com o ponto. É a operação de nó mais usada em qualquer app.
   ⚠️ Bônus: o tracker do Inkscape (issue #5077) admite que o refit deles *"fica muito chato"* —
   fazer o nosso melhor é **diferencial**, não paridade.
2. **Arrastar o SEGMENTO.** O `enum Part` (privado, `ph2d-vec-edit/src/lib.rs:57`) tem `Anchor` ·
   `In` · `Out` · `Radius` — **não há `Segment`**, e pressionar sobre a curva **INSERE um nó**. Não se
   pode reformar uma curva sem alterar a topologia dela, que é o gesto que Illustrator e Inkscape
   documentam como o normal.

E **a escala da seleção**, que é o que faz o app não parecer de fronteira: o marquee **exige Shift**,
**substitui** em vez de somar, e vê **um path só**; não há **lasso**, `Tab`, select-all de nós, select
por tipo, select de sub-caminho, nem **X/Y numérico do nó**. Trabalhar uma forma de 40 nós é
clique-a-clique.

⚠️ **Editar nós de VÁRIAS formas é ausência POR CONSTRUÇÃO** (o `selected_verts` pertence a um
`selected` único) ⇒ **G**, fica **fora** desta wave e nomeada aqui.

**Tamanho: M.** Smoke: apagar o nó do meio de um arco e a curva ficar; arrastar o meio de um segmento
e a topologia não mudar; marquee sem Shift, aditivo, sobre duas formas.

### ✅ W3a — ENTREGUE: as duas operações de nó

**Apagar preserva a forma.** `Delete` era `verts.remove` cru — a curva morria com o ponto. Agora os
handles dos vizinhos são re-ajustados para que a cúbica que sobra passe por onde as duas passavam.
⚠️ **Porta única `dissolve_vertex`**: o Simplify (que escolhe QUAL nó sacrificar) e o Delete (onde o
artista escolhe) chamam a MESMA função — o motor `merged_segment_fit` já estava no repo, servindo só
metade dos consumidores, e a divergência aparecia como *"o Simplify preserva a forma e o Delete
não"*. MEDIDO sobre um arco de 2×0,75: desvio **0,0799** (11% da altura) contra **0,5875** da
remoção crua (78% — a curva vira quase a corda). Fosso de **7,4×**; a barra do gate é 0,15.

⚠️ **Limite honesto, achado pela sonda:** com as tangentes das duas PONTAS paralelas nenhuma cúbica
alcança o ápice (todos os pontos de controle ficam na mesma reta) e o refit degrada para a corda —
o `det` do sistema 2×2 é zero e o fallback é o default. É o mesmo limite do Illustrator, e está
escrito no gate porque a **1ª fixture da sonda era exatamente esse caso**: os dois caminhos mediram
`1,0000` e a mutação não sangraria.

**Arrastar o SEGMENTO** (`Part::Segment`). Pressionar sobre a curva **inseria um nó**, então não
havia como reformar um trecho sem mexer na topologia — o gesto que Illustrator e Inkscape
documentam como o normal. ⚠️ **A inserção não se perdeu: mudou de ferramenta.** É a divisão do
Illustrator — a seta branca (Node) **reforma**, a Pen **acrescenta âncora** —, e o press da Caneta
já inseria no mesmo hit-test. Há gate exigindo que ela continue a inserir: mover o gesto teria
removido a capacidade em silêncio.

⚠️ **A distribuição é EXATA, não uma aproximação.** Uma cúbica é linear nos pontos de controle, e a
solução de norma mínima de `B₁ΔP₁ + B₂ΔP₂ = delta` devolve `ΔC(t) = delta` **por identidade
algébrica**. MEDIDO: pior erro **2,0e-16** sobre `t ∈ [0.05, 0.95]`, âncoras intocadas, contagem de
vértices intocada. O `t` é clampado nas pontas porque ali `B₁` e `B₂` vão a zero juntos — nenhum
movimento de handle move a curva NA âncora, que é o que uma âncora é.

⚠️ **A fixture do gate do editor teve de ser AMPLIADA 100×**: com `px_to_world = 1` o raio de
captura é 10 unidades de MUNDO, e um arco de 2 unidades cabe inteiro dentro dele — o press no meio
agarrava um handle do vizinho. É a cicatriz que o `node_hit_tests` já pregava (*"a escala é o
fenômeno, não decoração"*), e ela reincidiu.

8 gates novos (4 de motor + 4 de costura), **4 mutações, 4 sangram**. Nenhum schema, nenhum contrato
congelado. API pública nova: `ph2d_vec_scene::{dissolve_vertex, reshape_segment, point_on_segment}`.

Smoke: **`PH2D_BUILD_SMOKE=43`**.

**Aberto da W3 (W3b):** a **escala da seleção** — marquee sem Shift/aditivo, lasso, `Tab`,
select-all de nós, select por tipo, sub-caminho, X/Y numérico do nó.

## §7 — W4: OS CORTES

**Tesoura** (`nearest_point_on_path` + `split_segment` — **P**) · **borracha de caminho** (a matemática
por arco do `fx_trim`, destrutiva e local) · **faca** (o `Arrangement` cobre fechadas; ⚠️ **abertas
exigem rota nova** — o motor devolve `Error::NonClosedPath` e a nossa política é `Closing::Always`) ·
**Join como comando de seleção** (o `merge_path_into`/`weld_junction` já fundem; falta o botão e a
regra de seleção) · **Average** de nós (3 linhas sobre `selected_verts`) · e o botão do
**`reverse_path`**, que existe com **um chamador interno e nenhum id de UI**.

**Tamanho: M.** Smoke: cortar uma forma fechada em duas abertas com a tesoura; a faca a atravessar
três formas de uma vez; Join a fechar duas pontas.

## §8 — W5: O PATHFINDER BARATO (edit-time, não toca a D4)

Temos **4 das 10** operações (`Union`/`Intersect`/`Subtract`/`Exclude`). Quatro das seis que faltam são
baratas sobre primitivas que já existem:

| op | como | tam. |
|---|---|---|
| **Minus Back** | a fatia de z **invertida**; o fold já é `base − (∪ resto)` | **P** |
| **Trim** | por fonte, subtrai a união do que está **acima** — é o `subtract_all` do Build, verbatim | **P–M** |
| **Crop** | `Intersect` com a forma do topo + descartar o topo. Zero geometria nova | **P–M** |
| **Merge** | Trim + classe de equivalência por `fill` — e `VecPath.fill` já deriva `PartialEq` | **M** |

**As duas caras ficam nomeadas e fora:** **Divide** exige a varredura **N-ária única**
(`Topology::from_paths`, que existe no crate e o nosso `arrangement.rs:36-42` já nomeia como o escape
hatch; via `Arrangement` são `2^N` regiões a ~140 µs ⇒ **segundos** no teto de 16 formas) e **Outline**
exige saída de caminho **ABERTO**, hoje estruturalmente impossível (`compound_from` crava
`closed: true`; a conversão recusa < 3 vértices).

⚠️ **Enquadramento:** o **Rive não tem boolean nenhum** — nas 4 ops que temos já estamos à frente da
referência do ADR-0108; o gap é contra Illustrator/Figma/Affinity/Inkscape.

**Também nesta wave, e é higiene de robustez:** o `linesweeper::Error` é **engolido**
(`binary_op(...).ok()?`, `lib.rs:147`) ⇒ **falha do sweep e resultado legitimamente vazio são
indistinguíveis**, num crate que se autodeclara *early beta*, e o artista vê o mesmo nada. Propagar +
toast: **P**. O núcleo não tem **um único gate** de degeneração (tangência, área zero, NaN,
auto-interseção) — e a cura já está escrita duas portas ao lado (`the_offset_never_takes_the_app_down.rs`,
`probe_offset_fine_sweep.rs`, `robust_tests.rs`): **M**.

## §9 — W6: PRECISÃO (aprovada pelo Enio)

**O motor de snap existe, é bom e ESTÁ ligado** — módulo puro com eixos X/Y **independentes** (a lei do
Figma), a grade universal do editor injetada por closure, ativo no press da caneta, no arrasto de
âncora, nas shape tools, no translate e no scale do gizmo. **O que falta é a LISTA DE ALVOS:** o
`collect_targets` empurra exactamente **a âncora de cada vértice** e os **9 pontos-chave da bbox**.

| item | o que falta | reuso | tam. |
|---|---|---|---|
| **Snap a CAMINHO** | pousar um nó **sobre** uma curva — reivindicação **2D**, não por-eixo | `nearest_point_on_path` | **M** |
| **Snap a INTERSEÇÕES** | expor os cruzamentos + cachear por gesto | `Arrangement` já os computa | **M** |
| **Guias e réguas** | **ZERO hoje** (o único "guide" no repo é o caminho-guia do pattern) — guia arrastável, persistida, 3ª classe de alvo, régua nas bordas, origem móvel | `VecLabel` é o molde de componente derivado | **G** |
| **Mirror / simetria VIVA** | espelho ao desenhar; hoje só há **Flip H/V destrutivo** | o `fx_repeat` já multiplica com `spin`/`orbit` ⇒ um `Mirror` na pilha de LPE. É o que a Illustrator shipou em 2021 como *live symmetry* | **M** |
| **Rótulo de distância** nos smart guides | a linha-fantasma já existe (e é melhor que a média: segmento **entre** os pontos, não linha infinita) | `VecLabel` | **M** |
| **Snap a SPRITES** | nenhum competidor tem, porque nenhum mistura raster e vetor na mesma árvore — **nós misturamos** (ADR-0110) | a bbox já está na shell | **P** |

**Fora desta wave, nomeados:** grade **perspectiva** (**G**) · grade **polar** (10º `GridKind`, **M**) ·
os *planos* da isométrica · **Transform Again** (⚠️ **`KeyD` já é o boolean Subtract** — não pode nascer
com o atalho do Illustrator) · **Measure tool / cotas** (**M** sobre `VecLabel`) · **autotrace** (**G**,
motor espalhado).

## §10 — W0: A HIGIENE (defeitos que a auditoria achou, não features)

Barata, e paga-se sozinha dentro da W1:

1. ⚠️ **O memo do `fx_live` não inclui a GEOMETRIA** (`p.ops == ops && p.w == w && p.h == h`) e
   `fx_live_tests.rs` **não tem um único gate de invalidação de memo** (18 testes, todos sobre
   resolução de ops/cores/hit). Translação é correta **por construção** (o rect é recomputado e o
   conteúdo é idêntico); o caso alcançável é **mudar a cor do fill** de uma forma filtrada, que
   mantém os pixels velhos. **Repro antes de fix.**
2. ⚠️ **`has_derived_verts` tem UM consumidor de produção** (o press das ferramentas de quina,
   `input_dispatch.rs:3448`) ⇒ arrastar âncora de uma **Live Shape** no modo Node talvez seja aceito e
   **revertido em silêncio** pelo recook do frame seguinte. **Repro antes de chamar defeito** — eu vi
   a ausência de chamada, não o sintoma, e a diferença importa.
3. ⚠️ **O `cooked()` roda DUAS vezes por forma preenchida por frame** (`build_bezpath` +
   `build_fill_bezpath`, ambos por `build_path` → `path.cooked()`) e **não memoiza**: clona e roda
   `run_stack`. Uma forma com efeitos coze a pilha inteira duas vezes por frame.
4. ⚠️ **A bench de encode regrediu 1,66× sem ninguém notar** — o spike de 2026-07-05 registra 10k
   formas em **0,77 ms** (ADR-0108 §5 apoia o kill-criterion de N=10.000 nele) e hoje mede **1,278**.
   A bench é `#[ignore]` e **nenhum gate pina o número**. Candidato de mecanismo: o item 3.
5. Comentário que **contradiz o código shipado** (`ph2d-vec-edit/src/lib.rs:295` diz que a alça de raio
   é do modo Node; `:330` diz que não é mais) · o segmentado de tipo de vértice publica só o kind do
   **primário**, então seleção mista mostra um dos três como se fosse o todo.

**Tamanho: P**, exceto o gate de razão da bench (**P–M**).

### ✅ W0 FECHADA (2026-07-29) — e a auditoria errou em dois pontos, os dois corrigidos

| item | veredito | commit |
|---|---|---|
| 1 · memo do `fx_live` sem geometria | **DEFEITO REAL, curado.** A chave era `(pilha, w, h)` ⇒ mudar a cor do fill de uma forma filtrada **não mudava a tela**. A decisão virou porta única headless (`fx_live_memo::job_for` → `FxKey`) e carrega o que é DESENHADO. A translação fica FORA de propósito (pan não re-cozinha). 7 gates, 3 mutações | `5ad38ee27` |
| 2 · `has_derived_verts` com um consumidor | **FENÔMENO REAL, mecanismo ERRADO na nota.** O `recook_into` **não roda por frame**: um nó arrastado numa Live Shape sobrevive até a próxima **edição de parâmetro**. Curado com a política que o par Fillet/Chamfer já tinha (congelar a receita no press), pela porta nova `PenTool::node_edit_hit_at` — porque o retorno do press **não distingue** *agarrou um vértice* de *selecionou a forma*. ⚠️ Os hosts de RELAÇÃO ficam ABERTOS e nomeados no `corner_handles.rs` (congelar não serve ali; a cura é RECUSAR, decisão de produto) | `d64e0…` |
| 3+4 · `cooked()` 2× por forma · bench regredida 1,66× | **UM defeito, não dois.** O `draw_path` construía o caminho completo **incondicionalmente** e o jogava fora numa forma só-preenchida, e cada construção COZIA. Medido: **10k formas 1,323 → 0,901 ms** (1,47×). ⚠️ O resíduo contra os 0,77 do spike (**1,17×**) fica NOMEADO, não atribuído | `43b1c…` |
| 5 · comentário mentiroso · seleção mista | **OS DOIS reais.** A leitura devolvia o tipo do vértice PRIMÁRIO e o painel afirmava-o sobre a seleção inteira ⇒ `SelectedKind::{Uniform,Mixed}`, e misto **não acende chip** (com os chips ainda VIVOS — retipar é o gesto que sai do misto) | `dd0828c98` |

⚠️ **E um gate de perf nasceu não-discriminante e foi REESCRITO:** a 1ª versão do orçamento de encode
era uma **razão de tempo** entre a mesma geometria só-preenchida e só-traçada, e a mutação
**SOBREVIVEU** — medido com o defeito reinstalado, `fill 4,862 ms` contra `stroke 5,255 ms` = 0,93×.
Um encode de FILL e um de STROKE não fazem o mesmo trabalho no Vello, e a diferença é maior que a
construção inteira que o gate queria isolar. O oráculo virou o **NÚMERO** de construções e de
cozimentos (contadores `#[cfg(test)]`), que é exato e é literalmente a propriedade em questão.

## §11 — Fora desta rodada, por decisão, com o preço nomeado

- **A linha de RUNTIME** (*"agora não"*): a shell é **`[[bin]]` sem `[lib]`** e todo produtor vivo é
  módulo **privado do binário** ⇒ um jogo que embarque o PH2D **não alcança** o cozimento
  não-destrutivo. E **todo memo de geometria é chaveado no MUNDO**, que é o que a animação muda:
  medido pelas sondas do repo, em release, um **Contour de 16 anéis animando custa 12,25 ms/frame**
  (74% do quadro, sozinho) e 32 anéis **23,58** (não cabe); offset 0,40–1,07 ms/forma/frame; silhueta
  até 1,78; **pattern não tem memo**. A cura é local (memoizar em espaço **LOCAL** e assar o afim na
  saída — para similaridade offset e silhueta **comutam** com o afim) mais o **dirty-tracking** que o
  ADR-0108 D3 chama de *a* alavanca e que nunca foi construído.
- **Boolean vivo** (D4 mantida) — a rota por regra de preenchimento está descrita na §2 para não ser
  re-descoberta como novidade.
- **Divide · Outline · edição de nós multi-forma · autotrace · grade perspectiva** — todos **G**, todos
  nomeados no lugar onde a wave que os tocaria os encontraria.
- **Vetor → collider de física não existe** (zero referência a `VecPath` em `ph2d-physics-ecs`) e a
  **ADR-0063 é letra morta** — a ADR-0131 §12 a rejeita explicitamente por estar amarrada ao
  vector-runtime que a 0108 aposentou.

## §12 — A ordem recomendada

**W0** (higiene, e desarma armadilhas para as seguintes) → **W1 A MÃO** (o pedido do Enio; todas as
peças existem) → **W2 O WIDTH TOOL** (ADR primeiro; compartilha a espinha do perfil vivo com a W1) →
**W3 O ALCANCE DO NÓ** (as duas ausências que dividem um motor pronto) → **W4 OS CORTES** → **W5 O
PATHFINDER BARATO** → **W6 PRECISÃO**.

⚠️ **Estado da linha:** `line/Vector` está **FECHADA**, com **sete** handoffs a compartilhar
`PROJECT_SCHEMA` 38 e nenhuma integração autorizada. Nada deste plano começa sem ordem explícita do
Enio — e se a W2 tomar a rota do componente ECS (§5), **nenhuma destas waves bumpa schema**, o que as
mantém fora da disputa de número com as outras linhas.
