# P14 — O sprite vive na cena: o plano (09/11)

> Linha `line/PixelArt`, HEAD `f7688dbaad`, 7 commits à frente do `main` (P12 + pendências, smoke OK do dono).
> Plano do módulo: [02_plano_modulo.md §P14](02_plano_modulo.md). Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md).
> Molde: [P11_plano.md](P11_plano.md). Dono 09/11: P14 confirmada («siga»); oráculo que se CORRE = app gratuito triado (§3).
> **Estado: plano escrito; nada construído.**

## 0. O que entra

| Grupo | O artista faz | Hoje |
|---|---|---|
| **A — Animação** | as tags do Pixel Sprite tocam no Play como clips; Autoplay e a tag inicial no Inspector; `play("run")` no Luau | o Pixel Sprite mostra sempre o quadro do editor |
| **P — Pivô** | escolhe a slice cujo pivô é a origem do objecto (pode mudar por quadro) | a origem é fixa |
| **C — Colisão** | «Add Collider» numa slice: nasce a forma filha do corpo e segue a key da slice a cada quadro | slices não saem do módulo |
| **L — Luz** | o Pixel Sprite bloqueia, recebe e é sombreado pela camada **Normal Map** pintada com a roda de normais | invisível para a Luz 2D; a roda só escolhe cor |
| **X — Saída** | a folha leva `_n.png` (normais); export **Godot SpriteFrames** (`.tres`) | folha + JSON, `.ase` |
| **I — Sprite Inspector** | abre o `.ase`/folha que o Pixel grava com pivô, colliders e normal map | tags e durações sim; slices e normais não |

**Fora, de propósito:** a ponte Painter ↔ Pixel (§P14 do módulo: quando o Painter fechar) · P13 (onda própria).

## 1. Conferido no código (09/11, três exploradores + leitura direta)

| Afirmação | Conferido |
|---|---|
| O Pixel Sprite no mundo | `PixelSprite(u32)` `ph2d-ecs/src/pixel_sprite.rs:15`; doc fora do mundo (`state.rs` `Doc`), blob `PH2DPIXEL` `codec.rs` **`MINOR = 6`** |
| Desenho | `draw.rs:77-160` `art_of` → `doc.view.draw(scene, m)` na **VectorScene** (Vello), com `Transform` do mundo |
| Tag | `Tag { name, from, to, direction, repeat, color, user }` `sprite.rs:273`; duração **por quadro** `Frame.duration_ms` `:164`; tocador `play.rs` (medido no Trial) |
| Animação do motor | `SpriteAnimations` `sprite_anim.rs:243` (`insert` `:271`), `AnimationTag` com `per_frame_ms` `:179`, `SpriteAnimator { autoplay, start }` `:314`; relógio inteiro no passo fixo |
| As duas animações | doc ObjectsAndComponents §1/§123: intervalos = **conteúdo da imagem** (= `SpriteFrames` do Godot), Timeline = `AnimationPlayer`. ⇒ as tags do Pixel vão para o `SpriteAnimator`, não para `NamedClip` |
| Conversão de tag já existe | `shells/desktop/src/ase_import.rs:131` `tag_from_ase` (direção pelo construtor, `repeat 0 ⇒ None`, `per_frame_ms` só se não uniforme). **Segunda porta à espera** se o Pixel escrever outra |
| Slices | `Slice { id, name, user, keys }`, `SliceKey { frame, shape }`, `SliceShape { bounds, center, pivot }` `slice.rs:21-46`; `visible_at(frame)`. Nenhum leitor fora da família |
| Colisão | ADR-0199: forma = filha do corpo; `nasce_com_forma(world, body, shape)` `migra_formas.rs:83`; `ColliderShape { Ball, Cuboid, Capsule }` em **metros**, append-only; recusado: «sistema por quadro que muta o documento fora de um gesto» |
| Pivô do motor | `Sprite.anchor: [f32;2]` `ph2d-render/src/sprite/component.rs:115` (gravado) |
| Luz: quem entra | coleta `ph2d-light2d-ecs/src/coleta.rs:209-232`: `BloqueiaLuz` sem `Sprite`/`VecPathRef` ⇒ **outro**, desenhado pela família (`CamadaDeOclusao`, `ph2d-light2d-gpu/src/oclusao.rs:17`; Flip `pass_luz.rs`, Motion `motion_shape_luz.rs`) |
| Luz: o receptor | UMA função `luz_receber` em `ph2d-render/src/shaders/receber.wgsl:45`, concatenada no sprite E na colagem do vetor (`band_blit.wgsl`); `luz_em = F/2π`, isotrópico, **sem normal** (plano Luz2D §2.3) |
| Luz: o campo | a c0 grava só a **fluência escalar** `F` (`c2_rc.wgsl:436-439`, `soma·0.25`, `·TAU`); a C3 tem o centro de cada lâmpada no somatório (`c3_direta.wgsl:21`) |
| Normais no Pixel | `PixelSelector::NormalMap` `ph2d-panel-pixel-tools/src/seletor.rs:174-189`: `[128,128,255]`, `n·0.5+0.5`, **Y para cima**; só devolve cor, nenhuma camada sabe que é normal |
| Sprite Inspector | `.ase` `ase_import.rs:239` (tags sim, **slices não**); folha `sheet_import.rs:81` |
| Godot | `/usr/bin/godot` 4.7.2, **MIT** (`pacman -Qi`); precedentes corridos: `pixel-io/tests/folha.rs:445` `o_godot_recorta_igual`, `ferramentas/oraculo_godot_*.py` |
| Tecto `ph2d-app-pixel` | **11 239 de 11 278** (`the_shell_only_shrinks:222`): folga 39 ⇒ o código novo vive numa crate NOVA |
| Contrato §6 | grep `trait Tool|NodeOp|…|enum VectorOp|impl … for` nas 10 crates tocadas: **0** em todas (§4) |
| Passo 0 (a) — medido | a banda do Pixel é `BandSource::Vector` (`band_blit.rs:39`, `recebe_luz = true`): **toda** a banda é multiplicada por `acender → luz_em` (`band_blit.wgsl:68-84`), não por `luz_receber`; **sem** `NaoRecebeLuz` por objeto. O Pixel Sprite já recebe luz, plana e sem opt-out |
| Passo 0 (b) — medido | Play: `undo_jogo.rs:46-60` fotografa (`ProjectState::capture`); Stop: `restore` faz despawn + respawn do snapshot (`undo.rs:94-102`). Componente NÃO registado some no Stop e em todo undo. `tick_sprite_animations` (`ph2d-app-sprite-anim/src/tick.rs:23`) corre também no editor, só mexe em animadores a tocar |
| Passo 0 (c) — medido | ficheiro e undo gravam **só o registado**, sempre por `entry.serialize` (`save.rs:311`, `incremental.rs:456`, `snapshot.rs:290`); `GlobalTransform` é `PresentComponent` não registado. Registar não sobe `PROJECT_SCHEMA` por si (opcional, só gravado quando presente: `registry.rs:311`) |
| Passo 0 (d) — medido | 34 leitores de `SpriteAnimations` em 14 ficheiros de produção (6 crates) |

## 2. Estado da arte — o que os outros fazem, e o que abandonaram

⚠️ Conhecimento geral, **não verificado nesta pesquisa** salvo onde diz «medido»; o que decide é o oráculo corrido (§3).

| Quem | O que faz | Tentou e abandonou / onde para | Lição |
|---|---|---|---|
| **Godot 4** | `AnimatedSprite2D` + `SpriteFrames` (fps por animação × multiplicador por quadro); `CanvasTexture` com `normal_texture` (convenção OpenGL, Y para cima); `PointLight2D` com **altura** | o 3.x redesenhava cada item por luz (Luz2D §1); `SpriteFrames` não tem ms por quadro, só fps × múltiplo | a nossa folha → `.tres` tem de achar fps e múltiplos **exatos** (mdc das durações) |
| **Unity 2D** | Aseprite Importer: tags → `AnimationClip`; *Secondary Textures* (`_NormalMap`) com o MESMO recorte da folha; *Custom Physics Shape* por sprite | normal map de tela inteira chamado «very expensive» pela Unity; `PolygonCollider2D` **não** segue o quadro da animação (o artista anima à mão) | a normal mora AO LADO da cor com o mesmo recorte; o collider por quadro é pedido real que o motor não atende |
| **Spine** | *bounding box attachments* com chave na animação = hitbox por quadro | — | a hitbox animada pertence à **linha do tempo da arte**, não ao documento da física: é a nossa slice com keys |
| **Aseprite** | slices com pivô e 9-slice; nenhuma camada de normais, nenhum collider | — | a slice é o portador natural; a normal é camada nossa, gravada como user data no `.ase` |
| **Sprite Lamp / Laigter** | geram normais a partir da cor | geradas automaticamente deixam o pixel «mole»; o pixel artist pinta à mão com uma roda | a roda que já temos é o idioma certo; gerador fica fora |
| **Blender / AE / Illustrator / Rive** | não há sprite→cena de jogo com colisão e normal | — | sem lição aqui |

## 3. Oráculos (CLAUDE §0.9; licença por artefacto, 02_plano §3)

| Oráculo | Pergunta | Como |
|---|---|---|
| **Godot 4.7.2** (MIT) | a SEQUÊNCIA de quadros por tempo de cada tag (Forward/Reverse/PingPong, repeat, durações desiguais) | script `.gd` lê o NOSSO `.tres` e a nossa folha+JSON, avança o `AnimatedSprite2D` a passos de 1 ms, grava `(t, frame)` ⇒ fixture com cabeçalho; gate compara com o `SpriteAnimator` |
| Godot | o recorte de cada quadro e da `_n.png` | estende `oraculo_godot_remonta` (precedente `o_godot_recorta_igual`) |
| Godot | **sinal** da normal (Y para cima) | hemisfério normal + `PointLight2D` à esquerda ⇒ foto; o lado esquerdo é o claro. Só o SINAL e a ordem: a lei de luz dele (altura, N·L 3D) não é a nossa (§4.L) |
| `play.rs` (Trial, medido na P5) | o tocador do editor e o do jogo dizem o mesmo quadro | gate cruzado: o mesmo `t` nos dois |
| Gabarito `ph2d-light2d-ref` | a luz com normal na CPU, exata | §4.L |

## 4. O desenho — uma porta por pergunta

**Crate nova `ph2d-pixel-cena`** (drop-crate; a família compõe). Depende de `ph2d-pixel-core`, `ph2d-ecs`, `ph2d-physics-ecs`, `ph2d-light2d-gpu`; nunca o contrário.

| Pergunta | A porta ÚNICA | Quem lê |
|---|---|---|
| Que quadro este Pixel Sprite mostra agora? | `quadro_mostrado(world, e, doc) -> FrameId`: o do `SpriteAnimator` em Play, o `doc.frame` no editor | desenho, camada de luz, collider, pivô, cartão |
| Tag do Aseprite/Pixel → clip do motor | `ph2d_ecs::sprite_anim::tag_do_aseprite(nome, from, to, direção, repeat, durações) -> (AnimationTag, nota)`, **movida** de `ase_import.rs:131` | `ase_import` e `ph2d-pixel-cena` (censo: zero `AnimationTag {` construído fora dela nesses dois) |
| O que é derivado e não se grava? | `ComponentRegistry::derivado_quando::<C, Marca>()`: na entidade com `Marca`, o `serialize` de `C` devolve `None` (ficheiro, undo, foto do Play — as três passam por ele) | `(SpriteAnimations, PixelSprite)`, `(ColliderShape, SegueSlice)` |
| Quem reescreve o derivado? | o sistema da família `deriva(world, app)`: reescreve quando o `doc.stamp` muda **ou** o componente falta (undo e Stop fazem respawn sem ele) | — |
| Onde está a origem? | `pivo_px(doc, frame)`: o pivô da slice escolhida em `Sprite.pivot_slice` (doc), senão o centro | desenho, clique, collider, luz, JSON, `.tres` |
| Rect px → forma de colisão | `ph2d_physics_ecs::forma_de_retangulo(rect_px, pivo_px, m_por_px) -> (ColliderShape, offset_m)` | Pixel e import do Sprite Inspector |
| Que normal tem este texel? | a camada com `Layer.normal = true`; `compose_normal(sprite, frame)` (só essas camadas; a cor exclui-as) | desenho da banda normal, `_n.png`, `.ase` |
| Que luz recebe com esta normal? | `luz_receber(cor, mundo, n_xy, sem_luz)` em `receber.wgsl` (a MESMA, sprite e colagem) e `luz_em_n` no gabarito | todo receptor |

**A — derivado, nunca gravado (revisto pelo passo 0).** As tags são do documento Pixel (undo próprio, ADR-0196). Escrevê-las em `SpriteAnimations` GRAVADO era o defeito do ADR-0199: o undo do mundo veria um passo que ninguém fez, e duas verdades. A primeira escolha, um tipo `ClipsDerivados` lido por uma porta `biblioteca()`, obrigava a reescrever **34 leitores em 6 crates** de outras famílias (passo 0 d). ⇒ O componente continua a ser `SpriteAnimations` (o ECS conversa por componentes: os 34 leitores, o cartão, o Luau e o verbo `Play Clip` funcionam sem mudar), e o registo passa a saber que ele é **derivado quando a entidade tem `PixelSprite`**: o `serialize` dele devolve `None` ali, logo não entra no ficheiro, no diff do undo nem na foto do Play. O `SpriteAnimator` (Autoplay, tag inicial) é escolha do artista no OBJECTO ⇒ gravado, como em qualquer Animated Sprite.

**C — a forma por quadro.** «Add Collider» é um GESTO: `nasce_com_forma` + `SegueSlice { slice: SliceId }` (gravado). O `ColliderShape` da filha é **derivado quando ela tem `SegueSlice`** (a mesma regra do registo): a família escreve-o a cada quadro com a key em `quadro_mostrado`, e ele nunca entra no ficheiro nem no undo ⇒ não há passo que ninguém fez, e a recusa do ADR-0199 não se aplica. Ponte do rapier, gizmo e clique continuam a ler `ColliderShape`: zero leitores mudam. ⚠️ `ColliderShape` é append-only e **não** ganha variante. Slice escondida no quadro ⇒ a forma desliga (sensor mudo), não some da árvore. A conferir no P14b: a ponte do rapier refaz o collider quando o `ColliderShape` muda em execução.

**L — a lei da luz com normal.** Recusada a «Lambert com lâmpada a uma altura»: exige um parâmetro novo em cada luz e quebra o forno branco. Escolha: o núcleo de flatland **cardioide**, `k(ω, n) = (1 + n_xy·ω̂) / 2π`:
- `∫k dω = 1` para toda normal ⇒ o forno branco vale sempre (campo uniforme ⇒ a mesma luz);
- `n_xy = 0` (normal de frente, `[128,128,255]`) ⇒ é **exatamente** o `F/2π` de hoje;
- `|n_xy| ≤ 1` ⇒ `k ≥ 0`;
- `luz_em_n = F/2π + n_xy·M/2π`, com `M = ∫L(ω)ω̂ dω` (o primeiro momento, por canal). A c0 já percorre as direções: acumula `ω̂·L` ao lado de `L`; a C3 soma a direção de cada lâmpada; o rebote, idem.
- Custo: `M` são 2 texturas `Rgba16Float` à resolução da c0, **só alocadas se algum receptor com normal estiver à vista** (a guarda G6 continua byte a byte; contado em `quadro_leituras.rs`).
- A normal chega ao receptor: Sprite do motor ⇒ segunda textura com o mesmo recorte (como a Unity); o Pixel, desenhado na banda Vello ⇒ a família desenha `compose_normal` numa **banda normal** com o mesmo `Affine`, e a colagem lê-a. A banda só existe com normal à vista e luz ligada.
- O Pixel Sprite **bloqueia** como «outro»: implementa `CamadaDeOclusao` (alfa da cor em `quadro_mostrado`), entra em `com_oclusoes` (`present_luz.rs:46`).

**ADR novo** (número contado na integração): «o Pixel Sprite na cena: o documento é a verdade e o motor lê derivados não gravados (clips, forma por quadro); a normal de flatland é cardioide».

## 5. Formato e contratos (§0.11, §6)

- **§6:** zero. Prova: o grep da §1 (`trait Tool|NodeOp|OpResolver|NodeManifest|RasterEditTool|CanvasPaintTool|PanelEvent`, `enum VectorOp`, `impl … for`) nas crates `ph2d-ecs ph2d-physics-ecs ph2d-light2d-{ref,gpu,ecs} ph2d-app-light2d ph2d-pixel-{core,io} ph2d-app-pixel ph2d-render` dá 0. Repete-se no fecho sobre o diff.
- **Blob Pixel:** `Layer.normal: bool`, `Sprite.pivot_slice: Option<SliceId>` ⇒ `MINOR 6 → 7`, sem leitor de versão velha (§0.11).
- **Mundo:** `SegueSlice` gravado ⇒ entra no registo de componentes e na tabela `PORTAS` (Gravado). `derivado_quando` não muda bytes de projeto sem Pixel. Se algo exigir degrau, `PROJECT_SCHEMA` sobe e **conta-se** (`python3 scripts/schema-recount.py`), nunca se escolhe.
- **Sprite do motor com normal:** a segunda textura entra no `SavedAsset` (chave da célula + sufixo `_n`) ⇒ degrau contado.
- **`.ase`:** camada normal = user data da camada com a propriedade `ph2d.normal`; o pivô escolhido = propriedade do sprite. LibreSprite lê de volta (corrida), o Trial mostra (foto).

## 6. UI — as quatro condições por peça

| Peça | Existe | Pintada e registada | Clique chega ao barramento | A sequência leva a algum lugar |
|---|---|---|---|---|
| Layer Properties ▸ **Normal Map** (checkbox) | o diálogo existe (P12) | ícone na linha da camada | `PanelEvent` → op `SetLayerNormal` com inverso | pintar com a roda nessa camada ⇒ a cor do sprite não muda e a luz sombreia |
| Cartão **Pixel Sprite** ▸ **Pivot** (lista: *Center* + slices com pivô) | cartão `cartao.rs` | linha nova no cartão | op `SetPivotSlice` | o objecto gira/escala em torno do pivô e a origem no Play segue a key |
| Cartão **Animation** (Autoplay, *Start with*) | o do Animated Sprite (GEF) | aparece no Pixel Sprite por `biblioteca()` | o mesmo do GEF | Play ⇒ toca; `play("run")` no Luau ⇒ troca |
| Slices ▸ **Add Collider** | a janela SLICES (P11) | botão por slice | gesto ⇒ `nasce_com_forma` + `SegueSlice` | Play ⇒ o corpo cai e pousa pela slice; a forma muda no quadro da key |
| Add Component ▸ **Blocks Light** / **No Light** | cartões da Luz 2D | já registados | já ligados | o Pixel Sprite faz sombra e recebe |
| Export ▸ **Normal map sheet**, ▸ **Godot SpriteFrames (.tres)** | `ph2d-panel-pixel-export` | checkbox + formato | `event.rs:202` | o Godot abre o `.tres` (oráculo) |

Todas as strings em inglês, por chave i18n; tamanhos por token.

## 7. Gates (red-first: cada um visto VERMELHO antes da cura) e a fixture

**Fixture `heroi_p14`** (contém o fenómeno): 16×16, 6 quadros com durações `80 80 120 120 60 200`; tags `idle` 0–1 Forward, `run` 2–4 PingPong repeat 2, `hit` 5 Reverse; slice `body` com key nova no quadro 3 (o retângulo encolhe) e pivô nos pés que muda no quadro 4; camada Normal Map = hemisfério pintado pela roda.

| Gate | O que prende |
|---|---|
| `a_tag_do_pixel_e_a_do_aseprite_pela_mesma_porta` | `.ase` importado e Pixel Sprite dão `AnimationTag` iguais campo a campo |
| `o_jogo_e_o_editor_tocam_o_mesmo_quadro` | `SpriteAnimator` × `play.rs` em `t = 0..2000` ms, passo 1 |
| `a_sequencia_e_a_do_godot` | fixture do Godot (§3) × o nosso, por passo |
| `os_clips_derivados_nao_entram_no_undo_nem_no_ficheiro` | editar uma tag ⇒ zero passos no undo do mundo; o snapshot do Pixel Sprite não leva `SpriteAnimations`; um Sprite comum continua a levá-lo |
| `os_clips_voltam_depois_do_undo_e_do_stop` | undo e Stop (respawn) ⇒ no quadro seguinte o `SpriteAnimations` está lá, igual às tags |
| `a_forma_segue_a_key_da_slice` | quadro 2 → 3 ⇒ o cuboide do rapier encolhe; o gizmo mostra o mesmo |
| `o_collider_nasce_por_gesto_e_desfaz_num_passo` | Add Collider ⇒ 1 passo; Ctrl+Z ⇒ a filha some |
| `o_pivo_e_a_origem_em_todo_leitor` | desenho, clique, collider e JSON com o mesmo pivô por quadro |
| `forno_branco_vale_para_toda_normal` (gabarito) | campo uniforme ⇒ a mesma luz para 16 normais |
| `a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte` | `n_xy = 0` ⇒ quadro idêntico; G6 continua verde |
| `a_lampada_a_direita_acende_a_normal_a_direita` | ponto em +x: `n = (1,0)` ⇒ 2× o plano; `n = (−1,0)` ⇒ 0 (sem ambiente) |
| `a_placa_e_o_gabarito_concordam_com_normal` | GPU × `luz_em_n` dentro da tolerância da Luz2D |
| `o_sinal_da_normal_e_o_do_godot` | foto do Godot: lado claro = o nosso |
| `o_pixel_sprite_faz_sombra` | `BloqueiaLuz` ⇒ entra em `outros`, sombra medida atrás dele |
| `a_folha_normal_tem_o_recorte_da_cor` | `_n.png` × folha de cor, quadro a quadro |
| `o_tres_abre_no_godot` | Godot carrega o `.tres` sem erro e lista as 3 animações |
| `o_inspector_abre_o_que_o_pixel_grava` | grava `.ase` e folha ⇒ import ⇒ tags, pivô, colliders e normal iguais ao Pixel Sprite |
| censo | zero `AnimationTag {` construído fora da porta em `ase_import` e `ph2d-pixel-cena` |

Mutação (fecho): tirar a guarda `n_xy = 0`, trocar o sinal de `ω̂`, ler `doc.frame` em vez de `quadro_mostrado`, tirar o `derivado_quando` ⇒ cada uma sangra num gate acima.

## 8. Cenas de smoke (números MEDIDOS pela sonda headless antes da mensagem)

- **Cena 26 «O herói na cena»:** `heroi_p14` como Pixel Sprite sobre um chão (corpo estático), uma Luz pontual à esquerda e uma parede que bloqueia. Play ⇒ toca `idle`, cai, pousa pela slice `body`; a tecla troca para `run`; o lado esquerdo do herói está claro e o direito escuro; ele faz sombra na parede. A sonda mede, e a mensagem cita: o quadro em `t = 500/1000/1500 ms`, a altura onde pousa (px), a razão de luz esquerda/direita, a área de sombra. Fotografado antes de mandar.
- **Cena 27 «O mesmo herói pelo Sprite Inspector»:** o `.ase` que a cena 26 grava, importado como Sprite do motor ao lado do Pixel Sprite: os dois tocam, pousam e iluminam igual. Mede-se a diferença por píxel entre os dois (alvo: zero).

## 9. Ordem (série só onde o resultado muda o seguinte)

1. **Passo 0** — medir os três ❓ da §1 (uma rodada, um agente). Muda o desenho de A e L se o molde do derivado não existir.
2. **P14a ∥ P14c-gabarito** — animação (A, porta da tag, `biblioteca`, `quadro_mostrado`) e a lei cardioide na CPU (`luz_em_n`, forno branco): crates distintas.
3. **P14b** — pivô e colliders (depende de `quadro_mostrado`).
4. **P14c-placa** — momento `M`, banda normal, receptor, oclusão do Pixel.
5. **P14d ∥ P14e** — saída (`_n.png`, `.tres`, oráculos Godot) e import no Sprite Inspector.
6. Sonda, fotos, cenas 26–27, smoke do dono; `/pd-onda-fechar`.

## 10. ⛔ Recusas (desta onda)

| Recusado | Porquê |
|---|---|
| Tags → `NamedClip` da Timeline | a Timeline é o `AnimationPlayer`; quadros são conteúdo da imagem (ObjectsAndComponents §123) |
| Escrever as tags em `SpriteAnimations` gravado | duas verdades + passo de undo que ninguém fez (ADR-0199) |
| Tipo `ClipsDerivados` + porta `biblioteca()` | 34 leitores em 6 crates a reescrever (passo 0 d); a regra no registo dá o mesmo sem tocar nenhum |
| `FormaDerivada` + porta `forma_efetiva()` | idem: a mesma regra no registo cobre o `ColliderShape` |
| `ColliderShape::PixelSlice` | o enum da física não pode conhecer o Pixel; append-only seria uma dependência invertida |
| Normal com «altura da lâmpada» (Godot/Unity) | parâmetro novo em toda luz; quebra o forno branco e a identidade byte a byte do plano |
| Normal map de tela inteira sempre ligado (Unity) | custo em toda cena; a nossa banda só existe com normal à vista |
| Gerar normais da cor (Sprite Lamp) | pixel «mole»; o idioma é pintar com a roda |

## 11. Estado (09/11, fim da janela 1 — a próxima janela começa aqui)

Commits da onda (`git log --oneline 1eb5eff0a4^..HEAD`), cada um com o relatório na mensagem. Nada integrado.

| Parte | Estado | Commits |
|---|---|---|
| Gabarito CPU da luz com normal (`luz_em_n`, `momento`, miolo de 9 componentes, rebote) | ✅ gates verdes, vistos vermelhos | `d69428bd90` |
| P14a — animação: `derivado_quando` (o `grava` sai, o `serialize` fica), porta `tag_do_aseprite`, `ph2d-pixel-cena` (`clips_das_tags`, `quadro_mostrado`, `deriva`), ping-pong do Aseprite (N passadas ⇒ `ceil(N/2)` ciclos + `acaba_na_ponta_de_chegada`), `current` vazio ⇒ 1.º clip, cartão Animation (só Autoplay e Starts with; editores escondidos) | ✅ | `dedc76163c` `7dc42e2776` `bb340489fa` `96bfbc00ce` `270fb31a1d` `8671c7588f` |
| Censos do Inspector (o × de f7688dbaad é moldura: 18→19, 22→23) | ✅ | `a3c2fcbf9e` |
| P14b — pivô (`pivot_slice`, `Op::PivotSlice`, `MINOR 7`), `forma_de_retangulo`, `SegueSlice`, `campos_derivados_quando` (forma e offset do `Collider` não se gravam), gesto Add Collider, origem = pivô em todo leitor, forma segue a slice e slice escondida emudece | 🔶 em curso (agente P14b-2; relatório em §11 «P14b-2») | `da7fa11154` `f651bee09d` `d5072adb7c` … |
| P14c-placa — receptor `luz_receber_n` + banda normal (1); shaders de M `c3_momento.wgsl` (2); ligar M no Rust + gates placa×gabarito (3) | 🔶 (1)(2) feitos; (3) em curso (relatório em §11 «P14c-placa 3») | `790581ebf9` `980da1d295` … |
| `PROJECT_SCHEMA` | 225 → 227 (animação 226, `SegueSlice` 227), contado pela escada; o integrador reconta | — |

**Ainda por fazer (ordem):**
1. Fechar P14b-2 e P14c-placa 3 pelos relatórios abaixo.
2. **P14c-pixel:** camada `Layer.normal` (op + `MINOR 8`), `compose_normal`, a família desenha a BANDA NORMAL com o mesmo `Affine` (⚠️ rodar a normal pela base da instância — a banda assume eixo do mundo), liga `CenaDeLuz.com_normal`; Pixel Sprite **bloqueia** como «outro» (`CamadaDeOclusao`, `present_luz.rs:46`); `NaoRecebeLuz` por objeto na colagem (hoje a banda inteira recebe); Layer Properties ▸ Normal Map (4 condições).
3. **Sprite do motor com normal** (2.ª textura em `sprite.wgsl:376,382`) — necessário para a P14e.
4. **P14d:** `_n.png` na folha; export Godot `.tres` (fps = 1000/mdc das durações, múltiplos exatos; ping-pong desenrolado); oráculos Godot (sequência, recorte, sinal da normal) como fixtures com cabeçalho.
5. **P14e:** Sprite Inspector importa slices (pivô → `Sprite.anchor`, colliders) e a normal do `.ase`/folha que o Pixel grava.
6. Fixture `heroi_p14` completa (falta a camada Normal Map), sonda headless, cenas 26–27 fotografadas, smoke do dono, `/pd-onda-fechar`.

**Lições desta janela:** (a) o `serialize` do registo tem 13 leitores que NÃO gravam (Inspector, cópias, instâncias, Simular) — o derivado sai só do `grava`; (b) `ColliderShape` é campo do `Collider`, não componente — `campos_derivados_quando`; (c) um commit de UI (o × de f7688dbaad) que não corre os censos de moldura por nome deixa-os vermelhos para a onda seguinte.
