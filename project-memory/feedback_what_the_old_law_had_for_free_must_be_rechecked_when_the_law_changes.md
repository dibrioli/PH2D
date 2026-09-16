---
name: what-the-old-law-had-for-free-must-be-rechecked-when-the-law-changes
description: Trocar uma lei por outra melhor apaga as propriedades que a antiga tinha DE GRAÇA — um esqueleto a 400 unidades da arte passou a mandar nela porque o suporte deixou de ser finito
metadata:
  type: feedback
---

⛔⛔⛔ **Medido 2026-09-15, `line/Vector`, pesos de pele.** A lei antiga (um *bump* sobre a distância
ao osso) tinha **suporte FINITO**: fora do raio, influência zero. Ninguém escreveu isso como
requisito — era um efeito colateral da fórmula. O padrão-ouro que a substituiu tem suporte **global
por desenho**, e é isso que compra a suavidade.

⇒ Uma regra que era inofensiva (*«todo osso fica com pelo menos um vértice»*, a rede para um osso
curto perdido entre duas linhas da grelha) passou a deixar **um segundo esqueleto a `400` unidades
da arte roubar o vértice mais próximo e mandar nele com peso `1`**. O `Bind` sem semente apanha
todos os ossos da cena, logo é o caminho NORMAL do artista.

⭐ **Quem o apanhou foi um gate que já existia** e que media a propriedade — não o defeito:
`binding_to_the_whole_scene_draws_the_same_as_binding_to_the_right_skeleton`. Ele nasceu para provar
que o suporte é finito, e sobreviveu à troca da lei para acusar o que a troca partiu.

**A forma geral:** quando uma lei é substituída por outra melhor, as propriedades que a primeira
garantia **por construção** deixam de estar garantidas — e ninguém as reconfere, porque nunca
estiveram escritas como requisitos. ⇒ *antes de trocar uma lei, faça a lista do que a antiga tinha
de graça* (suporte finito · monotonia · simetria · positividade · custo limitado) e pergunte de cada
uma se a nova a tem. As que não tiver precisam de cerca explícita.

⚠️ **E o corolário do outro lado:** as propriedades que a lei antiga **não** tinha deixam de ser
desculpa. Uma recusa medida sobre a lei velha responde a uma pergunta sobre a lei velha — ver
[[measuring-only-smoothness-approves-a-global-blur-as-a-rig]], onde três famílias de régua se
revelaram ganháveis por respostas degeneradas, e o `CLAUDE.md` §0.0 (*quem move o número que tornava
algo inalcançável tem de reconferir a nota*).

O caso inteiro:
`docs/Skeleton/handoffs/HANDOFF_O_RESTO_DO_APP_ACHAVA_A_ARTE_PLANA_2026-09-15.md` §19.2.
