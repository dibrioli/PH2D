---
name: feedback-rebuilding-mid-measurement-splits-the-run-across-two-binaries
description: "Uma corrida de medicao ainda a iterar e um `cargo build` no meio: as primeiras pecas mediram um binario e as ultimas outro, e eu li isso como NAO-DETERMINISMO"
metadata:
  node_type: memory
  type: feedback
---

Lancei uma varredura de 14 pecas em background, li a saida parcial, e enquanto ela
ainda iterava editei o codigo e **reconstrui o binario que ela estava a invocar**.
As quatro primeiras pecas mediram a versao antiga e as restantes a nova.

Ao comparar depois com uma corrida directa, dois numeros da MESMA peca nao batiam
— e eu escrevi «isto pode ser nao-determinismo», que num modulo cujo contrato e' o
determinismo (HR-5) e' uma acusacao seria. Tres corridas seguidas do binario
parado sairam **identicas**: a cadeia e' determinista, e o defeito era o metodo.

**Why:** um `cargo build` substitui o ficheiro **no sitio**, e um laco de shell
que invoca `./target/release/examples/X` resolve o caminho **a cada iteracao**.
Nada avisa. ⇒ a saida de uma varredura longa nao e' de UM programa; e' de todos os
que existiram durante ela. *Uma tabela assim mistura duas leis e le^-se como ruido
— ou, pior, como uma propriedade do algoritmo.*

**How to apply:** antes de editar codigo, verifique se ha' medicao a correr contra
ele (`pgrep`, ou a lista de tarefas). Se houver, ou espere, ou **congele o
binario** primeiro (`cp target/release/examples/X /tmp/X_congelado`) e meca contra
a copia — que e' barato e torna a tabela atribuivel a uma versao. E quando dois
numeros da mesma entrada divergirem, a primeira hipotese a testar e' «media o mesmo
binario?», nao «o algoritmo e' instavel»: repetir a corrida com o binario parado
custa segundos e refuta-a ou confirma-a de vez.
Irma^ de [[feedback-a-stage-accused-of-creating-a-defect-needs-a-clean-input-control]].
