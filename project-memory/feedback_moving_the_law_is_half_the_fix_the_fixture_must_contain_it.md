---
name: feedback-moving-the-law-is-half-the-fix-the-fixture-must-contain-it
description: "Fazer o oráculo PERGUNTAR a lei do produto não basta — se a fixture vive no ponto onde o erro é invisível, ele segue cego"
metadata:
  type: feedback
---

Flip, 2026-07-18. O `gpu_fill_fit` montava a **própria** dilatação (cópia da constante +
cópia da fórmula), então quando a do produto ficou 100× grande demais os **oito** oráculos
de pixel ficaram verdes — inclusive o chamado *"a cor nunca transborda para fora da linha"*,
enquanto o produto transbordava.

O conserto óbvio é mover a lei para onde os dois lados a alcançam e fazer o oráculo
**perguntá-la**. Fiz isso — e mutar o bug de volta **ainda deixava os oito verdes**.

Motivo: toda fixture do arquivo descrevia a arte num mundo que **já era pixel**
(`px_per_world = 1`), que é o **único ponto da reta onde `2·0,5/1` e `2·0,5` são o mesmo
número**. O erro de UNIDADE era invisível por construção do fixture, não por descuido do
gate. Só a fixture nova na escala do produto (raio 1,1 a 100 px/unidade contra raio 110 a 1
px/unidade, exigindo a MESMA imagem) fez a mutação sangrar: 0,02% dos pixels diferem no
correto contra **43,4%** no bug.

**Why:** "o oráculo agora consome o número do produto" *soa* como o fim do trabalho e produz
a sensação de segurança sem a segurança — o gate passa a perguntar o número certo e continua
cego para o erro que motivou tudo. Duas perguntas diferentes: *de onde vem o número?* e *o
regime testado é capaz de distingui-lo do errado?*

**How to apply:** depois de ligar o oráculo à lei, **mute o defeito original de volta** e
exija que ele sangre **no oráculo**, não só no unit test. Se não sangrar, o problema é o
regime: procure o ponto onde a grandeza suspeita é *degenerada* (fator 1, ângulo 0, lista de
um elemento) — é quase sempre onde a fixture está. Para erro de unidade, o teste mais forte é
**a mesma coisa descrita em duas escalas** (a arte não sabe em que unidade foi escrita), com
o limite tirado do FOSSO medido entre certo e errado, nunca colado na observação.

Corolário da mesma rodada: **um instrumento novo prova o seu valor derrubando a hipótese de
quem o construiu.** A minha (transbordo cresce com o zoom) foi refutada — e as duas primeiras
tabelas a teriam *confirmado* por artefato meu (ler fora do alvo contando `(0,0)` como fundo;
e arte trêmula, cujo tremor cresce em tela junto com o eixo varrido). *Antes de ler uma
tendência, pergunte que outra coisa varia junto com o eixo da varredura.*

**E o instrumento honesto desfez um veredito meu, na rodada seguinte.** Eu tinha
implementado a compensação por ponto, **medido, julgado pior e revertido sem shipar** — o
critério era a *mediana da compensação* (0,0178 contra 0,005), ou seja **o tamanho do próprio
remédio**, e não o defeito visível. Uma compensação maior não é um resultado pior: ela é maior
porque o erro que ela cobre é maior. Medida no pixel, a mesma ideia melhora a cobertura sem
mexer no transbordo. **Um número que SOBE quando o remédio age não pode ser o critério de o
remédio estar funcionando** — meça o sintoma, nunca a dose.

Irmãos: [[feedback_two_doors_to_the_same_question_diverge]] ·
[[reference_topic_fixture_discipline]] · [[reference_topic_mutation_proofs]] ·
[[feedback_a_green_gate_may_be_green_by_accident]].
