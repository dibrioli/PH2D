# HANDOFF — Janela 7 (30/10), linha `line/PainterWatercolor` reaberta sobre o main `422fec2027`

## Feito (commits)
- `0d18a1e9ad` Bug 3: a COR do papel fora do Desfazer (só a separação do branco é passo).
- `cd07881b37` Bug 1 / BUGS #53: `assenta::na_caixa_das_tintas` no render (molhado sobre tinta fina não clareia num papel de cor); 11 miniaturas ≤ 2 níveis.
- `b1b0cdb513` censo `nenhum_pincel_de_fabrica_clareia_o_papel_de_cor` (66 presets, ~2 s; catraca `ABERTOS` = Marker Bullet, BUGS #54).
- `9318d76827` + `a038497be8` Bug 2 / BUGS #55: `solver::ladeira_do_oleo` (a altura do óleo assentado no nivelamento, só com pincel SEM Real Oil); mutação 6/6.
- `dc3f2add9e` sonda `diag_aguada_por_cima_do_oleo` (4 variantes: `scratchpad/quatro.sh` — reescreva-a, o scratchpad morre).

## Smoke do dono (30/10, depois destes commits)
1. **A queda da água no relevo: aprovada** («muito interessante e realista»). Fica.
2. **A mistura piorou aos olhos dele:** «antes a tinta e o relevo derretiam e se mesclavam com a aguada; agora a aguada
   lava a tinta do relevo, não afeta o relevo e deixa o relevo sem cor». Pedido: **o melhor dos dois mundos — a aguada
   dilui um pouco a tinta grossa, mistura as cores e escorre correctamente** (a ladeira).
   MEDIDO (sonda das 4 variantes): sem a ladeira o óleo INTEIRO fica pálido ao assentar (o «derreter» que ele via);
   em todas, a meio, amarelo sobre azul sai CREME — a `Ecra::Cobre` (`assenta::cor_e_alfa_tapando`) pinta o tapar extra
   com a tinta DILUÍDA EM BRANCO (`c_eff`), calibrado no Rebelle sobre papel branco. Sobre o óleo isso é um véu branco.
   Hipótese a SIMULAR antes de construir (REGRAS_DE_CUSTO §Metodologia): sobre célula com óleo, a água MISTURA (K-M / o
   Glaze da célula) em vez de tapar com branco, e levanta parte do pigmento do óleo (rewet) para escorrer pela ladeira;
   a mesma lei no render e no `assenta` (drying), por célula (`Grid::oleo`). Fotos antes/depois ao dono antes de comitar.
   Neutro ao bit nos aprovados sem óleo por baixo (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`).
3. **Grain Colors:** o claro é a rampa em modo RGB (meio de azul→amarelo = cinza). O dono pergunta onde trocar (o
   seletor «RGB» na linha da rampa, ao lado de «Linear») e propõe **OKLCH por omissão** — produto: SIM para rampas NOVAS
   do Painter. Cuidado: `ColorRamp::default()` é partilhado (ph2d-color, outros módulos) e `tool/paint/ramp.rs:159` grava a
   rampa só se ≠ omissão — um preset antigo sem rampa leria a omissão nova. Faça a omissão do PAINTER (state_default.rs
   259/265, ramp.rs:97/159, layers/texture.rs, shape_camadas.rs) e prove a leitura dos presets antigos ao bit.
4. **#54 Marker Bullet** (assentado guarda branco opaco): cura simulada muda 25 miniaturas até 10 níveis — decisão do dono pendente; provavelmente cai junto com o item 2 (mesma conta `Cobre`).
