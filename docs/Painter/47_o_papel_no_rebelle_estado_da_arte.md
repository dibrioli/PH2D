# 47 — O papel no Rebelle: o estado da arte, MEDIDO (2026-10-07)

> **Pergunta do dono** (2026-10-06, com fotos): *«após pintar com papel branco e depois escurecer o
> papel, as áreas pintadas não sofreram nenhum escurecimento e em wet paint pontos brancos apareceram
> ao redor. Corrija. Permita o papel escurecer a tinta como no mundo real»* — e depois: *«vá
> estudá-lo [o Rebelle] e testá-lo e encontre o estado da arte»*.
>
> **Oráculo:** Rebelle 8 Pro 8.3.4 (teste de 30 dias, Windows) sob Wine 11, prefixo
> `~/Apps/rebelle/wineprefix`. Proprietário, EULA do teste *«personal, non-commercial internal
> use»* ⇒ ORÁCULO que se corre (CLAUDE.md §0.9), nunca fonte: nada do programa foi lido, só a
> imagem que ele mostra a um utilizador.

## 1. A porta (como se corre sem tocar no ecrã do dono)

- **Tela:** `kwin_wayland --virtual --xwayland` 1920×1080 com `XDG_CONFIG_HOME` isolado que maximiza
  tudo (`~/Apps/rebelle/oraculo/inicia.sh` + `sessao.sh`, prazo 3 h; recusa `DISPLAY=:0` /
  `wayland-0`). O Rebelle corre em X11 na Xwayland virtual.
- **Rato e teclado:** o XTest é ignorado nessa Xwayland e o `ydotool` move o rato REAL — proibidos.
  A porta que funciona é **dentro do Wine**: o Python *embeddable* oficial para Windows
  (`~/Apps/rebelle/py`, python.org 3.12.7) corre `oraculo/inj.py`, que manda `SendInput` (mover,
  clicar, arrastar com o botão premido num evento SEPARADO — com o `LEFTDOWN` no mesmo evento do
  movimento o Rebelle não pinta), teclas, e lê a posição de uma janela por `GetWindowRect`
  (`inj.sh "rect <título>"`). Tudo isto passa pela fila de entrada do Wine, sem servidor de ecrã.
- **Imagem:** `import -display <virtual> -window <id>` (`oraculo/foto.sh`); a régua é
  `oraculo/regua.py` (média de uma caixa 8×3 por traço, duas fotos: papel branco · papel de cor).
- ⚠️ As coordenadas da foto da janela principal são as do ecrã + 28 px na vertical (a barra de
  título). ⚠️ O shell destas sessões é zsh: `set -- $m` não parte palavras — a 1.ª régua leu o
  cinzento da interface (`110`) em todos os traços; a régua passou a Python com coordenadas
  explícitas.

## 2. O que o Rebelle faz com o papel

- **O papel NÃO é uma camada.** É um item à parte no fundo do painel de camadas (nome do papel,
  cor, olho), e a cor dele muda no painel «Select Canvas» («Canvas Color»). Um desenho novo nasce
  com a camada **transparente** sobre o papel — mesmo quando o papel é branco.
- **A tinta guarda o seu ALFA por píxel, na mesma camada, em todos os meios.** O composite é «cobrir
  com transparência» (`tinta·a + papel·(1 − a)`), **não multiplicar**: a opacidade aparente
  `a = 1 − (W − B)/(255 − P)` sai IGUAL nos três canais em todo meio (tabela abaixo) — a assinatura
  dessa regra; multiplicar daria um `a` diferente por canal.
- **A transparência não é uma regra do MEIO, é o alfa que o pincel depositou.** A aquarela a
  Opacity `50` deposita alfa `0,50` (e a cor recuperada, `(23, 101, 179)`, é a do óleo, `(25, 103,
  180)`: o mesmo pigmento a meio alfa); a Opacity `100` deposita `0,8–1,0` (vermelho e azul; o
  verde tem só `16` níveis de contraste e sai ruidoso).
- **A ordem não importa.** A aquarela pintada DEPOIS de o papel ficar castanho mede `(67, 94, 115)`,
  exactamente a pintada ANTES no branco e escurecida depois; o óleo também (a menos da textura).

### A medida (fixture) — Rebelle 8.3.4, papel EX00 White Simple → `#705834`, a mesma camada

Cor de fábrica do pincel (azul); `W` = sobre o branco, `B` = depois de mudar o papel para
`(112, 88, 52)`; «multiplicar» = `W × papel / 255`.

| meio (preset) | W (branco) | B (castanho) | multiplicar daria | opacidade aparente (R, G, B) |
|---|---|---|---|---|
| Oils & Acrylics (Bristle Flat) | 25, 103, 180 | 25, 103, 180 | 11, 36, 37 | 1,00 · 1,00 · 1,00 |
| Express Oils (Hairy) | 65, 129, 192 | 39, 99, 156 | 29, 45, 39 | 0,82 · 0,82 · 0,82 |
| Watercolors (Flat 2, Opacity 50) | 139, 178, 217 | 67, 94, 115 | 61, 61, 44 | 0,50 · 0,50 · 0,50 |
| Watercolors (início do traço) | 224, 234, 245 | 100, 90, 69 | 98, 81, 50 | 0,13 · 0,14 · 0,13 |
| Inks (Bullet, linha fina) | 248, 250, 253 | 109, 88, 56 | 109, 86, 52 | 0,03 · 0,03 · 0,03 |
| Pencils (8B, grafite) | 161, 160, 159 | 94, 83, 65 | 71, 55, 32 | 0,53 · 0,54 · 0,54 |
| Pastels (Dry Thin) | 56, 123, 190 | 36, 99, 161 | 25, 42, 39 | 0,86 · 0,86 · 0,86 |
| Markers (Liner) | 224, 234, 245 | 100, 90, 69 | 98, 81, 50 | 0,13 · 0,14 · 0,13 |
| Airbrushes (Spray 2) | 209, 224, 240 | 94, 91, 77 | 92, 77, 49 | 0,20 · 0,20 · 0,20 |

(Os traços finos — tinta, marcador — foram amostrados na borda: o valor diz a regra, não o miolo.)

## 3. Porque o PH2D falhou o teste do dono

- **Um desenho novo do PH2D é tinta BRANCA opaca na camada** (BUGS #39), e o papel só passa a existir
  quando se escolhe uma cor. Tudo o que se pinta antes disso é misturado com esse branco e fica
  OPACO; o `separa_o_branco` (BUGS #42) recupera o alfa da ORLA de cada traço, mas um miolo pintado
  a meio alfa sobre o branco é, por construção, indistinguível de uma tinta clara opaca — e fica
  opaco (a regra «a opaca cobre»). Daí o «nada escureceu».
- Os **pontos brancos do Wet Paint** são píxeis quase brancos da orla do fluido que o
  `separa_o_branco` não reconheceu como tinta (fora da reta da cor dela) e ficaram opacos.
- O Rebelle não tem nenhum dos dois porque o papel nunca entrou na camada.

## 4. O que isto pede ao PH2D (para decisão do dono)

O estado da arte **não** é uma regra por meio nem um plano novo por píxel: é **o papel existir
desde o primeiro instante e nunca ser tinta**. Um desenho novo nasce com papel branco e a camada
transparente; cada traço guarda o alfa que o pincel lhe deu; mudar a cor do papel depois dá o mesmo
que pintar sobre ela (como no Rebelle, ao nível). A regra de composição do papel que o PH2D já tem
(«cobrir com transparência», `sobre_o_papel`, na CPU e na GPU) é a mesma do Rebelle — não muda.
O `separa_o_branco` continua só para desenhos que já nasceram brancos (a arte importada, os de antes).

⚠️ A MEDIR antes de construir: o alfa que a Aquarela e o Wet Paint do PH2D depositam numa camada
transparente com o pincel de fábrica (o Rebelle a Opacity 50 deposita `0,50`).

✅ **Feito (2026-10-07, BUGS #45):** a sprite toda branca nasce papel branco com a camada vazia (a porta
do documento, `abre_a_sprite`), e o chão óptico da Aquarela passou a ser o branco de referência com
qualquer papel (com o papel no chão, mesmo na camada transparente a ordem importava: `0,735`, pior
`62`). Escurecer o papel depois = pintar sobre ele, pior `0` níveis nos quatro meios, `0` pontos
brancos no Wet Paint. Medido o alfa do PH2D: Digital e Impasto = a Strength; Aquarela `0,70` no miolo
e Wet Paint `0,90`, a qualquer Strength (não a leem — #41).
