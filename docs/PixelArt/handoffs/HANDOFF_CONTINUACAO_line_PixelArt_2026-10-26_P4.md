# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P4 (Frames, cels e linha do tempo) + abertos da P3 — 2026-10-26

> Leitor: a próxima LLM que assume a linha numa janela nova. A onda está **fechada e com o smoke do
> dono APROVADO (26/10: «smoke OK»)**; a **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada
> integrado, nada pushado. Lei do módulo: [ADR-0192](../../architecture/decisions/0192-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) (**§11 = esta onda**)
> · [plano](../02_plano_modulo.md) (§5 P4 com a nota do que entrou) · anteriores: [P3](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P3.md) · [P2](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P2.md) · [P0](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou). **66 commits** na linha; os desta onda são `2e475015d3..HEAD` (12 + este handoff), 75 ficheiros, +5599/−287.

## 2. O que esta onda fez

**Abertos da P3.** (a) Enquadramento: `PixelApp::frame_in(…, area)`; `frame_of` passa `ph2d_app_host::canvas_area::frame_target`; o fit SEGUE a área enquanto o painel abre e cai quando o artista mexe na vista/começa um gesto/sai (`PixelApp::fit`); gate `entering_draw_frames_the_sprite_in_the_free_area…` (RED primeiro). Foto `smoke_P4_cena7_dono_2026-10-25.png`: a cena 7 inteira no layout do dono. (b) GPU MEDIDA e recusada por agora (`mede_compose`, load ~6): 64²×5 0,32 ms · 256²×8 9,2 · 512²×20 123 ms num núcleo → **16,7 ms em 8 faixas** · 1024²×20 65 ms em faixas. A vista compõe em faixas paralelas (`compose_bands`, metade dos núcleos) e guarda cada frame composto por versão (`FRAME_CACHE_BYTES` 256 MiB, o recurso é a RAM): tocar não recompõe. Gatilho escrito: ≥ 1024² com muitas camadas.

**Passo 1 — MVP em JSON:** `ferramentas/extrai_quadros_mvp.mjs` → `tests/fixtures/mvp_frames.json` (frame ops com undo/redo, frameAtTime, tagPlayOrder, tags × insert/delete/move). Os testes doc/persist/history_selection do MVP são da sessão do navegador, não leis de frame — não extraídos.

**Passo 2 — Trial observado** (`trial_ase_quadros.py` escreve 39 `.ase`; `trial_quadros.py` lê a tela / grava vídeo 50 q/s do tocar; `trial_fit_quadros.py` ajusta; `trial_fixture_quadros.py` → `oracle_trial_frames.json`). Leis, todas EXACTAS: opacidade do cel = `mul(camada, cel)` (7/7); ordem `(índice + z, z)` com a de baixo na ordem (10/10); papel vegetal Normal com `max(0, base − step·(d−1))` (68/28, 1+1), POR TRÁS do frame inteiro (um fundo opaco noutra camada esconde-o — `on_camadas.ase`), tinta Rec.709 `((255+v)/2, v/2, v/2)` / espelho azul (6144/6144 × 4 variantes); tocar: repeat 0 = laço na tag, N = N passadas e sai para `to+1`, ping-pong alterna sem repetir a ponta, começar no fim da 1.ª passada conta-a feita (17 sequências). Cel ligado: o Trial SEPARA ao abrir ligados com posição/opacidade diferentes; `z` diferente fica ligado ⇒ ligado partilha imagem+posição+opacidade, `z` é do cel. ⚠️ Trial: o `aseprite.ini` controla o papel vegetal (`[onionskin] prev_frames/next_frames/opacity_base/opacity_step/type/position`); a janela sem WM precisa `Frame = 0 0 1600 1000`; fechar a janela Preview; `Home` antes de navegar (o Trial lembra o frame por ficheiro); ⛔ nunca `pkill -f Aseprite` (mata o próprio shell) — `kill` por PID.

**Núcleo `ph2d-pixel-core`:** `Cel{opacity, z, link}` (`Cel::image`/`Cel::linked`; `Sprite::cel` resolve a ligação, `entry` é o guardado, `owner`, `linked_frames`); `Tag`/`Direction`; `Layer.continuous`; `Op::Frames(FrameSet{frames, cels, tags})`; `check_cels` no estado FINAL (camadas, frames, whole; dono vazio só com ligados) + `check_frames`; o Patch escreve no dono e guarda opacidade/z; `frames.rs` (insert/new_frame Copy·Empty·Linked, delete com a imagem passada ao 1.º ligado que fica, move_to, reverse, set_duration 1..=65535, set_cel, link, unlink, new/edit/delete_tag; tags por POSIÇÃO com a lei do MVP); `play.rs` (`Play::start/step/advance`, `PlayOptions{once, all}`, `tag_at`); compose achatado com `(índice+z, z)` e a opacidade efectiva; `Onion`, `ghosts`, `onion_tint`, `compose_with_onion`; merge_down respeita ligados e a opacidade do cel; formato **PH2DPIXEL 1.2**.

**Vista `ph2d-pixel-render`:** `SpriteImage::refresh_with(…, onion)` chaveada por (versão, frame, papel vegetal); faixas; cache de frames.

**Família `ph2d-app-pixel`:** `quadros.rs` (pedidos → um passo cada; navegação com volta; `tick(ms)` do tocar — o relógio real em `frame_of`; `onion_view` só parado e em Draw; `timeline()` do instantâneo); `Doc.play`, `Doc::sane_frame` depois de cada commit/undo/redo; teclas ← → Home End Enter F3 Tab; o painel `pixel_timeline` abre/fecha com o Draw e a linha do tempo DA CASA cede-lhe a banda (guardada e devolvida — `timeline_casa`); `smoke_quadros.rs` = **cena 8**.

**Painel `ph2d-panel-pixel-timeline` (novo):** camadas × frames na banda de baixo (`Slot::Bottom`, banda reservada em `frame_layout.rs` também para ele), barra de uma linha com ícones, tags, pílulas dos ligados, «z», opacidade, propriedades (Frame/Cel/Tag), rolar com a roda; a linha de camada partilhada mudou-se para `ph2d-editor-core/src/pixel_layer_row.rs` (um painel não depende de outro — `architecture_cycle_prevention`); o rodapé ⑤ sobe para cima da banda (`layout.rs::reserve_bottom_strip`). Vocabulário em `pixel_panel.rs` (`PixelFrameCol`, `PixelCel`, `PixelTagInfo`, `PixelDirection`, `PixelFrameAction`, pedidos novos) e id `PANEL_PIXEL_TIMELINE`.

## 3. Smoke do dono — APROVADO em 26/10

Cena `PH2D_PIXEL_SMOKE=8` (64×32, 16 frames, 4 tags Forward×2 · Reverse×2 · Ping-pong×2 · Ping-pong Reverse ∞, frame 4 a 480 ms, z +1 à frente do pilar no verde, cel a 50 % no azul, chão *continuous* ligado nos 16, papel vegetal ligado com tinta). Fotos: [`smoke_P4_cena8_2026-10-25.png`](smoke_P4_cena8_2026-10-25.png) (fábrica) · [`smoke_P4_cena8_dono_2026-10-25.png`](smoke_P4_cena8_dono_2026-10-25.png) (`HOME=/home/enio`, `layout.txt` sha256 `4db94086…` igual antes/depois). As fotos apanharam 2 defeitos antes do dono: a banda baixa demais (3.ª camada cortada, rodapé por cima) e, no layout do dono, a linha do tempo da casa a ocupar a banda — ambos curados.

## 4. Superfície de colisão NOVA

Soma-se às da [P3 §4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P3.md). Contadores ECS/schema/`MODULE_TRUTHS` NÃO mudaram (blob PH2DPIXEL 1.1 → 1.2).
- `ph2d-editor-core`: `pixel_panel.rs`, `pixel_layer_row.rs` (novo), `screens/hero/{panel_ids.rs, hero.rs, paint.rs, frame_layout.rs}`, `screens/layout.rs` (`reserve_bottom_strip`: o rodapé sobe sobre QUALQUER banda de baixo — afecta a linha do tempo da casa), `ids/chrome/pixel.rs`, `widget/scrollbar_ids.rs` (`NodeId(853)`).
- Registo do painel: `ph2d-panel-registry-init` (Cargo + `EXPECTED_TYPED`), `shells/desktop/Cargo.toml`, `fase_pixel_panel.rs`, os censos `every_panel_the_shell_drives…`, `paineis_armados.rs`, `quantas_entradas_tem_cada_painel.rs`, `architecture_every_editable_field_is_a_channel.rs`; `Cargo.lock`.
- `ph2d-i18n/src/pixel.rs` (~40 chaves `panel.pixel.timeline.*`).

## 5. Prova de fecho (por nome, tudo verde)

- `ph2d-pixel-core` inteira (motivo: formato 1.2): **79** verdes — `frames_oracle` 4 (Trial), `frames_parity` 4 (MVP), `frame_laws` 7.
- `ph2d-pixel-render` `view::tests` 2. `ph2d-app-pixel`: `quadros::` 4, `edit::` 17, `camadas::` 5, `paleta::` 19, `desenho::` 11, `painel_e_dialogo::` 6. `ph2d-panel-pixel-timeline` 8/8; shell 17/17 dos censos de painel.
- Binário `target/smoke/ph2d-host-desktop` refeito depois do último commit de código. ⚠️ Clippy NÃO corrido.

## 6. ABERTO

- ⛔ **Gates vermelhos a curar antes do fecho** (relatados pelo agente do painel; confirmar quais já vinham da P3): `pixel_panel.rs` 807 linhas (tecto 700 — mover o vocabulário da linha do tempo para ficheiro próprio); `ph2d-panel-pixel-tools/src/paint.rs` 695 (tecto 600); o painel PIXEL nos gates de cortes de rótulo, entradas, campo-canal, clamp, números mágicos, tipo de cor; `color_mode_ops` em `arch_mode_has_reconcile`; `hash3` em `pixel-core/brush.rs`; altura de abertura (inspector/painter layers) e isenção `widget_lab`. Correr pelo `verificador`, por NOME.
- O painel não emite `TagRange`, `ConstantFrameRate` nem `LayerContinuous` (a família trata-os): faltam os controlos (arrastar a ponta da tag, Constant Frame Rate, Layer Properties ▸ Continuous).
- Selecção de intervalos na linha do tempo (vários frames/cels — *Links Cels* de N, mover N) → P6/P12; Alt não chega à família (Alt+N/Alt+C do Aseprite).
- Papel vegetal com várias camadas medido só para «fundo opaco esconde»; *in front* (pos 1) sobre várias camadas e `loop_tag`/`current_layer` por medir. O z dentro de grupos assume a ordem achatada (não medido).
- P5: subtags, grupos de tags, *Play Subtags & Repetitions* como opção, Preview flutuante, velocidades, Rewind on Stop.
- **Mutação no fecho da linha** (somar às antigas): `check_cels` (cada lei), `rehome`, `tags_after_insert/delete`, a máquina do `Play` (passadas, saída `to+1`, ping-pong), `(índice+z, z)`, `mul(camada, cel)`, `ghosts` (clamp 0) e a tinta, a cache de frames (limpa por versão), `sane_frame`.
- Fecho da linha pendente: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`; a suíte completa é do integrador.

## 7. Próximo item

Primeiro curar os gates vermelhos da §6 (pelo `verificador`, por nome; cura por MOVER/cortar, nunca subir tecto). Depois a onda que o dono escolher: **P5 (resto)** ou **P6 — Seleção e transformação** (plano §5; P5 ∥ P6). Mesma disciplina: vetores do MVP em JSON primeiro, Trial observado, código, cena de smoke que MOSTRA cada coisa (a próxima é a **9**), fotos de fábrica e com `HOME=/home/enio` (copiar antes `~/.ph2d/layout.txt`).
