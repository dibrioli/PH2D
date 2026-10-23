# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W17 (o smoke da W16 sobre o main) → `main` (2026-10-23)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo integrador (DIRETRIZ §1.5.3).
> Pedido: [PROMPT_W17](PROMPT_W17_o_smoke_da_W16_sobre_o_main_2026-10-23.md). W8–W16 já estão no `main`
> (`eb081da774`); o [handoff W16](HANDOFF_INTEGRACAO_line_Macros_2026-10-22_W16.md) continua válido para a parte dele.

## §1 — Base e HEAD

- Base: `972f411a6f` (main de 23/10; a linha reabriu com `reset --keep main`, sem rebase a fazer).
- Commits: `d25c4262b1` (prompt; diagnóstico das `=5`/`=6` só no ficheiro; 2 gates do ColorDrop deslocado) ·
  `9c1c53896f` (as curas, a fixtura do dono, a cena, as fotos) · o commit deste handoff.
- Toca fora da família: `docs/Components/ferramentas/fotografa_cena.sh` (guarda do ecrã, abaixo).

## §2 — Os dois defeitos do relato do dono

### Defeito 2 — levar o ColorDrop para trás apagava a 3.ª etapa (cura no leitor)

- **Mecanismo (medido na fixtura, vermelho antes):** sem o Done, o modal «Fill adjust» fica aberto e o
  toque seguinte desce ainda em Fill. `Aberta::le` dava a esse Down a âncora `marco` (a `ColorDropComeca`
  do ColorDrop) ⇒ etapas `(1, 224..228)` e `(2, 224..305)`: **3 etapas, 2 âncoras**. Deslocar a 1 escrevia
  a edição na âncora 224, partilhada pelas duas; o `tocadas` não achava a faixa da 2.ª e fazia `break` ⇒ o
  painel com duas etapas e o ▷ sem tocar.
- **Cura:** (a) a entrada em Fill é do 1.º toque (`self.marco.take()`); (b) `e_um_colordrop` já não pára
  num Down/Up: um toque em Fill sem passo que o feche entre eles é preenchimento; (c)
  `depende_de_um_drop_aberto` + `ErroDeEdicaoDoPainter::ColorDropAberto(k)` (chave
  `panel.macros.error.fill_left_open`): um Fill sem fecho e o toque que herda o Fill dele **não andam**;
  (d) `tocadas`: etapa viva sem toque ⇒ `ErroDaMacro::Edicao(AncoraSemEtapa)`, nunca as seguintes caladas.
- **Gate:** `sobrepor_tests::o_fill_sem_done_nao_leva_o_toque_seguinte_e_nada_some` sobre a fixtura
  `roteiro::grava_o_traco_o_fill_sem_done_e_o_toque` (`Gota.done = false`). Os 79 testes `macro_track` do
  `ph2d-tool-painter` verdes depois da cura.

### Defeito 1 — o C&F da cena enchia a tela de preto (cura na CENA)

- **Hipótese do prompt REFUTADA:** `repeat_image=false`, `tiling=[false,false]` no instante do `CeFDesce`
  (sonda temporária, 3 corridas: HOME limpo 1930×1040; cópia do `~/.ph2d` do dono a 1920×1080 e 2560×1080).
  Nas três o arrasto ficou fora (`on_canvas=Some(false)`), o seletor abriu, a gravação teve 2 etapas.
- **Mecanismo (por eliminação):** entre o `CeFDesce` e o `CeFSobe` da cena (~250 ms) **qualquer
  `CursorMoved` real** chega ao mesmo `fill_drag_move` (`despacho_mover.rs:177`) na posição do cursor do
  dono; sobre a tela, isso é o Down do ColorDrop — com o preto do pincel, 1 048 576 texels (1024²), «a img
  branca ficar preta». Não é defeito do produto (num arrasto real o botão está premido); é a cena.
- **Cura:** a `sobrepor` desce e sobe o C&F no MESMO quadro (`caminho_cef.rs`) — um clique, sem janela para
  o rato real; gate `caminho_tests::a_sobrepor_recua_…` confere o par no mesmo instante.
- **Fica:** as variantes `cor` (o arrasto curto à esquerda do smoke de 19/10, de propósito) e `colordrop`
  continuam com a janela de ~250 ms ao rato real — o anúncio já diz «não toque no rato».

### Ferramenta — `fotografa_cena.sh`

A guarda recusava `WAYLAND_DISPLAY=wayland-0` fixo; hoje a sessão do dono é `wayland-1` e a virtual ficou com o
`wayland-0` ⇒ toda foto recusada. Compara agora com o `DISPLAY`/`WAYLAND_DISPLAY` de quem chama (expandidos na
geração do `sessao.sh`), mantendo o `:0`.

## §3 — Smoke fotografado

[`smoke_2026-10-23_W17/`](smoke_2026-10-23_W17/): `CeFDesce`+`CeFSobe` aos 3014 ms, o vermelho escolhido,
`steps=["1.63","3.50"]` → losango recuado `["1.63","1.65"]`, ▷ com `palco` a subir até `22537 = editor`;
fotos 01–02 = as duas a crescer no MESMO quadro, cada uma na sua cor; 03 = inteiras.

## §4 — Aberto / para o integrador

- Recontagens: nenhuma (sem schema, sem registo; uma chave i18n nova no censo `CHAVES`).
- Suíte inteira: não corrida (cadência; corridos por nome: `macro_track` do Painter, `a_sobrepor_recua`,
  `every_*` do `ph2d-panel-macros`; `check` de `ph2d-app-macros` + `ph2d-host-desktop`). Clippy fica para o fecho.
