# 12 — O cartão «Live» separado por tipo de ligação (W23, pedido do dono 18/10)

> Pedido (depois do smoke OK da cópia do link): *«Os quadros precisam separar o conteúdo por tipo de conexão para
> que fique claro e intuitivo.»* Foto de antes: [`dono_cartao_live_antes_da_separacao.png`](ferramentas/w23_fotos/dono_cartao_live_antes_da_separacao.png).

## 1. Estado da arte (o que se faz, e porquê)

- **Excalidraw** (oráculo instalado, `ferramentas/excalidraw_oracle`): o diálogo «Live collaboration» tem o link
  com «Copy link» num bloco e «Stop session» SOZINHO no fim, separado — a ação que acaba com tudo não se mistura
  com as de partilhar.
- **Figma / Miro** (diálogo «Share»): convidar por e-mail num bloco, o LINK noutro bloco com título próprio; as
  pessoas que já estão dentro numa lista à parte. Cada bloco responde a «como é que alguém chega aqui?» por UM
  caminho.
- **O nosso modelo de cartões** (`widget/section_cards`, dono 06/09: *«uma subsecção está com o seu título dentro
  do card da secção mas o seu conteúdo fica dentro de outro card de cor diferente»*): o título fica no cartão do
  pai, o corpo num cartão de outro tom. É a porta da casa para «isto é um grupo».
- Abandonado/recusado por nós: um risco entre secções (`section_cards` §«O CARTÃO substitui o RISCO» — o risco diz
  «acabou» e não diz «do quê»). Não há evidência MEDIDA de que os outros apps tenham abandonado um desenho.

## 2. O desenho (e a porta única de cada pergunta)

Anfitrião (`Hosting`): **Same network** [código · frase · endereços · firewall] · **Internet** [convidar / a preparar /
link copiado + rede estrita + copiar de novo / falhou] · **People** [lista] · fora dos cartões, no fim: «Bring
everyone here» · «End session».

Sem sessão (`Off`): **Same network** [«Open to the network» · firewall · «Join a board» · achados · Code · Address ·
Join] · **Internet** [«Invite over the internet» (abre a sala E faz o link: `Host` + `InviteInternet`, a mesma
sequência da cena 22) · a dica · campo **Link** NOVO · «Join by link» · a frase da falha].

A entrar (`Joining`) e convidado (`Guest`): o cartão do caminho por onde se entrou (Same network OU Internet) com a
espera / «In Ana's board», depois **People** e as ações.

Portas únicas:
- *por onde entrou este convidado?* → `LiveLink::by_link`, escrito UMA vez pela família em `join_by` (a mesma
  função que escolhe a rota), apagado em `LiveLink::ended`. O painel não deduz da `searching` (que some ao entrar).
- *em que secção está esta fila?* → a lista `rows()` com `Row::Section(..)`/`Row::End`; o pintor só lê.
- *o tom do corpo de uma secção* → `CardDepth::Section.fill(theme)` (a porta dos cartões, incl. o tema Oled).
- *de onde vem o link para entrar* → `join_link`: o campo Link; o campo Code só se tiver um link (compatível com
  quem já cola lá).

## 3. Contratos congelados / schema

Nenhum: só `ph2d-editor-core` (ecrã, `documents_live`), `ph2d-app-board` (uma linha em `join_by`), `ph2d-i18n`.
Nenhum formato gravado nem de fio (o `LiveView` vive na memória). Grep: `NodeOp|Tool|VectorOp|PROJECT_SCHEMA` sem
ocorrência nos ficheiros tocados.

## 4. UI (as quatro condições)

Campo Link: id `BOARD_LIVE_LINK_INPUT` · registado em `ensure_fields` · pintado por `paint_field` (hit-index) · o
`Submit`/Enter e «Join by link» levam a `LiveRequest::JoinLink`. «Invite over the internet» sem sessão: botão
existente, o clique pede `Host` e `InviteInternet`, a família serve-os pela ordem (cena 22).

## 5. Gates (red-first)

- `the_host_card_groups_each_way_in_its_own_card` — os botões clicáveis (o rect que o rato acerta) da firewall
  dentro do cartão Same network, «Copy the link again» dentro do Internet, as pessoas dentro do People, «Bring
  everyone here»/«End session» abaixo de todos; os três títulos pintados (glifos) por cima do cartão deles.
- `off_the_link_has_its_own_field_in_the_internet_card` — o link colado no campo Link + Enter e «Join by link» pedem
  `JoinLink`; «Invite over the internet» sem sessão pede `Host` e depois `InviteInternet`.
- `a_guest_sees_the_way_it_came_in` — por link: cartão Internet e não Same network; pela rede: o contrário.
- Família: `by_link` verdadeiro só pela rota do link, falso depois de sair.
- Mantêm-se: cenas 12/22 (`smoke_live_tests.rs`), `the_corner_rises_above_the_stats_hud_in_a_narrow_window`,
  `fit::cover` do cartão.

## 6. Cena de smoke

As cenas 12 (rede local) e 22 (internet) já abrem o cartão; a lição 3 da cena 22 passa a dizer «cole o link no
campo Link». Fotos 1366×768 e 1930×1040 de cada uma, mais a segunda janela sem sessão.
