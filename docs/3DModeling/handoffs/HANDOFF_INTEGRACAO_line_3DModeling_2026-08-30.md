# Handoff de integração — `line/3DModeling`, 2026-08-30

> ⚠️ **Onde o mecanismo mora.** O [doc 06](../06_resultados_cena_e_gizmo.md) está em **612 KB** —
> muito acima do joelho de 80–110 KB que o `CLAUDE.md` §5.0 nomeia, e onde o `Read` deixa de o
> alcançar. As waves de hoje ficam **aqui**, que é a casa que a lei já lhes dava. ⏳ **O corte do
> doc 06** (`python3 scripts/doc-split.py`, história verbatim para `docs/archive/`) fica **aberto** e
> nomeado: ele obriga a reancorar as citações `§65 · §69 · §70 · §73 · §82 · §83 …` que o `CLAUDE.md`
> §5 faz, e por isso é obra própria, não um item de fim de wave.

## §1 — W105: a peça DESAPARECIA ao ganhar formas (report do Enio)

*«quanto mais objetos colocamos na tela, mais artefatos e mais largos os vãos»*, com foto de tubos
filetados e riscos de fundo a atravessar as juntas.

⭐⭐⭐ **DOIS defeitos independentes, e só a soma dos dois explica o report.** Nenhum dos dois é
visível pelo outro: o primeiro sozinho deixa a peça inteira **preta**, o segundo sozinho deixa-a
**lenta**.

### §1.1 — A lei do passo era EXPONENCIAL no número de peças

Ela era `passo = 2^(−profundidade/2)` — `√2` por nível, com a profundidade a contar `n − 1` para um
grupo de `n` formas filetadas (o `combine_trees` dobra aos pares, logo um nó só já é uma corrente).

⭐ **A lei certa soma os QUADRADOS, e demonstra-se.** Na região de mistura da união exacta
(`ops::union_round`), com `u = max(r−a,0)` e `v = max(r−b,0)`:

```
∇f = (u·∇a + v·∇b) / ‖(u, v)‖          (o termo `max(min(a,b), r)` é constante ali)
‖∇f‖ ≤ (u·L_a + v·L_b)/‖(u,v)‖ ≤ √(L_a² + L_b²)          [Cauchy–Schwarz]
```

⇒ uma corrente de `n` folhas dá **`√n`**, não `√2^(n−1)`: a `n = 12` isso é `3,46` contra `45,3`.
O chanfro cabe na mesma lei, e também se demonstra: o termo dele é `(a+b−r)/√2`, com tecto
`(L_a+L_b)/√2`, e `(L+1)²/2 ≤ L²+1 ⟺ (L−1)² ≥ 0`.

**Medido nas quatro formas de árvore** (`measure_the_chain_of_fillets`, grelha 44³) — o tecto é
**tight**, e a medição fica abaixo dele em todas as linhas:

| árvore | `‖∇f‖` medido | tecto `√(ΣL²)` | folga |
|---|---:|---:|---:|
| plana `n = 2` | `1,4133` | `1,4142` | `0,1 %` |
| plana `n = 4` | `1,9913` | `2,0000` | `0,4 %` |
| plana `n = 8` | `2,7675` | `2,8284` | `2,2 %` |
| **equilibrada** (4 folhas, profundidade 2) | `1,9852` | `2,0000` | `0,7 %` |
| irmãs por junta **viva** | `1,4027` | `1,4142` | `0,8 %` |

⛔⛔ **É a linha da EQUILIBRADA que refuta toda lei escrita sobre PROFUNDIDADE** — a antiga e também
uma `√(1+profundidade)`: ela tem profundidade `2` e mede `1,985`, acima do `√3 = 1,732` que essa
concederia. *O que conta é quantas folhas chegam ao ponto por misturas, não quantos níveis a árvore
tem.*

⇒ o `inflation_depth` **morreu** e deu lugar ao `gradient_bound`, que é a grandeza de que o passo é
o recíproco (`crates/ph2d-field-eval/src/step.rs`).

### §1.2 — E o ORÇAMENTO DE PASSOS era fixo

A marcha aproxima-se geometricamente (`d ← d·(1−s)`), logo os passos necessários são `∝ 1/s`. Com o
tecto FIXO (`MAX_STEPS = 400`), o raio acabava o orçamento antes da superfície e era **largado em
silêncio** — pixel de fundo. Hoje o orçamento sai do passo, e ⚠️ **um documento sem inflação fica nos
`400` de sempre, ao bit** (`s = 1` ⇒ a divisão é a identidade).

| tecto | esgotados | furos | quadro |
|---:|---:|---:|---:|
| `400` (fixo) | `86`–`136` | `1` | `5,14 ms` |
| `1 200` | **`0`** | **`0`** | `5,06 ms` |

⭐ Sai de graça porque só ~`130` raios em `102 400` passam sequer dos `400`.

Instrumento novo: **`EXHAUSTED`** conta os raios largados por falta de orçamento. *Um caminho de
desistência sem contador é um defeito que só o artista vê.*

### §1.3 — O que o report media, em pixels

`measure_holes_versus_object_count`, `320²`, pixels acertados:

| formas | passo de ontem | acertos | passo de hoje | acertos |
|---:|---:|---:|---:|---:|
| 11 | `0,0312` | `13 992` | `0,3015` | `34 592` |
| 12 | `0,0221` | **`688`** | `0,2887` | `34 737` |
| 13 | `0,0156` | **`0`** | `0,2774` | `35 585` |

### §1.4 — E o CUSTO, que é o que torna a lei obrigatória

Amostras de campo por raio (contagem determinista, ⛔ não um relógio):

| formas | lei da SOMA | lei EXPONENCIAL |
|---:|---:|---:|
| 4 | `14,0` | `20,1` |
| 8 | `16,5` | `66,3` |
| 12 | `21,8` | `287,4` |
| 16 | **`31,9`** | **`1 464,9`** (`221 ms` de quadro) |

### §1.5 — ⛔⛔ A prova de mutação que EXPÔS o gate, e não o código

A mutação que repõe a lei exponencial **SOBREVIVEU** ao gate da imagem: com o orçamento derivado do
passo, um passo curto de mais deixa de furar a peça — passa só a pagar por ela. *Duas curas
independentes fazem um gate só medir a combinação delas.*

⇒ o gate da LEI é outro, e é uma **contagem**: `the_price_of_a_shape_is_not_exponential` afirma que
16 formas custam `≤ 4×` as amostras/raio de 4 formas (medido `2,28×`; a lei antiga dá `72,9×`).
A barra fica no meio do vazio entre as duas, e não colada a nenhuma.

**Gates:** `the_piece_does_not_vanish_as_shapes_are_added` (furos interiores `0` · esgotados `0` ·
`acertos(filetada) ≥ acertos(viva)`, com a fixtura a ser um bloco maciço de propósito e a união viva
a marchar a `1,0` como controle) · `the_price_of_a_shape_is_not_exponential` ·
`the_bound_sums_squares_and_a_live_joint_takes_the_max` e
`the_bound_counts_the_folding_steps_that_inflate` (migrados de profundidade para tecto, com a árvore
EQUILIBRADA acrescentada).

Commit `9762c9c76`.

## §2 — W106-bis: o ARCO PRETO na cruz — a caixa do mundo era 4,1 % menor que a peça

O `bounding_radius` da cruz ignorava a **largura** do braço (`hyp(arm, half_height)` em vez de
`hyp(hyp(arm, width), half_height)`): o ponto mais distante de uma cruz é o **canto** do braço, não o
meio da ponta. O recorte da marcha é uma **esfera**, e por isso o corte sai em arco.

⚠️ **DUAS versões do gate ficaram verdes por cima disto:** a de direcções (grelha 24×24) falhava a
quina por `5°`, e a de pontos **sobreviveu à mutação** (saliência de `0,003` contra passo `0,028`).
A terceira **bissecta a superfície** ao longo de `96 × 192` direcções e mede o ALCANCE — essa mata.

Duas hipóteses caíram com número antes: a marcha atravessar (`0` furos em 374 raios) e a normal
degenerar (`0` casos, folga de `0,60` a `0,002`).

Commit `f5961333f`.

## §3 — A Hierarquia ganha `Delete` e `Ctrl/Cmd+D` (report do Enio)

*«temos um bug: delete não funciona na hierarquia. Avalie também duplicate»*

⛔ **As teclas nunca existiram.** O `KEY_DELETE` do dispatcher vira `GraphKey::Delete`, e o único
consumidor dele em toda a árvore é o painel do grafo de motion. Apagar ou duplicar um objeto da cena
só era possível pelo item do menu de contexto de uma linha.

⚠️ **E TRÊS doc-comments do shell** (`keyboard.rs` ×2, `keyboard_painter.rs`) justificavam o próprio
`return` a invocar *«o caminho genérico de Delete, que apaga a ENTIDADE selecionada»* — uma rota que
não existe. Estão corrigidos. *Um comentário que descreve um caminho ausente faz cada leitor seguinte
assumir que a metade que falta já está feita.*

⭐⭐ **A cura é um segundo PRODUTOR do verbo, nunca uma segunda lei:** as teclas resolvem
`selecção → linha` pela ponte e empurram `EditorAction::HierDelete` / `HierDuplicate`, exactamente o
que o item do menu empurra. A multi-selecção, a limpeza do gizmo, a promoção do novo primário, a
cópia profunda, o passo da cascata e o undo ficam onde já estavam.

⚠️ **A área é o ponteiro sobre o painel** (a regra que o `cursor_over_timeline` já usa, e a do
Blender). Global, ela roubaria o `Delete` de **cinco** donos: o traço do Flip, o nó de curva, a figura
em mãos do Painter, a key da timeline e o nó do grafo.

⚠️ **A LEI MUDOU-SE PARA UMA FUNÇÃO PURA (`verb_for`) por MEDIÇÃO:** escrita dentro do `impl App` ela
só se pode gatear por texto, e a mutação `if false && self.hierarchy_key_chain(...)` **SOBREVIVEU** ao
gate da costura — cadeia morta, gate verde. Com a lei pura, a mutação que apaga a guarda da área
MORRE.

⚠️ **O controle negativo do gate proíbe ESCRITA e não leitura** — a 1.ª redacção proibia `gizmo.` e
reprovou sobre código certo: a cadeia **lê** a selecção para saber a quem o verbo se aplica, e isso é
a pergunta, não uma lei própria.

Commit `50b0c6a50`.

## §4 — Estado

Suíte do binário do shell: **4 083 verdes**. `cargo check --workspace --all-targets` limpo.
Clippy limpo nas crates tocadas.

## §5 — W107: a TORÇÃO, o primeiro deformador de espaço desde a inclinação

O vocabulário de modificadores tinha **oito** entradas e nenhum deformador além do `Taper` — sem
recusa medida em doc nenhum. Um deformador multiplica-se pelas 28 formas e por toda booleana de graça.

### §5.1 — A lei, e ela fecha em álgebra

O ponto vai para o espaço não torcido rodando `(x,y)` por `−k·z`; cada fatia de `z` sofre uma
**rotação** (isometria), logo não há escala a desfazer. O jacobiano do mapa inverso tem as duas
primeiras colunas ortonormais e a terceira igual a `(k·q_y, −k·q_x, 1)`; com `t = k·r`:

```text
σ_max(J) = t/2 + √(1 + t²/4)
```

⛔ **Não é `√(1 + t²)`.** Os dois termos podem **alinhar-se**, e por isso somam-se linearmente e não
em quadratura — `1,618` contra `1,414` em `t = 1`, e até `13,4 %` de diferença em `t ≈ 0,7`. *Treze
por cento acima da distância verdadeira não fica lento: fura.*

### §5.2 — ⛔⛔ A medição refutou a FORMA do divisor, não apenas a constante

Dividir por `σ(k·r)` **no ponto** parece mais apertado e é pior: o divisor varia com o ponto e a
derivada dele reentra em `∇(f/d) = ∇f/d − f·∇d/d²`. Medido a uma volta por unidade, com a margem a
subir: `1,78 · 2,11 · 2,32 · 2,51 · 2,55` — **subir a margem PIORA**.

O divisor **constante** `σ(k·R)` não tem gradiente próprio, e fecha **sem constante ajustada**:

| voltas/un | `σ(k·R)` | `‖∇f‖` |
|---:|---:|---:|
| 0,05 | `1,1421` | `0,9617` |
| 0,30 | `2,0802` | `0,8167` |
| 1,00 | `5,5129` | `0,7068` |
| 2,00 | `10,7559` | `0,7039` |

⭐ **É a diferença com o `taper`, e ela é do OPERADOR e não do cuidado:** ali a escala varia com `y`
**dentro** da conta e o `2` teve de sair da tabela; aqui a álgebra fecha e a medição só confirma.

### §5.3 — O que a feature contém

`Unary::Twist { turns, lower, upper }` — voltas por **unidade** (a forma do `Taper`; um número «sobre
a peça» mudaria de sentido ao esticá-la), em torno do **Z** (o eixo do `Radial`, pela razão dele).
⭐ **Os LIMITES não são enfeite:** sem eles um deformador só sabe agir na peça inteira, e não há
«torcer só o topo». A banda é um `clamp` do `z` que entra no **ângulo** — fora dela a peça roda como
corpo rígido; um corte no campo parti-la-ia em três sólidos. Nasce **torcida** (`0,25` voltas/un) com
a banda a cobrir a peça: um chip que não muda um pixel lê-se como morto. `FIELD_DOC_VERSION` **11**.

### §5.4 — ⛔⛔ E o divisor mudava a UNIDADE do campo — defeito PRÉ-EXISTENTE

| pilha | parede pedida | ANTES | DEPOIS |
|---|---:|---:|---:|
| `Shell` sozinho | `0,060` | `0,060` | `0,060` |
| `Taper 1,00` + `Shell` | `0,060` | `0,180` (`3,00×` = `1+2·declive`) | **`0,060`** |
| `Twist 1,00` + `Shell` | `0,060` | `0,337` (`5,62×` = `σ(k·R)`) | **`0,060`** |

Os dois factores batem a fórmula **exactamente**, o que fecha o diagnóstico: não é erro numérico, é a
unidade. ⚠️ A **inclinação carrega isto desde a W18** — o `Offset` e o raio de um filete depois dela
erram pelo mesmo factor. A cura é o divisor **acumular e aplicar-se uma vez, no fim da pilha**.

### §5.5 — Três buracos de infraestrutura que o censo achou

| buraco | o que acontecia | cura |
|---|---|---|
| `UnaryKind::ALL` era um array de tamanho fixo | acrescentar variante e esquecer a lista **compila limpo** e o modificador nasce inalcançável | `UnaryKind::index()` exaustivo + `every_modifier_kind_is_in_the_list` |
| `MAX_MODES = 8` com a `ALL` em **8** | o chip seguinte nasceria não-pintado e **sem id no store**: `apply_click` devolve `None`, o evento nunca nasce | sobe para `16`, com gate dos **dois** lados (incluindo a metade que exige **folga**) |
| `field3d_reach_tests` com `slots: 4` | desactualizado havia **quatro** modificadores — `MirrorZ`, `Array`, `Radial` e `Taper` nunca foram alcançados | derivado de `UnaryKind::ALL.len()` |

⚠️ O `MAX_MODES` nunca foi um teto **visual**: a fileira **wrappa** (`segmented_row_counts`).

### §5.6 — Aberto, nomeado

⏳ **BEND** — a matemática está derivada e o substrato pago (o bordo já anda ao lado da árvore):
`σ = max(1, 1/(1−κ·x))`, com uma **parede demonstrável** (`ângulo × meia-largura < comprimento`, e
`< 2π` pela representação) e uma bola de bordo que **não** se preserva (a conta ingénua explode a
grade de exportação a ângulos pequenos: use o sector, `√(R² + 4ρ(ρ+R)sin²(α/2))`).
⏳ **Eixo escolhível** e **origem** do deformador — as quatro referências têm-nos; aqui o eixo é o Z
pela lei da casa, e a discussão fica escrita.
⏸️ **Bias** (distribuição não-linear ao longo do eixo) — só o 3ds Max o tem, e o Houdini não; o `σ`
passaria a depender do perfil. Wave própria, com o preço medido.
⏳ **O orçamento de passos não vê um encolhimento LOCAL**: com o divisor no operador, `scene.step`
fica em `1,0` e a região torcida pede `~σ×` mais passos. O instrumento existe (`march::EXHAUSTED`) e
**ainda não foi corrido sobre um operador que encolce localmente** — é a medição que decide se algum
teto se mexe.
