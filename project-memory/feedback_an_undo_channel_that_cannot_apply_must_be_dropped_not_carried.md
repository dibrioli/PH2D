---
name: an-undo-channel-that-cannot-apply-must-be-dropped-not-carried
description: Um canal de desfazer com cerca de validade tem de LARGAR quando a cerca recusa — carregá-lo para a fila oposta é um defeito de DIRECÇÃO que só aparece dias depois
metadata:
  type: feedback
---

Numa fila de desfazer **involutiva** (aplicar instala o estado carregado e
devolve o que estava lá), um canal cuja escrita pode ser **recusada** por uma
cerca de validade tem duas saídas, e só uma é certa.

**A errada é a óbvia:** carregar a janela inalterada para a fila oposta. Ela
lê-se como conservadora — *«não apliquei, logo guardo»* — e é um defeito de
DIRECÇÃO: uma entrada carrega o estado de **ANTES**, e quem a aplica devolve o
de **DEPOIS**, que é o que o refazer instala. Uma janela que não se pôde
aplicar **não tem o «depois»** ⇒ o `Ctrl+Shift+Z` instala as cores de antes
outra vez, ou seja **desfaz duas vezes**.

⚠️ E o defeito só arma no dia em que a cerca voltar a aceitar (o artista
re-arma o mesmo degrau, a mesma topologia) — *dias depois, sem relação visível
com o gesto que o causou*.

**Why:** medido em 2026-09-21 no quarto canal do desfazer da tinta fina
(`ph2d-app-sculpt3d :: history_tinta_fina`). A cerca é a identidade do plano de
amostras; largar é a única saída que nunca escreve na direcção errada.

**How to apply:** quando um canal de desfazer tem cerca, faça a aplicação
devolver `Option<Janela>` e o `None` querer dizer **largada**, nunca «tenta
outra vez». Escreva o mecanismo da alternativa ao lado — *um payload que
sobrevive à recusa é pior que a recusa*. Relacionado:
[[what-the-undo-does-not-photograph-the-undo-does-not-restore]].
