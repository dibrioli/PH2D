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

## 4. «A 4K a luz apaga sem aviso» (11/10, medido na W5e)

- **O sintoma enganava:** parecia o governador, o nível ou falta de memória. Era a captura FINA (2× mais
  larga que a fluência) a passar o `max_texture_dimension_2d` 8 192 do dispositivo da casa a 2160p (10 240 /
  9 984): erro de validação em `luz2d/tex/cobertura_fina`, nos três níveis — a luz a calcular nada.
- **Por que nenhum gate o via:** todas as réguas eram a 1080p ou em grades de teste; o solver só conferia
  a largura da fluência, nunca a das outras texturas.
- **A cura:** `Config::cabe_na_placa` em toda textura da `PassList`; a fina que cede (`teto::fina_que_cabe`);
  o nascimento do `Solver` sob os escopos OutOfMemory + Validation (`ErroSolver::SemMemoria`). Gates:
  `os_tetos_da_placa` (6, com o controlo do degrau da W5d e a placa a recusar de verdade). Handoff W5e.

## 5. «Ao baixar o Opacity do azul, o laranja da sombra desapareceu — nem sempre» (12/10, cena 7)

- **O sintoma enganava:** parecia a sombra do laranja (o rebote dele que chega à sombra do azul). Era o
  CORPO do azul: «interior de opaco» era IGUALDADE a opacidade 1, nos dois lados (placa e gabarito). A 0,999
  o interior mostrava a luz transmitida (× 0,001: PRETO, 0,0003 contra 2,70) e o albedo caía a zero (o
  teal do azul sumia do anel e da sombra). O «nem sempre»: as duas bolas de cima são UM objeto Flip.
- **E um segundo salto, escondido pelo 1.º:** com a lei do interior já contínua, a face a 0,999 ainda
  caía −14 % no G/B. A cascata holográfica (`hrc_soma`) lê o texel na sonda «+1» adiante, salvo se ela
  cai num OPACO (`τ = 0`); a sonda num translúcido a 0,999 contava como «fora» e ainda multiplicava pelo
  τ dela — o texel colado à face lia o interior × 0,001 (o −43 % que a W5c mediu e curou só no opaco). O
  borrão em cruz (`hrc_saida`) misturava os dois lados da borda translúcida pela mesma razão.
- **Por que nenhum gate o via:** os portões do translúcido afirmavam a lei velha (`o_translucido_nao_
  reflete*`, «um vidro continua a atenuar por dentro») e nenhum varria a opacidade junto a 1.
- **A cura (W5k):** mostrado = `mix(transmitida, exterior/miolo, α)`, albedo `α·a`, o predicado
  geométrico «dentro de um corpo» = `τ < 1` (`num_corpo`); a sonda adiante e o vizinho do borrão pesam o
  τ (contínuos; em τ ∈ {0, 1} o bit de antes). Gates: `a_luz_e_continua_na_opacidade_do_oclusor` (a
  cena do dono), `o_interior_e_continuo_na_opacidade` e `um_vidro_mistura_a_transmitida_e_o_exterior_pela_
  opacidade` (gabarito), `o_translucido_reflete_a_sua_parte*` (gabarito e placa). Handoff W5k.
- **O peso τ era meia-cura (W5m):** a sonda de DENTRO de um corpo é o campo de dentro, também num
  translúcido — pesá-la por τ junto à face puxava a borda de um vidro a ½ −15 % (o g3 regrediu 0,0074 →
  0,0100) e, no rebote, perdia τ·(o brilho da parede) por iteração (a sala a 0,99 −1,25 %; o gabarito
  +0,14 %). Peso 0 nos três sítios (`peso_de_fora` da C3, o «+1» e o borrão da holográfica): g3 0,0065,
  a sala 0,08 %, bit a bit em α = 1, salto em α → 0 de 0,012 % do máximo. Handoff W5m.
