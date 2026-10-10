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

Contadores que sobem (somam entre linhas — **contar** na integração): as DUAS listas de registo da
shell (`shells/desktop/src/init.rs` e `tests/it/every_registered_component_is_described.rs`), o
catálogo `ph2d-component-desc` (família `light.rs`, categoria nova `Light` → `ALL` 16), os rótulos em
`ph2d-i18n/src/component_catalog.rs`. ⚠️ *Medido na W1:* o `ComponentRegistry` do ECS (105) e os dois
espelhos (106) contam só os registadores DELES — não sobem.

**Semântica fixada na W1 (a GPU responde à MESMA pergunta — gate G3):**
- `intensidade` = **fluência a 1 m, no vazio**: radiância do disco `cor·I/(2·tamanho_m)`; crescer a
  lâmpada amacia a sombra sem mudar o brilho; `tamanho_m = 0` é o ponto exato (`F = I/r`, o caso
  da C3). `Ambiente`: radiância do céu `cor·I/(2π)`, que chega pelas direções que ESCAPAM.
- Emissores com área são **opacos** (o raio termina no primeiro).
- `opacidade` do `BloqueiaLuz` é **por ENTRADA** do raio no objeto (`× (1−α)` por entrada, sem
  depender da espessura) — no RC isso é detetar a transição fora→dentro, inclusive na fronteira
  de cada intervalo de cascata.
- **Luz recebida (fixada na W3):** `luz_em(p) = F(p) / 2π` — a radiância MÉDIA sobre a volta. Um
  sprite/vetor é uma superfície DE FRENTE para a câmera, no plano da cena: a luz chega rasante de
  todas as direções do plano e não há normal no plano que pese umas mais que outras ⇒ o núcleo de
  reflexão é isotrópico, `k(ω) = 1/2π` (integral 1). O fator sai do **forno branco**: num campo de
  radiância uniforme `L`, uma superfície branca devolve `L` (`∫ L·k dω = L`); qualquer outro fator
  (`1/π` do Lambert 3D, `1/2` do Lambert de flatland com normal no plano) acende ou apaga a cena
  inteira. O receptor faz `linear(albedo) × luz_em` (sprite antes do tonemap; vetor na colagem, a
  MESMA conta). Consequências medidas pelo gabarito (`ph2d-light2d-ref::luz_em`, gates
  `luz_em_de_frente.rs`): Ambiente `cor·I` dá `luz_em = cor·I/2π` (`I = 2π` acende branco a 1); um
  ponto `I` a `r` dá `I/(2π·r)` — **a W4 precisa disto para o padrão do Inspector**: `I = 2π` é
  branco a 1 m.
- A forma que a coleta vê é o **quad** do sprite (`GlobalTransform` × âncora/tamanho); a cor de um
  emissor é `tint∘self_tint`·α·k. Os texels e o alfa por texel NÃO estão no `World` (só o
  `AssetId`): são da captura C0.

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

**Fixado na W2 (medido — [handoff W2](handoffs/HANDOFF_CONTINUACAO_W2_2026-10-08.md)):** a C0 escreve
`cobertura` (soma, 8×8 subamostras), `tau_dentro` e `emissao` do DONO do texel (cobertura própria ≥ ½); a
marcha diz «dentro» onde a cobertura INTERPOLADA ≥ ½ e lê o material do vizinho mais coberto. Ler o texel
mais próximo dilatava a silhueta (erro de captura 0,097 → 0,018 no Alto). O alfa de um sprite é a cobertura.

**Fixado na W3 (medido — [handoff W3](handoffs/HANDOFF_CONTINUACAO_W3_2026-10-09.md)):** a guarda é a
PRESENÇA de uma `Luz` (não «ou um emissor»: todo projeto com brilho escureceria); a luz acesa força o
quadro às FAIXAS e o vetor recebe na COLAGEM delas (o compositor não separa documento de chrome); o
tonemap é UMA função (`tonemap_conta.wgsl`) partilhada com a colagem; o vetor que bloqueia entra por uma
camada do Vello alinhada à grade, 1 px por texel, lida exata; o degrau padrão (texel 4 px, margem 12,5 %)
sai do K7 a 1080p.

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

### 7.1 Fila de implementação (ordem do dono, 10/10: «coloque na fila o que ainda não está bom»)

O que o smoke da W4b deixou — a próxima janela começa por AQUI, antes do resto da W5. Números e causas
medidas: [handoff W4b](handoffs/HANDOFF_CONTINUACAO_W4_2026-10-09.md) (secção «W4b — a grade FINA»).

| # | o que o dono vê | causa medida | cura candidata (medir antes) |
|---|---|---|---|
| ~~F1~~ ✅ W5a | ~~Em Low a sombra inteira sai uns 3–4 px deslocada para o lado da sombra (Medium ~2 · High ~1)~~ — agora −0,8 px em todo nível | a captura GROSSA arredonda o canto do oclusor (`meio` = cobertura bilinear ≥ ½): ½–¾ de texel grosso; a marcha mais fina só tira ~16 % | uma captura FINA para a luz direta (2.ª C0 + JFA na grade fina, com sprites e a camada do vetor desenhados nela) — régua: a média da `a_escada_da_sombra` |
| ~~F2~~ ✅ W5a | ~~Em Low, objetos muito finos (~5 cm na vista de fábrica) não fazem sombra~~ — a barra de 5 cm faz a sombra do gabarito (2,6 %) | a mesma captura grossa: um oclusor mais fino que ~1 texel some dela | a mesma do F1 (um bloco só, §0.10) |
| ~~F3~~ ✅ W5a | ~~Uma linha escura de 1 px junto à face iluminada das paredes, em todo nível~~ — a face sai a 0,98–0,99 do gabarito | o interior de um opaco recebe 0 e a bilinear do receptor mistura-o na borda (o ABERTO 1 da W4: paredes que recebem luz saem pretas) | o passe de PREENCHIMENTO (o interior recebe a fluência do exterior mais próximo — um JFA do complemento) e a mesma regra no gabarito |
| **F4** | Ao dar zoom e pan o aspecto da iluminação muda (smoke do dono, 10/10: a área à direita da parede pequena da cena 4 preta numa vista, cinza na outra) | MEDIDO: a lâmpada DIRETA ilumina de qualquer distância, mas um opaco FORA da grade de luz não a tapa — na vista de 1,5 m o raio dos pontos na sombra até a lâmpada grande cruza o muro do meio fora da grade: 0 · 2,45 · 2,38 contra o gabarito 0 · 0 · 0 (0,176 do máximo); nas vistas de fábrica, 4 m e 2,5 m, 0 · 0 · 0. Gate `a_luz_num_ponto_nao_muda_com_a_vista` (com `ignore` até a cura) | medir antes: (a) um mapa de sombra polar por lâmpada direta (o que o Godot faz) para o trecho fora da grade; (b) uma 2.ª captura, grossa e CONSERVADORA (dentro ⟺ cobertura > 0), do caminho até as lâmpadas; (c) os polígonos fora da grade testados analiticamente — o custo por lâmpada e a paridade com o gabarito decidem |

**F1–F3 fechados na W5a** ([handoff W5a](handoffs/HANDOFF_CONTINUACAO_W5a_2026-10-10.md)): a captura FINA
dos opacos (½ px), o passo da luz direta ½ junto à borda, o refino pela curvatura das sondas e o
PREENCHIMENTO do interior dos opacos (a mesma regra no gabarito). O teto de lâmpadas diretas voltou a
subir (6 → 7, medido).

Também abertos (técnicos, sem queixa do dono): as cascatas ainda calculam a grade inteira (81,5 % fora
da janela) e a região C3 pode encolher ao canvas; o G4 (K3) segue com pouca margem (3,98–4,64 % de 5 %,
sozinho); a luz das CASCATAS (céu, ambiente) sai baixa junto a um opaco por ~2 texels grossos (medido
na W5a: 0,17 → 0,01 contra 0,25) — do território do HRC.

Kill-criteria (definidos antes, do relatório): K3 — se o custo entre 1 e 10 000 emissores variar
> 5 %, há um caminho dependente do conteúdo e a W2 não fecha; K6 — se o *bilinear fix* custar > 2× o
RC sem ele, mede-se *nearest-interlaced* antes de aceitar. Regra de duas tentativas: na 2.ª
reconstrução de topologia do solver, PARAR e provar o modelo contra o gabarito.
