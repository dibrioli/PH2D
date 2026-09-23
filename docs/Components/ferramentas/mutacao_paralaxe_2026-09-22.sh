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

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; }

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
  '            autorada[0] + centro[0] * (1.0 - self.k[0]),' \
  '            autorada[0] + centro[0] * self.k[0],'

bloco "os dois eixos partilham o k do x" ph2d-app-components os_dois_eixos \
  "$LEI" 1 \
  '            autorada[1] + centro[1] * (1.0 - self.k[1]),' \
  '            autorada[1] + centro[1] * (1.0 - self.k[0]),'

# ⛔ A referencia deixa de ser a ORIGEM DO MUNDO e passa a ser a propria peca — a forma do Flip,
# que TELEPORTA para o centro da vista um fundo autorado num canto (ver o cabecalho da lei).
bloco "a referencia vira a propria peca (a forma do Flip)" ph2d-app-components o_deslocamento_nao_depende \
  "$LEI" 1 \
  '            autorada[0] + centro[0] * (1.0 - self.k[0]),' \
  '            autorada[0] + (centro[0] - autorada[0]) * (1.0 - self.k[0]),'

bloco "o neutro deixa de ser 1" ph2d-app-components o_neutro \
  "$LEI" 1 \
  '    pub const NEUTRO: [f32; 2] = [1.0, 1.0];' \
  '    pub const NEUTRO: [f32; 2] = [0.0, 0.0];'

echo "=== A PONTE: quem e' conduzido, e a partir de QUE pose ==="

# ⛔ Sem o salto do neutro, toda cena com o componente anexado passa a ter uma entrada viva no
# ledger — a captura paga uma varredura por nada, e o `n` que o painel mostra mente.
bloco "o neutro passa a ser declarado" ph2d-app-components o_neutro_nao_escreve \
  "$PONTE" 1 \
  '        if cfg.e_neutro() {' \
  '        if false {'

# ⛔ Sem camera de jogo a resposta e' NAO DECLARAR: uma corrida que dependesse de onde o artista
# rolou o ecra' seria outra corrida em cada maquina.
bloco "sem camera a vista vira a origem" ph2d-app-components sem_camera_de_jogo \
  "$PONTE" 1 \
  '    let Some(centro) = centro else {' \
  '    let Some(centro) = centro.or(Some([0.0, 0.0])) else {'

# ⛔ A populacao: um HUD ja' e' conduzido pela ponte dele, e dois motores sobre o mesmo `Transform`
# escrevem um por cima do outro.
bloco "o HUD entra na populacao" ph2d-app-components um_hud_nao_e_tocado \
  "$PONTE" 1 \
  '            .query_filtered::<(Entity, &Transform, &ScrollFactor), bevy_ecs::prelude::Without<UiCanvas>>()' \
  '            .query::<(Entity, &Transform, &ScrollFactor)>()'

# ⛔ Esta ponte so' escreve a TRANSLACAO: roubar o resto ao autorado apagaria o que outro motor
# tivesse escrito no mesmo quadro.
bloco "a rotacao autorada e' apagada" ph2d-app-components so_a_translacao \
  "$PONTE" 1 \
  '            ..era
        };
        let escreveu' \
  '            ..Transform::default()
        };
        let escreveu'

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

echo "=== A ORDEM no quadro, e o que ATRAVESSA ==="

# ⛔ A meia-janela atravessa para a lei ⇒ e' por ai' que a invariancia ao zoom se perde em silencio.
bloco "o rectangulo inteiro atravessa a chamada" ph2d-host-desktop a_paralaxe_corre_depois \
  "$FASE" 1 \
  '            sim,
            centro,' \
  '            sim,
            camera_rect.map(|(center, _half)| center),' \
  '--test it'

# ⛔ A paralaxe antes da camera desloca contra o enquadramento do quadro ANTERIOR.
bloco "a paralaxe corre ANTES da camera" ph2d-host-desktop a_paralaxe_corre_depois \
  "$QUADRO" 1 \
  '        let camera_rect = self.fase_game_camera(player_input, report);' \
  '        self.fase_paralaxe(None);
        let camera_rect = self.fase_game_camera(player_input, report);' \
  '--test it'

echo
echo "════════════════════════════════════════"
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL sangraram"
else
  echo "⛔ $FALHAS de $TOTAL NAO sangraram"
fi
exit "$FALHAS"
