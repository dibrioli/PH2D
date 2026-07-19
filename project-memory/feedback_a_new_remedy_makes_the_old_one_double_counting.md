---
name: feedback-a-new-remedy-makes-the-old-one-double-counting
description: "Ao acrescentar uma defesa, pergunte o que ela torna DESNECESSÁRIO — o mecanismo velho não fica errado, fica obsoleto, e obsoleto não dispara gate nenhum"
metadata:
  type: feedback
---

Flip, BUGS #21 (2026-07-18). O balde transbordava a linha *"um pouquinho"*, e o Enio disse a
frase que resolveu: **"Porque não ter como referência o centro da linha? Já tínhamos resolvido
isso."** Tínhamos — era o BUGS #14, e ele continuava certo.

A dilatação do fill tinha dois termos: a espessura da linha (leva a cor do eixo até a silhueta —
correto) e uma **margem extra**, que existia para cobrir o erro de vetorização do contorno.
Legítima no dia em que nasceu: não havia outra defesa. Depois chegou a **compensação por ponto**,
que cobre o MESMO erro — melhor, porque é por ponto e tem sinal. A partir dali a margem virou
**contagem dupla**, e a segunda parcela era paga em **pixels que o artista vê**.

**Why:** o mecanismo velho não ficou *errado* — ficou **obsoleto**. Errado quebra gate; obsoleto
não quebra nada. Ele continua passando em todos os testes, continua tendo um doc-comment
convincente e uma tabela de medição ao lado, e só se manifesta como um resíduo que ninguém sabe
atribuir. Foi por isso que sobreviveu a quatro rodadas de calibração.

**How to apply:** ao acrescentar uma defesa, pergunte **explicitamente "o que isto torna
desnecessário?"** e resolva no MESMO commit — remova, ou escreva por que fica. E o sintoma que
denuncia o caso quando já é tarde: **calibrar a mesma constante por várias rodadas**. Cada rodada
aqui teve medição séria e tabela honesta (0,5 → 0,25 → fração 0,06 → fração 0,03); nenhuma
perguntou *se o termo devia existir*. **Ao terceiro ajuste da mesma constante, pare e questione o
modelo** — é [[feedback_ergonomics_verdict_is_a_design_bug]] na sua forma interna, aplicada a um
número em vez de a um slider.

Corolário: **um invariante já conquistado tem de ser RE-CONFERIDO por quem acrescenta um termo.**
O #14 deixou a frase clara (*a referência é o eixo da linha*) e o #15 a violou sem notar, porque
a violação vinha embutida num fix correto. Invariante que vive só na prosa de um bug antigo não
sobrevive ao fix seguinte — escreva-o onde o próximo termo vai ser somado, não onde ele foi
descoberto. Irmão de [[feedback_documented_decision_chesterton_fence]] e de
[[feedback_stale_comment_and_dead_code_lie]].
