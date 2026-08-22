---
name: feedback_the_oracle_writes_its_intermediate_stages_compare_phase_by_phase
description: Um oráculo de referência costuma gravar as fases intermédias — comparar só o resultado final desperdiça o gabarito, e ler a SAÍDA de um binário GPL é legal
metadata:
  type: feedback
---

Passei dias a redescobrir dos papers duas fases (o campo cruzado e o traçado) cujo
código de referência é GPL e não pode ser traduzido. ⭐ **O binário do oráculo grava
as duas em disco** — `*.rosy` (uma direção por face) e `*.patch` (o patch dono de cada
face), na mesma malha remalhada que ele usou. A bancada comparava só a **malha final**.

**Why:** ler a **saída** de um programa não é obra derivada — é legal, é o padrão, e é
**mais forte** que ler o código: em vez de interpretar intenção, compara-se número com
número, face a face. ⚠️ E o contrário é caro: *"o meu dá 25,7° e o dele 13,7°"* é uma
diferença sem endereço; *"discordam nestas 400 faces, todas na aba da orelha"* é um
bug com sítio.

⚠️ **A comparação tem de correr sobre a malha DELE**, não sobre a nossa — senão a
diferença medida mistura o solver com a fase anterior.

**How to apply:** ao portar/reimplementar um pipeline contra uma referência, **liste o
que ela escreve em disco antes de escrever a primeira linha** (`ls` na pasta de saída
de uma corrida). Cada ficheiro intermédio é um gate de fase grátis. ⛔ Um corpus que
só compara o produto final não diz **qual** fase divergiu — e é nessa pergunta que o
tempo se vai. Irmã de
[[feedback_the_missing_piece_may_already_be_built_measure_its_structure_first]].

⚠️ **Licença, em três linhas:** estudar código GPL é permitido (algoritmo é ideia);
**traduzir é obra derivada**; e quem *lê* a fonte contamina o que escreve depois — é
por isso que existe sala limpa. Ler a **saída** não tem nenhum desses problemas.
