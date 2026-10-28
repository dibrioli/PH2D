# O `.msk` (Select ▸ Load from / Save to MSK file) — formato medido

Oráculo: **LibreSprite 1.3** (`pacman -Q libresprite` = `1.3-1.1`; o rodapé diz «1.2-dev (local build)»),
GPL-2, **corrido, nunca lido** (CLAUDE.md §0.9). O Aseprite Trial não grava ficheiros. Script:
[`oraculo_msk.py`](oraculo_msk.py) → fixtura `crates/ph2d-pixel-io/tests/fixtures/oracle_msk.json`
(cabeçalho com app, método, data e comando). Código: `crates/ph2d-pixel-io/src/msk.rs`; testes
`crates/ph2d-pixel-io/tests/msk.rs`.

Método: o app faz a selecção com as ferramentas DELE (API JS `selection.select(x,y,w,h)`/`selectAll()`,
Select ▸ Color Range sobre os brancos de uma imagem nossa) e grava; ou carrega um `.msk` que montamos
e regrava. Em todos os casos, depois, Edit ▸ Clear sobre a imagem opaca + Save As PNG: os pixeis
mudados são a selecção que o app tinha — prova independente do ficheiro.

## O formato (tudo little-endian)

| byte | tipo | valor |
|---|---|---|
| 0 | u32 | tamanho do ficheiro inteiro — tem de ser IGUAL ao comprimento (maior ou menor: erro) |
| 4 | u16 | `0x9500` (bytes `00 95`) |
| 6 / 8 | u16 | largura / altura da caixa |
| 10 / 12 | **i16** | x / y da caixa — pode ser negativo e sair da tela (`select(-3,-2,8,6)` grava `-3,-2`) |
| 14..63 | — | zeros, excepto u16 `1` no byte 18 (`0` ou `2` ⇒ erro); o resto é ignorado na leitura |
| 64 | u32 | tamanho do bloco = `6 + dados` |
| 68 | u16 | `2` (máscara) |
| 70 | bytes | `altura` linhas de `⌈largura/8⌉` bytes; bit 7 = x mais à esquerda (MSB primeiro) |

- **Escrita:** a caixa é a caixa JUSTA dos bits (também depois de carregar um ficheiro com
  margens vazias: `r_one_column` 1×11 regrava 1×10); bits de enchimento da linha a **`1`**.
- **Leitura:** enchimento ignorado (`pad0` e `pad1` dão o mesmo); a selecção NÃO é cortada à tela
  (fica guardada fora dela; só a parte dentro se vê/apaga).
- **Sem selecção** o Save responde «Error saving .msk file» (também depois de um Color Range sem
  nenhum pixel). Ficheiro sem bloco tipo 2 (`r_chunk_type_3`): carrega SEM erro e fica sem selecção.
  Ficheiro com bits todos a 0: idem.
- **Legado de 8000 bytes** (320×200, 1 bit, sem cabeçalho): carrega na origem com caixa 320×200,
  qualquer que seja a tela. 7999/8001 bytes: ver a fixtura (`r_legacy_7999`, `r_legacy_8001`).

## Desvios DELIBERADOS do oráculo (defeitos medidos, que não se copiam)

| caso | o oráculo | nós |
|---|---|---|
| `r_legacy_8000*` | lê o bitmap **6 bytes adiante** (os 6 do cabeçalho `u32+u16` que tentou ler primeiro) e enche os últimos 6 bytes com `1` | lemos da origem; o teste `msk_legacy_8000_reads_from_the_origin_not_six_bytes_ahead` prova que a diferença é só esse deslocamento |
| `r_trailing_bytes_sized`, `r_chunk_size_smaller` | um bloco de tamanho 0 ou um resto partido deixa o app **parado** com o OK premido | `MskError::BadChunk` |
| `r_negative_origin` | carrega certo (a regravação tem os bits de entrada) mas o **Clear** dele não apaga nada com máscara não-retangular em x/y negativos | só a regravação conta |

`r_chunk_size_bigger` (bloco declara mais 4 bytes do que tem) carrega normal; `r_reserved_garbage`
(bytes 14..17 e 20..63 a `0xA5`) também.

## Correr

```sh
python3 -I docs/PixelArt/ferramentas/oraculo_msk.py <pasta vazia> \
    > crates/ph2d-pixel-io/tests/fixtures/oracle_msk.json      # ~25 min, Xvfb :79
```

Display `PH2D_ORACLE_DISPLAY` (omissão `:79`). Armadilhas medidas, já no script: SIGTERM não fecha o
app (`kill -9`); um 2.º clique no mesmo ponto não chega ao OK (o rato sai e volta); caminho digitado
longo perde letras (vai por um atalho `/tmp/msk<pid>`); erro do app só conta repetido numa sessão
NOVA; PNG com dígito no fim do nome abre a pergunta de SEQUÊNCIA.
