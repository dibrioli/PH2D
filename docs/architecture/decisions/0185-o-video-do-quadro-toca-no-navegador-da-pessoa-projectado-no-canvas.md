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
   cima do retângulo dele), sem rotação, ≥ 200×200 px de ecrã e inteiro à vista. Ao falhar qualquer uma, o
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
  `live_allowed` (descoberto · sem rotação · ≥ 200×200 · inteiro à vista).
- O `.excalidraw`: ler e escrever o `embeddable` com `link` (o contador do aviso X3 desce).
- `deny.toml` `[bans]` (o `cargo deny` do `ship.sh`): as crates de extrair fluxos do YouTube (`rustube`,
  `rusty_ytdl`, `ytextract`, `youtube_dl`, `yt-dlp`, `ytdlp`) e os motores web recusados (`wry`, `cef`,
  `cef-dll-sys`, `servo`, `webkit2gtk`) — nomes reais conferidos por `cargo search` (12/10). Reabrir um deles é
  tirar a linha COM um ADR novo.
