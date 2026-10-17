# I0d — StyLua: versão, licença por artefacto, sha256, comportamento e tempo (16/10)

> Item I0(d) de [`03_onda2.md`](../../03_onda2.md); alimenta o **I13** (formatar ao salvar, decisão 6 de
> [`00_decisoes_do_dono.md`](../../00_decisoes_do_dono.md)). Medido em 2026-10-16, workstation (32 lógicos),
> perfil = os binários OFICIAIS de release do StyLua (não há build nosso). Rascunho do manifesto:
> [`../stylua/stylua.toml`](../stylua/stylua.toml); config de formatação: [`../stylua/ph2d-format.toml`](../stylua/ph2d-format.toml).

## Como re-correr (bash)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor
T=docs/CodeEditor/ferramentas/stylua; C="${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/stylua/2.5.2"; mkdir -p "$C"
# 1. baixar + conferir (até existir o fetch script do I13; os sha256 são os do manifesto)
for a in linux-x86_64 linux-x86_64-musl; do
  curl -sSfL -o "$C/stylua-$a.zip" "https://github.com/JohnnyMorganz/StyLua/releases/download/v2.5.2/stylua-$a.zip"; done
( cd "$C" && sha256sum stylua-linux-x86_64.zip stylua-linux-x86_64-musl.zip )   # bcb0d855… / ca6f1cf5…
unzip -o -q "$C/stylua-linux-x86_64.zip" -d "$C"; mkdir -p "$C/musl"; unzip -o -q "$C/stylua-linux-x86_64-musl.zip" -d "$C/musl"
# 2. entradas (o mesmo texto da bancada.rs) e as duas rodadas intercaladas
E=$(mktemp -d); python3 -I $T/gera.py crates/ph2d-code-syntax/tests/it/fixtures/bob.luau "$E"
cat /proc/loadavg; python3 -I $T/cronometra.py "$C/stylua" "$C/musl/stylua" "$PWD/$T/ph2d-format.toml" "$E" 60; cat /proc/loadavg
python3 -I $T/varre.py "$C/stylua" "$PWD/$T/ph2d-format.toml" "$E/bob51.luau" 40
# 3. comportamento: sempre com --config-path (ver §Achados 1)
"$C/stylua" --config-path "$PWD/$T/ph2d-format.toml" - < crates/ph2d-code-syntax/tests/it/fixtures/bob.luau
```

`cronometra.py`: cada volta corre TODAS as variantes uma vez, em ordem rodada; 3 voltas de aquecimento
descartadas; tempo = `Popen()` → `communicate()` devolve (spawn + stdin + formatar + stdout + saída), e cada
saída é conferida igual à primeira (as duas builds dão bytes iguais). `/usr/bin/true` é o chão do spawn.

## Versões

| | |
|---|---|
| Escolhida | **v2.5.2**, publicada **2026-05-16** (5 meses) — <https://github.com/JohnnyMorganz/StyLua/releases/tag/v2.5.2> |
| Mais nova | v2.6.0, publicada **2026-10-10** — 6 dias, **abaixo do piso de duas semanas**; elegível a **2026-10-24** |
| 2.5.0 → 2.5.2 | as três no mesmo dia; 2.5.1 e 2.5.2 só consertam a publicação no npm («no code changes») |
| O que a 2.6.0 traz para nós | conserta `{ read Foo }` → `{ readFoo }` (muda o significado, ver §Achados 4); `if local`/`if const`; `collapse_simple_statement = "Input"` |
| `stylua --version` | `stylua 2.5.2` (glibc e musl) |

## Triagem de licença por artefacto

| Artefacto | Licença | Como foi lido |
|---|---|---|
| Repositório (tag `v2.5.2`) | **MPL-2.0** | API GitHub `license` = `mpl-2.0` (`LICENSE.md`); o blob da tag (`git hash-object` = `fa0086a9…`) é o texto MPL-2.0 da Mozilla palavra a palavra (única diferença: `http` × `https` no URL do Exhibit A) |
| `Cargo.toml` do pacote | `license = "MPL-2.0"` | fonte da tag |
| **Cada zip de release** (os 7) | **nenhum ficheiro de licença dentro** — `unzip -l` lista SÓ o binário (`stylua` / `stylua.exe`) | medido nos 7 zips |
| Binário | sem texto de licença embutido (`strings` não acha MPL/MIT/Apache) | medido no linux-x86_64 |
| Dependências ligadas no binário (`cargo metadata --locked`, features da release: `lua52,lua53,lua54,luau,luajit,cfxlua` + omissão `editorconfig,lsp`) | 77 crates: 44 `MIT OR Apache-2.0`, 10 MIT, 5+3+1 MIT/Apache, 7 `Unlicense OR MIT`, **3 MPL-2.0** (`stylua`, `full_moon`, `full_moon_derive`), 2 Apache-2.0 só (`ec4rs`, `similar` — sem ficheiro `NOTICE`), 1 `Apache-2.0 OR BSL-1.0` (`ryu`), 1 `… AND Unicode-DFS-2016` (`unicode-ident`, proc-macro: não entra no binário) | nenhuma copyleft além da MPL-2.0 (por ficheiro); nenhum GPL/LGPL |

**Obrigações para distribuir o binário NÃO modificado (3 linhas):**
1. MPL-2.0 §3.2: dizer a quem recebe o executável **onde obter o código-fonte** (o link da tag v2.5.2) e que ele está sob a MPL-2.0 — no ecrã de créditos/licenças.
2. Não restringir nem alterar os direitos do destinatário sobre esse código (MPL §3.2(b)); levar o texto da MPL-2.0 (ou o link para ele) junto.
3. As deps MIT/Apache embutidas pedem o aviso de copyright + texto da licença na distribuição: um bloco «StyLua e componentes» com as licenças MIT, Apache-2.0, Unlicense, BSL-1.0 nos mesmos créditos (os zips oficiais NÃO trazem esses avisos — somos nós que os juntamos).

## Assets (o que o `luau-lsp.toml` cobre)

| Chave no manifesto | Asset | Bytes | sha256 local (= `digest` da API) |
|---|---|---:|---|
| `linux-x86_64` | `stylua-linux-x86_64.zip` (glibc, precisa `GLIBC_2.34`) | 3 303 824 | `bcb0d855e91f102f28a370e850f8566b3b44b79e6274d806ea5246837c0fd5ab` ✓ |
| (alternativa) | `stylua-linux-x86_64-musl.zip` (estático) | 3 409 105 | `ca6f1cf52eaf69e6632b81acef9c197aa24b85eb30d2455a35e7dbe28ae77c72` ✓ |
| `linux-arm64` | `stylua-linux-aarch64.zip` | 3 262 706 | `0ef2ebf0b7e5a652b65c4cb96c6d9ffb3981a98547de3c764465bbf54a8d761a` ✓ |
| (alternativa) | `stylua-linux-aarch64-musl.zip` | 3 289 322 | `b948df6b4bae41af9a70948174372f96a3c14d44e3e017288701539f3db8fb75` ✓ |
| `macos-x86_64` | `stylua-macos-x86_64.zip` | 3 127 859 | `53c50a1605d0a6345d160a1a5a21db40bcf2bf9cd23c17f7c277a63a1bff3a7f` ✓ |
| `macos-arm64` | `stylua-macos-aarch64.zip` | 3 033 174 | `92ff0889e16324801bc072692974bb67f8161e62010fc90f96c62a17f81f32c7` ✓ |
| `windows-x86_64` | `stylua-windows-x86_64.zip` (`stylua.exe`) | 2 859 281 | `e77d0ea1226b8b389b43f702240091249a96eea25857281f90ea24d0eb9eb969` ✓ |

- **macOS: um asset POR ARQUITECTURA** — não há Mach-O universal como no `luau-lsp`. O fetch script do I13
  tem de ramificar por `uname -m` também no Darwin (o do `luau-lsp` não ramifica).
- **Linux: escolhida a build glibc.** O piso `GLIBC_2.34` é o MESMO que o `luau-lsp` 1.70.1 já impõe
  (medido com `objdump -T`), então não sobe nada; e a musl formata **35–45 % mais devagar** (tabela abaixo).
- Cache: `~/.cache/ph2d/stylua/2.5.2/` com os 7 zips, `stylua` (glibc), `musl/stylua` e `src/` (o tarball da tag, para a triagem).

## Comportamento sobre entradas NOSSAS

Sempre `stylua --config-path <ph2d-format.toml> -` (stdin → stdout). Saídas gravadas em [`../stylua/saida/`](../stylua/saida/).

| Caso | Resultado |
|---|---|
| (a) `bob.luau` (53 linhas) | rc `0`, 53 → 55 linhas. Diff: [`bob.diff`](../stylua/saida/bob.diff). Duas mudanças: **os comentários de fim de linha alinhados em coluna perdem o alinhamento** (4 `ph2d.property(...)  -- …` passam a 1 espaço) e `if not self.active then return end` vira 3 linhas (`collapse_simple_statement = "Never"`; com `--collapse-simple-statement ConditionalOnly` fica numa linha — medido) |
| (b) sem o `end` do `init` | **rc `2`, stdout VAZIO (0 bytes)**, stderr: `error: could not format from stdin: failed to format from stdin: error parsing: unexpected token `` (30:1 to 52:4), expected `end` to close function body block` — a posição aponta o fim do ficheiro, não a linha 29 de onde o `end` saiu. Também rc 2 para `local x = = 1` (dois erros listados). Nunca devolve meio texto ⇒ «erro de sintaxe ⇒ salva sem formatar» é só ler o rc |
| (c) comentários | [`comentarios.diff`](../stylua/saida/comentarios.diff): `--!strict`/`--!native` na 1.ª linha, `-- @hook …` (linha própria e fim de linha), `--[[ bloco ]]`, `--@param`, acentos e emoji: **todos preservados byte a byte**; `-- stylua: ignore` respeitado. Muda aspas (`'x'` → `"x"`; `"com \"escape\""` → `'com "escape"'`, AutoPreferDouble) |
| (d) idempotência | formatar a saída de novo dá **os mesmos bytes** (`cmp`) em `bob` e `comentarios` |
| Isolamento da config | com um `stylua.toml` (Tabs, ForceSingle) no cwd, outro em `$XDG_CONFIG_HOME/stylua/` e um `.editorconfig` (tab): com `--config-path` a saída é a NOSSA (2 espaços, aspas duplas), também com `-s`; sem `--config-path` o do cwd vence, e só com `.editorconfig` ele também vence. `--config-path` encurta toda a procura (cwd, pais, XDG, editorconfig) |
| Bordas | stdin vazio → saída vazia, rc 0; sem `\n` final → acrescenta `\n`; CRLF → LF (`line_endings = "Unix"`) |

### Achados

1. **O manifesto NÃO pode chamar-se `stylua.toml` onde o StyLua possa correr.** Com o cwd na pasta do
   rascunho e sem `--config-path`, o StyLua leu o NOSSO manifesto como config e recusou: `rc 2`,
   `unknown field 'version'`. Ao mover para a crate, renomear (p. ex. `stylua-pin.toml`) e passar SEMPRE
   `--config-path` — que também impede que a config da casa do utilizador vaze.
2. A config padrão do StyLua é **Tabs** de 4; a nossa ([`ph2d-format.toml`](../stylua/ph2d-format.toml)) fixa
   **2 espaços** (o `INDENT` do editor) e escreve os outros padrões por extenso, para uma subida de versão
   que mude um padrão não mudar a nossa saída calada.
3. Os comentários alinhados de `bob.luau` são estilo do PRÓPRIO script canónico do projeto; o StyLua não
   tem opção que os preserve. Decisão de produto para o I13: aceitar (o StyLua é a norma do mundo Luau) ou
   marcar o bloco com `-- stylua: ignore start/end`.
4. ⛔ **A 2.5.2 muda o significado de `type A = { read number }` → `{ readnumber }`, com rc 0.**
   `--verify` apanha-o (rc 2, «Output AST may be different to input AST»). Custo do `--verify` na tabela.
   A 2.6.0 (elegível a 24/10) conserta.
5. O StyLua tem modo `--lsp` (servidor persistente com formatação) — tiraria só o spawn (~0,5–2 ms),
   não a formatação, que é o que domina acima de ~100 linhas. Não muda a decisão.

## Tempo

`loadavg` na rodada 1: `7.00 15.73 16.21` → `15.10 16.37 16.38`; rodada 2: `14.19 16.14 16.31` → `12.97 15.45 16.06`;
varredura: `15.56 15.86 16.18` → `19.60 16.69 16.45` (máquina ocupada por outras linhas; não esperei calma —
o **mínimo** é a leitura robusta; p50/p99 são de máquina carregada, ou seja, pessimistas). 60 corridas por
variante (40 na varredura), ms, Python `subprocess` (o `Command` de Rust usa o mesmo `posix_spawn`/`vfork`).

**Rodada 2** (a mais limpa; a rodada 1 dá os mesmos mínimos ±15 %):

| Variante | min | p50 | p99 |
|---|---:|---:|---:|
| `/usr/bin/true` (chão do spawn) | 0.21 | 0.41 | 0.87 |
| `stylua --version` glibc / musl | 0.46 / 0.42 | 0.70 / 0.78 | 1.60 / 1.32 |
| **bob 51 linhas** glibc | **2.28** | **3.05** | **6.97** |
| bob 51 musl | 2.95 | 4.82 | 6.96 |
| bob 51 glibc `--verify` | 3.16 | 4.11 | 7.37 |
| bob 51 glibc `--num-threads 1` | 1.91 | 3.09 | 6.09 |
| **3 009 linhas** glibc | **28.75** | **44.07** | **55.94** |
| 3 009 musl | 44.34 | 61.92 | 77.29 |
| **7 500 linhas** glibc (saída 7 794) | **76.84** | **107.66** | **153.42** |
| 7 500 musl | 107.39 | 145.21 | 179.84 |
| 7 500 glibc `--verify` | 156.03 | 232.60 | 280.40 |
| 7 500 glibc `--num-threads 1` | 76.20 | 104.92 | 146.36 |

Rodada 1 (p99 sob a subida de load 7→15): bob 51 glibc `2.41 / 4.75 / 15.48` (máx `19.88`); 3 009 `31.55 / 45.18 / 88.27`; 7 500 `91.99 / 109.38 / 198.22`.

**Varredura de tamanho** (glibc, cópias inteiras do bob):

| linhas | 51 | 102 | 204 | 306 | 408 | 510 | 612 | 714 | 816 | 1 020 | 1 530 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| min | 2.63 | 2.76 | 3.77 | 4.84 | 5.89 | 6.68 | 7.74 | 8.08 | 9.43 | 12.14 | 17.97 |
| p50 | 5.57 | 6.53 | 7.93 | 8.97 | 10.38 | 11.61 | 13.68 | 14.97 | **16.61** | 19.73 | 27.46 |
| p99 | 9.77 | 9.66 | 13.43 | 15.80 | **17.13** | 20.07 | 20.40 | 20.72 | 27.61 | 35.97 | 37.88 |

Leitura: spawn + arranque ≈ `0.5–2` ms; a formatação custa **~10 µs/linha** no mínimo (~14 µs no p50),
linear. `--num-threads` não muda nada (stdin é um ficheiro só). `--verify` **dobra** o custo a 7 500 linhas.

## O que isto decide

Orçamento: Ctrl+S → selo em **UM quadro a 60 Hz (16.7 ms)** ([`02_onda1.md`](../../02_onda1.md) §Medições E14,
decisão 3); o salvar sem formatar já gasta `0.05` ms (51) e `3.07` ms p50 (7 500).

| Tamanho | StyLua (min / p50 / p99) | + salvar | Cabe num quadro? |
|---|---|---|---|
| **51** linhas | 2.3 / 3.1–5.6 / 7.0–15.5 | +0.05 | **Sim** no p50; o p99 cabe à justa sob load 16 e estourou uma vez (19.9 ms) com o load a subir |
| **3 000** linhas | 29 / 44 / 56–88 | + < 3.07 (não medido a 3 000) | **Não** — 2–3 quadros |
| **7 500** linhas | 77 / 108 / 153–198 | +3.07 | **Não** — 6–7 quadros (14 com `--verify`) |

- **A partir de que tamanho deixa de caber:** p50 a **~800 linhas**, p99 a **~400 linhas** (máquina com
  load ~16); mesmo no melhor caso (o mínimo) a **~1 400 linhas**. O tecto é o do processo (spawn + ~10 µs/linha),
  não do nosso código.
- ⇒ O **kill-criterion do I13 dispara**: formatar no fio da UI não cabe para todo tamanho. A forma certa é
  UMA lei para todos os tamanhos (sem `MAX_*` de linhas, que seria tecto do caminho lento): Ctrl+S grava e
  acende o selo no mesmo quadro; o StyLua corre **fora do fio da UI**; o resultado entra como UM passo de
  undo **só se o buffer não mudou** desde o pedido (versão do documento), senão é descartado; rc ≠ 0 ⇒ fica
  o texto como está, com a recusa legível na consola. A 51 linhas o formatado chega no mesmo quadro ou no seguinte.
- `--verify`: obrigatório enquanto a versão fixada for a 2.5.2 (achado 4) — fora do fio da UI o custo
  dobrado não se sente. Reavaliar com a 2.6.0 a partir de 24/10.
- Build Linux: glibc (mais rápida, sem piso novo). Manifesto: renomear ao entrar na crate (achado 1).
