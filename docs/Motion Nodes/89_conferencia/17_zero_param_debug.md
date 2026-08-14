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

### §0.1 — Os dois `debug.*` ESTÃO no menu do artista, e ninguém decidiu isso

Nenhum dos dois chama `register_ui`. Mas `build_catalog`
(`shells/desktop/src/render_loop/motion_bridge_library.rs:57-70`) itera **`registry.manifests()`** —
*todos* — e faz fallback:

```rust
display:  ui.map(|u| u.display_name).unwrap_or(m.name),
category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
```

⇒ o command-palette (tecla **A**) mostra **`debug.const`** e **`debug.wave`**, com o nome canônico cru,
sob **Utility**, ao lado de `util.reroute`. *A ausência de `register_ui` lê como "escondido" e é o
oposto: é "aparece sem nome de produto".*

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
| `flip_uv` bits 3-4 (repeat) | **`0` cravado** | ⛔ inexprimível (mas o wrap só importa com `uv_xform`, também cravado) |
| **`flip_uv` bits 5-7 (BLEND MODE)** | **o `blend` do sink** (2026-08-13) | ✅ **exprimivel** — param do `motion.output`, argumento dos dois lowerings |
| `uv_xform` (tiling/scroll) | **identidade cravada** | ⛔ inexprimível |
| `anchor` (pivô) | **`[0,0]` cravado** | ⛔ inexprimível (ver a cadeia tentada abaixo) |
| `per_corner_tint` | **branco cravado** | ⛔ inexprimível |
| `z_order` | **`0` cravado** para TODA instância de Motion | **PARCIAL** — ver §1.2b |
| `sampling` (filtro/wrap) | **`0` = herda o default do projeto** | ⛔ inexprimível por-sink |
| `premultiplied` `clip_group` `clip_meta` | cravados | ⛔ (e corretos — não há hierarquia num stream) |

⚠️ **A rota da GPU crava os MESMOS valores** (`crates/ph2d-gpu-cook/src/lower.rs`:
`premultiplied` 0 · `anchor` 0 · `per_corner_tint` 1.0 · `opacity` 1.0 · ~~`flip_uv` 0~~ (**o `flip_uv` deixou de ser cravado em 2026-08-13** — e a licao do resto da linha continua valendo: um conserto tem de tocar os DOIS lados, e foi por isso que o tag virou constante de codegen la em vez de coluna) · `uv_xform`
identidade · `z_order`/`sampling`/`clip` 0) — então **não é um buraco de um caminho, é o contrato do
sink**, e um conserto tem de tocar os dois lados (a paridade CPU×GPU é gate).

---

## §1 — A tabela (colunas fixas da §3 do plano)

### §1.A — EXCLUSIVOS

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.integrate` | 0 | **SUB-STEPS / o timestep exposto.** Blender GN **Simulation Zone** dá **Delta Time** como *input* do nó (`referencia_pesquisa_blender_gn.md:82`, manual 4.5: *"keep the simulation playback consistent when the frame rate changes"*). Houdini: o solver é integrado por um DOP net com **substeps** (`referencia_pesquisa_houdini_mops.md:12` descreve o modelo POP→solver que é o nosso). E **a própria casa já porta a técnica**: `motion.spring` tem sub-step adaptativo `subDt²·tension < 0,05` ([doc 00 §325](../00_estudo_estado_da_arte.md), [doc 01 §195](../01_plano_modulo_motion_nodes.md), [doc 02 §76](../02_dinamica_m2_pesquisa_decisoes.md), *"portar literal"*) | **NÃO.** Tentado: (a) `sim.step` — tem clamp próprio (`MAX_DT = 0,05`) e **não** sub-passa; (b) `motion.spring` sub-passa **só a si mesmo**, não uma cadeia de `force.*`; (c) cozinhar N vezes por tick — o `ticks_owed` do `motion_bridge` é o relógio do `Playhead`, não um knob do grafo; (d) `sim.zone` roda o miolo **uma** vez por tick (não é o Repeat Zone, `referencia_pesquisa_blender_gn.md:51` marca o loop como **FALTA**) | **omissão** | **P1** | `substeps = 1` ⇒ um passo de `dt` = a aritmética de hoje, **bit-idêntica** |
| `motion.integrate` | 0 | **`MAX_DT` é constante escondida — e há DUAS, diferentes.** `motion.integrate` crava `0.1` (`lib.rs:72` + o WGSL `INTEGRATE_MAX_DT`), `sim.step` crava `0.05` (`lib.rs:55` + `STEP_MAX_DT`). Nenhuma traz medição ao lado (§0 do CLAUDE.md) | n/a — não é gap de referência, é **um número sem tabela** | omissão (de medição) | **P2** | — (o item é *medir e escrever o número*, não um param) |
| **`motion.output`** | 1 | **BLEND MODE.** Niagara: o Sprite Renderer desenha com um **Material**, e aditivo e o blend canonico de particula (`referencia_pesquisa_niagara_stardust.md:28`); Cavalry: blend por camada/shader; AE/Stardust: Add e um modo de camada | **SIM — `blend`, o param deste sink (2026-08-13).** O tag e o `ph2d_ecs::BlendMode::tag()`, empacotado nos bits 5-7 do `flip_uv` pelo `pack_blend_bits` que o renderer ja tinha ⇒ **custo de ABI zero**. ⚠️ **NAO pode ser coluna, e a razao e estrutural:** na GPU este no e `GpuKernel::PASSTHROUGH` — o sequenciador nao emite passe para ele —, entao nada que o `eval` escrevesse chegaria ao lowering do device; o tag viaja como ARGUMENTO dos dois lowerings, e na GPU ele e **constante de CODEGEN** (entra na `lower_signature`, sem uniform nem binding). A porta e UMA (`ph2d_eval_motion::sink_blend_tag`): o pump da CPU a pergunta no laco de sinks, a shell a pergunta pelo unico sink que a rota GPU aceita. `Mix` (o default) ⇒ `flip_uv = 0` ⇒ **byte-identico** nas duas rotas | **fechado** | — | cena `=36` |
| **`motion.output`** | 0 | **SORT / ordem de desenho.** Niagara Sprite Renderer tem `SortMode` + binding do atributo de sort; Cavalry ordena por camada | **PARCIAL — e a fronteira é exata.** Medido em `sort_render_order` (`crates/ph2d-render/src/sprite_collect.rs:42-70`): a chave é `(clip_anchor, z_order, texture_id, sampling)` e `sort_by_key` é **estável** ⇒ com `z_order = 0` em todas as instâncias, **dentro de um mesmo `texture_id` a ordem das LINHAS do stream É a ordem de desenho** (⇒ `motion.sort` da family 8 exprime isto hoje). **Através de `texture_id` diferentes a ordem do stream é DERROTADA** — as instâncias reagrupam por textura. É exatamente o *grupo de mídia MISTA* do [doc 86](../86_plano_objetos_engine_render_e_preview.md) | **omissão** (estreita) | **P2** | uma coluna `z_order` ausente ⇒ `0` para todos = hoje |
| **`motion.output`** | 0 | **PIVÔ / anchor.** Niagara Sprite Renderer: *Pivot Offset*; Cavalry: âncora por-cópia | **NÃO (provavelmente).** Tentado: (a) `motion.rotate` gira o **layout** em torno de um centro comum, não cada sprite em torno do próprio pivô; (b) compensar por `P += (I − R)·pivot` precisa do `rot` **do próprio elemento** dentro da fórmula — `motion.expression` (text param, VEX-lite) é o único candidato e **não foi verificado** que ele escreve `P` como vec2; (c) `size`/`uv_rect` não deslocam o centro de rotação | omissão | **P2** | `anchor = [0,0]` = hoje |
| **`motion.output`** | 0 | **FILTRO/sampler por sink.** Niagara: no Material; Cavalry: por layer. Importa em **pixel-art** (nearest) | **NÃO** — `sampling: 0` significa *herdar o default do projeto*, então existe **um** controle global e nenhum por-grafo. ⚠️ Isto NÃO é knob-morto: o artista tem a resposta no projeto | omissão (menor) | **P2** | `sampling = Inherit` = hoje |
| **`motion.output`** | 0 | **SUB-UV / flipbook.** Niagara Sprite Renderer: `SubImageSize` + `SubImageIndex`; Stardust/Cavalry: sprite-sheet | **NÃO.** Tentado: `uv_rect` **é** coluna de stream e o lowering a lê ⇒ animar a célula por-instância daria o flipbook — mas **nenhum nó escreve `uv_rect`**: o único produtor é `source.object` (uma tile fixa por objeto) e `motion.trail`/`motion.duplicator` só a **carregam** | omissão | **P2** | ⚠️ **o item é do `source.object` (family 14), não do sink** — anotado aqui porque foi a medição do lowering que o achou |
| `util.reroute` | 0 | **NADA.** Blender **Reroute**: zero propriedades no corpo e no N-panel — o tipo do socket é inferido, não escolhido. Nuke **Dot**: sem knobs além de label/note. ⚠️ **Citação NEGATIVA, que é o veredito da família** | n/a | **natureza** — um reroute *É* o seu tipo de porta e mais nada ([doc 45 §2](../45_reroute_e_socket_de_entrada_nota_adr.md)) | ⛔ **recusado com motivo** | — |
| `util.reroute_value` | 0 | idem | idem | natureza | ⛔ | — |
| `util.reroute_pulse` | 0 | idem | idem | natureza | ⛔ | — |
| `util.reroute` ×3 | 0 | **O LABEL** (a única coisa que Blender 4.1 acrescentou ao reroute: ele passa a **exibir** o próprio label, que é como um grafo grande fica legível) | ✅ **JÁ TEMOS** — [doc 61](../61_nomes_no_grafo_nota_adr.md): **F2** renomeia o card, e um reroute é um card como qualquer outro | — | ⛔ **fechado** | — |
| `debug.const` | 0 | **O VALOR.** Blender **Value** node (um campo de valor); Houdini **Constant** (valor + tipo) — e o repo afirma que temos: `referencia_pesquisa_houdini_mops.md:88` *"Constant \| valor fixo \| **TEMOS `debug.const`**"* e `referencia_pesquisa_cavalry.md:54` *"Value/Value2/Value3 \| … \| **TEMOS (debug.const, value.\*)**"* | ✅ **SIM, por duas cadeias de UM nó** — (a) **`value.pattern`** com `steps = 1, v0 = K` ⇒ constante K; (b) **`value.map_range`** com `out_lo = out_hi = K` (a entrada desligada lê 0 e cai dentro da faixa) ⇒ K. A CAPACIDADE existe; o que não existe é um nó **chamado** constante | **natureza** (é fixture de teste — o "1º nó" do W1.T3, `docs/plans/2026-05-node-waves.md:20`) | ⛔ **recusado como gap de param** | — |
| `debug.const` | 0 | — | — | — | **P2** | ⚠️ **o item real é de CATÁLOGO** (§0.1): ele aparece no palette como `debug.const`/Utility. Duas saídas: (a) tirá-lo do catálogo (o gesto que falta é um opt-out no `build_catalog`), ou (b) promovê-lo a `value.constant` com um param `v` (default **1.0** ⇒ reduz literalmente ao stream de hoje) |
| `debug.wave` | **1** (`gain`) | **NADA.** Não existe nó equivalente em referência nenhuma: ele é o **template canônico de fan-out** (`docs/IntegracaoMultiAgente/DIRETRIZ.md:358` — *"Templates … `-debug-wave/` (Temporal + ph2d-expr + golden)"*; `examples-fan-out.md:31-32,57`) | n/a | **natureza** — e o `gain` existe para o template **demonstrar** um param, não para o artista | ⛔ | — |
| `debug.wave` | 1 | — | — | — | **P2** | mesmo item de catálogo do `debug.const` |

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
| `motion.duplicator` | 0 | ✅ **SIM.** O modelo é o **Copy to Points** do Houdini (dois inputs, sem params). O Duplicator da **Cavalry** tem muitos knobs — mas esses já estão atribuídos: doc 63 §3 os põe em **`motion.clone` v2** (*"multi-fonte + time offset por clone + step cumulativo"*, **P0**) | — |

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

4. **Uma ordem de desenho que é uma COLUNA, não uma pilha de camadas.** Se `z_order` virar convenção
   de stream, `motion.sort` (que já existe) passa a ordenar o desenho **por qualquer atributo** —
   idade, distância à câmera, `id`, luminância — e isso atravessa `texture_id`, que é exatamente onde
   a ordem do buffer hoje é derrotada. Niagara tem `SortMode` com uma lista fechada; nós teríamos
   *qualquer campo do stream*, porque a ordenação já é um nó.

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
