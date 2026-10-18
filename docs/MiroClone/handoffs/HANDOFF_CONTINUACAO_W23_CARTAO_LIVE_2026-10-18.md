# CONTINUAÇÃO — `line/MiroClone` W23: o cartão «Live» separado por caminho de ligação (pedido do dono) fechado; o próximo é o bloco 3 «quadro» — 2026-10-18

> Leitor: a LLM da próxima janela. Lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) ·
> bloco 2: [HANDOFF_CONTINUACAO_W23_BLOCO2](HANDOFF_CONTINUACAO_W23_BLOCO2_2026-10-18.md). Plano:
> [12_cartao_live_por_ligacao.md](../12_cartao_live_por_ligacao.md). Commits: `ec6facde58` (o cartão) · `0375c64cae` (cena 23 + fotos +
> o enquadramento) · `967428b634` (README) · `e813cffc67` (a folha da busca compara). Sem integração nem push: só por ordem do Enio.

## 1. O que mudou

Pedido do dono (18/10): *«separar o conteúdo por tipo de conexão»*. O cartão «Live» passa a ter um cartão por caminho.

**Filas** — `crates/ph2d-editor-core/src/screens/hero/board_live_rows.rs` (NOVO): `Row::Section(LiveSection)` / `Row::End`,
`LiveSection{Network, Internet, People}`; títulos i18n `board.live.sec_network` «Local network», `board.live.sec_internet`
«Internet», `board.live.people`. **Pintor** (`board_live_paint.rs`, `place()`): título acima do cartão (altura `title_h()` = Sm +
`Spacing::Sm`), corpo recuado `card_pad_px` num cartão de tom `CardDepth::Section.fill(theme)`; os rects ficam em
`LivePanel::sections` (`documents_live.rs`).

**Conteúdo por estado**
- Anfitrião: Local network [código, frase, endereços, firewall] · Internet [convidar / a preparar / link copiado + rede
  estrita + copiar de novo / falhou] · People. «Bring everyone here» e «End session» ficam FORA, no fim.
- Off: Local network [Open to the network, firewall, Join a board, achados, Code, Address, Join] · Internet [Invite over
  the internet, «Got a link? Paste it here», campo **Link** novo (`BOARD_LIVE_LINK_INPUT`, +1 id em `ids/chrome/documents.rs`),
  Join by link, a frase da falha].
- Joining/Guest: o cartão do caminho por onde entrou (`LiveLink::by_link`, escrito em `join_by` em
  `ph2d-app-board/src/live_internet.rs`, apagado em `LiveLink::ended`), depois People e as ações.
- «Invite over the internet» sem sessão pede `Host` + `InviteInternet` (a mesma sequência da cena 22).
- `join_link` lê o campo Link; vazio → cai no campo do código (compatível). A lição 3 da cena 22 diz agora «paste the link in
  the Link field».

## 2. Cena 23 (nova)

`PH2D_BOARD_SMOKE=23` (`scene_join` em `smoke_live.rs`, ROUTERS `max_level` 23): a Retro sem sessão com o painel aberto;
lições `board.smoke.join.l1..l6`. Gate `the_join_scene_opens_the_panel_without_a_session`.
⚠️ O §5 do `CLAUDE.md` diz `PH2D_BOARD_SMOKE=1..22` — **o integrador conta 23**.

## 3. Defeito achado pelas fotos (A1, bloco 1)

`free_of` (`board_nav_fit.rs`) escolhia o lado de cada coberta uma a uma (a maior primeiro); a 1366×768 a cena 22 ia para
baixo do cartão e os botões do canto comiam-na. Agora `search` percorre as combinações de lados e fica com o maior zoom final;
o ramo cujo rect já não bate o melhor sai cedo (cortar nunca aumenta); a folha compara sozinha (`e813cffc67`: a mutação M11
mostrou que a poda DECIDIA o resultado — agora é só velocidade). Gate red-first
`each_side_is_chosen_by_what_is_left_after_every_cover` (antes 1,8, agora 2,5). Custo: pior caso 4^n cortes com n cobertas que
se cruzam (hoje ~5), com poda.

## 4. Gates

Editor-core, `board_live_sections_tests.rs` (NOVO; `board_live_tests.rs` está em 699 linhas — no tecto, teste novo vai para
ficheiro irmão):
- `the_host_card_groups_each_way_in_its_own_card` — onde o rato acerta cada botão, dentro do cartão certo; glifos do endereço e
  das pessoas dentro dos cartões; títulos pintados na faixa por cima; ações abaixo de todos.
- `off_the_link_has_its_own_field_in_the_internet_card` — Enter no campo Link e «Join by link» pedem `JoinLink` com o link
  exato; Invite sem sessão pede Host+InviteInternet.
- `a_guest_sees_the_way_it_came_in`.

App-board: `only_the_link_route_marks_the_session_as_internet` (`live_link_tests.rs`). Mutações M1–M11 todas vermelhas.

## 5. Prova de fecho

- `fmt` limpo; `clippy -D warnings --all-targets` limpo em `ph2d-editor-core`, `ph2d-app-board`, `ph2d-i18n`.
- Por nome, verdes: `live_board::` 21 · `board_nav`/`corner` 11 · cenas 12/22/23 e link 7.
- Suíte `PH2D_SUITE=onda nextest-impacted`: 19 341 corridos, 19 340 passaram. 1 falha:
  `ph2d-physics-ecs::it measure_player_budget::the_cost_of_a_player_is_linear_in_their_number` com loadavg ~58–66 — já
  listada em [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md); sozinha 1/1 verde (load 19); a crate NÃO tem diff nesta linha.

## 6. Fotos

`docs/MiroClone/ferramentas/w23_fotos/`: `live_cartao_anfitriao_{1366x768,1930x1040}.png` (cena 22 com
`PH2D_BOARD_INTERNET=local`) e `live_cartao_entrar_{1366x768,1930x1040}.png` (cena 23), por `fotografa_cena.sh` em kwin virtual.
A 1366×768 a cena (6 lições lado a lado) fica à esquerda do cartão e pequena demais para ler as notas (o texto some abaixo do
limiar de detalhe); a 1930×1040 lê-se tudo. Não é defeito novo: é o tamanho da cena contra o cartão aberto.

## 7. Superfície de colisão

`board_live.rs`, `board_live_paint.rs`, `board_live_rows.rs` (novo), `board_live_tests.rs`, `documents_live.rs`
(`LiveLink::by_link`, `LivePanel::sections`), `ids/chrome/documents.rs` (+1 id), `board_nav_fit.rs`;
`ph2d-app-board`: `live_internet.rs`, `smoke.rs`, `smoke_live.rs`, `smoke_live_tests.rs`; `ph2d-i18n`: `board_live.rs`.
Contadores como delta: +1 id de chrome, +1 cena de smoke (23), +1 ficheiro de testes em editor-core, chaves i18n
`board.live.sec_*`, `board.live.people`, `board.smoke.join.l1..l6`. Um merge pode partir: ROUTERS (`max_level`), a contagem de
ids de chrome, e qualquer cena/lição que cite o campo do código em vez do Link.

## 8. Smoke para o dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build -p ph2d-host-desktop --profile smoke && PH2D_BOARD_INTERNET=local PH2D_BOARD_SMOKE=22 ./target/smoke/ph2d-host-desktop`
   e, noutro terminal (depois do mesmo `cd`): `PH2D_BOARD_INTERNET=local PH2D_BOARD_SMOKE=23 ./target/smoke/ph2d-host-desktop`.
2. Na 1.ª janela o cartão «Live» mostra três caixas com título: «Local network» (o código grande e o endereço), «Internet»
   («Link copied…» e «Copy the link again»), «People»; «Bring everyone here» e «End session» ficam por baixo, soltos.
3. Na 2.ª janela o cartão mostra «Local network» (Open to the network, Code, Join) e «Internet» (Invite over the internet, o
   campo «Link», Join by link). Cole o link (Ctrl+V) no campo «Link» e carregue Enter (ou «Join by link»): entra no quadro e o
   cartão passa a mostrar «Internet» (In enio's board) e «People».
4. Deu errado se: algo da internet aparece na caixa da rede local (ou o contrário); colar no campo «Link» não entra; ou depois
   de entrar a caixa de cima diz «Local network».

## 9. ABERTO — o próximo é o bloco 3 «quadro»

Conferido no código pelo explorador (18/10); reconfira antes de construir:
- **A7 ABERTO** — o `surface` de `ph2d-board-words::keywords` é `case_fold`, «API» vira «#api». Plano: guardar a forma escrita
  quando há maiúscula além da 1.ª letra; baixar a maiúscula de começo de frase.
- **A10 PARCIAL** — realce feito; o foco ~4,3 px aberto (`ph2d-board-route/src/lib.rs`, `anchor_for`).
- **A13 ABERTO** — `PlayerInput` sem `Wheel`; a roda vai ao zoom em `board_view.rs`.
- **B1 ABERTO** — só Sticky/StickyStack/StickyWide; a lei do cofre é `is_note()`.
- **B3** (a) (b) (d) ABERTOS; (c) FEITO.
- **B4 FEITO** — `heads.rs` `size = min(base, leg/2)`; o doc 07 linhas 137/169 está desatualizado.
- **B5** — (1) cotovelo, (2) rótulo ao longo, (5) cursor nas pegas, (6) trackpad, (7) botão direito numa forma: ABERTOS;
  (3) pontas UML PARCIAL (o modelo tem, a barra só 5); (4) abas PARCIAL; (8) texto-guia PARCIAL.
- Fora desta onda (precisam do dono): o furo real pela internet, Windows/Mac reais.
