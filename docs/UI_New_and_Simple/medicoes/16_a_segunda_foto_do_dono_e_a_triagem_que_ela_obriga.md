# 16 — A 2.ª foto do dono: o que o idioma de teste acha, e o que ele NÃO distingue

> **Medido em 2026-09-17, `line/UIUX`.** A 11.ª fatia do HR-15 — a primeira conduzida por um report
> do DONO feito com o instrumento da fatia anterior.

## §1 — O report

Com `PH2D_LANG=teste` ligado, o dono fotografou seis painéis e listou **sete** bolsos de inglês:

| # | o que ele viu | o que a medição diz |
|---|---|---|
| 1 | vários botões de sculpt | ⛔ **FUGA REAL** — 124 rótulos, curada |
| 2 | objetos na Hierarchy | ⚠️ decisão escrita: *«é conteúdo, não chrome»* |
| 3 | alguns botões do Audio Mixer | ⛔ **FUGA REAL** — 4 rótulos, curada |
| 4 | quase todas as palavras do design Tokens | ⛔ **FUGA REAL** — 18 rótulos, curada |
| 5 | Authored UI inteiro | ⚠️ decisão escrita: o texto é do ARTISTA |
| 6 | Widget Gallery inteiro | ⚠️ decisão escrita: BANCADA |
| 7 | Widget lab inteiro | ⚠️ decisão escrita: BANCADA |

⭐⭐⭐ **O achado de método é a coluna da direita: o idioma de teste NÃO distingue uma fuga de uma
DECISÃO DECLARADA.** Ele responde *«esta palavra está escrita no código?»*, e a resposta é **sim**
nos sete casos — em quatro deles porque alguém decidiu isso e escreveu o mecanismo ao lado. *Um
instrumento que só mede uma propriedade sintáctica devolve a lista certa e a leitura errada; quem
separa é a triagem, e a triagem é do dono.*

## §2 — ⛔⛔ Três fugas, e as três na MESMA forma

As três crates que tinham texto cru **não são varridas por censo nenhum**, e as três são pintadas
por painéis que fecham a **ZERO**:

| motor | painel que o pinta | literais no motor | literais no painel |
|---|---|---:|---:|
| `ph2d-sculpt3d` + `ph2d-boundary` + `ph2d-pose` | `ph2d-panel-sculpt3d` | **124** | 0 |
| `ph2d-tokens` | `ph2d-panel-tokens` · `ph2d-panel-widget-gallery` | **18** | 0 |
| `ph2d-panel-audio-mixer` (o próprio) | — | **4** | — |

⚠️ *Um censo cuja crate não é DONA do texto que ela pinta fica verde sobre texto cru* — a **terceira**
e a **quarta** ocorrência, depois do `component_catalog` e do `paint_brush`.

⚠️⚠️ **E a régua do repo não as alcança por construção.** O `--resumo` do `ph2d-label-census` varre
os `ph2d-panel-*`, as `ph2d-app-*`, a `ph2d-editor-core`, a `ph2d-param-editors` e a shell. *Uma
lista de crates a varrer é exactamente onde a próxima fronteira se esconde.*

## §3 — ⭐⭐ O desenho do motor da escultura: uma lei, um acessório derivado

`Verb::label_key()` devolve `sculpt3d.verb.draw` e o texto vive na tabela. Mas **216 sítios** já
chamavam `.label()` — entre eles as tabelas de proveniência do dyntopo, que carregam o veredito do
dono escrito ao lado do nome do verbo. ⇒ `label()` passou a ser
`tr_em(Ingles, label_key())`: um **acessório derivado**, não uma segunda cópia.

| desenho | custo | o que se perde |
|---|---|---|
| renomear `label` → `label_key` em toda a parte | **216 sítios** numa crate de outra linha, VIVA | as tabelas de proveniência deixam de se ler |
| ~~uma segunda tabela no painel~~ | 1 ficheiro | ⛔ *uma lei escrita em dois sítios ainda não é uma lei* |
| **`label()` derivado da tabela** | 16 funções + 19 sítios de pintura | nada — e há gate a impedir a segunda cópia de voltar |

⛔ **As duas crates de LEI (`ph2d-boundary`, `ph2d-pose`) NÃO ganham o acessório**, e a razão está
escrita no `Cargo.toml` delas: *«a lib continua sem dependência nenhuma»*. Elas expõem `label_key()`
e mais nada.

## §4 — ⛔ O design system não podia ganhar a dependência, e isso mudou o desenho

O `Cargo.toml` da `ph2d-tokens` declara-a *«design-data puro — zero runtime deps»* e há **gate**
(`the_leaf_stays_dep_free`). ⇒ ela guarda a CHAVE e quem resolve é o painel.

⭐ **Quatro dos oito nomes de tema são NOMES PRÓPRIOS** (*Forge*, *Workshop*, *Sunstone*,
*Blueprint*) e entram na tabela na mesma: **um nome próprio na TABELA é uma decisão que um tradutor
pode tomar; um nome próprio no CÓDIGO é uma decisão que ninguém pode tomar.**

### ⛔⛔ Dois ÓRFÃOS, e a cura de um órfão é APAGAR

| função | o que devolvia | quem a lia | onde a palavra já vivia |
|---|---|---|---|
| `TextRendering::display_name` | `Default` · `Crisp Heavy` · `Crisp Heavy +` | **ninguém** | `chrome.menu.crisp_heavy` |
| `UiLook::label` | `Classic` · `Redesign` | **ninguém** | `chrome.color.classic` |

⚠️ **ÓRFÃO e MORTO leem-se igual numa tabela de risco** (`CLAUDE.md` §5.0) e as curas são **opostas**:
um controlo pintado sem consumidor **liga-se**; um texto que ninguém pinta **apaga-se**. A pergunta
que os separa é *isto chega a ser PINTADO?*, e a resposta foi medida.

## §5 — ⭐⭐⭐ O par do `dB` faltava ao lado do par do `LUFS`, no MESMO pintor

O `paint.rs` do mixer tinha `master.inf_lufs` e `master.lufs_value` na tabela desde 16/09 — e
`"-inf"`, `format!("{:.0} dB", …)`, `"M"` e `"S"` crus, **cinco linhas acima**.

⇒ **é um controlo, e o que os separa é a régua**: a `is_language` aceita `-inf LUFS` (`LUFS` é
GRITADO) e recusa `dB` (nem Capitalizado nem GRITADO) e uma letra solta (ela exige duas letras
adjacentes, para não acusar identificadores). *O mesmo ponto cego que apanhou o `default-scene` na
fatia anterior, agora com o lado APROVADO ao lado, no mesmo ficheiro.*

## §6 — ⚠️ O contrato da recusa, e uma nota minha que nasceu errada

A `NumRefusal::BadFormula` atravessa a fronteira como uma `String` de **duas espécies**: as da
`ph2d-tokens` (chaves) e a do `ph2d-token-math`, que é **dinâmica** e nomeia o identificador que não
foi entendido. ⇒ `CHAVE_DE_RECUSA`, o prefixo, com gate.

⛔ **Eu escrevi que a nota do toast prometia detalhe que o código nunca produz — e a nota estava
CERTA.** Eu tinha lido só o produtor da `ph2d-tokens` (frase fixa e genérica) e não o do motor de
fórmulas. Desfeito no mesmo dia. *Uma acusação de doc-que-mente precisa de contar os produtores,
não de ler um.*

⭐ E uma chave montada por `format!("{CHAVE}nome")` é **invisível aos dois censos**: o lexical lê o
modelo como língua e o de chaves lê as que estão **escritas**. ⇒ as chaves escrevem-se INTEIRAS, e
quem afirma o prefixo é o gate.

## §7 — ⛔ Um vermelho PRÉ-EXISTENTE, com o controlo medido

`ph2d-panel-sculpt3d::seam::every_command_reaches_the_shell` estoura a pilha e **reproduz sozinho a
`load 11`** — não é flake de carga. **CONTROLO:** corrido numa worktree limpa em `7fe4b8b71`, antes
desta fatia, ele estoura igual. Passa com `RUST_MIN_STACK=3145728`; o limiar do nextest é 2 MB e o
produto pinta na thread principal (8 MB), logo **não há defeito de produto**. Fica nomeado.

## §8 — Os números

| | |
|---|---:|
| chaves novas | **146** (124 escultura + 18 design system + 4 mixer) |
| crates que ganharam o primeiro censo da vida | **2** (`ph2d-sculpt3d`, `ph2d-tokens`) |
| gates novos | **9** |
| funções órfãs apagadas | **2** |
| sítios de pintura religados | **21** |
| sítios de `.label()` que NÃO precisaram de mudar | **216** |

**Prova de mutação: 8 de 8**, com controlo negativo verde e controlo sobre o próprio filtro.

## §9 — ⏳ O que fica, e é decisão do DONO

As quatro linhas da coluna «decisão escrita» do §1. Cada uma tem o mecanismo no código:

| item | onde a decisão está escrita | o argumento |
|---|---|---|
| objetos na Hierarchy | `ph2d-field-ecs/src/spawn.rs` | o nome é **dado do documento** e o artista renomeia-o; traduzi-lo ao pintar reescrevia o nome dele |
| Authored UI | `ph2d-panel-authored/Cargo.toml` | *«sem `ph2d-i18n`, e a ausência é a decisão»* — o texto do painel é o que o ARTISTA autorou |
| Widget Gallery | `ph2d-editor-core/tests/it/no_label_…` | BANCADA: os rótulos são os NOMES DOS NOSSOS WIDGETS (`Rect2Editor`, `BitmaskGrid32`) |
| Widget Lab | `ph2d-panel-widget-lab/tests/it/every_word_…` | BANCADA: cabeçalhos de estudo e legendas que são a CONTA de uma medição |

⭐ **A Hierarquia tem uma saída que as outras três não têm:** traduzir no **NASCIMENTO** (o
*Translate New Data* do Blender, que ali é uma caixa e nasce desligada). Custa ~65 chaves e faz um
projecto guardar os nomes do idioma em que foi criado — que é o que uma segunda língua real também
faz.
