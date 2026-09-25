---
name: topic-ui-seam-discipline
description: "Família: costura de UI — pintado/populado/despachado/clicado, portas únicas, defaults que são lei"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-07-21T01:05:09.375Z
---

# Costura de UI (índice de família — detalhe em cada arquivo)

- [[feedback_context_menu_closes_on_down_repaint]] — menu "não faz nada" = falta populate; grep o id PRIMEIRO
- [[feedback_tool_unit_green_integration_dead]] — unit-verde ≠ funciona no produto; só e2e pega
- [[feedback_a_default_feature_list_does_not_reach_a_consumer_that_disables_defaults]] — lista `default` não alcança quem desliga defaults; gate mora onde o binário compila
- [[feedback_a_click_is_a_press_that_drifted]] — clique humano é um press que DESLIZOU; Down/Up na mesma coord é robô
- [[feedback_a_capability_without_a_door_passes_every_gate]] — motor completo e INALCANCAVEL: sem porta, todo gate fica verde
- [[feedback_painted_is_not_populated_paint_gate]] — pintado ≠ populado; teste a PINTURA
- [[feedback_widget_is_done_when_a_test_clicks_it]] — widget pronto = um teste CLICA nele
- [[feedback_a_condition_that_enumerates_its_readers_rots]] — condição que ENUMERA seus leitores apodrece no 3º consumidor
- [[feedback_a_default_that_fits_the_majority_is_still_a_law]] — default da maioria ainda é LEI; sem porta o 3º caso não mora
- [[feedback_the_fullest_card_premise_rots]] — "o card mais cheio" apodrece; pergunte a CADA modo
- [[feedback_two_doors_to_the_same_question_diverge]] — duas portas para a mesma pergunta DIVERGEM
- [[feedback_disabled_button_still_dispatches]] — botão dimmed ainda despacha; recuse no event.rs
- [[feedback_ship_the_ui_in_the_same_wave_not_later]] — atalho com valores fixos é harness vazando; sem indicador não há diagnóstico
- [[feedback_one_parameter_two_roles_makes_the_wrong_call_defensible]] — parametro com dois papeis: o produto e o unico chamador que os separa
- [[feedback_the_fifth_seam_link_is_whoever_paints]] — as 4 condicoes verdes e o widget ainda le morto: o pintor desenhou o rect a mao e nao le o estado que o despacho JA escreve
- [[feedback_a_hit_rect_is_also_the_denominator_not_only_the_target]] — o rect registado tambem e' o DENOMINADOR: mais estreito que o pintado nao recorta, ESCALA (1,62x)
- [[feedback_a_shared_widget_slot_has_two_questions_and_only_one_was_answered]] — widget partilhado por N rows: quem COMITA tinha 4 armas, o que MOSTRA tinha 1; tocar no campo APAGAVA o valor
- [[feedback_alive_reachable_and_in_the_wrong_place_are_three_questions]] — VIVO · ALCANÇÁVEL · NO SÍTIO CERTO são três perguntas, e a terceira não tem instrumento
- [[feedback_an_indicator_drawn_over_a_widget_outside_its_state_table_dies_when_the_table_changes]] — um sinal traçado POR CIMA do widget, fora da tabela de estados dele, morre quando a tabela muda (o tema plano apagou o anel do modo Image Tools); o estado entra como INPUT do pintor
- ⛔ **`min-height` explícito num item de GRADE de altura presa desliga o mínimo do CONTEÚDO** (Pixel Lab W25, 13/09): a barra de opções com `min-height: 39px` encolheu a 39 px sob aperto e as fileiras quebradas escorreram POR BAIXO do palco (números e Aplicar sumidos); a vizinha sem `min-height` não encolhia. Cura: o mínimo mora num FILHO (conteúdo). E os gates de encaixe só perguntavam pela borda DIREITA ⇒ *um controle também some por baixo: meça a contenção na PRÓPRIA barra, nas duas direções* (achado pela FOTO do smoke, não por gate).
- ⛔ **A foto do Firefox headless (`--screenshot`) não espera NADA assíncrono** (Pixel Lab W28, 13/09, medido):
  um `setTimeout` de 1,5 s, `await` de topo num módulo, IndexedDB e um `await` de 8 s — nos cinco a foto saiu
  em ~2 s sem o fim. Só NAVEGAÇÃO a segura (o gate que espera iframes carregar funciona por isso). ⇒ o que é
  assíncrono por natureza (IndexedDB, o F5 de verdade) roda num Firefox headless SEM foto, a página faz
  `POST` do veredito ao servidor de dev, o script espera o arquivo com teto, e um veredito ausente é
  VERMELHO. Prove o caminho com um protótipo antes (a página que manda o veredito chegou em 3 ms).
  ⭐ E o "banco lento" se fabrica SEM mexer no app: a página de teste abre o mesmo IndexedDB e mantém uma
  transação de ESCRITA viva (um `get` no `onsuccess` do anterior) — o navegador enfileira a leitura da
  abertura do app atrás dela, e o gate age com a abertura PENDENTE (o traço antes de o banco responder, a
  gravação pedida cedo). Foi o que deu gate à guarda "nada grava antes de a abertura decidir".
  ⭐ E a FOTO de um estado assíncrono (o smoke depois do F5) sai pelo **Marionette**, que o Firefox já traz:
  `firefox --headless --marionette` com `marionette.port` no `user.js`, protocolo `len:json`,
  `ExecuteAsyncScript` espera a promessa, `TakeScreenshot` fotografa o DOM, e cliques/arrastes/teclas são
  de verdade (cliente: `docs/Pixel Art/app/tools/marionette.py`). ⚠️ Mova o mouse sobre a janela antes da
  foto: sem nenhum movimento os avisos de logo depois do recarregar não saíram; com ele, opacidade 1. E foi a
  foto que achou "Sessão restaurada: 0 passos" — um estado que nenhum gate perguntava.
- ⛔ **Um `fetch` depois do boot deixa o Firefox headless fotografar NO MEIO** (Pixel Lab W27, 13/09): as
  fontes do Texto vinham por `fetch` e o gate das demos, que espera cada iframe, saiu "…" — uma requisição
  que não é NAVEGAÇÃO não mantém a página ocupada, e a foto sai no primeiro instante ocioso. Em `file://` nem
  há `fetch`. ⇒ *dado que o primeiro quadro precisa vai num MÓDULO GERADO* (o molde do `build_icons.py`), com
  o oráculo conferindo que o módulo traz o arquivo original sem uma letra mudada; e a leitura fica
  preguiçosa (quem nunca pega a ferramenta não paga). Irmã: a 18ª ferramenta quebrou a fileira do meio
  (+32 px) e um gate de ENCAIXE de outra wave (a espada, 1000×640) reprovou — *um botão novo é medido nas
  demos mais cheias, não na dele* (5 px de margem compraram 46 px de folga).
- [[feedback_a_deferred_popover_that_never_publishes_its_rect_cannot_be_light_dismissed]] — popover pintado num passe DIFERIDO nunca fecha ao clique de fora sem `set_dropdown_popover`; os 4 seletores do Inspector viveram assim · e a IRMÃ: eles penduravam a lista SEMPRE abaixo do chip, e o clamp sem rolagem e' meia-cura
- ⛔⛔⛔ **«À VISTA» NÃO É «DENTRO DO RECTÂNGULO» — É ALCANÇÁVEL PELO DEDO** (Teste Cascadeur, 19/09): o auxiliar que rola um painel até um controlo parava assim que o centro dele caía na faixa `50 < y < altura − 50`, e o **RODAPÉ** da app está por cima dessa faixa. Medido: o botão parou a `y = 758`, o `elementFromPoint` ali devolvia `FOOTER`, o clique atravessou para o rodapé e o gesto NÃO ACONTECEU — e o portão, que media o efeito do gesto, reprovou **a apontar para a LEI**. ⚠️⚠️ E era LATENTE: bastaram **três linhas novas no painel (57 px)** para empurrar aquele botão para dentro do rodapé, numa wave que não tocou no código do arrasto. *Um instrumento que erra a apontar para o produto é o mais caro de todos* — fui procurar o defeito na lei, que estava intacta, e a árvore comitada passava. ⇒ o auxiliar exige que o `elementFromPoint` devolva o próprio elemento (ou um parente), senão continua a rolar; curado nas SEIS cópias.

---

### ⛔⛔ O guarda que aceita um ANTEPASSADO aceita um clique que NÃO CHEGA (2026-09-19, 2.ª volta)

⚠️⚠️ **A cura da 1.ª volta (a entrada acima) deixou o buraco escrito dentro dela:** *«exige que o
`elementFromPoint` devolva o próprio elemento (ou um parente)»*. **«Ou um parente» É o buraco** — e
mordeu no mesmo dia, noutro botão. *Uma cura que nomeia a folga que deixa está a marcar o sítio onde
vai ser paga outra vez.*

Os testes de rato desta bancada trazem um `trazerAVista(sel)` que rola até o elemento estar na vista e
confirma, com `document.elementFromPoint`, que o ponto é mesmo dele. O guarda era:

```js
!!(sob && (sob === e || e.contains(sob) || sob.contains(e)))
```

⛔ A terceira cláusula — `sob.contains(e)`, «o que está debaixo do ponto é um ANTEPASSADO» — é
exactamente o caso em que o clique não chega. **Um elemento recortado fora da área visível de um
painel que ROLA continua a devolver `getBoundingClientRect` normalmente**, e ali o `elementFromPoint`
devolve o **painel**. O guarda passava, o clique caía no painel, e o teste lia *«a base do apoio não
desapareceu»* — um defeito de PRODUTO inventado por um buraco do arnês.

⚠️ **A prova de que o portão estava a afirmar nada**: a mutação dele lia **o mesmo número** com e sem
a cura (`151 verdes` nos dois) — *ela estava vácua e lia-se como MORREU*. Depois da cura separa `0/0`
de `0/42`.

⇒ o guarda passa a aceitar **o próprio elemento ou um descendente dele**, mais um caso legítimo e
**NOMEADO**: um `LABEL` que comanda o elemento (clicar no texto liga a caixa). Curado nas **seis**
cópias. ⚠️ O gatilho foi acrescentar um `<select>` ao cabeçalho: ele fez a barra dobrar em duas linhas
(50 → 96 px) e tudo desceu 46 px. *Uma mudança de layout de um controlo novo paga-se num botão do
outro lado da página, e o teste acusa a lei.*
- ⛔⛔ [Uma AMOSTRA de selector NÃO emite evento — o despacho curto-circuita, e um braço de `Click` ali é código morto](feedback_a_picker_swatch_emits_no_event_and_a_click_arm_for_it_is_dead_code.md) (descido do índice na integração de 2026-09-25)
