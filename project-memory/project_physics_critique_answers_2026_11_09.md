---
name: project_physics_critique_answers_2026_11_09
description: Dono 09/11 aprovou as 5 recomendações sobre a crítica do corpo/forma/gatilho — Sensor fica, etiqueta de ouvintes, Width/Height, «More» fechado, tecla no cartão do Trigger
metadata:
  type: project
---

Dono 09/11 («faça conforme sugere»), sobre a crítica externa (`GameEngineSamples/critica_ph2d.html`) do corpo físico, forma e gatilho:

1. **Touch = Solid | Sensor fica** (reafirma 07/11); «Trigger» continua a ser o componente da tecla. Não renomear.
2. **Cada caixa de nome de sinal ganha etiqueta** «N listen» / «nobody listens» / «nobody sends» — etiqueta, NÃO lista de sugestões (a regra de 02/11 contra lista continua).
3. **Half Size → Width e Height** na forma.
4. **Campos raros do corpo vão para uma parte «More» FECHADA por omissão** — nada some (07/11: não perder poder de configuração).
5. **Tecla escolhida no próprio cartão do Trigger**; o Input Map continua.
Técnico (delegado): nome da camada no campo Layer, recibo «Removed — Undo» ao remover o corpo, aviso de corpo escalado, botão que acende as alças da forma.

**Why:** a crítica apontou erros mudos (nome de sinal errado) e campos que confundem; o dono aceitou o pacote recomendado.
**How to apply:** ondas 0 (mover código: app-physics no tecto exato, shell com 2 linhas) → 1 etiqueta de sinal → 2 Width/Height + nome da camada → 3 More/Undo/escala → 4 tecla no Trigger. Ver [[project_physics_cards_owner_decisions_2026_11_07]].

Dono 09/11 (smoke das ondas 2–3 OK): a dobra dos campos raros chama-se **More**; a curva da Timeline mostra o MESMO número do cartão (Width/Height inteiros, altura inteira da cápsula, °/s) — "resolva agora".

Dono 09/11 (onda 4, tecla no cartão do Trigger): linha **Key** no cartão com a tecla atual + botão que espera a próxima tecla (Esc cancela). **Troca**: a tecla nova SUBSTITUI as teclas de teclado da acção (botões de comando ficam; duas teclas = janela Input Map). **Tecla repetida noutra acção: aceita e avisa** em cinza («Space is also used by jump»). **Nome por criar: um clique cria a acção e já espera a tecla**; campo vazio = botão apagado.

Dono 10/11 (abertos da onda 4): **«Set Scale to 1» conserta TUDO num clique** — repõe a escala de todo elo da cadeia (forma e antepassados) cuja escala é desigual, num só passo de undo, espelho mantido; acabou o «dois cliques». **O botão antigo «Create this action» sai** (escolha delegada): o «Create and set key» já cria, e Esc depois dele deixa a acção criada sem tecla — dois botões para a mesma cura confundiam.
