# 103 — A DINÂMICA DOS CICLOS (o protocolo desta obra até ao fim)

> **Ordem do Enio, 2026-09-05 — literal, e vale até terminarmos:**
> *«A cada ciclo você deve escolher um grupo de nós já pensando num tutorial ultra interessante e
> didático sobre aquele grupo de nós. Uma vez escolhidos os nós você vai redesenhar e fazer o
> upgrade buscando superar o estado da arte existente no mundo e redesenhar para ficar tão belo
> como o MiniCavalry. Desejamos super performance, facilidade de uso, poder. Ao final do
> redesenho e do upgrade você escreve num PDF de tutos o tutorial referente ao grupo de nós
> reconstruídos. O smoke será o tutorial. Salve essa dinâmica pois assim será até finalizarmos.
> Como no Blender, os parâmetros dos nós devem ser desenhados nos nós e vamos retirar o painel
> lateral.»*

⚠️ **Este doc é o PROTOCOLO, não um plano de uma wave.** Um agente que assume esta linha lê:
este doc · o [102](102_o_outro_patamar_plano_dos_nos_2026-09-04.md) (as portas do código, §0) ·
o [101](101_pesquisa_cartoes_ricos_2026-09-04.md) (o cartão) · e o ciclo aberto na §5 daqui.

---

## §1 — O ciclo, em sete passos (todo ciclo faz os sete, nesta ordem)

| # | passo | entregável | quem valida |
|---|---|---|---|
| 1 | **Escolher o GRUPO** — uma família que o artista reconhece, escolhida **já com o tutorial em mente** | a linha do ciclo na §5 (nós + premissa do tutorial) | — |
| 2 | **Auditar o grupo contra o estado da arte** — params que faltam, poder que falta, o que as referências fazem e por quê | uma secção no doc do ciclo, com fonte por afirmação | — |
| 3 | **REDESENHAR o cartão** dos nós do grupo — params **no cartão**, beleza do Mini Cavalry (§2) | código + gates | o tutorial |
| 4 | **UPGRADE dos nós** — os params/poder que a auditoria achou, com o caminho no **device** | código + gates + prova de mutação | o tutorial |
| 5 | **MEDIR** — a tabela de performance do grupo (CPU · device · passes · objectos/ms) com `loadavg` ao lado | tabela no doc do ciclo | §0.0 |
| 6 | **Escrever o TUTORIAL em PDF** — `docs/Motion Nodes/tutoriais/` (§3) | `<n>_<slug>.pdf` + a fonte `.html` | — |
| 7 | **O SMOKE É O TUTORIAL** — o Enio segue o PDF do princípio ao fim; cada passo é um passo do tutorial | o report dele | **Enio** |

⛔ **Um ciclo não fecha com o passo 4.** Sem a tabela (5) e sem o PDF (6) o ciclo está **meio
feito**, e meio-feito é pior que não começado ([memória](../../project-memory/feedback_perfection_no_deferrals.md)).

⛔ **O tutorial não é documentação do que foi feito: é o SMOKE.** Ele tem de ser executável do
princípio ao fim por quem **nunca viu** aquilo (§0.8): comando completo com o `cd`, o que clicar
com o nome que aparece **na tela**, o que tem de acontecer, e como saber que deu errado. Se um
passo do tutorial não é possível no app, **o ciclo não acabou** — isso é a régua.

## §2 — As quatro leis que valem em TODO ciclo

1. **PERFORMANCE — somos uma game engine** (Enio, 04/09). Todo nó do grupo tem de dizer onde
   corre: no **device** (4,19 M objectos em 3,85 ms) ou na CPU, e **porquê**. Um nó do grupo que
   caia para a CPU sai do ciclo com a razão nomeada e o preço medido
   ([doc 98](98_auditoria_de_performance_2026-09-01.md) · [doc 102 §2](102_o_outro_patamar_plano_dos_nos_2026-09-04.md)).
2. **OS PARAMS VIVEM NO CARTÃO, e o painel lateral SAI** (Enio, 05/09 — decisão de produto).
   Ver §4: a medição que eu tinha escrito contra isto respondia a **outra** pergunta.
3. **A beleza é a NOSSA spec** — a do [plano 01](01_plano_modulo_motion_nodes.md), que o Mini
   Cavalry implementou (o `visual-tokens.js` dele abre com *«Doc PH2D §6»*): silhueta por papel ·
   cabeçalho por categoria · pino por espécie · fio por tipo. Do Blender vêm três
   **comportamentos**: o widget inline que some quando o pino é ligado · a forma do pino = a
   estrutura do dado · o LOD por zoom. Tabela executável: [doc 102 §1.1](102_o_outro_patamar_plano_dos_nos_2026-09-04.md).
4. **Produto final, nunca MVP** — cada nó do grupo shipa o conjunto de params que um profissional
   espera ([memória](../../project-memory/feedback_final_product_every_node_ships_the_full_pro_param_set.md)),
   e o que ficar de fora fica **nomeado com o preço**, nunca em silêncio.

## §3 — O TUTORIAL: formato, ferramenta e regra

- **Fonte** (diffável, versionada): `docs/Motion Nodes/tutoriais/src/<n>_<slug>.html` — HTML com
  os tokens do design system embutidos e CSS de impressão (`@page`).
- **Saída**: `docs/Motion Nodes/tutoriais/<n>_<slug>.pdf`, gerado por
  `bash scripts/tutorial-pdf.sh "docs/Motion Nodes/tutoriais/src/<n>_<slug>.html"`.
- **Ferramenta medida nesta máquina (2026-09-05):** ⛔ não há `typst`, `pandoc`, `weasyprint`,
  `wkhtmltopdf`, `xelatex` nem `libreoffice`. ✅ Há **`google-chrome-stable`** (headless
  `--print-to-pdf`), `rsvg-convert`, `inkscape`, `convert` (ImageMagick) e `pycairo`.
  ⇒ **HTML → Chrome headless → PDF** é o caminho, e é o único que não pede instalação.
- **Estrutura de todo tutorial** (a mesma, para o artista aprender o formato uma vez):
  1. **O que você vai fazer** — uma frase e a imagem do resultado.
  2. **Abrir** — o comando completo com o `cd`, copiável de uma vez.
  3. **Os passos numerados** — cada um: *o que fazer · o que aparece · como saber que errou*.
  4. **O que cada botão faz** — a tabela dos params do grupo, com a unidade e a faixa.
  5. **Vá além** — 3 variações que o artista tenta sozinho.
  6. **Se algo não bater** — os sintomas conhecidos e o que reportar.
- ⚠️ **As imagens do tutorial saem do PRÓPRIO app** (capturas do smoke), nunca desenhos à mão de
  um ecrã que não existe — um tutorial que ensina o contrário do que acontece é pior que nenhum
  ([§5.0 do CLAUDE.md](../../CLAUDE.md)).

## §4 — ⚠️ A minha medição contra «params no cartão» respondia a OUTRA pergunta

O [doc 101 §4](101_pesquisa_cartoes_ricos_2026-09-04.md) recusou *«todos os params no cartão»*
com a tabela *altura do cartão ÷ altura do ecrã* (48 % do iPad mini no `motion.oscillator`,
80 % no `motion.bezier_warp`). **A régua estava errada, e a decisão do Enio é a melhor das duas:**

- o **painel lateral** ocupa o slot do Inspector = **`chrome/inspector-w = 304 px` ABSOLUTOS**,
  em todo quadro, em toda cena: **22,3 %** da largura no iPad 12.9 · **25,5 %** no 11 ·
  **26,8 %** no mini. É área **permanente** e não se recupera;
- o **cartão alto** custa altura **só onde o artista está a trabalhar**, num canvas **infinito,
  panorâmico e com zoom** — e ainda dobra (secções + o nó inteiro).

⇒ *Uma recusa medida responde UMA pergunta, e a minha respondeu «cabe no ecrã?» quando a pergunta
era «o que custa área PERMANENTE?»*
([memória](../../project-memory/feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another.md)).
A recusa do doc 101 §4 fica **REVOGADA por medição**, e o que a substitui:

> **Lei do cartão (nova):** o cartão hospeda **todos** os params do nó, em **secções dobráveis**
> (`ParamGroup`, que já existem), com **LOD por zoom** e widgets vivos **só no cartão quente**.
> O painel lateral de params **sai**. A régua deixa de ser «% do ecrã» e passa a ser
> **`alcance × custo`**: todo param é alcançável sem abrir painel nenhum (gate de censo), e
> nenhum cartão frio regista um widget (gate de contagem).

## §5 — A FILA DOS CICLOS (o ciclo aberto é sempre o primeiro sem ✅)

Os grupos saem do que o **artista** vê na paleta (categoria + sub-cluster), nunca de uma lista
inventada. Contagens do registry em 2026-09-05.

| # | ciclo | nós | tutorial |
|---|---|---|---|
| **1** ✅ | **ARRANJO — pôr muitos objectos na tela** | `motion.grid` · `motion.scatter` · `motion.distribute_radial` · `motion.fibonacci` · `motion.lattice` · `motion.voronoi` · `motion.distribute_poisson` · `motion.distribute_curve` · `motion.path` · `motion.clone` (**10**) | **«Do primeiro objecto ao milhão»** |
| **2** ✅ | **ANIMADORES — fazer andar** | `motion.oscillator` · `value.lfo` · `motion.wiggle` · `motion.noise` · `motion.stagger` · `motion.orbit` · `motion.spring` · `motion.delay` | «O tempo entra no grafo» |
| **3** ✅ | TRANSFORMES & DEFORMADORES | `move` · `rotate` · `scale` · `transform` · `mirror` · `look_at` · `bend` · `twist` · `spherize` · `four_point_warp` · `bezier_warp` · `kaleidoscope` · `spline_wrap` | «Dobrar o mundo» |
| **4** ✅ ([doc 107](107_ciclo_4_foco_os_campos.md)) | FOCO — quem é afectado (campos) | `motion.falloff` · `field.box` · `field.radial_sweep` · `field.index_range` · `field.remap` · `field.combine` · `field.shape` | «Nem todos ao mesmo tempo» |
| **5** ✅ ([doc 108](108_ciclo_5_simulacao.md)) | SIMULAÇÃO | `sim.zone` · `sim.spawn` · `sim.step` · `sim.lifetime` · `sim.collide` · `motion.integrate` · as `force.*` | «Deixar a física decidir» |
| **6** ✅ ([doc 110](110_ciclo_6_valor_e_pulso.md)) | VALOR & PULSO — o cérebro | a família `value.*` e `pulse.*` (**35**) | «Um número que manda em tudo» |
| **7** ✅ ([doc 112](112_ciclo_7_aparencia.md)) | APARÊNCIA (Fx) | `tint` · `color_ramp` · `color_array` · `trail` · `strobe` · `glow` · `drop_shadow` · `rgb_split` · `sub_uv` · `slit_scan` | «A cor e o rasto» |
| **8** ✅ ([doc 113](113_ciclo_8_fontes_e_dados.md)) | FONTES & DADOS | `source.shape` · `source.object` · `source.text` · `source.table` · `source.lsystem` · `motion.emitter` · **`source.camera`** (nasceu no ciclo) | «De onde vêm as coisas» — cena `=119`, [tutorial 8](tutoriais/08_de_onde_vem_as_coisas.pdf) |
| **9** ✅ ([doc 114](114_ciclo_9_rig_e_corpos_moles.md)) | RIG & CORPOS MOLES | `rig.*` (**6**) · `soft_body` · `verlet_rope` · `wave` · `boids` — **10**, contados | «Coisas que se seguram» — cena `=120`, [tutorial 09](tutoriais/09_coisas_que_se_seguram.pdf) |
| 10 ⏳ ([doc 116](116_ciclo_10_o_carimbo_no_dispositivo.md)) | ⚡ **O CARIMBO NO DISPOSITIVO** — `source.shape` + `motion.duplicator` | (optimização, não um grupo novo) | ⛔ **não tem tutorial** (doc 116 §1) |
| **11** | ⚡ **A AVALIAÇÃO GERAL DE PERFORMANCE** — o módulo inteiro, cena a cena | (varredura) | — |
| **12** | ⚡ **OS TETOS CONFORTÁVEIS** — quantos objectos o sistema aguenta, com número | (decisão do Enio, com a tabela) | — |

> ⚠️ **Estado em 2026-09-20.** O ciclo **9** FECHOU — o dono correu a cena `=120`, seguiu o
> [tutorial 09](tutoriais/09_coisas_que_se_seguram.pdf) e aprovou (*«smoke OK»*); os **oito** pedidos
> que ele devolveu pelo caminho estão no [doc 114 §15](114_ciclo_9_rig_e_corpos_moles.md), com a
> leitura de conjunto: **nenhum dos oito foi apanhado por um gate, e cinco não podiam ser** (três
> medem o que a cadeia PRODUZ e a queixa era sobre onde ela é DESENHADA; um é de unidade e outro de
> escala, os dois dentro de números que os gates lêem e aprovam; e o oitavo é um ciclo de DOIS
> quadros, que uma régua de um quadro não vê). ⇒ o ciclo aberto passa a ser o **10**
> ([doc 116](116_ciclo_10_o_carimbo_no_dispositivo.md)), com os passos 1 e 2 fechados no mesmo dia.
> ⭐⭐⭐ **E a auditoria dele CORRIGE a frase desta fila:** o §5.1 abaixo nomeia **um** nó (*«uma
> contagem derivada no planeador»*) e a cadeia do report tem **TRÊS** cercas em três camadas — o
> carimbo, a **fonte** (a `source.shape` sozinha já é fronteira de CPU, medido) e a recusa da ponte.
> ⇒ curar só a contagem **não** põe a cadeia do report na placa; quem ela cura sozinha é o
> `motion.clone`, que é o caso puro.
>
> ⚠️ **Estado em 2026-09-17 (reabertura da linha).** A linha foi integrada e reaberta sobre o `main`
> novo; o ciclo **9 (RIG & CORPOS MOLES) ABRIU** ([doc 114](114_ciclo_9_rig_e_corpos_moles.md)), com
> os passos **1** e **2** fechados. O grupo são **DEZ** nós **contados** pelo censo da paleta — e eles
> vivem em **duas categorias**: cinco **produzem** a coisa que se segura (`Source`) e cinco **agem**
> sobre ela (`Transform`). ⭐ O achado que decide o ciclo é que as duas metades têm doenças
> **OPOSTAS**: os seis `rig.*` têm **o solver e não a interface** (9 params em 6 nós, contra o
> `Strength` que o Rive põe em 7 de 7 constraints) e os quatro corpos moles têm **a interface e não a
> placa** (11–17 params cada, e só o `motion.boids` regista kernel — **1 de 10** no dispositivo).
> ⛔⛔ E isto é pior que o mesmo número no ciclo 7: lá o nó era o **último** do grafo, aqui cinco são o
> **primeiro** e cinco são do meio ⇒ *todo grafo que segure seja o que for corre inteiro na CPU*.
> ⭐⭐ As duas metades curam-se pelo mesmo sítio — **`parent` e `len` não têm ESCRITOR**, e um escritor
> genérico de coluna destrava seis células de uma vez e serve o catálogo inteiro (doc 114 §4).
> ✅ **E a tabela do relógio do ciclo 8 FECHOU no mesmo dia** (doc 113 §7), a `load 2,17` e com duas
> corridas a concordar: o ficheiro de **100 000 linhas custa `0,74 ms`** (4,4 % de um quadro), o que
> é **`4,1×`** o controlo na placa — e a diferença **inteira** está em *publicar* (`0,57` contra
> `0,00`), não em *cozer* (`0,17` contra `0,18`). ⇒ *o que separa uma fonte de DADOS de uma gerada no
> dispositivo é a TRAVESSIA, não a cozedura.* O ciclo 8 não tem mais dívida.
>
> ⚠️ **Estado em 2026-09-16 (fecho da LINHA).** O ciclo **8** FECHOU — o dono correu a cena `=119`,
> seguiu o [tutorial 08](tutoriais/08_de_onde_vem_as_coisas.pdf) e aprovou (*«smoke OK»*). O achado
> que o decidiu **não estava no catálogo**: uma fonte é o PRIMEIRO nó de um grafo, então a costura
> cai NELA e era **re-enviada e re-lida a cada quadro** mesmo parada — `9,62 → 1,62 ms` a um milhão
> de linhas, por uma régua só (*«isto é o mesmo armazenamento»*) que serve as cinco membranas
> ([doc 113 §6](113_ciclo_8_fontes_e_dados.md)). O grupo ganhou um nó (`source.camera`, a VISTA
> dentro do grafo) e a linha **FECHA AQUI**, com o handoff de integração escrito; o ciclo **9 (RIG
> & CORPOS MOLES)** abre depois da integração. ⏳ Fica **a tabela do relógio do grupo** (§7): a
> sonda está comitada e a máquina não desceu de `load 5` nesta jornada.
>
> ⚠️ **Estado em 2026-09-16 (fim do dia).** O ciclo **7** FECHOU — o dono correu a cena `=118`,
> seguiu o [tutorial 07](tutoriais/07_a_cor_e_o_rasto.pdf) e aprovou (*«smoke OK»*). Os dez nós
> ficam na placa (`~2 ns` por linha contra `7–15×` na CPU), e a medição achou e curou um nó em série
> escrito na própria W1c (o `motion.slit_scan`, `16,33 → 3,49 ms` a um milhão —
> [doc 112 §4-septies](112_ciclo_7_aparencia.md)). ⇒ o ciclo aberto passa a ser o **8 (FONTES &
> DADOS)**.
>
> ⚠️ **Estado em 2026-09-16.** O ciclo **7** ABRIU ([doc 112](112_ciclo_7_aparencia.md)), e o achado que
> o decide é o do 6 um nível acima: **seis dos dez nós levavam a cadeia inteira para a CPU**, e um nó de
> aparência é por natureza o ÚLTIMO de um grafo — *todo grafo com brilho, sombra, separação RGB, rasto,
> estroboscópio ou slit-scan corria na CPU*. O primeiro (o `fx.glow`, um passa-tudo) já saiu.
>
> ⚠️ **Estado em 2026-09-15.** O ciclo **6** FECHOU — o dono correu a cena `=117`, leu o
> [tutorial 06](tutoriais/06_valor_e_pulso.pdf) e aprovou (*«tuto ok»* · *«smoke OK»*). ⇒ o ciclo aberto
> passa a ser o **7 (APARÊNCIA / Fx)**. ⏳ **Abertos do 6**, todos com o mecanismo no doc 110: a
> **W1(b)** (o pino ≠ 0 ligado continua a derrubar o nó para a CPU — é um `Compact` complementar, §6) ·
> a **W3b** (três chaves que respondem a perguntas diferentes: `clamp`, `step`, `value` — §9.7) · o
> **RELÓGIO** da W5 (a residência está medida em `32 de 35`; o tempo não, pela mesma falta de máquina
> calma do ciclo 5 — §11.5) · e o `value.cursor`/`value.table` a emitirem **102 400 cópias do mesmo
> número** (§11.4). ⭐ **E os nove `pulse.*` estão no dispositivo sem consumidor**: os três que os
> gastariam são o `sim.spawn` (contagem de nascimento dependente de dados — wave de substrato), o
> `motion.strobe` e o `motion.step`, e **os dois últimos são do ciclo 7** ⇒ *o ganho da W2 é cobrado
> pelo grupo seguinte, não por este* (§8.6).
>
> ⚠️ **Estado em 2026-09-13.** O ciclo **5** fechou: o smoke foi aprovado em 10/09 e o relógio mediu-se
> com a máquina calma ([doc 108](108_ciclo_5_simulacao.md) W5 — e a medição achou e curou um nó em
> série, o `force.buoyancy`, `4,1×`). ⭐ **Antes do 6 entra uma ORDEM DO DONO sobre o 5:** o colisor
> vai para a forma e as peças **colidem sozinhas** dentro da simulação (doc 108 W7 · plano no
> [doc 109](109_o_colisor_na_forma.md)). ⏳ A cena `=114` não foi smokada.
>
> ⚠️ **Estado em 2026-09-09 (fim do dia).** Os ciclos **3** e **4** fecharam com o smoke do
> dono aprovado, e o **5** abriu ([doc 108](108_ciclo_5_simulacao.md)).
>
> ⚠️ **Estado em 2026-09-09.** O ciclo **3** fechou com o smoke do dono aprovado
> ([doc 106](106_ciclo_3_transformes_e_deformadores.md)) e o **4** abriu
> ([doc 107](107_ciclo_4_foco_os_campos.md)). O ciclo **2** tem as cinco waves, a medição e o PDF
> feitos; ⏳ **o smoke do PDF dele nunca foi reportado em separado**, e o dono aceitou o do 3, que
> assenta nele — fica registado por honestidade, não como bloqueio.

### §5.1 — Os três últimos, e por que ficam no FIM

**Pedido do Enio, 2026-09-06**, com a bissecção feita por ele: *«no primeiro grafo tentei colocar
1000×1000 no grid e pesou muito. Retirando Shape e Duplicator fica um pouco melhor.»*

⭐ **Ele acertou o nó, e a medição diz por quê** (`measure_the_stamp_at_a_million`, release,
`load 3,1`):

| lado | objectos | só a grade | grade + carimbo | o carimbo custa | no dispositivo? |
|---|---|---|---|---|---|
| 100 | 10 000 | 0,22 ms 🟢 | 0,61 ms 🔴 | 2,8× | **NÃO** |
| 320 | 102 400 | 0,47 ms 🟢 | 1,71 ms 🔴 | 3,6× | **NÃO** |
| 640 | 409 600 | 1,44 ms 🟢 | 3,91 ms 🔴 | 2,7× | **NÃO** |
| **1000** | **1 000 000** | **3,15 ms 🟢** | **9,27 ms 🔴** | **2,9×** | **NÃO** |

⛔⛔ **O `2,9×` NÃO é o achado — o 🔴 é.** A grade sozinha é reivindicada pelo dispositivo; **com
o carimbo a cadeia INTEIRA cai na CPU**, porque o `motion.duplicator` declara
`lowerings: &[LoweringKind::Cpu]`. O que se perde não são os `6 ms` da tabela: é o caminho que a
[auditoria 98](98_auditoria_de_performance_2026-09-01.md) mediu em **`50,9×`** (4,19 M objectos
em `3,85 ms` no dispositivo contra `195,9 ms` na CPU). *Um nó CPU-only no meio de uma cadeia não
custa o que ele custa: custa o dispositivo inteiro.*

⚠️ **E é por isso que estes três ficam no FIM, e não porque sejam menos importantes:**

1. **`10` não pode vir antes dos grupos.** O carimbo muda a CONTAGEM de elementos
   (`formas × pontos`), que é estrutural e não um mapa por-elemento — pôr isso no dispositivo é
   uma wave de substrato (`StreamOp` com contagem derivada), e ela toca o planeador, que é o
   caminho por onde **todos** os grupos passam. Mexer nele a meio da fila mudaria o custo de cada
   ciclo já fechado, e nenhum deles teria a régua para notar.
2. **`11` mede o que existir, não o que existia.** Uma varredura de performance feita agora
   descreve nove grupos que ainda vão ser reescritos — ela tem de ser a última coisa antes do
   veredito, senão é uma fotografia de um sítio onde ninguém está.
3. **`12` é uma DECISÃO, e uma decisão precisa da tabela pronta.** ⚠️ A lei da casa
   (`CLAUDE.md` §0.0) é explícita: *nunca deixe o fallback definir o produto* — o caso registado
   é o teto posto em `16 384` porque a CPU seria lenta, num módulo que fazia `4,19 M` no
   dispositivo, **256× abaixo**. ⇒ o teto confortável só se escreve **depois** de o `10` decidir
   quem vai ao dispositivo, senão ele é o número da CPU outra vez.

⚠️ **O que o `10` NÃO é:** não é «tornar o duplicator mais rápido na CPU». A pergunta é *o que
tem de existir para uma cadeia com carimbo continuar no dispositivo* — e a resposta provável é
uma **contagem derivada** no planeador, não um kernel. ⛔ Otimizar o laço da CPU primeiro é
exactamente o erro que o §0.0 nomeia.

#### ⭐⭐⭐ O item `10` tem uma SEGUNDA metade, e ela foi MEDIDA em 2026-09-14

Report do dono sobre a cena `=116`: *«usando shape (exemplo: star) fps cai para 27»*. Reproduzido e
medido ([doc 110 §7](110_ciclo_6_valor_e_pulso.md)):

| caso | rota | cook | quadro | fps |
|---|---|---|---|---|
| quads, como a cena shipa | dispositivo | 2,87 ms | **16,92 ms** | 59 |
| quads, **forçados** à CPU | CPU | — | **16,68 ms** | 60 |
| **estrelas** (`PH2D_FIO_FORMA=5`) | CPU (forma viva) | 4,49 ms | **39,77 ms** | **25** |

⛔⛔ **A leitura óbvia está ERRADA, e a 2.ª linha é o controlo que a derruba.** O app imprime
*«CPU: o grafo traz uma FORMA vectorial viva (source.shape)»* e é tentador concluir que os 27 fps
são a queda de rota — **não são**: as MESMAS 102 400 peças forçadas à CPU seguram `60 fps`. Duas
hipóteses minhas caíram na mesma medição (a contagem **não** multiplica — `102 400` nos dois casos;
o cozimento sobe `+1,6 ms`, `4 %` do quadro), e os **`~35 ms`** que sobram estão no **DESENHO**:
cada estrela é um caminho vectorial construído e codificado a cada quadro (`cpu-encode = 39,80 ms`),
contra um quad que é uma textura.

⇒ **o `10` são DUAS perguntas, e a fila tem de as separar:**

1. **A CONTAGEM no planeador** — o que este §5.1 já dizia: uma cadeia com carimbo deixar de derrubar
   o dispositivo. É a metade que a auditoria 98 mede em `50,9×`.
2. **A FORMA como coisa DESENHÁVEL no dispositivo** — o que o report da estrela mede, e que a `1`
   **não** compra: mesmo com a rota curada, `102 400` caminhos vectoriais continuam a ser
   codificados um a um pela CPU. A cura aqui é de RENDER (instanciar a geometria assada), não de
   cook.

⚠️ **E o §0.0 aplicou-se a mim no mesmo dia:** a W1a do ciclo 6 mexeu no planeador e **não**
desbloqueia nenhuma das duas — o que ali falta é a forma chegar ao dispositivo, não um param.

⚠️ **O ciclo 1 carrega o SUBSTRATO** (o cartão passa a hospedar params e o painel sai) — é a única
vez; os ciclos 2+ só pagam o grupo deles. ⚠️ **A ordem dos 2..9 pode mudar** por decisão do Enio;
a do 1 não, porque o resto assenta nela.

> ✅ **FECHADO em 2026-09-17** — a crate `ph2d-panel-motion-params` foi APAGADA (`7 551` linhas em 35
> ficheiros, `66` testes), com o `ParamRow` a mudar de casa antes: [doc 114 §13](114_ciclo_9_rig_e_corpos_moles.md).
> ⛔ **E o achado que vale mais que a remoção está lá:** ela levou o **CONSUMIDOR** de três tectos
> medidos (`MAX_PARAM_ROWS`, `MAX_ENUM_OPTIONS`, `INSPECTOR_MAX_H`), que ficaram com **zero** leitores
> de produto e gates verdes a afirmá-los. *O texto abaixo é o pedido como ele foi escrito, e fica
> aqui porque a estimativa dele errou na direcção que interessa: ele dizia «~85 linhas a mover» e
> foram `592`.*
>
> ⛔⛔ **PEDIDO DO DONO, ABERTO E FORA DA FILA (2026-09-17): RETIRAR o painel lateral de params.**
> Ele está **desligado e não apagado** desde o ciclo 1, e o dono disse no smoke *«não temos mais o
> painel da direita. estamos retirando ele»*. ⚠️ **Não é «apagar 7 558 linhas»: são `85` que têm de
> MUDAR DE SÍTIO primeiro** — a crate do painel guarda o `MotionParamIntent` e a fila por onde as
> edições do **CARTÃO** viajam (`5` sítios empurram, `1` drena), e apagá-la sem a mover pára o
> cartão. Os números, a ordem prescrita, os `65` testes que morrem com ela e a pergunta que falta
> ao dono: [`104_pedido_retirar_o_painel_lateral.md`](104_pedido_retirar_o_painel_lateral.md).

## §6 — Onde cada coisa fica (para o agente não procurar)

| coisa | caminho |
|---|---|
| este protocolo | `docs/Motion Nodes/103_dinamica_dos_ciclos.md` |
| o plano técnico (portas do código) | `docs/Motion Nodes/102_o_outro_patamar_plano_dos_nos_2026-09-04.md` |
| o doc de um ciclo | `docs/Motion Nodes/1xx_ciclo_<n>_<slug>.md` |
| a fonte do tutorial | `docs/Motion Nodes/tutoriais/src/<n>_<slug>.html` |
| o PDF | `docs/Motion Nodes/tutoriais/<n>_<slug>.pdf` |
| o gerador | `scripts/tutorial-pdf.sh` |

---

## §7 — CICLO 1 · a MEDIÇÃO do substrato (2026-09-05, load 2,29 / 2,66 — §5.0 ok)

Antes de pôr um param dentro de um cartão (§0.0: medir antes de limitar). Duas sondas
`#[ignore]`, mesma tela (1200×800), `--release`, melhor de 3×200 pinturas:

| sonda | comando | resultado |
|---|---|---|
| `measure_row_cost` (params) | `cargo test -p ph2d-panel-motion-params --release -- --ignored --nocapture measure_row_cost` | **13,5 µs por row** (marginal: 13,46 · 13,32 · 12,89 · 13,72 · 14,18 de 4 a 33 rows) |
| `measure_card_cost` (grafo) | `cargo test -p ph2d-panel-motion-graph --release -- --ignored --nocapture measure_card_cost` | **11,3 µs por cartão** nu (10,63 · 10,63 · 11,23 · 11,18 · 11,33 · 11,88 de 5 a 120) |

⛔⛔ **E a extrapolação dos 13,5 µs para o cartão estava ERRADA — a construção mediu-a e
refutou-a.** Depois de as rows existirem no cartão, a mesma sonda
(`measure_card_cost`, secção *«20 cartões, R rows»*) dá **2,8 µs por row no cartão** (3 leituras:
2,82 · 2,78 · 2,81 a 8 rows), **4,8× mais barato** que a row do painel.

⚠️ *A mesma «row» custa dois números porque são duas coisas:* a do painel é um **widget vivo**
(slider + chip numérico + reset + registo no `HitIndex`/`WidgetStore` + balão), a do cartão é
**dois textos e dois rectângulos**. Medir uma para prever a outra é a família do
[doc 103 §4](#) outra vez — *uma medição responde à pergunta que lhe foi feita*.

**O orçamento REAL** (16,67 ms; `N × (11,7 + R × 2,8) µs`, medido 2026-09-05):

| cenário | custo | % do quadro |
|---|---:|---:|
| 120 cartões **nus** | 1,41 ms | 8 % |
| 20 cartões × 8 rows | 0,70 ms | 4 % |
| 40 cartões × 16 rows | 2,26 ms | 14 % |
| 120 cartões × 5 rows | 3,08 ms | 18 % |
| 20 cartões × 24 rows (o pior nó do catálogo) | 1,58 ms | 9 % |

⇒ ⭐ **Nenhum cenário do catálogo estoura o quadro** — a decisão do Enio é ainda melhor do que
a defesa que eu lhe tinha escrito. O **LOD fica** (`params_are_drawn`), mas pela razão certa: ele
é a **LEGIBILIDADE** (`11 px × zoom ≥ 9 px` ⇒ `zoom ≥ 0,818`) e o que poupa num grafo afastado
é bónus, não o que torna a feature possível.

⚠️ **A leitura foi tirada a load 6,7–9,9** (§5.0 pede ≤ 5); vale porque as **três** leituras a
loads diferentes (9,9 · 7,2 · 6,7) concordam a **2 %** — um custo absoluto pequeno, não uma
razão de dois relógios. Re-confirmar com a máquina calma no fecho do ciclo.

⚠️ **E a consequência para o modelo:** as rows do cartão **não podem ser `ParamRow`** (que
carrega `String` por rótulo e por valor): 20 cartões × 5 rows seriam 200 `String` por quadro.
O cartão leva `CardParam` — `&'static str` do registry + o `f32` vivo, **zero alocação** — e o
PINTOR formata o texto só dos cartões que de facto desenha.
