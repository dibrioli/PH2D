# Os matcaps: de onde vieram, sob que licença, e em que precisão

Este arquivo é a prova de que as dez imagens são redistribuíveis — não uma
lembrança de que alguém conferiu um dia. Ele também registra **uma procedência
que não é limpa** (§2), em vez de escondê-la atrás da licença do repositório que
a distribui.

---

## 1. Os oito do Blender — CC0 / domínio público

`basic_bright` · `basic_dark` · `basic_grey` · `basic_side` ·
`clay_brown` · `clay_green` · `clay_warm` · `red_wax`

Fonte: <https://projects.blender.org/blender/blender>, tag **`v5.2.0`**,
`release/datafiles/studiolights/matcap/*.exr`.

O diretório de origem carrega o próprio `license.txt`, curto o bastante para
caber aqui inteiro:

> These matcap images are licensed as CC0 or public domain.
>
> Thanks to the Blender community for contributing these matcaps.

⚠️ **A licença é do DIRETÓRIO de imagens, e não do Blender.** O Blender é GPL, e
nada de código dele entrou aqui — estes são *datafiles* com licença própria e
mais permissiva. Confundir os dois seria o erro caro nesta direção.

Cada arquivo foi conferido por **hash** contra o `oid` do ponteiro git-lfs no
momento do download, então o que foi cozido é o que o Blender publica:

| arquivo | sha256 do `.exr` de origem | bytes |
|---|---|---|
| `basic_bright` | `40086db175648e9256fa9fe75d710b9bdc78b5bd6a78b7e527b1fde54193e1b4` | 54 248 |
| `basic_dark`   | `ef4896291fe4d34e3beb1b26f6b1eacee10f996388958092ec88a457ec754618` | 36 896 |
| `basic_grey`   | `7266ec195b78964248bbd526c8895036ef7cda0e099b62bac67ab860d1ecf34c` | 47 801 |
| `basic_side`   | `4596e70ba9351907a93048c70660fdefca159a91ccdedc01d6fde3753ddf8c1e` | 50 527 |
| `clay_brown`   | `ab2f06971cca555e8d3d86ee6fa9ddceeab21b8dd7b50423c521f6c87bf671c2` | 57 812 |
| `clay_green`   | `f35b3b175f5526fbece6aa16ee7278f787b6eef86b79e4deb45ca67e0f0f97a6` | 50 917 |
| `clay_warm`    | `a91122d8e2aa0bdc17165e4a72ccaca991d5554d3a8afebce6e3fc35881df159` | 58 983 |
| `red_wax`      | `ff3e9c647952c9f5bedf33a82c91e4751b844b2e0f53dab29d4b01656ebe9722` | 144 643 |

⚠️ **O arquivo do Blender é `basic_grey.exr`, com a grafia britânica**, e o nome
que o artista lê aqui é `Basic Gray`. Os dois estão certos: o stem preserva a
procedência e o rótulo segue o inglês do resto da UI.

## 2. Os dois de pele — **HazardousArts**, e a procedência NÃO é limpa

`skinHazardousarts` (chip **Skin Haz**) ·
`skinHazardousarts2` (chip **Skin Haz 2**, e o **default do app**)

Chegaram até nós pelo SculptGL (<https://github.com/stephaneginier/sculptgl>,
`app/resources/matcaps/`), que é **MIT**:

    MIT License · Copyright (c) 2019 Stéphane GINIER

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

⚠️ **Mas o autor original é um TERCEIRO, e ele está no nome do arquivo.** São do
**HazardousArts**, publicados no DeviantArt em 2014 como *"Haz Skin Matcap"*
(<https://www.deviantart.com/hazardousarts/art/Haz-Skin-Matcap-495671758>) e
divulgados como gratuitos. **Os termos exatos do autor não estão documentados**,
e a seção *Credits* do SculptGL atribui a terceiros apenas os *environments* —
não estes.

O que isso significa, dito sem eufemismo:

- **não** é o CC0 explícito dos oito da §1;
- o que temos é a licença de quem os **redistribui** há mais de uma década, mais
  a publicação gratuita do autor;
- se algum dia isto tiver de ser defendido, é **este parágrafo** que descreve a
  situação — não um campo escrito "MIT".

É por isso que o [`Credit`](../../src/matcap.rs) deles se chama `HazardousArts` e
não `SculptGl`: o tipo obriga quem lê o código a encontrar esta seção.

O `environments/` do SculptGL **não** foi tocado — aquele tem outro dono
(hdrihaven), e é declarado.

| arquivo | sha256 do `.jpg` de origem | bytes |
|---|---|---|
| `skinHazardousarts`  | `0cb2a4c7a8cd9c443357b368edbdb588aa9cd62c7538f1dd536a575d039a72cf` | 41 866 |
| `skinHazardousarts2` | `ba0c5c776878b828272121102ce0fe8770c8a8f9e418db6f715ee3a231df3982` | 40 672 |

## 3. Como foram cozidos — **nada é quantizado abaixo da FONTE**

⚠️ **O primeiro corte desta wave guardava tudo em PNG de 8 bits, e era uma perda
que não estava marcada como tal.** A medição feita então respondia *"algum valor
passa de 1,0?"* (a faixa HDR) e a conclusão foi *"8 bits bastam"* — mas *"cabe em
[0,1]"* e *"8 bits chegam"* são perguntas diferentes. Medido de volta em
**linear**, que é o que o shader recebe:

| matcap | erro em 8 bits | erro em 16 bits | razão |
|---|---|---|---|
| `basic_bright` | **0,93** nível de 255 | 0,0036 | 259× |
| `basic_side`   | **1,09** | 0,121 | 9× |
| `clay_warm`    | **0,73** | 0,0029 | 253× |
| `red_wax`      | **0,78** | 0,0030 | 257× |
| `basic_dark`   | **0,40** | 0,0016 | 259× |

Um matcap é um gradiente liso sobre uma esfera — o caso clássico de banda
visível. Hoje cada fonte é guardada **na precisão em que foi autorada**:

- **Blender (`.exr`)** — cena-referida LINEAR, meio-float, DWAA, com `diffuse` e
  `specular` em camadas separadas. O matcap é a **soma** das duas (fiel porque o
  nosso caminho de matcap não multiplica nada por cor de vértice — a separação
  existe no Blender para tingir a difusa pela cor do objeto). A saída é um EXR
  **RGB simples, meio-float, ZIP**: a mesma informação, por uma porta que o nosso
  `ph2d-imageio-exr` lê.
  ⚠️ **Ele recusa o arquivo ORIGINAL por dois motivos escritos no doc dele** —
  *"custom channel layouts beyond RGBA"* e *"tile-based + DWA/DWB compression"* —
  e **nenhum dos dois é sobre precisão**. É exatamente por isso que re-embalar
  resolve em vez de degradar.
- **SculptGL (`.jpg`)** — 8 bits sRGB autorados. A saída é **PNG**, que guarda os
  MESMOS bytes que o JPEG decodifica: **bit-idêntico à fonte**. Promovê-los a
  float daria um arquivo maior dizendo a mesma coisa.

A decodificação entrega **meio-float linear** nos dois casos, para uma textura
`Rgba16Float`; a conversão sRGB→linear dos dois de pele usa a porta do repo
(`ph2d_color::srgb::srgb_to_linear_byte`), porque a curva sRGB tem **joelho** e
um `x^2,2` escrito à mão erra no escuro — que é metade de um matcap de pele.

**Preço**: 3,9 MB no repositório contra 724 KB do primeiro corte, por 259× de
precisão.

## 4. Reproduzir

```
bash docs/3D/ferramentas/cook_matcaps.sh <dir-com-as-fontes> <dir-de-saida>
```

Ele imprime os hashes de origem, que é como as tabelas das §1 e §2 são
**conferidas** em vez de copiadas. O cabeçalho dele traz os comandos de download
(os `.exr` do Blender vivem em git-lfs, e o mirror do GitHub **não** hospeda os
objetos — eles saem do servidor LFS do próprio Blender).
