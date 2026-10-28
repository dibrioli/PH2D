# REGRAS DE CUSTO (ordem do dono, J4: «muito gasto e pouco resultado») — valem a partir de AGORA
1. No máximo DUAS rondas de cura por pincel. Revisor cego achou o oráculo na 2.ª? O pincel fica reprovado/curado-não-aprovado, com a diferença escrita. Nada de 3.ª, 4.ª, 6.ª ronda.
2. Revisor cego: lance-o com `model: "sonnet"` (mais barato) e peça uma resposta curta (≤ 15 linhas). UM revisor por ronda.
3. Motor: só o PRIMEIRO item de motor da sua lista. O resto vai para o retorno como fila, com a medição que já tem — não o comece.
4. Não leia ficheiros grandes inteiros: use grep/offset. Não reabra pranchas antigas.
5. Pesados: um de cada vez; nada de grelhas de 20 variantes — no máximo 4 variantes por cura, escolhidas pela medição.
6. Acabe, comite e devolva o retorno curto assim que o item em curso fechar.

# METODOLOGIA (ordem do dono, 28/10, J6: «precisamos de metodologia correta») — vale ANTES das regras acima
Medido na J6: ~20 agentes, > 25 h, 2 aprovados. Causa: construir motor sem saber se vira aprovado; diagnósticos errados
descobertos tarde; pesado repetido (regerar 8 Oil/* = 2h45 a 1 085 % de CPU). Daqui em diante:
1. TRIAGEM PELO DONO, primeiro e barata: folha «nosso | oráculo» com fotos que já existem; ele marca serve / quase /
   não serve. «Serve» entra na fábrica; só «quase» recebe trabalho; «não serve» sai com o porquê.
2. SIMULAR O CONSERTO ANTES DE CONSTRUIR: retoque da imagem que imita a peça de motor (~15 min); construir só se o
   retoque passar (cega ou dono). Diagnóstico herdado de outra janela confere-se na foto antes de se acreditar nele.
3. PRAZO: 45 min por agente. Estourou ⇒ pára, devolve onde chegou e o que falta; o coordenador decide.
4. PESADO 1× NO FIM: regerar presets, tabela de largura e suíte só no fecho; a largura mede-se com o Real Oil
   DESLIGADO (luz não muda onde a tinta está); a luz do óleo calcula-se só na foto final.
5. RENDIMENTO: aprovados por hora a cada rodada; abaixo de 1/h ⇒ parar e reavaliar com o dono.
6. Janela nova por onda; coordenador pequeno (aviso dos 300 mil = handoff e janela nova).
