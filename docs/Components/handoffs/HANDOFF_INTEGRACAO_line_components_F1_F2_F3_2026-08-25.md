# Handoff de integração — `line/components`, 2026-08-25 (F1 completa + F2 + **F3**)

> DIRETRIZ §1.5.9. Sucessor do
> [handoff de 24/08](HANDOFF_INTEGRACAO_line_components_F0_F1parcial_2026-08-24.md), que foi escrito
> **a meio da F1** e nomeava uma assimetria visível. ⭐ **Ela FECHOU** — leia o §0.

---

## §0 ⭐ A condição que o handoff anterior levantava está RESOLVIDA

Aquele handoff dizia: *«o artista passaria a ver a física a aguentar um rename e a animação não, sem
nada que explique a diferença»*, e recomendava **esperar a F1 fechar**. Ela fechou.

| | antes da linha | **hoje** |
|---|---|---|
| renomear um corpo com junta | junta desliga | ✅ aguenta |
| renomear um objeto **animado** | binding desliga | ✅ aguenta (F1 passo 5b) |
| corte da `Sprite` (20 → 13 campos) | — | ✅ com migração de formato |
| custo de um passo de undo | `O(mundo)` | ✅ `O(edição)` — medido |
| Inspector de um objeto novo | **doze seções de zeros** | ✅ **duas** (Transform + Name) |

⛔ **Uma pendência ficou, e ela é do SHIP, não da integração:** o `physics_ecs_c9` está **por
re-capturar** — o `deterministic_hash` muda de valor com o snapshot v2. É o item mais provável de
reprovar a matriz 3-OS.

---

## §1 Identidade

| | |
|---|---|
| branch | `line/components` |
| HEAD | `963dd0e1bd22bff87733bfd4a22a88bea610ee29` |
| merge-base com `main` | `0f5ce8040c07742dc1bf7a5a2c5a7e8c2f41b6cb` |
| commits | **15** |
| arquivos tocados | **143** (7 269 inserções, 1 436 remoções) |

Ordem dos commits (linear; ⚠️ **não reordene** — a F2 depende do `StableId` da F1, e a poda da F3
depende de a porta da F3 existir):

```
551afcdf1  docs   a base do plano RE-MEDIDA depois da integração
acb31f9e7  F1.5b  a timeline aponta por IDENTIDADE
e5f96566b  F1.6   os TRÊS componentes do corte da Sprite nascem (aditivo)
91815ac7c  F1.6   os SETE campos SAEM da Sprite; 20 → 13, VERSION 5
2e2e2e908  F1.6   o degrau 97→98, os DOIS amendments — F1 FECHA
583ffc437  F2     a captura do desfazer passa a ser INCREMENTAL
233819b3c  F2     a instrumentação (`PH2D_UNDO_LOG`) + o LINT
5554f61af  F2     varredura em DUAS PASSAGENS + a paleta de componentes
3d9a6a783  docs   F2 FECHA com a medição
be883fee0  F3     o `+` do Inspector abre a paleta e ANEXA
4fbd06c27  F3     o objeto VAZIO na raiz + o censo de alcance
ced3fcef3  F3     o Inspector pinta por PRESENÇA (a poda) + os DOIS seeds
8901e8e37  F3     a CASCATA (`requires`) + a caixa *Show all*
963dd0e1b  docs   F3 FECHA
```

---

## §2 O que MUDA para quem usa o app

1. **A Hierarquia cria objetos.** O botão `Add` do cabeçalho era pintado e registado **desde a Fase
   C.2** e **nada consumia o clique**. Hoje ele faz um objeto vazio na raiz.
2. **O Inspector mostra o que o objeto TEM.** Oito seções deixaram de aparecer sobre entidades que
   não têm os componentes delas.
3. **Um `+` no cabeçalho do Inspector** abre a paleta de componentes — o mesmo modal do *Add Node*
   do Motion —, filtrada pelo tipo do objeto, com busca e uma caixa *Show all*.
4. **Anexar um componente traz o que ele não funciona sem**, e o rótulo **diz** o que vem junto
   antes do clique.

---

## §3 ⚠️ As CINCO coisas que uma leitura rápida do diff entende ao contrário

1. **`PlayerFieldEdit::Add` não foi «removido por limpeza».** Ele era a porta da §14, e a porta
   ficou fechada sobre a própria chave quando a seção passou a exigir o componente. O que ele fazia
   de insubstituível — semear a `float_height` da forma do collider — **mudou de casa** para
   `component_seed`, com gate. Procurar o comportamento no diff do `inspector_player.rs` e concluir
   que ele desapareceu é o erro fácil.

2. **A §14 deixou de recusar `BodyKind::Static`, e isso é uma REVERSÃO deliberada.** A regra antiga
   era o critério de *oferecer o botão*; com o botão na paleta, mantê-la faria o artista anexar pelo
   `+` e **nada aparecer**. A física continua verdadeira — ela é assunto da §11.

3. **`INSP_ANCHOR_ADD` e `INSP_ANIM_ADD` NÃO foram esquecidos.** O plano listava-os entre as cinco
   portas a subsumir; medido, eles acrescentam uma **linha** (uma âncora, uma tag) dentro de um
   componente **já presente** — não são portas de componente. `INSP_PHYS_ADD` sobrevive como atalho
   da §11 quando ela está visível (o caso do rig).

4. **A §4 Sprite Sheet ficou por gatear, de propósito.** Ela é sub-seção do `sprite_info` e hospeda
   o **Flip X / Flip Y**, que são campos da `Sprite` base — gateá-la no `SpriteGrid` tornaria os dois
   inalcançáveis.

5. **O `save.rs` encolheu de 704 para 345 linhas e nenhuma lógica saiu.** O módulo de testes mudou-se
   para `save_tests.rs` (o precedente da própria pasta). Ele estava **acima do teto de 700 desde a
   F2** e ninguém tinha corrido o gate.

---

## §4 ⚠️ As TRÊS premissas do plano que a implementação REFUTOU

| O plano dizia | A medição diz |
|---|---|
| «as CINCO portas de hoje são subsumidas» | **duas** delas não são portas de componente (§3.3) |
| a §12/§11/§11-física gateiam num componente cada | ⭐ **cada uma tem DUAS metades**, e gatear numa só apagava a UI do outro lado (`AnchorMount` · `SpriteAnimator` · `Collider` sozinho) |
| o `requires` é *wiring* | ⛔ **é load-bearing**: sem ele a poda abre um buraco próprio — anexar `PlatformPlayer` a um objeto sem corpo põe o componente lá e a §14 **não aparece** |

---

## §5 Superfície de colisão (o que outra linha pode tocar)

| Sítio | O que esta linha faz lá |
|---|---|
| `ph2d-component-desc/src/lib.rs` | campo `requires` + construtor `authored_requiring` (append-only) |
| `catalog/physics.rs` | duas entradas passam de `p(...)` para `pr(...)` |
| `ph2d-editor-core/src/action_bus.rs` | `HierAddRoot` · `InspectorAddComponentRequested`; ⚠️ `PlayerFieldEdit::Add` **removido** |
| `widget/command_palette.rs` | a BANDA saiu para `command_palette/header.rs`; `PaletteModel` ganha `toggle` (**6** sítios de construção) |
| `interaction/state/*` | 3 campos/ops novos no store |
| `ph2d-panel-inspector/src/event.rs` | duas funções irmãs novas; a catraca de LOC **desce** a 276 |
| `ph2d-ecs/src/scene/save.rs` | testes saem para `save_tests.rs` |
| `shells/desktop/src/render_loop/*` | 8 builders ganham o guarda de presença; `hierarchy.rs` perde 26 linhas para `hierarchy_add_root.rs` |
| **`tests/architecture_panel_loc_cap.rs`** | ⚠️ catraca **descida** — uma linha que a suba colide |

⚠️ **Números que SOMAM entre linhas** (CLAUDE.md §5.0): `PROJECT_SCHEMA` já está em **98** desde a
F1.6; **o próximo degrau é o 99**. Esta fase **não move o schema** (o `requires` e o `toggle` não são
formato).

---

## §6 Gate de fecho

```
cargo fmt --all                                      ✓
cargo check --workspace --all-targets                ✓  0 erros, 0 avisos
cargo test -p ph2d-host-desktop --bins               ✓  3622 passaram, 0 falharam, 244 ignorados
cargo test -p ph2d-editor-core                       ✓
cargo test -p ph2d-panel-inspector                   ✓
cargo test -p ph2d-component-desc                    ✓
cargo test -p ph2d-ecs                               ✓
```

Provas de mutação (controle verde antes, restore com `touch`): tirar o guarda de presença da §7 ⇒ a
lei RED · tirar o braço do `Collider` da tabela de seeds ⇒ o gate do seed RED.

---

## §7 O que fica ABERTO (F4–F8 do plano, e uma pendência de ship)

- ⛔ **`physics_ecs_c9` por re-capturar** — vide §0.
- **F4** (núcleo de instância) em diante, no [plano vivo](../05_plano_de_implementacao.md).
- ⏳ A §4 Sprite Sheet (§3.4) — se alguém quiser gateá-la, o Flip X/Y tem de sair de lá primeiro.
- ⏳ **12 categorias de componente mapeiam em 7 tokens `NodeCat*`**, logo pares de categorias
  partilham tinta na paleta. ⛔ A cura é acrescentar tokens ao `ph2d-tokens` (§7 do CLAUDE.md), nunca
  hex — e é decisão de design, não de quem ligou o botão.
