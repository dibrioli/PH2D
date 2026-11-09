# P13 — O que só a PH2D tem: o plano (09/11)

> Linha `line/PixelArt`, HEAD `cf50c1c999`, 104 commits à frente do `main` (P14 com smoke OK do dono, nada integrado).
> Plano do módulo: [02_plano_modulo.md §P13](02_plano_modulo.md). Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md).
> Molde: [P14_plano.md](P14_plano.md) §0–§10. Dono 09/11: sem licença do Aseprite; o oráculo que se CORRE é app gratuito triado (§3).
> **Estado: plano escrito; nada construído. Decisões do dono pedidas em §11.**

## 0. O que entra

Treze itens em **sete blocos** (CLAUDE §0.10: a mesma família resolve-se de uma vez).

| Bloco | Itens | O artista faz | Hoje |
|---|---|---|---|
| **R — Rampas** | rampas OKLCH · pincel de sombra na rampa · quantização que preserva rampas · color cycling · pincel de dither | gera uma rampa com o desvio de matiz (sombra fria, luz quente); pinta e o traço sobe ou desce na rampa; converte uma foto e as rampas dela continuam ordenadas; faz a água correr pela paleta; pinta meio-tom com padrão | paleta plana; o `Ink::Shading` anda na SELEÇÃO de índices; conversão por distância RGB; nada de ciclo; dither só no gradiente e na conversão |
| **L — Luz na rampa** | luz na rampa | o Pixel Sprite iluminado continua dentro da paleta: a luz troca o degrau da rampa em vez de multiplicar a cor | a luz da P14 multiplica (`luz_receber`/`_n`) |
| **G — Guias** | perspectiva 1/2/3 pontos · isometria guiada | põe pontos de fuga ou a grade iso 2:1; a Linha, o Retângulo e o lápis com Shift seguem a guia | só a grade quadrada e o snap de ângulo da Linha |
| **A — Análise** | linter | um painel lista os vícios de pixel art (órfãos, degraus irregulares, cantos duplos, banding, pillow shading, cores a mais); clicar leva ao pixel | nada |
| **V — Vista** | vista CRT | a prévia e a janela do jogo mostram o sprite como numa TV antiga, sem mexer no desenho | nada |
| **C — Camada** | auto-outline como efeito de camada | liga «Outline» numa camada; o contorno acompanha cada pincelada e cada quadro; «Apply» assa-o | o outline existe só como filtro destrutivo |
| **T — Tempo** | áudio na timeline · tween de cel · smudge | põe uma fala e vê a onda sob os quadros (lip-sync), ouve no Play; escolhe dois cels e gera os quadros do meio com easing; borra empurrando a cor | nada de áudio no Pixel; nada de tween; o «borrar» é só Blur/Jumble |

**Fora, de propósito:** a ponte Painter ↔ Pixel (quando o Painter fechar) · gerador de normais a partir da cor (recusa da P14).

## 1. Conferido no código (09/11, passo 0: cinco exploradores + leitura direta)

| Afirmação | Conferido |
|---|---|
| O «MVP» do módulo | app web antiga (`docs/Pixel Art/app/engine/*.js`); **não existe** no disco nem no git (procurado em `~` e com `git log --all`). Só sobrevivem os vetores copiados: `ph2d-pixel-core/tests/fixtures/mvp_color.json:1` (cita `ramp_quantize.js`, `cycling.js`, `PATTERNS`). ⇒ a lei de cada item define-se aqui, com oráculo corrido (§3) ou gabarito próprio |
| Paleta | `Vec<[u8;4]>` plano `ph2d-pixel-core/src/sprite.rs:300`; DB32 `palette.rs`; `find_duplicates` `palette.rs:103`; **nenhuma noção de rampa** |
| «Rampa» de hoje | uma seleção ordenada de índices: `ph2d-pixel-ponte/src/cor.rs:34` `rampa`, `gradient.rs:146` `palette_ramp(from,to)` |
| Shading | `Ink::Shading` `ink.rs:22-24`, `shade` `:67` (um degrau, para nas pontas); `stroke.rs:127` `with_shading`; ligado em `ph2d-app-pixel/src/input.rs:356-359` |
| OKLab/OKLCH | `ph2d-color` (`oklab.rs:50`, `oklch.rs`, `color_ramp.rs:32-35`, `ramp_lut.rs:50`), determinista CPU = WGSL (`ph2d-det-math`). Deps: `ph2d-det-math`, `ph2d-ease`, `bytemuck`, `serde` ⇒ folha leve. **`ph2d-pixel-core` hoje tem zero deps** |
| Conversão para Indexed | `convert.rs:173` `to_indexed`; `Mapper` `:275-309` = **cor mais próxima por euclides em RGB**; dither `Ordered`/`Old`/`FloydSteinberg` `:59-70`, `floyd` `:202`. Corte mediano só no GIF: `ph2d-pixel-io/src/gif_cores.rs:52` |
| Dither | Bayer 2/4/8 `dither.rs:9-14,59`; usado no gradiente (`stroke.rs:57,151`) e na conversão. `PatternBrush` `pattern.rs:30` carimba cores, não limiar |
| Color cycling | **nada** (o único ciclo é o de tags, `play.rs:14`) |
| Luz | `luz_receber` = `cor.rgb * luz_em(mundo)` `ph2d-render/src/shaders/receber.wgsl:49-54`; com normal `:70-76`; chamado em `sprite.wgsl:444,486`. O Pixel Sprite entra pela banda Vello + banda normal (P14, `fase_vector_bands_doc.rs`). Posterize só no Painter (`ph2d-painter-effects/.../basic.rs:63`), por canal |
| Grade e snap | `ph2d-pixel-render/src/prefs_view.rs:79,91,118` (só quadrada); `snap_pixel_angle` `raster.rs:357` (0/45/90, 2:1, 1:2) via `ponte/src/gesto.rs:208` com Shift, **só a Linha**; `pixel_perfect` `raster.rs:71` |
| Isometria reutilizável | `ph2d-grid/src/iso.rs:42` `dimetric_2_1`, `:53/:63` conversões, `snap.rs:85` `snap_world`; `ph2d-editor-core/src/grid_snap/state.rs:31` `GridKind` (Iso, Hex; **sem perspectiva**). O Pixel não os usa no `src` |
| Ponteiro e overlays | `ph2d-app-pixel/src/input.rs:47` `to_screen`, `:95` `cell`, `press :157` `moved :388` `release :529`; overlays em `draw.rs:197` (grade `:229`, slices `:274`, cursor `:297`, formigas `:328`) |
| Linter | nada; molde de lista de avisos: `ph2d-motion-diagnose/src/deficit.rs:13` `Diagnostic` (de grafo, não de píxeis) |
| CRT | nada. Prévia do Pixel = xadrez + composição em Vello (`ph2d-panel-pixel-preview/src/state.rs:63`); `ph2d-fx-cells` é filtro do Motion por forma |
| Camada | `Layer { name, visible, locked, opacity, blend, normal }` `sprite.rs:108-121`; `LayerKind` Image/Group/Tilemap `:95`; composição `compose.rs:45` `compose_window`. Outline destrutivo `filter/outline.rs:43` via `ph2d-app-pixel/src/filtros.rs:102` |
| Áudio | `ph2d-audio-decode/src/lib.rs:42` `decode`; `ph2d-audio-stream/src/lib.rs:148` `play_file(path, looping, region)` → `(StreamHandle, StreamPlayer)`, `stop :120`; o playhead `ph2d-audio/src/voice.rs:180` `cursor_frame` é **`pub(crate)`**; a onda desenha-se em `ph2d-panel-audio-editor/src/paint_*.rs`; a `ph2d-timeline` **não tem trilha de áudio** (só sinais, `signal.rs:14`) |
| Tocador do Pixel | `play.rs:245` `Play::advance(s, dt_ms)`, quartos de ms `:61,:249` |
| Cel | `Cel { bbox: Rect, opacity, z, link, … }` `sprite.rs:179`: posição sim, **rotação não**. RotSprite existe só no redimensionamento (`canvas.rs:24,211-227`); `Affine` `affine.rs:23`, `fast_rotation` `affine_fast.rs:34` |
| Easing | `ph2d-ease/src/lib.rs:397` `ease`, `:363` `bezier`, `:350` `spring`; `ph2d-tween/src/lib.rs:204`; `ph2d-curve/src/lib.rs:23` |
| Smudge | `filter/smudge.rs:13`: só **Blur** (média 3×3) e **Jumble** (vizinho sorteado), oráculo do Trial `oracle_trial_blur.json`; tecla R `input_teclas.rs:82` |
| Blob Pixel | `codec.rs:38` **`MINOR = 8`**; tag desconhecida dentro da MAJOR é saltada; o teste `ph2d-pixel-cena/tests/it/forma.rs:93` prende o MINOR |
| Tecto `ph2d-app-pixel` | **11 261 de 11 278** (`architecture_the_shell_only_shrinks.rs:222`): folga 17 ⇒ todo código novo vive em crates NOVAS ou nas de núcleo; a app só compõe |

## 2. Estado da arte — o que os outros fazem, e o que abandonaram

⚠️ Conhecimento geral, **não verificado nesta pesquisa** salvo onde diz «medido»; o que decide é o oráculo corrido (§3).

| Quem | O que faz | Onde para / abandonou | Lição |
|---|---|---|---|
| **Aseprite** | Shading ink sobre a seleção da paleta; color cycling **não**; outline só como comando; tween de cel **não** (pedido antigo dos utilizadores); sem áudio | a rampa é a seleção do momento, perde-se ao clicar outra cor | a rampa tem de ser **dado do documento**, não estado da seleção |
| **Pro Motion NG** | rampas («ranges»), color cycling, pincel de dither, guias de perspectiva | — | os ranges servem, ao mesmo tempo, o shading, o ciclo e o gradiente: **uma estrutura, três leitores** |
| **GrafX2 / Deluxe Paint** | color cycling por faixas de índices com velocidade; «shade mode» | só em modo indexado (o ciclo É a rotação dos índices) | o ciclo é do **modo Indexed** |
| **Pixelorama** (MIT) | camadas de áudio, efeitos de camada não destrutivos (outline, drop shadow, por shader), guias de perspectiva, grade iso | efeitos via shader do Godot | o efeito de camada é a mesma lei do filtro, aplicada na composição |
| **Lospec / «hue shifting»** | rampas com desvio de matiz (sombra fria, luz quente), croma máximo no meio | rampas só por luminosidade ficam lavadas | o gerador tem desvio de matiz e curva de croma, em espaço perceptual |
| **pngquant / libimagequant** | quantização perceptual com dither de difusão | não conhece rampas | a cor mais próxima em espaço perceptual é a base; preservar rampas é passo nosso |
| **Shaders CRT do RetroArch** (crt-royale, crt-geom) | scanlines, máscara de fósforo, curvatura, bloom | os pesados custam muito na GPU de entrada | uma lei barata com 3–4 controlos chega para uma prévia |
| **OpenToonz** (BSD-3) | trilha de som na X-sheet, lip-sync com fonemas (Papagayo) | — | a onda vive na linha do tempo da arte, com scrub |
| **Godot/Unity 2D** | luz que multiplica, sem paleta | o pixel iluminado sai da paleta (o ponto de dor do «luz na rampa») | ninguém faz luz dentro da paleta: é nosso |

## 3. Oráculos (CLAUDE §0.9; licença por artefacto — triagem 09/11)

| Oráculo | Instalado / licença do artefacto | Pergunta | Como |
|---|---|---|---|
| **ImageMagick 7.1.2-35** | sim; licença ImageMagick (`/usr/share/doc/ImageMagick-7/LICENSE`), permissiva | Bayer ordenado e Floyd–Steinberg; cor mais próxima (`-remap`) em RGB | `magick in.png -ordered-dither o8x8` / `-dither FloydSteinberg -remap pal.png` sobre entradas NOSSAS ⇒ fixture com cabeçalho; o gate compara o nosso `to_indexed` em modo RGB byte a byte |
| **Pillow 13.0.0** | sim; MIT-CMU | quantização simples + Floyd–Steinberg (segundo voto) | `python3 -I` script |
| **coloraide** (pip) | **não instalado**; MIT declarado | OKLab/OKLCH de referência (a matriz de Ottosson) para o gerador e para a cor mais próxima perceptual | instalar com aprovação do dono (§11); `pip show` confere a licença |
| **pngquant 3.0.3** (pacman) | **não instalado**; GPL-3.0+/ISC/BSD-2 | quantização perceptual de qualidade (o teto do que se espera) | só correr; métrica ΔE OKLab média, nunca bytes |
| **GIMP 3.2.6** | sim; GPL-3.0+ | conversão indexada com dither (terceiro voto) | `gimp -i -b`, só correr |
| **Pixelorama** (`~/.local/opt/pixelorama`, Godot 4.7.2) | sim; MIT no `.pck` | outline como efeito de camada, guias de perspectiva/iso, camadas de áudio, tween | **visual** (o CLI próprio não existe; `--headless` é do Godot): foto lado a lado; a lei do outline compara-se com o nosso filtro destrutivo, que é a verdade |
| **LibreSprite** | sim; GPL-2.0-only | smudge/borrar herdado (comportamento) | só correr; foto |
| **GrafX2 2.9** (pacman) | **não instalado**; GPL-2.0+ | color cycling (a velocidade e a direção) | só correr, visual |
| **RetroArch 1.22** (pacman) | **não instalado**; GPL-3.0 (shaders: licença por ficheiro) | aparência CRT | só visual; a nossa lei é própria |
| **OpenToonz 1.8.0** | sim; **BSD-3** (`/usr/share/licenses/opentoonz/LICENSE.txt`) | trilha de som, scrub, tween | não corre sem janela; **o fonte é permissivo** ⇒ pode ler-se (com atribuição se algo for portado) |
| **Aseprite Trial 1.3.18.6** | EULA | comparar na tela: Shading ink, outline | ⛔ não grava, sem script: só foto |
| Vetores do MVP | nosso | o que `mvp_color.json` traz | só o que já foi copiado |

**Itens sem oráculo que se corre:** luz na rampa, linter, guias de perspectiva ⇒ **gabarito próprio** (fixture que contém o fenómeno, lei escrita aqui, mutação no fecho).

## 4. O desenho — uma porta por pergunta

**Dependência nova:** `ph2d-pixel-core → ph2d-color` (folha leve, determinista CPU = WGSL). Uma verdade de OKLab no repo; recusa de copiar a matriz para o Pixel (§10). Confere-se `architecture_no_dependency_climbs_a_layer` no passo 0b.

**Crates novas** (drop-crate; a família compõe; a shell e a `ph2d-app-pixel` não crescem): `ph2d-pixel-rampa` (gerador, quantização, ciclo — se o núcleo passar do tecto próprio, senão módulos do núcleo), `ph2d-pixel-lint`, `ph2d-pixel-guia`, `ph2d-pixel-crt`, `ph2d-pixel-audio` (ponte para `ph2d-audio-*`; a família Pixel nunca vê `ph2d-app-audio`, regra `cbfdd16f28`).

| Pergunta | A porta ÚNICA | Quem lê |
|---|---|---|
| Que rampas tem esta paleta? | `Palette.ramps: Vec<Ramp { id, name, indices: Vec<u8> }>` no documento (MINOR 9); `rampa_de(pal, idx) -> Option<(RampId, pos)>` | Shading, gradiente, ciclo, quantização, luz na rampa, linter (cores a mais) |
| Gerar uma rampa | `gera_rampa(base: Oklch, passos, desvio_matiz_graus, curva_croma, faixa_l) -> Vec<[u8;4]>` (OKLCH → sRGB com corte de gama em croma, nunca em L) | botão «New Ramp» do painel de paleta |
| Para onde vai o traço de sombra? | `Ink::Shading` passa a andar na **rampa do documento** que contém a cor por baixo (a seleção vira só o atalho para criar a rampa); uma lei, `shade(pal, idx, ±1)` | pincel, linha, balde |
| Cor → índice | `Mapper` ganha `Espaço::{Rgb, Oklab}` (RGB fica para a paridade com o ImageMagick); `PreservaRampas`: cada cor de origem vai à rampa de matiz mais próxima e, dentro dela, ao degrau pela **ordem de L** — a ordem de claridade da origem nunca se inverte | Color Mode ▸ Indexed, Import, «Reduce Colors» |
| Que índice mostra este pixel agora? | `ciclo_em(pal, ciclos, t_ms) -> [u8;256]` (permutação); `Palette.cycles: Vec<Cycle { ramp: RampId, periodo_ms, direção }>`; **só Indexed** | composição do editor, prévia, Pixel Sprite na cena (o tempo vem do `quadro_mostrado`/relógio do Play), export (assa em quadros) |
| Dither entre duas cores | `PincelDither { nivel: 0..=64, matriz: Bayer2/4/8 }`: o texel `(x,y)` pinta primária se `bayer(x,y) < nivel`, senão secundária — **posição absoluta no canvas** (o padrão não «nada» com o traço) | lápis e balde com o ink Dither |
| Luz → degrau da rampa | §4.L | receptor do Pixel Sprite |
| Onde aponta esta guia? | `Guias { pontos_de_fuga: [Option<Vec2>;3], horizonte, iso: Option<Iso2x1> }` no documento; `direcao_guiada(guias, origem, ponteiro) -> Vec2` (a direção mais próxima: para um PF, ou um dos eixos iso) | Linha, Retângulo, lápis com Shift; overlay |
| Que vícios tem este cel? | `lint(sprite, frame, regras) -> Vec<Aviso { regra, células, severidade }>` puro | painel Lint, overlay de marcas |
| Contorno da camada | `Layer.efeitos: Vec<EfeitoDeCamada>`, `EfeitoDeCamada::Outline(OutlineParams)`; `compose_window` aplica, **por camada antes do blend**, a MESMA função de `filter/outline.rs` (uma lei, dois chamadores) | composição em todo leitor (editor, prévia, export, cena, `.ase` assa) |
| CRT | `crt(rgba, escala, CrtParams { scanline, mascara, curvatura, brilho }) -> rgba` (CPU, gabarito) e o mesmo em WGSL se o passo 0b o pedir | prévia, janela do jogo (§11) |
| Som do sprite | `Sprite.audio: Vec<FaixaDeAudio { asset, inicio_ms, ganho, mudo }>` (referência ao asset, **nunca as amostras**); `ph2d_pixel_audio::Relogio` (playhead do stream ⇒ `Play::advance` usa-o como `dt` quando há som) | timeline (onda por cache de picos), Play, scrub |
| Quadros do meio | `tween(sprite, camada, cel_a, cel_b, n, curva: ph2d_ease::Curve) -> Vec<Op>`: gera cels reais (posição inteira; rotação por **RotSprite** a partir do cel A); a curva entra por porta na composição, a `ph2d-app-pixel` não depende de `ph2d-ease` | gesto «Tween…» na timeline |
| Borrar empurrando | `Smudge::Push { força }` no mesmo `Smudge`: o carimbo arrasta a cor do texel anterior do traço para o atual; em Indexed o resultado volta à paleta pelo `Mapper` | lápis com a ferramenta Smudge (R alterna Blur/Jumble/Push) |

**L — a lei da luz na rampa.** O pixel iluminado tem de continuar na paleta (é a promessa). Dados que chegam ao receptor: a cor (banda Vello, como hoje) e uma **banda de rampa** (P14 fez o mesmo com a normal): `r = id da rampa`, `g = posição`, só para os texels cuja cor pertence a uma rampa; mais uma textura LUT `rampas[id][pos] → Oklab`. Lei: `L_alvo = L(cor) · lum(luz_em_n)` em OKLab; o degrau escolhido é o de L mais próximo **dentro da mesma rampa** (pontas param, como no Shading). Texel fora de rampa ⇒ a lei de hoje (multiplica). Propriedades com gate:
- luz neutra (`lum = 1`) ⇒ quadro **byte a byte** igual ao sem luz na rampa;
- monotonia: mais luz nunca escolhe um degrau mais escuro;
- a saída pertence sempre à paleta (censo de cores do quadro ⊆ paleta);
- gabarito CPU (`ph2d-light2d-ref` + a mesma função) × placa.
A banda e a LUT só existem com «Light: Palette» ligado num sprite à vista (o custo da P14 continua zero nas cenas sem ele; contado em `quadro_leituras.rs`).

**ADR novo** (número contado na integração): «rampas como dado do documento Pixel: uma estrutura, cinco leitores (shading, ciclo, quantização, luz, linter); a luz na rampa escolhe degrau em OKLab».

## 5. Formato e contratos (§0.11, §6)

- **§6:** zero contrato congelado tocado (grep da P14 repete-se no fecho sobre o diff).
- **Blob Pixel:** `Palette.ramps`, `Palette.cycles`, `Layer.efeitos`, `Sprite.guias`, `Sprite.audio` ⇒ **um** degrau `MINOR 8 → 9` para a onda (tags novas na tabela `codec.rs:1-30`; sem leitor de versão velha, §0.11). `forma.rs:93` acompanha.
- **`.ase`:** rampas e ciclos = user data da paleta (`ph2d.ramps`, `ph2d.cycles`); efeito de camada = user data da camada + o cel **assado** (Aseprite/LibreSprite vêem o contorno); guias = user data do sprite; áudio não vai ao `.ase` (Aseprite não tem), vai na folha JSON (`meta.audio`) e no projeto.
- **Mundo:** nada gravado novo (o Pixel Sprite continua a ler o documento); se a banda de rampa pedir componente, é derivado (`derivado_quando`, P14). `PROJECT_SCHEMA` só sobe se algo do mundo mudar — conta-se, nunca se escolhe.
- **Foundational append-only:** `ph2d-audio` expõe o playhead (`pub fn posicao_ms(&StreamHandle)`), `ph2d-render` ganha `luz_receber_rampa` em `receber.wgsl` ao lado das duas de hoje.

## 6. UI — as quatro condições por peça

| Peça | Existe | Pintada e registada | Clique chega | A sequência leva a algum lugar |
|---|---|---|---|---|
| Palette ▸ **New Ramp…** (diálogo: base, Steps, Hue shift, Chroma, Light range) | painel de paleta | botão + diálogo em cartões | op `AddRamp` com inverso | as cores entram na paleta; a rampa aparece como faixa com borda |
| Palette ▸ faixa de rampa (arrastar índices, **Cycle** com período e direção) | — | faixa sob a grelha de cores | `SetRamp`, `SetCycle` | Play ⇒ a faixa roda; Pixel Sprite na cena roda igual |
| Ink ▸ **Shading** (anda na rampa) · **Dither** (Level, Matrix) | barra de inks | já registado / item novo | `PanelEvent` existente | pintar sobe a rampa; o dither fica preso ao canvas |
| Color Mode ▸ Indexed ▸ **Keep ramps** · Color space RGB/Perceptual | diálogo de conversão | checkbox + lista | o diálogo existente | a foto convertida mantém a ordem de claridade |
| Cartão **Pixel Sprite** ▸ **Light: Smooth · Palette** | cartão da P14 | linha nova | op do cartão | a luz troca o degrau; os píxeis continuam na paleta |
| View ▸ **Guides** (Perspective 1/2/3 points, Isometric 2:1) | menu View | itens + alças arrastáveis no canvas | op `SetGuides` | Shift na Linha/Retângulo/lápis segue a guia |
| Window ▸ **Lint** | — | painel em cartões: lista de avisos por regra, cada regra liga/desliga | clique ⇒ o canvas centra e marca as células | corrigir o pixel ⇒ o aviso some |
| Preview ▸ **CRT** (Scanlines, Mask, Curvature) | painel Preview | botão + cartão | estado da vista, **não** do documento | a prévia parece TV; o ficheiro não muda |
| Layer Properties ▸ **Effects ▸ Outline** (cor/índice, Outside/Inside, 4/8 vizinhos) · **Apply** | diálogo de camada (P12) | secção nova | `SetLayerEffects` / `ApplyLayerEffects` | cada pincelada ganha contorno; Apply assa num passo de undo |
| Timeline ▸ **Add Audio…** (linha de onda sob os quadros, scrub) | timeline do Pixel | linha nova | op `AddAudioTrack` | Play ouve-se em sincronia; arrastar o cursor faz scrub |
| Timeline ▸ cels ▸ **Tween…** (frames, Ease, Rotate) | menu do cel | item + diálogo | gesto ⇒ `Vec<Op>` num passo | os quadros do meio nascem e retocam-se |
| Tool **Smudge** ▸ Blur · Jumble · **Push** | ferramenta Blur/Jumble | modo novo | tecla R já ligada | arrastar empurra a cor |

Strings em inglês por chave i18n; tamanhos por token; zero hex.

## 7. Gates (red-first: cada um visto VERMELHO antes da cura) e as fixtures

**Fixtures (contêm o fenómeno):** `paleta_rampas` (3 rampas: pele 5 degraus, água 4 com ciclo, folha 6; mais 3 cores soltas) · `foto_p13` (degradê céu + pele + folhagem 64×64, entrada nossa para ImageMagick/Pillow/pngquant) · `vicios_p13` (16×16 com 1 órfão, 1 canto duplo, 1 jaggy, 1 banding, 1 pillow — cada um numa célula conhecida) · `heroi_p14` reutilizado para luz na rampa e outline.

| Gate | O que prende |
|---|---|
| `a_rampa_gerada_e_a_do_coloraide` | `gera_rampa` × OKLCH do coloraide, ΔE < 1e-3 por degrau |
| `a_rampa_gerada_e_monotona_em_l` | L estritamente crescente; o corte de gama mexe só no croma |
| `o_shading_anda_na_rampa_do_documento` | o traço sobe/desce a rampa da cor por baixo, para nas pontas; cor fora de rampa não muda |
| `o_indexed_rgb_e_o_do_imagemagick` | `to_indexed` (RGB, sem dither / Bayer 8 / FS) × `magick -remap` byte a byte |
| `o_indexed_perceptual_nao_e_pior_que_o_pngquant` | ΔE OKLab médio ≤ o do pngquant na `foto_p13` (teto, não bytes) |
| `manter_rampas_nunca_inverte_a_claridade` | em cada rampa, a ordem de L da origem ⊆ a ordem dos índices escolhidos |
| `o_ciclo_e_uma_permutacao_que_volta` | `ciclo_em(t + período) = ciclo_em(t)`; Forward/Reverse/PingPong; fora da rampa fica |
| `o_ciclo_na_cena_e_o_do_editor` | Pixel Sprite em Play e prévia mostram o mesmo índice no mesmo `t` |
| `o_dither_fica_preso_ao_canvas` | dois traços sobrepostos com o mesmo nível ⇒ o mesmo padrão; nível 0 = secundária, 64 = primária |
| `a_luz_neutra_na_rampa_e_o_quadro_de_hoje` | `lum = 1` ⇒ byte a byte igual; G6 verde |
| `mais_luz_nunca_escurece_o_degrau` | varrer a luz 0..2 ⇒ posição na rampa monótona |
| `a_luz_na_rampa_fica_na_paleta` | censo de cores do quadro iluminado ⊆ paleta (só texels de rampa) |
| `a_placa_e_o_gabarito_concordam_na_rampa` | GPU × CPU, o mesmo degrau em todo texel |
| `a_linha_guiada_vai_ao_ponto_de_fuga` | Shift + Linha com 2 PF ⇒ a reta passa pelo PF (±0,5 px); iso ⇒ 2:1 exacto |
| `cada_vicio_e_achado_onde_esta` | `vicios_p13`: cada regra acha exactamente a sua célula; zero falsos na DB32 limpa |
| `o_efeito_de_contorno_e_o_filtro` | efeito Outline na composição = filtro destrutivo aplicado à camada, byte a byte |
| `aplicar_o_efeito_e_um_passo` | Apply ⇒ 1 passo; Ctrl+Z volta ao efeito vivo |
| `a_crt_desligada_e_a_previa_de_hoje` | CRT off ⇒ byte a byte; CRT on não muda o documento (hash) |
| `o_som_manda_no_relogio` | com áudio, o quadro em `t` segue o playhead (desvio ≤ 1 quadro a 60 Hz em 10 s) |
| `o_tween_chega_ao_cel_b` | o último quadro gerado é o cel B (posição); ease linear = interpolação exacta inteira |
| `push_arrasta_a_cor_e_fica_na_paleta` | Push em Indexed ⇒ toda cor ∈ paleta; a cor anda no sentido do traço |
| `o_blob_pixel_volta_igual` | rampas, ciclos, efeitos, guias, áudio: grava ⇒ lê ⇒ igual |

Mutação (fecho, agente `mutacao`): inverter o sinal do Shading, trocar L por croma na escolha de degrau, ler a seleção em vez da rampa, tirar o corte de gama só no croma, aplicar o efeito depois do blend, `dt` do host em vez do playhead.

## 8. Cenas de smoke (números MEDIDOS pela sonda headless antes da mensagem; próxima livre: **28**)

- **28 «Rampas»:** paleta com 3 rampas; um sprite pintado com Shading; a água com ciclo a correr.
- **29 «Foto em paleta»:** `foto_p13` convertida com e sem «Keep ramps», lado a lado.
- **30 «Dither»:** pincel de dither em 3 níveis, por cima de um gradiente.
- **31 «Luz na paleta»:** o herói da 26 com «Light: Palette», lâmpada a passar; mede-se que todo pixel aceso está na paleta.
- **32 «Guias»:** uma casa em 2 pontos e um bloco iso.
- **33 «Lint»:** `vicios_p13` com o painel aberto.
- **34 «CRT»:** a 26 na prévia com CRT.
- **35 «Contorno vivo»:** camada com Outline; pintar mostra o contorno a acompanhar.
- **36 «Fala e tween»:** um som curto sob os quadros e um cel tweenado com ease.
- O Push do Smoke entra na 30 (mesma ferramenta de pintura).

Cada cena fotografada (`fotografa_cena.sh`) e olhada antes de mandar.

## 9. Ordem (série só onde o resultado muda o seguinte)

1. **Passo 0b** (um agente): conferir `no_dependency_climbs_a_layer` para `ph2d-pixel-core → ph2d-color`; medir o custo da CRT em CPU a 8× sobre 256² (decide CPU × WGSL); medir onde a banda de rampa entra no `fase_vector_bands_doc.rs`; conferir o tecto do `ph2d-pixel-core`. Instalar os oráculos aprovados (§11).
2. **R1 — modelo de rampas + gerador** (MINOR 9 de uma vez com os campos de toda a onda). Tudo do bloco R depende disto.
3. Em paralelo, crates distintas:
   - **R2** Shading na rampa + pincel de dither + Push (pintura);
   - **R3** quantização perceptual + «Keep ramps» (conversão; oráculos ImageMagick/pngquant);
   - **R4** color cycling (paleta + composição + Pixel Sprite);
   - **G** guias (overlay + snap);
   - **A** linter (crate pura + painel);
   - **V** CRT (vista);
   - **C** efeito de camada Outline (composição).
4. **L** luz na rampa (depende de R1 e da banda da P14).
5. **T** áudio (foundational append-only no `ph2d-audio`) ∥ tween.
6. Sonda, fotos, cenas 28–36, smoke do dono; `/pd-onda-fechar`.

Cada agente: 45 min, escreve o relatório na §12 deste plano, testes por nome, nada de suíte.

## 10. ⛔ Recusas (desta onda)

| Recusado | Porquê |
|---|---|
| Copiar a matriz OKLab para o `ph2d-pixel-core` | duas verdades da mesma conversão; o `ph2d-color` já é determinista CPU = WGSL |
| A rampa como estado da seleção (Aseprite) | perde-se ao clicar outra cor e não serve aos outros quatro leitores |
| Color cycling em RGB | o ciclo é rotação de índices; em RGB seria reescrever píxeis por quadro (DPaint, GrafX2, Pro Motion: só indexado) |
| Luz na rampa por posterize por canal (Painter) | sai da paleta; a promessa é ficar na paleta |
| Luz na rampa com «altura» ou parâmetro novo na lâmpada | a mesma recusa da P14: a luz não muda, muda o receptor |
| Gravar as amostras de áudio no blob Pixel | o som é asset do projeto; o documento leva a referência |
| Tween «vivo» por cima da Timeline/Motion | o pixel artist retoca os quadros do meio; um tween vivo não se retoca (decisão do dono pedida, §11) |
| Efeito de contorno com lei própria | uma lei, dois chamadores: o efeito É o filtro destrutivo, aplicado na composição |
| Linter com IA ou heurística sem fixture | cada regra tem a sua célula conhecida e o zero-falsos na DB32 |
| CRT gravada no ficheiro | é vista, não documento |

## 11. Decisões do dono (09/11 — respondidas)

1. **Tween: quadros de verdade que se retocam** (não movimento vivo).
2. **Áudio: várias faixas de som por sprite**, como camadas ⇒ `Sprite.audio: Vec<FaixaDeAudio>` (já é o desenho da §4).
3. **CRT: na prévia E na janela do jogo** ⇒ a lei `crt` é uma só; o passe do jogo é um leitor a mais (a janela do jogo é da shell: composição, nunca a família a depender da shell).
4. **Oráculos: «todos que vc achar importante»** + pasta `~/Área de trabalho/Apps de Pixel` com os atalhos dos apps de pixel instalados (feita: Aseprite Trial, LibreSprite, Pixelorama, GIMP).
   - **coloraide 8.14.1** instalado num venv próprio `~/.local/opt/oraculos-py` (`pip show`: `License-Expression: MIT`). Correr: `~/.local/opt/oraculos-py/bin/python3 -I script.py`.
   - **pngquant, GrafX2, RetroArch** pedem `sudo` (senha do dono): `sudo pacman -S --needed pngquant grafx2 retroarch`. Depois: `pacman -Qi` em cada um e o atalho do GrafX2 entra na pasta.

Decididos tecnicamente (delegados): color cycling só no modo Indexed; «Light: Palette» desligado por omissão; Shading passa a andar na rampa do documento.

## 12. Estado (relatórios dos agentes entram aqui)

_vazio — nada construído._
