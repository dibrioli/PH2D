---
name: feedback-a-reverted-attempt-may-differ-only-in-lifetime-read-the-revert-reason
description: "Tentativa revertida com a MESMA fórmula pode diferir só no TEMPO DE VIDA — leia o motivo do revert, não o diff dele"
metadata:
  node_type: memory
  type: feedback
---

Antes de construir, o Enio manda conferir se já foi tentado ([[feedback_documented_decision_chesterton_fence]]).
Quando a busca acha um revert com **a sua fórmula dentro**, a pergunta certa **não** é *"então já foi
tentado?"* — é ***por quanto tempo aquela fórmula valia?*** Escopo e tempo de vida são o que mais mata
tentativa boa, e o diff não os mostra: o **motivo do revert** mostra.

**Caso real (Painter, o gate de proteção, 2026-07-25):** a cura do craquelado era
`canvas = base·(1−keep) + free·keep` aplicada UMA vez. A busca achou `38c1f725b`, revertido, com
**exatamente essa expressão**. Parar ali teria descartado a cura certa. O motivo do revert dizia outra
coisa: a fórmula vivia por **ÉPOCA** (*enquanto a declaração de proteção existir*), logo atravessava traços
e troca de ferramenta ⇒ virava teto cross-stroke e **vazou no brush normal**. Por **TRAÇO** o vazamento é
**estruturalmente impossível** (não há o que vazar), e os **22 sítios de commit**, os planos no snapshot de
undo e o gêmeo do preview que a época exigia **somem todos**. Mesma aritmética, um décimo do risco.

**Why:** um revert registra *que* uma tentativa falhou; quem escreve a nota raramente separa a LEI do
ESCOPO dela, então a nota afirma mais do que mediu. Ler o diff confirma que a fórmula é a mesma e faz você
concluir o oposto do que a evidência sustenta — o custo é jogar fora a cura certa e ficar com o bug.

**Segunda instância, no MESMO dia e no mesmo eixo — e ela fecha o círculo:** horas depois, a lei per-traço
foi medida deixando `1 − (1−keep)^N` passar (oito passadas e a proteção morria), e a resposta certa era **a
própria época** — a semântica revertida, inteira. O que mudou não foi a lei nem o tempo de vida: foi o
**mecanismo de fim de vida** (22 sítios enumerados à mão → **uma pergunta com três testemunhas**). Então a
pergunta a fazer a um revert é ainda mais estreita: *o que exatamente falhou — a lei, o escopo, ou a
MÁQUINA que fecha o escopo?*

**How to apply:** achou um revert que contém a sua ideia? (1) leia a **mensagem** do commit revertido e a
do revert, procurando o mecanismo da falha; (2) escreva numa frase *por quanto tempo* a regra valia lá e
*por quanto tempo* vale na sua; (3) se diferem, monte a **tabela lado a lado** (vida · o que muda entre
gestos · sítios a enumerar · o que entra no undo) e ponha-a no doc — é ela que autoriza a segunda
tentativa, e é ela que impede a terceira de repetir a primeira; (4) faça a mutação que **reinstala o
tempo de vida antigo** e prove que ela sangra, senão nada no repo distingue as duas.
