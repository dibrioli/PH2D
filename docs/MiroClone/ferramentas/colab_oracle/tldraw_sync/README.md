# Oráculo de CONTRASTE — colaboração ao vivo do tldraw sync, corrido por script, nunca lido

Corre o **tldraw 5.5.2 + `@tldraw/sync` / `@tldraw/sync-core` 5.5.2** com um servidor LOCAL mínimo
([`servidor.mjs`](servidor.mjs): `TLSocketRoom` + `ws`, `127.0.0.1:5202`) e uma página mínima
([`src/oracle.js`](src/oracle.js): `<Tldraw store={useSync(…)}>`, `127.0.0.1:5201`). Dois ou três
contextos do Playwright entram na MESMA sala e fazem os gestos dos 23 casos de
[`../casos.json`](../casos.json) com rato e teclado. O oráculo principal é o Excalidraw
(`../excalidraw_colab/`); este é o contraste.

**Disciplina:** nenhum código-fonte do tldraw foi lido. Leu-se só documentação de USO: o `DOCS.md` /
`README.md` dos pacotes (secções «Basic Usage», «Server-Side Room Management», «Static assets») e as
declarações públicas (`index.d.ts`) de `TLSocketRoom`, `useSync`, `RoomSnapshot`. O estado lê-se pela
API do editor exposta em execução (`window.oracle.editor`: `getCurrentPageShapesSorted`,
`getBindingsFromShape`, `getCollaborators`, `getInstanceState`, `getCamera`) e pelo snapshot público da
sala (`room.getCurrentSnapshot()` → `GET /snapshot/<sala>`, que traz `lastChangedClock` por registo — o
«campo de versão» que o produto expõe). As mensagens leem-se no fio (`page.routeWebSocket`): tipo,
chaves do diff e, nos pushes pequenos, o próprio diff.

## Licença (triada por artefacto instalado, 2026-10-10)

O `LICENSE.md` instalado de `tldraw`, `@tldraw/editor`, `@tldraw/driver`, `@tldraw/sync`,
`@tldraw/sync-core` e `@tldraw/assets` diz só *«This code is licensed under the [tldraw license](https://github.com/tldraw/tldraw/blob/main/LICENSE.md)»*
(byte a byte igual ao do `../../tldraw_oracle/`); o `package.json` deles diz `"SEE LICENSE IN LICENSE.md"`.
O `LICENSE.md` da tag `v5.5.2` e o do `main`, baixados hoje, têm o MESMO sha256
(`9578fcdd…fa533`) que [`../../tldraw_oracle/LICENCA_tldraw_v5.5.2.md`](../../tldraw_oracle/LICENCA_tldraw_v5.5.2.md):
o texto registado vale.

| pacote | versão | licença (`package.json` instalado) |
|---|---|---|
| `tldraw` · `@tldraw/editor` · `@tldraw/driver` · `@tldraw/sync` · `@tldraw/sync-core` · `@tldraw/assets` | 5.5.2 | **tldraw license** (não-OSI) |
| `@tldraw/state` · `state-react` · `store` · `tlschema` · `utils` · `validate` | 5.5.2 | MIT |
| `ws` | 8.21.3 | MIT |
| `react` / `react-dom` / `scheduler` | 19.2.8 / 19.2.8 / 0.27.0 | MIT |
| `nanoevents` | 7.0.1 | MIT |
| `playwright` / `playwright-core` | 1.63.0 | Apache-2.0 |
| `esbuild` / `@esbuild/linux-x64` | 0.28.2 | MIT (o binário sem ficheiro: vale o campo) |
| Chromium do Playwright | build partilhado (`~/.cache/ms-playwright`) | BSD-3 |

Contagem dos 159 pacotes instalados: 148 MIT · 6 tldraw license · 2 Apache-2.0 · 2 ISC · 1 0BSD
(`cd ~/Apps/tldraw-sync-oracle && node <repo>/docs/MiroClone/ferramentas/tldraw_oracle/licencas.mjs --md`).

**Uso permitido:** a tldraw license permite *«Use the Software in Development Environments»* (hospedagem
interna de teste, não acessível ao público). Tudo corre em `127.0.0.1`, num Chromium headless, como
oráculo de medição; `node_modules`, bundle e assets vivem FORA do repo (`~/Apps/tldraw-sync-oracle`,
`.gitignore` aqui); nada do tldraw embarca no PH2D e nenhum código foi copiado — o PH2D recebe só as
LEIS medidas e as fixtures JSON. **A marca d'água e a chave não foram tocadas:** a página não passa
`licenseKey` nem esconde nada; o elemento `tl-watermark_SEE-LICENSE` («Get a license for production»)
está visível em todas as páginas.

**Rede:** `context.route` aborta TODA a requisição fora do localhost e regista-a. Na primeira montagem
a página pedia à CDN (`cdn.tldraw.com/5.5.2/…`) traduções, ícones, fontes e ícones de embed — e a
interface ficava sem ícones nem rótulos. A cura é a documentada («Self-hosting assets»):
`@tldraw/assets` 5.5.2 instalado e `assetUrls={getAssetUrls({ baseUrl: "/" })}`, servidos pelo
`servidor.mjs`. Depois disso: **zero pedidos externos** em todos os casos (`externos_bloqueados: []` em
cada saída). Nenhum pedido de verificação de licença foi visto (com ou sem bloqueio).

## Uso

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone      # ou o primário depois da integração
bash docs/MiroClone/ferramentas/colab_oracle/tldraw_sync/instala.sh  # 1× por máquina: ~/Apps/tldraw-sync-oracle (110 MB)
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/colab_oracle/tldraw_sync/corre_colab.mjs \
  docs/MiroClone/ferramentas/colab_oracle/saidas/tldraw
# 2.ª corrida + comparação (o determinístico tem de ser igual):
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/colab_oracle/tldraw_sync/corre_colab.mjs ~/Apps/tldraw-sync-oracle/corrida2
node docs/MiroClone/ferramentas/colab_oracle/tldraw_sync/compara_corridas.mjs docs/MiroClone/ferramentas/colab_oracle/saidas/tldraw ~/Apps/tldraw-sync-oracle/corrida2
# um caso só: … corre_colab.mjs --so c09,c17 <saida>   (grava meta.parcial.json)
```

O corredor lança o `servidor.mjs` como FILHO (prazo próprio de 40 min) e mata-o pelo PID no fim. Portas
5201/5202 (as 5101/3102 são do oráculo Excalidraw).

**Como «ao mesmo tempo» se faz:** `page.routeWebSocket` fica entre cada página e o servidor; o caso
liga `holdOut` nos pares, os gestos acontecem (cada par vê só a sua edição — gravado em
`antes_de_libertar`), e as filas libertam-se pela ordem do caso com 500 ms entre pares. Duplicar
(c16) e reordenar (c17) fazem-se na mesma fila, nas duas pontas (servidor→B e A→servidor). Sem rede =
`context.setOffline` (fecha o socket roteado; o cliente reconecta sozinho ao voltar).

**Saída por caso** (`../saidas/tldraw/<caso>.json`): `resposta` (uma linha), e por subcorrida (cada
ordem de libertação / variante): `cena`, `gestos` (com instante), `deterministico` (estado final de
cada par e do servidor por forma: id, tipo, índice fraccionário, x, y, w, h, cor, fill, texto, z;
bindings; lápides; `clocks_servidor`; convergiu; e o específico do caso) e `variavel` (ms até
convergir, mensagens por direcção, e CADA mensagem: par, direcção, bytes, instante, tipo, chaves do
diff, retida/libertada). Ids de formas criadas por gesto aparecem como `A#1`, `B#1`…
`meta.json`: versões, data, sha256 de `casos.json`/corredor/servidor/página, carga da máquina, e
`duas_corridas` (comparação com a 2.ª corrida).

**Determinismo:** `Math.random`/`crypto.getRandomValues`/`randomUUID` semeados por caso+subcorrida+par
(ids e índices fraccionários reproduzem-se), sala nova por subcorrida.

## Os artefactos do gesto por script (medidos)

- **Edição de texto:** o duplo-clique entra em edição com o texto TODO seleccionado; a seta para a
  direita colapsa a selecção no DOM, mas o ProseMirror só a lê depois do `selectionchange` — sem 250 ms
  de espera o texto escrito SUBSTITUÍA o antigo (c09 dava «aaa» em vez de «helloaaa»).
- **Seguir:** o botão de olho da linha do colaborador no menu de pessoas ALTERNA; e o
  `followingUserId` guarda o `userId` da presença (`user:…`), não o `editor.user.getId()` (sem prefixo).

## O que se mediu (tldraw sync 5.5.2, 2026-10-10, duas corridas completas: 0 subcorridas com o determinístico diferente)

O fio: o cliente manda `push` com `clientClock` e um diff **por campo** (`{"X":["patch",{"x":["put",600]}]}`,
`{"X":["patch",{"props":["patch",{"color":["put","red"]}]}]}`); o servidor responde
`push_result:commit|discard` a quem empurrou e `patch@<serverClock>` aos outros. Um arrasto manda
**~30 pushes/s** (um por quadro de 33 ms), cada um com a presença junto.

| caso | lei medida |
|---|---|
| c01 | mesmo campo: **ganha o último a chegar ao servidor** (A→B ⇒ B, B→A ⇒ A); converge sempre |
| c02 / c03 | campos diferentes **fundem por campo**: X movido por A E com a cor/texto de B, nas duas ordens |
| c04 | apagar × mover: **o apagar ganha nas duas ordens** (o patch de um registo apagado não o ressuscita) |
| c05 / c06 | z = **índice fraccionário calculado LOCALMENTE com jitter aleatório**; o servidor não o reescreve; fica por cima quem sorteou o maior índice (6/6 e 6/6), não o último libertado (5/6 e 3/6 = acaso); A e B concordam sempre |
| c07 | B apaga Q enquanto A arrasta a ponta presa a Q: a seta **fica** com só o binding `start→P` (o de Q morre com Q) |
| c08 | B move Q: em A a seta **segue** (recalculada em cada cliente); o registo da seta **não muda** (clock parado) e ninguém a empurra |
| c09 | texto ao mesmo tempo: **o último inteiro ganha** («hellobbb» ou «helloaaa»), nada intercala |
| c10 | Ctrl+Z de A desfaz só o X de A; o Y de B fica |
| c11 | ⚠️ o Ctrl+Z de A manda `x:400` **E** `color:black` (o valor que A conhecia): **apaga a cor de B**; o Ctrl+Shift+Z refaz só `x:600` — o vermelho de B perde-se |
| c12 | Ctrl+Z de A depois de B apagar X: A manda um `put` do registo inteiro ⇒ **X ressuscita** na posição de antes do arrasto |
| c13 | B vê as 50 formas: 6 mensagens, **27 181 B recebidos** (o `connect` = 26 985 B ≈ 540 B/forma, o documento inteiro), 1 674 B enviados; `synced-remote` em 579–625 ms desde o `goto` (inclui carregar a página) e igual a A em 599–643 ms |
| c14 | offline: A move X e cria W; B (sem rede) pinta X e cria V; ao voltar B reconecta sozinho e **tudo se funde**: X em (600,250) **vermelho**, W e V ficam; nada se perde |
| c15 | a rede de A cai a meio do arrasto (B estava em x=550): ao voltar A reenvia e B acaba em (700,350), a posição final |
| c16 | duplicado servidor→B: idempotente. Duplicado A→servidor: o servidor responde `discard` ao repetido, o cliente lança *«Received push_result for a push request that is not at the front of the queue»*, **reconecta** e converge |
| c17 | ⚠️ reordenado servidor→B: B aplica pela ordem de chegada e **fica preso numa posição velha** (x=475 ou 550 conforme a mensagem que saiu primeiro), sem convergir em 15 s; só se cura com uma edição NOVA de A em X. Reordenado A→servidor: o servidor aplica pela chegada, o cliente vê `push_result` fora de ordem, reconecta e re-empurra ⇒ converge em 700. (O WebSocket/TCP não reordena: o cliente não tem guarda contra isso.) |
| c18 | cursor: A envia **29,3–30,2 msg/s, 147–150 B/msg (4,4 kB/s)**; B redesenha 23,1–23,5 posições/s (amostra a 60 Hz), salto mediano **15,5 px**, intervalo mediano 33,3 ms ⇒ **aos saltos, sem interpolação**; B vê o nome «A» numa etiqueta e o cursor na cor de A |
| c19 | arrastar 2 s: **~30 msg/s em qualquer N**; bytes/msg crescem com N: N=1 234 B/msg (7,1–7,3 kB/s) · N=10 980–986 B/msg (30,4–30,5 kB/s) · N=50 4 317–4 369 B/msg (132,5–135,0 kB/s); B vê o **caminho** (59–62 posições distintas) |
| c20 | B vê a selecção de A (presença `selectedShapeIds`) como contorno na cor de A (1 012 píxeis mudam quando A larga; [`c20_B.png`](../saidas/tldraw/c20_B.png)); o nome só no cursor |
| c21 | seguir = botão de olho na linha do colaborador (menu de pessoas); a câmara de B **alcança a de A em 49 ms**; interrompem: pan com a roda, zoom com Ctrl+roda; **não** interrompem: clique no vazio, Escape, trocar de ferramenta, arrastar uma forma |
| c22 | **não existe** «trazer todos»: 586 itens em 18 menus/diálogos (principal + submenus, pessoas, contexto, acções, atalhos) e nenhum casa |
| c23 | três pares: **converge sempre** e ganha o último libertado nas 6 ordens |
