# HANDOFF CONTINUAÇÃO — line/TimelinePrincipal, a cara dos menus do botão direito (dono 13/11)

Leitor: a próxima LLM. Sem rebase, sem push, sem integração. Tudo o que veio antes está commitado e anotado no
[handoff de integração](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md) (acréscimos 1–6).

## Feito em 13/11 (não refazer)
- Widget do fim do clipe na 2.ª fileira (`src/fim_do_clip.rs`): sem setinhas, a escolhida na cor do chip, placa de pare
  (`IconId::StopSign`), troca animada. Dono: «ótimo».
- Régua com riscos pelo zoom e contraste ≥ 3:1 (`src/ruler_degraus.rs`). Dono: «ótimo».
- Smoke do dono dos acréscimos 5–6: aprovado na conversa («ótimo»); falta só registá-lo no handoff de integração.

## PEDIDO ABERTO (dono 13/11, verbatim)
> «Agora é hora de melhorar a aparência os menus de botão direito»

Capturas do dono (o «antes»), gravadas: [menu da faixa](../fotos/30_menu_da_faixa_antes.png) (Delete Track ·
Convert to Motion Path · Extrapolation Pre ▶ · Extrapolation Post ▶) e [menu da chave](../fotos/30_menu_da_chave_antes.png)
(Hold · Nearest · Linear · Ease In ▶ · Ease Out ▶ · Ease In-Out ▶ · Custom (Bézier) · Rove Across Time).
Hoje: caixa escura simples, texto em negrito, sem ícones, sem atalhos, sem separadores, sem marca do estado atual,
submenus com «▶» colado ao texto.

## Por onde começar (não verificado a fundo — confirme no código)
- Textos: `crates/ph2d-i18n/src/chrome_menus.rs` (`chrome.timeline_menu.*`).
- Quem abre: `crates/ph2d-panel-timeline/src/event_track_menu.rs`, `event_stack_menus.rs`, `marker_menu.rs`.
- Pintor: `crates/ph2d-editor-core/src/widget/context_menu.rs` (220 linhas) só tem 2 usuários diretos por `context_menu::`
  ⇒ os menus da Timeline provavelmente passam por outra porta (menu do chrome / `popover.rs`). Ache a porta ÚNICA
  antes de mexer: mudar o pintor partilhado muda TODOS os menus do app (varrer por consumidor, memória
  `feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site`).
- Molde visual: os cartões (`docs/CartoesUI/README.md`) e a lista do fim do clipe (`fim_do_clip::corpo`: ícone + nome,
  a escolhida em `KanbanCardRaised`, sob o rato `CardFieldHover`).

## Proposta a mostrar ao dono (estado da arte, sem perguntar o acabamento — dono 09/11)
Ícone à esquerda de cada linha; marca do estado atual (a interpolação/extrapolação vigente) em destaque; separador
entre grupos (destrutivo «Delete Track» por último, em vermelho discreto); seta de submenu alinhada à direita (chevron);
atalho à direita onde existir; peso de texto normal (não negrito), altura de linha e cantos dos cartões; sombra.
Fotografe ANTES do smoke, as duas capturas do dono lado a lado com as novas.
