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

## O desenho aprovado

Toda a tabela de destino está na maquete, secção «Mapa dos campos». Resumo:

- **Cartão Body:** Body. Em Dynamic: Gravity Scale, Rotation, Mass, Density ou Mass. Dobras: Start moving, Advanced,
  Joint, Bake to Timeline. Botão Remove Physics Body, com recibo e Undo.
- **Cartão Shape:** Shape, Width, Height (Ball: Radius; Capsule: Radius e Height), estado da forma, Touch, Bounce, Friction,
  Layer, Hits, Edit shape on canvas. One-way à vista em Static. Dobras: Offset, Material rules, Surface, Signals.
- **Zona (Sensor):** On Hit, On Leave e Only for tag à vista; dobras Zone force e Bounce and Friction; sem One-way.
- **Forma sem corpo próprio:** cartão «Collision Shape», aviso inteiro e em destaque, Make Independent Body, dica de
  arrastar. Dentro de um corpo: «Part of: X». Density em dobra.
- **Density** some de corpo Static ou Kinematic e **o valor fica guardado**. Numa Zona Dynamic continua à vista.
- **Estado da forma:** só a caixa automática diz «Follows the picture»; as outras dizem «Custom size» com o botão Fit to picture, que mantém o tipo e ajusta uma vez.
- **Aviso de escala:** bola ou cápsula esticada avisa, com as saídas Set Scale to 1 e Use Box.
- **Camadas com nome:** o nome mora no projeto; o número continua a ser a identidade. «Hits» lê a regra do mundo.
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
| 3 | Alças no canvas, no molde da câmera (`ph2d-app-game/src/smoke_mao.rs`, doc 17b), com undo e Auto Key | feita, à espera do smoke do dono; aberto: Esc a meio com Auto Key deixa chaves no valor de partida |
| 4 | Cena de smoke do item + uso no jogo, fotos, smoke do dono | a fazer |
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
- **Aberto na onda 3:** Esc a meio com Auto Key devolve a forma, mas ficam chaves no valor de partida (`forma_esc_chave`: curvas 4, passos 0); falta um cancelar da fila da timeline que também acerte o baseline do Auto Key. Sem cursor próprio sobre as alças (a alça cresce sob o ponteiro). Clique parado no miolo fica com o objecto (a câmera o devolve à seleção, 17b A1); clicar em quem está atrás da forma pede clicar fora primeiro. O contorno do overlay nunca foi tracejado: o tracejado de Sensor é o desta edição.
- A preencher: smoke do dono.
