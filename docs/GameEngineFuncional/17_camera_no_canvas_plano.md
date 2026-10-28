# Plano GEF 17 — a câmera no canvas: ver e mexer no que o jogo vai filmar

Pedido do dono (27/10): *«a câmera tem o gizmo de objeto vazio; não podemos visualizar no canvas a área que ela
filma. Pesquise como as melhores engines a mostram e os desejos dos utilizadores — upgrade não de complexidade, mas
de aparência e funcionalidade»*. Antecessor: [plano 11](11_camara_e_parallax_plano.md) e o handoff
[`HANDOFF_CONTINUACAO_2026-10-25_11_CAMARA_E_PARALLAX.md`](handoffs/HANDOFF_CONTINUACAO_2026-10-25_11_CAMARA_E_PARALLAX.md).

## O que já existe (medido 27/10, não reconstruir)

- `ph2d_ecs::GameCamera { height_world (10 m), offset, priority, active, cull_mask, dolly }`, `CameraFollow { target,
  damping, dead_zone, lookahead, offset }`, `CameraLimits { min, max }`, `CameraRuntime` — `crates/ph2d-ecs/src/camera_2d.rs:88-192`;
  `half_extent(height, aspect)` `:221`. A lei do quadro: `ph2d-app-components/src/camera_2d.rs:110`, chamada de
  `shells/desktop/src/render_loop/fase_game_camera.rs:90` (a vista só se escreve com «Look through»).
- Inspector: secções Game Camera (com «Look through»), Follow, Limits — `ph2d-panel-inspector/src/sections/camera.rs`.
- No canvas: só o anel do objecto vazio (`render_loop/empty_object_overlay.rs:52`). **Nenhuma moldura da câmera.**
- ⚠️ **O aspecto mente hoje:** a janela do jogo abre a 1280 × 720 (`janela_do_jogo.rs:31`), mas a pré-visualização
  no editor usa o aspecto do CANVAS (`ph2d-app-game/src/encaixe.rs:11`). É a queixa nº 3/4 dos utilizadores da Godot
  (#91741, #38316): o retângulo tem de vir do cálculo da janela do jogo, nunca do editor.

## O que as engines fazem e o que os utilizadores pedem (pesquisa 27/10)

| engine | no canvas | edita-se no canvas | miniatura | zonas de seguimento |
|---|---|---|---|---|
| Godot 4.7 | 3 retângulos por cor: ecrã (magenta), limites (amarelo), margens (ciano); a activa em linha grossa | limites com alças (4.5+), sem roubar a seleção | só por plugin | margens e limites, só linha |
| Unity | retângulo branco SÓ com a câmera seleccionada; Pixel Perfect: área real + referência tracejada | mover | **Cameras overlay** (miniatura num canto) | Cinemachine: zonas PINTADAS (livre · suave azul · proibida vermelha), bordas arrastáveis |
| Construct 3 / GDevelop / Defold | um retângulo do tamanho da janela; GDevelop: botão «Mask» escurece o lado de fora | não | não | não |

Desejos recorrentes (fontes no relato da pesquisa): miniatura do que a câmera vê (godot-proposals #7627); ver a área
SEM seleccionar a câmera (#11597); o retângulo que mente (aspecto/escala do editor); limites com o rato (#11527); não
seleccionar a câmera ao clicar dentro dos limites (#104190); zona morta visível e arrastável; pré-visualizar outras
proporções (telemóvel, 21:9); saber onde fica a borda do ecrã enquanto se desenha o nível.

## A proposta (aparência e função, nenhuma complexidade nova no modelo)

- **A1 — o ícone:** a câmera deixa o anel do vazio e ganha um ícone de câmera no centro.
- **A2 — a moldura sempre à vista:** o retângulo EXACTO que o jogo vai mostrar (do tamanho e aspecto da janela do
  jogo, a mesma conta do `half_extent`), ténue sem a câmera seleccionada, forte com ela; a câmera activa distinta das
  outras; rótulo no canto («Camera · 17,8 × 10 m»). Cura junto o aspecto que mente (editor e jogo pela MESMA conta).
- **A3 — mexer na moldura:** arrastar um canto = zoom (`height_world`, a proporção fica); arrastar dentro = mover a
  câmera. Clicar dentro da moldura NÃO rouba a seleção de quem está por baixo (lição da Godot).
- **A4 — limites com o rato:** com a câmera seleccionada e Limits ligado, o retângulo dos limites aparece numa 2.ª cor
  e arrasta-se por bordas, cantos e centro.
- **A5 — a zona morta à vista** (a do Follow): pintada dentro da moldura (sem tinta = livre), bordas arrastáveis.
- **A6 — «o que a câmera vê»:** uma miniatura num canto do canvas, ao vivo, com a câmera seleccionada (ou fixada).
- **A7 — escurecer o lado de fora:** um botão no canvas que escurece tudo o que fica fora da moldura.

## Decisões do dono (27/10, não reabrir)

1. **Os blocos: escolha técnica (delegada).** Onda 17 = **A1 + A2 + A3 + A4 + o formato do jogo (D-F) + a escolha
   da visibilidade (D-V)**. A5 (zona morta), A6 (miniatura) e A7 (escurecer fora) = onda 18.
2. **D-F — o PROJECTO ganha o formato do jogo**, numa lista de predefinições: 16:9 · 16:10 · 4:3 · 3:2 · 21:9
   (cinema) · 9:16 (celular em pé) · iPad (4:3, deitado e em pé) · retro (Game Boy 160 × 144 · NES 256 × 240 ·
   SNES 256 × 224 · Game Boy Advance 240 × 160 · Mega Drive 320 × 224 · PICO-8 128 × 128) · personalizado (L × A).
   A janela do jogo, a moldura no canvas e o «Look through» usam TODOS a mesma resolução (uma conta só).
   ⚠️ Medir antes: onde mora a configuração do projecto (`ProjectSettings`/`hero.project`) e se gravá-la muda a
   forma do `ProjectFile` (⇒ degrau do `PROJECT_SCHEMA`, recontado).
3. **D-V — no painel da câmera, as duas opções:** a moldura **Sempre** (ténue sem seleção) ou **Só selecionada**.
   O formato do jogo também se escolhe ali (é do projecto; o painel da câmera é a porta).

## Gates (por nome) e fotos

A moldura no canvas = o retângulo do jogo (a mesma conta, aspecto da janela do jogo, em 16:9 e noutro aspecto) ·
arrastar o canto muda só `height_world` e mantém o centro · clicar dentro da moldura não muda a seleção · alças dos
limites escrevem `CameraLimits` e nunca invertem · a moldura segue a câmera durante o Play. Fotos: a câmera sem e com
seleção, o canto arrastado, os limites, a mesma cena no Play (a moldura = o ecrã).
