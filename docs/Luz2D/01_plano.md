# Luz 2D — plano de construção (desktop primeiro)

> Linha `line/Luz2D`, 2026-10-08. Fonte da escolha técnica: o relatório
> [`pesquisa/reports/Luz 2D estado da arte.md`](pesquisa/reports/Luz%202D%20estado%20da%20arte.md)
> (sete notas com fontes). Este doc é o **desenho** e a **ordem**; o porquê de cada técnica está lá.
> Ordem aceita pelo dono: **(1) o gabarito, (2) as cenas que ele abre no editor**; celular adiado
> («ainda é cedo»), mas o **tráfego de memória por quadro é contado desde o dia 1** (é a grandeza que
> decide o celular e não depende do aparelho). Aparelhos do dono para a fase de celular: iPhone 13
> (A15) · iPhone 16 (A18, RT) · POCO X5 (Snapdragon 695 / Adreno 619). ⚠️ Sem Mali na matriz.

## Em linguagem simples (para o dono)

Qualquer coisa que brilha passa a ser uma lâmpada; objetos marcados bloqueiam a luz e fazem sombra
suave que depende do tamanho da lâmpada; a luz rebate em paredes coloridas. O custo é o mesmo com uma
ou mil luzes. Primeiro construímos um «gabarito» — uma luz calculada devagar e exata — para que cada
versão rápida seja conferida contra ele. Depois vêm o objeto **Luz** no menu Add, a secção dele no
Inspector, e quatro cenas de teste.

---

## 1. Estado da arte — o que os outros fazem, e o que abandonaram

O detalhe com fontes está no relatório; aqui só a decisão que cada um ensina.

| Quem | O que faz | O que TENTOU e abandonou / onde para | Lição para o PH2D |
|---|---|---|---|
| **Godot 4** | forward de passe único; cada item percorre ≤ 15 luzes (contador de 4 bits) e descarta o resto sem aviso; sombra = mapa polar 1D + PCF de largura fixa | **abandonou** o 3.x (cada luz redesenhava os itens: `4 × luzes × oclusores` draw calls até a 4.4); o *unroll* do laço de luzes (PR #100501) foi parqueado | o tecto de luzes era **formato de dado**, não hardware (§0 nosso proíbe isso); penumbra por filtro ≠ física |
| **Unity URP 2D** | luzes rasterizadas em «light textures» a 0,5× por estilo de mistura × grupo de *sorting layers*; normal map = pré-passe de tela cheia | normal map chamado de «very expensive» pela própria Unity | o custo multiplica por **agrupamento**, não por luz — a nossa cena não pode ter eixo assim |
| **Path of Exile 2** | Radiance Cascades em espaço de tela, oclusão de 128 raios num bitmask | o RC de tela **não vê** emissores fora da tela | a nossa grade é **ancorada no mundo, com margem** |
| **Ori 2 / INSIDE** | luz **pintada** (30 000 mapas de direção à mão; decals de rebote) | Ori 1 → Ori 2: de pintado para dados pintados guiados por luz real | canais **autorais** convivem com a física (C4) |
| **Sannikov / comunidade RC** | RC clássico, *bilinear fix*, pré-média | o layout *direction-first* «2× mais rápido» foi **retratado** pelo próprio autor (era defeito do GameMaker) | medir no NOSSO motor antes de adotar um ganho alheio |
| **HRC (2025)** | RC holográfico: penumbra quase exata, 1,85 ms a 512² (RTX 3080 Laptop) | memória ~N× a grade; luzes < 8× o espaçamento ainda erram; **sem código público** | «Ultra» de desktop e juiz secundário; nunca o padrão no celular sem medir |
| **Blender / AE / Illustrator / Rive** | ⚠️ *conhecimento geral, não verificado nesta pesquisa:* Blender ilumina Grease Pencil com as luzes 3D da cena (EEVEE); AE só ilumina camadas 3D (os efeitos 2D de luz são falsos, por camada); Illustrator e Rive **não têm** luz de cena | — | nenhuma ferramenta de autoria 2D tem luz 2D de cena com sombra e rebote: o PH2D define a UI (§4), e a referência de UI é o par Godot/Unity (cor · intensidade · altura · alcance · sombra) |

**Escolha:** RC 2D clássico (bilinear fix + pré-média, f16) como solver padrão; HRC como «Ultra»; caminho
analítico para luzes minúsculas; composição *forward* no passe dos sprites; juiz = path tracer 2D em CPU.

---

## 2. O desenho — uma porta por pergunta

### 2.1 As crates (drop-crates; a shell só compõe)

| Crate | Papel | Depende de |
|---|---|---|
| `ph2d-light2d-ecs` | os componentes, a porta `register_light2d_components`, e a **coleta** (`world → CenaDeLuz`, CPU, pura) | `ph2d-ecs` |
| `ph2d-light2d-ref` | o **gabarito**: path tracer 2D em CPU, `f64`, determinístico; responde a mesma pergunta que a GPU | `ph2d-light2d-ecs` (só a `CenaDeLuz`) |
| `ph2d-light2d-gpu` | os passes C0–C3, os `.wgsl`, a **lista declarada de passes** e a contabilidade de bytes | `ph2d-gpu`, `ph2d-render` |
| `ph2d-app-light2d` | a família: roteador `PH2D_LIGHT2D_SMOKE`, entradas do menu Add, secções do Inspector, composição no quadro | as três acima + `ph2d-app-host` |

⚠️ **Nome:** a crate `ph2d-light` existente é o rig de 4 lâmpadas do **impasto** (sem oclusor, sem
atenuação) — fica intocada. O guia de stack que a chamava «stub de Radiance Cascades» foi corrigido
nesta linha (§11.4 e a tabela §2).

### 2.2 As perguntas e a porta ÚNICA de cada uma

| Pergunta | Porta única | Quem a consome |
|---|---|---|
| **Quem emite, com que cor e força?** | `coleta::cena_de_luz(&World) -> CenaDeLuz` | o gabarito **e** o passe C0 (a GPU nunca lê o ECS por outro caminho) |
| **Quem bloqueia?** | a mesma `CenaDeLuz` (lista de oclusores com opacidade) | idem |
| **Quanto a luz cai com a distância?** | `queda(t, alcance)` — UMA função em `queda.wgsl` + gêmea Rust em `ph2d-light2d-ref`, com gate que compara as duas em 1 000 pontos | o traçado de cada cascata (o RC sabe a distância do acerto, então a queda autoral por luz é exata no intervalo) e o gabarito |
| **Quanta luz chega a este ponto do mundo?** | `luz_em(mundo) -> irradiância` em `receber.wgsl`, **concatenado** (`concat!(include_str!…)`) no shader de sprite **e** no compositor que recebe o vetor — não há `#include` no repo, e duas cópias divergem em silêncio | sprite (HDR, antes do tonemap) e vetor (ver 2.4) |
| **Quantos bytes a luz move por quadro?** | `PassList::bytes_por_quadro(&Config)` — derivado da **mesma** lista de passes que cria as texturas | o governador, o gate de orçamento e a sonda |
| **Que degrau usar?** | `governador::degrau(perfil, folga)` | a montagem da `PassList` |
| **O que é a resposta CERTA?** | `ph2d-light2d-ref::fluencia(&CenaDeLuz, ponto)` | todos os gates de qualidade |

### 2.3 Componentes (registo novo, **sem** degrau de `PROJECT_SCHEMA`)

O `ComponentBlob` é chaveado por `blake3(nome canónico)` (`crates/ph2d-ecs/src/scene/registry.rs`,
comentário do `VecMorphMachine`): um componente **novo** viaja sem degrau — um ficheiro antigo
simplesmente não tem o blob. ⇒ **nenhum componente existente muda de forma.**

| Componente (nome canónico) | Campos | Notas |
|---|---|---|
| `ph2d::light2d::Luz` | `cor: [f32;3]` (linear), `intensidade: f32`, `tamanho_m: f32` (raio da FONTE — decide a penumbra), `alcance_m: f32` (queda autoral; `∞` = física pura 1/r do plano), `tipo: Ponto \| Ambiente` | o objeto do menu Add; `Ambiente` substitui um «settings de cena» (não há porta de recurso de cena persistido) |
| `ph2d::light2d::BloqueiaLuz` | `opacidade: f32` | a forma do oclusor = o **alfa** do sprite/vetor do mesmo objeto |
| `ph2d::light2d::NaoRecebeLuz` | — (marcador) | por omissão TUDO recebe; o marcador é o opt-out (HUD, fundo pintado com luz própria) |
| `SpriteEmissive` (existe) | `f32` — **inalterado** | a COR da emissão é a do próprio sprite (pixels × intensidade); a sprite que já brilha passa a iluminar sem migração |

Contadores que sobem (somam entre linhas — **contar** na integração): `ComponentRegistry` do ECS +
os dois espelhos (`ph2d-render/src/registry.rs`, `ph2d-script/src/registry.rs`), o catálogo
`ph2d-component-desc` (ordenado), os rótulos em `ph2d-i18n/src/component_catalog.rs`.

### 2.4 O quadro — onde cada passe entra

Hoje (`shells/desktop/src/render_loop/present.rs`): sprites → `GameRt` (HDR) · Flip · formas do Motion
· emissivo/glow (`present_fx`) · tonemap AgX · **Vello (vetor do documento, LDR, `Rgba8Unorm`)** ·
compositor.

Novo, **só quando a cena tem ao menos uma `Luz` ou um emissor ativo** (senão o quadro é o de hoje,
**byte a byte** — gate G6):

```
C0 captura   (fragment, resolução de luz, grade ANCORADA NO MUNDO + margem fora da tela)
             emissão RGB · opacidade · albedo · alcance — de sprites (re-render em isolamento, como o
             emissivo já faz) e de vetor/Flip/Motion pelo TILE de DPI fixo (o mesmo achado do
             `motion_glow_layer`: a luz em baixa resolução não precisa de nitidez)
C1 campo     JFA em resolução reduzida; camada estática em cache, dinâmica só nas regiões sujas
C2 RC        cascatas (ramificação 4, bilinear fix, pré-média, f16); merge de cima para baixo → fluência
C3 analítico luzes abaixo do limiar MEDIDO (não escolhido) — sombra pelo mesmo campo da C1
C4 receber   o shader de sprite amostra `luz_em()` (forward, sem buffer de tela cheia extra)
             o vetor (LDR) recebe no compositor: linear(albedo) × E → AgX — a MESMA conta do sprite
```

⚠️ O vetor vive depois do tonemap porque o Vello só escreve `Rgba8Unorm` (medido, ver
`crates/ph2d-app-motion/src/motion_glow_layer.rs`). Receber a luz ali é o ponto mais frágil do
desenho: **gate G7** (um sprite e uma forma vetorial da mesma cor, sob a mesma luz, dão o mesmo pixel)
é escrito ANTES do código do compositor.

O rebote: fluência do quadro anterior × albedo realimenta a emissão da C0 (um rebote por quadro;
gate G9 contra o gabarito com 2+ rebotes).

### 2.5 Orçamento

O guia de stack declara **2,5 ms a 60 Hz** e **80 MB** de luz no iPad/iPhone. São **metas**, não
medições (§0). Pela conta do relatório, 80 MB exclui o HRC no celular e exige c0 ≥ 2 px em tela de
iPad. A sonda da W3 transforma a meta em número.

---

## 3. Contratos congelados e schema — não encosta (prova por grep)

| Contrato (CLAUDE.md §6) | O que a linha acrescenta | Prova |
|---|---|---|
| Nodes `NodeOp=2/OpResolver=1/NodeManifest=8` | nenhum nó | `git grep -n 'NodeOp\|NodeManifest' -- 'crates/ph2d-light2d*' 'crates/ph2d-app-light2d'` → vazio (crates novas não dependem de `ph2d-nodegraph`) |
| Tools `Tool=12/RasterEditTool=5/CanvasPaintTool=1/PanelEvent=4` | nenhuma tool; o Inspector edita por `EditorAction`/`ComponentEdit` (não congelados: `crates/ph2d-editor-core/src/action_bus.rs:40`, `action_bus_component.rs:29`), como a secção Vida | o gate `architecture_tool_contract_surface.rs:96` só conta `pub enum PanelEvent {` — a linha não a edita |
| Vector `VectorOp≤16…` | nada no data-model vetorial; o vetor só é **amostrado** como tile para a C0 | `git diff <base> -- crates/ph2d-vector-doc` vazio no fecho |
| `PROJECT_SCHEMA` (187) | **sem degrau** (2.3) | `git diff <base> -- shells/desktop/src/project_schema*.rs` vazio no fecho; se mudar, recontar por `scripts/schema-recount.py` |
| `ObjectKind` | **nenhuma variante**: a Luz é objeto de JOGO (vazio + componente), como a Câmera (`ph2d-app-components/src/object_add.rs:23`) | `kind_of` não muda |

Foundational tocado (permitido, ADR-0107, anotar no handoff): `ph2d-render` (shader de sprite ganha o
`@group` da luz; compositor ganha o caminho do vetor), `ph2d-ecs` (só a chamada à porta de registo),
`ph2d-gpu/context.rs` (pedir `SHADER_F16` quando o adaptador tem — opcional, com caminho f32 igual).

---

## 4. UI — as quatro condições, por controle

| Controle | 1. existe | 2. pintado + registado | 3. clique → barramento | 4. a SEQUÊNCIA leva a algum lugar |
|---|---|---|---|---|
| Menu Add → **Luz** / **Luz ambiente** | `AddEntry` novas em `ph2d-app-light2d::object_add::ENTRIES` | entram na lista `fase_object_add.rs:14-17` (+1 linha da shell, composição) | `object_add::add` → `attach_by_name` (a porta que o `+` do Inspector usa) | aparece o objeto na cena **com um gizmo** que se seleciona e arrasta → a cena acende |
| Inspector **Luz**: Cor · Intensidade · Tamanho · Alcance · Tipo | secção na crate da família (molde `camera_inspector.rs`) | ids + populate + paint (molde Vida: `populate_vida.rs`, `paint_optional_vida.rs`) | `ComponentEdit::Luz` → `EditorAction` | arrastar Intensidade muda a luz no MESMO quadro; `Ctrl+Z` desfaz (undo por diff: componente registado ⇒ entra em `world`) |
| Paleta `+` → **Bloqueia luz** / **Não recebe luz** | descritores no catálogo | a paleta já pinta o catálogo | `attach_by_name` | a sombra aparece / o objeto fica fora da luz |
| Gizmo da Luz no canvas | ⚠️ **a medir antes**: como a Câmera (sem sprite) é escolhida no canvas — ler `shells/desktop/src/input_dispatch/despacho_clique_pick.rs` e `pick_objects_at` | — | — | sem gizmo, uma Luz sem sprite é **inalcançável** pelo clique: o artista só a acha na árvore |

Todo texto por `ph2d-i18n` (censo HR-15 da família); zero hex/f32 de UI.

---

## 5. Gates — red-first, com a fixture que contém o fenômeno

| # | Gate | Fixture (o fenômeno dentro dela) | Vermelho primeiro como |
|---|---|---|---|
| G1 | gabarito × fórmula fechada | disco emissor de raio `a`, radiância `L`, ponto a distância `r` no vazio: fluência = `2·L·asin(a/r)` | escrito antes do tracer (falha por não existir) |
| G2 | penumbra geométrica | disco + semi-plano oclusor: largura da penumbra = projeção do disco pela aresta (cresce com `a`) — **não** alinhada aos eixos (lição: fixture alinhada não mede base) | idem |
| G3 | GPU RC × gabarito, por degrau | réplica da HRC Fig. 8 (512², oclusor 14 px, luz 10 px) + caso «faísca» (luz de 2 px) | RC sem *bilinear fix* tem de reprovar o anel |
| G4 | custo independente das luzes | 1 · 10 · 100 · 1 000 · 10 000 emissores, mesma área | faixa isolada do nextest + linha em `FLAKES_DE_CARGA.md` (é gate de relógio) |
| G5 | bytes declarados = bytes alocados | `PassList` × soma das texturas que o wgpu criou | uma textura criada fora da lista tem de reprovar |
| G6 | **ponto neutro byte-idêntico** | cena sem `Luz` e sem emissor: quadro == quadro de hoje | ligar o passe sem a guarda tem de reprovar |
| G7 | sprite e vetor da mesma cor = mesmo pixel sob a mesma luz | quadrado sprite + quadrado vetorial lado a lado, uma luz colorida | o compositor sem o caminho tem de reprovar |
| G8 | costura da UI (`ph2d-ui-testkit`) | Add→Luz cria objeto com `Luz`; slider Intensidade muda o componente; `Ctrl+Z` volta | evento real dirigido, efeito observável |
| G9 | rebote converge | sala fechada, paredes vermelha/verde, uma luz branca | sem realimentação, o branco no chão não se tinge |
| G10 | determinismo | gabarito: duas corridas bit-idênticas; RC: cena estática, dois quadros idênticos | — |
| G11 | sem tremor sob pan | S6: pan sub-texel, pixel estático varia ≤ limiar medido | grade não ancorada tem de reprovar |
| G12 | `queda` WGSL = `queda` Rust | 1 000 pontos `(t, alcance)` | mudar uma constante de um lado reprova |
| — | censos da família | HR-15, `architecture_interactive_crate_has_behavioral_test`, `the_shell_only_shrinks`, `the_shell_links_exactly_the_registered_families` | — |

Provas de mutação (`mutacao`) em G3, G6, G7 e G12 no fecho — são as leis que mentem em silêncio.

---

## 6. Cenas de smoke (`PH2D_LIGHT2D_SMOKE=<n>`)

| n | Cena | O que o dono vê | Número que a sonda mede ANTES da mensagem |
|---|---|---|---|
| 1 | **Uma luz** — sala escura, uma Luz, uma parede, três sprites | sombra suave que alarga longe da parede | ms por passe (timestamps) · bytes/quadro · RMSE contra o gabarito |
| 2 | **Vaga-lumes** — 1 000 sprites emissivos voando sobre um cenário | cada um ilumina o chão em volta | ms com 1 e com 1 000 emissores (a curva tem de ser plana) |
| 3 | **Rebote** — sala com paredes coloridas | o chão tinge-se com a cor das paredes | quadros até convergir |
| 4 | **Tamanho da lâmpada** — duas Luzes, pequena e grande, mesmo oclusor | a pequena faz sombra nítida, a grande, macia | largura da penumbra medida × prevista pela geometria |

Cada cena é **fotografada** (`docs/Components/ferramentas/fotografa_cena.sh`) antes de ir ao dono, e o
número de cada uma vem da sonda headless (molde: `crates/ph2d-board-render/tests/it/measure_*.rs`).

---

## 7. Ondas (uma janela por onda)

| Onda | Entrega | Fecha com |
|---|---|---|
| **W0** | este plano · README · guia de stack corrigido | — (docs) |
| **W1 — o gabarito** | `ph2d-light2d-ecs` (componentes + coleta) e `ph2d-light2d-ref` (tracer CPU) | G1, G2, G10 (ref), G12 (lado Rust) |
| **W2 — o solver** | `ph2d-light2d-gpu`: C0, C1, C2, `PassList` + bytes, timestamps; sonda headless | G3, G4, G5, G10, G12 |
| **W3 — receber** | sprite forward + vetor no compositor; ponto neutro; ancoragem | G6, G7, G11 |
| **W4 — o artista** | `ph2d-app-light2d`: menu Add, Inspector, gizmo, paleta, undo, i18n; cenas 1–4 fotografadas | G8 + censos; **smoke do dono** |
| **W5 — o teto** | rebote, HRC «Ultra», C3 analítico (limiar medido), governador; rodada de medição desktop S1–S8 com o Godot por script como piso | G9 + rodada registrada |
| depois | celular (iPhone 13/16, POCO X5) — **adiado pelo dono** | K1–K9 do relatório |

Kill-criteria (definidos antes, do relatório): K3 — se o custo entre 1 e 10 000 emissores variar
> 5 %, há um caminho dependente do conteúdo e a W2 não fecha; K6 — se o *bilinear fix* custar > 2× o
RC sem ele, mede-se *nearest-interlaced* antes de aceitar. Regra de duas tentativas: na 2.ª
reconstrução de topologia do solver, PARAR e provar o modelo contra o gabarito.
