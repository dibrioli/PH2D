# F32 — o que ainda endereça POR POSIÇÃO e o resto da identidade (plano, 25/10)

Fila: [F31 handoff §6](handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-25_F31.md) · desenho herdado (não reabrir):
[F31_plano §1](F31_plano.md) · leis do dono: camada apagada leva a linha e o Desfazer devolve as duas (25/10); o Auto Key
grava tudo (19/10). Base: ponta `8477e1511b` sobre o main `c3662cd8e4` (o main não andou).

## 0 — Medição: o mecanismo de cada item (uma família, CLAUDE.md §0.10)

| # | item | existe? | mecanismo medido |
|---|---|---|---|
| 1 | binding de item apagado nunca se purga | **sim** | `refresh_liveness_and_rest` (`ph2d-timeline/src/apply.rs:280`) marca `item_missing`; a purga (`ph2d-timeline-persist/src/lib.rs:292,319`) e a cura (`persist.rs:116`) EXCLUEM-no de propósito (lei 25/10). Nada mais o tira ⇒ o binding vai para o ficheiro e volta escondido a cada load, para sempre — já nenhum undo o pode devolver (o load abre a sessão sem histórico). |
| 2 | `ObjectPose.filters` por posição | **sim** | a pose guarda só `Vec<FxOp>` (`ph2d-ui-state/src/pose.rs:69`); `capture` copia `f.ops()` (`ui_state_edit.rs:155`), `install` faz `troca_ops` (`:279`), que guarda os ids POR POSIÇÃO e `resize` com `NONE`. Default `[Blur]` × Hover `[Blur, Glow]`: cada entrada no Hover dá ao Glow um id NOVO (a chave dele fica órfã); Default `[Blur, Glow]` × Hover `[Glow, Blur]`: a animação do Blur passa ao Glow. |
| 3 | setas da StateMachine por índice | **em parte** | a composição JÁ resolve as setas: `RemoveState` apaga as que apontavam o estado e recua as de depois (`statemachine_inspector.rs:104-112`); não há reordenar. Sobram dois defeitos: (a) o `initial` não recua (`:113` só repõe 0 se ficar FORA) — apagar o estado 0 com inicial 2 faz do antigo 3 o inicial; (b) `StateMachineRuntime.current: u8` (`state_machine.rs:139`) só se reconcilia se ficar FORA (`:174`) — com o jogo a correr, apagar um estado anterior muda o estado corrente em silêncio. |
| 4a | `TimerRuntime` reconciliado por COMPRIMENTO | **sim** | `timer::reconcile` (`timer.rs:277`) só compara `len`; apagar o timer 0 de `[A a correr, B parado]` trunca o ÚLTIMO slot ⇒ B herda o relógio de A. |
| 4b | tween↔timer por posição | **sim** | `tween::a_escrever` (`tween.rs:105`) lê `cfg.0.get(i)`: apagar o timer 0 põe o tween 0 no relógio de B e deixa o tween 1 sem relógio. ⇒ cabe nesta linha: a `line/GameEngineFuncional` não toca `timer.rs`/`tween.rs`/`state_machine.rs` (só move `timer_tick.rs` e mexe `fase_sequences_tests.rs`, `inspector_timer.rs`). O `SequencePlayer` toma «o primeiro timer» — é um PAPEL, não um item: fica. |
| 5 | `.ph2dmacro` antigo com listas/`VecFilter` | **sim** | a família «mundo» guarda `Componente { nome, bytes }` em postcard da forma do registo (`ph2d-app-macros/src/mundo/formato.rs:37`) e o aplicar faz `insert_from_bytes` direto (`aplica.rs:294`). Uma macro gravada no main (listas e `VecFilter` sem ids) não se lê aqui. A migração por blob JÁ existe e reconhece a forma SEM versão (`item_id::legado`: lê `L` até ao último byte, senão `T`) — só não tem porta pública por blob. |
| 6 | re-arme da base do undo fora do `fase_object_mode` | **sim, e é geral** | `post_frame_undo_steps` (`undo_app.rs:447`) suprime todo quadro «sem entrada» e FUNDE a mudança no passo seguinte. Toda cena de smoke que mexe no documento ao longo de quadros sem entrada (as ~30 `PH2D_OBJECT_MODE_SMOKE` do `vector_mode`, a do Flip, a do Painter…) cai no 1.º gesto do dono: o 1.º Ctrl+Z desfaz a cena. Só as duas autokey ligam `rearmar`. |

## 1 — Desenho

**1 — purga no load.** `TimelineState` ganha `itens_por_decidir: bool` (sessão, nunca gravado), ligado só por
`install_from_project`. No `upkeep`, enquanto ligado: um binding de chave com item cuja entidade está viva e cuja família
PUBLICOU a entidade (porta nova `canal_item_vivo::decidido(world, bits, k) -> Option<bool>`, `None` = sem espelho) decide-se;
`Some(false)` ⇒ `purge_binding` (sem passo de undo nem cova: nada o pode devolver). Desliga-se quando nenhum binding de item
fica por decidir. Na sessão a lei de 25/10 fica intacta (a bandeira só existe entre o load e a 1.ª decisão).

**2 — ids na pose.** `ObjectPose` ganha `filter_ids: Vec<ItemId>` paralelo a `filters` (capture: `f.ids()`; install:
`VecFilter::troca_pares(ops, ids)`, que põe os dois juntos). A transição mistura por ÍNDICE como hoje (lei de 21/08) e o id
de cada degrau do meio é o do destino, senão o da partida. Formato: `PROJECT_SCHEMA` 196 → **197**; a 196 lê-se pelo
âmbito de leitura (precedente `PilhaLayoutV23`), ids `NONE`, e o install com id `NONE` adopta o id que a forma tem
nessa posição (o comportamento de hoje, só para poses antigas).

**3 + 4 — o vivo reconcilia-se pela IDENTIDADE do item** (uma lei, três sítios):
- `TimerRuntime` guarda o id de cada slot; `reconcile` casa por id (slot de id que saiu ⇒ fora; id novo ⇒ `born`).
- `StateMachineRuntime` guarda o id do estado corrente; `reconcile` relocaliza o índice pelo id (saiu ⇒ `born`).
- `RemoveState` recua também o `initial`.
- `Tween` ganha `relogio: ItemId` (o timer dele). Um passe adopta `NONE` ⇒ o id do timer na MESMA posição (tween
  novo, ficheiro antigo); `a_escrever` procura o timer pelo id; sem ele, o tween sai (lei de hoje). Formato: o
  mesmo degrau 197, forma 196 congelada (`Passe::de_ate`, padrão `item_id_v194.rs`).
- ⚠️ Fica aberto, e diz-se: uma CHAVE de animação sobre `from`/`to` de uma seta guarda o número do estado; apagar um
  estado anterior não a reescreve (a timeline não conhece a máquina). Animar o destino de uma seta é raro; curá-lo exige
  chaves que guardem ids, que o graph editor mostraria como números sem sentido.

**5 — macros.** Porta pública `item_id::migra_blob(canonical, bytes) -> Legado`; o aplicar da família «mundo» passa
cada `Componente` por ela antes do `insert_from_bytes` (no `Inicial` e em cada `Muda`/`Nasce`). Sem versão nova: a forma
reconhece-se a si própria. Cobre também o degrau 197 do `Tween`.

**6 — a base é do documento até à 1.ª entrada do artista.** `UndoLinks` ganha `virgem` (desliga na 1.ª entrada) e
`tira_a_base` devolve `true` em todo quadro quieto enquanto virgem — o boot, o load assíncrono e toda cena de smoke
ficam na base, nenhuma tem de se lembrar. Zero linhas na shell (`undo_app.rs` 600/600). O `rearmar` das autokey fica
(elas encenam DEPOIS de entrada). Custo: uma captura por quadro quieto ANTES da 1.ª entrada — medir.

## 2 — Gates (cada um visto VERMELHO por mutação antes)

| # | gate | mutação |
|---|---|---|
| 1 | load com binding de item apagado ⇒ purgado; sem espelho ⇒ espera; na sessão (sem load) ⇒ fica | `decidido` devolve sempre `None` / bandeira nunca desliga |
| 2 | Hover com filtro só dele: o id sobrevive a sair e voltar; reordenar entre estados leva a chave | install volta a `troca_ops` |
| 3 | apagar estado anterior: `initial` e `current` seguem o mesmo estado | tirar o recuo / reconcile por índice |
| 4 | apagar o timer 0: B fica com o seu relógio, o tween de B fica no B | reconcile por comprimento / `get(i)` |
| 5 | macro com `Timers` v194 e `VecFilter` v195 aplica-se | aplicar sem `migra_blob` |
| 6 | cena que monta em quadros quietos ⇒ o 1.º Ctrl+Z não a desfaz | `virgem` sempre `false` |
| F | fixture `PROJECT_SCHEMA` 196 (pose com filtros + tween) lê-se | tirar o âmbito |

## 3 — Smoke

Uma cena `PH2D_AUTOKEY_SMOKE=37`: um botão com Hover que tem um Glow só dele, animado, e um objecto com dois timers e dois
tweens. O dono apaga o 1.º timer (o 2.º tween continua a mexer) e mostra o Hover duas vezes (o Glow continua animado).
Fotografar antes.

## 4 — Formato (DELTA para a integração)

`PROJECT_SCHEMA` 196 → 197 (+1 na linha: F30 +2, F31 +1, F32 +1 ⇒ +4 sobre o main 193). Recontar com
`python3 scripts/schema-recount.py`.
