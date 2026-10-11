---
name: reference_rebelle_trial_installed_under_wine
description: Rebelle 8.3.4 (teste de 30 dias, Windows) instalado sob Wine 11 em ~/Apps/rebelle em 2026-10-07; abre em X11; a EULA do teste é só uso pessoal não-comercial
metadata:
  type: reference
---

Instalado em 2026-10-07 a pedido do dono, de `~/Downloads/Rebelle_8_64bit_v8.3.4_Windows.exe`
(Inno Setup 6, `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-`), num prefixo próprio:
`WINEPREFIX=~/Apps/rebelle/wineprefix`. Lançador `~/Apps/rebelle/rebelle.sh`; o próprio instalador
criou a entrada de menu do Wine (`~/.local/share/applications/wine/Programs/Rebelle 8/`) e o atalho
`~/Desktop/Rebelle 8.desktop`. ~1,3 GB.

- Medido numa sessão `kwin_wayland --virtual --xwayland` (receita de
  [[feedback_spectacle_in_a_virtual_kwin_photographs_the_owners_real_screen]]): pelo X11 abre a
  janela «Choose Language» e depois «Registration» (EULA) em < 30 s. A tela de pintura NÃO foi vista:
  a EULA tem de ser aceite pelo DONO (não se aceita por ele) e o XTest não clica na Xwayland virtual.
- Primeira execução: `settings.ini` em `drive_c/users/enio/AppData/Local/Escape Motions/Rebelle 8/Data/`
  (`[General] language=en` salta o diálogo de idioma — usado só no teste e desfeito).
- ⛔ EULA do teste: *«personal, non-commercial internal use»*, período de teste limitado, e contornar
  a expiração termina a licença. Serve de ORÁCULO que se corre (CLAUDE.md §0.9), nunca de fonte;
  para a aquarela ver [[project_rebecca_watercolor_cleanroom]].
- ✅ **A porta sem ecrã, MEDIDA em 2026-10-07** (o dono já aceitara a EULA; o «Try» → «Rebelle 8
  Pro» é só escolher o teste): `~/Apps/rebelle/oraculo/` — `inicia.sh` (kwin virtual 1920×1080,
  prazo 3 h, X11 na Xwayland), `inj.sh` (Python *embeddable* de Windows em `~/Apps/rebelle/py`
  a mandar `SendInput` DENTRO do Wine; `rect <título>` lê a posição de uma janela), `foto.sh`
  (`import -window`), `regua.py` (média 8×3 por traço). ⚠️ O botão premido tem de ir num evento
  SEPARADO do movimento, senão o Rebelle não pinta; a foto da janela principal = ecrã + 28 px.
  Resultado: [`docs/Painter/47_o_papel_no_rebelle_estado_da_arte.md`](../docs/Painter/47_o_papel_no_rebelle_estado_da_arte.md)
  — o papel não é camada, a tinta guarda o alfa, compõe-se por «cobrir com transparência», a ordem
  não importa. Fechar: matar só o `kwin_wayland … oraculo/sessao.sh` e `wineserver -k` NESTE prefixo.

- ⛔⛔ **A injecção de rato (`SendInput`) REGREDIU em 2026-10-07 à tarde:** clique pinta um ponto, ARRASTO não
  pinta (5 variantes de evento, Wine limpo com `wineserver -k`, `use_ink` on/off, documento novo ou de
  arranque — nada; só a ferramenta Inks pintou uma vez). Causa não encontrada; não gaste outra sessão nisso.
- ✅ **A porta que SERVE é a oficial: WebSocket Control (Pro).** `rebelle.sh -websocket-server-enable
  -websocket-port 8265 -websocket-allowed-ip-addresses 127.0.0.1,::ffff:127.0.0.1,::1` (sem o `::ffff:` o
  servidor recusa o localhost IPv4-mapeado). Eventos JSON Motion IO
  (`POINTER_PRESS/MOVE/RELEASE` com `pressure`, `pen_tilt`, `rotation`; `SET_BRUSH` com `tool`, `preset`
  "Grupo/Nome", `size_px`, `water`, `opacity`, `paint_type`, `dirty_brush`, `color`; `NEW_ARTWORK` com papel;
  `SET_RANDOM_SEED`; `SIMULATION`; `DRY`), **um evento por mensagem** (o documento `{"frames":…}` inteiro é
  ignorado em silêncio); `SAVE` ainda não existe por WebSocket (8.3.4) ⇒ fotografa-se a janela a zoom 100 %
  (campo do Navigator) e recorta-se a tela. ⚠️ `SET_BRUSH` só vale para o 1.º traço: reenvia-se COMPLETO
  antes de cada gesto. Scripts: `~/Apps/rebelle/oraculo/{arranque.sh,gera_eventos.py,ws_eventos.mjs}`
  (cópias em `docs/Painter/NewFeaturesWetPaint/ferramentas/`); o Node 24 traz `WebSocket` global.
  Referência pública: https://www.escapemotions.com/products/rebelle/motionio_doc/reference/json_events_reference
- Capturar um popup do Rebelle (não está em `_NET_CLIENT_LIST`): `python3 xtree.py :1 223x252` dá o id X
  (libX11 por ctypes; não há `xwininfo`), depois `import -window <id>`. ⛔ `import -window root` e
  `ffmpeg x11grab` dão PRETO (Xwayland rootless) e um `import` sem id fica à espera de um clique e PRENDE o
  rato virtual — mate-o por `pgrep -x import` (nunca `pkill -f` com o padrão na própria linha).
- ⛔ **O PAPEL pela porta WebSocket não serve de barra (medido 09/10, W1e):** sob o Wine o papel não se
  desenha (branco, desvio 0, até no RH00 de arranque); `SET_PAPER` derruba o 8.3.4 («bad optional
  access»); um `SET_ENGINE_PARAMS` (texture_influence…) depois de um `SET_BRUSH` reduz o pincel a um fio
  (o Flat Dry saiu com 12 px numa faixa de 30 — a «fração de papel branco» lia a faixa vazia). A MISTURA
  (cores sobre papel liso) mede-se bem: `docs/Painter/NewFeaturesWetPaint/ferramentas/mistura_papel*.{py,sh}`
  com foto DURANTE o traço (o RELEASE num ficheiro à parte). A tela virtual fecha-se sozinha em 1 h.
