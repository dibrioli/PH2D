# Os matcaps: de onde vieram e sob que licença

As nove imagens deste diretório são **redistribuíveis**, e este arquivo é a
prova disso — não uma lembrança de que alguém conferiu um dia.

Todas foram **cozidas** a partir das fontes originais (ver §3): 512×512, RGB,
**sRGB de 8 bits**. Nenhuma é o arquivo original byte a byte.

---

## 1. Os oito do Blender — CC0 / domínio público

`basic_bright` · `basic_dark` · `basic_grey` · `basic_side` ·
`clay_brown` · `clay_green` · `clay_warm` · `red_wax`

Fonte: <https://projects.blender.org/blender/blender>, tag **`v5.2.0`**,
`release/datafiles/studiolights/matcap/*.exr`.

O diretório de origem carrega o próprio `license.txt`, e ele é curto o bastante
para caber aqui inteiro:

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

## 2. O do SculptGL — MIT

`sculptgl_fv` (o chip **Studio**, e o **default do app**)

Fonte: <https://github.com/stephaneginier/sculptgl>,
`app/resources/matcaps/matcapFV.jpg`.

    MIT License

    Copyright (c) 2019 Stéphane GINIER

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

⚠️ **A seção *Credits* do SculptGL atribui a terceiros apenas os
*environments*** (de `hdrihaven.com`) — os matcaps não são creditados a
ninguém de fora, então caem sob a licença do repositório. É por isso que o
`environments/` **não** foi tocado: aquele sim tem outro dono.

## 3. Como foram cozidos

⚠️ **Duas fontes, UMA lei de saída:** o que entra no repositório é sempre sRGB de
8 bits, porque a GPU devolve linear de graça num formato `…UnormSrgb` e o shader
quer linear.

- **Blender (`.exr`)** — cena-referida **LINEAR**, meio-float, compressão DWAA,
  com as camadas `diffuse` e `specular` **separadas**. O matcap é a **soma das
  duas**, e a soma é fiel porque o nosso caminho de matcap não multiplica nada
  por cor de vértice (a separação existe no Blender para tingir a difusa pela cor
  do objeto). Depois: transferência sRGB e quantização para 8 bits.
  ⚠️ **MEDIDO antes de escolher 8 bits:** o máximo dos oito, já somado, é
  **0,941** — `0,00%` dos texels passam de 1,0 —, então nada é cortado pelo
  clamp e não há faixa HDR a preservar.
- **SculptGL (`.jpg`)** — já é sRGB autorado. Ele é apenas **re-embalado** em
  PNG; nenhuma transferência é aplicada, porque aplicá-la o clarearia duas vezes.

⚠️ **Por que não ler os `.exr` em tempo de execução:** o nosso decoder
(`ph2d-imageio-exr`) recusa estes arquivos por **dois** motivos independentes,
os dois escritos no doc dele — *"custom channel layouts beyond RGBA"* (estes têm
`diffuse.*` e `specular.*`) e *"tile-based + DWA/DWB compression"*. A conversão é
offline por medição, não por gosto.

## 4. Reproduzir

O script que cozinhou está em `docs/3D/ferramentas/cook_matcaps.py`, com os
comandos de download (LFS) no cabeçalho. Ele imprime os hashes de origem, que é
como a tabela da §1 é conferida em vez de copiada.
