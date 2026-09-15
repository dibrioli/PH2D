---
name: a-doc-that-states-the-law-the-code-does-not-implement-reads-as-audited
description: O comentário dizia «desmarcada = um degrau abaixo do painel» sobre código que pintava o cartão — e eu li o comentário, acreditei, e curei dois pintores de seis.
metadata:
  type: feedback
---

Num repo onde o *porquê* vive em doc-comments densos, um comentário **correcto sobre a intenção e
falso sobre o código** é pior que nenhum: ele consome exactamente a atenção que ia auditar aquela
linha. O leitor seguinte lê a lei, reconhece-a como certa, e segue em frente.

**Caso medido (`line/UIUX`, 2026-09-14).** Report do dono com foto: *«caixas de input numérico sem
cor de fundo»*. Curei os dois pintores que ele nomeava e escrevi dois gates. Horas depois, segundo
report com foto: *«Checkbox invisível»* — a mesma causa, um widget ao lado. O código da checkbox
trazia, na linha de cima do defeito:

> *«num tema moderno a caixa é plana (marcada = acento cheio, **desmarcada = um degrau abaixo do
> painel**)»*

…sobre um código que enchia com o token do **cartão**. Eram **seis** pintores com o mesmo defeito
(`number_input` · `text_area` · `checkbox` · `combobox` · `dropdown` · `radio_group`); o meu censo
da primeira volta media o vocabulário da família do sintoma (`fill_token`), não a pergunta.

**Why:** duas causas somam-se. (1) *Um censo escrito à volta do sintoma mede a família do
sintoma* — o report nomeia um widget, e a varredura nasce com o nome dele dentro. (2) Uma frase
que descreve a lei certa **desarma** a leitura da linha seguinte: ninguém verifica o que já leu
declarado.

**How to apply:** quando um report é sobre uma PROPRIEDADE VISUAL (*«não se vê»*, *«está cortado»*,
*«é igual ao fundo»*), a varredura da cura é sobre a **propriedade**, nunca sobre o widget — aqui,
*«quem pinta um corpo com o token da superfície em que ele assenta?»*, sobre todos os ficheiros de
widget, com piso de população. E trate um comentário que declara uma lei como uma **afirmação a
verificar**, não como documentação: se ele nomeia um número ou um degrau, meça-o; se ele estivesse
certo, o gate que o defende existiria.

Vizinhos: [[feedback-a-door-the-neighbour-does-not-call-is-not-a-door-yet]] ·
[[feedback-a-ported-law-carries-the-source-apps-premise-about-its-own-layout]] ·
[[feedback-a-blocker-written-over-an-argument-the-result-ignores-is-a-fake-price]] ·
[[reference-topic-gate-discipline]]
