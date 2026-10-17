# I6 / I11 — o que o luau-lsp 1.70.1 dá à definição e às cores (17/10)

**Reproduzir** (da raiz da worktree, ~3 s): `python3 -I docs/CodeEditor/ferramentas/lspprobe_i6_i11.py`
→ regrava `I6_I11_bruto.json`. Corre o servidor DUAS vezes: como o app (as definições e o `--docs`
gerados em `~/.cache/ph2d/luau-lsp/1.70.1/api/`) e como o I0b (`runtime/luau/ph2d.d.luau`, sem docs).
**As duas corridas dão respostas iguais** (116 tokens, as 12 definições). Entrada: o `bob.luau` (51
linhas) + seis linhas nossas (`print(math.sin(1))`, `pirnt(1)`, `local function ajuda`, `local quieto`,
`ph2d.proprety`, `print(quieto, self, ph2d.set)`). `loadavg` `4.27 4.71 3.06`.

## Tokens semânticos (`full`)

[MEDIDO] Só os NOMES têm token, e nem todos:

| Nome | Token | Tree-sitter dava |
|---|---|---|
| `ph2d` (a tabela) | `variable` · `readonly, defaultLibrary` | `Variable` |
| `ph2d.set`, `ph2d.get`, `ph2d.property` (o membro) | `function` · `readonly, defaultLibrary` | `FunctionCall` |
| `print`, `math`, `math.sin` | `function`/`variable` · `readonly, defaultLibrary` | `Builtin` / `FunctionCall` |
| `self` | `property` · `defaultLibrary` | `Builtin` |
| `self.amplitude`, `self.t`, campos de tabela | `property` | `Property` |
| parâmetro (`dt`, também no USO) | `parameter` | `Variable` no uso |
| `local function ajuda` (o uso) | `function` | `FunctionCall` |
| local (`brilho`, `d`, `quieto`) | **nenhum** | `Variable` |
| global que não existe (`pirnt`) | **nenhum** | `FunctionCall` |
| `ph2d.proprety` (membro que não existe) | `ph2d` sim, `proprety` **nenhum** | — |
| `function init`/`update` (global do script) | **nenhum** | `Function` |

⇒ O «não resolvido» não sai dos tokens: a legenda não tem esse tipo e o servidor simplesmente não
emite nada (como para um local). **A cor de alerta vem do diagnóstico `1001`** que a aba já segura
(I3), e segue-o. Texto partido a meio (`pirnt(z` sem fecho): só os nomes ANTES do erro têm token.
Sem `version` na resposta: a etiqueta é a do pedido.

## Definição

| Onde | Resposta |
|---|---|
| uso de local (`brilho`) | a declaração (`local brilho`) |
| uso de parâmetro (`dt`) | o parâmetro |
| `local function ajuda` (uso) / `local quieto` (uso) | a declaração |
| `self.t` (leitura) | a escrita `self.t = …` da mesma linha |
| `self.amplitude` (propriedade) | `[]` |
| `ph2d.set`, `ph2d` (a tabela), `print`, `math.sin`, `pirnt`, `ph2d.proprety` | `[]` |

⇒ I6: um sítio neste script = o cursor vai; `self.<nome>` de uma `ph2d.property` = a declaração NOSSA
(o servidor não liga o literal ao campo, I0b §1); o resto = a documentação no balão do I5.

## A pausa das cores (Godot 4.7.2, de fábrica, já no `I0b_bruto.json`)

`text_editor/completion/idle_parse_delay = 1.5` · `idle_parse_delay_with_errors_found = 0.5` (o
editor reanalisa o script depois de a escrita parar) ⇒ `semantic::IDLE_NS` / `IDLE_ERRORS_NS`.

## A cor de alerta (`code-unresolved`, gate `the_code_inks_read_and_differ`, 12 temas)

O `danger` como LETRA reprovou: `3,49`–`4,39:1` na linha do cursor em 6 temas (Forge, Workshop,
Sunstone, Blueprint, Light, Candy; Candy `3,89` também no fundo) e ΔE `0,044` do `code-constant` nos 6
temas escuros. Token novo, procurado por script (contraste ≥ 5,5 na paleta-fonte para dar folga aos
derivados, ΔE ≥ 0,075 de toda tinta, o mais saturado): forge `oklch(0.780 0.220 38)`, sunstone
`oklch(0.500 0.220 38)`, blueprint `oklch(0.530 0.220 30)`. A 1.ª tentativa (forge `0.720 0.220 35`)
reprovou em PlumberRed `4,32`, PlumberGreen `4,01` e Candy `4,10` na linha do cursor.

## ⛔ Recusas MEDIDAS

| Proposta | Medição | Veredito |
|---|---|---|
| `danger` como letra do global não resolvido | `3,49`–`4,39:1` na linha do cursor em 6 temas (Forge, Workshop, Sunstone, Blueprint, Light, Candy); ΔE `0,044` do `code-constant` em 6 temas escuros | recusada ⇒ token próprio `code-unresolved` |
| 1.ª cor `code-unresolved` forge `oklch(0.720 0.220 35)` | PlumberRed `4,32`, PlumberGreen `4,01`, Candy `4,10` na linha do cursor | recusada ⇒ `0.780 0.220 38` (contraste ≥ 5,5 na paleta-fonte para dar folga aos derivados) |
| Deduzir o «não resolvido» dos tokens semânticos | a legenda não tem esse tipo; o servidor não emite token para global inexistente, nem para local | recusada ⇒ a cor vem do diagnóstico `1001` seguro (I3) |
| Definição noutro script | cada script é um ambiente (I0b §6): nunca acontece | fora |
