# HANDOFF — `line/sculpt3d`, o REMESH (2026-08-10)

> **A linha NÃO está fechada.** Dois commits de pé, uma wave aberta com o
> mecanismo meio-entendido, e o próximo passo nomeado. Ele **supersede** o
> [`HANDOFF_CONTINUACAO_..._2026-08-10`](HANDOFF_CONTINUACAO_line_sculpt3d_2026-08-10.md)
> como *"comece aqui"* — aquele descreve a linha limpa, antes desta jornada.
>
> ⚠️ **Isto não é o estado do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../../CLAUDE.md)**.

---

## 1. O que foi pedido, e o que a medição fez com o pedido

O Enio escolheu a wave **"O REMESH GANHA MÃOS"**: a resolução vira slider (hoje o
botão usa `DEFAULT_RESOLUTION = 150` cravado), o *achatar* vira verbo explícito,
e o campo passa a carregar a máscara.

O `CLAUDE.md` §0 manda medir antes de escrever a faixa de um slider. A sonda
`measure_remesh` existia e nunca tinha sido rodada para isto. Ela achou outra
coisa, e a wave mudou de assunto.

---

## 2. ⬛ O remesh DESTRUÍA a escultura, e reportava sucesso — FECHADO (`21f091c0f`)

Acima de ~300 o `remesh` devolvia **`Ok` com zero vértices**, e o
`sculpt3d_history.rs` instalava isso no lugar da peça:

```rust
let (out, report) = ph2d_sdf::remesh(self.mesh(), resolution).ok()?;
let previous = core::mem::replace(self.mesh_mut()?, out);   // `out` VAZIO
```

O `Ctrl+Z` recuperava (o `StrokeUndo::Remeshed` guarda a malha anterior), então
não era perda permanente — mas a peça sumia da tela com log de sucesso.

⚠️ **E não é um teto: é ERRÁTICO, e alcança o default que shipa.** Varrendo a
esfera `uv(96,144)`, **onze das cem resoluções entre 100 e 200 vazam** — `112,
151, 160, 161, 168, 180, 181, 193, 194, 196, 197` —, e o default é **150**,
vizinho de uma delas. Quem decide não é a resolução e sim o alinhamento da grade
contra os triângulos, então **outra malha vaza noutros números**: o 150 não é
seguro, é sortudo. A resolução **316 funciona** no meio de uma faixa que falha.

**A cura:** `flood_fill` devolve quantas células ficaram DENTRO — *quem cria o
interior é quem sabe dizer se há um* —, e `remesh` recusa com
`RemeshError::NoInterior` **antes** da extração.

⚠️ **E a recusa do shell parou de mentir.** Três causas (pilha de multires ·
cena vazia · o motor) entravam num `Option` só, e o chamador elegia UMA mensagem
para as três — elegeu a da pilha, então um campo vazado mandava o artista
*"reverter os níveis"* que ele não tem. `RemeshRefusal` nomeia as três, nos
**dois** chamadores (a tecla `V` e o botão do painel, que carregavam cópias
divergentes da mesma mensagem).

---

## 3. ⬛ Uma causa do vazamento, fechada — o PROXY (`27eea417a`)

O voxelizador só disparava o raio de travessia quando o ponto mais **PRÓXIMO**
do triângulo caía à frente dentro de um passo:

```rust
let along = closest[ax] - p[ax];
if along < 0.0 || along > self.step { continue; }
```

É um pré-filtro que usa o ponto mais próximo como proxy do ponto de
**INTERSEÇÃO**, que é outro ponto. Num triângulo **oblíquo** o mais próximo cai
de lado, o proxy recusa, e uma aresta que fura a superfície fica sem marca.

⚠️ **O proxy é exato SÓ em geometria alinhada aos eixos, e é essa assimetria que
o denuncia:** um **cubo nunca vazou em 361 resoluções** enquanto uma esfera
vazava em 34% delas.

**Medido, 361 resoluções × 4 malhas:**

| malha | antes | depois |
|---|---|---|
| esfera uv(96,144) | 125 | **82** |
| esfera uv(24,32) | 177 | **80** |
| cubo | 0 | 0 |
| tubo aberto | 2 | 2 |

Tirar o proxy só pode **acrescentar** marcas, nunca inventá-las: a marca
continua saindo de um `ray_hit` real com alcance de um passo.

---

## 4. ⚠️ O vazamento NÃO está fechado — e o que já foi ELIMINADO

Sobram **82 e 80** resoluções vazando: há pelo menos um segundo mecanismo. A
recusa da §2 é o que impede que ele destrua trabalho enquanto isso.

**Quatro hipóteses derrubadas por medição — não as reconstrua:**

1. ⛔ **A caixa do triângulo termina cedo demais** (deixando células em
   `INFINITY`, que são não-guardadas). Alarguei a caixa em uma célula: varredura
   **idêntica** — 125/177/0/2 — com 20% mais tempo, o que prova que a mudança
   estava ativa. Revertida, não commitada. Confirmada uma segunda vez: **zero**
   células `INFINITY` a menos de um passo da superfície, em 600 amostradas.
2. ⛔ **O `dist` guardado mente para maior** (célula coberta pela caixa de um
   triângulo longe, não pela do perto). **Zero** ocorrências em 225 células da
   banda.
3. ⛔ **O `ray_hit` perde travessias.** Comparado contra um **Möller-Trumbore
   independente**, escrito do zero: **900 arestas, zero divergências**.
4. ⛔ **A malha de teste tem buraco.** `uv_sphere(96,144)` fecha: 0 arestas de
   beira, 0 buracos tapados, 27.360 triângulos, bounds `[-1,1]³`.

---

## 5. ⚠️ A lição mais cara: TODO oráculo degenera na superfície

Quatro instrumentos, quatro modos de errar **no mesmo lugar** — e cada um me
custou uma rodada:

| oráculo | como ele degenera |
|---|---|
| **a esfera ideal (`r = 1`)** | a malha é um poliedro **inscrito** e mergulha abaixo disso no meio de cada face, então passos inteiramente do lado de fora eram acusados de travessia |
| **força bruta com o `ray_hit`** | o mesmo algoritmo que **constrói** a parede, dos dois lados da comparação — razão entre dois doentes, cega a uma falha sistemática dele |
| **paridade em `+x` da origem** | `(1,0,0)` é exatamente um **VÉRTICE** desta esfera: o raio acerta várias faces de uma vez e a paridade sai par, dizendo *"fora"* sobre o **centro** |
| **paridade oblíqua** | sã longe da casca, **indefinida** numa célula com `dist = 0`, que está em cima dela — é onde a busca parou |

⚠️ **E as auditorias por AMOSTRA não decidem nada aqui.** As três da §4 cobriram
~0,1% da banda, e **um único furo basta para vazar**: ausência de evidência em
0,1% não é evidência de parede íntegra. Elas servem para *derrubar* uma
hipótese (um contraexemplo basta), nunca para *confirmar* a integridade.

**O que a instrumentação estabeleceu como fato:** o flood alcança **1902 de
1902** células verdadeiramente internas (paridade oblíqua) — o vazamento é
total, não marginal. E a cadeia de pais da onda até uma dessas células **nunca
fura a malha** pelo teste de segmento.

**O último candidato, plausível e NÃO provado:** um **corredor de células sobre
a superfície** (`dist ≈ 0`), cujas arestas internas não furam nada — a onda
entra nele vindo de fora sem cruzar, anda por ele, e sai para dentro sem cruzar.
É o problema clássico de **separabilidade digital**: um conjunto de arestas
marcadas só é barreira 6-conexa se a superfície for 6-separante. Provar isto
exige um oráculo de dentro/fora **não-degenerado na casca** — *generalized
winding number* é o candidato — e esse oráculo é o próximo passo, antes de
qualquer conserto.

---

## 6. O que NÃO foi medido, e por quê

O **custo** de ter tirado o pré-filtro. A máquina passou a jornada com `load
15-25` (dois `rustc` de outras linhas a 282% e 158%, mais o app aberto), e a
régua deste repo é que **nada acima de ~5 fala sobre o código**. As contagens de
vazamento acima são determinísticas e não dependem disso.

---

## 7. O estado da linha

| | |
|---|---|
| Branch | `line/sculpt3d`, 2 commits sobre `main 76788440a` |
| Árvore | **limpa** |
| Suítes | `ph2d-sdf` + `ph2d-mesh` + `ph2d-sculpt3d` **448 verdes** · shell verde · clippy limpo · LOC verde |
| Schema | `PROJECT_SCHEMA` **intocado** · registro do `ph2d-ecs` intocado · contrato congelado intocado |
| `Cargo.toml` | **zero** — nenhuma crate nova, nenhuma dep nova |

**Superfície pública nova:** `ph2d_sdf::RemeshError` · `flood_fill` devolve
`usize` (os dois outros chamadores o ignoram, e o doc diz que podem) ·
`remesh`/`remesh_default` devolvem `RemeshError` no lugar de `MeshError`.

**Sonda nova:** `crates/ph2d-sdf/tests/probe_leak.rs` — conta quantas resoluções
vazam, por malha. É o instrumento desta caçada e o oráculo dele é o único que
**não** degenera (contagem de interior igual a zero é inequívoca). ⚠️ Roda em
~400 s: `cargo test -p ph2d-sdf --release --test probe_leak -- --ignored --nocapture --test-threads=1`.

⚠️ **As sondas de diagnóstico foram RETIRADAS da árvore, de propósito** — 855
linhas com oráculos que esta jornada provou degenerados seriam armadilha para o
próximo. O que elas ensinaram está na §5. Ficaram parqueadas fora do repo.

---

## 8. O que fazer a seguir, em ordem

1. **O oráculo não-degenerado** (*generalized winding number*), sem o qual
   nenhuma hipótese sobre o furo é verificável. É o gargalo.
2. **O segundo mecanismo**, com ele em mãos.
3. **O SLIDER** — a wave original. ⚠️ Ele segue **bloqueado**: hoje o artista
   alcança um valor e ele calhou de ser bom; o slider lhe dá acesso às 82.
4. O verbo de **achatar** e a **máscara** no campo.

⚠️ **E o `measure_remesh` agora imprime a RECUSA em vez de morrer** — uma
resolução que vaza é exatamente o que a tabela precisa mostrar, e um `expect`
ali matava a varredura no primeiro vazamento.

---

## 9. Duas armadilhas de processo desta jornada

⚠️ **A cwd do Bash escorregou para a árvore PRIMÁRIA duas vezes** — uma me fez
rodar um gate contra o `main` (o *"0 tests"* não era resultado, era a árvore
errada) e outra escreveu uma sonda lá. **`main` foi conferida e está limpa nas
duas.** A regra é a do `MODELO_TROCA_DE_AGENTE_NA_LINHA`: **todo comando começa
com o `cd` da worktree**, sem exceção.

⚠️ **Um gate meu nasceu incapaz de falhar.** Ele afirmava que cada causa de
recusa *aparecia* no bloco de despacho; a mutação que colapsa duas num braço só
(`MultiresStack | EmptyScene`) deixa o nome no texto e **passou**. Agora ele
**conta braços**. O gate anterior, do `main`, tinha o mesmo vício por outra via:
ancorava no literal `return None` — uma **grafia**, não a propriedade.
