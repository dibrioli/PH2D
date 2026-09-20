---
name: feedback_an_order_with_two_halves_can_be_obeyed_only_in_the_half_that_removes
description: Uma ordem que MOVE tem duas metades — tirar e pôr — e elas caem em sítios diferentes do código; cumprir só a que RETIRA apaga a capacidade em silêncio, e a lei órfã passa em todos os gates
metadata:
  type: feedback
---

Quando o dono manda **mover** alguma coisa — *«deixa o colapsar apenas no menu»*, *«encurta esses
nomes»*, *«tira isto daqui»* — a ordem tem **duas metades: TIRAR e PÔR**, e elas caem em sítios
diferentes do código. A que remove é fácil, local e visível no diff; a que põe é noutro ficheiro,
às vezes noutra crate.

⛔⛔ **Cumprir só a que remove passa em TODOS os gates.** A capacidade desaparece, a lei que a
implementava fica **viva e órfã** (com os testes dela verdes, porque ela continua correcta), e
nenhuma sonda deste repo pergunta se uma PORTA ainda tem chamador.

**Dois casos medidos, ambos na `line/UIUX`:**

| ordem | a metade que se fez | a que faltou | custo |
|---|---|---|---|
| *«deixa o colapsar apenas no menu da barra superior»* (09/09) | o gesto de arrastar a borda saiu | o item de menu nunca foi escrito | **dez dias sem maneira nenhuma de fechar uma coluna**; o `dock_columns::close` ficou com **zero** chamadores de produto e a involução dele verde |
| *«encurtar esses nomes»* (19/09) | o rótulo encurtou e passou a caber | a explicação que o nome carregava | a frase que dizia o que a caixa FAZ deixou de existir, **em silêncio**, com todo gate de largura verde |

⭐ **A segunda foi apanhada porque alguém escreveu a metade que PÕE como gate**
(`as_caixas_que_encurtaram_guardam_a_explicacao.rs`, com controlo positivo: uma caixa irmã que
legitimamente não tem balão, senão um `tooltip_for` que devolvesse sempre `Some` ficava verde).

**Why:** um diff mostra o que saiu e não tem forma de mostrar o que devia ter entrado. E a lei
órfã é pior que código morto: ela está **correcta**, os testes dela passam, e por isso ninguém a
lê como dívida.

**How to apply:** ao receber uma ordem que move, **escreva as duas metades na mesma frase antes de
tocar no código** — *«sai daqui, entra ali»* — e **gate a que PÕE**, que é a que não aparece no
diff. Ao retirar um gesto, pergunte imediatamente *«quem chama agora a lei que ele accionava?»*: se
a resposta for «ninguém», ou a outra metade falta, ou a lei tem de sair com ele.

⛔⛔ **E uma lei sem chamador não é só inalcançável — ela deixa de ser MEDIDA.** O doc do
`dock_columns::close` dizia que fechar tem de passar a *ESCOLHA* de largura (`None` quando ninguém
arrastou) e nunca o número (que devolve o default); **a mutação que trocava os dois sobreviveu à
suíte inteira**, porque nenhum caminho de produto a exercitava. ⇒ ao ligar uma lei órfã, corra a
prova de mutação sobre o doc-comment dela: as leis que nunca foram exercitadas por um caminho de
produto são exactamente as que ninguém gateou.

Vizinhos: [[feedback_a_doc_that_states_the_law_the_code_does_not_implement_reads_as_audited]] ·
[[feedback_a_gate_calibrated_at_the_default_width_is_blind_to_the_width_the_artist_has]] ·
[[reference_topic_control_design_hazards]]
