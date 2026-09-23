#!/usr/bin/env bash
# Provas de mutação da PARALAXE (plano 24, W1) — o número (`ScrollFactor`).
#
# Arnês IDÊNTICO ao das waves anteriores desta linha — controlo sobre o próprio FILTRO (um filtro
# que casa ZERO testes imprime `ok` e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora
# não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_paralaxe_2026-09-22.sh
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; MUTADO="$1"; MUTADO_TAG="$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; MUTADO=""; }

# ⛔⛔ **A REDE, e ela nasceu de um defeito MEDIDO (2026-09-22):** canalizar este arnes por `head`
# mata-o com SIGPIPE **entre o `muta` e o `restaura`**, e o produto fica MUTADO na arvore. A
# corrida seguinte leu «ancora aparece 0 vezes» — o arnes a ser honesto —, mas uma que nao tocasse
# naquela ancora teria corrido a suite inteira sobre codigo mutado e chamado ao resultado verde.
# ⚠️ *Um arnes que restaura no caminho feliz nao restaura; ele restaura quando NADA corre mal.*
MUTADO=""
MUTADO_TAG=""
ao_sair() {
  if [ -n "$MUTADO" ]; then
    echo "⚠️  interrompido com $MUTADO mutado — a restaurar"
    cp "$TMP/$(basename "$MUTADO").$MUTADO_TAG" "$MUTADO"; touch "$MUTADO"
  fi
}
trap ao_sair EXIT INT TERM PIPE

# ⛔⛔ **O GUARDA CONTRA O CRASE, e ele nasceu de o mesmo defeito morder DUAS vezes nesta sessao:**
# um `` ` `` dentro de `"..."` abre substituicao de comando, engole ate' ao crase seguinte e desfaz
# a citacao do RESTO do ficheiro — e o `bash` acusa `erro de sintaxe` cinquenta linhas abaixo, num
# bloco intocado. ⚠️ Escrever «a`» por «à» e' o habito que o produz.
if grep -n '^bloco "[^"]*`' "$0" >&2; then
  echo "⛔ CRASE no nome de uma prova (ver as linhas acima): ele desfaz a citacao do ficheiro." >&2
  exit 2
fi

muta() { # ficheiro vezes antigo novo
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read()
c = s.count(old)
if c != n:
    sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() { # nome crate filtro [alvos]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  # shellcheck disable=SC2086
  out=$(timeout 900 cargo test -p "$2" ${4:---all-targets} -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome crate filtro ficheiro vezes antigo novo [alvos]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

LEI=crates/ph2d-ecs/src/scroll_factor.rs
PONTE=crates/ph2d-app-components/src/parallax_bridge.rs
LEDGER=crates/ph2d-preview-drive/src/lib.rs
FASE=shells/desktop/src/render_loop/fase_paralaxe.rs
QUADRO=shells/desktop/src/render_loop/fase_frame_open.rs

echo "=== A LEI: o declive e' 1 − k (a tabela MEDIDA no alvo) ==="

# ⛔ O erro que a sonda do oráculo apanhou em MIM: a coluna do «veredito» comparava o declive com o
# `scroll_scale`, e a identidade e' `1 − scroll_scale`. Escrita assim, `k = 0` deixaria o fundo
# PARADO no mundo (o oposto de um fundo de paralaxe) e `k = 1` colava-o a` camera.
bloco "o declive vira k e nao 1 − k" ph2d-app-components o_declive_e_um_menos_k \
  "$LEI" 1 \
  '            centro[0] * (1.0 - self.k[0]),' \
  '            centro[0] * self.k[0],'

bloco "os dois eixos partilham o k do x" ph2d-app-components os_dois_eixos \
  "$LEI" 1 \
  '            centro[1] * (1.0 - self.k[1]),' \
  '            centro[1] * (1.0 - self.k[0]),'

# ⛔ A referencia deixa de ser a ORIGEM DO MUNDO e passa a ser a propria peca — a forma do Flip,
# que TELEPORTA para o centro da vista um fundo autorado num canto (ver o cabecalho da lei).
# ⭐⭐ Depois da W2 a lei NAO consegue exprimir a forma do Flip — a `deslocamento(centro)` nao ve' a
# pose autorada, e a assinatura proibe-a. Ela so' pode voltar por quem COMPOE, e e' la' que a
# mutacao mora.
bloco "a referencia vira a propria peca (a forma do Flip)" ph2d-app-components o_deslocamento_nao_depende \
  "$PONTE" 1 \
  '        let d = cfg_d.deslocamento_confinado(centro, conf);' \
  '        let d = cfg_d.deslocamento_confinado([centro[0] - autorada.translation.x, centro[1] - autorada.translation.y], conf);'

bloco "o neutro deixa de ser 1" ph2d-app-components o_neutro \
  "$LEI" 1 \
  '    pub const NEUTRO: [f32; 2] = [1.0, 1.0];' \
  '    pub const NEUTRO: [f32; 2] = [0.0, 0.0];'

echo "=== A PONTE: quem e' conduzido, e a partir de QUE pose ==="

# ⛔ Sem o salto do neutro, toda cena com o componente anexado passa a ter uma entrada viva no
# ledger — a captura paga uma varredura por nada, e o `n` que o painel mostra mente.
bloco "o neutro passa a ser declarado" ph2d-app-components o_neutro_nao_escreve \
  "$PONTE" 1 \
  '        if cfg.e_neutro() && mov.is_none_or(|m| m.e_inerte()) {' \
  '        if false {'

# ⛔ Sem camera de jogo a resposta e' NAO DECLARAR: uma corrida que dependesse de onde o artista
# rolou o ecra' seria outra corrida em cada maquina.
bloco "sem camera a vista vira a origem" ph2d-app-components sem_camera_de_jogo \
  "$PONTE" 1 \
  '    let Some((centro, meia)) = vista else {' \
  '    let Some((centro, meia)) = vista.or(Some(([0.0, 0.0], [0.0, 0.0]))) else {'

# ⛔ A populacao: um HUD ja' e' conduzido pela ponte dele, e dois motores sobre o mesmo `Transform`
# escrevem um por cima do outro.
bloco "o HUD entra na populacao" ph2d-app-components um_hud_nao_e_tocado \
  "$PONTE" 1 \
  '            .query_filtered::<(Entity, &Transform, &ScrollFactor, Option<&ScrollRepeat>, Option<&ScrollLimits>, Option<&ScrollMotion>), bevy_ecs::prelude::Without<UiCanvas>>()' \
  '            .query::<(Entity, &Transform, &ScrollFactor, Option<&ScrollRepeat>, Option<&ScrollLimits>, Option<&ScrollMotion>)>()'

# ⛔ Esta ponte so' escreve a TRANSLACAO: roubar o resto ao autorado apagaria o que outro motor
# tivesse escrito no mesmo quadro.
bloco "a rotacao autorada e' apagada" ph2d-app-components so_a_translacao \
  "$PONTE" 1 \
  '            // skew, e roubá-los ao autorado apagaria o que outro motor tivesse escrito.
            ..era
        };' \
  '            ..Transform::default()
        };'

echo "=== O LEDGER: o autorado, e o deslocamento MEDIDO ==="

# ⛔⛔ O defeito que EU shipei por uma hora: declarar o VIVO como `before`. Ele escreve a pose
# DESLOCADA no documento, e no quadro seguinte o fundo salta o deslocamento outra vez.
bloco "o before do ledger vira o VIVO" ph2d-app-components arrastar_o_fundo \
  "$PONTE" 1 \
  '                Driven::ParallaxPose(autorada),' \
  '                Driven::ParallaxPose(era),'

# ⛔ Sem a manutencao da conducao, um quadro em que a vista nao andou faz a pre-visualizacao virar
# DOCUMENTO — esta ponte e' um condutor PERSISTENTE (a lei do `still_driving`).
bloco "a conducao nao e' MANTIDA" ph2d-app-components arrastar_o_fundo \
  "$PONTE" 1 \
  '        if escreveu || drive.still_driving(entity, Driver::ParallaxPose) {' \
  '        if escreveu {'

# ⛔⛔ A 1.ª redacção RE-DERIVAVA o deslocamento da vista de AGORA; aqui a recuperacao some de vez,
# e o autorado passa a ser a pose deslocada (a queixa «o fundo saltou» do arrasto).
bloco "a recuperacao do autorado some" ph2d-app-components arrastar_o_fundo \
  "$PONTE" 1 \
  '            era.translation.x - (escrito.translation.x - memo.translation.x),' \
  '            era.translation.x,'

# ⛔⛔ E o mesmo defeito visto do LEDGER: sem o `last_written` o deslocamento medido e' ZERO, e o
# declive volta a ler o que a 1.ª redacção lia.
bloco "o last_written devolve o AUTORADO" ph2d-app-components o_declive_e_um_menos_k \
  "$LEDGER" 1 \
  '        self.memo.get(&(entity, driver)).map(|e| e.last_written)' \
  '        self.memo.get(&(entity, driver)).map(|e| e.authored)'

echo "=== A REPETICAO INFINITA (W2) ==="

REP=crates/ph2d-ecs/src/scroll_repeat.rs

# ⛔⛔ A lei inteira: somar um RESTO em vez de um INTEIRO de ladrilhos. E' o que faz o erro de `f32`
# acumular, e ao decimo milesimo ladrilho a costura esta' aberta.
bloco "a correccao vira um RESTO" ph2d-app-components a_correccao_e_um_numero_inteiro \
  "$REP" 1 \
  '    d - tile * (d / tile).round()' \
  '    d - tile * (d / tile)'

# ⛔ A JANELA e' a nossa divergencia DECLARADA (centrada), e ela e' load-bearing: com `floor` o
# valor corrigido cresce sempre para um lado e as reguas de `± t/2` deixam de descrever a lei.
bloco "a janela deixa de ser centrada" ph2d-app-components a_repeticao_nao_envolve \
  "$REP" 1 \
  '    d - tile * (d / tile).round()' \
  '    d - tile * (d / tile).floor()'

# ⛔ O `0` e' a AUSENCIA e nao um erro — sem a guarda ele divide por zero e a pose vira `NaN`.
bloco "o ladrilho ZERO passa a corrigir" ph2d-app-components um_ladrilho_zero \
  "$REP" 1 \
  '    if !tile.is_finite() || !d.is_finite() || tile <= 0.0 {' \
  '    if false {'

# ⛔ A composicao morre: o componente existe, tem lei, tem gates — e a ponte ignora-o.
bloco "a ponte ignora o componente" ph2d-app-components a_fase_e_a_mesma \
  "$PONTE" 1 \
  '        let d = rep.map_or(d, |r| r.envolve(d));' \
  '        let d = { let _ = rep; d };'

# ⛔⛔ A repeticao envolve a SOMA e nao o DESLOCAMENTO ⇒ ela envolve tambem a pose que o artista
# autorou, e o fundo salta para a origem assim que ele o arrasta para alem de meio ladrilho.
bloco "a repeticao envolve a POSE somada" ph2d-app-components a_repeticao_nao_envolve \
  "$PONTE" 1 \
  '        let d = rep.map_or(d, |r| r.envolve(d));' \
  '        let d = { let p = [autorada.translation.x + d[0], autorada.translation.y + d[1]];
            let p = rep.map_or(p, |r| r.envolve(p));
            [p[0] - autorada.translation.x, p[1] - autorada.translation.y] };'

# ⛔⛔ **A MUTACAO «a desloca deixa de delegar» MORREU com a premissa dela, e fica registada:** ela
# defendia a delegacao entre `desloca` e `deslocamento` (duas respostas a` mesma pergunta). A W2
# fez a `deslocamento` ser a unica lei e a `desloca` ficou **sem um chamador de produto** — `pub`
# numa crate de BIBLIOTECA e' API, e API sem chamador e' divida ⇒ foi apagada. *A delegacao deixou
# de poder divergir porque um dos dois lados deixou de existir*, que e' mais forte que a mutacao.

echo "=== O CONFINAMENTO (W3) ==="

LIM=crates/ph2d-ecs/src/scroll_limits.rs

# ⛔⛔ O JOELHO deixa de descontar a meia-vista ⇒ ele poe-se na borda da REGIAO em vez de onde a
# borda da VISTA a alcanca, e o fundo mostra meia tela de borda antes de congelar.
bloco "o joelho ignora a meia-vista" ph2d-app-components o_joelho_esta_onde \
  "$LIM" 1 \
  '    let (lo, hi) = (min + meia, max - meia);' \
  '    let (lo, hi) = (min, max);'

# ⛔ O congelamento morre: o segundo termo da lei some e a camada faz a paralaxe autorada para
# sempre — a borda do fundo entra em cena, que e' o que a wave existe para impedir.
bloco "o congelamento some" ph2d-app-components a_curva_do_confinamento \
  crates/ph2d-ecs/src/scroll_factor.rs 1 \
  '            d[0] + self.k[0] * (centro[0] - confinado[0]),' \
  '            d[0],'

# ⛔⛔ A FORMA da composicao: `centro − k·confinado` da' a MESMA curva e perde a identidade ao bit
# com a W1/W2 (`c − k·c` e `c·(1 − k)` diferem por um ULP em `f32`).
bloco "a forma deixa de ser byte-identica a lei da W1" ph2d-app-components a_composicao_do_confinamento \
  crates/ph2d-ecs/src/scroll_factor.rs 1 \
  '        let d = self.deslocamento(centro);
        [
            d[0] + self.k[0] * (centro[0] - confinado[0]),
            d[1] + self.k[1] * (centro[1] - confinado[1]),
        ]' \
  '        [
            centro[0] - self.k[0] * confinado[0],
            centro[1] - self.k[1] * confinado[1],
        ]'

# ⛔ A regiao VAZIA deixa de ser a omissao ⇒ um componente anexado e nao tocado passa a fixar a
# vista na origem, e a cena deixa de ser byte-identica.
bloco "a regiao vazia passa a confinar" ph2d-app-components sem_limites_a_saida \
  "$LIM" 1 \
  '    if !min.is_finite() || !max.is_finite() || !c.is_finite() || max <= min {' \
  '    if false {'

# ⛔⛔ A guarda da regiao ESTREITA: sem ela o `f32::clamp` do Rust entra em PANICO com o limite de
# baixo acima do de cima — o mesmo caso que a camera do jogo ja' pagou.
bloco "a regiao estreita entra em panico" ph2d-app-components uma_regiao_mais_estreita \
  "$LIM" 1 \
  '    if lo > hi {' \
  '    if false {'

# ⛔ A ORDEM declarada: envolver ANTES de confinar deixa o deslocamento fora do ladrilho.
#
# ⚠️⚠️ **A mutacao tem de MOVER o envolvimento e nunca o antecipar:** com ele a ficar em ULTIMO no
# produto, qualquer envolvimento posto mais cedo e' **invisivel** — o do fim re-envolve e o
# resultado cai na mesma janela. *Um passo IDEMPOTENTE no fim de uma cadeia apaga toda mutacao de
# ordem que nao o remova de la'.*
bloco "a repeticao deixa de ser a ultima" ph2d-app-components com_limites_e_repeticao \
  "$PONTE" 1 \
  '        let d = cfg_d.deslocamento_confinado(centro, conf);' \
  '        let d0 = cfg_d.deslocamento(centro);
        let d0 = rep.map_or(d0, |r| r.envolve(d0));
        let d = [
            d0[0] + cfg_d.k[0] * (centro[0] - conf[0]),
            d0[1] + cfg_d.k[1] * (centro[1] - conf[1]),
        ];
        let rep: Option<ph2d_ecs::ScrollRepeat> = None;'

echo "=== O MOVIMENTO PROPRIO (W4) ==="

MOV=crates/ph2d-ecs/src/scroll_motion.rs

# ⛔⛔ A lei deixa de ser funcao do RELOGIO e passa a ser um passo fixo ⇒ um acumulador com outro
# nome: um scrub para tras deixa de a desfazer, e duas maquinas com quadros diferentes veem nuvens
# diferentes. E' a propriedade inteira da wave.
bloco "a deriva deixa de ler o playhead" ph2d-app-components um_scrub_para_tras \
  "$MOV" 1 \
  '            (f64::from(self.velocity[0]) * t) as f32,' \
  '            self.velocity[0],'

# ⛔ O `f64` da conta: ao fim de uma hora a `1 m/s` o ULP de um `f32` e' `2,4e-4`, e arredondar o
# PRODUTO acumula. A deriva e' a unica grandeza desta familia que cresce sem limite com o tempo.
bloco "o produto passa a ser feito em f32" ph2d-app-components a_deriva_e_velocidade_vezes \
  "$MOV" 1 \
  '            (f64::from(self.velocity[0]) * t) as f32,
            (f64::from(self.velocity[1]) * t) as f32,' \
  '            self.velocity[0] * (t as f32),
            self.velocity[1] * (t as f32),'

# ⛔⛔ A deriva deixa de SOMAR e passa a substituir ⇒ ela vira um segundo condutor com outro nome, e
# o deslocamento da camera evapora.
bloco "a deriva substitui em vez de somar" ph2d-app-components a_deriva_soma_se \
  "$PONTE" 1 \
  '            [d[0] + o[0], d[1] + o[1]]' \
  '            [o[0], o[1]]'

# ⛔⛔ O salto do neutro volta a esconder a deriva ⇒ uma nuvem que anda sozinha num plano NORMAL
# fica parada, e o painel diz que ela esta' a andar.
bloco "o neutro volta a esconder a deriva" ph2d-app-components a_deriva_acorda_um_objecto \
  "$PONTE" 1 \
  '        if cfg.e_neutro() && mov.is_none_or(|m| m.e_inerte()) {' \
  '        if cfg.e_neutro() {'

# ⛔ A ORDEM: envolver ANTES de somar a deriva deixa a nuvem a fugir.
bloco "a deriva entra DEPOIS da repeticao" ph2d-app-components uma_nuvem_que_deriva \
  "$PONTE" 1 \
  '        let d = mov.map_or(d, |m| {
            let o = m.deslocamento(playhead);
            [d[0] + o[0], d[1] + o[1]]
        });
        let d = rep.map_or(d, |r| r.envolve(d));' \
  '        let d = rep.map_or(d, |r| r.envolve(d));
        let d = mov.map_or(d, |m| {
            let o = m.deslocamento(playhead);
            [d[0] + o[0], d[1] + o[1]]
        });
        let rep: Option<ph2d_ecs::ScrollRepeat> = rep;'

# ⛔ A fase deixa de ler o relogio ⇒ a deriva congela, e nada na tela diz porque.
bloco "a fase crava o relogio em zero" ph2d-host-desktop a_paralaxe_corre_depois \
  "$FASE" 1 \
  '        let playhead = self.playhead.time();' \
  '        let playhead = 0.0;' \
  '--test it'

echo "=== O DOLLY — a multiplano (W5) ==="

# ⛔⛔ A lei inteira. `(1−δ)/(1−k·δ)` e' o tamanho aparente RELATIVO ao plano focal; sem o
# denominador ela vira o tamanho ABSOLUTO, que e' igual para todas as camadas ⇒ um ZOOM.
bloco "a escala vira um zoom (perde o denominador)" ph2d-app-components dois_planos_com_um_dolly \
  "$LEI" 1 \
  '        Some((1.0 - delta) / den)' \
  '        Some(1.0 - delta)'

# ⛔ A degenerescencia que o plano publica, escrita como um RAMO: ela diverge do limite da propria
# formula, e este e' o gate que a refuta.
bloco "o ceu volta a escala 1 que o plano prometia" ph2d-app-components o_ceu_encolhe \
  "$LEI" 1 \
  '        let den = 1.0 - k * delta;' \
  '        if k == 0.0 { return Some(1.0); }
        let den = 1.0 - k * delta;'

# ⛔⛔ A RECUSA vira um clamp ⇒ uma cena impossivel (a camera para la' do fundo) devolve um numero
# plausivel em vez de deixar o objecto onde o artista o pos.
bloco "a camera atravessada passa a ser clampada" ph2d-app-components a_camera_a_atravessar \
  "$LEI" 1 \
  '        if den <= 0.0 {
            return None;
        }' \
  '        let den = if den <= 0.0 { 1e-6 } else { den };'

# ⛔⛔ A escala multiplica o VIVO ⇒ ela COMPOE a cada quadro e o fundo cresce sem limite.
bloco "a escala multiplica o vivo" ph2d-app-components a_escala_do_dolly_nao_compoe \
  "$PONTE" 1 \
  '                Vec2::new(autorada.scale.x * esc[0], autorada.scale.y * esc[1])' \
  '                Vec2::new(era.scale.x * esc[0], era.scale.y * esc[1])'

# ⛔ O dolly deixa de mudar a FRACCAO ⇒ o objecto muda de tamanho e nao muda de velocidade, que e'
# a metade da multiplano que um zoom tambem nao faz.
bloco "o dolly nao muda a fraccao" ph2d-app-components o_dolly_muda_a_velocidade \
  "$PONTE" 1 \
  '        let d = cfg_d.deslocamento_confinado(centro, conf);' \
  '        let d = cfg.deslocamento_confinado(centro, conf);'

# ⛔ A camera activa deixa de ser a porta ⇒ o dolly de uma camera INACTIVA passa a mandar.
bloco "o dolly sai de qualquer camera" ph2d-app-components o_dolly_sai_da_camera_activa \
  "$PONTE" 1 \
  '    let dolly = ph2d_ecs::active_camera_of(sim.world_mut())' \
  '    let dolly = sim.world_mut().query::<(ph2d_ecs::Entity, &ph2d_ecs::GameCamera)>().iter(sim.world()).map(|(e, _)| e).next()'

echo "=== A ORDEM no quadro, e o que ATRAVESSA ==="

# ⛔⛔ **A MUTACAO DO ZOOM INVERTEU-SE COM A PREMISSA (W3).** Ate' a` W2 o defeito era a meia-janela
# ATRAVESSAR (a paralaxe e' um deslocamento e o zoom nao entra nela); desde o CONFINAMENTO o defeito
# e' o contrario — sem a meia-janela o joelho `(regiao − ecra)/2` deixa de ser calculavel e o fundo
# passa a mostrar a borda. ⭐ A metade antiga NAO ficou sem guarda: ela passou a ser a ASSINATURA
# (`deslocamento(centro)` nao ve' zoom nenhum), que e' mais forte que um gate.
bloco "so' o centro atravessa (o joelho deixa de ser calculavel)" ph2d-host-desktop a_paralaxe_corre_depois \
  "$FASE" 1 \
  '            camera_rect,' \
  '            camera_rect.map(|(c, _h)| (c, [0.0, 0.0])),' \
  '--test it'

# ⛔ A paralaxe antes da camera desloca contra o enquadramento do quadro ANTERIOR.
bloco "a paralaxe corre ANTES da camera" ph2d-host-desktop a_paralaxe_corre_depois \
  "$QUADRO" 1 \
  '        let camera_rect = self.fase_game_camera(player_input, report);' \
  '        self.fase_paralaxe(None);
        let camera_rect = self.fase_game_camera(player_input, report);' \
  '--test it'

echo "=== A SUPERFICIE (W7): a seccao, o dolly, a cena e o prologo ==="
POP=crates/ph2d-panel-inspector/src/populate_parallax.rs
SYNCS=crates/ph2d-panel-inspector/src/sync_sections.rs
EVP=crates/ph2d-panel-inspector/src/event_parallax.rs
EVC=crates/ph2d-panel-inspector/src/event_camera.rs
PINTOR=crates/ph2d-panel-inspector/src/sections/parallax.rs
VOC=crates/ph2d-editor-core/src/parallax_edits.rs
INSP=crates/ph2d-app-components/src/parallax_inspector.rs
CENA=crates/ph2d-app-components/src/parallax_smoke.rs
PROL=shells/desktop/src/components_scenes_suplentes.rs

# ⛔ Um id que o `populate` nao regista e' PINTADO e MORTO sob o dedo — a 8.a vez desta crate.
bloco "o ladrilho X fica por registar" ph2d-panel-inspector todo_campo_esta_vivo_sob_o_dedo \
  "$POP" 1 '    (ids::INSP_PARALLAX_TILE_X, 0.0,' '    (ids::INSP_PARALLAX_K_X, 0.0,' '--test it'
bloco "a semente da seccao nao corre" ph2d-panel-inspector os_campos_mostram_o_que_o_objecto_tem \
  "$SYNCS" 1 '    crate::sync_parallax::sync(host, inspector_state, entity_changed);' '' '--test it'
bloco "o par do ladrilho troca de eixo" ph2d-panel-inspector escrever_num_campo_chega \
  "$EVP" 1 'ParallaxFieldEdit::Repeat([f, rep[1]])' 'ParallaxFieldEdit::Repeat([rep[1], f])' '--test it'
bloco "o outro eixo do factor vem de um ZERO" ph2d-panel-inspector escrever_num_campo_chega \
  "$EVP" 1 'ParallaxFieldEdit::Factor([f, info.factor[1]])' 'ParallaxFieldEdit::Factor([f, 0.0])' '--test it'
bloco "o bloco do ladrilho aparece sempre" ph2d-panel-inspector os_blocos_aparecem_so_com \
  "$PINTOR" 1 '    if i.repeat.is_some() {' '    if true {' '--test it'
# ⭐ O DOLLY — a semente, a ARESTA nova da camera, e o dreno.
bloco "a semente do dolly some" ph2d-panel-inspector o_dolly_mostra_a_camera \
  "$SYNCS" 1 '        (crate::ids::INSP_CAMERA_DOLLY, f64::from(cam.camera.dolly)),' '' '--test it'
bloco "a camera volta a semear so na troca de objecto" ph2d-panel-inspector o_dolly_mostra_a_camera \
  "$SYNCS" 1 'sync_camera_fields(host, &cam, entity_changed || mudou);' 'sync_camera_fields(host, &cam, entity_changed);' '--test it'
bloco "o dolly escreve na altura" ph2d-panel-inspector o_dolly_mostra_a_camera \
  "$EVC" 1 'crate::ids::INSP_CAMERA_DOLLY => CameraFieldEdit::Dolly(f),' 'crate::ids::INSP_CAMERA_DOLLY => CameraFieldEdit::Height(f),' '--test it'
# ⭐ As queixas: a ORDEM da recusa, e a camera ACHADA no mundo.
bloco "a queixa da camera some" ph2d-app-components as_queixas_seguem \
  "$VOC" 1 '        if !self.tem_camera_do_jogo {' '        if false {'
bloco "o construtor nunca acha a camera" ph2d-app-components as_queixas_seguem \
  "$INSP" 1 '    let tem_camera_do_jogo = world.iter_entities().any(|x| x.contains::<GameCamera>());' '    let tem_camera_do_jogo = false;'
bloco "o dreno troca os eixos do ladrilho" ph2d-app-components o_dreno_escreve_o_par \
  "$INSP" 1 '            c.tile = *t;' '            c.tile = [t[1], t[0]];'
# ⭐ A CENA — cada gate e' uma frase do roteiro.
bloco "as colinas colam-se as arvores" ph2d-app-components os_planos_vizinhos \
  "$CENA" 1 'pub const K_COLINAS: f32 = 0.35;' 'pub const K_COLINAS: f32 = 0.55;'
bloco "as colinas passam a repetir" ph2d-app-components as_colinas_tem_cerca \
  "$CENA" 1 '        // ⚠️ **SEM `ScrollRepeat`, de propósito** — ver o cabeçalho: é o contraste que ensina.' '        ScrollRepeat { tile: [PASSO_COLINAS, 0.0] },'
bloco "a serra deixa de ter cerca a ALTURA certa" ph2d-app-components a_borda_da_serra \
  "$CENA" 1 'const SERRA_MAX_X: f32 = 30.0;' 'const SERRA_MAX_X: f32 = 90.0;'
bloco "a camera deixa de ficar presa em Y" ph2d-app-components a_camera_fica_presa \
  "$CENA" 1 '                min: [-400.0, -MEIA_VISTA_Y],' '                min: [-400.0, -2.0 * MEIA_VISTA_Y],'
bloco "a fileira das arvores fica curta" ph2d-app-components a_fileira_que_repete \
  "$CENA" 1 'const PECAS_ARVORES: i32 = 15;' 'const PECAS_ARVORES: i32 = 5;'
bloco "o ceu sai do ecra" ph2d-app-components toda_peca_cabe \
  "$CENA" 1 'const Y_CEU: f32 = 3.2;' 'const Y_CEU: f32 = 4.8;'
bloco "o heroi fica do solver" ph2d-app-components o_heroi_tem_corpo \
  "$CENA" 1 '            kind: BodyKind::Kinematic,' '            kind: BodyKind::Dynamic,'
bloco "nasce escolhida a camada errada" ph2d-app-components quem_nasce_escolhido \
  "$CENA" 1 '    entidade_por_nome(world, "Arvores")' '    entidade_por_nome(world, "Colinas")'
bloco "a deriva do ceu some" ph2d-app-components com_a_camera_parada \
  "$CENA" 1 '            velocity: [DERIVA_CEU, 0.0],' '            velocity: [0.0, 0.0],'
bloco "o roteiro escreve o nome a mao" ph2d-app-components o_roteiro_nomeia \
  "$CENA" 1 '        t("panel.inspector.parallax.repeat_m"),' '        "Repeat",'
# ⭐ O PROLOGO — as tres obrigacoes.
bloco "o prologo nao toma a vista do jogo" ph2d-host-desktop o_prologo_da_cena_da_paralaxe \
  "$PROL" 1 '        // ⭐⭐⭐ Ver o ponto 1 do doc — sem isto a wave inteira mede outra câmera.
        self.game_camera_preview = true;' '' '--test it'
bloco "o prologo abre a regua" ph2d-host-desktop o_prologo_da_cena_da_paralaxe \
  "$PROL" 1 '            hero.panel_visibility.insert("timeline", false);' '            crate::components_scenes::abre_a_regua_da_corrida(hero);
            hero.panel_visibility.insert("timeline", false);' '--test it'

echo
echo "════════════════════════════════════════"
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL sangraram"
else
  echo "⛔ $FALHAS de $TOTAL NAO sangraram"
fi
exit "$FALHAS"
