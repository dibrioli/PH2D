# 33 — Plano F22: o gesto do artista vence a máquina dos estados (e o olho diz «misto»)

> ABERTO do [handoff F21](../handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-19_F21_O_MISTO_COMPLETO.md) §5 e
> os herdados do F19 §5 (2, 5, 6) e do F18 §5 (6). Pedido do dono (19/10): *«escolha no ABERTO e siga; se
> possível, várias da mesma família no mesmo ciclo»*. A família é **a máquina dos estados de UI (Show,
> Preview, sinais) contra quem mais escreve a cena**. Quatro itens, um bloco (CLAUDE.md §0.10). O olho
> «misto» é o resto da família do F21 e entra como cauda.
> Fora, com o mecanismo nomeado em §8: a opacidade conduzida pela timeline no painel; o tracejado de vários
> pares na placa; `token_world`.

## §1 — O mecanismo (medido no código, 19/10)

| facto | onde |
|---|---|
| ⛔ **O Show em voo escreve a pose da máquina a CADA quadro**, sem olhar para a cena: `dispatch` faz `advance` e `install` de `m.pose()` em toda máquina que andava | `ph2d-app-vec/src/ui_state_bridge.rs:149-166` |
| A edição do painel (opacidade, camadas) escreve a MESMA `VecScene` noutra fase (`fase_transform_ops`, depois do `dispatch`); o gizmo e os commits da Hierarquia/Inspector escrevem antes. Nenhum avisa a máquina | `shells/desktop/src/render_loop/fase_hero_frame.rs:223,237,290`, `fase_transform_ops.rs:76,107` |
| ⇒ **uma edição feita durante um Show é apagada no quadro seguinte.** Com 150 ms é raro; com uma duração autorada de 2 s (o slider deixa), é o artista a mexer e o valor a voltar | idem |
| O único `retarget` existente corre ao pedir um Show novo, e re-alinha à TABELA, não à cena | `ui_state_bridge.rs:93`, `ph2d-ui-state/src/machine.rs:186-198` |
| `Machine::settle_on` (F18) já sabe «partir da cena», mas recusa de propósito em voo | `machine.rs:91-95` |
| A timeline escreve o `Transform` ANTES do quadro do herói (`fase_frame_open` → `timeline_bridge::run`) e declara-o no ledger (`PreviewDrive::drives(entity)`) | `fase_frame_open.rs:57`, `timeline_bridge.rs:~238`, `ph2d-preview-drive/src/consultas.rs:28` |
| **Os sinais chamam `request` e não `request_seen`**, mas só correm DENTRO do Preview (`acting = ui_preview.is_on()`), e o hover do Preview (`point`, `enter`) usa a MESMA `request`. ⇒ **não é defeito**: trocar só os sinais criaria duas portas dentro do mesmo modo | `fase_authored_controls_and_ui_states.rs:99-108`, `render_loop/ui_preview.rs:129,224,227` |
| O bloqueio do Preview tranca 3 painéis; Hierarquia e Timeline ficam abertas «porque persistem ao sair». Não há teste que o prove: `leave` reinstala as poses capturadas, inclusive de uma forma que a Hierarquia tenha apagado entretanto | `ph2d-app-vec/src/ui_preview_lock.rs:18-40`, `render_loop/ui_preview.rs:144-160` |
| A isenção das abas do encaixe (`panel_for_tab`) não tem gate: o teste diz que o registo de painéis não está instalado no binário. Mas `install_panel_registry` é público e idempotente | `ui_preview_lock.rs:38,66-70`, `ph2d-editor-core/src/panel/registry.rs:55` |
| O olho da camada pinta `Eye`/`EyeClosed` pelo primário; `RowMixed` não tem `enabled` | `ph2d-panel-vector/src/paint_stack_rows.rs:118-133`, `state_appearance.rs:56-64`, `ph2d-app-vec/src/paint_stack_mixed.rs` |

## §2 — Estado da arte (e o que foi tentado e largado)

| produto | lei | tentado e largado |
|---|---|---|
| **Apple** (*Designing Fluid Interfaces*, WWDC 2018) | **toda animação é interrompível**: o gesto do utilizador apanha o objeto onde ele está e manda dali; a animação não volta a escrever por cima | o modelo antigo do Core Animation, em que a animação explícita continua a desenhar por cima do valor do modelo até acabar: é o «pus o valor e ele voltou» que a sessão manda evitar |
| **Blender** | durante a reprodução, mexer num valor animado sem chave é apagado no quadro seguinte (o campo laranja) | é a queixa mais antiga do Blender; a casa já adoptou o pino «até o playhead mexer» (`autokey_pass.rs:27-33`) para não a repetir |
| **After Effects** | mudar uma propriedade animada no tempo atual grava uma chave: o gesto nunca se perde | — |
| **Figma** | a transição só existe no protótipo, que é outra janela: no editor nada anima, então nada há para interromper | — |
| **Este app** | Preview: o painel MOSTRA e não edita (F19, opção A do dono). Fora do Preview, o Show escreve o mundo e custa um passo de undo | — |

**As leis que saem disto:**
1. **O gesto vence o voo.** Se alguém que NÃO é a máquina escreveu um objeto dela entre dois quadros, a
   máquina pára ONDE A CENA ESTÁ (pousa sobre a cena, sem saltar) e não escreve mais. A edição fica.
2. **Motor não é gesto.** A timeline (e qualquer motor declarado no `PreviewDrive`) continua a ser
   «estado de fundo»: um objeto que um motor conduz não interrompe o Show. É o precedente do
   `driven_style.rs:17-22` (o gesto manda, o motor é fundo).
3. **O Preview continua a não editar** (F19). O que fica aberto (Hierarquia, Timeline) tem de sair do
   Preview sem desfazer nada e sem trazer de volta o que se apagou.
4. **O olho também diz «misto»** com o glifo do misto da casa (o travessão de `paint_mixed_swatch`).

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta única | escreve · lê |
|---|---|---|
| **O que a máquina deixou na cena?** | `Machine::note_written(read_back)`: depois do `install`, a ponte relê pela MESMA `capture` e guarda. Relido, e não a `pose()`: o `install` passa por `f32` e a comparação tem de ser pela mesma conversão | `ui_state_bridge::dispatch` (escreve) · `Machine` (guarda) |
| **Alguém mexeu desde então?** | `Machine::yield_to(now) -> bool`: em voo, se `now != written` ⇒ aborta o voo, `live = now`, limpa os recados (`bool_morphs`, `morph_steps`) como o `retarget` já faz. Uma função: a decisão e a aterragem não se separam | `ph2d-ui-state` |
| **Quem é motor?** | um predicado `driven: impl Fn(VecPathId) -> bool` que o `dispatch` recebe; a shell passa `PreviewDrive::drives` sobre o mapa. Objeto conduzido ⇒ não conta como gesto | shell → ponte |
| **O Show interrompido e o undo** | sem porta nova: quando a máquina pousa, o `ui_state_live` fica falso e o diff do undo regista. A medir (§6.3): ver se o passo cai no quadro da edição ou no seguinte | `undo_app.rs:334` |
| **O olho misto** | `RowMixed.enabled`, calculado em `paint_stack_mixed::row_mixed` como os outros campos; o pintor escolhe `IconId::Minus` quando `mixed.enabled` | `ph2d-app-vec` · `ph2d-panel-vector` |
| **A aba do encaixe passa no Preview** | o teste instala o registo (`install_panel_registry`) num binário próprio de `tests/it` e prova a isenção pela porta real | gate |

Recusas deste desenho:
- ⛔ avisar a máquina em cada escritor (painel, gizmo, Hierarquia, Inspector, camadas, teclado…): dez sítios,
  e o que faltar é o que ninguém vê. Ler a cena é uma pergunta, numa porta.
- ⛔ comparar com `m.pose()`: o `install` passa por `f32` (`ui_state_edit.rs:246`), e a máquina leria o
  seu próprio arredondamento como uma edição e nunca acabaria um Show.
- ⛔ trancar o painel durante o Show (como no Preview): o Show é um botão do editor, não um modo, e trancar
  por dois segundos o painel que o artista está a usar é o defeito ao contrário.
- ⛔ trocar os sinais para `request_seen`: ver §1. Fica medido e fechado, com o porquê no código.
- ⛔ deixar a timeline interromper o Show: um objeto animado e com estados nunca chegaria ao fim de um Show.

## §4 — Contrato congelado / formato

- `Machine` não é serializada (`ui_state_bridge.rs:37-40`): o campo novo é runtime. `ObjectPose` não muda.
- `ph2d-editor-core/src/tool.rs`, `architecture_tool_contract_surface.rs`, `ph2d-vector-doc/src/primitives.rs`:
  não tocados (`git diff --stat` no fecho). `PROJECT_SCHEMA` intocado. `RowMixed` é estado de VISTA (o
  construtor literal ganha um campo: os sítios são atualizados).

## §5 — A UI: as quatro condições

Nenhum controlo novo. O olho EXISTE, é pintado e hit-indexado (`vector_paint_eye_id`), o clique chega ao
`toggle` (F21). Muda o estado pintado e a SEQUÊNCIA:

| sequência | hoje | depois |
|---|---|---|
| estrela com Hover autorado a 2 s → Show Hover → a meio, Opacity 40 no painel | a opacidade volta à do tween e a estrela acaba no Hover | a estrela pára onde estava, a 40 %; o botão deixa de dizer «a ir para Hover» |
| idem, mas a meio arrastar a estrela no canvas | volta para o caminho do tween | fica onde o artista a pôs |
| um objeto animado na timeline e com estados → Show Hover | chega ao Hover | igual (o controlo do motor) |
| Show sem tocar em nada | chega ao Hover exatamente | igual (o controlo: a máquina não se interrompe a si mesma) |
| duas formas, a camada 1 visível numa e escondida na outra | o olho do primário | o olho com «—»; o clique iguala (F21) |
| Preview → apagar uma forma com estados na Hierarquia → sair | por medir | a forma continua apagada, nada volta; um Ctrl+Z traz a forma |

## §6 — Gates, vermelhos primeiro

Fixture: um retângulo com Default (x 0, opacidade 1) e Hover (x 10, opacidade 1), duração 2 s, linear; a
mesma montagem do `o_show_do_default_parte_da_cena_e_devolve_a_cor` (`SimWorld` + `VecEntityMap`).

1. `uma_edicao_a_meio_do_show_fica` (ponte): Show Hover, `dispatch` 0,5 s, a cena recebe opacidade 0,4,
   `dispatch` 0,1 s ⇒ opacidade 0,4, máquina parada, `x` igual ao do quadro da edição. **Vermelho hoje.**
2. `um_arrasto_a_meio_do_show_fica` (ponte): o mesmo com a translação. **Vermelho hoje.**
3. Controlos: `um_show_sem_toque_chega_exato` (a máquina não se lê a si mesma: o arredondamento por `f32`
   não aborta); `um_objeto_que_um_motor_conduz_nao_interrompe` (o predicado `driven` ligado);
   `interrupting_a_show_with_another_show_moves_the_scene` e o F18 continuam verdes.
4. `ph2d-ui-state`: `yield_to_parado_nao_faz_nada`, `yield_to_igual_nao_aborta`, `yield_to_diferente_pousa_e_limpa_os_recados`.
5. Undo (shell, `ui_preview_tests.rs` ou irmão): o Show interrompido por uma edição de opacidade deixa UM
   passo; o Ctrl+Z devolve a cena a antes do Show. Medido primeiro; se o passo ficar órfão (sem input no
   quadro em que a máquina pousa), a cura é marcar o input no pouso, e o gate fica vermelho antes dela.
6. Preview × Hierarquia (shell): entrar, apagar o hospedeiro, sair ⇒ a forma não volta e não há pânico.
   Medido primeiro; a cura só existe se ficar vermelho (série declarada: o resultado decide o passo seguinte).
7. `a_aba_do_encaixe_passa_no_preview` (`tests/it`, registo instalado): vermelho com a isenção apagada.
8. Olho: `o_olho_diz_misto` (`paint_stack_mixed`) e, no painel, `o_olho_misto_pinta_o_travessao`
   (`seam_paint_stack.rs`, pela porta real de pintura). **Vermelhos hoje.**
9. Cenas: `as_faixas_de_estagios_nao_se_cruzam` com as famílias novas.
10. Censos: `the_shell_only_shrinks`, tecto de LOC (`ui_state_bridge.rs` 358, `machine.rs`), `architecture_panel_wiring_parity`.
11. Mutação: `yield_to` nunca aborta (sangra 1, 2) · compara com `pose()` (sangra o controlo 3) · `driven`
    ignorado (sangra o controlo do motor) · `note_written` antes do `install` (sangra 1 ou o controlo) ·
    `RowMixed.enabled` sempre falso (sangra 8) · a isenção da aba apagada (sangra 7).

## §7 — Smoke (cenas novas `35` e `36`; a próxima livre conta-se em `vector_mode_scenes.rs`)

Estágios livres medidos: `248..=249` e `254..=255`. A 35 continua a família `interface` (desenha a estrela e
grava o Hover, como as 27–30) e acrescenta 2 estágios; a 36 continua a `misto` e acrescenta 1–2.

- `PH2D_OBJECT_MODE_SMOKE=35`: a estrela com Default e Hover (o Hover mais à direita), duração 2 s; o Show
  Hover é pedido e, a meio, o painel recebe Opacity 40 pela porta do campo ⇒ a estrela parada a meio
  caminho, a 40 %, e o readout do Show já não diz «a ir». Os números (x do meio, opacidade) **medem-se pela
  sonda headless** antes da mensagem ao dono.
- `=36`: duas formas com a mesma pilha (Fill + Stroke extra), a camada Stroke escondida numa ⇒ o olho com «—».
- Fotografadas (`docs/Components/ferramentas/fotografa_cena.sh`), em `vetor_nova_ui/fotos/F22_cena35.png`
  e `F22_cena36.png`.

## §8 — Fora, com o mecanismo nomeado

- **A opacidade conduzida pela timeline no painel:** o painel lê o `VecPath.opacity` (autorado), a tela mostra
  `VecDrivenStyle.alpha` (`driven_style.rs`). Corrigir é «o painel lê o que se vê» para um valor que a
  TIMELINE escreve, e isso inclui o marcador de campo animado (nenhum painel tem, nem o Inspector: o
  diamante só existe na dope-sheet). É a família timeline × painel: outra onda.
- **O tracejado de vários pares na placa:** `ph2d-shape-gpu/src/eixo.rs:129` só aceita um par e marca o
  resto como «só conforme». Família render/placa.
- **`token_world`** (F20 ABERTO 6): fica, outra lei (régua ≤ 0).
- **A cor por token** não mente: a rachura + o chip «Token» já dizem que a amostra é o autorado coberto.
