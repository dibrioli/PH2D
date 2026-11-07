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
- **Estado da forma:** «Follows the picture», ou «Custom size» com o botão Fit to picture.
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
| 1 | Os dois cartões, dobras, Width e Height, Touch, Density, nomes dos cartões, avisos, estado da forma, Fit to picture, recibo do Remove, dica | a fazer |
| 2 | Camadas com nome e «Hits» (mexe no formato salvo: `python3 scripts/schema-recount.py`) | a fazer |
| 3 | Alças no canvas, no molde da câmera (`ph2d-app-game/src/smoke_mao.rs`, doc 17b), com undo e Auto Key | a fazer |
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
- `cargo-test-narrow.sh ph2d-physics-ecs density_sem_efeito` · `ph2d-app-sprite-anim colisao_segue`.
- A preencher por onda: testes que dirigem o clique real (`seam_physics.rs`), smoke do dono.
