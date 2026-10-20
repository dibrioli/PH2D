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

## 6. «Quando damos o zoom e a lâmpada sai da área visível, a iluminação muda» (20/10, smoke da W6g r9)

- **O que o dono viu** (cena 5, o L rosa): com a lâmpada na vista ([foto](smoke/2026-10-20_bug6_lampada_na_vista.png))
  a sombra tem o tom quente do rebote e a face de cima do L é clara; com o zoom que tira a lâmpada da vista
  ([foto](smoke/2026-10-20_bug6_lampada_fora_da_vista.png)) a sombra fica mais escura e fria, uniforme, e a face
  do L muda. O resto do smoke («cantos vivos no padrão») passou.
- **Medido (20/10, handoff W6g §30; régua e logs FORA do git em `target/bug6/`):**
  1. **Já estava no `main`** — a mesma régua no `main` (W6a, `100d34c8e0`) e na linha: iguais ao 3.º decimal
     em toda vista. A `V2` do r9 (hipótese 3) e a 10 + 9 não entram.
  2. **A lâmpada a sair da grade NÃO é o gatilho** (hipótese 2 refutada): câmera parada, a lâmpada a andar
     para fora da grade em passos de 0,25 m — o erro contra o gabarito é contínuo nos dois zooms. Na vista do
     dono a lâmpada nem sai da grade (a grade cobre a vista com margem e oitava).
  3. **O gatilho é a OITAVA do texel** (0,0312 → 0,0156 m ao fechar o zoom): os refletores (disco, elipse,
     losango, barra) saem da grade da vista e o rebote deles passa a vir do ENTORNO. As fotos do dono, lidas
     nos mesmos pontos do mundo, mudam como a régua: a sombra junto ao L sobe R/G/B +80/+34/+32 % (linear)
     nas fotos, +67/+40/+38 % na régua.
  4. **O errado é o entorno:** a mesma vista com a grade ESTICADA sobre a sala (o mesmo texel, sem entorno)
     dá na sombra 0,223/0,115/0,181 — o valor da oitava larga (0,227/0,116/0,182); a vista + entorno dá
     0,378/0,163/0,251 (+69 %) e, abaixo da base, +141 %.
  5. **O mecanismo:** o entorno deixa a luz de FORA atravessar os opacos da vista. Cena mínima (só o L e um
     corpo que brilha fora da grade): atrás do L o gabarito e a esticada dão 0; a vista + entorno dá 0,047
     (24 % do que chega sem o L). Com o entorno 4× e 2× (em vez de 8×) a fuga cai a 0,028 e 0,018, sem
     sumir; sem o `miolo` no entorno, igual. O gate `o_rebote_de_fora_da_grade` não a via: o ponto dele
     (cena 3) vê o refletor de fora sem nada no meio.
  6. **O mecanismo (bissectado, handoff W6g §32):** o entorno TAPA (a c0 dele é 0 atrás do L); a fuga é o REBOTE
     da vista — a soma da c0 do entorno (sampler `Linear`, sem o alfa) leva a luz do INTERIOR do L aos texels de
     fora junto à face de trás, e o rebote acende essa face (sem o rebote da vista: 0). E há uma 2.ª, do método:
     uma parede de 0,03 m (¼ do texel do entorno) deixa passar 90 % da luz de fora.
- **Curado (20/10, handoff W6g §33):** a luz de fora ENTRA PELA BORDA da grade da vista (o R do entorno no lugar do
  céu, de uma 2.ª passagem dele com a vista em vácuo, guardado só sob a janela da vista) e os opacos dela tapam-na:
  atrás do L 0,047 → 0, atrás da parede de 0,03 m 0,166 → 0; a cena 5 na troca de oitava, sombra +69 %/+141 % →
  −14 %/−11 % contra a grade esticada (o resto é o rebote dos refletores de fora, ver §34.2). Gates
  `a_luz_de_fora_nao_atravessa::*` com prova de mutação (6/6). Custo: +0,6…+3,2 % de memória; +0,36 ms na vista
  e +0,32 ms no entorno em 1920×1080.
- **Smoke do dono (20/10, [foto 1](smoke/2026-10-20_bug6_cura_foto1.png), [foto 2](smoke/2026-10-20_bug6_cura_foto2.png)):
  «sombra ok. laranja (influência do objeto superior) diminui».** A sombra está curada; o que sobra é o REBOTE do
  refletor de cima quando ele sai da grade da vista e passa ao entorno — o mesmo −10…−14 % de vermelho que a régua
  já media nas faces e na sombra contra a grade esticada (handoff W6g §34.2).
- **O laranja curado (20/10, handoff W6g §35):** a 1.ª passagem do entorno (a fonte do rebote dos refletores de fora)
  ainda mascarava a vista — herança da soma da W5d — e o Disco recebia só metade da luz que a vista lhe devolve. Pela
  borda só o arquivo leva a vista em vácuo; a parte do Disco 0,66–0,76 → 0,90–1,03 da esticada, a cena 5 −14 % →
  +1/−5 % (o resto é a grossura 8× do entorno). Gate apertado a ±7 % com prova de mutação. Por fazer: o smoke do dono.
