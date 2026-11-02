# P7 — Pincéis e dinâmica: o plano (02/11)

> Linha `line/PixelArt`, base `48106f005a`. Plano do módulo: [02_plano_modulo.md §P7](02_plano_modulo.md).
> Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> (autocontido, sem o Painter — memória `project_pixel_art_module_standalone_aseprite_spine_2026_10_25.md`).

## 1. Estado da arte

| App | O que faz | Porquê / o que foi abandonado |
|---|---|---|
| **Aseprite 1.3** (referência do dono) | **Custom brush** = Edit › New Brush (Ctrl+B) da selecção; pincel com a COR da imagem até se escolher uma cor (aí vira a forma inteira na cor de frente); 3 modos do padrão: *aligned to source* (o padrão fica onde foi copiado), *aligned to destination* (fica onde o traço começou), *Paint brush* (o padrão viaja com o pincel). **Lista de pincéis**: «Save Brush Here» guarda o pincel com os parâmetros MARCADOS (Type/Size/Angle · fg/bg só nos de forma · Image Color só nos de padrão · Ink e Opacity · Shade gradient · Pixel-Perfect), com cadeado para editar. **Dinâmica** (popup da barra de contexto): linhas Size · Angle · Gradient, colunas Pressure (só caneta) · Velocity (caneta ou rato), cada linha UMA entrada; limiar com duas pegas (à direita = menos sensível; juntas = mais sensível); Min/Max por linha (ex.: tamanho 1..16); **Stabilizer** = o traço anda a uma distância fixa do cursor (corda). Opção «Same in all Tools». | Fonte: aseprite.org/docs/brushes, /docs/dynamics. O gradiente com matriz de dither perdia a matriz em 1.3.0–1.3.1 (corrigido 1.3.2: *«Fixed dithering matrix selection in Gradient tool»*) — a matriz do Gradient da dinâmica e a da ferramenta Gradient são estados separados; o «Same in all Tools» partilhado era a causa. ⇒ uma porta por estado. |
| **Pixelorama 1.2.3** (MIT, instalado em `~/.local/opt/pixelorama/`) | Pincéis Pixel/Circle/Filled + **custom** (da selecção, do projecto, de ficheiros PNG numa pasta `Brushes`), *random brush* (pasta com variantes), flip X/Y e rotação do pincel, «colored brush» × máscara, pressão → alpha e tamanho. | Licença PERMISSIVA (§0.9: porta aberta). Corre por **extensão GDScript** (pasta `extensions/`), que pode injectar `InputEventMouseMotion` com `pressure` (Godot `Input.parse_input_event`) — é o único caminho de medir PRESSÃO sem caneta (§5, R-P). |
| Krita / Photoshop / Clip Studio | Estabilizador por janela ponderada (Krita), *Smoothing* + *Pulled String* (Photoshop CC 2018), «Stabilization» 0–100 (CSP). | O Photoshop passou anos sem estabilizador — o mercado usava o Lazy Nezumi; quando o fez, escolheu a CORDA (*pulled string*) como modo, que é a lei do Aseprite. A janela ponderada (Krita) atrasa no tempo, não no espaço, e em pixel art um atraso no tempo deixa o fim do traço por desenhar. ⇒ corda. |
| Painter da casa | `Pen { floor, ceiling, s_curve, smoothing, pulled_string_px }` e `pulled_string_step` em `ph2d-painter-brush/src/pen.rs`; velocidade L12. | O Pixel **não depende do Painter** (ordem do dono 25/10). A corda é UMA lei pura; ver §2.6. |

## 2. O desenho — a porta ÚNICA de cada pergunta

1. **«Que células e que cor carimba o pincel?»** → `ph2d_pixel_core::brush::Brush` continua a ser o dono único. Ganha `kind: BrushKind { Shape(BrushShape), Pattern(Arc<PatternImage>) }` (o `Brush` deixa de ser `Copy`; `Arc` mantém o clone barato). `Brush::stamp(cell, origin) -> impl Iterator<(Cell, Option<Rgba>)>` substitui `offsets()` em TODO o traço: `None` = a cor do traço (forma, ou padrão com cor escolhida), `Some` = a cor da imagem.
2. **«Onde está o padrão?»** → `PatternAlign { Source, Destination, Brush }` e UMA função `pattern_at(img, align, captured_at, stroke_start, stamp_anchor, cell)`. Nenhum outro sítio calcula o desfasamento.
3. **«O pincel virado/rodado»** → `PatternImage::transformed(Flip|Rot90)` reusa `ph2d_pixel_core::transform` (as funções que o Flip/Rotate do sprite já usam), nunca uma cópia.
4. **«Que tamanho/ângulo/cor tem ESTE carimbo?»** → `ph2d_pixel_core::dynamics::Dynamics { size, angle, gradient: Channel, stabilizer: Option<u32> }`, `Channel { sensor: Off|Pressure|Velocity, threshold: (lo, hi), min, max }`, e UMA função `Dynamics::at(&self, base: &Brush, s: Sample) -> StampParams`. A pressão e a velocidade passam pela MESMA rampa (`Channel::map`): o Trial mede a rampa pela velocidade (rato), e a pressão herda-a por construção.
5. **«Que velocidade tem a mão?»** → `Sample { cell_pos: [f64;2], pressure: f32, t_ms: u64 }` nasce num só sítio do app (`gesture.rs`, que hoje monta o `Paint`), lendo a pressão de `ph2d_pen::tinta()` como o Flip (`shells/desktop/src/flip/reshape_app.rs:36`); a velocidade é `velocity(prev, now)` em `dynamics.rs`, com a unidade MEDIDA no Trial (R-V).
6. **«A corda»** → a lei pura `pulled_string_step` SAI do `ph2d-painter-brush` para uma crate NOVA sem dependências, `ph2d-pen-law` (drop-crate, ADR-0075); o `ph2d-painter-brush::pen` re-exporta-a (`pub use`), os chamadores do Painter não mudam e os testes deles guardam o mesmo bit. O Pixel chama a mesma. ⛔ Não pôr no `ph2d-pen` (puxa `wayland-client`/`ph2d-host` para o motor de pintura).
7. **«O gradiente da dinâmica»** → fg→bg com a matriz de dither do Aseprite. Hoje NÃO há dither no `ph2d-pixel-core` ⇒ `dither.rs` NOVO com as matrizes Bayer 2/4/8 (fonte publicada: Bayer 1973, a matriz é aritmética), estado próprio da dinâmica (a lição do bug 1.3.0 do Aseprite: não partilhar com a futura ferramenta Gradient da P8 — a P8 chama a MESMA função com o estado DELA).
8. **«A lista de pincéis»** (DONO 02/11: no computador) → `~/.ph2d/pixel_brushes/` (como as paletas em `~/.ph2d/palettes.txt`, `shells/desktop/src/palette_persist.rs`, e como o Aseprite: valem em todo projecto): `brushes.txt` (parâmetros marcados, uma linha por pincel) + `<n>.png` para os padrões. Leitura/escrita em `ph2d-pixel-io::brush_file` (como `palette_file.rs`). **Não toca no `PROJECT_SCHEMA`** — ver §3 e a pergunta ao dono (§7).
9. **Atalho** Ctrl+B = New Brush quando o Pixel tem o teclado (`input_teclas.rs`; `B` sem modificador continua o lápis; o `KeyB` da shell só corre fora do Pixel).
10. **Desfazer**: criar/guardar um pincel NÃO é passo do desfazer do sprite (o Aseprite também não); o traço com pincel de padrão é um passo como qualquer traço.

## 3. Contratos congelados (§6) e schema — não encosta

- `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`, `NodeOp`, Vector: os gates `architecture_contract_surface`, `architecture_tool_contract_surface`, `architecture_vector_contract_surface` não nomeiam nenhuma crate do Pixel (`git grep -l "ph2d_pixel\|ph2d_app_pixel"` sobre os três = vazio, exit 1, 02/11). As ferramentas do Pixel são o enum `PixelTool` do `ph2d-app-pixel`, fora do contrato.
- `PROJECT_SCHEMA`: intocado se a lista de pincéis vive em `~/.ph2d/` (a opção recomendada). O blob `pixel_sprites` (`persist.rs`, `write_library`) só mudaria se o dono quiser os pincéis NO projecto — aí é formato gravado, degrau novo e recontagem (§3 do handoff de integração).
- `ph2d-painter-brush`: só ganha um `pub use` e uma dep (`ph2d-pen-law`); a superfície pública dele não muda.

## 4. A UI — as 4 condições, por peça

| Peça | Existe? | Pintada + registada | O clique chega ao barramento | A sequência leva a algum lugar |
|---|---|---|---|---|
| Menu «Brush type» (3 formas + pincéis guardados + «Save Brush Here» + cadeado) | NÃO — hoje `SHAPES` são 3 botões (`ph2d-panel-pixel-tools/src/ids.rs:56`) | popover novo no idioma dos menus da paleta (`paint_paleta_menu.rs`, P6l) | ids em `ids.rs`, arm em `seam.rs` | escolher → `PixelOptions.brush` → `gesture.rs` → `Brush::stamp` |
| Seletor do padrão (Source/Destination/Paint brush) | NÃO | `seletor` dropdown já usado no painel | `seam.rs` | `PatternAlign` → `pattern_at` |
| Botões Flip H/V e Rotate do pincel | NÃO | botões de ícone da casa | `seam.rs` | `PatternImage::transformed` |
| Popup **Dynamics** (grelha 3×2 de botões, limiar de DUAS pegas, Min/Max, Stabilizer) | grelha de botões sim; **limiar de duas pegas NÃO existe** em `ph2d-editor-core` (grep `range_slider/dual/two_handle` vazio) | widget NOVO `RangeSlider` em `ph2d-editor-core/src/widget/` **com o gate escrito junto** (DIRETIVA §2: widget de tipo sem gate leva o seu) | `seam.rs` | `Dynamics` → `Dynamics::at` por carimbo |
| Ctrl+B | atalho livre no Pixel | — | `input_teclas.rs` | selecção → `PatternImage::from_selection` |

Prova de cada linha: um teste `ph2d-ui-testkit` que DIRIGE o clique real e afirma o carimbo na tela (DoD §5 da DIRETIVA).

## 5. Medição — UMA rodada intercalada no Trial (§0.10), antes de escrever lei

Método das rondas anteriores (`docs/PixelArt/ferramentas/trial_*.py`: Xvfb próprio, `xdotool`, foto, `trial_le_tela.py` lê o centro de cada célula; fixtura com cabeçalho `source`/`method`/`command`). ⚠️ Os `trial_fit_*.py` importam de um scratchpad de outra sessão: a 1.ª tarefa é trazer essas ferramentas para `docs/PixelArt/ferramentas/` (senão a ronda não se reproduz).

Um só script, `trial_pinceis.py`, com sprites NOSSOS (padrão assimétrico 3×2 de 6 cores, para que flip e rotação se distingam):

- **R-B1** Ctrl+B sobre selecção; carimbo único em 3 posições × 3 modos de padrão → que cor cai em cada célula (o desfasamento de cada modo).
- **R-B2** escolher uma cor depois do Ctrl+B → o padrão vira a forma na cor de frente; e se volta à imagem (como?).
- **R-B3** Flip/Rotate com o pincel activo (existe no Trial? em que menu?) → o carimbo.
- **R-B4** Ctrl+B sem selecção; com selecção não rectangular (máscara) → o que entra no pincel; a âncora do pincel (centro? canto?).
- **R-B5** «Save Brush Here» com cada parâmetro marcado/desmarcado → o que volta ao escolhê-lo.
- **R-V** Velocidade: arrastos rectos do `xdotool` a 5 velocidades (passos e intervalos fixos) com Size Min 1/Max 16 e limiar em 3 posições → largura do traço ao longo do arrasto ⇒ a rampa e a UNIDADE da velocidade. O mesmo para Angle (pincel Line) e Gradient (cor de cada célula, a matriz).
- **R-S** Stabilizer ligado: o atraso em células entre o cursor e a ponta do traço, a 3 tamanhos de pincel.
- **R-P** (pressão) o Trial não a recebe sem caneta (`/dev/uinput` fora do grupo do utilizador, nenhum tablet virtual instalado — levantamento 02/11). A pressão herda a rampa medida em R-V (§2.4). A **forma** da resposta à pressão confere-se no Pixelorama por extensão que injecta pressão (sonda de viabilidade primeiro; se não injectar em 1 tentativa, fica registado como não medido e a prova é a mão do dono no smoke com a Wacom).

Kill-criterion: se o jitter do `xdotool` no Xvfb impedir R-V de dar a mesma largura em 3 corridas iguais depois de 2 tentativas de método, a velocidade fica pela rampa documentada (linear entre limiares) com a recusa MEDIDA escrita, e o smoke do dono decide.

## 6. Gates, red-first, e as fixturas

- `ph2d-pixel-core/tests/it/brush_pattern.rs` contra `oracle_trial_brush_pattern.json` (R-B1..B4): vermelho antes de `pattern_at` existir.
- `ph2d-pixel-core/tests/it/dynamics.rs` contra `oracle_trial_dynamics_velocity.json` (R-V, R-S): rampa, limiar, Min/Max, matriz do gradiente, a corda.
- `dither.rs`: Bayer 2/4/8 contra a matriz aritmética (lei publicada) e contra as células de R-V Gradient.
- `ph2d-pen-law`: paridade — o Painter dá o MESMO bit antes e depois do movimento (o teste da corda que já existe no `ph2d-painter-brush` corre por nome dos dois lados).
- `ph2d-pixel-io`: `brush_file` ida-e-volta byte a byte + ficheiro corrompido recusado sem tocar na lista.
- Seam (`ph2d-ui-testkit`, `shells/pixel-scenes`): Ctrl+B → carimbo; menu Brush type → escolhe/guarda; Dynamics → limiar arrastado muda a largura; `RangeSlider` com o gate de widget próprio.
- Mutações no fecho: `pattern_at` (troca de modo), `Channel::map` (limiar invertido), `Bayer` (índice), a corda (comprimento 0), o `None/Some` da cor do carimbo.

## 7. Smoke — cena 18 (`PH2D_PIXEL_SMOKE=18`; `max_level` 17 → 18 em `crates/ph2d-app-pixel/src/smoke.rs:31`)

Sprite com um desenho pequeno de 6 cores já seleccionado; o painel com o popup Dynamics aberto em Size ← Velocity (1..16). Passos: Ctrl+B · carimbar nos 3 modos · Flip · traço lento vs rápido com o rato · Stabilizer · a Wacom (pressão). Os números da mensagem ao dono (larguras, desfasamentos) saem da sonda headless da cena 18 corrida ANTES de escrever a mensagem, e a cena é fotografada (fábrica + layout do dono, `layout.txt` com sha256 antes/depois).

## 8. Ordem de trabalho (um bloco)

1. Ferramentas do Trial para o repo + `trial_pinceis.py` + a rodada R-B/R-V/R-S (e a sonda R-P no Pixelorama) — UMA corrida, fixturas gravadas.
2. Núcleo: `Brush::stamp` + `pattern_at` + `dither.rs` + `dynamics.rs` + `ph2d-pen-law` (gates vermelhos → verdes).
3. App: `Sample` com pressão e tempo, Ctrl+B, corda; `ph2d-pixel-io::brush_file` + `~/.ph2d/pixel_brushes/`.
4. UI: `RangeSlider` (+ gate), popup Dynamics, menu Brush type, seletor do padrão, Flip/Rotate; seam tests.
5. Cena 18, sonda, fotos, mutações, handoff.
