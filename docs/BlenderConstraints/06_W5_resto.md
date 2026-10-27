# W5 — o resto: valor que segue valor, *behaviours* de um clique, layout de HUD (27/10/2026)

> Plano: [`01_plano.md`](01_plano.md) §7 (linha W5). Anterior: [`05_W5_diferenciais.md`](05_W5_diferenciais.md) (a cola,
> FECHADA e aprovada — não se toca). Escrito ANTES do código; cada número abaixo foi LIDO no código desta árvore
> (merge-base `c3662cd8e4`). Cena de smoke seguinte: **9**.

## 0. O que o artista vai conseguir fazer

1. **Valor segue valor, com mola** — o ponteiro de um mostrador acompanha a vida do herói e balança antes de parar; a
   luz acende quando a porta abre; a opacidade de um aviso segue a distância do inimigo. Qualquer número de um
   objecto alimenta qualquer número de outro, com faixa → faixa e mola opcional.
2. ***Behaviours* de um clique** — «Tremer», «Flutuar», «Orbitar», «Atrair»: um clique num objecto qualquer (Sprite,
   Vetor, Flip…) e ele mexe-se, sem montar grafo, com o MESMO movimento do nó do Motion de igual nome.
3. **HUD que se arruma sozinho** — o ícone da moeda fica sempre ao lado do número da pontuação quando ele cresce de
   `99` para `12 345`; quando o ecrã estreita, a pontuação encolhe antes de invadir o minimapa.

## 1. Medido antes de desenhar — o que a composição JÁ exprime

### 1.1 Valor → valor: **não exprime** (faltam 3 peças, a lei existe)

| Pergunta | Resposta no código |
|---|---|
| O «drive» da W3 lê qualquer propriedade? | Não. É `Kind::Transformation` (`ph2d-constraints/src/poderosas.rs:71`): só `Canal` X·Y·Rot·Sx·Sy da POSE do alvo → `Canal` da pose do dono; mapa linear faixa→faixa |
| A Spring da W3 actua num escalar qualquer? | Não. `Kind::Spring` (`poderosas.rs:349`) atrasa a pose animada do próprio dono; `Simulador` preso a `[f32; 3]` (`ph2d-constraints-ecs/src/relogio.rs:24-40`) |
| Empilhar Transformation → Spring encadeia valores? | Não: cada item lê a pose do dono e do alvo, nunca a saída escalar de outro item |
| A LEI da mola serve? | **Sim**: `ph2d-node-motion-spring/src/law.rs:124` `passo(valor, vel, alvo, tension, friction, dt)` é escalar e pura |
| Há porta por nome para LER/ESCREVER qualquer propriedade? | **Sim**: `ph2d-ecs/src/scene/property.rs:90` `resolve` · `:147` `get` · catálogo `ph2d-script/src/api_table.rs:342` `property_names()` (campos de componente; a pose vem à parte) |
| Há porta de ESCRITA em preview por campo? | **Sim**: `ph2d-preview-drive/src/fields.rs:19` `FieldRef` / `:60` `FieldDrive` — os canais dos constraints já a usam (`ph2d-app-constraints/src/canais.rs:85`) |
| A ordem entre entidades cobre «A lê um campo de B»? | Sim sem mudança: a aresta vem de `it.target` (`ordem.rs:100`), não do que se lê |

Faltam: (a) **fonte por nome** num item de constraint; (b) **mola sobre N escalares** (o `Simulador` generalizado
por `const N`, `relogio.rs:24-40, 243-281`); (c) **destino por nome** pelo `FieldDrive` (pose → `Driver::ConstraintPose`).

### 1.2 *Behaviours*: os nós existem, **nenhum chega a um objecto qualquer**

| Nó | Onde | Actua sobre | Função pura de (t, seed, params)? |
|---|---|---|---|
| `motion.wiggle` | `ph2d-node-motion-wiggle/src/lib.rs:46` (`eval` :165) | colunas de instâncias do Motion | sim (fBm por hash inteiro, sem trig) |
| `motion.oscillator` | `ph2d-node-motion-oscillator/src/lib.rs` | idem | sim |
| `motion.orbit` | `ph2d-node-motion-orbit/src/lib.rs:68` | idem | sim |
| `motion.falloff` | `ph2d-node-motion-falloff/src/lib.rs:32` | produz o PESO dos outros (coluna `falloff`) | sim — não anima |
| `force.attractor` | `ph2d-node-force-attractor/src/lib.rs:44` | aceleração de partículas | não — é força, precisa de estado |

A expressão da timeline (`TargetBinding.expr`, `ph2d-timeline/src/binding.rs:104`, `wiggle()` em
`ph2d-expr-parse/src/lib.rs:462`) liga uma função do tempo a uma propriedade, mas: só a pose de Sprite
(`prop.rs:21` `ALL`), **nenhum painel emite** `SetBindingExpr` (`intent.rs:274` sem produtor de UI), e o plano já
recusou a expressão como superfície do artista (`01_plano.md` §1: ilegível). Não serve de casa.

Precedente de «um clique» no app: o **Perfil** do CameraShake (`ph2d-app-components/src/shake_inspector.rs:102-126`,
`ph2d_shake::Perfil::from_tag(..).numeros()`).

### 1.3 HUD: a cerca escrita contra o Cassowary e porque a premissa dela caiu

`ph2d-ecs/src/vec_anchors.rs:8-14` (03/08) e `ph2d-hud/src/lib.rs:6-11` recusam o Cassowary: *«o caso relacional já
tem resposta — o align/distribute»*. A recusa é ARGUMENTADA, não medida, e a premissa não se confirma:

| Premissa da cerca | O código hoje |
|---|---|
| align/distribute responde «alinhe A com B» | É um gesto de UM passo, só no Quadro (`ph2d-board-edit/src/organize.rs:278, 304`); não há equivalente no vetor nem regra viva por quadro |
| O resto é aritmética fechada | Certo para A↔PAI (`VecAnchors`, `hud_anchors.rs:65-83`) e para a raiz (`ph2d-hud` `place`, oráculo Godot); **não há âncora a IRMÃO** |
| — | O texto da pontuação muda em runtime (`LabelSource::Counter`, `ph2d-ecs/src/hud.rs:76-87`); **não há largura de texto medida por quadro** |

A cerca continua certa para o que cobre (âncora ao pai, raiz do HUD: ficam procedurais). Ela **não** cobre irmãos
nem desigualdades («encolhe antes de invadir»), e é aí — e só aí — que o solver entra.

## 2. Triagem de licença (por artefacto)

| Artefacto | Versão | Licença lida | Uso |
|---|---|---|---|
| `kasuari` (`~/.cargo/registry/.../kasuari-0.4.13/Cargo.toml:47`, `LICENSE-MIT`, `LICENSE-APACHE`) | 0.4.13 | MIT OR Apache-2.0 | dependência (Cassowary incremental) |
| deps: `hashbrown` 0.17.1 · `thiserror` 2.0.21 · `portable-atomic(-util)` · `document-features` (opcionais) | — | MIT OR Apache-2.0 | transitivas — reconferir por `cargo metadata` ao entrar no lock |
| `python-kiwisolver` (`pacman -Qi`) | 1.5.1 | BSD-3-Clause | **oráculo que se CORRE** (Cassowary de referência; `python -I -c 'import kiwisolver'` = `1.5.1` ✓) |
| `gtk4` `GtkConstraintLayout` | 4.24.1 | LGPL-2.1+ | recusado como oráculo: o kiwisolver basta e corre sem ecrã |

Oráculos dos outros dois: os nós do Motion (o MESMO código, chamado — paridade bit a bit) e `law::passo`.

## 3. O desenho

### 3.1 Valor segue valor — `Kind::Valor` (append ao `Kind`, `ph2d-constraints/src/kind.rs:162`)

- Campos: `alvo: StableId` · `fonte: Prop` · `destino: Prop` · `de_faixa`/`para_faixa` + aparar/extrapolar (o mapa do
  `Transformation`, a MESMA fn) · `mola: Option<(duracao, ressalto)>` (os números da Spring, a MESMA conversão) ·
  `influence`. `Prop` = canal da pose OU `"componente.campo"`, resolvido pela porta única `scene::property`.
- Avalia: `ler(B, fonte)` → mapa → (mola) → `escrever(A, destino)`. Pose → `Driver::ConstraintPose`; campo →
  `FieldDrive` (preview, nunca documento; Ctrl+Z intocado).
- Mola: `Simulador<const N>` (o atual vira `N = 3`, mesmo bit — gate); o `Valor` usa `N = 1`. Scrub = play pelo
  mesmo STRIDE/re-sim.
- Nome desconhecido ou campo não numérico ⇒ `Inerte::Fonte`/`Inerte::Destino` com frase no cartão (nunca pânico).

### 3.2 *Behaviours* — `Kind::Comportamento { tipo, params, seed, peso_por: Option<StableId> }`

- `tipo`: **Tremer** (lei de `motion.wiggle`) · **Flutuar** (`motion.oscillator`) · **Orbitar** (`motion.orbit`, em
  volta do alvo ou de um pivô) · **Atrair** (lei de `force.attractor` integrada com o `Simulador` — a única com estado).
- `peso_por`: o **falloff** não é um *behaviour*, é o peso dele — a força cai com a distância a outro objecto, pela
  lei de `motion.falloff`.
- **Lei única, dois chamadores:** cada crate de nó expõe a sua lei pura (`pub fn`, append) e o nó e o `Comportamento`
  chamam-na; gate de paridade bit a bit contra o `eval` do nó numa instância. Nada de cópia.
- **Um clique = preset:** `Perfil` com números de fábrica (como o CameraShake), aplicado no Add Constraint; depois
  ajusta-se no cartão como qualquer constraint. Escreve a POSE (soma ao autorado, `Mix::Add`).

### 3.3 HUD que se arruma — crate nova `ph2d-hud-layout` sobre `kasuari`

- Relações entre IRMÃOS de um `UiCanvas`: `ao lado de` (esq./dir., folga) · `acima/abaixo de` · `centrado em` ·
  `não invade` (desigualdade) · `largura ≤ conteúdo` com força (`REQUIRED`/`STRONG`/`MEDIUM`/`WEAK`).
- Fica como está: âncora ao PAI (`VecAnchors`) e a raiz (`ph2d-hud::place`). O layout corre DEPOIS deles, só sobre os
  irmãos que têm relação; quem não tem relação não paga nada.
- Precisa de **largura medida do rótulo** por quadro (não existe — a ler onde o rótulo é desenhado antes de codar).
- Solver incremental: as larguras dos rótulos e a caixa do canvas entram como *edit variables*; resolve-se só quando
  mudam. Determinístico (simplex exacto, ordem de inserção fixa — gate do mesmo bit).
- Actualiza a cerca de `vec_anchors.rs`/`ph2d-hud` com o link para aqui e um ADR provisório **0194** (reconta-se).

## 4. Gates red-first (cada um visto VERMELHO antes do código)

| Peça | Gate |
|---|---|
| Valor | `a_value_follows_another_value_through_the_range` · `a_springy_value_is_the_spring_law_step_by_step` · `a_followed_value_scrubs_to_the_same_bit` · `a_followed_field_never_touches_the_document` · `an_unknown_property_is_inert_with_a_reason` · `a_value_link_survives_save_and_open` · `the_pose_spring_is_the_same_bit_after_the_generic_simulator` |
| *Behaviours* | `each_behaviour_is_its_motion_node_bit_for_bit` (×3 puros) · `attract_scrubs_to_the_same_bit` · `a_falloff_object_weighs_the_behaviour_by_distance` · `one_click_applies_the_factory_profile` |
| HUD | `the_layout_matches_kiwisolver_case_by_case` (fixture `fixtures/kiwisolver_1_5_1_hud.json`, 5 famílias: ao lado, pilha, centrado, não invade, largura com força) · `the_coin_stays_beside_a_growing_score` · `a_narrow_screen_shrinks_the_score_before_it_overlaps` · `the_layout_is_the_same_bit_every_frame` |
| Todas | registo de componente/descritor (`every_registered_component_has_a_descriptor`), gravar/abrir, mutação na lei de cada peça |

## 5. Kill (mesma rodada intercalada, `--profile smoke`, mínimo)

`0,5 ms / 1 000` por peça: 1 000 `Valor` com mola · 1 000 `Comportamento` (mistura dos 4) · 1 000 elementos de HUD
com relações (re-solve completo, o pior caso). As três variantes no MESMO processo, intercaladas.

## 6. Cena 9 — «Painel do jogo» (formas e simulação)

Um mostrador cujo ponteiro segue a vida com mola; uma moeda que Tremer e outra que Orbita um planeta, com a força a
cair pela distância a um íman; o HUD com a pontuação a crescer e a moeda colada ao lado; a janela do jogo estreita e
o número encolhe sem invadir o minimapa.

## 7. Para o dono (PRODUTO — pendente)

1. Ordem das três.
2. Onde mora cada uma na tela (proposta: Valor e *Behaviours* no **Add Constraint** do cartão Constraints; HUD como
   **ferramenta** na barra esquerda com gizmos de seta entre os elementos — o idioma das regras de desenho, 26/10).
