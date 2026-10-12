# CONTINUAÇÃO — `line/MiroClone` W19: os defeitos do vídeo no smoke do dono, os NOSSOS controles, o Premium e o que ficou aberto (2026-10-12)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: a W18 FECHOU ([handoff de integração W18](HANDOFF_INTEGRACAO_line_MiroClone_W18_2026-10-12.md),
> mecanismo do vídeo em [`10_video_do_youtube.md`](../10_video_do_youtube.md), decisão [ADR-0185](../../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md)),
> mas o **smoke do dono (12/10) achou defeitos no vídeo** — NÃO é «smoke OK». A linha integra-se junto com W12+W13…W18
> (**não integre, não pushe**: só por ordem do dono).

## O que já existe e NÃO se reconstrói

Tudo o que os handoffs W14–W18 §1 listam — em especial da W18: `ElementKind::Embed` + `Field::Embed` (`PROTOCOL` 7), a
lei `ph2d-board-embed` (gate sobre as 24 sondas do oráculo), `ph2d-board-video` (miniatura HTTPS com cache; o `Player`
no navegador família-Chrome da PESSOA por CDP num tubo lançado por `sh -c 'exec … 3<&0 4>&1'`, sem `unsafe`; página
servida por `Fetch` em `https://ph2d.local/`; pausa/retoma pela API oficial com `postMessage`), o desenho
(`ph2d-board-render::video`), o editor (`board_video.rs`), o app (`ph2d-app-board::video`), o export (`/URI`, `<a href>`),
o largar de links, o ao vivo, a cena 19. As medições dos motores (`ferramentas/youtube_motor/README.md`) e a decisão
do dono «janela virtual» (memória `project_youtube_in_the_board_uses_the_persons_own_browser_2026_10_12`). ⛔ Recusas
medidas de pé: janela-filha (`wry` só X11), WebKitGTK (não toca), Servo embutido (360p, sem MSE), CEF (+364 MB),
tirar o fluxo (`yt-dlp`, termos) — o `deny.toml` bane-os.

## Os defeitos do smoke do dono (12/10), verbatim — UM bloco (§0.10): reproduza cada um com um teste VERMELHO antes de curar

1. **«As setinhas do youtube de esticar o vídeo não mantêm a proporção.»** O dono confirmou: é o botão de **ecrã inteiro
   DO TOCADOR** (as setas no canto de baixo à direita), não as pegas do quadro. Mecanismo provável (confirme): na aba sem
   ecrã o «fullscreen» enche o viewport, que é o `setDeviceMetricsOverride` do tamanho da caixa; o screencast passa a
   vir noutra proporção e o `video::stretch` estica QUALQUER quadro à caixa. Duas curas, as duas: (a) o desenho nunca
   deforma — um quadro de proporção diferente da caixa desenha-se com faixas (letterbox), como o tocador faz; (b) o
   ecrã inteiro passa a ser NOSSO (ver o item 4): o vídeo enche a janela do PH2D mantendo a proporção, e o `fs=0`
   tira o botão do YouTube.
2. **«Às vezes dar zoom pausa o vídeo.»** Mecanismos prováveis, meça os dois: (a) a regra `OnScreen::whole` («inteiro
   à vista») é MINHA, não do YouTube — a *Required Minimum Functionality* pede ≥ 200×200 px VISÍVEIS e nada à frente; um
   vídeo meio fora da vista ao dar zoom não viola nada. Troque `whole` pela área VISÍVEL ≥ 200×200 (gate na
   `ph2d-board-embed`; o `live_allowed` e o enquadrar ao activar mudam com ela). (b) cada passo de zoom manda `Resize` →
   `setDeviceMetricsOverride` + recomeçar o screencast: o tocador refaz o layout e o YouTube pode re-bufferizar/pausar.
   Cura a medir: o tocador NÃO segue o zoom passo a passo — fica num tamanho estável (o do ecrã quando activou, e só
   muda depois de o zoom assentar, com um atraso medido) e o desenho escala a textura. Régua: um zoom contínuo de 1×→2×→1×
   durante 5 s com o vídeo a tocar — zero pausas (leia o estado do tocador pela API, item 4), quadros distintos/s.
3. **«Clicar no ícone do canal deixa o vídeo preto.»** Um link dentro do embed navega a aba (ou abre um alvo novo) e o
   screencast passa a mostrar outra página/nada. Corra o oráculo primeiro (o Chrome sem ecrã pela sonda
   `youtube_motor/sonda_chromium_som_clique.mjs` ampliada: clicar no ícone do canal, no título, no «Watch on YouTube», no
   «Mais vídeos» — registe se nasce um alvo novo (`Target.targetCreated`), se a moldura navega
   (`Page.frameRequestedNavigation`/`frameNavigated`) e o que se pinta). Cura: nenhuma navegação sai do embed — um alvo
   novo fecha-se e o URL abre-se no navegador do SISTEMA (`ph2d_board_video::open_in_browser`); uma navegação da moldura
   cancela-se e abre-se fora; o tocador continua onde estava. Com o item 4 a maior parte destes links desaparece, mas a
   guarda fica (gate: clicar o canal não deixa o tocador preto).
4. **Decisão do dono (12/10): «Nossos controles + Premium».**
   - **Os NOSSOS controles:** o embed com `controls=0`, `fs=0`, `disablekb=1`, `rel=0`, `iv_load_policy=3` (parâmetros
     oficiais — confirme cada um no oráculo: o que some, o que fica); o estado (`currentTime`, `duration`,
     `playerState`, volume/mudo) lê-se pela API oficial (`postMessage` com `{"event":"listening"}` → `infoDelivery` /
     `onStateChange`), trazido do iframe pela página nossa por `Runtime.addBinding` (sem polling do fio da janela); as
     ordens (`playVideo`, `pauseVideo`, `seekTo`, `mute`/`unMute`, `setVolume`) pelo mesmo `Cmd::Command` da W18.
     ⛔ **As regras do YouTube proíbem QUALQUER coisa à frente do tocador** (*«must not display overlays … in front of
     any part of an embedded player»*) ⇒ a NOSSA barra (play/pausa, barra de tempo com o tempo, volume, ecrã inteiro
     NOSSO, «Abrir no YouTube») desenha-se **por baixo da caixa do vídeo, fora dela** — nunca por cima. UI só tokens +
     `ph2d-i18n` (HR-15); widgets pelos 7 sítios da costura (DIRETIVA §2) com teste de costura pelo despacho real.
     Emende o ADR-0185 (secção nova, não um ADR novo) com isto e com o porquê.
   - **O Premium (sem anúncios, pelo caminho legítimo):** ⛔ esconder/saltar anúncios viola os termos — NÃO se faz. O
     legítimo é a PESSOA entrar na conta dela (Premium = sem anúncios). Investigue e MEÇA, por esta ordem: (a) um perfil
     PERSISTENTE do PH2D para o navegador (`~/.local/share/ph2d/browser-profile`, em vez do temporário) onde a pessoa
     entra no YouTube UMA vez — lançado COM janela («Entrar no YouTube…» abre o mesmo perfil visível; fechada, o sem
     ecrã reusa-o); o embed com sessão iniciada mostra sem anúncios para Premium? (b) ⛔ copiar cookies do perfil real da
     pessoa: frágil e sensível (o perfil do Chrome tranca-se; cookies cifrados pelo keyring) — só como recusa medida.
     Privacidade: o perfil persistente nunca viaja nem entra no projecto; diga ao dono onde fica e como se apaga.

## O que ficou aberto da W18 (o handoff §4) — no mesmo bloco onde couber

- (a) **Windows**: o tubo do CDP vai por `sh` — no Windows não há tocador (abre no navegador). Rota a medir:
  `--remote-debugging-pipe` com handles herdados (a crate std não os dá sem `unsafe`) vs `--remote-debugging-port` com
  porta aleatória só em `127.0.0.1` + o `DevToolsActivePort` (a porta TCP local é o risco que o tubo evitava — meça e
  decida no ADR). macOS: tem `sh`, nunca foi corrido.
- (b) A **roda do rato** não vai ao tocador (o `PlayerInput::Wheel` existe e ninguém o manda).
- (c) Na **apresentação** o vídeo mostra a miniatura com o play (não toca lá). Decida com os nossos controles.
- (d) O estado de reprodução não se sincroniza ao vivo — POR DESENHO (ADR-0185); não mude sem o dono.
- (e) Sem rede: o lugar só com o play + o aviso; nenhum título na tela.
- (f) Arrastar um link do navegador não mostra pré-visualização (só o largar).
- (g) Os limites do texto dos SVG: [`09_texto_dos_svg.md`](../09_texto_dos_svg.md) §6.
- (h) `~/Apps/w18_motores` (~190 MB, fora do repo) pode apagar-se.
- (i) O número do ADR-0185 é provisório (o integrador reconta).
- (j) Da W17 (f): o gate M5 da costura da shell lê o FONTE.
- Não corridos no portão da W18: `cargo audit`, `check-standalone-optional.sh`, `check-workflow-packages.sh`.

## Critérios de morte (antes do build)

Os nossos controles respondem (clique → quadro com o estado novo) em ≤ 100 ms; um zoom contínuo de 5 s com o vídeo a
tocar dá zero pausas; o ecrã inteiro nosso mantém a proporção (gate de píxeis: as faixas). Se o Premium não ficar sem
anúncios com a sessão iniciada no perfil persistente depois da 2.ª tentativa, a parte Premium sai desta onda e a
decisão volta ao dono com a medida.

## Fecho

Portão batched 1× (`nextest-impacted` com `BASE=merge-base`, `CARGO_BUILD_WARNINGS=deny cargo check --workspace
--all-targets`, clippy do `ship.sh`, `fmt`, `typos`, `machete`, `deny`, `file_loc_caps`, censos), auditoria de 2 lentes,
mutação nas leis novas (⚠️ a mutação NUNCA ao mesmo tempo que o portão: ela muta o fonte que o portão compila — a W18
teve de a parar). A régua do navegador real (`the_person_s_browser_plays_at_24_fps_and_answers_a_click`) corre à mão e
continua verde. Smoke: a cena 19 actualizada (ou a 20, contada no roteador `smoke.rs`, hoje `max_level: 19`) com os
nossos controles por baixo do vídeo, o zoom que não pausa, o canal que abre fora e o ecrã inteiro com proporção —
lições em notas que CABEM (fotografe com `no_app.sh` e LEIA a foto; a W18 cortou a lição 1 da cena 18). Handoff de
integração W19 (≤ 15 KB; o mecanismo no `10_video_do_youtube.md`), binário `smoke` compilado 2×, `rm -rf
target/*/incremental`, a máquina limpa (nenhum navegador sem ecrã, nenhum `/tmp/ph2d-cdp-*`). Relatório ao dono curto
e sem jargão, smoke em passos numerados.
