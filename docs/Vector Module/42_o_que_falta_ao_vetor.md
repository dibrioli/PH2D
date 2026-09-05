# 42 — O que falta ao Vetor (PH2D × Illustrator × Rive, 2026-09-04)

> **Estudo a pedido do Enio** (2026-09-04): *"Avalie as ferramentas de desenho que temos e as
> ferramentas que o Illustrator e o Rive têm e que nós ainda não temos, mas que seria importante ter
> já que seremos a mais poderosa game engine do mundo."*
>
> **Página do estudo (a versão que o Enio lê):**
> <https://claude.ai/code/artifact/1e36eff5-5341-4b94-90c0-19e3d7e53ac5>
>
> ⚠️ **Um estudo PROPÕE, não decide** — a ordem do §5 é recomendação, não fila autorizada.
> ⚠️ **Este doc descreve o mundo no dia em que foi escrito.** O estado vivo é o `CLAUDE.md` §5.
>
> **Antecessor:** [`20_pesquisa_ferramentas_de_artista.md`](20_pesquisa_ferramentas_de_artista.md)
> (2026-07) fez a mesma pergunta contra Inkscape/Cavalry/CorelDRAW e produziu a espinha dos Live
> Effects. **A Faixa A dela FECHOU inteira** (efeitos como pilha, gizmo de quina, texto em caminho,
> trim, repeater, blend, largura variável) — este doc é a pergunta seguinte, e a resposta mudou de
> terreno: já não falta ferramenta de desenho, falta o desenho ganhar vida.

---

## §0 — O método, e por que ele importa aqui

| Lado | Como foi levantado |
|---|---|
| **PH2D** | Capacidade a capacidade **no código da linha**, com `file:line` por afirmação. ⛔ Nenhuma nota de doc foi aceite sem conferência — e a §4 mostra o preço disso. |
| **Illustrator** | Página oficial *Learn about Illustrator tools* (11/03/2025, **89 ferramentas**), notas de versão 28.x–30.0, páginas de brushes/symbols/SVG (27/10/2025). ⚠️ O `helpx.adobe.com` bloqueia fetch; tudo foi lido por snapshot do Wayback com a data de *Last updated* registada. |
| **Rive** | `rive.app/docs` (índice `llms.txt`), changelog (só serve até 28/02/2025 — depois migrou para a comunidade), blog de releases, GitHub. |

⛔ **Fora do levantamento:** preço, licença e ecossistema de plugins.

---

## §1 — O terreno: o que o PH2D TEM (verificado)

**16 modos** (`crates/ph2d-tool-vector/src/params_mode.rs:146-163`, gate
`the_list_and_the_enum_agree_on_the_population` a afirmar `16`) · **47 formas**
(`crates/ph2d-vec-scene/src/kind.rs:98`) · **41 secções de painel**
(`crates/ph2d-panel-vector/src/paint_sections.rs:242-362`) · **10 efeitos de caminho**
(`crates/ph2d-vec-scene/src/effect.rs:177-213`) · **15 filtros raster**
(`crates/ph2d-fx-op/src/kinds.rs:119-369`).

| Família | O que existe |
|---|---|
| **Desenhar** | Pen · Pencil (RDP + ajuste de Hobby) · Shape (47) · Text (fontes variáveis, texto em caminho, wrap) · Frame · Connect |
| **Editar** | Node (+ laço) · Fillet · Chamfer · Width (perfil vivo) · Trim · Cut (a lâmina é um OBJETO) · Weld · Bucket |
| **Construir** | Booleanas **vivas** (8 verbos + um verbo por forma) · Build (Shape Builder) · Simetria viva (espelho/radial/fuse) · Repeat (grade/radial/spin/orbit) · Blend (2..=5 formas) · Envelope (Perspective/Coons/**MLS-rigid** = puppet warp) + 9 presets de warp |
| **Pintar** | Solid · Linear · Radial · **MultiPoint** (freeform IDW) · Pattern (fill **e** stroke) · **Brush** (arte que percorre a linha, com quinas) · dash · 8 markers · caps/joins/align |
| **Efeitos de linha** | Trim · ZigZag · Repeat · Bloat · Warp · **Falloff** · Twist · **Knot** · **Sketch** · **Hatch** |
| **Filtros** | Blur · Glow · Drop/Inner Shadow · Inner Glow · Outline · **Feather** · Bevel · Color Overlay · Turbulence · Grow/Shrink · Color Adjust · Duotone · Luma→Alpha · Gradient Map |
| **Sistema** | Componentes + instâncias + **variantes** · Estados + Smart Animate · Máquina de estados do morph · Auto Layout (`taffy`) · Constraints de âncora · Clip content · Grupos · Tokens de design · Guias/régua · **Export SVG** |
| **Ligado ao jogo** | Input Map (ações nomeadas) · Signals · estados disparados por sinal · física/partículas/pintura/3D **no mesmo app** |

---

## §2 — A matriz (40 capacidades)

`✓` tem · `~` parcial · `—` não tem.

### §2.1 — Só nós temos, ou temos melhor (11)

| Capacidade | PH2D | AI | Rive |
|---|---|---|---|
| Booleanas **vivas** (peças continuam editáveis) | ✓ | ~ compound | — |
| Efeitos de caminho encadeáveis e reversíveis | ✓ 10 | ~ appearance | ~ só trim |
| Filtros de imagem sobre o vetor | ✓ 15 | ~ rasteriza | — |
| Formas paramétricas | ✓ 47 | ✓ ~10 | ~ 4 |
| Largura variável no traço | ✓ | ✓ | — |
| Padrão e pincel **no traço** | ✓ | ✓ | — |
| Envelope / puppet warp | ✓ | ✓ | — |
| Simetria viva enquanto se desenha | ✓ | ~ repeat | — |
| Balde por região que sobrevive à edição | ✓ | ✓ | — |
| Conectores que grudam e seguem | ✓ | — | — |
| Trim/Weld/Cut como objetos vivos | ✓ | ~ destrutivo | — |

⚠️ **A booleana é a assimetria mais forte da tabela, e ela tem um MOTIVO do outro lado:** o Rive
nunca as fez, e a razão declarada pela equipa é **custo em tempo de execução**. ⇒ se o vetor for ao
runtime, a booleana viva tem de levar a **medição** ao lado — é cerca de Chesterton alheia, e é a
única desta lista que vem com um argumento técnico.

### §2.2 — Eles têm, nós não (24)

| Capacidade | PH2D | AI | Rive | Evidência da ausência |
|---|---|---|---|---|
| **Ossos + pesos** no vetor | — | — | ✓ | grep `VecBone`/`bone_weight` = vazio; os 6 `ph2d-node-rig-*` deformam nuvem de pontos do grafo |
| **IK** no vetor | — | — | ✓ | `ph2d-node-rig-ik-2bone`/`-fabrik` são do motion graph |
| Constraints (follow path, distância, look-at) | ~ | — | ✓ 7 | `motion.path`/`look-at` no grafo; no documento só `VecConnector`/`VecTextPath` |
| Timeline anima **cor/traço/efeito** | — | — | ✓ | `ph2d-timeline/src/prop.rs` tem 13 `PropKind`, nenhum de tinta/largura/efeito |
| ⚠️ Timeline `Opacity` num `VecPath` | **INERTE** | ✓ | ✓ | `apply_prop.rs:73` exige `ph2d_render::Sprite` |
| **N fills + N strokes** por forma | — | ✓ | ✓ | `VecPath` tem `fill: Option<Paint>` e `stroke: Option<StrokeSpec>` (`ph2d-vec-scene/src/lib.rs:436`) |
| **Blend modes** por forma | — | ✓ 16 | ✓ 16 | grep `blend_mode` em `crates/ph2d-vec-*` = **vazio** |
| Opacidade do OBJETO | ~ | ✓ | ✓ | só alfa da tinta + `ObjectPose.opacity` (`ph2d-ui-state/src/pose.rs:44`) |
| **Data binding** tipado (listas, converters) | ~ tokens | — | ✓ | `BoundProp` = `{Fill, StrokeColor, StrokeWidth, LayoutGapMain, LayoutGapCross}` (`ph2d-ecs/src/vec_bindings.rs:80-90`) |
| Binding de um **text run** | — | ✓ | ✓ | `BoundProp` não tem alvo de texto |
| **Listeners** (clique/hover em runtime) | — | — | ✓ | grep `VecListener`/`pointer_event` = vazio; só `ui_preview_gesture.rs` no editor |
| Eventos + áudio pela animação | ~ sinais | — | ✓ | `ph2d-runtime::Signal` existe; sem canal de áudio autorado |
| Texto: **estilos por trecho** | — | ✓ | ✓ | `VecTextParams` tem UM `family/size/weight/align/axes` (`ph2d-ecs/src/vec_shape.rs:28-59`) |
| Texto: **kerning/ligaduras** | — | ✓ | ✓ | `vec_glyph.rs:291` — *"o `parley` NÃO decide isto… este cozedor é advance-only por desenho"* |
| Campo de texto editável em jogo | — | — | ~ | Rive: `TextInput` "coming soon" |
| Modificadores de texto (onda letra a letra) | — | ~ | ✓ | Rive: ranges + falloff + follow path |
| **Vetorizar imagem** (image trace) | — | ✓ | ~ só malha | grep `image_trace`/`vectorize`/`potrace` = vazio (⛔ `ph2d-trace` é do quad-remesh 3D) |
| Deformar foto por **malha** | — | ~ envelope | ✓ | Rive: mesh + Auto-Trace (2026) |
| Pincéis de **arte** e **dispersão** | ~ só padrão | ✓ 5 tipos | — | `StrokePaint::{Solid,Pattern,Brush}` (`stroke_style.rs:188`) |
| Malha de gradiente | ~ multi-ponto | ✓ | — | ⛔ **recusa nossa**, ver §4.3 |
| Borda suave **sem rasterizar** | ~ rasteriza | ~ rasteriza | ✓ | Rive: feathering analítico (integral da normal na cobertura) |
| **9-slice / N-slicing** no vetor | — | ~ símbolos | ✓ | `ph2d-ecs/src/slice_nine.rs` é do **sprite** |
| **Importar SVG** | — | ✓ | ✓ | ⛔ `crates/ph2d-imageio-svg/src/lib.rs:14-17` — *"the VectorDoc body is **intentionally empty**"* |
| Máscara por opacidade/luminância | — | ✓ | ~ clip | só `VecClipContent` (recorte geométrico) |
| Solo (só um filho visível) | — | — | ✓ | — |
| Ordem de desenho **por regra** | — | — | ✓ | — |
| Scripts do artista na ferramenta | — | ✓ | ✓ | Rive: Luau com 9 protocolos, incl. **Path Effect** e **WGSL shader** |
| Recolorir a arte por paleta | — | ✓ | — | Recolor Artwork + Generative Recolor |

### §2.3 — Ninguém tem (5) — onde um MOTOR ganha sozinho

| Capacidade | PH2D | AI | Rive |
|---|---|---|---|
| A forma desenhada vira **corpo de física** | — | — | — |
| Luz 2D que acende o vetor | — | — | — |
| Partículas nativas ligadas à forma | ~ nos nós | — | — |
| Exportar folha de sprites do vetor | — | ~ por prancheta | — |
| Pintar à mão e vetorizar **no mesmo canvas** | ~ dois módulos | — | — |

⚠️ **Vetor → collider:** `ColliderShape::{Ball, Cuboid, Capsule}` (`ph2d-physics-ecs/src/components.rs:98-116`);
grep `ColliderShape::Polygon` = vazio. ⛔ **A ADR-0063 é letra morta** — a ADR-0131 §12 rejeitou-a
**por estar amarrada ao vector-runtime que a ADR-0108 aposentou**. *O motivo morreu; a capacidade
nunca foi julgada pelo próprio mérito* (§0.0 do `CLAUDE.md`: quem move o número que tornava algo
inalcançável tem de reconferir a nota).

---

## §3 — As lacunas, por retorno ÷ esforço

| # | Lacuna | Tam. | Por que agora |
|---|---|---|---|
| **1** | **A timeline não anima o desenho** | **P** | Ela move um `VecPath` de 3 maneiras (pose, trajetória, morph `t`) e mais nada. ⭐ **O interpolador JÁ EXISTE e é testado**: o `ObjectPose` do Smart Animate interpola `fill` (incl. gradientes e padrões), `stroke`, `geometry`, `width profile` e `filters` (`ph2d-ui-state/src/pose.rs:36-96`, `transition.rs:274-330`). Falta a timeline **alcançá-lo**. Maior distância esforço↔retorno da lista |
| **2** | **Ossos no vetor** | **G** | Sem eles um personagem só anima como recorte de papel. Temos a matemática de deformar curva por alças (`ph2d-vec-envelope/src/mls.rs`) — é o mesmo problema com outra UI e um binding por vértice. É a aposta inteira do Rive |
| **3** | **N fills + N strokes** | **M** | Cada camada de estilo exige HOJE duplicar o objeto. Mudança de data model; paga-se em todo o resto |
| **4** | **Blend modes + opacidade por objeto** | **P** | Existe no pipeline de sprite (`ph2d_ecs::BlendMode`) e **não chega ao encode Vello**. Correção visual mais barata que há |
| **5** | **A casca de jogo** | **G** | ⛔ **Decisão do Enio, adiada por escrito.** `ls shells/` = só `desktop`; a shell é `[[bin]]` sem `[lib]`. Ela sozinha destrava listeners, data binding, contextos de Input Map e o morph em jogo — **4 linhas da §2.2 de uma vez** |
| **6** | **Texto a sério** (kerning + estilos por trecho) | **M** | O cozedor é advance-only; o `parley` está na árvore e não é usado para moldar. Aparece na 1.ª tela de UI de qualquer jogo |
| **7** | **Importar SVG** | **P** | Devolve documento vazio hoje. Enquanto durar, nenhum acervo de artista entra |
| **8** | **Vetorizar rascunho** | **P/M** | `vtracer` é Rust puro e já está nomeado no manual (`Estudos/PH2D_manual_features_vetoriais.md` §6). Fecha o ciclo papel→motor e casa com o Painter |
| **9** | **Pincéis de arte e dispersão** | **M** | Temos o Pattern brush; faltam o *Art* (estica a arte de ponta a ponta) e o *Scatter* (espalha cópias) |
| **10** | **Vetor → collider** | **M** | O único item que **ninguém** tem. Ver a cerca da §2.3 |

---

## §4 — O que não se reconstrói (recusas alheias e nossas)

### §4.1 — Adobe

| Item | O que aconteceu |
|---|---|
| Ferramentas de **gráfico** (9) | Paradas há 20+ anos; o estilo volta ao default quando os dados mudam |
| **Perspective Grid** | Desde CS5 (2010) e os tutoriais ensinam a *não* usar: *"imprecise, buggy, and asks a lot of the user"*; há como perder planos irreversivelmente |
| **Symbol Sprayer** | O próprio admin da Adobe (UserVoice, 07/02/2024): *"introduced in version 10 in 2001, and was never really improved since then"* |
| **Creative Cloud Charts** | Anunciado em 19.0 (jun/2015), **removido em 19.2** (nov/2015) — seis meses |
| **Gradient Mesh** | Mantido, mas o **Freeform Gradient** (23.0, 2018) foi lançado por a malha ser *"much more complicated and time-consuming"* |
| Exportação **SWF**, **FXG**, painel **Kuler**, apps **Draw/Sketch** | Todos removidos/encerrados |

### §4.2 — Rive

| Item | O que aconteceu |
|---|---|
| ⭐ **Booleanas** | Nunca existiram; razão declarada = **custo em runtime**. O Shape Builder (jul/2026) faz união/subtração **destrutiva** |
| **Stroke inside/outside** | Só centrado; "planned", **bloqueado pela ausência de path ops** |
| **Flare → Rive 2** | Reescrita total; perderam-se **filtros raster** e **jelly bones** (os filtros nunca voltaram — o substituto de 2025 é o feathering vetorial) |
| **Renderer** | Skia removido do Android (v10); renderer próprio (PLS) desde 2024 |
| **Import Lottie** | Só Enterprise e a ser retirado: *"isn't the workflow Rive was built for"* |
| **Rive GameKit (Flutter)** | Preview técnico de 2023, silenciosamente abandonado; a ponte Rust/Bevy também estagnou |
| **Sem** pincéis, quadro-a-quadro, física, partículas, luz 2D, tilemaps, mixer de áudio, sprite sheet | Assumidos fora de escopo até 2026 (brush e frame-by-frame "via scripting") |
| **Exportar virou pago** | out/2025 — equipas reclamaram no meio de projeto |

### §4.3 — Nossas

- ⛔ **Malha de gradiente** — recusada em [`13_fora_de_escopo.md` §13.9](13_fora_de_escopo.md) em favor de
  **diffusion curves**; o `Paint::MultiPoint` cobre boa parte. Reverter exige ADR amendment.
- ⛔ **Vector Networks** (topologia do Figma) — avaliado em [`20 §2.4`](20_pesquisa_ferramentas_de_artista.md):
  refactor de fundação para ganho de workflow. Faixa C.
- ⛔ **Multi-line text** ([`13 §13.11`](13_fora_de_escopo.md)) — ⚠️ **a nota envelheceu**: o `wrap_width`
  existe (`ph2d-ecs/src/vec_shape.rs:59`) e há `wrapped_lines`.
- ⛔ **ExtendScript/CEP** ([`13 §13.6`](13_fora_de_escopo.md)) — o scripting do artista, se vier, é
  Luau/WASM/MCP. ⚠️ O Rive escolheu **Luau** em 2026 pela mesma razão que o nosso doc dá.

---

## §5 — Recomendação (um estudo PROPÕE)

> **Não precisamos de mais ferramentas de desenho.** Nesse terreno ganhamos do Rive com folga e
> empatamos com o Illustrator no que um artista de jogo de facto usa (Shape Builder, booleanas,
> quinas vivas, largura variável, balde por região, padrões no traço). O que temos é **uma
> ferramenta de ilustração muito boa presa dentro de um motor**.

| Ordem | O quê | Por quê |
|---|---|---|
| 1º | Animar tinta/traço/efeito na **timeline** | O interpolador já existe nos Estados; falta a ponte. Quase de graça |
| 2º | **Blend modes + opacidade** por forma | A correção visual mais barata, e a mais visível numa cena montada |
| 3º | **Importar SVG** | Devolve vazio hoje; bloqueia todo acervo de artista |
| 4º | **N fills + N strokes** | Estrutural; destrava estilizar sem duplicar objetos |
| 5º | **Ossos no vetor** | Separa ilustração de animação de personagem |
| depois | **A casca de jogo** | Decisão do Enio. Destrava 4 linhas da matriz de uma vez |

---

## §6 — ⚠️ DEZ afirmações dos nossos docs que o código desmente

*A auditoria que produziu o §1 mediu isto de caminho. Cada uma faria alguém construir o que existe
ou procurar no sítio errado.*

| # | O doc diz | O código diz |
|---|---|---|
| 1 | *"13 modos"* (README §1) / *"14 modos"* (`CLAUDE.md` §5) | **16** (`params_mode.rs:146-163` + gate `assert_eq!(vistos, 16)`) |
| 2 | *"1 de 39 secções consultava o modo"* | São **41**, e **todas** passam pela tabela de escopo |
| 3 | — | **Um dos 16 modos não tem pílula nem rótulo i18n**: o `PickBlend` só existe atrás do botão hardcoded `"Pick Shapes"` (`paint_blend.rs:44`) |
| 4 | README lista o **hit-test do mapa fundido** como ABERTO | **Fechado** (`App::vec_live_drawn` + 6 sítios de pick em `input_dispatch.rs`) |
| 5 | README lista **F5/E4/C2/D1** como abertos | Os quatro fecharam; sobra a **D2** (partículas) |
| 6 | Dois docs citam **`DRAG_RATE_X = 50`** como número vivo | O símbolo **não existe** no repo (grep vazio) |
| 7 | `ph2d-imageio-svg` na árvore sugere **import de SVG** | Devolve `VectorDoc::default()` — *"intentionally empty"* |
| 8 | Seis crates `ph2d-node-rig-*` sugerem **bones no vetor** | Deformam nuvem de pontos do grafo; `ph2d-vec-scene/src/lib.rs:12` promete *"Rig/bones… entram na Fase 1"* — não entraram |
| 9 | A timeline parece **animar tudo** | Num `VecPath`: pose + trajetória + morph `t`. A track `Opacity` é **inerte** (`apply_prop.rs:73`) |
| 10 | *"Clip content"* lê-se como máscara universal | `VecClipContent` **não alcança sprites** e o `ClipChildren` **não alcança um caminho** (`vec_clip_content.rs:25-28`) — *uma moldura vetorial não recorta uma imagem* |

*Bónus:* o README diz *"os 26 bugs estão fechados"*; são **27**, e **zero abertos**
([`BUGS_vector.md:9`](BUGS_vector.md)).
