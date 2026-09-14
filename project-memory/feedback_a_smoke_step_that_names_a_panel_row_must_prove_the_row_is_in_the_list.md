---
name: feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list
description: "Um passo de smoke que manda clicar numa LINHA de painel é uma afirmação sobre o que está na lista — e uma mudança noutro sítio pode tê-la esvaziado"
metadata:
  type: feedback
---

Um passo de smoke que diz *«clique na linha X»* não é uma instrução: é uma **afirmação de que a
linha X está na lista**. E essa afirmação é feita num ficheiro (a cena) e decidida noutro (o filtro
do painel) — logo envelhece sozinha, sem que nada reprove.

Medido em 2026-09-06 (`line/components`). Uma **receita não é uma linha da cena**: a Hierarquia
retira da lista tudo o que um predicado acusa, e a raiz da receita também o satisfaz — logo a
receita **inteira** sai. Duas cenas mandavam agir sobre linhas dela:

- uma escrita a **27/08**, verdadeira nesse dia e falsa desde **30/08**, quando o filtro nasceu;
- outra escrita **hoje**, por eu ter lido a árvore no mundo em vez da lista no painel.

⚠️ **O dono aprovou o smoke com o passo impossível dentro.** Ele faz o que consegue e diz *«OK»* —
*a aprovação de um smoke não é uma verificação de que cada passo é executável.*

⛔ **E o modo de falha é o pior possível:** o report que volta é *«não achei»*, indistinguível de um
defeito real, e custa uma jornada a bissecar.

⭐ **O instrumento que fecha isto tem duas metades, e uma sozinha não serve:**

1. um gate que corre o **predicado do painel** sobre as entidades que o passo nomeia — com a
   **metade justa primeiro** (sem a condição, as linhas *não* estão lá), senão uma implementação que
   nunca escondesse nada passaria;
2. um **censo de dois sentidos** entre a cena e o texto dela: *quem abre diz, e quem diz abre*.
   Abrir sem dizer põe na tela um objecto que ninguém explicou; dizer sem abrir é o passo impossível
   de volta.

⛔⛔ **E havia um GATE a defender a instrução impossível — verde o tempo todo.** Ele exigia que cada
cena mandasse *«clique na linha da receita»*, porque era esse o gesto no dia em que foi escrito.
Depois de o filtro nascer, ele passou a **obrigar** a manter uma frase que já não descrevia nada:
*um gate sobre o TEXTO de uma instrução mede a presença da frase, nunca se o gesto que ela nomeia
ainda existe*. Foi ele que expôs a terceira cena — ao reprovar quando as outras duas foram curadas.

⛔⛔ **E há uma segunda espécie, do mesmo dia e do mesmo dono:** um passo que demonstra uma
**RECUSA** tem de pedir um gesto que a recusa **apanhe** — e o estado deixado pelo passo anterior
decide isso. O `PASSO 1` movia a peça para a cabeça em todas as cópias; o `PASSO 2` mandava
arrastá-la *para a cabeça*, onde ela já estava. Mesmo pai ⇒ a guarda deixa passar de propósito, e a
mensagem prometida nunca vem. ⇒ *quando um passo promete uma mensagem, gate o gesto que ele pede
contra a lei que a produz — com as duas metades: o gesto novo é apanhado, o antigo não.*

⛔⛔ **E há uma TERCEIRA espécie, medida em 2026-09-08: o passo que nomeia um GESTO.** Um selector
de cartão deixou de **ciclar** no clique e passou a ter **duas setas e uma lista** (report do Enio,
07/09, com a foto do Blender). Os **três** tutoriais em PDF do módulo continuaram a ensinar o gesto
antigo — o do ciclo 1 em três sítios, o do ciclo 2 em quatro passos (um deles a prometer o percurso
inteiro: *«Sine → Tri → Square → Saw → Spike → Custom e volta ao começo»*), e o do ciclo 3, escrito
**no dia seguinte**, herdou-o da minha memória em vez de o ler do código.

⚠️ **A linha estava lá, o rótulo estava certo, e o passo continuava impossível** — é o que separa
esta espécie das duas de cima: nenhuma pergunta sobre a *lista* a apanha. E o ciclo 2 estava com o
smoke do dono **pendente**: ele bateria nisto no primeiro selector que tocasse.

⇒ **a medição de um gesto é ler quem o trata** (aqui, `interact_param_row.rs`: `RowZone::Prev` /
`Next` / `Centre`), nunca a memória de quem escreve o passo. E quando um gesto MUDA, os tutoriais
são o último sítio lembrado e o primeiro que o dono lê — `grep` os textos de smoke pelo verbo que
mudou, no mesmo commit que o muda.

⛔⛔ **E uma QUARTA espécie, medida em 2026-09-13 sobre um smoke MEU: o botão cuja SECÇÃO inteira
deixou de ser alcançada.** Eu escrevi *«carrega em **Add Physics Body**»* e **verifiquei** — o
rótulo está na tabela de textos (`panel.inspector.physics.add_physics_body`), o pintor da face vazia
da §11 desenha-o, o `HitIndex` regista-o. Três leituras, todas verdadeiras, e o passo era
**impossível**: desde o ADR-0166/F3 o `build_physics_info` devolve `None` quando o objecto não tem
corpo nem collider, logo num sprite pelado **a secção inteira não é pintada** e nada daquilo corre.

⚠️ **O que engana é que as três leituras estão no lado certo da costura.** A régua que decide é o
**construtor do snapshot**, que vive noutra crate e cujo `return None` não menciona botão nenhum. ⇒
*a pergunta não é «quem pinta este botão?», é «quem decide se esta SECÇÃO chega a ser pintada, e para
que objecto?»* — uma cláusula acima do que as três espécies anteriores olhavam.

⚠️⚠️ **E no mesmo smoke eu acertei nos outros três nomes** (*Add Component*, *Platform Player*,
*Timers* — este corrigido de *Timer* pela leitura do catálogo), o que é o pior resultado possível:
**verificar quase tudo produz a confiança de ter verificado tudo**. O report do dono veio no dia
seguinte, e a frase dele foi *«não temos mais Add Physics Body no painel»*.

**Why:** o texto de um smoke é **superfície de produto** — é onde o dono aprende a ferramenta
(CLAUDE.md §0.8) —, e é a única superfície do repo cuja correcção nada media.

**How to apply:** ao escrever um passo que nomeia uma linha de painel, pergunte **quem decide se
aquela linha é pintada**, e gate essa decisão com as entidades da cena. E quando alterar um filtro de
lista, `grep` as cenas de smoke pelo nome do que ele passa a esconder. Ver
[[feedback_a_smoke_step_that_needs_three_decisions_at_once_is_not_a_step]] e
[[feedback_a_reserved_band_is_not_a_painted_band_and_geometry_gates_go_green_over_a_blank_screen]].

⛔ **E outra vez, noutra forma (16/09, modelador):** mandei o dono desenhar no Vector com o app
aberto por `PH2D_FIELD_SMOKE=5` — passo tirado dos nomes no código, sem o conduzir (a placa estava
com o smoke de outra linha). O passo era impossível: a env do smoke armava o módulo sem olhar o
painel e ele comia o clique do Vector (`BUGS_3dmodeling.md` #3). *Um passo não conduzido é uma
hipótese sobre o produto* — e o COMANDO do smoke faz parte do passo: uma env de smoke dirigido pode
mudar o que as outras ferramentas conseguem fazer.
