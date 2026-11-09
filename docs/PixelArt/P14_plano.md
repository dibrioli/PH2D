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

### P14b-2 (09/11, agente) — ✅ entregue

Commits `f651bee09d` (forma no app + slice escondida) · `d5072adb7c` (origem = pivô) · `17adbb0ed7` (Add Collider) · o commit deste bloco (Pivot do cartão).
- **Forma no app:** `ph2d_app_pixel::deriva(app, world, hero)` (`crates/ph2d-app-pixel/src/lib.rs:80`) chama `deriva_formas` com o ppm do projecto, nas duas costuras (`fase_frame_open.rs`, `project_load.rs`).
- **Slice escondida:** `ph2d_physics_ecs::FormaDesligada` (`components.rs:198`, derivado, fora do registo), posto/tirado por `deriva_formas` pela key vazia; a ponte (`bridge/bodies.rs:234`) chama `PhysicsWorld::set_colliders_muted` (`ph2d-physics/src/world/body_build.rs:129`): grupos de colisão vazios, MASSA mantida. ⛔ `set_enabled(false)` medido e recusado: tira a massa, o corpo nem cai (gate parou em y=3).
- **Origem = pivô:** porta única `ph2d_pixel_render::image_to_world(pivo, …)` (`view.rs:304`); a única leitura do pivô é `input::origem_px` (`input.rs:83`, `pivo_px` no `quadro_mostrado`); desenho, clique, onion, Tiled, chrome, miniatura e `pick_at` passam por ela; gizmo de objecto por `draw::caixa_local` (`draw.rs:55`, centro relativo ao pivô).
- **Add Collider** (SLICES): `ids::ADD_COLLIDER`, `Pedido::AddCollider`, `Contexto.collider` («Collider added» desligado); `slices::adiciona_collider` (`slices.rs:268`) no `frame_in` = uma mutação do mundo num quadro (o diff do `post_frame_undo`); `collider_da_slice` (`forma.rs:97`) impede duplicar.
- **Pivot** (cartão Pixel Sprite): `PixelSpriteCard.{doc,pivots,pivot}`, `PixelRequest::PivotSlice`, `cards_pixel_pivot.rs` (lista Center + slices com pivô) ⇒ `Op::PivotSlice` (um passo do Pixel, `panel.rs` `aplica`).

Gates (todos vistos VERMELHOS por mutação, depois verdes): `ph2d-pixel-cena corpo::o_corpo_pousa_pela_slice` (física real; pivô fixo no centro ⇒ vermelho) · `corpo::a_slice_escondida_nao_colide` · `ph2d-host-pixel-scenes pivo::o_pivo_e_a_origem_em_todo_leitor` (centro fixo na porta ⇒ vermelho) · `pivo::o_centro_fixo_so_mora_na_porta` (censo) · `pivo::o_pivot_do_cartao_muda_a_origem_num_passo` · `slices_cena::o_collider_nasce_por_gesto` · `ph2d-panel-registry-init o_add_collider_chega_a_familia_e_nao_duplica` · `ph2d-panel-inspector o_pivot_do_cartao_pede_a_slice_ao_documento`. Regressão por nome verde: `edit::`, `slices`, `cartao::`, `o_cartao_do_pixel_sprite::`, `the_pixel_slices_answers_a_real_pointer::`, physics-ecs `hot_path`/`deterministic`/`forma_filha`.

**Falta:** (a) o gate `o_collider_nasce_por_gesto_e_desfaz_num_passo` com o undo REAL da shell (Ctrl+Z tira a filha num passo) — hoje provado só «uma mutação num quadro»; (b) a camada de luz ainda não lê o Pixel Sprite (entra na P14c-pixel pela mesma `origem_px`); (c) o Pivot do cartão só na saída em cartões (`PH2D_CARDS=0` não tem a linha); (d) o JSON/`.tres` com o pivô (P14d); (e) `a_forma_segue_a_key_da_slice` contra o cuboide do rapier (a key do quadro 3 tem o mesmo fundo; a altura de repouso não a distingue — o gate de pouso usa o centro como contraste).

### P14c-placa 3 — o momento `M` ligado na placa (09/11)

- **Entrou:** `crates/ph2d-light2d-gpu/src/solver_momento.rs` (novo): `Momento::novo` (c0 de `M` = `cascata_momento` sobre `rc_grupos[0]`, nível 0; fina de `M` = `c3_momento` com g2 {0,1,2,5,30–33} e as constantes da fina + `MOMENTO`/`MOMENTO_EXATA`), `gravar` (depois da C3, `solver_gravar.rs:406`), `Solver::{momento, momento_texturas, bytes_do_momento}` e `acertar_o_momento` (`solver_gravar.rs:97`: aloca só com `CenaDeLuz::com_normal`, solta sem ela). `modulo_rc` partilhado (o `solver.rs` desceu para < 700). `c2::constantes` (`solver_c2.rs`). `C3.{cena, cena_layout, uniforme}` `pub(super)`.
- `CenaDeLuz::com_normal` (append-only, `ph2d-light2d-ecs/src/cena.rs`); a coleta põe `false` (o Pixel Sprite liga-o na P14c-sprite). `quadro.rs` `recebida()` passa `momento: r.solver.momento()`.
- ⛔ **Defeito da `980da1d295` curado:** o `c0_momento_em` dentro do `luz_do_texel` punha as ligações 30–31 em TODA pipeline que o chama — a `luz2d c3 fina` de hoje não nascia (`[gpu-erro] binding 30 missing`, F = lixo, rmse 1,22; o `g5_` não o via). Agora o `luz_do_texel` só guarda os pontos das duas faces (`momento_faces`, `momento_f2`) e a `c3_momento` soma a c0 de `M` neles (linear: o mesmo).
- **Gates** (`ph2d-light2d-gpu/tests/it/momento_na_placa.rs`, RADV): `a_placa_e_o_gabarito_concordam_com_normal` — rmse relativo `n=0` 0,0151 · `(1,0)` 0,0336 · `(0,1)` 0,0164 · `(−0,6,−0,8)` 0,0225, erro máximo absoluto 0,1011 (teto 0,045); `a_lampada_a_direita_acende_a_normal_a_direita_na_placa` — claro 0,21215 · plano 0,10608 · escuro 0,00000 (razão 2,0000); `sem_normal_nao_aloca_o_momento` — 0 bytes sem, 524 288 com, F bit a bit igual. Visto vermelho (`ω̂ → −ω̂`): `(1,0)` 1,167 e razão 0. Verdes por nome: `g5_` (5), `g6_` (2), `g7_` + `a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte` (4).
- **Falta:** (a) contar o `bytes_do_momento` em `relatar_memoria`/`relatar_custo` (`quadro_leituras.rs`) e declará-lo na `PassList`; (b) `M` na holográfica, no rebote e no miolo (hoje `M` = só c0 + lâmpadas; com a holográfica ou `so_a_c0` o momento não nasce); (c) a mistura de 4 texels perto de opacos e a `c3_fina_cantos` sem `M`; (d) a c0 de `M` sem a máscara `fora`; (e) a fina de `M` corre a grade inteira (não só a região).

### P14c-placa 4 (09/11, agente) — (a) e (e) feitos; (c, d) medidos e recusados; (b) aberto

Commit `04024854e3`.
- **Entrou:** `PassList::com_momento(cfg)` (`crates/ph2d-light2d-gpu/src/passlist_momento.rs:22`, módulo novo em `lib.rs` — o `passlist.rs` está em 699 LOC): as quatro texturas `Rgba16Float` no FIM da lista (c0 de `M` na grade das cascatas, fina de `M` na da fluência) e os passes `luz2d momento c0` (lê o nível 1) e `luz2d momento fina` (lê c0 de `F`, visibilidade, preenchimento, c0 de `M`). O `Momento` cria as texturas DA declaração (`solver_momento.rs:81`) e guarda a lista; `bytes_do_momento` = soma declarada (`:277`); `Solver::pass_list()` mudou para `solver_momento.rs:259` e devolve a lista com o momento enquanto há normal à vista (sem ela, a de `PassList::de`: G6 intacta); `recursos_da_lista` encadeia as texturas do momento (`solver.rs:614`), logo o `relatar_memoria` mede-as; o `relatar_custo` imprime os bytes do momento da vista e do entorno na linha `onde` (`ph2d-app-light2d/src/quadro_leituras.rs:135`).
- **Gate** `g5_o_momento_e_o_que_a_pass_list_declara` (`ph2d-light2d-gpu/tests/it/g5_bytes_declarados.rs:348`, RADV): sem normal, nada do momento na lista e 0 bytes; com normal, as 4 texturas alocadas UMA vez cada pelo alocador do wgpu, nenhuma alocação fora da lista, alocado ∈ [declarado, 2·declarado + 64 KiB], `bytes_do_momento` = soma declarada, `bytes_alocados` = base + momento, os dois passes na lista; sem normal outra vez, a lista volta igual. **Visto vermelho** com `pass_list()` a ignorar o momento (`left: 0, right: 4`), depois verde.
- **Regressão por nome (verde):** `g5_` (3) + `momento_na_placa` (3) · `ph2d-app-light2d g6_` (2) · `ph2d-render g7_` + `a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte` (4).
- **2.º commit `488628eb09`:** (e) a fina de `M` despacha só a região (`C3::grupos_da_fina`, `solver_c3_uniformes.rs`; o `c3_momento` já deslocava por `fina.inicio`, logo a saída é a mesma, só com menos grupos: nenhum gate de saída a vê, e a regressão fica verde). Gate novo `junto_ao_opaco_o_momento_e_o_do_gabarito` (`momento_na_placa.rs`: só céu, os texels de fora a ≤ 4 texels de uma parede opaca): rmse `n=0` 0,0489 · `(1,0)` 0,0616 · `(−1,0)` 0,0877 · `(0,1)` 0,0554 · `(0,−1)` 0,0629, teto 0,10; **visto vermelho** sem a c0 de `Mx` (0,4123 · 0,5276).
- ⛔ **Medido e recusado (c, d):** `M` lido pelas MESMAS sondas e pesos da c0 de `F` (`c0_de(fonte, P)` com o alfa da `c0_fluencia` + `peso_de_fora`, e `c0_ao_longo_de` na face remapeada). Fora da parede dá o MESMO resultado que o bilinear cru, até à 4.ª casa, nas cinco normais. Dentro dela (3 texels) fica pior em ±x (0,46/0,52 contra 0,30/0,40) e melhor em ±y (0,15/0,13 contra 0,21/0,16). Revertido: a árvore tem o bilinear cru. O receptor DENTRO do corpo (onde o Pixel Sprite que bloqueia vai estar) precisa de outra lei para `M`, a desenhar contra o `luz_em_n` do gabarito no ponto remapeado.
- **Regressão por nome (verde):** `momento_na_placa` (4), `g5_` (3), `g6_` (2), `g7_` + `a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte` (4).
- **Falta (por ordem):** (b) `M` na holográfica e no entorno `so_a_c0`: a `hrc_soma` só acumula `R0` do cone 0 por quadrante, e os `R` de um quadrante são reescritos pelo seguinte. ⇒ é preciso uma `hrc_soma_momento` PRÓPRIA no laço dos quadrantes (`R0·ω̂_q·sin h/h`, com h = π/4) e uma `hrc_saida_momento` (o borrão e a c0 do entorno também para `M`). Pipelines próprias, para não pôr ligações nas de hoje (o defeito da `980da1d295`). O `so_a_c0` só existe com a holográfica: entra com ela. Depois o rebote e o miolo, e estender `a_placa_e_o_gabarito_concordam_com_normal` a esses modos. (c/d) o `M` do receptor dentro de um opaco (ver a recusa acima) e a `c3_fina_cantos` sem `M`. ⚠️ A `com_momento` declara o que a fina lê pela visibilidade de `Opcionais` sem o filtro `com_visibilidade` da C3 (só a contagem de bytes movidos o sente).

### P14b-3 (09/11, agente)

Fecha as alíneas (a) e (e) do «Falta» do P14b-2. Commit deste bloco.
- **(a) Undo REAL:** `shells/desktop/src/undo_pixel_collider_tests.rs:37` (`mod pixel_collider_tests`, `undo.rs:594`; dev-dep `ph2d-pixel-cena` na shell). Sem `gfx` o `App::capture_project` dá `None` (`headless_app`), logo o gate conduz as peças do `post_frame_undo` uma a uma: `ProjectState::capture` com `init::build_component_registry` (o registo do produto), a regra do auto-commit (passo ⇔ estado ≠ baseline), a fila `ProjectUndo` e o `ProjectState::restore` do Ctrl+Z/Y. O gesto é a mutação do `slices::adiciona_collider` (`adiciona_collider_da_slice`); o percurso pedido → família continua em `slices_cena::o_collider_nasce_por_gesto`.
- **(e) Rapier:** `PhysicsBridge::body_aabb(entity)` (`ph2d-physics-ecs/src/bridge/inspect.rs:70`, append-only, `#[doc(hidden)]`): a caixa do solver, não o `Collider`. Gate `ph2d-pixel-cena corpo::a_forma_segue_a_key_da_slice_no_rapier` (`tests/it/corpo.rs:158`): a key do quadro 3 trocada no teste por `BODY_3_ALTA` 4×8 em (4, 0), pés (8, 15) — meia-extensão (0,125, 0,25) contra (0,25, 0,4375), offset (−0,125, 0,6875) contra (0, 0,375), fundo 8 px acima. Com o Play a correr (animador, editor fixo no 0), 10 passos no quadro 2, troca para o 3, 1 passo: a caixa do rapier (centro − pose, meia-extensão; `LockRotation`) e a do gizmo (`Collider::local_aabb`) = as esperadas a 1e-4; mesmo número de corpos na arena (troca no lugar); o corpo ainda no ar; de volta ao 2, volta.
- **A ponte refaz o collider em execução — conferido, sem cura:** a meio do Play `reconcile_structure` vê `sem_pose(rest) ≠ sem_pose(desc)` e chama `PhysicsWorld::retune_body` (`ph2d-physics/src/world/body_build.rs:164`), que faz `set_shape` do colisor no lugar; em repouso, respawn. O §4 C deixa de estar «a conferir».

Gates (vistos VERMELHOS, depois verdes): `ph2d-host-desktop o_collider_nasce_por_gesto_e_desfaz_num_passo` — `restore` sem o despawn ⇒ vermelho na linha 83 («o Ctrl+Z deixou a filha»); `campos_derivados_quando` tirado do `registar` ⇒ vermelho na 68 (a forma a seguir o quadro fez um 2.º passo) · `corpo::a_forma_segue_a_key_da_slice_no_rapier` — `deriva_formas` com o quadro do editor em vez do `quadro_mostrado` ⇒ vermelho (e `forma::a_forma_segue_a_key_da_slice` também); `retune_body` sem o `set_shape` ⇒ vermelho na caixa do quadro 3. Regressão por nome verde: `ph2d-pixel-cena corpo::` (3) e `forma::` (7), `ph2d-physics-ecs forma_filha` (5).

**Falta (do P14b-2):** (b) a camada de luz a ler o Pixel Sprite (P14c-pixel); (c) o Pivot do cartão só na saída em cartões; (d) o JSON/`.tres` com o pivô (P14d). Nenhuma de (a)/(e).

### P14c-pixel 1 (09/11, agente)

Só o lado do documento Pixel e a UI (a banda normal, a oclusão e o `NaoRecebeLuz` ficam para a parte 2). Commit deste bloco.
- **Documento:** `Layer.normal` (`crates/ph2d-pixel-core/src/sprite.rs`), bit 7 das bandeiras da camada no blob, **`MINOR 7 → 8`** (`codec.rs:38`, sem leitor de versão velha). `Op::LayerNormal { layer, normal }` (= `SetLayerNormal`, `op.rs:50`; `apply_normal` em `op/camadas.rs`, que também recebeu o `check_layers` para o `op.rs` voltar a < 700 LOC; inverso = o valor de antes; grupo recusado) e o construtor `layers::set_normal` (`layers.rs:179`, `None` sem mudança); nome do passo `pixel.history.layer_normal` («Normal Map»).
- **Composição:** `compose_normal(sprite, frame)` (`compose.rs:24`) + `NORMAL_FLAT = [128,128,255]` (`:18`); `Purpose::Normal` no `flatten` (só camadas `normal`; a cor — View/Export, `compose_picked`, o conta-gotas indexado — exclui-as). **Alfa:** RGB = as camadas Normal Map compostas sobre a de frente (cobertura parcial mistura com ela); **A = o alfa da COR composta (Export)**. Porquê: a colagem desenha a banda normal com o mesmo `Affine` sobre a dos outros objectos — com A = 255 em todo o retângulo o sprite apagaria a normal dos vizinhos fora da arte; com o alfa da cor a banda cobre exactamente onde a luz vai receber.
- **`.ase`:** `user::saida_camada` / `user::tira_normal` (`crates/ph2d-pixel-io/src/ase/user.rs:205,221`): `ph2d.normal = Bool(true)` no mapa do artista (chave 0) da camada; ao ler vira a bandeira e sai do user data.
- **UI:** `PixelLayerRow.normal`, `PixelRequest::LayerNormal(u32)` (alterna), `PixelLayerWidget::Normal` (ícone `Light`, pintado só na camada Normal Map, `pixel_layer_row.rs:173`, nos dois painéis); Layer Properties ▸ caixa **Normal Map** (`ids::LAYER_NORMAL`, `paint_camadas.rs:145`, em cartões e na saída `PH2D_CARDS=0` pelo mesmo `caixas`); `Toggled` → `pedido_normal` (`seam.rs:198,471`); a família aplica-o como um passo (`ph2d-app-pixel/src/camadas.rs:100`). Strings `panel.pixel.layer.normal`/`normal_tip` (inglês).

Gates (todos vistos VERMELHOS por mutação, árvore restaurada, depois verdes): `ph2d-pixel-core normal_map::pintar_na_camada_normal_nao_muda_a_cor_composta` e `::compose_normal_devolve_so_as_camadas_normal` (o `flatten` sem o filtro `normal` ⇒ os dois vermelhos) · `::set_layer_normal_desfaz_num_passo` (inverso devolve o valor novo ⇒ vermelho) · `::o_blob_leva_a_bandeira_normal` (sem ler o bit 7 ⇒ vermelho) · `ph2d-pixel-io a_camada_normal_map_vai_e_volta_pelo_ase` (sem `tira_normal` ⇒ vermelho; sem `saida_camada` ⇒ vermelho) · `ph2d-panel-registry-init the_pixel_layer_normal_map_answers_a_real_pointer` (caixa não pintada ⇒ vermelho; `Toggled` → outro pedido ⇒ vermelho; ícone não pintado ⇒ vermelho) · `ph2d-host-pixel-scenes camadas::normal_map_from_the_panel_leaves_the_color_in_one_undo_step` (a família não reconhece o pedido ⇒ vermelho). Regressão por nome verde: pixel-core `layer_laws codec user_data tiles history compose` (50), pixel-io `user_data ase` (11), pixel-tools `every_word os_cartoes seam_pixel` (65), pixel-timeline `os_cartoes seam` (28), registry-init `the_pixel paineis_armados` (56), pixel-scenes `camadas` (9), editor-core `node_id_collisions` (14).

**Falta:** (a) a banda normal desenhada pela família com o mesmo `Affine` (rodar a normal pela base da instância), `CenaDeLuz.com_normal`, `CamadaDeOclusao` do Pixel Sprite, `NaoRecebeLuz` por objecto — a parte 2; (b) a 4.ª condição completa («a luz sombreia») só com (a); (c) Merge Down/Flatten de uma camada Normal Map numa de cor deita-a fora da cor (o `Export` exclui-a) — decidir se o menu recusa; (d) em Indexed a normal passa pela paleta (sem sentido): recusar `SetLayerNormal` fora de RGBA ou avisar; (e) a camada Normal Map na fixture `heroi_p14`; (f) `_n.png` pelo `compose_normal` (P14d).

### P14c-pixel 1b (09/11, agente)

Fecha (c), (d), (e) do «Falta» do P14c-pixel 1. Commit deste bloco.
- **(c) Merge Down / Flatten.** `merge_down` recusa (`None`) normal × cor nos dois sentidos (`crates/ph2d-pixel-core/src/layers.rs:287`); duas Normal Map fundem-se e a de baixo continua Normal Map. Predicado do menu `layers::merge_down_mixes_normal` (`layers.rs:257`) → `PixelPanelInfo.merge_down_mixes_normal` (`ph2d-editor-core/src/pixel_panel/mod.rs:462`, preenchido em `ph2d-app-pixel/src/panel.rs:306`). `flatten` compõe a cor (`Purpose::Export`) numa camada e, havendo Normal Map entre as fundidas, as normais (`Purpose::Normal`) numa «Flattened Normal Map» logo acima (`layers.rs:381`); sem normal, uma camada só como antes.
- **(d) Indexed — MEDIDO: a camada NÃO pode guardar RGB.** `Cel.data` tem `bbox.area() * bpp` bytes com `bpp = sprite.mode().bpp()` (`sprite.rs:181`, `ColorMode::bpp` `sprite.rs:27`): o modo é do SPRITE, não da camada, e todo escritor de pixel passa por `Sprite::value_of_rgba` (ink, edit, filter, gradient, pattern, canvas, `cel_from_rgba`), como o `.ase` (profundidade de cor por ficheiro). Guardar RGB numa camada exigiria `bpp` por camada em todos esses leitores — torcer o modelo. Logo: `SetLayerNormal(true)` fora de RGBA é `OpError::BadLayers("normal_needs_rgb")` (`op/camadas.rs:53`; desligar continua livre, o inverso do passo); `convert::normal_needs_rgb` (`convert.rs:82`) e `convert` devolve `None` (`convert.rs:90`); o OK do Color Mode avisa (erro) com a chave `panel.pixel.layer.normal_needs_rgb` e não dá passo (`ph2d-app-pixel/src/paleta.rs:501`).
- **UI desligada e dita.** `seam::desliga_o_que_a_normal_recusa` (`ph2d-panel-pixel-tools/src/seam.rs:174`, por quadro em `column.rs:90`): Merge Down `ButtonState::Disabled` + dica `panel.pixel.layer.merge_normal_tip` («Can't merge a Normal Map layer with a color layer»); caixa Normal Map `CheckboxState::Disabled` fora de RGB + dica `panel.pixel.layer.normal_needs_rgb` («Normal maps need RGB color mode»). O dispatch já recusa o Down num desligado (`is_focusable`); o rect fica registado para a dica chegar ao ponteiro. **O botão de cartão passou a pintar o `Disabled` do store** (`ph2d-editor-core/src/panel/cards_linhas.rs:577`, como o `ButtonOff`, mas com rect) — antes um desligado em cartão pintava-se aceso.
- **(e) Fixture `heroi_p14`** (`crates/ph2d-pixel-cena/src/fixture.rs:90`): disco de cor `COR` raio 7,5 centrado em (8,8) na camada 0 e, por cima, a camada «Normal Map» com `hemisferio` (`:75`, `n·0,5+0,5`, y para cima — a codificação da roda), mesmo recorte, nos 6 quadros.

Gates (todos vistos VERMELHOS por mutação, árvore restaurada, depois verdes): `ph2d-pixel-core normal_map_fusao::merge_down_entre_normal_e_cor_recusa_nos_dois_sentidos` (sem a guarda ⇒ vermelho; `merge_down_mixes_normal` sempre falso ⇒ vermelho) · `::merge_down_de_duas_normal_funde_e_fica_normal` · `::flatten_achata_a_cor_numa_e_a_normal_noutra` (sem a camada normal ⇒ vermelho) · `::a_normal_so_existe_num_sprite_rgb` (sem a guarda do `apply_normal` ⇒ vermelho; `normal_needs_rgb` falso ⇒ vermelho) · `ph2d-panel-registry-init the_pixel_normal_map_refusals_answer_a_real_pointer` (Merge Down não desligado ⇒ vermelho; caixa não desligada ⇒ vermelho; ponteiro real: hot = o id, dica = a chave, clique sem pedido; religados, o clique chega) · `ph2d-pixel-cena normal_da_fixture::a_normal_da_fixture_e_um_hemisferio_no_recorte_da_cor` (centro ≈ [128,128,255] ±12, borda direita x > 192, esquerda < 64, topo y > 192, A = alfa da cor, cor = `COR`, 6 quadros; sinal de `nx` trocado ⇒ vermelho; sem a bandeira normal ⇒ vermelho).
- **Vermelho herdado curado:** `ph2d-pixel-cena forma::o_pixel_formato_7_le_o_que_grava` estava vermelho desde `4baae726d9` (MINOR 7 → 8); passa a `o_pixel_formato_le_o_que_grava`, lê `codec::MINOR` e exige a Normal Map da fixture (§0.11).
- **Regressão por nome (verde):** pixel-core `normal_map` (8) `layer_` (18) `codec` (8) `history` (7) `color_` (19) · pixel-cena `corpo` `forma` `derivado` `quadro_mostrado` `porta_da_tag` `censo_da_porta` (20) · app-sprite-anim `pixel` (6) · registry-init `the_pixel` (57) · pixel-tools `every_word` `os_cartoes` `seam_pixel` (65) · pixel-timeline `os_cartoes` `seam` (28) · editor-core `cards` (58) `node_id_collisions` (14) · host-pixel-scenes `camadas` (9).

**Falta:** (a), (b), (f) do P14c-pixel 1 (a parte 2 e o `_n.png`); o aviso do Color Mode recusado (`paleta.rs:501`) não tem gate próprio (a lei mora no `convert`, gateada); o pintor do botão de cartão desligado não tem gate de pixel (só o estado do store); um `.ase` Indexed com `ph2d.normal` lido de fora ainda entra com a bandeira (o leitor não a recusa).

### P14c-placa 5 (09/11, agente) — (b) holográfica e rebote feitos; miolo e entorno W5d abertos

Commits `a8a899fb71` (holográfica) e o deste bloco (rebote + plano).
- **Entrou:** `crates/ph2d-light2d-gpu/src/wgsl/hrc_momento.wgsl` (novo, concatenado depois de `c2_hrc.wgsl` em `shaders.rs`): `hrc_soma_momento` (`:33`) e `hrc_saida_momento` (`:55`), pipelines PRÓPRIAS no grupo 1 (acumulado 40, c0 de `M` 44–45, grade 46) — nenhuma ligação nova nas de hoje. A soma partilha a parcela da `hrc_soma` (`parcela_da_soma`, `c2_hrc.wgsl:815`: o «+1», o τ do translúcido; F bit a bit igual) × `ω̂_q·sin(π/4)/(π/4)`; a saída faz o borrão em cruz. `MomentoHrc` (`solver_momento.rs:311`) corre logo depois de cada passo Soma/Saída no laço dos quadrantes (`momento::depois`, chamado em `solver_hrc.rs:637`). A saída não liga o grupo 3 da holográfica: 5 texturas de escrita passavam o limite de 4 por estágio (visto: `Too many bindings of type StorageTextures`, F lixo — por isso a grade vai no uniforme 46). `Momento::novo` aceita a holográfica (o `so_a_c0` continua sem `M`); `PassList::com_momento` declara nela o acumulado (2·16 B/texel), a grade (16 B) e os passes `luz2d momento hrc soma/saída` (`passlist_momento.rs:49`); `bytes_do_momento` = o que a lista declara a mais. Com momento, o `gravar_por_fase` (instrumento) não corre.
- **Rebote:** sem código novo — a luz do rebote entra na emissão, logo nos `R0` (holográfica) e no nível 1 (clássica): o `M` já a tem. Provado pelo gate.
- **Gates** (`momento_na_placa.rs`, RADV; `modos()` = clássica e holográfica, `:52`): `a_placa_e_o_gabarito_concordam_com_normal` holográfica `n=0` 0,0148 · `(1,0)` 0,0331 · `(0,1)` 0,0168 · `(−0,6,−0,8)` 0,0215 (teto 0,045; a clássica igual a antes) · `junto_ao_opaco_o_momento_e_o_do_gabarito` holográfica 0,0285 · 0,1298 · 0,1551 · 0,0515 · 0,0585 (teto 0,17) · `sem_normal_nao_aloca_o_momento` nos dois (holográfica 0 / 1 048 592 B, F bit a bit) · `com_o_rebote_a_placa_e_o_gabarito_concordam_com_normal` (`:295`, novo: a sala do G9, degrau Baixo, 60 quadros, 4 pontos × 4 normais, erro / luz do plano): clássica 0,1832 (teto 0,20) · holográfica 0,1329 (teto 0,15) · `g5_o_momento_e_o_que_a_pass_list_declara` nos dois (`g5_bytes_declarados.rs:374`; a holográfica com 6 recursos e os 3 passes).
- **Vistos vermelhos (árvore restaurada):** `ω̂ → −ω̂` só na holográfica — concordam `(1,0)` 0,0506, junto 0,8542, rebote 0,5530; sem acumular por quadrante — concordam `(0,1)` 0,1171, junto 0,5008; a grade não declarada — G5 «alocado e não declarado».
- **Regressão por nome (verde):** `momento_na_placa` (5), `g5_` (3), `holografica` (8), `ph2d-app-light2d g6_` (2), `ph2d-render g7_` + `a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte` (4).
- **Falta:** (1) o `±x` junto a paredes na holográfica (0,13–0,16): o cone de 90° do nível 0 dá ao quadrante todo `ω̂_q`; a cura medida a fazer é partir o `R0` nas DUAS metades da descida do nível 0 (`soma_da_descida`, `lado_do_cone` 0/1, cada uma × `ω̂(±22,5°)·sinc(π/8)`), num passe próprio depois do `Desce` de nível 0 — com os quadrantes juntos (`hrc_quadrantes_juntos`) as metades precisam de um buffer por quadrante. (2) O miolo: `M` dentro dos opacos continua o bilinear cru (o gabarito difunde `[F; M]`, `Miolo<N>`); falta a difusão de `M` na pirâmide do `c3_miolo`. (3) O entorno W5d (`PH2D_LIGHT2D_BORDA=0`, a c0 do entorno somada na saída): o `M` de fora não entra (o entorno `so_a_c0` não tem `M`); com a borda do bug 6 (o produto) a luz de fora já chega pelos `R` da vista. (4) Os da placa 4: (c/d) e a `c3_fina_cantos` sem `M`.

### P14c-pixel 2 (09/11, agente) — ✅ o Pixel Sprite entra na luz (banda normal, oclusão, No Light), editor e vista da câmera

Commits `bf4b7e4370` (família, 1.ª volta) · `64306ca279` (costura + crate nova) · `abe5262c13` (gates de placa) · `7980bc3fe2` (gates mudados para fora da `ph2d-app-light2d`, catraca) · o deste bloco.
- **Crate nova `ph2d-pixel-luz`** (sem `PixelApp`; `src/lib.rs`): `ObjetoPixel` (`:19`, o objecto já posto pela porta imagem → mundo) · `SEM_LUZ = [128,128,0]` (`:34`) · `base_da_instancia` (`:39`, `L = A·diag(1,−1)` de `image_to_world` com ppm 1) · `normal_no_mundo` (`:58`, `n_xy ← L⁻ᵀ·n_xy·√|det|`, renormalizada com `n_z`) · `banda_normal` (`:126`, `cam · img_mundo` = o afim da cor; `None` sem Normal Map VISÍVEL nem No Light; cache por objecto `(stamp, quadro, base, sem_luz)`) · `OclusaoDoPixel` (`:141`, `CamadaDeOclusao` em `:230`: alfa e albedo da `compose_rgba` no quadro mostrado, só a caixa do sprite).
- **`ph2d-app-light2d`:** `NormaisDasFaixas` (`src/normais.rs:11`; `colar` `:34`: banda → intermediário do Vello → cópia para textura própria → cor por `blit_com_normal`; sem banda, a colagem de sempre e a textura solta) mora no `LuzDoQuadro` (troca com a vista da câmera pelo `troca` que já existia); `normais_do_quadro` (`quadro.rs:643`) esvazia e dá os `NaoRecebeLuz` só com a luz acesa; `cena.com_normal = normais.alguma()` no `preparar` (`:232`) e no `recolher` (`:339`).
- **`ph2d-app-pixel/src/luz.rs`:** `objetos` (`:14`, pela porta `to_view` + `quadro_mostrado`), `faixa_normal` (`:44`), `oclusao` (`:59`, todos os `PixelSprite`; a luz filtra por `quem`).
- **Shell (composição):** `fase_vector_bands_doc.rs:53` (uma banda por faixa, ao lado da cena; a vista da câmera passa `&mut v.luz2d`), `present_bands.rs:192,310` (`g.normais.colar`, no lugar das duas colagens de faixa de cena), `present_luz.rs:48` (a oclusão do Pixel no `com_oclusoes`).
- **`NaoRecebeLuz`:** `band_blit.wgsl` `fs_main`: com a banda, `nb.a > 0 && nb.b < ¼` ⇒ ramo sem luz.
- **Catraca:** `ph2d-app-pixel` entrou nesta onda já a 11 353 contra o tecto de 11 278 (P14b/P14c-pixel 1); as cenas 1 e 3 (`FACE`, `BALL*`) e as funções puras da cena 9 foram para `ph2d-pixel-cenas` (`basicas.rs`, `selecao.rs`): −75, logo esta parte é −8 líquida. ⚠️ A catraca das `ph2d-app-*` continua VERMELHA: `ph2d-app-pixel` 11 345 / 11 278 e `ph2d-app-sprite-anim` 5 228 / 5 209, nenhuma das duas desta parte. A `the_shell_only_shrinks` (a da shell) está VERDE.

**Gates de placa** (`crates/ph2d-pixel-luz/tests/it/na_luz.rs`, RADV, 64 × 32 px a 16 px/m, fixture `heroi_p14`, lâmpada de fábrica em (−1,5, 0); cada um visto VERMELHO por mutação real, árvore restaurada, depois verde):
- `o_pixel_sprite_faz_sombra`: fluência 0,85 m atrás do disco **0,0000** contra **2,6785** sem a família (à frente 9,96 contra 9,91). Vermelho: alfa da oclusão a 0 ⇒ × 1,000.
- `a_normal_map_sombreia_e_nao_muda_a_cor` (4.ª condição): cor composta igual byte a byte; luma claro/escuro a ±0,3 m do centro: sem normal **119,3 / 99,0 = 1,205** (momento 0 B), com a roda **141,7 / 51,3 = 2,760** (momento 385 040 B). Vermelho: `com_normal = false` ⇒ 1,205 nos dois.
- `o_no_light_tira_so_esse_objeto_na_colagem` (sala escura, a mesma faixa): o vizinho **0,0** (apagada 130,0), o marcado **130,0** = apagada. Vermelho: sentinela desligada no wgsl ⇒ o marcado 0,0.
- `g6_o_pixel_sem_normal_a_vista_nao_aloca_nada`: momento **0 B**, banda **0 B**. Vermelho: `com_normal = true` sempre; `colar` com banda sempre. ⚠️ Os `ph2d-app-light2d g6_` de antes ficam VERDES sob `com_normal = true` (não o viam): o G6 do Pixel é este.
- 📏 `medida_o_limiar_da_sentinela_na_borda`: o marcado por cima do vizinho, deslocado ½ px: **0 píxeis intermédios** na linha do centro (`b` só 255 ou 0, alfa 255) — o Vello com `ImageQuality::Low` não mistura a borda da imagem, logo hoje o ¼ não decide nenhum píxel. «Sem normal» nos gates = a camada Normal Map ESCONDIDA (tirar a bandeira faz dela cor: visto no 1.º vermelho).
- **CPU** (`ph2d-pixel-luz src/testes.rs`, os 5 da 1.ª volta, verdes): `a_normal_roda_com_a_instancia`, `a_banda_da_fixture_girada_aponta_para_o_mundo`, `sem_normal_a_vista_nada_e_alocado`, `o_no_light_tira_so_esse_objeto`, `a_oclusao_do_pixel_e_o_disco_no_lugar_dele`.
- **Regressão por nome (verde):** `ph2d-app-light2d g6_` (2), `ph2d-render a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte`, `ph2d-host-pixel-scenes scene_9_shows_each_thing_of_the_wave` / `smoke_3_is_an_indexed_ball…` / `smoke_1`, `ph2d-host-desktop a_vista_da_camera_leva_os_palcos_e_as_formas_fora_da_placa`, `the_shell_only_shrinks`.

**Falta:** (a) a catraca das `ph2d-app-*` vermelha por `ph2d-app-pixel` (+67, de blocos anteriores) e `ph2d-app-sprite-anim` (+19), não desta parte; (b) a oclusão compõe a cor do sprite inteiro a cada quadro (sem cache como a banda) — não medido em cena cheia; (c) o receptor DENTRO do sprite que bloqueia (o `M` dentro de um opaco, P14c-placa 4/6); (d) o Pixel no palco do jogo (`sem_pixel`) não pinta, logo não tem banda; (e) uma cena de smoke do Pixel na luz (número a contar no roteador) e o smoke do dono.

### P14c-placa 6 (09/11, agente) — M dentro dos opacos, as metades do cone 0 e o canto adiado

Commits `0412317ece` (miolo de `M`), `56e2ace4da` (metades) e o deste bloco (canto + plano). Tudo em `crates/ph2d-light2d-gpu`.
- **(2) Miolo de `M`:** `MioloDoMomento` (`solver_momento.rs`): depois do miolo de `F` (`solver_gravar.rs`, `gravar_miolo`), a pirâmide DELE (livre até o quadro seguinte) corre outra vez para `Mx` e para `My` — dois `DoMiolo` próprios (pipelines próprias) com a fluência trocada pela fina de `M`, o conhecido fino trocado por um próprio (`luz2d/tex/momento_miolo_conhecido`) e uniforme próprio (`luz2d/buf/momento_miolo`). O classifica é `miolo_classifica_momento` (`c3_miolo.wgsl`): o rgb da fina de `M` com o PESO do conhecido de `F` — a lei do gabarito, que difunde `[F; M]` (`Miolo<9>`). A «transmitida» é a OUTRA fina de `M` (alfa 1): `M` não atravessa translúcidos. Sem o anel de fora da região (o uniforme de `F` o tem): com região < grade, Neumann (a lei da W5h). `Entradas.momento` (`solver_miolo.rs`, 699 LOC) e `regiao_de`. Declarado em `PassList::com_momento` (o passe `luz2d momento miolo`).
- **(1) Metades do cone 0 (holográfica):** `hrc_metades_momento` (`hrc_momento.wgsl`, pipeline própria logo depois do `Desce`; só o nível 0 escreve) guarda `R₊ − R₋` das duas metades (a conta da `descer_cones`) no acumulado de `M` (um bloco por quadrante com os quadrantes juntos; `bytes_do_acumulado`); a `hrc_soma_momento` soma `(R₊ − R₋)·ω̂_q⊥·sin(π/8)·sinc(π/8)` na sonda que a parcela leu (`ParcelaHrc.sonda/escala`, `c2_hrc.wgsl`; a `hrc_soma` não os lê). A parte `R0·ω̂_q·cos(π/8)·sinc(π/8)` é exatamente o `sinc(π/4)` de antes. Override `METADES_DO_MOMENTO` (controlo).
- **(3) `c3_fina_cantos`:** MEDIDO que a composição já o exprime — a `c3_momento` compila com as constantes `comuns` (o canto LIGADO, sem `CANTO_ADIA`): calcula o texel de duas faces EM LINHA com o mesmo código que o passe próprio usa para `F`. Nada de código novo; gate novo prende-o.
- **Gates** (`tests/it/momento_na_placa.rs`, RADV): `dentro_do_opaco_o_momento_e_o_do_gabarito` (novo; corpo 0,8×1,8 m, lâmpada e céu, texels a > ½ texel da borda; n = 0 · (1,0) · (−1,0) · (0,1) · (0,−1)): clássica 0,0645 · 0,1779 · 0,0716 · 0,0941 · 0,0728 (teto 0,20; antes/vermelho com o miolo de `M` desligado 0,0645 · **0,8590** · 0,2770 · 0,1795 · 0,1527); holográfica 0,0546 · 0,1703 · 0,0678 · 0,0826 · 0,0715 (teto 0,19). · `junto_ao_opaco_o_momento_e_o_do_gabarito` holográfica 0,0285 · **0,0555 · 0,0621** · 0,0378 · 0,0373 (teto 0,17 → **0,075**; vermelho com `METADES_DO_MOMENTO = false`: 0,1298 · 0,1551, os da placa 5 exatos). · `com_o_rebote_…` holográfica 0,1329 → **0,0407** (teto 0,15 → 0,05). · `a_placa_e_o_gabarito_concordam_com_normal` holográfica `(1,0)` 0,0331 → 0,0289 (teto 0,045 mantido). · `com_o_canto_adiado_o_momento_e_o_do_canto_em_linha` (novo; fina 2, `Canto::passe_proprio`): `M` bit a bit igual com e sem o passe próprio; no vértice (≤ 3 texels das duas faces) clássica 0,3013 · 0,2966 · 0,3427 · 0,3616 · 0,3314, holográfica 0,3102 · 0,3169 · 0,3458 · 0,3649 · 0,3443 (teto 0,38; o grosso é o `F` do vértice, `n = 0`); vermelho com a `c3_momento` em `CANTO_ADIA` («mexeu no M»). · `g5_o_momento_e_o_que_a_pass_list_declara`: clássica 6 recursos, holográfica 8, + passes `luz2d momento miolo` e `luz2d momento hrc metades`. Os quadrantes juntos (`hrc_quadrantes_juntos`) medidos à mão: os mesmos números.
- **Regressão por nome (verde):** `momento_na_placa` (7), `g5_` (3), `holografica` (8), `o_miolo_na_placa` (6) · `ph2d-app-light2d g6_` (2) · `ph2d-render g7_` + `a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte` (4). Árvore nunca deixada mutada.
- **Falta:** (a) o `(1,0)` dentro do opaco (0,17–0,18): face de COSTAS para a lâmpada — o relativo de uma luz pequena; medir em absoluto antes de mexer. (b) o vértice do opaco: 0,30 já no `F` (o miolo de `F` no canto, não o `M`). (c) `M` em translúcidos dentro do miolo é só a difusão (a `transmitida` de `M` não existe; o gabarito com τ < 1 não foi medido para `M`). (d) o miolo de `M` sem o anel de fora da região: com a região < grade a moldura é Neumann (medir com `regiao_fina` parcial). (e) o entorno W5d (`PH2D_LIGHT2D_BORDA=0`) continua sem `M` de fora (placa 5 «Falta» 3). (f) a memória: o miolo de `M` reusa a pirâmide de `F` (só +8 B/texel fino e o uniforme); o acumulado da holográfica +16 B/texel da grade (×4 com os quadrantes juntos).
