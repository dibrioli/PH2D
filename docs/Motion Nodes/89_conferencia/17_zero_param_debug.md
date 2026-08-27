# 89 · CONFERÊNCIA — Família 17: ZERO-PARAM + DEBUG (7 exclusivos + 8 também-conferidos)

**Data:** 2026-08-09 · **Plano-mãe:** [89_plano_conferencia_dos_nos.md](../89_plano_conferencia_dos_nos.md) §3/§4
**Exclusivos:** `motion.integrate` · `motion.output` · `util.reroute` · `util.reroute_value` · `util.reroute_pulse` · `debug.const` · `debug.wave`
**Também conferidos (só a metade *"zero params é o contrato deles?"*):** `value.switch` · `pulse.sample_hold` · `motion.combine` · `motion.luminance` · `motion.make_point` · `motion.morph` · `sim.zone` · `motion.duplicator`
**Status:** conferência (claims). Nada implementado, nada priorizado em definitivo (§5/§7 do plano são do Enio).

---

## §0 — O que a família é hoje (lido do `MANIFEST`, não do doc)

| nó | params | efeito | o que ele faz |
|---|---|---|---|
| `motion.integrate` | **0** | Temporal | Euler semi-implícito; consome `accel`, lê `inv_mass`, emite `P/vel/sim_d/sim_t` |
| `motion.output` | **0** | Pure | pass-through; o shell escolhe os nós `motion.output` como sinks do cook |
| `util.reroute` | **0** | Pure | pass-through, `(Instances, Vec2, Frame)` |
| `util.reroute_value` | **0** | Pure | pass-through, `(Instances, Scalar, Frame)` |
| `util.reroute_pulse` | **0** | Pure | pass-through, `(Instances, Scalar, Event)` |
| `debug.const` | **0** | Pure | emite `Stream::new(1).with("v", [1.0])` — o literal `1.0`, hardcoded |
| `debug.wave` | **1** (`gain`) | Temporal | `out = in.v · gain + sin(t)` — o TEMPLATE de fan-out |

⚠️ **A premissa do título é falsa para um deles:** `debug.wave` **tem** um param (`gain`, com hint
desde o [doc 55](../55_uma_regua_nao_pode_ser_funcao_do_que_ela_mede_nota_adr.md)). Os outros seis são
literalmente `params: &[]`, conferidos por leitura do `MANIFEST`.

### §0.1 — ~~Os dois `debug.*` ESTÃO no menu do artista, e ninguém decidiu isso~~ ✅ **FECHADO 2026-08-25**

Nenhum dos dois chama `register_ui`. Mas `build_catalog`
(`shells/desktop/src/render_loop/motion_bridge_library.rs:57-70`) itera **`registry.manifests()`** —
*todos* — e faz fallback:

```rust
display:  ui.map(|u| u.display_name).unwrap_or(m.name),
category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
```

⇒ o command-palette (tecla **A**) mostrava **`debug.const`** e **`debug.wave`**, com o nome canônico
cru, sob **Utility**, ao lado de `util.reroute`. *A ausência de `register_ui` lê como "escondido" e é o
oposto: é "aparece sem nome de produto".*

✅ **CURADO em 2026-08-25**, e a cura tem uma decisão de forma dentro. Os dois nós declaram-se
**fixturas** (`NodeRegistry::register_fixture`, side-metadata como todos os outros canais — fora do
`NodeManifest` congelado), e o `build_catalog` filtra por `!registry.is_fixture(m.id)`: quem responde é
o REGISTO, nunca uma lista escrita na shell (*uma lista à mão ao lado de um predicado é a segunda
resposta à mesma pergunta, e a que o artista vê é a que envelhece*).

⚠️⚠️ **A rota DERIVADA foi tentada primeiro e a MEDIÇÃO refutou-a.** A regra óbvia era *«sem
`NodeUiManifest` ⇒ não é do artista»* — zero declarações novas. O censo dos 130 tipos registados
devolveu **três** sem manifesto de UI, e o terceiro era o **`pulse.signal`**: um nó de artista (o último
item aberto da folha 12) a que ninguém deu nome, que aparecia na paleta como `pulse.signal`, cinzento.
⇒ *«não tem metadados de UI» quer dizer «é fixtura» **e também** «alguém esqueceu», e as duas leem
igual.* Com o opt-in explícito, esquecer põe o nó na paleta com o nome cru — **visível**; com a regra
derivada, esquecer fá-lo-ia **desaparecer em silêncio**, que é o pior dos dois.

⇒ **Três achados, não um:** os dois `debug.*` saem do catálogo, e o `pulse.signal` ganha
`NodeUiManifest` (*Signal*, categoria `Output`). O censo virou gate permanente:
`shells/desktop/tests/every_offered_node_has_a_name_and_every_fixture_has_none.rs`.

### §0.2 — O que o LOWERING deixa passar do grafo para a tela (a medição que decide o `motion.output`)

`lower_to_instances_onto` (`crates/ph2d-eval-motion/src/lower.rs:41-131`) lê **sete** coisas do stream
e **crava as outras dez em identidade**. A coluna da direita é a conferência de expressibilidade:

| campo do `RenderInstance` | de onde vem hoje | exprimível pelo grafo? |
|---|---|---|
| `world_pos` `size` `atlas_uv` `tint` `basis` | colunas `P` `size` `uv_rect` `tint` `rot` | ✅ |
| `texture_id` | coluna `texture_id` (doc 86) | ✅ |
| *(vetor)* `geometry_id` | coluna `geometry_id` (ADR-0154) | ✅ |
| `opacity` | **`1.0` cravado** | ✅ **por outra via** — `tint.a` ([doc 51](../51_fade_de_verdade_opacity_nota_adr.md), canal Opacity do `motion.drive`) |
| `flip_uv` bits 0-1 (flip x/y) | **`0` cravado** | ✅ **por outra via** — `size` negativo espelha o quad (o shader aplica `basis · (corner·size + anchor)`) |
| `flip_uv` bit 2 (tint_fill) | **`0` cravado** | ✅ **por outra via** — `WHITE_TILE_KEY` + `tint` |
| `flip_uv` bits 3-4 (repeat) | **`0` cravado** | ⛔ inexprimível — e a razão de continuar a ser é MEDIDA: com `uv_xform` a escalar para DENTRO de `[0,1]` (que é o que o sub-UV faz), as três leis de wrap concordam ⇒ um knob de repetição no sink seria **morto** até alguém pedir ladrilho (`scale > 1`) |
| **`flip_uv` bits 5-7 (BLEND MODE)** | **o `blend` do sink** (2026-08-13) | ✅ **exprimivel** — param do `motion.output`, argumento dos dois lowerings |
| `uv_xform` (tiling/scroll) | **a coluna `uv_cell`** (2026-08-25) | ✅ **exprimível** — o `motion.sub_uv` escreve-a, e ela é RELATIVA ao ladrilho da linha |
| `anchor` (pivô) | **o `pivot_x`/`pivot_y` do sink** (2026-08-25) | ✅ **exprimível** — fracção do `size` de cada linha, argumento dos dois lowerings |
| `per_corner_tint` | **branco cravado** | ⛔ inexprimível |
| `z_order` | **`0` cravado** para TODA instância de Motion — e continua | ✅ **exprimível pela SUB-ORDEM** (2026-08-25, [ADR-0070-amendment-9](../../architecture/decisions/0070-amendment-9.md)): o `z_order` fica em `0` de propósito (é o rank da HIERARQUIA), e quem ordena as linhas é o `sub_order`, lido logo a seguir |
| `sampling` (filtro/wrap) | **o `filter` do sink** (2026-08-25) | ✅ **exprimível** — 7 modos, o teto DERIVADO do `FilterMode::from_tag`; o `repeat` fica em `Inherit` (ver acima) |
| `premultiplied` `clip_group` `clip_meta` | cravados | ⛔ (e corretos — não há hierarquia num stream) |

⚠️ **A rota da GPU cravava os MESMOS valores** — *"não é um buraco de um caminho, é o contrato do
sink"*, e um conserto tem de tocar os dois lados (a paridade CPU×GPU é gate). ✅ **Em 2026-08-25 os
quatro campos desta wave passaram a viajar nas DUAS rotas por UMA porta**
(`ph2d_eval_motion::sink_style` → `ph2d_render::SinkStyle`, o tipo mora no `ph2d-render` porque o
`ph2d-gpu-cook` mantém o `ph2d-eval-motion` como dep de DEV de propósito). Na GPU eles são **constantes
de codegen** e entram na `lower_signature`, senão o cache de pipelines serviria a fonte do estilo
anterior. O que continua cravado lá: `premultiplied` 0 · `per_corner_tint` 1.0 · `opacity` 1.0 ·
`z_order` 0 · `clip` 0.

⚠️ **E o `PLAIN` mantém a assinatura BYTE-a-byte**: se o estilo neutro passasse a hashear, toda
combinação de colunas ganharia chave nova e a 1.ª corrida depois da wave recompilaria todos os módulos
— sem erro, visível só como um engasgo. Há gate (`the_plain_style_keeps_the_signature_every_document_already_had`).

---

## §1 — A tabela (colunas fixas da §3 do plano)

### §1.A — EXCLUSIVOS

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.integrate` | 0 | **SUB-STEPS / o timestep exposto.** Blender GN **Simulation Zone** dá **Delta Time** como *input* do nó (`referencia_pesquisa_blender_gn.md:82`, manual 4.5: *"keep the simulation playback consistent when the frame rate changes"*). Houdini: o solver é integrado por um DOP net com **substeps** (`referencia_pesquisa_houdini_mops.md:12` descreve o modelo POP→solver que é o nosso). E **a própria casa já porta a técnica**: `motion.spring` tem sub-step adaptativo `subDt²·tension < 0,05` ([doc 00 §325](../00_estudo_estado_da_arte.md), [doc 01 §195](../01_plano_modulo_motion_nodes.md), [doc 02 §76](../02_dinamica_m2_pesquisa_decisoes.md), *"portar literal"*) | ~~**NÃO.** Tentado: (a) `sim.step` — tem clamp próprio (`MAX_DT = 0,05`) e **não** sub-passa; (b) `motion.spring` sub-passa **só a si mesmo**, não uma cadeia de `force.*`; (c) cozinhar N vezes por tick — o `ticks_owed` do `motion_bridge` é o relógio do `Playhead`, não um knob do grafo; (d) `sim.zone` roda o miolo **uma** vez por tick~~ — ✅ **FECHADO em 2026-08-19.** ⚠️ **As rotas (c) e (d) caíram em 2026-08-12 e a célula nunca foi reconferida** (§0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*): o motor do sub-tique aterrou na folha 13 (`Cook::substep`) e **nada nele sabe o que é uma zona** — ele subdivide o **playhead**, e o `dt` deste nó é `playhead − sim_t`. Faltava só o nó **declarar**: a convenção é de MANIFESTO (`SUBSTEPS_PARAM`), então o param novo é a feature inteira, **zero linha de motor**. Medido: com força constante o erro cai **exactamente pela metade** a cada dobra (0,3333 → 0,1667 → 0,0834 → 0,0417 → 0,0208 · razão **2,000×**, Euler de 1ª ordem) e com um `force.wind` a rajar dentro do tique a resposta **ANDA** (21,715 → 21,535 → 21,423 em 1/4/16) — é a cadeia de `force.*` a ser re-cozinhada, que é o que a referência chama substep e o que um laço dentro do `eval` não pode dar. **O teto é 64 e a tabela está no `MAX_SUBSTEPS`** (a 16.384 elementos `sub=64` custa 24,03 ms na CPU de referência = 144% de um quadro; o device marcha o plano pela mesma porta) — ⚠️ **o mesmo número da `sim.zone`, por CORREÇÃO:** o ritmo é do grafo, então um teto menor aqui seria contornado por uma zona ao lado. Faixa confortável **1..16** (erro 0,10%). **4 gates + 3 mutações, 3 sangram.** ⚠️ **UMA banda na cena, de propósito:** dois integradores a pedir `1` e `16` correriam **os dois a 16** — o que separa as leituras é o slider. Cena `=61` | **fechado** | ✅ | `substeps = 1` ⇒ um passo de `dt` = a aritmética de hoje, **bit-idêntica** (gate ao bit, com controle) |
| `motion.integrate` | 0 | **`MAX_DT` é constante escondida — e há DUAS, diferentes.** `motion.integrate` crava `0.1` (`lib.rs:72` + o WGSL `INTEGRATE_MAX_DT`), `sim.step` crava `0.05` (`lib.rs:55` + `STEP_MAX_DT`). Nenhuma traz medição ao lado (§0 do CLAUDE.md) | n/a — não é gap de referência, é **um número sem tabela** | omissão (de medição) | ~~P2~~ ✅ **FECHADO 2026-08-23** | — (o item é *medir e escrever o número*, não um param) ✅ **MEDIDO e curado (bloco Z, [doc 91](../91_os_tetos_que_ninguem_mediu.md) §5).** ⚠️ **A `0,1` ele não guardava NADA:** o laço fechado real (`motion.grid → motion.integrate`, `pre` de volta por uma `force.attractor`) atira uma grelha nascida em raio `1,0` a **127,19** — com uma força que se alcança ARRASTANDO. O irmão `sim.step`, a `0,05`, segurava a MESMA cena em `2,49`: *o dissidente era o número mais certo dos dois*. Os dois passam ao joelho medido, **`0,03`**, e o critério está escrito — *um passo legítimo não muda a RESPOSTA, só a resolução*. ⚠️ A busca tem de ser por **prefixo-máximo**: a excursão é RESSONANTE em `dt` (`0,0325` mede 3,57 e `0,0333` mede 0,89), então o primeiro cruzamento é uma ressonância e não a fronteira. ⛔ O `motion.spring` NÃO entra (ele deriva três tetos do dele); ⏳ `motion.boids` e `motion.wave` ficam por medir. |
| **`motion.output`** | 1 | **BLEND MODE.** Niagara: o Sprite Renderer desenha com um **Material**, e aditivo e o blend canonico de particula (`referencia_pesquisa_niagara_stardust.md:28`); Cavalry: blend por camada/shader; AE/Stardust: Add e um modo de camada | **SIM — `blend`, o param deste sink (2026-08-13).** O tag e o `ph2d_ecs::BlendMode::tag()`, empacotado nos bits 5-7 do `flip_uv` pelo `pack_blend_bits` que o renderer ja tinha ⇒ **custo de ABI zero**. ⚠️ **NAO pode ser coluna, e a razao e estrutural:** na GPU este no e `GpuKernel::PASSTHROUGH` — o sequenciador nao emite passe para ele —, entao nada que o `eval` escrevesse chegaria ao lowering do device; o tag viaja como ARGUMENTO dos dois lowerings, e na GPU ele e **constante de CODEGEN** (entra na `lower_signature`, sem uniform nem binding). A porta e UMA (`ph2d_eval_motion::sink_blend_tag`): o pump da CPU a pergunta no laco de sinks, a shell a pergunta pelo unico sink que a rota GPU aceita. `Mix` (o default) ⇒ `flip_uv = 0` ⇒ **byte-identico** nas duas rotas | **fechado** | — | cena `=36` |
| **`motion.output`** | 0 | **SORT / ordem de desenho.** Niagara Sprite Renderer tem `SortMode` + binding do atributo de sort; Cavalry ordena por camada | **PARCIAL — e a fronteira é exata.** Medido em `sort_render_order`: a chave é `(clip_anchor, z_order, texture_id, sampling)` e `sort_by_key` é **estável** ⇒ com `z_order = 0` em todas as instâncias, **dentro de um mesmo `texture_id` a ordem das LINHAS do stream É a ordem de desenho** (⇒ `motion.sort` da family 8 exprime isto hoje). **Através de `texture_id` diferentes a ordem do stream é DERROTADA** — as instâncias reagrupam por textura. É exatamente o *grupo de mídia MISTA* do [doc 86](../86_plano_objetos_engine_render_e_preview.md) | **omissão** (estreita) | ~~P2~~ ✅ **FECHADO 2026-08-25** | param `sort = Texture` = hoje ✅ **O param `sort` do sink, e o preço foi PAGO NA FUNDAÇÃO ([ADR-0070-amendment-9](../../architecture/decisions/0070-amendment-9.md)).** ⚠️ **As duas saídas óbvias foram medidas e são as DUAS erradas:** dar às linhas `z_order = i` **espalha o bloco pelo espaço de ranks da CENA** (o `z_order` dela é um contador de DFS denso, e as partículas passariam a interpenetrar as sprites da hierarquia); pôr uma BASE acima de toda a cena faz o grafo **saltar para a frente de tudo** ao ligar um knob de *ordenação*; e tirar o `texture_id` da chave regride os draw calls da cena INTEIRA para servir um caso. ⭐ *A grandeza que faltava não era «mais fundo», era «mais à frente **dentro do mesmo fundo**»* — e é isso que o `RenderInstance::sub_order` é: um campo CPU-only no tail, `0` em tudo o que a cena extrai (⇒ ordenação byte-idêntica), lido logo a seguir ao `z_order`. ⚠️ **O preço é draw calls, e ele é o próprio pedido**: honrar a ordem de um stream que alterna texturas obriga a um run por linha. ⚠️ **A sub-ordem é o índice na FILEIRA, não no buffer** — vários sinks compõem no mesmo `out`, e um contador global faria o 2.º sink desenhar sempre por cima do 1.º |
| **`motion.output`** | 0 | **PIVÔ / anchor.** Niagara Sprite Renderer: *Pivot Offset*; Cavalry: âncora por-cópia | **NÃO (provavelmente).** Tentado: (a) `motion.rotate` gira o **layout** em torno de um centro comum, não cada sprite em torno do próprio pivô; (b) compensar por `P += (I − R)·pivot` precisa do `rot` **do próprio elemento** dentro da fórmula; (c) `size`/`uv_rect` não deslocam o centro de rotação | omissão | ~~P2~~ ✅ **FECHADO 2026-08-25** | `pivot_x`/`pivot_y` = `0` = hoje ✅ **Dois params do sink, e a UNIDADE é a decisão: FRACÇÃO do `size` do próprio elemento, nunca metros.** Um stream tem um tamanho por linha, e um pivô em metros deslocaria as peças pequenas de outra maneira que as grandes — a conversão vive numa função só (`SinkStyle::anchor_for`), com o gate a medi-la sobre uma fixtura de tamanhos MISTOS (uma fixtura uniforme não distingue as duas leis). ⚠️ **O campo `RenderInstance::anchor` já existia e já era honrado pelo shader** (`local = anchor + quad·size`, ANTES do `basis`) — o que faltava era quem o escrevesse. ⚠️ **O teto (`±1`) é da MOLDURA DE CULL**, não da aritmética: o renderer decide o que desenhar pelo `world_pos`, e mais longe que um tamanho inteiro a peça pode sumir estando visível. ⭐⭐ **E ele vale para TODO tipo de objecto** (veredito do Enio, 2026-08-25: *«o sistema deve ser compatível com todos os tipos de objetos como vector e flip e no futuro 3d»*): a `VectorInstance` ganha o mesmo `anchor`, pela MESMA função (`SinkStyle::anchor_for`), e o encoder do passe vectorial põe-no **entre o `basis` e o `size`** — somá-lo depois faria a peça girar no centro e apenas deslocar-se, o mesmo desenho para todo ângulo, que é precisamente o que um pivô NÃO é. ⚠️ **A resposta honesta separa os quatro campos:** pivô e ordem são GEOMETRIA e valem em qualquer rota; filtro e sub-UV só existem onde há IMAGEM. A separação é **declarada** em `ph2d_render::StyleReach` (uma entrada por rota, com o motivo obrigatório de cada ausência) e um gate recusa uma rota que não a declare — é o que impede o 3D de nascer a ignorar os quatro em silêncio. ⚠️ E a ORDEM já era universal sem uma linha nova: o `draw_shared_instances` encoda na ordem do iterador e **nunca reagrupa por forma**, então num vector a ordem das linhas é sempre a ordem de desenho — o `sort` do sink existe para a rota das sprites, onde o desempate por textura a derrotava. ⛔ A ordem ENTRE as duas rotas continua inexprimível (são dois passes), e isso está nomeado |
| **`motion.output`** | 0 | **FILTRO/sampler por sink.** Niagara: no Material; Cavalry: por layer. Importa em **pixel-art** (nearest) | **NÃO** — `sampling: 0` significa *herdar o default do projeto*, então existe **um** controle global e nenhum por-grafo | omissão (menor) | ~~P2~~ ✅ **FECHADO 2026-08-25** | `filter = Project` = hoje ✅ **O param `filter`, com os SETE modos que o renderer sabe amostrar** — e a contagem é **derivada**, não escrita: o `ph2d_render::image_filter::FILTER_TAG_MAX` é verificado contra o `FilterMode::from_tag` (o tag é concreto, o seguinte cai no fallback) e o gate da shell exige a lista de rótulos do mesmo tamanho. Menos rótulos ⇒ um modo inalcançável; mais ⇒ um item de menu que a porta clampa de volta. ⚠️ **O `repeat` fica FORA, e por medição**: com o `uv_xform` a escalar para dentro de `[0,1]` as três leis de wrap concordam ⇒ o knob seria morto até alguém pedir ladrilho. ⛔⛔ **E o smoke desta célula achou um DEFEITO DE PRODUTO pré-existente, app-wide:** o `material_bg` do `renderer_draw` honrava a `sampling` **só para o átlas partilhado**; para toda textura INDIVIDUAL ele devolvia um grupo construído contra o sampler *default do projecto* ⇒ **o filtro por-nó do Inspector (§9) estava inerte em toda textura individual**, e uma sprite promovida a Individual por um `commit_edited_texture` perdia o filtro dela em silêncio. ⚠️ *E o caso que o expôs é aquele para que o filtro EXISTE* — pixel-art chega por importação, e portanto quase nunca está no átlas. ✅ Curado: cada entry ganha uma cache de bind groups por-amostragem (o gémeo do `atlas_sampler_bgs`), **na loja e não no renderer**, porque ela referencia a `view` daquela textura e tem de morrer com ela. Gate de PIXEL com adapter real (`individual_texture_honours_its_sampling`), com a amostra tirada FORA do centro — no centro exacto os dois modos medem a mesma média e o gate passaria sobre o defeito. ⛔ A loja **cozida** (KTX2) fica de fora, nomeada |
| **`motion.output`** | 0 | **SUB-UV / flipbook.** Niagara Sprite Renderer: `SubImageSize` + `SubImageIndex`; Stardust/Cavalry: sprite-sheet | **NÃO.** Tentado: `uv_rect` **é** coluna de stream e o lowering a lê ⇒ animar a célula por-instância daria o flipbook — mas **nenhum nó escreve `uv_rect`**: o único produtor é `source.object` (uma tile fixa por objeto) | omissão | ~~P2~~ ✅ **FECHADO 2026-08-25** | sem o nó ⇒ a identidade `[1,1,0,0]` = hoje ✅ **NÓ NOVO: o `motion.sub_uv`**, e a cadeia tentada nomeou a razão pela qual ele NÃO escreve `uv_rect`: aquela coluna é o rectângulo **ABSOLUTO** no atlas, e quem sabe qual é o ladrilho de uma linha é o `source.object` — ou, sem objecto, a **shell**, que só o fornece no instante do lowering. ⭐ Ele escreve a coluna **`uv_cell`** (`[escala_u, escala_v, desloc_u, desloc_v]`), **RELATIVA**, que os dois lowerings depositam no `RenderInstance::uv_xform` — o transform que o shader **já** aplica DENTRO do sub-rect da própria sprite. ⇒ a célula **compõe** com o ladrilho em vez de o substituir, e o mesmo grafo serve o atlas partilhado e a textura individual de um objecto. ⚠️ A ordem das células é a da CASA (linha-maior, `col = k % cols`) — por colunas dá uma folha bonita e **todas as animações trocadas**. ⚠️ O embrulho é `rem_euclid` e o WGSL não o tem: um `%` lá daria a célula errada exactamente onde um `stagger` negativo vive. Params: `cols`/`rows`/`cell`/`speed`/`stagger` + uma porta `cell` com a escada de sempre (vazia ⇒ o param · 1 ⇒ broadcast · n ⇒ por elemento) |
| `util.reroute` | 0 | **NADA.** Blender **Reroute**: zero propriedades no corpo e no N-panel — o tipo do socket é inferido, não escolhido. Nuke **Dot**: sem knobs além de label/note. ⚠️ **Citação NEGATIVA, que é o veredito da família** | n/a | **natureza** — um reroute *É* o seu tipo de porta e mais nada ([doc 45 §2](../45_reroute_e_socket_de_entrada_nota_adr.md)) | ⛔ **recusado com motivo** | — |
| `util.reroute_value` | 0 | idem | idem | natureza | ⛔ | — |
| `util.reroute_pulse` | 0 | idem | idem | natureza | ⛔ | — |
| `util.reroute` ×3 | 0 | **O LABEL** (a única coisa que Blender 4.1 acrescentou ao reroute: ele passa a **exibir** o próprio label, que é como um grafo grande fica legível) | ✅ **JÁ TEMOS** — [doc 61](../61_nomes_no_grafo_nota_adr.md): **F2** renomeia o card, e um reroute é um card como qualquer outro | — | ⛔ **fechado** | — |
| `debug.const` | 0 | **O VALOR.** Blender **Value** node (um campo de valor); Houdini **Constant** (valor + tipo) — e o repo afirma que temos: `referencia_pesquisa_houdini_mops.md:88` *"Constant \| valor fixo \| **TEMOS `debug.const`**"* e `referencia_pesquisa_cavalry.md:54` *"Value/Value2/Value3 \| … \| **TEMOS (debug.const, value.\*)**"* | ✅ **SIM, por duas cadeias de UM nó** — (a) **`value.pattern`** com `steps = 1, v0 = K` ⇒ constante K; (b) **`value.map_range`** com `out_lo = out_hi = K` (a entrada desligada lê 0 e cai dentro da faixa) ⇒ K. A CAPACIDADE existe; o que não existe é um nó **chamado** constante | **natureza** (é fixture de teste — o "1º nó" do W1.T3, `docs/plans/2026-05-node-waves.md:20`) | ⛔ **recusado como gap de param** | — |
| `debug.const` | 0 | — | — | — | ~~P2~~ ✅ **FECHADO 2026-08-25** | ⚠️ **o item real é de CATÁLOGO** (§0.1). ✅ Saída **(a)**, e por medição: a capacidade *«uma constante»* já existe por duas cadeias de UM nó, então o que faltava não era um nó — era ele deixar de ser **oferecido**. `NodeRegistry::register_fixture`, e o `build_catalog` filtra pelo REGISTO. ⚠️ A rota derivada («sem `NodeUiManifest` ⇒ não é do artista») foi **refutada pelo censo** — leia a §0.1 antes de a repropor |
| `debug.wave` | **1** (`gain`) | **NADA.** Não existe nó equivalente em referência nenhuma: ele é o **template canônico de fan-out** (`docs/IntegracaoMultiAgente/DIRETRIZ.md:358` — *"Templates … `-debug-wave/` (Temporal + ph2d-expr + golden)"*; `examples-fan-out.md:31-32,57`) | n/a | **natureza** — e o `gain` existe para o template **demonstrar** um param, não para o artista | ⛔ | — |
| `debug.wave` | 1 | — | — | — | ~~P2~~ ✅ **FECHADO 2026-08-25** | mesmo item de catálogo do `debug.const` — e a mesma cura |

**Contagem (DERIVADA, reconciliada em 2026-08-25):** 15 linhas — **P0 = 0** · **P1 = 0** · **P2 = 0** · ✅ fechadas **9** · ⛔ recusadas/refutadas **6**. ⭐⭐⭐ **A FOLHA FECHOU.** Em 2026-08-23 o bloco Z fechou a célula das duas constantes escondidas (e a medição virou a leitura dela do avesso: **a `0,1` o grampo não guardava nada**, 127× o raio de nascimento, e o dissidente a `0,05` era o mais certo). Em **2026-08-25** fecharam as seis restantes de uma vez — *o sink não sabia dizer nada sobre COMO desenhar*: as quatro do `motion.output` (pivô · filtro · ordem · sub-UV) mais as duas de catálogo. ⚠️ **Três das quatro do sink são params que viajam por UMA porta** (`sink_style` → `SinkStyle`, argumento dos dois lowerings); a quarta é uma **coluna** (`uv_cell`) e um **nó novo** (`motion.sub_uv`), porque uma célula é por-elemento e um param do sink não é. ⚠️ **A da ordem custou fundação**: [ADR-0070-amendment-9](../../architecture/decisions/0070-amendment-9.md), o `RenderInstance::sub_order`. Cena de smoke: **`PH2D_MOTION_OBJ_SMOKE=9`** — ⚠️ **não** um nível de `PH2D_GPU_COOK_DEMO`, e a razão é medida: aqueles demos amostram um ladrilho **branco chapado**, sobre o qual o filtro, o sub-UV e a mídia mista são todos invisíveis.

Re-medir: `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` — ⚠️ **esta linha é DERIVADA da coluna `P` da tabela acima; não a edite à mão** (a contagem desta conferência envelheceu SEIS vezes, e a folha 13 chegou a contradizer a própria prosa três parágrafos abaixo).

### §1.B — TAMBÉM CONFERIDOS NOUTRA FAMÍLIA (só a metade *"zero params é o contrato deles?"*)

| nó | params hoje | zero params é o contrato? (referência CITADA) | resíduo honesto |
|---|---|---|---|
| `value.switch` | 0 | ✅ **SIM.** O `select` é um **input de VALOR, não um param** — e é o ponto inteiro (doc 12/17: o seletor **anima**). Blender **Index Switch** (4.1) e **Menu Switch**: o índice é um *socket*. TD **Switch CHOP** / Nuke **Switch**: o `which` é dirigível | ⚠️ **DOC 63 ERROU** — ver §3. E o resíduo real vs Blender é a **lista DINÂMICA de sockets** (add/remove item): a nossa é fixa em **4** (`in0..in3`), pela mesma razão do doc 45 (`inputs` é `&'static`, contrato congelado) |
| `pulse.sample_hold` | 0 | ✅ **SIM.** Os dois operandos (`value`, `pulse`) são inputs. Max **`sah~`**, TD **Hold CHOP**: sem knobs além do gatilho | — |
| `motion.combine` | 0 | ✅ **SIM.** Houdini **Merge** SOP e Blender **Join Geometry** não têm parâmetro nenhum | **cap de 4 inputs** (`in0..in3`) contra o Merge **variádico** do Houdini — de novo o `&'static` do contrato |
| `motion.luminance` | 0 | **QUASE.** Blender **RGB to BW** não tem controle; AE idem. ⚠️ Mas **Nuke tem** um dropdown *luminance math* (Rec709 / Ccir601 / Average / Maximum) no `Saturation`/`ColorMatrix` — e o nosso crava Rec. 709 no código (`0.2126/0.7152/0.0722`) | ⚠️ citação **externa** (os refs do repo não cobrem cor do Nuke). Um enum `weights` com default **Rec709** reduziria literalmente. **P2** |
| `motion.make_point` | 0 | ✅ **SIM.** Blender **Combine XYZ**: só sockets, zero propriedades | — |
| `motion.morph` | 0 | ⚠️ **QUASE.** O `blend` é input (certo — anima). Mas o **Mix** do Blender TEM propriedades (`Data Type`, `Factor Mode` Uniform/Non-Uniform, `Clamp Factor`, `Clamp Result`) | ⚠️ **E há um resíduo MAIOR que um param, medido noutro módulo desta casa:** o `motion.morph` pareia os dois streams **por ORDEM DE LINHA** (`min` das contagens). O **Tween v2 da `line/FLIP`** (integrado 2026-07-23, [doc 11 do Flip](../../Flip/11_tween_v2.md)) mediu exatamente essa escolha: correspondência **ordinal** foi trocada por **custo geométrico + atribuição ótima (Hungarian)**, e o lerp de coordenadas foi trocado por **espiral logarítmica** porque *"um lerp corta pela CORDA, então todo giro encolhia o traço"*. O `motion.morph` tem hoje as duas doenças que aquela linha já curou. **Não é param — é o motor**, e é o achado mais transferível desta conferência |
| `sim.zone` | 0 | ✅ **SIM, e a leitura fina importa:** o que a Blender Simulation Zone oferece **são SOCKETS, não propriedades** — **Delta Time** (`referencia_pesquisa_blender_gn.md:82`) e **Skip** (`:83`, *"repassa o estado de entrada direto ao output ignorando o miolo"*; 4.3 **escondeu o checkbox** porque o clicavam sem querer, deixando só o socket) | ⚠️ **conferido no `MANIFEST`: `inputs = [init, state]`** — **não há `skip`, não há `delta_time`**. Zero params está certo; o que falta são **duas portas**. E o **Repeat Zone** (Iterations + Inspection Index) já está marcado **FALTA** em `:51`. Item da family 13 |
| `motion.duplicator` | 3 (`pick`·`seed`·`point_scale`) | ✅ **SIM.** O modelo é o **Copy to Points** do Houdini (dois inputs, sem params). O Duplicator da **Cavalry** tem muitos knobs — mas esses já estão atribuídos: doc 63 §3 os põe em **`motion.clone` v2** (*"multi-fonte + time offset por clone + step cumulativo"*, **P0**) | — |

---

## §2 — `SUPERAR:` (o que nenhuma referência tem, derivado do que só nós temos)

1. **O blend do sink é DIRIGÍVEL, e nas referências ele não é.** Em Niagara o blend mora no
   *Material* (asset, trocado por variante) e na Cavalry na *camada*. Aqui, se `blend` nascer como
   **param do `motion.output`**, o [doc 58](../58_params_dirigidos_nota_adr.md) o torna **uma aresta**
   de graça: um `pulse.beat` ou um `value.step` troca `Normal → Add` no compasso. *Nenhuma das
   referências pode animar a categoria de blend sem trocar de asset.* E se nascer como **coluna de
   stream**, o renderer já roteia por-instância (`compute_runs` chaveia em `unpack_blend`) — o que dá
   **blend por PARTÍCULA**, que Niagara não tem (um emitter = um material).

2. **Sub-step BIT-EXATO sob scrub.** O sub-step do Houdini/Niagara é um acumulador: rebobinar
   re-simula e **diverge**. O nosso integrador já é reprodutível por checkpoint (`Cook::checkpoint`
   /`restore`, GGPO, M2.N2) e o `dt` é **derivado do `sim_t` que o próprio estado carrega** — então
   `substeps = N` continua sendo função pura do par `(playhead, estado)` e o scrub para trás continua
   bit-exato. *Sub-step com scrub exato é uma combinação que nenhuma das referências entrega.*

3. **O reroute é o único que já ganhou tudo de graça, e é o modelo a citar.** Doc 45 fez do ponto um
   **nó**; doc 61 deu-lhe **nome**; o `Cook::peek`/readout inline (doc 43) faz o dot **mostrar o que
   passa por ele**. Blender e Nuke têm o dot; nenhum dos dois mostra o *valor* nele. É a prova de que
   *"conferir e não achar gap"* também é resultado — e de onde a próxima capacidade barata sai.

4. ✅ **ENTREGUE em 2026-08-25 — e por uma rota melhor que a que este item propunha.**
   *"Uma ordem de desenho que é uma COLUNA, não uma pilha de camadas"*: com `sort = Stream` no sink,
   a ordem das LINHAS é a ordem de desenho, e ela atravessa `texture_id` — que é exactamente onde o
   buffer era derrotado. ⭐ Como a ordenação **já é um nó** (`motion.sort`), o que se ordena é
   **qualquer atributo** — idade, distância à câmera, `id`, luminância —, contra a lista FECHADA de
   `SortMode` do Niagara. ⚠️ **E o item propunha uma coluna `z_order`, que teria sido pior**: o
   `z_order` é o rank da HIERARQUIA, e escrevê-lo por linha espalha o bloco de Motion pelo espaço de
   ranks da cena (as partículas interpenetram as sprites). O campo que faltava era uma SUB-ordem
   dentro da fatia — [ADR-0070-amendment-9](../../architecture/decisions/0070-amendment-9.md). *Um
   `SUPERAR:` pode nomear a capacidade certa e o mecanismo errado; o que se guarda dele é a
   capacidade.*

---

## §3 — `CERCAS:` (as decisões já registradas que encontrei — grepadas antes de propor)

- **[Doc 45 §2](../45_reroute_e_socket_de_entrada_nota_adr.md) — por que TRÊS reroutes e não um genérico:**
  *"`NodeManifest.inputs/outputs` são `&'static` (contrato congelado) … descongelar o contrato pra
  economizar dois `const` seria um péssimo negócio"*. **Não proponha um reroute genérico.** A mesma
  cerca explica o cap de 4 inputs do `value.switch` e do `motion.combine`.
- **[Doc 45 §1](../45_reroute_e_socket_de_entrada_nota_adr.md) — o waypoint DECORATIVO foi deletado**
  (o doc 44 está superseditado). *"Era um afordance que mentia sobre a própria capacidade."* Não
  reintroduza um bend sem nó.
- **[Doc 61](../61_nomes_no_grafo_nota_adr.md) — o nome do nó JÁ EXISTE (F2)**, e a §2 dele é uma
  correção pública de uma varredura que afirmou o contrário por grep. *Antes de dizer "não dá para
  nomear", rode o seam.*
- **[Doc 51](../51_fade_de_verdade_opacity_nota_adr.md) — opacidade é `tint.a`**, canal 4 do
  `motion.drive`, com clamp em [0,1] deliberado. Não proponha um campo `opacity` separado no sink.
- **`motion.integrate` — `Effect::Temporal` é decisão registrada no próprio arquivo** (`lib.rs:105-111`):
  *"Convention: reads playhead ⇒ Temporal"*, porque sob `Pure` um re-cook no mesmo tick com playhead
  movido (checkpoint/restore) devolveria `sim_t` obsoleto.
- **`motion.integrate` — `inv_mass` ausente ⇒ `1.0`, e `·1.0` é exato** (`lib.rs:75-80`): todo grafo
  pré-pin integra **bit-idêntico**. É o molde de "default que reduz literalmente" a copiar.
- **`motion.output` — o sink é ESCOLHIDO pelo tipo, não por um toggle** (`lib.rs:1-11`): *"a
  Material-Output / render node, not a hidden toggle"*, e vários `motion.output` **compõem** (o
  `lower_to_instances_onto` **acrescenta**, `lower.rs:37-40`). Um param "enabled" seria a 2ª porta do
  bypass **H** que já existe.
- **ADR-0155 / `Coupling::Consumes("accel")`** (`motion-integrate/src/lib.rs:398-406`): o diagnóstico
  do grafo **DERIVA** os papéis daqui. Um param novo no integrador que mude *o que ele consome*
  quebra o `ph2d-motion-diagnose` em silêncio.
- **ADR-0130 / `register_dense_window`** (`lib.rs:416-417`): a rota de GPU do integrador reivindica a
  janela densa de ids. Qualquer coisa que reordene ou reescreva `id` recua para a CPU.
- **[Doc 86](../86_plano_objetos_engine_render_e_preview.md) — `texture_id` é CONVENÇÃO de stream,
  não campo do `NodeManifest`**, *"fallback 0 ⇒ byte-idêntico"*. É **o precedente exato** para
  `blend`/`z_order`/`anchor`: o contrato congelado não é tocado.

---

## §4 — `O DOC 63 ERROU EM:`

| item do doc 63 | o que ele diz | o que está no `main` hoje |
|---|---|---|
| **§3, linha 121** — `value.index_switch` | *"N entradas por inteiro \| Blender 4.1 \| **P1**"* (isto é, FALTA) | ⚠️ **EXISTE**: `value.switch` tem `select` + `in0..in3` e escolhe por `clamp(round(select), 0, N-1)`. É literalmente o Index Switch. O resíduo honesto é a lista **dinâmica** de sockets do Blender vs os nossos 4 fixos — e isso é a cerca do doc 45, não um nó a construir. *Mandar construir o que já existe é o custo exato que a §1 do plano 89 nomeia.* |
| **`referencia_pesquisa_niagara_stardust.md:114`** — *"Sprite renderer + orient \| billboard/along-velocity \| **TEMOS** (instâncias 2D nativas)"* | TEMOS | ⚠️ **Meia-verdade.** Temos a *geometria* (quad instanciado com `basis` de rotação). **Não** temos nada do resto do render module: blend, sort, pivô, sub-UV e filtro estão **todos cravados em identidade** nas duas rotas de lowering (§0.2). Marcar o renderer como TEMOS por causa do quad é a leitura que manteve este buraco invisível. |
| **`referencia_pesquisa_houdini_mops.md:88`** — *"Constant \| valor fixo \| TEMOS `debug.const`"* | TEMOS | ⚠️ **Certo pelo motivo errado.** O `debug.const` emite o literal **1.0** e não tem param nenhum — ele não é um "Constant". A capacidade existe mesmo, mas por `value.pattern`/`value.map_range` (§1.A). Creditar a um fixture de teste é como o fixture acabou no catálogo do artista sem ninguém decidir. |
| **`referencia_pesquisa_cavalry.md:54`** — *"Value/Value2/Value3 … TEMOS (debug.const, value.\*)"* | TEMOS | ⚠️ Mesma coisa: a metade `value.*` é verdadeira, a metade `debug.const` não. |
| **doc 63 §Baseline (linha 6)** — *"87 nós reais · 318 params"* | baseline | ℹ️ envelheceu no sentido bom: o censo `param_census` de 2026-08-09 diz **118 nós · 411-420 params**. Não é erro — é a razão de a §1 do plano 89 mandar **conferir** o doc 63 em vez de confiar nele. |

---

## §5 — Placar da família

- **15 nós conferidos** (7 exclusivos + 8 da metade estreita).
- **9 CONFIRMADOS como zero-param por contrato, com referência** — os três `util.reroute` (citação
  **negativa**: Blender Reroute e Nuke Dot não têm propriedade nenhuma) · `pulse.sample_hold` ·
  `motion.combine` · `motion.make_point` · `value.switch` · `sim.zone` · `motion.duplicator`.
- **2 recusados como gap de param mas com item de CATÁLOGO** — `debug.const` e `debug.wave`
  (fixtures/template visíveis no palette do artista).
- **2 com resíduo pequeno e citado** — `motion.luminance` (o *luminance math* do Nuke) e
  `motion.morph` (as propriedades do Mix do Blender — **e o motor de correspondência, que é o item
  grande**).
- **2 viraram gap REAL** — **`motion.output`** (blend ~~**P0**~~ ✅ **FECHADO em 2026-08-13**,
  cena `=36`; sort/pivô/filtro/sub-UV P2) e
  **`motion.integrate`** (sub-steps **P1**; os dois `MAX_DT` sem medição P2).

⚠️ **O veredito em duas linhas, para a §10 do plano:** *o `motion.output` é magro por **OMISSÃO**, e a
omissão tem nome exato — a ponte grafo→render lê sete colunas e crava dez campos, entre eles o
**blend**, cuja máquina de roteamento por-instância já está construída e testada no renderer.* Um
grafo de Motion hoje **não consegue fazer uma faísca aditiva**, e o conserto custa **zero ABI** e
**zero contrato congelado** (é a convenção de stream do `texture_id`, ou um param no sink).
