---
name: feedback-the-approved-reference-may-already-be-in-the-product
description: "Antes de inventar um limiar, pergunte se outra rota do mesmo sistema já responde a pergunta — e se o usuário já disse qual delas está certa; um gate que a referência aprovada REPROVA descreve o seu modelo, não o requisito"
metadata:
  type: feedback
---

Flip, BUGS #22 (2026-07-18). O balde extravasava a linha havia **cinco smokes**. O Enio tinha
nomeado a resposta certa dois smokes antes, e eu li como descrição de sintoma o que era uma
**especificação**:

> *"Diferente do **Draw:Filled** que faz exatamente como eu estou dizendo."*

O Draw:Filled é outra rota do MESMO balde: ele põe a cor até o eixo da linha e não dilata nada.
A rota do contorno dilatava pela espessura da linha. **Duas rotas do mesmo sistema respondiam
diferente à mesma pergunta, e o usuário já tinha dito qual estava certa.** Um oráculo que
renderiza as duas e conta os pixels discordantes decidiu em uma execução o que quatro rodadas de
calibração não decidiram (12.223 pixels contra 11).

**Why:** um limiar que eu escolho carrega a minha teoria do defeito. Uma referência que o produto
já contém carrega o julgamento do usuário. Quando existe uma segunda rota — um caminho legado, um
modo irmão, um fallback — ela é um oráculo *de graça*, e é imune ao meu erro de modelo. Foi
exatamente o erro de modelo que sustentou este bug: eu media a *dose do remédio* em vez do
defeito visível.

**How to apply:**

1. Ao caçar um defeito visual, **enumere as outras rotas do próprio sistema** que produzem algo
   comparável. Se o usuário elogiou alguma, ela é a especificação — compare por pixel, não por
   regra.
2. **Teste de lei-ou-opinião, e custa uma sonda:** *a referência aprovada passa neste gate?* Aqui
   a resposta foi não — o gate `a_soft_line_never_shows_the_background_through_the_fill_edge`
   exigia do balde algo que o Draw:Filled não faz (medido: a referência deixa 2956 pixels de
   fundo sob a linha macia; a lei nova deixa os MESMOS 2956). Um gate que a referência reprova
   está descrevendo o modelo de quem o escreveu, e vai **sustentar** o bug em vez de pegá-lo.
3. Corolário de fixture: a suíte tinha **onze** oráculos de pixel verdes e todas as onze fixtures
   usavam UM traço fechado — a topologia em que o produto vai pela rota que não dilata. A função
   defeituosa nunca era chamada. Antes de confiar numa suíte, **verifique que a fixture exercita
   a ROTA suspeita**, não só o assunto dela ([[reference_topic_fixture_discipline]]).

Irmão de [[feedback_a_new_remedy_makes_the_old_one_double_counting]] (o termo derrubado era a 4ª
instância seguida da mesma contagem dupla) e de
[[reference_topic_oracle_discipline]] (*o oráculo modela a aparência, nunca a regra*).
