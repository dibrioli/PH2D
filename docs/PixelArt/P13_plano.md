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

**Dependências (revisto pelo passo 0b, §12):** o núcleo **continua sem dependências** (`the_pixel_core_declares_no_dependency`, ADR-0196). As rampas como DADO (`Palette.ramps`, `rampa_de`, `shade` na rampa, `ciclo_em`) não fazem conta de cor e ficam no núcleo, em ficheiros novos (`ramp.rs`, `cycle.rs`; `sprite.rs` tem folga 10 do tecto de 700, `op.rs` 16 ⇒ Ops novas num ficheiro irmão). Toda a conta OKLab mora na crate nova **`ph2d-pixel-rampa`** (`ph2d-pixel-core` + `ph2d-color`): `gera_rampa`, a métrica OKLab do `Mapper` (o núcleo recebe a métrica por porta), `PreservaRampas`, a LUT da luz. Uma verdade de OKLab, sem ADR de dependência.

**Crates novas** (drop-crate; a família compõe; a shell e a `ph2d-app-pixel` não crescem): `ph2d-pixel-rampa`, `ph2d-pixel-lint`, `ph2d-pixel-guia`, `ph2d-pixel-crt` (lei em WGSL na prévia e no jogo; `crt()` CPU = gabarito e export — medido: 1 fio 25,7 ms em 2048², não cabe no quadro), `ph2d-pixel-audio` (ponte para `ph2d-audio-*`, que ganha `StreamPlayhead` append-only; a família Pixel nunca vê `ph2d-app-audio`, regra `cbfdd16f28`).

**Luz na rampa — onde liga (passo 0b):** banda irmã `banda_rampa` em `ph2d-pixel-luz` ao lado de `banda_normal`; `faixa_rampa` em `fase_vector_bands_doc.rs:45-49`; `band_blit` grupo 0 binding 4 (banda) e 5 (LUT), neutras 1×1 como a `normal_neutra`, `Flags::com_rampa`. Só se lê o id com `a == 1` (o antialias da borda mistura o id).

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
| `ph2d-pixel-core → ph2d-color` | o núcleo é sem dependências por lei (ADR-0196, `pure_core.rs:44`); a conta vai para `ph2d-pixel-rampa` (passo 0b) |
| CRT na CPU por quadro | 25,7 ms a 1 fio em 2048² (passo 0b); com rayon cabe mas rouba 4–8 fios ao jogo ⇒ WGSL |
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

### Passo 0b (09/11, agente)

Só medições; nada de produto editado. Os oráculos (§11) não foram instalados aqui: o pngquant, o GrafX2 e o RetroArch pedem `sudo`.

**1. `ph2d-pixel-core → ph2d-color`: a camada deixa, mas outra regra BARRA.**
| Regra | Veredito |
|---|---|
| `architecture_no_dependency_climbs_a_layer` (`ph2d-editor-core/tests/it/…:59-98`) | as duas crates são **folha** (`especie`, :59-86); `permitido` (:89-97) só restringe quem depende de família, painel ou ferramenta ⇒ folha → folha **pode**. `ph2d-color` só depende de `ph2d-det-math` (`libm`), `ph2d-ease` (zero deps), `bytemuck` e `serde`, e nenhuma delas chega ao Pixel ⇒ sem ciclo |
| `the_pixel_core_declares_no_dependency` (`ph2d-pixel-core/tests/it/pure_core.rs:44-61`) + ADR-0196 §1 (`0196-…md:23`: «`ph2d-pixel-core` … **sem dependências**») | **REPROVA** qualquer linha em `[dependencies]`. Mudar isto pede ADR |
⇒ **Proposta:** as rampas como DADO (`Palette.ramps`, `Ramp{id,name,indices}`, `rampa_de`, `shade` na rampa, `ciclo_em`) não precisam de cor e ficam no núcleo, sem deps. Tudo o que faz conta em OKLab (`gera_rampa`, `Mapper` Oklab/`PreservaRampas`, a LUT da luz) vai para a crate nova **`ph2d-pixel-rampa`** (deps: `ph2d-pixel-core` + `ph2d-color`). O `Mapper` do núcleo recebe a métrica por porta (`fn(&[u8;4], &[u8;4]) -> u32` ou um trait), e a OKLab entra pela `rampa`. Assim fica uma só verdade de OKLab, sem ADR e sem tocar o gate.

**2. Tectos de LOC.** Nenhum gate tem tecto por crate para `ph2d-pixel-*`: o `the_shell_only_shrinks` só lista `ph2d-app-pixel` (`:222`, 11 278). O que limita o núcleo é o **tecto de 700 linhas por ficheiro** (`architecture_workspace_file_loc_cap.rs:39`, nenhum ficheiro do Pixel na lista de exceções).
| Ficheiro do núcleo | LOC | Folga até 700 |
|---|---|---|
| `sprite.rs` (onde mora a `Palette`, :300) | 690 | **10** |
| `op.rs` | 684 | 16 |
| `affine.rs` / `tiles.rs` | 669 | 31 |
| `codec.rs` (MINOR 9) | 653 | 47 |
| `palette.rs` · `convert.rs` · `compose.rs` · `ink.rs` | 324 · 381 · 503 · 83 | ≥ 197 |
Núcleo hoje: 13 134 LOC (io 5 400, text 2 552, keys 2 162, tiles 1 737, render 1 571, ponte 1 509). ⇒ `Ramp`/`Cycle` entram em ficheiros NOVOS (`ramp.rs`, `cycle.rs`); a `Palette` só ganha os dois campos (≈ 4 linhas, cabe nos 10). As Ops novas vão para um ficheiro irmão, porque o `op.rs` não aguenta um bloco. A matemática OKLab vai para a `ph2d-pixel-rampa` (item 1).

**3. CRT na CPU** (bench descartável: 256² RGBA → 2048², scanline na linha ímpar do sub-pixel, máscara RGB por coluna, barril com amostra mais próxima, brilho; `--release`; mínimo de 20 rodadas intercaladas; load ≈ 5,2 durante a corrida, por isso o número é limite superior).
| Variante | Mínimo (ms) | Cabe em 16,6 ms? |
|---|---|---|
| 1 fio | **25,7** | **não** (1,55×) |
| rayon 4 fios | 8,6 | sim |
| rayon 8 fios | 4,3 | sim |
| rayon 16 fios | 2,65 | sim |
⇒ Com 1 fio não cabe. Com rayon cabe, mas leva 4 a 8 fios a cada quadro (só a saída tem 16 MB) e disputa a CPU com o jogo. **Decisão: WGSL na janela do jogo e na prévia**, com o `crt()` da CPU como gabarito e para o export (lei, não tempo real). O gate CPU × placa sai daqui.

**4. Onde a banda de rampa se liga** (o caminho da P14, verificado):
| Ponto | arquivo:linha |
|---|---|
| a composição pede a banda por faixa Pixel | `shells/desktop/src/render_loop/fase_vector_bands_doc.rs:45-49` (`faixa_normal` → `normais.cenas.push`); as faixas que não são Pixel recebem `None` em `:51`. **A terceira banda entra aqui**: `ph2d_app_pixel::luz::faixa_rampa(…)` → `rampas.cenas.push` |
| o produtor | `ph2d-app-pixel/src/luz.rs:44-55` → `ph2d-pixel-luz/src/lib.rs:126` `banda_normal` (Vello, `ImageQuality::Low` = vizinho mais próximo) ⇒ o irmão `banda_rampa` (r = id, g = posição, a = 255 só para texel em rampa) |
| a textura da banda | `ph2d-app-light2d/src/normais.rs:11-14` `NormaisDasFaixas` guarda **uma** textura; `colar` (:36-83) faz Vello → intermediário → `copy_texture_to_texture` (:55). A rampa precisa de uma **segunda textura** (cópia desse tipo ou um campo irmão) e de mais um render do Vello por faixa, só com «Light: Palette» ligado (G6, `bytes()` :26) |
| a colagem | `ph2d-render/src/band_blit.rs:391` `blit_com_normal` → `colar` :404; bind group :419-437 |
| o shader | `ph2d-render/src/shaders/band_blit.wgsl:83-103` `fs_main`; a lei entra em `acender` (:71): com o texel em rampa, `L_alvo = L·lum(luz_em_n)` e o degrau vem da LUT, e não do `rgb * luz` |
| **ranhuras livres** | o layout é `[Some(bgl), None, None, Some(luz_bgl)]` (`band_blit.rs:247`) ⇒ **os grupos 1 e 2 estão VAZIOS**. No grupo 0 ocupam-se os bindings 0-3 (`:180-215`); o 4 em diante está livre. Sugestão: grupo 0 binding 4 = banda de rampa, 5 = LUT `rampas[id][pos]` (2D, `Rgba32Float`/OKLab), com 1×1 neutras como a `normal_neutra` (:168, :268) e um flag `com_rampa` no `Flags` (:20-29). `band_blit.rs` tem 596 LOC (folga de 104) |
| o sprite direto (não a banda) | `sprite.wgsl:466-486` `fs_main_n` → `receber.wgsl:70` `luz_receber_n`. O Pixel Sprite passa pela banda Vello, não por aqui; só é preciso mexer se o Sprite do motor também ganhar luz na rampa |
⚠️ Na borda do quadrado do sprite, o Vello pode misturar o id com o transparente por antialias. Por isso a banda só se lê com `a == 1`; o resto cai na lei de hoje.

**5. Posição do stream em ms.** O voice de stream já tem o cursor em quadros da fonte e enrola no loop (`ph2d-audio/src/voice.rs:375-449`, `render_streamed`; `cursor_frame` :180 é `pub(crate)` e só o preview o publica, `engine.rs:560` → `meter.store_preview`). O `StreamHandle` (`stream.rs:128`) **é movido** para a thread de áudio por `Engine::play_stream` (`engine.rs:213`) depois de sair de `ph2d_audio_stream::play_file` (`ph2d-audio-stream/src/lib.rs:148`). Mudança mínima, só acrescentando:
1. `stream.rs:110` `Shared` ganha `played: AtomicU64`;
2. `StreamHandle` ganha `pub(crate) fn publicar(&self, f: u64)` (store `Relaxed`) e `pub fn playhead(&self) -> StreamPlayhead`, que guarda um `Arc<Shared>` e o `rate`;
3. o novo `pub struct StreamPlayhead { shared, rate }` (Clone) tem `frame()` e `ms() = frame·1000/rate`;
4. `voice.rs:447`: antes de `self.source = Some(Source::Streamed(h))`, `h.publicar(self.cursor.max(0.0) as u64)`, uma vez por bloco, sem alocar;
5. em `ph2d-audio-stream`, nada muda (o chamador faz `handle.playhead()` antes de `play_stream`). Precisão: um bloco do mixer, sem a latência de saída.

### R1 (09/11, agente)

Commit `c4188778e9` (código + fixture + script do oráculo). Sem UI (R2+). `ph2d-app-pixel`: zero linhas.

**Núcleo (`ph2d-pixel-core`, continua sem dependências — `pure_core` verde).** Ficheiros novos: `ramp.rs` (`RampId`, `Ramp {id,name,indices}`, `PaletteRamps {ramps, cycles}`, `rampa_de(&PaletteRamps, idx) -> Option<(RampId, pos)>`, `check_ramps`, `remap`, getters `Sprite::ramps/guias/audio`), `cycle.rs` (`Cycle {ramp, periodo_ms, direcao}`, `CycleDirection::{Forward,Reverse,PingPong}`, `ciclo_em(&PaletteRamps, t_ms) -> [u8;256]`; `periodo_ms` = a volta INTEIRA), `guia.rs` (`Guias {pontos_de_fuga:[Option<[f32;2]>;3], horizonte, iso: Option<Iso2x1{origem}>}`), `audio.rs` (`FaixaDeAudio {asset, inicio_ms, ganho, mudo}`), `efeito.rs` (`EfeitoDeCamada::Outline(OutlineParams)`, `OutlineParams = filter::outline::Outline`). Os `f32` comparam por bits (o `Sprite` é `Eq`). A `Palette` é um `Vec` plano, por isso os campos entraram no `Sprite` (`ramps`, `guias`, `audio`) e na `Layer` (`efeitos`). Ops em `op/doc.rs`: `Op::Doc(Box<DocOp>)` com `AddRamp` (o id sai do `next_id`) · `SetRamp` · `RemoveRamp` (leva o ciclo) · `SetCycle` · `SetRamps` (o inverso das quatro) · `SetGuias` · `SetAudio` · `SetLayerEffects`; `OpError::BadRamps`. Leis: índice dentro da paleta e em no máximo UMA rampa, rampa não vazia, um ciclo por rampa, período > 0. `Op::Palette`/`Whole` recusam paleta que parte uma rampa; `Op::Remap` leva as rampas (inverso exacto). Rótulo do histórico `pixel.history.palette_data` («Ramps, Guides and Sound»).
LOC: `sprite.rs` 698 · `op.rs` 697 · `codec.rs` 667 (tecto 700) ⇒ **o próximo campo no `Sprite` ou variante no `Op` não cabe**: vai para ficheiros irmãos.

**Codec MINOR 8 → 9** (`codec/doc.rs`): Sprite 17 Ramp* · 18 Cycle* · 19 Guias · 20 Audio* · Layer 9 Efeito*; o `validate` corre `check_ramps`. Sem leitor velho (§0.11). `forma.rs:93` lê a constante `MINOR` ⇒ não precisou de edição (verde).

**`ph2d-pixel-rampa`** (crate nova, `ph2d-pixel-core` + `ph2d-color`): `planeada` → `na_gama_pelo_croma` (bissecção só no croma, resolução 1e-6, L e h intactos) → `gera_rampa_oklch` → `gera_rampa([u8;4], ConfigRampa{passos, desvio_matiz_graus, curva_croma, faixa_l}) -> Vec<[u8;4]>`; `op_rampa_nova(sprite, nome, cores)` = um passo (Palette + AddRamp). Lei: `L` linear na faixa; `C = C_base·max(0, 1−curva·u²)`, `u = 2t−1`; `h = h_base + sentido·desvio·u`, com `sentido` = a volta mais curta até à matiz OKLCH do amarelo sRGB (109,77°, calculada pela `ph2d-color`, sem constante). Só usa `ph2d_color::perceptual::{to_oklab, from_oklab, oklab_to_oklch, oklch_to_oklab}` e `srgb::*_unit`. ⚠️ O `perceptual::gamut_map` existente NÃO serve: é o MINDE do CSS com JND 0,02, que corta canal e mexe no L.

**Oráculo:** `docs/PixelArt/ferramentas/oraculo_rampas_coloraide.py` (coloraide 8.14.1 MIT, `fit('srgb', method='minde-chroma', pspace='oklch', jnd=0)`) → `crates/ph2d-pixel-rampa/tests/fixtures/coloraide_rampas.json`, com cabeçalho; 3 bases DB32 (pele `d9a066`, água `5b6ee1`, folha `6abe30`) × 2 configurações (5 degraus/20°/0,6/[0,25;0,92] e 8/35°/0,9/[0,12;0,97]); **12 degraus cortados pelo croma**.

| Gate | Vermelho visto | Verde |
|---|---|---|
| `o_blob_pixel_volta_igual` (rampas, ciclo, guias, 2 faixas de som, Outline) | sim, antes do codec: MINOR 8; com MINOR 9 e sem os registos, «grava ⇒ lê ⇒ igual» falha | sim, e os mesmos bytes ao regravar |
| `a_rampa_gerada_e_a_do_coloraide` | só por mutação (o código veio antes do gate): sinal do desvio trocado, corte em L e sem corte, todos vermelhos | **pior ΔE OKLab 5,4e-5** (< 1e-3), comparado antes dos 8 bits; os 8 bits ficam a ±1 por canal |
| `a_rampa_gerada_e_monotona_em_l` | por mutação: o corte em L sangra (`water_5b6ee1_a5[3]: o corte mexeu no L`) | L e h por bits iguais ao plano; C ≤ plano; L dos 8 bits a subir |
| também: `cada_op_do_dado_tem_inverso_exacto`, `o_remap_da_paleta_leva_as_rampas`, `o_ciclo_roda_so_a_rampa_e_volta_no_periodo`, `a_rampa_nova_entra_na_paleta_num_passo` | — | sim |

Mutações (restauradas, árvore conferida): o sinal do desvio sangra no coloraide e sobrevive na monotonia, como esperado; corte em L sangra nos dois; sem corte sangra no coloraide. Por nome e verdes: core `rampas`, `codec`, `history`, `pure_core`, `user_data`, `tiles`; cena `o_pixel_formato_le_o_que_grava`; `check` da `ph2d-app-pixel` verde.

**Falta / para os próximos:** (1) o `Op::Whole` (Color Mode, New Palette from Sprite) passa a RECUSAR quando a paleta nova parte uma rampa: o R3 (`PreservaRampas`) tem de remapear as rampas no mesmo passo. (2) Rampa de um degrau é aceite; o `shade` do R2 decide se ela conta. (3) O `.ase` (user data `ph2d.ramps`/`ph2d.cycles`, efeito assado) e a folha JSON `meta.audio` não foram feitos (§5, são do R4/C/T). (4) A gravação de `Guias` com coordenada não finita não é recusada.

### Espaço na ph2d-app-pixel (09/11, agente)

**A regra do gate** (`architecture_the_shell_only_shrinks.rs`, `the_shell_only_shrinks_and_every_app_crate_too`): reprova `loc > tecto`; só obriga a descer quando `loc + censo_da_crate(tecto) < tecto`, com `censo = max(tecto/10, 500)` = **1 127** para 11 278. ⇒ cortar NÃO obriga o tecto a descer até 1 127 linhas de folga; o tecto fica **11 278** (inalterado) e a folga é o corte. ⚠️ O integrador pode recontar para `medido + folga_da_crate` (2 %, = 222 sobre 11 147 → 11 369): também deixa folga, nunca zero.

**O que moveu (comportamento idêntico, só endereço):**
1. O registo do que o quadro desenhou (`Drawn`, `DRAWN`/`PENDING`, `pick_at`, `caixa_local`, `begin_frame`) → `ph2d-pixel-render::desenhados` (nova API `desenhados`/`junta_pendentes`/`tira_pendentes`/`regista`); `draw.rs` re-exporta os três `pub`. −54.
2. O ficheiro dos pincéis guardados (`FILE`, `pasta_do_utilizador`, `le`, `shape_in`/`shape_out`) → `ph2d-pixel-ponte::pinceis` (irmão de `paletas`); `pinceis.rs` re-exporta os dois `pub`. −43.
3. Os diálogos modais (`dialog_open`, `dialog_text`) → `ph2d-pixel-ponte::dialogo`; `compose.rs` re-exporta (a shell chama `compose::dialog_*` sem mudar). −21.
Zero dependência nova (as três folhas já tinham as deps) ⇒ `Cargo.lock` intocado; nenhuma aresta sobe (folha → folha). Sem `cfg(feature)`, `cfg(test)`, `include_str!` nem censo por prefixo no código movido (HOWTO §2.4–2.7 conferidos); itens privados que passaram a `pub` levaram doc (§2.21: todos `pub` numa lib, nenhum privado sem chamador).

**Linhas:** 11 261 → **11 147** (−114, contadas por `conta()`, já com `rustfmt`); tecto 11 278 ⇒ **folga 131** (era 17). Abaixo do alvo de 300: o que resta na crate é quase tudo `impl PixelApp`/`&mut PixelApp` — as ondas anteriores já tiraram o puro (`ponte`, `cenas`, `formas`, `keys`). Os ~1 960 dos `smoke*.rs` dirigem o `PixelApp` e NÃO podem sair: família → família e folha → família são arestas proibidas (`architecture_no_dependency_climbs_a_layer`), e fingir-se `*-registry-init` seria contornar.

**Gates por nome (verdes):** `check` `ph2d-app-pixel` e `ph2d-host-desktop`; `ph2d-editor-core::the_shell_only_shrinks_and_every_app_crate_too`; `ph2d-host-pixel-scenes`: `the_slots_live_in_the_users_folder_and_survive_a_new_session`, `enquadr*` (3), `a_shell_pergunta_a_caixa_ao_pixel_e_a_luz`.

**Padrão de ligação recomendado para as 12 peças da §6** (131 linhas não chegam para ~12 peças à moda antiga): cada peça de UI nasce num `ph2d-panel-pixel-<peça>` (painel: pode depender de folhas, a família depende dele — como `export`, `tiles`, `slices`, `keys`, `history`), com a lei numa folha `ph2d-pixel-*` (`lint`, `guia`, `crt`, `rampa`); o painel publica pedidos num canal próprio (o `canal::{Pedido, Tela}` do `ph2d-panel-pixel-export`) e a app só os drena e aplica ao documento. Custo na app por peça: 1 linha `Cargo.toml` (não conta: não é `.rs`) + ~1 linha de registo/`frame` + ~3–6 linhas do braço que aplica o pedido ao `Doc` (`Op` já é do núcleo). **Total estimado: 12 × ≈6 ≈ 70–80 linhas** ⇒ cabe na folga de 131 com ~50 de margem. Regra para a onda: braço com mais de ~10 linhas na app = sinal de que a lei está do lado errado — desce para a folha.

### V — CRT (09/11, agente)

**Crate nova `crates/ph2d-pixel-crt`** (deps: `ph2d-gpu`, `wgpu 30.0.1`; nada do núcleo — a CRT é da VISTA e nunca grava no ficheiro, §11). `Cargo.lock` NÃO commitado: o diff dele tem também os hunks de `ph2d-pixel-guia`/`-lint` (outros agentes) — o integrador regenera.

**Params (dado da lei, não UI):** `CrtParams { scanline, mascara, curvatura, brilho, bloom }`, `Default` = desligada (`0,0,0,1,0`).

**A lei** (doc-comment de `lib.rs`), saída `W×H = w·escala × h·escala`:
- **Curvatura barril em ponto fixo Q14 (só inteiros)** — `u = (2x+1−W)·2¹⁴/W`, `r² = (u²+v²)>>14`, `f = 2¹⁴ + (k·r²>>12)` com `k = round(curvatura·2¹²)`, `du = (u·f)>>14`; fora de `[−2¹⁴,2¹⁴)` ⇒ preto opaco; senão `sx = ((du+2¹⁴)·W)>>15`, texel `sx/escala` (nearest). Escolhido inteiro para a placa e a CPU amostrarem o MESMO texel (f32 com FMA mudaria o `floor` em bordas ⇒ um pixel inteiro errado). Tectos com o recurso dito: `LADO_MAXIMO = 2¹⁴` (identidade exacta e `i32` sem transbordo; = o `max_texture_dimension_2d` 16384 das placas de secretária), `CURVATURA_MAXIMA = 1` (`u·f < 2³¹`).
- **Cor** por canal, em níveis: `g = brilho·(y ímpar ? 1−scanline : 1)·(c == x mod 3 ? 1 : 1−mascara)`, `out = clamp(floor(s·g + bloom·(e+s+d)/3 + ½))` (e/d = vizinhos horizontais na origem). `floor(+½)` nos dois lados (o `round` do WGSL é meio-para-par). Alfa = o da origem.

**WGSL** `src/crt.wgsl`: triângulo de ecrã cheio, `textureLoad` (sem sampler), uniforme de 32 B. **Rust** `gpu::CrtGpu::{new(device, formato), codifica(device, queue, encoder, origem, alvo, (w,h,escala), &params, timestamps)}` + `gpu::{prepara, le_alvo, crt_na_placa}` (gabarito/medida).

**Gates** (`ph2d-pixel-crt::testes`, verdes): `a_crt_desligada_e_a_identidade` (4 tamanhos × escala, e o mapa `k=0` = identidade até `2¹⁴−1`) · `a_curvatura_nao_mexe_o_centro_e_apaga_os_cantos` · `a_scanline_escurece_so_a_linha_impar` · `a_placa_e_o_gabarito_concordam_na_crt` (3 casos, incl. tudo no máximo; SKIP explícito sem adaptador) — **pior diferença 0 níveis** nos 3 (tolerância ≤ 1), RTX 5060 Ti. ⚠️ Red-first honesto: escrevi lei e gates na mesma passada; o vermelho veio das mutações.

**Mutações à mão (restauradas, `grep -c` = 1 em cada):** M1 trocar linha par/ímpar na CPU ⇒ vermelho em `a_scanline_…` e em `a_placa_e_o_gabarito_…`; M2 tirar a curvatura no WGSL (`f = UM`) ⇒ vermelho em `a_placa_e_o_gabarito_…`.

**Medida** (`mede_a_crt_na_placa_a_2048`, 512² × 4 ⇒ 2048², params ligados, timestamp query, 3 de aquecimento): **mínimo de 20 = 0,098 ms** na RTX 5060 Ti (perfil `dev`; a placa não depende do perfil). Contra 25,7 ms a 1 fio na CPU (passo 0b) ⇒ ~260×: a vista corre a CRT por quadro sem custo que se veja.

**A porta (onda seguinte):** a prévia e a janela do jogo criam UMA vez `CrtGpu::new(&device, formato_do_alvo)` e, por quadro, `codifica` com a textura do quadro composto (RGBA8 não-sRGB, tamanho do documento) e um alvo `w·escala × h·escala` (no lugar da ampliação nearest que já fazem; com `CrtParams::default()` basta não chamar). Num alvo `*Srgb` a placa codifica por cima — usar vista não-sRGB para os níveis da lei. O export chama `crt_cpu` (mesma resposta byte a byte).

**Falta:** UI dos params (cartão), onde os params moram (preferência da vista, não o `Doc`), a ligação nos dois sítios, e o `Cargo.lock` no integrador.

### R2 (09/11, agente)

Commit `b35bf15584` (núcleo + gates). Sem UI; `ph2d-app-pixel` e `ph2d-pixel-ponte`: zero linhas (a ponte compila sem mudar).

**Leis (`ph2d-pixel-core`):**
- **Shading na rampa do documento** — `ink::shade_in_document(sprite, selecção, dest, dir)` (o braço `Ink::Shading` de `ink::apply`): a rampa de `Sprite.ramps` que contém `dest` manda, ±1, pára nas pontas; **rampa de 1 degrau = não anda e manda mais que a selecção** (fecha a pendência 2 do R1). Fora de toda rampa do documento ⇒ a selecção de hoje, byte a byte (`color_laws` verde; o gate compara com `shade(sel, …)` sem rampas). RGB/Gray: a rampa são as cores da paleta (`value_of_rgba(palette[i])`).
- **Ink Dither** — `dither::PincelDither { nivel: 0..=64, matriz: Dither }` + `Stroke::with_dither(secundária, d)`: a célula `(x, y)` do CANVAS pinta a primária se `level(x,y) · 64/side² < nivel` (Bayer8 = a lei literal; Bayer2/4 escalam para que os 65 níveis valham em todas). Entra no funil `ink_value` ⇒ lápis, formas, spray e balde; a tinta (`Ink`) aplica-se depois. Não é variante de `Ink` de propósito: `ph2d-pixel-ponte::gesto::ink_out` faz `match` exaustivo e o `PixelInk` do `ph2d-editor-core` não tem Dither.
- **Smudge::Push** — `Kind::Push { forca: u8 }` (`forca/255` = 0..1) + `Smudge::arrasta(&sprite, centro, cells)`: cada célula recebe a cor da célula anterior do traço (desvio do cursor desde o carimbo anterior), mistura RGBA, volta à paleta por `Sprite::value_of_rgba` (a mesma métrica do Indexed; `convert.rs` intacto, não expõe «mais próxima»). O 1.º carimbo só regista. `arrasta` em Blur/Jumble = `dab`; `dab` com Push não faz nada. Blur/Jumble byte a byte (`fx_oracle` 8/8 com `oracle_trial_blur.json`).

**Gates (todos vistos VERMELHOS com stub antes):** `o_shading_anda_na_rampa_do_documento` (20≠4: andou na selecção) · `o_dither_fica_preso_ao_canvas` (9≠3) · `push_arrasta_a_cor_e_fica_na_paleta` ((6,6) sem cor) → verdes. Vizinhos por nome verdes: `color_laws`, `fx_oracle`, `dither_laws`, `stroke::`, `gradient`.

**Mutações à mão (restauradas, todas sangram):** sinal do Shading invertido (`:79`) · ler a selecção em vez da rampa (`:79`) · padrão relativo ao início do traço (`:151`) · Push lê a célula atual em vez da anterior (`:186`) · Push sem voltar à paleta (`:199`).

**Chaves i18n (para quem fizer a UI; nenhuma criada):** `pixel.ink.dither` («Dither») · `pixel.ink.dither.level` («Level») · `pixel.ink.dither.matrix` («Matrix») · `pixel.ink.dither.matrix.bayer2|bayer4|bayer8` («Bayer 2×2/4×4/8×8») · `pixel.ink.dither.tooltip` («Paints a fixed pattern between the foreground and background colors; the pattern stays put on the canvas») · `pixel.smudge.push` («Push») · `pixel.smudge.push.strength` («Strength») · `pixel.smudge.push.tooltip` («Drags the color under the brush along the stroke») · `pixel.ink.shading.tooltip` (revisto: «Steps one shade along the palette ramp under the brush; outside any ramp, along the selected colors»).

**A porta para a UI:** (1) `PixelInk::Dither` no `ph2d-editor-core::pixel_panel` + `Options.dither: PincelDither` (Level 0..=64, Matrix Bayer2/4/8); `gesto::ink` mapeia `PixelInk::Dither ⇒ Ink::Simple` e `ink_out` não precisa de braço novo. (2) No premir (`input.rs`, ao lado do `shading`), se o ink é Dither: `stroke.with_dither(s.value_of_rgba(bg), options.dither)` — botão direito troca primária/secundária. Mesmo no balde/formas. (3) Smudge: a tecla R passa a alternar Blur → Jumble → Push; no `degrade.rs` o Push chama `smudge.arrasta(s, at, &carimbo(..))` no premir e em cada célula do `line_path` do arrasto (em vez de `dab`), força num slider «Strength».

**Falta / avisos:** (1) a UI acima (outro agente). (2) Push em camada não Indexed fica em RGBA exacto; o alfa também mistura (empurrar para o transparente desbota). (3) ⚠️ Corri `cargo fmt -p ph2d-pixel-core` uma vez: pode ter reformatado `compose.rs`/`convert.rs`/`efeito.rs`/`lib.rs` em curso de outros agentes (só formato; não os commitei). (4) Cena de smoke 30 «Dither» ainda não montada.

### C + R4 (09/11, agente)

Commit `d61573253c` (núcleo + gates). Sem UI. `ph2d-app-pixel`: zero linhas. **O `.ase` NÃO foi feito** (prazo de 45 min): ver «Falta».

**Efeito de camada (uma lei, dois chamadores).** `efeito.rs`: `imagem_com_efeitos(s, layer, frame, pending) -> Option<Vec<u32>>` monta a imagem da camada no quadro (cel + traço em curso) e aplica cada efeito com o MESMO `Filter::at` do Edit ▸ FX ▸ Outline sobre o sprite inteiro, canais todos, sem selecção, gravando por `value_of_rgba` como o `filter::patch` (em Indexed, o índice mais próximo). `None` = sem efeitos, tilemap (o filtro recusa-a), ou nada a desenhar. `compose.rs` `Ctx::image` lê-a ANTES do blend; com efeito a área é o sprite inteiro (o contorno exterior que sai do bbox do cel aparece). Custo: `w·h·9` por camada com efeito por composição (sprite inteiro, porque o *Tiled* dá a volta); não medido em sprites grandes. `aplicar_efeitos(s, layer) -> Option<Op>` = **Apply**: `Op::Batch` de `Patch` (um por cel dono, os ligados uma vez, bbox = o do filtro) + `DocOp::SetLayerEffects` vazio — sem Op nova, inverso exacto pelo `apply`.

**Color cycling.** Porta nova `compose_window_em(sprite, frame, pending, region, purpose, t_ms: u64)` ao lado da `compose_window` (que é `t_ms = None`, a de hoje, byte a byte). Em Indexed a LUT de 256 lê `perm = ciclo_em(s.ramps(), t)`: a célula de índice `i` mostra `rgba_of_value(perm[i])`; o índice transparente NÃO roda (célula transparente fica transparente). RGB/Gray: a LUT não existe, o ciclo não faz nada.

| Gate (`tests/it/efeitos_camada.rs`) | Vermelho visto | Verde |
|---|---|---|
| `o_efeito_de_contorno_e_o_filtro` (RGB e Indexed × Outside/Inside × 2 quadros × View/Export; o contorno fora do bbox em (5,2)) | por mutação M1 (efeito desligado) | sim |
| `sem_efeito_e_a_composicao_de_hoje` (referência independente Normal opaco; efeito posto e tirado) | M1 sangra (o «tem de se ver») | sim |
| `aplicar_os_efeitos_assa_todos_os_quadros_num_passo_com_inverso_exacto` | M1 | sim |
| `o_ciclo_na_composicao_e_o_do_ciclo_em` (3 direcções, t=0 = hoje, volta no período, índice fora da rampa intacto) — o R1 já tem `o_ciclo_roda_so_a_rampa_e_volta_no_periodo`, não dupliquei | M3 (perm ignorada) | sim |
| `em_rgb_o_ciclo_nao_muda_nada` (células com canal = número de índice da rampa) | M2 (ciclo também em RGB) — **sobreviveu na 1.ª versão** (as cores da DB32 não têm R=1..4); o gate foi reforçado e então sangrou | sim |

Mutações restauradas (diff do `compose.rs` conferido). ⚠️ «Efeito DEPOIS do blend» não foi mutado literalmente; M1 (efeito ausente) cobre a mesma família nos três gates de efeito. Também verdes por nome: `layer_parity`, `normal_map`.

**Quem chama `t_ms` na onda seguinte** (hoje todos chamam a porta sem ciclo): prévia/canvas `crates/ph2d-pixel-render/src/view.rs:223` (e `:222`, o onion — `compose_with_onion_of` ainda não tem `_em`); Play/miniatura `crates/ph2d-app-pixel/src/cartao.rs:93`; export `crates/ph2d-pixel-io/src/lib.rs:28` e folha `crates/ph2d-pixel-io/src/folha_json.rs:19` (export assado leva o efeito AUTOMATICAMENTE, porque passa pela composição; o ciclo no export por quadro é decisão: o tempo é a soma das durações até ao quadro). Pixel Sprite na cena: `crates/ph2d-pixel-ponte` não chama a composição directamente — passa pelo `ph2d-pixel-io/src/lib.rs:28`.

**Chaves i18n (nenhuma criada):** `pixel.layer.effects` («Effects») · `pixel.layer.effects.outline` («Outline») · `pixel.layer.effects.apply` («Apply») · `pixel.layer.effects.apply.tooltip` («Bakes the effects into every frame of this layer and removes them») · `pixel.layer.effects.remove` («Remove») · `pixel.palette.cycle.preview` («Play color cycling»).

**Falta:** (1) **`.ase`** — desenho decidido, não codificado: na escrita, um clone com `aplicar_efeitos` (o PNG/folha/`.ase` vêem o contorno); a camada leva `ph2d.efeitos` (Vector de Map com os campos do `Outline`) no mapa do artista (como `ph2d.normal`, `ase/user.rs:199`) e cada cel dono assado leva `ph2d.cru` (Rect + Vector U8 dos bytes originais) ⇒ o leitor do Pixel repõe o cel cru e recria o efeito = ida-e-volta exacta; outros apps ignoram as props. Rampas/ciclos: o `ph2d-aseprite` não modela user data do chunk de paleta ⇒ vão no user data do SPRITE (`ph2d.ramps`, `ph2d.cycles`), desvio do §5. `ase.rs` tem 689/700 linhas: o código vai em `ase/efeitos.rs` com um gancho de 1 linha na escrita e outro na leitura. Gate `o_ase_leva_rampas_ciclos_e_efeitos` por escrever. (2) `compose_with_onion_of` sem `t_ms`. (3) A UI (outro agente).

### R3 (09/11, agente)

**Commit** `a1cec84d70`. **Núcleo** (`convert.rs`, sem deps): `trait Metrica { mais_proxima(palette, candidates, c) -> u8 }`, `Rgb` (o `two_nearest` de antes — a 2.ª dele era morta), `convert = convert_com(.., &Rgb)`. **`ph2d-pixel-rampa::quantiza`**: `Oklab` (euclides OKLab + alfa, empate → menor), `PreservaRampas` (se a OKLab mais próxima é SOLTA ela ganha; senão a rampa de menor distância no plano `ab` e, nela, o degrau de `L` mais próximo — monótona em `L`), `converte_oklab`, `converte_preservando_rampas`, `srgb8_de_oklch`.
**Oráculo**: `docs/PixelArt/ferramentas/oraculo_indexed_imagemagick.py` gera a `foto_p13` (64×64, LCG) e a `paleta_rampas` (pele 5 · água 4 · folha 6 · 3 soltas) e grava `tests/fixtures/imagemagick_indexed.json` (cabeçalho: ferramenta, versão, licença, comandos, data).
**Achado**: o `magick +dither -remap` (7.1.2-35 Q16 HDRI) NÃO é a cor exata: a árvore de cubos poda e erra **124/4096** píxeis (ex.: (183,146,127) → (186,218,120), d² 5242, com (222,160,128) a d² 1717); o Pillow 13 `quantize` erra **129**. Por isso o gate não é «ao byte»: onde diferimos, a nossa está ESTRITAMENTE mais perto, e as contagens ficam fixas. `-ordered-dither o8x8` (posteriza cada canal a 2 níveis antes do remap) difere **3529**, o FS **1089**: por desenho, não forçado.

| Gate (`ph2d-pixel-rampa`) | Vermelho visto | Verde |
|---|---|---|
| `o_indexed_rgb_de_hoje_nao_mudou_um_byte` (fixture `rgb_antes_r3.txt` gravada ANTES da porta; 4 dithers) | mutação empate `<=`: «o8x8: 1 píxeis mudaram» | ✓ |
| `o_indexed_rgb_e_o_do_imagemagick` | «nenhum 124 ≠ 0» (a premissa «ao byte» caiu) e contagens 0 ≠ 3529/1089 | ✓ |
| `manter_rampas_nunca_inverte_a_claridade` (armadilha 2 cores + foto) | mutações: croma em vez de L ⇒ `[1,0]`; ignorar as rampas ⇒ `[1,0]` | ✓ |
| `manter_rampas_com_dither_fica_na_paleta_e_nas_rampas` (None/o8x8/Old4/FS; `Op::Whole` aceita) | — | ✓ |
| `o_indexed_perceptual_nao_e_pior_que_o_pngquant` | `#[ignore = "pngquant por instalar (sudo do dono)"]` | — |

Medido: na `foto_p13` sem dither as inversões são 0 para rampas, OKLab e RGB — os dentes da régua são a armadilha (a OKLab pura dá `[1,0]`, as rampas `[0,1]`). Core `dither_laws`/`tiles` verdes por nome.
**Pendência 1 do R1**: o Color Mode mantém a paleta ⇒ nenhuma rampa a remapear; o `Op::Whole` aceita (gate acima). Quem mudar a paleta na conversão (New Palette from Sprite) continua a ter de levar as rampas.
**Chaves i18n para a UI**: `pixel.convert.metric` («Color matching»), `pixel.convert.metric.rgb` («Exact RGB»), `pixel.convert.metric.perceptual` («Perceptual (OKLab)»), `pixel.convert.metric.keep_ramps` («Keep ramps»), `pixel.convert.metric.keep_ramps.tip` («Each color goes to the ramp with the closest hue, and to the step with the closest lightness, so light and dark never swap inside a ramp.»).
**Falta**: o script ainda não CORRE o pngquant nem grava `pngquant.de_medio` (só regista a versão se instalado) — a acrescentar quando o dono instalar; a UI (seletor de métrica no diálogo Color Mode).

### G — guias (09/11, agente)

**Crate nova `ph2d-pixel-guia`** (dep só `ph2d-pixel-core`; a `ph2d-grid` NÃO entrou: a iso que se precisa é o passo inteiro 2:1 do `snap_pixel_angle`, e a grade do overlay são duas famílias de retas ±½ — 10 linhas, sem subir camada). Usa o `Guias` do R1 tal como está (nenhum campo novo no núcleo nem no codec).
**Leis.** Espaço das guias = células, centro da célula `(x,y)` em `(x+½, y+½)` (`centro`). `direcao_guiada(&Guias, origem, ponteiro) -> Cell`: famílias = a reta origem→PF de cada PF finito a mais de ½ px do centro da origem; **1 PF ⇒ + horizontal e vertical; 2 PF ⇒ + só a vertical** (as horizontais do objecto correm para os PF — com H em 2 PF o gate apanhou gestos «para o PF» a cair na horizontal); 3 PF ⇒ nada; iso ⇒ (2,1), (2,−1), (0,1). Escolhe o maior `|cos|` com o gesto (a reta serve nos dois sentidos). Passo inteiro (iso, H/V) = `k = Math.round(proj/|passo|²)` passos exactos (a lei do `snap_pixel_angle`). PF = varre o eixo dominante em comprimentos de **½ a 3× a mão**, cada coluna arredondada à reta, e fica a célula mais perto da mão cuja reta pelos centros, prolongada, passa a ≤ `TOLERANCIA_PF = 0,5` px do PF (sem nenhuma, a de menor falha). **Medido** (3 744 gestos, 2 PF): ±25 % ⇒ pior 1,498 px (`(10,15)→(30,16)`, inclinação 1/26 só existe a 26 células); ½..2× ⇒ 0,578 px (`(66,55)→(38,46)`); ½..3× ⇒ pior 0,499 px, o mais longo 2,04× a mão, 88 (2,4 %) fora de ±25 %. Sem família (guias vazias, só horizonte, PF não finito/na célula da origem) = `snap_pixel_angle` bit a bit.
`linhas_da_guia(&Guias, [x0,y0,x1,y1]) -> Vec<Segmento>`: feixe de `RAIOS_POR_PF = 24` retas por PF (meia volta, a reta a `a` e `a+π` é a mesma), o horizonte, a grade iso (inclinação ±½ pela `origem`, espaçamento vertical `PASSO_ISO = 8` células); cada reta cortada ao retângulo (Liang–Barsky); não finito não sai.
**Gates (tests/guia.rs)** — vermelhos com o stub (5 de 6; o do NaN passava com o stub `p`), verdes com a lei: `a_linha_guiada_vai_ao_ponto_de_fuga` (a `line_cells` do núcleo de o→d contém o e d; a reta prolongada passa a ≤0,5 px do PF de menor ângulo com o gesto; 208 dos casos visam o OUTRO PF — o fixture tem os dois regimes; comprimento ½..3× e ≤ 5 % fora de ±25 %), `a_linha_guiada_com_um_pf_tambem_vai_ao_horizontal_e_ao_vertical` (e 3 PF sem H/V), `a_iso_e_dois_por_um_exacto` (61×61 ponteiros: Δ é `(2k,±k)` ou vertical, cada linha da Bresenham leva 2 células), `sem_guia_e_o_snap_de_hoje` (3×51×51 + só horizonte), `pf_no_infinito_ou_na_origem_nao_da_nan` (∞, −∞, NaN, centro e dentro da célula da origem ⇒ o snap de hoje; `f32::MAX` finito usável; overlay finito), `o_overlay_e_o_feixe_e_a_grade_dentro_do_retangulo`.
**Mutações à mão (restauradas, `cmp` com a cópia):** M1 escolher a família de MENOR `|cos|` ⇒ 2 sangram; M2 tirar o passo inteiro do 2:1 (`round(t·u)`) ⇒ `a_iso_e_dois_por_um_exacto` sangra; M3 aceitar PF não finito/na origem ⇒ `pf_no_infinito…` sangra; M4 tirar a preferência pela mão ⇒ `a_linha_guiada…` sangra (≤ 5 %); M5 janela ½..2× ⇒ sangra (0,578 px).
**A porta (onda seguinte, não tocada):** (1) Linha: `crates/ph2d-pixel-ponte/src/gesto.rs:207-211` troca `snap_pixel_angle(*from, *to)` por `ph2d_pixel_guia::direcao_guiada(sprite.guias(), *from, *to)` (o `reshape` já recebe `sprite`; getter `ph2d-pixel-core/src/ramp.rs:98`) — sem guias é o mesmo bit. (2) Retângulo: `gesto.rs:172-176` (`boxed`, Shift = `square_box`) — com guias o canto oposto devia seguir a guia; decisão: o Shift do Retângulo em perspectiva é um quadrilátero (dois lados para cada PF), não uma caixa: precisa de forma nova, não cabe no `boxed`. (3) Lápis: `crates/ph2d-app-pixel/src/input.rs:311` (Shift = reta desde `doc.last_point`) — passar `at` por `direcao_guiada(…, last_point, at)`. (4) Overlay: `linhas_da_guia(sprite.guias(), rect visível em células)` desenhado pela `ph2d-pixel-render` como as linhas da grade; alças arrastáveis nos PF e na origem iso ⇒ `DocOp::SetGuias`.
**Chaves i18n para a UI** (EN, a criar): `pixel.guides` «Guides» · `pixel.guides.perspective_1` «Perspective (1 point)» · `pixel.guides.perspective_2` «Perspective (2 points)» · `pixel.guides.perspective_3` «Perspective (3 points)» · `pixel.guides.iso` «Isometric 2:1» · `pixel.guides.none` «No guides» · `pixel.guides.tip` «Hold Shift while drawing a line to follow the nearest guide: toward a vanishing point, or along a 2:1 isometric axis.» · `pixel.history.guides` «Move Guide».
**Falta:** o `Iso2x1` não guarda o tamanho do losango (`PASSO_ISO = 8` é do overlay; ir ao núcleo + codec quando a UI o pedir); o Shift do Retângulo em perspectiva (forma nova); a origem iso não muda a DIREÇÃO (só o overlay) — correcto para eixos, mas um «snap à grade iso» (destino num vértice do losango) não foi feito; Cargo.lock NÃO commitado (tem hunks de outros agentes — `ph2d-pixel-crt`, `ph2d-pixel-lint`); o meu hunk é só `ph2d-pixel-guia → ph2d-pixel-core`.

### A — linter (09/11, agente)

Commit `24ccda3b61`. Crate nova `ph2d-pixel-lint` (só `ph2d-pixel-core`). Sem UI. `lint(&Sprite, FrameId, &Regras) -> Vec<Aviso { regra, celulas: Vec<[u32;2]>, severidade }>` sobre `compose_window(.., Purpose::Export)` (referência fora); `lint_rgba(&Img, &Regras)` para quem já tem o RGBA. `Regras` = um `bool` por regra + `limite_cores` (32 = DB32). Opaco = `alfa > 0`; mesma cor = RGBA igual. Leis (doc-comment de cada `src/regra/*.rs`):
- **Orphan** (Baixa): opaco com os 4 vizinhos transparentes E nenhum 8-vizinho da mesma cor. Brilho de 1 px num fundo pintado tem vizinhos 4 opacos ⇒ não acusa.
- **Doubles** (Média): `b` cujos ÚNICOS 4-vizinhos da cor são `b+d1`, `b+d2` perpendiculares, o quarto `b+d1+d2` de outra cor, e um braço de 1 px. Braços ≥ 2 (canto de caixa) = intencional; os dois cantos do mesmo Z = um aviso.
- **Jaggies** (Média): linha de 1 px (4-vizinhos; diagonal só sem 4-vizinho comum; grau ≤ 2) partida em trechos de eixo e sentido constantes; corridas certas se `max−min ≤ 1` (também sem as pontas cortadas) ou ≤ 1 extremo interior que muda ≤ 1 de cada lado. Senão acusam-se as corridas-extremo (3-1-3 ⇒ o 1).
- **Banding** (Média): pixel `B` entre dois vizinhos opostos `A ≠ C` com luma Rec.601 de `B` estritamente entre; grupo 8-ligado da mesma cor, do MESMO eixo e do mesmo par `(A, C)` (fronteiras paralelas), ≥ 3 células. Só largura 1.
- **Pillow** (Alta): profundidade = distância de xadrez ao contorno (transparente/borda); região 4-ligada com fundo ≥ 3, cada nível de UMA cor e ≥ 3 cores entre níveis ⇒ aponta os anéis que não são nem o contorno nem o miolo.
- **Too many colors** (Baixa): cores distintas > limite ⇒ as células das mais raras (uso desc., empate RGBA). **Quase-iguais FORA**: precisa de OKLab, e o núcleo do lint não o tem; a distância RGB mente nos escuros — se o painel o pedir, vai pela `ph2d-pixel-rampa`/`ph2d-color`.

**Fixtures (código, `tests/it/fixture.rs`):** `vicios_p13` 16×16 — órfão (1,1) cor única · Z (7,3)+(7,4) · 3-1-3 ⇒ (3,7) · faixa (1..4,11) · almofada 8×8 em (8..15,8..15), anel 2–3 · cores a mais com limite 7 ⇒ (1,1). Limpos: caixa DB32 (contorno, sombra 2 px à direita/baixo, brilho, junta escura entre tábuas) e bola (disco r 6,5). `heroi_p14` NÃO usado: mora na `ph2d-pixel-cenas` (dependência nova, e arrasta bevy).

| Gate | Vermelho visto | Verde |
|---|---|---|
| `cada_vicio_e_achado_onde_esta` (exactamente as células; com limite 32 nada de cores) | por mutação: Doubles, Jaggies, Too many colors | sim |
| `o_sprite_limpo_nao_tem_aviso` | sim, de verdade: a bola com a 4-fronteira teve 4 cantos duplos REAIS nos 45° (o artista apaga-os; a fixture agora também) e a caixa deu um falso Banding no anel à volta do brilho ⇒ a lei ganhou «mesmo eixo, mesmo par»; por mutação: Orphan, Doubles, Banding, Pillow | sim |
| `desligar_a_regra_cala_so_ela` · `o_lint_de_256_cabe_ao_vivo` | — | sim |

**Mutações** (script com `assert` de contagem, restauradas e conferidas): Orphan sem «4 vizinhos transparentes» · Doubles braço `≥ 1` · Jaggies sem extremo · Banding sem a luma intermédia (sobrevivia; a junta escura da caixa é que a fez sangrar) · Pillow sem «nível de uma cor» · cores sem a ordem por uso — **6/6 sangram**.
**Tempo:** 256×256 com estrutura (blocos de 12 cores + 2000 pixels de ruído, 53 avisos), composição incluída, **mínimo de 10 = 10,9 ms em `--profile smoke`** (80,8 ms em dev). Cabe por edição (não por quadro): o painel corre-o ao soltar o traço, não a cada frame.
**Chaves i18n** (EN): `pixel.lint` «Lint» · `pixel.lint.rule.orphan` «Orphan pixels» · `.rule.doubles` «Doubles» · `.rule.jaggies` «Jaggies» · `.rule.banding` «Banding» · `.rule.pillow` «Pillow shading» · `.rule.too_many_colors` «Too many colors» · uma `.tip` por regra (ex. `pixel.lint.rule.doubles.tip` «An L-shaped corner in a 1-pixel line where a diagonal step is enough.») · `pixel.lint.severity.low|medium|high` «Low|Medium|High» · `pixel.lint.color_limit` «Color limit» · `pixel.lint.none` «No issues found» · `pixel.lint.count` «{n} issues».
**Falta:** painel e overlay (onda seguinte); Banding só de largura 1; Jaggies não lê S-curvas com inflexão brusca (acusa) nem pontas de 1 px depois de corridas longas em curva (aceita); quase-iguais (OKLab); o Cargo.lock NÃO commitado (tem hunks de outros agentes: `ph2d-pixel-crt`, `ph2d-pixel-guia`) — o meu hunk é só `ph2d-pixel-lint → ph2d-pixel-core`.

### U-Paleta (09/11, agente)

**Commit** `2f2be34070`. Só a parte 3 (o `t_ms` em todo leitor) ficou feita; a UI (1, 2, 5) e a 4 NÃO (prazo de 45 min; o diagnóstico da UI está abaixo). `ph2d-app-pixel`: **+2 −3 linhas** (`cartao.rs`: a miniatura passa a `ph2d_pixel_io::frame_rgba`).
**Lei do tempo (decidida):** `instante_do_quadro(sprite, frame) -> u64` em `ph2d-pixel-core/src/cycle.rs` = soma das durações antes do quadro. É o `t_ms` de TODO leitor: função só do documento e do quadro ⇒ o `key (stamp, frame, onion)` da `SpriteImage` e a cache por quadro continuam válidos (custo zero sem ciclo; em RGB a imagem é a de hoje).
**Leitores ligados:** vista `ph2d-pixel-render/src/view.rs::window` (sem papel vegetal `compose_window_em`; com ele a porta nova `compose_with_onion_em(.., (under, layer), Option<t>)` — `compose_with_onion_of` delega com `None`, os fantasmas rodam no mesmo t) · **Pixel Sprite na cena**: desenha pela MESMA `d.view` com o quadro do `quadro_a_desenhar` (`tiles.rs::compoe`), logo herda sem linha nova · miniatura/Play `cartao.rs` · export `ph2d-pixel-io::frame_rgba` e `compose_picked` (o `render::composto` de todo export por quadros). `folha_json.rs:19` é IMPORT de PNG (sem ciclo: não mexi).

| Gate (`ph2d-pixel-render/tests/ciclo.rs`) | Vermelho visto | Verde |
|---|---|---|
| `o_ciclo_na_cena_e_o_do_editor` (Indexed, rampa 1..4 a 400 ms, 4 quadros × 100 ms: vista com e sem papel vegetal = export = `ciclo_em(instante)`, 4 degraus distintos) | M1 vista com `t = 0` (quadro 1 troca índices) · M2 `compose_picked` sem t | ✓ |

Por nome, verdes: core `ciclo` (3), render `onion`, io `folha` (5). ⚠️ O `check` da `ph2d-app-pixel` não correu verde no fim: quebra de OUTRO agente a meio (`ph2d-panel-pixel-preview`: `TextInputState`, `CardInk` privado; antes `ph2d-editor-core` E0308). O meu hunk lá só troca a chamada por `frame_rgba` (dep já existente).
**Limite consciente:** sprite de UM quadro tem `t = 0` sempre ⇒ o ciclo não anda num cenário estático (a cachoeira de um quadro). A cura padrão-ouro é um relógio contínuo do Play somado ao instante (`t = instante + relógio_do_play`) entrando no `key` da `SpriteImage` só quando `!ramps().cycles.is_empty()`; pede `set_cycle_clock` na `SpriteImage` e ~3 linhas na app (o relógio da P14). Não feito.
**Falta (para o próximo agente U-Paleta):**
1. **UI (New Ramp…, faixa, Cycle).** A grelha da paleta mora em `ph2d-panel-pixel-tools/src/paint_paleta.rs` e lê `PixelPanelInfo` (`ph2d-editor-core/src/pixel_panel/mod.rs`, que outros agentes estão a editar). Padrão recomendado: crate nova `ph2d-panel-pixel-rampa` com `canal::{Pedido::{NovaRampa{cores}, FazRampa{indices}, TiraRampa(RampId), Ciclo{ramp, Option<Cycle>}}, Contexto{ramps, modo, fg}}` + o diálogo em cartões; a faixa pinta-se pela própria crate chamada de `paint_paleta.rs` (painel → painel é PROIBIDO pela `architecture_no_dependency_climbs_a_layer` lei 2 ⇒ a faixa tem de viver na `panel-pixel-tools` com os dados no `PixelPanelInfo`, OU o diálogo+faixa num painel próprio registado como o Export). Braço na app: `ph2d_pixel_rampa::op_rampa_nova`, `DocOp::AddRamp/RemoveRamp/SetCycle` (≈ 8 linhas). Cycle desligado em RGB com tooltip.
2. **New Palette from Sprite × rampas — decisão (padrão-ouro):** LEVAR, nunca recusar: a paleta nova = cores usadas ∪ cores das rampas (a rampa é dado autorado; «a chave do artista manda mais que a correção automática»), com as rampas remapeadas por cor no mesmo `Op` (o `Op::Whole` já valida). Lei pura na `ph2d-pixel-ponte::leis` ou `ph2d-pixel-rampa`, não na app (`paleta.rs::do_sprite`). Não codificado.
3. Gates de ponteiro da UI (New Ramp ⇒ rampa no documento; ligar Cycle ⇒ ciclo no documento) — dependem de 1.
**Chaves i18n propostas (nenhuma criada):** `pixel.palette.new_ramp` «New Ramp…» · `.ramp.base` «Base» · `.ramp.steps` «Steps» · `.ramp.hue_shift` «Hue shift» · `.ramp.chroma` «Chroma» · `.ramp.light_range` «Light range» · `.ramp.make` «Make Ramp» · `.ramp.remove` «Remove Ramp» · `.cycle` «Cycle» · `.cycle.period` «Period (ms)» · `.cycle.direction.forward|reverse|ping_pong` «Forward|Reverse|Ping-pong» · `.cycle.rgb_off.tip` «Color cycling works only in Indexed mode: in RGB a pixel stores its color, not a palette index.»

### U-Guias (09/11, agente)

**Commit** `b65943d66b` (o braço da Linha em `ph2d-pixel-ponte/src/gesto.rs` — Shift ⇒ `direcao_guiada(sprite.guias(), from, to)` — e o `.chain(&ids::GUIDE_ACTIONS)` do `seam.rs` entraram por arrasto no `6e47fa23be` do U-Pintura, que commitou os ficheiros inteiros; o `b65943d66b` fecha-os: sem ele o HEAD não compilava).
**Núcleo + codec:** `Iso2x1 { origem, passo: u16 }` (`Iso2x1::PASSO = 8`, `Default` à mão); Sprite 19 **tag 6** = passo (sem campo = 8; 0 recusa `guide.iso_step`); sem leitor velho, MINOR fica 9. `o_blob_pixel_volta_igual` grava `passo: 5` (≠ omissão ⇒ prova a tag). A grade do overlay usa `iso.passo`; `PASSO_ISO` saiu da `ph2d-pixel-guia`.
**Lei (`ph2d-pixel-guia`):** `guias_iniciais(pontos, iso, (w,h))` — horizonte a h/3; 1 PF a (w/2, h/3); 2 PF a w/3 e 2w/3 no horizonte; o 3.º (vertical) a (w/2, 2h), fora do sprite; iso no meio da borda de cima. `alvo_do_lapis` (guias vazias ⇒ o ponteiro, o Shift de hoje); `alcas`/`move_alca` (mexer o PF 1 ou 2 leva o horizonte e o outro PF; o 3.º não).
**Interacção (`ph2d-pixel-ponte::alcas`):** premir a ≤ `ALCA_GRAB_PX` (10 px de ecrã) de uma alça agarra-a, seja qual for a ferramenta (vem antes do `cell`: a alça pode estar fora do sprite); arrastar = prévia em thread-local (`vistas`), ao meio píxel, sem passo; soltar = UM `DocOp::SetGuias` «Move Guide» (nada mexido ⇒ nenhum passo). Menu ⇒ `op_do_menu` (as mesmas guias ⇒ nenhum passo).
**Overlay (`ph2d-pixel-render::guias::desenha`):** `linhas_da_guia` recortadas aos 4 cantos da janela em células + discos nas alças (`GUIDE_HANDLE_PX`), `Accent` / `BgElev`, `LINE_PX`; chamado no `draw.rs` logo depois de `draw_doc_grids` (acima da grade, abaixo de selecção e cursor).
**UI:** `PixelEditAction::Guides(PixelGuides{None, Perspective1..3, Isometric})` (editor-core, append); `GUIDE_ACTIONS` (5 ids `pixel.view.guides.*`) numa secção **Guides** no fim do menu Edit, com a dica — ⚠️ o painel Pixel **não tem menu View**; a secção mora no Edit, como o Grid Settings (que no Aseprite também é View). Chaves EN em `ph2d-i18n/src/pixel_guias.rs` (as do relatório G + `pixel.guides` e `pixel.history.guides`).
**Linhas gastas na `ph2d-app-pixel`: 19** (input 14: `use` 1, premir 6, lápis 1, mover 1, soltar 4 + 1 modificada; draw 4; edicao 1).
**Gates** `shells/pixel-scenes/tests/it/guias.rs`, pelo gesto real (press/move/release com Shift, cena 9): `escolher_perspective_2_poe_as_guias_e_arrastar_a_alca_move_o_pf_num_passo` (menu = 1 passo; arrasto não muda o documento a meio; soltar = 1 passo; Ctrl+Z volta; não pinta) · `a_linha_com_shift_segue_a_guia` (5 destinos; a reta pintada = `line_cells(o, fim)` e passa a ≤ 0,5 px de um PF, ou é a vertical; ≥ 3 visam PF) · `o_lapis_com_shift_segue_a_guia_e_sem_guias_e_o_de_hoje` · `sem_guias_o_shift_da_linha_e_o_de_hoje` (= `snap_pixel_angle`, 5 destinos) · `o_iso_poe_a_grade_e_a_linha_vai_a_dois_por_um`. **Verdes 5/5.** Vermelho: o 1.º e o do lápis reprovaram na 1.ª corrida (meio píxel do arrasto; a origem já pintada) — defeitos do TESTE, corrigidos; ⚠️ a mutação da porta (Linha ⇒ `snap_pixel_angle`) foi aplicada e restaurada mas o alvo não compilava por código alheio a meio: **o vermelho por mutação NÃO foi visto**. Verdes também: `ph2d-pixel-core::o_blob_pixel_volta_igual`, os 6 da `ph2d-pixel-guia`.
**Falta:** (1) mutações a correr (Linha ⇒ snap; `alvo_do_lapis` ⇒ ponteiro; `solta` sem op; `vistas` = documento). (2) Um gate que CLIQUE no id `pixel.view.guides.perspective_2` no painel (o meu usa o `PixelRequest` que o clique emite). (3) **Cargo.lock**: as arestas ponte → guia e render → guia não foram commitadas (o lock tem hunks alheios). (4) O Retângulo em perspectiva (quadrilátero para os PF: forma nova, fora do `boxed`). (5) Sem UI para o tamanho do losango (o dado e o codec existem; o menu põe 8). (6) A alça agarra com QUALQUER ferramenta a 10 px; pintar mesmo em cima de um PF pede afastá-lo antes — decidir com o dono. (7) A alça não tem cursor/hover próprio nem entrada no `.ase` (user data `ph2d.guias`, §5).

### U-Lint (09/11, agente)

Crate nova **`ph2d-panel-pixel-lint`** (Window ▸ **Lint**, nasce fechada, cartões Rules + Issues). Rules: uma caixa por regra (tooltip `pixel.lint.rule.*.tip`) e **Color limit** (campo 1..256, o `u8` do índice). Issues: um grupo por regra (nome, `N issues · severidade` à direita) e um aviso por linha (`x, y` da 1.ª célula). Lei em `canal.rs`: a família chama `canal::corre(doc, quieto)` a cada quadro; o lint SÓ corre com a janela aberta, sem traço em curso (`app.gesture.is_none()`) e com (chave, quadro, `stamp`, regras) mudado — por edição/troca de quadro, nunca por quadro de ecrã. Clicar num grupo/aviso marca as células e deixa um foco; `canal::foca(area, screen_to_world)` devolve o delta da câmera (a 1.ª célula ao meio da área do desenho, com o afim do último desenho); `canal::desenha` chama **`ph2d_pixel_render::draw_lint_marks`** (ficheiro novo `lint_marks.rs`: contorno de cada célula com halo, cor `ColorToken::Warn`). Corrigir o pixel ⇒ a corrida seguinte tira o aviso e a marca.

**Registo (sítios):** ids `PIXEL_LINT_PANEL`/`TOPBAR_PIXEL_LINT` em ficheiro novo `ids/chrome/pixel_lint.rs` (o `pixel.rs` está no índice de outro agente) · `panel_ids` (const, nasce fechado, canónico) · `menu_bar::MODULE_TRUTHS` (26→27) · `menu_tables` (Window, rótulo `pixel.lint`) · `hero/paint.rs` (z-walk) · `pixel_toggle` · `topbar` (registo do clique) · `ph2d-panel-registry-init` (dep opcional, feature `panel-pixel-lint` no default, push + contagem) · shell (feature + `SHELL_DRIVEN_PANELS`) · `PAINEIS_COM_ID_DERIVADO`. i18n: `crates/ph2d-i18n/src/pixel_slices/lint.rs` (submódulo de `pixel_slices.rs`: a `lib.rs` está a 710/700 com hunks alheios — zero linhas minhas lá).

**`ph2d-app-pixel`: 9 linhas** (`panel.rs` 3: corre · `frame.rs` 4: foca → `camera.center` · `draw.rs` 2: desenha) + 1 linha `Cargo.toml`.

| Gate (`crates/ph2d-panel-pixel-lint/tests/it/main.rs`, fixture `vicios_p13` por `#[path]`) | Vermelho | Verde |
|---|---|---|
| `abrir_o_lint_na_fixture_da_seis_grupos_pintados` (6 grupos, cada grupo e caixa com rect, Color limit) | — | sim |
| `clicar_no_doubles_centra_o_canvas_na_celula_e_corrigir_tira_o_grupo` (clique real ⇒ marcas `[7,3],[7,4]`; delta = meio da célula − meio da área; foco drena 1×; apagar (7,3) ⇒ grupo e marca somem) | mutação: marca que não limpa | sim |
| `o_lint_nao_corre_a_cada_quadro_nem_durante_o_traco_nem_fechado` (10 quadros = 0 corridas; traço = 0; soltar = 1; fechado = 0) | mutação: sem a guarda `quieto` | sim |
| `a_caixa_da_regra_cala_o_grupo_dela` | — | sim |

4/4 verdes (`PH2D_SUITE="motivo: crate NOVA"`), 2 mutações sangram (restauradas). **Não corridos:** `ph2d-panel-registry-init` (quantas_entradas, janelas no menu Window, `paineis_armados`) e o check da `ph2d-app-pixel`/shell — a árvore não compila por código alheio a meio (`ph2d-pixel-ponte`: `outline_of`; `smoke.rs`: `ColorModeChoice.metric`). **Falta:** correr esses gates na árvore verde; o centrar NÃO foi provado sobre o `PixelApp` real (só o delta); `paineis_armados` pode pedir uma entrada `pixel_lint`; a 3.ª mutação (foco que não drena) não feita; smoke do dono.

Commit: ver `git log --oneline -- crates/ph2d-panel-pixel-lint` (o Cargo.lock vai no mesmo commit só com os hunks desta crate; o `draw.rs` e o `ph2d-pixel-render/src/lib.rs` só com as minhas linhas).

### U-Pintura (09/11, agente)

**Commit** `6e47fa23be` (um, por índice: só os hunks deste agente; `input.rs`, `ids/chrome/pixel.rs`, `ponte/Cargo.toml` e `pixel-scenes/tests/it/main.rs` tinham hunks alheios e foram montados a partir do HEAD). **`ph2d-app-pixel`: +11 linhas líquidas** (27+/16−: `input.rs` +7, `degrade.rs` +5, `paleta.rs` −1, `smoke.rs` +1; `input_teclas`/`tiles` 0).
- **Ink Dither**: `PixelInk::Dither` + `PixelOptions.{dither_level (32), dither_matrix (Bayer4), push_strength (255)}` (`ph2d-editor-core`, append). `gesto::ink` ⇒ `Ink::Simple`; `gesto::pincel_dither(&options, tool)` (ponte) dá o `PincelDither` (nunca na borracha/Gradient/Blur/Jumble/Push); o premir faz `stroke.with_dither(value_of_rgba(outra), d)` — `outra` = bg no esquerdo, fg no direito. Vale para lápis, spray, balde, linha e formas (o funil `ink_value` do R2).
- **Painel** (`ph2d-panel-pixel-tools`): ficheiros novos `ids_pintura.rs` (ids, faixas Level 0..=64 / Strength 0..=255, `sem_tinta`, `e_numero`) e `paint_pintura.rs` (Level + Matrix Bayer 2×2/4×4/8×8 sob a fila de tintas só com Dither; Strength só com o Push). `paint.rs` ficou em 600/600 e `apply_event` abaixo de 200 (as condições foram para `ids_pintura`).
- **Push**: `PixelTool::Push` no grupo Smudge (ícone `Smear`, dica `pixel.hint.push`, histórico `pixel.history.push`); `R` cicla Blur → Jumble → Push (`input_teclas.rs`, `tool_keys`). `degrade.rs` chama `Smudge::arrasta` no premir e em cada célula do `line_path` (em Blur/Jumble `arrasta` = `dab`, por isso o mesmo caminho).
- **Shading**: o bloco do painel lê `pixel.ink.shading.tooltip` («…along the palette ramp under the brush…; outside any ramp, along the selected colors»). Condição de mostrar (selecção < 2) inalterada.
- **Color Mode ▸ Indexed ▸ Color matching**: `ColorModeChoice.metric: PixelColorMetric {ExactRgb (omissão), Perceptual, KeepRamps}`, `ids::COLOR_MODE_METRICS`, pintada nas duas saídas do diálogo (cartões e clássica), registada e despachada (`set_color_mode_metric`). `ph2d_pixel_ponte::leis::converte(sprite, c)` escolhe `convert` (Rgb) / `converte_oklab` / `converte_preservando_rampas`; a ponte passa a depender de `ph2d-pixel-rampa`.
- **i18n**: `crates/ph2d-i18n/src/pixel_pintura.rs`, mas como submódulo da `pixel_pincel.rs` (a `lib.rs` está em 700/700; uma linha `mod` reprovava o tecto).

**Gates** (`shells/pixel-scenes/tests/it/pintura_p13.rs`; `degrade.rs` actualizado: o 3.º R é Push): `o_ink_dither_do_painel_pinta_o_padrao_preso_ao_canvas` (pedido Options do painel ⇒ cada célula do traço = `PincelDither::primaria`) · `r_tres_vezes_e_o_push_e_o_arrasto_empurra_a_cor` · `o_color_mode_com_keep_ramps_converte_pela_lei_das_rampas` (cena 5, OK do diálogo = `converte_preservando_rampas` e ≠ Exact RGB). ⚠️ **NÃO CORRIDOS — nem vermelho nem verde**: o meu ficheiro compila, mas o alvo `it` não compilava por `tests/it/guias.rs` (E0502, agente G) e, antes, a `ph2d-panel-pixel-preview` (agente V). Check verde de `ph2d-editor-core`, `ph2d-panel-pixel-tools`, `ph2d-pixel-ponte`, `ph2d-pixel-keys`, `ph2d-i18n`.

**Falta:** (1) correr os 3 gates + `shift_p_picks_the_gradient_and_r_toggles_blur_and_jumble`, `the_pixel_color_mode_dialog_is_alive`, `seam_pixel` (a coluna tem 1 ferramenta a mais no grupo Smudge; contagens de PLAIN/SHAPES_AT subiram) e ver vermelho por mutação (Dither ⇒ `None`; R sem Push; métrica ignorada). (2) O gate do Keep ramps prova o FIO (cena 5 sem rampas ⇒ = OKLab); a ordem de claridade fica no gate do R3. Um gate de ponteiro que CLIQUE no id `pixel.opt.ink.dither` no painel (molde `seam_pixel.rs`) não foi escrito — o meu usa o `PixelRequest::Options` que o clique emite. (3) **`Cargo.lock`**: a aresta ponte → `ph2d-pixel-rampa` não foi commitada (o lock tem hunks alheios). (4) Tooltip `pixel.ink.dither.tooltip`/`pixel.smudge.push.tooltip`/`keep_ramps.tip` existem como chaves mas nenhum `set_tooltip` os liga ainda. (5) Ícone próprio do Push (usa `Smear`).

### U-Camada (09/11, agente)

Commits `94999b5ab1` (UI + gates), `67bfcdb55a` (o `mod pixel_efeitos` no `ph2d-i18n/src/lib.rs` partilhado, só estes hunks, por índice temporário) e `d09703ad19` (`.ase`, por um fork). **`ph2d-app-pixel`: 4 linhas** (o braço `LayerEffect` em `camadas.rs`).

**UI.** Layer Properties ▸ **Effects**: caixa **Outline** (liga com a cor PRIMÁRIA, `info.fg`; em Indexed o `value_of_rgba` do filtro dá o índice mais próximo); com efeito, **Inside**, **8 neighbors** (SQUARE; desligado = CIRCLE, o padrão do Aseprite), **Use Primary Color**, **Apply** e **Remove** — sem efeito estes não se pintam (nenhum controlo morto). Ícone `FilterOutline` na linha da camada com efeito (painel e timeline; clique = activa a camada). Vocabulário: `PixelLayerRow.outline: Option<PixelOutline>`, `PixelLayerEffect::{On, Off, Inside, Square, Color, Apply}`, `PixelRequest::LayerEffect(layer, e)`. A tradução para `Op` mora em `ph2d-pixel-ponte::camadas::efeito_op` (um `SetLayerEffects`, ou o `Batch` de `aplicar_efeitos`); `outline_of(l)` dá a linha. Nome do passo: «Layer Effects» / «Apply Effects». Strings em `ph2d-i18n/src/pixel_efeitos.rs`.

| Gate | Vermelho visto | Verde |
|---|---|---|
| `the_pixel_layer_effects_answer_a_real_pointer` (registry-init: ponteiro real nas 6 peças ⇒ o pedido certo; ícone só com efeito; Apply não pintado sem efeito) | braço `Toggled` do `seam.rs` desligado ⇒ `left: []` | sim |
| `the_layer_outline_from_the_panel_shows_bakes_in_one_step_and_undoes_alive` (pixel-scenes, cena 7, «Opacity 50%»: ligar = 1 passo e a célula FORA da caixa ganha o contorno; Apply = 1 passo, efeito sai, imagem igual; Ctrl+Z = efeito vivo; 2.º Ctrl+Z = sem contorno) — é o `aplicar_o_efeito_e_um_passo` do §7 pelo caminho real | Apply como `SetLayerEffects` vazio (sem assar) ⇒ «e a imagem fica a do efeito» | sim |

Mutações restauradas (`grep -c` conferido).

**`.ase` (fork).** `ase/efeitos.rs` (`assa` na escrita, `repoe` na leitura; `ase.rs` 694/700): sprite sem rampas nem efeitos sai byte a byte como hoje; senão um clone leva `ph2d.ramps`/`ph2d.cycles` no user data do sprite, `ph2d.efeitos` na camada, o efeito assado por `aplicar_efeitos` e `ph2d.cru` (bbox + bytes) em cada cel dono assado (gravado DEPOIS do assado: o Patch que alarga o bbox refaz o cel). Leitura: `AddRamp`/`SetCycle`, `SetLayerEffects`, Patch com o bbox cru; propriedade não entendida fica no user data. Gate `o_ase_leva_rampas_ciclos_e_efeitos` (`tests/ase_efeitos.rs`): o cel cru lido só pelo `ph2d-aseprite` tem o contorno; grava ⇒ lê ⇒ igual, 2.ª gravação byte a byte; vermelho com M1 leitura desligada, M2 sem assar, M3 ciclos não gravados. Verdes por nome: filtros `ase` (11, incl. `o_libresprite_le_o_nosso_ase`), `normal`, `user`.

**Oráculo LibreSprite** (GPL-2.0-only, só corrido): `libresprite -b p13_efeitos.ase --save-as out.png` funciona; o PNG 8×8 = o nosso composto assado, **0/64 células diferentes**. Comparação manual (o gate grava as entradas só com `PH2D_ORACULO_DIR`).

**Falta:** o LibreSprite como fixture com gate; `ph2d.cru` como Vector U8 dobra o tamanho nas camadas com efeito; cel ligado numa camada com efeito não testado; a crate `ph2d-pixel-io` inteira não correu; sem controlo de cor por ÍNDICE explícito (a cor é a primária); custo do efeito em sprites grandes por medir (C + R4); smoke do dono.
