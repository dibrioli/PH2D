# HANDOFF DE INTEGRAÇÃO — `line/Vector`, a W4c.1 (a CAMADA NUMÉRICA)

**Status:** FECHADO 2026-08-06 · no `main` em `0015befbf` (o commit que trouxe este arquivo).

> Para o **agente integrador**. Branch `line/Vector`, HEAD **`7c2ab764c`**, base `main` (a linha
> estava em dia: `git rebase main` disse *"up to date"* no início da jornada).
> 4 commits desde `e0f44d1b2`. **Pendente de smoke** — integrar não é aprovar.

---

## 1. O que entra

A **W4c.1** do [`PLANO_UI_UX_padrao_figma.md`](../Estudos/PLANO_UI_UX_padrao_figma.md) §W4c: *a escala do
design system passa a ser autorável*, no molde exacto da camada de cor que a W4b deixou.

O artista abre o painel de Tokens (`T` ou o pill **TOK**), rola até **Scale (px)**, e cada token de
`spacing.*` / `radius.*` / `stroke.*` tem uma linha: **um campo numérico, um elo, um Reset**. O
valor sobrevive ao arquivo.

⚠️ **E ele NÃO move o app ainda** — essa é a fronteira da wave, não um defeito. O app lê a escala
por um caminho `const` (compile-time), e trocar os 15 sítios por leitura viva é a **W4c.2**. O
roteiro do smoke tem um passo inteiro a dizer isto, porque sem ele o `=59` reportaria a fronteira
como bug.

### A medição que reordenou a wave (do handoff anterior, e ela decidiu o desenho)

A parede que segurou os tokens de escala **nunca foi de performance**: `ColorToken::resolve` já paga
um lookup thread-local **mais uma varredura LINEAR comparando STRINGS** sobre ~350 folhas, por
chamada, por widget, por frame — e o app entrega 60 fps. A parede é **contexto de compilação**
(`const PAD: f32 = Spacing::Sm.px();`). Logo: `px()` fica `const fn` e continua a valer a FÁBRICA, e
nasce a irmã VIVA ao lado.

---

## 2. Foundational tocado, e por quê

| Arquivo | O quê | Por quê |
|---|---|---|
| `ph2d-tokens/src/alias_walk.rs` | **NOVO** — o kernel do ciclo, genérico | A pergunta *"este elo fecha um laço?"* é sobre o GRAFO e não sabe o que um slot vale. A camada de cor passa a **delegar**; duas cópias seriam a segunda a esquecer o auto-alias. |
| `ph2d-tokens/src/num.rs` | **NOVO** — `NumToken` + `px_live` | A identidade de um token numérico, e o acessor vivo. |
| `ph2d-tokens/src/num_overrides.rs` | **NOVO** — a camada | Irmã da `overrides.rs`, arquivo próprio (regra B'). |
| `ph2d-tokens/src/overrides.rs` | `closes_a_loop` delega | *Pure code motion* — os 361 gates de cor são o oráculo, e ficaram verdes. |
| `ph2d-editor-core/src/ids/chrome/tokens.rs` | +3 fns de id | **Append-only**, e os ids são **hash de string** ⇒ sem contador a colidir com outra linha. |
| `ph2d-i18n/src/lib.rs` | +1 chave (`panel.tokens.numeric`) | Append. |

**Nada mais** de foundational. `cargo check --workspace` limpo.

---

## 3. ⚠️ COLISÃO: `PROJECT_SCHEMA` 57 → **58**, e o valor é PROVISÓRIO

**Conte-o contra o `main` do dia da integração** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
São **DOIS** sítios, e os dois têm de andar juntos:

- `shells/desktop/src/project.rs:263` — a const;
- `shells/desktop/src/project_schema_tests.rs` — a tripla pinada, hoje **`(58, 13, 14)`**.
  `FLIP_SCHEMA` (13) e `VEC_SCENE_SCHEMA` (14) **não se movem** por esta linha.

⚠️ **O modo de falha desta colisão é MUDO, e o repo já o pagou em 01/08:** se outra linha escrever o
mesmo `58`, o `project.rs` **não conflita** (o literal é o mesmo dos dois lados, e o git não sabe o
que o número significa) — o bump de um dos dois evapora com a suíte verde. **Quem denuncia é o
conflito no `project_schema_tests.rs` ao lado.** Se ele conflitar, o valor certo não está em nenhum
dos dois lados: some os degraus.

**O que o bump paga:** o `SavedValue` ganhou a variante **`Number(f32)`**. Apender variante **não
move** `Literal`(0) nem `Alias`(1) ⇒ **todo arquivo já salvo continua a ler**; o bump é pelo caminho
**INVERSO** (um build antigo a ler um arquivo novo bateria num índice de variante que não tem), o
mesmo raciocínio do `JointKind::Weld` (v28) e do `Cap::Square` (v48).

⚠️ **UMA lista para as duas famílias.** O autorado numérico viaja na MESMA `ProjectFile.tokens`, e a
**CHAVE** diz de que família a entrada é (`"accent"` × `"spacing.md"`). Não é economia: é a forma que
o DTCG (W4c.5) fala, e duas listas seriam duas respostas a *"que tokens o artista autorou"*. Isto só
é seguro porque as chaves são **provavelmente disjuntas** — e há gate a afirmá-lo
(`no_key_is_claimed_by_both_families`), sem o qual o load teria de escolher um dono em silêncio.

---

## 4. Ids / consts / variants novos (para o integrador detectar colisão)

| Novo | Valor | Nota |
|---|---|---|
| `tokens_num_chip_id(row)` | hash de `"tokens.num.chip.{row}"` | **Hash de string, não contador** ⇒ nenhuma outra linha pode reclamar o mesmo número. Cobertos pelo `node_id_collisions`. |
| `tokens_num_reset_id(row)` | hash de `"tokens.num.reset.{row}"` | idem |
| `tokens_num_link_id(row)` | hash de `"tokens.num.link.{row}"` | idem |
| `SavedValue::Number(f32)` | variante **apendada** (índice 2) | ver §3 |
| `TokensIntent::{NumReset,NumSet,NumLink}` | variantes apendadas | `TokensIntent` perdeu o `Eq` (o `f32` do `NumSet`); `PartialEq` fica. |
| `panel.tokens.numeric` | chave i18n | append |

**Nenhum ADR novo.** **Nenhuma crate nova.** **Nenhuma dep externa nova.** **Zero `Cargo.toml`
tocado.**

**Contrato congelado: INTACTO**, e conferido por gate (não por auto-relato) —
`architecture_tool_contract_surface`, `architecture_contract_surface` (nodegraph),
`architecture_vector_contract_surface`, `architecture_adr_numbers_are_unique`: **4/4 verdes**.

---

## 5. O que só o `ship.sh` pega, e o que rodei

Rodado nesta worktree, 1× sobre o diff acumulado:

- **`scripts/nextest-impacted.sh` (BASE=main): 8834 testes, 8834 passaram**, 764 skipped.
- `cargo clippy --all-targets` nas 5 crates tocadas: **limpo**.
- `cargo fmt --all -- --check`: **limpo**. `cargo check --workspace`: **limpo**.
- LOC caps: `architecture_workspace_file_loc_cap` **e** o `file_loc_caps` da shell (são gates
  DIFERENTES) — verdes. `no_tofu_glyphs`, `node_id_collisions`,
  `architecture_panel_wiring_parity`, `no_magic_numeric` — verdes.
- **`design_token_sync` 9/9** — o oráculo que o handoff da wave nomeia: ele mede a tabela GERADA, e
  é ele que prova que a camada nova é **inerte enquanto ninguém autora**.

⚠️ **O que o fechamento PEGOU, e é a família que só a varredura impactada alcança** (a mesma que
physics, motion-value e Vector já documentaram):

1. o pin da tripla do schema não tinha sido atualizado — o gate fez o que existe para fazer;
2. `no_magic_numeric` (mora na `ph2d-editor-core/tests/`) reprovou o `1000.0` do `PX_DRAG_MAX`;
   ⚠️ o marcador `LITERAL-PX-OK` tem de estar **NA** linha (o rustfmt já reflowou um para fora antes
   neste repo) — conferido depois do `fmt`;
3. `TokenFamily` era `pub` com **zero** consumidores fora da crate — superfície pública à espera de
   um chamador; virou `pub(crate)`.

**Não roda aqui:** o CI cross-OS (replay-hash / matriz 3-OS) e o `ship.sh` completo (machete, deny,
audit, typos) — são do integrador.

---

## 6. Auditoria (DIRETIVA §3)

```
LENTE:  wiring (a costura painel↔shell)
CLAIM:  os três controlos da linha numérica (chip, elo, Reset) são pintados,
        REGISTADOS e ROTEADOS — nenhum nasce morto sob o mouse.
TRAÇO:  paint_num.rs:93 (chip, hit_index.register) → populate.rs:57 (register
        NumberInput + set_number_range) → event.rs:36 (ValueChanged →
        num_row_of) → push_intent(NumSet) → tokens_bridge.rs:131 →
        set_num_override → num_overrides.rs:191.
ASSERÇÃO-VERMELHA: seam_tokens_num::every_numeric_token_gets_a_row_whose_chip_
        is_a_number_input_with_a_range + a_chip_edit_names_the_row_and_the_number.
        Mutações M7 (chip como Button) e M8 (sem faixa) sangram as duas.
NÃO-CHECADO-PELA-COMPILAÇÃO: registar como Button compila; esquecer a faixa
        compila (e o chip vira interruptor min↔max SÓ no arrasto, com a
        digitação a funcionar); rotear o índice errado compila.
LOC LIDAS: ~1400 (a crate de tokens inteira, o painel inteiro, a ponte, o
        project_tokens, e o mold de cor que serviu de referência).
```

```
LENTE:  correção (a lei da camada)
CLAIM:  a camada é INERTE enquanto ninguém autora, e a chave é o par
        (modo, token) — autorar num modo não move os outros três.
TRAÇO:  num.rs::px → num_overrides::resolved_num_override (early-out numa
        leitura de bool no `ANY`) → factory_px → o `px()` const de cada família
        → crate::generated::* (a tabela do tokens.json).
ASSERÇÃO-VERMELHA: num_tests::an_empty_layer_reads_the_factory_bit_for_bit_in_
        every_mode (comparação por BITS, não `==`: `NaN != NaN` e `-0.0 == 0.0`
        dariam verde por vácuo) + o design_token_sync que já existia.
        Mutações M2 (o slot ignora o modo) e M4 (o px não consulta a camada)
        sangram.
NÃO-CHECADO-PELA-COMPILAÇÃO: ler a fábrica ignorando o modo compila; consultar
        a camada de COR a partir da numérica compila (os dois `ANY` são
        distintos, e há gate: the_two_layers_do_not_share_their_fast_path_flag).
LOC LIDAS: idem.
```

**Mutações: 13 escritas, 13 SANGRAM.** M1 `is_a_length` sempre true · M2 o slot ignora o modo ·
M3 o kernel do ciclo compara DEPOIS do salto (o auto-alias escapa) · M4 `px` não consulta a camada ·
M5 uma chave reclamada pelas duas famílias · M6 a contagem soma só as cores · M7 o chip como Button ·
M8 o chip sem faixa · M9 o elo ignora a família · M10 a ponte escreve sempre · M11 o `ResetAll`
deixa a escala de pé · M12 o load pula a família vazia · M13 o load não roteia por família.
Restauradas por `cp` do backup + `touch` (`git status` vazio depois).

⚠️ **Um erro meu, registado:** o 1º gate do `alias_walk` cravava `Some(1)` no braço da casa dos
pombos e falhou **sobre produto correcto** — naquele braço o token devolvido é *onde a caminhada
parou*, não *onde o laço fecha*. A expectativa é que estava errada; a diferença entre os dois braços
ficou escrita no doc, porque é ela que decide se o produto pode construir uma frase sobre o valor.

---

## 7. Mudanças de comportamento (nomeadas)

1. **`Reset This Mode` passa a limpar as DUAS famílias** do modo vigente. Deixar a escala de pé
   depois de um reset que se anuncia total é a metade que ninguém procura.
2. **O readout `N authored` soma as duas famílias.** Sem isso, um modo com a escala inteira
   re-vestida diria *"0 authored"* e **não ofereceria o botão que a desfaz** — trabalho preso sem
   gesto que o solte (gate: `the_reset_all_appears_when_only_a_numeric_token_is_authored`).
3. **Um projeto salvo por este build não abre num build anterior** (§3). O contrário abre.

---

## 8. O que smoke-testar

**`env PH2D_BUILD_SMOKE=59 cargo run -p ph2d-host-desktop --release`** — a MESMA cena da wave de
cor (é o mesmo painel; uma cena nova abriria o mesmo painel e diria *"agora role"*).

⚠️ **A cena imprime o número que a torna válida:**
`[tokens] painel ABERTO: N tokens de cor + 21 de escala (px) …`. **Se o 21 não aparecer, PARE** — a
tabela não chegou e o resto do roteiro não diz nada.

O roteiro impresso tem os passos; os desta wave são o **8** (a escala), o **9** (⚠️ *a escala ainda
não move o app, e isso é a wave*) e o **10** (o arquivo, agora com um número junto). Vale a pena ler
o 9 antes de julgar o 8.

Para ler o roteiro sem abrir a janela:
`cargo test -p ph2d-host-desktop --bins show_the_script -- --ignored --nocapture`

---

## 9. Aberto, com o preço ao lado

- **W4c.2 — os 15 sítios `const`.** É onde está o trabalho real, e é mecânico.
  ⚠️ `TOOL_RAIL_WIDTH_PX` **cascateia**: ele alimenta dois `pub const RAIL_W` (em
  `screens/layout.rs` e `screens/hero/style.rs`), então `const` → `fn(theme)` arrasta os
  consumidores. Não é surpresa — é o churn que o plano previu —, mas é o sítio a orçar primeiro.
- ⚠️ **O TETO de um valor autorado ainda não existe, de propósito**, e a W4c.2 é quem tem de o
  medir. A porta recusa o que não é um comprimento (não-finito, negativo) e **não inventa um
  máximo** (§0: um cap sem medição é um palpite). O que aquela wave tem de medir antes de ligar a
  leitura viva: **o painel de Tokens desenha-se a si mesmo com estes tokens**, então um valor
  absurdo pode empurrar para fora da tela o *Reset* que o desfaria. Hoje é inofensivo porque
  ninguém lê; no dia em que alguém ler, é um brick com escape só pelo arquivo.
- **`Spacing::px_live` / `Radius::px_live` / `StrokeToken::px_live` não têm chamador de produção** —
  são a API que a W4c.2 consome, um delegate de uma linha cada, com gate a provar que não podem
  divergir do `NumToken::px`. Declarado, não esquecido.
- **`Motion` (ms), `Density` e `chrome.*` ficaram FORA da família**, cada um com o motivo escrito no
  `num.rs`: outra unidade · já é uma escolha do artista · sem identidade de token para o arquivo
  guardar. Nenhum é adiamento por preguiça; os três mudam de forma antes de entrar.
- **W4c.3 (math / `TokenValue::Expr`)** entra como uma **variante nova no `NumValue`**, nunca num
  mapa ao lado — o doc dele já diz isso no sítio.

---

## 10. Ordem de trabalho, se a linha continuar

O handoff da linha ([`HANDOFF_line_Vector_tokens_2026-08-06.md`](HANDOFF_line_Vector_tokens_2026-08-06.md))
mantém a fila: **W4c.2** (os 15 sítios) → **W4c.3** (math) → **W4c.4** (escala) → **W4c.5** (DTCG).
A W4c.4 deve custar **fiação e mais nada** — se custar mais, o (1) foi feito estreito demais, e o
teste é: a família já cobre as três escalas de px, então acrescentar uma quarta é uma entrada na
macro `num_tokens!` e um `match` de três braços.
