# ARQUIVO — 25_plano_ferramentas_de_desenho.md (história, 1191 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`25_plano_ferramentas_de_desenho.md`](../../../Vector%20Module/25_plano_ferramentas_de_desenho.md) em 2026-08-18, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-66,103-122,166-195,630-633,738-741,856-864,1068-1074,1193-1222,1362-1493` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

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

### ✅ W1c — ENTREGUE: a largura viva (ADR-0148)

A espinha está de pé, e **pela metade que serve às DUAS waves**: crate-folha `ph2d-stroke-width`
(`WidthStops` — a lista de paradas — com o `WidthProfile` de quatro números como FACE dela, e a
redução preset→lista provada **bit a bit**) · componente `VecStrokeProfile` (**zero bump**: registo
38→39, `PROJECT_SCHEMA` fica em **38**) · `power_stroke` passou a consumir paradas · `profile_live`
coze por frame com memo e desenha no z da forma · e **os quatro sliders `W *` AUTORAM**: arrastá-los
arma o perfil na seleção e a fita aparece na hora; o botão virou **Apply Power Stroke** e
materializa, o par exato do Offset. A porta é **uma** (`vec_expand::power_stroke_layers`): o preview
desenha o que ela devolve e o Apply insere o que ela devolve, com gate byte a byte.

Isto **fecha a consequência que o ADR-0148 nomeava** (*"os sliders passam a escrever no perfil do
caminho e o Apply assa o que está lá"*) — antes o preview mostrava uma espessura e o Apply assava
outra. Smoke: **`PH2D_BUILD_SMOKE=41`**.

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
clique-a-clique. *(Os seis fecharam: `Tab`/Ctrl+A/Subpath/Same e a escala nesta §6, o laço na W6.5,
o X/Y do nó na **W6.9**.)*

~~⚠️ **Editar nós de VÁRIAS formas é ausência POR CONSTRUÇÃO** (o `selected_verts` pertence a um
`selected` único) ⇒ **G**, fica **fora** desta wave e nomeada aqui.~~ ✅ **FECHADO (2026-08-10)** —
ver *"W6.4 — a seleção de nós ganha DONO"* no fim desta §6.

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

#### ⚠️ *"Fica um pouco diferente — esse é o esperado?"* (Enio, smoke da 43) — SIM, e aqui está o preço

Ele apagou o nó do meio de um arco e o resultado ficou **mais pontudo** que o controle. É esperado,
e a razão não é o ajuste: **um nó carrega informação**. Fixadas as duas âncoras e as duas direções
de tangente, sobram **2 graus de liberdade** (os comprimentos das alças) para reproduzir uma curva
que tinha 3 nós — alguma mudança de forma é matemática, não defeito.

O que se pode perguntar é *quanto* da mudança é evitável, e a sonda mediu as quatro respostas
(`measure_node_reach.rs`, arco de 2,0 × 0,75):

| ajuste | desvio | o que custa |
|---|---|---|
| remoção CRUA (o de antes) | **0,5875** | a curva morre com o ponto |
| **ATUAL** — passa pela âncora, tangentes fixas | **0,0782** | nada (forma fechada, sem iteração) |
| Schneider — LSQ + reparametrização, 8 iterações | 0,0687 | um fitter **iterativo** no gesto |
| piso teórico — tangentes **LIVRES** | 0,0358 | a tangente da ponta gira **8,4°** |

⚠️ **O ajuste atual está a 12% do piso alcançável com as tangentes preservadas**, e comprar esses
12% custa um fitter iterativo — **medido e recusado**. ⚠️ E a outra metade (0,069 → 0,036) exige
**girar a tangente das pontas**, o que num vértice `Smooth` propaga para o segmento **ANTERIOR**:
apagar um nó mudaria a forma de um trecho que o artista não tocou. É por isso que Illustrator e
Inkscape preservam as tangentes, e é a mesma decisão aqui.

⚠️ **Um LSQ ingênuo é PIOR que o que shipa** (0,1675 contra 0,0782) — sem a reparametrização de
Newton o alvo está mal parametrizado, e o "melhor ajuste" ajusta a coisa errada. Fica escrito para
ninguém trocar o fit por um LSQ direto achando que melhora.

Smoke: **`PH2D_BUILD_SMOKE=43`**.

### ✅ W3b — ENTREGUE: a escala da seleção

A queixa era operacional: *"trabalhar uma forma de 40 nós é clique-a-clique"*. Seis alcances,
todos sobre a mesma lista de nós.

**O retângulo deixou de exigir Shift.** Ele o exigia — e o Shift é o modificador de **adição** em
todo app de desenho, então quem quisesse somar nós não tinha tecla e quem quisesse só a caixa tinha
de descobrir uma. Agora: arrastar do vazio abre a caixa, **Shift SOMA**, sem Shift substitui, e um
clique nu (caixa de tamanho zero) **desseleciona** — sem essa última metade não haveria como largar
uma forma a não ser clicando noutra.

⚠️ **O ramo corre ANTES do `on_press_node`, e a ordem é load-bearing:** ele desseleciona quando não
acerta nada, então um retângulo aditivo aberto depois dele somaria a uma seleção que acabou de ser
apagada. A pergunta *"o press acerta alguma coisa?"* vai à porta que já existe (`node_edit_hit_at`),
e não a uma segunda busca que discordaria da que o press usa.

⚠️ **E metade do *"o marquee vê um path só"* fechou:** a preferência pelo caminho selecionado era
**incondicional**, então arrastar a caixa sobre OUTRA forma mirava a selecionada, apanhava zero nós
e devolvia seleção vazia — o artista via o retângulo passar por cima dos nós e nada acender. Agora
a preferência só vale se a caixa de facto apanhar o selecionado. ~~A outra metade (nós de
**várias** formas ao mesmo tempo) continua ausência **por construção**~~ — e ela **FECHOU** na
W6.4, abaixo: a eleição de um caminho pelo marquee morreu junto com a ausência que a exigia.

**`Tab` / `Shift+Tab`** percorre os nós (e dá a volta nos dois sentidos); **`Ctrl+A`** apanha todos.
⚠️ O Tab **substitui** a seleção: percorrer é olhar um de cada vez, e somar ao andar tornaria a tecla
um *select all* lento.

**Dois botões na seção Vertex** — **Select Subpath** (o contorno que a seleção toca; num compound é
o que separa *este buraco* de *a forma inteira*, distinção que o `Ctrl+A` não faz) e **Select Same**
(todos os nós do tipo do primário — o gesto que transforma *"afiar as 12 quinas desta estrela"* de
doze cliques em dois). São **botões e não atalhos** porque um atalho que ninguém descobre é uma
feature que não existe; o `Ctrl+A` fica na tecla porque esse o artista tenta sozinho.

⚠️ **Nenhum dos dois abre passo de undo:** eles mudam QUEM está selecionado, e a seleção não é
estado de documento — uma linha na fila que o Ctrl+Z não teria o que desfazer.

10 gates novos (6 de motor + 1 de seam + 4 arch-gates de shell), **6 mutações, 6 sangram**. Nenhum
schema, nenhum contrato congelado. Smoke: **`PH2D_BUILD_SMOKE=43`**, passos 9-15.

**Aberto, e nomeado:** ~~o **lasso** (a caixa cobre o caso comum; o laço quer captura de polígono +
overlay próprio — wave dela)~~ ✅ **FECHADO pela W6.5** (no fim desta §6) · ~~**X/Y numérico do nó**
(é *precisão*, e cai naturalmente na **W6**, que é a wave da precisão)~~ ✅ **FECHADO pela W6.9**
(no fim da §9) · ~~e **editar nós de várias formas ao mesmo tempo**, que segue **G** e
por construção~~ ✅ **FECHADO pela W6.4** (abaixo) — e era ele o **pré-requisito do lasso**: um laço
que varre os nós de duas formas não significa nada enquanto a seleção só souber guardar os de uma.

### ✅ W6.4 — ENTREGUE: a seleção de nós ganha DONO

`selected_verts: Vec<usize>` (índices planos dentro de um `selected` único) virou
**`Vec<(VecPathId, usize)>`**. Não é uma feature a mais: é a **remoção de três casos especiais**
que existiam só para conter a ausência.

**Medido pela porta do produto ANTES de uma linha ser escrita** (sonda `multi_probe`, duas formas
lado a lado):

| gesto | apanhava | devia |
|---|---|---|
| caixa sobre AS DUAS | **4 de 8** | 8 |
| somar B a A (aditivo) | **4** (trocava de alvo) | 8 |
| Shift+clique em A, depois em B | **1** (o 2º substituía) | 2 |
| nudge depois da caixa | movia **4 de 8** | 8 |

**Os três casos especiais que MORRERAM**, cada um com a frase que o justificava:

- *"somar só vale dentro do MESMO caminho"* — o `box_select_with` elegia UM caminho (o
  selecionado, ou o de mais nós na caixa) e SUBSTITUÍA. Com o dono no par não há eleição: a caixa
  apanha os nós de todas as formas que cobre, e as três perguntas que a eleição obrigava a
  responder (*quem é o alvo? · a caixa apanhou o selecionado? · é o mesmo caminho?*) saíram do
  corpo da função.
- *"um vértice de OUTRO path TROCA o alvo"* — o `toggle_vert_at`. Agora soma; o **primário** segue
  o último tocado (é ele que o painel de estilo edita) e a forma entra na seleção de OBJETO em vez
  de a substituir.
- *"a vertex in the multi-selection (**selected path only**)"* — o overlay, que tinha o defeito
  escrito no próprio comentário: um índice sem dono só podia falar da forma primária, então os nós
  escolhidos das outras eram desenhados como **não-escolhidos**. O artista via metade da sua
  seleção apagar-se ao tocar a segunda forma, com o motor a mover as duas corretamente.

⚠️ **A metade que carrega a wave não é a CONTAGEM, é o ESPAÇO.** Somar nós de duas formas falha de
modo visível. Já mover as duas com um único `delta_to_local` **compila, roda e deforma em
silêncio**, com a contagem certa o tempo todo: a conversão mundo→local é POR FORMA (ADR-0111), e a
forma escalada 2× andaria o dobro — a seleção se desmontaria sob o dedo. O `nudge` já convertia por
forma; o **arrasto** e o **Average** não, e os dois eram correção:

- o arrasto em grupo descia o delta uma vez, ao local da forma agarrada (bastava, porque o grupo
  não podia sair dela);
- o **Average** mediava coordenadas **locais**, e a média de frames distintos não significa nada.
  Agora o centroide é do MUNDO — ⚠️ e com **uma** forma o resultado é o MESMO que sempre foi, não
  por sorte: um mapa afim comuta com o centroide (`xf⁻¹(média(xf(aᵢ))) = média(aᵢ)`).

⚠️ **E a wave CRIOU uma exigência:** o `box_select` passou a respeitar **escondido/travado**.
Enquanto ele elegia um caminho, a falta do `is_pickable` quase nunca era observável; apanhando
todas as formas cobertas, uma invisível entraria na seleção em silêncio — e o Delete seguinte
apagaria nós que ninguém vê, que é literalmente o modo de falha que o comentário antigo usava para
justificar a eleição.

**Alcance da mudança:** `Ctrl+A` cobre as formas selecionadas · `Select Subpath`/`Select Same`
varrem as formas que a seleção toca · Delete apaga nas duas e **uma forma que morre não leva as
outras** · o `dragging_anchors` devolve pares prontos (a re-montagem à mão que a shell fazia
SUMIU, e com ela o pressuposto de que todas as âncoras em movimento são da mesma forma). O `Tab`
segue percorrendo a forma do nó primário, de propósito: atravessar formas em silêncio ao chegar ao
último nó seria o Tab a mudar de assunto sem ninguém pedir.

**16 gates novos** (12 no motor + 2 no overlay + o invertido + o de controle), **9 mutações, 9
sangram**. ⚠️ **Duas lições de fixture, as duas minhas:** o gate do Average nasceu sobre uma
fixture cujos locais **diferiam** — ali a lei errada também move alguma coisa, então ele mediria a
lei certa contra uma resposta meramente diferente em vez de contra um **no-op**; e uma mutação do
overlay **sobreviveu** porque eu compus duas mudanças que se cancelam (restaurar o `is_sel &&` *e*
comparar só o índice acende o nó 0 da forma errada, e a contagem fica igual). ⚠️ E a **sonda**
passou a mentir no dia em que a ausência fechou: ela contava o primário, porque no mundo que ela
mediu a resposta só podia ser 0 ou 1.

**Nenhum schema, nenhum contrato congelado, nenhum ADR, nenhuma dep.** Smoke:
**`PH2D_BUILD_SMOKE=70`** — três formas, a do meio **escalada 2×** (a premissa que torna a cena
capaz de reprovar; a cena IMPRIME a escala que encontrou).

### ✅ W6.5 — ENTREGUE: o LAÇO

A região que o arrasto no vazio desenha deixou de ser só um retângulo. O plano nomeava o laço como
*"wave dela"*, e a W6.4 era o pré-requisito declarado — um laço que varre os nós de duas formas não
significa nada enquanto a seleção só souber guardar os de uma.

**⚠️ O laço não é uma segunda seleção; é um segundo PREDICADO.** O corpo — o filtro de
escondido/travado, o modo aditivo, o primário que segue o último tocado, o `selected_paths` — é o
MESMO do retângulo, pela porta `select_verts_where`. Duas cópias divergiriam no dia em que uma
ganhasse um caso especial, e o artista veria o laço deixar de somar (ou de respeitar uma forma
travada) sem nada na tela dizer porquê.

**⚠️ E o gate que carrega a wave não conhece a implementação:** *um laço cujo polígono É um
retângulo apanha exatamente o que a caixa apanha* — cinco regiões, comparando `selected_verts`,
`selected_paths` e o primário entre as duas portas públicas. Se alguém der ao laço uma cópia do
corpo, ele continua verde no dia da cópia e fica vermelho no primeiro refino de uma só.

**⚠️ O discriminador é o laço CÔNCAVO.** Um laço implementado como a caixa envolvente do próprio
caminho passa em todo gate de contagem — é a forma mais provável de a feature nascer errada, porque
*funciona* em toda fixture convexa. O gate usa um "C" cuja caixa cobre as duas formas e cujo
interior não contém nó nenhum: **0 contra 8**.

**A GEOMETRIA vive uma vez.** O teste de cruzamento (a regra semi-aberta
`(a.y > p.y) != (b.y > p.y)`, que é o que impede um vértice à altura do raio de contar duas vezes)
saiu para `ph2d_vec_scene::inside::crossing_counts`, com **dois** consumidores: o `contains_point`
de uma forma (que soma sobre os contornos e só então aplica a regra de preenchimento) e o
`point_in_polygon` do laço (um polígono, paridade). ⚠️ **E a mutação óbvia era INVÁLIDA:** trocar
`>` por `>=` **não** é um defeito — são as duas ancoragens da MESMA regra semi-aberta, e as duas
contam cada vértice uma vez. A que erra é a **assimétrica** (`>=` de um lado, `>` do outro), que
conta duas; ela sangra.

**O GESTO: pegajoso E momentâneo, por uma porta.** O chip `Marquee: Box | Lasso` diz qual é a de
sempre; o **Ctrl** troca a de UM gesto (`MarqueeShape::for_gesture`). Não são duas portas para uma
pergunta — é uma função com duas entradas; o que este repo evita são duas *implementações*.

- **O chip existe** porque *um atalho que ninguém descobre é uma feature que não existe* — a mesma
  lei que fez do *Select Subpath* um botão e não uma tecla.
- **O Ctrl existe** porque a região é um **gesto**, não um lugar onde se está: o laço serve a uma
  seleção em cinco, e obrigar a ida-e-volta ao painel por causa dela é o que torna um modo pior que
  um modificador. Um chip pegajoso esquecido também faz o próximo arrasto ser um laço — um
  retângulo pior.
- ⚠️ **`Alt` está fora, e é MEDIDO:** este repo já registrou que o KDE o rouba (a nota do
  `PH2D_STAGGER_SMOKE`). E `Ctrl` estava **livre** no press de canvas em modo Node — os dois usos de
  `cmd_held` do `input_dispatch` ficam depois do `return` do marquee.

**⚠️ A forma congela no PRESS.** Relê-la por movimento faria largar o Ctrl a meio do arrasto morfar
a região sob a mão: o artista veria o caminho que desenhou virar um retângulo entre dois pontos que
ele nunca escolheu. É a lei da régua congelada no `Begin` do arrasto de exposição da tira do Flip.

**⚠️ E a soltura PROMOVE a amostra que o piso recusou.** O laço grava um ponto a cada 2 px (sem
piso, um rato de 960 Hz descreve com milhares de vértices uma curva que dois píxeis descrevem), e o
fecho é onde a mão soltou — não no último ponto aceito. É a lição do motor de traço do Flip (*"o
traço acaba onde a mão soltou"*), e aqui ela decide uma **seleção**: o vão entre o penúltimo ponto e
o dedo é uma aresta de fecho que passa por onde o artista não desenhou.

**⚠️ O chip mora colado à fileira TOOL, e não na seção VERTEX** — que seria o vizinho temático
óbvio. Aquela seção só existe **com um vértice já selecionado**, então um controle de *como
selecionar* moraria lá exatamente onde não se precisa dele: invisível no estado em que o artista o
procura, que é antes de ter selecionado o que quer que seja. O precedente do lugar é a linha do
CORTE, uma função acima, com a razão já escrita.

**23 gates, 11 mutações, 11 sangram** (mais uma inválida, registrada acima). **Nenhum schema**
(`PROJECT_SCHEMA` 72 e `VEC_SCENE_SCHEMA` 14 intocados, por `git diff`), **nenhum contrato
congelado** (4/4 + 3/3), **zero `Cargo.toml`**, **nenhuma dep**. Ids novos: `VECTOR_MARQUEE_BOX` ·
`VECTOR_MARQUEE_LASSO` (hash de string ⇒ fora de todo contador).

**LOC — dois cortes por ASSUNTO, os dois pelo gate certo:** `ph2d-vec-render/src/lib.rs` (715 > 700)
cedeu os dois pintores da região para `marquee.rs` — *tudo o mais na crate desenha o que o documento
É; estes desenham uma coisa que ainda não existe e some ao soltar*; e `paint_modes.rs` (628 > 600, o
cap de PAINEL, que é **outro gate**) cedeu a família de TEXTO para `paint_text_sections.rs`, na
fronteira que o cabeçalho dele **já declarava**.

⚠️ **E um arch-gate existente reprovou — pelo motivo certo.** O
`the_marquee_release_adds_with_shift_and_deselects_on_a_bare_click` ancorava no padrão
`Some((start, cur))`, que esta wave trocou por `Some(m)` quando o gesto passou a carregar a forma. A
**propriedade** que ele afirma continua verdadeira; o **endereço** é que se mudou. Re-ancorado na
chamada (`self.vec_marquee.take()`) — e foi o `expect` do helper (o controle positivo) que tornou
isto uma falha alta em vez de uma varredura vazia a passar.

**Smoke: `PH2D_BUILD_SMOKE=71`** — uma fileira ALTERNADA (azul · âmbar · azul · …), e o pedido é
*"os nós das três azuis e de nenhuma âmbar"*: **nenhum retângulo separa esse conjunto**. Num par de
formas separadas o retângulo faz tudo o que o laço faz, e um smoke sobre essa cena aprovaria um laço
que fosse a caixa envolvente do próprio caminho.

## §7 — W4: OS CORTES

**Tesoura** (`nearest_point_on_path` + `split_segment` — **P**) · **borracha de caminho** (a matemática
por arco do `fx_trim`, destrutiva e local) · **faca** (o `Arrangement` cobre fechadas; ⚠️ **abertas
exigem rota nova** — o motor devolve `Error::NonClosedPath` e a nossa política é `Closing::Always`) ·
**Join como comando de seleção** (o `merge_path_into`/`weld_junction` já fundem; falta o botão e a
regra de seleção) · **Average** de nós (3 linhas sobre `selected_verts`) · e o botão do
**`reverse_path`**, que existe com **um chamador interno e nenhum id de UI**.

**Tamanho: M.** Smoke: cortar uma forma fechada em duas abertas com a tesoura; a faca a atravessar
três formas de uma vez; Join a fechar duas pontas.

### ✅ W4 (A + B) — ENTREGUE: o corte existe, e a tesoura é o 1º consumidor dele

**O achado que reescreveu o desenho da wave inteira:** as quatro ferramentas de corte — tesoura,
faca, borracha de caminho e o *break path* do modo Node — **são UM primitivo só**, *abrir um
contorno num vértice*, e ele **não existia**. A diferença entre elas não é geométrica; é apenas
*de onde vem o vértice*:

| ferramenta | de onde vem o vértice |
|---|---|
| **Tesoura** | um clique: o vértice sob o cursor, senão `split_segment` no ponto mais próximo |
| **Faca** | cada cruzamento de uma lâmina reta com a curva |
| **Borracha de caminho** | as duas pontas de um arrasto AO LONGO da curva (dois cortes + um delete) |

⚠️ **Corolário que apaga metade do trabalho previsto:** a borracha **não terá aritmética de arco
própria** — dois cortes deixam o trecho do meio como um contorno inteiro, e apagá-lo é
`remove_contour`/`remove_path`, que já existem. A linha do plano que mandava portar *"a matemática
por arco do `fx_trim`"* fica **revogada por este parágrafo**: uma segunda resposta a *"onde este
caminho termina?"* divergiria da primeira no dia em que uma quina viva entrasse no meio.

**O primitivo** (`ph2d-vec-scene::path_cut::cut_path_at_vertex`), com três respostas — e a última é
uma pergunta de **fill rule**, não de conveniência:

- contorno **FECHADO** → vira **ABERTO**, re-enraizado no corte, com a costura nas duas pontas;
- contorno **ABERTO**, vértice **interior** → parte em dois, e o vértice fica nos dois lados;
- **ponta** de contorno aberto → `None` (não há ali o que abrir; o Illustrator também recusa).

A 2ª metade vira um **path novo** num path de contorno único e um **contorno irmão** num compound:
separar um contorno de compound em dois OBJETOS mudaria o que a `FillRule` significa — o buraco
deixaria de ser buraco no clique que era para ser um corte.

⚠️ **Os handles do vértice cortado sobrevivem nas DUAS cópias**, e re-fechar devolve a curva **ao
bit**: um contorno aberto só consome o `out` do primeiro e o `in` do último, então zerar os
"mortos" pareceria higiene e destruiria a tangente autorada.

**A junção** (`path_join`) é o inverso, e tem **zero geometria nova**: a receita de *que ponta
encosta em que ponta* já vivia no `weld_new_shape`. A diferença é de GATILHO — lá o par sai de uma
tolerância, aqui o artista escolheu os objetos e a tolerância só decide *se a emenda funde num
vértice ou se nasce um segmento*.

**As três de seleção:** **Average** (colapsa os nós no centroide, movendo o vértice INTEIRO) ·
**Join** (solda 2+ caminhos numa cadeia) · **Reverse** (que agora vira **todos** os contornos — num
compound, inverter metade deixa sentidos misturados e sob `NonZero` isso muda qual região é buraco,
em silêncio).

⚠️ **A auditoria achou que "fechar" JÁ TINHA botão** — o `Close Path` da seção PATH —, então o
**Join não fecha um caminho só**, ao contrário do `Ctrl+J` do Illustrator. Uma segunda porta para a
mesma pergunta divergiria dela no primeiro refino. O que o Join deu ao `Close Path` foi a **SOLDA**:
ele virava só o flag, e fechar um laço cujas pontas o artista tinha acabado de encostar deixava
**dois vértices sobrepostos** no mesmo ponto — invisível no desenho e presente em todo
Delete/Average/Simplify seguinte.

**A tesoura** é o 13º modo (`DrawMode::Scissors`), e o corpo dela é literalmente a pergunta *de onde
vem o vértice?*. ⚠️ Clicar **em cima** de uma âncora corta NELA: sem essa metade, o clique inseria
um vértice coincidente e deixava um segmento de comprimento zero. E o overlay de âncoras fica
**ligado** nela — não é decoração: é o que torna visível onde essa distinção acontece.

**Smoke: `PH2D_BUILD_SMOKE=44`.** Seis formas, com os vãos MEDIDOS pela própria tabela da cena
(`0,1200` e `1,0000`, contra uma tolerância de solda 120 000× menor).

### ✅ W4 (C) — ENTREGUE: a FACA, e ela não tem geometria própria

Uma lâmina reta arrastada corta **tudo** o que atravessa. É a tesoura repetida em cada cruzamento
— zero geometria nova além de *onde uma reta cruza uma curva*, que é amostragem + bisseção
(`blade_crossings`, sem transcendental e sem solver).

⚠️ **As peças ficam ABERTAS** — a escolha do Affinity, e a razão é a que aquele produto documenta:
fechar em silêncio destrói informação (a peça deixa de poder ser reaberta como estava), enquanto
fechar é **um clique** no `Close Path`, que esta mesma wave ensinou a soldar. A previsão do plano
de que uma origem fechada re-fecharia pela corda fica **revogada por decisão**, não por
impossibilidade: a corda de facto É a lâmina, mas fechar não é o default de ninguém que dê escolha.

⚠️ **O laço re-deriva os cruzamentos a cada corte** em vez de os pré-calcular: cortar rota e
re-indexa o contorno inteiro, então uma lista feita antes descreveria vértices que já não existem.
E **só a metade NOVA volta à fila** — a fonte fica com tudo até ao corte, que foi tomado no
PRIMEIRO cruzamento restante, logo ela não pode ter outro (mutação-provado).

⚠️ **Duas camadas independentes** impedem a costura recém-criada — que assenta exactamente sobre a
lâmina — de ser reencontrada para sempre, e **cada uma basta sozinha** (medido: com as duas
removidas, três gates ficam vermelhos; com qualquer uma, verdes). A semântica é o
`blade_crossings` excluir `t` nas pontas; o cinto é o conjunto de pontos já cortados.

⚠️ **O preço, nomeado:** uma lâmina que passa EXACTAMENTE por um vértice não corta ali — a
travessia cai fora como *ponta*. É medida zero, o modo de falha é *não cortar* (nunca *cortar no
sítio errado*), e a alternativa seria cortar o mesmo ponto duas vezes.

**A faca é o 14º modo**, ao lado da tesoura: as duas cortam e a diferença é só o GESTO (um clique ×
um arrasto). ⚠️ E o overlay de âncoras é **desligado** nela — ela age sobre tudo o que a lâmina
atravessa, e as âncoras que o overlay desenha são as do caminho SELECIONADO: mostrá-las anunciaria
um escopo que a ferramenta não tem.

**Smoke: a mesma cena `PH2D_BUILD_SMOKE=44`**, passos 11-15.

## §8 — W5: O PATHFINDER BARATO (edit-time, não toca a D4)

### ✅ W5 — ENTREGUE (2026-08-01): as quatro ops + as duas higienes

**Minus Back · Trim · Crop · Merge** shipam, e nenhuma trouxe geometria nova: são composições do
fold que já existia (`ph2d-vec-boolean::pathfinder`). **Divide** e **Outline** seguem FORA, pelas
razões abaixo — não foram tentadas.

⚠️ **Dois enums, e não é cerimônia:** `BoolOp` é o vocabulário do MOTOR (o que o `linesweeper`
entende, e o que o Build/Expand consomem); `PathfinderOp` é o do ARTISTA. Os quatro primeiros
coincidem; os quatro novos **não são operações de conjunto**, são receitas sobre elas — Trim
devolve uma forma POR FONTE, cada uma com o SEU estilo, e metê-las no `BoolOp` daria dois
significados a `apply_many`.

⚠️ **Divergência declarada do Illustrator:** o Trim dele REMOVE os traços; nós mantemos (apagar em
silêncio o que o artista autorou é destruir trabalho). Uma linha, se o smoke pedir.

⚠️ **Duas da mesma cor que NÃO se tocam continuam DUAS** no Merge — o motor agrupa por CONTENÇÃO.
Eu esperava o contrário e a medição corrigiu; é também o que o Illustrator faz (*adjacent or
overlapping*).

**As duas higienes fecharam, e a segunda era pior do que a auditoria dizia:**

- o `linesweeper::Error` deixou de ser engolido (`apply_many_checked` / `pathfinder` devolvem
  `Result`), então **`Ok(vazio)` e `Err` deixaram de pintar o mesmo nada**;
- ⚠️ **o motor PANICA com `NaN`** (`geom.rs:63`, `assert!(x.is_finite())`) em vez de devolver o
  `Error::NaN` que declara: o `binary_op` dele só examina o BOUNDING BOX, e `min`/`max` com NaN
  devolve o outro operando. A guarda de finitude é NOSSA, no choke point único
  (`engine::binary_grouped_checked`) — e cobre o Expand e o Shape Builder, que não sabem que ela
  existe. **É a diferença entre um toast e um crash**, e a entrada é alcançável (um `Transform`
  degenerado assado na geometria, ADR-0111).

Gates: 12 no motor (área como oráculo, nunca contagem) + a varredura dos **8** botões com ponteiro
REAL + o mapeamento dos 8 ids. **8 mutações, 8 sangram** (a M7 sangra com o pânico do próprio
`linesweeper` — a prova). LOC: `lib.rs` bateu 730 ⇒ split `engine.rs` (as OPERAÇÕES × a PASSAGEM).

⚠️ **Um conceito meu foi REMOVIDO antes de shipar:** eu tinha um `consumes_every_source()` para
decidir o que apagar do documento — e as oito instalam igual (todas as fontes saem, os resultados
entram). Era distinção sem diferença, e o gate dela testava um conceito inventado.



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
| ~~**Rótulo de distância** nos smart guides~~ ✅ **W6.6** | ⚠️ e ele obrigou a reconciliar a UNIDADE: a régua era a única superfície do app que não convertia — e não convertia por não CONSEGUIR (`paint_rulers` não recebia as settings) | `LengthDisplay` (porta nova), **não** o `VecLabel` — a ficha é chrome de um frame, não um objeto do documento | **M** |
| **Snap a SPRITES** | nenhum competidor tem, porque nenhum mistura raster e vetor na mesma árvore — **nós misturamos** (ADR-0110) | a bbox já está na shell | **P** |

**Fora desta wave, nomeados:** grade **perspectiva** (**G**) · grade **polar** (10º `GridKind`, **M**) ·
os *planos* da isométrica · **Transform Again** (⚠️ **`KeyD` já é o boolean Subtract** — não pode nascer
com o atalho do Illustrator) · **Measure tool / cotas** (**M** sobre `VecLabel`) · **autotrace** (**G**,
motor espalhado).

### ✅ W6.1 — ENTREGUE (2026-08-01): a reivindicação 2-D (path · cruzamentos · sprites)

O motor de snap ganhou uma **segunda espécie de reivindicação**, e é a distinção que a wave
inteira gira em torno.

O que já existia é **ALINHAMENTO**: uma restrição 1-D por eixo, e é por isso que ela se
decompõe (o X vem de uma vizinha, o Y da grade, e o resultado faz sentido — são duas retas que
se cruzam). Encaixar **sobre uma curva** não é disso: é uma **POSIÇÃO**, restrição 0-D.
*"Alinhar meu X com o X do ponto mais próximo daquela curva"* não quer dizer nada — todo X
dentro da faixa da curva é o X de algum ponto dela. Uma posição vence os dois eixos ou nenhum.

⚠️ **A lei que mantém as quinas alcançáveis.** A curva passa POR CIMA de cada âncora, então
perto de um vértice as duas espécies competem — e se a posição vencesse sempre, o nó pousaria a
uma fração de pixel do canto **para sempre**, sem gesto que corrigisse. A regra é *vértice vence
curva* (a do Inkscape), enunciada como propriedade do **RESULTADO**: se o alinhamento já pousa
exactamente sobre UM alvo (os dois eixos vindos do mesmo ponto), isso **é** uma coincidência com
um ponto distinto, e a reivindicação 2-D **se retira**. O corolário é o que torna a mudança
segura — sem curvas na lista, a lei nunca dispara e o encaixe é **byte-idêntico** ao que shipava
(gate `geometry_in_the_target_list_changes_nothing_while_the_toggles_are_off`).

**O kernel é geometria, não política** (`ph2d-vec-scene::curve_probe`, puro, sem kurbo):
`nearest_on_segs` e `crossings_near` **amostram para escolher a bacia e refinam por NEWTON**.
Amostrar não serve: numa reta de 1000 unidades as 17 amostras deixam o pé da perpendicular
**31 unidades** fora, e o snap pousaria o nó onde o ímã não prometeu — a um zoom alto, meia tela.


**Sprites entraram junto** (o item **P** da tabela): nenhum concorrente tem, porque nenhum mistura
raster e vetor na mesma árvore — nós misturamos (ADR-0110). A caixa vem da porta que o GIZMO usa
(`anchor ± half`), e cada ponto sobe pelo afim **um a um**: sob rotação o alvo é o quadrilátero
girado, não a caixa dele.

**UI:** a seção **Snap** que já existia ganhou duas linhas — `Path` e `Cross` —, ambas nascendo
**desligadas** (um ímã que agarra a linha inteira muda como todo gesto se comporta; ligá-lo por
default mudaria o app debaixo de quem não pediu). **Visual:** quatro espécies, quatro marcas —
tracejado+✕ (alinhamento) · ✕ (grade) · **anel** (sobre a curva) · **+** (cruzamento).

**Zero schema** (`PROJECT_SCHEMA` 46, `VEC_SCENE_SCHEMA` 13 intactos — os ajustes de snap são
estado de FERRAMENTA), **zero contrato congelado**, **zero `Cargo.toml`**. Ids novos no irmão
`vector_snap.rs` (`vector.rs` estava em 685/700). **14 mutações, 14 sangram.**

⚠️ **Quatro correções que a mutação me impôs, e valem mais que os números:**
1. **Um bug de SINAL no Newton 2×2** (`J·Δ = −F` *e* subtrair = negação dupla; o refino caminhava
   para longe da raiz). Um gate entre duas RETAS **não pode vê-lo**: ali a corda amostrada já é a
   resposta, `F = 0`, e o Newton quebrado fica parado sobre o valor certo. Foi preciso curvatura.
2. **A regra da ponta parecia gateada e não estava:** o helper de fixture fazia cúbicos
   degenerados, cuja derivada é **zero nas pontas**, então o encontro morria no guard de
   degeneração antes de chegar à regra — duas defesas em camada, uma gateada.
3. **`each_toggle_governs_only_its_own_species` era verde por vácuo:** a fixture não tinha
   cruzamento nenhum, então ligar ou desligar "Cross" dava o mesmo resultado.
4. **Uma rotação de 90° não distingue "os pontos" de "a caixa dos pontos"** — ela leva caixa
   alinhada em caixa alinhada. A fixture teve de ir para 45°.

⚠️ **E um gate meu foi RENOMEADO por afirmar o que não podia ver:** `each_row_shows_its_own_state`
não falhava quando as duas linhas liam o mesmo bool, porque `painted_rect` devolve GEOMETRIA e as
opções são pintadas nas mesmas posições esteja qual estiver acesa. A prova da fiação mudou-se para
onde ela é observável (os arch-gates `the_snap_toggles_are_not_crossed` e
`each_pending_snap_toggle_lands_on_its_own_field`, na shell).

⚠️ **Uma proteção NÃO é observável e está escrita em vez de creditada:** o guard `den ≈ 0` do
Newton pode ser removido com os doze gates verdes, porque `0/0` vira `NaN`, o `clamp` o propaga e
a aceitação final o recusa. Ele fica por tornar a intenção visível — depender de propagação de
`NaN` morre em silêncio no dia em que alguém acrescentar um `is_finite` acima.

### ✅ W6.2 — ENTREGUE (2026-08-01): as GUIAS e a RÉGUA (o único **G** da tabela)

**O que uma guia É decide o resto.** Ela é uma reta **alinhada a um eixo**, logo uma restrição
**1-D**: a horizontal fixa o `y` e não diz nada sobre o `x`. Isso a põe exatamente na espécie que
o motor já tinha — o **ALINHAMENTO** por eixo, aquele que se decompõe — e **não** na de POSIÇÃO
que a W6.1 acrescentou. Confundi-las seria fatal na direção óbvia: uma guia que reclamasse os dois
eixos prenderia o ponto num lugar arbitrário DA linha, que é precisamente o que uma guia não faz.

⚠️ **A lei nova é uma só: a guia vence o EMPATE** contra um ponto de forma. Ela é a restrição que
o artista **autorou**; a borda de uma vizinha é incidental. Um ponto estritamente mais perto ainda
ganha — prioridade no empate, não imunidade (medido na cena de smoke: com a guia em `x=1,00` e o
vértice em `0,94`, a guia passa a vencer a partir de `x=0,970`, o ponto médio exato que a lei
prevê).

⚠️ **Duas guias que se cruzam são um ponto distinto** tanto quanto um vértice é, então a
reivindicação de POSIÇÃO se retira diante delas. Elas **não** passam pelo teste de alvo-único (cada
uma congela só a sua coordenada, então os dois `target` diferem por construção), e sem essa segunda
cláusula uma curva por perto roubaria o cruzamento de duas linhas que o artista pôs de propósito.

**A crate é própria** (`ph2d-guides`, leaf, só `serde`): uma guia não é geometria vetorial, e são
três consumidores independentes (o motor de snap, o desenho, o arquivo) sem que nenhum seja dono do
fato. Mesmo argumento e mesmo precedente da `ph2d-stroke-width`.

⚠️ **Sem teto no número de guias, e o §0 é o motivo:** um teto tem de dizer de que recurso ele é, e
aqui não há nenhum — 9 bytes por guia, e o `nearest` sobre **mil** delas custa **0,305 µs** (o
número vive no gate `the_cost_of_a_guide_is_a_comparison`, não numa nota).

**A RÉGUA mora ao lado da GRADE** (`ph2d-editor-core::ruler`) e **não tem projeção própria**: as
duas respondem à mesma pergunta — *onde a coordenada `x` do mundo pousa na tela?* — e um traço
marcado em 100 que não coincida com a linha de grade de 100 é o tipo de discordância que ninguém
atribui a um bug de projeção, só a *"o app está torto"*. O `ticks()` é puro e chama
`grid::world_to_screen_*`; o `world_at()` (o pouso do gesto) chama a **inversa**, que nasceu no
mesmo arquivo da ida para que não possam divergir.

⚠️ **A cadência dos traços NÃO é a da grade**, e é decisão: a grade pode estar desligada, e uma
régua tem de ser legível em qualquer zoom — ela escolhe o passo `1/2/5 × 10^k` que mantém os
rótulos separados. **A grade marca a REDE; a régua mede o MUNDO.** O **zero** é a origem da grade:
um número, dois consumidores.

⚠️ **Um vocabulário, com a ponte explícita:** `RulerAxis::{Top,Left}` e
`GuideAxis::{Horizontal,Vertical}` são enums distintos porque a régua de CIMA mede o **X** e o que
nasce dela é uma linha **HORIZONTAL** — a inversão exata que se troca sem o compilador reclamar.
`RulerAxis::spawns()` é a ponte, com gate.

**O gesto:** arrastar da régua cria · arrastar move · arrastar de volta para **qualquer** faixa
apaga (o modelo universal — Figma, Illustrator, Photoshop). **Criar e mover são o MESMO caminho**:
um press na régua já empurra a guia para o documento e o gesto continua como um arrasto dela, então
desistir de uma guia nova e descartar uma antiga são o mesmo gesto sem que nada precise saber qual
era. ⚠️ **E a régua vive com a ferramenta VECTOR em mãos** — uma CORREÇÃO achada auditando a própria
wave, não escopo escolhido: a faixa **ocupa** a borda do canvas e o gesto dela corre **antes de
toda ferramenta**, então uma régua permanente comeria o pen-down do **Painter** nos 20 px de cima
(o artista pincela ali e nasce uma guia). O invariante que importa fica de pé — ***visível ⇔
vivo***, por uma **porta única** (`HeroScreen::rulers_live`) que o paint e o gesto perguntam: uma
faixa que desenha e não responde, ou que responde sem aparecer, é o chrome morto sob o mouse que
esta codebase varre a cada wave. ⚠️ A mutação que apaga a segunda metade da porta **sobreviveu a
todos os outros gates** — a correção estava shipada e desguardada até o gate próprio nascer.

⚠️ **Com as réguas fora as guias ficam INERTES** — visíveis e magnéticas, mas não agarráveis. É o
*lock de guias* que o Illustrator e o Photoshop escondem num booleano de menu (e que o **Figma não
tem**, com os usuários a pedi-lo no fórum): aqui ele é o mesmo interruptor que já se vê na tela,
então não há como o estado travado ficar invisível.

**UI:** a seção **Snap** ganhou duas linhas — `Guides` (o ímã, nasce LIGADO: num documento sem
guias ele é inerte por construção) e `Rulers` (as faixas, e com elas o lock; nasce LIGADO porque
uma afordância que ninguém acha é uma que não existe).

**`PROJECT_SCHEMA` 48→49** — as guias viajam no `ProjectState`, que é a unidade do UNDO: é isso que
lhes dá desfazer e salvar de graça, pelo mesmo diff que já cobre o mundo, o vetor e o Flip. ⚠️ O 49
é **PROVISÓRIO**: o valor se CONTA contra o `main` do dia da integração. `VEC_SCENE_SCHEMA` **13
intacto**, contrato congelado **intacto**, **nenhum ADR**.

**36 gates novos** (7 modelo · 9 régua · 8 snap · 6 gesto · 1 porta · 3 arch · 2 seam estendidos),
**15 mutações, 15 sangram**.

⚠️ **E dois gates MEUS nasceram errados, os dois reprovando código correto:**
1. A metade *"o passo não é folgado"* da régua enumerava `step/2`, `step/2.5`, `step/5` como
   candidatos anteriores — e `step/2` de 5000 é **2500**, que não é um degrau da escada (ela vai
   1000, 2000, 5000). O oráculo virou a escada **construída fora da função sob teste**. *Uma
   propriedade se afirma, não se enumera.*
2. O arch-gate da ordem de despacho comparava a posição do press de guia com a **definição** de
   `fn vec_path_pick_click` — e a posição de um `fn` no arquivo não diz nada sobre ordem de
   despacho. A âncora virou a **CHAMADA**.

**Smoke: `PH2D_BUILD_SMOKE=45`** (a cena imprime o que montou e o roteiro de 7 passos).

### ✅ W6.3 — ENTREGUE (2026-08-01): o MIRROR, a simetria VIVA

A forma ganha um eixo e o outro lado passa a ser **derivado**: editar um nó move os dois. O repo
só tinha **Flip H/V destrutivo**, que vira a forma uma vez e esquece; isto é um `PathEffect`
(ADR-0132), então o reflexo re-cozinha a cada frame e segue a caneta, o arrasto de âncora e o
Width Tool **de graça**.

**Por que não é um parâmetro do Repeater — e é um FATO, não gosto.** O Repeater compõe rotações e
translações, cujas matrizes têm **determinante +1** sempre; uma reflexão tem **determinante −1**.
Nenhuma combinação de `spin`/`orbit`/`move` a alcança — os dois geram *grupos diferentes*. O gate
`a_reflection_is_out_of_the_repeaters_reach` afirma-o pela **área com sinal**, que é o
determinante a fazer o seu trabalho.

**O neutro é `Axes = 0`, e é o Blender.** A pilha tem uma lei **executável**
(`every_kind_is_born_neutral`) e um espelho não tem um *amount* contínuo que se possa zerar —
reflectir *um pouco* não quer dizer nada. O que ele tem é **quantos eixos** espelha, e o
modificador Mirror do Blender é exactamente isso: nenhuma caixinha marcada é um no-op. ⚠️ Não é
uma segunda porta para o `FxEntry.enabled` — o Repeater tem a mesma forma (`copies = 1` também é
geometricamente igual a desarmar a entrada), e o `is_neutral` existe para nomear a versão em
espaço-de-parâmetros.

**O eixo é ângulo + deslize, sem segunda geometria.** O deslize é percentagem do **SUPORTE da
caixa** na direcção da normal (`|n.x|·hx + |n.y|·hy`), então `100` põe a linha **tangente à
caixa em QUALQUER ângulo** — a propriedade *"um número redondo dá um encaixe exacto"* do
*Relative Offset* do Array do Blender, que o cabeçalho do Repeater já defende. Uma referência
isotrópica (`ref_size`) só acertaria numa forma quadrada.

⚠️ **E o default é `100`, não `0` — um defeito de PRODUTO que eu ia shipar.** Com o eixo no
CENTRO da caixa o reflexo cai **em cima** da forma (mesma caixa, virada): ligar o espelho quase
não muda nada, e um meio-perfil espelha sobre o meio de si mesmo. Com a linha tangente, subir
`Axes` a 1 **duplica a forma ao lado** e o meio-perfil **funde** no vaso — o caso de uso inteiro,
sem tocar num slider. ⚠️ A decisão não tinha gate nenhum, porque *cada gate declarava o seu
próprio deslize*: **um default só é testado por um teste que não o menciona**
(`the_defaults_alone_turn_a_half_profile_into_a_fused_vase`).

**O winding é REPOSTO.** Uma reflexão inverte o sentido de percurso, e sob `NonZero` dois
contornos sobrepostos de sentidos opostos **cancelam-se** — o artista veria um *buraco* onde
espelhou. Cada contorno reflectido é invertido de volta, e como a inversão é **uniforme** o
buraco de um compound continua buraco.

**A FUSÃO faz do meio-perfil um vaso.** Um contorno ABERTO com as duas pontas no eixo funde-se
num **único contorno FECHADO** (o *Fuse paths* do LPE do Inkscape / o *Merge* do Blender), com as
alças da costura reflectidas. ⚠️ **A costura fica lisa quando a alça é PERPENDICULAR ao eixo** —
a mesma regra do Blender, documentada em vez de descoberta; uma alça oblíqua dá um bico simétrico,
que às vezes é o que se quer. Onde não se aplica (contorno fechado, pontas fora do eixo) degrada
para o espelho simples, **visivelmente**.

**`Axes = 2`** é o mesmo espelho aplicado duas vezes (o segundo eixo é a perpendicular pelo mesmo
ponto) ⇒ 4 dobras. É deliberadamente **equivalente a empilhar dois Mirror**, e a equivalência é
uma virtude: há **uma** lei, e a contagem é a forma barata de a pedir sem gastar um dos quatro
slots da pilha.

**Porta única nova:** `reverse_contour` saiu do `reverse_path` (o botão **Reverse** da seção
Vertex) para o `compound.rs`, porque *"como se inverte um contorno?"* ganhou um segundo consumidor.

⚠️ **`MAX_FX_KINDS` 21→22, e o gate do teto tinha um buraco que esta wave tornou load-bearing:**
o que mora na `ph2d-vec-scene` compara contra uma **cópia literal** do número (a crate não alcança
a `editor-core` — seria ciclo), então ele pega o motor a **crescer** além do painel e é **cego ao
painel a encolher** abaixo do motor. Medido: baixar `MAX_FX_KINDS` deixa aquela suíte **inteira
verde** com o último tipo inalcançável no menu Add. O gate novo mora na **shell**, que vê os dois
lados, e lê as duas constantes **ao vivo**.

**Nenhum schema** (`PROJECT_SCHEMA` 49 e `VEC_SCENE_SCHEMA` 13 intactos — apender um variant não
bumpa, o precedente desta própria linha na wave do Falloff) · **nenhum ADR** · **zero `Cargo.toml`**
· contrato congelado intacto.

**LOC:** o `lib.rs` cruzou 700 com a declaração do módulo novo ⇒ **split por ASSUNTO** em
`paint.rs` (`Rgba8`, os gradientes e o `Paint` — *com que tinta a forma aparece*, contra *o que a
forma É*, que fica no `lib.rs`), com re-export na raiz ⇒ **nenhum caminho do workspace muda**.
702 → 581.

⚠️ **E um vermelho LATENTE da W6.2 fechou de carona:** um `→` unicode num `assert` do
`ruler_tests.rs` disparava o `no_tofu_glyphs` — que mora em `tests/` e **a bateria daquela wave
não o alcançou**. É a mesma causa estrutural que a `line/physics` e a `line/motion-value` já
documentaram: *um gate em `tests/` não é alcançado por uma corrida filtrada*.

**16 gates novos, 9 mutações, 9 sangram.** **Smoke: `PH2D_BUILD_SMOKE=46`** (quatro sujeitos — o
vaso que funde, a sobreposição que prova o winding, a roseta de 4 dobras e o **controle** com o
espelho neutro; a cena **mede a própria geometria cozida** e imprime os números).

### ✅ W6.6 — ENTREGUE: o RÓTULO DE DISTÂNCIA, e a unidade que ele obrigou a reconciliar

A última linha da tabela §9. A guia de alinhamento já dizia **com o quê** a forma alinhou —
faltava **quanto**, que é a pergunta que faz de um encaixe uma medição e a razão de esta wave
se chamar PRECISÃO.

**O que se vê:** ao encaixar, o segmento tracejado ganha uma ficha com o número no meio dele,
com sufixo (`150 px` / `1.5 m`).

⚠️ **Mas a feature não é o rótulo — é a PERGUNTA que ele forçou.** Um número novo na tela tem
de dizer *quanto* em ALGUMA unidade, e a resposta já existia em duas versões que discordavam,
sem nenhuma saber da outra:

| superfície | convertia? | o que ela dizia para a MESMA distância |
|---|---|---|
| Inspector (`Position`) | **sim** (`panel-inspector/src/sync.rs`, e o rótulo diz `Position (px)`) | `150` |
| painel **Grid Snap** | **sim** (`grid_snap/inspect.rs`) | `150` |
| **RÉGUA do canvas** | **não** | `1.5` |

⚠️ **E a régua não convertia por não CONSEGUIR:** `paint_rulers` nem sequer recebia
`ProjectSettings` — a divergência era estrutural, não um `if` esquecido. Medido: com os
defaults (100 px/m, unidade **Pixels**) a linha de grade que o artista digitou como **100**
era rotulada **1**.

⚠️ **O header da própria régua afirmava o contrário** — *"world-units, a mesma régua que o
Inspector mostra nos campos X/Y"* —, e era **FALSO**: o Inspector mostra px. A frase estava
certa na INTENÇÃO (*um ponto, um número*) e o código a contradizia; ela foi reescrita, não
apagada.

**A cura é uma porta:** `ph2d_editor_core::LengthDisplay` (`length.rs`) — *o que este app
imprime quando imprime um comprimento*. Ela decide **o número e as casas**; não decide onde o
texto pousa, com que corpo, nem se há sufixo, que é de quem desenha.

⚠️ **O passo entra em MUNDO e é convertido pela MESMA porta.** Converter só o valor imprimiria
`150` com as casas de um passo de `0,5` — uma casa decimal que o número não tem resolução para
honrar.

⚠️ **A GEOMETRIA da régua não muda:** o passo segue escolhido em mundo a partir do zoom, e só o
NÚMERO cruza a fronteira. É isso que faz de um projeto em **metros** um caso **byte-idêntico**
ao que já shipava (`from_meters` é a identidade ali) — e é o CONTROLE do gate.

⚠️ **Duas larguras, UMA regra:** `DisplayUnit::from_meters_f64` nasce e a versão `f32` **delega**.
A largura importa — uma coordenada de régua de `1e6 m` em pixels é `1e8`, que o `f32` não
carrega até o dígito que o rótulo imprime.

**Três donos, e a divisão é o desenho:**

- **quais guias merecem número e onde ele pousa** — `ph2d_vec_render::snap_labels`, geometria
  pura, sem noção de unidade;
- **que número, com que casas** — a porta acima, a mesma da régua;
- **como uma ficha é desenhada** — `render_loop/vec_snap_labels.rs`, e só ele.

**A lei de quem recebe número, em duas frases.** *Só a guia de ALINHAMENTO tem o que medir* —
as outras quatro espécies dizem *você está AQUI*, e um `0` flutuante ao lado de cada encaixe
seria ruído com aparência de informação. *E o rótulo mede o que se VÊ* — um alinhamento que
pousa exatamente sobre o alvo (a coincidência que a lei *vértice vence curva* trata como caso
normal) desenha um segmento de comprimento zero e cai pela MESMA regra, sem caso especial.

⚠️ **O piso de visibilidade é DERIVADO, não escolhido:** a guia é capeada por uma cruz em cada
ponta, cada uma medindo `2 × TICK_PX`; um segmento menor que as DUAS cruzes está inteiramente
coberto pelas próprias marcas.

⚠️ **A régua imprime o número NU e a ficha imprime COM sufixo**, e não é gosto: uma régua é
entendida pela faixa graduada em que ela vive, e a ficha **paira sobre a arte** sem eixo nenhum
ao lado que a explique. É também o que torna a unidade ativa visível sem abrir o menu.

⚠️ **E um gate MEU nasceu incapaz de reprovar.** O `only_the_alignment_guide_gets_a_number`
construía as quatro espécies de ponto com `a == b` — que é o que os produtores de HOJE fazem —,
então elas eram filtradas pelo **PISO de comprimento** e a lei do KIND nunca era exercitada: a
mutação que aceita toda espécie **passou**. A fixture passou a dar `a != b` às espécies de
ponto, e a mesma mutação sangra. *A lei é sobre o SIGNIFICADO da guia, não sobre um acidente de
quem a constrói.*

**9 gates novos** (7 na porta + 5 no motor de rótulo + 4 arch-gate, entre unidade e shell),
**9 mutações, 9 sangram** — a central (`label_text` volta a formatar o valor cru) reprova com
`régua 0 contra painel 50`, e **só ela**: as outras seis testam o *quê*, não o *acordo*.

**Nenhum schema** (`PROJECT_SCHEMA` **72**, `project.rs` com diff VAZIO) · **nenhum ADR** ·
contrato congelado intacto · **zero `Cargo.toml`** · nenhum id/token novo.

**Smoke: `PH2D_BUILD_SMOKE=72`.** ⚠️ A cena põe as **DUAS** superfícies na mesma tela de
propósito — a pergunta da wave não é *"aparece um número?"* e sim *"a régua e a ficha dizem o
MESMO número?"*. O passo **4 é o que prova a wave**: trocar para **Meters** no menu Settings, e
as duas têm de mudar JUNTAS (`150 px` vira `1.5 m`); se só uma mudar, são duas portas outra vez.

#### ⚠️ Correção pós-smoke: a ficha NÃO empresta a cadência da régua

O parágrafo que estava aqui dizia que a ficha usar `label_step` era *"a política que as mantém
coerentes"*. **O smoke refutou:** *"em metros, só mede metros inteiros, mas deveria ser metros e
cm"* (Enio). E era pior que grosseiro — no zoom de trabalho a ficha imprimia **`2`** para uma
distância de **1,5 m**.

**As duas cadências respondem perguntas diferentes**, e eu tinha colapsado as duas:

| pergunta | quem responde | o que ela mede |
|---|---|---|
| *que números merecem ser IMPRESSOS nesta faixa graduada?* | `ruler::label_step` | **LAYOUT** — dois rótulos não podem colidir, daí os 56 px de `MIN_LABEL_PX` |
| *quanta resolução este zoom DISTINGUE?* | `length::world_per_pixel` | **UM pixel de tela** |

Medido, a 100 px por metro de mundo: o passo dos traços vale **1 m** e um pixel vale **1 cm**.

**Para a RÉGUA as duas coincidem por construção**, e é por isso que ela fica exatamente como
está: o rótulo dela senta SOBRE um traço, logo o valor **é múltiplo do passo** e não há nada
abaixo dele a perder. A ficha flutuante imprime um número **arbitrário** — o que o arrasto do
artista produziu —, e emprestar o passo joga fora tudo o que está abaixo dele.

⇒ `text(world, resolution_world)` continua **uma regra**, com **dois argumentos**: a régua passa
o passo dos traços, a ficha passa um pixel. A porta não se duplicou.

**A regra nunca esconde um dígito que o artista possa ver**; nos zooms que caem no meio de uma
década ela mostra **um a mais** (resolução de 5 mm imprime milímetros), e esse é o lado certo
para errar — é o que faz cada pixel de arrasto mexer no número em vez de o deixar gaguejar.

⚠️ **E em PIXELS nada muda** — no zoom de trabalho um pixel de tela É um pixel de display, então
o número segue inteiro. **É por isso que o defeito era invisível na unidade default** e
atravessou a wave inteira até o smoke; o gate novo carrega essa metade como CONTROLE.


#### O que ficou aberto e foi construído: a QUARTA superfície

A régua, o Inspector e o painel de Grid Snap respondem à mesma pergunta — *onde está esta coisa,
e que tamanho tem?* A W6.6 pôs a régua de acordo com os outros dois. **O painel do VETOR era a
quarta**, e a única ainda a responder em metros de mundo:

| superfície | para a mesma forma |
|---|---|
| Inspector `Position` | **150** |
| régua do canvas (W6.6) | **150** |
| painel de Grid Snap | **150** |
| **painel do VETOR, X** | **`1.5`** |

E é o painel que o artista tem aberto **enquanto desenha**.

#### A fronteira tem TRÊS lados, e esquecer um deles compila

1. **A publicação** — os quatro números saem por `LengthDisplay::value`. Posição e tamanho pela
   mesma porta, porque a conversão é escala pura (sem deslocamento): `x` e `w` não precisam de
   leis diferentes.
2. **A volta** — o número DIGITADO entra por `LengthDisplay::to_world`, a porta nova. ⚠️ Uma porta
   que MOSTRA por um caminho e LÊ por outro é como o artista digita de volta o mesmo `150` que o
   campo lhe mostrou e a forma salta cem vezes; `to_world(value(w)) == w` é gate.
3. **A TAXA DE ARRASTO** — ela é *comprimento por pixel de cursor*, logo é um comprimento.
   Esquecê-la é o defeito que **compila e parece funcionar**: o chip mostraria centenas e andaria
   `0,01` por pixel, parecendo travado.

#### ⚠️ E o que NÃO atravessa é o que carrega o gate

`apply_vec_transform` tem **dois** chamadores e só um carrega um número do artista. O outro é o
**preset de dispositivo**, e `DevicePreset::size` devolve *unidades de DOCUMENTO* (o aspecto do
aparelho normalizado ao `LONG_SIDE`) — dado **AUTORADO**, não a face de nada. Converter dentro da
operação seria o corte errado e **compila**: toda moldura de aparelho encolheria cem vezes sem que
nada dissesse uma palavra.

⇒ A conversão mora no sítio do **DRENO**, e o gate afirma a **ausência** dela no bloco vizinho,
com `expect` como controlo positivo (se o bloco se mudar, o gate falha alto em vez de varrer o
vazio).

#### O cabeçalho DIZ a unidade

`Transform (px)` — o precedente do Inspector. Os números chegam já na face do artista, então sem
esta palavra ele não sabe se `150` é pixel ou metro. Um sufixo por ROW seriam quatro cópias, e o
**`R` ficaria a mentir junto** (é em GRAUS — não é comprimento, e por isso não herda o sufixo da
seção). ⚠️ O painel recebe o **sufixo, nunca a regra**: guardar a escala ali seria a segunda cópia.

---

### ✅ W6.8 — A SEGUNDA FRONTEIRA: o auto layout, e as TRÊS naturezas de número

⚠️ **O censo da W6.7 dizia "outros quatro" e estava CURTO: são DEZ.** Ele contou as *rows* que se
veem (`gap`, `min.w`, `min.h`, recuo) e o que atravessa a fronteira são os CAMPOS —
`gap[2]` + `pad[4]` + `min[2]` + `max[2]`. *Um censo por linha de tela conta o que se vê, não o
que viaja.*

E a diferença que faz desta uma fronteira própria, não um copiar-colar da anterior:

| natureza | campos | atravessa? |
|---|---|---|
| **comprimento** | vão, recuo, piso, teto (10) | **sim** |
| **contagem** | `Columns` | **não** — dividida por cem, a grade nasce com zero |
| **razão** (flexbox) | `Grow`, `Shrink` | **não** — quem não é comprimento não tem unidade |

Os três partilham o mesmo `f64` e o mesmo dreno, então a conversão tem de ser **CONDICIONAL** — e
⚠️ **converter os três compila**, sem uma palavra de ninguém.

#### A pergunta mora no TIPO, e não numa lista ao lado

`LayoutField::is_length()` é um `match` **exaustivo** sobre o enum: uma variante nova é *cobrada
pelo compilador*. Uma lista de ids ao lado da conversão é a que apodrece — o 15º campo nasce fora
dela e converte (ou não) em silêncio.

Do outro lado, `flow_in_display` é a porta: os dez comprimentos do `LayoutFlow` cruzam e o
`columns` — que mora no **mesmo struct** — atravessa cru. Mapear o struct em bloco seria dividir
*"três colunas"* por cem.

#### ⚠️ E o cabeçalho do Layout NÃO ganha sufixo, de propósito

A seção Transform é toda de comprimentos e por isso pode dizer `(px)` uma vez. A do Layout é
**genuinamente mista** — um `(px)` ali reivindicaria a unidade também para `Grow 1` e
`Columns 3`, que não a têm. O rótulo da seção é o lugar errado para uma verdade que só vale em
parte dela; rotular as rows de comprimento uma a uma é mudança de UI que o smoke deve julgar.

#### E o `R` passou a carregar o próprio símbolo

`R°`. É ele que torna o `(px)` do cabeçalho do Transform **honesto**: sem isto o sufixo da seção
reivindicaria também a rotação, que é em graus. *Um campo que se auto-rotula é mais barato que uma
exceção escrita num doc-comment que o artista não lê.*

**O censo fecha: 14 de 14** — os quatro do Transform (W6.7) e os dez do layout.

### ✅ W6.9 — O X/Y NUMÉRICO DO NÓ (o último item da lista da §6)

A §6 nomeava-o desde a wave do alcance do nó: *"trabalhar uma forma de 40 nós é lento porque não há
X/Y numérico do nó"*. Havia como **arrastar** um nó e como **encaixá-lo**; não havia como **DIZER
onde ele vai**. As duas fileiras entram na seção Vertex, ao lado dos chips Corner/Smooth/Symm.

#### O modelo é o do BLENDER, e a escolha não é gosto

| ferramenta | com 1 nó | com N nós |
|---|---|---|
| Illustrator · Rive · Figma | escreve o alvo | **campo não existe** (só seleção única) |
| Inkscape | escreve o alvo | **escreve o alvo em CADA nó** ⇒ todos colapsam num X |
| Blender (*Median*) | escreve o alvo | mostra a **mediana**, aplica o **deslocamento** |

⚠️ **Com um nó os três modelos dão o MESMO resultado** — a mediana de um conjunto de um é o
elemento —, então o defeito do Inkscape é invisível no caso comum e destrói a forma exatamente no
caso que a multi-seleção existe para servir (a W6.4 deu dono à seleção de nós, e ela atravessa
formas). O de Illustrator/Figma não é errado, é **ausente**: eles simplesmente não oferecem o campo
com mais de um nó, o que devolve o artista ao arrasto.

#### A escrita passa pela porta que já existia

`PenTool::nudge` é a porta das setas do teclado: recebe um delta de **MUNDO**, converte para local
**por forma** e move a âncora **e os handles**. O dreno subtrai a mediana do alvo e chama-a — um
`set_vertex_position` seria a segunda resposta a *como um nó se move*, e as duas divergiriam no dia
em que uma delas ganhasse um caso especial (a lição que o ADR-0128 pagou cinco vezes).

#### A LEITURA é em MUNDO, e é a metade que pode mentir em silêncio

`selected_anchor_world` é nova, e a regra-mãe do módulo decide o corpo dela (ADR-0111): *o que se
vê/aponta/encaixa é MUNDO; o que o documento guarda é LOCAL*. Sob a pose de uma forma escalada, ler
`v.anchor` cru dá um número que **discorda da régua sob o próprio nó** — e compila.

⚠️ **E ela é uma MEDIANA, não o primário.** O irmão `selected_vertex_kind` já pagou isto: ele
reportava o tipo do vértice primário, o que faz o painel afirmar sobre três nós uma verdade de um
só. Uma mediana é uma afirmação sobre o conjunto inteiro.

#### O que o gate mediu, e a mutação que sobreviveu

Sete mutações, sete sangram — mas a do dreno **sobreviveu à primeira versão do arch-gate**, e o
buraco era meu: o bloco tem um braço por eixo, então `contains("target - now[")` era satisfeito por
**um** deles. Trocar só o braço do X pelo alvo cru deixava o gate verde sobre exatamente o defeito
que ele nomeia. A asserção passou a pedir os **dois** eixos.

**Números da cena de smoke, medidos:** viga de `4,0` de mundo ⇒ os dois nós de baixo distam
**400 px**, a mediana deles está em **X = 0 px, Y = −50 px**, e o alvo do passo 3 é **150 px**
(escala de default, 100 px/m). Cena: **`PH2D_BUILD_SMOKE=73`**.

