# Plano: Shape «Outline» pelo contorno, Collision Shape pela Hierarchy, várias formas num corpo (`line/GameEngineFuncional02`)

Leitor: a próxima LLM desta linha. Aprovado pelo dono em 08/11/2026 («faça como você sugere»). Oráculo do contorno:
[`../oraculo_contorno/README.md`](../oraculo_contorno/README.md) (Godot 4.7.2, MIT; fixtures + `cobertura.txt`).

## Decisões do dono (08/11)

- Animated Sprite: **um contorno só** = união (OR) do alfa dos quadros usados pelas tags (todos, se não houver tags).
- Reentrância = **sólida** (peças convexas). **Buraco fechado continua buraco.**
- ⭐ **Dono 09/11 («SIM» a «a Godot deixa o contorno sair um pouco para fora da imagem para usar menos pontos»):
  Detail = desvio máximo PARA FORA da arte (nunca para dentro; no buraco, «fora» é para dentro do buraco). Moeda
  r = 20 px: 92 pontos em qualquer Detail antes → 92 · 14 · 8 · 7 a 0 · 2 · 4 · 8 px; herói parado a 2 px 57 → 26.
  Tabela antes × depois e mutações: `crates/ph2d-outline-trace/README.md` §Outward law; portões `tests/fora.rs`.
- Controlos: **Detail** (px, 2), **Alpha cutoff** (0.5), **Ignore specks under** (px), pontos editáveis no canvas
  (arrastar · duplo clique na aresta acrescenta · Alt+clique tira); «Follows the picture» × «Custom size» + «Fit to picture».
  Auto Key grava também Detail e Alpha cutoff (lei de 19/10; retraçar custa 80–310 µs, medido no oráculo).
- **Add ▸ Collision Shape:** sem seleção → objeto próprio com `RigidBody{Static}` invisível; nunca entra noutro corpo.
  Com objeto selecionado → **objeto FILHO** (`ChildOf`, linha nova na Hierarchy), nunca componente do selecionado. Pai
  (ou antepassado) com corpo → mais uma forma desse corpo (W-Compound, «Part of: X»). Pai sem corpo → o filho nasce com
  o seu próprio Static (Inspector: «Own body (Static) — make the parent a body to join it»).
- **Várias formas num corpo: UM jeito só — os filhos** (ADR-0110, `ph2d-physics-ecs/src/bridge/parts.rs`). O cartão Shape
  ganha a lista **«Shapes»**, que é VISTA dos filhos (linha 0 = a forma do corpo; depois cada parte na ordem da árvore;
  tipo, tamanho, offset, rotação; clicar seleciona o filho; «+ Add shape» = o spawn acima; «Remove shape» = apagar o filho,
  um passo de undo). Canvas: «Edit shape on canvas» mostra as alças de TODAS as formas do corpo.
  ⛔ Recusado: `Collider.shapes: Vec<…>` (segunda verdade ao lado dos filhos, perde `Copy`, degrau de schema).
- **Dono 09/11 (A, substitui a de `13902fc1ec`):** Outline que segue a imagem (`custom` falso): Detail, Alpha cutoff e
  Ignore specks under livres sempre. Depois da 1.ª edição de pontos à mão (`custom`): os três TRAVAM fora de «Edit shape
  on canvas» (só inativos — esmaecidos, sem arrasto/escrita/foco, sem losango que grave; o bloco dá a dica
  `card.insp.phys.outline_locked_tip`); dentro da edição destravam e mudar um deles RETRAÇA da imagem (os pontos à mão
  dão lugar aos traçados, um passo de undo, `custom` volta a falso). Lei: `physics_cartao_contorno::travado`. Provas:
  `o_traco_trava_fora_da_edicao_depois_dos_pontos_a_mao`, `o_detail_dentro_da_edicao_retraca_um_outline_a_mao`.
  A **forma solta** perde «Remove Shape» (como a filha): apaga-se na Hierarchy (`o_detail_trava_e_a_solta_sem_remove.rs`).
- **Dono 09/11 (B, pontos animados):** forma com trilha de pontos (51/52) e Auto Key DESLIGADO: todo gesto de pontos
  (arrastar, acrescentar na aresta, Alt+clique, Delete, fundir ao soltar, «Merge», e o retraço de A) não muda nada e
  abre um modal «This shape's points are animated» / «Editing them now would be undone by the animation. Delete the
  points animation to edit them freely?» [Delete animation] [Cancel]; aceitar apaga as trilhas 51/52 num passo de undo
  (a mesma função de `canais_pontos::apagar_do_fit`) e a edição volta; o gesto recusado não se reaplica. Auto Key
  LIGADO: inalterado (grava chave). **Estado: por construir** (ver o handoff da onda).

## Desenho técnico (padrão-ouro)

- `ColliderShape::Outline { half_x, half_y }` (variante ANEXADA, `Copy`, caixa de limites em m) + componente registado
  novo `ColliderOutline { rings: Arc<[Ring]>, detail_q4, alpha_cutoff, min_island_px, follows, source, cell_px }`.
  Anéis em inteiros Q4 (1/16 px), externo CCW, buracos CW. Peças convexas DERIVADAS (função pura), nunca gravadas.
  ⛔ Recusado: `Arc`/`Vec` dentro do enum (tira `Copy` de `Collider`/`ShapeDesc`/`BodyDesc`, 351 ficheiros).
- `PROJECT_SCHEMA` 219 → **220** só pelo componente novo (escada + tripla + espelhos `ph2d-render`/`ph2d-script`).
- Crate pura nova **`ph2d-outline-trace`**: máscara → marching squares inteiro (ordem fixa) → Douglas–Peucker UNILATERAL
  (nunca corta a arte; predicados i64; cruzamento devolve o vértice) → ponte dos buracos (Eberly) → ear clipping exato →
  Hertel–Mehlhorn → fusão de vizinhas dentro do Detail sem cobrir buraco → ilhas abaixo do mínimo saem. Oráculo
  interno: `parry2d::transformation::hertel_mehlhorn_idx` (peças antes da fusão iguais). ⛔ VHACD (aproximado, float).
- Cola `ph2d-physics-outline`: recurso `PictureAlpha` publicado por `sheet::refresh` (sprite-anim); `physics_apply` e
  `follow_auto_collider` chamam `trace()` → `queue_set` (um passo de undo). Rapier: `ColliderBuilder::compound` de
  `ConvexPolygon`. ⚠️ Espelho: escala COM sinal e inverter o sentido quando o determinante < 0 (`scale.rs:75` usa `abs()`).
- Tecto de peças/pontos: MEDIDO antes (W0, `measure_outline_pieces`, `--profile smoke`); acima dele o Detail sobe e o
  cartão diz («Too detailed — Detail raised to {x} to stay under {n} pieces»); nunca corta a arte.

## Ondas

| Onda | O quê | Formato salvo? | Estado |
|---|---|---|---|
| W3 | Add ▸ Collision Shape (3 casos) + lista «Shapes» no cartão | não | **agora** (antes da integração da noite) |
| W1 | `ph2d-outline-trace` contra as fixtures do Godot (0 % de arte fora; anel com buraco; disco em 1 peça; L em 2) | não | **agora** |
| W0 | medir o custo das peças; escrever o tecto | não | **feito 09/11**: `MAX_PIECES = 256` (tique do solver), `MAX_POINTS = 512` (retraço), `trace_fitted`; tabela no README da crate |
| W2 | motor e formato (schema 220), rapier, rewind, espelho | **sim** | depois da integração |
| W4 | Outline no cartão + Auto Key | sim | depois |
| W5 | canvas: alças de todas as formas, pontos do Outline, seguir a imagem | sim | depois |
| W6 | cena 120 (contorno, argola, corpo de 3 formas) + fotos + smoke do dono | — | depois |

Riscos: LOC sem folga em `ph2d-app-physics` e na shell (cura = cortar/MOVER); `InspectorPhysicsInfo` sem `Default`
(outras linhas precisam dos campos novos); Static filho de pai Dynamic é corpo separado; nav usa a caixa de limites.

## ⛔ REVISÃO 08/11 (smoke do dono): opção B — toda forma é um objeto filho

Dono: *«tirar collision shapes dos componentes e deixar apenas como Objeto aparecendo como filhos na hierarchy como
objetos independentes (invisíveis). Sai dos componentes e suas configurações não aparecem no inspector de outros
objetos.»* A lista «Shapes» (W3-B, `986c158873`) é REVOGADA. ADR novo previsto: **0199 «A forma de colisão é um objeto
filho do corpo»** (confirma e estende o ADR-0110; não o revoga).

- **Motor, UMA lei:** as formas de um corpo = o seu `Collider` (se tiver) e depois as formas descendentes em ordem de
  árvore, parando num corpo filho e atravessando grupos (`ph2d_physics_ecs::shapes_in_order`, `parts.rs:150`). A 1.ª é a
  **forma principal**, viaja no `BodyDesc` como hoje (Crouch, linha d'água, força de zona, `rest`, nav e gatilhos leem-na);
  as outras são partes (`bridge/parts.rs`). `BodyDesc.shape_pose: [f32;3]` anexado (zero = hoje). O lado-forma do catálogo
  (`row_of("ph2d::physics::Collider")`: material, one-way, superfície, zonas, sinais, e o futuro `ColliderOutline`) é lido
  da entidade da FORMA. Mesmo hash de 300 ticks construída das duas maneiras (gate de equivalência).
- **Editor, UMA lei:** nenhuma porta do editor põe `Collider` num corpo (gate `nenhuma_porta_do_editor_poe_a_forma_no_corpo`).
  «Collision Shape» sai do Add Component (fica registada). Corpo nasce com a filha (`nasce_com_forma`, physics-ecs).
- ⛔ Recusados: corpos sem collider próprio com todas as formas como partes (parte os 5 leitores da forma principal e a
  ordem dos handles) · motor que recusa `Collider` no corpo (234 ficheiros de teste sem ganho) · sistema por quadro que
  converte (muta o documento e o undo) · `Collider.shapes: Vec` · lista «Shapes» no Inspector do corpo.
- **Migração** `migra_formas_para_filhos(world)` no load, `ver <= 219` → **schema 220** (o Outline passa a **222**: o 221 é a exceção de camadas da forma, B9): por
  `StableId`, filha «Collision Shape» primeira entre irmãs, Transform identidade, leva o `Collider` e o lado-forma; mestre
  só (instâncias materializam); trilhas da Timeline do lado-forma re-alvejadas para a filha; macros resolvem
  `main_shape(body)` ao tocar.

| Onda | O quê | Est. |
|---|---|---|
| B0 | tirar a lista W3-B (fica `shapes_in_order`, os cortes de LOC e a rota `set_command_pick(COLLISION_SHAPE)` = «Add Shape») | 1 h |
| B1 | motor: `BodyQuery` sem `&Collider`, forma principal resolvida, lado-forma da forma, `shape_pose`, ordem das partes | 1 d |
| B2 | migração + schema 220 (+ instâncias, Timeline) | 1–1,5 d |
| B3 | ações: catálogo, seed do `RigidBody`, Player/Character/Static nascem com filha, `pendura_forma`, `follow_auto_collider` pelas formas da imagem, Make Independent Body | 1 d |
| B4 | Inspector: corpo só Body; filha com o cartão Shape inteiro + Collision Layers + «Shape of: {body}» | 1 d |
| B5 | canvas: corpo selecionado mostra as alças da forma filha | 0,5–1 d |
| B6 | força de zona em formas extra (efetor por collider) | 0,5 d |
| B7 | cenas (120, jogo, macros 8) por `nasce_com_forma`; fotos; smoke | 0,5 d |

Outline (W0–W6) passa para a FILHA: `ColliderOutline` na forma, `source` = antepassado com imagem mais próximo até ao
corpo. **Último ponto seguro para integrar:** o commit da B0 (sem a lista revogada, sem mudança de formato).

### Respostas do dono à B (08/11)

1. Add ▸ Collision Shape **sem seleção** → objeto raiz «Static Body» (Static, sem imagem) + filha «Collision Shape», a filha selecionada.
2. **Com uma imagem sem corpo selecionada** → a imagem VIRA corpo Static (um passo de undo) e a forma nasce filha dela (opção a).
   A nota «Own body (Static) — make the parent a body to join it» sai.
3. **A forma é como qualquer outro objeto:** alças só aparecem, e ela só se edita, quando ELA está selecionada.
   ⇒ **B5 muda:** corpo selecionado NÃO mostra alças da forma filha (o contorno no canvas continua, como hoje).

## W2 — desenho fechado (09/11, depois da W0; vale sobre o «Desenho técnico» acima onde divergirem)

**Modelo (ECS, `ph2d-physics-ecs`), na forma FILHA (opção B):**
- `ColliderShape::Outline { half_x, half_y }` — variante ANEXADA (`Copy`). É a caixa do RETÂNGULO DA IMAGEM em
  metros, não a dos anéis: o anel mapeia o retângulo da imagem para `[-half_x, half_x] × [-half_y, half_y]`.
  «Follows the picture» é DERIVADO como no `Cuboid` (`shape_fitted_to(picture)` = metade da imagem; `follows_picture`
  compara) — **sem campo `follows`**. «Custom size» = mudar `half_*`; «Fit to picture» = `shape_fitted_to`.
- Componente registado novo **`ColliderOutline { rings: Arc<[Ring]>, picture_px: [u32; 2], detail_q4: u16,
  alpha_cutoff: u8, min_island_px: u16 }`** na mesma entidade do `Collider`. `rings` em Q4 (1/16 px, y para baixo,
  origem no canto da imagem de `picture_px`) — é a VERDADE editável (pontos do W5); as peças são DERIVADAS
  (`ph2d_outline_trace::convex_pieces(rings, detail_q4)`), nunca gravadas. **Sem `source`**: a imagem é derivada
  (o antepassado com imagem mais próximo, até ao corpo; o próprio objeto se ele tiver imagem). Sem `cell_px`
  (o Animated Sprite traça a UNIÃO das células do tamanho de uma célula = `picture_px`).
- Ponto em metros locais da forma: `x = (qx / (16·pw) − ½)·2·half_x·sx`, `y = −(qy / (16·ph) − ½)·2·half_y·sy`,
  com a escala do `Transform` COM SINAL. Espelho (`sx·sy < 0`): cada peça entra por `SharedShape::convex_hull`,
  que não depende do sentido — nada a inverter à mão. ⚠️ `scale.rs` usa `abs()` para as outras formas; o Outline
  precisa do sinal (uma forma assimétrica espelhada é OUTRA forma).

**Motor (`ph2d-physics`):**
- `ShapeDesc::Outline { half_x, half_y, key: OutlineKey }` (`Copy`; `OutlineKey(u64)` = hash de conteúdo das
  peças JÁ em metros e escaladas). `PhysicsWorld::register_outline(Vec<Vec<[f32; 2]>>) -> OutlineKey` guarda em
  `outlines: BTreeMap<OutlineKey, Arc<[Vec<Vector>]>>` — só insere (conteúdo endereçado; o checkpoint clona os
  conjuntos do rapier com o compound dentro, e o rewind do ECS que re-spawna por `BodyDesc` acha a chave porque
  ela nunca sai). `shape_builder` → `ColliderBuilder::compound` de `convex_hull` por peça; chave ausente ou vazia
  → `cuboid(half_x, half_y)` (nunca pânico). Massa = densidade × área do compound (o rapier calcula).
- Cada `match` sobre `ShapeDesc` ganha o braço: nav e sombras leem a caixa (`half_*`) como um `Cuboid`.

**Ponte:** onde um `BodyDesc`/peça nasce (`scale.rs::body_desc`, `bridge/bodies.rs`, `bridge/parts.rs`), o
`Outline` lê o `ColliderOutline` da entidade da FORMA, deriva as peças (cache por `(hash dos anéis, detail)` —
nunca por tique), converte com a escala com sinal, regista e põe a chave. Forma sem `ColliderOutline` ou de
anéis vazios → a caixa.

**Formato:** variante + componente novo ⇒ `PROJECT_SCHEMA` sobe UM degrau, CONTADO (`python3 scripts/schema-recount.py`,
o main está em 225). Sem leitor antigo (§0.11).

**Produtor (W2c):** quem escreve o `ColliderOutline` é o «trace» da imagem-fonte (máscara pelo `alpha_cutoff`;
Animated Sprite: OR das células usadas pelas tags, todas se não houver tags) → `trace_fitted` → `queue_set` (um
passo de undo); o cartão diz quando o tecto subiu o Detail (W4).

### Endereços conferidos (09/11) para W2b–W6

- **Pixels:** `SpritePixels(AssetId)` (`ph2d-ecs/src/sprite_pixels.rs:48`) ou atlas → `AssetId` (`ph2d-render/src/sprite/image_of.rs:31`
  `texture_of`) → `AssetDb::image_rgba8` (`ph2d-asset/src/asset.rs:152`); folhas decodificadas em `ph2d-anim-sheets/src/sheet_cache.rs`
  (`:135` `image_rgba8`, `:147` `refresh`). Animated: `SpriteAnimations` (`ph2d-ecs/src/sprite_anim.rs:243`), folhas extra
  `sprite_sheets.rs:51`, células de uma tag `sprite_anim_frames.rs:141` (`tag_cells`), `:72` (`sheet_cells`). Tamanho em metros:
  `Sprite.size` (`ph2d-render/src/sprite/component.rs:66`). Precedente de leitor de alfa: `ph2d-sprite-edit/src/pivot.rs:31,51`.
- **Cartão Shape:** seletor `INSP_PHYS_SHAPE` (`ph2d-panel-inspector/src/ids/inspector_physics_body.rs:56`, `sections/physics_cartao_forma.rs:143`);
  `PhysicsFieldEdit` (`ph2d-editor-core/src/screens/hero/inspector_model_physics.rs:359`, `FitToPicture :371`, `HalfX :388`); evento
  `ph2d-panel-inspector/src/event_physics.rs:177-179,285`; aplicação `ph2d-app-physics/src/physics_apply.rs:95-107` (Fit), `:422-530`
  (por tipo; ficheiro NO tecto de 600); leitura `inspector/body.rs:252-263,334`; strings `ph2d-i18n/src/cartoes.rs:9,32-35`.
- **Auto Key:** `ph2d-ecs/src/canal_fisica.rs` (`RAIO = 21 :45`, `CAMPOS :79`, `e_da_forma :156`, `info :162`; só APPEND);
  ler/escrever `ph2d-app-physics/src/canais_corpo.rs:57,96,114,156,232,276,304`.
- **Canvas:** `ph2d-shape-handles` (`lib.rs:112` `alcas`, `:183` `alca_em`, `:226` `puxa`; `ponteiro.rs:16,61,165-245`; `tinta.rs:21,50`);
  cola `shells/desktop/src/forma_no_canvas.rs`; contorno sem seleção `ph2d-app-physics/src/overlay/outline.rs:77`, `overlay/shapes.rs:29`.
  Vetor: inserir na aresta `ph2d-vec-edit/src/node_hit.rs:105`, apagar vértice `selection.rs:594` (o «duplo clique / Alt+clique» não existe lá).
- **Cena 120:** `ph2d-physics-scene/src/ficha.rs:120` `monta`; com imagem: Crate (`azulejo`), Stretched Ball (`COIN_SHEET`), Updraft Zone.
- **`queue_set`** (`ph2d-inspector-ordering/src/lib.rs:179`) serializa por postcard qualquer `T: Serialize` registado (no-op calado se não
  registado); precedentes não-`Copy`: `JOINT` (`ph2d-app-physics/src/joint.rs:327`), `SpriteAnimMarkers`. `serde` com `rc` no `ph2d-ecs`.
- ⚠️ **Tecto de LOC:** `ph2d-app-physics` está 64 linhas ACIMA (55 579 > 55 515, `crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs:221`)
  por commits anteriores desta linha. Cura = MOVER para crate não-`app` antes de W4 (que acrescenta lá).

## Auto Key dos pontos do Outline — desenho (FEITO 09/11)

Lei do dono (19–20/10, `project-memory/project_autokey_records_everything_2026_10_19.md`): o Auto Key grava TUDO, pontos
incluídos; mexer um ponto grava a FORMA INTEIRA numa chave; acrescentar/apagar ponto vale em TODAS as chaves (chave por
ponto recusada). Hoje (W5, `ph2d-shape-handles/src/pontos.rs`) os gestos sobre os pontos são passo de undo mas NÃO têm canal:
`every_editable_field_is_a_channel` só vê Detail/Alpha/Specks (`canal_fisica.rs` 48–50). Não coube nesta janela sem meio-feito.

**Medido no vetor (a lei a seguir, não outra):** cada vértice é escalar `FieldKey(VETOR, campo, i = contorno, j = vértice)`
(`canal_vetor.rs` `ANCORA_X..TIPO_VERTICE`), todos no MESMO grupo `FORMA` (uma chave = a forma inteira), interpolação
contínua (linear, a das âncoras); a topologia corre em `shells/desktop/src/render_loop/autokey_pass.rs:105`
`propagar_topologia` → `ph2d_app_vec::canais_topologia::{mudou, propagar}` (alinhamento LCS por âncora exacta, um passo de
undo da timeline próprio ou o do gesto aberto). Um blob de anéis com degrau seria uma SEGUNDA lei — recusado.

**Representação:** `canal_fisica::CONTORNO_PONTO_X = 51`, `CONTORNO_PONTO_Y = 52`, `FieldKey(FISICA, campo, i = anel,
j = ponto)`, valor = o `[i32; 2]` Q4 de `Ring` como `f32` (exacto até 2^24 — o tecto de pontos `raised_q4` fica muito
abaixo), grupo `canal.fisica.contorno_pontos` (um grupo = uma chave por forma), na trilha da FORMA (`dono_da_chave`, opção B).
Publicados por `canais_corpo::publicar` **só quando `ColliderOutline.custom`**: um Outline traçado da imagem é derivado
(Detail/Alpha/Specks são as suas chaves); o Fit (volta ao traçado) é estrutura — apaga a trilha dos pontos num passo de undo.

**Interpolação:** linear entre chaves com o MESMO nº de anéis e de pontos por anel (garantido pela propagação: a topologia de
hoje é levada a todas as chaves, como no vetor). Se o anel interpolado reprovar `pontos::validos` (auto-intersecção a meio),
esse quadro segura a chave anterior (degrau só nesse caso, medido por quadro) — nunca entrega ao solver um anel inválido.

**Onde:** (1) `ph2d-ecs/src/canal_fisica.rs` — 51/52 em `CAMPOS` só se a família aceitar `i,j` (hoje os da FISICA são
`i=j=0`: conferir `e_da_forma` e a tabela de nomes); (2) `ph2d-app-physics/src/canais_corpo.rs` `publicar`/`escrever` — leitor
e escritor dos anéis; (3) `ConducaoFisica` (`canais_corpo.rs:356`, já guarda o par autorado `(Collider, ColliderOutline)`
desde `e6e90d8364`) — a tocar, a chave dos pontos é pré-visualização: a fotografia do undo vê os anéis autorados;
(4) a propagação: extrair o alinhamento LCS de `canais_topologia` para uma função pura parametrizada por (família, campos de
âncora) e chamá-la em `propagar_topologia` também para FISICA — UMA função, dois chamadores; (5) `pontos::escreve` não muda:
o Auto Key grava ao soltar pela mão da família (`ponteiro::publica_a_mao`, já publicada).

**Gates a escrever:** arrastar um ponto com Auto Key armado = UMA chave com todos os pontos (pelo caminho do quadro, não por
chamada directa); duplo clique numa forma com 2 chaves ⇒ as duas chaves ganham o ponto (alinhado); a tocar, quadro entre
chaves = anéis interpolados e nenhum passo de undo; Fit apaga a trilha; `every_editable_field_is_a_channel` classifica os
gestos dos pontos como `Canal`. Mutações: sem propagar à outra chave ⇒ vermelho; publicar com `custom = false` ⇒ vermelho;
interpolar sem a guarda de `validos` ⇒ vermelho no caso da auto-intersecção.

**Fase 1 FEITA (09/11)** — `ph2d-physics-outline::canais_pontos` (publicados só `custom`; `escritos` com a guarda; `aplicar`;
`na_chave_anterior`; `e_fit`/`apagar_do_fit`); `canais_corpo` só chama (`aplicar_pontos`, o par autorado na `ConducaoFisica`,
`no_tempo` = o relógio da vista, posto em `fase_timeline_drain.rs`). ⭐ A guarda é DETERMINÍSTICA: o quadro inválido dá a chave
ANTERIOR no instante do clip (nunca «o último quadro válido do mundo», que depende do caminho — recusado pelo coordenador).
Também: com `custom`, uma curva de 48–50 NÃO retraça (`retrace_followed`). O Fit apaga a trilha em `propagar_topologia`, no
passo de undo dela. `every_editable_field_is_a_channel` NÃO se aplica aos pontos: a população dele são linhas pintadas com
losango no painel, e os pontos são gesto de canvas (uma entrada não casaria nada e o censo de obsolescência reprovaria).
**Fase 2 encaixa:** `shells/desktop/src/render_loop/autokey_pass.rs` `propagar_topologia` (o 2.º chamador do LCS, ao lado do
`apagar_do_fit`), com o alinhamento extraído de `ph2d-app-vec/src/canais_topologia.rs:41` `alinhar`; `escritos` já assume a
topologia de hoje em todas as chaves.

**Fase 2 FEITA 09/11** — `2d7eb1435a`: o LCS saiu para `ph2d_timeline::topologia::{contornos, alinhar}` (const-genérico
em N campos; as duas crates já dependiam da timeline, sem ciclo) — UMA função, dois chamadores (vetor, 8 campos; FISICA,
2); `canais_pontos::{mudou, pede_topologia, propagar}` (ponto novo no mesmo parâmetro do segmento dos vizinhos
alinhados, em cada chave; trilhas órfãs purgadas; o Fit dentro); a shell só troca a chamada (+0). Gate
`acrescentar_e_apagar_um_ponto_vale_nas_duas_chaves` (mutações: só a 1.ª chave · sem a posição alinhada).
`c263478e25`: `a_forma_outline_atravessa_o_arquivo` leva `custom` e a trilha 51/52 pelo `read_project_file` (mutação:
`#[serde(skip)]` no `custom`). `983c9864bc`: o undo pela shell (`render_loop/autokey_pontos_tests.rs`: o passe do quadro, `push_linked` +
`liga_gesto_da_cena`, o Ctrl+Z de `apply_undo`) — `arrastar_um_ponto_com_auto_key_e_um_ctrl_z`,
`acrescentar_um_ponto_com_duas_chaves_e_o_fit_sao_um_ctrl_z_cada` (mutações: a shell sem `canais_pontos::propagar` · sem
a ligação do passo à chave). Espaço: os render-and-look `wet_*_look_probe` mudaram para `ph2d-tool-painter/tests/it`.
