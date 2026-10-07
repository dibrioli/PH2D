---
name: a-ruler-objective-can-contradict-the-product
description: O objectivo de uma régua (fechar toda amostra de vão) colou o que o dono quer separado; e um refino por limiar faz o desenho SALTAR com a pose — continuidade mede-se entre poses VIZINHAS, na entrada do produto.
metadata:
  type: feedback
---

⛔⛔ **Uma régua tem um OBJECTIVO, e ele pode contradizer o produto.** A régua do «fio» da imagem presa
queria fechar toda amostra de vão entre membros; cinco leis (F48, F49, A5-a ×3) foram afinadas contra ela
e o dono disse em 07/10: *«sempre grudou, nunca foi corrigido; mesmo se sobrepondo não puxe nada da imagem
cuja influência é do outro osso»*. A ponta natural de uma cunha NÃO é defeito; a cura foi REMOVER a
costura (`3f32ffaa3`), com o gate `nada_de_um_membro_e_puxado_para_o_outro` (vermelho em 14/18 poses).

⛔ **E o meu refino `k = 2` do A13 saltava:** um limiar binário liga/desliga pontos de corte com a pose
(`0,041`, `124×` a mudança real, a `160,1 → 160,2`) e nenhum gate comparava poses VIZINHAS. A «ponta»
da tampa era a lei da pele (média circular de ângulos), não a amostragem.

**Why:** uma régua verde prova que o número desceu, não que o dono quer aquilo; e um gate de continuidade
que mede uma RÉPLICA do orçamento (e não o desenho do produto) fica verde com o produto a saltar (7 de 36
mutantes sobreviveram assim).

**How to apply:** (1) antes de afinar uma lei contra uma régua, pergunte se fechar/zerar a grandeza é o
que o dono pede — leia o report dele contra o OBJECTIVO da régua; (2) todo limiar que muda o desenho
precisa de um gate de continuidade entre poses vizinhas, a conduzir a entrada do PRODUTO (`recook_desenhando`),
com controlo; (3) um «dente» que só a densidade expõe é fragilidade do passo de união — varra a densidade
(`×1/×2/×4`), não acredite no HEAD que passa «por sorte». Fila §F65; BUGS #36/#37.
