# 00 — A triagem de licença, e a porta que ela achou aberta

> `CLAUDE.md` §0.9: *«Passo 1 é sempre a TRIAGEM DE LICENÇA, e ela pára na primeira porta ABERTA.»*
> *«Uma semana de parede gasta onde havia porta aberta é a forma mais cara deste erro.»*

E a unidade da triagem é o **artefacto instalado**, nunca o nome do projecto.

## §1 — O que esta máquina tem, medido por `pacman -Qi`

| artefacto | versão | licença | porta |
|---|---|---|---|
| **`materialx`** | **1.39.5** | ⭐ **Apache-2.0** | **ABERTA** — porta-se, com atribuição |
| **`usd`** | 26.08 | ⭐ **Apache-2.0** | **ABERTA** |
| **`godot`** | 4.7.2 | ⭐ **MIT** | **ABERTA** |
| `blender` | 5.2.1 | GPL-2.0-or-later (+8 outras) | **fechada** para o fonte da app |
| Frostbite (o alvo do dono) | — | proprietário, sem fonte | fechada — mas ver §4 |

## §2 — ⭐⭐⭐ O que estava dentro do `materialx`, e por que ele fecha a pergunta central

O dono mostrou o **Principled BSDF** do Blender e disse *«o principal é o BSDF»*. Ele tem razão, e a
resposta é melhor do que copiá-lo: **o padrão aberto que a indústria assinou está instalado aqui**.

```
/usr/share/materialx/libraries/bxdf/open_pbr_surface.mtlx
/usr/share/materialx/libraries/bxdf/translation/open_pbr_to_standard_surface.mtlx
/usr/share/materialx/libraries/bxdf/translation/standard_surface_to_open_pbr.mtlx
```

- **`open_pbr_surface`** declara **41 entradas de artista**, medidas pela API:
  `base_weight` · `base_color` · `base_diffuse_roughness` · `base_metalness` · `specular_weight` ·
  `specular_color` · `specular_roughness` · `specular_ior` · `specular_roughness_anisotropy` ·
  `transmission_*` (7) · `subsurface_*` (5) · `fuzz_*` (3) · `coat_*` (5) · `thin_film_*` (3) ·
  `emission_luminance` · `emission_color` · `geometry_opacity` · `geometry_thin_walled` ·
  `geometry_normal` · `geometry_coat_normal` · `geometry_tangent` · `geometry_coat_tangent`.
  ⭐ **É a foto que o dono mandou**, secção por secção — *Base · Subsurface · Specular · Transmission
  · Coat · Sheen · Emission · Thin Film* — mas na versão que Blender, Autodesk, Adobe e a Academy
  Software Foundation assinaram em conjunto.
- **807 nodedefs** na biblioteca — um catálogo inteiro de nós de shading, **como DADO**.
- **Tradutores** de e para o *Autodesk Standard Surface* já escritos: um material que venha do
  Houdini, do Substance ou do Maya **converte-se sem perda declarada**.

## §3 — ⭐⭐⭐ E ele NÃO é só uma tabela: ele GERA o shader

O §0.9 desta casa manda **correr** o alvo por script, sem interface. Feito, e o resultado é a
capacidade que decide o desenho inteiro:

```python
gen = mxglsl.GlslShaderGenerator.create()
sh  = gen.generate("opbr", material, ctx)
sh.getSourceCode("pixel")
```

| medida | valor |
|---|---:|
| linhas de GLSL geradas para o OpenPBR completo | **2 177** |
| bytes | **99 353** |
| contém GGX | **sim** |
| contém compensação de energia (multiscatter) | **sim** |

Geradores instalados: **GLSL · MSL · OSL · MDL · Slang** (`libMaterialXGenSlang.so`), mais
`MaterialXView` e `MaterialXGraphEditor` como binários, e as ligações **Python** a funcionar.

⇒ ⭐⭐ *Nós não escrevemos o modelo de material. Nós geramo-lo a partir do padrão, e a nossa parte é
o que fazemos com ele.*

⛔⛔ **REFUTADO em 13/09 ([`04`](04_a_remedicao_contra_a_arvore.md) §2):** o `PyMaterialXGenGlsl`
traz **`WgslShaderGenerator`** e `VkShaderGenerator` — o censo acima listou MÓDULOS e a classe vive
dentro de um deles. A rota medida que funciona é esse gerador → `naga` `glsl-in` (`5 598` linhas de
WGSL validadas); as três saídas abaixo ficam como a redacção de 09/09.

⚠️ **Não há gerador de WGSL** em 1.39.5, e é o único buraco desta porta. Três saídas, por ordem de
preço, e **nenhuma foi ainda medida** — é a primeira medição da implementação:
1. **Slang** (que está instalado) compila para SPIR-V, e o `wgpu` come SPIR-V.
2. **GLSL → `naga`** (o tradutor que o `wgpu` já traz nesta árvore).
3. Escrever um `GenWgsl` — o gerador da MaterialX é extensível por desenho, e os cinco existentes
   são o molde.

## §4 — O alvo do dono é FECHADO, e isso não bloqueia nada

O *Plants vs. Zombies: Battle for Neighborville* corre no **Frostbite** (EA/DICE), proprietário e
sem fonte público. ⚠️ **Mas o §0.9 não pede o fonte** — ele diz que *ler o fonte é o método pior*,
porque responde *como* quando um gate precisa de *o quê*.

⭐ E o que a Frostbite fez está **publicado**: o curso *«Moving Frostbite to PBR»* (Lagarde &
de Rousiers, SIGGRAPH 2014) é documentação, não código — a mesma porta que o importador de Aseprite
desta casa usou (a spec do formato é documentação; a app é GPL).

⇒ o alvo entra como **decomposição de ingredientes** ([`01_o_alvo_decomposto.md`](01_o_alvo_decomposto.md)),
nunca como fonte.

## §5 — A conclusão da triagem

| pergunta | resposta | porquê |
|---|---|---|
| inventar um modelo de superfície? | ⛔ **não** | o padrão está no disco, é Apache-2.0, e gera código |
| clean-room de alguma coisa? | ⛔ **não** | nenhuma das portas necessárias é *walled* |
| copiar o Principled do Blender? | ⛔ **não** | o OpenPBR **é** a versão padronizada dele, e vem com tradutores |
| o que fica por nossa conta | ⭐ **o RENDERER e a AUTORIA** | é aí que o alvo do dono vive, e é aí que se pode ganhar |
