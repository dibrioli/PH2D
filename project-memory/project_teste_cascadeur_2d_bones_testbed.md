---
name: project_teste_cascadeur_2d_bones_testbed
description: "Teste Cascadeur (13/09): app HTML/JS de ossos 2D FORA do repo, em ~/Área de trabalho/Teste Cascadeur, com física automática + pose automática + secundário sobre o rig da Cascy tirado do Cascadeur; o dono quer os 3 em 2D e 3D e só assina o Indie se o aprendizado render"
metadata:
  type: project
---

**Pedido do Enio (2026-09-13):** aprender com o Cascadeur *física automática, pose automática e movimento secundário*, **2D e 3D**. Começa no plano **Free**; assina o Indie *«se realmente conseguirmos aproveitar o aprendizado»*. Primeiro passo pedido: um app HTML/JS de ossos 2D com rig simplificado, usando o rig padrão do Cascadeur.

**Onde está:** `/home/enio/Área de trabalho/Teste Cascadeur/` (fora do repo PH2D). `index.html` abre por duplo clique (scripts clássicos, sem módulos — `file://` bloqueia ES modules e `fetch`). `motor.js` (cálculo) · `app.js` (tela) · `exemplos.js` (pulo, mortal, soco, em pé — descritos por ÂNGULOS) · `teste.js` (`?teste=1`, 67 verificações) · `gesto_pes.js` e `gesto_chaves.js` (rato REAL por CDP, 4 + 6) · `extrair_rig_cascadeur.py` (corre DENTRO do Cascadeur) → `cascy_bruto.json` → `gerar_rig.py` → `rig_cascy.js`.

**O oráculo da FÍSICA foi colhido (13/09, pedido do Enio):** `bash rodar_no_cascadeur.sh pulo mortal` (~1 min, Cascadeur fechado) monta a nossa animação na Cascy, liga a física DELE, aplica, lê de volta e escreve `cascadeur_resultados.js`; o app mostra o boneco laranja e os números. Montagem conferida: articulações a 0,0 cm (mortal: joelho 5,5 cm de cabeça para baixo; pontas dos dedos 12,7 cm, torção do punho é do rig) e centro de massa a ≤ 1 cm. ⚠️ Para isso o rig 2D passou a ser a PROJEÇÃO EXATA da Cascy (bacia rígida, centro de cada parte com desvio lateral `cmPerp`). **O que a comparação mediu (refeita 13/09 com a entrada corrigida):** voo nos MESMOS quadros (pulo 17–39, mortal 15–37); altura igual no pulo (68 × 68 cm), mortal 66 × 69; no ar o centro de massa dele é parábola (g −9,77 / −10,33, desvio 0,1 / 0,8 cm — o voo dele se acha pelos PÉS, nunca por «caindo com g»); giro no ar igual (mortal 360° × 359,9°, média 1,5° de diferença); anda menos para a frente (1,06 × 1,22 m/s; mortal −0,26 × −0,58). ⭐ **A diferença de desenho que ficou (CURADA em 14/09, ver abaixo): a física dele mexe nos MEMBROS** — no mortal joelho até 44°, cotovelo até 75° e braços/pernas 27 cm para o LADO; no pulo cotovelo 38°; a nossa só move e gira o corpo inteiro. ⚠️ Medir isso DE PERFIL mente (membro apontado para o lado não tem ângulo 2D) — mede-se a dobra em 3D. ⛔⛔ **DOIS «achados» da 1.ª rodada eram FABRICADOS pela nossa entrada defeituosa** (pé afundando até 10 cm entre chaves + braço dando volta no ombro): «ele inclina o corpo 17°/24° na decolagem» (hoje 0°/0,2°) e «decola 2 quadros antes» (hoje os mesmos quadros). Quem os derrubou foi o smoke do Enio, não a comparação. ✅ **MOVIMENTO SECUNDÁRIO comparado (13/09, `rodar_no_cascadeur.sh soco`):** o osso que mais atrasa é o MESMO e quase do mesmo tamanho (mão direita, 45,2° nosso × 45,8° dele), com o pico dele **6 quadros depois** do nosso; ⭐ a diferença é o FIM — o nosso volta exactamente à pose gravada (`0°`) e o dele fica `26°` fora (mão esquerda caída) com o corpo `2,4 cm` deslocado, porque no Cascadeur isto é FÍSICA (o braço relaxa, o corpo reequilibra), não uma mola que puxa para a pose. ⚠️ Lá não existe ferramenta de secundário: aplica-se no «Snap to Physics». ✅ **POSE AUTOMÁTICA comparada pela metade (`rodar_no_cascadeur.sh pose`, 6 arrastos):** os dois levam o ponto ao alvo (nós ≤ 1,9 cm, ele 0) e mantêm os pés; as poses ficam perto quando o alvo é alcançável (mão à frente 10 cm de pior osso, 2,4 cm de média; cabeça 9,8/2,9) e muito diferentes quando não é (mão longe demais 59,7/24,7); o nosso equilibra melhor (peso a 0,1 cm do meio dos pés × 2,1 dele) menos ao levantar a perna (15 × 8,1). ⛔ **Mas isso é o RIG dele, não a rede neural.** ⭐⭐⭐ **A REDE NEURAL foi alcançada em 13/09, por ordem do Enio** (`bash rodar_no_cascadeur.sh arrasto`, ~2 min): o rato é simulado dentro da janela invisível, com a câmera posta de lado e sem perspetiva (o plano do nosso boneco 2D), o pixel de cada controlador tirado da própria IMAGEM da janela e o arrasto feito pelas setas do gizmo com UM controlador escolhido por script. **Seis arrastos, os seis pegaram, e a segunda corrida deu os MESMOS números ao décimo.** O que a rede dele faz: leva o ponto ao alvo (1,1–1,3 cm; nós 0–1,3 cm) e **mexe no corpo inteiro — 57 a 65 articulações**, inclusive no braço que não se tocou; a nossa muda o mínimo. ⭐ **Onde ele ganha:** o passo à frente (peso a 3 cm do meio dos pés contra 15 cm nossos). ⭐ **Onde nós ganhamos:** a mão longe demais (peso a 1,9 cm contra 17,5 cm dele — ele inclina-se e larga o equilíbrio) e a mão no alto (4,1 × 0,3, ele ganha). ⚠️ **A rede RECUSA o inalcançável:** puxar a cabeça 15 cm à frente move-a 10,8 e ela pára a 8,7 cm do alvo, por mais que se insista; a nossa vai a 1,9 cm. As poses ficam perto quando o alvo é razoável (mão à frente: pior osso 16,9 cm, média 3,1) e muito longe quando não (mão longe demais 108,6 / 24,5 — o braço do outro lado é que faz a diferença). ⛔⛔ **E um «espelhamento» que parecia dele era NOSSO:** de perfil as duas mãos caem no mesmo pixel, um clique pega as duas e o arrasto move as duas — lia-se como simetria perfeita (deslocamentos laterais opostos ao centésimo, nos pés também). Sem a cura, a comparação teria sido publicada errada. ⭐ **O app MOSTRA isto:** exemplo «Pose automática: nós × a rede do Cascadeur», um arrasto por quadro, o nosso cinza e o dele laranja, com os números do quadro no painel. Método e as seis armadilhas: [[reference_cascadeur_oracle_door_measured]]. ⏳ 3D não começou.

**Armadilhas medidas:**
- ⛔⛔ **O RIG DELE SE DESMONTA sobre uma pose que não alcança, e nada avisa** (medido 14/09): quando a nossa pose é inalcançável (joelho muito dobrado, corpo a girar depressa), o solver dele ABRE o osso — a canela encolhe 4,6 cm **na entrada** e estica 2 cm na saída, **nas duas pernas com os mesmos valores**. São **12 dos 25 quadros de voo do mortal** e **19 do `ar_giro`**, e é exatamente a janela de onde saíam os números do mortal que este arquivo citava. ⇒ toda comparação passa por um portão de RIGIDEZ (cada osso tem de manter o comprimento a 0,5 cm) e os quadros reprovados ficam fora das médias. *Um ângulo lido de um osso que mudou de comprimento não é um ângulo daquele esqueleto* — [[reference_topic_measurement_discipline]].
- O Python do Cascadeur tem `sys.getfilesystemencoding() == 'ascii'`: um caminho com «Á» morre com `UnicodeEncodeError` e sai com código 255 **sem** escrever o ficheiro de erro. Cura: caminhos como BYTES (`'…'.encode()`), inclusive no `exec(open(...))` do `-c`.
- Cena Cascy: **centímetros** (gravidade declarada 1,0889/quadro a 30 fps = 980 cm/s²), Y para cima, +Z para a frente, 66 articulações, 21 corpos rígidos cuja soma (76,27 kg) é exatamente a massa do «Center of Mass» da cena. Está em pose A (braço aberto some de perfil ⇒ o 2D usa o COMPRIMENTO e pendura o braço).

**Desenho que a medição escolheu (não refaça):**
- Física: prender os pés ANTES de calcular o voo (senão a decolagem salta 9–37 mm); o giro por quadro é resolvido na MESMA forma discreta que o mede (momento angular constante a 1e-13, contra «quase» com integração por trapézio).
- Secundário: modelo de «osso-mola» (ponta = massa em mola presa ao comprimento). O modelo por desvio relativo forçado pelo pai **ressonava** (cabeça a 149°, sem assentar).
- ⛔ **Arrasto — a regra antiga «resolver cada movimento DO ZERO a partir da chave» ERA a instabilidade** (report do Enio, 13/09: *«mover os pontos provoca instabilidade no rig»*): 514 saltos >3× o rato em 21 arrastos, até 1,42 m num passo. Hoje o passo é do motor (`iniciarArrasto`/`moverArrasto`/`assentarArrasto`): parte do movimento anterior e resiste a sair dele, com atração 0,5 à chave (sem ela a pose derivava 113 cm ao fechar o círculo); equilíbrio só com pé no CHÃO e com os braços — e, desde 13/09, TODA perna sem apoio — fora do centro de massa dele (eram o contrapeso que virava); pé preso no AR segura com preço menor (com o preço cheio vira pivô; sem preço nenhum é atirado 55 cm); chão como limite + pouso por IK; ≤ 10°/3 cm por movimento, o resto assenta a cada quadro. Rato real: no ar 2,2 cm de corpo por 2,2 cm de rato; em pé pior 27 cm (alavanca, ≤ 11°/junta); pé preso 0,4 mm; ao soltar ≤ 2,5 cm porque a física acompanha ao vivo (a animação inteira custa 1,6 ms). Equilíbrio peso 25; preso 30 contra 1 do arrastado.
- ⛔⛔ **Uma perna que não SUSTENTA vira contrapeso do equilíbrio e é atirada** (report do Enio, 13/09: *«arrastar um pé provoca o arraste involuntário e exagerado do outro pé»*). Medido pelo rato de verdade: com o pé esquerdo levantado 12 cm, arrastar o direito 25 cm levava o esquerdo **54,7 cm**. ⭐ **O equilíbrio é o amplificador** (sem ele, 16 cm) — a cura é pôr as pernas sem apoio no `foraDoEquilibrio`, onde os braços já estavam pela MESMA razão. ⚠️ **E a regra «pé preso no ar NÃO segura» era a outra metade do defeito:** ela tirava-lhe o preço por inteiro, e sem preço nenhum «preso» não quer dizer nada. ⛔ Mas pôr-lhe o preço CHEIO traz de volta o pivô (uma junta a virar 53° entre dois movimentos do rato, e o próprio pé 59 mm fora): fica com preço **menor** (`PESO_PRESO_NO_AR`) e **só um pé no chão** entra no replantio por IK. Hoje: `0,0 cm`. Gate: `gesto_pes.js` (rato real, com prova de mutação) + um autoteste.
- ⛔⛔ **Prender tudo o que o artista TOCA trava o boneco** (report do Enio, 14/09: *«uma vez que movo o ponto/osso, o osso fica de algum modo preso e não consigo mover novamente»*). O app fazia `travados.add` no ponto arrastado: ao 4.º ponto tocado, puxar o peito 15 cm movia **3,2 cm** (14,6 com o rig limpo). ⭐ **A cura tem duas metades:** um **PÉ** arrastado continua preso (é apoio, não pode escorregar) e **qualquer outro ponto fica só POSADO**, com preço `PESO_POSADO = 0,1` contra `30` de um preso. Medido: a mão posada deriva os mesmos 1,9 cm com preço 30 ou 0,1, e o peito passa de 2,4 para **13,9 cm** — *o que segura a pose é o preço EXISTIR, não a força dele* ([[reference_topic_authored_state_and_clocks]]). ⛔ Subir a atração global à chave foi medido e **recusado** (de 0,5 a 10 a deriva não sai dos 12 cm). Gates: dois autotestes + `gesto_pes.js` (rato real, prova de mutação 3,4 → 14,9 cm).
- ⛔⛔⛔ **«MOLA DE BRAÇO MECÂNICO» — a lei dos membros no ar foi CONSTRUÍDA, MEDIDA e SUBSTITUÍDA no dia seguinte** (report do Enio, 14/09: *«não vi melhorias; as mãos se comportam sem nenhum realismo, como mola de braço mecânico — esse efeito seria melhor numa estrutura sem força voluntária, como mamas grandes»*). Ele tinha razão e a medição foi além do report: **a feature era PIOR QUE NÃO EXISTIR nos três grupos**. Erro médio contra a série do Cascadeur quadro a quadro — pernas / braços / coluna: **não mexer em nada 3,49 / 6,16 / 0,74**; a lei que shipava (f 4, z 0,35, todos os grupos, só no ar) **6,75 / 9,64 / 2,34**. ⭐⭐ **A causa de raiz: os parâmetros tinham sido escolhidos contra RESUMOS** («a dobra máxima do joelho dele é 44°»), e um máximo não distingue uma mola que toca três ciclos de uma que dá um pico e morre — [[feature_worse_than_not_existing]].
- ⭐⭐⭐ **O que o oráculo DE FATO faz, medido por dois casos feitos de propósito** (`ar_parado`: no ar a pose autorada não muda entre os quadros 20 e 36; `ar_chicote`: no ar o braço sobe 170° em 4 quadros e para): **(1) a lei NÃO é «no ar»** — no soco, com os pés no chão o tempo todo, o cotovelo dele desvia 9,8° exatamente no quadro mais rápido (340°/s) e volta; no pulo os braços mudam **24° perto do chão e 2,5° no meio do voo**. **(2) Nas PERNAS não há nada a modelar** — com a pose congelada no ar o joelho dele muda **0,1°** e o tornozelo 0,1°; os nossos mudavam 19,3° e 26,4°. **(3) É quase criticamente amortecida** — depois de o braço parar no ar o cotovelo dele passa **+4,5°** do alvo e acabou; o nosso ia a **+54,5°** e continuava a balançar. ⇒ a lei que ficou: **só os BRAÇOS, sempre ligada, f 2 · z 0,9 · intensidade 0,25** (`bracosQueAtrasam`), erro 3,49 / **5,17** / 0,74. ⚠️ **E mesmo escolhida assim a mola explica 16% do que ele faz nos braços** — o resto não é afinação: parte é lateral (21–27 cm, que um boneco de perfil não tem onde fazer) e parte é um desvio FIXO de 1 a 2° que o rig 3D dele deixa e que não volta a zero nunca.
- ⛔⛔⛔ **A auditoria do MOVIMENTO SECUNDÁRIO (escolha do Enio, 14/09) achou o MESMO defeito e mais dois.** Os números que shipavam (100%, 4 Hz, 0,45, coluna+braços) tinham sido escolhidos contra o resumo *«a nossa mão passa 45,2° do ponto, a dele 45,8°»*. **(1) Aquele osso é o único cujo PISO de medição é 38,9°** — o punho é o rig 3D dele que resolve, e a ponte 3D→pose infere a mão de um par de marcadores: 85% do 45,8° era a régua, e o *«ele fica 26,1° fora do lugar no fim»* estava INTEIRO abaixo do piso. Com a régua certa (dobra, de posições 3D, piso zero) o campeão dele é o **antebraço, 11,3°** no soco. **(2) «Ele assenta no quadro 72 e nós no 32» não era balanço:** parte do desvio dele **não volta nunca** (3,4° parados no soco) — é o rig dele a pousar a pose noutro lugar, e mola nenhuma reproduz isso. **(3) ⭐⭐ Eram DUAS DOSES DA MESMA LEI:** o app aplicava a mola na física (o atraso do braço) e outra vez no painel. Grade de 80 combinações sobre a física: **toda célula pior que a linha de base** (braços 6,56° sem o painel, 6,70° na melhor delas), e com os números medidos o painel dá **exatamente o mesmo resultado** que a física sozinha — porque é a mesma lei. ⇒ **uma lei, um lugar**: a seção «Movimento secundário» passou a ser o painel do atraso do braço (padrão 25%, 2 Hz, 0,9, só braços), ligada por omissão, aplicada UMA vez — como no Cascadeur, onde o secundário não é ferramenta separada. ⚠️ **Subir os sliders é escolha de ESTILO, e o painel diz isso**: afasta do Cascadeur, e serve ao que não tem força própria (cabelo, roupa, carne mole — a ideia foi do Enio). Portão: `veredito_secundario.js` (uma dose 6,56° contra dupla 7,71°, antiga 10,34° e nada 7,48°).
- ⭐⭐ **E a ordem ficou mais simples porque a lei é da ANIMAÇÃO, não da física:** o atraso entra ANTES (é o braço a ficar para trás do ombro que VOCÊ mexeu) e a física corre **uma vez só** — com isso o momento angular do voo é exato por construção (1e-13). A versão anterior corria a física **duas** vezes e emendava só o voo, porque mexia nos membros depois dela e tinha de refazer o giro. Prova de mutação: pôr o atraso depois da física leva o momento angular de 1e-13 para **4,7**.

- ⭐⭐⭐ **A POSE AUTOMÁTICA MEXER NO CORPO INTEIRO (escolha do Enio, 14/09) — e a auditoria achou OUTRA coisa, muito maior.** O buraco é real e está medido com régua nova: quando se arrasta um ponto, a rede dele dobra **6 a 15** articulações e a nossa **5 a 9** (das 11 a 15 que dá para comparar de perfil). ⚠️ **A régua anterior mentia:** ela nomeava «a maior diferença» em centímetros no plano e apontou a MÃO em cinco dos seis arrastos — e a mão é justamente o que mais sai do plano na resposta dele (até **26 cm para o lado** num arrasto). Hoje só entram ossos 80% de perfil nos dois estados, e a contagem diz quantos foram medidos.
- ⛔⛔ **E há uma razão MATEMÁTICA para o buraco, não um defeito:** um solver de alcance não tem como mexer numa junta que não muda a posição do alvo — um braço não move um pé, então ele fica parado. A dele é uma rede treinada em poses humanas. ⛔ **A alavanca óbvia foi construída, medida e RECUSADA:** pôr os braços e as pernas soltas a contar no EQUILÍBRIO (graduável, `fracaoForaDoEquilibrio`) mexe mais juntas e deixa a pose **menos** parecida com a dele (erro 19,3° → 21,5°) — *mover junta na direção errada é pior que não mover*. O que sobra é lei aprendida, não derivável: quando ele agacha, os braços dele vêm 38° à frente porque é o que um humano faz.
- ⛔⛔⛔ **O ACHADO DE VERDADE: nenhum portão perguntava se um gesto é REVERSÍVEL.** Arrastar a mão num círculo de 25 cm no sentido anti-horário e voltar ao ponto de partida deixava a mão a **0,0 cm** e o **corpo destruído** — cabeça **155 cm** fora do lugar, coxas **138°** viradas —, com 88 verificações verdes ([[feedback_a_gesture_that_returns_must_give_back_the_pose]]). ⭐ Curado varrendo os TRÊS números do arrasto contra quatro réguas: **equilíbrio 25 → 2,5 · atração à pose de partida 0,5 → 3 · peso de um ponto preso 30 → 45**. Resultado: erro contra a rede dele **19,65° → 13,8°**, erro do ponto **1,3 → 0,9 cm**, e dos 16 círculos que deixavam o corpo fora do lugar, **14 estavam acima de 5 cm e agora nenhum está**. ⚠️⚠️ **A deriva é CAÓTICA nos parâmetros** (5 cm numa célula, 54 na vizinha): a escolha é pela CONTAGEM num corpus de 60 círculos, nunca pelo pior caso. ⚠️ **Troca declarada:** a maior virada de uma junta por movimento do rato sobe de 11,1° para 15,4° (vem do peso do preso, que teve de subir porque o pé passou a escorregar 4,59 mm) — a barra do portão subiu de 15° para 18° **com os dois números escritos ao lado dela**. ⚠️ E o portão «com equilíbrio o peso fica sobre os pés» foi partido em dois: num alvo IMPOSSÍVEL o equilíbrio ajuda mas não manda (o oráculo deixa o peso sair 17,5 cm e nós insistíamos em 2,0, pagando 52° de contorção nas coxas); num alvo ALCANÇÁVEL ele segura (0,6 cm).
- ⭐⭐⭐ **OS BONECOS E AS ANIMAÇÕES QUE ELE TRAZ EMBUTIDOS (pedido do Enio, 14/09) — a porta abriu e deu DUAS respostas, uma negativa e uma muito forte.** A negativa: das **19 amostras** só **duas** são animações (as outras são bonecos em pose), e nenhuma serve ao nosso pipeline — o mortal deles é **3D de verdade** (cada junta anda até 73,3 cm para o lado, 31,1 de média, contra 3,6 nas nossas) e o `Dracorex_walk_cycle` é quadrúpede com outro esqueleto; quanto aos corpos, **quase todos pesam os mesmos 76,27 kg da Cascy**, os mannequins da Unreal não têm `stomach`/`neck`/`toe_l`, e o único que fala o nosso vocabulário — **OpenPose** — é a Cascy **reduzida 4,7% por igual**. *Não há corpo diferente ali.* ⇒ `importar_amostra.js` recusa com o número, e `sondar_amostras.py` imprime a tabela inteira (quadros · juntas · corpos · massa · altura · fala Cascy?).
- ⭐⭐⭐ **A resposta FORTE: a animação deles confirma a nossa lei, em 3D e sem piso nenhum.** Medida a dobra de cada articulação entre ANTES e DEPOIS da física dele, sobre o movimento feito por ELES (um salto de 8,3 m com queda, 124 quadros, **dois** quadros a tocar o chão): **no apoio, pernas 22,8° e braços 24,5°; no voo livre, 2,0°**. É a MESMA forma que medimos sobre as minhas animações (braços 14,9° na decolagem e 2,0° no meio do voo) — *a lei que shipou não estava ajustada ao meu jeito de animar.* ⭐ E o voo: a altura deles vai de **53,0 cm fora de uma parábola com gravidade falsa de −3,35 m/s²** para **0,4 cm fora, com −9,80** — exatamente o que a nossa física faz. O avanço vai de 65,5 cm fora da reta para 4,7 (o nosso é reta exata).
- ⏳⏳ **E daí saiu a PRÓXIMA melhoria, com número e fixtura: no impacto forte, as PERNAS dele absorvem 22,8° e as nossas ~3°.** A nossa lei é só-braços (medida e escolhida em 14/09 porque no VOO as pernas dele não mexem — 0,1° com a pose congelada). Numa queda de 1,7 s isso muda: ali o chão empurra, e é a perna que recebe. A fixtura já está gravada (`cascadeur_Backflip_animation.json`) e a régua também (`medir_mortal_deles.js`).
- ⛔⛔⛔ **A PERNA NO IMPACTO: hipótese CONSTRUÍDA, MEDIDA e REFUTADA — e quem a derrubou foi um corpus de pessoas reais** (pedido do Enio, 14/09: *«busque mais arquivos gratuitos na net»*). A hipótese vinha da animação embutida do Cascadeur (pernas dele 22,8° no apoio contra ~3° das nossas). Com **três saltos de captura de movimento real** (CMU, sujeito 16), a resposta é outra: **no pouso as pernas dele mudam 2,8° e as nossas 2,8°** — já somos iguais. ⭐ **A demo deles é deliberadamente NÃO-FÍSICA** (a parábola dela dá gravidade **−3,35 m/s²**, e −9,80 depois da física): aqueles 22,8° eram o tamanho do CONSERTO, não uma lei ([[feedback_a_vendors_demo_fixture_demonstrates_the_tool_not_the_truth]]). ⭐ E no voo os três concordam: a captura dá −11,6 a −12,4 m/s², ele entrega −9,60 a −9,73 e nós −9,81, com 0,0 a 0,3 cm fora da parábola.
- ⭐⭐ **A diferença que EXISTE, achada pelo mesmo corpus: o nosso pé é mais preso que um pé humano.** Todo o excesso das nossas pernas vem de «pés apoiados não escorregam» (com ele desligado mexemos **0,0°**). Medido num apoio real: o ponto do pé **percorre 11,5 cm** (calcanhar → rolar → ponta) e **deriva 5,7 cm**; a física do Cascadeur deixa 11,1 / 5,1; a nossa corta para **6,6 / 4,1**. É por isso que no chão mexemos 4,9–6,1° onde ele mexe 0,9–3,6°. ⏳ Mudar isto é decisão de produto: a mesma trava é o que cura o pé a escorregar numa animação feita à mão.
- ⛔⛔⛔ **«NÃO CONSIGO POSICIONAR COMO EU DESEJO — abri as pernas e o boneco mantém as pernas fechadas»** (report do Enio com foto, 14/09). Reproduzido: no quadro 55 do pulo, uma abertura de **65 cm virava 0**. A trava do pé ancorava-o onde ele TOCOU o chão e desfazia toda chave posta depois, no meio do apoio. ⭐ **A cura é «quem PÔS manda»**: uma chave criada pelo artista é marcada `minha: true` e re-ancora a trava; uma chave do exemplo, não. ⛔ **Um LIMIAR não serve e está medido**: a deriva que a trava cura chega a 15,9 cm no mortal e um pé posto de propósito anda 45 — e pela velocidade também não separam (4,7 contra 6,4 cm num quadro). ⚠️ **Três costuras pagaram o caminho:** a marca tem de viajar com a ANIMAÇÃO (`amostrar` marca-a) senão cada instrumento mede outro programa; ela **não atravessa** a passagem do atraso dos braços sozinha (array novo, marca perdida, defeito de volta); e o app respondia à mesma pergunta por outra via (passava TODAS as chaves), o que destravava o pé por completo — **quem apanhou isso foi o teste de gesto real, não a suíte** ([[feedback_the_artists_key_outranks_the_automatic_correction]]). Portões: um autoteste com prova de mutação nas três costuras + `gesto_pernas.js` (rato de verdade, 4 verificações).
- ⭐⭐⭐ **AS DUAS LEIS DO PÉ APOIADO estão no app, por ordem do Enio** (*«coloque as duas opções disponíveis para o usuário»*, 14/09) — um seletor debaixo da caixa «Pés apoiados não escorregam», com o preço de cada uma escrito no painel: **«Preso no lugar»** (`folgaApoio = 0`, o que sempre houve) dá **0,0 cm** de escorregão numa animação feita à mão e paga **3,9°** de perna a mais no chão; **«Rola como o de uma pessoa»** (`folgaApoio = 0,12 m`) reproduz o pé real — num apoio o ponto do pé percorre **11,5 cm** numa pessoa, **11,1** no Cascadeur e **11,1** aqui, e a perna cai para **2,0°** — e paga o pé a escorregar **5,1 cm no pulo** e **12,0 no mortal**. ⚠️ **O valor 0,12 é MEDIDO**, não escolhido: é a folga que faz o caminho do pé bater com o dele (varredura 0 → 20 cm). ⚠️ A folga é uma FAIXA em volta do ponto onde o pé tocou, dentro da qual ele segue a animação: continua a ser uma trava (5,1 cm contra 6,8 sem trava nenhuma), não o pé solto. Quatro portões novos, com prova de mutação (`tol = 0` mata dois deles).
- ⭐⭐ **O CORPUS e as ferramentas ficam** (`animacoes/` com `ORIGEM.md`): 11 clipes da **CMU Motion Capture Database** (licença: *«may be copied, modified, or redistributed without permission»*), convertidos por `bvh_para_estado.js` para o mesmo formato que a tarefa do Cascadeur grava — daí em diante TUDO o que já existia funciona neles. **8 aproveitáveis**, e só para pernas+tronco: num salto real os braços andam ~50 cm fora do plano de perfil, e o portão de `importar_amostra.js` julga **por grupo**. ⚠️ Quatro defeitos meus que só o dado real expôs: uma junta de offset ZERO do BVH envenenou os 11 clipes de uma vez (43 cm, sempre no mesmo osso); transferir pelo QUADRIL deixava o boneco a flutuar 24 cm e a física não achava contato nenhum ([[feedback_transferring_motion_to_another_body_preserves_the_contact]]); poses humanas reais **não cabem** nos nossos limites de junta (cotovelo 33,7° fora em 56 de 81 quadros); e o guarda da cópia única do Cascadeur não via um processo pendurado.
- ❓ **«Por que as trajetórias do manipulável e do resultante são tão diferentes?»** (Enio, 14/09) — não é defeito, é o que a física automática É, e tem duas metades **medidas**: (1) o TEMPO no ar é o que ele animou (24 quadros = 0,80 s) e a altura passa a ser a que a gravidade dá para esse tempo — pulo **29 → 68 cm**, mortal **46 → 66**; (2) no ar nada empurra de lado, então o avanço horizontal vira uma RETA — o autorado foge dela **22 cm** (pulo) e **33 cm** (mortal). As duas curvas coincidem **a 0,00 mm** na decolagem e na aterrissagem: só o meio é substituído. ⭐ A alavanca: para os 29 cm que ele desenhou, o voo teria de durar **15** quadros, não 24. O painel passou a dizer isto com os números do exemplo aberto.
- ⭐ **As chaves arrastam-se na linha do tempo** (pedido do Enio, 14/09): pegar no losango amarelo muda a chave de quadro, com a pose a viajar junto e o cursor do tempo a acompanhar. ⚠️ **Uma chave NUNCA passa por cima de outra** — pára coladinha à vizinha; deixá-la cair em cima apagaria a de baixo sem avisar. Gate: `gesto_chaves.js` (rato real, 6 verificações).
- ⛔⛔ **A pose automática segue o CAMINHO do ponto: saltar para o alvo final escolhe OUTRA solução** (report do Enio, 13/09: *«na terceira pose o nosso boneco está todo embolado»*). O defeito **não era do motor, era do meu banco de comparação**, que arrastava de uma vez enquanto o lado do Cascadeur era puxado aos poucos. «Mão 45 cm para o alto»: de uma vez, cotovelo no limite (150°), ombro no outro extremo (−80°), **seis** juntas encostadas e o ponto a 1,3 cm do alvo; aos poucos, cotovelo 107°, ombro +41°, duas juntas, e **0,0 cm**. ⇒ *quando os dois lados de uma comparação recebem o mesmo alvo por caminhos diferentes, a diferença medida é do CAMINHO* — e o caminho certo é o que o rato faz.
- ⛔ **Interpolar ângulo pelo caminho mais curto atravessa o LIMITE:** o braço do pulo e do mortal dava uma volta inteira no ombro (−50° → +175° pelo lado de trás). Junta com limite interpola dentro da faixa. E `clamp(wrap(x))` salta de um limite ao outro na virada de ±180° (braço −80..180); dentro do solver a junta volta pelo lado de onde o PASSO saiu — o «limite mais próximo no círculo» jogou um cotovelo para o outro lado (mão a 25 mm do alvo, dois autotestes).
- ⛔ **Interpolar ângulos afunda o pé entre chaves** (pulo: 33 quadros, até 10 cm — e era isto que ia para o Cascadeur, cujo boneco laranja afundava 19 cm): ponto do pé no chão nas duas chaves de um trecho fica nele por IK (`plantarPes`), chaves intactas.
- ⛔ **O laço «aponta a ponta, o calcanhar cede» sem freio DIVERGE com o joelho no limite** (0,4 → 140 mm em 6 voltas, ficava a última): `prenderPe` guarda a melhor volta. Mordia a física e a interpolação também.
- O instrumento que achou os três: rato real (CDP) num círculo de 25 cm em 72 passos, cada ponto, em pé e no ar, com a réplica em Node VALIDADA contra ele (mesmos 33,2/101,0 cm) para testar curas em segundos.
- Tornozelo no limite: quem cede é o calcanhar, a ponta apoiada fica.
- Secundário: o giro ANIMADO do próprio osso entra no estado sem inércia (gira ponta, ponta anterior e alvo anterior em torno do pivô); sem isso a mola perseguia o golpe e o soco não esticava. Braço com rigidez 0,3 na pose automática (0,08 virava contrapeso esticado para trás).
- ⛔ **Dois defeitos só apareceram no teste com RATO REAL** (Chrome sem janela + `Input.dispatchMouseEvent` pelo protocolo de depuração, com `Node` 24 e `WebSocket` nativo), com os 37 autotestes verdes: a vista se REENQUADRAVA a cada edição (o «pé saiu do lugar» era a câmera) e o `pointerup` chegava com o último movimento agrupado pelo navegador (a chave ficava gravada uns pixels atrás). ⇒ um recurso de arrastar se prova pelo gesto, não pela função que o gesto chama.

- ⭐⭐⭐ **UPGRADE DA POSE AUTOMÁTICA (15–16/09), SMOKE APROVADO pelo Enio («gostei do resultado»).** Veio da auditoria no próprio Cascadeur (`auditoria/ACHADOS.md`: 13 leis do instrumento, 16 da rede; corpus de 301 arrastos, 170 com resposta, 46 com mão presa a sério; régua `veredito_pose.js`, que imprime SEMPRE a linha de controle). **Entrou:** (1) **a bacia tomba ao agachar** — resíduo que puxa o giro da raiz para −K·anca, `PELVE_K = 0,396` por regressão sobre 46 gestos dele (ele tomba 12–31°, nós tombávamos 1°; a coxa acaba no MESMO sítio nos dois — o que difere é a repartição bacia/anca); (2) **a mão presa é apoio** (entra no polígono; casamento pela PONTA do membro, `rig.membros[m].fim === k.j` — o idioma do pé devolve vazio EM SILÊNCIO); (3) **tornozelo 40° → 53°**, medido em 3D no oráculo (o 40 era palpite e travava: o défice ia para o calcanhar, 8,65 mm, enquanto a PONTA nunca escorregou); (4) tela: botão «Prender as mãos», duplo clique ALTERNA entre círculos empilhados (de perfil as duas mãos e os dois pés caem no MESMO pixel — 0,0 px — e o duplo clique desprendia a primeira). **Números:** agachar 20 cm leva a cabeça a +25,0 cm (ele +23,4; antes −0,9); erro de cabeça 12,3 → 9,0 cm (controle 18,0), CoM 7,4 → 6,3; pé preso 0,10 → 0,07 mm; reversibilidade 2,1 → 1,2 cm. 133 verificações, 14 mutações mortas, 5 gestos de rato. ⛔ **REFUTADAS com número (não reconstruir):** o equilíbrio como FAIXA (perde nas 3 colunas e quebra pé preso + deriva), afrouxar a RIGIDEZ (0,31° contra barra de 1°, e a deriva volta), apertar o TORNOZELO (pior monotonicamente), o braço que sustenta sair do `foraDoEquilibrio`, subir `PESO_EQUILIBRIO` (virada 49,5°). ⏳ **ABERTO:** a lei da bacia foi ajustada só em gestos de PERNA e entra em toda chamada — fora do domínio pode inventar 8–14° de anca (sem cerca); o app não mostra o polígono de apoio; o −60° do tornozelo continua palpite; `h3_*.diff`/`h3_motor_minimo.js` são fotografias da árvore de 14/09 e REVERTEM três coisas se aplicados. ⛔ Duas armadilhas da entrega: um `<b>` dentro de um rótulo `display:flex` vira ITEM e parte o texto em colunas (só a foto apanha; gate (g) em `gesto_maos.js`), e uma prova de mutação corrida sobre a pasta REAL escreveu as fotos da entrega a partir da árvore mutada (o modo `--rapido` já não fotografa).
- ⭐⭐⭐ **OS CÍRCULOS DO MEIO DOS MEMBROS — cotovelo e joelho «à moda 2D» (16/09, escolha do dono entre duas propostas).** No Cascadeur estes três pontos são TORÇÕES 3D (`ACHADOS.md` B18: o joelho dá meia volta à volta do eixo anca→tornozelo e o pé fica a apontar para trás), e o ombro foi **retirado** por isso. Modo derivado da ESTRUTURA do rig em `iniciarArrasto` (a junta do controlador é o `meio` de um `membro`; é perna se o contato daquele membro é um pé), nunca de nomes: **`gira`** (só o osso de cima roda; o resto do corpo congelado no solver por `soLivres`/`raizLivre`) · **`dobra`** (pé preso: agacha — todos os MEMBROS livres, raiz só em y, tronco a prumo) · **`recusa`** (mão presa: nada se mexe e a tela **diz porquê**). ⭐ **E a régua diz que a escolha de produto também é a mais FIEL:** mesmo corpus de 130 casos, com os modos **12,8° · 6,2 · 8,3 cm** contra **13,5° · 8,6 · 10,9** da pose geral; só o joelho **17,2°/1,1/1,5** contra 26,8/31,6/32,9 — quando ELE puxa o joelho o corpo fica parado, e ficar parado acerta-lhe melhor. ⛔⛔ **A armadilha do desenho: um membro com as DUAS pontas fixas não tem para onde levar o meio, de perfil** — três controles ficaram MORTOS e só a medição os mostrou (joelho com pé preso no ar 0,1 cm de 10; cotovelo com mão presa 0,0; joelho com as duas mãos presas na mesa 0,0 **no corpo inteiro**, porque «dobra» só soltava as PERNAS e um braço rígido amarra a bacia à mesa). Curas: pé preso é pé preso (chão ou ar); em «dobra» livres são todos os membros e o que fica a prumo é o TRONCO; e para o braço **recusar com voz** — as três folgas alternativas foram medidas e as três TRAEM o pino (mão presa anda 1,7–8,4 cm; peito inclinado leva a cabeça a 50 cm). ⚠️ O controle que absolve o escorregão de 2–3 cm das mãos presas é o MESMO agachamento pedido pela bacia (arrasto de sempre: 2,3 e 3,1 cm). ⚠️ Duas guardas ficaram **inertes no efeito** (`!s.modo` no equilíbrio, e soltar mais juntas do que é preciso): a primeira é gateada pelo que ela PROMETE — a caixa «Equilíbrio» não muda a pose **ao bit** (sem ela, 1,07e-5 rad). 144 verificações · 26 mutações · `gesto_cotovelo_joelho.js` (8 gestos de rato + 6 mutações). ⛔ **Defeito do INSTRUMENTO que quase passou:** ler o «antes» do osso arrastado DEPOIS do arrasto lê a pose nova (a chave já foi gravada) — lia-se «o cotovelo andou 0,0 cm» com o produto correto.
- ⭐⭐⭐ **A DOBRADIÇA DA ANCA (17/09) — «o corpo desloca-se pelas PERNAS; os braços ALCANÇAM».** O B17 tinha a lei em GRAUS (razão bacia/anca −1 nos gestos de mão) e ficou aberto um dia; medida pelo que se VÊ (quanto cada parte ANDA, `medir_dobradica_da_anca.js` sobre o `--json` do veredito) ela sai sozinha do corpus: no gesto da mão **os dois chegam ao alvo** (ele 24,7 de 25,5; nós 25,4) e só o CORPO difere — a bacia dele anda **5,4 cm** e a nossa **13,8**; puxando a cabeça, 1,1 contra 18,4. ⇒ num gesto de **ALCANCE** a raiz paga `RAIZ_PESADA = 15` vezes a rigidez de posição dela, e o papel sai da **ESTRUTURA** (junta do controlador no grupo `pernas`, ou a raiz ⇒ gesto de pernas). Veredito: pose 12,84 → **12,50**, CoM 6,18 → **5,54**, erro da BACIA 8,11 → **5,9**, corpo inteiro 6,80 → **6,24**; ⚠️ troca declarada: cabeça 8,29 → **8,51** (mora em 2 gestos). ⭐ **O número é ANCORADO NELE:** nos 10 alcances em que a bacia DELE anda (9,9 cm) a nossa passa de 18,9 para 9,6 — e é aí que o erro dela é mínimo. ⛔⛔ **A régua do CORPO INTEIRO aprova o DEGENERADO** (congelar a bacia ganha na média até ao fator 400, porque na maioria dos casos a bacia dele também não anda): só a população «ele MEXEU» rejeita o congelamento — *uma régua de um lado só aprova a resposta degenerada*. ⛔ **Quatro alavancas medidas e RECUSADAS:** tornozelo caro (cura a mão 10,1→5,3 e DESTRÓI o pé 4,8→12,3) · a raiz pesada em TODO gesto (pé 4,8→8,2) · o TOMBO da bacia nos alcances (a razão −1 do B17: erro da cabeça na mão 5,8→16,0) · a raiz pesada a GIRAR (inerte). ⚠️⚠️ **E a lei TIROU O FENÓMENO DA FIXTURA de outro portão:** o tornozelo medido (53°) era apanhado por «um pé preso fica no lugar» e a mutação passou a SOBREVIVER — sem a bacia empurrada o tornozelo deixou de ser forçado ali; o portão novo mede o que o limite de facto faz (com 40° a bacia pára a 15,0 cm de um agachamento de 20,0). ⛔ A cópia de medição foi **apagada** depois de medir (a receita para a refazer está no `motor.js`), para não apodrecer como os `h3_*`. ⏳ **ABERTO e medido:** nos 18 alcances em que a bacia dele FICA (< 1 cm) a nossa ainda anda 10,5 — são os gestos de CABEÇA e PEITO, em que ele **RECUSA** o alvo (cabeça 14,4 de 23; peito 4,4 de 27,5) e nós vamos até ao fim. É a lei seguinte. 149 verificações · 32 mutações · `gesto_dobradica.js` (4 gestos de rato + 3 mutações + foto).
- ⛔⛔ **A RECUSA DELE (17/09): construída, medida inteira e REJEITADA pelo dono — não reconstruir.** Ele leva ao alvo o EFETOR e a bacia e **recusa um alvo da COLUNA** (cabeça: anda 11,3 de 15 e **21,2 de 40**; peito 4,5 de 15). O mecanismo, junta a junta: ele dobra o **pescoço −123°** com a cabeça a contra-rodar +124° e as pernas paradas; nós **agachamos** (coxa 43°, canela −52°, bacia 22,8 cm). A lei estava pronta — o alvo arrastado pesa `0,10` quando a junta do controlador é do grupo `coluna` e não é a raiz — e dava a cabeça a andar 10,4/19,4 contra os 10,9/21,0 dele, com **TODOS** os números a melhorar (pose 12,48→11,71 · CoM 5,54→4,35 · cabeça 8,51→8,12 · corpo 6,24→5,52). ⛔ **Decisão do dono: «ficar obediente»** — copiá-la deixaria o boneco menos obediente exatamente nos pontos com que se posa o tronco, e o report de 13/09 («não consigo posicionar como eu desejo») é o que o app existe para não repetir; *paridade com a rede dele não é o produto do dono*. ⚠️ **E ela tinha um preço escondido que só a suíte mostrou:** `0,10` é o preço de um ponto POSADO ⇒ um ponto tocado passa a empatar com o que se arrasta (peito 6,1 cm com cinco posados contra 5,6 com os cinco PRESOS, 9,9 livre); manter a ordem cura esse e parte o vizinho (o posado é arrastado 6,7 cm) — *a ordem entre «o que se arrasta» e «o que se posou» é uma corda curta*. ⛔ Recusadas pelo caminho: o alvo que **satura com a distância** (Huber, δ = 5/10/20 cm) é **INERTE** porque o arrasto é aos poucos — *a recusa dele não é sobre distância*; **amolecer a coluna** piora a cabeça de 8,5 para 18,8; **endurecer as pernas** tira a bacia (12,0→5,1 cm) e não trava a cabeça. ⏳ ABERTO: a **tabela de rigidez do rig é escrita à mão** (perna 0,03–0,05 contra coluna 0,5–0,6) e nunca foi medida — é ela que nos faz agachar onde ele dobra o pescoço (pernas ×10: pose 12,48 → 11,8, cabeça +0,9).
- ⭐⭐ **O APOIO VÊ-SE (17/09):** a tela desenha a BASE no chão (pés presos que tocam o chão + mãos presas) e o FIO DE PRUMO do centro de massa — verde sobre o apoio, vermelho fora, com os centímetros. ⭐ **Uma porta, dois consumidores** (`M.apoioDaPose`): é a mesma conta do equilíbrio do motor, e o portão ata-as no único instante em que têm de dar o mesmo número — o começo do arrasto (7,320 = 7,320 cm). ⚠️ A cerca não é «o pé está no chão», é «o pé está PRESO» (no motor um pé pousado e solto não sustenta nada) ⇒ depois de «Soltar todos» a barra some, que é o que o equilíbrio faz. ⛔⛔ **E o CONTROLE DO FILTRO da prova de mutação apanhou uma LEI QUE EU ESCREVI DUAS VEZES** (o casamento da ponta da mão) com a mensagem «este texto casa 2 vezes — a mutação NÃO correu»: sem esse controlo, ela teria sido lida como «a mutação sobreviveu» e a duplicação ficava. Hoje a lei vive em `contatoDaPonta`, com dois consumidores. 153 portões · 35 mutações · `gesto_apoio.js` mede os PIXELS da barra (42 verdes em pé · 49 vermelhos fora do apoio · a base passa de 45 para 161 px com as mãos presas · 0 com tudo solto).
- ⛔⛔⛔ **A CAMINHADA (17/09): construída TRÊS vezes, medida, e RETIRADA por ordem do dono — «Não ficou bom! Retire a caminhada dos exemplos». NÃO RECONSTRUIR sem ler `auditoria/ACHADOS.md` B24** (árvore no commit `48004f8`, foto em `fotos/caminhada_RETIRADA.png`). As três: (1) poses minhas calibradas contra uma captura real ⇒ *«não está bom como no app original»*; (2) **a própria captura, retargetada** ⇒ *«parece um robô horrível»*; (3) **as poses do manual de animação** (contato · baixo · passagem · alto) ⇒ *«Não ficou bom!»*. ⚠️⚠️ **A leitura que importa: as três estavam CERTAS em todos os números que eu sabia medir** — a 3.ª batia a captura real na régua da bacia (5,7 cm contra 4,5 de um humano), pé apoiado a 0,00 cm, zero pé dentro do chão, zero quadros de voo, braços em contrafase com correlação −0,94 (a mesma de uma pessoa real). *O que falta não é nenhuma grandeza que eu tenha instrumento para ler*, e a próxima tentativa tem de resolver ANTES de escrever poses: **como se mede «bom» aqui?** ⛔⛔⛔ **Lição transferível: um movimento CAPTURADO de um corpo com pernas 33% mais longas não fica bom no nosso boneco por ser REAL — fica CORRETO e morto. E o aviso está no PREÇO:** a versão capturada precisou de TRÊS leis de compensação para os contactos fecharem (pé de trás a PIVOTAR por IK · altura da bacia pelo pé de apoio · pé assente) — *quando uma fonte precisa de três leis para caber no corpo que a recebe, ela não é a fonte certa* —, e duas delas mediram-se **INERTES** assim que as poses passaram a ser feitas à mão. ⛔⛔ **O rig manda mais que o estilo: o nosso pé tem DOIS pontos e NENHUM calcanhar**, e `plantarPes` só prende quem esteja no chão nas DUAS chaves de um trecho ⇒ um pé que ROLA (calcanhar→ponta, o que um andar humano faz) desliza inteiro, 34 cm medidos. ⚠️ **A régua do escorrega usa 1 cm e não os 3 do motor** (aqueles são a faixa de DETECÇÃO e um pé SAI do chão atravessando-a: 0,00 → 2,80 cm de altura enquanto avança 8,5). ⚠️ **A régua do CONTATO não pode ser o chão** (captura ⇒ pés a flutuar 8–25 cm ⇒ zero contatos): mede-se o pé **relativo à bacia**, com janela de velocidade **centrada**. ⭐⭐ **E três portões nasceram do «robô» e valem para qualquer animação futura, porque NENHUM portão de contacto os vê** (um andar que arrasta os pés, com o corpo a deslizar e os braços colados, passa em todos eles): pé levanta · corpo sobe e desce · **correlação** braço×coxa do mesmo lado — com as barras tiradas do **lado REPROVADO**. ✅ **Ficaram na árvore:** `medir_caminhada.js` (a régua do andar, ainda a medir a captura real) e `foto_tira.js` (a tira de QUALQUER exemplo fotografada do app a correr — generalizado quando a caminhada saiu, porque o nome `foto_caminhada` passava a mentir). **157 verificações · 33 mutações.**

---

## 17/09 — A FLUIDEZ e o BLOCO (report do dono: «o esqueleto é meio travado e não transita livremente entre poses»)

⭐⭐⭐ **A lei da curva mudou: «a tangente é LIVRE, e quem a trava é o LIMITE DA JUNTA».** A
interpolação era MONÓTONA (Fritsch–Butland) e por isso o corpo **parava em cada pose**: a tangente
dela é sempre ≤ a menor das duas encostas e ZERO em toda inversão. Hoje é a derivada do polinómio
pelas **cinco chaves** à volta (com três, é a Catmull-Rom — o mesmo estimador uma ordem acima, nada
afinado), encolhida em forma fechada até a cúbica caber na faixa da junta.

⭐⭐ **A barra sai do lado APROVADO e é maçã com maçã** (`medir_fluidez.js`): reconstrói-se uma
CAPTURA REAL a partir de uma pose a cada S quadros e mede-se, NOS MESMOS QUADROS, a velocidade ao
passar pela pose ÷ o pico local — na nossa reconstrução e na captura. **43 % → 55 %**, a captura
**63 %**, o tecto da família (derivada verdadeira) 68 %. No gesto do dono, **20 % → 37 %**.

⚠️ **Três cercas que a lei exige, cada uma com o número:** um canal SEM limite fica com a curva
conservadora (a raiz passava 8,5 cm e 15,3° além das chaves, com o pé 44 mm no chão) · **a PAUSA é
PAUSA** (duas chaves iguais; sem isso o braço respirava 3,75° numa pose congelada) · e o **chão entre
as chaves** (`naoAfundarEntreChaves`, 31,4 → 0,00 mm).

⛔⛔ **Três defeitos PRÉ-EXISTENTES que a curva nova destapou — e o que os escondia era um ACIDENTE:**
`fixarApoios` não era idempotente (a física chama-o 2×; a decolagem saía 13,6 mm fora) · o IMPULSO
enfiava o pé 1,8 cm no chão · e o contacto que o próprio impulso cria era invisível ao pino. **A
curva monótona mergulhava a ponta do pé a 0,4 cm do chão entre duas chaves que a põem a 5,6** — a
descida espúria caía na faixa de 3 cm e fazia o pino armar. *Um defeito tapado por outro.*

⛔ **O CHÃO É CHÃO PARA TODO PONTO DE CONTACTO, não só para o pé:** o dono arrastou a mão 33,8 cm
para DENTRO do chão e a física puxava-a de volta ao tocar ⇒ **34,1 cm** entre a pose feita e a
tocada. Curado: fidelidade da sessão **34,11 → 3,35 cm**.

⭐⭐⭐ **O BLOCO — perguntado ao Cascadeur, correndo-o:** ele **NÃO tem osso nem objeto pai de todos**.
289 objetos, nenhum move os 43 pontos; o ponto da bacia move 27 e só 19 sem deformar. O bloco dele é
a **SELEÇÃO** (os mesmos +10 cm nos 43 ⇒ os 43 andam 10 cm; o `selector` tem `select`/`selected`/
`pivot`). Aqui a seleção inteira é a RAIZ da pose, e a alça vive **no chão**, debaixo da bacia, 46 px
abaixo da linha (acima dela moram a barra do apoio e a etiqueta do peso). ⚠️ Ela vem **ANTES dos
ossos** no hit-test — com os ossos à frente o arrasto pegava no pé e o corpo andava 0,0 cm de 30,
calado — e `pontoDoCorpo()` devolve coordenadas **da janela**.

**Instrumentos novos:** `medir_fluidez.js` · `gesto_sequencia.js` (animar seis poses com rato de
verdade: alcance · fidelidade · fluidez · o chão · o bloco) · `cascadeur_tarefa_esqueleto.py` +
`medir_esqueleto_deles.js` (`bash rodar_no_cascadeur.sh esqueleto`).
**Estado:** 164 verificações, 43 provas de mutação, 8 testes de rato de verdade — todos verdes.
Detalhe inteiro com as tabelas: `auditoria/ACHADOS.md` B25 e B26.

## 18/09 — O PIVÔ do bloco: ele roda o corpo à volta do QUADRIL

⭐⭐⭐ **Perguntado ao Cascadeur CORRENDO-O** (`bash rodar_no_cascadeur.sh pivo`): o `selector()` dele
devolve um `Pivot` com quatro campos, e o que decide é o **`center_of_top_objects`**. Lido o
`position` por escolha: as duas mãos → o meio das duas; os dois pés → o meio dos dois; **os 43 pontos
(o corpo inteiro) → `0.0009, 95.6335, −5.1921`, que é A BACIA ao quarto decimal** — e não o meio dos
43 (`0.0013, 88.4123, −1.8463`) nem o centro de massa (`0.0012, 98.1457, −4.0223`).

⇒ aqui é a operação mais simples que existe: **a raiz da nossa pose É a bacia**, logo rodar o bloco é
somar ao ÂNGULO da raiz e mais nada. O gesto são dois botões com seta curva nas pontas da base de
mover (no chão, onde não disputam com círculo nem osso), e o ângulo do rato mede-se **no pivô**, não
no botão — é daí que vem o controlo fino. ⚠️ O pivô é DESENHADO durante o gesto (cruz na bacia + fio
+ graus): um giro à volta de um ponto que não se vê lê-se como o boneco a fugir. ⚠️ E o chão continua
a ser chão: a bacia pode SUBIR num giro, e o que não muda é o **x** dela.

Medido com rato de verdade: 30° pedidos, **30,00°**, deformação **0,0000°**, bacia **0,000 cm**.
O `gesto_sequencia.js` passa a dar VEREDITO sobre os FATOS (chão · bloco rígido · giro); a fluidez e
a fidelidade ficam como medidas impressas. Detalhe: `auditoria/ACHADOS.md` B27.

## 18/09 — A ATERRISSAGEM DE PERNAS ABERTAS: a âncora do apoio era uma ESCADA

Report do dono: «se abro as pernas no penúltimo Key, a transição entre o penúltimo e o último fica
bizarra, com saltos de posição e trancos». Cena nova `aterrar_aberto` (pedida por ele): um salto que
aterra numa passada de 55 cm e acaba de pé com os pés juntos. ⚠️ Os ângulos das duas chaves abertas
foram **PROCURADOS** (varrer canela/pé dos dois lados até os dois pés assentarem à mesma altura,
0,1 e 0,5 mm), não escolhidos: mexer num à mão tira um pé do chão.

⛔⛔ **A âncora do pino (`fixarApoios`) era reposta em cada chave — um DEGRAU.** Com a canela direita
apoiada do quadro 40 ao 63 de uma vez, ela era `xs[40]=110` até ao 47, saltava para `xs[48]=139` num
quadro e caía para `xs[63]=112` no último: dois teleportes de 29 e 27 cm. ⭐ **A lei certa já estava
escrita no `plantarPes`** («andando em linha reta de onde uma chave o põe até onde a outra o põe») —
no pino faltava, e as duas metades do mesmo apoio discordavam. Hoje é uma RAMPA entre marcos (o
início da faixa e cada chave dentro dela). Tranco do pé no último trecho: **44,1 → 4,8 cm** (o
controlo de pés juntos: 1,2 → 2,3).

⛔⛔ **E um segundo, mais fundo: `porChaves` não propagava a marca `minha`** ⇒ NENHUM exemplo tinha
colocação de pé. No `aterrar_aberto` o pé ficava pregado em 93 cm do quadro 40 ao 63 enquanto as
chaves o levavam a 135 — 50 cm de erro, e a tela a dizer «o peso está 10 cm FORA do apoio» numa pose
final equilibrada na chave. ⚠️ **E a minha primeira régua não o via: ela punha `minha: true` à mão e
media OUTRO PROGRAMA que o app.** Hoje a marca atravessa da chave, e só se marca a chave em que o pé
está de facto no chão. ⚠️ `pulo`/`mortal`/`soco` não marcam nenhuma e por isso não mudam.

⚠️⚠️ **O PISO DE POPULAÇÃO foi metade do portão da colocação**: sem a marca há ZERO chaves marcadas,
o laço varre nada e a distância lê 0,00 cm — verde sobre 50 cm de erro. Quem o apanhou foi a prova de
mutação. Instrumento: `medir_aterrissagem.js` (o controlo de pés juntos corre na mesma passagem).

## 18/09 (2.ª) — O GELO: deslize e tranco são as DUAS PONTAS DO MESMO PAU

Report do dono, horas depois de aprovar a cura acima: *«desliza como no gelo. o que eu desejava era
apenas que os trancos e pulos não acontecessem. **Pelo menos um pé deveria ficar fixo.** Não acha?
Estude no cascadeur.»*

⛔⛔ **Ele tem razão e o portão da rampa estava VERDE sobre o defeito** — ele media só o TRANCO.
Escada: deslize **0,0** / tranco **44,1 cm**. Rampa: deslize **41,3** / tranco **4,8**. Pessoa real
(46 apoios, 6 capturas, medida com o MESMO código): **5,2** (mediana) / **8,0**. ⇒ *uma régua que
mede uma ponta do pau APROVA o defeito da outra ponta*, e as duas passam a viver no mesmo bloco
(`medir_deslize.js`). Depois da cura: **8,9 / 3,6**.

⭐ **O ORÁCULO respondeu as duas metades, e nenhuma por leitura de fonte:** (a) a documentação
pública do Cascadeur receita, contra deslize de pé, pôr a interpolação em `Fixed` e **colar a mesma
posição em todos os quadros** do trecho — a posição de um apoio é CONSTANTE, nunca interpolada; (b)
na caminhada embutida dele (`Dracorex_walk_cycle.casc`, corrida por `rodar_no_cascadeur.sh pisada`)
**41 dos 41 quadros têm pelo menos um pé pousado**, e o pé muda de sítio pelo AR (voo = 44 % do
ciclo, sobe 0,114 do comprimento do passo). ⛔ Só **2 das 19 amostras** dele têm animação; as outras
são rigs de 1 quadro.

⚠️⚠️ **A sonda do oráculo mentiu DUAS vezes antes de dizer a verdade:** escolheu o eixo «para cima»
como o de **maior extensão** e leu o COMPRIMENTO do dinossauro em vez da altura (caminhada deitada);
e definiu o chão como o ponto mais baixo da cena, que ali é a **CAUDA** (respondeu «2 pontos tocam o
chão» e os dois eram da cauda). ⭐ O discriminador certo do eixo vertical é o **CHÃO**: só ele tem os
dados todos de um lado do zero.

**A lei:** *um apoio NÃO ANDA — ele SAI DO CHÃO para mudar de sítio, e dois pés nunca o fazem ao
mesmo tempo.* Um sítio só (`janelaDoPasso`/`noPasso`/`repartirPassos`), **dois consumidores**
(`plantarPes` entre chaves · `fixarApoios` no pino) — ⛔ os dois são precisos: só no pino a cena volta
a ter um quadro com os dois pés no ar; só no `plantarPes` a física do mortal reprova.

⚠️ **A CERCA é geométrica e sai do RIG:** um contacto não pode andar mais que o próprio pé (17,1 cm)
sem o pé sair do chão. ⛔⛔ Com a cerca na mediana da captura (5,2 cm) o `mortal` ganha dois saltinhos
a 0,2 s de uma decolagem e **TRÊS portões da física reprovam** — e ao medir isso apanhou-se o que
aquelas chaves fazem: o tornozelo FICA e a **PONTA é arrastada 14,7 cm** para trás (elas rodam o pé à
volta do tornozelo). ⚠️ **Margem de 2,4 cm (14,7 de 17,1)**, escrita no portão.

⚠️ O PISO da subida (`√2 · TOL_APOIO`) é o único número que não vem de fora: vem do **recurso desta
casa** — abaixo de `TOL_APOIO` o motor ainda chama o pé de «apoiado» —, e `√2` é a altura em que o pé
fica FORA da faixa de contacto durante metade do passo. Sem ele o pé anda 22 cm sem sair da faixa.
Detalhe: `auditoria/ACHADOS.md` B29.

## 18/09 (3.ª) — «os pés ainda deslizam»: a régua que aprovou o defeito era a CERCA DO MOTOR

⛔⛔⛔ **A barra da lei do passo estava CONTAMINADA.** Ela dizia «uma pessoa deriva 5,2 cm num apoio»,
e esses 5,2 saíam de contar como apoiado todo ponto a menos de **3 cm** do chão — que é a janela em
que o MOTOR decide prender um pé, não «o pé está no chão». Medida a **5 mm** sobre as mesmas seis
capturas: **0,92 cm** de mediana, 3,2 no percentil 90. *A régua media a janela de decisão do motor e
chamava-lhe o chão.* ⚠️ E ela estava em três sítios, um deles **dentro do portão**, que por isso ficou
verde sobre um pé a arrastar 8,9 cm.

⭐⭐ **Apertar a cerca revelou que a FÍSICA DESFAZIA O PASSO, por duas vias:** o IMPULSO desce o corpo
e a trava do chão gasta exatamente a folga do pé no ar (o pé ia a 0,00 e escorregava 5,4 cm no
`pulo`); e a AUGMENTAÇÃO do `det` declarava contacto nesses quadros, fundindo a faixa do apoio POR
CIMA do passo (no `mortal` a âncora do quadro de DECOLAGEM saltava de −1,24 para 14,64 cm).

⭐ **A cura é arquitetura:** o passo é planeado UMA vez onde as chaves são conhecidas (`plantarPes`),
GRAVADO nas poses e REIMPOSTO pelo pino — o alvo é do MUNDO. O pino **deixou de planear passos**.

⚠️⚠️ **E a marca teve de ser levada à mão TRÊS vezes — a lição já estava escrita no ficheiro**, para
as chaves: *«uma propriedade que atravessa uma transformação tem de ser levada à mão»*. Sintomas
diferentes por travessia: sem a do atraso dos braços, o atraso passava a mexer as **PERNAS 16,2°**.

Deslize: `aterrar_aberto` **8,9 → 3,9 cm** (e os 3,9 são o CHOQUE da aterrissagem), `pulo`/`mortal`/
`soco` **0,0**. ⏳ ABERTO: falta **deslocar o peso** antes de levantar o pé — a cerca «um pé só anda se
o corpo puder ficar sobre o outro» foi construída, medida, e **recusa o passo que o dono quer** (para
fechar 55 cm de passada o peso está a 41,4 cm do pé que fica). Decisão do dono. Detalhe: `ACHADOS` B30.

## 18/09 (4.ª) — auditoria da aterrissagem CONTRA a física do Cascadeur

Pedido: «auditoria buscando melhorias simulando a mesma animação no cascadeur». A cena com os passos
entrou como chave em todos os quadros e a física automática dele correu por cima
(`medir_aterrissagem_deles.js`; `auditoria/AUDITORIA_aterrissagem_vs_cascadeur_2026-09-18.md`).
⭐ **Na queixa do dono a nossa física é 4× melhor que a automática dele: 3,9 contra 16,2 cm de deslize**
— ele ACHATA o 2.º passo e arrasta o pé; a ferramenta dele de prender pés é manual. ⭐ **Achado e
curado:** o pé que pousa do passo recuava 3 cm (a faixa do pino nascia onde o pé a DESCER roçava a
janela de 3 cm, ainda a andar; um quadro de passo deixou de semear faixa). ⏳ **Aberto com desenho:**
o pino prega o CALCANHAR no chão quando o artista o deixa 3,5 cm levantado (ele honra-o) — a cura é a
âncora 2D nos marcos. ⏳ **Decisão:** o choque — ele 4,6 g em 4 quadros afundando 8,7 cm abaixo da
chave, nós 6,4 g num quadro afundando 3,7. ⛔ O deslocamento do peso **não tem oráculo**: ele também
não o faz (1,5 cm). *Um oráculo que se corre responde também «isto ele NÃO faz».*


## 19/09 — «porque no PULO os pés não deslizam e ao POUSAR de pernas abertas sim?»

Pergunta do dono. A resposta não era a física: era **quem manda no pé**. As chaves do `pulo` não são
COLOCAÇÕES DE PÉ (vieram com o exemplo), então o pino prega o pé onde tocou e ignora-as; as da
aterrissagem são (é o que o artista grava), e ali a chave ganha — *a animação ATERRA num sítio e a
colocação manda o pé estar NOUTRO dois quadros depois* (93,4 → 97,3 cm), e o motor arrastava-o.

⭐ A cura tem TRÊS metades e nenhuma basta: **(1)** o pé PROCURA o sítio enquanto ainda está no ar
(`mirarAPousada`; janela = o comprimento do pé, nunca atravessa chave; régua adimensional: a última
passada antes de encostar, 0,97× com mira e 1,37× sem — *pôr a âncora no sítio da chave SEM mirar
troca o arrasto no chão por um SALTO no ar, e um número de deslize não vê isso*); **(2)** no trecho
em que o pé acabou de pousar a âncora do pino é já a da colocação (a única metade que alcança o
TORNOZELO: 1,34 → 0,04 cm); **(3)** um contacto que o IMPULSO criou não manda num que já existia (ele
desce o corpo, o tornozelo encosta, e «manda quem tocou primeiro» arrastava a PONTA 3,39 cm para fora
da colocação pela distância fixa do osso).

Deslize: `aterrar_aberto` **8,9 → 3,9 → 0,8 cm**; os outros 0,0. Detalhe: `ACHADOS` B32.


## 19/09 (2.ª) — o CALCANHAR: o impulso encostava-o e o pino declarava o pé ASSENTE

Ordem do dono («ataque o calcanhar»). ⛔⛔ **A minha nota da auditoria dizia a causa ERRADA** — que o
`det` «roçava a janela de 3 cm»; medido, ele NUNCA marca aquele tornozelo. Quem o marca é a
**augmentação do impulso**, e foi a BISSECÇÃO que o mostrou. *Uma causa escrita numa auditoria e não
bissectada é uma hipótese com cara de medição.*

O impulso desce o corpo, o calcanhar encosta, a augmentação declara-o contacto (ela existe para ele
não ESCORREGAR, 70 mm medidos) e o ramo «os dois no chão» impõe a geometria de pé ASSENTE (o `dy` sai
das SOLAS): o tornozelo era pregado a 0,00 por quatro quadros e SALTAVA de volta no quinto.

⭐ A lei ganhou um degrau: de «não MANDA num que já existia» (um desempate) para **CEDE POR INTEIRO**
— não conta sequer como segundo ponto de apoio. ⚠️⚠️ E **quem cede a geometria não cede o CHÃO**: a
1.ª redacção deixava o ponto enterrado 0,84 mm; o resgate é o `tirarDoChao`, que roda o pé sobre o
outro ponto sem mexer na altura de quem manda. Calcanhar: **3,49 fora → 0,01 cm**; o pé inteiro
4,47 → 1,24 cm da colocação. ⚠️ A régua é NA CHAVE e só nela — entre chaves o calcanhar desce e sobe
de propósito. Detalhe: `ACHADOS` B33.


## 19/09 (3.ª) — a prova de mutação dizia «SOBREVIVEU» sobre uma mutação que MORREU

A prova final do calcanhar fechou vermelha com uma sobrevivente. Aplicada à mão, ela **reprova** — na
fixtura sintética da metade simétrica da lei de cedência. Não era a lei: era a **escrituração**. O
caso nomeava o portão da colocação, que é consequência do ramo de CIMA (o calcanhar é obra do
impulso); o ramo mutado é o simétrico (a ponta é obra do impulso), e **nenhum exemplo desta bancada
pousa de calcanhar**, logo ele não alcança a colocação. Medido ramo a ramo: `fimCriado` defende
TRÊS portões (a mira 1,51× · a colocação 3,39 cm · o calcanhar 3,49 cm) e `pontaCriada` defende
**só** a fixtura própria (3,90 → 0,00 cm).

⇒ o portão da colocação não tinha **nenhuma** mutação que o matasse. ⛔⛔ E o arnês escondia a prova
de que a mutação tinha morrido: na sobrevivência ele imprime só os portões ESPERADOS que ficaram
verdes, e um vermelho que ninguém nomeou só aparece **quando ela morre**. Curado nas duas leituras.
Detalhe: `ACHADOS` B34.


## 19/09 (4.ª) — as DUAS opções: deslocar o peso · a força do impacto

Ordem do dono: *«como opções e não substituir o que temos»*. Desligadas, os 17 exemplos saem **byte
a byte** iguais, com portão.

**(a) O peso** são **DUAS leis num selector**, e a partição foi decidida pela FOTO: *o que a física
deixa* (o corpo anda **0,2 cm**, o desequilíbrio 135,3 → 135,0 — o boneco fica em cima do laranja) ×
*o quanto for preciso* (desequilíbrio 135,3 → **0,6** em seis exemplos, e o corpo anda **54,5 cm num
quadro**, ficando 55 cm atrás do laranja, sentado atrás dos pés). ⛔ Nenhuma é shipável sozinha —
uma é uma caixa que não faz nada, a outra é pior do que não existir — e nenhum NÚMERO o dizia.
Nas duas, o corpo mexe-se **e mais nada** (chave 0,0000 cm · ponta 0,0000 · nada no chão escorrega).
⚠️⚠️ A cerca é o PÊNDULO INVERTIDO (`ẍ = g·d/h`): na chave da aterrissagem o peso já está **em cima
da borda da frente do apoio** (`d ≈ 2 cm`) ⇒ o pé não tem de onde empurrar. Ela deixa **7,4 cm** em
dez quadros, `7,4×` menos do que equilibrar pede, e dar 5× mais tempo move 1,7 cm ⇒ *falta ALAVANCA,
não tempo*. ⭐ E o número nomeia o defeito verdadeiro: o motor levanta **o pé em que o peso está**
para o mover 5 cm — a cerca do passo (B33), decisão do dono.

**(b) O impacto** cai de **6,62 para 4,70 g** (o oráculo: 4,6), afundando 8,1 → 12,6 cm (ele 8,7), e
**só amacia**: o `pulo` (3,29 g) e o `mortal` (3,62) saem byte a byte iguais. ⭐⭐⭐ A lei é *o corpo
volta à pose o mais depressa que consegue sem nunca passar do pico* — controlo de tempo mínimo de um
duplo integrador; o afundamento é a SAÍDA, não uma entrada.

⛔ **Quatro redacções derrubadas por medição** (uma janela por trecho em vez de por pé · o perfil
aplicado à RAIZ e não ao PESO · a travagem medida da POSE que já desce · a altura lida do quadro que
a lei acabara de baixar) e **três linhas que nenhuma mutação matava**, duas delas apagadas com a
medição escrita. Detalhe: `ACHADOS` B35.


## 19/09 (5.ª) — «um salto anómalo nos keys 49 e 50»: eu tinha o número e shipei-o como PREÇO

Report do dono sobre a opção do peso no modo *forçado*. O perfil era um **PULSO QUADRADO** (`q48 0,0
· q49 −54,5 · q50 −53,1 · q51 0,0`): dois teletransportes por passo, com o corpo a andar 55,9 cm num
quadro onde ele anda 1,4.

⛔⛔⛔ **A suíte imprimia esse número em toda corrida, como linha MEDIDA, e eu escrevi-o como o PREÇO
da opção.** *Documentar um defeito numa linha medida não o transforma num preço aceitável* — a
distinção que eu não fiz é entre ANDAR 54 cm e SALTAR 54 cm entre dois quadros.

⛔⛔ **E a causa foram as DUAS linhas que eu tinha apagado por «inertes»**: o apoio a saltar o pé que
está a dar o passo (sem ele a base salta de larga para estreita a meio da janela) e a rampa. Eu medi
a SOMA e o PIOR do desequilíbrio, iguais ao cêntimo — *uma soma e um extremo não veem um degrau no
meio*.

⭐⭐⭐ **A cura não tem um número escolhido: o corpo nunca anda mais depressa do que ele já anda nesta
animação** (a velocidade própria máxima da raiz, 8,2 cm/quadro), limitada em duas passagens.
⚠️ E ela mede-se UMA VEZ na animação como chega — dentro do laço ela CRESCIA (8,2 → 10,3 → 13,2 →
14,5), cada janela a ganhar licença porque a anterior já tinha andado.

Resultado: desequilíbrio somado 135,3 → **60,9** (pior 44,7 → 30,6) com o corpo a ganhar **1,18×** a
própria velocidade (era 6,6×). Barra no vale medido: 1,18 · 1,37 sem a rampa · 6,6 sem a taxa.

⛔⛔⛔ E uma mutação — *«a opção deixa de ser opção»* — ficou **invisível a todos os portões** uma
volta inteira, porque a lei da física move 0,2 cm. O portão da inércia ganhou a **terceira metade**:
*ligar tem de mudar alguma coisa*. Detalhe: `ACHADOS` B36.


## 19/09 (6.ª) — o botão que escolhe QUAL simulação é o resultado

Pedido do dono. No topo de *Física automática*: **Resultado final: a NOSSA física / a física do
CASCADEUR**. O resultado passa de **45,68 cm** de distância da simulação dele para **0** (é a dele ao
bit) e o *Exportar animação* vai junto. O laranja passa a ser sempre **A OUTRA** física; os apoios e
o «no ar» seguem o resultado; e o painel de baixo diz, na 1.ª linha, que descreve a nossa.

⛔⛔⛔ **A FOTO apanhou o que o número não via:** com a dele escolhida o número dizia `0 cm` e o ecrã
mostrava o boneco **tingido de laranja**, porque o laranja era desenhado depois. *Um número certo com
a ordem de desenho errada lê-se, no ecrã, como um botão partido.* ⇒ o RESULTADO fica sempre por cima.

⛔⛔ **E a régua desse portão media no sítio onde o defeito não pode aparecer:** ela escolhia a
articulação onde as duas físicas mais se AFASTAM — e ali o laranja nem está. A mutação da ordem
sobreviveu com o portão a ler «28 contra 0». O sítio é onde elas se **CRUZAM**, o discriminador é o
**AZUL** (sólido B = 243 · tingido B = 151), e o ponto é o **meio do osso** (a junta da cabeça cai
fora do desenho). Medido: 81 limpos / 0 tingidos · 0 / 81 com a ordem invertida.

⚠️ Este portão só pode viver no teste de RATO: a `teste.js` corre fora do navegador e o
`cascadeur_resultados.js` não existe naquele mundo. Detalhe: `ACHADOS` B37.

## 2026-09-19 — os DOIS AJUSTES da física do Cascadeur (report: «superiores em tudo, EXCETO…»)

Ordem do dono: *«As simulações Cascadeur parecem superiores em tudo, exceto na fixação dos pés e na
mola das mãos. Coloque opções de ajustes para cascadeur»*. As duas afirmações foram **medidas antes
de se escrever código** (`node sonda_ajustes_do_cascadeur.js`) e as duas são verdade.

**O que existe agora** (as duas caixas só são alcançáveis quando a física dele é o resultado final, e
nascem DESLIGADAS — desligadas, a animação dele chega byte a byte):

* `Motor.prenderOsPesDele(rig, poses, opts)` — a nossa lei de apoio corrida por cima da saída dele.
  ⚠️ **As chaves NÃO se passam**: sobre a animação dele não existe «o quadro em que manda a pose do
  artista». Passá-las corta a deriva só para metade e faz o tranco **crescer**.
* `Motor.tirarAMolaDasMaos(rig, poses, antes, quanto)` — mistura entre a animação autorada e a dele,
  **só nos ossos `mao_*`**, com cursor de dose. A mão é uma folha ⇒ nada mais no boneco se mexe
  (`0,0000 cm`, com portão).

**Os números que decidem** (deriva/tranco piores, em cm, régua de 5 mm, exemplos do menu):
`16,2 / 6,5` como ele fez → `0,3 / 0,3` com o ajuste. Punho do `mortal`: `1188°` → `0°` (a nossa
física põe `238°`; a animação autorada põe **`0°`**, porque o artista nunca anima o punho — é o único
osso onde 100% do que existe é física).

⛔ **A lição que custou a wave está em [[a-laws-two-fences-must-be-measured-in-the-same-ruler]]**: a
1.ª entrega tinha a cerca do contacto emprestada do motor (3 cm) e a barra da velocidade medida nessa
cerca (1,39 cm/quadro) — internamente consistente e **sem curar nada** (a aterrissagem ficava nos
16,2 cm). A célula certa é `0,5 cm · 2,20 cm/quadro`, e a grade está em
`node sonda_ajustes_do_cascadeur.js --grade`.

**Portões:** 6 em `teste.js` (196 verificações no total) sobre fixtura construída — ⚠️ eles **não
lêem** o `cascadeur_resultados.js`, que é gerado, senão ficariam verdes a medir nada quando ele
faltasse; 7 em `gesto_ajustes_dele.js`, que corre o app a sério com a animação dele e com o rato a
carregar nas caixas, mais 4 provas de mutação; 7 casos novos no `prova_mutacao.js`.

⚠️ **E a cerca da velocidade não pôde ser gateada no teste de gesto**, o que é outra medição: o único
caso do corpus em que ela morde é a captura `amostra_16_07`, e ela **não é alcançável no navegador** —
as amostras carregam DEPOIS do ficheiro que monta `Exemplos.LISTA`. ⇒ ela é gateada na fixtura
construída da `teste.js`, e o gesto gateia o que o dono vê (`6 exemplos do menu · 4 melhoram · 0
pioram`).

### 2026-09-19 (2.ª volta) — «os braços têm mola além das mãos»

Report do dono depois do 1.º smoke. Ele tem razão, e **o número já estava na minha tabela**: eu li a
coluna «a animação autorada vale ZERO» (que só a MÃO tem) e não li a do que a física dele
ACRESCENTA — `491°` no ombro e `729°` no cotovelo, contra `57°` e `180°` da nossa.

⭐ O diagnóstico é a coluna `fim`: no ombro e no cotovelo o que ele acrescenta **acaba em zero**, que
é a assinatura de mola; e o período **encurta** do ombro para a mão (8 · 6 · 4 quadros), que é o que
uma cadeia de molas faz.

⇒ `Motor.tirarAMolaDosBracos(rig, poses, antes, quanto, { modo })`, com **dois modos** — na mão
«tirar a mola» e «pôr o osso como a animação o pôs» são a mesma coisa, no braço **não são**, e a
diferença é o braço a ficar para trás do ombro. ⭐⭐ **A janela do filtro é o PERÍODO MEDIDO e não um
número afinado**: a média de um período inteiro de uma oscilação é zero. É o período do modo mais
LENTO da cadeia (o do ombro), senão a mão mal é suavizada.

⛔ **Duas mutações sobreviventes, e as duas são lições de fixtura** (a 1.ª repetiu uma que eu tinha
registado duas horas antes): *um corpus no ponto neutro de um osso não testa esse osso* (a mutação
que alarga a lei à COLUNA ficava verde porque a fixtura não mexia na coluna), e *para gatear uma
propriedade de FASE é preciso uma fixtura que tenha fase* (a mutação que torna a média CAUSAL ainda
mata a oscilação — o que ela estraga é a HORA, e sobre uma rampa isso é invisível; a fixtura ganhou
uma CORCOVA).

⚠️ E uma régua minha estava certa na fixtura e errada nos dados reais: «o braço continua a
acompanhar» medido no ÚLTIMO quadro lê `1,6°` na animação dele, porque ali o delta acaba em zero. Nos
dados reais a régua é a **EXCURSÃO**. *As duas réguas são diferentes porque as duas fixturas são
diferentes.*

### 2026-09-19 (3.ª volta) — copiar e colar a posição de um ponto (botão direito)

Pedido do dono. ⛔⛔ **«A mesma posição» tem DUAS leituras e a MEDIÇÃO escolheu**: entre dois quadros
dos exemplos dele a bacia anda 34 a 123 cm e o braço estica 49 cm do ombro ⇒ colar a posição do PALCO
fica fora de alcance no pulo, no mortal e na aterrissagem. As duas ficam, no menu.

⭐⭐⭐ **E o que as separa é a RAIZ**: com ela livre o ponto chega ao alvo (0,0–0,2 cm) mas a BACIA anda
13 a 17 cm, e o erro medido no corpo de DEPOIS vai a 36 cm — ⛔ e iterar NÃO cura (36 → 14 → 7 → 6 →
11). Com a raiz PRESA no modo do corpo: 0,00–0,15 cm e o corpo não anda nada.

⛔⛔ **O gesto novo expôs uma dívida de portão**: `PORTAO_CIRCULO` estava declarada e não era usada
por mutação nenhuma. Ao escrevê-la, duas armadilhas — a 1.ª mutação era INERTE (a `raiz` do braço
neste rig é o OMBRO, de comprimento ~0) e a 1.ª cura do portão era uma **TAUTOLOGIA** (medir o raio dá
sempre o comprimento do osso, porque o cotovelo É a ponta dele). A afirmação com conteúdo é a
IDENTIDADE DO PONTO MAIS PERTO: ele NÃO alcança o rato e fica a exactamente o quanto o rato está fora
do círculo. Escrita nos dois portões, e a mutação mata os dois.

### 2026-09-19 (4.ª volta) — o DINOSSAURO: um segundo esqueleto 2D

Pedido do dono («o T-Rex do Cascadeur»). O bípede embutido dele chama-se **Dracorex**, e vem com uma
caminhada feita por eles. 100 juntas · 30 corpos · 62 pontos · **413 kg** (o número que o próprio
centro de massa declara) → **29 ossos** aqui, com cauda de 4 peças e a perna de pássaro.

⭐⭐⭐ **A HIERARQUIA NÃO SE LÊ — DERIVA-SE, e isso é uma medição com resíduo.** A API dele não expõe
pai nenhum (introspeção: nem `get_parent` no `model_viewer`, nem dado «Parent» numa junta), e a
saída óbvia seria adivinhar a cadeia pelos NOMES. O que há em cada junta é a posição **LOCAL** e a
matriz **GLOBAL** ⇒ *o pai é a única junta cuja matriz leva a local do filho à global dele*. A
convenção da matriz também se mede (por linhas `1e-5 cm`, por colunas `100 cm` — não é um empate), e
o **CONTROLO** é a Cascy: a derivação reproduz **12 de 12** ligações que o gerador tem escritas à
mão, pior resíduo `2,31e-5 cm`. *É o controlo que separa uma medição de um palpite com cara dela.*
⛔ A mesma porta NÃO serve para os pontos de controle (36 de 43 não têm posição local; dos 7 que têm,
a derivação dá disparate) — tentado, medido, revertido com a razão escrita no extractor.

⛔⛔ **A régua do «cima» era HUMANOIDE e falhou em silêncio.** O gerador da Cascy escolhe o eixo
vertical perguntando *onde a cabeça está acima da anca*; neste bicho a cabeça está **1,4 m à FRENTE**
e 13 cm acima ⇒ o rig saía **DEITADO** e nada no ficheiro o dizia. A régua que fica é **a anca acima
do PÉ**, com portão (focinho à frente, anca acima do pé, cauda atrás).

⭐⭐ **A perna de pássaro decide-se por medição, não por anatomia.** Os quatro segmentos trabalham
(fémur 82°, tíbia 76°, tarso 67°, dedos 108°) e a IK do motor dobra DOIS; o `fim` do membro tem de
ser o ponto que **toca o chão**, e o tornozelo dele anda a **29 cm** ⇒ o membro é
`joelho → tornozelo → planta`. ⛔ A alternativa (fémur+tíbia, ponta no tornozelo) foi recusada com
número: ali `rig.sola` do tornozelo daria 29 cm e a lei de apoio leria *«o tornozelo toca o chão»*
sempre que o bicho voltasse à altura de estar de pé. As duas recusas têm mutação que as mata.

⭐ **Os LIMITES saem da caminhada dele; o que é LEI é a CONTINÊNCIA.** Os 15 limites foram medidos nos
41 quadros; o número escrito é decisão nossa (**alargar** — uma caminhada é UM andamento), e o portão
afirma que o limite CONTÉM o medido (folga mais apertada `6,9°`).

⛔⛔ **MEDIDO e não curado:** a caminhada dele abana a cauda **2,25 m para o lado** (138 cm de média
fora do plano). O importador declara por GRUPO — pernas `11,5` entram, braços `22,3`, tronco `14,0` e
cauda `138,2` não —, e a reconstrução fica a `43,4 cm` no pior osso e `4,0 cm` em média.
