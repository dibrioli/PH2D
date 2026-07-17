# HANDOFF — `line/Vector`: assumir a linha e construir **Envelope / Puppet Warp**

**Para:** o agente que vai assumir esta linha (contexto novo).
**De:** o agente da sessão de 2026-07-16.
**Estado:** a linha está **limpa e toda smokada**. O próximo item da fila é o teu: envelope / warp.

> **Leia primeiro:** `CLAUDE.md` (inteiro, é curto) + `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`.
> **Depois:** o handoff irmão [`HANDOFF_line_vector_continuacao_2026-07-16.md`](HANDOFF_line_vector_continuacao_2026-07-16.md)
> — ele tem a identidade da linha, os riscos de INTEGRAÇÃO (as contas que somam, os ids, as deps) e
> as minas do Blend/Morph. Este documento aqui é só sobre o **envelope**: o terreno que eu escoteirei
> para tu não começares do zero.

---

## §1 — O estado que tu recebes

| | |
|---|---|
| **Branch** | `line/Vector` (worktree `Worktrees/line-Vector/`) |
| **HEAD** | `9e052ea9` |
| **Base do fork** | `4d203d48` (merge-base com `main`) |
| **Commits** | 38 |
| **Árvore** | limpa |
| **Workspace** | **7059/7059 verde** (`cargo nextest run --workspace --features panel-vector`), clippy limpo |
| **Contratos congelados encostados** | **NENHUM** |
| **Smoke** | **tudo aprovado pelo Enio** — ver §2 |

**Não integres nem faças ship** (Modo L, CLAUDE.md §0.7): fecha a tua fatia, escreve o handoff, e
PARA. Integração e ship são de um agente dedicado, só por ordem explícita do Enio.

---

## §2 — O smoke: a linha está LIMPA (e isto é raro, aproveita)

O Enio smokou e **aprovou tudo** o que estava pendente (2026-07-16):

| O quê | Veredito |
|---|---|
| Blend Object vivo (Fases A→C2b) + o modelo de arrasto | ✅ aprovado (ele iterou 5× na interação) |
| **Expand / Release** (Fase D) | ✅ aprovado |
| **Pontos livres do spine** | ✅ aprovado |
| **Morph vivo** (o `t` keyável) | ✅ *"parece perfeito"* |
| **Blend com rosquinhas** (compound path) | ✅ *"parece perfeito"* |
| Slider Steps + a caixa numérica | ✅ aprovado |
| **Stroke width chega a zero** | ✅ aprovado |

**A única coisa NÃO confirmada** (herdada, e pequena): o **Shift+clique em ponto** no modo Node
(`f0706d0b`, sessão anterior) — o Enio aprovou a mensagem, nunca relatou o clique. Se fores mexer no
`input_dispatch.rs`, confirma-o de passagem.

---

## §3 — ENVELOPE / PUPPET WARP: o terreno

### 3.1 A ARMADILHA, e ela vem antes de tudo

**Já está escrita**, em [`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md`](../Vector%20Module/20_pesquisa_ferramentas_de_artista.md) §4.
Lê essa seção **antes de escrever uma linha**. O resumo, porque ele decide a arquitetura inteira:

> Deformar os **pontos de controle** de uma Bézier por uma função **não-afim** (um warp, um
> envelope) **não deforma a curva corretamente** — a curva resultante **não é a imagem** da curva
> original. Só transformações **afins** comutam com a avaliação de Bézier.
>
> Os apps lidam com isso **subdividindo até a tolerância** e, às vezes, refitando. É por isso que um
> envelope no Illustrator "solta" pontos.

Ou seja: `for v in verts { v.anchor = warp(v.anchor) }` **está errado**, e erra de um jeito que
*quase* funciona — uma forma pouco curva parece certa, e a errada aparece quando o artista curva o
envelope. Se te vires a escrever esse laço, para.

O caminho honesto é **densificar → deformar → refitar**, com a tolerância como parâmetro.

### 3.2 O que o repo JÁ tem (e o que ele não tem)

| Peça | Onde | Serve? |
|---|---|---|
| **Homografia projetiva** (Heckbert 1989, corner-pin de 4 cantos) | `crates/ph2d-node-motion-four-point-warp` | **A math sim, o consumidor não.** Ele deforma *posições de elementos* (uma coluna do Motion), não curvas. E é **não-afim** — é o caso da armadilha, em pessoa. |
| **LBS skinning** | `crates/ph2d-node-rig-skin-deformer` | Idem: math de referência, consumidor errado. E o rig está **deferido pro fim de tudo** (CLAUDE.md §5). |
| `smooth_path` / `sharpen_path` / **`simplify_path`** / **`subdivide_path`** | `crates/ph2d-vec-scene/src/reshape.rs` | **Sim, e são o par que a armadilha pede** (`subdivide` densifica, `simplify` refita). Já são **per-contorno** (varrem o compound inteiro). |
| **Fit de cúbica (Levien 2021)** | `crates/ph2d-vector-doc/src/cubic_fit.rs` | Existe, mas ⚠️ é do foundational **VELHO** — o motor novo (`ph2d-vec-*`) **não depende dele** (confirmado). Puxar essa dep é uma decisão de arquitetura, não um detalhe. |
| **Refit Schneider corner-split** | `crates/ph2d-tool-painter/.../curve_refit.rs` | Existe, mas mora no **Painter**. Reusá-lo = extrair para uma crate comum, ou re-portar. |
| **A costura fonte ≠ cozido** (`VecPath::cooked()`) | `ph2d-vec-scene/src/corner_live.rs` (ADR-0121) | **É o pré-requisito, e ele está PAGO.** Ver §3.4. |

### 3.3 A pesquisa NÃO existe ainda, e o doc diz para não improvisar

O `20_*` §4 nomeia as quatro famílias e para aí, de propósito:

- **FFD** — Sederberg & Parry 1986 (free-form deformation por grade de controle).
- **MLS** — Schaefer, McPhail & Warren 2006 (moving least squares; *"Image Deformation Using
  Moving Least Squares"*).
- **ARAP** — Igarashi, Moscovich & Hughes 2005 (*"As-Rigid-As-Possible Shape Manipulation"*) — **é
  literalmente o paper do Puppet Warp** do Photoshop.
- **BBW** — Jacobson et al. 2011 (bounded biharmonic weights).

E o doc conclui: ***"Se formos fazer warp, a família de algoritmos vale um estudo próprio. Não fazer
isso de improviso."***

**Isso é uma instrução, e eu concordo com ela.** A DIRETIVA §1 diz a mesma coisa por outro lado:
*"Existe algoritmo de referência publicado? Porte-o antes de escrever a sua versão. Constante de
magia inventada = PARE e ache a fonte."*

**A minha recomendação de primeiro passo:** uma wave de **pesquisa** (fan-out, como a que precedeu o
compound path — ela achou que *ninguém* resolvia correspondência de contornos, e isso mudou o
código), fechada num **ADR** que escolha a família **antes** de qualquer implementação. As perguntas
que eu levaria:

1. **Envelope e Puppet Warp são a MESMA feature?** Não me parece. O *Envelope* do Illustrator
   deforma por **4 lados / uma forma / uma grade**; o *Puppet Warp* do Photoshop deforma por
   **pinos** sobre uma malha (ARAP). São dois gestos e possivelmente dois ADRs. **O Enio pediu os
   dois numa linha da fila** — vale perguntar-lhe se é um ou dois.
2. **Qual família?** FFD é a mais simples e a que casa com "envelope de 4 lados". ARAP é a que casa
   com "pinos". MLS é a que dá o meio-termo (pinos, sem malha). BBW precisa de malha e de
   pré-computo.
3. **O CorelDRAW é a referência mais rica, e a doc oficial dele dá os nomes**: 4 modos de restrição
   (Straight Line / Single-Arc / Double-Arc / Unconstrained) × 4 modos de mapeamento (Original /
   Putty / Horizontal / Vertical). *"O Illustrator não tem nada disso."* (`20_*` §4.)
4. **A tolerância de densificação é do artista ou do motor?** É ela que decide se o envelope "solta
   pontos" (Illustrator) ou preserva a estrutura.

### 3.4 Onde ele ENCAIXA — e esta parte já está resolvida

**O envelope é um efeito VIVO, e o padrão dele já existe nesta linha, três vezes.** Não inventes um
quarto:

- **ADR-0121 (Live Corners)** estabeleceu a costura: **o documento guarda a fonte (a quina afiada +
  o raio); o mundo consome a COZIDA (`VecPath::cooked()`)**. É o `inkscape:original-d` + `d`. Sem
  raio, `cooked()` é `Cow::Borrowed` — mesmo ponteiro, custo zero. Foi isso que permitiu ligar o
  cozido em TODO consumidor (render / hit-test / bbox / booleana / gradiente) sem mudar
  comportamento nenhum.
- **É o pré-requisito declarado dos Live Path Effects**, e um envelope **é** um Live Path Effect. A
  pergunta *"onde mora o envelope?"* já tem resposta: **na fonte**, e o `cooked()` aplica-o.
- **O ADR-0122 (Blend/Morph vivo)** e o `connector_live`/`morph_live` dão o outro padrão, para quando
  a geometria é função de uma RELAÇÃO: componente ECS guarda a relação, um `*_live::recook` re-coze
  por frame, a entidade vive na **identidade**. Um envelope cujo deformador é **outra forma** (o
  "Envelope by top object" do Illustrator) é exatamente essa forma.

**A escolha entre os dois é a tua primeira decisão de desenho**, e ela tem um critério: se o
deformador é **parâmetro** (uma grade de 4 lados que mora no próprio path) → `cooked()`. Se ele é
**outra entidade da cena** (uma forma que deforma outra) → componente + `*_live`.

⚠️ **Se fores pelo `*_live`:** o `settle_origins` tem um `filter` que ENUMERA os componentes de
geometria derivada, e o 5º que esquecer a linha quebra em silêncio. O gate
`shells/desktop/tests/settle_skips_every_derived_geometry.rs` cobra-o — **acrescenta o teu
componente a `DERIVED` e ao filter, e o gate fica verde**. Ele foi escrito nesta sessão exatamente
para tu não descobrires isso pelo olho.

---

## §4 — As lições desta linha que tu vais precisar (não as re-aprendas)

Cada uma custou um bug, esta semana:

- **Uma porta só produz um passo.** O `recook` e o `expand` chamam a MESMA `cook_links` — uma 2ª
  porta faria as formas saltarem no clique. Se o teu envelope tiver "preview" e "aplicar", eles têm
  de sair da mesma função. [[feedback_two_doors_to_the_same_question_diverge]]
- **Um gizmo sobre geometria que se MOVE dobra.** Cinco tentativas de dar gizmo ao spine do Blend
  foram revertidas (ADR-0122 lista as cinco). Se o teu envelope tem alças, elas não são um gizmo de
  sprite — são alças próprias, no modo Node.
- **Fixture simétrico não arma desempate.** Duas rosquinhas *idênticas* dão centroides exatamente
  iguais (`0.0`) e diferentes dão `1e-16` de ruído — o float fazia, por acidente, o trabalho do
  filtro que eu queria testar. [[feedback_identical_fixtures_hide_the_tiebreak_you_meant_to_test]]
- **O probe do oráculo não pode cair na fronteira** da hipótese contradita: um ponto EM CIMA da
  borda não tem resposta, e o gate vira cara-ou-coroa.
- **Gate de ausência precisa do irmão de PRESENÇA.** *"Largura 0 não desenha"* fica verde num
  renderer que não desenha nada. [[feedback_absence_gate_needs_a_presence_sibling]]
- **Um escape que nunca ajuda é enfeite.** Escrevi um gate para largura negativa, ficou vermelho, e
  **removi-o** ao provar que o estado é inalcançável — em vez de escrever uma guarda para ele.
  [[feedback_an_escape_that_never_helps_is_a_design_bug]]
- **Cerca de Chesterton:** eu chamei o `set_number_range` ausente do Width de bug. **Estava errado** —
  o `slider_chip` deliberadamente não o chama (um chip ligado a slider é limitado pelo ESPELHO, não
  por um range). Verifica antes de "consertar".
- **`HashMap` é tipo PROIBIDO** no repo (ordem de iteração não-determinística). Usa `BTreeMap`.
- **Desfaz mutação com `cp`, nunca `git checkout`** — o checkout apaga a feature e o gate "passa".

---

## §5 — Como eu abriria esta wave (a minha recomendação, a fila é do Enio)

1. **Pergunta ao Enio se Envelope e Puppet Warp são uma feature ou duas.** Eles são gestos
   diferentes (4 lados/forma vs pinos) e provavelmente famílias de algoritmo diferentes (FFD vs
   ARAP). A fila diz *"Envelope / puppet warp"* numa linha só, e isso pode ser abreviação — ou não.
2. **Wave de pesquisa** (o `20_*` §4 manda, e a DIRETIVA §1 também), fechada num **ADR** que escolhe
   a família + o conjunto de aceitação **concreto** + o **kill-criterion ANTES do build** (DIRETIVA
   §5: alvo irrefutável não é done-definition).
3. **Só então** o código, e o 1º gate é o da armadilha: **uma forma muito curva, deformada por um
   envelope curvo, tem de ficar na imagem da curva** — não nos pontos de controle deformados. Se
   esse gate não existir, tu vais ter um envelope que *quase* funciona.

**Cenas de smoke prontas** (o Enio não deve montar nada — `feedback_ready_to_smoke_example`); o `cd`
é ABSOLUTO e vai JUNTO ([[feedback_run_command_include_cd]]):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && PH2D_BUILD_SMOKE=10 cargo run -p ph2d-host-desktop --features panel-vector   # Morph
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && PH2D_BUILD_SMOKE=1  cargo run -p ph2d-host-desktop --features panel-vector   # Shape Builder
```

Faz uma para o envelope no MESMO arquivo (`shells/desktop/src/build_smoke.rs`, o `match` do frame).

---

## §6 — O resto da fila, depois de ti

4. Do Illustrator, o que falta no Blend: **Replace Spine** (os passos seguem um caminho desenhado) e
   **Smooth Color** (o nº de passos sai do degradê).
5. Backlog antigo: **Live Path Effects como nós** (o multiplicador — e a costura do ADR-0121 já é o
   pré-requisito; o teu envelope provavelmente é o **primeiro** LPE) · tipos de quina (chamfer é
   quase de graça: reta em vez de arco) · texto em caminho · trim path · repeater · largura variável
   · mais primitivas.

**Aberto por cima do Morph** (não é bloqueio para ti, é contexto): ele liga só DOIS objetos (uma
cadeia é o Blend) · o `t` não tem alça no canvas, só o slider · não há Expand/Release para ele.
