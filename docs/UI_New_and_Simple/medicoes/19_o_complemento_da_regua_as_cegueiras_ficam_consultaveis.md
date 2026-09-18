# 19 — O complemento da régua: as cegueiras dela deixam de ser prosa e passam a ser uma lista

> **Medido em 2026-09-18, `line/UIUX`.** A 16.ª fatia do HR-15 — a segunda da caça proactiva, e a
> primeira em que o INSTRUMENTO veio antes das curas.

## §1 — Porque um instrumento, e não mais uma leitura

A fatia anterior curou as LETRAS SOZINHAS depois de as achar **a ler um painel à mão**. A cegueira
que as escondia estava escrita no doc-comment do `is_language` desde sempre, como exclusão
declarada — e ficou lá enquanto três painéis pintavam letras cruas com o censo verde. *Uma regra sem
instrumento é uma nota que envelhece* (`CLAUDE.md` §2), e a prova era essa.

⇒ o critério da régua passou a devolver **QUAL cerca recusou o texto** (`Cegueira`), e o
[`is_language`] é hoje o **acessório derivado** dela — ⛔ nunca uma segunda cópia do critério, que
divergiria no dia seguinte.

| cerca | o que ela recusa | porque existe |
|---|---|---|
| `SemDuasLetras` | `X` · `%` · `m/s` | senão acusaria todo param `"x"`, `"n"`, `"b"` |
| `TokenNu` | `snake_case` · `repeats` · `EQ` · `9-Slice` | senão acusaria todo id |
| `ParecemChave` · `ParecemCaminho` · `NomeDeFicheiro` | `chrome.fill.title` · `src/main` · `Brush.png` | têm forma própria |

**`cargo run -q -p ph2d-label-census --example censo -- --cegos <raiz>`** lista as duas primeiras —
⛔ **é TRIAGEM, nunca um gate**: quase tudo ali é identificador legítimo, e *alargar a régua para os
apanhar acusaria centenas de ids, que é o defeito oposto e pior.*

⭐ **E a metade NEGATIVA do instrumento valeu um terço da lista:** o literal **VAZIO**
(`Dropdown::new(id, "", …)`, o dropdown sem rótulo) saiu — *uma palavra não se esconde no vazio, e
uma lista de triagem com um terço de ruído é uma lista que ninguém lê* (Inspector `200 → 126`).

## §2 — ⛔ O que ele achou na primeira corrida: nove rótulos pintados, cinco painéis

| painel | texto | porque a régua não o via |
|---|---|---|
| Audio Mixer | **`EQ`** (cabeçalho de secção) | uma palavra GRITADA conta a partir de **três** letras (senão `UV` e `RGBA16` seriam língua) |
| Inspector | **`9-Slice`** (título de secção) | começa por **dígito** ⇒ nem Capitalizado nem GRITADO |
| Inspector | **`PP`** (direcção ping-pong) | idem — e estava **entre quatro irmãs** que já vinham da tabela |
| Inspector | **`RGBA8`** · **`RGBA16`** | idem |
| Equalize Sizes · Painter Layers ×2 · Vector · Color EQ | **`{} px`** · **`{px} px`** · **`{:+.2} EV`** | tirado o marcador sobra `px`/`EV`, que é um token nu |

⭐ **Dois deles são a forma mais legível que este defeito tem: um ESTRANHO numa lista de chaves.** O
`EQ` estava ao lado de `tr(…delay)` e `tr(…comp)`; o `PP` ao lado de quatro `tr(…)`.

⚠️ E os cinco leitores de unidade são a **mesma família** do `{db} dB` que a fatia do mixer já tinha
curado — *é a TABELA que decide de que lado da frase a unidade vive*.

## §3 — ⭐ O que protege a cura, e o que NÃO protege

⭐⭐ **A migração traz a própria guarda:** assim que o texto vira uma chave com o prefixo do painel,
o gate `every_key_of_this_panel_exists_on_both_sides` — que **já existia** — reprova se alguém
apagar a entrada da tabela (medido: apagar `panel.audio_mixer.master.eq` põe-no vermelho).

⚠️ **A direcção contrária — alguém escrever OUTRA vez um literal cego — não tem gate**, porque a
régua é cega a ele por construção. Onde o pintor é **privado da crate**, a cura é o TIPO: o
`section_header` do mixer passou a receber `TextKey` e escrever `"EQ"` lá **não compila**. ⛔ Onde o
pintor é partilhado (`ph2d_editor_core::widget::section_header`, que dezenas de painéis chamam),
tipá-lo é uma wave própria — **dívida NOMEADA**, e a rede entretanto é correr o `--cegos` ao fechar
a linha.

## §4 — Os números

| | |
|---|---:|
| rótulos migrados | **9** em 5 painéis |
| chaves novas | **9** |
| pintores TIPADOS (`&str` → `TextKey`) | **1** |
| portas novas na régua | **2** (`cegueira` · `blind_literals_in`) + o modo `--cegos` |
| gates novos | **1** (2 metades: a tabela de razões · o complemento, com controlo do vazio e do que a régua aceita) |
| provas | a chave em falta reprova o gate que já existia · o literal cru **não compila** no mixer |
| suítes | i18n 17 · mixer 29 · inspector 293 · equalize 10 · painter-layers 179 · vector 298 · color-eq 20 · censo 21 |
| clippy `--workspace --all-targets -D warnings` | **0** |

⚠️ **Uma correcção ao meu próprio teste:** eu escrevi que `assets/brush.png` é recusado pela
extensão, e ele é recusado pelo **caminho** — a barra vem primeiro. *Uma lista de razões só é honesta
se a ORDEM delas estiver medida*, e a linha ficou no gate com o porquê.
