# HANDOFF CONTINUAÇÃO — line/TimelinePrincipal, o widget do fim do clip (dono 13/11)

Leitor: a próxima LLM. Sem rebase, sem push, sem integração. Tudo o que veio antes está commitado e anotado no
[handoff de integração](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md) (acréscimos 1–4).

## Feito em 13/11 (não refazer)
- Abas Keys/Containers/Arrange de pasta na borda esquerda (`src/abas_laterais.rs`); smoke do dono OK.
- Done = ✓ verde no chip do clip; a pílula «Animating» saiu; o ● e o ▶ pulsam ligados; Edit Path é um botão que acende (`src/alternador.rs`).

## PEDIDO ABERTO (dono 13/11, verbatim)
> «Botões de controle de loop, pp e stop visíveis na linha de baixo em um widget original: um dropdown com setinhas
> <> nas laterais, contudo não apenas as setinhas, mas também os ícones das outras opções que existem. Exemplo: se
> loop está selecionado, nas laterais perto das setinhas teremos de um lado o ícone stop, de outro lado o ícone
> pingpong. Desse modo o usuário pode, se quiser, abrir o dropdown ou clicar diretamente no ícone. A troca de lugares
> dos ícones e nomes deve ser animada.»

Imagens do dono (não gravadas em disco; descritas):
1. Hoje: «When the clip ends» com três segmentos `Stop | Loop | Ping-Pong` dentro da janelinha da engrenagem
   (`barra_menus`), Stop escolhido.
2. Seta amarela a apontar da fila de cima (Edit Path · Animation: Herói · engrenagem) para o espaço vazio à DIREITA,
   na 2.ª fileira (a dos clipes): o widget mora aí, visível na barra.

Leitura proposta (confirmar com a FOTO, não perguntar o que a imagem diz):
- Um controlo carrossel de 3 opções na 2.ª fileira, encostado à direita: `‹ [ícone vizinho] [ícone + NOME do escolhido ▾] [ícone vizinho] ›`.
  O nome do escolhido abre a lista suspensa de sempre. As setinhas andam ciclicamente. Clicar no ícone de uma vizinha escolhe-a direto.
- Animação: ao trocar, ícones e nome deslizam/trocam de lugar (o relógio `WidgetStore::on_pulse`/`clock_ns` ou o
  `hero.motion` via `hover_live`; com movimento reduzido, troca seca — `WidgetStore::note_reduced_motion` já existe).
- O estado e o evento são os de hoje (os ids `TIMELINE_LOOP` / `TIMELINE_PINGPONG` e o Stop do segmento da janelinha;
  achar em `barra_menus.rs` e `transport_cartoes::caixas`). Sair da janelinha da engrenagem (um só lugar).
- Ícones: ver se `IconId` tem stop/loop/ping-pong; senão desenhar SVG novo (precedente: `film-frame.svg`).
- Gates a tocar: `os_cartoes` (a 2.ª fileira e os alvos), `o_cartao_estreito` (nada fora), `os_chips_da_barra`;
  tecto de 600 linhas por ficheiro (`transport_cartoes` 568, `barra_clips` 600 ⇒ ficheiro novo).

## Como fotografar
`bash docs/Components/ferramentas/fotografa_cena.sh PH2D_ANIMACAO_SMOKE=1 1930 1040 <saida.png> 6`; rajada com
`FOTO_RAJADA=3 FOTO_INTERVALO=0.27` para ver a animação. Gancho temporário por env (nunca commitado) para estados que pedem clique.
