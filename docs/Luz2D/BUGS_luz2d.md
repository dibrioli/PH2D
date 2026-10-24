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
  +1/−5 % (o resto é a grossura 8× do entorno). Gate apertado a ±7 % com prova de mutação. **Smoke do dono (20/10): «smoke ok» — o bug 6 está FECHADO.**
- **O resto (−4 %) curado pela grossura (20/10, handoff W6g §37, por ordem do dono):** o erro segue o texel do entorno
  em METROS e só a Baixa a 8× era grossa demais; o entorno da Baixa passa a 4× (Média e Alta 8×): a cena 5 −4,0…−4,7 % →
  −1,2…−1,4 %, o emissor encostado à borda +11 % → ±1,6 %, por +0,48 ms a 1080p. Gates 0,025 e 0,03, mutação M8.

## 7. «Numa placa integrada, 7 testes da Luz 2D reprovam» (23/10, achado da `line/Velocidade`) — FECHADO (24/10, W6i)

- **O achado:** o `main` em `972f411a6f` (sem a W6h) só na Radeon 610M do 9950X (RADV / radeonsi; o `nvidia-smi`
  sem nenhum processo da corrida): 5 573 / 5 580 verdes, os 7 vermelhos todos da Luz 2D, os mesmos 7 verdes na
  RTX 5060 Ti (junit do ship 23/10 05:28). A Luz 2D corre ~5,6× mais lenta lá (2 004 s contra 358 s na
  `ph2d-app-light2d`, a NVIDIA com `load` ~40: ordem de grandeza).
- **Três famílias, por separar:**
  1. *O mesmo bit que só vale na NVIDIA:* `a_descida_sem_o_t_nulo_nao_muda_um_bit` (19 texels, Baixa 1080p) e
     `o_nivel_0_por_um_cone_nao_muda_um_bit` (495). Hipóteses por medir: a FMA do driver (o mecanismo do W6g §53 b —
     a RADV contrai `a·T.r + T.t·R` noutro sítio) ou o subgrupo 64. A lei «não muda um bit» é por dispositivo ou
     de toda a placa: decide-se e o teste di-lo.
  2. *A memória:* `g5_o_que_a_placa_alocou…` — `luz2d/buf/hrc_r0` com enchimento 524 288 contra o teto 3 072 (um
     alinhamento do adaptador?). Régua ou o produto a reservar 170× numa integrada: medir primeiro.
  3. *Precisão ou lei de custo:* `a_holografica_junto_a_uma_face_e_o_gabarito` (0,0042), `…_nao_muda_com_o_pan`
     (0,00072), `g9_o_rebote_na_holografica` (0,0154), `o_custo_do_miolo_desce_com_a_base` (1,001 contra 0,8).
- **Reprodução sem a NVIDIA:** o bloco do prompt da `line/Velocidade` (as variáveis `VK_DRIVER_FILES` /
  `VK_ICD_FILENAMES` / `__EGL_VENDOR_LIBRARY_FILENAMES` / `__GLX_VENDOR_LIBRARY_NAME` para a Radeon + `PH2D_GPU_VEZ`
  numa pasta com `pronto` — só COM as variáveis); conferir pelo exemplo `o_que_a_placa_anuncia` (só `RADV
  RAPHAEL_MENDOCINO` e `radeonsi`). Material bruto copiado para `target/radeon_achado/` (nextest e os dois junit).
- **Consequência:** a Luz 2D fica presa à NVIDIA na fila da placa até isto fechar; quando passar na Radeon, o
  handoff di-lo. ⚠️ Toda cura «de mesmo bit» desta linha (a da borda do W6h §13 incluída) prova-se também na Radeon.
- **Os mecanismos MEDIDOS (23/10, W6i, `ac7366c6a0` + os instrumentos `bug7_*`, `--ignored`; saídas em
  `target/radeon_w6i/`).** Reproduzidos os 7 na Radeon (só RADV e radeonsi visíveis; zero processos desta árvore no
  `nvidia-smi`); a cura da borda (W6h §13, `a_leitura_da_borda_sem_desvio_nao_muda_um_bit`) PASSA lá, 5/5 vistas.
  1. *O mesmo bit:* o produto contra ele mesmo dá 0 texels nas duas placas (determinista); as 19 e 495 diferenças
     são TODAS de 1 ULP do f16; com `ACO_DEBUG=noopt` (o otimizador do compilador da AMD desligado, o mesmo binário)
     caem a **0 e 0**. ⇒ é a reescrita da conta pelo compilador (contração/reassociação), que o WGSL permite; a
     NVIDIA dá 0 por acaso do compilador dela, não por lei.
  2. *A memória:* com a holográfica em textura, os `luz2d/buf/hrc_*` são TEXTURAS `Rg32Uint` (o rótulo é o dos
     buffers que substituem); o `hrc_r0` é 8192 × 8 = 524 288 B e a RADV dá-lhe 1 MiB (a altura de 8 sobe a 16 no
     ladrilho dela); as T de 16 linhas cabem exatas. E não é só ele: na Radeon quase TODA textura passa o teto de
     3 072 (`cantos`/`vertices` +65 536, `miolo_*` +43 776, `emissao` +65 296, constantes entre as grades; `borda`
     +32 768 a +57 344 varia) — o `hrc_r0` é só o 1.º por ordem alfabética. Um buffer de 512 KiB sozinho sai exato
     nas duas placas (o relatório soma o vão de alinhamento ao tamanho da peça: `aligned_size = padding + size` no
     `gpu-allocator`). ⇒ a RÉGUA (o teto é o da NVIDIA) e não o produto a reservar 170×; o `hrc_r0` desperdiça
     512 KiB na AMD (forma da textura).
  3. *A holográfica:* `pack2x16float` arredonda PARA ZERO na RADV (32 768/32 768 a ¾ de ULP) e para o MAIS PRÓXIMO
     na NVIDIA (32 768/32 768); a escrita numa textura `Rgba16Float` é para zero nas DUAS. Os T e R da holográfica
     guardam-se por `pack2x16float` ⇒ na Radeon cada nível trunca para baixo: o G9 sai TODO negativo, −1,2 a −1,6 %
     no Alto e no Médio contra −0,6 a −1,0 % na NVIDIA (o viés extra, do mesmo sinal, é o arredondamento).
  4. *O custo do miolo:* na Radeon `c3_miolo` ~10,7 ms com a base contra ~10,9–11,0 sem ela (razão 0,97–1,02 em 3
     corridas; controlo 1,20–1,62, `load` 27–140: a régua tem ruído, mas a razão não desce em nenhuma); a base corta
     os despachos 605 → 247 e o tempo não se move ⇒ na integrada o miolo é TRABALHO, não despachos (a lei W5o é da
     NVIDIA, 0,72 contra 1,07 ms).
- **A cura (W6i, commits `2e7bfd84b9` · `a047753a1b` · `c46ae5b4ff`).** Os 7 passam na Radeon (RADV) e continuam a
  passar na RTX 5060 Ti; bateria de 23 testes verde nas duas placas (holográfica inteira, G5, custo do miolo, os 5
  «mesmo bit» entre formas, a vista renasce, instrumentos `bug7_*`).
  1. *Mesmo bit (família 1).* Decisão do dono 24/10 («A»): a lei é por placa. `comum::mesmo_bit_por_placa` (crate
     `ph2d-app-light2d`, testes) exige 0 texels, salvo numa placa cujo compilador provadamente reescreve a conta
     (RADV: as diferenças morrem com `ACO_DEBUG=noopt`), onde exige ≤ 1 ULP do f16; aplicada aos 5 testes entre
     formas (borda sem desvio, recorte, quadrantes juntos, um cone, T nulo). Radeon: até 88 texels a 1 ULP; NVIDIA:
     0 em todos. A «vista renasce» (o mesmo programa duas vezes) continua 0 em toda placa.
  2. *G5 (família 2).* O teto 3 072 só onde a placa arranja em linha (`arranjo_linear`: textura 8192×8 `Rg32Uint`
     sozinha — NVIDIA 525 024 B, RADV 1 052 432). Lei nova em toda placa: a sobra varia menos que
     max(1 024 B, 1/8 da variação do declarado) (`FRACAO_DA_VARIACAO`); o controlo (pirâmides sem a cauda dos
     mips) reprova nas duas placas.
  3. *Holográfica (família 3).* `Config::hrc_meio_float_mais_proximo` / `override MAIS_PROXIMO_HRC` / `meio_float`
     em `c2_hrc.wgsl`: f16 ao mais próximo (empate ao par), comparando o resultado do pack com o vizinho de cima.
     Ligada só onde a placa trunca: `ph2d_light2d_gpu::o_pack_da_placa_trunca` (`src/arredondamento.rs`) mede uma
     vez por adaptador; o `Solver` desliga-a na config efetiva onde a placa já arredonda (o mesmo bit). Viés do G9
     na Radeon: Alto −1,42 → −0,89 % (NVIDIA −0,87 %). Custo do c2 (mínimo de 9, intercalado, com controlo): sempre
     ligada custava +6–13 % na RTX (recusado, critério 2 %); com a prova por placa, RTX razão 0,996–1,001, Radeon
     +9–10 %. Portão `holografica::o_f16_da_holografica_e_o_mais_proximo_em_toda_placa`; prova de mutação 4/4
     sangram (nunca liga → Radeon; liga sempre → NVIDIA; prova invertida → NVIDIA; shader sem correção → Radeon,
     o controlo).
  4. *Custo do miolo (família 4).* `custo_de_despachos_ms` mede na própria corrida o que custam os despachos que a
     base corta; a lei ≤ 0,8 só onde valem ≥ 20 % do miolo (RTX 27–29 %, razão 0,708; Radeon 2 %: o teste diz «não
     se aplica»); `a_base_do_miolo_corta_os_despachos` continua em toda placa.
  5. *Reprodução.* O wrapper da Radeon (`VK_DRIVER_FILES`/`VK_ICD_FILENAMES=radeon_icd.json`,
     `__EGL_VENDOR_LIBRARY_FILENAMES=50_mesa.json`, `__GLX_VENDOR_LIBRARY_NAME=mesa` + `PH2D_GPU_VEZ` numa pasta
     falsa com `porta` FIFO, `porteiro` vivo e `pronto`), corrido DENTRO de `bash scripts/ph2d-run.sh` (o porteiro
     real nunca é pedido). ⚠️ Scripts soltos em `target/` e no scratchpad desapareceram duas vezes durante a W6i —
     guarde-os fora de `target/`.
  6. *Consequência:* a Luz 2D pode correr na fila da placa onboard.
- **O miolo na integrada (item 4, 24/10; réguas `medida_luz2d_miolo_por_placa` e `medida_luz2d_miolo_formas`,
  `--ignored`; corridas `docs/Luz2D/ferramentas/item4_formas.sh`).** As 8 fases do instrumento W5o, mínimo de 9
  intercaladas, ms (Radeon 610M RADV · RTX 5060 Ti), a cena do bug 7 (`o_custo_do_miolo`) | a vista vazia Low 1080p:

  | fase (despachos) | Radeon bug 7 | RTX bug 7 | Radeon vazia | RTX vazia |
  |---|---|---|---|---|
  | classifica+puxa (10) | 1,398 | 0,076 | 1,156 | 0,064 |
  | cascata: mais grosso (1, a base) | 0,086 | 0,063 | 0,036 | 0,030 |
  | cascata: desce (58) | **3,415** | 0,267 | **3,124** | 0,240 |
  | ciclo: defeito+restringe (21) | 1,508 | 0,092 | 1,096 | 0,077 |
  | ciclo: mais grosso (3, a base) | 0,071 | 0,061 | 0,063 | 0,055 |
  | ciclo: sobe (57) | 1,834 | 0,177 | 1,657 | 0,161 |
  | ciclo: soma (6, todos no nível 1) | **2,336** | 0,081 | **2,194** | 0,078 |
  | final (1) | 0,480 | 0,023 | 0,359 | 0,013 |
  | `c3_miolo` inteiro | 11,28 | 0,82 | 12,12 | 0,71 |

  ⇒ na Radeon o tempo segue o TAMANHO do nível, não os despachos: a base (um grupo só, níveis minúsculos) custa o
  mesmo nas duas placas (×1,1–1,4), e os 6 despachos da soma (nível 1 = 1280×768 inteiro) custam 0,39 ms cada
  (×29 a RTX). ~40 MB por despacho do nível 1 em 0,39 ms ≈ 100 GB/s: a largura da memória partilhada; na RTX os
  16 MB de cada textura cabem nos 32 MB da L2. E o nível acima da fina resolve-se INTEIRO (W5n), quase todo fora
  dos opacos (o disco do bug 7 é ~4 % do nível 1; a vista vazia, 0 %).
  - *Forma recusada — o ladrilho* (`docs/Luz2D/ferramentas/item4_recusado/c3_miolo_ladrilho.wgsl`: as vizinhas das
    relaxações e das bilineares lidas uma vez por grupo na memória dele, ~9 → ~3 leituras por texel, 0 texels
    diferentes): RTX ×1,53–1,68, Radeon ×1,47–2,40. A hipótese (limitada pelas leituras de textura) errada: a cache
    já as reaproveita; o que pesa é a DRAM.
  - *Forma que entra — saltar o fora* (`Miolo::salta_o_fora`, `override SALTA_FORA` do `c3_miolo.wgsl`): os passes
    dos níveis ≥ 1 não gravam o texel com `k.a < 0` (fora dos opacos: toda leitura da solução, do defeito e do
    erro pergunta antes ao conhecido; o anel `k.a > 0` continua a gravar o seu 0). `c3_miolo`, mínimo de 9, antes
    → depois: RTX bug 7 0,824 → 0,767 (×0,93) · vazia 0,700 → 0,644 (×0,92) · cena 5 Low 0,776 → 0,717 (×0,92), A × A
    1,000; Radeon 11,28 → 6,22 (×0,55) · 9,91 → 5,81 (×0,59) · 13,14 → 5,46 (×0,42), A × A 0,79–1,46 (`load` 30–47:
    a forma fica abaixo do mínimo das duas cópias do antes nas três). Bits: 0 texels diferentes nas 3 cenas e nas duas
    placas; controlo (um V-ciclo a menos) 155 496 e 43 913 texels (a vista vazia não tem miolo: 0). Ligada em toda
    placa; `PH2D_LIGHT2D_SALTA_FORA=0` volta a gravar. Portão `o_custo_do_miolo::saltar_o_fora_do_miolo_nao_muda_um_bit`
    (0 texels em toda placa — a conta é a mesma).
