---
name: reference_corel_painter_trial_installed_under_wine
description: Corel Painter 2023 (teste 15 dias) sob Wine em ~/Apps/corelpainter — «Error -1» era IPM.msi/IPMContent.msi instalados SEM SKU=0/PCUSOURCEID=610005; reparado 27/10, abre na EULA (o dono aceita)
metadata:
  type: reference
---

⛔ **O dono DESISTIU do Corel Painter em 28/10** (EULA destravada, mas nunca passou dela). Não retomar sem ele pedir.

Prefixo `~/Apps/corelpainter/wineprefix` (instalado 19/10, instaladores em `~/Apps/corelpainter/instalador/zip`).
Ecrã real: `~/Apps/corelpainter/painter-tela.sh` (atalho «Corel Painter 2023» na Área de trabalho e no menu);
tela invisível (Xvfb :73, só software) para sondas: `painter.sh` + `~/Apps/pincel-oraculos/`.

- ⛔ **«Error -1 — reinstall from the original source»** = o PCU (licença Corel) diz `CreatePCUCore failed` em
  `%TEMP%\Painter 2023 x64_PCULog*.txt` (UTF-16). Causa: a 1.ª instalação correu os MSI sem as propriedades que
  o `Setup.exe` passa (`Setup.xml`, `<Dbm … cmdline=…>`). Os `PNPCUNLR.DLL`/`PTPCUNLR.DLL` escolhem-se por
  `SKU` (tabela Component: `0`=TBYB/teste, `2`=Retail.Man, `8`=Retail.Sub, `3`=Corp) — sem SKU nenhum é posto
  em `Painter 2023\`; e a cache de mensagens caiu numa pasta literal `Messages\LICID_PCUSOURCEID`.
- ✅ Cura (27/10): `msiexec /x` + `/i` de `Painter\IPM.msi` e `Painter\IPMContent.msi` com `PCUSOURCEID=610005
  PREMIUMFEATURES_PCUSOURCEID=999923 SERIALNUMBER=PF23T22-ABV39N5-Q867MDW-6UPPHEA SKU=0 SHOW_EULA=0
  ALLOW_PRODUCTUPDATES=1` (serial de teste vem no `Setup.xml`). `SHOW_EULA≠1` mantém `ShowEULA=1` ⇒ a EULA
  aparece no 1.º arranque e é o DONO que a aceita.
- ⛔ **Botão «Agree» da EULA nunca acende sob Wine** (28/10): a EULA é HTML (`eula_container.htm` na cache
  `Messages\<id>\<lang>\MessageCache*\Workflow\ui\`, em ProgramData E em Roaming); o iframe avisa o fim da rolagem
  por `postMessage` e isso não chega no Wine Gecko. Cura: trocar `if (!window.parent.postMessage) {` por `if (true) {`
  (a saída da própria Corel; `.orig` ao lado). Botão = HTML, `EnableWindow` não o alcança. ⛔ `ShowEULA=0` no
  registo NÃO serve: dá «Error 4». Se a cache for renovada do servidor, repor o remendo.
- ⛔ O «Repair» do `Setup.exe` rebenta sob Wine; o `.lnk` «Continue … Installation» foi arrumado em `_atalhos_antigos/`.
- Ler as tabelas de um MSI sem msitools: `wine cscript` com JScript sobre `WindowsInstaller.Installer`.
- Wine sob o guarda da placa: todo comando com `wine` vai por `PH2D_GPU=1 bash scripts/ph2d-run.sh`.
