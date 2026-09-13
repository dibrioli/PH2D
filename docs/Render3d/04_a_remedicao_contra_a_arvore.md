# 04 — A remedição do plano contra a árvore de 13/09

> O estudo (`00`–`03`) é de **09/09** e não tinha nada construído. Antes de pegar a `W1`, cada
> afirmação dele sobre o que *já existe* foi conferida contra o código de 13/09 — e **três caíram**.
> *Uma lista de «temos hoje» escrita sem correr nada envelhece no dia em que é escrita.*

⚠️ Nada aqui está no produto: são **medições** e a **ordem** que elas impõem. Os instrumentos vivem
em [`ferramentas/`](ferramentas/) e reproduzem cada número.

## §1 — ⛔ As três premissas que a árvore desmentiu

| o estudo dizia | a árvore diz | onde |
|---|---|---|
| ingrediente 1 (gestão de cor) *«⚠️ meio: há `tonemap.wgsl`»* | ⛔ **zero, não meio.** `BYPASS_LUT = true` — o passe devolve `clamp(hdr, 0, 1)`; o bake AgX **nunca existiu** (`tools/bake_agx_lut/` continua TBD); e o caminho do LUT com a identidade aplica uma curva log **errada** (o «dull look» recusado na ronda 7 do M14.5). *Ter o ficheiro não é ter o ingrediente.* | `ph2d-render/src/shaders/tonemap.wgsl` |
| `W1` *«o fundamento barato»*, primeiro e sozinho | ⛔ **não tem consumidor honesto** — ver a tabela abaixo | §1.1 |
| *«⚠️ Não há gerador de WGSL em 1.39.5»* (`00` §3) | ⛔⛔ **há.** `PyMaterialXGenGlsl` traz `WgslShaderGenerator` **e** `VkShaderGenerator`. O censo do estudo listou **módulos** (`GenGlsl · GenMsl · GenOsl · GenMdl · GenSlang`), e a classe vive dentro de um deles | §2 |

### §1.1 — Por que a `W1` sozinha não se mostra: os consumidores, um a um

| quem pinta 3D | onde a imagem entra | é HDR? | a `W1` muda o quê |
|---|---|---|---|
| **o modelador** (`ph2d-field-render::shade`) | CPU → **RGBA8 sRGB** pré-multiplicado → `draw_stable_image` no passe do **chrome** (Vello), **depois** do tonemap | ⛔ não: é **matcap**, uma fotografia de luz, e a doutrina do módulo diz *«o matcap é do OLHO, o rig é do DOCUMENTO»* (`ph2d-field-render/src/lib.rs`) | exposição e AgX sobre uma fotografia LDR — **cosmético**, e o AgX apaga-a |
| **a escultura** (`ph2d-mesh-render`) | GPU → `game_rt` (`Rgba16Float`) com `LoadOp::Load` sobre a cena 2D → o tonemap **global** | ✅ sim: rig até `intensity 2` mais o realce **somado** (`mesh.wgsl`: *«quem faz o roll-off é o tonemap»*) — e o bypass **corta em 1** | curaria um corte real — ⚠️ mas é da `line/sculpt3d`, **viva em 13/09** |
| **a arte 2D** (sprites · Flip · glow do Motion · emissiva) | o **mesmo** `game_rt` | só a de 16 bits | ⛔ acender o tonemap global **muda a arte 2D** — e o gate `tonemap_descent_gpu` exige passagem byte-exacta |

⇒ ⭐ **o modelo certo é o do Godot:** a transformação de vista pertence à **cena 3D**, e o canvas 2D
fica *display-referred*. Uma vista 3D escreve já **transformada** (valores `≤ 1` em luz de ecrã) no
`game_rt`, e o bypass de hoje passa-a intacta. Nada no passe global muda.

## §2 — ⭐⭐⭐ A ponte para WGSL, MEDIDA (a 1.ª medição da `W2`)

O `open_pbr_surface` (41 entradas, `/usr/share/materialx`) gerado pelos três geradores GLSL
([`gerar_openpbr.py`](ferramentas/gerar_openpbr.py)) e levado ao `naga 29.0.4` — o mesmo major do
`wgpu` desta árvore — pela [`naga_probe`](ferramentas/naga_probe/):

| gerador | GLSL | rota | resultado |
|---|---|---|---|
| ⭐ **`WgslShaderGenerator`** | `#version 450`, textura e sampler **separados** · pixel `2 195` linhas | **A:** `naga` `glsl-in` | ✅ **FRONTEND OK (`5,7 ms`) · VALIDAÇÃO OK · WGSL `5 598` linhas, `118` funções, `9` globais**; vértice `206` linhas ✅; e o WGSL escrito **volta a entrar** (`5 626` linhas, validado) |
| `WgslShaderGenerator` | o mesmo | **B:** `glslangValidator -V` → SPIR-V (`spirv-val` ✅, `177 992` B) → `naga` `spv-in` | ⛔ fragmento: `VALIDAÇÃO ERRO — EntryPoint Fragment main, Argument(0, NotIOShareableType)`; vértice ✅ |
| `VkShaderGenerator` | `#version 450` · `2 190` linhas | A | ⛔ `Not implemented: variable qualifier` |
| `VkShaderGenerator` | o mesmo | B | ⛔ `spv-in`: `invalid id %2050` |
| `GlslShaderGenerator` | `#version 400` · `2 177` linhas (o número do estudo) | A | ⛔ três erros: versão `400` · bloco sem `layout(binding)` · `variable qualifier` |

⇒ **a rota é a A**, e o terceiro candidato do plano (escrever um `GenWgsl`) **não é preciso**.

⚠️ **O que esta tabela NÃO prova, e é a régua da `W2`:**
- **validar no `naga` prova que o módulo é WGSL legal, nunca que ele devolve os pixels do
  `MaterialXView`** — o oráculo continua a ser o render dele, comparado por passo;
- o próprio gerador avisa duas vezes *«WGSL does not allow boolean types to be stored in uniform or
  storage address spaces»* — o `naga` validou, e **não foi medido** o que ele fez aos `bool`;
- nem o *layout* de bindings contra um `wgpu::RenderPipeline` real, nem o custo por pixel.

⚠️ **O `glsl-in` puxa o `pp-rs`, que o workspace não tem.** ⇒ a tradução é um **bake** (script +
WGSL commitado com cabeçalho: versão do MaterialX, do `naga`, e o hash da nodedef), e o CI valida o
ficheiro com o `wgsl-in` que **já** é dependência (`ph2d-render`, `ph2d-gpu-cook`). *O CI não
precisa do MaterialX para provar que o WGSL é legal; precisa dele — localmente — quem o regenera.*

## §3 — ⭐⭐ O ORÁCULO DE COR, corrido sem interface

OpenColorIO **2.5.1 (BSD-3-Clause)** sobre o `config.ocio` do Blender 5.2, pela ligação Python
([`oraculo_de_cor.py`](ferramentas/oraculo_de_cor.py)). Saída em sRGB de ecrã:

| linear → | `0` | `0,01` | `0,18` | `0,5` | `1` | `2` | `4` | `8` | `16` | vermelho `(4; 0,2; 0,1)` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `Standard` | 0 | 0,0999 | 0,4614 | 0,7354 | 1,0 | 1,3533 | 1,8248 | 2,4542 | 3,2944 | `1,825 · 0,485 · 0,349` |
| **`AgX`** | 0 | 0,0725 | **0,4613** | 0,6652 | 0,7710 | 0,8518 | 0,9137 | 0,9616 | 0,9985 | `0,991 · 0,646 · 0,594` |
| **`Khronos PBR Neutral`** | 0,0003 | 0,0081 | 0,4113 | 0,7107 | 0,9364 | 0,9819 | 0,9925 | 0,9966 | 0,9983 | `0,993 · 0,611 · 0,596` |
| `ACES 2.0` | 0 | 0,0092 | 0,3492 | 0,5655 | 0,7067 | 0,8209 | 0,8999 | 0,9479 | 0,9747 | `1,0 · 0,533 · 0,451` |
| `Filmic` | 0 | 0,1011 | 0,5002 | 0,6950 | 0,8072 | 0,8923 | 0,9494 | 0,9832 | 0,9997 | `0,949 · 0,521 · 0,388` |

- ⚠️⚠️ **A 1.ª corrida devolveu a rampa INTACTA nas cinco vistas** — o `applyRGB` do PyOpenColorIO
  2.x **devolve** a cor e não escreve no argumento, e a sonda lia o argumento. *Plausível e falso*;
  o script agora corre um **controlo** antes da tabela (`Standard(0,5) = 0,7354`, o encode sRGB).
- ⭐ **O `Khronos PBR Neutral` confere com a fórmula PUBLICADA em três pontos** (`0,01 → 0,0081` pelo
  joelho `6,25·x²` · `0,18 → 0,4113` pelo desvio `0,04` · `1 → 0,9364` pela compressão) — a fórmula é
  documentação da especificação, logo **implementa-se sem porta de fonte nenhuma**.
- ⭐ **O `AgX` ancora o cinza médio** (`0,18 → 0,4613`, contra `0,4614` do `Standard`).
- ⛔ **Não há AgX de licença permissiva NESTE disco:** os LUTs do Blender apontam para um
  `ocio-license.txt` que **não está instalado**; o *tonemapper* do Qt Quick 3D é da licença do Qt; o
  VTK não tem AgX. ⇒ o oráculo corre-se à mesma (a saída é livre), mas **portar** um AgX pede a
  triagem de uma porta — ela ainda não foi feita.

## §4 — A ordem que isto impõe

1. ⭐ **A primeira fatia que se VÊ não é a `W1` sozinha: é um modo *Render* ao lado do matcap no
   modelador**, com o material gerado (`W2` mínima), a luz do **documento** (a `ph2d-light`, que já
   existe — ⛔ nunca um segundo rig) e o céu como fonte (o `env_ambient` que já existe, `W3`
   mínima), com a exposição e a transformação de vista **por cima** (`W1`). É aí que *«a mesma peça
   com exposição −2, 0, +2»* deixa de ser cosmético.
2. ⏳ **A medição seguinte é de ARQUITECTURA, e decide o resto:** como uma crate de família põe um
   passe de GPU no quadro **sem a shell crescer** (a catraca `the_shell_only_shrinks`). Hoje a malha
   da escultura entra por uma chamada escrita em `render_loop/present.rs`, atrás de uma feature.
3. ⏳ **E a costura CPU↔GPU do modelador:** o traçado é CPU (a `fidget`), o material gerado é GPU —
   o G-buffer (normal, posição, nó) tem de subir como textura, ou o material tem de descer à CPU. A
   segunda contradiz o *«gera-se, não se escreve»* do `00`, e é por isso que a primeira é a candidata.
4. **Classificação escrita antes de haver código:** a exposição e a transformação de vista do
   modelador são **VISTA** (como o *Viewport Shading* do Blender), com o tempo de vida do `View` da
   W43; passam a **documento** no dia em que houver uma saída de render que as grave.
