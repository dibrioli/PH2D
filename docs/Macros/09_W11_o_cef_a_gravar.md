# W11 — o botão C&F a gravar: trocar a cor não pára a macro, e o ColorDrop grava-se (2026-10-19)

> Pedido: [PROMPT_W11](handoffs/PROMPT_W11_trocar_a_cor_nao_quebra_a_macro_2026-10-19.md) (smoke do dono de 19/10,
> depois da W10). Handoff: [W11](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W11.md).

## §1 — O defeito, medido

A gravar, o dono trocou a cor pelo **C&F**. O painel MACROS mostrou `Refused: begin_colordrop_fill` ·
`Refused: fill_commit` · `Refused: restore_after_…` · `Refused: fill_commit`, e o ▷ parou na 1.ª recusa
(`tocar.rs`, `ErroDaMacro::PortaRecusada`) — as pinceladas seguintes não tocavam.

Duas causas, as duas confirmadas:

1. **A shell entrava em Fill antes de saber se o ponteiro estava na tela.** Cada `CursorMoved` com o C&F
   armado chamava `begin_colordrop_fill` enquanto `!on_canvas` (`fill_drag.rs`), e soltar fora da tela
   chamava `restore_after_colordrop`. Um clique que só abre o seletor passava por 2 portas do
   preenchimento (e pelos `fill_commit` que o `set_paint_tool_mode` chama por dentro — `macro_recusa` não
   abria nível, logo os de dentro também gravavam recusa: é a 4.ª linha do painel do dono).
2. **As 4 portas do ColorDrop eram `Recusa`** (`portas.rs`, «forma, curva, preenchimento…: fora do Wet
   Paint (W2)»). Mesmo o ColorDrop de verdade parava a macro.

Gate vermelho antes da cura (`cc43e8b0f8`): `tests/it/macro_colordrop.rs`, os dois testes reprovaram na
asserção «recusas na trilha».

## §2 — A cura

- **Shell** (`deliver_canvas_pointer_colordrop`): o C&F passa à entrega o modo a que o ColorDrop volta, e
  ela chama `begin_colordrop_fill` **depois** de o `Down` passar a pegada, imediatamente antes do
  `on_canvas_pointer`. A margem do gizmo do Deform não conta para esse `Down` (Fill não tem gizmo) — como
  antes, quando o modo já era Fill no teste da pegada. O ramo do seletor deixou de chamar
  `restore_after_colordrop` (nada começou). Um arrasto que não chega à sprite não muda o modo.
- **As 4 portas gravam** o que recebem (a lei do ADR-0186): `Passo::ColorDropComeca(modo)` = 53,
  `LimiarDoFill(f32)` = 54, `FillCommit` = 55, `ColorDropRestaura` = 56. O preenchimento é determinístico
  dado o estado (`fill.rs::refill_from_snapshot`: semente `floor`, tolerância `round(limiar·255)`, a cor
  do pincel em `u8`, a componente da seleção), e o resto do gesto já chegava por portas gravadas (o
  ponteiro na tela; o modal pelo `handle_panel_event`). As portas de dentro (`set_paint_tool_mode` →
  `fill_commit`) deixam de gravar porque o `porta!` abre nível.
- **A etapa «Fill»** (`etapas.rs`): o ColorDrop vai do `Down` ao passo que o fecha — o `Done`/`Cancel` do
  modal (`Painel(Click(PAINTER_FILL_MODAL_DONE|CANCEL))`) ou o `FillCommit` do gesto parado. O gesto parado
  **não tem `Up`**: sem esta regra não havia etapa e as cores das pinceladas seguintes desalinhavam no
  painel (as propriedades leem-se uma por `Down`). O tipo decide-se pelo último marco antes do `Down`
  (`ColorDropComeca` ⇒ Fill). `TipoDaEtapa`/`TipoDaEtapaView::Preenchimento(n)`, «Fill {n}» no i18n.
  Editar a cor de um Fill cobre o limiar afinado depois de soltar (a cor volta depois do fecho, não do
  `Up`); o tamanho é recusado (`NaoEPincelada`) e o painel não o mostra.

## §3 — Decisão: o ColorDrop é passo pelas suas portas, não um `Passo::ColorDrop` agregado

O prompt propunha um `Passo::ColorDrop { semente, cor, limiar, commit }`. Recusado: a trilha grava o que
cada porta recebeu, e o preenchimento já é a soma de portas gravadas (o ponteiro, o modal). Um passo
agregado seria uma segunda descrição do mesmo gesto, com o dwell e o limiar ao vivo a reconstruir à mão
— duas fontes que divergem no primeiro caso que só uma conhece.

## §4 — ⛔ Recusas e limites medidos

| O quê | Porquê |
|---|---|
| Deixar o `restore_after_colordrop` no ramo do seletor «por segurança» | nada começou; gravava um passo morto |
| Testar a pegada com a margem do Deform do modo anterior | aceitaria um `Down` fora da sprite com um Transform vivo; o Fill não tem gizmo |
| Etapa do ColorDrop a acabar no `Up` | o limiar afinado no modal depois de soltar re-inunda com a cor ORIGINAL quando a cor da etapa foi editada |
