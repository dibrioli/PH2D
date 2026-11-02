# HANDOFF DE CONTINUAÇÃO — Pixel Art P7b (pincéis e dinâmica: app, mão, lista) — 02/11

> Linha `line/PixelArt`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt` (`cd` + `pwd` +
> `git branch --show-current` antes de ler). Plano: [P7_plano.md](../P7_plano.md). Anterior:
> [P7a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_P7a.md) (leis medidas do Trial, §3–§4, e as decisões do dono, §6).
> **Feitos os itens 1–3 do plano; ABERTOS o 4 (UI) e o 5 (cena 18, mutações, fecho).**

## 1. Commits desta jornada (mais antigo primeiro)

Base = main `3df5894489` (rebase no-op ao assumir).
`f65967862b` Ctrl+B no app · `fa489f1631` virar/girar (`PatternBrush::permuted`) · `c03c134d59` núcleo da dinâmica ·
`6ce1febd36` a mão no traço livre · `4c0d91515a` lista de pincéis no computador.

## 2. O que JÁ existe (não reconstruir)

**Ctrl+B.** `PixelEditAction::NewBrush` (editor-core `pixel_panel/selection.rs`); tecla em
`ph2d-app-pixel/src/selecao.rs::key`; lógica em `ph2d-app-pixel/src/pincel.rs` (`novo_pincel`, `pega_pincel`,
`escolhe_fg`, `recorta_pincel`, `press`).
- Com selecção: o pincel vem do cel activo e a selecção cai (UM passo do desfazer: o deselect; não medido se o Trial o
  conta como passo).
- Sem selecção: `app.recorte` → o próximo arrasto (Laço com `pincel: Some(corta)`) define o pincel; botão direito também
  corta (UM passo); Esc larga. `B` sozinho continua o lápis.
- Decisão do dono 6.1 (P7a): `PixelApp::escolhe_fg` (paleta, painel Fg, conta-gotas, troca X/Swap) põe
  `pincel_imagem=false` ⇒ o padrão pinta INTEIRO na cor de frente. O Trial muda até com a MESMA cor
  (`rb2b_frente_paint_cores`), por isso é um setter, não uma comparação. Recuperar o espaço devolve as cores da imagem.
  `gesture::paint` recebe `Option<&(Arc<PatternBrush>, bool)>`.
- Decisão 6.2 no núcleo: `PatternBrush::permuted(f)` com as permutações de `transform.rs` (flip_h/flip_v/rot90cw/
  rot90ccw/rot180); valores e máscara juntos; o ponto da âncora no sprite (origin+âncora) não se mexe. **FALTA o botão.**
- `PatternBrush::to_rgba/from_rgba` (Indexed = cor mais próxima).

**Núcleo da dinâmica** `ph2d-pixel-core/src/dynamics.rs`: `Sensor`, `Channel{sensor,lo,hi,min,max}` (UMA rampa `ramp/map`
para pressão e velocidade), `Dynamics{size,angle,gradient,dither}`, `Dab{size,angle,gradient}` (`floor` com folga 1e-9),
`Velocity` (filtro no VECTOR, `v += (1−e^(−dt/τ))(passo−v)`, τ=50 ms; o 1.º evento só marca a partida; parado não corre).
Escala da barra: pega f = `66·f − 1` px/evento (`VELOCITY_FULL_PX`, `VELOCITY_ZERO_PX`). `dither.rs` ganhou `lit(t)`.
`Stroke`: `set_dab`, `with_dab`, `with_gradient(bg, Dither)` (fundo→frente, «BG > FG»: devagar = fundo), `replacing(from)`,
`start` público.

**Gates novos.** `ph2d-pixel-core` `dynamics_oracle` (refaz os 80 arrastos `round2_size_velocity_rows` pelo NOSSO traço;
erro médio 0,146 em 41 776 colunas, pior 0,670; tectos 0,16 / 0,75; o Trial nunca pinta o ÚLTIMO trecho — a comparação
pára um raio antes; o r5 a 200 % lê da célula 10; constantes antigas — 66·f sem −1, passo desde o premir — davam 0,31) e
`dynamics_laws` (vector, pausa, 1.º evento, rampa da pressão, carimbo por ponto, gradiente Bayer). `brush_pattern` ganhou
rb6 (âncora 6×4 com buraco), a cor escolhida inteira e virar/girar.

**App.** `PixelOptions.dynamics: PixelDynamics` (editor-core `pixel_panel/dynamics.rs`: `PixelSensor`,
`PixelChannel{sensor,min}`, `PixelThreshold{lo,hi}` em ‰ (omissão 100/890), `PixelGradientDither`, `stabilizer` (0 = off,
`PIXEL_STABILIZER_ON`=16), `active()`); Max do Size e do Angle = o da barra (foto 55). `ph2d-app-pixel/src/mao.rs`:
`Mao` (filtro + corda `ph2d_pen_law::pulled_string_step` em px de ecrã, `STRING_PX_PER_UNIT`=8), `tinta()` lê
`ph2d_pen::porta::{tinta, evento_ns}` (a shell já chama `poe_tinta` em TODO evento,
`shells/desktop/src/input_dispatch.rs:154,402`), `move_hand`. Só Pencil/Eraser (Kind::Free). `app.mao`. Deps novas do
app-pixel: `ph2d-pen`, `ph2d-pen-law`. Testes `ph2d-app-pixel` `dinamica::` (pressão, velocidade, corda) e `pincel::` (6).

**Lista de pincéis.** `ph2d-pixel-io::brush_file` (`brushes.txt` canónico `ph2d-pixel-brushes 1`, secções `brush … end`,
padrão em RGBA hex, `.`=fora, `-`=transparente; `BrushParams::NEW_BRUSH` = type+image_color; `SAVE_HERE` =
type/size/angle/image_color/ink; recusa com a linha), teste `ph2d-pixel-io/tests/brush_file.rs`. App
`ph2d-app-pixel/src/pinceis.rs`: pasta = a das paletas (`PH2D_PIXEL_BRUSHES_DIR` manda; senão
`$XDG_DATA_HOME/ph2d/pixel_brushes`, omissão `~/.local/share/ph2d/pixel_brushes`; a memória
`project_pixel_brushes_live_on_the_computer_2026_11_02.md` foi corrigida, dizia `~/.ph2d/`); `espacos()`,
`espaco_do_ctrl_b` (cada Ctrl+B um espaço novo, destrancado), `guarda_aqui` (trancado; SEM entrada de UI),
`recupera` (Alt+1..9 em `input_teclas.rs`, só os parâmetros guardados), `apaga_espaco` e `tranca_espaco` (SEM UI).
`PixelApp.pinceis` (None nos testes/palcos); `with_armed_smoke` liga a pasta.

## 3. Mapa da UI para o item 4 (tudo em `crates/ph2d-panel-pixel-tools`)

- Ids: `src/ids.rs` (`NUMBERS [_;7]` :92 com faixas, `CHECKS [_;10]` :110, `SEGMENTS` :124, `INKS` :148,
  `ACTIONS [(NodeId,PixelRequest,&str);14]` :194); `ids_selecao.rs` (`EDIT_ACTIONS [;17]` :136 — falta «New Brush»).
- Pintura `src/paint.rs` (`opcoes` :233; helpers `campo` :344, `escolha` :364, `caixas` :381, `botoes` :451).
- Clique `src/seam.rs` (`pedido` :340, `com_segmento` :271, `numero` :209/`com_numero` :222, `caixa` :253; eventos
  ValueChanged/Toggled → `PixelRequest::Options`).
- Dropdown genérico `ph2d-editor-core/src/widget/dropdown/mod.rs:159` + `popover.rs`. **NÃO há slider de duas pegas**:
  copiar `widget/slider.rs` (testes `widget/slider/tests.rs`); tecto de widget 500 LOC (`architecture_widget_loc_cap`).
- Seam tests `tests/it/seam_pixel*.rs` (helper `so()` :89); i18n `every_word_this_panel_shows_comes_from_the_string_table.rs`;
  HR-15 (`// LITERAL-PX-OK:`).
- ⚠️ **Tectos:** `seam.rs` 584/600 (`architecture_panel_loc_cap`) — CORTAR por responsabilidade ANTES de crescer;
  `paint.rs` 505/600; `pixel_panel/mod.rs` 533/700.
- Consumo de `PixelRequest`: `ph2d-app-pixel/src/panel.rs:40` + listas `edicao.rs:118` (`assenta_antes`), `paleta.rs:159`.
  As acções de pincel (guardar aqui, apagar, trancar, virar/girar, escolher espaço) precisam de `PixelRequest` novos
  (ex.: `PixelRequest::Brush(PixelBrushAction)`), com o app em `pinceis.rs`/`pincel.rs`.

## 4. Próximo, na ordem

4. **UI:** (a) cortar `seam.rs`; (b) menu do tipo de pincel (dropdown: formas + espaços + «Save Brush Here» + cadeado/
   apagar por espaço + virar H/V e girar 90° quando há padrão); (c) seletor do `pattern_align` (Source/Destination/Paint)
   — hoje ninguém escreve `PixelOptions.pattern_align` (não é órfão: é este item); (d) popup Dynamics (Stabilizer caixa+
   slider, grelha 3×2 uma coluna por linha, Min, dropdown do Gradient, «BG > FG», barra de DUAS pegas por sensor activo
   = widget NOVO com o gate dele); (e) botão «New Brush» no menu Edit; seam tests de cada um; strings em inglês simples
   (o app é SÓ em inglês).
5. **Fecho:** cena de smoke 18 (`crates/ph2d-app-pixel/src/smoke.rs:31`, `max_level` 17→18), sonda headless, fotos,
   provas de mutação, fmt/clippy das crates tocadas 1× (`PH2D_SUITE=fecho`; o app-pixel mudou deps ⇒ crate inteira com
   `motivo: deps`), handoff de integração. Mutações a ver vermelhas: âncora ⌊w/2⌋; `escolhe_fg` sem
   `pincel_imagem=false`; o −1 da barra; o 1.º evento a contar; filtro escalar em vez de vector; 8 px/unidade da corda;
   parâmetro desligado que mantém o actual; o corte do botão direito.

## 5. Decisões do dono

As do §6 do P7a (ambas implementadas no núcleo; a 6.2 falta o botão). Nenhuma pergunta nova pendente.

## 6. Não medido / aberto

- Angle ← Velocity (Max = o ângulo da barra por analogia com o Size, foto 55); Gradient até saturar.
- Se o deselect do Ctrl+B é passo do desfazer no Trial.
- Pressão real só com a Wacom do dono (Pixelorama: rampa linear).
- O atraso do último trecho do Trial NÃO se copia (pintamos até ao cursor).
