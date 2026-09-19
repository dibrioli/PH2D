---
name: feedback-every-ruler-measured-the-goal-and-none-measured-the-cost
description: "Todos os portões mediam o OBJECTIVO da lei e nenhum media o PREÇO dela — e a cerca que cura o preço pode matar a feature"
metadata:
  node_type: memory
  type: feedback
---

⛔⛔⛔ **Uma família inteira de portões pode medir o OBJECTIVO de uma lei e nenhum medir o PREÇO
dela.** (Teste Cascadeur, 19/09, a opção «deslocar o peso antes de levantar o pé».)

A lei tinha de pôr o peso sobre o apoio durante o passo, e conseguiu: **44,7 cm fora → 0,1**. Cinco
portões verdes, prova de mutação a sangrar. E ela mandava o corpo andar **50,3 cm NUM QUADRO** (onde
ele normalmente anda 1,2) — um teletransporte à vista — porque *todas as réguas mediam o EQUILÍBRIO
e nenhuma media o MOVIMENTO*.

**Why:** as réguas de uma wave nascem da frase da lei, e a frase da lei fala do que ela quer, nunca
do que ela gasta. A régua que apanhou isto foi a mais simples possível: **o resultado contra ele
próprio, com e sem a opção**, na grandeza que a lei NÃO nomeia.

⛔⛔ **E a cerca que cura o preço pode matar a feature.** A cerca certa existia e era física pura (o
pêndulo invertido: `ẍ = g·d/h`, sem uma constante escolhida). Posta a LIMITAR a lei, a opção passou
a mover **0,2 cm** em vez dos 55 que equilibram — *uma caixa que não faz nada, que é o pior dos
controlos* ([[reference_topic_control_design_hazards]]). ⇒ a lei ficou a fazer o que o rótulo diz, e
a cerca virou **INSTRUMENTO**: ela é impressa como linha MEDIDA em toda corrida, ao lado do preço.

⭐⭐⭐ **E quem decidiu foi a FOTO, não o número.** Com a cerca, a opção move `0,2 cm` — *uma caixa
que não faz nada*. Sem ela, o equilíbrio fica perfeito e o boneco fica **55 cm atrás da referência,
sentado atrás dos próprios pés** — *uma feature pior do que não existir*
([[feature_worse_than_not_existing]]). Nenhuma das duas era shipável sozinha, e nenhum NÚMERO o
dizia: os dois estavam bons cada um na sua coluna. ⇒ a opção ganhou um **selector com as duas leis**
e a decisão foi para quem vê.

⛔⛔⛔ **E o epílogo, que é a lição maior: eu TINHA o número e shipei o defeito na mesma.** A régua
do preço foi escrita, entrou na suíte como linha MEDIDA — *«o corpo anda 54,5 cm num quadro»* — e eu
tratei-a como o **preço** da opção. O dono viu-a como o que era: *«um salto anómalo nos keys 49 e
50»*. *Documentar um defeito numa linha medida não o transforma num preço aceitável*, e a distinção
que eu não fiz é entre **andar** 54 cm e **saltar** 54 cm entre dois quadros.

⛔⛔ **E as duas linhas que eu tinha apagado por «inertes» eram exactamente a cura.** Eu media a SOMA
e o PIOR do desequilíbrio — iguais ao cêntimo com e sem elas — e o que elas guardavam era a **FORMA**
(um pulso quadrado: `0 · −54 · −53 · 0`). *Uma soma e um extremo não veem um degrau no meio; a régua
que vê é o perfil quadro a quadro.*

⛔⛔ **E a terceira forma, no mesmo dia: uma régua pode medir com todo o rigor NO SÍTIO ONDE O
DEFEITO NÃO PODE APARECER.** O portão da ordem de desenho escolhia a articulação onde os dois
bonecos mais se AFASTAM — e ali o de cima nem está, logo a ordem é irrelevante: a mutação que a
inverte sobreviveu com o portão a ler «28 contra 0». O sítio certo era onde eles se **CRUZAM**.
*Antes de escrever a barra, pergunte em que ponto do domínio o defeito é sequer observável.*

**How to apply:**
1. Ao fechar uma wave, escreva UMA régua que meça a grandeza que a lei **não** nomeia, comparando o
   resultado consigo próprio com e sem ela. É barata e é onde o defeito mudo vive.
2. Antes de transformar uma cerca em limite, meça o que a feature passa a fazer COM ela. Se a
   resposta for «quase nada», a cerca é um **instrumento** (que explica o preço) e não um limite.
3. E o número que a cerca produz é muitas vezes o achado: aqui ele disse que o desequilíbrio não vem
   da falta da lei — vem de o motor levantar **o pé em que o peso está**, que é outra decisão.
4. ⛔ **Uma linha MEDIDA documenta um preço, nunca autoriza um defeito.** Antes de escrever um
   número como preço, pergunte se ele descreve um MOVIMENTO ou uma DESCONTINUIDADE — e se for a
   segunda, é defeito, por maior que seja a nota ao lado.
5. ⛔ **Antes de apagar uma linha por «inerte», meça a FORMA e não só os totais.** Uma soma e um
   extremo são cegos a um degrau no meio do intervalo.
6. ⭐ **Fotografe a cena antes de decidir entre duas leis.** Aqui as duas colunas de números eram
   boas — a da cerca no movimento, a sem cerca no equilíbrio — e só a imagem mostrou que uma delas
   não fazia nada e a outra partia a pose. Quando as duas se defendem por medição, o formato certo
   é **oferecer as duas** com o preço de cada uma escrito, não escolher em silêncio.

⛔⛔ **E a mesma forma outra vez, no portão da INÉRCIA de uma opção:** ele comparava «não passar
opções» com «passar `false`» e, sob a mutação que fazia a lei correr SEMPRE, as duas concordavam —
ele ficava verde sobre uma opção que deixara de ser opção. *Um portão de inércia sem um CONTROLO DO
LADO DOENTE afirma que as duas maneiras de desligar coincidem, nunca que desligar não faz nada.*
⇒ ele passou a exigir, na mesma corrida, que o defeito que a opção cura **continue lá** com ela
desligada.
