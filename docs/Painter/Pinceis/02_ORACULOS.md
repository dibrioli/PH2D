# 02 — Os ORÁCULOS que correm (e os que não correm)

> Entrega C da JANELA 1. Lei: CLAUDE.md §0.9 — um app é um programa que se CORRE sobre entradas NOSSAS
> (`ferramentas/bateria.json`, a fonte única), nunca um fonte que se lê. Cada secção diz: o comando, a prova, a
> licença, quando expira, o que mede e o que não mede. Krita e MyPaint/libmypaint têm a sua secção (escrita pelo
> agente deles); esta parte cobre os oráculos **sob Wine** e o Procreate (só documentado).

## Resumo

| app | corre? | porta | fotos | expira |
|---|---|---|---|---|
| **ArtRage 6 Demo 6.1.2** | ✅ | script `.arscript` do próprio app (pontos e pressão exactos) | **105** (35 pincéis × 3 cenas; o Knife só `sobre`) | não expira |
| Clip Studio Paint 5.1.5 | ❌ | — | 0 | (avaliação 30 dias com conta) |
| Clip Studio Paint 1.13.2 | ❌ (arranca, bloqueia) | — | 0 | modo «Trial version: limited features», sem conta |
| Corel Painter 2023 | ❌ | — | 0 | (teste de 15 dias) |
| Photoshop (Creative Cloud) / Fresco | ❌ | — | 0 | exige Adobe ID / Microsoft Store |
| Procreate | — (só iPad) | documental | 0 | — |

## A infra comum dos oráculos sob Wine (`~/Apps/pincel-oraculos/`, fora do repo)

| script | o quê |
|---|---|
| `env.sh <app>` | prefixo `~/Apps/<app>/wineprefix`; tela **Xvfb** `:71` artrage · `:72` clipstudio · `:73` corelpainter · `:74` photoshop · `:75` clipstudio1; Mesa **llvmpipe** forçado e nenhum ICD Vulkan |
| `tela.sh <app>` | abre a Xvfb 1920×1080 da app (prazo 4 h) |
| `w.sh <app> <prazo> …` | `wine …` no prefixo e na tela da app (`PO_WINEDEBUG=err+all` para diagnóstico) |
| `x.sh <app> …` · `foto.sh <app> <png> [geo]` · `espera.sh <app> <prazo>` | `xdotool` · `import -window root` · espera a tela MUDAR |
| `wt.sh <app> <prazo> <verbos>` | `winetricks` no prefixo (sem root) |
| `gecko/`, `mono/` | wine-gecko 2.47.4 e wine-mono 11.3.0 (dl.winehq.org) para os prefixos que precisam de HTML/.NET |

**Porquê Xvfb e não `kwin --virtual`:** a Xvfb é X em software — com o llvmpipe forçado, o oráculo **não entra na
placa** (não disputa a vez de `A_VEZ_DA_PLACA.md`) e não usa a tela/instância do Rebelle (`:1`, kwin), que outro
agente fotografa ao mesmo tempo. Fotos por `import -window root` no display da app; ⛔ nunca `spectacle`.

## ArtRage 6 — ✅ CORRE

- **Comando:** `bash ~/Apps/artrage/inicia.sh` (abre `:71`, fecha o «Buy ArtRage», maximiza) e, em
  `docs/Painter/Pinceis/`: `python3 -I ferramentas/fotos_artrage.py ferramentas/lista_artrage.tsv fixtures/artrage_bateria`
  (`--so <nome>` para um pincel; ≈ 26 s por cena).
- **A porta:** o ArtRage toca **scripts** (`File > Play Script…`; o Demo só recusa GRAVAR). O conversor
  `ferramentas/bateria_artrage.py` escreve cada traço da bateria como `<StrokeEvent>` com os pontos e a pressão
  exactos; a ferramenta por `CID_ToolSelect`, o preset pelos bytes do `.prs` INSTALADO (lido do prefixo em tempo de
  corrida, nunca copiado), a cor por `SetForeColour`, o tamanho por `CID_SetSpecificToolSize` com `k` medido por
  ferramenta. Formato lido na página oficial «Example Script Code» e confirmado no programa.
- **Prova:** [`pranchas/oraculo_artrage.png`](pranchas/oraculo_artrage.png) — Oil Brush «Thick Gloss» e Watercolor
  «Wet on Wet» nas 3 cenas. Fixture: [`fixtures/artrage_bateria/`](fixtures/artrage_bateria/README.md) (105 fotos, 34 MB).
- **Licença:** Demo grátis, **sem prazo**, sem conta, «may only be used for evaluation purposes»; não grava,
  não exporta, não grava scripts. Instalador oficial (sha256 no README da fixture) em `~/Apps/artrage/instalador/`.
- **Expira:** não.
- **Mede:** a forma e a textura de cada pincel nos 9 gestos (inclusive as rampas de pressão), a resposta ao tamanho
  (5/40/200 px nominais), a mistura física do óleo, da aquarela e do Palette Knife sobre tinta molhada (mesma camada)
  e o comportamento sobre tinta «seca» (camada nova).
- **Não mede:** o tempo (o ArtRage não tem secagem nem simulação ⇒ foto B ≡ foto A), a inclinação/rotação da caneta
  (`Ti`/`Ro` fixos), o **Chalk** (nenhum id da ferramenta pela porta de script: `4903` ≡ Crayon byte a byte), o
  lápis acima de 41 px (o tecto dele), e as velocidades fora dos 8 ms por evento.
- **Achado de porta (medido):** pressão **1,0 exacta** é lida como rato e corta o traço do Crayon/Felt Pen ⇒ o
  conversor usa no máximo 0,999 (detalhe e as outras recusas no README da fixture).

## Clip Studio Paint — ❌ NÃO CORRE (≈ 1 h gasta)

- **Instaladores oficiais** (clipstudio.net, `~/Apps/clipstudio/instalador/`): `CSP_515w_setup.exe` (5.1.5, sha256
  `9955e533…3b251`) e `CSP_v1_setup.exe` = `CSP_1132w_setup.exe` (1.13.2, sha256 `70525c4a…a6abf8`).
- **EULA** (`clipstudio.net/en/dl/eula/`, lida): uso «solely for User's own internal use»; a versão de avaliação é
  «for a limited time period only» (30 dias com conta CLIP STUDIO; sem licença arranca «com funções limitadas, sem
  gravar nem exportar»); proíbe engenharia reversa e «interfere with the normal functionality». Correr o programa e
  fotografar o ecrã é uso interno de avaliação.
- **5.1.5** (prefixo `~/Apps/clipstudio/`, `:72`): instala; arranca até ao splash «Ver.5» e à transferência de
  materiais, e pára num **diálogo modal branco** 1194×834 (o diálogo web de início/conta). Com rede: deadlock no
  `wininet` (`init_sechandle_cs`, SSL); com um proxy morto o deadlock some mas o modal continua branco. É o defeito
  conhecido do CSP 5 sob Wine (menus e diálogos brancos; fórum CachyOS, sem cura).
- **1.13.2** (prefixo `~/Apps/clipstudio1/`, `:75`, com wine-gecko): instala e **arranca** — o diálogo «Getting
  Started» desenha-se, «PRO» entra em «*Trial version: You can use limited features — CLIP STUDIO PAINT PRO*» SEM
  conta. Mas, a seguir ao «Setting up…», abre um popup web que fica **branco e não recebe entrada**, e a janela
  principal fica desactivada (modal) — com rede e com o proxy morto. `windowunmap` do popup não solta o modal.
- **Porta prevista** (por fazer): o CSP não tem script de traços; seria injecção de ponteiro. Para a PRESSÃO sob
  Wine falta uma mesa digitalizadora: o caminho é um `wintab32.dll` nosso que devolva pacotes WT com a pressão da
  bateria (o CSP, o Painter e o ArtRage leem Wintab). Não feito — não havia app para o receber.
- **Próximo passo:** achar a preferência que desliga o popup de arranque do 1.13 (as `Preference/*.sqlite` do
  prefixo) ou correr o CSP numa VM Windows.

## Corel Painter 2023 — ❌ NÃO CORRE (≈ 50 min gastos)

- **Instalador oficial:** o «stub» `CorelPainter2023.exe` (2,8 MB, link da página `painterartist.com/en/product/painter/`)
  — a página só oferece o teste do 2023 (o 2024/2025/2026 dão 302). Sob Wine o stub falha o download
  (`TransferDownloadError 800c0008`); o pacote que ele pede (`…/Painter/2023/Stub/CorelPainter2023_en.zip`, 570 MB,
  sha256 `fa344c98…49fcc`, CDN da Corel) instala-se à mão com `msiexec` (os 8 MSI do `Setup.xml`, + VC++ e ICA).
- **Licença:** «FREE 15-day trial» (página da Corel); a EULA vem no ecrã do instalador, que nunca se desenhou sob
  Wine (janela HTML branca) ⇒ **não lida**.
- **Falha:** `Painter 2023 x64.exe` sai com «**Error: -1 — Corel Painter has encountered a problem. Please reinstall
  from the original source**», logo após `class {7d096c5f-ac08-4f1f-beb7-5c22c517ce39} not registered` (o
  licenciamento Protexis/PSI) e `mscoree:FixupVTableEntry` (assembly C++/CLI). Com `winetricks dotnet48`: igual.
- **Sem porta:** os «Scripts» gravados do Painter seriam a porta (texto com os traços), mas o app não abre.

## Photoshop (Creative Cloud) e Fresco — ❌ NÃO CORREM (registo da tentativa, 15 min)

- `Creative_Cloud_Set-Up.exe` (2,1 MB, CDN da Adobe `prod-rel-ffc-ccm.oobesaas.adobe.com`, sha256 `d0b11eb1…2c68e4`)
  sob Wine (`~/Apps/photoshop/`, `:74`): abre uma `AXWIN Frame Window` preta e não instala nada em 15 min
  (`secur32`/`rpcss` não arrancam no prefixo). E o teste do Photoshop exige **Adobe ID** (a conta do dono) ⇒ não há
  caminho autónomo. O **Fresco** é só Microsoft Store (MSIX) no Windows ⇒ sem caminho no Wine.
- **Referência documental** (só comportamento): [Painting tools](https://helpx.adobe.com/photoshop/using/painting-tools.html)
  (Brush Settings: forma, dispersão, textura, dual brush, dinâmica de cor, transferência, wet edges) e
  [Fresco Live Brushes](https://helpx.adobe.com/fresco/using/live-brushes.html) (aguarela e óleo «vivos»).

## Procreate — 📄 só documentado (iPad)

- **Fonte oficial:** o Procreate Handbook —
  [Brush Library](https://help.procreate.com/procreate/handbook/brushes/brush-library) (18 conjuntos na Procreate
  Library: Pencils, Pens, Inks, Markers, Pastels, Oils, Paints, Gouache, Watercolors, Charcoals, Basics, Lettering,
  Comics, Design, Grunge, Street Art, Digital, Creative; e os 18 da Classic Library: Sketching, Drawing, Inking,
  Painting, Artistic, Calligraphy, Airbrushing, Textures, Abstract, Charcoals, Elements, Spraypaints, Materials,
  Vintage, Luminance, Industrial, Organic, Water),
  [Brush Studio](https://help.procreate.com/procreate/handbook/brushes/brush-studio) e
  [Brush Studio Settings](https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings) (Stroke path,
  Taper, Shape, Grain, Rendering — wet edges, glazing/blending modes —, Wet mix, Color dynamics, Dynamics, Apple
  Pencil, Properties): a definição de cada parâmetro é a especificação do COMPORTAMENTO a reproduzir.
- **Alvo VISUAL** (só como referência de olho; ⛔ não descarregar para o repo): as pré-visualizações de traço de cada
  pincel na própria Brush Library do Handbook e na página [procreate.com/procreate/brushes](https://procreate.com/procreate/brushes)
  («Over 300 handcrafted brushes»).
- **Não mede nada:** sem app a correr, nenhuma métrica; a prancha do EXAME de um pincel «estilo Procreate» usa como
  oráculo o pincel equivalente de um app que CORRE (Krita, MyPaint, ArtRage, Rebelle) e o Handbook como lei escrita.
