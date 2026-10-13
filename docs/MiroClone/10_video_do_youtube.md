# O vídeo do YouTube no quadro — o mecanismo (W18, 2026-10-12)

> Decisão: [ADR-0185](../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md)
> (número provisório). Medições dos motores: [`ferramentas/youtube_motor/README.md`](ferramentas/youtube_motor/README.md).
> Oráculo: [`ferramentas/excalidraw_oracle/README.md`](ferramentas/excalidraw_oracle/README.md) §W18. Handoff:
> [integração W18](handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W18_2026-10-12.md).

## O mecanismo

Ordem do dono: um link do YouTube no quadro exibe o vídeo «como o navegador». **Motores medidos** (README `youtube_motor/`):
janela-filha do `wry` impossível no Wayland (só X11) · WebKitGTK 4.1 fora de ecrã não toca (falha GBM / preso em 0:00, mesmo
com `gst-plugins-good`) · Servo 0.7 embebido toca a 29,9 fps mas a 360p (sem MSE) e a API parte todo mês · CEF +364 MB ·
**Chrome por CDP: 29,97 fps, som, clique 7–28 ms**. O dono decidiu (12/10) a «janela virtual»: o navegador da família Chrome
da pessoa, sem cabeça, CDP, projectado no canvas ([ADR-0185](../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md),
**número PROVISÓRIO**: máximo no main 0181, linhas 0184 — recontar). Lei do YouTube (Required Minimum Functionality):
nada à frente do leitor, ≥ 200×200, Referer ⇒ **vivo só destapado, sem rotação, inteiro à vista, ≥ 200 px**
(revogado na W21: o vídeo desenha-se à frente, só outro vídeo o tapa; e na W20, «algum pedaço à vista»).

6. **Modelo.** `ElementKind::Embed(Box<Embed{link, video: Video::YouTube{id,start,vertical}, style}>)` no **FIM** do enum
   (`FORMAT_VERSION` fica 13: lei do repo, variante no fim não muda o formato); `Field::Embed` (`Field::ALL` 38→39);
   `ph2d-board-sync` `PROTOCOL` 6→**7**. Geometria: `shape_of` trata o Embed como imagem (seleccionar/mover/redimensionar;
   redimensionar **uniforme** — divergência do oráculo, que estica livre); duplo-clique num embed não abre texto.
7. **Crates e camadas** (mecanismo no ADR e no [`youtube_motor/`](ferramentas/youtube_motor/README.md)):
   `ph2d-board-embed` (lei pura do link, `live_allowed`/`MIN_LIVE_SIDE`, gate contra os **24 links do oráculo** com correcções
   NOMEADAS) · `ph2d-board-video` (miniatura HTTPS `ureq`/`rustls`/`ring`/`rustls-native-certs`, sem `webpki-roots` por licença,
   cache em `$XDG_CACHE_HOME/ph2d/video`; `Player` por CDP em `--remote-debugging-pipe` lançado por `sh -c 'exec … 3<&0 4>&1'`,
   **sem `unsafe`**, o navegador sai sozinho 0,05 s depois de o PH2D morrer; página servida por `Fetch` em `https://ph2d.local/`)
   · render `ph2d-board-render/src/video.rs` (`Videos`, `THUMBS` global, *tapado* = elemento não-frame por cima) · editor
   `ph2d-editor-core/src/screens/hero/board_video.rs` (Ctrl+V de link; play/2.º clique activa; activo+vivo → rato/teclas ao
   leitor; activar pequeno/meio fora **enquadra**; tapado/rodado → aviso; `Item::Video` no FIM de `items()` e `TOOLBAR_TAIL`) ·
   app `ph2d-app-board/src/video.rs` (arranca **só se vivo**, pausa ao deixar de estar, permanência 60 s; `NoBrowser` → abre o
   navegador do sistema + aviso).
8. **Régua do critério de morte** `the_person_s_browser_plays_at_24_fps_and_answers_a_click` (`#[ignore]`, navegador real):
   **29,99 fps distintos, clique→quadro 9–24 ms, a `loadavg` 74 e a 5**.
9. **`.excalidraw`** lê/escreve `embeddable` com o link (o X3 já não salta os do YouTube). **Exportação:** PDF `/URI`
   (`pdf::write_with_uris`), SVG `<a href>` (`svg::write_with_uris`), PNG miniatura+play. **Largar links:** `UriList::links`
   (`ph2d-wayland-dnd`) + `video::drop_links`; a shell só avisa dos recusados. **Ao vivo:** o vídeo de um convidado chega aos
   outros como o LINK (gate `a_guest_video_reaches_the_others_as_its_link`). `deny.toml` proíbe `rustube`, `rusty_ytdl`,
   `ytextract`, `youtube_dl`, `yt-dlp`, `ytdlp`, `wry`, `cef`, `cef-dll-sys`, `servo`, `webkit2gtk`.
10. **Memória:** `project-memory/project_youtube_in_the_board_uses_the_persons_own_browser_2026_10_12.md` (+ linha no
    `MEMORY.md`) está no checkout PRIMÁRIO (não rastreado lá) — recolha-o ao integrar.


## §W19 — os defeitos do smoke do dono e os NOSSOS controles (12/10)

Decisão: [ADR-0185 §W19](../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md).
Sonda: [`ferramentas/youtube_motor/sonda_w19_tocador.mjs`](ferramentas/youtube_motor/sonda_w19_tocador.mjs)
(`FASES=P,F,Z,C,A`, `ECRA=1`), saídas em `youtube_motor/saidas/w19*` (Chrome 155, Big Buck Bunny).

| pergunta | medido | onde |
|---|---|---|
| o tocador NOVO do YouTube (2026) | classes `ytm*`/`ytSpec*`; os seletores `ytp-*` antigos não existem — a sonda acha os alvos pelo nome acessível | `w19/`, `w19b/` |
| o que `controls=0&fs=0&disablekb=1&rel=0&iv_load_policy=3` tira | some: a barra, o tempo, volume, legendas, definições, ecrã inteiro, «Mais vídeos» (⚠️ 13/10: o «Mais vídeos» APARECE, §W20); FICAM: título, canal, «Copiar link», logotipo «YouTube», play ao meio | `w19/P_*_rato.png` |
| duplo-clique / `f` com os nossos parâmetros | não entram em ecrã inteiro | `w19c/` |
| ecrã inteiro do tocador, Playwright | não deforma (o Playwright emula o ecrã) | `w19d_ecra0/` |
| ecrã inteiro do tocador, O NOSSO `Player` | quadros **420×315** numa caixa 560×315 (o ecrã imaginado 800×600 → 4:3) | teste `the_player_s_fullscreen_corner_…` |
| zoom: o tamanho a cada 16 ms, 5 s (4 variantes × 2, intercaladas) | **0 pausas** em todas; só métricas sem recomeçar o screencast: 12–14 q/s (o screencast não segue); recomeçar: 36 q/s; assentar 250 ms: 30 q/s | `w19/w19_tocador.json` `Z` |
| canal / título / «YouTube» | cada um abre uma ABA NOVA (`targetCreated`, `popup`); a moldura NUNCA navega; «YouTube» também pausa o vídeo | `w19b/` `C` |
| … no NOSSO `Player` | 1 quadro em 3 s, luz 9,5/255 (preto) → com a guarda: 90 quadros, luz 179 | teste `clicking_the_channel_…` |
| a API oficial | `initialDelivery`, `onReady`, `infoDelivery` (~3,7/s a tocar; parado, cala-se), `onStateChange` (depois do `addEventListener` no `onReady`); `mute`/`setVolume`/`seekTo` 4–10 ms; `pauseVideo` ~630 ms; `playVideo` ~255 ms | `w19b/` `A`, teste `our_controls_…` |
| anúncios (sem sessão, Big Buck Bunny) | nenhum (`ad-showing` ausente, sem `adState`) — a linha de base do Premium | `w19b/` `A` |
| anúncios nos MONETIZADOS, sem sessão (`dQw4w9WgXcQ`, `kJQP7kiw5Fk`, `9bZkp7q19f0` × `youtube-nocookie.com` e `www.youtube.com`, 15 s cada) | **0 de 6** (a classe `ad-*` nunca aparece) e, pelo OLHO (o detector não tinha controlo positivo), aos 3 s o Despacito mostra o próprio vídeo nos dois domínios ⇒ o Premium não teria o que tirar: a parte Premium NÃO se constrói (o perfil persistente com a sessão da pessoa custaria privacidade por nada); volta ao dono com a medida | `w19_premium/` |

**O mecanismo da cura.** `Player` (`ph2d-board-video/src/player.rs`): a página nossa pede o estado (`listening` a
cada 250 ms até à 1.ª resposta), regista `onStateChange` no `onReady` e passa cada mensagem pelo binding `ph2dState`;
`Playback` (`playback.rs`) guarda fase/tempo/duração/volume/mudo e a fase PEDIDA (`want`, ≤ 1,5 s) para o botão
mudar no quadro seguinte. `Target.setDiscoverTargets` → cada aba nova (não a nossa) com endereço `http*` fecha-se,
a nossa volta à frente, e o endereço vai para `take_outside` (o app abre-o no navegador do sistema). As métricas
levam `screenWidth/Height` = a caixa. O app (`ph2d-app-board/src/video.rs`): `held` guarda se tocava quando as
regras o pausaram; `settled` só muda o tamanho do tocador 250 ms depois de o do ecrã parar. O editor
(`board_video_bar.rs`): a barra por baixo, o ecrã inteiro (a colocação do vídeo passa a ser a do ecrã inteiro — o
rato e o tamanho seguem-na), os atalhos em `board_video::key`, e a apresentação (`paint_show`: um botão por vídeo
por cima da sala).

**Cena de smoke:** 20 (`PH2D_BOARD_SMOKE=20`, `smoke_w19.rs`, gate `the_w19_scene_does_what_its_lessons_say`).

## §W20 — o pan não pausa, arrastar move, a decoração (13/10)

Decisão: [ADR-0185 §W20](../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md).
Oráculo: [`ferramentas/excalidraw_oracle/sonda_w20.mjs`](ferramentas/excalidraw_oracle/sonda_w20.mjs) (16 sondas, a
página servida como `https://ph2d.local/` — de `http://127.0.0.1` o YouTube recusa: «This video is unavailable»),
saídas em `excalidraw_oracle/saidas/w20/` (`w20_sondas.json`, fotos, `w20_E2_gravado.excalidraw`).

| pergunta | o oráculo | aqui (W20) |
|---|---|---|
| arrastar um vídeo parado, pelo play | move, não activa (A1) | igual |
| clique rápido num seleccionado | activa no LARGAR (logo após o Down: `hover`); 600 ms quieto + largar não activa (A2) | activa no largar sem arrasto (sem prazo) |
| arrastar um seleccionado pelo meio | move (A3) | igual |
| arrastar um ACTIVO, de dentro ou da borda | não move, o clique chega ao YouTube e pausa (A4) | move; o tocador não recebe nada |
| clicar fora | desactiva, **toca** (P_clique_fora) | igual (antes: pausava) |
| pan: meio / roda / mão / espaço no vazio | toca (a mão desactiva) | toca, fica activo |
| espaço+arrastar SOBRE o activo | não faz pan, **pausa** (defeito) | faz pan, toca |
| meio fora · 150 px · todo fora · de volta | toca sempre | toca · toca · **pausa** (o dono) · retoma |
| decoração de um `embeddable` | os grupos do rectângulo; a borda no contorno, o iframe recuado pela largura do traço, o fundo só por trás (E1, E2) | igual, com a moldura das formas (à mão se o rascunho) |

**Os três caminhos do «pan pausa»** (os três vermelhos em `a_pan_never_reaches_the_video`,
`clicking_outside_takes_the_mouse_not_the_picture`, `a_video_plays_until_it_is_wholly_out_of_view`): (a) o carregar do
pan passava pelo `board_video::pointer_down`, que desactivava — e desactivar pausava; (b) com o vídeo activo o
Espaço era o atalho tocar/pausar e o quadro nunca recebia o pan; (c) a regra «≥ 200 px VISÍVEIS» pausava o vídeo
levado até à beira.

**O mecanismo.** `Videos::player` (o tocador) ≠ `Videos::active` (o rato e as teclas); `live_allowed` lê o
tamanho do tocador e se há algum pedaço à vista. `documents::VideoClick` guarda o carregar; `pointer_move` mede o
limiar (`board_view::metrics().drag`) e devolve o mover ao editor; `pointer_up` decide. `space_tap` arma no
carregar do Espaço e qualquer carregar do rato desarma-o. No desenho, `player_in_box` dá o recuo (o alcance da
moldura, `image_frame::reach`) e o afim do tocador normalizado à caixa — a colocação (`Placement::to_screen`) é a
do tocador, por isso o ponto do clique, o play e a barra seguem-no sem mudar; a moldura no rascunho nunca preenche
(lê o estilo do elemento).

**Pausado, o ▶ não chegava ao quadro** (o dono, depois do smoke da W20). Sonda
[`ferramentas/youtube_motor/sonda_w20_pausa.mjs`](ferramentas/youtube_motor/sonda_w20_pausa.mjs) (fotos directas da aba,
`saidas/w20_pausa/` e `saidas/w20_pausa_montagem_ph2d/` — a nossa montagem: contexto por omissão, 2.ª aba):

| depois da pausa confirmada | screencast | foto directa (`captureScreenshot`) |
|---|---|---|
| Playwright, contexto novo | 1 quadro e silêncio | ▶ |
| Playwright, a NOSSA montagem | 1 quadro e silêncio | ▶ |
| o nosso `Player` | 0–1 quadros (a aba visível, com foco, `rAF` a 60/s) | — |
| o nosso `Player` + foco emulado / + flags anti-«background» | 0–1 (recusadas) | — |
| o nosso `Player` + as fotos +150/+500/+1200 ms (a cura) | 12–15 quadros em 2 s, ▶ à vista | ✓ |

**«Mais vídeos» na mesma janela** (`sonda_w20_mais_videos.mjs`, `saidas/w20_mais_videos/`): o botão abre um painel no
tocador; a sugestão é um `<a target=_blank href=youtube.com/watch?v=…&feature=endscreen>` — aba nova, e o de origem a
tocar. O app (`video::outside`): outro vídeo → `board_video::navigate` (a mesma janela, um passo de desfazer, o
trilho `documents::VideoTrail`); o mesmo vídeo → o navegador + pausa; o resto → o navegador. `video::tune` recarrega
o tocador quando a MESMA janela muda de vídeo (a sugestão, ‹ ›, «voltar ao original», o desfazer). As setas e o
«voltar ao original» são `Ctl::{Back, Forward, Reset}` (no fim dos ids), só com navegação. ⚠️ A linha da tabela §W19
que diz que os parâmetros tiram o «Mais vídeos» envelheceu: em 13/10 ele aparece.

**Cena de smoke:** 21 (`PH2D_BOARD_SMOKE=21`, `smoke_w20.rs`, gate `the_w20_scene_does_what_its_lessons_say`). As
lições da cena 19 (l1: «clique fora para parar») e da 20 (l2: «enquanto 200×200 à vista») ensinavam o contrário e
foram corrigidas.

## §W21 — segurar parado não é clique; com outra ferramenta o clique é do vídeo (13/10)

Decisão: [ADR-0185 §W21](../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md).
Briefing e decisões do dono: [`HANDOFF_CONTINUACAO_W21_…`](handoffs/HANDOFF_CONTINUACAO_W21_VIDEO_CLIQUE_FERRAMENTAS_WINDOWS_2026-10-13.md).
Oráculo: [`ferramentas/excalidraw_oracle/sonda_w21.mjs`](ferramentas/excalidraw_oracle/sonda_w21.mjs) (Excalidraw 0.18.1,
Playwright, YouTube real, a página como `https://ph2d.local/`; 24 sondas em `saidas/w21/w21_sondas.json`; ficaram só as
fotos de rectangle/freedraw/text/arrow — o JSON ainda nomeia as podadas). A duração do segurar conta-se no relógio da
PÁGINA (`event.timeStamp` de pointerdown→pointerup), não no `waitForTimeout` do Node: a máquina estava a `load` 10–130.

**O que o oráculo faz**

| pergunta | o oráculo (medido) |
|---|---|
| vídeo NÃO seleccionado, clique de 68 ms no meio / 607,6 ms | activa / não activa (T0) |
| vídeo seleccionado, segurar parado no meio, 13 tempos a subir e a descer | activa a 82 · 153 · 203 · 253 · 283,5 ms; NÃO activa a 303,3 · 324 · 354 · 403 · 455 · 502 · 605 · 807 ms; idêntico nos dois sentidos ⇒ a fronteira está em (283,5; 303,3] ms, «perto de 300 ms» (T1) |
| 11 ferramentas (rectangle, ellipse, diamond, arrow, line, freedraw, text, eraser, laser, hand, frame) sobre o vídeo parado e sobre o activo a tocar | a ferramenta faz SEMPRE o seu por cima e nunca activa o vídeo. Clique sem mexer: arrow/line/freedraw criam um elemento, text abre o editor, rectangle/ellipse/diamond/frame nada (tamanho zero); arrasto: cria o elemento por cima (F) |
| escolher outra ferramenta com o vídeo activo | desactiva-o (`activeEmbeddable` → `null`) mas ELE CONTINUA A TOCAR, mesmo com um rectângulo desenhado por cima (ignora a regra do YouTube de nada à frente do tocador; nós pausávamos, ADR-0185 — *revogado na W21 §W21.4: o vídeo desenha-se à frente*) |
| borracha · mão · laser | APAGA o vídeo (o iframe desaparece e ele pára; a nossa só apaga traços da caneta) · faz pan · não desenha |

**A medida não decidia o item 2.** O oráculo não faz nem (a) «clique = play/pause com qualquer ferramenta, o arrasto é
da ferramenta» nem (b) «sobre o vídeo a ferramenta não começa»: faz o que nós já fazíamos. Perguntado ao dono com as
duas opções e um desenho de cada, escolheu **(a)**.

| pergunta | o oráculo | aqui (W21) |
|---|---|---|
| segurar parado num vídeo e largar | não activa depois de ≈ 300 ms | não activa a partir de `HOLD_NS` = **400 ms** |
| outra ferramenta, clique rápido num vídeo | a ferramenta faz o seu (não activa) | **é do vídeo**: activa/selecciona, activo e vivo vai ao tocador, o que o tocador já tem dá tocar/pausar |
| outra ferramenta, arrastar a partir do vídeo | desenha por cima | igual: desenha a partir de onde se carregou; o vídeo não se move |
| outra ferramenta, segurar parado e largar | a ferramenta faz o seu clique | igual (o frame põe um frame do tamanho de nascença) |
| laser · mão | não desenha · pan | ficam de fora (o laser não toma o carregar; a mão já saltava o vídeo) |

**1. O clique longo** (`b4c03a4e8a`). O largar sem arrasto só é clique se `now_ns − at_ns < HOLD_NS`, com
`HOLD_NS = crate::interaction::LONG_PRESS_THRESHOLD_NS` (400 ms): o «segurar» do rato da casa, o long-press da
Hierarchy que o dono escolheu em 26/05. A casa tem também `DOUBLE_CLICK_WINDOW_NS` (350 ms) e o `LONG_PRESS_NS` do
toque (500 ms). **Uma lei da casa vence uma constante nova**: o mesmo segurar em todo o app; o oráculo (≈ 300 ms)
fica registado, e a diferença é que segurar 300–400 ms ainda é clique aqui. `documents::VideoClick` ganhou `at_ns` (o
`board_view::Input::now_ns` do carregar, nunca `Instant::now()`); `board_nav::pointer_{down,up}` e
`board_video::pointer_{down,up}` recebem o `now_ns`. Parado: segurar no play não activa; activo e vivo: segurar dentro
não chega ao tocador; desactivado com o tocador a segurá-lo: segurar não dá `TogglePlay`.

**2. Outra ferramenta.** Em `board_video::pointer_down`, `other = !select && tool != Laser`. Com `other` e o botão
principal num ponto do tocador de um vídeo (parado: qualquer ponto do tocador, não só o play; activo e vivo: o ponto
do tocador), guarda um `VideoClick { tool: true }` e CONSOME o carregar — o editor não o vê. Três saídas:
- **clique rápido** (< `HOLD_NS`, sem passar `metrics().drag`): é do vídeo (`click`); o largar é consumido;
- **vira arrasto**: `board_video::deferred_drag` devolve o ponto onde carregou e `board_view::pointer_move`
  reentrega-o ao editor por `board_view::canvas_down` (o Down do quadro, extraído de `pointer` para isto) — a
  ferramenta desenha a partir de onde se carregou;
- **largado parado depois de `HOLD_NS`**: `board_video::deferred_hold` faz o mesmo no `Up` e a ferramenta faz o seu
  clique.

**Gates** (`crates/ph2d-editor-core/src/screens/hero/board_video_tests.rs`):
- `a_long_still_press_on_a_video_is_not_a_click` — o caso parado, o activo e vivo, o desactivado; `HOLD_NS − 1`
  activa. Mutação: apagar a condição de tempo sangra («segurou e activou»). Mutar só o VALOR (10 s) sobrevive e é
  equivalente (o teste segura `HOLD_NS`); por isso o teste fixa `HOLD_NS == LONG_PRESS_THRESHOLD_NS`.
- `with_another_tool_a_click_on_a_video_is_the_videos_and_a_drag_is_the_tools` — caneta, rectângulo, nota, seta,
  frame, mapa mental e borracha: o clique activa e nada se desenha; activo, o clique vai ao tocador como Down+Up;
  caneta/rectângulo/seta/frame: arrastar desenha 1 elemento e o vídeo não se move; o frame segurado `HOLD_NS` põe um
  frame e não activa. Mutação `other = false` sangra («o clique não activou»).

**Testes antigos.** Os ajudantes `click` dos testes do vídeo passaram a largar 80 ms depois de carregar
(`hold`/`QUICK_NS`): o `inp` dos testes do quadro anda 1 s por evento, e com 1 s todo clique seria «segurado».
`cargo test -p ph2d-editor-core --lib screens::hero` 444/444 ✓; `cargo test -p ph2d-app-board` 74 ✓ + 7 ignorados.

**Cena de smoke:** continua a 21 (nenhuma cena nova; o roteador mantém `max_level` 21). Lições 8 («Hold the mouse still
on the video for a moment, then let go: nothing happens. Only a quick click plays or pauses») e 9 («Pick the pen (P) or
a shape: a quick click on the video still plays or pauses. Drag to draw over it») — `LESSONS` 7 → 9 (três linhas de
três); o gate `the_w20_scene_does_what_its_lessons_say` conduz as duas com `Mouse::hold` (novo, em
`smoke_w16_tests.rs`). Commit `d14f8c5276`.

## §W21.3 — o tocador no Windows

O mecanismo, as medidas e o que falta medir estão no
[ADR-0185 §W21 item 4](../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md):
o CDP por dois handles herdados (`--remote-debugging-io-pipes=<leitura>,<escrita>`; a porta TCP continua recusada),
lançado pela crate de FFI `ph2d-win-pipe-spawn` (`PROC_THREAD_ATTRIBUTE_HANDLE_LIST` + um job
`KILL_ON_JOB_CLOSE`). Sob o Wine 11.19 com o Chrome for Testing 155 win64 os testes reais do tocador passam
(30,6 q/s, clique → quadro 28,7 ms). **Por medir num Windows real**: o Edge, o sandbox ligado (no Wine só corre com
`--no-sandbox`, que existe SÓ nos testes) e os órfãos do fecho (2 em 11 no Wine). O recuo «abrir no navegador» fica
para quando não há navegador.

## §W21.4 — depois do smoke do dono (13/10)

Commit `0cb5eede4f`. Decisão: [ADR-0185 §W21 itens 3 e 5](../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md).
O dono reportou três coisas: (a) «nem sempre o primeiro clique toca o vídeo; às vezes preciso clicar muitas vezes»;
(b) «o clique com a caneta toca o vídeo» — perguntado, «está certo assim» (é a opção (a) do item 2: fica);
(c) «se notas, traços ou shapes sobrepõem parcialmente o vídeo, ele não toca. Corrija».

**(c) O vídeo desenha-se à frente.** Oráculo (`saidas/w21/w21_F_rectangle_activo_03_arrasto_largou.png`,
`w21_F_freedraw_activo_03…`, `w21_F_rectangle_parado_04…`): no Excalidraw o rectângulo e o traço desenhados por cima
de um vídeo — a tocar ou parado — ficam ESCONDIDOS atrás dele (o iframe vive numa camada acima do canvas). Isso
decide sem perguntar ao dono e cumpre a regra do YouTube (nada à frente do tocador). Antes, qualquer elemento acima no
z cuja CAIXA ENVOLVENTE tocasse a do vídeo pausava-o (até um traço cuja caixa o tocava sem o tocar). Cura:
`BoardDoc::live_in_paint_order` (ordem de z com os vídeos por cima do resto; entre vídeos, o z). Quem a lê: o desenho
(`ph2d-board-render::paint`, o laço dos elementos; os recortes dos frames reentram por elemento, por isso vale com
diapositivos), o «o de cima» do clique (`Editor::hit`: o clique na parte sobreposta apanha o vídeo) e a exportação
(o que se exporta é o que se vê). `covered` conta SÓ outro vídeo mais acima. Aviso `board.video.covered`: «Another
video covers this one: YouTube only plays it uncovered — move one of them».

**(a) O clique durante o arranque.** Medido no navegador real (Chrome 155, teste `#[ignore]`
`a_click_while_the_player_starts_never_stops_its_autoplay`, variantes no mesmo processo): sem clique o tocador vai
Unstarted → Buffering ~1,2–1,5 s → Playing ~1,5–1,9 s (máquina calma). Um clique a 0,3 s e a 1 s (Unstarted) não faz
nada; a 2 s/3 s (já Playing) pausa — correcto; e **um clique no instante em que passa a `Buffering` devolve-o a
`Unstarted` e ele NUNCA chega a tocar** (8,5 s observados): o «preciso clicar muitas vezes». E durante o arranque o
quadro tirava o play e mostrava a miniatura parada, o que convidava a clicar de novo. Cura: `Playback::started`
(true em Playing/Paused/Ended/Cued; o `load` repõe) e `Player::mouse` só manda o rato à página depois dele (antes, o
vídeo toca sozinho, `autoplay=1`); `PlayerStatus::starting = !started` e o desenho põe «a carregar» (disco `BgScrim`
com um arco de ¾ que gira com os desenhos do quadro, `loading_mark`) no lugar do play. RED antes da cura:
`[(Buffering, true, Unstarted)]`; verde depois: as três variantes acabam em Playing.

| gate | o que prova |
|---|---|
| `a_video_draws_over_what_overlaps_it_and_only_another_video_covers_it` (render) | uma nota acima no z pinta-se antes do quadro do tocador e ele fica vivo sem play; outro vídeo por cima tapa-o |
| `shapes_over_part_of_a_video_never_stop_it_and_the_video_takes_the_click` (editor, despacho real) | rectângulo + traço da caneta sobre parte do vídeo: o clique no play toca-o; o clique na parte sobreposta selecciona o vídeo |
| `a_starting_player_shows_it_is_loading` (render) | o «a carregar» no lugar do play |
| `a_click_while_the_player_starts_never_stops_its_autoplay` (tocador real, `#[ignore]`) | as 3 variantes acabam em Playing |
| gate da cena 18 · gate da cena 21 | lição 2 da cena 18: «Put a note over part of the video: the video stays in front and keeps playing. Rotate it: it pauses (YouTube's rule).» (prova as duas metades; antes ensinava o contrário, `tapado não está vivo`); lição 9 da cena 21: «… Drag across it: the video stays in front and keeps playing» (o gate verifica que o vídeo continua vivo com o traço) |

Mutação (desligar a ordenação + `covered` antigo): os dois gates do (c) sangram («nota por cima = true: o tocador
parou», «tapado em parte e não toca»). Testes depois da volta: `ph2d-board-model` 124 · `ph2d-board-render` 54 (+2
ignorados) · `ph2d-board-edit` 218 · `ph2d-board-video` 7 + 9 ignorados (16/16 com `--include-ignored`, release) ·
`ph2d-app-board` 74 + 7 ignorados · `ph2d-editor-core --lib screens::hero` 445 · os 18 testes do vídeo ✓.

**Aberto.** (i) O limiar de arrasto de um clique é o da casa (`TAB_DRAG_THRESHOLD_PX`); se o dono ainda vir cliques que
não tocam com o vídeo já carregado, a próxima suspeita é um tremor da mão acima dele (não medido). (ii) O anel «a
carregar» gira com os desenhos do quadro (`RenderCache::frame`): se o quadro não redesenhar durante o arranque, fica
parado (não medido no app).
