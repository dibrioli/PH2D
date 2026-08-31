# Plano 38 — a ferramenta **TRIM**

> Pedido do Enio (2026-08-31): *"Tool: Trim. Como no Fusion, deleta segmentos entre pontos ou entre
> linhas sobrepostas."*

## §1 — A pesquisa, e o que a indústria **abandonou**

| onde | o gesto | o que decide a fronteira |
|---|---|---|
| **Fusion 360** (*Sketch > Trim*, `T`) | passa o cursor, o pedaço acende a **vermelho**, clica e some | *"trims to the nearest **crossing or node**"* — e **sem cruzamento nenhum, a entidade INTEIRA é apagada** |
| **AutoCAD ≥ 2021** (`TRIM`, *Quick mode*) | idem — clica no pedaço | **tudo** é aresta de corte, por omissão |
| **AutoCAD ≤ 2020** (`TRIM`, *Standard*) | escolhe as arestas de corte, `Enter`, **depois** os pedaços | as arestas ESCOLHIDAS |
| **Rhino** (`Trim`) | escolhe os objectos de corte, `Enter`, depois os pedaços | idem |
| **Illustrator** | **não tem** — a via é *Tesoura* + `Delete` (dois passos) ou o *Shape Builder* com `Alt` | — |

⭐⭐⭐ **A decisão de desenho já foi tomada pela indústria, e há data:** a Autodesk **trocou o modo por
omissão** do `TRIM` em **2021**, de *"escolha as arestas de corte primeiro"* para *"tudo corta, clique
no pedaço"* — o modo antigo sobrevive atrás de uma variável (`TRIMEXTENDMODE = 0`). ⇒ **Nascemos no
modo rápido.** ⛔ O modo de dois passos é uma **recusa medida por outra pessoa**: quem o quiser de
volta tem de explicar por que o dono do `TRIM` o tirou do caminho.

⚠️ **A queixa nº 1 do Fusion é o corolário da regra dele:** num círculo sem cruzamentos, o Trim apaga
**o círculo inteiro** — *"I seem to only be able to delete entire circle sketches"*. A causa é aquele
*"or node"*: um círculo do Fusion **não tem nós**, então não há fronteira nenhuma e o pedaço é a
peça toda.

## §2 — A lei

> **O pedaço sob o cursor é a extensão de caminho entre as duas FRONTEIRAS mais próximas, uma de
> cada lado. Clicar apaga-o.**

**Fronteira** é qualquer uma destas, numa lista só:

1. um **cruzamento** com outro caminho visível;
2. um **auto-cruzamento** do próprio caminho;
3. um **nó** (âncora) do próprio caminho;
4. uma **ponta aberta** do caminho.

⭐ **É exactamente o *"entre pontos ou entre linhas sobrepostas"* do pedido**, e é a regra do Fusion
com o *"or node"* dentro.

⚠️⚠️ **E ela cura a queixa nº 1 do Fusion de graça.** Aqui um círculo autorado **tem nós**
(o `ShapeKind::Ellipse` cozinha âncoras), então clicar entre dois deles tira **um quarto**, e não a
peça toda. *A regra é a mesma; a diferença está no substrato, e é a nosso favor.*

### O que sobra depois do corte

| o caminho | o pedaço | o resultado |
|---|---|---|
| aberto | no MEIO | **dois** caminhos |
| aberto | numa PONTA | **um**, mais curto |
| fechado | qualquer | **um**, agora aberto |
| qualquer | **a peça toda** (sem fronteira nenhuma) | apagado |

⚠️ Estilo e pose viajam para os sobreviventes: um traço aparado continua com a cor, a largura, o
tracejado e o `Transform` que tinha.

## §3 — As portas únicas

- **Quem é o pedaço** — uma função pura `(caminho, fronteiras, t do cursor) -> (t_inicio, t_fim)`.
  O **realce** e o **corte** leem-na, senão o que acende e o que some divergem no primeiro ajuste.
- **Quais são as fronteiras** — uma função só, que junta as quatro espécies. ⛔ Não há uma lista para
  o hover e outra para o clique.
- **Partir o caminho** — a porta que já existe (§5), nunca uma segunda.

## §4 — Os gates (red-first)

1. as quatro espécies de fronteira aparecem na lista (uma fixtura por espécie);
2. o pedaço entre dois nós de um polígono é **um lado**, nem mais nem menos;
3. um cruzamento **mais perto** que o nó vence — é o caso do «entre linhas sobrepostas»;
4. aberto-no-meio dá **dois** caminhos, fechado dá **um aberto**, sem fronteira dá **zero**;
5. o estilo e a pose sobrevivem;
6. **o realce e o corte leem a MESMA porta** (a mutação que os separa tem de matar);
7. costura: o gesto REAL no canvas chega ao corte (⛔ `Click` sintético não mede a rota do ponteiro).

## §5 — O que reusa (e o que a extracção custou)

⭐⭐⭐ **O motor já existia dentro do `fx_knot`.** Ele corta vãos por fracção de arco e emite as
fitas que sobram — exactamente o que o Trim faz, com os vãos DERIVADOS (uma travessia por baixo) em
vez de AUTORADOS (um clique). O `fx_trim::Piece` já carregava a frase que decidiu isto: *"duas
cópias do corte por arco divergiriam no 1.º ajuste"*.

⇒ [`arc_cut.rs`](../../crates/ph2d-vec-scene/src/arc_cut.rs) — a porta única, extraída:
`Geom`/`Edge` (a poligonal de detecção com a fracção de cada ponta) · `seg_cross` · `crossings` ·
`keep_ranges_closed`/`_open` (o complemento) · **`strands_of`** (vãos explícitos) com
`strands_uniform` como adaptador do Knot. O `fx_knot.rs` foi de **400 para 156** linhas e os gates
dele saem verdes sobre o mesmo desenho.

⛔⛔ **E a extracção revelou um FALSO NEGATIVO que o Knot tolerava e o Trim não pode.** O
`seg_cross` exigia que a travessia estivesse **estritamente dentro** da aresta, e uma travessia que
cai **exactamente sobre uma amostra** da poligonal era recusada. Medido: duas retas em cruz em
`x = 4`, com a vertical de `−5` a `5`, põem-na no 8.º de 16 pontos ⇒ o cruzamento não existia;
deslocar a ponta `0,1` fazia-o aparecer. ⚠️ *É o caso mais comum que há* — um artista desenha em
coordenadas redondas. A janela passou a ser inclusiva, e **a defesa que a estreita dava já existia
uma função abaixo**: o `crossings` funde travessias a menos de `MERGE_FRAC` uma da outra, que é
precisamente o duplicado que uma ponta partilhada produz. *A cerca estava em dois sítios, e a de
cima recusava o que a de baixo sabia fundir.*

## §6 — A cena de smoke

**`PH2D_BUILD_SMOKE=80`** ([`trim_smoke.rs`](../../shells/desktop/src/trim_smoke.rs)) — quatro casos
lado a lado, um por espécie de fronteira, com a ferramenta **já armada** (sem isso o artista abre o
smoke, não vê realce nenhum e conclui que a feature não existe):

1. a **CRUZ** (o cruzamento) — ⚠️ em coordenadas REDONDAS de propósito: foi esta a fixtura que
   apanhou o falso negativo do §5;
2. o **RECTÂNGULO** (os nós) — aparar tira **um lado**, e é aqui que se vê a diferença para o
   Fusion, que apagaria a peça toda;
3. o **ZIGUE-ZAGUE** aberto — aparar o meio parte-o em **dois**;
4. a **RETA SOLTA** — a peça toda ⇒ ela **some**.

## §7 — ⏳ O que fica ABERTO e nomeado

- ⏳ **O custo do realce não foi medido.** O `hit` cozinha e transforma **todos** os caminhos da cena
  para o espaço local do alvo, uma vez por quadro enquanto a ferramenta está na mão. ⛔ **Sem
  relógio** — a acusação é a complexidade com endereço, e a cura óbvia (filtro por caixa + memo por
  revisão da cena) só se escreve depois de haver número.
- ⏳ **Os pedaços ficam no MESMO caminho, como contornos dele.** É o que preserva estilo, pose, id e
  o passo de undo de graça; o Fusion faz o contrário (entidades separadas). ⚠️ É **decisão de
  produto** — a alternativa obriga a duplicar o estilo e a escolher qual dos dois herda o id.
- ⏳ **Sem `Extend`**, o par canónico do Trim nas três referências (estender uma curva até à próxima
  fronteira). É a mesma maquinaria de fronteiras com o sinal trocado.
