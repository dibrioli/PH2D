# 30 — Plano: CAMADAS e EFEITOS do Painter direto na peça (etapa 4)

> **Ordem:** o dono, 02/10 (*«camadas e efeitos do Painter direto na peça»*), depois de a etapa 3b
> fechar ([29](29_plano_o_relevo_do_impasto_na_peca.md) §9). Este doc é o PLANO — nenhuma linha de
> produto foi escrita. As decisões de PRODUTO pedidas ao dono estão na §8; o resto é técnico e
> delegado ao padrão-ouro.

## 1. O estado da arte (pesquisa de 02/10, fontes no fim)

| app | camada | onde vivem os dados | efeitos | o que abandonou / onde falha |
|---|---|---|---|---|
| **Nomad Sculpt** (a referência de UX) | pilha por objecto; cada camada leva sculpt (aditivo) + cor/rugosidade/metal | vértices | só destrutivos (fill) | *uma opacidade só para os três canais* — o manual admite que fundir muda a imagem; voxel remesh degrada as camadas |
| **Substance 3D Painter** | pilha por Texture Set, todos os canais, modo + opacidade POR CANAL; **fill + máscara** | UV/UDIM | filtros, geradores (AO, curvatura) | geradores liam só o *bake* e ignoravam o relevo pintado ⇒ **anchor points** (2017); blur **sempre deixa costura** (filtra em 2D) |
| **Blender** 5.x | não há camadas nativas | — | — | *Layered Textures* (2022) abandonado; #155954 (03/2026): *«filtering on arbitrary geometry domains is hard»* ⇒ filtros só com bake; camadas de sculpt adiadas para não acoplar ao Multires |
| **ZBrush** | 3D Layers (forma + polypaint) | vértices | — | presa à topologia/nível; modo Record faz exportar ver UMA camada |
| **3DCoat** | opacidade separada para Depth/Color/Gloss/Metal; relevo com Add/Replace | pixel/Ptex/vértice | — | vertex paint descarta atributos ao subdividir |
| **Mari** | pilha POR CANAL; ajuste sobre tudo abaixo ou preso a uma camada | Ptex/UDIM | adjustment layer/stack, «bake» | Ptex: faces sem coerência espacial |
| **Procreate 3D** | Color/Rough/Metal por camada | UV | HSB, ruído, halftone | **Gaussian Blur não funciona em 3D** |

**Lições que este plano adopta:**

1. Uma camada carrega os canais dela com **modo e opacidade próprios** — e o relevo NÃO usa os modos
   de cor: **Add / Level** com profundidade com sinal (exactamente o que o Painter 2D já tem).
2. **Efeito ponto a ponto** (HSB, curvas, levels, gradient map…) = camada de ajuste VIVA, uma conta
   por amostra. **Efeito de vizinhança** (blur, sharpen, bloom) é onde todos falharam: o mesh colors
   tem a vizinhança sem costura de graça, logo filtra-se **na retícula, com raio em unidades do
   MUNDO**, nunca num atlas 2D.
3. As camadas **não se prendem a uma topologia**: toda mudança de malha passa por UMA porta que
   reamostra todas as camadas.
4. Nada de um sistema paralelo de camadas: o artista já conhece o painel de Layers do Painter.
5. ⛔ O que não fazer: opacidade única para canais distintos (Nomad); filtro que assume imagem 2D
   (Substance/Procreate); exportar/mascarar a ver UMA camada (ZBrush Record); relevo que entra e não
   se edita (Procreate).

## 2. O desenho — e a porta ÚNICA de cada pergunta

O achado que decide tudo (levantamento de 02/10): no Painter 2D a **pilha** (`LayerStack`: modo,
opacidade, máscara, recorte, grupos, ajustes, `impasto_depth`, `impasto_composite`) é METADADO puro e
os **pixels** vivem à parte (`BTreeMap<LayerId, …>`); o compositor (`compositor::composite_region`)
lê os pixels por uma interface (`LayerPixelSource::layer_rgba`) e a composição ponto a ponto **não
depende da forma da imagem**. Um plano de tinta é uma lista de amostras ⇒ é uma imagem.

| pergunta | a porta única | consequência |
|---|---|---|
| que camadas a peça tem, em que ordem, com que modo/opacidade/máscara/recorte? | **o MESMO `ph2d_tool_painter::LayerStack`** | o painel de Layers lê-o como já lê (`set_current_layers`) |
| que cor tem a amostra `i` de uma camada? | `PilhaDaPeca` (nova, `ph2d-sculpt3d`): `BTreeMap<LayerId, PlanoDaCamada>` com `rgba8` sRGB direito (4 B/amostra, a precisão das camadas do Painter) | implementa `LayerPixelSource` com o plano DOBRADO em linhas de `1 024` |
| que cor tem a peça? | **`ph2d_tool_painter::composite_region`** sobre a pilha | o resultado vai para o plano `Tinta` de hoje ⇒ placa, shader, bake e doação **sem mudança** |
| que relevo tem a peça? | a dobra do 2D (`ReliefFields::height_at`: `own·depth`, `Add`/`Level` pela cobertura, teto suave) **extraída para uma função pura** que o 2D e a peça chamam | o relevo composto vai para o canal `[altura, corpo]` de hoje ⇒ inclinações (§9 do 29) sem mudança |
| que vizinhos tem uma amostra (efeitos espaciais)? | um gancho de vizinhança no compositor: a grelha 2D (o comportamento de hoje, ao bit) e a **retícula** da peça (`vizinhanca.rs`, raio no mundo) | sem o gancho, um blur na peça borraria pela ORDEM das amostras — o gate de §5 reprova isso |
| onde pousa o traço? | a `tela` pousa no plano da **camada activa** (com alfa) | a lei `nova = base + k·(c − s)` (etapa 2) lê a camada activa |
| que operação de camada mexe em amostras (nova, apagar, duplicar, fundir abaixo, limpar)? | UMA porta em `PilhaDaPeca` que muda pilha e planos juntos | gate: todo `Raster` da pilha tem plano e todo plano tem `Raster` |
| a malha mudou de topologia? | a porta de reamostragem que hoje refaz o plano (`tinta_da_peca::garante`) passa a refazer **cada** camada | camada nenhuma fica presa à malha de antes |

⚠️ **Por que NÃO um sistema próprio de camadas na escultura:** seria a segunda resposta a «o que é
uma camada» (22 modos, recorte, grupos, 20+ ajustes) — duas portas que divergem em silêncio, e o
artista a aprender duas pilhas. ⚠️ **Por que RGBA8 e não `f32`:** é a precisão das camadas do Painter
(a que o compositor lê) e custa `4×` menos (tabela §6); a composição corre em `f32` linear como no 2D.

## 3. Contratos congelados e schema — a prova por busca

```
LayerPixelSource · LayerStack · composite · Tinta · TintaDoc · SCULPT_DOC_VERSION · Inclinacoes
  → 0 ocorrências em crates/ph2d-editor-core/src/tool.rs (Tool=12, PanelEvent=4)
  → 0 ocorrências em crates/ph2d-nodegraph/src/node.rs
```

O painel de Layers fala pelo `PanelEvent` que já existe (`ToolPanelEvent` → `handle_panel_event`):
**nenhuma variante nova**. O formato muda **dentro do blob da escultura** (`SCULPT_DOC_VERSION` 5 → 6,
com degrau: um plano `v5` abre como UMA camada opaca «Camada 1»), sem tocar no `PROJECT_SCHEMA` — o
precedente do `TimelineDoc` e dos degraus v4/v5. O `ph2d-tool-painter` não é congelado: o gancho de
vizinhança e a função da dobra do relevo são aditivos e gateados ao bit contra o 2D de hoje.

## 4. A UI — as quatro condições, uma a uma

| condição | estado | o que falta |
|---|---|---|
| o componente EXISTE | ✅ `ph2d-panel-painter-layers` | — |
| é pintado e registrado | ✅ (2D) | nada de novo: a peça publica a pilha dela pela mesma `set_current_layers` |
| o clique chega ao barramento | ✅ `ToolPanelEvent` → `PainterTool::handle_panel_event` | com o Painter na peça, as operações que mexem em amostras desviam para a porta da `PilhaDaPeca` |
| a SEQUÊNCIA leva a algum lugar | ⛔ hoje não há pilha na peça | o teste de costura (`ph2d-ui-testkit`): «nova camada → pintar → baixar opacidade → `Ctrl+Z`» muda a COR DA PEÇA e volta ao bit |

E dois estados que o painel tem de DIZER (nada de botão mudo): fora do Painter-na-peça a pilha da peça
não aparece; um efeito espacial antes da W6 aparece **desligado com a frase do porquê**.

## 5. Gates (red-first) e as fixturas que contêm o fenómeno

| gate | fixtura | o que reprova |
|---|---|---|
| a composição da peça É a do Painter, ao bit | um plano `N` amostras e a MESMA imagem 2D `1024×⌈N/1024⌉` | uma segunda redacção da lei |
| um `v5` abre igual (dentro de um degrau de sRGB8) | um documento `v5` com relevo | a migração |
| a dobra do relevo é UMA (2D e peça, ao bit) | pilha com `Add`, `Level` e profundidade negativa | duas dobras |
| pilha ↔ planos em sincronia | cada operação de camada + desfazer | camada sem plano (pintar nela perde o traço) |
| pintar escreve SÓ a camada activa | duas camadas, traço na de cima, `Ctrl+Z` | o traço a cair no composto |
| blur na peça é na SUPERFÍCIE | uma esfera com uma linha fina pintada: o blur alarga-a no mundo; CONTROLO: borrar pela ordem das amostras espalha cor para faces longe | o atlas implícito |
| o raio é do MUNDO | a mesma linha em dois degraus (`8x`, `32x`): a largura borrada no mundo é a mesma | o raio em amostras |
| reamostrar leva todas as camadas | remesh com 3 camadas | camada presa à malha antiga |
| o incremental compõe o que a peça inteira compõe | traço por pedaços contra composição do zero | faixas esquecidas |

Mutação: um arnês por porta (composição, dobra, sincronia, vizinhança), com o pré-voo de âncoras.

## 6. Medições já feitas (antes de construir)

Sonda `diag_o_compositor_do_painter_sobre_o_plano_da_peca` (perfil `smoke`): o compositor de hoje,
SEM uma linha nova, sobre planos do tamanho da peça de fábrica, 3 camadas (Normal, Multiply, Overlay)
+ um ajuste HSB:

Medido em 02/10, `load 3,8`:

| degrau | amostras | a peça inteira `N×1` (uma thread) | DOBRADA `1024×H` (o compositor reparte as linhas) | uma faixa de `1 024` amostras |
|---|---|---|---|---|
| `8x` | 47 k | `5,9 ms` | `5,6 ms` | `0,114 ms` |
| `16x` | 188 k | `21 ms` | `3,2 ms` | `0,114 ms` |
| `32x` | 754 k | `85 ms` | `7,8 ms` | `0,114 ms` |
| `64x` | 3,0 M | `346 ms` | **`33,5 ms`** | `0,114 ms` |

⇒ **o traço é barato em todo degrau** (um quadro suja `137–323` amostras em poucas corridas:
`< 0,05 ms`); **compor a peça inteira a `64x` passa de um quadro** — e é isso que um arrastar de
opacidade de uma camada faz em cada passo. A CPU fica como a REFERÊNCIA (a resposta igual), e o teto
é o da placa: ver a **W1b** na §7.

Memória por camada (RGBA8): `8x` `0,19 MB` · `16x` `0,75 MB` · `32x` `3,0 MB` · `64x` `12,1 MB`.
O tecto de camadas do Painter (`HARD_CAP_LAYERS = 999`) é de outro recurso (o arena); a peça herda-o
e o orçamento real é a RAM: ⏳ medido na W1 com o histórico de desfazer.

## 7. As ondas e os critérios de desistência

| onda | entrega | desiste-se desta forma se… |
|---|---|---|
| **W1** a pilha | `PilhaDaPeca` + `LayerPixelSource` + composição (CPU, dobrada) para o `Tinta` + documento `v6` com degrau | — (a CPU é a referência; o teto já está medido na §6) |
| **W1b** compor na placa | as camadas dobradas sobem como texturas `1024×H` e o compositor de GPU do Painter (`ph2d-render::layer_compositor`) compõe-nas; o `tinta.wgsl` lê o composto pelo índice da amostra | a paridade placa/CPU não fechar ao bit-de-sRGB8, ou compor a `64x` na placa passar de `4 ms` |
| **W2** pintar na camada | a `tela` pousa na camada activa com alfa; modos que lêem por baixo lêem a activa; desfazer por camada | o incremental (faixas sujas) passar de `1 ms` por quadro a `8x` |
| **W3** o painel | o painel de Layers mostra a pilha da peça; a porta das operações; desfazer das operações | — |
| **W4** relevo por camada | a dobra única; `Add`/`Level`/profundidade por camada | — |
| **W5** ajustes ponto a ponto | todos os não-espaciais, vivos | — |
| **W6** efeitos de vizinhança | o gancho no compositor; blur/sharpen/bloom/sombras-luzes na retícula, raio no mundo | blur de raio razoável a `32x` passar de `100 ms` (vira «aplicar», não vivo) |
| **W7** remesh e fecho | reamostrar todas as camadas; cena de smoke; arneses | — |

## 8. Perguntas de PRODUTO ao dono — RESPONDIDAS em 02/10

> ✅ **Decisões do dono (02/10), as quatro recomendadas:** (1) o painel de **Layers do Painter**
> mostra as camadas da peça; (2) cada camada leva **cor + relevo** (rugosidade/brilho por camada ficam
> para depois); (3) os efeitos de vizinhança entram **VIVOS**, como no 2D (W6); (4) exporta-se e
> doa-se o **COMPOSTO**. O plano das §2–§7 é este; nada muda de forma.


1. O painel de **Layers** do Painter passa a mostrar as camadas da PEÇA quando se pinta nela
   (recomendado), ou um painel próprio da escultura?
2. Uma camada leva **cor + relevo** (o que a peça já tem). Rugosidade e metal por camada ficam para
   depois (a peça hoje não tem material por amostra)?
3. Os efeitos de **vizinhança** (desfoque, nitidez, brilho) entram vivos, como no 2D (recomendado, W6),
   ou só como «aplicar» destrutivo?
4. O que se exporta / doa ao 2D é sempre o **composto** (recomendado — como hoje)?

## 9. Fontes

Nomad: nomadsculpt.com/manual/layers · /painting · /topology · Substance: experienceleague.adobe.com
(layer-stack, baking, sparse-virtual-textures) · magazine.substance3d.com (anchor points) · Blender:
projects.blender.org/blender/blender/issues/155954 · code.blender.org/2022/02/layered-textures-design ·
devtalk.blender.org (2025-1-14 sculpt-paint meeting) · ZBrush: help.maxon.net (3D layers) · 3DCoat:
3dcoat.com/documentation (blending panel, polypaint) · Mari: learn.foundry.com (adjustments, ptex) ·
Procreate: help.procreate.com (3d-painting/layers) · Yuksel 2010, *Mesh Colors*.

## 10. W1 — a pilha (02/10)

**Entregue:** `ph2d_app_sculpt3d::pilha_da_peca` (`PilhaDaPeca` · `PlanoDaCamada` · `achata`) e o
documento **v6** (`doc_camadas.rs`). Gates em `pilha_da_peca_tests.rs` e `doc_camadas_tests.rs`.

| porta | onde |
|---|---|
| a pilha + os planos, e o invariante | `PilhaDaPeca` · `sincronizada()` |
| as operações que mexem em amostras | `nova_camada` · `nova_mascara` · `novo_ajuste` · `duplica` · `apaga` (+ `define_*`, metadado) |
| a cor da peça | `compor()` / `compor_faixa(a, b)` = `ph2d_tool_painter::composite_region` sobre a dobra `1024 × ⌈N/1024⌉`, ao bit |
| a peça composta no `Tinta` | `pinta_tinta(tinta, fundo)` → `achata` (opaco = `byte/255` exacto; senão mistura em luz sobre o fundo) |
| o fundo por baixo da pilha | `tinta_da_peca::semente` (a cor por vértice — a mesma porta de onde um plano nasce) |
| «este ajuste lê a disposição da imagem?» | `AdjustmentKind::reads_the_image_layout` (`ph2d-painter-effects`) — o compositor 2D passou a ler a mesma pergunta |

**Premissas que o código derrubou (técnicas, delegadas):**

1. **A `PilhaDaPeca` mora em `ph2d-app-sculpt3d`, não em `ph2d-sculpt3d`**: ela implementa
   `LayerPixelSource` e guarda um `LayerStack`, e a `ph2d-sculpt3d` (o núcleo da escultura) não
   depende da `ph2d-tool-painter` — puxar a ferramenta 2D inteira para o núcleo é a direcção errada.
2. **Até à W2 a peça NÃO segura uma pilha em memória.** Seis sítios escrevem o `Tinta` hoje
   (`tinta_fina`, a tela do Painter, `preenche`, `tela_semente`, `uniformiza`, o desfazer): uma pilha
   guardada ao lado ficaria velha no 1.º traço e o `Ctrl+S` gravaria a velha. ⇒ o `encode` deriva a
   pilha do plano (`PilhaDaPeca::de_tinta`: UMA camada opaca, `nome_da_base()`, chave `app.sculpt3d.pilha_da_peca.camada_de_base`) e o `decode` compõe a
   pilha gravada no `Tinta`. A W2 põe a pilha na `SceneObject` **e** redirecciona os seis escritores
   — as duas coisas no mesmo passo.
3. **A cor atravessa o ficheiro a ≤ ½ degrau de sRGB8, não ao bit** (era `f32` até ao v5). É a
   precisão das camadas que a §2 escolheu; o relevo continua `f32` ao bit. Os gates
   `o_plano_de_tinta_fina_atravessa_o_ficheiro_a_meio_degrau` e
   `um_v5_abre_igual_e_regrava_a_meio_degrau` dizem-no; um v5 ABRE ao bit (a leitura dele não passa
   pelas camadas) e é a regravação que quantiza.
4. **O fundo**: onde a pilha não é opaca a peça mostra a SEMENTE (gate
   `uma_pilha_transparente_assenta_na_semente`) — nem branco nem preto.
5. **`Noise` entra** (hash por píxel, não lê vizinhos); **`Halftone` fica de fora** com os seis
   kernels (padrão de coordenada); **camadas `Texture` ficam de fora** (textura 2D sobre a ORDEM das
   amostras). As três recusas estão em `sincronizada()`.
6. **O relevo grava-se por camada**, mas até à W4 só a BASE o tem (`relevo_composto` com
   `debug_assert`): a dobra de várias camadas é a do 2D extraída, e é a W4.
7. O documento v6 **guarda `niveis`** (vazio = uniforme) como o v5: um plano graduado só nasce de um
   ficheiro antigo, mas o escritor aceita-o e o leitor converte-o como antes.
8. A máscara nova é branca só nas `N` amostras — a cauda da dobra fica a zero, senão o ficheiro não
   voltava igual (achado pelo gate `a_pilha_atravessa_o_ficheiro`).

**Para a W2:** o traço acumula em `f32` dentro do gesto e quantiza à camada NO FIM (o que o 2D faz):
quantizar a cada quadro pararia um traço de `k` pequeno (`base + k·(c − s)` abaixo de meio degrau
não muda o byte). As operações da porta estão em `#[cfg(test)]` até a W3 lhes dar o painel.

## 11. W2 — pintar na camada activa (03/10)

**O desenho (o que o código derrubou do §2):** a `tela` NÃO pousa na camada directamente — todo
escritor de cor (o dab de tinta fina, os dois verbos do anel, a tela do Painter com as suas duas
misturas, o balde, a cadeia molhada) passa pelo `TintaDoTraco`, que congela a `base` de cada amostra
no 1.º toque e escreve `f(base)`. ⇒ **o traço passa a pintar uma CÓPIA DE TRABALHO da camada
activa**: um `Tinta` com o canal de opacidade novo (`ph2d_mesh_colors::alfa`, `None` = opaco), cor
**pré-multiplicada**. Com cor pré-multiplicada toda mistura «por cima» desta casa
(`antes·(1 − w) + alvo·w`) vale igual nas quatro componentes, e sobre `alfa = 1` é a de antes AO BIT
(gate `numa_camada_opaca_o_traco_e_o_de_hoje_ao_bit`, os três verbos de cor). O plano da peça (o
composto) FICA na peça durante o traço: é ele que a placa lê.

| porta | onde |
|---|---|
| o canal de opacidade | `ph2d_mesh_colors::alfa` (`tem_alfa`, `opacidade`, `define_opacidade` — recusa transparência solta num plano opaco —, `opacidade_tri/quad`) |
| as leis com opacidade | `tinta_fina` (pintura: alvo `1`; anel: a média pré-multiplicada, 4.ª componente) · `tela_na_malha_pousa::pousa` (`Sobre` nas 4; `Diferenca` na cor DIREITA, opacidade intacta) · `preenche_plano` · `tela_semente::semente` (divide pela opacidade interpolada e escreve-a no alfa) |
| byte ↔ trabalho | `pilha_da_peca_traco::{de_bytes, para_bytes}` — a ida e volta é a identidade nos 65 280 píxeis com opacidade (gate exaustivo) |
| pen-down | `tinta_da_peca::pilha::empresta_da_peca` → `PilhaDaPeca::trabalho_da_activa` (regista `em_traco`) |
| cada quadro | `pilha::desce_do_traco`: `drena_sujas` → `recebe_do_traco` (RGBA8; o relevo desce à BASE) → `compoe_amostras` (corridas de índices consecutivos) → sobe o plano da peça |
| pen-up | `pilha::devolve_camada` (última descida, cor por vértice da peça) |
| desfazer | `JanelaFina::do_traco_na_camada` (os bytes de antes saem da `base` `f32` por `para_bytes` — exactos) · `troca_na_peca` · `PlanoInteiro::da_camada` (o balde) |
| a pilha anda com o plano | `pilha::garante_com_pilha` sobre `garante_e_diz` (`Desfecho { origem: Parque · Semente · Nenhum, estacionou }`) · `pilha::acompanha` — e o plano é RECOMPOSTO da pilha quando ela nasce: **a peça é sempre a composição da pilha** (invariante dos gates do produto) |
| o ficheiro | `doc_camadas::encode` (o escritor único: a pilha da peça) · o `decode` v6 instala a pilha lida |

**Medido** (sonda `diag_o_preco_do_traco_na_camada`, perfil `smoke`, `load 3,7–4,7`, peça da lição,
3 camadas + HSB, a de cima activa):

| degrau | amostras | pen-down (cópia de trabalho) | um quadro: 300 sujas em 10 corridas, mediana · pior |
|---|---|---|---|
| `8x` | 47 k | `0,30 ms` | `0,038` · `0,045 ms` |
| `16x` | 188 k | `1,25 ms` | `0,038` · `0,050 ms` |
| `32x` | 754 k | `5,1 ms` | `0,038` · `0,053 ms` |
| `64x` | 3,0 M | **`22,7 ms`** | `0,038` · `0,058 ms` |

⇒ o critério de desistência da W2 (*o incremental passar de `1 ms` por quadro a `8x`*) passa **26×**,
e o quadro não depende do degrau. ⏳ **O preço novo é o pen-down a `64x`** (um quadro): a cura,
se o dono a sentir, é guardar a cópia de trabalho entre traços (invalidada pelo desfazer, pelo balde e
por mudar de activa) — `16 B`/amostra persistentes (`48 MB` a `64x`).

**Gates** (W2): núcleo `tinta_fina_camada_tests` (4) · `alfa_tests` (1) · `pilha_da_peca_traco_tests`
(4: ida e volta ao byte, a descida só à activa, a recomposição incremental = a inteira ao bit, as
trocas do desfazer são involuções) · produto, com placa: `camadas::pintar_escreve_so_a_camada_activa_e_o_desfazer_a_tira`
(base intacta ao byte, a de cima com opacidade, a peça = a composição em todo o momento, `Ctrl+Z` →
transparente, refazer → os mesmos bytes, gravar → a mesma pilha) · `camadas::na_peca_de_uma_camada_o_traco_pinta_a_base`.
Os 42 gates de produto da tinta fina (Painter, aquarela que escorre, balde, desfazer) passam.

**Fica para depois (nomeado):**
- A activa que não é raster (ajuste, grupo, máscara) — o pen-down não empresta a cópia e o traço pinta
  só a cor por vértice; o balde recusa com a frase no terminal. Só é alcançável com o painel (W3), que
  tem de DIZER porquê.
- Remesh / mudança de topologia: o plano novo nasce da semente e a pilha nasce com UMA camada (o
  detalhe das camadas perde-se como o do plano se perdia) — reamostrar cada camada é a W7.
- O relevo continua da peça (mora na BASE) até à W4.

## 12. W3 — o painel de Layers sobre a pilha da peça (03/10)

**O desenho (o que o código derrubou do §4):** o painel não podia «ler a pilha da peça pela mesma
`set_current_layers`» e seguir o barramento até ao `PainterTool` — com a tela da vista presa, a pilha
da FERRAMENTA é a da tela (o traço em voo, uma camada só), e os gestos do painel mexiam nela em
silêncio. ⇒ a ferramenta guarda um **espelho** da pilha da peça, publicado pela escultura a cada
quadro, e o painel mostra o espelho; cada gesto vira um **pedido** que a escultura drena e aplica pela
porta da `PilhaDaPeca`. Contratos congelados intocados: nenhuma variante de `PanelEvent`/`Tool`.

| porta | onde |
|---|---|
| a leitura ÚNICA do fio do painel (ids por camada, cargas `"camada:canal:…"`) | `ph2d_tool_painter::tool::layer_edit::{decode, LayerEdit, ParamEdit}` — a pilha 2D (`apply_layer_edit`) e a da peça leem-na |
| o espelho e os pedidos | `tool::piece_layers`: `sync_piece_layers` · `panel_layers` / `panel_selection` / `panel_shows_the_piece` · `take_piece_layer_ops` → `PieceLayerOp { NewLayer, NewMask, NewAdjustment, Duplicate, Delete, Metadata { stack, gesture } }` · `piece_layer_refusal` |
| o metadado calculado com as leis do 2D | `LayerStack::set_*` + `apply_param_edit` (as funções puras dos ajustes; as curvas passaram a `set/add/remove_curve_point_in`, que os métodos 2D também chamam) · o arrasto pela lei única `reparent_in` |
| a porta da pilha (sem `#[cfg(test)]`) | `nova_camada` · `nova_mascara` · `novo_ajuste` (semente `seed_user_adjustment`, a activa fica) · `duplica` (só pintura, sem relevo) · `apaga` (devolve os planos; a BASE fica) · `troca_metadado` (recusa estrutura e base fora do fundo) · todas recusam com traço aberto |
| o desfazer estrutural | `TrocaDaPilha` (metadado de antes + planos tirados) · `troca_estrutura` (instala e devolve a inversa) · `StrokeUndo::Camadas { level, passo }` com a cerca `IdDoPlano` |
| a escultura a cada quadro | `painter_na_malha::camadas`: `camadas_do_painel` (pedidos → porta → UMA recomposição → espelho) · `aplica_pedidos_da_pilha` · `a_activa_recusa_o_traco` |
| o painel | `ph2d-panel-painter-layers::peca`: `offered` (barra), `adjustment_offered` (o menu de ajustes APAGA os de vizinhança — `DropdownOption::disabled`, novo, no `ph2d-editor-core`), `paint_piece_notes` (as frases do fundo, no lugar do Apply) |

**Premissas que o código derrubou (técnicas, delegadas):**

1. **Um arrasto é UM passo de desfazer**: o `PanelEvent::SetValue` não tem «largou», e um passo por
   quadro do arrasto enchia a fila. O pedido leva o id do controlo (`gesture`), e a escultura junta-o
   ao passo anterior enquanto nada mais mexeu na peça (`camadas_arrasto` = `(id, edits)` e o topo da
   fila é um `Camadas`).
2. **A base fica no fundo e não se apaga**: até à W4 o relevo da peça mora na camada de BASE
   (`relevo_composto`); apagá-la ou subi-la levava o relevo com ela. Duplicar a base copia a cor e não
   o relevo (duas camadas com relevo partiriam o `debug_assert` da W1).
3. **A máscara nova não rouba a activa** (no 2D rouba, para se pintar): pintar uma máscara na peça
   ainda não existe — a activa continua a dona, e escolher a linha da máscara dá a frase.
4. **A activa que não é de pintura** (ajuste, máscara): o pen-down do Painter RECUSA e a frase vai ao
   painel (`piece_layer_refusal`); antes, o traço pintava só a cor por vértice, por baixo da
   composição.
5. **A pilha nasce quando o painel a mostra** (`acompanha` no quadro), não só no pen-down: o artista
   vê «Layer 1» antes do primeiro traço, e o «+» já tem onde cair.

**Desligado na peça, com a frase do porquê** (`panel.painter_layers.piece.off_here`): grupos, camadas
de textura, Lock, Ref, o relevo por camada (W4), a vista em cinzento e o Apply da máscara, o Apply do
documento (a peça É a composição), e os ajustes que leem a vizinhança (W6) — apagados no menu.

**Gates** (W3): ferramenta `piece_layers_tests` (4: os gestos vão ao espelho e a pilha da tela não
muda, com o controlo do 2D · o arrasto de opacidade · o parâmetro de um ajuste é o do 2D ao bit · o
que a peça não oferece não pede nada) · porta `pilha_da_peca_porta_tests` (4: cada passo desfaz e
refaz ao bit · o metadado recusa estrutura, base e traço aberto · a base fica e a cópia não leva o
relevo · o ajuste novo nasce como o do 2D) · produto, com placa e o painel REAL
(`tinta_no_produto_tests::painel`): **«nova camada → pintar → baixar a opacidade → `Ctrl+Z`» muda a
cor da peça e volta ao bit**, os dois `Ctrl+Z` seguintes até à peça de antes e o `Ctrl+Shift+Z` dos
três · a activa que é um ajuste recusa o traço e o painel diz porquê (controlo: a base escolhida, o
traço pousa).

**Medido** (03/10, perfil `smoke`, `load 3,3`, peça da lição, 3 camadas + HSB, 20 passos por
degrau). Sondas: `sonda_camadas::diag_o_preco_de_arrastar_a_opacidade` (CPU: porta + peça inteira
recomposta + cor por vértice) e `tinta_no_produto_tests::painel::diag_o_preco_de_arrastar_a_opacidade_de_ponta_a_ponta`
(o caminho do produto: `aplica_pedidos_da_pilha` e a subida do plano por `sync_mesh`):

| degrau | amostras | um passo, CPU (sonda pura) mediana · pior | um passo, produto: porta+recompor · subir o plano (medianas) |
|---|---|---|---|
| `8x` | 47 k | `2,95` · `3,36 ms` | `2,44 ms` · `0,10 ms` |
| `16x` | 188 k | `2,28` · `4,29 ms` | `2,47 ms` · `0,24 ms` |
| `32x` | 754 k | `8,08` · `9,51 ms` | `7,52 ms` · `0,66 ms` |
| `64x` | 3,0 M | **`37,8` · `39,3 ms`** | **`36,4 ms` · `2,85 ms`** |

⇒ até `32x` um passo do arrasto cabe num quadro; **a `64x` são `~39 ms` (dois quadros e meio)**, e
quase tudo é recompor a peça inteira na CPU (a subida à placa é `7 %`). Pelo critério do §7 a
**W1b (compor na placa) passa a ser a onda seguinte** — o número que ela tem de bater: o passo a
`64x` abaixo de `4 ms` (o critério de desistência dela) com a paridade ao bit-de-sRGB8.

**Fica para depois (nomeado):**
- Um arrasto é UM desfazer, e dois arrastos SEGUIDOS no mesmo controlo, sem nada entre eles, também
  (o `PanelEvent` não tem «largou»; ver premissa 1).
- Pintar uma MÁSCARA na peça (a activa máscara recusa o traço; a máscara nasce branca e inverte-se).
- O pincel de pintura da PRÓPRIA escultura com a activa não-raster ainda pinta só a cor por vértice
  (herdado da W2): o painel só é visível no Painter; a recusa do pen-down cobre o Painter.
- Grupos, camadas de textura, Lock e Ref na peça; o relevo por camada é a W4; os ajustes de
  vizinhança a W6.

## 13. W1b — compor na placa: a 1.ª tentativa, MEDIDA e PARADA no critério (03/10)

**O que existe (commit da medição, nada ligado ao produto):** o tradutor pilha → operações do
compositor de GPU saiu de `ph2d-app-painter` para a crate nova `ph2d-painter-layer-ops`
(`flatten_for_gpu`; o Painter reexporta-o em `painter_gpu_flatten`; ⚠️ casa final na §13.1) — o `ph2d-render` só conhece a
`LayerStack` nos testes e uma crate de app não depende de outra. `composto_na_placa::CompostoNaPlaca`
compõe a `PilhaDaPeca` pelo `LayerCompositor` do Painter; cada `PlanoDaCamada` leva `NaPlaca`
(versão única no processo + linhas sujas da dobra desde a última subida; todo escritor de `rgba8`
marca-a). Gates/sondas em `composto_na_placa_no_produto_tests.rs` (`tinta_no_produto_tests::placa::`).

**O preço — passa o critério com folga** (`diag_o_preco_de_compor_na_placa`, perfil `smoke`, `load 3,4`,
peça da lição, 3 camadas + HSB, 20 passos; o passo = metadado pela porta + composição + `poll` à espera):

| degrau | amostras | 1.ª composição (sobe as camadas) | um passo do arrasto, mediana · pior | CPU (§12) |
|---|---|---|---|---|
| `8x` | 47 k | `0,35 ms` | `0,047` · `0,091 ms` | `2,44 ms` |
| `16x` | 188 k | `0,46 ms` | `0,054` · `0,071 ms` | `2,47 ms` |
| `32x` | 754 k | `1,11 ms` | `0,091` · `0,113 ms` | `7,52 ms` |
| `64x` | 3,0 M | `3,70 ms` | **`0,248` · `0,744 ms`** | `36,4 ms` |

**A paridade — NÃO fecha ao bit** (`a_placa_compoe_a_pilha_rica_como_a_cpu`, VERMELHO de propósito):
na pilha rica a `8x`, **23 de 188 424 bytes** diferem da CPU, todos por **1**. A ablação
(`diag_que_ingrediente_difere_da_cpu`, base + UM ingrediente):

| | nada · normal 1,0 · screen · softlight · HSB · invert · curves | normal 0,55 | multiply 0,7 | multiply + máscara | overlay recortado | color 0,8 |
|---|---|---|---|---|---|---|
| base opaca | `0` | `0` | `0` | `1` | `0` | `0` |
| base `0,85` | `0` | `6` | `3` | `1` | `5` | `2` |

⇒ **não é `pow`/`sqrt`** (HSB, SoftLight, Curves dão zero): é a **divisão** — `premul / ao` no
`over` e `(0,299·r + …) / 255` na máscara (`layer_composite.wgsl`), as mesmas fórmulas da CPU, mas
o `/` do WGSL tem `2,5 ULP` e o da CPU é arredondado exacto; um valor na fronteira de um degrau sai
`±1`. É o contrato que o Painter 2D já vive (`layer_compositor/mod.rs`: *«the GPU↔CPU parity gate
asserts agreement within ±1 byte»*). Uma 2.ª tentativa ao bit pediria divisão correctamente
arredondada no shader com `fma` exacto, que o WGSL não garante (nem impede a contracção) — dependente
do driver.

**Achado do caminho, ANTERIOR à W1b** (`diag_uma_pilha_translucida_recomposta_anda`): com a pilha
TRANSLÚCIDA (base a `0,5`, pintada) cada `recompoe` sem nada mudar ANDA a cor — `50`, `37`, `27`
degraus de sRGB8. O fundo é a `semente`, que lê a cor por vértice, e a `recompoe` reescreve a cor
por vértice com o composto: um laço. Cada passo de arrasto, desfazer do painel ou balde empurra-a.

Parou-se aqui pelo critério do §7. ✅ **Decisão do dono (03/10): aceita-se o degrau** — a peça mostra
a cor da placa enquanto se mexe; o que se grava, exporta e doa continua calculado pela CPU, exacto.

### 13.1 W1b entregue (03/10)

**O desenho (o que o código derrubou do §7):** o `tinta.wgsl` NÃO mudou — ele continua a ler o plano
`[f32; 3]` por amostra; quem muda é QUEM ESCREVE esse plano na placa. Um passe de compute
(`ph2d-mesh-render` `tinta_achata.rs` + `shaders/tinta_achata.wgsl`) achata o composto do compositor
do Painter no buffer das amostras do slot, com a lei do `achata` (opaco = `byte/255` pela tabela da
CPU, ao bit; translúcido em luz sobre o fundo, que chega linear da CPU).

| porta | onde |
|---|---|
| a pilha composta e achatada na placa | `composto_na_placa::CompostosDaCena::compoe_e_achata` (um `LayerCompositor` por peça; o fundo sobe uma vez por pilha e degrau) |
| quando | `slots::sync_mesh` (agora recebe o `GpuContext`): `SceneObject::compor_na_placa`, ou o plano acabou de subir INTEIRO com a CPU atrasada |
| a recomposição do painel, do balde e dos desfazeres deles | `tinta_da_peca::pilha::recompoe` (+ a cor por vértice) / `recompoe_o_plano` (o desfazer do balde, que repõe a cor por vértice ele mesmo) — na CPU só o prefixo dos vértices (`PilhaDaPeca::por_vertice`, ao bit) |
| o plano da CPU ATRASADO | `PilhaDaPeca::atrasada` (sessão); `em_dia` (CPU inteira); `tinta_da_peca::pilha::para_ler` (a peça inteira para quem a lê na CPU — o `assa` da exportação/doação) |
| a recusa | a placa não exprime a pilha (os seis ajustes sem código de GPU) ou o slot não tem o plano ⇒ a CPU compõe e sobe como antes |
| o tradutor pilha → operações | `ph2d_tool_painter::flatten_for_gpu` (`compositor/gpu_ops.rs`, ao lado da `LayerStack`; 2 consumidores) sobre o vocabulário da folha nova `ph2d-layer-ops` (`LayerOp`, `LayerMask`, saídos do `ph2d-render`, que os reexporta). ⚠️ A 1.ª casa (uma crate `ph2d-painter-layer-ops`) subia uma camada — folha → ferramenta, gate `architecture_no_dependency_climbs_a_layer` — e uma família não depende de outra |
| o FUNDO fixo | `pilha_da_peca_fundo`: a cor por vértice no nascimento da pilha; documento **v7** (`CamadasDoc::fundo`; v6 congelado em `doc_migracao`, abre com a cor por vértice gravada) |
| a cor por vértice sem o plano | `SceneObject::cores_sujas` → `upload_region_at` com todos os vértices |

**Premissas que o código derrubou:** (1) o `tinta.wgsl` não lê o composto — o plano da placa é
escrito por compute, o shader fica intocado; (2) quase todos os leitores do plano na CPU leem só o
PREFIXO dos vértices (`devolve`, `desparqueia`, `devolve_camada`) ou o comprimento (o ficheiro, que
grava a pilha e não o composto) — mantê-lo em dia custa `0,04 ms`; o único leitor da peça inteira é o
`assa`; (3) o RELEVO não passa pela recomposição do painel (só a base o leva até à W4); (4) o
desfazer do balde não pode escrever a cor por vértice (ele repõe-na ao bit).

**Medido** (perfil `smoke`, `load 3,5`, as réguas do §12):

| degrau | um passo do arrasto: porta+recompor (CPU) · `sync_mesh` (placa) | W3 (CPU) |
|---|---|---|
| `16x` | `0,04` · `0,09 ms` | `2,47` · `0,24 ms` |
| `32x` | `0,04` · `0,11 ms` | `7,52` · `0,66 ms` |
| `64x` | **`0,04` · `0,27 ms`** (pior `0,06` · `0,45`) | `36,4` · `2,85 ms` |

⇒ o passo a `64x` cai de `~39 ms` para **`~0,3 ms`** (`~125×`), `13×` abaixo do critério. A 1.ª
composição de uma pilha sobe as camadas (`3,7 ms` a `64x`, uma vez; depois só as linhas sujas).

**Gates** (W1b): sem placa `composto_na_placa_tests` (2: todo escritor muda a versão e marca as
linhas, o metadado não · a cor por vértice é o prefixo da peça ao bit, translúcida) ·
`uma_pilha_translucida_recomposta_fica` · `um_v6_abre_com_o_fundo_da_cor_por_vertice` · a forma
gravada re-pinada (`83`/`3 590`, a conta fecha à mão); com placa `placa::` (3: a pilha rica a um
degrau, `FRACCAO_A_UM_DEGRAU` · o painel muda a pilha e a placa tem a peça, + a subida inteira com a
CPU atrasada · a pilha que a placa recusa compõe na CPU) · `ph2d-mesh-render`
`a_grade_cobre_cada_amostra_uma_vez`. Mutação: `docs/3D/ferramentas/muta_a_pilha_na_placa.sh`,
**18/18** — a 1.ª corrida deixou 3 vivas: o gate da deriva comparava recomposições ENTRE SI (desde a
cura a cor por vértice sai do fundo fixo, e a deriva já não aparecia lá — agora compara com a peça de
ANTES da 1.ª recomposição); a grade do despacho só tem 2.ª linha acima de `16,7 M` amostras (virou
função pura com gate de tecto pequeno); e «outra dobra, outro compositor» era uma 2.ª resposta ao
`ensure_array` do compositor, que já reconstrói as fatias — saiu.

### 13.2 Os degraus de topo (`128x`, `256x`)

O chip `Paint Detail` vai até `256x` (`NIVEL_MAX = 8`), e a dobra de `1 024` de largura dava `11 776` e
`47 104` linhas — acima do lado de textura que toda placa garante (`8 192`): o compositor recusava e a
peça caía EM SILÊNCIO na CPU. ⇒ `dobra(n)` alarga em potências de 2 só quando a altura passaria de
`ALTURA_MAX_DA_DOBRA` (`1 024` até `64x`; `2 048` a `128x`; `8 192` a `256x`); cada plano guarda a sua
largura. Gates `a_dobra_cabe_na_textura_garantida_e_e_a_mais_estreita` e a paridade com placa a `128x`.

| degrau | amostras | antes (CPU, a placa recusava) | depois: `sync_mesh` mediana · pior |
|---|---|---|---|
| `128x` | 12,1 M | `153 ms` | `0,82` · `2,24 ms` |
| `256x` | 48,2 M | `646 ms` | `5,7` · `12,6 ms` |

⚠️ A `256x` são `~2 GB` de memória movidos por passo (compor + achatar `48 M` amostras): é o tecto
da placa, dentro de um quadro a `60 Hz`. ⏳ **E o tecto de camadas do compositor**: o orçamento de
cache é `1 GiB` numa placa discreta (`ph2d-render` `layer_cache_budget`, a política do Painter 2D) e
uma camada a `256x` são `193 MB` ⇒ **5** camadas (máscaras contam); com mais, a placa recusa e a peça
compõe na CPU (`~650 ms`). A `128x` cabem `21`. Subir o orçamento para a peça é decisão de memória
de placa, medida quando o dono a pedir.

**Fica para depois (nomeado):** o traço continua a compor as amostras sujas na CPU (`0,04 ms`, sem
razão para a placa); a pilha translúcida num traço semeia o fundo inteiro em cada quadro (`fundo_semeado`
na `compoe_amostras` — o caso da base a menos de `100 %`; cachear o fundo por amostra é a cura se o
dono o sentir).

## 14. W6 — os efeitos de vizinhança na peça (03/10)

**O desenho (o que o código derrubou do §2):** o «gancho» não é uma lista de vizinhos — é o
**passa-baixo**. Cada efeito de vizinhança (Gaussiano, Nitidez, Bloom, Sombras/Realces) é uma lei
escrita UMA vez (a fronteira pré-multiplicada em tons de ecrã, a máscara de nitidez, o passo claro e a
soma em luz do brilho, a correcção de tom local) por cima de UMA pergunta: *«um Gaussiano de
σ = raio/3 sobre os vizinhos»*. Duas respostas:

| porta | onde |
|---|---|
| a pergunta | `ph2d_painter_effects::Neighbourhood` (`blur4` · `blur1` · `glow` · `window` · `image_plane`) e o despacho único `apply_adjustment_on`; `apply_adjustment_windowed` é ele com a grelha |
| a grelha 2D | `impl Neighbourhood for AdjustWindow` — os kernels de sempre (`separable_blur_premul`, `separable_blur_scalar`, a pirâmide do brilho `grid_glow`): **ao bit** |
| a superfície | `ph2d_mesh_colors::difusao::Difusao` (a lei) embrulhada em `ph2d_app_sculpt3d::vizinhanca_da_peca::VizinhancaDaPeca` |
| σ | `gaussian_sigma(raio) = raio/3` — UMA conta para a grelha, a superfície e a placa (cópia da placa gateada em `spatial_weights_parity`) |
| o compositor sobre uma superfície | `ph2d_tool_painter::composite_over` (a pilha inteira, a superfície como vizinhança de cada ajuste) |
| a pilha da peça | `PilhaDaPeca::le_a_vizinhanca` · `garante_vizinhanca` (refeita só quando a `Impressao` — posições, níveis — muda) · `faixa_da_superficie` |
| a placa | `LayerCompositor::set_surface` + o módulo `surface` (`ph2d-render`, o grafo em buffers + `surface_heat.wgsl`): o calor no lugar do desfoque separável nos estágios Gaussiano/Nitidez, Bloom e Sombras/Realces; a mistura de volta é a de sempre. `composto_na_placa::garante_superficie` sobe o grafo uma vez por geometria |
| o traço por baixo de um desfoque | `tinta_da_peca::pilha::recompoe_sujas` (uma vizinha de uma amostra suja também muda ⇒ a peça inteira na placa, a CPU atrasada; o relevo das sujas desce sempre) |
| a unidade do raio | `SpatialUnits` (`Pixels` · `Surface { size }`): na peça o raio grava-se no mundo e lê-se em **% da diagonal**; o slider guarda as proporções do 2D (`SURFACE_RADIUS_MAX = 2,5 %`); um ajuste novo nasce com o polegar onde o 2D o põe (`rescale_spatial_params`) |
| fora da peça | Motion, Chromatic Aberration e Halftone (`reads_the_image_plane`): pedem uma DIRECÇÃO, um CENTRO ou uma TRAMA de imagem plana; uma superfície não tem nenhum (um campo de direcções numa esfera tem de se anular — o teorema da bola cabeluda). Apagados no menu e recusados na porta (`LeOPlanoDaImagem`) |
| o tecto de degrau | `NIVEL_MAX_DA_VIZINHANCA = 6` (`64x`); acima, a porta recusa (`DegrauAlto`, a frase no painel) |

**A lei na retícula:** o desfoque é o calor `e^{−tA}`, `t = σ²/2`, `A = M⁻¹L` com `L` o laplaciano de
COTANGENTES da triangulação conforme da retícula (sub-triângulos de um triângulo são a face em ponto
pequeno; as células de quad partem-se pela diagonal mais curta) e `M` a massa baricêntrica. Aplica-se
por um polinómio de **Chebyshev** de `g(λ) = (1 − e^{−tλ})/λ` em `[0, λ_sup]` (Gershgorin), erro
`< 1e-6`, na forma `u − g(A)(A u)` — um campo constante fica constante AO BIT. Numa retícula regular o
calor acrescenta `2t` de variância por eixo em qualquer resolução: `σ²` EXACTO no mundo.

**Premissas que o código derrubou:**

1. **Passos explícitos não servem:** o passo estável é `~h²/8` ⇒ `8(σ/h)²` passos (`2 048` a `σ = 16`
   amostras); o polinómio pede `~7σ/h` termos (`~120`).
2. **O estêncil de eixos nos quads (a 1.ª lei) era inconsistente em células trapezoidais** — e a esfera
   UV é feita delas: uma amostra de ARESTA da malha desviava `28×` mais que duas do meio de faces à
   mesma distância de uma risca (média `0,195` contra `0,007` degraus, pior `3`). Com a triangulação
   conforme: pior `1` (o meio-meio também `1`), `5,7 %` das amostras de aresta a `1` degrau.
3. **A cor por vértice com uma pilha que lê vizinhos:** o prefixo dos vértices só existe com a peça
   inteira composta — ele fica atrasado COM o resto e IGUAL à cor por vértice (o estacionamento compara
   as duas, `desparqueia`); o `em_dia` (exportar, a recusa da placa) e o abrir de um ficheiro
   (`cor_por_vertice_da_composta`) levam as duas à composição exacta.
4. **A cor debaixo de alfa nulo** diverge entre a CPU e a placa (o despré-multiplicar corta em `1e-6`
   contra `f32::EPSILON` e a cauda do calor deixa `~1e-7`): `2 560` bytes a `8x` na pilha translúcida,
   TODOS de alfa `0` — ninguém os lê (o `achata` usa o fundo). A paridade compara a cor onde o alfa é
   visível. Limpar o resto na lei foi escrito, medido COSMÉTICO e retirado.
5. **O raio em px não tem sentido numa peça** (o `Bloom` nascia com `20` unidades numa peça de diagonal
   `3,5`): na peça grava-se no mundo e lê-se em %.

**Medido** (a esfera de fábrica `24×32`, perfil `smoke`; a placa é uma RTX 5060 Ti; sondas
`sonda_vizinhanca::diag_o_preco_do_desfoque_na_superficie` e
`tinta_no_produto_tests::placa_vizinhanca::diag_o_preco_do_desfoque_na_placa`):

| degrau | amostras | rigidez `λ` das amostras: p90 · p98 · máx sobre a mediana | um passo do arrasto NA PLACA: `25 %` · `50 %` · `100 %` do curso de `5 %` |
|---|---|---|---|
| `8x` | 47 k | `6,0×` · `14,8×` · `17,9×` | `0,53` · `0,99` · `2,08 ms` |
| `16x` | 188 k | `6,0×` · `14,8×` · `17,9×` | `1,66` · `3,18` · `6,90 ms` |
| `32x` | 754 k | `5,8×` · `14,8×` · `17,9×` | `25,5` · `52,7` · **`111,9 ms`** |
| `64x` | 3,0 M | `5,9×` · `14,8×` · `17,9×` | `162` · `340` · `732 ms` |

⇒ **`13 %` das amostras são `4–15×` mais rígidas que a mediana, `98 %` delas nas duas primeiras filas dos
pólos** (faces minúsculas com o mesmo número de amostras): o grau do polinómio cresce com `√λ_sup`, ~`4×`
o que a mediana pediria. O tecto do curso fica em **`2,5 %` da diagonal** (`52,7 ms` a `32x`, abaixo do
critério de `100 ms` do §7 com `1,9×` de folga) e o critério de desistência da W6 **passa**: um raio
razoável (`1 %` da peça) custa `~22 ms` a `32x`. A CPU (a referência) não é caminho vivo: a `32x` um raio
de `1 %` são `89` termos sobre `754 k` amostras (`~0,2 s`).

Paridade placa↔CPU (`a_placa_desfoca_a_peca_como_a_cpu`, os quatro ajustes a mexer): `5 / 36 / 621`
bytes a um degrau a `8x / 16x / 32x` (`0,02 %` a `32x`), nunca mais de um; a pilha translúcida `0` no que
se vê.

**⛔ Recusas MEDIDAS**

| recusado | medida | porquê |
|---|---|---|
| passos explícitos do calor | `2 048` passos contra `~120` termos (`σ = 16` amostras) | o grau do polinómio cresce com `σ/h`, os passos com o quadrado |
| estêncil de eixos nas células de quad | aresta `28×` pior (pior `3` degraus) | inconsistente numa célula trapezoidal |
| cortar a rigidez dos pólos (piso de massa) | `13 %` das amostras, `~3 %` da área | desfocaria MENOS nos pólos (`σ` efectivo `~½` lá) |
| limpar o resto do calor abaixo de `1e-6` | `0` bytes visíveis antes e depois | código sem efeito observável |
| a CPU como caminho vivo | `~0,2 s` por passo a `32x` (`1 %` da peça) | o critério é `100 ms` |
| tecto do curso em `5 %` da diagonal | `111,9 ms` a `32x` | passa do critério |
| borrar pela ordem das amostras (a dobra como imagem) | o CONTROLO dos gates vaza cor para faces longe | o §1, lição 2 |

**Gates** (W6): retícula `difusao_tests` (8) · gancho/unidades `units_tests` (4) e os 105 da crate de
efeitos · peça `vizinhanca_da_peca_tests` (11: na superfície e nada vaza — CONTROLO pela ordem —, a aresta
lê o meio, constante e raio `0` ao bit, `8x = 32x`, o alfa feathera, só os três do plano de fora, o raio
novo em %, a recusa acima de `64x`, esculpir refaz a vizinhança, a cor por vértice segue o prefixo) ·
`scenes_vizinhanca_tests` (1) · painel `seam_peca` (o menu, o chip do raio em %) · com placa
`placa_vizinhanca` (3: a paridade incl. translúcida, a saída da placa borra na superfície, o traço por
baixo de um desfoque) e o seam test do produto `painel::o_desfoque_pelo_menu_borra_a_peca_e_o_ctrl_z_o_tira`.
Mutação: `docs/3D/ferramentas/muta_os_efeitos_de_vizinhanca.sh` (49).

**Fica para depois (nomeado):**
- **`64x` com um raio grande é aos solavancos** (`340 ms` no fim do curso) e **`128x`/`256x` recusam** os
  efeitos de vizinhança. A cura medida possível: uma pirâmide pelos degraus da retícula (o desfoque
  largo num degrau mais grosso, interpolado de volta — custo independente do raio) e/ou um estêncil
  implícito por face na placa (as faces são retículas regulares: só as arestas precisam do grafo).
- Com uma pilha que lê vizinhos, a **cor por vértice** fica um gesto atrás (igual ao prefixo atrasado)
  até ao `em_dia`/gravar-abrir: invisível com o plano armado (o shader lê o plano); com o plano
  desarmado mostra a peça sem o último efeito. Uma composição exacta em segundo plano é a cura.
- O limiar do despré-multiplicar da CPU (`1e-6`) e da placa (`f32::EPSILON`) diverge também no 2D (só se
  vê em cor debaixo de alfa nulo) — anterior à W6.

## 15. W4 — o relevo por camada na peça (04/10)

**O desenho (o que o código derrubou do §2):** a «função pura extraída» são DUAS portas na
`LayerStack` (`ph2d-tool-painter/src/layers/relief_fold.rs`), não uma: *quem entra*
(`relief_layers_bottom_up`: visível com os grupos acima, de baixo para cima) e *como entra*
(`fold_relief_step`: `own · depth`, `Add` soma, `Level` enterra pela cobertura da PRÓPRIA camada).
O 2D (`ReliefFields::height_at`) e a peça (`PilhaDaPeca::relevo_composto`) chamam as duas, cada um
sobre os seus planos. A lei do curso do arrasto (`set_impasto_depth_norm`, `0,5` = zero) e o chip
(`toggle_impasto_composite`) foram para a mesma casa: o painel 2D e o espelho da peça leem-nas.

| porta | onde |
|---|---|
| a dobra (2D e peça) | `LayerStack::relief_layers_bottom_up` · `fold_relief_step` · `RELIEF_FOLD_SEED` |
| o relevo da peça | `pilha_da_peca_relevo.rs`: `relevo_composto` (inteiro) · `relevo_nas` (as sujas, ao bit) · `AssinaturaDoRelevo` / `redobra_o_relevo` · `marca_relevo` (`has_relief` = projecção dos planos) · `troca_relevo(id, …)` (o desfazer, por camada) · `pinta_camada` (fixtura das cenas) |
| o traço | `trabalho_da_activa` (a cópia leva o relevo DA activa: é sobre ele que o impasto empilha, alisa e corta) · `recebe_do_traco` (o relevo desce à activa) |
| a recomposição do painel | `tinta_da_peca::pilha::recompoe_o_plano`: a cor na placa como antes; o relevo redobrado na CPU só se a assinatura mudou → `SceneObject::relevo_sujo` |
| a subida | `slots::sync_mesh` → `MeshRenderer::upload_tinta_relevo_at` (`tinta_gpu_relevo.rs`: o relevo inteiro, as inclinações das amostras de altura mudada; recusa → o plano inteiro) |
| as inclinações a refazer | `Inclinacoes::refaz` (`ph2d-mesh-colors`): o incremental, ou — com mais de metade do plano sujo — do zero em paralelo; a MESMA escolha da `Inclinacoes::nova` (que era dela só) |
| o painel | a linha 3 (`paint_relief_line`) pinta-se também na peça; `piece_edit` pede `Metadata` com a lei do 2D; `mesma_estrutura` aceita profundidade e modo (recusa `has_relief`) |
| o gate cruzado | `PainterTool::composed_relief_of_planes` (`#[doc(hidden)]`, `relief_fold_probe.rs`): uma tela nova com estes planos, lida pelo amostrador da luz do 2D |

**Premissas que o código derrubou (técnicas, delegadas):**

1. **O tecto de vidro do 2D NÃO entra na dobra partilhada.** É a aparência da tinta numa tela em
   PÍXEIS (`H_KNEE = 24 px`) e a peça nunca o teve (o relevo da peça está nas unidades do objecto,
   `docs/3D/29`); metê-lo na porta mudaria a peça de uma camada só. Fica no `height_at` do 2D, depois
   da dobra. O gate cruzado mede abaixo do joelho (identidade).
2. **A dobra começa em `-0,0`** (`RELIEF_FOLD_SEED`): é a identidade exacta da soma em IEEE
   (`+0 + -0 = +0`). Com `+0` o `-0` gravado deixava de atravessar o ficheiro
   (`o_relevo_atravessa_o_ficheiro_ao_bit` reprovou na 1.ª corrida). No 2D só troca o sinal de um
   zero: a luz lê gradientes — nenhum byte muda (197 gates `impasto`/`relief` do 2D).
3. **O corpo da peça é o MÁXIMO dos corpos** — a cobertura do 2D (`cover_at`), que de propósito não
   escala com a profundidade (tinta de relevo mudo continua a ser tinta) e pesa a inclinação no shader
   (`tinta_relevo_n`). ⇒ «profundidade `0` = no-op ao bit» vale para a ALTURA; o gate di-lo.
4. **A BASE perde o papel de dona do relevo e fica permanente e no fundo — pela lei do 2D**
   (`PainterTool::delete_layer`: *«the base sprite is permanent — Apply bakes into it»*). A recusa
   `ABase` continua; mudou o porquê (e a frase: `app.sculpt3d.camadas.recusa.a_base`).
5. **`SCULPT_DOC_VERSION` fica em `7`.** O v7 já grava o relevo POR CAMADA (W1, premissa 6) e a
   `LayerStack` serializada já leva `impasto_depth`/`impasto_composite`/`has_relief`; o
   `de_partes` re-projecta o `has_relief` dos planos (um ficheiro não mente ao painel).
6. **O relevo não se compõe na placa:** na CPU está sempre em dia (o traço dobra as sujas, `0,04 ms`),
   e só uma mudança da FORMA da dobra (assinatura) o redobra inteiro — o arrasto da opacidade não paga
   nada a mais. A subida é só do relevo: a cor da CPU pode estar atrás da placa (§13.1) e uma subida
   inteira apagaria a composição dela.
7. **Os efeitos de vizinhança (W6) não mexem no relevo** — no 2D a dobra lê só as camadas com relevo,
   nunca os ajustes. Opacidade e máscara também não entram (no 2D a «opacidade da espessura» é a
   profundidade).
8. **A leitura do relevo da placa pedia `COPY_SRC`** nos buffers das alturas e das inclinações: sem
   ele a cópia falha em silêncio e o destino lê zeros (o 1.º vermelho do seam test eram 439
   «diferenças» = as amostras com relevo). Só os gates leem; o `amostras` já o tinha pela mesma razão.

**Medido** (sonda `relevo_painel::diag_o_preco_de_arrastar_a_profundidade`, perfil `smoke`, peça da
lição; PIOR caso: a camada de cima com relevo em TODAS as amostras, logo cada passo muda o relevo e as
inclinações da peça inteira; 20 passos). ⚠️ **A carga não desceu de `7–10`** (as linhas Skeleton e UIUX
compilavam): é um tecto por cima do número calmo, não o número calmo.

| degrau | amostras | 1.ª redacção: porta+redobra · `sync_mesh` (medianas) | depois: porta+redobra · `sync_mesh` (medianas · pior) |
|---|---|---|---|
| `8x` | 47 k | `0,08` · `1,83 ms` | `0,11` · `0,64` · `1,70 ms` |
| `16x` | 188 k | `0,27` · `6,33 ms` | `0,16` · `1,36` · `2,51 ms` |
| `32x` | 754 k | `1,86` · **`34,9 ms`** | `0,34` · **`4,65`** · `6,17 ms` |
| `64x` | 3,0 M | `4,36` · `118,6 ms` | `1,04` · `17,9` · `18,9 ms` |

⇒ a 1.ª redacção passava do critério (um quadro, `16 ms`, a `32x`) **2,2×**: o `sync_mesh` refazia as
inclinações pelo INCREMENTAL com todas as amostras sujas (épocas e anel por amostra), quando a
`Inclinacoes::nova` já sabia que acima de metade se recalcula do zero, em paralelo (`7,5 ms` a `32x`,
`diag_o_preco_de_refazer_as_inclinacoes`). A escolha passou a uma porta (`Inclinacoes::refaz`), e a
dobra inteira a `rayon` (ponto a ponto: o mesmo ao bit). **A `32x` um passo cabe num quadro com `3×` de
folga; a `64x` é `~19 ms`** (um quadro e um pouco), quase tudo as inclinações na CPU e a subida de
`24 + 36 MB`. ⇒ **CPU incremental** (a dobra e as inclinações), decidido pela medição; as inclinações
num passe de compute na placa são a cura medida possível para `64x`+ (nomeada abaixo). Um arrasto de
OPACIDADE não paga nada disto (a assinatura não muda: gate `so_a_forma_da_dobra_redobra_o_relevo`).

**Fica para depois (nomeado):**
- `64x`+ com uma camada de relevo que cobre a peça: `~19 ms` por passo da profundidade (as inclinações
  refeitas na CPU). Cura: as inclinações num compute da placa a partir das alturas que já lá estão.
- `128x`/`256x` não foram medidos neste gesto (a sonda vai até `64x`, o tecto dos efeitos de vizinhança).

**Gates** (W4): dobra `relief_fold_tests` (3) · ferramenta `piece_layers_tests::a_profundidade_e_o_level_na_peca_sao_os_do_2d`
(e o «não oferecido» sem a profundidade) · pilha `pilha_da_peca_relevo_tests` (5: **a dobra da peça = a
do 2D ao bit** com `Add`, `Level`, profundidade negativa e uma escondida, CONTROLO de ordem · o neutro e
a profundidade `0` · o traço só na activa e o desfazer · duplicar leva o relevo · só a forma da dobra
redobra) · os gates antigos que afirmavam «o relevo é da base» re-escritos
(`o_traco_desce_a_camada_activa_e_so_a_ela`, `a_base_fica_e_a_copia_leva_o_relevo`,
`as_trocas_do_desfazer_sao_involucoes`) · cena `scenes_relevo_camadas_tests` (1) · produto, com placa e
o painel REAL: `relevo_painel::a_profundidade_pelo_painel_achata_a_camada_de_cima_e_o_ctrl_z_a_devolve_ao_bit`
(o arrasto muda o relevo só onde a de cima pinta; a placa tem os bits da peça; as inclinações da placa
são as da CPU; o `Ctrl+Z` devolve ao bit na peça e na placa; o `Level` enterra).

**Supersede:** §10 premissa 6 e §11 («o relevo desce à BASE»), §12 premissa 2 (o porquê da base),
§13.1 premissa 3 (o relevo e a recomposição do painel).

## 16. Report do smoke da W4 (04/10): o impasto numa camada nova não pintava a cor

*«ao usar impasto e pintar numa Layer 3 a cor do pincel não apareceu, mas apenas o relevo»*. Medido
(`relevo_painel::o_impasto_numa_camada_nova_pinta_a_cor_e_o_relevo`): a mesma pincelada de impasto muda a
cor de `327` amostras na base e de **`0`** numa camada nova (o relevo entrava: `427`).

**O mecanismo (anterior à W4, da W2):** a tela do Painter é semeada com o retrato da **camada activa**,
com o alfa dela (`tela_semente`), e a pousada lê a diferença tela − retrato. Numa camada nova o retrato
é transparente, e a regra «sem cor num dos lados não mexe» (`Mistura::SemCor`, pensada para a borracha e
para o fundo fora da silhueta) descartava as `2 013` amostras da pincelada; o relevo tem caminho próprio
e entrava. ⇒ com o destino a ter canal de opacidade (uma camada), a leitura dá `Mistura::Camada { dpm,
da }` — a diferença em cor **pré-multiplicada e em opacidade** —, e a pousada soma as duas, pesadas pela
máscara. Tinta numa zona transparente entra com a opacidade da tela; a borracha passa a tirar opacidade
numa camada; numa camada opaca é a `Diferenca` de antes, ao bit (gate).

⛔ **Recusa MEDIDA** (escrita, corrida e retirada na mesma tarde): compor as camadas de baixo no pen-down
e pôr a mudança «sobre o que está por baixo» (cor → alfa). Partia da premissa de que a tela semeada é o
COMPOSTO; a sonda mostrou que é a camada (`SemCor` em todas as amostras) — não havia o que corrigir ali.

Gates: núcleo `tela_na_malha::pousa::alfa_tests::a_tela_semeada_com_uma_camada_poe_e_tira_opacidade` ·
produto com placa `relevo_painel::o_impasto_numa_camada_nova_pinta_a_cor_e_o_relevo` (CONTROLO: a base).
Mutação: R22, R23 no `muta_o_relevo_por_camada.sh` (sangram em corridas dirigidas).

## 17. ✅ Decisão do dono (04/10): os efeitos de vizinhança borram TAMBÉM o relevo — a próxima onda

Report do smoke da W4: *«Gaussian Blur borra a cor mas não borra o relevo»*. Medido no código: no
Painter 2D é igual — a dobra do relevo lê só as camadas com relevo (`relief_fold.rs`) e a luz do
impasto vem depois da composição; a peça seguiu a lei do 2D (§15, premissa 7). Pergunta de produto
feita ao dono, resposta: **borrar também o relevo, no 2D e na peça, com a MESMA regra** (as outras
opções eram «só a cor» e «um interruptor por efeito»).

**A onda (por abrir, janela nova):** os quatro efeitos de vizinhança (Gaussiano, Nitidez, Bloom,
Sombras/Realces) passam a agir também sobre o relevo das camadas por baixo deles — no 2D (a dobra do
relevo passa pelos ajustes de vizinhança, em ordem de pilha) e na peça (o calor da retícula sobre a
altura, na placa: a CPU da peça não é caminho vivo, §14). O desenho por decidir (técnico): o que é
«o relevo por baixo de um ajuste» (a dobra até ali), o que o Bloom/Sombras-Realces fazem a uma altura
(talvez só o Gaussiano/Nitidez tenham sentido físico — medir e escrever o porquê), e o custo (critério
§7 da W6: `100 ms` a `32x`).

## 18. A 2.ª foto do dono (04/10): «a tinta ficou com um offset em relação ao relevo»

A foto: uma pincelada vermelha de impasto numa camada nova sobre as riscas da cena `=55`, com uma orla
cinzenta lisa à volta. **Não é um deslocamento** — medido (`relevo_painel::diag_onde_cai_a_cor_e_onde_cai_o_relevo`):
o centro da cor e o do relevo distam `0,005` (o mesmo na base, `0,004`). O relevo é mais LARGO que a cor
(`427` contra `319` amostras). Três achados, por ordem de peso:

1. **O fantasma (o que mudava a imagem):** a assinatura da dobra (§15) ficava velha quando um TRAÇO dava
   o 1.º relevo a uma camada (o traço escreve a peça aos bocados e não a renovava); esconder essa camada
   voltava à assinatura guardada e a redobra dizia «nada mudou» — a peça ficava com o relevo da camada
   escondida (`941` amostras; foto com a camada escondida: um fantasma liso sobre as riscas). ⇒ a
   assinatura renova-se em `relevo_nas`, onde a peça é escrita aos bocados. ⚠️ A 1.ª redacção renovava-a
   onde a CAMADA muda (`recebe_do_traco`/`troca_relevo`): contrato frágil — um gate que escreve o relevo e
   recompõe a peça inteira apanhou-a (`a_profundidade_de_uma_camada_que_cobre…`).
2. **A encosta fora da tinta de uma camada de cima:** o alisamento do impasto espalha relevo para fora da
   tinta (`293` amostras com relevo e corpo `0` — a «encosta» do anel de 01/10). Numa camada só, o corpo
   `0` apaga-a na luz; com camadas, o corpo da peça é o MÁXIMO, e a tinta cheia da de baixo acendia-a. ⇒ a
   dobra única pesa o relevo de cada camada pela SUA tinta: `relief_share = min(1, c / min(W_SOLID,
   c_max))` (`W_SOLID = 0,75`, o filme cheio do pincel, `ph2d-painter-brush`). Numa camada só é `1` ao
   bit; sem tinta dela sobre tinta de outra é `0`. Vale para o 2D (mesma dobra): duas camadas de impasto no
   2D tinham o mesmo defeito. ⛔ **Recusa MEDIDA:** a razão crua `c / c_max` desenhava uma MOLDURA onde a
   tinta de baixo começa debaixo de uma tinta de cima parcial (foto do cruzamento de duas pinceladas).
3. **O que fica e é o desenho:** um fio de luz à volta da tinta — a parede do impasto sobe dentro do
   pigmento meio-transparente (alfa `64–191`: corpo médio `0,50`, altura `0,08`), como no 2D
   (`W_TAIL..W_SOLID`). Medido: a opacidade da tinta é a mesma sobre a azul e fora dela (`221` contra
   `225`); a «moldura» que se vê no cruzamento de duas pinceladas é a luz do tubo de baixo vista através
   da tinta de cima somada (`Add`) — físico.

⚠️ **Consequência nomeada da lei 2:** uma camada de profundidade `0` deixa de mudar a altura ao bit quando
a tinta dela é mais forte que a de baixo (a parte da de baixo é a sua tinta sobre a maior); o gate passou a
afirmar a propriedade honesta — profundidade `0` = a mesma peça que o relevo dela a ZERO, ao bit.

Gates: `relief_fold::tests::a_parte_do_relevo_de_uma_camada_e_a_sua_tinta_sobre_a_maior` (2D) ·
`pilha_da_peca::relevo::tests::{a_encosta_fora_da_tinta_de_uma_camada_de_cima_nao_conta,
esconder_uma_camada_que_ganhou_relevo_num_traco_tira_o_relevo_dela}` · o neutro re-escrito. Sondas com
foto: `relevo_painel::{diag_a_orla_da_camada_de_cima, diag_a_orla_sobre_as_riscas}`. Mutação: R24, R25.
