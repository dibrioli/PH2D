---
name: feedback_reproduce_the_foreign_tools_own_result_before_feeding_it_yours
description: Antes de medir uma ferramenta alheia, reproduza o resultado DELA com os insumos DELA — e só então troque UM insumo de cada vez; trocar malha, campo e formato juntos faz horas de acusação à ferramenta por defeito do próprio insumo
metadata:
  type: feedback
---

⭐⭐⭐ **Antes de medir uma ferramenta que não é sua, reproduza o resultado DELA com os
insumos DELA.** Só depois troque **um** insumo de cada vez.

**Why:** medido em 2026-08-24, ao avaliar uma biblioteca MPL-2.0 de extração de malha
quad contra o nosso corpus. Eu troquei **três** insumos ao mesmo tempo — a malha, o campo
direccional e o formato — e passei horas a escrever que *«a extração é frágil»*.

⛔ **Era o meu insumo, e o controlo cruzado provou-o numa corrida:**

| corrida | resultado |
|---|---|
| malha **deles** + campo **deles** | ⭐ completa em segundos |
| malha **deles** + campo **meu** | cai — o meu campo tinha `curl = 5724` |

⇒ *Um controlo positivo que passa com os dados deles e falha com os teus mede o **teu
insumo**, não a ferramenta.*

⚠️ **Os quatro erros, porque a forma deles repete-se:**

1. ⛔ **Formato:** aliment*ei* uma biblioteca de **triângulos** com malhas de
   **quadriláteros** — o corpus inteiro era quad e eu nunca perguntei. *A ferramenta de
   referência tolerava porque triangula sozinha; a nova não.*
2. ⛔ **Pré-requisito não lido:** o passo exigia campo de **curl reduzido**, e os tutoriais
   dela leem sempre o irmão corrigido. Eu passei o campo liso cru.
3. ⛔⛔ **Balde vazio lido como perfeito:** `curl = 1,47e-15` parecia excelente e era uma
   peça com **zero** restrições sobre uma malha que já era lixo (erro 1).
   Irmã: [[feedback_a_bucket_nobody_fills_reads_as_perfect]].
4. ⛔ **`pkill -f <padrão>` matou a própria janela que o executava** — o texto do script
   estava na linha de comando dela. Use `pkill -x <nome>` ou o pid.
   E `| tail` mascarou três códigos de saída ([[feedback_pipe_masks_script_exit_code]]).

⭐ **O que se salva quando isto acontece:** o controlo cruzado ainda entrega o número que
interessa. A qualidade dela, medida com a **nossa** régua no caso que terminou, deu
enviesamento mediano `5,0°` — a classe do oráculo de produção — contra `27°` do nosso
caminho, **com a cauda de aspecto pior**. ⇒ *classe, nunca placar*: são peças diferentes
([[feedback_n_sources_need_the_cross_check_not_n_self_checks]]).

⭐⭐ **E o pré-requisito descoberto vale para NÓS:** a extração exige campo de baixo curl, e
o nosso campo foi ilibado por **contagem de singularidades**, nunca por curl. *Uma
propriedade que ninguém mediu não está ilibada* —
[[feedback_ask_what_number_the_opposite_answer_would_print]].

**How to apply:**
1. **Corra o exemplo dela, com os dados dela, primeiro.** Se não reproduzir, pare: o
   problema é o ambiente, e nada a jusante vale.
2. **Troque um insumo de cada vez**, e nomeie qual.
3. **Leia os pré-requisitos de entrada de cada fase** antes de acusar a fase.
4. **Toda coluna de diagnóstico precisa da contagem da amostra ao lado.**
