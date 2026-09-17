---
name: feature_worse_than_not_existing
description: Uma feature pode ser PIOR que não existir, e só uma linha de CONTROLE «não fazer nada» mostra isso — sem ela um erro pequeno lê-se como sucesso
metadata:
  type: feedback
---

Toda tabela que compara o nosso resultado com um alvo precisa da linha **«e se não fizéssemos nada?»**. Sem ela, um erro pequeno lê-se como sucesso mesmo quando **desligar a feature seria melhor**.

Medido no testbed do Cascadeur (14/09), sobre a física que mexia nos membros no ar. Erro médio contra a série do oráculo, quadro a quadro, em graus:

| | pernas | braços | coluna |
|---|---|---|---|
| a física NÃO toca nos membros (controle) | 3,49 | 6,16 | 0,74 |
| a lei que shipava (f 4, z 0,35, todos, só no ar) | **6,75** | **9,64** | **2,34** |
| a lei medida (só braços, sempre, f 2, z 0,9, int 0,25) | 3,49 | 5,17 | 0,74 |

A feature era **pior que a sua própria ausência nos três grupos** — e tinha 10 portões automáticos verdes, todos escritos para defender o desenho dela em vez de a comparar com nada.

**Why:** os parâmetros dela tinham sido escolhidos contra **RESUMOS** do oráculo («a dobra máxima do joelho dele é 44°»). Um máximo não distingue uma mola que toca três ciclos de uma que dá um pico e morre: as duas têm o mesmo máximo. O dono viu em dois segundos o que dez portões não viam — *«as mãos parecem mola de braço mecânico»*.

**How to apply:** ao afinar qualquer lei contra um alvo externo, (1) o alvo é a **série** do alvo, quadro a quadro, nunca um resumo dela; (2) a primeira linha da tabela é sempre **o controle que não faz nada**; (3) o portão que fica no repo compara a feature com esse controle, e reprova se ela não for melhor — não com uma barra escolhida. Vale para toda "melhoria" com um oráculo do outro lado: um ganho que não bate o zero não é um ganho.

⭐⭐ **E procure a SEGUNDA DOSE.** A auditoria seguinte, no mesmo app, achou a mesma lei aplicada **duas vezes** por dois interruptores diferentes (a mola dentro da física e a mola do painel). Uma grade de 80 combinações da segunda dose sobre a primeira: **toda célula pior que a linha de base**. O oráculo tinha a resposta escrita na cara — lá aquilo não é ferramenta separada, é parte da física. *Quando a mesma lei tem dois donos, um deles está a estragar o trabalho do outro.* Ver [[reference_topic_measurement_discipline]] e [[project_teste_cascadeur_2d_bones_testbed]].
