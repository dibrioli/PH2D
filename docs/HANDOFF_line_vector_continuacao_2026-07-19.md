# HANDOFF de CONTINUAÇÃO — `line/Vector` (para o próximo implementador)

**Para:** a LLM que assume a linha `line/Vector` daqui pra frente.
**De:** a sessão de 2026-07-18/19, que fechou os **Live Path Effects** ([ADR-0132](architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)).
**Estado:** ✅ **a linha INTEGROU e está RE-PREPARADA** — o worktree foi fast-forwardado para a
`main` (0 ahead / 0 behind, árvore limpa). Você começa numa árvore limpa; há uma **fila** (§4) e um
punhado de **armadilhas já pagas** (§3) para não re-aprender.

> **Leia primeiro, nesta ordem:** `CLAUDE.md` (o roteador — os 8 inegociáveis + §5 do Vector) ·
> `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` (a CADA passo). Depois, só o que a sua
> tarefa exigir:
> [ADR-0132](architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)
> (a pilha de efeitos — o que esta sessão entregou) ·
> [ADR-0121](architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md)
> (fonte≠cozido — o pré-requisito de metade da fila) ·
> [ADR-0128](architecture/decisions/0128-vector-blend-object-live-non-destructive.md) (Blend vivo) ·
> [ADR-0129](architecture/decisions/0129-vector-envelope-warp-one-spine-cage-as-container-entity.md)
> (Envelope). Este handoff é o **mapa da linha + a fila**; ele NÃO duplica os ADRs.

---

## §0 — ⬛ A ÚNICA COISA QUE MUDA COMO VOCÊ TRABALHA

**Um efeito novo entra na FOLHA DE CONTACTO antes de entrar na tabela `KINDS`.**

Esta sessão construiu três efeitos com **238 gates verdes e mutações a sangrar**, e **três deles
saíram maus na mesma leva** — o Enio: *"três implementações paupérrimas"*. A causa foi uma só:

> Todos os oráculos perguntavam *"o buffer diz o que eu disse que dizia"*. Nenhum perguntava
> *"isto parece a ferramenta cujo nome tem"*.

O aparelho que faltava agora existe — **`crates/ph2d-vec-scene/tests/fx_look.rs`** + `tests/look/`:

```
PH2D_FX_LOOK_DIR=/tmp/look cargo test -p ph2d-vec-scene --test fx_look --release -- --ignored --nocapture
```

Uma linha por capacidade, uma coluna por valor do parâmetro, num PNG. Preenchimento = a forma ·
linha fina = a geometria · **cruzes = as âncoras**. Sem dependência nenhuma (a crate é
`serde`+`postcard`; o PNG usa blocos deflate não comprimidos). É o irmão do `push_look_probe` do
Painter. **Acrescente uma linha nela para o seu efeito e OLHE, antes de dizer que está feito.**

---

## §1 — Onde a linha está (30 s)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **HEAD** | `3746216b` = **exatamente a `main`** (0 ahead / 0 behind, árvore limpa) |
| **Modo** | **Modo L** (workstation, worktree próprio, [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)) |
| **Schemas na main** | `VEC_SCENE_SCHEMA_VERSION` = **13** · `PROJECT_SCHEMA` = **23** · tripla do gate `(23, 8, 13)` |

**Como a linha foi re-preparada** (não refaça): a sessão anterior fechou os efeitos (28 commits), o
Enio mandou integrar, e o integrador re-aplicou a linha na `main` (as SHAs mudaram — os commits têm
outra identidade lá). Depois o worktree levou `git merge --ff-only main`, trazendo junto **106
commits de outras linhas**. O `check --workspace` foi re-rodado na árvore combinada — um merge
textualmente limpo pode estar semanticamente quebrado, e é ele que cruza isso.

⚠️ **No início da SUA jornada, faça `git rebase main` de novo** (DIRETRIZ §1.5.2.3) — outras linhas
podem ter integrado desde este handoff.

---

## §2 — O que JÁ ESTÁ na `main` (não reconstrua)

**Os Live Path Effects (ADR-0132) fecharam.** A espinha é uma **pilha por-caminho avaliada dentro do
`VecPath::cooked()`** — não um grafo de nós. É o `inkscape:original-d` + `d` generalizado: o
documento guarda a fonte autorada + a lista de efeitos, o mundo consome o cozido.

**Onde vive** (tudo em `crates/ph2d-vec-scene/src/`, ZERO dependência nova):

| ficheiro | o quê |
|---|---|
| `effect.rs` | a pilha (`PathEffect`, `FxEntry{effect, enabled}`, `FxCtx`, `run_stack`), a tabela `KINDS`, os acessores |
| `arclen.rs` | comprimento de arco de cúbica (Gauss-Legendre 16) + inverso + `subsegment` — escrito à mão porque a crate não alcança kurbo |
| `fx_trim.rs` | **Trim Path** — revela um trecho (o *draw-on*), medindo por arco |
| `fx_zigzag.rs` | **Zig Zag / Roughen** — cristas por arco; a UNIÃO com as âncoras de entrada faz o efeito COMPOR |
| `fx_repeat.rs` | **Repeater** — 2 eixos (grelha) + 2 rotações (Spin/Orbit), o *Array* do Blender |
| `fx_warp.rs` | **Pucker & Bloat** — âncoras para um lado, curva para o outro |

**O menu tem 4 efeitos.** A seção **Effects** no painel (`ph2d-panel-vector`) é **dirigida pela
tabela**: cada efeito é um **card** (nome · ↑ ↓ ordenar · 👁 olho · ✕ apagar) e os parâmetros
DESCRITOS por baixo. **Acrescentar um efeito custa ZERO mudança de painel** — foi a promessa do ADR,
e ela foi MEDIDA (os três últimos efeitos entraram sem uma linha de painel).

**Como se acrescenta um efeito** (a receita inteira, hoje):
1. um variant novo em `PathEffect` (**apendado por último** — postcard é posicional);
2. os braços em `is_neutral`/`from_kind`/`kind_index`/`label`/`params`/`get`/`set`/`apply` + os
   acessores `as_*` (o `match` é exaustivo de propósito e obriga-o a decidir cada um);
3. uma linha em `KINDS`;
4. **uma linha na folha de contacto (§0), e OLHAR.**

Nada de shell, nada de painel. Se você tocar no painel para acrescentar um efeito, parou algo.

---

## §3 — Armadilhas já pagas (não re-aprenda)

Cada uma custou um smoke ou uma leva de gates. A lição, não a saga.

1. **Renderize e OLHE — §0.** É a maior. Um gate de buffer verde não vê uma forma rasgada.
2. **A escala de um parâmetro é RELATIVA à forma.** As formas da cena têm ~2-3 unidades; um slider
   em unidades de mundo é inútil. `Size` do ZigZag e `Move X/Y` do Repeater são percentagens da
   dimensão (o `FxCtx`). O ZigZag mede pela MÉDIA (uma onda não tem eixo); o Repeater mede POR EIXO
   (x pela largura, y pela altura — é o que faz `100` encaixar).
3. **O `FxCtx` sai do caminho AUTORADO**, não do que chega a cada efeito — senão a ordem da pilha
   mudaria o significado de um botão. **Exceção deliberada:** o Repeater mede a ENTRADA dele
   (ladrilhar é uma operação sobre a coisa ladrilhada; é o que faz Repeater∘Repeater dar grelha).
4. **Um campo NÃO-AFIM tem de SUBDIVIDIR antes de mapear** — senão torce um lowpoly. Foi a causa do
   Twist. Uma escala uniforme (Bloat) é afim e não precisa. ⚠️ Ver §5: o Twist foi CORTADO mesmo com
   a subdivisão correta.
5. **Um efeito que MULTIPLICA contornos não passa pelo `apply_per_contour`** (um buraco de compound
   copiado sozinho deixa de ser buraco). O Repeater constrói a saída direto; é o único assim.
6. **Um id não pode ter dois tipos de widget.** Uma caixinha pintada como botão precisa de id
   PRÓPRIO — um slider não emite `Click` no Up, e o ramo do toggle vira código morto silencioso.
7. **O oráculo tem de conter o fenómeno.** Um gate do Twist media um ponto NO centro (invariante sob
   qualquer rotação) e ficava verde com o campo errado. Distância a PONTO amostrado ≠ distância a
   SEGMENTO. A sonda preenchia em even-odd e mostrava rasgo onde o produto tem tinta cheia.
8. **Um teto de eixo não é o teto de uma grelha** — o custo é o PRODUTO (`MAX_TOTAL=1024`). E todo
   teto é MEDIDO (a tabela mora ao lado do número em `effect.rs`).
9. **O pivô mede o AUTORADO, não o cozido** (`vec_transform.rs::settle_origins` → `path_bbox`). Um
   sistema por-frame que lê geometria cozida e escreve no documento reage a efeitos e mina um passo
   de undo. (Ver §5 — o undo continua aberto.)

---

## §4 — A FILA (o que fazer a seguir)

Nenhum é bloqueante; escolha com o Enio. A ordem abaixo é a minha recomendação de valor/custo.

### §4.1 — Fechar o UNDO da pilha ⚠️ o único aberto que o Enio já reportou

O Enio reportou três vezes que os efeitos **não têm undo**. Eu **não consegui reproduzir** — dois
agentes e quatro varreduras confirmaram que **não há assimetria nenhuma** entre os efeitos e o resto
do documento no caminho input→bus→mutação→diff→restore (`effects` está no `PartialEq`, o restore
reinstala a cena, a mutação corre antes do hook). Corrigi o único ponto onde eles se comportavam
diferente (o pivô, §3.9). **Não afirmo que fechou.**

O protocolo que decide numa corrida (o log foi retirado; re-instrumente as três saídas antecipadas
de `post_frame_undo` se preciso):

```
PH2D_UNDO_LOG=1 cargo run -p ph2d-host-desktop
```
Adicione um efeito e leia. **Nenhuma linha** ⇒ o passo não é registado (a montante: `sole_path`
devolveu `None`, ou o Click não chegou ao drain — e então o efeito também não teria sido aplicado).
**`vec=true` + passos a mais em frames seguintes** ⇒ passo espúrio, o 1º Ctrl+Z gasta-se nele (a
classe do `vec_zorder_fixpoint_tests`). Detalhe: §11/§14 do
`docs/HANDOFF_line_vector_integracao_2026-07-18b.md`.

### §4.2 — Chamfer (tipo de quina)

**Quase de graça** sobre o `corner_live` (ADR-0121): reta em vez de arco. Não é efeito de pilha — é
um modo do raio de quina, por-vértice. O ponto de extensão já existe.

### §4.3 — Texto em caminho

Ficou **muito mais barato**: o `arclen.rs` que o Trim trouxe é o pré-requisito (posicionar glifos a
espaçamento igual pede inverso de comprimento de arco). É um subsistema de fontes, não um efeito —
uma linha inteira, não uma entrada na pilha.

### §4.4 — ⚠️ Offset Path — como COMANDO, não como efeito

**Achado arquitetural desta sessão, e poupa um dia:** offset correto exige tratamento de quinas e
remoção de auto-interseções — isto é, o **motor booleano** (`ph2d-vec-boolean`). A `ph2d-vec-scene` é
sem-dependências de propósito, e como todo efeito da pilha é avaliado DENTRO dela, **nenhum efeito
alcança a booleana**. O Offset tem de ser um **comando de edição**, como as booleanas que já existem.

### §4.5 — O TWIST, quando houver como verificá-lo

Foi cortado (§5). A subdivisão adaptativa que ele precisa **funciona e tem gate**; o que faltou foi
um modelo do campo que não rasgue sobre quinas, e uma referência que eu conseguisse verificar. Volta
quando alguém o souber especificar — o motor está no histórico (`fx_warp.rs` antes do commit da
sonda).

### §4.6 — Restante da fila herdada

Largura variável · mais primitivas · blend em cadeia (>2 formas) · morph vivo (o `steps()`/`morph(t)`
do `ph2d-vec-blend` já serve). Rig+skinning (LBS port do Rive, MIT) fica **para o FIM de tudo**.

---

## §5 — O que foi TENTADO e REPROVADO (não repita)

- **O Twist NÃO entrou.** Quatro campos diferentes (força↑ com raio, força↓, raio pela média, raio
  pelo máximo, subdivisão 6× mais fina) — **todos rasgavam** sobre uma forma com quinas: qualquer
  queda radial cria um diferencial enorme numa aresta e o canto chicoteia. **A subdivisão não é o
  problema** (há gate a provar que ela não move a curva); o problema é o modelo do campo. *"Um item
  de menu que produz geometria rasgada é pior do que um item que falta."*
- **O Pucker & Bloat como ESCALA** — a 1ª versão escalava âncoras e alças pelo mesmo fator: é o
  gizmo, não um efeito. A definição da Adobe é um PAR de fatores OPOSTOS.
- **O Repeater com rotação orbital como ÚNICA** — a 2ª versão substituiu a órbita pelo spin.
  Substituir foi o erro: as duas rotações fazem coisas diferentes e as duas servem.

---

## §6 — Estado dos gates (verifique no início da sua jornada)

Rodados na árvore combinada, nesta sessão:
- `cargo check --workspace --all-targets` ✅
- `cargo test` das 4 crates (`ph2d-vec-scene` · `ph2d-panel-vector` · `ph2d-host-desktop` ·
  `ph2d-editor-core`) ✅ **1964**, 0 falhas
- `cargo clippy --workspace --all-targets` ✅ 0 warnings

⚠️ Os arch-gates de arquivo (`no_magic_numeric`, LOC cap, tofu-glyphs) moram na **`ph2d-editor-core`**
e **NÃO** rodam com `cargo test -p` de outra crate. Rode-os no fechamento — a linha pagou esse
pedágio três vezes.

⚠️ **Toque foundational com cuidado:** a sessão passada tocou `ph2d-editor-core` (o scrub das caixas
numéricas inteiras — o bug era do app inteiro, não só do vetor). Se você tocar foundational, a
integração passa pelo `scripts/foundational-integrate.sh` (ADR-0107) — anote no seu handoff.

---

## §7 — Documentos-fonte desta sessão (leia sob demanda, não inteiros)

- [`docs/HANDOFF_line_vector_integracao_2026-07-18b.md`](HANDOFF_line_vector_integracao_2026-07-18b.md)
  — o handoff de INTEGRAÇÃO desta sessão. §15 (a leva de efeitos), §16 (o método), §11 (o protocolo
  do undo) são os que valem reler.
- [ADR-0132](architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)
  — por que a pilha, e não um grafo. §1 registra que o contrato congelado foi MEDIDO e não bloqueia.
- Os handoffs `continuacao_2026-07-13*..2026-07-18` — históricos das waves anteriores (Live Corners,
  Shape Builder, Blend, Envelope). Não precisa deles para os efeitos.
