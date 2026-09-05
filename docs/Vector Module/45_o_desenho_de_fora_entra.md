# 45 — O desenho de fora ENTRA (importar SVG)

> **Estudo 42, item 3.** *"Devolve vazio hoje; bloqueia todo acervo de artista."*
> Wave de 2026-09-05, linha `line/Vector`, worktree `Worktrees/line-Vector`.

---

## §1 — O que existia, e o que era falso

O app **exportava** uma curva desde 02/09 ([doc 41](41_exportar_svg.md)) e **não sabia ler nenhuma**.
A crate `ph2d-imageio-svg` validava o ficheiro e devolvia `VectorDoc::default()` com o comentário
*"intentionally empty"*; o `.svg` não estava no `SUPPORTED_IMAGE_EXTENSIONS`, então largá-lo na
janela dava *"Skipped: not an image or an Aseprite file"*.

⚠️ **E a metade que existia estava ERRADA.** O cabeçalho do exportador dizia:

> *"Geometria COZIDA, em coordenadas de MUNDO (**Y para baixo, como o SVG**)."*

As duas metades da frase contradizem-se. O mundo do PH2D mede o Y para **cima** —
`ph2d_render::Camera2d::world_to_screen_affine` é `translate ∘ scale(k, **−k**) ∘ translate`, e o
doc dela guarda o report que a fixou (*"mouse e grid descem enquanto sprites sobem"*); o assador de
tiles do Motion escreve a mesma lei por outras palavras (*"sem o `-BAKE_DPI` a estrela assada aponta
para BAIXO"*). O exportador escrevia as coordenadas **cruas** dentro de um `<svg>`, logo **todo
ficheiro exportado saía verticalmente espelhado**.

⭐⭐ **E a lei já estava escrita CERTA em três sítios** — a câmara (`scale(k, −k)`), o assador de
tiles do Motion (`bake_camera`, com o report *"a estrela no grid fica de cabeça para baixo"* no doc)
e o pintor de ícones (`widget_icon`, `scale_non_uniform(s, −s)`, porque a *viewBox* de um ícone é
SVG). O exportador foi o **quarto** consumidor e o único que não a escreveu. *Três cópias certas de
uma lei não impedem a quarta de nascer errada — só uma porta impede.*

⛔ **Ninguém o viu, e a razão é estrutural em duas frentes:**

1. O consumidor era uma **LLM a ler números** — o pedido do Enio foi *"precisamos de um meio de
   exportar o path para que vc possa analisar melhor"*. Quem lê coordenadas não vê uma imagem
   espelhada.
2. **Nenhum dos seis gates media ORIENTAÇÃO.** Eles mediam tinta, pose, marca de balde, a nota do
   cabeçalho, o corte fill/stroke e o gradiente. *Uma família inteira de gates pode ser correcta e
   cega ao mesmo tempo, se nenhum deles fizer a pergunta que falta.*

---

## §2 — A LEI DOS EIXOS tem uma porta

`ph2d-vec-svg` (crate-folha nova) é a dona da tradução **nos dois sentidos**:

| Porta | Quem chama |
|---|---|
| [`svg_to_world(pixels_per_meter)`] | o importador |
| [`world_to_svg(pixels_per_meter)`] | o exportador (`vec_svg_export`) |

São inversas exactas, e há gate a prová-lo (`the_two_directions_of_the_axis_law_are_inverses`) mais
um que atravessa o produto inteiro (`a_drawing_that_goes_out_as_svg_comes_back_the_same_shape`:
exportar → texto → usvg → importar → comparar a forma).

> *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*

⚠️ **A escala é a lei que já existia: um px é um px.** Um `.svg` de 512 unidades entra com o mesmo
tamanho de mundo que um `.png` de 512 px, porque passa pelo mesmo divisor (`pixels_per_meter`).
⛔ Sem ele um ícone de 1024 nasceria com 1024 unidades de largura — cem vezes fora do ecrã — e o
artista concluiria que o import se partiu.

---

## §3 — A armadilha do usvg, e a fixtura que a apanha

O doc do `usvg::Path::data()` diz:

> *"All segments are in absolute coordinates."*

⚠️⚠️ **Ali *absolute* quer dizer COMANDOS absolutos** (o `M` contra o `m` do atributo `d`), **e não
ESPAÇO absoluto.** A prova está no construtor: o `Path::new` guarda `data` intacto e calcula
`abs_bounding_box = bounding_box.transform(abs_transform)` — se os dados já estivessem em espaço
absoluto, essa linha transformaria duas vezes.

⇒ o importador compõe `abs_transform` do nó com a moldura, e **a fixtura do gate tem um `<g
transform>` ANINHADO**: um ficheiro sem transform nenhum lê **igual das duas maneiras**, e teria
deixado a hipótese errada passar.

⭐ **Uma porta leva tudo ao mundo**: [`ph2d_vec_scene::bake_xform`] carrega âncoras, handles,
geometria de gradiente, raio de quina **e a largura do traço** pelo mesmo afim. Escalar a largura à
mão aqui seria a segunda lei — e é exactamente o defeito que o report de 30/08 (*"a proporção muda
no stroke"*) já custou a esta casa.

---

## §4 — O que atravessa, e o que é NOMEADO

| SVG | Documento |
|---|---|
| geometria (com `<g transform>` resolvidos) | `verts` + `subpaths`, cúbicas |
| `Q` (quadrática) | cúbica **exacta** (elevação de grau — identidade algébrica, sem tolerância) |
| `Z` | `closed`, com o vértice repetido **fundido** (o handle de entrada dele passa ao primeiro) |
| `fill` sólido / `linearGradient` / `radialGradient` | `Paint::Solid` / `Linear` / `Radial` |
| `fill-opacity`, `stop-opacity` | multiplicados na cor de cada parada |
| `fill-rule` | `FillRule` |
| `stroke` (largura, `linecap`, `linejoin`) | `StrokeSpec` |
| `stroke-dasharray` | `dash` em **múltiplos da largura** (é a unidade do documento) |
| `opacity` | `VecPath::opacity` — a v19 do schema, do item 2 do mesmo estudo |
| `mix-blend-mode` (16 modos) | `VecPath::blend` — nome exacto nos 22 do W3C |
| `<g id>` | um GRUPO da Hierarquia |

⛔ **O que não atravessa sai NOMEADO** (a lei do importador `.ase`: *um importador que ignora em
silêncio é pior do que um que recusa*), uma linha por espécie com a contagem: `<text>`, `<image>`,
`clip-path`, `mask`, `filter`, `<pattern>`, gradiente no traço, gradiente com `spreadMethod`,
radial com foco deslocado, `dasharray` com mais de dois números, `paint-order=stroke`.

### §4.1 — As duas aproximações, e porque só existem quando são OBSERVÁVEIS

Um grupo com `opacity` compõe-se **inteiro** e só depois desvanece; dar a mesma fracção a cada filho
deixa as sobreposições entre eles à vista. ⭐ **Com UMA forma lá dentro as duas contas são a mesma**
— e é esse o caso comum, porque o usvg embrulha num grupo toda forma que traz `opacity` própria.
⇒ a nota só aparece quando o grupo tem **mais de uma** forma. Idem para o `mix-blend-mode`.

### §4.2 — ⛔ O `<text>` fica de fora, e é uma recusa MEDIDA

Ligar a feature `text` do usvg arrasta `fontdb` + `harfrust` + `skrifa` + três tabelas `unicode-*`, e
**nenhuma delas está no `Cargo.lock`** (medido 2026-09-05) — seria uma **segunda stack de fontes** ao
lado do `parley`, que é o motor de texto desta casa.

⚠️ Com a feature desligada o usvg **apaga** os `<text>` da árvore, então nenhuma travessia os vê: a
contagem sai do XML **cru**, pelo `roxmltree` que o próprio usvg re-exporta. *Uma perda que a árvore
não regista tem de ser lida onde ela ainda existe.*

---

## §5 — Onde o desenho aterra (a metade da shell)

Três leis, e as **três já existiam nesta casa**:

1. **Um px é um px** — o divisor do import de imagens.
2. **Um path ⟺ uma entidade** — quem as cria é o `vec_entities::sync`, chamado no mesmo gesto.
   ⛔ Um segundo criador de entidades seria a porta pela qual um path órfão nasce.
3. **Agrupar é o verbo que já existe** (`vec_entities::group_entities`) — ele põe o grupo entre os
   filhos, compensa a pose de cada um e ordena a lista.

⚠️⚠️ **E a ORDEM entre 2 e 3 é load-bearing:** o `settle_origins` só toca em formas **sem pai** e na
identidade. Agrupar primeiro punha um `ChildOf` em cada uma e elas ficavam **para sempre** com o
pivô na origem do mundo — e o grupo, cuja pose é a média das poses dos membros, nascia lá também,
com o gizmo longe do desenho. É o mesmo defeito que o report do Enio de 30/08 curou para o verbo
*Group*, por outra porta. ⇒ `sync` → `settle_origins` → nomear → agrupar.

⭐ **O ficheiro inteiro vira UM objecto** quando traz mais de uma coisa de topo: sem isso um logótipo
de 40 formas aterra como 40 raízes na Hierarquia, e não há gesto que o mova inteiro.

⭐ **Cada forma leva o `id` que o ficheiro lhe deu**, pela porta do nome ÚNICO — nesta casa o nome é
identidade (a animação reencontra o objecto pelo hash dele).

### §5.1 — UMA porta para o que este app importa

O `.svg` entra como **espécie própria** no `import_router` (`Importables { ase, svg, images,
unknown }`), e por isso o **arrastar-e-largar** e o **File > Import…** o ganham ao mesmo tempo, por
construção. ⛔ Um item de menu *"Import SVG…"* separado teria recriado exactamente o defeito que
aquele módulo existe para matar (Enio, 23/08: *"`.ase` não aparece no dialog de import"*).

⛔ **E o `.svg` NÃO entra no `SUPPORTED_IMAGE_EXTENSIONS`** — se entrasse, viraria uma sprite de
pixels, que é o contrário do que ele é. Há gate.

---

## §6 — Os gates

**Na crate** (`ph2d-vec-svg`, 15):
`a_tip_that_points_up_in_the_file_points_up_in_the_world` ·
`the_two_directions_of_the_axis_law_are_inverses` · `one_pixel_is_one_pixel` ·
`a_nested_transform_places_the_shape_where_the_file_says` · `a_quadratic_becomes_an_exact_cubic` ·
`what_the_file_carries_and_we_do_not_is_named` ·
`a_named_group_becomes_a_group_and_an_invented_one_does_not` ·
`the_files_opacity_and_blend_reach_the_shape` ·
`a_group_opacity_over_many_shapes_is_named_and_over_one_is_not` ·
`the_dash_arrives_in_multiples_of_the_width` · `a_gradient_travels_with_the_geometry` ·
`the_fill_rule_comes_from_the_file` · `an_explicit_return_to_the_start_does_not_leave_a_twin` ·
`something_that_is_not_an_svg_is_refused_not_silently_empty` ·
`the_frame_is_applied_after_the_nodes_own_transform`.

**Na shell** (5 novos + 2 no exportador):
`an_svg_is_a_drawing_and_never_an_image` · `the_drawing_lands_where_the_gesture_asked` ·
`each_shape_carries_the_files_own_name_and_a_group_becomes_a_group` ·
`a_file_with_many_loose_shapes_still_lands_as_one_object` ·
`a_file_with_nothing_drawable_is_refused_and_says_what_it_had` ·
`what_is_up_in_the_world_is_up_in_the_file` ·
`a_drawing_that_goes_out_as_svg_comes_back_the_same_shape`.

**Noutra crate:** `the_two_xml_size_ceilings_are_one_law` (`ph2d-imageio-svg` é a única que vê os
dois tectos de bytes; a dependência é **dev-only**, senão o registo de imagens passaria a depender
do modelo vectorial por causa de uma constante).

⚠️ **Dois gates novos reprovaram primeiro por defeito DELES, sobre produto certo**, e ficam
registados porque a forma repete-se:
- `what_is_up_in_the_world_is_up_in_the_file` comparava `ys[0] < ys[1]` — num vértice de **quina** os
  handles coincidem com a âncora, então o `d` traz o mesmo `y` três vezes seguidas. A régua é o
  **extremo**, não o vizinho.
- o ajudante `ys_do_d` partia em `d="` e apanhava o `data-ph2d-id="`, que **acaba** nessas quatro
  letras: lia o número do id e devolvia lista vazia.

### §6.1 — ⭐⭐ Uma mutação SOBREVIVEU, e o gate é que estava fraco

`a_named_group_becomes_a_group_and_an_invented_one_does_not` ficou **verde** com a guarda do `id`
vazio apagada — ou seja, sobre exactamente a doença que o nome dele promete apanhar.

⚠️ **A fixtura não continha o fenómeno.** Ela era um `<g id="cabeca" opacity="0.5">` com um filho:
ali a opacidade vive no grupo que **já tem id**, então o usvg não fabrica ninguém, e aceitar todo
grupo dá o mesmo resultado. ⇒ a fixtura passou a ter as **duas** espécies — o `<g id>` do artista
**e** um `<rect opacity>` solto, que o usvg embrulha num grupo próprio sem `id` só para compor a
opacidade. Com ela, a mutação morre e a mensagem imprime o intruso:
`SvgGroup { name: "", parent: None }`.

*Um gate cujo nome fala de duas espécies precisa das duas na fixtura.*

**Seis mutações, seis mortas** (a sexta só depois da correcção acima): a lei dos eixos no import ·
o `abs_transform` ignorado · os braços do `Rotate90` trocados · o `settle_origins` removido do
import · a lei dos eixos no export · o grupo fabricado aceite.

---

---

## §7 — O que fica ABERTO

1. **`<text>`** — recusa medida (§4.2). O gatilho que a reabre é o `parley` ganhar uma porta
   «moldar esta string com esta fonte e devolver contornos», que o cozedor de texto do vector já
   pede por outras razões.
2. **`<image>` embutida** — um `.svg` com um `<image>` traz pixels; o documento vectorial não os
   carrega. A saída natural é nascer uma **sprite ao lado**, e isso é decisão de produto (duas
   espécies de objecto de um ficheiro só).
3. **`clip-path` / `mask`** — o documento tem `VecClipContent`, e o §5 do `CLAUDE.md` já regista que
   ele *"não alcança sprites"* e que o `ClipChildren` *"não alcança um caminho"*. Ligar o import ao
   clip pede primeiro fechar esse buraco.
4. **Opacidade de GRUPO a sério** — hoje ela desce para cada forma (§4.1). Exprimi-la exigia uma
   camada por subárvore no renderer, que é a mesma obra que o `blend` de grupo pediria.
5. **`<g>` com um filho só** é achatado, porque o `group_entities` exige dois membros — a mesma
   regra que o artista lê no menu.

---

## §8 — Smoke

```
env PH2D_VEC_SVG_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

⭐ **A cena ESCREVE o próprio `.svg`** (a lei do smoke do `.ase`): ele é escrito em
`/tmp/ph2d_smoke_desenho.svg`, o caminho é impresso — para o poder **arrastar outra vez** ou abrir
noutro programa e comparar — e depois entra pela **mesma porta** que o arrastar-e-largar usa.

O ficheiro carrega, cada coisa a provar uma pergunta: um triângulo com a ponta em cima (a lei dos
eixos), um quadrado dentro de dois `<g transform>` (a pose aninhada), uma curva `Q`, um
`<linearGradient>`, um traço tracejado, um `<g opacity mix-blend-mode>` sobre uma barra, e um
`<text>` — que **não entra**, e cuja ausência tem de aparecer nomeada na consola.

---

## §9 — ⭐⭐⭐ O botão *Rotate CW* girava para a ESQUERDA (achado do mesmo dia)

A lei dos eixos, uma vez construída, torna-se uma **régua para procurar irmãos**. O `grep` por
*"Y para baixo"* devolveu um segundo sítio com a mesma premissa falsa:

```rust
// Screen convention (Y down): CW maps (dx,dy)→(−dy, dx); CCW →(dy, −dx).
Rotate90::Cw => [cx - dy, cy + dx],
```

⚠️ **A conta fecha em duas linhas:** com `Cw`, um ponto à DIREITA do pivô — `(dx, dy) = (1, 0)` —
vai para `(0, 1)`, **para cima**. Direita → cima é anti-horário num eixo Y para cima, e o Y do mundo
é para cima **também no ecrã** (a câmara vira o eixo, então o alto do mundo é o alto da janela).

⛔ E **não há compensação em nenhum dos dois chamadores**: o `vec_rotate_for_id` manda o botão
`VECTOR_ARRANGE_ROTATE_CW` direto para `Rotate90::Cw`.

⇒ os dois braços do `match` trocados, mais um gate novo. ⚠️ **A régua do gate é um ponto às 3 horas,
e não um canto de rectângulo**: num canto as duas coordenadas mudam de sinal e a figura lê-se bem
nas duas direcções — foi isso que deixou o gate que existia (*"o canto `(0,0)` cai em `(7,−3)`"*)
**defender o defeito** durante toda a vida da função. À direita do pivô o ponteiro de um relógio
desce, e num eixo Y para cima descer é `y` a diminuir: não há segunda leitura.

⭐ **O número do gate antigo mudou, e a mudança É a cura** — o canto passa a cair em `(3, 7)`. Duas
mutações confirmam-no: trocar os braços de volta mata `clockwise_means_clockwise_on_the_screen` **e**
`rotate_path_quarter_turn_is_cyclic_and_exact`.
