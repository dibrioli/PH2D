# 24 — A fronteira dos MOTORES, e a régua que atravessa a crate

> **2026-09-19, `line/UIUX`.** A frente que eu tinha anunciado — *«medir quais painéis ainda não têm
> a conferência automática de texto que os outros têm»* — estava **fechada** antes de eu lá chegar.
> Medi-la achou outra, que nenhuma régua deste repo conseguia ver, e que já tinha custado **três**
> fotografias do dono.

---

## 1. A primeira medição matou a minha própria frente

`grep -rl language_literals` devolve **7** crates, e eu li isso como *«só 7 painéis têm censo»*.
Falso: os painéis migrados a partir de 2026-09-16 chamam o corpo partilhado
(`ph2d_label_census::gate`) e **não** mencionam a função. Medido pela dependência em vez de pelo
nome:

| população | tem censo |
|---|---:|
| crates `ph2d-panel-*` | **28 de 29** |
| a 29.ª (`ph2d-panel-registry-init`) | é **gerada** por `cargo run -p ph2d-app-sync` |

⛔ *Uma ausência afirmada por um `grep` de NOME é um palpite com cara de medição.* A frente que eu
anunciara não existia: todos os painéis têm a conferência.

---

## 2. A frente a sério: o rótulo que ATRAVESSA a fronteira de crate

As duas réguas do HR-15 são cegas a ele, e por construção:

| régua | vê o literal? | sabe se ele é PINTADO? |
|---|---|---|
| lexical (`ph2d-label-census`, 30 gates) | sim — **se a crate estiver na lista** | não |
| de porta (`scripts/censo-texto-pintado.py`) | sim | sim, **dentro da MESMA crate** |

Quando o motor publica a palavra e o painel a pinta, **as duas ficam verdes**: do lado do motor não
há pintor nenhum a seguir, e do lado do painel o literal não existe.

> ⛔⛔ O cabeçalho do `ph2d-i18n/src/sculpt_engine.rs` já escrevia isto por extenso, depois da
> **terceira** ocorrência: *«um censo cuja crate não é DONA do texto que ela pinta fica verde sobre
> texto cru»* e *«nenhuma das 30 réguas lexicais o podia ver — o motor não está na lista»*. As três
> foram achadas por uma **fotografia do dono**. ⇒ *uma cegueira escrita em prosa não é medida, e uma
> que só o dono encontra custa um report por ocorrência.*

---

## 3. A régua nova: `ph2d_label_census::fronteira`

Um literal com cara de língua que sai da crate por uma de duas portas:

- **`Publicacao::Funcao`** — devolvido por uma `fn … -> &str` (com `'static` ou sem);
- **`Publicacao::Campo`** — o valor de um campo `label`/`name`/`title`/`text` num literal de struct
  (é assim que um CATÁLOGO publica: o `ShapeDesc` do vector, os moldes do L-System).

### ⛔ A cegueira que ela pagou na primeira hora

A 1.ª redacção exigia `-> &'static str` e lia **zero** nos **dez nomes de ferramenta** —
`fn label(&self) -> &str` é a assinatura que o contrato `Tool` (§6, congelado) impõe.

> *Uma função de trait não escolhe o tipo de retorno dela.* A régua aceita as duas formas e o
> controlo `a_regua_ve_a_assinatura_que_o_contrato_impoe` guarda-o.

---

## 4. O que ela achou, e a TRIAGEM que decide a cura

**203** nomes publicados em 16 crates; **115** sem ponte na primeira corrida honesta. A triagem é
a terceira espécie do `CLAUDE.md` §5.0 — *um ÓRFÃO lê-se exactamente como um MORTO, e as curas são
opostas* —, e ela partiu a lista em três:

| veredito | quantos | cura |
|---|---:|---|
| **órfão** (zero consumidores de produto) | **27** | apagar |
| **pintado cru** | **43** | chave derivada da VARIANTE + tabela |
| **não é rótulo** (isenção com mecanismo) | o resto | uma linha, com o porquê |

### 4.1 Os órfãos, provados pelo compilador

A sonda renomeou cada acessório e correu `cargo check --workspace --all-targets`: a workspace
**inteira** compilou.

| acessório | palavras | quem pinta de facto |
|---|---:|---|
| `ph2d_ecs::BlendMode::label` | 6 | o Inspector, com a tabela de CHAVES dele (`panel.inspector.material.*`) |
| `ph2d_ecs::AnimDirection::label` | 4 | o Inspector (`anim_rows.rs`, cujo doc já dizia *«espelha `ph2d_ecs::AnimDirection::label`»*) |
| `ph2d_ecs::BoundProp::label` | 5 | o painel de Vector, pelo DISCRIMINANTE |
| `LutPreset::group` | 5 | **ninguém** — prometia um agrupamento que o selector nunca fez |
| `ShapeGroup::label` | 7 | o `group_i18n_key` do painel, chaveado pela VARIANTE |

⚠️ Os três primeiros são cópias que existem **de propósito**: aqueles painéis não dependem do
`ph2d-ecs`, e o snapshot leva só o `tag()`. ⇒ traduzi-las criaria uma **segunda** chave para a mesma
palavra. *Duas respostas à mesma pergunta concordam até ao dia em que uma muda.*

### 4.2 Os pagos

| motor | palavras | onde o artista as lê |
|---|---:|---|
| `AudioBus` | 4 | o selector de barramento da secção **Audio** do Inspector |
| `SignalVerb` | 7 | as linhas de **Signal Actions** do Inspector |
| `BrushFalloff` | 4 | os chips de queda do painel **Bg Removal** |
| `LutPreset` | 16 | o selector de *look* do **Color Equalization** |
| `ReshapeKind` | 8 | os oito botões de remodelar do painel **Flip** |
| `PaintMedia` | 4 | o dropdown de **meio** do Painter (`Digital`/`Watercolor`/`Impasto`/`Wet Paint`) |

---

## 5. ⛔⛔⛔ E a fatia que escreveu a régua caiu no defeito que ela descreve

Migrar um motor tem **duas** metades, e o molde do `sculpt_engine` só gateava a primeira. Eu escrevi
`label_key()` nos seis motores e deixei **três painéis** (bgremoval · flip · color equalization) a
chamar o `label()` inglês. As 28 palavras continuavam presas ao inglês, e **tudo ficava verde**:

- `cada_chave_de_motor_esta_declarada_na_tabela` — as chaves existem;
- o censo lexical de cada painel — não há literal nenhum lá;
- `a_fronteira_dos_motores` — do lado do motor já não há palavra crua.

> ⚠️ *Um acessório de conveniência que devolve a língua de omissão é indistinguível, no ecrã de hoje,
> da porta certa — e só deixa de o ser no dia em que existir uma segunda língua.*

A cura é o terceiro gate, e ele é **sólido** por uma razão que vale a pena escrever: um receptor pode
ser anónimo (`kind.label()` — foi assim que o painel do Flip me escapou), mas **para chamar o método
o ficheiro tem de NOMEAR o tipo**. ⇒ a pergunta *«este ficheiro nomeia o tipo E chama o método?»*
nunca erra para o lado BAIXO.

⚠️ E ela lê o fonte **sem comentários** (`ph2d_label_census::sem_comentarios`): dois painéis do
Inspector citam `SignalVerb::uses_arg` em **prosa**, e *um tipo nomeado num comentário não pode ser
chamado*. Sem essa porta, a cura seria uma lista de isenções por ficheiro — que esconderia a chamada
REAL que aparecesse ali amanhã.

---

## 6. O que fica

- **`POR_PAGAR` está VAZIA.** Uma catraca vazia é a mais apertada que existe: já não há linha onde
  escrever um rótulo cru em silêncio.
- **`ISENTOS` do gate do pintor está VAZIA** também.
- As isenções do gate do motor são **nomeadas, com o mecanismo** — a tabela de strings a si própria,
  os rótulos de depuração do `wgpu`, a bancada de widgets, uma cor em hexadecimal, as três frases de
  TERMINAL do Painter (§0.8), a proveniência inglesa dos moldes do L-System, e os dez nomes de
  ferramenta.

### O custo do gate, medido — e o que a medição NÃO diz

`52,7 s` para as duas metades, a `load 51,69`. ⛔ **Não leia isso como um relógio**: nenhuma leitura
desta workstation vale acima de `load ~5` (§5.0), e uma corrida anterior deu `39 s` a `load ~15`
sobre código **pior**. O que se pode afirmar é uma **contagem de operações**: a 1.ª redacção do
`published_names` lia e partia em linhas o ficheiro **uma vez por literal** (no `ph2d-i18n`, 165
vezes o mesmo `lib.rs`), e hoje é uma vez por FICHEIRO.

⚠️ O grosso do que resta é a varredura lexical sobre 16 árvores, **duas vezes** — a memória por raiz
vive no processo e o `nextest` dá um processo a cada teste. Fica nomeado: se este gate se aproximar
do tecto de morte de `180 s`, a cura é juntar as duas metades num teste só, e o preço disso é perder
os dois nomes.

### ⏳ Aberto e nomeado

- **Os dez nomes de ferramenta** ficam crus por MEDIÇÃO, não por preguiça: o nome que o artista lê ao
  escolher uma ferramenta vive no rail (`chrome.rail.*`), e a `Tool::label()` chega a pixel só na
  barra de título (uma linha de diagnóstico) e na paleta do caminho legado sem-herói.
- **Um painel flutuante SEM PINTURA continua a ser testado ao toque**: a pintura do `FloatingPanel`
  legado foi retirada em 2026-05-17 e o `build_panel()` sobrevive como fonte de RECTÂNGULOS para o
  hit-test (`input_handlers.rs`). Achado desta medição; curá-lo mexe no contrato `Tool` (§6).
- O molde do `sculpt_engine` é anterior ao terceiro gate: as **sete** famílias que ele já migrou
  ficam fora da varredura do pintor, e acrescentá-las é uma linha por família.

---

## 7. A segunda metade: perguntar ao ECRÃ, e não ao fonte

A régua do §3 lê **fonte**. Ela responde *«esta crate publica uma palavra crua?»* e não responde
*«a palavra que o painel PINTOU saiu da tabela?»* — que é a pergunta do HR-15. Até aqui quem a
respondia era o **dono**, a olho, com `PH2D_LANG=teste` numa fotografia; três defeitos desta família
foram achados assim, um report cada.

### A varredura já existia — faltava-lhe a pergunta

O `nenhum_rotulo_do_app_pinta_nada` pinta **todo painel do registo** e regista cada rótulo medido.
⇒ o gate novo usa a mesma varredura e pergunta, de cada texto, se a **tabela sabe produzi-lo**:
exacto, ou preenchendo um modelo `{marcador}`.

> ⚠️ **Ele é sólido num sentido só, e está declarado:** um texto que a tabela não produz está, por
> construção, escrito no código; um que ela produz **pode** ser coincidência. *O erro fica do lado
> barato.*

### Como a lista encolheu, e o que cada corte ensinou

| passo | acusados | o que o corte diz |
|---|---:|---|
| a 1.ª corrida | **231** | — |
| filtrar pelo que é uma PALAVRA (`is_language`, a régua dos 30 censos) | **39** | ⚠️ a maioria eram VALORES: `"0.010"`, `"-9.81"`, `"0:00.0 / 0:00.0"`, `"▶"`, `"☰"`. *Um número que um painel pinta não é uma palavra, e uma lista de isenções sobre eles seria uma lista de números escritos à mão.* |
| ⛔ descodificar `\u{…}` no leitor da tabela | **40** | a tabela declara `"+ Track  \u{25be}"` e o painel pinta `+ Track ▾` — **a régua comparava `u{25be}` com `▾`**. Curá-lo tirou seis acusações e acrescentou uma (o piso dos modelos caiu de `606` para `415`, porque a chaveta do escape disfarçava 191 textos de modelo). |
| a BANCADA e o painel AUTORADO | **6** | decisão do dono, já registada; e o `authored` é o painel que **o app escreve** a partir da árvore que o artista desenhou — `Design`/`Code` são conteúdo do documento |
| a regra do texto **já cortado** | **3** | `"Mas…"` é `Master` medido outra vez depois da elisão. ⇒ regra, não isenção: *uma lista de isenções sobre texto elidido muda sempre que uma coluna muda de largura.* |

### ⭐ As TRÊS que sobraram eram dívida, e nenhuma seria achada de outra maneira

| painel | o que estava cru | a cura |
|---|---|---|
| **Asset Browser** | `Name · Type · Recent` | o `ph2d-asset-index` publicava-os; hoje publica a CHAVE |
| **Physics** | `Bodies: 0` | só a PALAVRA vinha da tabela; o `": "` estava no `format!` |
| **Tokens** | `Forge  —  0 authored` | as duas palavras vinham da tabela; a FORMA da linha não |

> ⭐⭐ *Uma frase composta é um MODELO. Traduzir só as peças dela deixa a gramática no fonte* — e há
> línguas em que o dois-pontos leva espaço antes, e em que a ordem das peças muda.

### ⛔⛔ E o `Recent` expôs a cegueira da régua do §3

O filtro de população dela é *«a crate depende da `ph2d-editor-core` ou da `ph2d-i18n`»*, e o
`ph2d-asset-index` **não dependia de nenhuma** — ele ficava fora da varredura e aquele gate fechava
verde sobre ele. A relação verdadeira é *«algum painel depende desta crate»*, que é o grafo inteiro.

⇒ a cegueira fica **declarada no cabeçalho do gate**, com o cúmplice nomeado: o gate de RUNTIME
**não tem filtro de população nenhum**. *Duas réguas com cegueiras COMPLEMENTARES valem mais que uma
com a população certa — desde que esteja escrito qual cobre o quê.*

---

## 8. E o leitor da tabela tinha duas cegueiras, uma delas com 23 % da tabela dentro

Havia **dois** leitores das tabelas de string: o `keys_declared` (que aprendeu a forma TUPLO
`("k", "v")` em 17/09) e a cópia dentro do gate do português (que ficou a conhecer só o `match`).

⇒ **1 432 entradas — 23 % da tabela — nunca tinham sido conferidas contra o português**
(`node_options` 601 · `node_params_motion` 430 · `node_params` 398).

> ⚠️⚠️ **E o piso de população não o podia dizer:** ele exigia `>= 5 000`, e a forma que a régua
> conhecia traz `4 815` sozinha. *Um piso satisfeito pela forma que a régua conhece não afirma nada
> sobre a forma que ela não conhece.* Hoje a leitura é de **6 706** e o piso está em `6 400` — acima
> do que a forma antiga produz sozinha, que é a única posição em que ele afirma alguma coisa.

A cura é uma **PORTA** (`ph2d_label_census::keys::declared_pairs_in`), com as duas formas, o
`\u{…}` descodificado e a continuação de linha corrigida — ela inseria um espaço que o Rust **não**
insere. *Inócuo para um censo de língua; decisivo para quem compara ao bit.* Corridas as 1 432
entradas recém-visíveis: **zero português**.

### ⛔ E uma mutação SOBREVIVEU porque o meu controlo não continha o fenómeno

A cerca do parêntese (*«um tuplo reconhece-se pelo `(` que o abre»*) foi testada com duas linhas de
`match` lado a lado — e ali ela é inerte: depois de emparelhar por `=>` o percurso segue sem guardar
o valor. Quem a discrimina é um **ARRAY** (`&["Paint", "Erase"]`), onde há uma vírgula entre dois
literais e o que abre é um `[`. *Uma mutação que o corpus não discrimina lê-se exactamente como uma
lei que não existe.*

---

## 9. ⛔⛔ E o gate fechou VERDE com `-p` e acusou 2 na árvore inteira

Metade dos painéis do registo está atrás de uma **feature opcional** (`panel-wet-tuning`, …), e um
`cargo test -p ph2d-panel-registry-init` **não as acende** — a unificação de features de um build de
WORKSPACE acende. ⇒ o gate novo passou com `-p` e acusou **dois** rótulos na varredura da árvore.

> ⚠️ **A assimetria já estava medida no cabeçalho do próprio ficheiro** desde 18/09 — a tabela dele
> tem duas colunas, *«`-p` sozinho (24 painéis)»* e *«árvore inteira (28)»* —, e o
> `PISO_DE_PAINEIS` está no número MENOR de propósito, para o gate passar das duas maneiras.
> *Um piso posto no menor dos dois deixa de afirmar o que acontece no maior* — e é lá que o app
> corre.

### E os dois acusados eram PEDAÇOS de um parágrafo

```
wet_tuning · "extensions (diffusion, backrun, fingering,"
wet_tuning · "the tuning registry's hidden group."
```

O painel pinta prosa com `paint_text_block`, que a **quebra em linhas**, e o censo das elisões mede
**cada linha**. A tabela declara a frase inteira.

⇒ regra: um texto que é **substring contígua** de um texto da tabela é uma linha dele.
⚠️ **O preço está declarado:** aceitar substring afrouxa a régua — um rótulo curto escrito à mão que
por acaso caia dentro de uma frase longa passa. Ele fica do lado BARATO contra a alternativa, que
seria uma lista de isenções sobre pedaços de frase, e esses mudam sempre que uma coluna muda de
largura.

---

## ⛔⛔⛔ Duas armadilhas de MÉTODO que esta fatia pagou

1. **`open(p, "w").write(open(p).read()…)` trunca antes de ler.** Python avalia o receptor primeiro.
   Perdi um módulo de 154 linhas acabado de escrever, e o sintoma foi um erro de compilação **noutra
   crate**.
2. **Prosa dentro de um heredoc sem aspas é CÓDIGO.** As crases de um comentário meu dentro de um
   `<<EOF` executaram o `spectacle` — o programa que esta casa proíbe porque fotografa o ecrã real
   do dono (449 s vivo, zero ficheiros gravados; verificado). ⚠️ E eu **repeti-a na mesma hora**, num
   `<<PY` de Python, onde as crases tentaram correr `wet_tuning`, `nextest` e `-p`. *A cura é
   estrutural — prosa fora do heredoc, e um guarda sobre o ficheiro gerado —, nunca «ter cuidado».*

---

## 10. O report do dono — *«prefab e Image ainda errados»* — e as TRÊS cegueiras que ele destapou

A foto dele mostra, na mesma fileira: `[Ŧýþé··]` e `[Ŕéçéñt···]` deformados (vieram da tabela) e
**`Prefab` · `Image` em inglês normal** (presos no código). O pintor era o `kind_chip_label`.

### ⛔ 1. A lista do gate era escrita à MÃO e eu não a fiz crescer

O `nenhum_pintor_chama_o_acessorio_ingles` tinha os **seis** tipos da fatia da manhã. À tarde migrei
o `AssetKind` e o `SortBy` **e não acrescentei a lista** ⇒ o gate que existe exactamente para isto
fechou verde sobre isto.

> ⇒ a lista passa a ser **DERIVADA** da árvore: todo par `(tipo, acessório)` cujo corpo é um
> `tr_em(…Ingles…)`. *Uma lista que decide o que um gate VÊ tem de crescer com a migração — e a
> única que cresce sozinha é a que se deriva.* (24 pares hoje, com piso de população.)
>
> ⚠️ A 1.ª redacção da derivação leu `impl crate::Verb` como o tipo **`crate`** — uma palavra que
> aparece em todo ficheiro Rust — e acusou meia shell. O tipo é o **último** segmento do caminho.

### ⛔⛔ 2. O gate de RUNTIME não o podia apanhar, e isso estava declarado

O acessório inglês é `tr_em(Ingles, chave)`: a palavra que chega ao pintor **veio da tabela**, logo
a pergunta *«a tabela sabe produzir isto?»* responde **SIM**. Era a limitação que aquele gate já
escrevia de si mesmo, e este é o caso dela. *Duas réguas com cegueiras complementares — e é preciso
que esteja escrito qual cobre o quê.*

### ⛔⛔⛔ 3. A minha isenção cegou o pintor, e a mutação SOBREVIVEU

A sonda `probe_index_summary` e o pintor `kind_chip_label` vivem **no mesmo ficheiro**, sobre o
**mesmo tipo**. Isentar o par `(ficheiro, tipo)` cegou os dois: a mutação que devolvia o
`AssetKind::label` ao pintor passou. ⇒ a granularidade é a **LINHA** — o ficheiro só é isento se
**toda** linha com o acessório casar com um trecho declarado.

### ⛔⛔⛔ E mesmo assim ela sobreviveu OUTRA vez: o acessório como VALOR

A mutação escreve `map_or(…, AssetKind::label)` — o acessório passado como **valor de função**, sem
parênteses. A régua procurava `.label()`, a forma de **chamada**.

> ⭐ *Um acessório passado como valor de função não tem a forma de uma chamada.* A régua passou a ver
> as **duas** (`.metodo()` e `Tipo::metodo`), e só então a mutação sangrou.

⚠️ E a mutação que estreita a régua de volta **não sangra**, por uma razão declarada: hoje não há um
único `Tipo::metodo` na árvore, logo o corpus não a discrimina. *A prova do alargamento é o
antes/depois da MESMA mutação* — ela sobreviveu duas vezes e sangra agora.
