# 17 — O cartão pintava a CHAVE, e o gate do cartão estava VERDE

> **Medido em 2026-09-18, `line/UIUX`.** A 14.ª fatia do HR-15 — a segunda conduzida por uma
> FOTOGRAFIA do dono, e a primeira em que ele mandou eu próprio abrir o painel.

## §1 — O report

Foto do cartão do `source.lsystem` com `PH2D_LANG=teste`: tudo à volta deformado
(`[Ĺ-Šýšŧéɱ·····]`, `[Ṁóðé··]`, `[Ŕúĺéš···]`) e **três cabeçalhos de secção em cru** —
`node.group.shape`, `node.group.leaves`, `node.group.grammar`. Mais a ordem: *«VC mesmo deve abrir
o painel e conferir. Não consigo fazer o smoke.»*

## §2 — ⛔⛔ A cegueira: o gate varria uma lista e o texto vivia na IRMÃ

A fatia anterior (`2182a31c9`) converteu **247** sítios de `ParamGroup` em chaves e curou o
**painel lateral** (`rows_paint_sections::paint_header`, que resolve no último instante porque o
título ali é identidade *e* legenda). O **CARTÃO** tem caminho próprio — `stamp_card_params` →
`CardSection` → `paint_card_params::draw_section_header` — e ficou a pintar a chave.

⚠️⚠️ **E o gate que existe exactamente para isto ficou verde:**
`no_card_param_of_any_node_paints_a_raw_key` varre `v.params` e o título de uma secção vive em
`v.sections`. Os três testes daquele ficheiro varrem **uma lista cada** (`snap.rows` · `v.params` ·
os rótulos de canal) e **nenhum** percorria a quarta.

⭐⭐⭐ *Uma família de texto NOVA não herda régua nenhuma só por o consumidor dela já ter uma.* A lei
que fica: **quem acrescenta uma lista ao snapshot escreve o teste dela** — e o cabeçalho daquele
ficheiro passou a dizê-lo, com a contagem (`QUATRO superfícies`).

## §3 — ⚠️⚠️ A cura: dois campos, porque são duas perguntas

O nome do grupo é **identidade** (o `ToggleParamSection` endereça a memória de dobra por ele) *e*
**legenda**. O cartão declara por lei que o snapshot é feito de primitivos **RESOLVIDOS** (um `tr`
no pintor sobre algo que já não é chave faz `Box::leak` por quadro e por linha). ⇒ as duas leis só
se conciliam com **dois campos**:

| campo | o quê | quem lê |
|---|---|---|
| `CardSection::key` | `node.group.shape`, crua | `interact_param_row` (a dobra) · o `aberta(g)` da ponte |
| `CardSection::label` | a palavra, `tr` **uma vez** na ponte | `paint_card_params` |

⛔ O `title` foi **renomeado**, não duplicado: com o nome antigo o pintor continuaria a compilar a
pintar a identidade. *A troca tem de ser um erro de compilação em todos os leitores.*

## §4 — ⭐ O gate novo tem DUAS metades, e elas reprovam erros OPOSTOS

`no_card_section_of_any_node_paints_a_raw_key`:

- a `label` **não pode** ser uma chave — o defeito da foto (o `tr` em falta);
- a `key` **tem de continuar** a ser uma chave — a «cura» errada (traduzir a montante), que faria
  a secção fechada abrir sozinha ao trocar de idioma.

**Prova de mutação: 2 de 2.** A primeira devolve a mensagem com `node.group.shape` à letra; a
segunda lê `"Shape"` onde tem de estar a chave. ⚠️ *Só a primeira metade deixaria a segunda cura
passar como correcção.*

## §5 — ⛔ A fotografia NÃO chegou para verificar, e isso é um achado do instrumento

`fotografa_cena.sh` abriu a cena `=108` em `1930×1040` com o idioma de teste. O grafo aterra
**afastado** e, a esse LOD, `draw_section_header` desenha o chevron e **salta o texto**
(`com_texto == false`) — a foto do dono foi tirada com o grafo aproximado, e os eventos sintéticos
não chegam à Xwayland virtual (cabeçalho do roteiro).

⇒ a verificação correu pela **porta do produto** (`snapshot_from` + `stamp_card_params`), numa sonda
temporária com `PH2D_LANG=teste`:

```
=== source.lsystem ===
  CHAVE node.group.shape          -> PINTA "[Šĥáþé···]"
  CHAVE node.group.leaves         -> PINTA "[Ĺéáṽéš····]"
  CHAVE node.group.growth         -> PINTA "[Ĝŕóŵŧĥ····]"
  CHAVE node.group.lean_and_look  -> PINTA "[Ĺéáñ & Ĺóóķ········]"
```

⚠️ **A foto continua a valer como metade:** ela prova que o painel abre e que tudo o resto do ecrã
está deformado. *Um instrumento que não alcança o regime do report não o desmente — diz que é preciso
outro.*

## §6 — Os números

| | |
|---|---:|
| ficheiros de produto tocados | **4** |
| sondas/gates realinhados (`.title` → `.key`/`.label`) | **8** |
| gates novos | **1** (duas metades) |
| provas de mutação | **2 de 2** |
| suítes | `ph2d-app-motion` 1 149 · `ph2d-panel-motion-graph` 191 |
| clippy `--workspace --all-targets -D warnings` | **0** |
