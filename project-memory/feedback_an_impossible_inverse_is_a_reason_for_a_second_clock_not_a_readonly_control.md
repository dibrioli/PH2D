---
name: feedback_an_impossible_inverse_is_a_reason_for_a_second_clock_not_a_readonly_control
description: Mapa não-inversível não justifica travar o controle; justifica dar-lhe um relógio/estado PRÓPRIO que não precisa inverter nada
metadata:
  type: feedback
---

Sob a pilha de clips o mapa timeline→clip não é inversível (um strip em loop manda muitos
instantes num só). Concluí que a régua do modo Keys tinha de ser **read-only** — "não há para
onde arrastar". Errado: travei o fluxo mais básico (mover o playhead para autorar keys). A
correção foi dar ao modo Keys um **relógio de clip INDEPENDENTE** (o precomp do AE), que não
inverte nada — ele scrubba em tempo de clip direto, e a cena mostra o clip ativo sozinho.

**Why:** eu estava **certo sobre o mapa e errado sobre a solução**. "O inverso não existe" é uma
verdade sobre UMA direção de UM mapa; ela não obriga o controle a ser passivo. Quase sempre a
saída é um **segundo estado** (relógio, seleção, buffer, cursor) que responde à pergunta
diretamente, em vez de derivá-la por um inverso que não existe. Travar o controle transfere o
custo do meu problema (não sei inverter) para o usuário (não posso trabalhar).

**How to apply:** ao pegar-se tornando algo read-only/dimmed "porque não dá pra derivar de volta",
pare e pergunte: **e se essa vista tivesse o próprio estado?** O AE não inverte comp→precomp — a
precomp tem o próprio relógio. Sinais de que é a hora: o usuário precisa AUTORAR ali (não só ver);
o degenerado (sem pilha = um relógio só) já funciona e a trava só aparece no caso avançado. E
quando o estado novo só faz sentido no caso avançado, **mantenha o comum intocado** (`keys_mode =
Keys tab && stacked`): sem pilha, nenhum segundo relógio, nenhuma deriva. Ver
[[feedback_ergonomics_verdict_is_a_design_bug]], [[feedback_the_representation_can_delete_the_special_case]],
[[feedback_one_ruler_measures_one_clock]].
