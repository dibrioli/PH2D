# ADR-0185 — O vídeo do YouTube no quadro toca no navegador da PESSOA, invisível, projectado no canvas

- **Status:** Accepted — decisão do DONO (12/10, *«Sim, janela virtual»*, depois de ver as medições; a ideia foi
  dele: *«uma alternativa mais inteligente que não exige instalar nada… uma janela virtual do próprio navegador
  que a pessoa tem instalado, projectada no quadro»*); o desenho técnico é delegado.
- **Data:** 2026-10-12
- **Linha:** `line/MiroClone` (W18, [continuação](../../MiroClone/handoffs/HANDOFF_CONTINUACAO_W18_ABERTOS_E_VIDEO_YOUTUBE_2026-10-12.md))
- **Amparo:** as medições [`youtube_motor/README.md`](../../MiroClone/ferramentas/youtube_motor/README.md) ·
  o oráculo Excalidraw 0.18.1 corrido ([`excalidraw_oracle/README.md`](../../MiroClone/ferramentas/excalidraw_oracle/README.md)
  §W18) · [ADR-0184](0184-a-imagem-do-quadro-e-um-ficheiro-por-conteudo-que-viaja-sozinho.md) (a imagem, o
  grupo de fios) · [ADR-0181](0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md) (ao vivo).

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 12/10 contra o máximo do `main` (`0181`) e das linhas (`0184`, esta).
> Quem integrar reconta-o.

## Contexto — a força que obriga a decidir agora

Ordem do dono (12/10): *«quadro youtube (através de um link o quadro exibe um vídeo youtube como o navegador)»* —
tocar ALI, com som, pausa, avançar e ecrã inteiro; ⚠️ *não* é «abrir no navegador». O PH2D é Rust + Vello + winit
0.30 no KDE **Wayland** do dono, sem motor web. O tocador do YouTube só existe como página web (o embed
`/embed/<id>`); tirar o fluxo do vídeo e descodificá-lo nós violaria os termos. Logo: um motor web tem de correr
algures e os píxeis dele têm de chegar ao canvas. O Excalidraw resolve-o com um `<iframe>` vivo por cima do canvas
(medido §W18) — a forma que o PH2D terá na versão de navegador (decisão do dono de 06/10), não aqui.

**As regras do YouTube para o tocador embutido** (*Required Minimum Functionality*,
`developers.google.com/youtube/terms/required-minimum-functionality`): nada à FRENTE de nenhuma parte do tocador
(nem a cobri-lo), a área visível de pelo menos **200×200 px**, e a identificação pelo cabeçalho `Referer`.

## Decisão

**O vídeo é um elemento do quadro cujo tocador oficial corre no navegador da família Chrome que a pessoa JÁ tem
(Chrome, Chromium, Edge, Brave, Vivaldi, Opera), lançado invisível e conduzido pelo protocolo dele (CDP); os
quadros chegam por screencast e desenham-se no canvas como uma imagem; o rato e o teclado vão por
`Input.dispatch*`. Sem um navegador desses, a miniatura com «Abrir no navegador».**

1. **O modelo não sabe de motor** (o PH2D web troca-o por um `<iframe>` sem mudar nada): `ElementKind::Embed` com
   o link canónico, o id do vídeo, o início em segundos e a proporção; viaja ao vivo como o LINK (cada par carrega
   o seu tocador; o estado de reprodução NÃO se sincroniza).
2. **O motor está atrás de uma porta** (trait) numa crate própria; a primeira implementação é o navegador do
   sistema por `--remote-debugging-pipe` (nunca uma porta TCP: qualquer processo local mandaria no navegador),
   com perfil temporário, sem ecrã, sem o `--mute-audio`, e morre com o PH2D (`PR_SET_PDEATHSIG` no Linux).
   Arranca na 1.ª activação (não no arranque do PH2D) e fecha-se quando nenhum vídeo toca há um minuto.
3. **UM tocador vivo de cada vez** — o vídeo ACTIVADO (o idioma do oráculo: um clique activa, clicar fora
   desactiva). Os outros mostram a miniatura com o símbolo de play.
4. **Vivo só quando as regras se cumprem**, uma função com gate (`live_allowed`): descoberto (nenhum elemento por
   cima do retângulo dele), sem rotação e ≥ 200×200 px VISÍVEIS (W19: era «inteiro à vista», a regra era nossa e
   pausava o vídeo a cada zoom — §W19). Ao falhar qualquer uma, o
   desenho volta à miniatura (com o quadro em que parou) no mesmo quadro em que deixou de cumprir.
5. **A miniatura SEMPRE no canvas**: export, pan, zoom, rotação, sem rede. Busca-se fora do fio da janela, com
   prazo (o molde do `clipboard.rs`), entra na cache do quadro; sem rede, o lugar com o título/link e uma frase.
6. **A lei de reconhecer um link é UMA função com gate** sobre a tabela medida no oráculo (24 formas, §W18), com
   os defeitos do oráculo corrigidos (o `t=1m30s` em segundos, `/live/`, `?feature=share&v=`, o domínio em
   maiúsculas, o id de 11 caracteres).

## Alternativas REJEITADAS (com a medida)

| alternativa | porquê não (medido em 12/10, `youtube_motor/saidas/`) |
|---|---|
| **(i) webview nativa como janela-FILHA** por cima do retângulo (`wry`) | o `wry` só aceita janela-filha em **X11** (docs do `wry` 0.46–0.57: *«Linux (X11 Only)»*; para Wayland manda usar GTK); no Wayland um cliente não posiciona janelas; o XWayland foi RECUSADO na W14. Mesmo em X11, uma janela por cima não roda nem fica por baixo das notas |
| **WebKitGTK 4.1** (instalado) fora do ecrã | com aceleração: **0 quadros** (`Failed to create GBM buffer`); sem: o tocador abre e **fica parado em 0:00** — faltam `matroskademux`/`vpx` (o `gst-plugins-good`), e continua parado MESMO com eles ao lado (`GST_PLUGIN_PATH`, com e sem a caixa de areia). Só Linux. E o `GtkOffscreenWindow` é o caminho que o próprio GTK abandonou (sai no GTK4) |
| **Servo 0.7 embutido** | toca: **29,9 q/s**, 0,09 núcleo, 646 MB, build release frio 172 s. MAS sem MSE ⇒ o YouTube cai no MP4 de **360p**; a API de embutir **quebra todo mês** (*«not a 1.0»*); pede GStreamer também no Windows/macOS; o Verso (o webview do Tauri sobre o Servo) foi **arquivado** em 10/2025. ⇒ plano B quando houver MSE (NLnet «Servo Multimedia», 06/2026) |
| **CEF embutido** (o motor do Chrome dentro do PH2D) | toca (o mesmo motor da decisão), mas **+364 MB** instalados / 116 MB baixados por build; o `accelerated_osr` do crate `cef` 154 pede `wgpu` **30** (o nosso é 29, preso pelo `vello`); o dono preferiu zero a instalar |
| **(iii) tirar o fluxo do vídeo** (`yt-dlp` + descodificar) | ⛔ viola os termos do YouTube |
| **o tocador vivo por baixo das notas / rodado** | ⛔ as regras do YouTube proíbem cobrir qualquer parte do tocador e modificar a aparência dele ⇒ miniatura |

## O PREÇO

- **Depende do navegador da pessoa.** Sem um da família Chrome (só Firefox ou Safari), o vídeo **não toca dentro
  do quadro** — miniatura e «Abrir no navegador». No Windows o Edge está sempre lá; no Linux e no Mac, não.
- **Memória do navegador**: ~1,3 GB enquanto um vídeo está activo (a árvore do Chrome, medido), devolvida ao
  fechar.
- **O `Page.startScreencast` é marcado EXPERIMENTAL** no CDP: uma versão do Chrome pode mudá-lo; o gate da porta
  corre contra o navegador real (`#[ignore]`, à mão) e o fallback é a miniatura.
- **Um tocador vivo de cada vez**; o estado de reprodução não viaja ao vivo.
- **Os quadros chegam em JPEG** (o que o screencast dá): uma descodificação por quadro (~30/s a 560×315) no fio da
  porta, nunca no da janela.
- No iPad (quando houver) só o WKWebView da Apple, por cima — outra implementação da porta.

## Critério de morte (antes do build)

Se, depois da 2.ª tentativa, no KWin do dono: o vídeo activo não mantiver **≥ 24 quadros/s** a 560×315, ou o
clique não chegar ao tocador em **≤ 100 ms**, ou o PH2D gastar mais de **0,5 núcleo** a desenhá-lo — a forma
falha e a decisão volta ao dono (o Servo é o plano B).

## O que fica gateado (para ninguém re-litigar por prosa)

- `ph2d-board-embed` (crate da lei): `youtube::parse` contra a tabela das 24 formas do oráculo (+ as correcções);
  `live_allowed` (descoberto · sem rotação · ≥ 200×200 VISÍVEIS — §W19).
- O `.excalidraw`: ler e escrever o `embeddable` com `link` (o contador do aviso X3 desce).
- `deny.toml` `[bans]` (o `cargo deny` do `ship.sh`): as crates de extrair fluxos do YouTube (`rustube`,
  `rusty_ytdl`, `ytextract`, `youtube_dl`, `yt-dlp`, `ytdlp`) e os motores web recusados (`wry`, `cef`,
  `cef-dll-sys`, `servo`, `webkit2gtk`) — nomes reais conferidos por `cargo search` (12/10). Reabrir um deles é
  tirar a linha COM um ADR novo.

## §W19 — emenda: os NOSSOS controles (12/10, depois do smoke do dono)

**Os três defeitos do smoke, cada um reproduzido VERMELHO antes da cura** (mecanismo e números:
[`10_video_do_youtube.md`](../../MiroClone/10_video_do_youtube.md) §W19; sonda
[`sonda_w19_tocador.mjs`](../../MiroClone/ferramentas/youtube_motor/sonda_w19_tocador.mjs)):

- **O ecrã inteiro DO TOCADOR deformava.** O navegador sem ecrã imagina um ecrã 800×600; o ecrã inteiro manda
  quadros 4:3 (420×315 numa caixa 560×315, teste real) e o desenho esticava-os. Cura em três camadas: o «ecrã»
  da aba é a caixa (`screenWidth/Height`), o tocador vem sem ecrã inteiro (`fs=0`, sem a permissão `fullscreen`
  no `<iframe>`), e o desenho NUNCA estica (faixas, como o tocador).
- **O zoom pausava.** A sonda mudou o tamanho do tocador a cada 16 ms durante 5 s em 8 corridas: **zero pausas**.
  A pausa era a NOSSA regra «inteiro à vista». O YouTube pede ≥ 200×200 px **visíveis**: é essa a lei agora.
  O tocador segue o tamanho do ecrã 250 ms depois de ele assentar (medido sem perda de quadros).
- **Clicar no canal deixava preto.** Os links do tocador (canal, título, «YouTube») abrem uma ABA NOVA no navegador
  invisível, que fica à frente: o nosso screencast parava (1 quadro em 3 s, luz 9,5/255). Cura: a aba nova
  fecha-se no `Target.targetCreated`, a nossa volta à frente, e o link abre-se no navegador do SISTEMA.

**Decisão do dono (12/10): «Nossos controles + Premium».**

1. **O tocador vem sem os controles dele**: `controls=0&fs=0&disablekb=1&rel=0&iv_load_policy=3` (cada um medido
   no oráculo: ficam o título, o canal, «Copiar link», o logotipo «YouTube» e o play ao meio, que as regras do
   YouTube não deixam esconder).
2. **A nossa barra fica POR BAIXO da caixa do vídeo, fora dela** (por cima se em baixo não couber, e some se não
   couber em lado nenhum) — *«must not display overlays … in front of any part of an embedded player»*. Tocar/
   pausar, o tempo, a barra de tempo, o mudo e o volume, o NOSSO ecrã inteiro e «Abrir no YouTube» (no ponto em
   que está). O estado vem da API oficial (`listening` → `infoDelivery`/`onStateChange`, ~4 mensagens/s, passado
   ao PH2D por `Runtime.addBinding`); as ordens por `postMessage`. Medido: `seekTo`/`mute`/`setVolume` chegam ao
   estado em 4–10 ms, `pauseVideo` em ~630 ms — por isso a ordem dada MANDA no botão até o tocador a confirmar
   (≤ 1,5 s).
3. **As teclas são nossas** (o tocador, com `disablekb=1`, ignora-as): os atalhos do YouTube — espaço/K, ←/→ 5 s,
   J/L 10 s, ↑/↓ volume, M, F; `Esc` sai do ecrã inteiro. O encaminhamento de teclas ao tocador saiu.
4. **O nosso ecrã inteiro** enche a área do quadro com a proporção do vídeo e a barra por baixo; as barras do
   quadro não se pintam nem agarram, o rato no preto e a roda não chegam ao quadro.
5. **A roda do rato é do quadro** (o zoom), nunca do tocador — o caminho dela até ao tocador saiu.
6. **Na apresentação o vídeo do diapositivo toca** (era só a miniatura com um play inerte): um clique activa-o,
   os seguintes tocam/pausam, a barra por cima da sala; mudar de diapositivo deixa-o de fora da vista, sair pára-o.
7. **Premium** (sem anúncios pelo caminho legítimo — esconder ou saltar anúncios viola os termos e NÃO se faz):
   a pessoa entra na conta dela. ⚠️ Por medir com a conta do dono: o domínio sem cookies (`youtube-nocookie.com`)
   não leva a sessão de `youtube.com`, por isso uma sessão iniciada só conta com o tocador em `www.youtube.com/embed`
   e um perfil PERSISTENTE do PH2D. **Medido (12/10, `youtube_motor/saidas/w19_premium/`): SEM sessão o tocador
   embutido não mostrou anúncios em 3 vídeos muito monetizados, nos dois domínios (0 de 6, e pelo olho aos 3 s)**
   ⇒ o Premium não teria o que tirar; a parte NÃO se constrói (o perfil persistente com a sessão da pessoa
   custaria privacidade e abriria o risco do item 8 por nada) e a decisão volta ao dono com a medida: se ele vir
   um anúncio num vídeo do quadro, esse vídeo é a entrada da medição seguinte.
8. **Windows** (aberto da W18): ⛔ continua sem tocador (abre no navegador). A porta TCP (`--remote-debugging-port`)
   fica RECUSADA: qualquer processo local mandaria no navegador — e, com o perfil persistente do item 7, na
   sessão do YouTube da pessoa. A rota é o tubo com handles herdados (`--remote-debugging-io-pipes`), que pede
   marcar dois handles como herdáveis; não há Windows nesta bancada para o medir.

**Gateado:** `live_allowed` com `visible` (`ph2d-board-embed`); o desenho com faixas
(`a_frame_of_another_shape_draws_with_bands_never_stretched`); a barra nunca à frente e cada ordem pelo despacho
real (`our_controls_sit_under_the_video_and_each_gives_its_order`, `the_bar_goes_under_or_over_the_video_never_on_it`);
o ecrã inteiro 16:9 (`our_fullscreen_fills_the_board_keeping_the_shape_and_esc_leaves`); a apresentação
(`a_video_on_a_slide_plays_in_the_presentation_with_our_controls`); e no navegador real (`#[ignore]`, à mão):
`the_player_s_fullscreen_corner_never_changes_the_frame_shape`, `clicking_the_channel_keeps_the_player_playing_and_lit`,
`a_continuous_zoom_never_pauses_the_player`, `our_controls_reach_the_player_and_come_back_in_its_state`.
