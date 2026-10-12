# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W19 → `main` (2026-10-12)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> **Integra-se JUNTO com a W18…W12** ([W18](HANDOFF_INTEGRACAO_line_MiroClone_W18_2026-10-12.md) e os anteriores que ele
> lista, válidos para o que é deles). A W19 cura os três defeitos do vídeo achados no smoke do dono (12/10) e põe os
> NOSSOS controles («Nossos controles + Premium», decisão do dono). Briefing:
> [W19](HANDOFF_CONTINUACAO_W19_VIDEO_DEFEITOS_E_CONTROLES_2026-10-12.md). Mecanismo e medições:
> [`10_video_do_youtube.md`](../10_video_do_youtube.md) §W19. Decisão: [ADR-0185 §W19](../../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md)
> (emenda, não ADR novo). Sonda: [`youtube_motor/sonda_w19_tocador.mjs`](../ferramentas/youtube_motor/sonda_w19_tocador.mjs).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | ver `git log -1` (este handoff é o último commit) |
| base (merge-base) | `f4a6df7a34` = `main` de 12/10 (rebase limpo de 116 commits, sem `Solved`) |
| commits da W19 (do mais velho) | `346aa1fcf6` motor · `3a3b8d75c2` controles · `dea04028e0` cena 20+ADR+doc · `1dd3ab36bf` Premium medido · `380fc90bf3` fmt · `3eac6a1c30` auditoria · `dc0db6cf2f` portão · `e3e1c77101` mutação · + este handoff |
| diff da W19 | `git diff --stat 1a8c911eaa HEAD`: 52 ficheiros (+7078 / −226, a maior parte JSON das sondas). **`shells/` intocada** (delta 0) |
| smoke | cena **20** (`PH2D_BOARD_SMOKE=20`, `smoke_w19.rs`; roteador `max_level: 20`). Foto: `ferramentas/soltar_wayland/quadro20_depois.png` |

### O que mudou (cada defeito reproduzido VERMELHO antes da cura)

1. **Ecrã inteiro do tocador deformava** — o navegador sem ecrã imagina um ecrã 800×600; o ecrã inteiro mandava quadros
   **420×315** para a caixa 560×315 (teste real `the_player_s_fullscreen_corner_never_changes_the_frame_shape`, vermelho) e
   o `stretch` esticava. Cura: `setDeviceMetricsOverride` com `screenWidth/Height` = a caixa; o tocador com `fs=0` e sem a
   permissão `fullscreen` no `<iframe>`; o desenho `fit` (faixas, nunca esticado — gate de geometria vermelho→verde).
2. **Zoom pausava** — a sonda mediu **0 pausas** com o tamanho do tocador a mudar a cada 16 ms (8 corridas, 4 variantes
   intercaladas). A pausa era a regra «inteiro à vista» (nossa). `OnScreen::whole` → `OnScreen::visible`: vivo com
   ≥ 200×200 px **visíveis** (a *Required Minimum Functionality*). O tocador segue o tamanho do ecrã 250 ms depois de ele
   assentar (`settled`, variante medida «assenta»: 30 q/s). Real: `a_continuous_zoom_never_pauses_the_player` — 215
   quadros distintos em 5 s, 0 pausas.
3. **Canal → preto** — os links do tocador abrem uma ABA NOVA no navegador invisível, que fica à frente: **1 quadro em
   3 s, luz 9,5/255** (real, vermelho). Cura: `Target.setDiscoverTargets`; cada aba nova (não a nossa) fecha-se, a nossa
   volta à frente (`activateTarget`), o link vai para `Player::take_outside` e o app abre-o no navegador do sistema.
   Depois: 90 quadros, luz 179, o link uma vez.
4. **Os NOSSOS controles** (`ph2d-editor-core/src/screens/hero/board_video_bar.rs`, novo): tocar/pausar · tempo ·
   barra de tempo · mudo · volume · ecrã inteiro NOSSO · «Abrir no YouTube» (no ponto em que está, `link_at`). Por
   baixo da caixa, por cima se em baixo não couber, **some** se não couber — nunca à frente (o painel fica a `Xs` = 4 px
   do vídeo). Estado pela API oficial (`listening` → `infoDelivery`/`onStateChange`, `Runtime.addBinding "ph2dState"`;
   `ph2d-board-video/src/playback.rs`, novo); a ordem dada manda no botão ≤ 1,5 s (medido: `pauseVideo` chega ao estado
   em ~630–740 ms, `seekTo`/`mute`/`setVolume` em 4–10 ms). O tocador vem com
   `controls=0&fs=0&disablekb=1&rel=0&iv_load_policy=3` (`ph2d_board_embed::OUR_CONTROLS`).
5. **Teclas**: o tocador ignora-as (`disablekb=1`) ⇒ `PlayerInput::Key`/`PlayerKey`/`Player::key`/`dom_key` **saíram**;
   os atalhos do YouTube são nossos em `board_video::key` (espaço/K, ←/→ 5 s, J/L 10 s, ↑/↓ ±5, M, F; `Esc` sai do ecrã
   inteiro). No ecrã inteiro TODAS as teclas são do vídeo.
6. **Ecrã inteiro NOSSO**: a colocação do vídeo passa a ser a do ecrã inteiro (o rato e o tamanho do tocador seguem-na);
   `paint.rs` do hero não pinta `board_nav`/`board_bar` nele (nem os seus alvos); o preto e a roda (`board_view::wheel`)
   não chegam ao quadro.
7. **Roda do rato** (aberto (b) da W18): fica do quadro — `PlayerInput::Wheel`/`Mouse::Wheel` saíram.
8. **Apresentação** (aberto (c) da W18): o vídeo do diapositivo TOCA — `Videos::presenting` deixa o desenho em modo
   export registar a colocação; `board_video_bar::paint_show` regista um botão por vídeo (`show_node`) por cima da sala e
   a barra por cima de tudo; só os vídeos DENTRO do diapositivo ficam; `show::stop` pára-o; `Esc` no ecrã inteiro sai só
   dele.
9. **Premium**: medido **0 anúncios em 6** (3 monetizados × `youtube-nocookie.com`/`www.youtube.com`, 15 s, sem sessão) e
   pelo olho aos 3 s ⇒ não se constrói o perfil persistente (privacidade por nada); volta ao dono (§4).
10. **Auditoria**: `Player::load` não esquecia o estado do vídeo anterior (tempo, duração, pausa pedida) — teste
    vermelho `loading_another_video_forgets_the_previous_ones_state`, cura.

## 2. Foundational / partilhado tocado — porque é aditivo

| onde | o quê | merge |
|---|---|---|
| `ph2d-editor-core` `screens/hero.rs` | `pub mod board_video_bar;` (+1 linha) | aditivo |
| `ph2d-editor-core` `hero/document_tabs.rs` | `board_video_bar::populate` e `apply_event` na cadeia (+2) | aditivo; ordem: depois do `board_bar` |
| `ph2d-editor-core` `hero/paint.rs` | `board_nav`/`board_bar::paint` dentro de `if !board_video::fullscreen(hero)`, + `board_video_bar::paint` | toca 2 linhas existentes — conflito textual possível com quem mexer ali |
| `ph2d-editor-core` `hero/board_view.rs` | a roda consumida no ecrã inteiro (1 condição) | aditivo |
| `ph2d-editor-core` `hero/board_team_show.rs` | `presenting`, o `retain` do diapositivo, `paint_show`, `stop` pára o vídeo, `Esc` | só MiroClone |
| `ph2d-board-render` `video.rs` | `Videos` + `status`, `commands`, `fullscreen`, `presenting`; `PlayerStatus`, `VideoCommand` novos; `PlayerKey` e `PlayerInput::{Key,Wheel}` **removidos**; `stretch`→`fit` | removidos: grep de `PlayerKey`/`PlayerInput::Key`/`PlayerInput::Wheel` no workspace = 0 |
| `ph2d-board-embed` | `OnScreen::whole` → `visible` (**removido**); `OUR_CONTROLS`, `player_url_at`, `link_at` | `whole` só tinha 2 leitores (render, editor), os dois migrados |
| `ph2d-board-video` | `playback.rs` novo; `Player::{seek,set_muted,set_volume,playback,take_outside}`; `Key`/`Player::key`/`Mouse::Wheel` **removidos** | só o app lê |
| `ph2d-i18n` `board_images.rs` | 10 chaves `board.video.*` + 8 `board.smoke.w19.*` | aditivo |

**Ids novos (colisão!)**: `board_video_bar::SALT = 0xb0a2_d7ab_71de_0000` (`^ 1..=6`) e `SHOW_SALT = 0xb0a2_d7ab_72de_0000`
(`^ ElementId`). Os outros sais do quadro: `0xb0a2_d7ab_5ba2_0000` (`board_bar`), `0xb0a2_d7ab_0000_0003` (abas).
**Roteador de smoke**: `PH2D_BOARD_SMOKE` `max_level` 19 → **20** (a cena 20 conta-se no `match`; se outra linha também
criar a 20, renumere a que chegar depois).
**Contratos congelados (§6)**: nenhum — `git diff --name-only 1a8c911eaa HEAD` não toca `ph2d-nodegraph`, contratos de
tool nem o modelo vetorial congelado. `PROTOCOL`/`FORMAT_VERSION` do quadro: inalterados (nada do modelo mudou).
**Catracas baixadas / itens partilhados apagados**: nenhuma catraca; os itens apagados estão na tabela (todos da família).

## 3. O portão (1×, BASE=`f4a6df7a34`)

Verificador (`loadavg` 0,24 no início). Corrido sobre `380fc90bf3`; os 6 ✗ curados em `dc0db6cf2f` e re-corridos (os
alvos que falharam + clippy `-p` + `fmt --check`), a auditoria em `3eac6a1c30`, a mutação em `e3e1c77101`.

| item | resultado |
|---|---|
| `nextest-impacted` (BASE=merge-base) | 18 793 / 18 796 → **3 ✗ curados**: `arch_safe_clamp_only` (o salto passa por `safe_clamp_f64`), `no_magic_numeric` (os segundos dos atalhos nomeados), `no_row_label_carries_its_own_unit` (lia «(M)» como metros: «(key M)») — 411 s |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ 84 s |
| clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | 2 ✗ curados (`chunks_exact_to_as_chunks` num teste meu) → ✓ |
| `fmt --check` · `typos` (com `--force-exclude`) · `cargo deny` · `cargo audit` (1 aviso permitido, `memmap2`) | ✓ |
| `cargo machete` | 1 ✗ da W18: `image` no `servo_spike` — falso positivo com mecanismo (liga a função `png` do `image` que o Servo devolve; o machete não vê unificação de funções) → declarado no `Cargo.toml` com o porquê |
| `file_loc_cap` · `workspace_file_loc` · `node_id_collisions` · `no_literal_color` · `every_widget_event…` · `the_painted_control…` · `max_level` · i18n (23) | ✓ (maiores ficheiros: `player.rs` 549, `board_video_bar.rs` ~420, tecto 700) |
| `censos-da-arvore-combinada.sh` | ✓ 117/117, 12 censos |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | ✓ ✓ (os dois não corridos na W18) |
| navegador real (`#[ignore]`, à mão, código final, `loadavg` 5) | ✓ 6/6: régua da W18 **29,99 q/s, clique → quadro 13,7 ms**; zoom 5 s 249 quadros e 0 pausas; canal 90 quadros, luz 177; ecrã inteiro `{560×315}`; salto+volume+mudo confirmados pelo tocador em 364 ms |

**Auditoria (2 lentes).** *Correção* — CLAIM: o estado mostrado é o do vídeo activo · TRAÇO: `app::player` → `Player::load`
→ `Playback` → `Videos::status` → `board_video_bar::paint` · ASSERÇÃO-VERMELHA: `loading_another_video_forgets_the_previous_ones_state`
(**achado e curado**: o tocador reaproveitado mostrava o tempo/duração/pausa do anterior). *Regras do YouTube* — CLAIM:
nada à frente do tocador · TRAÇO: `layout` (por baixo `video.y+h+Xs`, por cima `video.y−Xs−h`) → `paint_panel` (+`Xs`) ⇒ o
painel fica a 4 px do vídeo · ASSERÇÃO-VERMELHA: `the_bar_goes_under_or_over_the_video_never_on_it`,
`our_controls_sit_under_the_video_and_each_gives_its_order` (M8 sangrou nos três). NÃO-CHECADO-PELA-COMPILAÇÃO: o
`held` do app (aberto (e)); as pegas da selecção pintam-se nos cantos do vídeo seleccionado (herdado da W18).

**Mutação (15, uma de cada vez, controlos verdes antes):** 13 sangraram (live/visible, `fs=0`, faixas, a parte visível, a
pausa pedida, o `load`, o assentar, a barra por baixo, a seta, o preto do ecrã inteiro, as barras do quadro escondidas,
o `stop` da apresentação, o 2.º clique). **M13** (registar o botão do vídeo a cada quadro) é EQUIVALENTE: medido com um
teste novo que repinta entre o Down e o Up — o clique não se perde; o comentário que dizia o contrário foi corrigido.
**M15** (a cena 20 encolhida) sobrevive: a área do teste não tem os painéis laterais (1268 px) — ver aberto (f).

## 4. Abertos (nomeados)

- (a) **Premium — decisão do dono.** Sem anúncios a tirar (0/6). Se ele vir um anúncio num vídeo do quadro, esse vídeo é
  a entrada da medição seguinte (sonda: `HOST=www.youtube.com FASES=A … sonda_w19_tocador.mjs`; o perfil persistente com
  a sessão dele só então).
- (b) **Windows**: sem tocador (abre no navegador). Porta TCP RECUSADA (ADR §W19.8); o tubo com handles herdados
  (`--remote-debugging-io-pipes`) por medir — não há Windows nesta bancada. macOS nunca corrido.
- (c) Na apresentação, clicar num vídeo TAPADO activa-o sem tocar e sem aviso (no quadro o aviso existe).
- (d) Com o ecrã inteiro ligado, um aviso do app (ex.: «não abriu o navegador») só aparece ao sair dele (as barras do
  quadro, onde o aviso se pinta, não se pintam ali).
- (f) **O enquadramento de uma cena não desconta os painéis laterais** (Hierarquia/Inspector tapam os lados do quadro na
  janela de 1280×800): a 1.ª foto da cena 20 cortava as lições das pontas (curado na cena, lições em 3+2); a cena 19
  tem o mesmo corte. É do `show_box`/`fit_box`, de todas as cenas — não desta onda.
- (e) Sem teste: o `held` do app (retomar só se tocava) — a lógica é pequena, mas o `Player` não se constrói fora da
  crate dele; o `settled` tem gate.
- Da W18, sem mudança: (d) o estado não viaja ao vivo (por desenho) · (e) sem rede, nenhum título · (f) arrastar um link
  sem pré-visualização · (g) texto dos SVG · (h) `~/Apps/w18_motores` (~190 MB) pode apagar-se · (i) o número do ADR-0185
  é provisório · (j) o gate M5 lê o FONTE.
- O `no_app.sh` corre-se agora por `PH2D_GPU=1 bash scripts/ph2d-run.sh bash …/no_app.sh` (a porta de dentro sem placa;
  antes esperava pela de fora: 300 s e `exit 75` mudos).

## 5. O integrador

- Ordem: com a W12…W18 (é a mesma linha). Reconte o ADR-0185 e o `max_level` do roteador do quadro.
- `paint.rs` do hero é o único sítio partilhado com risco de conflito textual (§2).
- Os testes do navegador real são `#[ignore]` (rede + Chrome): `bash scripts/ph2d-run.sh cargo test -p ph2d-board-video
  --release --lib -- --ignored --nocapture` — 6, verdes em 12/10 (a régua da W18 incluída).

## 6. Perfil do agente (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (11% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                164   alvo: <= 800
  ✗ cargo test : cargo check                597 : 144   alvo: <= 1,0  razao 4.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  42%   alvo: >= 80%  (685 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         348 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
`target/*/incremental` reclamado: 36,7 GB.

## 7. Smoke do dono

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_BOARD_SMOKE=20 ./target/smoke/ph2d-host-desktop
```
Precisa de rede e de um navegador da família Chrome. Prova do binário (2.ª compilação, a mesma linha de comando): `Finished smoke profile [optimized] target(s) in 0.22s`,
zero `Compiling`.
