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
