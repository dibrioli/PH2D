# HANDOFF de continuação — line/BarraDeMenus, onda 5 (12/11): a caneta move e redimensiona a janela

Antes: [onda 4](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA4.md) · plano: [01_plano.md](../01_plano.md).

## O que mudou
- **Mecanismo:** a faixa, com a caneta pousada, chama `ph2d_pen::porta::arrasta_janela(janela, borda)`. A porta tira o
  `xdg_toplevel` do winit (`WindowExtWayland::xdg_toplevel`, PÚBLICO no 0.30.13 — sem patch) e entrega-o ao fio da caneta,
  que manda `xdg_toplevel.move/resize` com o `serial` do `zwp_tablet_tool_v2.down` e o `wl_seat` da mesa
  (`wayland.rs::cumpre_arrasto`). A mesma `wl_display` do winit ⇒ o mesmo cliente: o serial vale.
- **KWin 6.8.2 aceita-o** (lido no código da tag `v6.8.2`): `XdgToplevelWindow::handleMoveRequested/handleResizeRequested` →
  `InputRedirection::implicitGrabPositionBySerial`, que aceita ponteiro, toque **e** `TabletSeatV2Interface::hasImplicitGrab`
  (o `downSerial` da ferramenta). O `MoveResizeFilter` segue `tabletToolAxisEvent` e acaba no `tabletToolTipEvent` (ponta sobe).
- O compositor fica com o contacto: o `up` nunca chega. `Sessao::solta` (fio) e `Conversor::solta` (porta) largam a ponta
  sem `Up` — teste `a_contact_taken_by_the_window_drag_leaves_no_tip_stuck_down` (mutação: sem os dois `solta` sai
  `(Move, 0.6)` — a ponta presa; com eles verde).
- `ph2d-pen` ganhou a feature `winit` (dep opcional; só a shell desktop a liga — o `ph2d-pixel-ponte` não paga o winit);
  fora do Linux `arrasta_janela` devolve `false`.
- `PH2D_FAIXA_LOG=1` imprime `[faixa única] caneta: Mover serial N` quando o pedido sai.

## Provas
- `kwin_faixa_unica.sh` (rato falso, KWin virtual): passos 1–8 iguais à onda 4 (sem regressão do rato) — `fotos/onda4_kwin.txt`.
- ⚠️ **A caneta NÃO tem prova automática:** o KWin `--virtual` não tem libinput nem tablete falso (`fake_input` = rato,
  toque, teclado). A prova é o smoke do dono. Um tablete `uinput` mexeria na sessão real do dono: recusado.

## Smoke do dono (por fazer)
Com `PH2D_FAIXA_UNICA=1`: arrastar a faixa com a caneta, redimensionar pela borda/canto, duplo toque maximiza. E (da onda 4,
ainda sem resultado) a barra de tarefas com a caneta **com a barra do sistema**: se falhar também, é do KDE.

## PRÓXIMO
1. Smoke verde ⇒ `faixa_unica::ligada()` volta à omissão (`PH2D_FAIXA_UNICA=0` desliga) e o `kwin_faixa_unica.sh` deixa
   de a forçar. Vermelho ⇒ correr com `PH2D_FAIXA_LOG=1` e ler se o pedido saiu (linha `caneta:`) ou se o KWin o ignorou.
