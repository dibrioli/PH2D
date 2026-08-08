# Handoff de integração — `line/motion-value` (os PARÂMETROS dos nós)

> DIRETRIZ §1.5.9. **A linha NÃO integra e NÃO pusha** — este documento é o que o Enio passa ao
> agente integrador. Escrito por MEDIÇÃO: todo número aqui saiu de um comando, não de memória.

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/motion-value` |
| HEAD | `839188d6b` |
| Merge-base com `main` | `a4018d203` |
| Commits | **63** |
| Diff | **205 arquivos, +19125 / −2877** |
| Janela | 2026-08-05 → 2026-08-08 |

---

## 2. O que a wave entrega (quatro clusters, e eles são independentes)

**(A) A GPU do `source.object` + o VETOR VIVO** (`044abbf8e` … `ecb5232f2`, 4 commits) — o objeto
**cozinha e renderiza no device**; um `source.object` de VETOR renderiza *crisp* em vez de virar tile
raster; LOD híbrido + cache de tesselação por-frame. ⚠️ Isto **reabre** a recusa que o ADR-0155
instalou em 04/08 (*"um documento com fonte de APARÊNCIA recusa o cook GPU"*): a recusa continua,
mas agora só onde ela ainda é necessária — o gate `the_gpu_cook_recusal_placement` é quem pina
**onde** ela mora, e `gpu_texture_id` prova que o lowering do device escreve o id REAL.

**(B) O doc 88 — os parâmetros dos nós** (o corpo da wave, ~20 commits; plano novo em
`docs/Motion Nodes/88_plano_parametros_nos_unidades_e_slider.md`):

- o **vocabulário de UNIDADE** (`ParamUnit`) — *o que o número É*, nunca como se mostra;
- a **fronteira de DISPLAY** — o número sai UMA vez na face do artista e volta pela mesma porta;
- o **piso duro** por param (`ParamHardMin`) — e a assimetria morava em DOIS lugares;
- a unidade chega a **43 nós** por varredura de opt-in, com **censo que a tranca**;
- as **SEÇÕES** de params — a parede de treze sliders vira três perguntas (**10 nós**);
- o **reset ao default** (a seta que devolve o valor de fábrica);
- o **teto de linhas** do painel (escondia params: 8 contra os 13 do `field.remap`);
- o **oscilador** ganha régua de tempo; o **ruído** fecha o ciclo e o WGSL dele deixa de existir
  em três cópias.

**(B2) O SLIDER DUAL — a caixa vai ALÉM do slider** (`f60999baa` · `779cf4f9f` · `f9e6cf597`, o
item A1 do doc 88). O `ParamUiHint.max` passa a ser **só a faixa confortável do arrasto**, e o
`ParamHardMax` — canal *side-metadata* que já existia no registry — diz **onde o disfuncional
começa**: o número que a caixa de texto ainda aceita. Onze nós de contagem carregam hoje um teto
**MEDIDO**, com a tabela ao lado dele no doc-comment do próprio nó.

⚠️ **A feature NÃO funcionava em lugar nenhum, e três gates ficavam VERDES sobre isso.** O
espelho chip↔slider re-escrevia o chip com a re-projeção do slider **saturado**, e depois — já com
o chip certo — o slider **também** emitia, com o thumb parado no topo, e *o último vencia*. Os
gates de faixa do painel escrevem o valor com `set_number_value` e **nunca PINTAM**; a faixa
registrada e o link com o slider nascem no `paint`, então a fixture deles não continha o espelho
que diziam exercitar. A cura: a faixa do CHIP é a autoridade, e o evento do slider passa por uma
**porta única** (`push_mirrored_slider_event`) que se cala quando o thumb é um substituto saturado
— *dois sítios emitem esse evento hoje, e um `if` copiado é a regra que o terceiro nasce sem*.
Ferramenta nova que tornou o gate possível: **`MockPanelHost::type_into_number`** — o testkit só
sabia ESCREVER no store, que pula o caminho de commit inteiro. **Smokado pelo Enio.**

⚠️ **E a varredura dos tetos achou mais defeito na SONDA que nos nós** — a lei que sobra está
gravada em dois gates: **um teto digitável não pode passar do que o kernel HONRA** (`lattice` 400,
`kaleidoscope` 256; uma caixa que aceita 5.000 sobre um clamp de 400 *aceita e mente*). E o
`motion.boids` media 3,2 ns por agente porque **nunca dava um passo**: o estado dele chega por uma
aresta DELAYED que o editor auto-liga e que `Graph::add_node` não faz. Com o grafo certo o
quadrático aparece — 500 → 0,475 ms · 2.000 → 10,392 · 8.000 → **186,388**.

**(B3) O CORPO DO PAINEL ROLA** (`a27ee6e81`, o §B3 do doc 88). `MAX_PARAM_ROWS` respondia a
**duas** perguntas com um número: quantos ids o pool tem, e quantas linhas cabem na altura do
inspector. **Medido:** uma linha escalar ocupa **34,0 px** e o dock comporta **24** delas, contra
um teto de **16** e um pior nó (`motion.tint`) de **15** params — oito linhas de folga para uma
varredura que promete a TODO nó o conjunto PRO. O gate `a_full_panel_of_rows_fits_the_inspector`
já dizia o que fazer no dia: *"o painel precisa ROLAR antes de o teto subir mais"*.

⚠️ **O FATO FICA NOMEADO, não escondido: a rolagem está INERTE hoje.** O teto mora **dentro** do
`paint_rows` (`.take(MAX_PARAM_ROWS)`), então o corpo mede no máximo **~544 px contra um dock de
880** e nenhum nó transborda. Ela não é enfeite — é o que **remove o dock da lista de razões para
o teto não subir**, que é o pré-requisito da varredura B3 propriamente dita.

⚠️ **As QUATRO edições que um painel rolável exige** (o arch-gate `scrollable_panels_intercept_the_wheel`
as nomeia): id do thumb → arm no `scrollbar_panel_for_id` → o painter lê `panel_scroll` e publica
`content_h`/`visible_h` → **e o id no `cursor_over_hero_panel`**, que é a que *compila, pinta,
arrasta, e deixa a roda dando ZOOM na câmera em silêncio*. A mutação que tira a quarta produz
exatamente essa mensagem no gate.

⚠️ **E a cerca cuja premissa isto matou foi ENCARADA em vez de deixada verde:** o gate que afirmava
*"as linhas CABEM"* pediria um teto **MENOR** no dia em que uma não coubesse — o oposto da cura.
Ele virou **`the_reported_height_is_the_true_height_of_the_rows`**, a propriedade que passou a ser
load-bearing: o scrollbar deriva `max_scroll` de `content_h − visible_h`, e uma altura que
saturasse convenceria o painel de que tudo cabe — com as linhas desenhadas, o thumb ausente e **a
cauda perdida em silêncio**.

⚠️ **Uma mutação SOBREVIVEU e o defeito era da FIXTURE:** clampar a altura na altura do **dock**
passa despercebido, porque o cap do `.take()` mantém o conteúdo abaixo dela — só um clamp abaixo
de 544 é observável hoje. **E o nome que eu dei ao 1º gate de rolagem afirmava mais do que ele
mede** (*"mais conteúdo do que o dock mostra"*), corrigido para o que ele prova de fato.

**(B4) A VARREDURA POR FAMÍLIA COMEÇA — e um CENSO a escolhe** (`0e3e6270b` · `87d57bdcb`,
o §B3 propriamente dito; o mapa curado vive na **§9 nova do doc 88**).

A §6 do plano dizia *"nó a nó, ou família a família"*, e a §0 manda medir antes de decidir
⇒ a wave abre com uma **sonda de censo** (`param_census`, na `ph2d-node-registry-init` — a
crate que registra os 118 nós, e o build mais barato que os enxerga). **Retrato:
118 nós · 411 params · 395 com hint · 105 com unidade**; **um** nó tem param sem hint
nenhum e **cinquenta** têm ≤ 2 controles. ⚠️ *Magro por natureza* e *magro por omissão*
são coisas diferentes e o censo não as distingue — quem distingue é a referência, e é
disso que a §9 do doc 88 é a tabela (com o veredito **recusado-com-motivo** para as
famílias VALUE, ESTRUTURAIS e RIG, para ninguém as "completar" depois).

**Ele escolheu a família TRANSFORM, porque dois nós CONTRADIZIAM a coluna que escrevem:**

- **`motion.scale`** escrevia `size`, uma coluna **Vec2**, a partir de **UM** número.
  *Squash & stretch* não era difícil no grafo — era **inexprimível**. Agora `uniform` é o
  *link* de corrente do AE/Cavalry/Figma e destravá-lo revela `amount_y`.
- **`motion.mirror`** pregava a linha de espelho no **CENTROIDE**, então a simetria só
  sabia acontecer contra si mesma. `offset` move a linha — e é medido **a partir do
  centroide**, porque um offset absoluto não teria como exprimir *"no centroide"* e o
  zero deixaria de ser o comportamento antigo.

⚠️ **Os dois defaults são byte-idênticos ao que shipava** (`uniform` nasce ligado, `offset`
nasce zero), cada um reduzindo **literalmente** à expressão anterior — a demo de boot
sozinha põe treze `motion.scale`, e uma varredura de params é o que mais facilmente
destrói arte já autorada.

⚠️ **Duas lições de gate desta wave, as duas por MUTAÇÃO:**

1. **O gate de paridade GPU antigo era CEGO ao ramo novo.** Com o link no default o
   `select` do WGSL nunca entra no braço do segundo eixo — mutar o kernel para ignorá-lo
   deixa `scale_kernel_matches_the_cpu_within_epsilon` **VERDE** e só o irmão novo
   (`the_unlinked_scale_axes_match_the_cpu_within_epsilon`) sangra. *Um ramo de kernel que
   nenhum gate percorre é o que ship quebrado com a suíte verde.*
2. **Uma capacidade sem PORTA passa em todo gate de função pura.** Mutar o `eval` do mirror
   para não ler `ctx.param("offset")` deixa os cinco gates do kernel verdes; só o gate que
   atravessa o **COOK** sangra.

**A 2ª família — ECHO** (`5867f2c22`): o censo mediu `motion.trail` com **3 params contra
os 8** da referência, e o que faltava não era enfeite — um fantasma por TICK é um rastro
**contínuo**, que a 60 fps lê como borrão; o *sprite echo* (que o catálogo traz com
`spacing 2` no default DELE) era **inexprimível**. ⚠️ **O `spacing` não custou estado
novo**, e é o desenho inteiro: a coluna `trail_age` já sabia há quantos ticks o último eco
foi deixado, então a promoção da cabeça é uma pergunta ao **ESTADO**, não a um contador —
e com `spacing = 1` a faixa consultada é vazia, a promoção acontece sempre, e o motor é
**byte a byte** o que sempre shipou.

⚠️ **E o gate do SMOKE achou o que a suíte do nó não via:** com a janela ingênua
(`length × spacing`) o `length` passava a significar **duas coisas** — LINHAS em
espaçamento 1 e linhas + 1 acima dele. A janela correta é `(length − 1) × spacing + 1`, e
quem a pina é a **igualdade de CONTAGEM** entre as duas esteiras da cena. *Uma cena que
compara dois ajustes lado a lado mede o que uma suíte que olha um ajuste por vez não
alcança.*

⚠️ **O `hueShift` do mesmo catálogo NÃO entrou, com o motivo no próprio nó:** girar matiz
em RGB linear com uma matriz YIQ é o atalho que todo motor de partícula usa e que este app
**não usa em lugar nenhum** — a cor aqui passa por OKLCH. Seria uma segunda resposta a
*"o que é girar uma cor"*, divergindo no único lugar onde ninguém lê um número.

**(B5) O SMOKE REPROVOU, E OS TRÊS REPORTES FECHARAM** (`a690e470a` · `ba5f40ba9`).

**(i) *"Fade e Shrink não têm efeito algum"* — VERDADE**, e o mecanismo é exato: os dois
multiplicam colunas que **precisam existir**, e um stream posicional puro não carrega
`tint` nem `size`. Medido na cena real: as colunas eram `["Count","Index","P","trail_age"]`
— um `motion.grid`, a fonte mais comum que existe, não emite nenhuma das duas. ⚠️ **E o
gate PINAVA o defeito:** `a_bare_positional_stream_still_echoes` afirmava `tint.is_none()`
e explicava, no próprio doc-comment, que *"o fade/shrink simplesmente não têm o que
tocar"* — descrevendo como contrato exatamente o que o artista reportou como bug. A cura é
a do `motion.scale`: começar da **identidade que a própria lowering assume**, o que torna o
1º tick byte-idêntico no render.

**(ii) *"a transparência das cores não está sendo respeitada"* — VERDADE, e são TRÊS
degraus**, cada um jogando a alfa fora por conta própria: `serialize_gradient` (`g1`)
**dropava** a alfa (`let [r,g,b,_a]`) — a transparência era inexprimível **no formato**;
`apply_gradient_stop_pick` cravava `1.0`; e `apply_palette_pick` **descartava a alfa
escolhida** sob um comentário afirmando que *"o picker OKLCH é opaco"* — **ele não é**
(tem a 4ª linha de canal R+G+B+**A** e um campo `#RRGGBBAA`). ⚠️ *A premissa estava escrita
em dois lugares e cada um implementou um erro diferente por causa dela.* O formato ganhou
**`g2`**, com a versão **escolhida pelo conteúdo**: um ramp opaco serializa `g1` byte a
byte como antes, só quem usa a alfa paga o header novo. **Zero schema** — o gradiente
viaja como texto e carrega a própria versão.

**(iii) O padrão-ouro do rastro** (ordem: *"quero mais que os outros e não menos"*): de 3
para **SETE** knobs — `length · spacing · fade · shrink · hue_shift · saturation · spin` —
com as rows em três perguntas (soltas no topo · seção **Decay** · seção **Colour**).
⚠️ **Nenhuma referência tem `spin`.** E a minha recusa anterior do `hueShift` estava na
**camada errada**: *"a cor neste app passa por OKLCH"* é verdade da AUTORIA e falsa do
COOK, que é linear-RGB por construção — o operador certo é a rotação que **preserva a
luma** (o `feColorMatrix type="hueRotate"` do SVG, especificado em linearRGB pelo mesmo
motivo), com `sincos` **uma vez por tick** e as duas matrizes compostas numa só.

⚠️ **Dep NOVA:** `libm = "=0.2.16"` na `ph2d-node-motion-trail` — **mesmo pin** de
`ph2d-ecs`/`physics`/`wet-paint`/`platformer`: uma **aresta**, não um pacote novo.

⚠️ **Não construído, com o mecanismo:** `include_original` é **estruturalmente duro** (a
saída **É** o anel: esconder a cabeça do render a esconde do estado e o rastro morre) e a
**curva de cauda** pede que o desbote vire função da IDADE — as duas na §9 do doc 88.

**Smokes: `PH2D_TRANSFORM_SMOKE=1`** · **`PH2D_ECHO_SMOKE=1`** (duas esteiras iguais em
órbita, só o espaçamento difere — se os dois rastros forem iguais, o param não chegou).
O primeiro: — a cena monta 4 pontos esticados 2.2×/0.45× e
espelhados numa linha a 1.2 m do centroide (8 instâncias), com 4 testes na mensagem.
⚠️ **Se os oito saírem quadrados, PARE**: o eixo Y não chegou.

⚠️ **A cena `PH2D_ECHO_SMOKE=1` foi RE-AJUSTADA** para a lei nova e ganhou três testes novos na
mensagem: as duas esteiras levam agora o **mesmo** `Tail Alpha` e o **mesmo** `Tail Size` e têm de
**terminar no mesmo lugar** (sob a lei antiga a de cima terminava em 0,33 e a de baixo em 0,02 — um
fator de dezesseis); arrastar `Tail Saturation` de 1 a 0 tem de desbotar **progressivamente ao longo
de todo o curso**; e com um valor escolhido, arrastar o `Length` de 2 a 32 e o `Spacing` de 1 a 16
**não pode mudar o tom da ponta** — o que muda é quantos ecos há entre a cabeça e ela.

**Zero schema, zero contrato congelado, zero dep, zero `Cargo.toml`.**

---

**(C) A wave B** (`bd0bc6d7a` … `2378cfd10`) — a **paleta vira SWATCHES** (sem limite de
comprimento, por construção) e o **look-at ganha alvo por NOME e pelo CURSOR**. Inclui o fix do
**drift crônico do Motion** (o cursor era projetado pela janela CHEIA — **terceira vez** que este
defeito aparece no módulo).

**(D) O painel e a row dirigida** (`178fab5b1` … `9f1b8ff63`, 5 commits) — o editor **abre VAZIO**
(a neve sai do boot e vira fixture `#[cfg(test)]`), o painel **prova que CABE** no dock, e a **row
dirigida diz QUEM a dirige** (elo + nome do card, pela porta única `card_title`).

**(E) O BALANCEAMENTO DOS SLIDERS** (2 commits; report do Enio de 2026-08-08 — *"sliders mal
balanceados… Saturação 0.9 já fica quase todo dessaturado. Reveja tudo"*; plano §10 do
[doc 88](../docs/Motion%20Nodes/88_plano_parametros_nos_unidades_e_slider.md)) — os cinco knobs
de decaimento do `motion.trail` eram **taxas por tick**, e o preço está medido: na esteira do
próprio smoke (vão 20) `saturation 0.90` entregava **0,17** na ponta, e a faixa útil inteira do
controle cabia em **5,2%** do curso dele (1,9% a `spacing 8`). ⚠️ **São DOIS defeitos:** a resposta
era exponencial no slider **e** o `spacing` multiplicava todo decaimento — a nota do módulo que
dizia *"a semântica «por eco» cai de graça"* só era verdade em `spacing 1`, e a const do smoke
chamada `HUE_PER_ECHO` valia na verdade **700° no total**.

Agora os knobs são **ALVOS na ponta da cauda** (o `satMin/satMax` do catálogo de referência, que a
wave anterior citou e não seguiu): o motor segue geométrico e a TAXA é **derivada** do alvo
(`rate^vão == target`). O slider fica linear no que se vê, **o número não se move quando o Length
ou o Spacing mudam**, e os ângulos viram totais percorridos. ⚠️ **Os defaults não foram escolhidos**
— `0.10`/`0.65` são o que as taxas `0.72`/`0.94` que já shipavam produziam no default do nó, então
o rastro no default é o mesmo. ⚠️ **Piso de `1/255`** (um nível de 8 bits, o número do renderer):
sem ele um alvo de zero faz a taxa ser zero e a cauda colapsa no PRIMEIRO eco.

⚠️ **E a medição achou no ANEL algo que PRECEDE esta wave:** com `spacing > 1` a idade do eco mais
velho **CICLA** numa faixa de `s − 1` ticks (13↔14 a `sp 2`, 49..56 a `sp 8`), com a contagem
estável — a lei antiga oscilava junto. Está pinado num gate próprio para ninguém o ler como
regressão; curá-lo é o item da CURVA de cauda (decaimento por IDADE, ~28 B/linha).

**A varredura que o *"reveja tudo"* pede** separa duas classes: `damping`/`friction`/`tension` são
**FÍSICA** (por-tick É o modelo) e ficam; o **`motion.strobe.decay`** tem a doença idêntica — e o
doc-comment dele já confessava a tradução (*"0.85 ≈ a ~0.2 s flash at 60 Hz"*): **86%** do curso
cobria 5..34 ticks e **14%** cobria 34..551. Virou **`Flash Length` em TICKS** (default 34).
⚠️ **RECUO REGISTRADO:** a v1 usava SEGUNDOS via `ctx.dt()`, e `dt` é `0.0` num time scope — um
`dt` errado faria a taxa virar `1.0`, isto é **o flash nunca apagaria**. Ticks não dependem de nada.
⚠️ O param **mantém o nome de fio `decay`**: renomeá-lo faria o `validate` recusar todo grafo salvo
que o sobrescreve (a cicatriz do `motion.color_ramp`).

⚠️ **Vermelho-latente curado de carona:** `ph2d-node-motion-trail/src/lib.rs` estava em **987 > 700**
e não está na allowlist — o gate mora na `ph2d-editor-core`, então uma bateria por-crate **nunca o
alcança** (a mesma causa estrutural que esta linha já documentou três vezes). Split por assunto em
`lib_tests.rs` (a mecânica do anel) + `tail_target_tests.rs` (a lei do alvo), **filhos** por
`#[path]` para o `use super::*` seguir alcançando os privados.

---

**(F) O WIDGET DIZ O QUE O NÚMERO É** (`5a5678007`; doc 88 §11.1). Todo param é um `f32` no
`NodeManifest` — **contrato CONGELADO** —, então o TIPO não distingue uma magnitude de uma
escolha nem de uma semente; quem distingue é o `ParamWidget`, que é **side-metadata**. ⚠️ **E
side-metadata sem gate apodrece por STRAGGLER, com número:** a varredura mediu **11 nós pintando
`seed` como campo de dado e 4 como slider**, e **17 pintando `mode` como palavras e 1 como o
número cru**. Curados: os quatro `seed` (`field.remap` · `value.instance_field` · `value.noise` ·
`motion.distribute_poisson`) — *arrastar uma semente não quer dizer nada: toda vizinha é tão boa
quanto ela, e o que o artista quer é OUTRA* · o `motion.delay.mode` vira **Enum** com as palavras
que o próprio `ParamSpec` já nomeia · e o `motion.boids.spread` vira **Toggle**, porque o `eval`
o lê como bool (`> 0.5`) e o comentário ao lado dele já dizia *"a 2-position slider"*.

⚠️ **A 3ª lei do gate novo é a mais forte por ser PROPRIEDADE em vez de nome:** *um slider cujo
passo mede o curso inteiro tem duas posições* — nenhum nome precisa ser conhecido para ela morder.
E `rig.skin_deformer.falloff` aparece na varredura **por nome e NÃO é defeito** (ali o falloff é
um expoente, magnitude de verdade), que é por que o gate casa **nome exato** e nunca família.

**(G) O SLIDER ARRASTA ONDE A MÃO TRABALHA; A CAIXA DIGITA ATÉ O TETO** (`ba3c2aba3` ·
`5c6785342`; doc 88 §11.2-§11.3). A generalização do A1 (o slider dual) do onze para o
**catálogo inteiro**: **24 params em 18 crates** — o teto de hoje migra para `ParamHardMax` e o
slider baixa para a faixa de AUTORIA. ⚠️ **Nada fica inalcançável**: a caixa numérica ao lado já
digita até lá.

⚠️ **A barra é DERIVADA da geometria, não escolhida:** o track do painel mede **~154 px**
(`inner_w` 320 − label 70 − caixa 72) e o mapeamento é estritamente linear ⇒ `span / 154` **é o
menor passo que um arrasto consegue**, e acima de `span / default = 154` esse passo mínimo passa
do próprio default. Com o `emitter.max` arrastando até **4.194.304, um pixel valia 27.000
partículas** e o default de 512 não cabia no primeiro cinquentavo de pixel. A régua sai de **28
params fora da linha para QUATRO**, e os quatro são `IntSlider`s cujo curso cabe no track (todo
inteiro alcançável, que é a propriedade que importa numa contagem).

⚠️ **Duas cercas de Chesterton PRESERVADAS em vez de revogadas:** o `emitter.max` era `MAX_ALIVE`
**de propósito**, contra a divergência que o comentário nomeia — a derivação não sumiu, mudou
para o campo que quer dizer TETO (o `voronoi.count` ganhou a mesma cura); e o gate
`the_mesh_sliders_reach_exactly_the_clamp` do `soft_body` afirmava *capacidade-alcançável* **e**
*slider-arrastável* com um número só — agora afirma as duas, **cada uma do seu lado do par**.

⚠️ **E o emitter tinha os dois knobs descrevendo CENAS diferentes:** o comentário do `rate` dizia
*"12.000/s é uma fonte densa a 1 s de vida"* e o `life` tem default **3**, onde isso são 36.000
vivas — **setenta vezes** o `max` default de 512. Agora `1.200 × 3 = 3.600` cabe no `max` de 4.096.

⚠️ **O 3º gate é o MODO DE FALHA deste padrão:** `static PARAM_HARD_MAX` **declarado e nunca
registrado** — a tabela existe, o `cargo` não reclama (o próprio crate a lê em gates), e o painel
nunca a vê: o artista fica com o slider estreito **e** sem o teto digitável, que é pior que não
ter feito a wave. **Hoje 25 crates-nó declaram e registram** (o 26º hit do grep é o gate).

**(H) A CENA DO EMITTER** (`839188d6b`) — pedido do Enio no smoke: *"Emitter não sei se funciona
corretamente"*. ⚠️ **E ela achou um defeito que os gates de CONTAGEM não podiam ver:** a 1ª versão
da sonda cozinhava sem `Cook::advance_tick`, e **toda partícula ficava no bico** — `EvalCtx::dt` é
`playhead − prev_playhead`, e o `prev_playhead` só existe depois do `advance_tick`, que é também
quem carrega as arestas `delayed`. Os três gates de contagem ficavam **VERDES sobre essa cena
imóvel**, porque o `motion.emitter` é **stateless** e conta certo com o laço parado — quem estava
morto era o `motion.integrate`, e **nada que contasse partículas podia perceber**. *O oráculo tem
de ser o LUGAR delas*, e é isso que o `the_fountain_flies_and_stays_in_frame` afirma (ele é
também o gate de ENQUADRAMENTO: as outras cenas deste painel vivem em `|x|,|y| ≲ 1,5`, e um jato
que sai da moldura faz o artista julgar um terço do que a mensagem descreve).

**Zero schema, zero contrato congelado, zero dep, zero `Cargo.toml`** nos três clusters.

---

## 3. Foundational / compartilhado tocado, e por quê

Tudo **aditivo** salvo onde marcado.

| Arquivo | O quê | Forma |
|---|---|---|
| `ph2d-color/src/palette_text.rs` **(NOVO)** + `lib.rs` (+2) | o formato TEXTO de uma paleta, ao lado do do gradiente — *uma crate é dona de como uma cor se escreve* (precedente: `motion-color-ramp`) | arquivo próprio ⇒ isolado por construção |
| `ph2d-editor-core/src/screens/layout.rs` | **`INSPECTOR_MAX_H`** — const `pub` NOVA (era literal solto no `Rect::new`) | ⚠️ **símbolo novo, ver §4** |
| `ph2d-editor-core/src/project.rs` | ⚠️ **SÓ doc-comments.** Os dois que contradiziam o `Default` real | **zero mudança de comportamento — ver §6** |
| `ph2d-editor-core/src/interaction/dispatch/{mod,tick}.rs` | ⚠️ **MUDANÇA DE COMPORTAMENTO** no espelho chip↔slider: a **faixa do chip é a autoridade** (um valor digitado além da trilha sobrevive) e o evento do slider sai por **`push_mirrored_slider_event`**, que se cala com o thumb saturado | `pub(super) fn` nova + os 2 sítios de emissão roteados por ela |
| `ph2d-editor-core/src/widget/scrollbar.rs` + `widget/mod.rs` | **`MOTION_PARAMS_SCROLLBAR_ID = NodeId(839)`** — ⚠️ **símbolo colidível**; próximo livre **840** | aditivo (const nova + re-export) |
| `ph2d-editor-core/src/interaction/dispatch/scroll.rs` | o braço que ARMA o painel para a roda | aditivo (um `match` arm) |
| `shells/desktop/src/forwarding.rs` | `MOTION_PARAMS_PANEL` no `cursor_over_hero_panel` | aditivo — **sem ele a roda dá ZOOM** |
| `ph2d-ui-testkit/src/lib.rs` | **`type_into_number`** — digitar de verdade (foco → `dispatch_text_input` por caractere → Enter) | aditivo (`pub fn` novo) |
| `ph2d-nodegraph/src/graph.rs` | `clear_param` / `clear_text_param` | aditivo (`pub fn` novos) |
| `ph2d-nodegraph/src/external.rs` | o **namespace reservado `$`** (`RESERVED_PREFIX`, `is_reserved`, `CURSOR`, `position_of`) — o alvo do look-at pelo cursor | ⚠️ **símbolos novos, ver §4** |
| `ph2d-node-registry/src/` (+ `unit.rs` NOVO) | **6 canais side-metadata** novos: `param_units` · `param_groups` · `param_hard_min` · `live_vector_source` · `object_source` · `card_title` | **o padrão canônico** — nenhum toca `NodeManifest` |
| `ph2d-render/` (`clip_pass`, `renderer_draw`, `sprite/instance`, `sprite/mod`) | os **texture runs** do draw extra da GPU | gate próprio novo |
| `ph2d-vec-render/` (`instance.rs` NOVO, `lib.rs`) | o **vetor vivo** do `source.object` | |
| `ph2d-gpu-cook/` (`tex_runs.rs` NOVO + 6 arquivos) | o lowering do objeto no device | |
| `ph2d-panel-motion-graph/src/snapshot_build.rs` | passa a chamar `card_title` | **porta única** (era escada de fallbacks duplicada) |
| `shells/desktop/` | o bridge de params partido em duas metades, os censos, as 4 cenas de smoke, o schema | o grosso do diff |

**Nenhuma crate nova.**

---

## 4. Símbolos que podem COLIDIR (literais, para o integrador grepar)

| Símbolo | Valor | Onde |
|---|---|---|
| ⚠️ **`PROJECT_SCHEMA`** | **56** (main dizia **55**) | `shells/desktop/src/project.rs:247` |
| `INSPECTOR_MAX_H` | `f32 = 880.0` | `ph2d-editor-core/src/screens/layout.rs` |
| `RESERVED_PREFIX` | `char = '$'` | `ph2d-nodegraph/src/external.rs` |
| `CURSOR` | `&str = "$cursor"` | idem |

⚠️ **O `56` é PROVISÓRIO e se CONTA, não se escolhe** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
Ele carrega **um** degrau: `ProjectFile.settings` (`SavedSettings` — a escala e a unidade do
projeto passam a viajar no arquivo). Se outra linha da janela também bumpar, o valor certo é
contado a partir do `main` do dia — e ⚠️ **este é o caso que já passou MUDO três vezes no repo**:
duas linhas escrevendo o mesmo literal **não conflitam no git**, porque o git não tem opinião
sobre o que o número significa. O sinal é o conflito no `project_schema_tests.rs` ao lado.

**Não há:** `NodeId(NNN)` numérico novo · chave i18n nova · token novo · id de gizmo novo.

⚠️ **Os clusters (F)/(G)/(H) não acrescentam NADA a esta tabela** — conferido: zero `Cargo.toml`,
zero símbolo novo, `PROJECT_SCHEMA` segue em **56**. Eles mexem em `PARAM_HINTS` /
`PARAM_HARD_MAX` (side-metadata do registry, uma tabela `static` por crate-nó) e acrescentam
`shells/desktop/src/emitter_smoke.rs` **novo** + duas linhas de fiação (`mod` no `main.rs`, a
chamada no `render_loop/mod.rs`) — as mesmas duas linhas que toda cena de smoke desta shell tem.

**`Cargo.toml` tocados — 3, todos arestas de PATH, zero pacote externo novo:**

- `ph2d-gpu-cook` → `ph2d-node-source-object` em **`[dev-dependencies]`** (só o gate de paridade;
  o `src/` não o usa ⇒ **machete-safe**, o padrão das 5 crates-nó de 23/07);
- `ph2d-node-motion-color-array` → `ph2d-color` (dep real: o formato texto da paleta);
- `shells/desktop` → **o bloco `[dev-dependencies]` é o PRIMEIRO da shell** (`ph2d-ui-testkit`,
  para o censo de ALTURA medir os retângulos que o painel de fato registra).

---

## 5. Contratos congelados (§4) — **nenhum encostado**

Rodado, não auto-relatado:

```
cargo test -p ph2d-nodegraph  --test architecture_contract_surface       → 3 passed
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface → 4 passed
```

`NodeOp=2` / `OpResolver=1` / `NodeManifest=8` e `Tool=12` / `RasterEditTool=5` /
`CanvasPaintTool=1` / `PanelEvent=4` intactos. **Nenhum ADR novo.** É o que os 6 canais
side-metadata do §3 compram: todo fato novo sobre um param mora no REGISTRY, nunca no manifesto.

---

## 6. ⚠️ Duas coisas que um integrador vai ler errado se este parágrafo não existir

**(a) `ph2d-editor-core/src/project.rs` parece trocar dois defaults de produto. NÃO troca.**
O diff mostra `Meters → Pixels` e `PixelArt → Smooth`, mas **só nos doc-comments**: o `impl
Default` real já dizia `Pixels` e `Smooth`, e é **byte-idêntico ao `main`** (medido). Os
comentários é que estavam mentindo. Commit `5bc53584e`, e ele é `docs(...)` de propósito.

**(b) `1735bc726 style(fmt)` é drift PRÉ-FORK**, não formatação desta wave — sete arquivos que o
`ship.sh` acusaria como vermelho latente. Se o rebase conflitar ali, o lado do `main` ganha.

---

## 7. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **machete** — as 3 arestas novas do §4. As três são usadas; machete é quem confirma, e o caso
  de risco é o `[dev-dependencies]` da `gpu-cook` (usada só por `tests/`).
- **typos** e **fmt do repo inteiro** — inclusive o drift pré-fork do §6(b).
- **clippy `--all-targets --all-features`** — a linha rodou `--all-targets` no que tocou; a
  matriz de features não.
- **RUSTSEC / `cargo deny`** — nenhuma dep externa nova, então o risco é herdado do `main`.

⚠️ **E um `✗` que NÃO é do código, medido nesta máquina em 2026-08-08:** o `/home` estava em
**946 G de 950 G (132 M livres)** e a suíte parou com `mold: failed to write to an output file.
Disk full?` → `clang: Bus error` em `ph2d-host-desktop` e `ph2d-timeline`. **Nada disso é a
linha** ([[feedback_a_ship_x_can_be_the_environment_not_the_code]]). O combustível são os
`target/` das worktrees — medidos: **Painter 264 G · physics 205 G · Vector 162 G ·
motion-value 100 G · sculpt3d 40 G · runtime 28 G = ~799 G**. Liberei **só o meu**
(`target/debug/incremental`, 24 G, cache regenerável da minha própria árvore) e não toquei em
árvore de outra linha. **Se o `ship.sh` do integrador falhar no LINKER, cheque o `df` antes de
ler o erro como código.**

---

## 8. Ordem, dependências e o que smoke-testar

**Ordem:** os 63 commits são sequenciais e o rebase deve preservá-los. O cluster **(A)** (GPU do
objeto) é **independente** de (B)/(C)/(D); (B) → (B2) → (C) → (D) → (E) → (F) → (G) compartilham o
painel `motion-params` e o bridge, então **não reordene**.

**Smokes (todos `--release`, da worktree):**

| Cena | Comando | Estado |
|---|---|---|
| Unidades nos params | `env PH2D_UNITS_SMOKE=1 cargo run -p ph2d-host-desktop --release` | aprovado |
| Régua do oscilador / loop | `env PH2D_OSC_RULER_SMOKE=1 …` | aprovado |
| Objeto/vetor vivo na GPU | `env PH2D_MOTION_OBJ_SMOKE=1 …` | aprovado |
| Caminho de nós | `env PH2D_MOTION_NODE_PATH_SMOKE=1 …` | aprovado |
| **Row dirigida** | `env PH2D_DRIVEN_ROW_SMOKE=1 …` | **aprovado 2026-08-07** |
| **Slider dual** | a MESMA cena: `Grid` → caixa **Rows** → digite `5000` → Enter | **aprovado 2026-08-08** |
| Transform (espelho + eixo Y) | `env PH2D_TRANSFORM_SMOKE=1 …` | aprovado |
| Echo / rastro | `env PH2D_ECHO_SMOKE=1 …` | **aprovado 2026-08-08** |
| **A FONTE (emitter)** | `env PH2D_EMITTER_SMOKE=1 …` | **aprovado 2026-08-08** |

⚠️ **A cena do emitter é onde a régua da wave (G) se julga:** arraste **Rate** (default 40, teto
de arrasto 1.200) e o jato adensa **sem bater no pool**; baixe **Max** (default 4.096) a ~300 e o
jato **corta no ar**; e digite `20000` na caixa de **Max** — o teto DURO continua digitável.

⚠️ **O slider dual se julga digitando, nunca arrastando.** O número tem de **FICAR** (a fileira
cresce para 5.000 e o thumb estaciona em 20, que é a ponta da faixa confortável). E o controle da
outra ponta é o **`Scatter`**: ali digitar `50000` ainda clampa em **3.000**, porque aquele teto é
um RECURSO medido (o quadro quebra entre 3.000 e 4.000), não ergonomia.

⚠️ **A cena da row dirigida imprime o que montou.** Se a linha `[driven-row smoke]` não aparecer,
pare: o resto do smoke não diz nada.

⚠️ **O que MUDA para quem abre o app e não roda smoke nenhum: o editor de Motion abre VAZIO.**
A neve (`motion_demo_strobe`) saiu do boot por ordem do Enio (*"tire a cena da cachoeira"*) e virou
fixture `#[cfg(test)]` — ela **não foi deletada**, e `MotionState::with_snow()` é a porta única que
a monta para os gates que dependiam dela. Um integrador que abrir o app e vir tela vazia está
vendo o produto correto.

---

## 9. Gate de fechamento (rodado nesta worktree)

- **`cargo test --workspace` → 12.851 passed / 0 failed** (rodado em 2026-08-07, sobre o cluster
  (E); ⚠️ **a re-corrida de 2026-08-08 NÃO fechou por DISCO** — ver o aviso abaixo)
- `cargo fmt --check` nas crates tocadas → **limpo (exit 0)**, conferido depois do `2a5b67db2`
- `cargo clippy --all-targets` na shell + nas crates-nó da varredura → limpo
- `cargo test -p ph2d-host-desktop` → 2.439 passed / 0 failed
- **Contratos congelados rodados hoje:** `architecture_contract_surface` **3 passed** ·
  `architecture_tool_contract_surface` **4 passed**
- LOC: todo arquivo tocado sob o teto (`emitter_smoke.rs` **319** · `param_range_conventions.rs`
  **159** · `param_widget_conventions.rs` **119** · `param_census.rs` **101**)

⚠️ **A re-corrida de `cargo test --workspace` de 2026-08-08 parou em `mold: failed to write to an
output file. Disk full?` → `clang: Bus error`**, com o `/home` em **946 G de 950 G**. É a mesma
falha do §7, e ela **não é da linha**: as crates que caíram (`ph2d-timeline`,
`ph2d-panel-motion-params`, `ph2d-host-desktop`) morreram no LINKER, não no compilador. Liberar
espaço é decisão do Enio — eu limpei **só o meu** `target/debug/incremental` (24 G) e o rebuild
o consumiu de volta em uma corrida.

⚠️ **Rode a suíte, não o `cargo check`.** Esta wave deu verde no `cargo check -p` sobre **dois
erros de compilação** — ele não compila código `#[cfg(test)]`, e a sonda de medição vive lá.

⚠️ **E NÃO leia o exit code através de um pipe.** Custou uma rodada nesta sessão: um
`cargo fmt --check … | head -5` imprime o diff **e** devolve `0` (o exit do `head`), então o
vermelho latente do `2a5b67db2` passou por verde. Redirecione para arquivo e leia o `$?`
([[feedback_pipe_masks_script_exit_code]]).

---

## 10. Aberto e NOMEADO (não é dívida escondida)

- **O doc 88 não fechou inteiro, mas o A1 (o slider dual) FECHOU** — mecanismo, produto e
  varredura. Onze nós de contagem carregam teto **medido**: `motion.grid` rows/cols ·
  `motion.fibonacci` · `motion.distribute_radial` · `motion.distribute_curve` → **1.000.000** ·
  `motion.pin_constraint` first/count → **1.000.000** (o eixo é PLANO: 0,534 / 0,490 / 0,516 ms) ·
  `motion.clone` → **10.000 CÓPIAS** (1 M instâncias, 4,05 ms) · `motion.verlet_rope` → **50.000**
  (linear) · `motion.scatter` → **3.000** e `motion.boids` → **2.000** (os dois O(n²)) ·
  `motion.lattice` → **400** e `motion.kaleidoscope` → **256** (ali o teto **é o clamp do kernel**).
- ⚠️ **Quatro nós ficam de FORA da varredura, cada um com o motivo escrito:** `motion.wave` e
  `motion.soft_body` já têm **soft == clamp do kernel** (60 e 512), então não há folga a abrir sem
  mexer no KERNEL — o que é wave própria, com a medição de `rows × cols` ao lado; e o `steps` de `field.remap` e
  `value.pattern` **não é uma contagem que carrega recurso** — no `value.pattern` ele é
  estruturalmente limitado pelos oito slots declarados (`max: SLOTS`), e no `field.remap` é uma
  quantização do falloff, custo O(1) por elemento. *Um teto sobre param que não carrega recurso
  seria cerimônia; e no `value.pattern` o soft JÁ é o limite estrutural.*
- ⚠️ **`motion.voronoi` fica NOMEADO e não capado**, de propósito: o soft dele é **165.000** (medido
  pela linha da GPU) e o cook de **CPU** já passa o quadro em ~1.500 (8.000 = 259 ms). Derivar um
  teto do caminho de REFERÊNCIA seria deixar o mais lento definir o teto do mais rápido — o erro
  que o §0 nomeia. Quem quiser fechar isto precisa medir o **device**, não a CPU.
- ⚠️ **E a sonda de contagem mentiu na 1ª versão** — vale ler antes de escrever a próxima: ela dava
  `0,00 ms` em toda célula enquanto o teste levava **1402 s**, porque o `Cook` **memoiza** e eu
  descartava a 1ª corrida (a cautela de *first-touch*). *A lição certa noutro lugar cega o
  instrumento*, e quem denunciou foi o relógio de parede. Ela hoje carrega um CONTROLE próprio.
- **O `value.gain` da cena de smoke ensina uma armadilha real e vale reler:** ele opera em `[0,1]`
  e **clampa**, então alimentá-lo fora da banda o torna mudo (a cena v1 fazia isso e o fio ficou
  inerte com a suíte verde). Quem for construir cena com ele: `map_range` antes e depois, como a
  doc do nó prescreve.
- ⚠️ **A lição de gate desta wave, para o integrador não repetir:**
  `the_wire_actually_moves_the_scene` media a extensão em Y — que só o fio da AMPLITUDE move — e
  ficou **verde sobre um fio de frequência morto**. *Um gate que mede uma metade fica verde sobre
  a outra morta.* Os dois gates que faltavam existem agora
  (`the_frequency_wire_walks_over_the_cycle`, `the_drivers_knob_steers_the_wire`).

---

- ⚠️ **A varredura B3 fechou UMA família (TRANSFORM) e o mapa das demais está na §9 do
  doc 88, com o veredito de cada uma** — inclusive as três **recusadas com motivo**
  (VALUE, ESTRUTURAIS, RIG), que existem para ninguém as "completar" por parecerem magras.
  **Duas fechadas** (TRANSFORM · ECHO); a próxima aberta é **DEFORMERS** (`spherize` e
  `slit_scan` com 1 param cada — a medir se são magros por natureza).
- ⚠️ **O passe VISUAL do painel (a §11) fechou em três leis, e as três têm gate com controle
  positivo:** *o widget diz o que o número É* (F) · *o slider arrasta onde a mão trabalha* (G) ·
  *declarar não é registrar* (G). O que **não** foi varrido é a terceira pergunta da §11 — o
  **AGRUPAMENTO** (quais params merecem seção própria) —, que é decisão por-nó e não tem
  propriedade mecânica que um gate possa afirmar: um gate ali seria cerimônia.
- ⚠️ **Os QUATRO params que a régua ainda reporta fora da linha ficam de propósito**, e a razão é
  que a propriedade que importa numa contagem é *todo inteiro é alcançável* — os quatro são
  `IntSlider`s cujo curso inteiro cabe no track. Baixá-los tiraria alcance sem comprar precisão.
- ⚠️ **E o `motion.duplicator` tem ZERO params, o que está QUASE todo certo** — a tabela
  do Cavalry é grande e a leitura ingênua ("faltam sete params") é falsa: *Distribution*
  são 21 nós aqui, os transforms por-cópia são `motion.move`/`rotate`/`scale` a jusante, e
  *Skip Invisible* é o `motion.cull`. O gap REAL é **um**: *que forma vai em que ponto*
  (nós fazemos o produto cartesiano; a referência cicla ou fixa por id) — semântica de
  contagem, não um knob, ⇒ wave própria.

---

**Resumo:** linha `motion-value` pronta (HEAD `839188d6b`, **63 commits**, 205 arquivos,
+19125 / −2877). Foundational tocado é aditivo salvo os doc-comments do §6(a); símbolos colidíveis
são `PROJECT_SCHEMA = 56` (**provisório**), `INSPECTOR_MAX_H`, `RESERVED_PREFIX` e `CURSOR`;
contratos congelados **3/3 + 4/4 verdes**; zero pacote externo novo, zero crate nova, zero ADR.
**Todos os smokes aprovados pelo Enio.** **Aguardo ordem de integração.**
