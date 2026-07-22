# Handoff de INTEGRAÇÃO — `line/FLIP` → `main` (Tween v2, 2026-07-22)

> **Para o agente INTEGRADOR.** A linha fechou a wave do **Tween v2**. O implementador parou
> aqui (CLAUDE.md §0.7).
>
> ✅ **SMOKE APROVADO pelo Enio (2026-07-22).** Todos os gates estão verdes, e o Enio rodou a
> cena (`PH2D_FLIP_TWEEN_SMOKE=1`) e aprovou: *"funcionou tudo como vc disse. Smoke OK."* — o
> boneco de palito, com o braço mantendo o comprimento ao longo do arco, o tronco casado sem
> deslizar, e o chapéu órfão desvanecendo só com Fade. O veredito deixou de ser condicional.
> A linha aguarda **ordem explícita de integração** do Enio (CLAUDE.md §0.7). O S1 (§7) foi o
> smoke que fechou o veredito.

## 1. Identidade

| | |
|---|---|
| branch | `line/FLIP` |
| HEAD | ver `git log -1 --format=%H line/FLIP` (o último commit é de docs) |
| base do fork (merge-base) | `13a04c7aab68` |
| commits à frente do `main` | **21** (8 do Tween v2 + o resto da **correção de pares**, o fix do S2 e docs — ver **§9**; use `git rev-list --count main..line/FLIP` para o número exato, o ff-only não depende dele) |
| `main` andou desde o fork? | **não** (`git rev-list --count HEAD..main` = 0) ⇒ **fast-forward limpo** |

> **Este handoff cobre DUAS entregas da mesma wave.** O **Tween v2** (§2–§8) está **SMOKE
> APROVADO**. A **correção de pares** (§9, o overlay CACAni + o re-par manual) está
> **construída e gateada, pendente de smoke** (`PH2D_FLIP_TWEEN_PAIRS_SMOKE=1`). As duas
> integram juntas; a segunda é aditiva sobre a primeira.

```bash
cd /home/enio/Documentos/Projetos/PH2D     # a árvore PRIMÁRIA
git status --short                          # limpa
git merge --ff-only line/FLIP
```

Se o `--ff-only` recusar, **PARE**: o `main` andou depois desta escrita (DIRETRIZ §1.5.5 —
resolva pelos **ESTÁGIOS do índice**, nunca pelos marcadores, e rode `cargo check --workspace`
depois).

## 2. O que este delta entrega

O inbetween do Flip deixa de ser o port literal do Grease Pencil em **duas** frentes, e o
doc completo (com todas as tabelas medidas) é [`docs/Flip/11_tween_v2.md`](Flip/11_tween_v2.md).

| | antes (o GP, e o nosso v1) | agora |
|---|---|---|
| correspondência | curva *i* ↔ curva *i*, puramente ordinal | custo geométrico + **atribuição ótima** (Hungarian); a ordem de desenho vira um TERMO do custo |
| interpolação | lerp de coordenadas — o traço corta pela CORDA | **espiral logarítmica** (BetweenIT/Disney): gira e escala em torno do ponto fixo |
| órfãos | só o lado de B tinha fade, e por índice | simétrico, e o órfão **viaja com o vizinho** enquanto some |
| UI | Ease e Fade existiam no motor e **não** na barra (dívida T3.7) | chip `Ease` + toggle `Fade`, pela porta única `tween_options()` |

**Nenhum knob novo governa o motor** — as duas metades são *subsunção*: a ordem de desenho
ainda decide quando tudo empata, e a translação pura sai **byte-idêntica** ao v1.

**Mais três defeitos PRÉ-EXISTENTES**, todos no auto-flip, todos que só a espiral tornou
visíveis (com o lerp o resultado já saía torto):

1. anel invertido pelas "pontas" (num traço fechado elas são vizinhas — a costura);
2. `da·db < 0` lia *"girou mais de 90°"* como *"desenhado ao contrário"*;
3. a comparação era de distância **ao quadrado**, que decide o oposto da distância real no
   braço de 120°.

As três heurísticas viraram **uma** pergunta: *qual dos dois jeitos de parear as pontas
percorre menos caminho?*

## 3. ⚠️ O que o integrador precisa saber ANTES de mesclar

### 3.1 Superfície pública NOVA na `ph2d-flip` (foundational), toda aditiva

```rust
pub use tween::{TweenOptions, TweenRequest, tween_drawing, tween_drawing_with}; // +1
pub use tween_match::{PAIR_REJECT_COST, StrokeFeatures, TweenPlan, features};   // NOVO
pub use ph2d_anim::{Easing, EasingFamily, EasingMode};                          // NOVO (re-export)
```

Nada foi removido; `tween_drawing` mantém a assinatura (agora delega). Consumidores da crate
no workspace: `ph2d-flip-render`, `ph2d-flip-fill`, `ph2d-flip-colorize`, `ph2d-flip-reshape`,
`ph2d-panel-flip*`, `ph2d-tool-flip`, `shells/desktop`.

### 3.2 Dependência nova: `libm` na `ph2d-flip`

`libm = { version = "=0.2.16", default-features = false }` — o MESMO pin das outras crates que
precisam do mesmo resultado em toda plataforma (`ph2d-ecs`, `ph2d-physics`, `ph2d-editor-core`,
`ph2d-wet-paint`). É `atan2`/`sincos`/`pow` de faixa cheia, que não cabem numa série truncada
como o Twist do reshape. **`Cargo.lock` só ganhou a aresta** (a crate já está no lock).

⚠️ Isto **contradiz na aparência** um comentário do `ph2d-flip-reshape::brushes.rs` que diz
*"ao contrário da libm, um polinômio é bit-idêntico entre plataformas"* — a frase vale para a
libm do **SISTEMA** (o `sin` da `std`), não para a crate `libm`, que é um porte puro-Rust do
MUSL e portanto igual em todo lugar. O comentário lá **não foi tocado** (é de outra wave); se
alguém o citar contra esta escolha, é este parágrafo que responde.

### 3.3 Arquivos COMPARTILHADOS tocados (onde um merge futuro morde)

| arquivo | o que a linha fez | risco |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/flip.rs` | **+2 consts + 1 fn de id** (`FLIP_TWEEN_EASE_DD`, `FLIP_TWEEN_FADE`, `flip_tween_ease_option_id`) | **só ADIÇÃO** — lista compartilhada, adicionar é seguro ([[feedback_a_shared_list_is_merged_against_todays_main]]) |
| `ph2d-ui-testkit/src/lib.rs` | **+2 métodos** (`set_dropdown_open`, `dropdown_is_open`) | aditivo; ver §3.4 |
| `ph2d-panel-flip-frames/src/{ids,state,toolbar_plan,paint,paint_toolbar,populate,event}.rs` | o 2º dropdown da barra | `PendingCycle` virou `{id, chip}` — **mudança de forma**, mas o tipo é `pub(crate)` |
| `shells/desktop/src/{main,render_loop/mod}.rs` | `mod flip_tween_smoke;` + a chamada no prólogo | adição de 1 linha em cada |
| `shells/desktop/src/{flip_strip,flip_strip_tests}.rs` · `render_loop/flip_bridge.rs` | os 2 campos novos do strip + a porta `tween_options()` | adições |

### 3.4 Por que o testkit ganhou 2 métodos

O open/close de um dropdown é feito pelo **dispatch genérico do SHELL**, não pelo
`apply_event` do painel — então um seam test que dirige `apply_event` **não alcançava** o
estado *"este popover está aberto"*, e a regra *"abrir um fecha o outro"* era intestável
naquela costura. Os dois métodos espelham exatamente os setters tipados que já existiam
(`set_toggle_on`, `set_number_value`), com o mesmo `panic!` em id ausente/tipo errado.

### 3.5 O que NÃO mudou

- **Nenhum SCHEMA foi bumpado.** O tween só ACRESCENTA chaves ao documento; `TweenOptions`
  não é serializado (é estado de barra, classe do `Record`). `PROJECT_SCHEMA`, `DOC_VERSION`
  e `VEC_SCENE_SCHEMA_VERSION` intactos.
- **Nenhum contrato congelado** (CLAUDE.md §6). `PanelEvent` foi reusado como está.
- **Nenhum ADR.** A wave implementa a spec que o `04_alem_do_blender.md §2` já tinha escrito.

## 4. Como VERIFICAR

```bash
cd /home/enio/Documentos/Projetos/PH2D                      # DEPOIS do merge
cargo test -p ph2d-flip -p ph2d-panel-flip-frames \
           -p ph2d-ui-testkit -p ph2d-editor-core -p ph2d-host-desktop --release
bash scripts/nextest-impacted.sh
```

**Rodado na worktree, verde:**

| gate | resultado |
|---|---|
| `nextest-impacted.sh` | **5145 testes, 5145 passaram** (inclui os 17 gates da §9) |
| `ph2d-flip` | 137 (+3 réguas `#[ignore]`) — **em debug E em release** |
| `ph2d-panel-flip-frames` (seam, com o botão **Pairs**) | 9 |
| `ph2d-host-desktop` (inclui o gesto/pick/overlay/seam da §9) | 962 |
| `cargo clippy --all-targets` nas crates tocadas | limpo |
| `file_loc_caps` (shell) · `architecture_workspace_file_loc_cap` · `architecture_panel_wiring_parity` · `node_id_collisions` · `no_tofu_glyphs` | verdes |

⚠️ **Rode as duas** (debug e release): a wave anterior desta mesma linha shipou um pânico que
só aparecia em debug porque o brief mandava rodar com `--release`.

### 4.1 As RÉGUAS (`#[ignore]`) — os números das constantes

```bash
cargo test -p ph2d-flip --release the_cost_ruler    -- --ignored --nocapture
cargo test -p ph2d-flip --release the_outlier_ruler -- --ignored --nocapture
cargo test -p ph2d-flip --release the_spiral_ruler  -- --ignored --nocapture
cargo test -p ph2d-flip --release the_plan_cost_ruler -- --ignored --nocapture
```

### 4.2 Provas de mutação (7, todas sangram)

| mutação | gate que morre |
|---|---|
| o rígido da translação volta a ser identidade | os 2 do órfão + a byte-identidade |
| recusa só absoluta (sem a metade relativa) | buraco · traço sozinho · panorâmica |
| recusa só relativa (sem o piso absoluto) | o limiar cercado dos 2 lados |
| termo ausente contado como zero | o da média ponderada |
| volta o teste de direções opostas do GP | o do giro > 90° |
| distâncias ao quadrado | idem |
| o botão Add ignora a barra · o Fade não escreve | os 2 de seam do shell |

## 5. As lições deste delta (leia antes de mexer no que ele tocou)

### 5.1 Duas colunas que SE CRUZAM não têm limiar entre elas

A régua do custo nasceu para achar o vão entre "legítimo" e "espúrio" e mostrou que **não
existe**: o pior par legítimo (`0,3352`) custa MAIS que o melhor "espúrio" (`0,2774`) — porque
o "cotoco" não é espúrio (um braço que encolhe muito É esse par). O que a tabela separa é a
zona AMBÍGUA do claramente-alheio, e é aí que o limiar mora.

### 5.2 Um limiar ABSOLUTO sozinho orfana uma panorâmica inteira

Quando a cena toda se desloca, todos os custos sobem juntos. Um custo alto só significa "não
é o mesmo traço" quando os VIZINHOS não subiram junto ⇒ a recusa passou a ter duas perguntas.
E a forma escolhida **apaga o caso especial**: com um par só, ele é a própria mediana.

### 5.3 Uma variante pode MENTIR sobre o que representa

`StrokeMotion::Lerp` dizia *"o rígido aqui é a identidade"*. Numa translação pura o rígido É a
translação — dentro do par o resíduo cobria a mentira, e **de fora** (um órfão que precisa
viajar junto) a resposta era "para lugar nenhum".

### 5.4 Três gates meus nasceram sem CONTER o fenômeno

- o da omissão do termo comparava retas × círculos DESENHADOS (que diferem também em arco e
  em régua) — media três coisas ao mesmo tempo;
- o da panorâmica usava `dx=140`, cujo custo (`0,255`) fica **abaixo** do limiar: a mutação
  que remove a metade relativa não o matava;
- o do chapéu órfão media contra `x = 0`, um número que eu **supus** — o polígono do fixture
  repete o ponto de fecho e a média dele é `−0,1`.

### 5.5 Três números que eu escrevi ANTES de medir estavam errados

O limiar (`0,21`/`0,44` esperados × `0,3352`/`0,2774` medidos) · a anisotropia da elipse 1.1:1
(`0,091` × `0,0715`) · e a frase *"o erro explode duas ordens de grandeza por década abaixo"*
no `DET_MIN`, que a tabela desmentiu (as linhas de baixo já caem no ramo da corda).

### 5.6 A meia-volta exata é AMBÍGUA, e o fixture não pode sentar nela

Girar 180° para os dois lados dá a mesma pose: não há informação no par que diga qual. O
fixture de 180° nasceu vermelho medindo a moeda que o último bit do `π` de `f32` jogou. O gate
principal foi para 170° e a ambiguidade ganhou gate próprio — *mesmo ali o traço não colapsa*.

## 6. O que fica ABERTO (nomeado, não escondido)

| item | onde | gatilho |
|---|---|---|
| **O overlay de PARES + o re-par manual** (lição CACANi) | [11 §7.2](Flip/11_tween_v2.md) | o `TweenPlan` já publica `pair_of_a`/`cost_of_a` **para isto**; um pino é custo 0 na célula + `BLOCKED` na linha/coluna |
| **Alinhamento de FASE da costura** em traço fechado | [11 §7.1](Flip/11_tween_v2.md) | dois anéis de mesmo sentido com o ponto 0 em lugares diferentes tweenam torcidos; a resposta é o `phase_only` do `ph2d-vec-blend` |
| **Torção em rotação grande** (o resíduo é lerp) | [11 §7.3](Flip/11_tween_v2.md) | Sederberg 1992 / Alexa 2000 — a correspondência era o pré-requisito |
| Backlog anterior da linha (pré-segmentação 4K · `trap_px` × `MAX_SIDE` · o `reach` do Gap Closure · a exceção `rayon`) | handoff de 21/07 | inalterados |

## 7. O SMOKE (S1) — o que falta para o veredito deixar de ser condicional

```bash
cd /home/enio/Documentos/Projetos/PH2D && \
  env PH2D_FLIP_TWEEN_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

A cena imprime `[tween-smoke] cena montada: 2 chaves (0 e 8) …` — **se essa linha não
aparecer, pare**: o resto não significa nada. Aperte **Add** e folheie 0 → 2 → 4 → 6 → 8.

| # | conferir |
|---|---|
| a | o **braço não encolhe** (percorre o arco, não a corda) |
| b | o **ombro fica parado** (é o ponto fixo) |
| c | **nada atravessa a figura** (B foi desenhado na ordem trocada de propósito) |
| d | com **Fade** marcado, o **chapéu some viajando** com a cabeça (sem Fade ele fica parado — é o default) |
| e | o chip **Ease** muda onde os inbetweens se acumulam |

Os itens (a), (b), (c) e (d) estão **medidos por gate** (`the_smoke_scene_shows_what_its_message_promises`
e `the_orphan_hat_travels_only_when_fade_is_armed`, ambos encenando pela MESMA `stage()` da
cena) — o smoke julga a APARÊNCIA, que é o que nenhum deles pode julgar.

## 8. Depois da integração

1. `./scripts/ship.sh` **completo**, e corrija todo `✗` antes de qualquer push.
2. **Push só por ordem EXPLÍCITA do Enio** (CLAUDE.md §0.7).
3. **Atualize a §5 do `CLAUDE.md`** com a entrada do Tween v2 — uma §5 que não descreve o que
   está no `main` faz a próxima LLM reconstruir o que existe.

## 9. A correção de pares (continuação da mesma wave — pendente de smoke)

O escape manual que a lição CACAni exige (o matcher erra, o artista corrige). Doc completo:
[`docs/Flip/11_tween_v2.md §8`](Flip/11_tween_v2.md).

### 9.1 Superfície pública NOVA na `ph2d-flip` (foundational), toda ADITIVA

```rust
impl TweenPlan {
    pub fn repair(&mut self, a: usize, b: usize) -> bool;   // força A[a] <-> B[b]
    pub fn unpair_a(&mut self, a: usize) -> bool;           // orfana A[a]
    pub fn unpair_b(&mut self, b: usize) -> bool;           // orfana B[b]
    pub fn a_len(&self) -> usize;                            // dims (para a guarda)
    pub fn b_len(&self) -> usize;
}
impl FlipObject {
    pub fn tween_with_plan(&mut self, req: TweenRequest, plan: &TweenPlan) -> u32; // commit corrigido
}
```

Nada removido; `tween` mantém a assinatura (agora delega a um `tween_inner` privado). Um par
manual **perde o `cost`** (`cost_of_a` = `None`) — a confiança do matcher não descreve uma
escolha do artista. O `tween_with_plan` tem **guarda de dimensões**: plano cujo `(a_len,
b_len)` não bate com os desenhos-chave é descartado e cai no automático (nunca pareia pelo
índice errado).

### 9.2 Módulos NOVOS no shell (isolados)

| arquivo | o quê |
|---|---|
| `shells/desktop/src/flip_tween_correct.rs` | a sessão (`TweenCorrect` na `FlipStrip`, estado de autoria) + o gesto puro (`apply_click`) + o pick em tela (`nearest_stroke`) + `build`/upkeep |
| `shells/desktop/src/render_loop/flip_tween_overlay.rs` | o overlay esquemático (linhas por confiança + anéis de órfão), px de tela, irmão do `flip_selection_overlay` |
| `shells/desktop/src/flip_tween_pairs_smoke.rs` | a cena `PH2D_FLIP_TWEEN_PAIRS_SMOKE=1` |

### 9.3 Arquivos COMPARTILHADOS tocados (onde um merge futuro morde)

| arquivo | mudança | risco |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/flip.rs` | `+FLIP_TWEEN_PAIRS` (id novo) | append |
| `ph2d-panel-flip-frames` (`ids/state/toolbar_plan/populate/event`) | +botão **Pairs** (snapshot `tween_pairs`, toggle, `BUTTONS` 17→18) | append em listas |
| `shells/desktop/src/flip_strip.rs` | porta única `current_tween_interval`; toggle Pairs + Add usa o plano corrigido | 581 LOC |
| `shells/desktop/src/{main,render_loop/mod}.rs` | `mod flip_tween_correct/pairs_smoke` + a chamada de overlay/upkeep/smoke no prólogo | +poucas linhas |
| `shells/desktop/src/render_loop/mod.rs` (`suppress_gizmo`) | **Pairs suprime o gizmo do objeto** — a caixa dele registra hits no `hit_index`, `on_canvas` vira falso e o clique de re-par seria roubado (o MESMO caso das tools de vetor, ao lado do qual entrou) | +2 linhas na condição |
| `shells/desktop/src/input_dispatch.rs` | 1 branch (`flip_wants_tween_pairs` no pen-down, antes dos modos) | append |
| `shells/desktop/src/render_loop/flip_bridge.rs` | `tween_pairs: strip.tween_correct.is_some()` no snapshot | 1 linha |

### 9.4 Gates (17 novos) + LOC

`tween_match_edit_tests.rs` (motor: repair/unpair/no-op) · `tween_tests.rs` (a correção
dirige o inbetween; plano de tamanho errado cai no automático) · `flip_tween_correct_tests.rs`
(o gesto + o pick) · `flip_tween_overlay_tests.rs` (a cor da confiança + a geometria de tela) ·
`flip_strip_tests.rs` (o seam: toggle abre/fecha · sem intervalo não abre · o Add usa a
correção) · `flip_tween_pairs_smoke_tests.rs` (a cena contém o fenômeno). **`tween_match_tests.rs`
foi splitado** (741→661) para o irmão `tween_match_edit_tests.rs` (cap de LOC).

### 9.5 O SMOKE (S2) — o que falta para o veredito da 2ª entrega

```bash
env PH2D_FLIP_TWEEN_PAIRS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

A cena imprime `[pairs-smoke] cena montada: … Pairs ja esta ABERTO.` e um guia em pt-BR. O que
olhar: o **overlay** aparece (duas poses, A azul / B laranja, linhas por confiança, **um anel
magenta em cada faísca órfã**, uma de cada lado do corpo); clicar a faísca esquerda depois a
direita **funde as duas numa linha âmbar**; o **Add** faz a faísca **atravessar** (sem a
correção, ela pisca de um lado ao outro no quadro 8). Os gestos: mesmo traço = orfana · vazio =
desmarca.

⚠️ **1ª rodada do S2 (Enio) achou DOIS defeitos, os dois corrigidos (`29e2af6a5`):** as
faíscas estavam em **±5** e a câmera padrão mostra só **±3** ⇒ a demonstração inteira ficava
**fora da tela** (só se via o corpo-demo + o gizmo do objeto, cuja caixa larga denunciava o
conteúdo off-screen); e o **gizmo do objeto roubava o clique** de re-par (`on_canvas` falso
sobre a caixa dele). Faíscas → ±2 (órfão agora pela diferença de FORMA, gate confirma) + Pairs
entra no `suppress_gizmo`. **Re-smoke pendente.**
