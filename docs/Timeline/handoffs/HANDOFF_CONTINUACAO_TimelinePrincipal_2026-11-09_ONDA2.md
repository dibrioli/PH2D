# Continuação — `line/TimelinePrincipal`, onda 2 (a fita) → onda 3 (Containers e Arrange)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commit da onda `8694c376c4` sobre `a5d54be996`.
- Plano: [`15_plano_redesenho_molde_rive.md`](../15_plano_redesenho_molde_rive.md) (versão B; «Keys | Curves» FORA; sem função nova).
  Onda anterior: [`..._ONDA1.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA1.md).
- Fotos: [`15_onda2_depois.png`](../fotos/15_onda2_depois.png) (app, `PH2D_ANIMACAO_SMOKE=1`, 1930×1040) ·
  [`15_onda2_lado_a_lado_maquete_B1.png`](../fotos/15_onda2_lado_a_lado_maquete_B1.png) (em cima o recorte da maquete B1 da onda 1b,
  em baixo o app; a captura headless nova da maquete não foi refeita).

## O que a onda 2 fez (só cartões; `PH2D_CARDS=0` intacto)

1. **Nomes em árvore** (`linhas::layout`): nos cartões entra `Linha::Objeto(entity)` — uma linha de cabeçalho por objecto,
   nome em negrito (`paint_text_title_elided`) e um losango pequeno (`OBJ_DIAMOND_H`, `Text3`) por instante com chave.
   As faixas de um objecto juntam-se sob ele pela ordem da 1.ª aparição (o smoke trazia Herói, Espada, Herói intercalados).
   A faixa diz só a propriedade (`tracks_label::propriedade_snap`), inteira, com o valor à direita como antes; o cabeçalho
   de grupo idem. Unidade «(px)», «(°)» só quando o canal tem grandeza (`tracks_value::sufixo_da_unidade`, `Field`).
   O cabeçalho é só pintura: sem alvo, `keys_in_rect` ignora-o.
2. **Barra entre chaves** (`linhas::paint_barras`, `cor_da_barra`): por baixo dos losangos, 3 px, na cor da interpolação da
   chave de saída — `Hold` tracejada, `Linear`, o resto (eased/bézier/weighted) = curva. Segue o `preview_dx` do arrasto.
3. **Tokens novos** (apelidos, `ph2d-tokens`): `TimelineInterpCurve`→`Info`, `TimelineInterpLinear`→`Accent`,
   `TimelineInterpHold`→`Text3`, `TimelineRecord`→`Danger`. `color.rs` estava em 697/700: os 4 entram com `#[doc]` na mesma
   linha e o comentário da secção encolheu (698).
4. **Auto Key**: com `snapshot.auto_key` a folha ganha borda `TimelineRecord` (2×`FIELD_BORDER`), em `paint.rs` depois da régua.
5. **Depois do fim**: o `BgScrim` de `ruler::beyond_end_shade` ganha listras a 45° (`Border`) — peça genérica nova
   `ph2d-editor-core/src/panel/cards_banda.rs::listras_em` + `LISTRA_PASSO` (append-only).
6. **Enquadrar**: `state.enquadrado: Option<usize>`; trocar `active_clip` nos cartões faz o `apply_fit` (o `F`) sozinho.
   O 1.º quadro só anota (ver divergências).

## Testes (por nome) e fecho

- Novos em `linhas_tests.rs`: `nos_cartoes_o_objeto_e_o_cabecalho_das_suas_faixas`, `a_faixa_diz_so_a_propriedade_e_o_cabecalho_o_objeto`,
  `a_barra_veste_a_interpolacao_da_chave_de_saida`.
- Testes de disposição PLANA (`linhas` ×3, `geom` ×3, `box_select` ×4) passaram a forçar o clássico
  (`force_cards_on_this_thread(Some(false))`): medem a aritmética das faixas, não a árvore.
- Resultado: `linhas` 15 ✓ · `geom` 12 ✓ · `box_select` 7 ✓ · `tracks` 10 ✓ · `summary` 12 ✓ · `interact` 12 ✓ · `key_drag` 11 ✓ ·
  `os_cartoes` 21 ✓ (inclui o gate de altura estrito, sem mudança) · `_seam` 81 ✓ · `ph2d-tokens color` 17 ✓.
- `cargo fmt --all`; clippy `-D warnings --all-targets` limpo em `ph2d-panel-timeline`, `ph2d-editor-core`, `ph2d-tokens`;
  smoke `--profile smoke` do `ph2d-host-desktop` compilado.
- Nenhum controlo mudou de sítio nesta onda (sem teste `onde_mora` novo); ids e eventos os mesmos.

## Divergências / interpretação da maquete B1

- As propriedades dizem a palavra da Timeline («Translate X»), não a do Inspector («Position X»); a unidade só aparece em
  canais `Field` com grandeza — para `TranslationX/Y`/`Rotation` embutidos não confirmei a unidade do valor mostrado.
- Cores: curva `Info`, reta `Accent` (no tema escuro ambos azuis próximos; a maquete usa ciano × violeta). Medir a distância
  OKLab nos 8 temas como no `BoneHandle` se o dono pedir mais contraste.
- **Sem legenda no pé** (o plano pede); fica para a próxima.
- **Enquadrar ao ABRIR** não feito: só ao trocar de clipe. Fazê-lo no 1.º quadro mexe na vista de dezenas de testes que fixam
  `px_per_s`/`view_start` — precisa de medir o estrago primeiro.
- No smoke a régua está em 10–17 s (o relógio da cena anda, T≈9,3) e o clipe tem 2 s: a fita aparece toda listrada e as
  barras ficam fora da vista até o dono trocar de clipe ou carregar `F`. A foto por isso não mostra as barras.

## Superfície de colisão

`ph2d-panel-timeline`: `linhas.rs`, `tracks.rs`, `tracks_label.rs`, `tracks_value.rs`, `paint.rs`, `ruler.rs`, `state.rs`,
testes `linhas_tests.rs`, `geom_tests.rs`, `box_select.rs`. `ph2d-tokens`: `color.rs` (698/700), `color_alias.rs`, `skin_body.rs`
(+4 tokens no `ColorToken::ALL`). `ph2d-editor-core`: `cards_banda.rs` (append). Sem `PROJECT_SCHEMA`, contrato §6 nem formato gravado.
Campo de estado +1 (`enquadrado`).

## Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Na Timeline em baixo, coluna da esquerda: «Herói» em negrito e por baixo «Translate X» e «Translate Y» com o número à
   direita; depois «Espada» e «Rotation». Nada cortado com «…».
3. Clicar no chip «pular» (ou passar o rato na fita e carregar `F`): a vista encaixa no clipe e entre as chaves aparecem
   barras coloridas (reta, curva; a «segura» tracejada). À direita do fim do clipe a fita fica listrada.
4. Ligar «Auto Key»: a borda da folha fica vermelha; desligar: some.
5. Errado se: um nome aparece cortado, um objecto aparece duas vezes, clicar numa chave já não a seleciona, ou nada encaixa
   ao trocar de clipe.

## Próximo

- Pendências desta onda: legenda das cores no pé; enquadrar ao abrir; palavra/unidade do Inspector nos canais embutidos.
- **Onda 3 do plano**: Containers e Arrange — chips das peças («used 2×» / «not placed»), lanes com peso e olho na coluna,
  o mesmo desenho de strips; sem gesto novo.
- Smoke do dono NÃO feito; sem push nem integração.
