# Physics Body, Static Body e Collision Shape (`line/GameEngineFuncional02`)

Estudos lidos: crítica «Physics Body», «Static Body», «Collision Shape»; estudo «Corpo físico», «Colisor»; 12 ideias.
Código conferido em: 07/11/2026, sobre `652b506b58`. Prefixo `f1_` para não colidir com a numeração da linha irmã.
Maquetes aprovadas: [`f1_maquete.html`](f1_maquete.html) (abra no navegador; a opção B é a aprovada). Fotos de hoje:
[`../handoffs/fotos_fisica/`](../handoffs/fotos_fisica/).

## Como está (conferido)

- Corpo = `RigidBody.kind` (Dynamic, Static, Kinematic); forma = `Collider` (Ball, Cuboid, Capsule). «Static Body» é atalho
  do menu Add (`ph2d-app-game/src/object_add.rs`). Collision Shape é objeto, sem entrada em Add Component.
- Cartão: `ph2d-panel-inspector/src/sections/physics*.rs`, `event_physics.rs`, `populate_physics.rs`; aplica em
  `ph2d-app-physics/src/physics_apply.rs`; testes de costura em `tests/it/seam_physics.rs`.
- Campos medidos nas fotos: Dynamic 35 + 3 botões · Static e Kinematic 21 + 2 · Zona 28 + 2 · forma sem corpo 19 + 2.
- **O que a crítica errou:**
  - Contou 30, 17 e 14 campos, e não dá destino a Grip, Belt e Join As.
  - A colisão acompanha Width (`tiled.rs::follow_auto_collider`). Teste: `colisao_segue_a_largura`, e foto
    `fisica_baixa`/`fisica_larga`: Width 2,4 → Half Size 120; Width 4 → 200.
  - Existe aviso de forma solta e linha do dono (`physics_part.rs:60`), mas **a frase aparece cortada** no cartão.
  - «Make Independent Body» já existe; ela o dava como novo.
- **Density em corpo Static e Kinematic não tem efeito** (`density_sem_efeito`: idêntico bit a bit com 0,01, 1 e 1000;
  Dynamic é o controlo). Numa peça de corpo Dynamic ele conta: fica.
- Defeitos: o cartão da forma chama-se «Physics Body»; a dica do Static Body manda escalar, contra a decisão de 26/10;
  Offset tem rótulos diferentes em corpo («Offset (px)») e peça («Offset X (px)»).
- Não acompanham a imagem, sem aviso: forma mudada à mão, bola e cápsula. Bola esticada vira elipse sem aviso.

## O que as 9 ensinam

Unity e Cocos: botão que acende as alças na cena. Godot: camadas em grade com nomes. Roblox, GDevelop: campo que abre
os detalhes abaixo. Evitar: escala que quebra sem aviso (Godot), Is Trigger perdido numa caixa (Unity).

## Decisões do dono

Vigentes: forma nasce com o corpo (14/09); blocos que repetem = Scale 1, Width e Height (26/10); app só em inglês;
cartão compacto (04/11); [não perder poder](../../../project-memory/project_physics_cards_keep_all_config_power_2026_11_07.md).

Desta conversa, 07/11, todas por «faça como você sugere»:

1. **Opção B:** dois cartões, Body e Shape.
2. **Alças da forma no canvas entram agora**, com botão «Edit shape on canvas»; os números continuam.
3. **Rótulo Touch, opções Solid | Sensor.** «Trigger» continua a ser o componente da tecla.
4. **Polígono pelo contorno: item próprio, logo depois desta revisão.** Mexe no motor e no formato salvo.
5. **Static Body continua a nascer com o bloco de pedra.**

**Decisão do dono, 07/11/2026 (onda 7):** o sistema de camadas sai do cartão Shape para um **cartão próprio «Collision Layers»**, logo depois dele: Layer, Collides with, a frase, Rename…, Edit matrix… e o campo Layer name. Touch e One-Way ficam no Shape. Mesmos ids e mesmas edições: só o lugar. Corpo, Zona e peça: cartão inteiro. Forma solta: só Layer e Rename…. Sem `Collider`: sem cartão. Resumo dobrado: `Props · Ground, Props, Zones`; na solta só o nome.

**Decisão do dono, 07/11/2026 (colisão):** a colisão fica em **matriz global + fila «Collides with» de quadros clicáveis no objeto** (a fila é a onda 6). **32 camadas**, mostrando só as que importam, com um botão «+ Layer». Limite: `MAX_LAYERS = 32` porque o motor usa `Group` de 32 bits; é o teto do recurso.

## O desenho aprovado

Toda a tabela de destino está na maquete, secção «Mapa dos campos». Resumo:

- **Cartão Body:** Body. Em Dynamic: Gravity Scale, Rotation, Mass, Density ou Mass. Dobras: Start moving, Advanced,
  Joint, Bake to Timeline. Botão Remove Physics Body, com recibo e Undo.
- **Cartão Shape:** Shape, Width, Height (Ball: Radius; Capsule: Radius e Height), estado da forma, Touch, Bounce, Friction,
  Layer, Collides with (revoga a linha Hits), Edit shape on canvas. One-way à vista em Static. Dobras: Offset, Material rules, Surface, Signals.
- **Zona (Sensor):** On Hit, On Leave e Only for tag à vista; dobras Zone force e Bounce and Friction; sem One-way.
- **Forma sem corpo próprio:** cartão «Collision Shape», aviso inteiro e em destaque, Make Independent Body, dica de
  arrastar. Dentro de um corpo: «Part of: X». Density em dobra.
- **Density** some de corpo Static ou Kinematic e **o valor fica guardado**. Numa Zona Dynamic continua à vista.
- **Estado da forma:** só a caixa automática diz «Follows the picture»; as outras dizem «Custom size» com o botão Fit to picture, que mantém o tipo e ajusta uma vez.
- **Aviso de escala:** bola ou cápsula esticada avisa, com as saídas Set Scale to 1 e Use Box.
- **Camadas com nome:** o nome mora no projeto; o número continua a ser a identidade. «Collides with» lê e edita a regra do mundo.
- **Dica do Static Body:** «Set Use to Repeated and give it a Width».

⚠️ **Width e Height são só leitura e escrita de tela:** o `Collider` continua a guardar metades, e o ficheiro não muda.
Capsule: Height = 2 × (Half Height + Radius). Os ids `INSP_PHYS_*` ficam, para Auto Key, script e testes.

## Aceitação

- **Poder:** cada ajuste do mapa continua alcançável e com efeito; uma varredura dos ids prova que nenhum perdeu leitor.
  Novas: alças, nomes de camada, Fit to picture.
- **Facilidade:** à vista, Dynamic 12 de 35, Static 9 de 21, Zona 9 de 28, forma 7 de 19. Travar rotação e One-way: 1 clique.
  Digitar Width 200 dá 200 px.
- **Intuitividade:** o aviso cabe inteiro; a forma diz de quem é; um estado diz se segue a imagem; camadas têm nome.

## Ondas

| Onda | O quê | Estado |
|---|---|---|
| 1 | Os dois cartões, dobras, Width e Height, Touch, Density, nomes dos cartões, avisos, estado da forma, Fit to picture, recibo do Remove, dica | feita, à espera do smoke do dono |
| 2 | Camadas com nome e «Hits» (schema `217 → 218`) | feita, à espera do smoke do dono |
| 3 | Alças no canvas, no molde da câmera (`ph2d-app-game/src/smoke_mao.rs`, doc 17b), com undo e Auto Key | feita, à espera do smoke do dono |
| 4 | Curas da onda 3, cena de smoke `PH2D_PHYSICS_SMOKE=120`, uso no jogo, fotos | feita; smoke do dono 07/11: 12 de 13, passo 10 por repetir |
| 5 | 32 camadas: formato, canais, matriz com as visíveis, «+ Layer» (schema `218 → 219`) | feita, à espera do smoke do dono |
| 6 | Fila «Collides with» no objeto, «Edit matrix…», curas da 5 | feita, à espera do smoke do dono |
| 7 | Cartão próprio «Collision Layers», logo depois do Shape (decisão do dono) | feita, à espera do smoke do dono |
| depois | Polígono pelo contorno: item próprio | fila |

## ⛔ Recusas

- Juntar Mass e Density em um campo: perderia Density em Auto (crítica B; dono: não perder poder).
- Tirar Bake channel do cartão: fica em dobra.
- Um «More» único com tudo: 23 campos num saco; dobras com nome.
- Opção C e Feels like: custo maior, e os valores não foram medidos.
- Tirar Collision Shape do menu Add: mata a parede invisível.

## Provas

- `PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_FOTO=fisica_{dynamic,static,kinematic,sensor,bola,player,solta,peca,vazia,baixa,larga}`,
  com `docs/Components/ferramentas/fotografa_cena.sh` em 1930 × 2700.
- `cargo-test-narrow.sh ph2d-physics-ecs density_sem_efeito` · `ph2d-panel-registry-init colisao_segue`.
- **Onda 1:** `cargo-test-narrow.sh ph2d-panel-inspector o_cartao_da_fisica` (varredura do mapa por tipo, com as dobras
  abertas: gesto real → edição certa; Density, Fit, escala, recibo, 200 px) · `seam_physics` · `seam_px_m` ·
  `ph2d-app-physics fit_to_picture` · `density_fica` · `undoing_remove` (o Undo é real: a captura do undo repõe corpo e
  forma) · `canais`. Fotos: [`../handoffs/fotos_fisica/onda1/`](../handoffs/fotos_fisica/onda1/); `PH2D_FISICA_DOBRAS=1` abre as dobras.
- **Regras da onda 1:** «Follows the picture» = caixa do tamanho da `Sprite`, a mesma pergunta de `follow_auto_collider`
  (`Collider::follows_picture`); bola e cápsula são sempre «Custom size». O recibo cai ao mudar de objecto ou mover o objecto, e pela regra do topo da fila, abaixo.
- **Curas depois da onda 1 (07/11):** o recibo só vale enquanto o topo da fila de undo é a remoção, e a fila global é quem responde (`o_recibo_so_vale`; a shell publica em `post_frame_undo`). Fit to picture mantém o tipo: Box = a caixa da imagem; Ball = metade do menor lado; Capsule: radius = metade da largura, half_height = max(0, metade da altura − radius); offset 0 (`Collider::shape_fitted_to`, `fit_to_picture_mantem`). `Collider::follows_picture` é a mesma pergunta de `follow_auto_collider`, só de caixa. A regra anterior «troca bola ou cápsula por caixa» fica revogada. Fotos e `estado_do_player` moraram para `ph2d-physics-scene::fotos_dos_cartoes`; o teste de largura para `ph2d-panel-registry-init` (família não leva aresta de dev de folha).
- **Onda 2:** o nome mora em `ph2d_physics_ecs::LayerNames` (recurso do mundo, blob `PH2DLAYN` no fim do `ProjectFile`, vazio sem nomes; `Regras` leva-o, undo cobre). Bus `EditorAction::LayerRename`. Cartão Shape: lista suspensa `INSP_PHYS_LAYER_PICK` cujas opções são os `INSP_PHYS_LAYER` de sempre, «Hits» (`camadas_fisica::hits`), «Rename…» abre o campo ali mesmo. Painel Physics: um campo de nome por camada debaixo da matriz. Nav mostra os nomes. Testes: `layer_names` (ecs e shell: ida e volta + `um_v217`) · `as_regras_do_mundo` · `a_lista_da_camada` · `hits_le_a_matriz` · `rename_abre_o_campo` · `cada_camada_tem_o_seu_nome`. Fotos: [`../handoffs/fotos_fisica/onda2/`](../handoffs/fotos_fisica/onda2/): `fisica_{camadas,camada_sozinha,camada_com_pares,camadas_no_painel,player_cartoes,static}`.
- **Fica com o número, de propósito:** o campo Layer do Ray Sensor (índice 0–31, outra faixa) e os chips da face clássica (`PH2D_CARDS=0`).
- **Onda 3:** folha `crates/ph2d-shape-handles` (`ph2d-app-physics` estava a ~800 linhas do tecto): lei, ponteiro em ecrã, tinta e a mão das fotos; a shell só liga as portas (`forma_no_canvas.rs`). Estado no recurso `FormaNoCanvas`; o cartão lê `shape_edit_on`. Um arrasto = `PhysicsFieldEdit::ShapeDrag`, medidas e Offset num valor só (dois campos no mesmo quadro perdiam o primeiro: 17b A3), só para o objecto agarrado, sem fan-out. Durante o arrasto a família publica `MaosDasFamilias`: sem isso, com Auto Key, cada quadro era um passo da timeline e Ctrl+Z parava a meio (medido: `half_x 0,738`). A lei corre no referencial escalado `q = R⁻¹(p − T)`: centro = Offset × escala com sinal, meia-medida = medida × |escala|, a mesma conta do contorno e do solver. O mínimo é `MIN_EXTENT`, o de `Collider::sanitize`.
- **Regras da onda 3:** o Radius da cápsula mantém a Height e pára na bola (`Radius ≤ Half Height + Radius`); topo e base não mexem no Radius. Bola e Radius da cápsula são simétricos por natureza. O miolo só arma depois de 2 px. Ctrl+Z mantém a edição acesa (o undo pode refazer a entidade com outros bits: `StableId`).
- **Testes:** `ph2d-shape-handles` (18, inteira: cada alça em 5 poses × 3 zooms, pisos, Alt, Shift, Esc, dedo em ecrã, mão) · `ph2d-app-physics alcas` (funil real, vértice do contorno pintado, peça com pai, canais, Custom e Fit, motor: bola em 1,0 → 2,0 e 2,5 com espelho) · `ph2d-panel-inspector o_botao_` · `a_lista_da_camada`. **No app real:** `PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_FOTO=fisica_forma_{caixa,arrasto,bola,player,rodado,espelho,pivo,desfaz,chave,chave_desfaz,esc,esc_chave,alt,shift,desliga,peca,sensor,solta}`; o clique no botão, o ponteiro e o Esc vão pelas portas reais e `[forma-mao]` no log dá medida, `erro_px`, curvas e passos. Fotos: [`../handoffs/fotos_fisica/onda3/`](../handoffs/fotos_fisica/onda3/). Mutações que sangraram: sinal da compensação, rotação ignorada, Offset por `|escala|`, Esc sem devolver, Alt a mexer no Offset, Shift ignorado, ShapeDrag sem Offset, valor da opção da camada = 0.
- **Curas da onda 4 (07/11):** Esc a meio com Auto Key devolve forma, Offset, chaves e base, sem passo: `AutokeyState::desiste_do_gesto` + `TimelineHistory::desiste` (`esc_a_meio_do_arrasto`, visto vermelho; no app `forma_esc_chave`: k=8 curvas 4, k=20 curvas 0, passos 0). Clique parado no miolo volta à seleção, a mesma porta da câmera (`clique_para_a_selecao`; o clique no anel ou na imagem do próprio objecto fica com ele). Cursor por alça, na direcção em ecrã (`o_cursor_da_alca_segue_a_rodacao`; a ligação ao `set_cursor` não tem prova no app: a foto não mostra o cursor). A lista da camada marca a escolhida (foto; antes: onda 2). O contorno do overlay nunca foi tracejado: o tracejado de Sensor é o desta edição.
- **A prova no app real:** `bash docs/GameEngineFuncional/ferramentas/prova_alcas.sh` lê `[forma-mao]` e reprova com exit 1. Precisa de placa: a `App` segura uma surface, por isso não é `cargo test`; não há harness que dirija o gesto dentro da `App` em teste.
- **Cena 120** (`PH2D_PHYSICS_SMOKE=120`, corpos em `ph2d-physics-scene/src/ficha.rs`: `ph2d-app-physics` estava a 200 linhas do tecto): gates `ficha::` (Crate atravessa a forma solta e pousa; zona com controlo; One-way com controlo; matriz; Player atravessa o Crate). `PH2D_PHYSICS_SMOKE_{SELECT,TOCA,VISTA,PAINEL}` servem às fotos. **Uso no jogo:** `a_plataforma_alargada_por_width` (`ph2d-host-game-scenes`) e, no app, `PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_DRIVE=editor` (+ `fisica_larga`). Fotos: [`../handoffs/fotos_fisica/onda4/`](../handoffs/fotos_fisica/onda4/).
- **Cura depois da onda 4 (07/11):** a forma sem corpo acima não mostra «Hits» (nada bate nela); a lista Layer e o «Rename…» ficam (`a_forma_solta_nao_mostra_hits`, visto vermelho). Peça e corpo continuam com a linha.
- **Conduzido no app, cena 120** (`prova_alcas.sh`, casos `forma_ponta` e `forma_fit`, com `PH2D_PHYSICS_SMOKE_SELECT`; o campo Width recebe as teclas pelo `key_input`). Width 800: `half_x 4.0`, edição acesa; clique parado 15 px para dentro da alça direita, sobre as nuvens: `alvo=Updraft Zone editando=false`. Fit to picture no Player: `Capsule { half_height: 0.15, radius: 0.2 }` (20 e 70) → `{ 0.0, 0.5 }` (cartão: Radius 50, Height 100, Capsule) → Ctrl+Z → `{ 0.15, 0.2 }`, um passo só. Fotos: `onda4/prova_ponta_updraft_zone.png`, `onda4/prova_fit_player.png`; as da cena refeitas depois da cura.
- **Aberto:** as teclas da cena 120 não foram conduzidas no app, só o dedo direto nos gates.
- **Smoke do dono, 07/11, cena 120:** aprovados os passos 1–9 e 11–13. Passo 10 («Não encontrei Layers») sem aprovação: achado abaixo.
- **Decisão do dono, 07/11: a forma solta APARECE no canvas.** Revoga a lei «não é simulado, logo não se desenha». Traço tracejado (6 × 5 px de ecrã) e apagado, `ColorToken::Text3` do tema, a mesma geometria do solver (escala, rotação, Offset, pai), sem seta nem anel de corpo; o solver continua a não a ver. `overlay/outline.rs`: `LOOSE_INK` + `tinta_do_contorno`. Testes: `loose_tests.rs` (dois vistos vermelhos antes da cura; o da tinta e o Offset provados por mutação) e `a_collider_with_no_body_above_it_is_marked_loose_never_as_a_body`, a antiga `..._is_not_drawn`, vermelha pela cura e corrigida de propósito. No Play o Crate passa-lhe por dentro (`ficha::`). Fotos: `onda4/solta_sem_selecao.png`, `onda4/solta_play.png`.
- **Achado do passo 10, medido em 1930 × 1012 úteis, cartões de nascimento, HOME isolado** (o painel vê até y ≈ 1000; fotos `onda4/camada_tela_*.png` e, de corpo inteiro, `onda4/camada_inspector_inteiro.png`, janela de 2700):

| Objecto | Cartão Shape começa | Rótulo Layer em y | Rolagem até à lista | Na 1.ª tela? |
|---|---|---|---|---|
| Floor | ≈ 1030 | ≈ 1346 | ≈ 380 px | não |
| Crate | ≈ 1160 | ≈ 1456 | ≈ 490 px | não |
| Player | ≈ 2020 | ≈ 2349 | ≈ 1385 px | não |
| Updraft Zone | ≈ 978 (só o título) | ≈ 1357 | ≈ 390 px | não |
| Collision Shape | ≈ 530 | ≈ 898 | 0 | sim, e sem «Hits», por lei |

  A linha não está em dobra nenhuma e nenhuma dobra nasce fechada por acidente: só em rolagem, e o Image, Look e Body de cada objecto vêm antes. A barra de rolagem é fina, à direita. O rótulo é uma legenda pequena e apagada, igual a Bounce; o que se lê é o valor («Ground», «Props»): a palavra «Layer» quase não se vê. «Hits» é uma frase apagada logo debaixo da lista, por isso também fica abaixo da 1.ª tela. **Não há «Layers» no menu Window** (`MENU_BAR_WINDOW_ROWS`); o plural mora em Window ▸ Physics («Collision Layers», «Layers that collide») e no painel do Painter (modo Draw, imagem). **Causa provável:** na 1.ª tela de um objecto com corpo não há cartão Shape; o dono procurou a palavra «Layers», que ali não existe. **Não provado:** em que objecto ele estava e se rolou; o Width dos passos 3 e 8 não prova rolagem se foi na Collision Shape (Width em y ≈ 711), e a rolagem de roda não se conduz na tela virtual. **Proposta, não aplicada porque decide texto de produto:** rótulo «Collision Layer» na linha do cartão, a mesma expressão do painel Physics.
- **Onda 5, 32 camadas (07/11):** `LayerMatrix` em `[u32; 32]`. Schema `218 → 219`: `layer_matrix` muda no MEIO do fio; os `<= 218` leem-se sob `LayerRowsV218` (a mesma técnica de `WireLayout`). **Regra da migração** (`LayerMatrix::rows_from_legacy`): os pares `0..8` ficam; todo par com uma camada nova colide, porque é o valor de fábrica de uma camada; a matriz de fábrica antiga é a de fábrica nova. Os canais: o índice é o triângulo por `max(a, b)`, as 36 células ficam nos mesmos `i` (`as_36_celulas_antigas`). Os ids são tabela `const` gerada, iguais aos literais de antes.
- **Visíveis** (`LayerNames::visible`, `ph2d_physics_ecs::visible_layers`): a 0, as com nome, com objeto (`Collider`, `RaySensor`), com linha diferente do padrão, e as mostradas à mão (`LayerNames.1`, gravada no blob `PH2DLAYN` v2). Um projeto `<= 218` entra com as oito na `.1`: mostra as mesmas de sempre. Projeto novo: só a 0. Camada escondida volta assim que houver objeto. «Hits» e a lista Layer só citam as visíveis (e a do objeto).
- **A grelha** (`cards_linhas_triangulo.rs`): célula fixa em `GRID_CELL` (22 px; antes encolhia em painel estreito), colunas que não cabem rolam de uma em uma pela barra `PHYSICS_LAYER_SCROLL`, acima da grelha; rótulos à esquerda ficam, com número e nome; a linha sem célula à vista sai. A roda não rola as colunas. Face clássica (`PH2D_CARDS=0`): mostra as visíveis, só as colunas que cabem, **sem barra**: lá as colunas além da largura não se alcançam.
- **Medido (`--profile smoke`, sonda em `5653c7c372`, load ~5–9, mínimo de 200):** um quadro do funil (`publicar` + `aplicar`, 528 células) = **86 µs**, 0,5 % de 16,6 ms; só `publicar` dos canais: 36 = 0,9 µs, 528 = 17,4 µs. Memória: `PhysicsSettings` 172 B; store `72 B × 528 = 38 KB`; os três mapas `32 B × 528 × 3 ≈ 51 KB` antes do custo dos nós. Painel (`mede_o_painel_com_8_e_32`, mock): 8 visíveis = 12 ms, 32 = 25 ms por `assenta`. **Não medido:** o documento depois de um clique no losango, que grava as 528 curvas (antes 36).
- **Mesmos bits:** `sonda_bits_das_oito_camadas`, a mesma saída com o filtro inteiro e com o filtro cortado a 8 bits (a conta de antes), em 8 corridas nas camadas 0–7. Não é o binário antigo.
- **Ray Sensor:** limite 31 por `MAX_LAYERS - 1`; o campo continua por número, de propósito. Chips da face clássica: por número.
- **Testes:** `ph2d-physics`, `ph2d-physics-ecs`, `ph2d-panel-physics`, `ph2d-app-physics` e `ph2d-host-desktop --bins` inteiras · `a_celula_alta_20_25` (motor) · `o_clique_na_celula_alta` · `o_mais_layer` · `as_32_rolam` · `um_v218_abre` · `um_v219_guarda` · `a_forma_de_oito_nomes` · `visivel_e_nome`. **Mutações que sangraram:** `from_rows` a 8 bits, filtro a 8 bits, camadas novas sem colidir, índice triangular trocado, `next_free`, linha e objeto fora da regra, foco no campo errado. Fotos: [`../handoffs/fotos_fisica/onda5/`](../handoffs/fotos_fisica/onda5/), `PH2D_PHYSICS_SMOKE=120` + `PH2D_PHYSICS_SMOKE_PAINEL=1` + `PH2D_PHYSICS_SMOKE_CAMADAS=8|16|32` (e `PH2D_PHYSICS_SMOKE_ROLA=1`).
- **Para a onda 6:** visíveis = `ph2d_physics_ecs::visible_layers(world, &matriz)` (bits) e `LayerNames::of(world).label(i)`; editar um par = `LayerMatrix::set(a, b, on)` sobre o autorado, pelo mesmo caminho da célula (`PHYSICS_LAYER_CELL[physics_layer_cell_index(a, b)]`, evento em `ph2d-panel-physics/src/event.rs`, `PhysicsIntent::SetSettings`). No Inspector só há as linhas e as visíveis em `camadas_fisica` (a crate não vê a física), por isso a fila precisa de uma ação nova no bus para a edição.
- **Onda 6, «Collides with» (07/11):** cartão Shape, abaixo de Layer, no lugar da linha Hits, que saiu (`camadas_fisica::hits` e as chaves). `sections/physics_colide.rs`: um quadro por camada de `visivel(j) || j == camada`, ids `INSP_PHYS_COLLIDES` (tabela de 32, `tabela_por_camada`), quebra pelo nome, «+» no fim se houver camada livre, frase «Applies to everything on X», e na linha de baixo `[Rename…] [Edit matrix…]` meio a meio. Forma solta: só Layer e Rename.
- **A mesma edição:** bus `LayerPairToggle { a, b }` → `ph2d_panel_physics::alterna_par`, que é também o braço da célula. O mesmo `editar`, o mesmo `Toque`, o mesmo `set_settings`. Losango: `marcas_mundo::QUADROS` entra no grupo `CAMADAS` (`CAMPOS` continua um id por linha; `toda_linha_do_painel_tem_canal` lê-a) e o clique usa `canal_do_campo`.
- **«Edit matrix…»:** bus `LayerMatrixOpen { layer }` → `panel_bridge::acao_das_camadas` → `ph2d_panel_physics::abre_a_matriz`: painel visível, secção aberta, barra das colunas com a linha à vista, `state::destaque`. Durante 45 quadros o painel rola até a faixa ficar no terço de cima (a dobra abre em mola). O painel abre na aba dele, em cima do Inspector: «Inspector» no topo volta. O destaque cai ao fechar (`painel_fechado`, também chamado por `dispatch`).
- **Curas da onda 5:**
  - **a)** `SetSettings(Box<PhysicsSettings>, Box<Toque>)`, sem `expect`; `PhysicsIntent` perdeu o `Copy`.
  - **b)** Medido abaixo: não é mau, **sem cura**.
  - **c)** A face clássica encolhe: passo = `min(largura / colunas, 22 + vão)`, vão ≤ ¼ do passo, legendas uma em cada tantas (`a_face_classica_alcanca_as_32`: 528 células dentro do painel). Decisão: encolher, porque não pede estado novo numa saída de referência. Nas 32 a célula fica com cerca de 7 px.
  - **d)** Porta nova `interaction::roda_lateral`: a grelha publica a região ao pintar, `dispatch_wheel` anda a barra com Shift+roda ou roda horizontal; a roda simples rola o painel. Só com o painel dono por cima (`por_cima == painel`). Cartão solto (alfinete) não anda: `panel_at` devolve o id solto; lá fica a barra.
- **Medida b)** (sonda `mede_o_losango_da_matriz` no commit `18b89ec804`, saiu para pagar a catraca; `--profile smoke`, intercalado, mínimo de 200, load ~2–3; `apply_active_clip` + `publicar` + `aplicar`):

| Curvas | Blob da timeline | Salvar | Abrir | Clonar (passo de undo) | Um quadro | `marcas` por quadro | O clique |
|---|---|---|---|---|---|---|---|
| 1 | 72 B | 0,4 µs | 0,5 µs | 0,1 µs | 90,6 µs | 23,1 µs | 23 µs |
| 36 (antes) | 1 262 B | 4,0 µs | 6,2 µs | 2,2 µs | 87,8 µs | 24,9 µs | 80 µs |
| 528 (agora) | 19 193 B | 50 µs | 85,5 µs | 35,7 µs | 241 µs | 50,3 µs | 13,9 ms |

  Um clique no losango custa 19 KB no ficheiro e 13,9 ms uma vez (quadrático nas `AddKey`: 174 × para 14,7 × as curvas). Tocar a timeline com a matriz chaveada: 241 µs, 1,4 % de 16,6 ms, contra 88 µs. Só quem clica no losango paga. **Não medido:** o projeto inteiro em disco e o `autokey_pass` com as 528 curvas. Se o clique doer: chavear só as células que mudam, sem perder o poder.
- **Cena 120:** as camadas 4–7 ficam no padrão de fábrica (`POR_USAR`): antes a linha delas a zero as punha na lista e os quadros mostravam 4, 5, 6, 7 vazios. Fotos: [`../handoffs/fotos_fisica/onda6/`](../handoffs/fotos_fisica/onda6/). Conferidas: Floor, Crate, Player, Updraft Zone, Collision Shape e o painel do Edit matrix.
- **Conduzido no app:** `PH2D_PHYSICS_SMOKE_CLIQUE="quadro:0,desfaz,quadro:0,matriz"` + `PH2D_PHYSICS_SMOKE_SELECT=Crate`, janela de 2700 (`panel_bridge::passo_da_fila`, ponteiro real). `[fila-mao]`: linhas `[6,1,d,4]` → clique → `[2,1,c,4]` (os dois lados do par) → desfaz → `[6,1,d,4]` → clique → `[2,1,c,4]`. O clique na tela de 1400 não acha o quadro: fica abaixo da dobra.
- **Catracas pagas a mover:** o prólogo da cena (`smoke::aplica_o_pedido`, `resolve_os_nomes`) e as fotos da camada (`panel_bridge::arruma_a_foto`) saíram da shell (189 039 de 189 041). A lei do dedo (`player_input`, 244 linhas) desceu para `ph2d-physics-ecs`, com um invólucro de 10 linhas em `ph2d-app-physics`; a sonda saiu e o gate do motor foi para `ph2d-panel-physics` (55 511 de 55 515: quase no tecto).
- **Testes:** `ph2d-panel-inspector os_quadros_da_colisao` (7, clique real) · `ph2d-panel-physics o_par_alternado` · `o_quadro_decide_no_motor` · `edit_matrix_abre` · `a_face_classica` · `a_roda_de_lado` · `ph2d-app-physics quadros::` (Auto Key, losango, «+») · shell `o_quadro_da_fila_e_um_passo_de_desfazer`. **Mutações que sangraram (11 de 11):** um lado só, linha errada, par trocado, fila com todas, solta com fila, sem abrir painel (sobreviveu: o `MockPanelHost` força o painel visível ao pintar; o gate foi corrigido e sangrou), sem abrir secção, sem destaque, barra que não vai, Shift ignorado, clássica a cortar.
- **Limite das provas:** a mutação de quem pinta a caixinha (`Some(colide(camada, j))` em `colide_com`) não foi corrida: a dica usa a mesma função, mas o estado pintado só tem a foto como prova. A dica (`tooltip`) e o losango do cartão não foram vistos em uso no app: gates e fotos.
- **Onda 7, cartão «Collision Layers» (07/11):** `CARD_PHYS_LAYERS` com secção própria, em `paint_frame` (`cards::active() && has_collider`), depois do Shape; dobra, ordem, pega e alfinete vêm do `CardDef`. A face clássica (`PH2D_CARDS=0`) não muda: o ramo é só de cartões, e `physics_body.rs`/`physics_part.rs` continuam com a Layer deles. `camada`, a lista e o resumo passaram para `sections/physics_colide.rs`; o `cartao_de_fisica` é o cabeçalho e a dobra dos dois cartões. Nasce aberto por `set_collapsed_if_unchosen` em `prepara`; `PH2D_FISICA_CAMADAS_DOBRADAS=1` é a foto dobrada. `Remove` tira `RigidBody` e `Collider` (`physics_apply`), logo a face vazia não tem cartão, e o Undo devolve-o.
- **Resumo:** os quadros acesos, os mesmos da fila; sem par, `None`; sem nome, os números. **Medido:** a largura de fábrica do Inspector não leva `Props · Ground, Props, Zones` com o título «Collision Layers». O pintor do título é inteiro ou nada e o cartão ficaria mudo, por isso `curto_se_a_fila_nao_cabe` cai para o nome inteiro (`Props`). Cabem `Ground · Player, Props` e `Player · Ground`. Nunca corta a meio de uma palavra. A fila inteira do Crate só com o Inspector mais largo: prova em `a_fila_longa_cede_ao_nome_da_camada_sem_cortar`, sem foto.
- **Provas onda 7:** `ph2d-panel-inspector o_cartao_das_camadas` (9) · `ph2d-panel-registry-init collision_layers_solta` (solta e alfinete). **Conduzido no app:** `PH2D_PHYSICS_SMOKE_CLIQUE="quadro:0,desfaz,quadro:0,matriz"` com o Crate, janela 2700: `[fila-mao]` linhas `[fffffff6, fffffff1, fffffffd, fffffff4]` → clique → `[fffffff2, fffffff1, fffffffc, fffffff4]` → desfaz → a primeira → clique → a segunda; a foto `clique_matriz` mostra a matriz com a linha Props em destaque. Fotos: [`../handoffs/fotos_fisica/onda7/`](../handoffs/fotos_fisica/onda7/). **Mutações, 7 de 7 sangraram:** cartão nenhum (é o vermelho de antes do cartão), sem conferir o `Collider`, resumo que ignora a camada, forma solta com fila, fila longa sem ceder, resumo fora do título, nasce dobrado (na 1.ª corrida sobreviveu, porque a cena abria o cartão à mão; teste corrigido).
- **Não feito nesta onda: dois vermelhos de `ph2d-panel-physics` que já estavam no HEAD `c24f1d293b`** (conferido retirando o diff da onda 7): `quantas_entradas_tem_cada_painel::a_carga_de_comandos…` (`physics: 4 contra 3`) e `a_altura_de_abertura…` (`physics: 1636 px contra 1236`); e `no_magic_numeric` em `paint/matrix.rs:63` (`4.0`) e `paint.rs:41` (`3.0`). Vêm das ondas 5 e 6. Nenhum número subiu. O `+ Layer` é capacidade nova; a altura pede leitura: o gate manda semear recolhido e nunca subir o número.

## Onda 8, o modal «Collision Layers» (07/11)

**Decisão do dono, 07/11, com as palavras dele:** «Collision layers deve se transformar com tipo de modal central como é com o editor de animações do animated sprite. É o único jeito de aparecer a matriz inteira de 32 layers. O rename deve ser possível tanto com duplo clique no nome no painel inspector como também na label que fica ao lado esquerdo da matrix. Ao criar um layer o nome do layer deve seguir a formatação: Layer n (exemplo Layer 20). O aspecto visual do modal da matrix deve ser parecido com o painel de animação da sprite na versão atual da linha GameEngineFuncional.» Pedidos seguintes do dono (via coordenador): os nomes também no TOPO (rodados), destaque cruzado ao pairar (linha, coluna e os dois nomes, como a Unity), duplo clique no rótulo de cima, e a dica com os dois nomes por extenso, a caber na janela.

- **O modal:** `crates/ph2d-panel-physics/src/modal.rs`. Pinta-se no fim do `paint` do painel Physics (aberto ou fechado; `pinta_com_a_aba_atras` = aberto) pela `camada_das_listas::por_cima` (por cima de todo o chrome), com `widget::Modal` (novo `sem_rodape()`, sem os dois botões: cada clique já grava). As células são os MESMOS ids (`PHYSICS_LAYER_CELL`), o mesmo despacho, undo, Auto Key, arrasto (`toggle_paint`) e losango (no canto dos rótulos; a isenção `POR_FUNCAO` seguiu para `modal.rs::paint`). Rótulos: `PHYSICS_LAYER_ROW_LABEL` (esquerda) e `PHYSICS_LAYER_COL_LABEL` (em cima, `paint_text_rotated_ccw`). Passo da célula = o menor entre o que cabe e `ROW_H_PX`, piso `Spacing::Md`; numa janela de 1930 × 1040 cabem as 32 sem rolagem (foto). **Não feito:** rolagem dentro do modal com rótulos fixos para janelas abaixo de ~600 px de altura (abaixo do piso o cartão sai da janela).
- **Abre:** «Edit matrix…» do Inspector (`LayerMatrixOpen` → `abre_a_matriz` → `modal::abre(Some(camada))`, linha acesa; o painel já NÃO abre) e o «Edit matrix…» novo do cartão Collision Layers do painel (`PHYSICS_MATRIX_OPEN`, também na face `PH2D_CARDS=0`). **Fecha:** Esc (novo `dispatch::key::esc_fecha`: Esc sem campo de texto em foco clica o X do modal aberto), o X, e o clique fora (`PHYSICS_MATRIX_SCRIM`); o clique no cartão fora de controlo é engolido (`PHYSICS_MATRIX_CARD`).
- **O painel:** o cartão Collision Layers fica com o resumo («n layers shown · m pairs that do not collide») e «Edit matrix…». A grelha, o «+ Layer», a barra das colunas e os campos dos nomes saíram dele (`paint/matrix.rs` apagado; `traz_a_linha_a_vista` apagado). O «+ Layer» continua no cartão do Inspector (o «+» da fila); no modal não faz sentido: as 32 estão sempre à vista. A porta declara-se no MESMO composto das células (`modal::declara_a_porta`): é a face da matriz no painel, um valor, não um comando.
- **Os três vermelhos:** `a_carga_de_comandos…` 4 → **3** (o tecto); `a_altura_de_abertura…` 1 636 → **1 064** (catraca escrita 1 236 → 1 064); `no_magic_numeric` verde (as duas linhas saíram com o código que as tinha).
- **Nomes:** sem nome, «Layer n» (`card.physics.layer_default`) no modal, no Inspector (`camadas_fisica::rotulo`, o resumo, a lista, o placeholder) e nas dicas; nunca gravado (`LayerNames::to_bytes` do projeto sem nomes continua vazio). A dica da célula: «Props × Ground: click or drag to switch their collision».
- **Renomear:** modal: duplo clique no rótulo da esquerda ou de cima abre o campo sobre o rótulo da esquerda, com foco; Enter grava UM `LayerRename` (só se mudou), Esc desiste, vazio grava vazio (= «Layer n»). Inspector: duplo clique no chip da Layer abre o campo «Layer name» com foco (o mesmo do «Rename…», que FICA: é a porta para quem não sabe do duplo clique); escrever já não grava a cada tecla (`event.rs` consumia `TextChanged` com um `LayerRename` por tecla): Enter grava um passo, Esc desiste (`mark_cancel_on_escape`). **Não conduzido no app:** se o despacho promove o 2.º clique num `Dropdown` a `DoubleClick` (a prova é por evento, como nos testes do Flip).
- **Provas:** `ph2d-panel-physics o_modal_das_camadas` (8: abrir pelos dois sítios, destaque, célula (25, 20) com as duas metades CRUAS, Esc, X, fora, pairar e dica, duplo clique Enter/Esc/vazio nos dois rótulos, «Layer n» e sem bytes, face clássica, arrasto, losango) · `--test seam` (27; `every_matrix_cell_answers_a_real_click` agora nas 32) · `os_cartoes` (16) · `ph2d-panel-inspector o_cartao_das_camadas` (10, com `duplo_clique_no_chip_da_layer_renomeia_num_passo`) · `o_cartao_da_fisica` (19) · `seam_physics` (43) · `ph2d-panel-registry-init quantas_entradas_tem_cada_painel` (15) · `architecture_panel_wiring_parity` · `architecture_every_editable_field_is_a_channel` · `no_magic_numeric` · `workspace_file_loc_cap` · `no_dependency_climbs` · `the_shell_only_shrinks` · `every_word` · editor-core `modal` (32).
- **Mutações:** «uma metade só» em `LayerMatrix::set` (`ph2d-physics/src/world/layers.rs`) SOBREVIVEU à 1.ª versão (o `from_rows` faz o E e esconde a meia-escrita); com a asserção nas linhas cruas, sangrou («a metade espelho 20→25»). «Vazio fica vazio» (modal e Inspector): sangrou nos dois. «Esc que grava» (modal e Inspector): sangrou nos dois.
- **Fotos** ([`../handoffs/fotos_fisica/onda8/`](../handoffs/fotos_fisica/onda8/)), cena `PH2D_PHYSICS_SMOKE=120` com `PH2D_PHYSICS_SMOKE_MODAL=abre|nomes|nome|pairar`: `modal_abre` (linha Player acesa), `modal_nomes` (28 nomes compridos), `modal_nome` (edição da 20 aberta), `modal_pairar` ((25, 20) sob o ponteiro: coluna e os dois nomes acesos). Referência: `referencia_v2_anim_editor.png` (maquete da linha irmã). **Diferenças:** a referência é uma ABA do centro («Animations — Anim Strip» na barra de abas), de largura inteira, sem moldura de diálogo nem X próprio; o nosso é um diálogo flutuante com fundo escurecido, título e X, raio `Lg` e `BgElev`. Igual: fundo escuro plano, títulos `Md` em `Text1`, células com `Radius::Sm`. **Defeitos vistos na foto:** o campo de edição tem a largura do rótulo (o «Enemy» rola: «nemy»); a faixa da LINHA pairada fica quase toda sob as células (só o nome acende); a coluna lê-se bem.

### Onda 9a — a aba central «Collision Layers» (versão B) · 07/11

- **Decisão do dono, 07/11:** a **versão B** da crítica (`critica_ph2d.html#colisao`, «Lista, grade e camada aberta»), numa **aba central** ao lado de Scene com o aspeto da aba do editor de animações. A janela flutuante da onda 8 SAIU (`modal.rs` apagado; a grade e o duplo clique reaproveitam-se em `ph2d-panel-physics/src/aba/`). Não se perde configuração: as mesmas 528 células (só as visíveis à vista), os mesmos ids, undo, Auto Key, arrasto e losango.
- **O que há:** lista «Layers» (nome + quantos objetos, `ph2d_physics_ecs::objects_per_layer`: cada objeto UMA vez, o `Collider` manda), clique abre a camada, duplo clique renomeia ali (Enter/Esc/vazio = padrão); «+ Add» mostra a 1.ª camada escondida e abre-a; **Remove fica FORA** (é a 9b). Grade «Who bumps into whom · N pairs» com nomes nas duas bordas (duplo clique renomeia), faixas de linha e coluna de ponta a ponta (a da linha já não se esconde atrás das células), sem dica flutuante: frase FIXA no pé (par por nome + efeito; sem ponteiro, o resumo; camada nova, «bumps into everything»; só a 0, a frase do primeiro passo). Embaixo, só a linha de título da camada aberta («Player · 1 object» · «4 bump · 1 pass through»).
- **Nome padrão:** camada 0 sem nome = «Default», as outras «Layer n» (Inspector e aba); nada se grava.
- **Portas:** «Open in tab…» do cartão do Inspector (dica nova: «Open the Collision Layers tab at this layer.») abre já na camada do objeto; a porta do painel Physics abre-a também. Fecha pelo × da aba; voltar a Scene não a fecha.
- **Cura:** o campo de nome na grade tem a largura da lista («Enemy» já não vira «nemy»).

## Onda 9b (07/11) — camada aberta, «Remove», a aba como a maquete B

A aba ganhou «Bumps into» (a mesma edição da célula e da fila do Inspector), «Objects on this layer» com botões que selecionam, «Select all» e «+N more»; «Remove» (Default desligado; vazia sai; com objetos pergunta o destino; «Move and remove» / «Cancel»); cartões do tamanho do conteúdo, célula desligada com contorno, diagonal tracejada, par em negrito, pílula na linha, cor por camada. Detalhe, provas, mutações, fotos e diferenças que restam: [handoff, «Onda 9b»](../handoffs/HANDOFF_CONTINUACAO_2026-11-07_FISICA.md).
