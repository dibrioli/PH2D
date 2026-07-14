# Handoff de CONTINUAÇÃO — linha `line/FLIP` (a partir de 2026-07-14)

> **Você é o agente da linha `line/FLIP`.** Modo L (ADR-0106/0107): trabalhe **dentro da worktree**
> `Worktrees/line-FLIP` (branch `line/FLIP`). Você **não integra, não pusha, não roda ship** — fecha
> o bloco, escreve o handoff de integração (DIRETRIZ §1.5.9) e **PARA**.
>
> Todo comando com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && …` — o cwd volta
> ao repo primário a cada turno, e um caminho relativo escreve na **árvore errada**.

---

## 1. Estado (2026-07-14)

A jornada anterior **integrou** (6 linhas, `--ff-only`; registro em
[`docs/REGISTRO_integracao_jornada_2026-07-13.md`](REGISTRO_integracao_jornada_2026-07-13.md)).
A branch da linha foi **resetada ao `main` integrado** — não há nada pendente de merge.

- **Base:** `main` (`4d203d48`) + 1 commit local (`ace54f41`, o teto do `pack_perf` por perfil).
- **Gates na árvore integrada:** verdes (`ph2d-flip` 71 · shell/bins 110 no filtro `flip` · painéis ·
  `ph2d-flip-reshape`).
- **Esquema:** o pin virou **TRIPLA** — `(PROJECT_SCHEMA, FLIP_SCHEMA_VERSION, VEC_SCENE_SCHEMA_VERSION)
  = (13, 5, 8)`. O integrador **contou** as 6 quebras das 4 linhas em vez de escolher um lado
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). **Se você bumpar o `FlipDoc`, suba os
  dois e atualize a tripla.**

### 🔴 ITEM 0 — três ondas entraram no `main` **SEM SMOKE**

O registro da integração marca a linha como **parcial**: **W7 (multiframe)** · **W7.1 (Instance /
Unlink)** · **W7.2 (a pose do quadro)** foram integradas **antes** do smoke do Enio.

**Antes de abrir feature nova, o smoke dessas três é a prioridade.** Roteiro:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && cargo run --release --bin ph2d-host-desktop
```

1. Desenhe no quadro 0 → **🔗 Instance** (Key ops da tira) → nasce a chave seguinte com o **mesmo
   desenho** (pontinho na célula).
2. Modo **Edit**: clique na forma e **arraste** → ela anda **só neste quadro** (a arte segue
   compartilhada). Volte ao quadro 0: o original não se moveu. *(É a correção do W7.2 — o Enio
   reprovou a versão sem pose: "a instância não pode ser movida sozinha".)*
3. **Sculpt** num dos dois → a deformação aparece **nos dois** (posição é do quadro; forma é da arte).
4. **Ghost** ligado → o fantasma aparece **no lugar dele**, não em cima da arte de agora.
5. **Tween** entre duas instâncias em lugares diferentes → os inbetweens **deslizam**.
6. **Unlink** (🔗 quebrada) → o pontinho some e os quadros divergem de verdade.
7. **Multiframe:** Shift/Ctrl+clique em 2+ células → um gesto de Sculpt edita todas. Toggle
   **Falloff** = os vizinhos recebem menos influência que o quadro ativo.

Se algo reprovar: o fix é um **commit novo nesta linha** (o `main` já tem o código), e a linha volta
ao ciclo normal de integração.

---

## 2. A fila (ordem recomendada)

### A. **Girar / escalar a seleção** — *o buraco que o smoke vai encontrar*

Hoje o Edit Mode só **translada**. Assim que o Enio mover uma instância, ele vai tentar **girar**.

- O caminho é o **gizmo de sprite** (o mesmo que move o objeto), mas ele é **por-entidade** (escreve
  num `Transform` do ECS) e **um traço não é entidade** → exige um **consumidor novo**: bbox da
  seleção → `GizmoView` → delta assado.
- **Decisão de design que esta wave herda:** a pose da chave (`FlipFrame.offset`) é hoje **`Vec2`
  (translação)**. Girar/escalar uma **instância** não pode escrever geometria (a arte é dos dois
  quadros) → a pose tem de virar **afim** (`[f32; 6]`), com bump de `FLIP_SCHEMA_VERSION`. O render e
  o funil de entrada **já compõem um `Xform`** (`flip_transform::{art_to_world, world_to_art}`), então
  a troca é local: só o TIPO do campo muda, os dois consumidores continuam iguais.
- Em arte **exclusiva**, girar/escalar continua sendo geometria (o caminho comum).
- **Depende do veredito do smoke** (§ITEM 0): se a pose for reprovada, esta wave muda de forma.

### B. **Seleção no domínio POINT** — *independente do smoke*

Hoje a seleção é **por traço** (`FlipStroke.selected`, domínio Curve do GP). O GP também seleciona
**pontos** — é o que permite mover uma âncora só, e o que torna a máscara do Sculpt fina de verdade.
Custo: um vec paralelo de flags no traço (schema) + hit-test de ponto + o desenho das âncoras.

### C. **Refinos do balde** — *independente*

Gap Closure com overlay **vivo** (hoje o número é cego: você não vê o alcance antes de clicar) ·
modo **Gap Radius** · **Colorize** (LazyBrush / trapped-ball: pintar a cena inteira com poucos
rabiscos).

### D. **Refinos de camada / borracha** — *independentes, baratos*

`duplicate_layer` (o modelo não tem) · reorder por **drag** (só ↑↓ por botão hoje) · **máscaras de
camada** na UI (`FlipLayer.masks` existe no modelo, o compositor v1 não aplica) · borracha com **raio
próprio** (hoje = tamanho do brush) + preview do círculo · curva de pressão editável.

### E. **W6 — timeline global** — *pergunte ao Enio antes*

Estava **ADIADA** porque a timeline principal estava em obra. Ela **avançou muito** desde então
(composição de clips, save da animação, relógio único). O playhead do Flip **já é o global**, então
não há relógio a reconciliar. **É decisão do Enio reabrir.**

### F. Deferidos antigos (W1)

Round caps, bevel/round joins (v1 = flat + miter clampado) · LRU no cache de tesselação (hoje cresce
com o nº de desenhos únicos — bounded pelo documento).

---

## 3. As armadilhas VIVAS deste módulo (leia antes de tocar em geometria)

1. **Pincel ABSOLUTO:** a largura do traço é em **px de TELA**; a geometria é em unidades de
   **DOCUMENTO**. Misturar as duas quebra sob zoom — foi o que matou o balde **três vezes**
   (`BUGS_flip.md` #11/#14/#16).
2. **Render e autoria saem do MESMO par de funções** (`flip_transform::art_to_world` /
   `world_to_art`). Se você acrescentar algo à cadeia da arte (pose, deformador, o que for), ele entra
   **nas duas** — o gate `the_render_and_the_input_are_exact_inverses` prende o par.
3. **A pose sai pelo mesmo mapa do desenho** (`offset_at_cycled`): sob Loop, arte e lugar têm de vir
   do mesmo quadro-fonte ([[feedback_derived_coordinate_seed_must_match_sample]]).
4. **Multiframe é ancorado na ARTE, não no mundo** — um quadro-alvo deslocado é esculpido no mesmo
   ponto da geometria DELE. (Eu escrevi a compensação de mundo primeiro; um gate a derrubou.)
5. **Postcard é POSICIONAL:** qualquer mudança de forma no `FlipDoc` ⇒ `FLIP_SCHEMA_VERSION` **e** o
   par no `project.rs` (hoje uma tripla). Sem o bump, um arquivo velho passa na checagem e é lido com
   o layout novo — **não dá erro, dá geometria embaralhada**.
6. **Vello: o transform do `stroke` MULTIPLICA a espessura da caneta.** Chrome/realce se desenha em px
   de tela com `Affine::IDENTITY`, transformando os PONTOS (o realce da seleção virou um borrão por
   causa disso).
7. **Undo mutação com `cp`, nunca `git checkout`** — o checkout apaga a feature junto e o gate "passa".

---

## 4. Como fechar um bloco

Gate batched **1× no fim** (não por task): `cargo test -p ph2d-flip -p ph2d-flip-render
-p ph2d-flip-reshape -p ph2d-panel-flip -p ph2d-panel-flip-frames -p ph2d-tool-flip` +
`cargo test -p ph2d-host-desktop --bins flip` + `cargo clippy --all-targets` + `cargo fmt`.
Todo claim vira **teste com mutação provada** (mute o CÓDIGO, veja vermelho, restaure com `cp`).

Depois: handoff de integração (DIRETRIZ §1.5.9) e **PARE**.
