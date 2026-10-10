# Oráculo da colaboração ao vivo do Excalidraw — corrido por script, nunca lido

Corre o **app do excalidraw.com** (`excalidraw-app/` do monorepo) e o **servidor de salas**
`excalidraw-room` (socket.io), os dois em `127.0.0.1`, e mede os 23 casos de
[`../casos.json`](../casos.json) com 2–3 contextos do Playwright na MESMA sala. Saídas em
[`../saidas/excalidraw/`](../saidas/excalidraw/) (`<caso>.json` + `meta.json`). CLAUDE.md §0.9: o
fonte do Excalidraw e do excalidraw-room **não foi aberto** — só `LICENSE`, `package.json`, `.env*`,
`README` e a config de build (`vite.config.mts`, para saber o `envDir`).

## Artefactos fixados (fora do repo, em `~/Apps/excalidraw-colab-oracle/`)

| artefacto | commit / versão |
|---|---|
| `excalidraw` (monorepo; o app é `excalidraw-app/`) | `4c00f31ddcc20086bac1020428819fe0b9a30bce` (master, 2026-10-08) — `@excalidraw/excalidraw` 0.18.0 do workspace |
| `excalidraw-room` | `03ff435860b508d7cd9e005cfc90f7977ae2a593` (master; último commit 2023-12-15, «follow mode») |
| `socket.io-client` (o que o app embarca) / `socket.io` (sala) | 4.7.2 / 4.6.1 |
| yarn (local, `tools/`) · Playwright · Chromium | 1.22.22 · 1.63.0 · o do Playwright (`harness/node_modules` → `~/Apps/excalidraw-oracle/node_modules`) |

## Licença (triada por artefacto INSTALADO, 2026-10-10 — `node licencas.mjs ~/Apps/excalidraw-colab-oracle`)

| artefacto | versão | `license` do `package.json` instalado | ficheiro LICENSE | papel |
|---|---|---|---|---|
| clone `excalidraw` | 4c00f31 | — (monorepo privado) | MIT | o app |
| clone `excalidraw-room` | 03ff435 | MIT | MIT | o servidor de salas |
| `@excalidraw/excalidraw` · `common` · `element` · `math` (workspace) | 0.18.0 | MIT | — | o editor, dentro do app |
| `socket.io-client` (app) · `engine.io-client` · `socket.io-parser` | 4.7.2 · 6.5.4 · 4.2.4 | MIT | MIT | cliente da sala |
| `socket.io` · `engine.io` · `ws` · `express` (sala) | 4.6.1 · 6.4.2 · 8.11.0 · 4.17.1 | MIT | MIT | servidor |
| `firebase` (embarcado; bloqueado) | 11.3.1 | Apache-2.0 | — | persistência |
| `react` · `react-dom` · `jotai` · `roughjs` · `perfect-freehand` · `uqr` | 19.0.0 · 19.0.0 · 2.11.0 · 4.6.4 · 1.2.0 · 0.1.2 | MIT | MIT | app |
| `idb-keyval` | 6.0.3 | Apache-2.0 | — | app |
| `pako` · `dompurify` | 2.0.3 · 3.3.1 | MIT AND Zlib · MPL-2.0 OR Apache-2.0 | — | app (dompurify: escolhe-se Apache-2.0) |
| `@sentry/browser` (embarcado, desligado) | 9.0.1 | MIT | MIT | — |
| `khroma` (mermaid) | 2.1.0 | (sem campo) | MIT | app |
| `dotenv` · `yarn` | 10.0.0 · 1.22.22 | BSD-2-Clause | BSD-2 | sala · ferramenta |
| `playwright` · `playwright-core` | 1.63.0 | Apache-2.0 | Apache-2.0 | harness |

Varredura dos 1 666 `package.json` instalados (app + sala): todos permissivos; fora do padrão só
`axe-core` (MPL-2.0 — dev, do eslint; **não** está no build), `json-schema` (AFL-2.1 OR BSD-3), `opener`
(WTFPL OR MIT), `union`/`vite-plugin-sitemap` (sem campo; LICENSE MIT; dev). Nada embarca no PH2D: o
oráculo só CORRE.

## O que foi preciso para funcionar em local

- **Sala:** `PORT=3102 CORS_ORIGIN=http://127.0.0.1:5101` (as duas variáveis do `.env.development` dela) e
  `node -r so_local.cjs` — um pré-carregamento que força todo `listen()` a `127.0.0.1` sem tocar no fonte.
- **App:** build de produção (`vite build --mode production`) com as variáveis do processo por cima dos
  `.env*`: `VITE_APP_WS_SERVER_URL=http://127.0.0.1:3102`; `BACKEND_V2_*`, `LIBRARY_*`, `PLUS_*`, `AI_BACKEND`
  → `http://127.0.0.1:9` (porta fechada); `VITE_APP_FIREBASE_CONFIG` = projecto FICTÍCIO
  `ph2d-oracle-offline`; Sentry, tracking e PWA desligados. Servido por `serve_app.mjs` (estático, 127.0.0.1:5101).
- **Rede no Playwright:** `context.route` aborta e REGISTA tudo o que não seja `127.0.0.1`/`localhost`;
  `serviceWorkers: "block"`. Tentado e bloqueado (em todos os casos): `POST firestore.googleapis.com/…/
  ph2d-oracle-offline/…:batchGet` (o app tenta guardar/ler a cena no Firebase), `GET
  scripts.simpleanalyticscdn.com/latest.js` (está no `index.html` mesmo com tracking off) e 4 pesos da fonte
  Assistant no CDN. As outras fontes do CDN (`/oss/fonts/…`) são servidas do PRÓPRIO build local.
- **A colaboração NÃO precisa de nenhum serviço externo:** a sala é só o relé socket.io; quem entra recebe a
  cena dos pares por WebSocket (c13). O Firebase só serviria para persistir a sala — sem ele nada falha à vista.

## Como se mede (`corre_colab.mjs`)

- **Sala:** A abre o app → «Live collaboration…» → «Start session»; o link que o app gera
  (`#room=<20 hex>,<chave 22 b64>`) é aberto pelos outros contextos. Espera-se até cada par ver os N
  colaboradores.
- **Gestos:** só rato e teclado (ferramentas pela barra, cor pelos «top picks» do painel, `Delete`,
  `Ctrl+Shift+]`, `Ctrl+Z`/`Ctrl+Shift+Z`, duplo clique para texto, clique no avatar para seguir).
- **Leitura do estado:** o gancho padrão das devtools do React (`__REACT_DEVTOOLS_GLOBAL_HOOK__`, injectado
  antes da página) acha a instância do editor e lê-se pela API pública dela
  (`getSceneElementsIncludingDeleted()`, `.state`); mais o DOM (avatares, selo «Following …») e os pixels do
  canvas interactivo (que só leva as sobreposições: cursores e selecções alheias).
- **Concorrência:** `page.routeWebSocket` liga cada página à sala através do harness. Cada mensagem
  socket.io é UM grupo (cabeçalho `45N-[evento,…]` + N anexos binários encriptados — regista-se evento,
  bytes e instante). «Ao mesmo tempo» = os grupos de saída dos pares ficam RETIDOS enquanto cada um faz o
  seu gesto e são libertados pela ordem do caso (200 ms entre pares); c16 duplica, c17 inverte lotes.
- **Sem rede (c14, c15), dois regimes:** `offline` = `context.setOffline(true)` (o navegador SABE — medido: o
  Chromium fecha o WebSocket na hora e o app volta a ligar-se ~0,3 s depois de a rede voltar); `buraco` = a
  ligação fica aberta e os quadros ficam retidos nas duas direcções, entregues ao voltar (queda curta de
  TCP, sem o navegador saber).
- **Convergência:** lista por z (ordem do array da cena) com tipo, x, y, w, h, cores, texto, apagado e
  `index`, igual em todos os pares (sondada a cada 100 ms, até 8 s; 30 s em c14/c15; 40 s em c23).
- **Determinismo:** duas corridas completas; o campo `deterministico` de cada caso tem de ser igual nas
  duas (`compara.mjs`). O que depende do `versionNonce` (aleatório por edição) vai para `variavel`.

## O que mediu (duas corridas completas, 10/10/2026 — detalhe e números em `../saidas/excalidraw/`)

A lei que atravessa quase tudo: **a unidade de fusão é o ELEMENTO INTEIRO, decidida pela `version`**
(contador de edições do elemento) e, em empate, pelo **menor `versionNonce`** (aleatório a cada edição).
Não há fusão por campo nem por carácter; a ordem de chegada não decide.

| caso | resposta | prova |
|---|---|---|
| c01 mesmo campo | converge sempre; ganha quem tem o MENOR `versionNonce` (empate de versão 15/15) — determinista dado o nonce, aleatório para o utilizador (12 rodadas: A e B ganham ~metade cada) | `c01` `rodadas[].nonce_local` |
| c02 mover × cor | perde a edição INTEIRA: fica só o mover (14 versões contra 1–2 da cor) | `c02` |
| c03 mover × texto | idem: fica o mover; o texto novo de B perde-se inteiro (o texto preso acompanha a forma) | `c03` |
| c04 apagar × mover | o apagado RESSUSCITA nas duas ordens (o mover tem mais versões) | `c04` |
| c05 criar no mesmo z | os dois nascem com `index a0`; convergem e concordam; quem fica por cima é desempate (varia entre corridas) | `c05` |
| c06 trazer para a frente | concordam; Z em baixo; X/Y no topo por desempate (ZYX ou ZXY entre corridas) | `c06` |
| c07 seta × forma apagada | Q apagado, a seta fica SOLTA (sem `endBinding`), não desaparece | `c07` |
| c08 seta segue forma alheia | segue; quem recalcula é quem MOVE (B): a seta sobe de versão 12→20 nos dois e A não reemite nada | `c08` |
| c09 texto a dois | fica UM inteiro («baseaaa» OU «basebbb»), nunca intercala; empate v9/v9 → nonce | `c09` |
| c10 desfazer só o meu | sim: o Ctrl+Z de A desfaz a cor e depois a criação de X; Y de B nunca é tocado | `c10` |
| c11 desfazer depois do outro | o 1.º Ctrl+Z volta X ao sítio E mantém a cor de B (o desfazer é por campo, sobre o estado actual) | `c11` |
| c12 desfazer o que o outro apagou | X NÃO ressuscita: o Ctrl+Z de A mexe no X apagado (volta-o a (400,300)) mas ele continua apagado | `c12` |
| c13 entrar a meio | B vê as 50; a cena chega numa só mensagem de A (~23,7 KB, ~475 B/forma); ~240 ms do `goto` a igual | `c13` |
| c14 sair e voltar | nada desaparece (W e V chegam aos dois), o mover de A ganha, a COR de B em X perde-se; a convergência só chega ~17 s depois de voltar (a ressincronização periódica), não ao religar | `c14` |
| c15 rede cai no arrasto | B acaba com a posição final de A (nos dois regimes) | `c15` |
| c16 mensagem duplicada | idempotente | `c16` |
| c17 reordenadas | B fica com a posição FINAL (o lote velho é ignorado por versão) | `c17` |
| c18 cursor | ~23 msg/s (estrangula ~42 ms com o rato a 58/s), ~355 B/msg, ~8,1 KB/s; B desenha SEM interpolação (saltos de ~17 px); nome aleatório «Adjectivo Animal», cor do avatar | `c18` |
| c19 arrastar | ~60 msg/s (29,5 fiáveis + ~30 cursor); o caminho chega todo a B; custo ∝ seleccionadas: 1 → 31 KB/s · 10 → 171 KB/s · 50 → 795 KB/s (~500 B por forma por mensagem) | `c19` |
| c20 selecção alheia | B vê o contorno tracejado na cor do avatar de A | `c20` |
| c21 seguir | existe (clicar no avatar); câmara igual 1 quadro depois (~16 ms); interrompe qualquer gesto de câmara/canvas de B e o × do selo; A desenhar não interrompe | `c21` |
| c22 trazer todos | NÃO existe | `c22.prova` |
| c23 três pares | 1 s depois da libertação os três ficam DIVERGENTES em ~metade das ordens; convergem sempre, mas só ~17–19 s depois (ressincronização periódica) | `c23` |

**Divergências entre as duas corridas:** só `c14` (regime «buraco»): numa das 6 repetições o X acabou em
(575,300) — uma posição intermédia do arrasto de A — nos dois pares, depois de A o ter a (600,300). E, antes
do reforço de c14, em 2 de 5 repetições «offline» o V criado por B sem rede (verde em B antes de voltar)
acabou ROSA (a cor que B dera a X) nos dois. Raro, não determinista, registado em `meta.json`.

**O que não se mediu:** o Firebase (bloqueado; o app mostra «Couldn't save to the backend database» ~10 s
depois de editar — o harness regista o diálogo e fecha-o); ficheiros/imagens (vão pelo Firebase Storage);
ligações muito lentas/perda parcial de pacotes (só os dois regimes acima); quedas maiores que o
`pingTimeout` (45 s).

## Uso

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone   # ou o primário depois da integração
bash docs/MiroClone/ferramentas/colab_oracle/excalidraw_colab/instala.sh      # 1× por máquina (~1,5 GB em ~/Apps/excalidraw-colab-oracle)
bash docs/MiroClone/ferramentas/colab_oracle/excalidraw_colab/corre_tudo.sh   # 2 corridas × 23 casos (~16 min) → ../saidas/excalidraw/
# um caso só (servidores arrancados e parados pelo próprio script):
bash docs/MiroClone/ferramentas/colab_oracle/excalidraw_colab/corre.sh /tmp/x c01_mesmo_campo
```

Os servidores correm sob `timeout` (3 h) e são parados pelo PID guardado em `~/Apps/excalidraw-colab-oracle/run/`
(`servidores.sh stop`: a folha `node` primeiro); nunca por padrão.
