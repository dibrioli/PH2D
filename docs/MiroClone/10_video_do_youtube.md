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
nada à frente do leitor, ≥ 200×200, Referer ⇒ **vivo só destapado, sem rotação, inteiro à vista, ≥ 200 px**.

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
| o que `controls=0&fs=0&disablekb=1&rel=0&iv_load_policy=3` tira | some: a barra, o tempo, volume, legendas, definições, ecrã inteiro, «Mais vídeos»; FICAM: título, canal, «Copiar link», logotipo «YouTube», play ao meio | `w19/P_*_rato.png` |
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

**Cena de smoke:** 21 (`PH2D_BOARD_SMOKE=21`, `smoke_w20.rs`, gate `the_w20_scene_does_what_its_lessons_say`). As
lições da cena 19 (l1: «clique fora para parar») e da 20 (l2: «enquanto 200×200 à vista») ensinavam o contrário e
foram corrigidas.
