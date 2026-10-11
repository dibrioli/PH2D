# BUGS da Luz 2D — os que enganavam

> Um bug entra aqui quando o sintoma apontava para OUTRO lugar. O mecanismo, a medição e o gate
> que o guarda; o resto vive no handoff da onda.

## 1. «Com a luz ligada o app custa a maximizar e a encaixar na tela» (11/10)

- **O sintoma enganava:** parecia o compositor, a janela ou o layout do editor. Era a luz: a GRADE
  dela segue o tamanho da janela, e cada tamanho novo recriava o `Solver` inteiro — o maximizar anima
  o tamanho e o recriava a CADA quadro.
- **Medido** (o caminho do produto, `LuzDoQuadro::preparar` + `gravar`, maximizar de 12 quadros de
  1024×768 a 1920×1040, cena 3): ~1 850–2 100 ms por quadro que muda a grade contra 3–8 ms com o
  tamanho fixo; alocar texturas e buffers ~10 ms; o resto é compilar as pipelines da C3 («c3 fina»
  390 + 730 ms nas duas variantes, «c3 rebote» 120 + 210 ms). Recriar só o módulo com os layouts
  guardados saía rápido — o caro é compilar TUDO de novo (módulo, layouts e pipelines).
- **Por que nenhum gate o via:** todas as réguas da luz medem o tempo de PLACA (os carimbos) de um
  solver já montado, num tamanho fixo; o custo de nascer é de CPU e nunca entrou em nenhuma conta.
- **A cura:** `ph2d_light2d_gpu::Fabrica` — os módulos, os layouts e as pipelines guardados pela
  descrição inteira (o `Debug` do descritor traz a identidade dos objetos que referencia), partilhados
  pelos solvers de uma mesma `LuzDoQuadro` (`Solver::novo_com`). Uma grade nova só realoca. O zoom a
  atravessar oitavas e a troca de nível (a mesma família: grade nova) reaproveitam o módulo e os
  layouts — 2–10 ms medidos.
- **Gate:** `redimensionar_a_janela_nao_recompila_a_luz` (`tests/it/redimensionar.rs`): num
  maximizar, os solvers das grades novas compilam ZERO pipelines (antes 190 em 10 grades). Mutações
  4/4: sem a fábrica no editor; o módulo sem cache; o cômputo sem cache; a fábrica a esquecer a cada
  chamada — esta cegava a régua enquanto o contador vivia dentro do cache (ele passou para fora).

## 2. «O disco sai como um cone em fatias, com a aresta da sombra serrilhada» (11/10, cena 5)

- **O sintoma enganava:** o cone parecia o defeito e a suspeita (handoff W5c) era a c0 junto à borda a
  saltar quando a sonda de fora trocava. O CONE é a lei (o interior é o exterior mais próximo — o
  gabarito com o rebote também sai em cunhas) e a c0 contínua não mudou nada (medido).
- **Medido** (a régua dos anéis: polígonos de 96 lados contra o gabarito, `as_formas_redondas.rs`): três
  causas sob o cone — (1) o iso-½ da bilinear desvia-se da borda até ~0,08 texel e o raio rasante que sai
  da passagem luz/sombra reentrava no contorno (a passagem em escada: 4 cruzamentos a mais do meio-tom);
  (2) com o rebote, as sondas da cascata coladas a uma superfície que reflete variam ±15 % de uma para a
  outra (poucos raios curtos veem o brilho dela) e a lei do interior projetava o ruído até ao centro (as
  fatias finas do lado escuro); (3) num SPRITE a cobertura capturada era a média do alfa — a rampa do
  disco do atlas não é área de nada, e a borda exata (que lê área) saía em zigue-zague.
- **Por que nenhum gate o via:** os gates do interior eram de CAIXAS (bordas retas: o iso-½ quase não
  erra, e a média do alfa de uma textura branca é área) e sem rebote.
- **A cura:** a borda exata da cobertura no interior (`c3_borda.wgsl`, o AAEDT), a cobertura do sprite
  pelo meio (alfa ≥ ½; a camada do vetor fica com a média — `alfa_e_area`) e a c0 do ponto de fronteira
  pela MEDIANA ao longo da mesma face (a média espalhava o vazamento da cascata num canto). Gates: `as_formas_redondas_nao_saem_em_facetas`,
  `o_disco_de_imagem_nao_sai_em_facetas`, `o_interior_com_o_rebote_nao_sai_em_riscos` (cada um com o
  controlo na mesma corrida). Handoff W5d.

## 3. «As formas desenhadas da cena 6 não aparecem» (11/10, a foto antes do dono)

- **O sintoma enganava:** a luz e a sombra delas estavam lá — parecia a camada do vetor a não desenhar.
  Era a ORDEM: o chão da cena (um sprite sem `RootOrder`) desenha na frente (`root_key` = `u32::MAX`) e os
  caminhos que o editor sincroniza recebem a ordem 0 e 1 — por baixo dele.
- **A cura:** o chão com `RootOrder(0)`; o gate `a_cena_6_ensina_as_formas_desenhadas` exige-o.
