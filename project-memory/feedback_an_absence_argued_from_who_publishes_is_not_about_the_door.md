---
name: feedback_an_absence_argued_from_who_publishes_is_not_about_the_door
description: "«Este painel precisa de um mundo/grafo/device» quase sempre descreve QUEM PUBLICA, não o que atravessa a porta — três painéis foram declarados inatingíveis e os três liam um snapshot"
metadata:
  type: feedback
---

Uma varredura de UI declarou **três** painéis impossíveis de encher, cada um com uma frase
plausível sobre a cena que os alimenta:

| painel | a frase escrita | o que a porta de facto recebe |
|---|---|---|
| `motion_params` | *«pinta os params do nó ESCOLHIDO no grafo»* | `set_current_params(ParamsSnapshot)` |
| `model3d` | *«a árvore vem de um `FieldDoc` COZIDO da hierarquia»* | `publish(ModelSnapshot)` |
| `sculpt3d` | *«a cena dele segura uma surface de wgpu»* | `set_current_sculpt3d(Sculpt3dSnapshot)` |

As três frases são **verdadeiras sobre o produtor** e **falsas sobre a porta**: em nenhum dos casos
o painel vê mundo, grafo ou device — ele lê uma struct `thread_local`, que é DADOS. A última é a
mais instrutiva, porque o doc de um campo daquele snapshot (`matcap_keys`) **já escrevia a aresta
de dependência por extenso**: *«o painel não fala com device nenhum»*.

Custo medido: `402` rótulos (`97` + `305`) de dois painéis que o artista usa nunca tinham passado
por régua de largura nenhuma, e a varredura ficava **verde** porque o piso dela era global.

**Why:** «precisa de X» é uma afirmação sobre a **cadeia de produção**, e o que decide se um arnês
consegue armar algo é a **assinatura da porta de entrada**. As duas leem-se igual numa nota, e a
primeira é sempre a que vem à cabeça, porque é a história de como o dado nasce.

**How to apply:** antes de escrever «este componente não é testável sem \<coisa cara\>», faça
`grep '^pub fn ' <crate>/src/*.rs | grep -i 'set_\|publish\|current'` e leia o **TIPO** do
argumento. Se for uma struct de dados, a ausência não existe. É a mesma família de
[[feedback_a_probe_that_arms_a_module_by_env_var_measures_another_program_than_the_pill]] e o caso
canónico de *uma ausência afirmada sem olhar a API é um palpite com cara de medição*
([[reference_topic_measurement_discipline]]).

⭐ E a régua que torna isto visível é o **piso POR SUJEITO**: um piso sobre a SOMA («2 800 rótulos
medidos») fica verde com oito painéis a medir zero — *zero lê-se como aprovação, e um piso sobre a
soma não pergunta por ninguém*.
