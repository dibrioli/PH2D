# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W20 → `main` (2026-10-13)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> Briefing: [W20](HANDOFF_CONTINUACAO_W20_VIDEO_PAN_E_DECORACAO_2026-10-12.md) (os três pedidos do dono depois do
> smoke da W19). Decisão: [ADR-0185 §W20](../../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md)
> (emenda). Mecanismo e a tabela do oráculo: [`10_video_do_youtube.md`](../10_video_do_youtube.md) §W20. Oráculo:
> [`sonda_w20.mjs`](../ferramentas/excalidraw_oracle/sonda_w20.mjs) (saídas em `excalidraw_oracle/saidas/w20/`).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | ver `git log -1` (este handoff é o último commit) |
| base (merge-base) | `757afebd58` = `main` de 13/10 (a W19 já estava no main: `git cherry` = 1, só o briefing) |
| commits (do mais velho) | `86678fcb0b` briefing · `1c63fa5682` oráculo · `0391c40d9a` feat · `7208cb2c03` docs · `3b1c7731e1` fecho · + este handoff |
| diff | `git diff --stat 757afebd58 HEAD`: 56 ficheiros, +7097/−126 (a maior parte é o JSON e as fotos do oráculo). **`shells/` intocada** |
| smoke | cena **21** (`PH2D_BOARD_SMOKE=21`, `smoke_w20.rs`; roteador `max_level: 21`). Foto: `ferramentas/soltar_wayland/quadro21_depois.png` |

### O que mudou (cada pedido reproduzido VERMELHO pelo despacho real antes da cura)

1. **«Pan pausa o vídeo»**: havia TRÊS caminhos, e os três estão gateados. (a) O carregar do pan passava por `board_video::pointer_down`,
   que desactivava o vídeo, e desactivar pausava. (b) Com o vídeo activo, o Espaço era o atalho tocar/pausar
   e o quadro nunca recebia o pan. (c) A regra «≥ 200×200 VISÍVEIS» da W19 pausava o vídeo levado até à beira.
   Cura: `Videos::player` (o vídeo que o tocador tem) ≠ `Videos::active` (o rato e as teclas), porque o oráculo
   continua a tocar depois de clicar fora. O app segue o `player` (`video::target`, puro e gateado). Os pans (botão do
   meio, que também é o dos dois dedos, sintético; Espaço; mão) saltam o vídeo. O Espaço com um vídeo activo só
   toca/pausa num TOQUE (`BoardLive::space_tap`). `live_allowed` passa a ler o TAMANHO do tocador (≥ 200×200, a
   RMF do YouTube) e se há **algum** pedaço à vista: só todo fora pausa (o pedido do dono). Isto não viola o
   YouTube e é o que o oráculo faz com menos.
2. **«Decoração como um retângulo»**: a barra de estilo tem `Selected::Videos`, com fundo, cor, espessura e traço da
   borda, cantos e opacidade (o oráculo dá ao `embeddable` os grupos do rectângulo). No desenho
   (`ph2d-board-render::video`), a borda é a moldura das formas (`image_frame`, generalizada para `&Style`) no
   contorno da caixa. O TOCADOR RECUA para dentro dela pelo alcance da moldura (`player_in_box`; o oráculo recua o
   iframe pela largura do traço), e o fundo pinta-se por trás. A colocação (`Placement::to_screen`) é a do tocador,
   por isso o ponto do clique, o play e a barra seguem-no sem mudar. Os nossos controles ficam fora da borda. O
   `.excalidraw` lê a decoração de um `embeddable` de fora (antes zerava-a) e escreve-a. A entrada do gate é o
   ficheiro que o oráculo gravou (`w20_E2_gravado.excalidraw`).
3. **«Segurar e arrastar move, clicar dá play/pause»**: o carregar num vídeo, com a ferramenta de seleccionar, fica à
   espera do largar (`documents::VideoClick`) e segue para o editor. O arrasto move o vídeo, parado ou ACTIVO; o
   largar sem passar o limiar da casa (`metrics().drag`) é o clique. Num vídeo parado, activa-o; num activo e vivo,
   vai ao tocador como carregar+largar; num vídeo que o tocador já tem, é tocar/pausar. O oráculo também activa no
   LARGAR (A2), mas não move um activo (A4). Aqui move, porque com `controls=0` o tocador não tem gesto de arrastar.
4. **Achado no caminho**: o enquadramento de um vídeo pequeno (`show_box(hero, false, false, …)`) FECHAVA o minimapa
   e o painel de Frames, e só valia no desenho seguinte. Agora é `board_nav::fit_now`, no evento.
5. As lições das cenas 19 (l1: «clique fora para parar») e 20 (l2: «enquanto 200×200 à vista») ensinavam o
   contrário e foram corrigidas.

6. **Depois do smoke do dono (13/10): «ao apertar o botão de pause, o ícone de play deveria aparecer».** Pausado, a
   página do tocador mostra o ▶, mas o screencast manda 1 quadro e cala-se (a mudança é dentro da moldura do YouTube).
   Por isso o quadro ficava o de quando tocava, com o «‖». Medido no Playwright e na nossa montagem; a emulação de
   foco e as flags anti-«background» foram recusadas por medida. Cura em `ph2d-board-video::Player`: fotos directas
   (`Page.captureScreenshot`) a +150/+500/+1200 ms depois de cada ordem, clique, `load` e `onStateChange`, que entram
   como quadros. Portão real `pausing_brings_the_player_s_play_mark_to_the_board` (12–15 quadros depois da pausa
   confirmada; sangra para 1 com as fotos desligadas). Os 13 testes do tocador passam no navegador real: régua da
   W18 30,00 q/s, clique → quadro 9,5 ms.

7. **Depois do smoke (13/10): «Mais vídeos» abria o vídeo no navegador e os dois tocavam sobrepostos.** O dono pediu
   primeiro TIRAR o link, o «Mais vídeos» e o logotipo — recusado com o texto das regras do YouTube (ADR-0185 §W20.8).
   Escolheu (pergunta com as duas opções) **«troca na mesma janela»**, e depois pediu **setas ‹ › e «voltar ao
   original»**. Medido (`sonda_w20_mais_videos.mjs`): a sugestão abre `youtube.com/watch?v=<outro>&feature=endscreen`
   numa aba nova e o de origem continua a tocar. Cura: `app::video::outside` (outro vídeo → a mesma janela; o mesmo →
   navegador + pausa; o resto → navegador), `app::video::tune` (recarrega quando a mesma janela muda de vídeo),
   `board_video::{navigate, step, reset, trail}` + `documents::VideoTrail` (a verdade é o documento; o `Ctrl+Z` anda
   no mesmo trilho; sugerir depois de recuar corta o que estava à frente), `Ctl::{Back, Forward, Reset}` na barra (no
   FIM dos ids; só com navegação; a seta sem destino pinta-se apagada). Cena 21: lição 7. Mutações: 6, 5 sangram, e
   a 6.ª era equivalente (a guarda «≥ 2 no trilho» era código morto e saiu).

## 2. Foundational / partilhado tocado

| onde | o quê | merge |
|---|---|---|
| `ph2d-editor-core` `documents.rs` | `VideoClick` (struct nova), `BoardLive::{video_click, space_tap}` | aditivo |
| `ph2d-editor-core` `hero/board_nav.rs` | `fit_now` (fn nova) | aditivo |
| `ph2d-editor-core` `hero/board_bar.rs` / `board_bar_look.rs` | `Selected::Videos` (variante nova, no FIM) e o grupo dela; na lista de ids não entra item novo (todos já existiam nas formas) ⇒ nenhum id se move | aditivo |
| `ph2d-editor-core` `hero/board_video.rs`, `board_video_bar.rs`, `board_team_show.rs` | o gesto, a barra fora da borda, `stop` limpa o `player` | só MiroClone |
| `ph2d-board-render` | `Videos::player` + `Videos::activate`; `player_in_box` exportado; `image_frame::{reach,paint}` recebem `&Style`; `video::paint` recebe o id do quadro | `Videos` só se constrói por `Default` (grep: nenhum literal) |
| `ph2d-board-embed` | `live_allowed`: a LEI mudou (tamanho do tocador + algum pedaço à vista) | 2 leitores (render, a barra no ecrã inteiro), os dois do quadro |
| `ph2d-board-excalidraw` `video.rs` | `foreign` deixa de zerar fundo/borda | só MiroClone |
| `ph2d-board-video` `player.rs` | as fotos directas (`Shared::{shots, changed}`, `SHOTS_AFTER`) | só MiroClone |
| `ph2d-editor-core` `documents.rs` (W20.7) | `VideoTrail`, `BoardLive::video_trails` | aditivo |
| `ph2d-i18n` `board_images.rs` (W20.7) | `board.video.{back,forward,reset}`, `board.smoke.w20.l7` | aditivo |
| `ph2d-i18n` `board_images.rs` | 8 chaves `board.smoke.w20.*`; 2 textos de lições corrigidos | aditivo |

**Roteador de smoke**: `PH2D_BOARD_SMOKE` `max_level` 20 → **21** (a cena 21 conta-se no `match`; se outra linha também
criar a 21, renumere a que chegar depois). **Ids/sais novos**: nenhum. **Contratos congelados (§6)**: nenhum.
`PROTOCOL`/`FORMAT_VERSION` do quadro: inalterados (o `Style` do `Embed` já existia; nada no modelo mudou).
**Catracas baixadas / itens partilhados apagados**: nenhum (o `Pending` que existiu entre commits saiu no fecho).

## 3. O portão (1×, BASE=`757afebd58`)

Verificador, máquina PARTILHADA e carregada (`loadavg` 21 → 54 → 21 durante o portão, outras linhas a compilar).

| item | resultado |
|---|---|
| `nextest-impacted` | 17 305 / 17 307 → **2 ✗, determinísticos, curados em `3b1c7731e1`**: `architecture_the_foundation_modules_form_a_dag` (a aresta `documents → screens` do `Pending`; a cura leva o tipo para `documents`, sem re-export) e `no_tofu_glyphs_in_ui_strings` (a seta `→` em duas mensagens de teste). Re-corridos os dois + os 13 do vídeo + a cena 21: ✓ |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ 73 s |
| clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | ✓ 72 s; depois do fecho, `clippy -p ph2d-editor-core -p ph2d-app-board --all-targets -D warnings` ✓ |
| `fmt --check` · `typos` · `machete` · `deny` · `audit` | ✓ |
| `censos-da-arvore-combinada.sh` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | ✓ ✓ ✓ |

**Mutação (16, uma de cada vez, controlos verdes antes, restauro com `touch`): 16 sangraram.** Na 1.ª passagem, a M4
(apagar «algum pedaço à vista») sobreviveu no teste do EDITOR, onde um vídeo todo fora não se desenha e por isso a lei
é redundante ali. Sangra na tabela da lei (`ph2d-board-embed`, `tests.rs:161`). A M14 (a barra sem o alcance da borda)
sobreviveu porque a borda da barra de estilo sai menos que a folga `Xs` da barra. O teste passou a usar uma borda de
24 e sangra (`board_video_tests.rs:543`).

**Auditoria.** *Correção*. CLAIM: desactivar não pausa. TRAÇO: `board_video::pointer_down` (fora → `active = None`)
→ `render::video::paint` (`live = player == id && live_allowed`) → `app::video::target` (segue o `player`) → `hold` só
se `!live`. ASSERÇÃO-VERMELHA: `clicking_outside_takes_the_mouse_not_the_picture` (M2) e
`the_player_follows_the_video_it_holds_not_the_active_one` (M15). *Regras do YouTube*. CLAIM: nada à frente do
tocador. TRAÇO: `player_in_box` (recuo = alcance da moldura) → a camada recorta o tocador num rectângulo concêntrico
→ a moldura no contorno. ASSERÇÃO-VERMELHA: `a_framed_video_keeps_its_border_and_fill_around_the_player_never_on_it`
(M9–M11). NÃO-CHECADO-PELA-COMPILAÇÃO: o carregar+largar entregue no LARGAR chega ao YouTube como clique (os
testes reais `#[ignore]` da W18/W19 já mandam os dois seguidos e o tocador responde; não re-corridos nesta onda,
porque o `ph2d-board-video` não mudou).

**Portão re-corrido (13/10, depois de W20.6 e W20.7, BASE=`757afebd58`, `loadavg` 19 → 76 → 58):** `nextest-impacted`
17 311/17 311 ✓ · `CARGO_BUILD_WARNINGS=deny` check ✓ · clippy do `ship.sh` ✓ · fmt · typos ✓ · censos 117/117 ✓ ·
machete · deny · audit ✓. Os 13 testes reais do tocador (`--include-ignored`) ✓ depois de W20.6.

## 4. Abertos (nomeados)

- (a) **As pegas da selecção pintam-se nos cantos do vídeo seleccionado** (herdado da W18, ainda vivo): com o tocador
  vivo, são «algo à frente dele». A borda nova não as tapa.
- (b) Um clique LONGO (sem mexer) num vídeo conta como clique. O oráculo não activa com 600 ms parado (A2). Seguimos
  só o limiar de distância da casa, por desenho.
- (c) Com outra ferramenta (caneta, forma), o vídeo não toma o carregar: a ferramenta desenha por cima dele, e o
  traço tapa-o, logo pausa pelas regras. Medir com o dono se ele espera outra coisa.
- (e) O trilho das setas é da SESSÃO (o documento guarda o vídeo de agora): fechar e abrir o projecto perde o
  «voltar ao original».
- (f) O canal aberto fora não pausa o vídeo daqui (a W19 decidiu assim; a página de um canal pode tocar o trailer).
- (d) Fundo com preenchimento à mão (tracejado) num vídeo: pinta-se sempre sólido (num anel fino o tracejado não diz
  nada). O `fill_style` viaja no ficheiro, mas não se desenha no vídeo.
- Da W19, sem mudança: (a) Premium · (b) Windows/macOS · (c) clicar num vídeo TAPADO na apresentação · (d) o aviso no
  ecrã inteiro · (f) o enquadramento das cenas não desconta os painéis laterais (a cena 21 também abre com o vídeo
  abaixo de 200 px na janela de 1280×800, e o 1.º clique enquadra-o) · (e) o `held` do app.

## 5. O integrador

- Ordem: só esta linha mexe no vídeo. Reconte o `max_level` do roteador do quadro.
- O vídeo toca no navegador real: `bash scripts/ph2d-run.sh cargo test -p ph2d-board-video --release --lib -- --ignored --nocapture`.

## 6. Perfil do agente (`bash scripts/agent-loop-profile.sh`)

```
  ✓ respostas por sessao (mediana)                154   alvo: <= 800
  ✗ cargo test : cargo check                575 : 130   alvo: <= 1,0  razao 4.4x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  46%   alvo: >= 80%
  ✗ contexto relido por passo (media)         364 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## 7. Smoke do dono

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_BOARD_SMOKE=21 ./target/smoke/ph2d-host-desktop
```
Precisa de rede e de um navegador da família Chrome. Prova do binário: ver o fim deste ficheiro.

**Prova do binário** (2.ª compilação, a mesma linha de comando, depois de reclamar ~28 GB de `target/*/incremental`):
`Finished smoke profile [optimized] target(s) in 0.27s`, zero `Compiling` (refeita depois da cura do ▶, `8099a1da18`).
