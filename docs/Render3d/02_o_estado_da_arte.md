# 02 — O estado da arte em 2026, e onde é que se pode ganhar

⚠️ **Verificado contra fontes de 2026**, não afirmado de memória — as ligações estão no fim.

## §1 — Materiais: a Unreal já reescreveu a dela, e a norma aberta apanhou-a

| | Unreal | o mundo aberto |
|---|---|---|
| modelo | **Substrate** — «lajes de matéria» empilháveis, que **substituem** a lista fixa de *shading models* (`Default Lit`, `Clear Coat`, …) | **OpenPBR Surface** — 41 entradas, uma superfície só, com camadas dentro |
| estado | **pronto para produção e ligado por omissão** em projectos novos desde a **5.7** | **1.39.5**, assinado por Adobe · Autodesk · Blender · ASWF |
| autoria | grafo de material da Unreal | **MaterialX** — 807 nós, como dado, com gerador de código |
| intercâmbio | fraco (o material não sai) | ⭐ **é o formato de intercâmbio** (USD/MaterialX) |

⭐ **A leitura estratégica:** a Unreal foi de *modelos fixos* para *matéria composta*, e a norma
aberta chegou ao mesmo sítio pelo outro lado. **Adoptar o OpenPBR não é ficar atrás — é chegar ao
mesmo destino sem escrever o motor de materiais.**

## §2 — Luz indirecta: as três famílias vivas

| família | quem a usa | o que a caracteriza |
|---|---|---|
| **Lumen** | Unreal 5 | traça contra *proxies* — **campos de distância da malha** e um *surface cache*; robusto, caro, e a qualidade depende dos proxies |
| **ReSTIR PT + denoiser de IA** | o caminho «traçado a sério» | `1` amostra por pixel + reamostragem espaço-temporal; ideia simples, ganha adopção, exige hardware de raios |
| ⭐ **Radiance Cascades** | *Path of Exile 2* (2D); **Split Radiance Cascades** (2026) leva-a a sondas esparsas em 3D | resolução espacial alta perto e angular alta longe — **sem ruído e sem aliasing**, que é a diferença que se vê |

⚠️ **A terceira é a novidade real desta década** e é a mais alinhada com uma engine estilizada: ela
não produz ruído, logo **não precisa de um denoiser de IA** — e um denoiser é exactamente o que
estraga um look de cores chapadas.

## §3 — O resto do arsenal da Unreal, e o que dele nos serve

| tecnologia | serve-nos? | porquê |
|---|---|---|
| **Nanite** (geometria virtualizada) | ⛔ **não** | resolve malhas de milhares de milhões de triângulos importadas; as nossas nascem de **campos** e de escultura |
| **Virtual Shadow Maps** | ⭐ **sim, o princípio** | sombras com resolução onde o olho está — mas a versão simples (cascatas + contacto) entrega `80 %` do valor |
| **TSR / DLSS / FSR** | ⏸️ mais tarde | é orçamento, não beleza |
| **Path tracer offline** | ⭐ **sim, como ORÁCULO** | é a régua contra a qual o tempo real se mede |

## §4 — ⛔ Onde NÃO vamos bater a Unreal, e dizê-lo poupa uma jornada

- **Escala de conteúdo.** Nanite + Virtual Textures + streaming são anos-pessoa de engenharia de
  dados. Não é o nosso problema nem o do alvo do dono.
- **Ecossistema.** Marketplace, MetaHuman, gente formada.
- **Hardware de raios.** Se a resposta certa for ReSTIR, eles chegaram primeiro e melhor.

## §5 — ⭐⭐⭐ Onde PODEMOS bater, e as três razões são estruturais

### 5.1 — A cena já É um campo de distância

O trabalho mais difícil do Lumen é ter uma representação da cena que se percorra depressa. A Unreal
**aproxima** cada malha por um campo de distância pré-cozido, com todos os erros que isso traz.

⭐⭐ **O nosso modelador não aproxima nada: ele é o campo.** O `ph2d-field-eval` já avalia distância
assinada em qualquer ponto, já tem marcha de esferas, já tem limites de gradiente medidos, já
especializa a árvore por ladrilho. *A esquisitice do nosso módulo 3D é, literalmente, o substrato que
a Unreal constrói à mão.*

⇒ **GI traçada contra o modelo verdadeiro**, sem proxy e sem o erro dele.

⚠️ **E o preço já está medido**: o quadro de movimento do modelador custa `26,7 ms` contra um
orçamento de `16,7`, com a marcha a ser `80 %` disso. *Um traçado de GI por cima disto não é grátis,
e a wave que o fizer começa por aí.*

### 5.2 — A autoria é onde a Unreal é fraca, e onde esta casa já tem lei

O editor de materiais da Unreal é poderoso e **difícil**; o Substrate tornou-o mais poderoso e **não
mais simples**. O dono pediu o contrário: *«intuitivo para artistas, de fácil uso mas grande poder»*.

⭐ E esta casa já pagou a lei que isso exige, noutro módulo:
- **o painel oferece exactamente o que o gesto faz** (a lei da W34 do modelador),
- **nenhum knob morto** (a caça de 2026-08-30: 34 mortos em ~504 controlos),
- **os params vivem no CARTÃO do nó** (a dinâmica dos ciclos dos Motion Nodes),
- e um **painel derivado de uma TABELA** foi o único a sair `42/42` limpo.

⇒ ⭐⭐ **Um grafo MaterialX com o OpenPBR como nó único de superfície é simultaneamente mais simples
E mais padrão do que o editor da Unreal.** O artista vê a foto que o dono mandou; quem quer mais,
abre o grafo.

### 5.3 — Um só stack de shader, e a web incluída

O `wgpu` 29 entrega Vulkan · Metal · DX12 · **WebGPU** com um código. A história de web da Unreal é
fraca. Não é beleza — é alcance.

## §6 — O «suprasumo», nomeado

Se a pergunta for *«qual é o tecto?»*, o tecto de 2026 é:

> **superfície `OpenPBR` composta** · **luz indirecta sem ruído** (cascatas de radiância) ·
> **sombras com contacto** · **céu como fonte** · **gestão de cor a sério** · e **uma camada de
> estilo com botões**, tudo autorado num **grafo padrão** que entra e sai de outras ferramentas.

⭐ Nada nessa lista exige hardware de raios, e **três dos seis itens são baratos**. É por isso que o
plano do `03` começa por eles.

---

## Fontes

- [Substrate Materials in Unreal Engine 5.8 (Epic)](https://dev.epicgames.com/documentation/en-us/unreal-engine/substrate-materials-in-unreal-engine)
- [Overview of Substrate Materials (Epic)](https://dev.epicgames.com/documentation/unreal-engine/overview-of-substrate-materials-in-unreal-engine?lang=en-US)
- [280 Substrate materials para UE 5.7 (Unreal Engine)](https://www.unrealengine.com/news/get-over-280-production-ready-automotive-substrate-materials-for-ue-5-7-free-on-fab)
- [Split Radiance Cascades: Real-Time GI via Sparse Radiance Probes (arXiv 2607.20384)](https://arxiv.org/abs/2607.20384)
- [Radiance Cascades for Real-Time 2D Global Illumination (IEEE)](https://ieeexplore.ieee.org/document/11307155/)
- [Radiance Caching with On-Surface Caches (ACM)](https://dl.acm.org/doi/10.1145/3675382)
- [Plants vs. Zombies: Battle for Neighborville (Wikipedia — motor Frostbite)](https://en.wikipedia.org/wiki/Plants_vs._Zombies:_Battle_for_Neighborville)
- [PopCap sobre levar a Frostbite ao Switch (Nintendo Life)](https://www.nintendolife.com/news/2021/03/feature_plants_vs_zombies_producer_on_bringing_eas_frostbite_engine_to_switch)
