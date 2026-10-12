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

