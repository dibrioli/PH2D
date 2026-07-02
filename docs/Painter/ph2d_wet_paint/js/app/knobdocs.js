// Documentação PT-BR dos knobs do painel de ajustes (spec seção 16).
// Cada entrada: texto físico ("o que este knob faz de verdade"), uma receita
// concreta, e letras de correlação com os quatro subsistemas:
//   D = deposição de tinta · A = água/secagem · F = física do fluxo · R = render
// com força 1 (fraca) a 3 (dominante). Conteúdo autoral, derivado da spec.

export const CORR_LABELS = {
  D: 'deposição', A: 'água', F: 'fluxo', R: 'render',
};

export const KNOB_DOCS = {
  pigmentPerDab: {
    doc: 'Massa de pigmento que cada toque do pincel acumula na trilha antes de pousar na tela. É o ganho bruto da tinta: dobre e cada pincelada carrega o dobro de carga, saturando a tabela de opacidade mais cedo.',
    recipe: 'Aguadas leves e transparentes: baixe para 200–350. Tinta de tubo, quase guache: 1200+.',
    corr: [['D', 3], ['R', 1]],
  },
  paperGate: {
    doc: 'A barreira do grão: quanto o vale do papel rejeita o carimbo. deposit = stamp − (1−dente)·gate — nos picos a subtração some e a tinta sempre pega; nos vales ela é recusada. É ESTA rejeição por pixel que cria a granulação.',
    recipe: 'Lavado liso sem granulação: baixe para 0.2–0.35. Textura seca e quebrada: suba para 0.8–1.0.',
    corr: [['D', 3], ['R', 2]],
  },
  felt: {
    doc: 'O piso de "feltro" entre as cerdas da textura do pincel. Quase zero por padrão (~94% da pegada não deposita nada) — é a porosidade que deixa a mancha virar uma peneira de canais úmidos, de onde nascem filetes discretos.',
    recipe: 'Lavado uniforme e cheio: suba para 0.05–0.1 (perde o rendilhado). Pincel ralo de pelos duros: mantenha 0.01 ou menos.',
    corr: [['D', 3], ['F', 1]],
  },
  bristleCount: {
    doc: 'Quantidade de pontas de cerda estampadas na textura 128×128 do pincel. Mais cerdas = pegada mais densa e contínua; menos = riscos esparsos e rajados.',
    recipe: 'Pincel velho e falhado: 300–500. Mop encharcado: 2000+.',
    corr: [['D', 3]],
  },
  drag: {
    doc: 'Arrasto de tinta: a cada janela da trilha, a célula puxa massa, água e umidade da posição da janela ANTERIOR. É o "puxão" do filme molhado sob a pincelada — cores atravessadas se esticam na direção do traço.',
    recipe: 'Traço limpo sem rastro: 0.02–0.06. Arrastão pastoso: 0.3–0.5.',
    corr: [['D', 2], ['A', 1]],
  },
  pickup: {
    doc: 'Captura do pincel sujo: fração de cor da tela que a ponta absorve ao passar sobre tinta existente (1 − retenção). Pintar através de uma poça arrasta a cor dela pelo resto do traço.',
    recipe: 'Pincel sempre limpo: 0.001. Sujeira expressiva estilo óleo: 0.02–0.05.',
    corr: [['D', 2]],
  },
  intensity: {
    doc: 'Multiplicador global da intensidade do carimbo, aplicado DEPOIS do clamp de pressão. Escala tudo que o pincel deposita sem mudar a dinâmica de pressão.',
    recipe: 'Ajuste fino de carga geral; prefira mexer aqui antes de recalibrar pressão/pigmento.',
    corr: [['D', 3]],
  },
  bristleStrength: {
    doc: 'Força (altura) das pontas de cerda na textura. Pontas mais fortes furam a barreira do grão com mais facilidade — pegada mais escura por cerda.',
    recipe: 'Cerdas macias de esquilo: 0.5. Cerda dura tipo porco: 2+.',
    corr: [['D', 2]],
  },
  bristleSize: {
    doc: 'Raio das pontas de cerda (em px de tela — a textura é ancorada 1 texel = 1 px em qualquer tamanho de pincel). Pontas maiores = grão do pincel mais grosso.',
    recipe: 'Grão fino de aquarela: 0.7–1. Pinceladas rústicas: 2–3.',
    corr: [['D', 2], ['R', 1]],
  },
  spacing: {
    doc: 'Distância em px entre toques consecutivos ao longo da spline do traço. Menor = traço mais denso (mais toques por cm, mais depósito); maior = contas espaçadas.',
    recipe: 'Deixe em 2. Suba para 4–6 para efeitos pontilhados e para aliviar CPU em pincéis enormes.',
    corr: [['D', 2]],
  },
  tipClean: {
    doc: 'Auto-limpeza da ponta: a cada janela, cada texel da ponta relaxa esta fração de volta à cor escolhida. Controla em quantas centenas de transferências o pincel sujo volta a ficar limpo.',
    recipe: 'Pincel que nunca limpa (mistura eterna): 0. Limpeza rápida: 0.01.',
    corr: [['D', 1]],
  },
  blendForce: {
    doc: 'Força da máscara saturante da ferramenta Misturar: quanto cada passada contribui para a mistura total. Esfregar o mesmo ponto acumula rumo a uma mistura completa.',
    recipe: 'Esfumado sutil: 0.2–0.4. Liquidificador: 1.0.',
    corr: [['D', 2]],
  },
  gateSaturation: {
    doc: 'Massa total (suspensa+assentada) a partir da qual a célula deixa de ler o dente real do papel e passa a ler um dente fixo 0.45 — tinta pesada enterra o grão e aceita/rejeita de forma uniforme.',
    recipe: 'Granulação persistente mesmo em camadas grossas: suba para 4000+. Cobertura rápida: 800.',
    corr: [['D', 2], ['R', 1]],
  },
  waterPerDab: {
    doc: 'Multiplicador da água que cada toque entrega (a água é inversa à pressão: toque leve = mais molhado). Alimenta tanto o pincel de pintura quanto a ferramenta Molhar.',
    recipe: 'Técnica seca: 0.3–0.6. Encharcado que escorre sozinho: 2+.',
    corr: [['A', 3], ['F', 2]],
  },
  waterCap: {
    doc: 'Teto de água por célula no DEPÓSITO (a advecção não tem teto — frentes empilham além dele e o excedente é "derramado" pela média da janela). Define quanta água uma pincelada consegue empilhar.',
    recipe: 'Poças profundas que demoram a secar: 15–20. Papel sedento: 4.',
    corr: [['A', 3], ['F', 1]],
  },
  evaporation: {
    doc: 'Multiplicador global da evaporação (escala o fator adaptativo: água calma evapora mais por passada, água corrente menos). Mais evaporação = tudo seca e assenta mais rápido.',
    recipe: 'Clima úmido, sangramentos longos: 0.3. Secador de estúdio: 3–5.',
    corr: [['A', 3], ['D', 1], ['R', 1]],
  },
  rewet: {
    doc: 'Multiplicador do reumedecimento: quanto pigmento ASSENTADO a água parada consegue levantar de volta à suspensão. O levante cresce com o excesso de água sobre o que a suspensão já ocupa.',
    recipe: 'Aquarela clássica re-trabalhável: 1–2. Tinta que mancha e não levanta: 0.1.',
    corr: [['A', 3], ['D', 1]],
  },
  retention: {
    doc: 'Retenção multiplicativa do filme por passada de secagem (0.995 = perde 0.5%). É o vazamento lento e uniforme; o escurecimento de borda entra como perda ADITIVA separada.',
    recipe: 'Papel encerado que segura água: 0.999. Mata-borrão: 0.98.',
    corr: [['A', 3]],
  },
  edgeDarkening: {
    doc: 'Evaporação extra nas BORDAS de uma mancha (células com poucos vizinhos pigmentados evaporam até ~26× mais). A água morre primeiro na borda, o pigmento assenta lá — o clássico anel escuro da aquarela. Cessa quando a célula já assentou ≥1000.',
    recipe: 'Anéis dramáticos: 100–200. Lavados uniformes sem anel: 5–15.',
    corr: [['A', 3], ['D', 2], ['R', 2]],
  },
  baseEvaporation: {
    doc: 'Termo aditivo de evaporação em toda célula molhada, mesmo no interior da mancha (a borda soma o edgeDarkening por cima). O relógio base da secagem.',
    recipe: 'Sessão longa wet-on-wet: 0.5–1. Croqui rápido: 6+.',
    corr: [['A', 3]],
  },
  leveling: {
    doc: 'Nivelamento: a água flui do grosso para o fino (diferença de filme entre vizinhos × este ganho, com trava). É o que espalha uma poça até virar lençol plano.',
    recipe: 'Poças que ficam onde caem: 0.1. Água super líquida: 1–1.5.',
    corr: [['F', 3], ['A', 1]],
  },
  capillary: {
    doc: 'Capilaridade: puxão de descida mais íngreme rumo ao papel mais baixo, só onde a tinta é fina (< barreira capilar). É o termo que canaliza a água nos sulcos do grão — filetes que SEGUEM a fibra do papel.',
    recipe: 'Escorrimento que ignora o papel: 0. Rios que desenham a textura: 0.4–0.6.',
    corr: [['F', 3], ['R', 1]],
  },
  brake: {
    doc: 'Viés do freio de absorção: o fluxo sonda alguns px à frente; chão seco (pouco filme+umidade) freia a velocidade até 5%, chão molhado deixa passar. Aplicado só 1 frame em 4 — nos outros 3 a gravidade passa livre (por isso a gota anda).',
    recipe: 'Gotas que atravessam papel seco: baixe para 0.8–1.2. Fluxo que só corre no molhado: 2–2.5.',
    corr: [['F', 3], ['A', 1]],
  },
  gravity: {
    doc: 'Magnitude do vetor de gravidade injetado (sem freio!) na velocidade persistente, escalado pelo filme local. O dial de inclinação dá direção e fração; este knob dá a escala física.',
    recipe: 'Cavalete quase deitado: 0.002. Papel vertical pingando: 0.02–0.05.',
    corr: [['F', 3], ['A', 1]],
  },
  levelClamp: {
    doc: 'Trava por eixo do termo de nivelamento — impede que um degrau brusco de filme dispare velocidades explosivas num único frame.',
    recipe: 'Deixe em 0.2. Suba com cuidado se aumentar muito o nivelamento.',
    corr: [['F', 2]],
  },
  viscosity: {
    doc: 'Limiar de filme acima do qual a célula mistura sua velocidade com a média dos vizinhos (difusão de momento). Água funda se move como corpo; filme raso se move célula a célula.',
    recipe: 'Mel espesso: 0.05 (quase tudo difunde). Água solta: 0.5+.',
    corr: [['F', 2]],
  },
  maxVelocity: {
    doc: 'Trava CFL: componente máximo de velocidade (células/frame) nos dois campos. Acima de 1 a advecção salta células e pode ficar instável.',
    recipe: 'Mantenha 1. Reduza para 0.5 se quiser tudo em câmera lenta.',
    corr: [['F', 3]],
  },
  projection: {
    doc: 'Sub-relaxação da projeção de pressão (1 iteração Jacobi a cada 3 frames). Mata o empilhamento divergente mas preserva a deriva uniforme para baixo — as gotas sobrevivem.',
    recipe: 'Fluxo mais incompressível e comportado: 0.9–1.1. Empilhamentos expressivos: 0.3.',
    corr: [['F', 3]],
  },
  brakeReach: {
    doc: 'Distância (px) da sonda do freio de absorção rio-abaixo. Sondas curtas reagem tarde (gotas afoitas); longas frenam cedo (frentes tímidas). Sonda além da borda da folha = sem freio (a gota escorre para fora).',
    recipe: 'Gotas nervosas: 2. Frentes cautelosas: 8–10.',
    corr: [['F', 2]],
  },
  capillaryGate: {
    doc: 'Massa total acima da qual a capilaridade desliga — tinta pesada não é mais puxada pelos sulcos do grão, só a aguada fina segue a fibra.',
    recipe: 'Tudo segue o grão: 6000. Só os véus mais finos: 500.',
    corr: [['F', 2]],
  },
  eraser: {
    doc: 'Força da borracha (multiplicada pelo slider de borracha e pela barreira do grão — apagar também granula). Remove suspenso, assentado e água multiplicativamente, nunca zera de um golpe.',
    recipe: 'Levantar luz suavemente: 0.15. Apagar de verdade: 0.8.',
    corr: [['D', 3], ['A', 1]],
  },
  dryer: {
    doc: 'Força do secador: multiplica o filme por (0.999 − stamp·força) e SELA o papel (umidade = 0) — sangramentos futuros param na área secada. O filme nunca chega a zero exato.',
    recipe: 'Barreira anti-sangramento: passe uma vez em 0.32. Secagem agressiva: 0.6+.',
    corr: [['A', 3], ['F', 1]],
  },
  blow: {
    doc: 'Força do sopro: injeta o deslocamento do cursor (clampado fraco e levemente anti-frente) na velocidade persistente das células molhadas e arrasta o byte de umidade. Botão direito = soprar com qualquer ferramenta.',
    recipe: 'Canudo de verdade: 0.4–0.6. Bafo suave para guiar gotas: 0.1.',
    corr: [['F', 3], ['A', 2]],
  },
  smear: {
    doc: 'Força do esfregaço: arrasta pigmento SUSPENSO e água do deslocamento anterior (amostra bilinear ponderada por massa de um snapshot). O assentado fica — igual borrar tinta fresca sobre tinta seca.',
    recipe: 'Puxar véus molhados: 0.8. Toques sutis de direção: 0.3.',
    corr: [['D', 2], ['A', 1], ['F', 1]],
  },
  paperContrast: {
    doc: 'Multiplica o desvio-padrão alvo do relevo do papel na próxima recozedura. Mais contraste = vales mais fundos → granulação e capilaridade mais fortes ao mesmo tempo.',
    recipe: 'Papel prensado a quente suave: 0.6. Torchon dramático: 1.6+.',
    corr: [['R', 2], ['D', 2], ['F', 2]],
  },
  paperFibres: {
    doc: 'Multiplica a quantidade de fibras estampadas no ladrilho do papel (milhares de sulcos curtos quase-periódicos). São os trilhos que os filetes de escorrimento seguem.',
    recipe: 'Feltro denso: 1.5–2. Papel liso de gravura: 0.3.',
    corr: [['R', 2], ['F', 2], ['D', 1]],
  },
  paperGrooves: {
    doc: 'Profundidade dos sulcos das fibras (as fibras continuam lá, mas mais rasas ou mais fundas). Fundo = granulação por rejeição mais dura e capilares mais decididos.',
    recipe: 'Grão visível mas gentil: 0.5. Sulcos que capturam tudo: 2.',
    corr: [['R', 2], ['D', 2], ['F', 2]],
  },
  visualGrain: {
    doc: 'Ganho APENAS VISUAL da granulação no render (o offset do dente impresso dentro da cor do pigmento, sumindo em massas > 1000). Não toca a física.',
    recipe: 'Pigmento liso tipo anilina: 0.3. Granulado mineral: 2.',
    corr: [['R', 3]],
  },
  emboss: {
    doc: 'Ganho do relevo assinado no render: derivada da massa total (vertical pesa 2×) vira um bisel suave — um lado clareia, o outro escurece. Só aparência.',
    recipe: 'Aguada chapada: 0. Impasto de acrílica: 2–3.',
    corr: [['R', 3]],
  },
  paperVisibility: {
    doc: 'Mestre de aparência do papel: interpola o dente em direção a um cinza plano 0.5 SÓ no render — base, granulação e bisel esmaecem juntos. A física nunca vê este valor.',
    recipe: 'Papel invisível para inspecionar a tinta: 0. Textura exagerada de scan: 1.5.',
    corr: [['R', 3]],
  },
};
