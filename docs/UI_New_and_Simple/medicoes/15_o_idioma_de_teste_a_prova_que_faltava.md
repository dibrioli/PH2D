# 15 — O idioma de teste: a prova que faltava às nove fatias

> **Medido em 2026-09-17, `line/UIUX`.** A 10.ª fatia do HR-15, e a primeira que não move texto
> nenhum: ela constrói o **instrumento** que torna as nove anteriores falsificáveis.

## §1 — ⛔⛔ Trinta censos lêem o CÓDIGO e nenhum lê o ECRÃ

Nove fatias moveram **4 986** chaves e **3 692** sítios de chamada para uma tabela, com **30**
censos a defendê-los. Os trinta são **lexicais**: eles abrem o fonte de uma crate e perguntam
*«há aqui um literal com cara de língua?»*.

⚠️ **Essa não é a pergunta do HR-15.** A pergunta é *«a palavra que o artista LÊ saiu da tabela?»*,
e as duas divergem no caso que importa: **um rótulo esquecido no pintor pinta-se exactamente igual
ao que veio da tabela**. Nada nesta árvore os conseguia distinguir — e o dono, a olhar para o ecrã,
muito menos.

⇒ `PH2D_LANG=teste`: cada palavra que sai da tabela volta **acentuada e mais comprida**. O que ficar
em inglês normal no ecrã está, por construção, escrito no código. ⭐ *É o único censo deste repo que
o dono pode correr sozinho.*

## §2 — ⭐⭐ O alongamento é MEDIDO, e a medição MUDOU o desenho

A 1.ª redacção levava `+35 %` com a justificação *«que é quanto uma língua cresce»* — um número sem
recurso nomeado, que o `CLAUDE.md` §0.0 proíbe. O oráculo estava instalado (§0.9): **quatro editores
da nossa classe** — Krita · Inkscape · GIMP · Synfig Studio — trazem os catálogos `.mo` deles em
`/usr/share/locale/`, e o que se lê ali é **saída**, nunca fonte. **89 832** pares rótulo↔tradução,
**5** línguas (pt-BR · de · fr · es · it), excluindo o que não é rótulo:

| original (caracteres) | n | p50 | p75 | **p90** | p95 |
|---|---:|---:|---:|---:|---:|
| 1..6 | 7 981 | 1,17 | 1,50 | **2,00** | 2,40 |
| 7..12 | 20 375 | 1,20 | 1,50 | **1,83** | 2,08 |
| 13..20 | 23 831 | 1,25 | 1,47 | **1,70** | 1,87 |
| 21..35 | 18 806 | 1,24 | 1,42 | **1,61** | 1,73 |
| 36..70 | 13 171 | 1,19 | 1,33 | **1,48** | 1,58 |
| 71+ | 5 668 | 1,16 | 1,26 | **1,37** | 1,44 |
| TODOS | 89 832 | 1,21 | 1,42 | 1,67 | 1,88 |

⭐⭐ **O achado é que o alongamento NÃO é uma constante: ele cresce quando o rótulo encolhe.** Um
rótulo de até 6 caracteres **dobra** no percentil 90 e um parágrafo cresce `1,37` — ⇒ um número
único estaria `1,65×` abaixo do necessário exactamente nos rótulos curtos, que são os que vivem nas
colunas apertadas deste app. *A tensão faltaria onde a interface parte.*

⚠️ **A barra é o p90 e é uma escolha declarada:** nove em cada dez traduções reais cabem. As outras
não — *isto é uma tensão, não uma prova de suficiência*. E **`11,1 %` dos pares MEDIDOS encolhem**:
o idioma de teste nunca encolhe, e não precisa.

Nenhuma string dos alvos entra neste repo: o que saiu da medição é **um número por balde**.

## §3 — As três leis, e o que cada uma impede

| lei | o que acontece sem ela |
|---|---|
| `{marcador}` sai **verbatim** e não conta para o alongamento | o `tr_with` deixa de o achar e a frase pinta `{n}` na tela |
| a chave **desconhecida** fica CRUA | `tr(k) != k` é a pergunta *«esta chave existe?»* em meio repo — deformar o caminho da falha parte **todos** esses censos de uma vez |
| o inglês devolve o **mesmo ponteiro** de sempre | a fatia podia ter mudado o produto em silêncio, com os outros cinco gates verdes: eles medem o idioma de TESTE |

⚠️ E a deformação é **memorizada** (`BTreeMap` atrás de `RwLock`, a chave é o ponteiro do literal):
deformar por chamada seria um `Box::leak` **por quadro**, que é o defeito que o doc da chave
desconhecida já nomeia. ⛔ `BTreeMap` e não `HashMap`, que é **tipo proibido** neste repo por lint
estrutural — *uma excepção a uma lei estrutural custa mais do que os `log n` que ela poupa*.

## §4 — ⭐⭐⭐ A PRIMEIRA foto achou uma palavra que a régua lexical não pode ver

O ecrã de arranque fotografado no idioma de teste tinha **uma** palavra inglesa: **`default-scene`**,
no canto da barra de estado.

⛔⛔ **E os trinta censos são cegos a ela POR CONSTRUÇÃO.** A `is_language` recusa um token nu que
não seja Capitalizado nem GRITADO, para não acusar identificadores. Medido:

| literal | `is_language` | está no ecrã? |
|---|---|---|
| `Window` · `EDIT` · `Hierarchy` | **true** | sim |
| `default-scene` · `sprites` · `bodies` | **false** | sim |

⇒ *um rótulo em minúsculas é invisível a toda a família lexical*, e o idioma de teste achou-o na
primeira fotografia. As duas réguas têm pontos cegos **complementares**, e só a segunda olha para o
ecrã.

⚠️ **A cura é keyar, e a pergunta de fundo fica aberta:** `default-scene` é um **marcador de lugar**
e não um leitor — o HUD não sabe o nome da cena (o vizinho `100%` é o mesmo caso, e safa-se de ser
visto por não ter letras). Ligá-los ao projecto é obra de produto.

**Depois da cura: zero palavras inglesas no ecrã de arranque.**

## §5 — ⚠️ O CONTROLO desmentiu metade do que eu ia escrever

A foto no idioma de teste mostra rótulos cortados por todo o lado, e a 1.ª leitura foi *«a tensão
revelou que a moldura não tem folga»*. ⛔ **A foto de CONTROLO, em inglês, já corta em treze
sítios** — `Windo…` · `Hierarc…` · `Inspect…` · `Anima…` · `M…` · `SC…` · `PI…` · `SP…` · `VI…` ·
`U…` · `RE…` · `GI…` · `S…`, mais a frase do Inspector.

⇒ *a elisão é PRÉ-EXISTENTE e não é achado desta fatia.* Se o idioma de teste corta **mais** é uma
pergunta que precisa de uma medição em pixels (a porta existe: o
`cada_nome_deste_painel_cabe_na_coluna_da_seccao` corrido com o idioma ligado), não de dois olhos
sobre duas imagens.

## §6 — ⚠️ E a sonda que tentou DIMENSIONAR o ponto cego mediu a própria heurística

Escrita uma varredura por «palavra minúscula» sobre as crates de UI, ela devolveu **1 592**
achados — e a amostra é `«guard matched»`, `«deterministic»`, `«slider missing»`, `«a shell publicou
metros»`: mensagens de `assert`, nomes de campo, identificadores.

⛔ **Aquele número não mede o ponto cego; mede o meu filtro.** Uma heurística lexical não separa um
RÓTULO minúsculo de um IDENTIFICADOR minúsculo — que é exactamente a razão de a `is_language` os
recusar em bloco. ⇒ *para esta classe, o ECRÃ é o único oráculo honesto*, e é por isso que a fatia
entrega um instrumento visual em vez de mais uma varredura. A sonda foi apagada.

## §7 — ⛔ O que ele NÃO prova, declarado

O idioma de teste é **derivado do inglês**, logo duas chaves diferentes com a mesma palavra
deformam-se igual. ⇒ o `the_tab_and_the_menu_call_a_panel_the_same_thing` **continua a comparar uma
palavra com ela própria** e fica por acordar. *Aquele gate só acorda com uma língua AUTORADA.*

⚠️ Salgar a deformação com a chave acordá-lo-ia — e poria ~13 painéis **permanentemente vermelhos**
sobre um estado já declarado, com bloqueador nomeado (`medicoes/12` §6.1). Fica de fora **de
propósito**.

⚠️ E um **nome por omissão** criado no idioma de teste nasce deformado e **grava-se assim**
(`Palette {n}` e irmãos vivem na tabela, por lei da casa). *Um projecto feito no idioma de teste
guarda os nomes desse idioma* — é o que uma segunda língua real também faz.

## §8 — ⛔ Um tecto de LOC estava VERMELHO desde a fatia 7, e era meu

O `ph2d-i18n/src/lib.rs` passou `695 → 722` no commit da aba (`721f3dfa9`) e ficou em **725** contra
o tecto de **700**, vermelho durante três commits. ⚠️ **Passou despercebido porque as minhas corridas
cobriam a família dos censos e as crates editadas**, e aquele gate vive em
`ph2d-editor-core/tests/it/` — a cegueira que este repo já registou seis vezes.

Curado por **CORTE por responsabilidade**, nunca por uma entrada de dívida, e em quatro assuntos:

| saiu | para | porquê |
|---|---|---|
| `input_map.*` (12) | `input_map.rs` (novo) | uma JANELA com plano próprio não é assunto do encaminhador |
| `tool.*` (24) | `chrome_rail.rs` | ⭐ **o cabeçalho do destino já os nomeava por escrito** — eles são a fila horizontal daquela barra |
| `edit.*` (4) | `chrome_menus.rs` | o menu Edit |
| `tr_with` (41 L) | `formato.rs` (novo) | ⭐ e ela fica **ao lado da lei que a tem de respeitar**: o `pseudo::deforma` copia `{nome}` verbatim precisamente porque esta função o vai procurar |

`725 → 649`.

## §9 — Os números

| | |
|---|---:|
| chaves na tabela | **4 986** |
| sítios de `tr`/`tr_with` na árvore | **3 692** |
| linhas de chamada tocadas para ligar o idioma novo | **0** |
| palavras inglesas no ecrã de arranque, antes / depois | **1** / **0** |
| gates novos | **7** |
| `lib.rs` | `725 → 649` |

**Prova de mutação: 7 de 7 sangram**, com controlo negativo verde e **controlo sobre o próprio
filtro** (o arnês reprova se o filtro casar zero testes) — a palavra a deixar de ser deformada · o
marcador a ser acentuado · a chave desconhecida a ser deformada · o alongamento a virar constante ·
a memória a desaparecer · o inglês a passar pelo caminho novo · e a varredura das tabelas a voltar a
colher todo `.rs`.

⚠️ **O M7 existe porque a 1.ª corrida dos gates acusou duas «chaves» que nunca o foram:** com
prefixo **vazio** a régua perde o filtro que a protege nos outros ~30 censos, e leu `"ingles" =>` e
`"xx" =>` do `match` de **variável de ambiente** do próprio `idioma.rs`. *Um censo com o filtro
vazio mede tudo o que a FORMA dele casa, e a forma de um braço de `match` é a mesma em toda parte.*

## §10 — ⏳ O que fica

| alvo | nota |
|---|---|
| a elisão em pixels **com o idioma ligado** | a porta existe (`cada_nome_deste_painel_cabe_na_coluna_da_seccao`); o que falta é corrê-la no idioma de teste e escrever a escada |
| o MENU e a ABA | §7 — só uma língua **autorada** acorda aquele gate; a ponte é o registo em runtime (`medicoes/12` §6.1) |
| `default-scene` e `100%` | marcadores de lugar no HUD; ligá-los ao projecto é produto |
| o `keys_used` conta USOS em ficheiros de TESTE | ver `13_…` §8 |
