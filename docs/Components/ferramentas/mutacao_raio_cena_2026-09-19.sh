#!/usr/bin/env bash
# Provas de mutação da W6 do suplente #21 — A CENA DO OLHO e a SEMENTE do painel.
#
# ⚠️ As da LEI estão no `mutacao_raio_2026-09-19.sh` e as do DESENHO no
# `mutacao_raio_gizmo_2026-09-19.sh`. Os três correm-se.
#
# ⭐⭐⭐ QUATRO destas mutações são de defeitos que a SUÍTE INTEIRA aprovava e que só uma FOTO
#      apanhou (`fotografa_cena.sh`, 19/09): o overlay projectado contra a janela errada, o painel
#      a mostrar os valores de FÁBRICA, o controlo fora do ecrã, e a luz apagada que não se via.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_raio_cena_2026-09-19.sh
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

CENA=crates/ph2d-app-components/src/ray_smoke.rs
SYNC=crates/ph2d-panel-inspector/src/sync_sections.rs
SYNCR=crates/ph2d-panel-inspector/src/sync_ray.rs

echo "════ A CENA — a lição, o controlo e o enquadramento ════"

# (1) O CONTROLO deixa de ser um controlo: os dois olhos olham para o mesmo lado.
bloco "cena: o controlo olha para a frente" ph2d-app-components os_dois_olhos_diferem \
  "$CENA" 1 \
  "        Vec2::new(-1.0, 0.0)," \
  "        Vec2::new(1.0, 0.0)," \
  "--lib"

# (2) A caixa nasce DENTRO do alcance — o passo (2) do roteiro passa a mentir.
bloco "cena: a caixa nasce dentro do alcance" ph2d-app-components a_caixa_comeca_fora \
  "$CENA" 1 \
  "const X_CAIXA: f32 = 6.0;" \
  "const X_CAIXA: f32 = 1.0;" \
  "--lib"

# (3) O poste volta a `−6` e a luz do CONTROLO sai do ecrã, atrás da Hierarquia (o defeito da foto).
bloco "cena: o controlo sai do ecra" ph2d-app-components a_cena_inteira_cabe \
  "$CENA" 1 \
  "const X_POSTE: f32 = -4.0;" \
  "const X_POSTE: f32 = -6.0;" \
  "--lib"

# (4) O SUPORTE desaparece: «não acendeu» e «não há luz deste lado» passam a ler-se igual.
bloco "cena: a luz apagada deixa de se ver" ph2d-app-components as_duas_luzes_ouvem \
  "$CENA" 1 \
  "        Visibility::visible(),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [LUZ_LADO * 1.4, LUZ_LADO * 1.4],
            SUPORTE_RGBA,
        )," \
  "        Visibility::hidden(),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [LUZ_LADO * 1.4, LUZ_LADO * 1.4],
            SUPORTE_RGBA,
        )," \
  "--lib"

# (5) As duas luzes passam a ouvir o MESMO nome — elas acendem juntas e o controlo evapora-se.
bloco "cena: as duas luzes ouvem o mesmo" ph2d-app-components as_duas_luzes_ouvem \
  "$CENA" 1 \
  "const VIU_ATRAS: &str = \"vi-atras\";" \
  "const VIU_ATRAS: &str = \"vi\";" \
  "--lib"

# (6) O OLHO volta a ter Sprite: a secção `Ray Sensor` cai tres ecras abaixo da dobra.
bloco "cena: o olho volta a ter sprite" ph2d-app-components os_olhos_nao_tem_sprite \
  "$CENA" 1 \
  "            Transform::from_translation(Vec2::new(X_POSTE, y)),
            Visibility::visible()," \
  "            Transform::from_translation(Vec2::new(X_POSTE, y)),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [0.55, 0.55], LUZ_RGBA)," \
  "--lib"

# (7) O roteiro deixa de nomear um rótulo que está na tela.
#
# ⛔⛔ **A 1.ª redacção desta mutação SOBREVIVEU, e o defeito era dela:** ela trocava UMA das TRÊS
#     ocorrências de `Reach` no ficheiro, logo o rótulo continuava lá e o gate tinha razão em ficar
#     verde. *Uma mutação que não muta lê-se exactamente como uma que sobreviveu* — a lição que o
#     roteiro do GOLPE já tinha pago no mesmo dia, com a âncora do `rebobina`/`Reset`.
# ⚠️ E o filtro passou a nomear o MÓDULO: `o_roteiro_nomeia_rotulos` casa DOIS testes desta crate
#     (o do golpe e o do olho), e um deles reprovar bastava para a prova se ler como boa. ⚠️⚠️ O
#     caminho é `ray_smoke::tests::…` e não `ray_smoke_tests::…` — um módulo declarado por
#     `#[path]` chama-se pelo NOME que o `mod` lhe dá, nunca pelo do ficheiro; a 1.ª tentativa casou
#     **zero** e só o controlo do próprio filtro a distinguiu de uma sobrevivência.
bloco "cena: o roteiro traduz um rotulo" ph2d-app-components \
  ray_smoke::tests::o_roteiro_nomeia_rotulos \
  "$CENA" 3 \
  "\`Reach\`" \
  "\`Alcance\`" \
  "--lib"

echo "════ A SEMENTE do painel — o defeito que a FOTO apanhou ════"

# (8) A semente desliga-se: o painel volta a mostrar os valores de FÁBRICA do `populate_ray`,
#     com a linha desenhada a 6 m ao lado e a leitura viva certa. (O defeito, à letra.)
bloco "painel: a seccao mostra os valores de fabrica" ph2d-panel-inspector \
  os_campos_mostram_os_numeros_do_objecto \
  "$SYNC" 1 \
  "    crate::sync_ray::sync(host, inspector_state, entity_changed);" \
  "    let _ = &crate::sync_ray::sync;" \
  "--test it"

# (9) A semente deixa de respeitar o FOCO: o artista digita e a letra desaparece.
bloco "painel: a semente pisa o campo em foco" ph2d-panel-inspector a_mao_do_artista_ganha \
  "$SYNCR" 1 \
  "        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }" \
  "        let _ = drag;" \
  "--test it"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"
