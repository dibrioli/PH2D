# HANDOFF de continuação — line/UIUX, F46 (02/11): lados CAD pela curva

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `1153ed3853` (F46). Linha reaberta em 02/11
sobre o main `3df5894489` (F40–F45 já integradas: [handoff](HANDOFF_INTEGRACAO_line_UIUX_2026-11-02_F40_A_F45.md)).
**Nada integrado, nada enviado.** Antes de ler: `cd` + `pwd` + `git branch --show-current`.
Formato anterior: [F44c](HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F44c.md).

## 1 — Defeito curado (era o ABERTO 1 do handoff F40–F45)

Report do dono 01–02/11: «linha residual» depois de mudar o tipo de alça / Smooth no Vector
(fotos `img_F45/dono_linha_residual_1.png` e `_2.png`).

**Mecanismo (medido).** NÃO era o fio do caminho (`path_wire`, que já seguia a curva). Era a cor CAD dos lados,
`crates/ph2d-app-vec/src/cad_tool_plano.rs` (só formas com regras CAD), pintada com `line_to` entre âncoras:
- cor medida na foto (179,204,255) = role `info` do tema escuro (`ph2d-tokens/src/derive.rs`, `Rgb::new(0.7,0.8,1.0)`) = `ColorToken::Info`;
- largura ~1,5 px = `TRACO_PX` de `cad_tool_pinta.rs`.
Depois do Smooth a forma curva e as rectas ficavam por cima da arte. As duas sondas da F45 não reproduziam porque a
cena 47 não tinha regra CAD.

**Hipóteses descartadas no caminho (não repetir):** memo do `fx_live` (a chave leva o `VecPath` inteiro) · `align_live` ·
`paint_dilate` (chaves corretas) · contorno de hover (`hover_outline`: recalculado a cada quadro e não desenhado sobre o
seleccionado) · redesenho parado (o app redesenha contínuo).

## 2 — Cura, num bloco (a família inteira)

Módulo novo `crates/ph2d-app-vec/src/cad_tool_lado.rs` (+ `cad_tool_lado_tests.rs`, registados em `lib.rs`):
- `Curva = [[f64;2];4]`, `Marca { Ponto, Lado }`;
- `cubica` / `no_ecra`: o lado no ecrã é a cúbica do segmento (âncora, alça de saída, alça de entrada, âncora), nos dois
  sentidos; recta SÓ se os pontos não são vizinhos;
- `distancia` (kurbo `nearest`), `meio_e_normal`, `caixa`.

`cad_tool_plano.rs`: `Plano.lados: Vec<(Curva, ColorToken)>`, `Plano.segmentos: Vec<(SegCad, Curva)>`,
`Plano.ligados: BTreeMap<RegraId, Vec<Marca>>` montado pela `Lei` (`marcas_da_lei`), não por pares de pontos. `acerta()`
mede à curva; os realces (hover, escolha, regra) acendem a curva; o selo pousa no meio da curva com a normal dela.
`cad_tool_pinta.rs` pinta a curva. **Distância e Coincidente continuam rectas (são medida). Solver intocado.**

## 3 — Cena nova `PH2D_OBJECT_MODE_SMOKE=48`

`vector_mode_fill_stroke.rs` `com_regra_e_um_ponto_puxado` (`qual == 3`): a cena 47 com o canto de baixo à direita
puxado 0,3 e o de baixo à esquerda preso por `Lei::Fixo`. Tabela `vector_mode_scenes.rs` (9..=48 / 43..=48 / 31..=34);
rota `vector_mode_route.rs`. **Próxima cena: 49.** Fotos: `img_F46/lados_cad_antes_reta.png` (cura desligada, reproduz o
report) · `img_F46/lados_cad_depois_curva.png` · `img_F46/cena48.png`.

## 4 — Prova de fecho (02/11)

- `cargo fmt --all`; `clippy -p ph2d-app-vec --all-targets -D warnings` verde.
- Testes por nome verdes: `cad_tool` (16), `a_cena_48`, `a_tabela_das_cenas`, `as_faixas_de_estagios`. Binário `smoke` compilado.
- Gates novos: `a_curved_side_is_painted_and_hit_along_the_curve_not_the_chord`,
  `a_rule_on_a_curved_side_lights_the_curve_and_its_badge_sits_by_it` (a régua do meio é a kurbo, não a função medida),
  `a_straight_side_keeps_the_chord_midpoint_normal_and_distance`, `a_cena_48_comeca_no_fill_and_stroke`.
- **Mutação 3/3 sangrou:** cúbica→corda · sentido inverso→corda · meio da curva→meio da corda (a 1.ª versão do teste caía
  na linha de controlo por usar a função mutada; corrigido com a kurbo como régua).
- **Suíte completa NÃO corrida** (é da integração).
- ⚠️ `cargo fmt --all` também reformata 3 ficheiros que vêm do main por formatar
  (`crates/ph2d-panel-pixel-preview/src/paint.rs`, `crates/ph2d-panel-registry-init/tests/it/the_vector_control_bar.rs`,
  `crates/ph2d-vec-entities/src/connector_handles.rs`): foram REVERTIDOS, não são desta linha — o integrador corrige.

## 5 — Formato e contadores (DELTA)

Nenhum: `PROJECT_SCHEMA`, `VEC_SCENE`, `DOC_VERSION` intocados. Sem ADR. Cenas `PH2D_OBJECT_MODE_SMOKE`: `1|2|4|6..48`.
Para o integrador: a linha UI/UX do CLAUDE.md §5.1 passa os smokes a `6..48` e o link do handoff.
Superfície de colisão: só `crates/ph2d-app-vec` (7 ficheiros de `cad_tool_*`, `lib.rs`, `vector_mode_{fill_stroke,route,scenes}.rs`) e `img_F46/`.

## 6 — Smoke do dono (APROVADO em 02/11)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=48 ./target/smoke/ph2d-host-desktop`
2. Na barra flutuante sobre a forma, 2.ª fileira, 1.º botão (Smooth).
3. Deve acontecer: as linhas azul-claras seguem a borda curva.
4. Deu errado se: ficarem rectas por dentro da forma.

## 7 — Observação de outra linha (não mexida)

`PH2D_CONSTRAINT_SMOKE=7` («Caixa com tampa») imprime «Edit + Regras», mas na foto fica no modo Object, sem a tool
Regras: a cena ensina o contrário do roteiro (linha Constraints).

## 8 — ABERTO (por ordem)

1. ⭐ **NOVO pedido do dono, 02/11:** no Stroke do Vector, uma opção para o traço ficar POR CIMA dos Filtros. Hoje a cor
   do stroke é sempre afetada pelos filtros (FX raster da forma, `fx_live`: a imagem de FX substitui o desenho inteiro da
   forma em `crates/ph2d-vec-render/src/dispatch.rs`). O dono quer o stroke desenhado por cima do fill já filtrado.
   Medir os leitores antes de escolher onde mora a opção (campo da base × por camada de traço; Auto Key; schema).
2. ~~Weight cortado («16» → «1(»)~~ — curado na integração de 02/11.
3. Esc não cancela o arrasto de linha de painel (família `panel_row_drag`, pré-existente).
4. Perfil de largura não é por camada de traço.
5. Ids de Position fora do censo `seam`.
6. `migra_pilha_v23` sem chamadores.
7. Com nada seleccionado, o ⚙ do traço de base fica activo e o do Fill apagado (F44; confirmar com o dono).
8. ⛔ Janela baixa: a régua de ferramentas FLUTUANTE regista botões FORA da janela (430 px de altura, botões até
   `y≈774`: inalcançáveis). Falta recorte/rolagem pela altura da região. Sem gate ainda: tem de medir no ECRÃ, não no
   mundo, e reprovar antes da cura. Detalhe no §8 do handoff F40–F45.

## 9 — Prompt de continuação

```
/pd-linha-assumir line/UIUX — Worktrees/line-UIUX
Próximo item: ABERTO 1 (stroke do Vector por cima dos Filtros, opção no Stroke).
Handoff: docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F46.md
Meça os leitores de fx_live/dispatch ANTES de escolher onde mora a opção. Só tokens e i18n, app só em inglês,
Auto Key em todo campo. Cena fotografada antes do smoke (próxima = 49); PARE no meu smoke. Sem integrar nem push.
```
