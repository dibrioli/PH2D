# BUGS — 3D Modeling (campo implícito)

Registro dos bugs do modelador cuja **causa enganava**: o **sintoma** que o Enio viu, a **causa
real** (que quase nunca é a primeira suspeita), a **correção** e o **gate executável** que impede a
volta. Espelha o [`docs/Vector Module/BUGS_vector.md`](../Vector%20Module/BUGS_vector.md).

Regra deste arquivo: um bug só é considerado fechado quando existe um teste que **falha** se a
correção for revertida — provado por mutação. "Não reproduzi mais" não fecha nada.

---

## Padrões que se repetem (leia antes de caçar o próximo)

- ⛔⛔ **O modelador tem DOIS motores e cada modo usa um.** O modo **RENDER** traça na **placa**; o
  modo **MODEL** (o sombreamento *matcap*, o de todos os dias) traça na **CPU**
  (`smoke_draw_thread::traca` — a placa só entra com `Shading::Render`). Uma cura medida num só
  motor pode deixar o outro exactamente como estava, **e o smoke do dono é quase sempre no MODEL**.
  ⇒ *meça os DOIS antes de dizer «curado»* (`device_probes::mede_as_faixas_do_vaso` faz isso).
- ⛔ **A CPU não avalia a árvore global: ela ESPECIALIZA-A por ladrilho** (`RegionCompiler`), e a
  folha especializada lê o perfil por um `ProfileIndex`. Uma lei nova ensinada à árvore global e
  não ao índice chega à placa e **não** chega ao modo MODEL.
- ⚠️ **A régua da suavidade é a NORMAL** (W54, e outra vez aqui): duas descrições da mesma curva a
  menos da tolerância podem ter normais que diferem vários graus, e é isso que a luz mostra. *«Os
  valores concordam» não prova que a imagem concorda.*
- ⚠️ **Uma grelha nunca cai num conjunto de medida nula** — um ponto EXACTAMENTE numa corda, numa
  aresta, num canto de célula. Os pontos desses casos PÕEM-SE (ver o Bug #1, que escondia dois
  defeitos de empate que nenhuma grelha apanharia).
- ⛔⛔ **A app traça o que o PREVIEW lhe dá** (`preview::coarse_doc`, a mexer e parado): um gate que
  chama o cozedor ou o traçador directamente mede outro programa. Os gates do produto passam por
  ele (Bug #2).
- ⚠️ **Todo gate e todo smoke corriam no nível `1` do `Resolution`.** Um defeito que só existe com o
  botão noutro sítio é invisível a um corpus todo no ponto de omissão (Bug #2).

---

## Índice — o mecanismo de cada um, em uma linha

| # | data | sintoma | mecanismo | gate |
|---|---|---|---|---|
| 1 | 2026-09-16 | *«arestas ainda visíveis»* no ombro do vaso (foto), **depois** da wave que pôs as quinas como arcos | o modo MODEL traça na CPU, e a folha especializada por região lia a **polilinha densa** pelo `ProfileIndex` — a normal ficava constante em cada segmento | `vaso_sem_facetas_tests::o_vaso_nao_tem_facetas_no_traçado_do_modo_model` · `profile_arc_tests::*` |
| 6 | 2026-09-16 | (auditoria) um contorno **MISTO** — quinas arredondadas + curvas livres — perdia os arcos no quadro do modo MODEL ao subir o `Resolution`, e quando não os perdia não era engrossado | o `coarsen` devolvia sempre uma polilinha sem arcos (o doc dizia *«é o que se quer»*), e o `coarse_doc` só a aceitava quando ficava mais barata — nunca num contorno de arcos e rectas, cedo num misto | `vaso_sem_facetas_tests::o_preview_engrossa_so_o_que_esta_tesselado` · `profile_coarsen_tests::*` (`ph2d-field`) |
| 5 | 2026-09-16 | (auditoria, depois do smoke aprovado do #4) largar uma forma sobre outra que não está na origem fazia-a **saltar** | o arrasto da Hierarquia conserva a pose de mundo (escreve a local no referencial do novo pai), e a promoção passava os filhos para um grupo de pose identidade sem compor a pose do anfitrião | `tests::a_shape_dropped_onto_a_posed_shape_stays_where_it_was` (`ph2d-field-ecs`) |
| 4 | 2026-09-16 | *«o modo como vc montou a cena com o vaso como model e não como filho de model, nenhum outro objeto acrescentado aparece na cena»* (foto da Hierarquia: o *Extrude* dentro de «Model») | a paleta pendura a forma nova na RAIZ, e a promoção «só uma operação tem filhos» saltava a raiz com a nota *«um caso que não existe hoje»* — as cenas `2` e `5` nascem com a raiz numa forma | `tests::a_root_shape_that_gets_a_child_becomes_a_group_in_place` (`ph2d-field-ecs`) · `group_tests::a_shape_added_to_a_piece_whose_root_is_a_shape_appears` |
| 3 | 2026-09-16 | *«o Modo model não está permitindo usar o modo Vector. não consigo desenhar o cilindro»* — a seguir um passo de smoke | a env `PH2D_FIELD_SMOKE` armava o módulo **sem olhar o painel**: pegar no Vector fechava o painel e os ganchos de entrada continuavam a comer o clique — o report de 22/08, curado só no caminho do PILL | `mode_tests::the_directed_smoke_disarms_with_the_panel_too` |
| 2 | 2026-09-16 | (auditoria, depois do smoke aprovado do #1) **subir `Resolution` desfazia os arcos**; **um círculo nunca era arco**; **uma meia-lua de dois pontos era recusada** | a barra de «esta cúbica é um arco?» era a tolerância de ACHATAMENTO — mais apertada do que a precisão com que qualquer app escreve um círculo —, os dois arredondadores de quina escreviam arcos acima de `90°` numa cúbica só, **o preview (`coarse_doc`) trocava os arcos por polilinha** comparando segmentos em vez de primitivas, e a porta dos arcos pedia 3 primitivas (a lei do polígono) | `o_arco_sobrevive_a_todo_nivel_de_resolution` · `o_quarto_canonico_define_a_barra_do_arco` · `o_preview_nunca_troca_arcos_por_uma_polilinha_mais_cara` · `subir_o_resolution_nao_parte_o_labio` · `a_meia_lua_de_dois_pontos_coze` · `corner_split_tests::*` · `profile_meia_lua_tests::*` |

---

## Bug #1 — as faixas do vaso sobreviveram à cura dos arcos (2026-09-16)

**Sintoma.** Smoke do dono, modo MODEL, vaso da cena `5`, vista de frente: *«arestas ainda
visíveis»* — faixas horizontais de luz no ombro arredondado. A wave anterior (`86033ef31`) tinha
posto as quinas do perfil como **arcos exactos** e medido o vaso a ficar liso **na placa**.

**Por que enganava.** A cura estava no binário (recompilado às 17:04 com ela dentro) e **funcionava**
— no motor que a medição dela olhou. O modo em que o dono faz o smoke usa o **outro** motor.

**Mecanismo, medido.** A régua é a assinatura da faceta: descendo uma coluna, a normal fica parada,
**salta**, e volta a ficar parada (pico isolado entre vizinhos calmos, com continuidade no mundo).
Cena `5`, frente, `1920×1080`:

| motor | picos de faceta | maior pico |
|---|---:|---:|
| CPU (o modo MODEL) | **12 196** | **11,42°** |
| placa (o modo RENDER) | 0 | — |
| CPU com a especialização por região **desligada** (experiência) | **0** | — |

⇒ a especialização por região é o sítio. A folha especializada chama `sd_profile_in_region` com um
`ProfileIndex` construído de `profile.contours()` — a polilinha densa, só segmentos.

⛔ **A 1.ª régua desta caça caiu**: medir o salto MÁXIMO da normal deu `82–85°` nos DOIS motores, no
mesmo pixel — era o EIXO (o pólo do torno) e as fronteiras de OCLUSÃO (o lábio à frente da parede
interna). A régua que separa exige continuidade no mundo e conta PICOS.

**Suspeitar do chamador.** O componente (a lei de distância a um segmento) estava certo. O defeito
era **quem o chamava com a descrição errada da curva**: a lei do arco tinha sido escrita num sítio
(a árvore global) e o chamador por região continuava a ler a polilinha. *Uma lei escrita em dois
sítios ainda não é uma lei.* ⇒ a cura foi uma **porta** (`ph2d_field_eval::profile_arc`) lida pelos
TRÊS leitores — a árvore global, a árvore por região e o índice.

**E a caça achou mais DOIS defeitos, e um deles era meu, da wave anterior:**

1. ⛔⛔ **O empate sobre a corda (árvore global).** O sinal é o enrolamento das cordas + a correcção
   da meia-lua. Um ponto **exactamente sobre a corda** (a `0,0437` de profundidade dentro da peça)
   lia `+0,0437` — **fora**: o raio `+x` punha-o do lado `−dir` e a meia-lua usava um teste estrito
   que o punha sempre fora dela. O gate da wave anterior nunca o viu — a grelha dele nunca caiu numa
   corda. ⇒ cada caminho do enrolamento tem a sua regra de empate, e a meia-lua copia a do caminho
   que a acompanha (raio: `−dir`, ou `sinal(eₓ)` numa corda horizontal; caminho âncora→ponto: `≥ 0`).
2. ⛔ **O canto de célula sobre uma aresta (índice, antigo).** O `inside` do índice partia de um
   canto da grelha cujo enrolamento saía do RAIO, e caminhava até ao ponto pela regra do CAMINHO —
   sobre uma aresta as duas discordam. É o defeito da W56 (curado na árvore por região com
   `anchor_in`) um nível abaixo. A polilinha densa quase nunca punha uma aresta num canto; as cordas
   redondas põem. `NonZero` mascarava-o (lia `−2` onde devia ler `−1`, os dois «dentro»); a
   **paridade** denunciou-o. ⇒ um ponto de partida SEGURO por célula.

**O gate que estava VERDE sobre isto, e porquê.**
`the_specialised_tree_agrees_inside_its_region` mede exactamente a concordância árvore-por-região ×
árvore-global — mas o corpus dele nasce todo por `Profile::new`, **sem um único arco**, e ele mede
**valores**, nunca a normal. Antes da wave do arco as duas árvores eram a mesma polilinha e
concordavam por construção. *Um gate cujo corpus não contém o fenómeno não o testa.*

**Correção + gates + mutações (8 de 8 mortas).**

| mutação | gate que a mata |
|---|---|
| M1 o índice ignora a decomposição (o estado de antes) | `a_regiao_especializada_tem_o_campo_e_a_normal_da_arvore_global` + o gate do produto |
| M2 a árvore global esquece o empate sobre a corda | `um_ponto_sobre_a_corda_tem_o_sinal_certo_nos_tres_leitores` |
| M3 a região esquece as meias-luas | o mesmo |
| M4 o empate do caminho fica estrito para arcos à esquerda | o mesmo (figura espelhada) |
| M5 o índice esquece a meia-lua no dentro/fora | o mesmo (leitor «índice») |
| M6 o índice volta a partir do canto da célula | o mesmo (`horário · EvenOdd`) |
| M7 o corte esquece a flecha (minorante) | `o_corte_por_regiao_guarda_a_primitiva_que_a_flecha_decide` (cintura) |
| M8 o corte esquece a flecha (majorante) | o mesmo (pé) |

⚠️ **M7 e M8 SOBREVIVERAM à primeira ronda** — nenhuma região do corpus caía onde a flecha decide o
corte. As duas regiões do gate novo foram construídas à mão, com a conta ao lado. *Um corpus onde
uma desigualdade nunca aperta não testa a desigualdade.*

**Oráculo.** O quadrado redondo tem distância com sinal **analítica** (a do *round box*), e é ela —
não uma das árvores — que julga os três leitores nos pontos postos sobre as cordas, nas quatro
combinações de sentido (horário/anti-horário) e preenchimento (`NonZero`/`EvenOdd`).

---

## Bug #2 — o arco que não sobrevivia ao botão (2026-09-16, auditoria depois do smoke aprovado do #1)

**Sintoma (medido, não reportado).** Com o smoke do #1 aprovado no nível de omissão, a auditoria
perguntou o que acontece quando o artista sobe o `Resolution`. Resposta: **as quinas deixavam de ser
arcos** — e ninguém o via, porque todo gate e todo smoke corriam no nível `1`.

**TRÊS mecanismos, e nenhum era o que se procurava:**

1. ⛔⛔ **A barra de «esta cúbica é um arco?» era a tolerância de ACHATAMENTO**, e o doc ao lado dela
   dizia-o com orgulho (*«não um número novo»*). Mas uma cúbica não É um círculo: o quarto canónico
   erra `2,7253e-4·r` por construção — a precisão com que TODO formato vectorial escreve um círculo.
   A tolerância divide-se pelo nível e esse erro não:

   | arcos / primitivas | nível 1 | nível 4 | nível 64 |
   |---|---:|---:|---:|
   | vaso da cena 5 | `10/10` · `22` | `8/10` · `71` | `6/10` · **`384`** |
   | **círculo** `r/extensão = ½` | **`0` · `168`** | `0` · `332` | `0` · `1 328` |
   | pílula | `4` · `8` | **`0` · `186`** | `0` · `730` |

   ⇒ **um círculo nunca foi arco** (o erro intrínseco, `1,36e-4` da extensão, está acima da
   tolerância do nível 1), e subir o botão desfazia os que havia. Cura: a barra é
   `max(tol, ERRO_DO_QUARTO·r)` ([`ph2d-field-profile`](../../crates/ph2d-field-profile/src/lib.rs)).
2. ⛔ **Os dois arredondadores de quina escreviam um arco acima de `90°` numa cúbica SÓ** — os únicos
   escritores de arco da casa a violar a lei que o `shapes::arc` já tinha escrita (*«segmentos de
   ≤90° para o bézier aproximar bem»*). Uma ponta de estrela (`144°`) errava `~0,5 %` do raio no
   próprio 2D, e acima do quarto de círculo não havia barra honesta que a aceitasse como arco.
   Cura: `ph2d_vec_scene::corners::circular_fillet`, uma porta para os dois — até `90°` **byte a
   byte** o de sempre; acima, duas metades com um vértice liso no meio. ⚠️ Só quando os dois
   recuos são iguais (é a condição de existir um círculo tangente nos dois pontos); um blend
   assimétrico fica numa cúbica.
3. ⛔⛔ **A APP não traçava o que os gates mediam.** O `coarse_doc` (o preview — a mexer **e**
   parado desde a W85) trocava o perfil pelo engrossado sempre que a **polilinha** encolhia, e o
   engrossado não tem arcos. Com as curas 1 e 2 dentro, o vaso no nível 16 ainda ia ao traçador
   como `(0 arcos, 329 primitivas)` parado e o círculo como `(0, 332)` — **83×** o custo dos seus
   `4` arcos. Cura: comparar o `prim_count`, que é o custo da marcha.

**E um defeito da wave do arco, achado pela fixtura destes gates:** a porta `Profile::with_arcs`
pedia **três** primitivas (a lei do polígono, emprestada) e recusava INTEIRA a meia-lua que a caneta
desenha com dois pontos (`Err(BulgeMismatch)`), que antes da decomposição cozia pela polilinha.
Duas bastam quando uma é arco; duas rectas continuam recusadas.

**Suspeitar do chamador — duas vezes.** A cura 3 é a lição do Bug #1 um nível acima: os gates do
cozedor estavam certos sobre o cozedor, e **a app traça o que o preview lhe dá**. ⇒ os gates do
produto passam pelo `coarse_doc` (`o_preview_nunca_troca_arcos_por_uma_polilinha_mais_cara`,
`subir_o_resolution_nao_parte_o_labio`).

**O gate da IMAGEM mentiu DUAS vezes antes de medir** (o registo é a lição):
- a 1.ª redacção olhava o vaso inteiro e ficou **verde com as curas desfeitas** — de longe cada
  faceta do lábio tem menos de um pixel e a régua dos picos precisa de normal PARADA;
- a 2.ª aproximou-se em **perspectiva**, e aproximar é trazer o olho: a `half_extent 0,05` punha-o a
  `0,11` do eixo, **dentro do vaso** (parede a `0,33`), a medir a parede interna do outro lado —
  verde outra vez. ⇒ lente **paralela**. Com ela: `0` picos com a cura, **`10 020` (maior `6,23°`)**
  sem a divisão, e a conta previa `~5,6°`.
- ⚠️ e a sonda que comparava duas imagens leu **`91 770` pixels diferentes entre duas corridas
  IGUAIS**: `acos` de duas normais idênticas com `|n| < 1` em `f32` dá até `0,04°`. *Uma diferença
  que é a mesma entre todos os pares é a assinatura da régua.*

**Provas: 12 mutações, 12 mortas** (duas sobreviveram à 1.ª ronda e cada uma pediu um gate):

| mutação | gate que a mata |
|---|---|
| N1 a barra volta a ser só a tolerância | `o_arco_sobrevive_a_todo_nivel_de_resolution` |
| N2 a barra 10× mais larga | o mesmo — ⚠️ **sobreviveu** até ao controlo de `95°` numa cúbica só (erra `1,38×` o quarto); o de `150°` (`22×`) nunca apertava |
| N3 o filete não parte acima de `90°` | o do cozedor + `corner_split_tests` + `subir_o_resolution_nao_parte_o_labio` |
| N4 o meio do lado errado | o do cozedor + `corner_split_tests` |
| N5 parte já a `60°` | `corner_split_tests` + 8 gates antigos da `ph2d-vec-scene` (a identidade até `90°`) |
| N6 a porta volta a pedir três primitivas | `a_meia_lua_de_dois_pontos_coze` + `profile_meia_lua_tests` |
| N7 a porta aceita duas rectas | `duas_rectas_nao_fecham_area_e_a_porta_recusa` |
| N8 o vivo nunca parte | o do cozedor + `corner_split_tests` + a imagem do lábio |
| N9 o vivo parte blends assimétricos | `um_blend_assimetrico_nao_e_partido` |
| N10 a flecha do meio pela metade | o do cozedor + `corner_split_tests` |
| N11 o alçapão das metades é o do arco inteiro | o do cozedor + `corner_split_tests` |
| N12 o preview volta a comparar a polilinha | `o_preview_nunca_troca_arcos_por_uma_polilinha_mais_cara` + a imagem do lábio — ⚠️ **a imagem sobreviveu** até passar pelo `coarse_doc` |

**Oráculos analíticos.** A meia-lua tem distância com sinal fechada (disco ∩ semiplano) e julga os
três leitores; com duas primitivas a corda e a recta de volta **cancelam-se** no enrolamento e o
sinal inteiro sai da meia-lua — o caso em que a correcção trabalha sozinha.

**Raio de alcance medido:** a divisão acima de `90°` muda a geometria de toda quina aguda da casa
(estrela, triângulo). Portão dos impactados: **`15 236` de `15 237`**, e o vermelho era uma contagem
escrita à mão (`ph2d-vec-edit`: a estrela de 5 pontas passa de `15` para `20` vértices). Workspace:
`22 885` de `22 886` — o vermelho é `the_cost_of_sampling_a_path_is_flat_in_its_anchors`, da família
de flakes de carga (load `52`; `3/3` verde sozinho a load `14–17`, zero linhas de diff na
`ph2d-timeline`).

### Adenda (mesmo dia, auditoria): as duas metades que o #2 deixou nomeadas

- ⛔ **A barra que o reconhecedor APLICAVA era `1,0535×` a escrita.** Ele conferia a cúbica em seis
  parâmetros, e o pico do erro de um arco numa cúbica cai em `t = (3 − √3)/6 ≈ 0,2113`, entre dois
  deles — um arco de `90,75°` numa cúbica só passava por círculo. A nota da auditoria dizia `~8 %`, e
  era também uma leitura por amostras. ⇒ grelha de `16` + secção áurea em cada máximo local
  (`pico_do_desvio`). ⚠️ **Nenhuma grelha fixa bastava:** `32` amostras ainda leem o arco de `90,03°`
  (`1,002×` a barra) como `1,000×`. Preço medido: `~1,2 µs` por aresta que é arco. Gate
  `a_barra_escrita_e_a_barra_que_o_reconhecedor_aplica`; mutações R0 (seis amostras), R1 (grelha de
  16 sem refinar) e R2 (grelha de 32 sem refinar) mortas.
- ⛔ **O terceiro escritor de arco — a suavização de quina (`smooth_corner`) — também escrevia o arco
  curto numa cúbica só.** Ele varre `α·(1 − s)`: uma quina que vira `120°` com `s = 0,05` dá `114°`.
  Hoje só o `RoundRect` (quinas de `90°`) o alcança no produto, mas a porta é pública e declara-se
  para qualquer ângulo. ⇒ passa pela mesma `circular_fillet`; **byte a byte igual até `90°`**
  (impressão de `687` vértices de retângulos, pentágonos, triângulos e setas, antes = depois) e a
  diferença acima de `90°` é exactamente a divisão (`+6` vértices numa seta com duas quinas de
  `135°` × três suavizações). Gate `corner_split_tests::a_quina_suavizada_parte_o_arco_do_meio_acima_de_90_graus`;
  mutação S1 (descartar o vértice do meio) morta.

---

## Bug #3 — o smoke dirigido prendia o canvas (2026-09-16)

**Sintoma.** Dono, a seguir o passo 3 do smoke do #2 (desenhar um círculo no Vector com o app aberto
por `PH2D_FIELD_SMOKE=5`): *«o Modo model não está permitindo usar o modo Vector. não consigo
desenhar o cilindro»*.

**Por que enganava.** É **o mesmo report** de 22/08 (*«ainda não consigo usar outros modos como
vector»*), curado na W42 com um gate verde — e o gate continua verde. A W42 curou o caminho do
**pill**; o smoke dirigido é o **outro** caminho de armar, e é por ele que todo passo de smoke
deste módulo abre o app.

**Mecanismo (lido e provado).** `smoke_requests::armed_scene()` devolvia a cena **sempre que a env
existia**, e só olhava o painel sem ela. Pegar no Vector fecha o painel (`mode::note_owner`, W40) e
escreve `set_armed_by_panel(false)`; com a env definida, `with_smoke` continuava a devolver o
módulo, e os ganchos de entrada da modelagem — que correm antes dos do Vector — comiam o clique.

**O gate que estava VERDE, e porquê.** `disarming_the_module_actually_disarms_it` mede exactamente
«fechar desarma» — mas **nenhum teste corre com a env definida** (`set_var` é `unsafe` e o
`cargo test` partilha o processo entre threads). *Um corpus sem o caminho do dono não mede o caminho
do dono.*

**Cura.** A lei ficou pura — `scene_for(env, painel_aberto)`: a env **escolhe** a cena, o painel
**liga e desliga**. A abertura automática do smoke passa a perguntar à **env** (se perguntasse pelo
armado, a porta ficava trancada por dentro — o defeito da W45). A env é lida por `smoke_env()`, com
uma sobreposição **por thread** só nos testes (`com_env_do_smoke`), que é o que põe o caminho do
smoke dirigido no corpus.

| mutação | gate que a mata |
|---|---|
| D1 a env volta a armar sem o painel | `the_directed_smoke_disarms_with_the_panel_too` (a lei) |
| D2 o caminho real passa por cima do painel | o mesmo (o caminho real) |
| D3 a abertura automática volta a perguntar pelo armado | o mesmo |
| D4 a sobreposição de teste é ignorada (o controlo do próprio gate) | o mesmo |

Impactados (`ph2d-app-field3d` e dependentes, incluída a shell): `2 656` de `2 656`.

⚠️ **E o passo de smoke que o revelou era meu e não tinha sido conduzido** — escrito a partir dos
nomes no código, com a placa ocupada pelo smoke de outra linha. *Um passo não conduzido é uma
hipótese sobre o produto; este acertou num defeito, e podia ter só mandado o dono a um sítio que não
existe.*

---

## Bug #4 — a forma acrescentada a uma raiz-forma não aparecia (2026-09-16)

**Sintoma.** Dono, logo a seguir à cura do #3, com a foto da Hierarquia: o *Extrude* do cilindro
aparece como filho de «Model» (o vaso da cena `5`), e *«nenhum outro objeto acrescentado aparece na
cena»*. Ele leu-o como defeito da montagem da cena.

**Por que enganava.** A cena **é** a reprodução, mas o defeito é do produto: a lei *«só uma
OPERAÇÃO pode ter filhos»* (W31, `promote_leaf_hosts`) repara uma forma que recebe filhos — **menos
a raiz**, que ela saltava com uma nota: *«uma folha SEM pai é a raiz da peça… quem chega aqui é um
caso que não existe hoje»*. Existe: as cenas `2` e `5` nascem com a raiz numa forma (medido por
censo das `18` cenas vivas), e a paleta pendura toda forma nova **na raiz** quando nada está
escolhido. O cozimento nunca olha os filhos de uma forma ⇒ a Hierarquia mostra-a, o ecrã não.
*A nota ao lado do código afirmava o que o código não verificava — e era parte do defeito.*

**O gate que estava VERDE, e porquê.** `a_shape_dropped_onto_a_shape_is_not_lost` mede exactamente
«uma forma com filhos não some» — mas o corpus dele tem a raiz numa **união**. *Um gate cujo corpus
não contém o fenómeno não o testa* (a mesma frase do #1).

**Cura.** A razão da nota é real — a raiz é DONA da peça —, e por isso ela não é embrulhada:
`promote_root_in_place` faz da própria raiz a união (fica com o nome, o `FieldObject`, o
`Transform`, a ordem e a **pose** da peça) e a forma desce para uma entidade nova, **primeira** dos
filhos, com pose identidade e com o que é **dela**: a geometria, os modificadores, o verbo, o
material e o vínculo ao desenho. A peça na tela não muda; a forma acrescentada passa a entrar.

| mutação | gate que a mata |
|---|---|
| E1 a raiz-forma volta a ser saltada | os dois (o dos componentes e o da tela, pela paleta, nas cenas `2` e `5`) |
| E2 o material fica na raiz | `a_root_shape_that_gets_a_child_becomes_a_group_in_place` |
| E3 os filhos antigos ficam à frente da forma (a base de uma subtração mudaria) | o mesmo |
| E4 o vínculo ao desenho fica na raiz | o mesmo |
| E1 (outra vez, depois da correcção dos dois gates da importação) | `a_shape_picked_in_the_palette_is_born_in_the_part` + `a_loaded_sculpture_becomes_a_node_the_evaluator_resolves` |

⛔⛔ **E DOIS gates estavam verdes SOBRE A PEÇA INVISÍVEL, na mesma cena.** Os da costura da
importação (`a_shape_picked_in_the_palette_is_born_in_the_part`,
`a_loaded_sculpture_becomes_a_node_the_evaluator_resolves`) nascem da cena `2` — raiz-forma — e
mediam a **arena** do documento (`doc.nodes()`, e o `sampled_count` do avaliador). O cozimento emite
também os filhos de uma forma, **soltos**, sem ninguém os referenciar: a escultura e o cone entravam
na arena e não na árvore. O doc do segundo chamava a isto *«o pior dos três… a peça existe na
Hierarquia e some da tela»* — e media-o pela metade que não o vê. Hoje os dois contam **formas** na
árvore e procuram a forma entre os nós que a **raiz alcança**; sem a cura (E1) os dois ficam
vermelhos.

✅ **E o `cook` deixou de emitir nós soltos** (mesmo dia, depois do smoke aprovado): só uma
OPERAÇÃO desce aos filhos. Gate `the_cooked_arena_holds_only_what_the_root_reaches`; mutação F2
morta. *O que a raiz não alcança não entra no documento* — uma régua que conte a arena deixa de
poder mentir por aqui.

⚠️ A outra nota desta secção (a promoção de um anfitrião **não-raiz** tirava a pose aos filhos) era
um defeito visível, e é o Bug #5.

---

## Bug #5 — a forma largada sobre outra saltava (2026-09-16, auditoria)

**Sintoma (medido, não reportado).** Largar na Hierarquia uma forma sobre outra que não está na
origem — o gesto da W31 (*«se coloco um objeto como filho do outro ele some»*) — faz a largada mudar
de sítio: uma esfera em `(0, 1, 0)`, largada sobre um anfitrião em `(1, 0, 0)`, aparecia em
`(−1, 1, 0)`.

**Mecanismo.** Duas metades certas, cada uma sozinha: o arrasto da Hierarquia **conserva a pose de
mundo** (re-escreve a local no referencial do novo pai, `set_world_xform`), e a promoção do
anfitrião (`promote_leaf_hosts`) põe os filhos num grupo novo de pose identidade no referencial do
**avô** — sem compor a pose do anfitrião na deles. A pose do anfitrião evaporava da cadeia.

**O gate que estava VERDE, e porquê.** `a_shape_dropped_onto_a_shape_is_not_lost` (W31) tem o
anfitrião na **origem**, onde compor e não compor dão o mesmo — e mede se a forma EXISTE, não
ONDE. *Uma fixtura na identidade de uma transformação não testa a transformação* (a mesma família
de «um corpus no neutro de um knob»).

**Cura.** Os filhos que mudam para o grupo recebem `pose_do_anfitrião ∘ pose_deles`. A raiz (#4) não
tem o problema: ela fica com a pose e os filhos não mudam de pai.

| mutação | gate que a mata |
|---|---|
| F1 os filhos passam ao grupo sem a pose do anfitrião | `a_shape_dropped_onto_a_posed_shape_stays_where_it_was` |
| F2 o cozimento volta a descer aos filhos de uma forma (#4) | `the_cooked_arena_holds_only_what_the_root_reaches` |

---

## Bug #6 — o preview deitava fora os arcos de um contorno MISTO (2026-09-16, auditoria)

**Sintoma (medido, não reportado).** Um desenho com quinas arredondadas **e** curvas livres (a
caneta a fazer uma onda) vira peça com arcos nas quinas — e o quadro do modo MODEL, que traça o
que o `coarse_doc` lhe dá, recebia-a **sem arcos** assim que o `Resolution` subia (as faixas do
Bug #1 de volta nas quinas); quando não os perdia, a parte tesselada seguia **inteira**, sem
engrossar.

| ondas | nível | documento | a mexer, antes → depois | parado, antes → depois |
|---:|---:|---|---|---|
|  1 | 16 | `(2, 121)` | **`(0, 92)`** → `(2, 21)` | `(2, 121)` → `(2, 35)` |
|  1 | 64 | `(2, 237)` | **`(0, 93)`** → `(2, 22)` | **`(0, 182)`** → `(2, 37)` |
|  4 |  4 | `(2, 221)` | `(2, 221)` → `(2, 154)` | `(2, 221)` → `(2, 205)` |
| 12 | 64 | `(2, 1 974)` | **`(0, 785)`** → `(2, 716)` | **`(0, 1 315)`** → `(2, 1 172)` |

`(arcos, primitivas)` do que vai para o traçador; tabela inteira no doc do gate.

**Mecanismo.** O `coarsen` decimava a polilinha e devolvia um perfil por `Profile::new` — sem
decomposição. O doc do módulo dizia *«é o que se quer: a decomposição exacta não sobrevive a uma
decimação que apaga vértices»*. Ela sobrevive: um arco não se achata, e só os troços TESSELADOS
(cúbicas genéricas) têm vértices a apagar. A cura do #2 (`prim_count` no `coarse_doc`) tapava o caso
de arcos e rectas exactas — nunca fica mais barato — e deixava o misto a meio.

**O gate que estava VERDE, e porquê.** `o_preview_nunca_troca_arcos_por_uma_polilinha_mais_cara`
corre o vaso e o círculo — contornos **sem nada tesselado**. *Uma fixtura só de arcos não mede a
parte que não é arco.*

**Cura.** `decimate_arcs_by_turn`: a mesma decimação por giro na decomposição, com a cláusula de
que um vértice onde um arco começa ou acaba fica sempre. Um vértice só sai quando as duas arestas
que o tocam são rectas, logo o que fica entre dois mantidos é ainda uma recta. A polilinha sai
**byte a byte** a de sempre (há gate).

| mutação | gate que a mata |
|---|---|
| C0 o `coarsen` de antes | `o_preview_engrossa_so_o_que_esta_tesselado` |
| C1 as pontas de um arco decimáveis por giro | `um_arco_raso_nao_perde_as_pontas` (⚠️ **só** este: nas outras fixturas as pontas viram `90°` e o orçamento mantinha-as de qualquer forma) |
| C2 a decomposição nunca decimada | `engrossar_guarda_os_arcos_e_so_decima_o_tesselado` · `um_arco_raso_nao_perde_as_pontas` · `o_preview_engrossa_so_o_que_esta_tesselado` |
