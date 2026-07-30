---
name: feedback_a_silenced_instrument_reads_as_a_result
description: Um contador que ninguém mais alimenta imprime zero, e zero lê-se como "não custa nada" — gateie a PRESENÇA do instrumento
metadata:
  type: feedback
---

Movi a simulação de água para fora da thread do frame. O medidor do passo ficou onde
estava, e quem passou a dar o passo foi o worker — então o log do produto imprimiu
**`agua: sim media 0.00ms x0`** por uma wave inteira. Aquela linha lê-se como *"a
simulação não custa nada"* e significava *"ninguém mede a simulação"*, sobre
exatamente o número que decidia a frente seguinte.

**Why:** um instrumento **ausente** faz você procurar; um instrumento **silencioso**
te tranquiliza. Zero é um valor válido, então nada na saída distingue *"medi e é
zero"* de *"não medi"* — e um refactor que move QUEM faz o trabalho move o medidor de
lugar sem quebrar compilação nem teste.

**How to apply:** ao mover trabalho entre threads/camadas, liste os contadores que
observavam aquele trabalho e re-ancore cada um no novo executor **no mesmo commit**.
Gateie a **presença**, não o valor: *"depois de N frames o balde tem `n > 0` e soma
> 0"*, com uma mutação por balde (tirar cada `note_*` tem de sangrar). E prefira
baldes que **PARTICIONAM** a janela (busy/away/sleep) a um número solto — a partição
mostra sozinha que alguém parou de alimentá-la, porque a soma deixa de fechar.

Irmão de [[feedback_stale_comment_and_dead_code_lie]] e
[[feedback_a_negative_search_needs_a_positive_control]].
