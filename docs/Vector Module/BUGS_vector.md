# BUGS — Vector

Registro dos bugs do módulo Vector: o **sintoma** que o Enio viu, a **causa real** (que
quase nunca é a primeira suspeita), a **correção** e o **gate executável** que impede a
volta. Espelha o [`docs/Painter/BUGS_painter.md`](../Painter/BUGS_painter.md).

Regra deste arquivo: um bug só é considerado fechado quando existe um teste que **falha**
se a correção for revertida. "Não reproduzi mais" não fecha nada.

> **O que está VIVO aqui:** os **27 bugs estão TODOS fechados**, então o que vale hoje não é
> nenhum deles — é o que eles ensinaram. Ficaram, nesta ordem: as **recusas com medição atrás**
> (⛔ — não as reconstrua), os **padrões que se repetem** (leia ANTES de caçar o próximo) e o
> **índice de uma linha por bug, com o MECANISMO**.
>
> O post-mortem completo de cada um foi movido **verbatim**, em 2026-08-18, para
> [`docs/archive/docs-2026-08-18/Vector Module/BUGS_vector.md`](../archive/docs-2026-08-18/Vector%20Module/BUGS_vector.md)
> — vá lá pela seção `## Bug #N` quando a linha do índice bater com o que você está vendo.
> ⛔ Nada foi resumido: as duas metades remontam o original byte-a-byte (sha256).

---

## ⛔ As recusas e as leis que NÃO se remexem (com o motivo medido)

⚠️ **A arquitetura foi decidida pelo COMPILADOR, não por gosto:** montar o offset na pilha de Live
Path Effects **não compila** — um efeito é avaliado dentro da `ph2d-vec-scene` (crate pura) e o
offset precisa da `ph2d-vec-boolean`, que depende dela; `error: cyclic package dependency`. A saída
"registrar o motor por ponteiro global" foi **recusada**: faria `cooked()` — a resposta a *"o que
este documento desenha?"* — depender de alguém ter chamado um instalador, e o mesmo arquivo
desenharia diferente em processos diferentes, **em silêncio**.

⚠️ **A razão é formada como `h / ROW_H_PX`, nunca `CHECKBOX_BOX_PX / ROW_H_PX`,** e a ordem é
load-bearing: `h / h` é `1.0` EXATO em IEEE-754, então a identidade é **por construção**. (Com os
valores de hoje — 18 e 28 — a forma ingénua também acerta, *por acidente aritmético medido*.)

⛔ **E a saída (1) — escalar o fragmento por um `Affine` — fica REJEITADA com o motivo:** ela move a
caixa **e** o raio de canto, e um raio é um token que não se mede em frações de moldura. O que a
pele faz é o que os outros dez já faziam: a moldura cresce, os tokens de detalhe (canto, borda,
corpo da letra) ficam em px. Um widget grande é um widget grande, não uma foto ampliada de um
pequeno.

### O que NÃO fazer

⛔ Não "consertar" o `CHECKBOX_BOX_PX` nem o teto do slider. Eles governam **todos os painéis do
app**; mexer neles para agradar ao canvas re-dimensiona a interface inteira — a definição de mover o
número do consumidor errado. **A correção não os tocou:** `None` continua a ser a lei deles, e a
mutação que tira o teto da rota do painel derruba **três** gates.

---

## Padrões que se repetem (leia antes de caçar o próximo)

1. **O sintoma quase nunca é a causa.** "Cone de cabeça para baixo" era uma convenção de
   eixo em quatro módulos. "Panic no clamp" era uma janela geométrica que colapsa.

2. **Teste que não morde é pior que teste nenhum** — dá confiança falsa. Antes de aceitar um
   teste de propriedade, **quebre a propriedade de propósito** e confirme que ele fica
   vermelho. Três bugs desta lista foram encontrados assim, e não pela leitura do código.

3. **Varra a INTERAÇÃO, não os eixos.** Bugs de parâmetro moram nas combinações. Um gate que
   move um slider por vez é barato, roda rápido e **não encontra o bug que derruba o editor**.

4. **Renderize.** O agente do padrão-ouro dos balões descobriu três defeitos *desenhando* as
   formas, com todos os testes verdes. O do banner idem — a dobra tinha um buraco e nenhuma
   asserção olhava para lá.

5. **Uma fronteira mundo↔tela é sempre DUAS.** O Bug #2 (formas espelhadas) e o Bug #6
   (previews espelhados) são o mesmo `−1` faltando, em dois lugares diferentes. Ao corrigir
   uma travessia de coordenadas, **procure as outras**: quem mais pinta geometria de mundo
   sem passar pela câmera? (Resposta: o painel. E qualquer export futuro.)

6. **Quando um subsistema discorda dos outros, ele é o errado — mas confirme.** O hit-test
   já pulava contornos abertos; o renderer não. Duas implementações da mesma pergunta
   ("o que é o interior desta forma?") com respostas diferentes é sempre um bug. A maioria
   costuma estar certa, mas o valor está em *notar a discordância*, não em contar votos.

7. **Um parâmetro que nenhum teste exercita não está implementado — está escrito** (Bug #7).
   O sinal é visível a olho nu na suíte: se **todos** os testes passam o mesmo valor para um
   campo (`spread: 0.0`, seis vezes), esse campo nunca foi testado. Grepar o nome do parâmetro
   nos testes é mais rápido que ler a implementação, e encontra a mesma coisa.

8. **A FIXTURE é parte do gate, e é a parte que ninguém audita** (Bugs #12-#14). Dezesseis
   gates verdes usavam quadrados eixo-alinhados, construídos à mão, na identidade — enquanto
   o produto entrega formas do **catálogo** (curvas), **centradas no local 0**, com a pose num
   `Transform` (ADR-0111). Mutar o código e ver vermelho **dentro de um universo de quadrados**
   só prova coisas sobre quadrados. Duas perguntas, antes de confiar numa suíte:
   *quantos gates usaram uma forma do catálogo?* e *quantos usaram um `Transform`?* Se a
   resposta for zero, a suíte não fala do seu produto.

9. **Instrumente o app antes de teorizar.** Os três bugs acima foram achados montando a cena
   do print DENTRO do app (`PH2D_BUILD_SMOKE`), dirigindo o gesto no frame de verdade e
   olhando a tela — em ~20 minutos. A hipótese principal do handoff anterior (xform stale)
   estava **errada**, e o gate novo do arranjo nasceu **verde**. Uma tarde de leitura de
   código não teria chegado lá.

10. **O DIAL pode ser o bug** (#18). Quando o mecanismo de X sai inocentado de teste após teste e o
    usuário insiste que "X não funciona", pergunte **onde o gesto natural estaciona o parâmetro** — e
    se ALI X é inerte por construção (aniquilação, fora da tela, clamp). O relato então é verdadeiro
    e o mecanismo também: o que está errado é o mapa do controle. Gate: a fração do curso do slider
    que cai em regime morto.

11. **"Eles diferem" é um oráculo fraco** (#18). O gate dizia que Round, Bevel e Miter produzem
    resultados distintos — e essa asserção **passa mesmo com o defeito** (compor bevel sobre round
    também difere). O que pega é a **identidade com um resultado FRESCO** computado fora do caminho
    sob teste. Antes de confiar num gate de diferença, pergunte: *que defeito ele deixaria passar?*

12. **Remendo sobre a premissa errada** (#18). Três correções seguidas atacaram sintomas — undo,
    âncoras, faixa — enquanto a premissa (*"o offset materializa no release"*) ficou de pé; uma delas
    ainda causou regressão. Quando o mesmo relato volta pela terceira vez com palavras diferentes, o
    que precisa mudar é **o modelo**, não mais um detalhe dele. E a pista costuma estar no
    vocabulário do usuário: *"só apply aplica definitivamente"* é uma frase sobre **quando a
    geometria nasce**, não sobre botões.

14. **Uma premissa que só a cobertura PARCIAL pode contradizer não é contradita por fixtures suas**
    (#20). O módulo afirmava *"a fonte é premultiplicada"*; num texel opaco e num vazio as duas
    convenções dão os MESMOS bytes, então só a banda macia as separa — e toda fixture de banda macia
    tinha sido escrita pela mesma mão que escreveu a premissa. **Pergunte ao produtor a montante**,
    não ao seu modelo dele: o gate que faltava renderiza com o Vello de verdade e conta os texels.

15. **Todo gate de um módulo pode medir o MESMO eixo, e o outro fica cego** (#20). Os gates de FX
    mediam *variação AO LONGO de uma aresta* — ondulação, pente, dente. Um defeito **constante ao
    longo dela** (uma linha dura, uma cor lavada) é invisível a esse oráculo, e passou duas vezes.
    Antes de confiar numa suíte, pergunte de que EIXO ela fala; o defeito seguinte costuma estar no
    outro.

16. **Copiar o enredo de um gate irmão sem medir** (#23). Escrevi *"a reentrância quase não acende"*
    para o Glow porque é o que vale no irmão de DENTRO — e num halo EXTERNO o sinal se inverte (há
    MAIS silhueta perto de uma reentrância; quem morre é a ponta convexa). A asserção passava e a
    prosa mentia. Mede-se primeiro, escreve-se depois.

17. **Suprimir um overlay globalmente para resolver o caso de UMA forma é grande demais** (#18).
    O raciocínio ("esta geometria é transiente, não a decore") estava certo; o alcance
    (`draw_overlays` inteiro, todas as formas, enquanto a janela vivesse) apagou o modo Node.
    Overlay é política por-ALVO; quando a política nasce global, o gate que falta é o que pergunta
    pelas formas que **não** são o alvo.

18. **Um caminho de fallback que ninguém consegue FOTOGRAFAR acumula bugs** (#24). O campo tem duas
    rotas e a sonda desenhava só a boa; o pente vivia na outra havia meses, e o que o revelou foi uma
    chave de ambiente de três linhas (`PH2D_FX_RASTER=1`). Quando um módulo diz *"pior, mas nunca
    trava"* sobre uma rota, pergunte como se OLHA para ela — senão *"pior"* é uma palavra sem imagem.

19. **A cena de smoke tem de conter o fenômeno, e a lista de fixtures apodrece sozinha** (#24). O
    smoke tinha dezasseis estrelas, uma traçada e uma biselada — **nenhuma as duas**. Cada wave
    acrescentou o seu caso e ninguém perguntou pelas COMBINAÇÕES; o bug reportado não podia aparecer
    ali. Ao fechar um bug, o passo final não é o gate: é pôr o caso na cena que alguém olha.

---

20. **Uma FAMÍLIA de controlos cujas rotas painel→barramento vivem em allowlists LONGAS nasce
    muda** (#29). A pilha de aparência shipou com o documento, o transform, o renderer, o
    exportador e o painel gateados, e **as três rotas desligadas** — cada uma era uma linha a
    acrescentar a uma lista de 200 que outras waves editam. ⚠️ **O que torna isto invisível é a
    outra metade estar PRONTA:** `stack_verb_for_id`, `apply` e o dreno da shell existiam, com
    gates de unidade verdes sobre um produto 100 % morto. ⛔ E o `hit_indexed_ids_are_registered`
    estava verde **com razão** — ele mede focabilidade, e os controlos eram focáveis, acendiam sob
    o rato e consumiam o gesto. ⇒ *uma família nova responde às perguntas dela num MÓDULO, e ganha
    um `seam_*` com gesto real cujo oráculo é o `EditorAction`, nunca o `WidgetEvent`.*

## Índice dos 29 FECHADOS — o mecanismo de cada um, em uma linha

### #29 — a PILHA DE APARÊNCIA nascia inteira e MUDA ✅ 2026-09-05

> ⚠️ **A wave seguinte (v21) acrescentou o DESLOCAMENTO por camada**, de uma pergunta do Enio no
> mesmo dia (*"o fill não deveria ter um offset? não seria mais útil?"*) — sem ele dois
> preenchimentos ocupam os mesmos pixels. Mecanismo, as duas medições que partem o pedido em duas
> features de preços muito diferentes, e as três leis que só a construção revelou:
> [doc 46 §8-ter](46_a_forma_tem_N_tintas.md). ⛔ Não é um bug — é o registo de que aquela wave
> **encostou** neste ficheiro (o censo `the_artless_draw_routes_are_declared` reprovou, e a cura foi
> actualizar a agulha, não apagar a linha).


**Sintoma** (Enio, no 1.º smoke do item 4): *"O olho não funciona. Clicar no nome não exibe a
largura, a opacidade e a mistura. Demais botões não funcionam"*.

**Causa — TRÊS defeitos que se leem como um.** Nenhum controlo da pilha atravessava o barramento:

| rota | o que faltava | quem ficava morto |
|---|---|---|
| clique | a família nas allowlists do `event_clicks::forwards_plain_click` | olho · nome · ↑ · ↓ · ✕ · *+ Fill* · *+ Stroke* |
| campo numérico | `VECTOR_PAINT_WIDTH` em `is_shell_owned_number` | a largura da camada |
| slider | `VECTOR_PAINT_OPACITY` na lista de tracks (e o `_NUM` na de engolidos) | a opacidade da camada |

⚠️ **A metade da SHELL estava inteira** — `stack_verb_for_id`, `apply`, os quatro slots de captura
e o dreno existiam e estavam gateados. O que faltava era o painel **emitir**, e por isso os gates
de unidade da shell ficavam todos verdes sobre um produto 100 % morto.

⚠️ **E o `hit_indexed_ids_are_registered` estava verde com razão:** os ids ESTAVAM registados, logo
os controlos eram focáveis e **acendiam sob o rato**. Ele mede focabilidade; entre ele e os testes
de unidade da shell há um passo que nenhum dos dois atravessa — *o clique sai do painel?*

**Mais DOIS defeitos que a caça revelou e o report não nomeia:**

- **O campo de largura MENTIA.** Ninguém o semeava do documento, então ele mostrava o que a última
  edição deixou no store, e o `PaintRow::width` publicado não tinha leitor nenhum. O artista lê `3`
  numa camada que mede `12`, escreve `4`, e engorda o traço errado. (Cura em `paint.rs`, o mesmo
  passe de sementes do Transform/Vertex, com o guarda de FOCO.)
- **A swatch de uma camada abria o picker no CINZENTO de fábrica.** As duas swatches de base
  sincronizam `widget_color` na shell; as das camadas não sincronizavam em lado nenhum, e o
  `pointer_down` cai no `unwrap_or([0x88; 3])` — a primeira mexida no picker **carimbava cinzento**
  por cima da cor da camada.

**E um verbo INALCANÇÁVEL POR CONSTRUÇÃO, removido:** o `StackVerb::Swatch` abria a camada ao tocar
na swatch, e uma *picker swatch* **nunca emite `Click`** (o `pointer_down` curto-circuita o Down
dela). Nenhum produtor o podia emitir; só um teste que o chamava à mão o fazia parecer vivo.

**A cura estrutural.** As três rotas da família passam a viver **num módulo**
([`event_paint_stack.rs`](../../crates/ph2d-panel-vector/src/event_paint_stack.rs)), à vista uma da
outra, em vez de três linhas em três allowlists longas que outras waves editam. A pilha de EFEITOS
tinha a mesma forma espalhada por dois ficheiros e foi cortada igual (`event_fx_stack.rs`).

**O instrumento que faltava:** [`seam_paint_stack.rs`](../../crates/ph2d-panel-vector/tests/seam_paint_stack.rs)
— 7 gates com **gesto real** (Down+Up sobre o rectângulo que o painel pintou), cujo oráculo é o
`EditorAction` e nunca o `WidgetEvent`. Três nasceram vermelhos sobre o produto shipado; a semente
da cor foi provada por mutação. ⚠️ Esta crate tinha **40 ficheiros `seam_*`** e a pilha não tinha
nenhum — *uma família de controlos sem `seam_*` é uma família que ninguém provou que fala*.


### #28 — o Build DUPLICAVA a forma que o gesto não tocou ✅ 2026-09-05

**Sintoma** (Enio): *"a ferramenta Build duplica a forma resultante"*.

**Causa.** A sobra de uma fonte TOCADA era `fonte − faces_pintadas`, e uma fonte carrega tudo o que
tem **escondido debaixo das outras**. Medido na fixtura dos gates (rectângulo por baixo, pentágono e
estrela por cima): pintar o **corredor** do rectângulo — a única parte dele que se vê — deixava a
sobra dele com a área **exacta** do pentágono (`3,7151`) e a **exacta** da estrela, que continuavam
lá, intactas. **Cinco formas onde o artista vê três**, e mover a de cima revela a gémea.

**A cura: a LEI DO DONO.** Cada ponto pertence à forma mais ALTA que o contém. As fontes já vêm em
ordem de z, então basta acrescentar as de cima aos cortadores da sobra — **uma chamada só**, porque
o `Subtract` do `apply_many` já dobra a união. Com ela as sobras são disjuntas *por construção*.
É o que o Shape Builder do Illustrator faz (ele reparte a arte), e só toca em fontes que o gesto
tocou.

⛔ **Fronteira NOMEADA:** se a forma de cima não tiver preenchimento, o que se via ali era a de
baixo — e ela perde essa área.

⚠️⚠️ **E o achado maior é sobre a RÉGUA, duas vezes no mesmo ficheiro no mesmo dia.** A fixtura
guarda geometria **LOCAL** com a pose num `Xform` (é como a Shape tool a deixa) e as constantes de
teste são pontos de **MUNDO**. O helper `hits` comparava as duas cruas:

- ele **nunca via o pentágono** (local `x ∈ [−1,25; 1,25]` contra o ponto `−1,75`);
- o que fazia `hits(IN_PENT) > 0` passar era **a sobra do rectângulo**, que nasce em mundo e cobria
  a área do pentágono — ou seja, *o gate da lua crescente estava a PINAR o defeito*;
- e a 1.ª redacção do gate novo tinha a mesma doença ao contrário: acusou uma duplicata de `3,19`
  onde não havia nenhuma.

*Um gate que compara dois espaços diferentes pode ficar verde sobre o defeito e vermelho sobre a
cura, e as duas leituras parecem igualmente convincentes.*

### #27 — o traço virava uma CANETA ELÍPTICA sob Scale não-uniforme ✅ 2026-08-23

**Sintoma** (Enio, com as duas fotos): *"o stroke não mantém sua espessura e sua sanidade ao usar
Scale não simétrico"*.

**Causa.** O traço é emitido sob `camera * afim_do_path`, e **no Vello o transform de um `stroke`
multiplica a CANETA**, não só a geometria. Com `sx ≠ sy` a caneta redonda de raio `w/2` vira uma
**elipse** de semi-eixos `w·sx/2` e `w·sy/2`. Não era perda de espessura — era perda de
**circularidade**.

⚠️ **A mesma lei já estava escrita DUAS vezes nesta crate** (o `marquee.rs`, que a nomeia por ela
ter transformado um realce do Flip num borrão, e o `hover_outline.rs`). Ela mordeu uma terceira
porque o caminho do **documento** nunca a tinha encontrado. *Uma lei escrita em dois sítios ainda
não é uma lei — só uma porta é.*

**Decisão de produto** (Enio, 2026-08-23): *"quando engrossa, engrossa por igual nos dois eixos"*.
⇒ o traço escala com o objecto, por um fator **único**: `√|det|`, a média geométrica dos
semi-eixos. Para escala uniforme `s` ela é `s` (o caminho comum não muda um pixel); para `(2, ½)`
é `1` (a área não mudou, o traço não muda); e é **invariante à rotação**.

**Correção.** [`stroke_uniform.rs`](../../crates/ph2d-vec-render/src/stroke_uniform.rs) — porta
única para os **três** sítios que emitiam traço no `draw_path_with`. Afim **conforme** ⇒ a chamada
de sempre, byte a byte; senão a geometria atravessa o afim e a caneta fica redonda sob `IDENTITY`.

⚠️ **O caminho rápido é intocado de propósito:** pré-transformar geometria por instância é o que o
`draw_path_with` recusa por escrito (*"o cache de tesselação não remove — a 160k estrelas era
metade do que sobrava"*). **Só o caso partido paga.**

**Gates** (5, todos com mutante morto): a caneta que chega ao Vello é REDONDA e tem raio `w·√|det|`
— nem o maior eixo nem o menor · rotação e escala uniforme tomam o caminho rápido · cisalhamento
**não** (com a fixture difícil: colunas de comprimentos IGUAIS, não perpendiculares) · o tracejado
escala com a caneta.

⚠️ **O gate que faltava, e por que nenhum o via:** a suíte media o que o traço DESENHA em escala
uniforme, e uma caneta elíptica não move régua nenhuma porque nenhuma perguntava *"a espessura é a
mesma nos dois eixos?"*. E um gate sobre o `is_conformal` **também não bastaria** — ele não prova o
desenho. O que decide o pixel é o par *(caneta, afim)* que chega ao Vello, e é ele que o `pen_for`
devolve para poder ser medido.

---

### Os 26 anteriores

> Post-mortem completo (sintoma · causa · gates · lição) no
> [arquivo](../archive/docs-2026-08-18/Vector%20Module/BUGS_vector.md), na seção `## Bug #N`.

| # | O MECANISMO (é isto que se repete, não o sintoma) | Data |
|---|---|---|
| 1 | Panic nos params do balão: uma **janela geométrica que colapsa** — o clamp recebia um intervalo invertido. | 2026-07-12 |
| 2 | Formas assimétricas nasciam **espelhadas**: um `−1` faltando numa travessia mundo↔tela. **Uma fronteira mundo↔tela é sempre DUAS** (o gêmeo é o #6). | 2026-07-12 |
| 3 | A "seta curvada" era um **polígono** — a curva nunca chegou a ser curva; achado *desenhando*, com todos os testes verdes. | 2026-07-12 |
| 4 | Balão com contorno **auto-intersectante** — a construção da cauda não respeitava a moldura. | 2026-07-12 |
| 5 | Contorno **ABERTO** recortava o preenchimento: hit-test e renderer respondiam **diferente** à mesma pergunta (*o que é o interior?*). Discordância entre subsistemas é sempre bug. | 2026-07-12 |
| 6 | Previews do painel **espelhados**: o mesmo `−1` do #2, na segunda travessia — **o painel pinta geometria de mundo sem passar pela câmera**. | 2026-07-12 |
| 7 | O `spread` de conectores paralelos era **PLACEBO**: *um parâmetro que nenhum teste exercita não está implementado — está escrito*. O sinal: seis testes passando `spread: 0.0`. | 2026-07-12 |
| 8 | O filete caía num fallback frouxo: **sinal de Cramer trocado** — o caso exato existia e nunca era escolhido. | 2026-07-13 |
| 9 | A "reta" é um **smoothstep**, e de Casteljau a envenenava (subdividir uma curva não preserva a parametrização que o efeito assume). | 2026-07-13 |
| 10 | A alça **escorregava do dedo**: o arrasto era absoluto-de-Down em vez de delta. | 2026-07-13 |
| 11 | A alça **funcionava e depois esquecia**: o estado vivo não voltava ao documento — e os gates passavam porque nenhum reabria a sessão. | 2026-07-13 |
| 12 | Um **CLIQUE** no Shape Builder dissolvia a arte (medido no app, não deduzido): clique sem arrasto caía no braço de "selecionar tudo". | 2026-07-13 |
| 13 | A borda do véu tinha **150 pixels**: unidade de mundo consumida como unidade de tela. | 2026-07-13 |
| 14 | O realce **pairava sobre nada**: o alvo do hover e o alvo do desenho eram listas diferentes. | 2026-07-13 |
| 15 | *"O undo só faz uma etapa"*: e o gate expôs uma **segunda mina** que o relato não continha. | 2026-07-13 |
| 16 | O Build deixava **"pedaços de linha"**: a 1ª teoria estava errada, **a fixture escondia o bug**, e o gate **quase nasceu MORTO**. | 2026-07-13 |
| 17 | O quadrado **GIRAVA** a caminho do círculo: a causa não era a busca, era o **CONJUNTO DE CANDIDATOS** — a pergunta certa era *"o que é uma FEATURE?"*. E o Reverse Match era um botão que **nunca** ajudava. | 2026-07-14 |
| 18 | *"Round não muda"* / *"só apply aplica"*: em cinco rodadas, **o DIAL era o bug** (o gesto natural estaciona o parâmetro em regime morto) e depois **o MODELO** (o offset materializava no release). ⚠️ Três remendos sobre a premissa errada. | 2026-07-21 |
| 19 | A gaiola do envelope não aparecia mas o efeito se aplicava — e **a fixture do smoke mascarava o bug**. | 2026-07-24 |
| 20 | Contorno **TRACEJADO** do Feather: o módulo afirmava *"a fonte é premultiplicada"* e só a **cobertura PARCIAL** podia contradizer a premissa — todas as fixtures de banda macia eram da mesma mão. | 2026-07-26 |
| 21 | A fixture do Feather era uma **CÓPIA**, e ficou para trás quando o original mudou. | 2026-07-26 |
| 22 | A sombra interna **DESCOLAVA** do contorno quando deslocada: o offset era aplicado depois do recorte. | 2026-07-26 |
| 23 | O halo **externo** não tinha a escolha que os de dentro tinham — e **a prosa copiada do gate irmão mentia** (num halo externo o sinal se inverte). | 2026-07-26 |
| 24 | *"Linhas no Bevel"*: o **PENTE** era o caminho do **RASTER**, e toda forma com traço caía nele — *um caminho de fallback que ninguém consegue FOTOGRAFAR acumula bugs* (a cura foi `PH2D_FX_RASTER=1`, três linhas). | 2026-07-27 |
| 25 | Renomear na Hierarquia **disparava os atalhos** do Vector: a cura é **porta única**, não mais um `&&` no despacho. | 2026-08-05 |
| 26 | Checkbox não redimensiona / Slider com altura fixa: **UM mecanismo, não dois bugs** — a razão é `h / ROW_H_PX` (⛔ ver as recusas no topo). | 2026-08-05 |
| 27 | O traço virava **CANETA ELÍPTICA** sob Scale não-uniforme: no Vello o transform de um `stroke` multiplica a CANETA, não só a geometria. | 2026-08-23 |
| 28 | O **Build duplicava**: a sobra de uma forma tocada era `fonte − pintado`, e isso inclui o que está **escondido por baixo** das outras — ela re-criava, com a área EXACTA, as formas que o gesto não tocou. ⚠️ E a régua dos gates comparava geometria **local** com pontos de **mundo**, então ela pinava o defeito em vez de o acusar. | 2026-09-05 |
