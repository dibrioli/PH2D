# 00 — MISSÃO: os melhores pincéis do mundo no Painter (contrato das 5 janelas)

> Leitor: cada uma das 5 janelas do Claude (e os agentes que elas lançam). Escrito a 2026-10-19 depois do smoke do
> P53. Este é o CONTRATO comum; cada janela tem o seu `JANELA_<n>.md`; o estado vivo é `ESTADO.md` (nesta pasta).

## §0 — A ordem do dono (19/10, verbatim no essencial)

*«Vc tem em suas mãos um módulo Painter extremamente capaz que reúne 4 motores de pintura que juntos podem criar
praticamente qualquer pincel que exista. Vai definir vários tipos de pincéis úteis, inspirados nos grandes apps —
Procreate, Krita, Photoshop e Rebelle, dentre outros. Pode usar qualquer um dos motores. Busque os pincéis mais usados
e reproduza-os na nossa engine — uma grande quantidade dos melhores e mais amados. Faça tudo que for necessário,
inclusive instalar apps, estudá-los. Ao acabar, tenhamos os melhores pincéis do mundo.»* E: *«você vai completar
sozinho, sem minha ajuda e sem solicitar nenhuma permissão. Qualquer solicitação de permissão está proibida pois não
haverá ninguém aqui para permitir.»*

O veredito dele sobre o P53 (a lição que manda em TUDO aqui): **Filbert = nada novo · Flat Oily e Knife Smooth =
bizarros e inúteis · «faltou um trabalho sério de teste e comparação com o Rebelle».** Os 11 passavam TODAS as
réguas do traço recto. ⇒ [`feedback_a_brush_that_passes_the_ruler_can_still_be_useless`](../../../project-memory/feedback_a_brush_that_passes_the_ruler_can_still_be_useless.md).

## §1 — Autonomia (inegociável)

- ⛔ **Nunca** `AskUserQuestion`, nunca «aguardo a sua decisão», nunca parar à espera do dono. Decisão técnica = padrão-
  ouro (CLAUDE.md §0.6); decisão de PRODUTO = o critério do §3 aplicado e escrito no `ESTADO.md` com o porquê.
- Um comando recusado por um GUARDA do repo (hook R5–R8, `ph2d-run`) não é pedido de permissão: leia a mensagem e
  corrija o comando. Nada de `sudo`. Instalar app: `pacman`/`paru` sem root não dá ⇒ AppImage/tarball em `~/Apps/`,
  `pip --user`, Wine prefix próprio; se exigir root, ESCOLHA outro caminho e registe.
- ⛔ **Nenhum comando pode PARAR à espera de autorização** (dono, 20/10: um «allow Bash command» de um agente parou a
  missão). Há um hook local que aprova tudo (`~/.claude/hooks/aprova-sem-perguntar.sh`, em `.claude/settings.local.json`;
  os guardas do repo continuam a recusar), mas não confie só nele — as formas que disparam o pedido estão PROIBIDAS a
  janelas e agentes: `cd X && …` (caminhos absolutos), `bash -c '…'`/`sh -c` (escreva um `.sh` e corra-o),
  `rm -rf` com variável/glob (`rm -r -- /caminho/literal`), `kill` de PID que não gravou, escrever em `.git/`,
  `.claude/`, `~/.bashrc`. Ponha esta linha no briefing de TODO agente.
- ⛔ Continua proibido (CLAUDE.md §0.7): `git push`, integrar no `main`, tocar noutras worktrees, matar processos de
  outras linhas (`kill` do pai de um órfão = logout do dono), `spectacle` em `kwin --virtual` (use `import -window`).
- Árvore: SÓ `Worktrees/line-PainterWatercolor` (branch `line/PainterWatercolor`), Modo L. FASE 0 de toda janela e de
  todo agente: `cd … && pwd && git branch --show-current`. Commits por caminho, `--no-verify`, `git add` dos novos.

## §2 — O material

- **Os 4 motores** (`PaintMedia` em `crates/ph2d-tool-painter/src/tool/paint/media.rs`): **Digital** (o carimbo do
  `ph2d-painter-brush`: forma, grão, jitter, taper, cápsula), **Watercolor** (lavagem óptica Beer–Lambert, orla,
  granulação), **Impasto** (espessura, relevo, 10 ferramentas, luz), **Wet Paint** (`ph2d-wet-paint`: fluido, fios,
  papel, secagem, K–M). O dono liberou o USO dos 4 (supera a decisão de 07/10); o COMPORTAMENTO que já existe nos 3
  antigos fica byte-idêntico (a cerca da linha) — capacidade nova neles entra ADITIVA, desligada por omissão.
- **Os presets**: `assets/brushes/WetPainter/<meio>/<grupo>/<nome>.ph2d-brush.png` (`ph2d-brush-presets`), receita em
  `crates/ph2d-tool-painter/tests/it/receitas_de_fabrica.rs`, gerador `presets_de_fabrica.rs`, portão por foto
  `midias_de_fabrica.rs` (+ a lei dos IRMÃOS). 28 de fábrica hoje (eram 31; o Filbert e as lâminas saíram, 53 §8). Ferramentas da régua:
  `docs/Painter/NewFeaturesWetPaint/ferramentas/` (`avalia_53.py`, `identidade_53.py`, `prancha_53.py`,
  `tamanho_dos_presets.py`, `fotos_lote.sh`, `midias_regua.py`). Medição e recusas: docs 18, 52, 53 da mesma pasta.
- **Oráculos** (CLAUDE.md §0.9: um app é um programa que se CORRE sobre entradas NOSSAS, nunca um fonte que se lê;
  passo 1 = licença por artefacto, `pacman -Qi`): **Rebelle 8** sob Wine (porta WebSocket; ⚠️ o teste expira ~06/11 —
  as fotos dele são a PRIMEIRA coisa a tirar), **Krita 6.0.4** (GPL: só correr, por script Python/linha de comando),
  **MyPaint 2.0.1 + libmypaint 1.6.1 (ISC) + mypaint-brushes 2.0.2** (licença por ficheiro: o que for CC0/permissivo
  pode ser PORTADO). ⛔ **GIMP NÃO é referência** (dono 19/10: *«GIMP 3 é fraco. Estude os fortes»*). **Os FORTES
  são o alvo**: Procreate, Photoshop (+ Fresco), Krita, Rebelle, **Clip Studio Paint**, **Corel Painter**,
  **ArtRage**. Os que têm versão de avaliação para Windows tentam-se sob **Wine** como o Rebelle (`~/Apps/<app>/`,
  prefixo próprio, a porta de automação de cada um — script/plugin/injeção de eventos numa tela virtual), com a
  licença/EULA da avaliação lida e registada antes; o que não correr (Procreate é só iPad; Photoshop/Painter recentes
  podem falhar no Wine) tem como referência o comportamento DOCUMENTADO (manuais oficiais, Brush Studio do Procreate,
  as páginas da Adobe/Corel) e fotos/vídeos públicos como alvo VISUAL. ⛔ Nunca copiar ficheiros de pincel, texturas
  ou código de nenhum deles (EULA/copyright) — copia-se o COMPORTAMENTO medido; grão e forma desenhamo-los nós.
  Código GPL ⇒ `/pd-cleanroom`.

## §3 — O EXAME do pincel (a barra; um pincel só entra se passar os SEIS)

1. **Oráculo lado a lado** numa BATERIA de gestos igual nos dois lados: recto lento e rápido · curva em S · vertical ·
   espiral · rampa de pressão 0→1 · tamanhos pequeno/médio/grande (ex. 5/40/200 px) · cruzamento sobre outra cor ·
   sobre papel e sobre tinta seca. Uma prancha por pincel (oráculo · nosso · vizinho mais perto da biblioteca).
2. **Métricas** onde o oráculo corre (a régua dos docs 18/52/53, generalizada aos gestos) — mas ⚠️ régua verde é
   condição necessária, NUNCA suficiente (a lição do P53).
3. **Distinto da biblioteca INTEIRA** (não só dos irmãos novos): matriz contra todos os presets; o vizinho mais perto
   tem de estar claramente longe, e a prancha tem de o MOSTRAR. Pincel que um existente já pinta = não entra.
4. **Razão de uso** numa linha: para quê um artista o escolhe (o que nenhum outro da biblioteca faz).
5. **Revisão cega por um agente que NÃO o fez**: recebe a prancha sem rótulos, diz qual é o oráculo, lista TODA
   diferença visível. Diferença visível ⇒ cura-se (valores ou motor) ou o pincel é reprovado. ⛔ Nunca «aberto» num
   pincel entregue (as lâminas do P53 saíram com o defeito escrito num «aberto»).
6. **Uso real**: pintar com ele a 60 Hz até ao tamanho grande (meça `ms/quadro` no perfil `smoke`), com a caneta
   (pressão) e com o rato; sem estouro de memória (o Dabber do P53: 48 GB antes do conserto).

Recurso de motor em falta para passar o exame ⇒ **implementa-se o motor** (aditivo, medido, com portão e mutação) ou
o pincel não sai. Pincel do P53 que falha o exame (Filbert, Flat Oily, Knife Smooth — veredito do dono) sai da
fábrica ou é refeito (a lâmina que gira com o arrasto é motor: `26_ABERTOS` C38).

## §4 — Como trabalhar sem esgotar o contexto

- A janela principal é o COORDENADOR: decide, desenha, revê. O mecânico vai para agentes (`.claude/agents/`:
  `explorador`, `verificador`, `mutacao`, `documentador`, `integrador`; `general-purpose` para implementar uma família
  de pincéis de ponta a ponta). Cada agente recebe um briefing auto-suficiente (árvore, regras, o §3, o que entregar)
  e devolve só a conclusão. Agentes independentes em PARALELO, numa mensagem.
- ⚠️ Recursos partilhados: a fatia da linha é UMA (24 G, `ph2d-run`) ⇒ no máximo **2** comandos pesados ao mesmo
  tempo; o Rebelle é UMA instância (uma tela virtual) ⇒ UM agente de fotos de cada vez (`gotas_inicia.sh` com
  `PH2D_GPU=1`, fecho só por `gotas_fecha.sh`, `inj.sh "rect e"` antes de cada lote); Krita/MyPaint/GIMP também por
  fila. Medições num lote (CLAUDE.md §0.10).
- O aviso de 300 mil tokens por passo (hook) = hora de passar a vez: escreva o estado e sinalize a próxima janela
  (§5) — mesmo a meio de uma fase. A janela seguinte continua a MESMA fila.

## §5 — O protocolo entre janelas

- **`ESTADO.md`** (≤ 8 KB, nesta pasta): a FILA da missão — o que está feito (com SHAs), o que está a meio, a ordem do
  que falta, as decisões de produto tomadas e porquê. Toda janela o lê primeiro e o actualiza antes de sair.
- **Passar a vez**: (1) commit de tudo; (2) `HANDOFF_JANELA_<n>.md` (≤ 6 KB); (3) `ListAgents` e `SendMessage` à
  sessão cujo nome/título tenha «JANELA <n+1>» com: `SINAL: JANELA <n+1>, comece. Leia docs/Painter/Pinceis/ESTADO.md e
  HANDOFF_JANELA_<n>.md.`; (4) crie `SINAL_JANELA_<n+1>.ok` com o SHA. Se a sessão seguinte não for achada ou a
  mensagem falhar, ⇒ **NÃO pare**: continue nesta janela a fila (lendo o `JANELA_<n+1>.md`) — a missão nunca fica à
  espera de ninguém.
- As janelas 2–5 nascem em espera: respondem «JANELA <n> pronta — aguardo o sinal» e param. Ao receber o sinal
  (`<cross-session-message>`), começam pela FASE 0.
- A JANELA 5 não passa a vez: fecha a missão (o `JANELA_5.md`). Se o trabalho não couber, ela continua até caber.

## §6 — O que é «missão cumprida»

Uma biblioteca de fábrica grande (alvo: **80–120 pincéis** depois de cortar os duplicados), organizada por categoria
como os grandes apps a organizam, cada pincel com o EXAME do §3 passado e registado (prancha + linha no catálogo), os
motores com o que faltava implementado e gateado, o fecho batched verde, o handoff de integração, e o smoke do dono
pronto (o binário compilado, um tutorial em passos com a foto de cada categoria). Integração e envio continuam
**só por ordem do dono**.
