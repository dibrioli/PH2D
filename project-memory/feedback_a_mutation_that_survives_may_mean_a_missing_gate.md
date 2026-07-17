---
name: feedback-a-mutation-that-survives-may-mean-a-missing-gate
description: "Uma mutação que NÃO sangra tem três causas, não duas — e a terceira é a mais valiosa: o gate certo não existe ainda"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 62ac077f-09f4-41be-9a44-14a0a85668a9
---

A disciplina diz: mute o código; se o gate não cai, **ou o gate é frouxo, ou o seu comentário está
errado** ([[feedback_mutate_the_code_not_just_the_test]]).

Há uma **terceira** causa, e ela é a que rende: **o gate que devia pegar aquela mutação não existe.**
A que ficou verde está *certa* — ela simplesmente não fala sobre aquilo.

**O caso (Sculpt W1, 2026-07-13).** Mutação: fazer o `render_sculpt` ler o relevo VIVO (`target[i]`) em
vez da fonte congelada (`pre[i]`). O gate de idempotência dos shape editors ficou **verde** — e estava
correto: o re-stamp *restaura* o relevo de `pre` antes de re-carimbar, então naquele caminho as duas
leituras são **literalmente a mesma leitura**. Nada a apertar ali.

A distinção só existe no traço **cumulativo** (mão livre), onde dabs de eventos de ponteiro diferentes
caem no mesmo texel e nada restaura nada no meio. Ler o vivo faz cada *batch* re-interpolar sobre o
anterior — e o número de batches **é a taxa de polling do mouse**. Um mouse de 1000 Hz esculpiria mais
fundo que um de 125 Hz, no mesmo gesto, nos mesmos ajustes. O artista sente e nunca consegue nomear.

O gate que faltava: *"a mesma geometria, entregue grosseiramente e entregue finamente, deixa o mesmo
relevo"* — byte a byte (a lista de dabs é idêntica; o espaçamento é por DISTÂNCIA, só o batching muda).
Ele não existia. A mutação foi quem o pediu.

**Why:** a leitura "não sangrou ⇒ afrouxe o gate ou conserte o comentário" convida a mexer no que está
certo. Antes disso, pergunte: *por que essa mutação é inofensiva NESTE caminho?* A resposta costuma
nomear o caminho onde ela **não** é — e esse caminho não tem gate.

**How to apply:** quando uma mutação sobrevive, primeiro **explique por quê**. Se a explicação for "porque
neste caminho as duas versões coincidem", você acabou de encontrar o caminho onde elas divergem — escreva
*esse* gate. Não relaxe o que sobreviveu, e não apague a mutação: aponte-a para o gate novo.

## Um QUARTO desfecho: inobservável por construção (GPU/M5 Fase 3, 2026-07-16)

Às vezes a explicação é *"não existe caminho onde elas divergem"* — e aí não é gate frouxo, comentário
errado, **nem** gate faltando. É **nenhum gate possível**, e dizer isso vale mais que fingir cobertura.

Tirar o clamp `depth.max(1e-3)` do kernel do `force.buoyancy` sobreviveu. Por quê: o `clamp(sub, 0, 1)`
a jusante já doma o ±inf, então os dois caminhos só diferem numa instância **exatamente** na linha
d'água — onde a CPU lê `0/1e-3 = 0` (seca, não move) e a GPU sem clamp lê `0/0 = NaN`, que o guard de
finitude do `motion.integrate` rejeita (**também** não move). **O guard converge os dois.** Persegui um
fixture por um bom tempo antes de perceber que ele não existe.

O clamp fica — ele espelha o `eval`, que é o contrato. Mas o doc do kernel agora diz *quais* clamps têm
gate e qual não tem **e por quê**, em vez de um "os clamps são o modelo, não higiene" que soa protegido
e não está. Compare com o irmão `wave_length`, que é fortemente observável (sem clamp, `phase = x/0 =
inf` → `frac(inf) = NaN` → o campo inteiro NaN-a) e ganhou gate.

**A distinção:** "sobreviveu e eu não sei por quê" é dívida. "Sobreviveu, eu sei por quê, e o porquê é
uma prova de que nenhum fixture alcança" é **documentação**. A 2ª só é honesta se você tiver feito a
conta — [[feedback_a_threshold_must_live_where_the_domain_is_empty]] é onde procurar primeiro, porque a
resposta usual é que o fixture está longe do domínio vazio.
