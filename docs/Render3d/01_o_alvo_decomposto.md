# 01 — O alvo do dono, decomposto em ingredientes que se medem

> *«UM dos mais belos sistemas de shaders/render/iluminação que conheço é o do game mais recente da
> série Plants vs. Zombies: Battle for Neighborville. Desejo isso para essa game engine.»*

⚠️ **«Bonito» não é uma especificação.** Este documento parte a palavra em coisas que se constroem
uma a uma e se medem — porque *uma ambição que não se decompõe vira uma lista de features sem ordem*.

## §1 — ⭐⭐⭐ A tese: o que faz aquele jogo ser bonito NÃO é um shader

O BfN é **estilizado**: cores saturadas, formas simples, personagens de desenho animado. A tentação é
concluir *«então é um shader estilizado»*, e é aí que a maioria das engines amadoras pára — com
*cel-shading* e contornos pretos, que não é aquilo.

⭐ **O que ele de facto é: um pipeline fisicamente correcto, com a direcção de arte a mentir por cima
DE PROPÓSITO.** A luz obedece à física (energia conservada, indirecta a sério, exposição real); as
*cores* e as *formas* é que são inventadas. É por isso que a peça lê como **matéria** — folha,
plástico, metal pintado — em vez de ler como um desenho colorido.

⇒ **a ordem de construção sai daí**: primeiro a honestidade física, depois os botões para a trair.

## §2 — Os OITO ingredientes, e o que cada um vale

Ordenados por **salto visual por unidade de trabalho**, que é a ordem em que se constroem.

⛔⛔ **A última coluna é a FOTOGRAFIA de 2026-09-09, e não o estado de hoje** — ela fica **verbatim**
porque cada célula traz a medição que abriu o estudo. *O estado de HOJE está logo abaixo da tabela*,
e ⚠️ **ela já mentiu uma vez com o cabeçalho antigo** (*«temos hoje?»*): em 17/09 o `2` e o `6`
fecharam no mesmo dia e a linha de fecho desta secção ainda dizia que o `2` não existia.

| # | ingrediente | o que muda na tela | tínhamos em **2026-09-09**? |
|---|---|---|---|
| 1 | **Gestão de cor** — espaço linear, exposição, tonemapper (AgX/ACES) | ⭐⭐⭐ a diferença entre «parece de 2005» e «parece moderno» | ⚠️ **meio**: há `tonemap.wgsl` e `bloom.wgsl` no `ph2d-render`, sem exposição autorada nem espaço de trabalho declarado — ⛔ *remedido 13/09: o passe está em **bypass** (corta em 1) e o bake AgX nunca existiu, logo é **zero** ([`04`](04_a_remedicao_contra_a_arvore.md) §1)* |
| 2 | **Luz indirecta (GI)** | ⭐⭐⭐ o maior contribuinte isolado; sem ela, sombra é preto e a peça flutua | ⛔ **não** — há um `env_ambient` constante no `ph2d-light` |
| 3 | **Resposta de material que lê como MATÉRIA** (GGX + energia conservada + metalness) | ⭐⭐ um plástico deixa de ser «brilhante» e passa a ser plástico | ⛔ **não** — o `mesh.wgsl` faz matcap ou um rig analítico simples |
| 4 | **Céu/ambiente como FONTE de luz** (IBL) | ⭐⭐ é o que põe cor no lado escuro sem o lavar | ⛔ não |
| 5 | **Sombras com contacto correcto** (macias ao longe, duras no contacto) | ⭐⭐ é o que **pousa** o objecto no chão | ⛔ não (há SSAO, que é outra coisa e não substitui) |
| 6 | **Sub-superfície** — a folha translúcida com o sol atrás | ⭐⭐ é literalmente a assinatura do PvZ (plantas) | ✅ **SIM, no modelador desde 17/09** ([`10`](10_a_luz_que_atravessa_a_peca.md)) — os DOIS caminhos, com a curvatura tirada do CAMPO. ⭐ E o `sss.rs` do esculpir integra **o mesmo integral**: a nota *«parcial»* estava certa e a medição mostrou-a mais forte |
| 7 | **Pós** (bloom sobre HDR real, DOF, AA) | ⭐ acabamento; sem o `1`, o bloom mente | ⚠️ meio (bloom existe) |
| 8 | **A camada de ESTILO** — mentir com botões: rim light, tinta por curvatura, grade por zona | ⭐⭐ é o que faz *aquele* jogo e não «um jogo PBR» | ⛔ não |

### ⭐ O ESTADO DE HOJE (2026-09-19), auditado contra o código

| # | ingrediente | hoje | onde fechou |
|---|---|---|---|
| 1 | Gestão de cor | ✅ | `W1` ([`03`](03_o_plano.md)) — a `ph2d-view-transform`, com a `Neutral` medida contra o OCIO |
| 2 | Luz indirecta | ✅ | `W5` — **SONDAS** de irradiância, e o dispositivo a `100,000 %` de paridade ([`08` §12, §14](08_a_luz_indirecta.md)) |
| 3 | Material como MATÉRIA | ✅ no subconjunto declarado | `W2` — a `ph2d-material`; ⛔ ficam `transmission_*`, `fuzz_*`, `thin_film_*`, `geometry_opacity` e a anisotropia |
| 4 | Céu como FONTE | ✅ | `W3` — o estúdio analítico; ⛔ sem HDRI de ficheiro, **por desenho** |
| 5 | Sombras que POUSAM | ✅ | `W4` — o chão invisível ([`07`](07_o_chao_que_so_recebe.md)) |
| 6 | Sub-superfície | ✅ com **dívida nomeada** | 17/09 ([`10`](10_a_luz_que_atravessa_a_peca.md)); a borda mole só corre no caminho de REFERÊNCIA ⇒ `W10` |
| 7 | Pós | ⛔ | `W7`, por construir |
| 8 | Estilo | ✅ | `W8` — a [`11`](11_a_camada_de_estilo.md), 19/09, por ordem do dono |

⭐⭐ **SETE de oito.** O que falta ao ALVO é **o `7`**, e o dono já o pôs a seguir (*«8 e depois do
smoke o 7»*, 19/09); o resto da fila é **autoria** (`W6`), **medição** (`W9`) e **uma dívida**
(`W10`).

⚠️ **E a linha do `8` mudou no MESMO dia em que esta tabela foi auditada** — a auditoria de 19/09
escreveu-a `⛔` e o dono mandou construí-la a seguir. *Uma tabela de estado envelhece em horas quando
alguém lê a fila que ela descreve.*

⛔ **E a frase que aqui esteve — *«o `6` já existe e o `2` não, e isso é a ordem invertida»* — MORREU
em 2026-09-17**, quando a `W5` fechou o `2`. Ela foi verdade por **um dia**: *a inversão que ela
denunciava era real e foi desfeita pela wave seguinte, e a nota sobreviveu-lhe.*

## §3 — O que este repositório JÁ tem, medido

| peça | tamanho | o que faz |
|---|---:|---|
| `ph2d-mesh-render` | **6 106** LOC | pipeline `wgpu`, câmera, `mesh.wgsl` (**972** linhas), SSAO, SSS |
| `ph2d-light` | **1 069** LOC | o rig do artista: lâmpadas, `env_ambient` constante |
| `ph2d-render` | — | `tonemap.wgsl`, `bloom.wgsl`, compositor, `frost.wgsl` |
| shaders WGSL na árvore | **20** | todos escritos à mão |
| ⭐ `ph2d-field-eval` | — | **a cena como campo de distância** — ver `02` §5 |

O `mesh.wgsl` de hoje escolhe entre **matcap** (sombreamento função-da-normal, com as lâmpadas
cravadas na textura) e um **rig analítico** de poucas lâmpadas. ⚠️ *É um bom sistema de escultura e
não é um sistema de render de jogo* — e o doc dele já diz isso por extenso.

## §4 — ⛔ As três armadilhas que este alvo costuma provocar

1. **Começar pelo estilo.** Cel-shading e contornos sobre um pipeline sem GI e sem gestão de cor dão
   um resultado que parece um protótipo. *O estilo é a camada `8`, e ela precisa das sete de baixo.*
2. **Confundir SSAO com sombra de contacto.** O `ph2d-mesh-render` já tem SSAO, e ele escurece
   cantos — **não** põe o objecto no chão. São ingredientes diferentes (`5` contra parte do `2`).
3. **Perseguir o Nanite.** Geometria virtualizada resolve um problema que nós não temos: as nossas
   peças nascem de **campos** e de escultura, e a densidade é nossa por construção.
