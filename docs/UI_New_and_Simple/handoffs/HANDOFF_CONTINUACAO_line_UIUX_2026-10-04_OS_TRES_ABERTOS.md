# Continuação — `line/UIUX`, 2026-10-04 — os TRÊS abertos dos modos (Ctrl+Tab · Image ▸ Mask · Model em Object)

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd Worktrees/line-UIUX && pwd && git branch --show-current` ANTES de ler. HEAD esperado = o commit
> deste ficheiro; árvore limpa; base (merge-base = `main`) `1ad60a1ce`, nenhum commit integrado.

## §1 — Estado

- O objecto vetorial (F3 ▸ Vector refeita) está entregue e com **smoke do dono APROVADO (04/10)**:
  [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_OBJECTO_VETORIAL.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_OBJECTO_VETORIAL.md)
  (§5b: os três reports do dono depois do fecho e as curas; §8: as decisões dele sobre a Hierarquia
  e o duplicar — a última, «a cópia fica no mesmo pai», curada em `987769702`).
- A linha acumula OITO handoffs de integração (A_ESCALA, O_MENU_ADD, OS_MODOS, O_SCULPT, O_FLIP,
  O_MODEL, O_VETOR, O_OBJECTO_VETORIAL). ⛔ Nada integrado; integrar é só por ordem do dono.

## §2 — A TAREFA (ordem do dono, 04/10: *«depois»*, os três que estavam na lista)

Por esta ordem (do mais barato ao mais caro — meça antes de cada um):

1. **`Ctrl+Tab` abre a lista dos modos** (spec/06 §3: *«Tab alterna; Ctrl+Tab abre a lista»*).
   Medido: o `Tab` vive em `shells/desktop/src/input_handlers.rs:125` (empurra
   `EditorAction::ObjectMode(ModeRequest::Toggle)`); a lista JÁ existe — é o seletor «Mode» do
   cabeçalho da área, `ph2d_editor_core::ids::area_menu_button(0)`, que as cenas de smoke abrem com
   `hero.apply_event(WidgetEvent::Click(chip))` (`ph2d_app_vec::vector_mode::smoke_step`, estágio 10).
   ⇒ provavelmente o `Ctrl+Tab` é ESSE clique, com o mesmo cuidado de foco de texto que o `Tab`
   tem. Confirme que o `Ctrl+Tab` não está já tomado por outra porta (grep `Tab` nos atalhos).
2. **Image ▸ Mask** (D6: Image = Object · Paint · **Mask**; `ObjectMode::Mask` já está no
   vocabulário; spec/06 linha da tabela §3.4 marca ⏳). ⚠️ **A frase do produto ainda não existe**:
   meça o que o Painter já tem de máscara (há `mask_tests` em `ph2d-tool-painter`; camadas de
   máscara?) e, se o que «Mask» quer dizer não for óbvio pela medição, **pergunte ao dono com o
   que mediu** antes de codar (a lição do §5 do handoff O_VETOR: perguntar com o número).
3. **A peça do Model desenhada no canvas 2D em Object** (spec/06 F3 ▸ Model, «fica de fora,
   nomeado»: D9 — *«o 3D como camada entre camadas é outra obra»* —, e o gizmo do objecto move um
   `Transform` que o traçado não lê). ⚠️ Pode ser grande: meça o custo (o que o traçado do Model
   precisa para se compor como uma camada 2D, e quem o lê) e, se for uma obra própria, **diga-o ao
   dono com o número** antes de começar — ele pode preferir uma linha nova.

## §3 — Restrições (medidas no HEAD deste ficheiro)

- Shell (`the_shell_only_shrinks`): **196 982 / 196 990 — folga 8**. Código de família vai para a
  crate dela; a fase da shell só compõe.
- `ph2d-ecs/src/scene/registry.rs` **700/700**; `publish_gizmo` (`render_loop/snapshots.rs`) perto
  do tecto de 200 linhas de função (`fn_loc_caps`).
- `ModeFamily` (fundação) ganhou nesta linha `joins`/`enter_with`/`parts_take_the_object_gizmo`;
  `ModeState::part_gizmo`; `mode_drive::lasso_admits`.

## §4 — Lições desta linha que valem aqui

- ⛔ **Uma cópia com gate e ZERO escritores do lado de lá**: o `editing` copiado no
  `view_state_for_pick` tinha gate, ninguém o publicava no `view_derived` — o Select em Edit morria.
  Gateie o ESCRITOR, não só a porta.
- ⛔ Uma lei «só em Object» (o gizmo, o laço) escrita para os modos de objecto inteiro parte o
  primeiro modo de PARTES — pergunte pela família.
- A foto apanha o que nenhum gate vê (três vezes nesta onda); os cliques NÃO se fotografam (o XTest
  é ignorado na tela virtual) — prove-os por gate e deixe-os ao smoke do dono.

## §5 — Como a onda fecha

DIRETRIZ §1.5.9, como as anteriores: gate batched 1× desde `1ad60a1ce` (verificador), mutação dos
gates novos (agente `mutacao`, controlo verde com população > 0), fotos (`fotografa_cena.sh`),
handoff de integração NOVO que cita os oito, `rm -rf target/*/incremental`, e por último o build de
smoke 2× com a 2.ª saída colada. ⛔ Não integrar nem pushar.
