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

---

## Índice — o mecanismo de cada um, em uma linha

| # | data | sintoma | mecanismo | gate |
|---|---|---|---|---|
| 1 | 2026-09-16 | *«arestas ainda visíveis»* no ombro do vaso (foto), **depois** da wave que pôs as quinas como arcos | o modo MODEL traça na CPU, e a folha especializada por região lia a **polilinha densa** pelo `ProfileIndex` — a normal ficava constante em cada segmento | `vaso_sem_facetas_tests::o_vaso_nao_tem_facetas_no_traçado_do_modo_model` · `profile_arc_tests::*` |

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
