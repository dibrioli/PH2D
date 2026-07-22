---
name: feedback-ready-to-smoke-example
description: Sempre deixe a feature nova PRONTA PRA SMOKE num grafo/documento default ou demo — nunca instrua o Enio a montar à mão
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 14afaada-70a5-49d0-a3c1-e84cd2bb2756
---

Ao entregar um nó/feature nova, **autore um exemplo que a exercita já no documento default** (ou um preset de 1-ação) que abre mostrando/rodando a coisa — não peça pro Enio montar à mão.

**Why:** o Enio valoriza smoke imediato e sem atrito; montar 20×20 + 5 nós manualmente é tedioso e vira barreira. Quando o gate M1 virou o documento default (grid→tint→falloff→stagger→oscillator→output, auto-play), ele respondeu "tudo perfeito! Mantenha esse padrão de já deixar o exemplo pronto para smoke" (2026-07-08).

**How to apply:** feature nova → autore o exemplo mínimo que a exercita no grafo default/demo (ou preset carregável) + o comando `cd <worktree> && cargo run -p ph2d-host-desktop` copiável ([[feedback_run_command_include_cd]]). Combine com o teste headless irrefutável ([[feedback_painter_inefficiency_4_causes]]): o teste prova a costura, o exemplo deixa o smoke instantâneo. Vale pro smoke 1× no fim ([[feedback_smoke_at_end]]).

## Áudio: **você cria o material de teste** (Enio, 2026-07-13)

*"Para os smokes desse módulo vc deve criar o audio para testes e montar no app. Então eu testo."*
Efeito de áudio **não tem documento default** — sem clipe, não há o que ouvir. Então a linha
**sintetiza o clipe** e **monta a rack**, atrás de um env (`PH2D_AUDIO_*_SMOKE=1`, o padrão que
`editor_loop_smoke` já usava). Zero file-picking, zero discar knob.

Três requisitos, e o terceiro é o que quase se esquece:

1. **O material tem que conter o problema que o efeito resolve.** Um multibanda é indistinguível de
   um compressor comum em material sem tilt espectral — o clipe precisa do kick grave forte CONTRA
   agudo firme e mais quieto. Fixture sem "o outro" não mede nada
   ([[feedback_first_case_rescued_by_side_effect_test_repetition]]).
2. **O A/B tem que ser um gesto, não uma re-discagem.** Monte a cadeia com os dois candidatos no
   MESMO ajuste (`FxStage.enabled` é exatamente o A/B por-stage da rack) — comparar dois desenhos,
   não dois conjuntos de números.
3. **Gateie o MATERIAL, não só o efeito.** Um clipe que não reproduz o sintoma faz o Enio ouvir
   "nada" e a única evidência vira a sua palavra. Meça: no Multiband, o agudo balança **0,532** com
   Compress e **0,002** com Multiband (fonte seca: 0,000) — 266×, e o gate falha se o clipe deixar
   de expor isso.

## A cena tem de ARMAR o gesto, e DIZER o que montou (Enio, 2026-07-21)

*"Não sei se você preparou corretamente o arquivo de teste."*

Montar a fixture **não basta**: se a feature só age atrás de um gesto (uma seleção, um modo,
um toggle) e a cena não o arma, o produto entrega o caminho de sempre — e o resultado é
**indistinguível da feature quebrada**. No onion fill do Flip a cena criava as 3 chaves e não
marcava nenhuma; marcar era Shift/Ctrl+clique célula a célula, que a cena nem dizia onde
ficava. Um Apply coloria um quadro só, exatamente como se o fan-out não existisse.

Explicar melhor no texto **não é a correção** — a cena arma. (Deixe o caminho de volta: *"para
ver um quadro só, clique sem modificador"*.)

E a cena **imprime o que construiu** (`3 chave(s) em [0,4,8], 3 marcada(s)`): *"o arquivo de
teste está certo?"* é pergunta que o smoke responde sozinho, senão a única evidência vira a
sua palavra de novo. Nomeie também o que aparece na tela e **não é arte** (um fantasma de
onion skin lê como um segundo objeto que ninguém desenhou).

Gateável por arch-gate sobre o fonte da cena — o smoke precisa de janela, mas o texto dele não.
