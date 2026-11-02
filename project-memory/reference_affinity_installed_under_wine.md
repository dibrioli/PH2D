---
name: reference-affinity-installed-under-wine
description: "Affinity 3.3.1 (Canva) REINSTALADO 26/10 pela AppImage do ryzendew (Wine 11.19 + patch set Affinity); cura das fontes «\\??\\unix» que derrubam o WPF; abaixo, a receita antiga do Wine do sistema (11/10)"
metadata:
  node_type: memory
  type: reference
  originSessionId: 0f78f672-4fca-4b87-b9fa-13b0fd26b120
  modified: 2026-10-26T15:10:00.000Z
---

## ✅ Instalação actual (26/10/2026)

O «render ruim» de 11/10 era o Wine sem patches: DXCore `IsHardware: No` ⇒ caminho de reserva, e o d3d9
em Vulkan sem o meio-pixel (patch `0052` do ryzendew: «blurred edges and soft text»). Reinstalado pela
AppImage `github.com/ryzendew/Linux-Affinity-Installer` release `3.3.1` (Wine 11.19 com o patch set
`0001-0085`, sha256 `bab79bdd…44e3`), em `~/Apps/affinity/Affinity.AppImage`; prefixo
`~/.affinity-appimage-wineprefix` (copiado da AppImage, 2,8 GB); DPI `96` pré-gravado em
`~/.affinity-appimage-dpi*.conf` (senão pergunta por zenity). Menu `~/.local/share/applications/affinity.desktop`
+ Área de trabalho → `~/Apps/affinity/affinity.sh`.

- ⛔⛔ **Na 1.ª abertura o Affinity rebenta: `UriFormatException` em `CombineUriWithFaceIndex`.** O Wine 11 dá
  às fontes de fora do prefixo um caminho NT `\??\unix\usr\share\fonts\…` (e às dele `\??\unix\tmp\.mount_*`),
  o DirectWrite passa-o ao WPF e o `Uri` não o lê. Reescrever o `.reg` para `Z:\` não serve (o Wine
  repõe). **Cura (no `affinity.sh`):** `FONTCONFIG_FILE` vazio (o Wine não vê fontes do Linux) + cada
  fonte do `fc-list` como symlink `lx-*` em `windows\Fonts` (caminho `C:\`, 927 ficheiros) + as 6 fontes do
  Wine (marlett, symbol, tahoma, tahomabd, wingding, webdings) copiadas para `windows\Fonts` (originais
  corefonts em `~/Apps/affinity/_backup_fonts/`) + apagar linhas `\??\unix` do `.reg` com o Wine parado.
- Gosto do dono (26/10): **inglês** (`LC_ALL=en_US.UTF-8` no `affinity.sh` — o Affinity segue o locale do
  Wine), **DPI 120** (125 %, em `~/.affinity-appimage-dpi.conf`), **ClearType** (`FontSmoothingType=2`,
  gamma `1400`, RGB) + `Avalon.Graphics\DISPLAY1` `ClearTypeLevel=100`/`TextContrastLevel=2` (o WPF lê
  daí; antes era cinza simples). Aplicado por `wine regedit /S` com o Affinity fechado.
- ⛔ `pkill -f -- '--appimage-mount'` também mata o próprio shell (o padrão está no texto do comando).
- ⛔ O AppRun só respeita `WINEDEBUG` se `WINE_DEBUG` estiver definido (verifica o nome errado).
- ⛔ O AppRun faz `wineserver -k` a cada arranque ⇒ o `affinity.sh` sai se já há janela aberta.
- Fechar com calma: `wine taskkill /im Affinity.exe` (sem `/f`) com o Wine do `/tmp/.mount_Affini*`.
- ⛔ `pkill -f -i affinity.exe` mata o próprio shell — use `pkill -i -x affinity.exe`.

## Instalação anterior (11/10, Wine do sistema) — removida

⛔ **DESINSTALADO em 2026-10-11** (a pedido do Enio, logo depois da cura do desenho; não disse porquê — antes queixara-se de que «a renderização fica ruim»): `~/Apps/affinity`,
atalhos, ícones, `wine-extension-af`/`wine-protocol-affinity` e o `.menu` apagados. O resto é a receita, se voltar.

Instalado em 2026-10-11 a pedido do Enio. Instalador oficial `https://downloads.affinity.studio/Affinity%20x64.exe`
(assinado «Canva», DigiCert; sha256 `9e05d95a…018b`) em `~/Apps/affinity/_instalador/`.

- **Prefixo:** `WINEPREFIX=~/Apps/affinity/wineprefix`, Wine do sistema (11.19) + `winetricks remove_mono vcrun2022
  dotnet48 corefonts win11` (`setup.sh`, ~3 min). App em `drive_c/Program Files/Affinity/Affinity/` (1,6 GB).
- **Lançador:** `~/Apps/affinity/affinity.sh` → `Affinity.exe` DIRETO; atalhos (menu `wine/Programs/Affinity.desktop`
  e Área de trabalho) apontam para ele. Abre no editor sem pedir login Canva (medido 90 s estável, 11/10).
- ⛔ **APL + WineFix v0.3.0 (noahc3) FECHA o Affinity 3.3.1:** `NullReferenceException` em
  `Serif.Affinity.Application.PostLoad_Patch1` (relatórios em `AppData/Roaming/Affinity/Affinity/3.0/CrashReports/Backtrace/*-report.json`).
  Ficheiros continuam na pasta do app mas o lançador não os usa. Custo: as preferências podem não gravar.
  Reavaliar quando sair APL > 0.3.0 (issues #46/#56/#61 do repo tratam 3.2.x).
- ⛔ **Os `.winmd` que o Wine 11 põe em `system32/winmetadata` (minúsculas) derrubam o Affinity ao abrir:**
  `FileLoadException 0x80131047` em `AboutDialog..ctor`, com ou sem APL (o guia AffinityOnLinux diz que no
  Wine ≥ 11 não é preciso — falso aqui). Cura: os da Microsoft (`archive.org/download/win-metadata/WinMetadata.zip`,
  em `_extras/wm/`); o lançador apaga `windows.*.winmd` e repõe-nos a cada arranque (um update do Wine recoloca-os).
  Originais do Wine guardados em `_extras/winmetadata.wine-original/`.
- ⛔ **UI branca (só a tela e o selo «Vetor» pintados) com o renderer por omissão do Wine** — cura medida:
  `winetricks renderer=vulkan` (NVIDIA RTX 5060 Ti); depois disso desenha tudo (11/10).
- ✅ **Desenho bom = DXVK 3.1.1 (`winetricks dxvk`) + `d3d9=b` + `d2d1=n,b`** (tudo no `WINEDLLOVERRIDES` do lançador):
  o DXVK no d3d9 deixa a janela TODA PRETA (a UI WPF vai por d3d9) ⇒ d3d9 fica do Wine; o d2d1 nativo é o do
  WineFix (`d2d1.dll` na pasta do app — corrige curvas Bézier; só esse ficheiro, sem o APL). O DXCore continua a
  dizer `IsHardware: No` (o Affinity não vê a GPU por DXCore), mas o DXGI lista a RTX 5060 Ti.
- ⛔ **Ao fechar, o `Affinity.exe` às vezes fica vivo SEM janela**; a 2.ª abertura diz «Não foi possível se
  comunicar com outros aplicativos Affinity» (ipc.log: `HandshakeFailure`). O lançador: janela visível ⇒ traz à
  frente; processo sem janela ⇒ `wineserver -k` antes de abrir. Um `-k` com o app aberto corrompe as definições
  («Ocorreu um erro ao carregar suas configurações» — inofensivo, OK).
- Falta WebView2 ⇒ os painéis de ajuda/aprender ficam vazios (inofensivo).
- O Wine criou 10 associações `wine-extension-*.desktop` (crt, reg, chm…) — apagadas; o lançador desliga o
  `winemenubuilder` para não voltarem.
- Captura da janela: `xdotool search --onlyvisible --class affinity.exe` + `import -window <id>`; o `xdotool`
  NÃO clica nas janelas do Wine na sessão Wayland do dono — os cliques são dele.
