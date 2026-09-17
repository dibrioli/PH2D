# Render 3D — shading, luz e materiais

> **Pedido do Enio, 2026-09-09:** *«Faça um estudo/pesquisa de como podemos chegar ao estado da arte.
> Talvez até superar a Unreal. Descubra o suprasumo do render/shaders/light para games e descubra
> como podemos alcançá-lo. Queremos um sistema intuitivo para artistas, de fácil uso mas grande
> poder.»*

⚠️ **Isto é um ESTUDO. Nada aqui está no produto** — o que ele entrega é a decisão de partida com a
medição ao lado, para a implementação não começar por um palpite.

| Leia isto | quando a sua pergunta for |
|---|---|
| [`00_a_triagem_e_a_porta_aberta.md`](00_a_triagem_e_a_porta_aberta.md) | *«temos de inventar o modelo de material?»* — ⭐ **NÃO**, e a razão é medida |
| [`01_o_alvo_decomposto.md`](01_o_alvo_decomposto.md) | *«o que faz o PvZ:BfN ser bonito?»* — os oito ingredientes, separados |
| [`02_o_estado_da_arte.md`](02_o_estado_da_arte.md) | *«o que a Unreal tem, e onde podemos ganhar?»* |
| [`03_o_plano.md`](03_o_plano.md) | *«por onde se começa, e o que se mede em cada passo?»* |
| ⭐ [`04_a_remedicao_contra_a_arvore.md`](04_a_remedicao_contra_a_arvore.md) | *«o que o estudo dizia que JÁ existia ainda é verdade?»* — ⛔ **três premissas caíram em 13/09** (o tonemap está em bypass · a `W1` sozinha não tem consumidor · **há** gerador de WGSL), e ⭐ **a ponte para WGSL está MEDIDA** |
| ⭐⭐⭐ [`07_o_chao_que_so_recebe.md`](07_o_chao_que_so_recebe.md) | *«porque é que a peça flutua?»* — ⭐ **a `W4` FECHA (16/09)**: o chão INVISÍVEL (ordem do dono), a sombra e o contacto que ele recebe, e a lei do céu do chão que **não** são os cones da peça (eles desenham anéis num plano) |
| ⭐⭐⭐ [`05_o_modo_render_do_modelador.md`](05_o_modo_render_do_modelador.md) | **A 1.ª FATIA QUE SHIPA (13/09)** — o modo *Render* no modelador: o OpenPBR como lei de referência em CPU (port Apache-2.0, provado contra o MaterialX renderizado sem interface), o céu e o rig traduzidos com o `π` e o sinal de `y` gateados, e a exposição e a vista da cena. Traz as barras de cada gate, a tabela que justifica a vista existir, as recusas desta fatia e a armadilha do CORREDOR que fez 8 gates reprovarem sob `cargo test` |

## O resultado do estudo em três linhas

1. ⭐⭐⭐ **O modelo de material não se inventa: ele está neste disco, é o padrão da indústria, e é
   permissivo.** O `MaterialX 1.39.5` (**Apache-2.0**) traz o **OpenPBR Surface** — que É o
   *Principled BSDF* que o dono apontou, na versão que Blender, Autodesk, Adobe e a Academia
   assinaram — e **gera o código do shader sozinho**: 2 177 linhas de GLSL, por script, medido.
2. ⭐⭐ **O que faz o alvo do dono ser bonito não é um shader — é o pipeline ser fisicamente honesto
   para a direcção de arte poder depois mentir de propósito.** Metade do salto visual vem de duas
   coisas baratas que ainda não temos: **gestão de cor a sério** e **luz indirecta**.
3. ⭐⭐⭐ **Onde podemos genuinamente bater a Unreal:** o problema mais difícil do Lumen é ter uma
   representação da cena rápida de percorrer — a Unreal **aproxima** a malha por um campo de
   distância. **O nosso modelador JÁ É um campo de distância.** *A nossa esquisitice é o substrato
   deles.*
