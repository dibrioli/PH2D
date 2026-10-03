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
