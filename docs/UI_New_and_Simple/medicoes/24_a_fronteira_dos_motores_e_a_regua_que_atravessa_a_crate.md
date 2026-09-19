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
