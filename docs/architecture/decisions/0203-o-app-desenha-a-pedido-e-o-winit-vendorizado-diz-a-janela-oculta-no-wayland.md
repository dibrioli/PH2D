# ADR-0203 — O app desenha a pedido (`quadro::Laco`) e o `winit` vendorizado diz a janela oculta no Wayland

- **Status:** Accepted — missão Painter 4K (dono 10/11, «todas as recomendações dos estudos»); desenho técnico delegado
  (padrão-ouro).
- **Data:** 2026-11-11
- **Linha:** `line/PainterWatercolor`
- **Amparo:** `docs/Painter/missao_4k/BACKLOG.md` B23 (B23.1–B23.6); medição de partida em `missao_4k/medicoes/ANTES.md`
  (parado: 48–70 % de um núcleo).

> Número recontado na integração de 13/11 (§5.0): a linha escreveu `0202`; a linha JointEditor tomou o `0201`.

## Contexto

O laço da shell era `ControlFlow::Poll` com `request_redraw` incondicional (`present_title.rs`): o app desenhava e
apresentava sem entrada, 48–70 % de um núcleo parado e 97–100 % com a janela oculta. No Wayland o `winit` 0.30.13 não
emite `WindowEvent::Occluded` para o estado `suspended` do `xdg_toplevel` (só o X11 tem `VisibilityNotify`), por isso a
shell nunca sabia que a janela estava minimizada no KWin.

## Decisão

1. **A lei do quadro mora em `ph2d_app_host::quadro::Laco`** (não na shell): guarda foco, última entrada (segura o quadro
   `PRAZO = 250 ms`), oculta, e se o processo é um smoke (`PH2D_*SMOKE*`/`*DEMO*`, `PH2D_EXIT_AFTER_FRAMES`) — esses nunca
   dormem. `Laco::acorda` → `None` = desenhar já, `Some(t)` = `WaitUntil(t)`. Cada família diz os seus produtores
   (`PainterTool::quer_quadro`: traço aberto, papel a secar, sessão Wet viva, prévia do Stencil; o Motion parado e a rede do
   Quadro também). `PH2D_QUADRO_A_PEDIDO=0` volta ao laço contínuo.
2. **Janela oculta não apresenta**; acorda a `SEM_PRESENT` (16,67 ms) para os produtores do documento continuarem.
3. **`winit` 0.30.13 vendorizado** em `crates/ph2d-app-host/vendor/winit` (Apache-2.0, `[patch.crates-io]`, fora da
   workspace, fmt/typos fora; cada hunk marcado `PH2D (VENDOR.md)`): o `suspended` do configure vira
   `WindowEvent::Occluded(now)`. Sai quando o upstream o emitir.

Gates: `ph2d-app-host` `a_entrada_segura_o_laco_so_o_prazo`, `o_smoke_nao_dorme_nem_desfocado`,
`oculta_sem_present_o_documento_corre_a_60_e_nao_gira`; `ph2d-editor-core` `o_wayland_suspenso_chega_a_shell_como_occluded`
(vermelho com o `event_loop/mod.rs` do upstream). Medido: oculta 97–100 % → 44–46 % de CPU a 60 q/s.

## Alternativas recusadas

- **`pre_present_notify` como sinal de oculta:** minimizada não há frame callbacks; os produtores do documento (papel a
  secar) parariam e o byte mudaria.
- **Segunda ligação Wayland para ler o estado:** um cliente não vê os estados da superfície de outro.
- **`Publicado<T>` para o caso (e) do B23.4:** ≤ 0,3 ms medido; não paga a peça.

## Preço e risco

- ~63 k linhas de código de terceiros na árvore (os ficheiros acima de 700 linhas aparecem no `collision-surface.sh`; o
  gate de tecto não varre `vendor/`). Atualizar o `winit` passa a ser re-aplicar 3 hunks.
- Um produtor que corre sem evento e não se declara em `quer_quadro` congela até à próxima entrada — o gate de cada família
  é o censo dos ramos do seu tique.
- Aberto: o app panica no quadro 0 dentro do `kwin_wayland --virtual` (WSI, `buffer invalid`); a medição minimizada no
  Wayland real fica para o smoke.
