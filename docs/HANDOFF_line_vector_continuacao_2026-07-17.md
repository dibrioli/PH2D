# HANDOFF de CONTINUAÇÃO — `line/Vector` (para o próximo implementador)

**Para:** a LLM que assume a linha `line/Vector` daqui pra frente.
**De:** a sessão de 2026-07-17 (construiu o Envelope/Warp — Fatias A+B do ADR-0129).
**Estado:** **a linha está RE-PREPARADA e sincronizada com a `main`.** O envelope já INTEGROU.
Você começa numa árvore limpa, em cima de tudo o que já foi shipado. Não há nada meio-feito seu
para herdar — há uma **fila** (§4) e um punhado de **armadilhas já pagas** (§3) para não re-aprender.

> **Leia primeiro, nesta ordem:** `CLAUDE.md` (o roteador — os 7 inegociáveis + §5 do Vector) ·
> `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` (a CADA passo) · e então o
> **ADR-0129** ([`docs/architecture/decisions/0129-vector-envelope-warp-one-spine-cage-as-container-entity.md`](architecture/decisions/0129-vector-envelope-warp-one-spine-cage-as-container-entity.md))
> — a fonte da verdade do desenho do envelope. Este handoff é só o **mapa da linha + a fila**;
> ele NÃO duplica o ADR nem os handoffs anteriores.

---

## §1 — Onde a linha está (o estado, em 30 s)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **HEAD** | `cdc3acc1` = **exatamente a `main`** (0 ahead / 0 behind, árvore limpa) |
| **Modo** | **Modo L** (workstation, worktree próprio, [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)) |
| **Gates da linha** | **verdes nesta máquina** (envelope: 79 passed; os 6 gates de host + os 2 do settle passam) |

**Como a linha foi re-preparada** (o que você NÃO precisa refazer): o worktree foi
`git merge --ff-only main` — como a `main` já continha todo o trabalho da linha (o integrador
fez o merge por ordem do Enio), o fast-forward foi limpo e agora **HEAD == main**. Você começa
do zero-de-dívida.

**O que JÁ ESTÁ na `main` (não reconstrua):**

- **Blend Object vivo — ADR-0128 COMPLETO** (Fases A/B/C1/C2a/C2b/D). Objeto não-destrutivo que
  interpola 2..=5 formas e as segue; spine editável no Node; Expand/Release. Handoff:
  [`HANDOFF_line_vector_continuacao_2026-07-16.md`](HANDOFF_line_vector_continuacao_2026-07-16.md).
- **Envelope / Warp — ADR-0129, Fatias A + B** (esta sessão). Detalhe abaixo.

> ⚠️ **Renumeração:** os dois ADRs desta família foram renumerados pelo integrador na integração
> (dedupe de colisão, gate `architecture_adr_numbers_are_unique`): **Blend = ADR-0128**,
> **Envelope = ADR-0129** (eram 0122/0123 nos rascunhos das sessões antigas — se um comentário
> antigo citar 0122/0123 para vetor, é o número velho).

---

## §2 — O que o Envelope entrega HOJE (o alicerce da fila)

**Deformar geometria Bézier por um mapa NÃO-AFIM** (ADR-0129). Já está construído e smokado:

- **O motor — crate `ph2d-vec-envelope`** (nova, glob-membership `crates/*`, dep `ph2d-vec-scene`
  + `kurbo`; `ph2d-vec-scene` continua pura). A espinha é **`densificar → deformar → refitar`**:
  - `trait Warp { map([f64;2])->[f64;2]; jacobian(...); }` — o gesto é só isto.
  - `warp_path(path, &warp, accuracy)` — deforma um `VecPath` inteiro, **por-segmento**
    (`warp_contour`), preservando as âncoras autoradas; usa o `fit_to_bezpath` do kurbo 0.13
    (o fitter de Levien — a doc dele nomeia "perspective transform" como caso de uso).
  - **1º gesto: `QuadWarp`** (`quad.rs`) — homografia de Heckbert 1989 §3.1 em `f64` (forma
    fechada, 2 Cramer 2×2, sem solve 8×8; ramo paralelogramo→afim; convexo mantém o horizonte
    `w=0` FORA da gaiola). `QuadWarp::new` devolve `None` numa gaiola degenerada.
- **O host — `shells/desktop/src/envelope_live.rs`** (espelho do `morph_live`): o Envelope é uma
  **entidade ECS** com o componente `ph2d_ecs::VecEnvelope { source: Vec<u8> (postcard), corners }`.
  `recook` desserializa a fonte congelada, aplica a `QuadWarp` dos `corners`, escreve a saída em
  MUNDO no path da cena, e **força a identidade** na entidade (a geometria é mundo). Roda a cada
  frame (`upkeep` antes do settle, `recook` depois do build).

**A cena de smoke (aprovada pelo Enio 2026-07-17):**

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && PH2D_BUILD_SMOKE=11 cargo run -p ph2d-host-desktop --features panel-vector
```

Uma elipse nasce deformada por uma gaiola de perspectiva (topo pinçado a 35% da base). **Não há
UI ainda** — a gaiola é autorada pela cena de smoke; **arrastar os cantos é a Fatia 1 da fila**.

---

## §3 — As armadilhas JÁ PAGAS (não re-aprenda estas)

Cada uma custou tempo nesta linha ou nas anteriores. Elas são a razão do desenho ser o que é.

1. **A armadilha-mãe do warp:** `for v in verts { v = warp(v) }` **está ERRADO.** Só mapa AFIM
   comuta com avaliação de Bézier; o ingênuo acerta os cantos e MENTE no meio de cada segmento
   (é o bug aberto do Inkscape #10547). Por isso a espinha é `densificar→deformar→refitar`, e o
   gate-mãe é **invariância à subdivisão** (`ph2d-vec-envelope/tests/split_invariance.rs`): partir
   uma cúbica e deformar cada metade == deformar a inteira. **Se você adicionar um gesto novo
   (`impl Warp`), ele passa por esse gate de graça** — não reescreva o pipeline.
   [[feedback_test_with_product_numbers_not_convenient_ones]]

2. **O `break_cusp` do `WarpedCubic` volta `None` DE PROPÓSITO** (`fit.rs`, com ⚠️ na doc). Para
   Quad/afim/Coons isso é correto (esses mapas não dobram). **A Fatia E (pinos/MLS) DOBRA a ~90°**
   e o fitter precisa saber quebrar a cúbica na cúspide — quem for fazer MLS **tem de** implementar
   o `break_cusp`, senão o refit atravessa a dobra e produz lixo.

3. **Um gizmo sobre geometria que se MOVE dobra** (a lição do Blend, ADR-0128, 5 tentativas
   revertidas). A gaiola do envelope re-cozinha a forma em MUNDO a cada frame; pendurar o gizmo
   de sprite nela faz a matemática dele girar em torno de uma bbox que muda debaixo dos pés. Por
   isso **as alças da gaiola (Fatia 1) são PRÓPRIAS e vivem no modo Node** — não são o gizmo de
   sprite. No modo Select, quem se move é a FORMA (a Fatia 2).

4. **`settle_origins` tem de PULAR o `VecEnvelope`** (já está feito — `vec_transform.rs` + o
   componente está em `DERIVED` no gate `settle_skips_every_derived_geometry.rs`). Geometria de
   MUNDO reescrita por frame; assentar o pivô somaria geometria+Transform e deslocaria a forma.
   **Todo host `*_live` novo que force a identidade cai nesse gate** — o irmão de presença cobra.
   [[feedback_a_condition_that_enumerates_its_readers_rots]]

5. **A fonte autorada sobrevive no COMPONENTE, não no path da cena.** O `recook` sobrescreve o
   path com a deformada; a fonte afiada fica nos bytes do `VecEnvelope` (é ela que undo/save
   recuperam). Sem isso, o 1º frame varreria o desenho — o *"funciona e depois esquece"*.
   [[feedback_works_then_silently_forgets_recook_wipes_authored_state]]

6. **O `grep` desta shell MENTE.** Durante a investigação do smoke, `grep` (e até `command grep`)
   devolveram resultados vazios/inconsistentes 3+ vezes; só **`rg` (ripgrep)** foi confiável. Toda
   busca NEGATIVA ("não existe X") precisa de um controle POSITIVO — prove que a busca acha o que
   você SABE estar lá. [[feedback_a_negative_search_needs_a_positive_control]]

7. **NÚMEROS QUE SOMAM:** três gates contam componentes registrados (`ph2d-ecs`/`-render`/`-script`
   registry). O `VecEnvelope` já soma +1 em cada (31→32 / 32→33 / 32→33). Se você registrar OUTRO
   componente, **some — não escolha um lado**. [[feedback_numbers_that_sum_across_lines_count_dont_pick]]

---

## §4 — A FILA (a ordem é do Enio; o ADR-0129 §Plano é a fonte)

### 4.A — Fechar o Envelope (a prioridade — o motor já existe, falta a UI e os gestos)

1. **Arrastar os cantos da gaiola** (modo Node) — o 1º gesto vivo. Alças **próprias** (não o gizmo
   de sprite, §3.3). Isto transforma o `create`/`attach` de hoje (que hoje só o smoke exercita)
   numa ferramenta de verdade.
2. **Mover o objeto-envelope inteiro** (modo Select) — a fonte está congelada em MUNDO no
   componente; mover o conjunto aplica um afim aos `corners` + à fonte (ou re-baka).
3. **O container multi-filho** — o ADR §2 quer **1 gaiola para N formas**; hoje é **1-para-1** (o
   "Make with Warp" de um objeto só). Esta fatia generaliza.
4. **Release / Expand** — materializar os passos (assar a deformada como forma comum) e soltar a
   gaiola. Espelha o Expand/Release do Blend (ADR-0128).
5. **O painel** (seção Envelope, docado no slot do Inspector como o resto da tool): Fidelity
   (a `accuracy` do fit) + presets + escolha de gesto (Quad / 4-curvas / Pinos). **Lembre:**
   UI de tool é **painel docado** (`ph2d-panel-vector`), NUNCA `FloatingPanel`; zero string
   hardcoded / hex / f32 de UI (HR-15).
6. **Os outros gestos** (cada um é só um `impl Warp` novo — o pipeline não muda):
   - **Fatia C — presets** (geradores de gaiola: arco, bandeira, etc.).
   - **Fatia D — 4 curvas de lado / Coons** (a gaiola deixa de ser 4 pontos e vira 4 curvas).
   - **Fatia E — pinos / MLS-rigid** (Moving Least Squares, Schaefer 2006). ⚠️ **A mais delicada:**
     usa `f_r(v) = S·(v−p_*)/|S| + q_*` (α=1), **NUNCA** a Eq.8 do paper (NaN em `v=p_*`); 1 pino
     só = NaN; α **não** controla localidade; e **exige o `break_cusp`** (§3.2). "Rigid" ≠
     localmente rígido: dobra ~90°, σ∈[0.38,1.44] a 45°.

### 4.B — A fila herdada do Vetor (pós-envelope, dos handoffs anteriores)

Nenhuma bloqueia; escolha por ordem do Enio. Fonte:
[`HANDOFF_line_vector_continuacao_2026-07-13c.md`](HANDOFF_line_vector_continuacao_2026-07-13c.md)
+ [`HANDOFF_line_vector_continuacao_2026-07-16.md`](HANDOFF_line_vector_continuacao_2026-07-16.md).

- **Live Path Effects como NÓS** — o multiplicador do módulo. A costura fonte≠cozido do ADR-0121
  (`VecPath::cooked()`) **já é o pré-requisito** e existe.
- **Morph vivo** (o `t` animável do Blend) — o desenho é o do CONECTOR (entidade cuja geometria é
  função pura da relação, re-cozida por frame); o `steps()`/`morph(t)` do motor já serve os dois.
- **Blend em cadeia** (>2 formas) · **tipos de quina** (chamfer é quase de graça: reta no lugar do
  arco) · **texto em caminho** · **trim path** · **repeater** · **largura variável** · **Replace
  Spine / Smooth Color** do Blend · mais primitivas.
- **Rig + skinning** (LBS port do Rive, MIT) — **deferido pro FIM de tudo**, após o módulo de
  desenho estar completo.

---

## §5 — Disciplina da linha (o não-negociável, resumido)

- **Fecha e PARA (§0.7).** Você implementa, escreve o handoff de integração (DIRETRIZ §1.5.9), e
  **espera**. **NÃO integra, NÃO faz ship, NÃO faz push** — isso é só por ordem EXPLÍCITA do Enio,
  via agente integrador. [[feedback_integration_only_enio_command_end_of_all_lines]]
- **Contrato congelado** (`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`; o
  `architecture_vector_contract_surface` do `ph2d-vector-doc`): **mexer exige ADR + parar e
  reportar ao Enio** (§0.2). O motor novo `ph2d-vec-*` NÃO é congelado (re-congelar é follow-up).
- **Fast mode:** `cargo check -p <crate>` no inner loop; teste/clippy 1× no fechamento. Commit
  `--no-verify` (instantâneo), sem push. Antes de declarar VERDE, **`cargo nextest run --workspace`**
  — o gate `no_tofu_glyphs` mora em `ph2d-editor-core` e varre `shells/desktop`; verde-de-crate ≠
  verde-de-workspace. [[feedback_no_tofu_arrows_in_string_literals]]
- **Cada fatia nasce com um exemplo pronto pro smoke** (`PH2D_BUILD_SMOKE=<n>`), não peça montagem
  ao Enio. [[feedback_ready_to_smoke_example]]
- **Gates são mutation-tested** — mute o código, veja o gate ficar VERMELHO, restaure. Um gate que
  nunca ficou vermelho não protege nada. [[reference_topic_mutation_proofs]]

---

## §6 — Referências (onde a verdade mora)

- **ADR-0129** — o desenho do envelope (a espinha, os gestos, o modelo entidade-container).
- **Pesquisa:** [`docs/Vector Module/21_pesquisa_envelope_warp.md`](Vector%20Module/21_pesquisa_envelope_warp.md)
  (a wave que decidiu a família de algoritmos: kurbo fit + entidade + um-spine-dois-gestos + Quad-first).
- **Handoff de integração desta fatia:**
  [`HANDOFF_line_vector_envelope_integracao_2026-07-17.md`](HANDOFF_line_vector_envelope_integracao_2026-07-17.md)
  (§3 os riscos de integração, §5 os gates, §6 a fila — este handoff a resume).
- **ADR-0128** + [`HANDOFF_line_vector_continuacao_2026-07-16.md`](HANDOFF_line_vector_continuacao_2026-07-16.md)
  — o Blend vivo (o irmão do envelope; mesmo padrão entidade-derivada).
- **Estado geral do módulo:** `CLAUDE.md §5` (a entrada "Vector Module").
