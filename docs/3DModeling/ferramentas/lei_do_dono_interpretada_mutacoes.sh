#!/usr/bin/env bash
# 2026-10-01 — as PROVAS DE MUTAÇÃO da lei do dono INTERPRETADA e da compilação fora da medição
# (handoff `HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md` §11).
#
# Cada mutação: cópia → substituir a agulha (a CONTAGEM tem de ser exactamente 1, senão ela não é
# aplicada e o script DIZ) → correr os gates que a devem matar → restaurar → `touch` (um `mv` devolve
# o mtime antigo, e o cargo guardaria o build DA MUTAÇÃO).
#
# Duas corridas, porque metade dos gates são de PLACA:
#   cpu — `cargo nextest` no perfil `ci-test`, filtro por expressão;
#   gpu — `cargo test --release … -- --ignored --test-threads=1` (o nextest morre com SIGSEGV à
#         saída nesta crate), e a população é `passed + failed` do `test result:` — ⚠️ NUNCA o
#         `running N tests`, que CONTA os ignorados.
#
# Quatro veredictos, e nenhum é lido como outro:
#   MORTA        — compilou, correu testes, e algum reprovou
#   SOBREVIVEU   — compilou, correu testes, e todos passaram
#   NÃO COMPILOU — não é prova de nada
#   FILTRO VAZIO — zero testes correram (um «sobreviveu» de zero testes é um byte, não uma prova)
#
# ⚠️ A corrida inteira toca na placa: corra-a com o cadeado —
#   PH2D_GPU=1 PH2D_GPU_ESPERA=900 bash scripts/ph2d-run.sh \
#     bash docs/3DModeling/ferramentas/lei_do_dono_interpretada_mutacoes.sh [saída]
# `MUTA_SO_ANCORAS=1` só confere que cada agulha casa UMA vez (segundos, sem correr um teste).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
out="${1:-target/lei_do_dono_mutacoes.txt}"
mkdir -p target
: >"$out"

ancora_ok() {
  python3 - "$1" "$2" <<'PY'
import sys
path, old = sys.argv[1], sys.argv[2]
n = open(path, encoding="utf-8").read().count(old)
sys.exit(0 if n == 1 else f"{path}: a agulha casou {n} vezes (esperava 1)")
PY
}

corre() {
  local modo="$1" pkg="$2" filtro="$3" log="$4"
  if [ "$modo" = cpu ]; then
    cargo nextest run -p "$pkg" --cargo-profile ci-test --no-fail-fast -E "$filtro" >"$log" 2>&1
  else
    # shellcheck disable=SC2086
    cargo test --release -p "$pkg" --lib -- --ignored --test-threads=1 $filtro >"$log" 2>&1
  fi
}

veredito() {
  local modo="$1" name="$2" rc="$3" log="$4"
  if grep -qE '^error\[E[0-9]+\]|could not compile' "$log"; then
    echo "NÃO COMPILOU $name ($log)"
    return
  fi
  local n
  if [ "$modo" = cpu ]; then
    n=$(grep -oE '[0-9]+ tests? run' "$log" | tail -1 | grep -oE '^[0-9]+')
  else
    n=$(grep -E '^test result:' "$log" | sed -E 's/.* ([0-9]+) passed; ([0-9]+) failed.*/\1 \2/' |
      awk '{s += $1 + $2} END {print s + 0}')
  fi
  if [ -z "$n" ] || [ "$n" -eq 0 ]; then
    echo "FILTRO VAZIO $name"
  elif [ "$rc" -eq 0 ]; then
    echo "SOBREVIVEU   $name — $n teste(s)"
  else
    echo "MORTA        $name — $n teste(s), rc=$rc"
  fi
}

mutate() {
  local name="$1" file="$2" old="$3" new="$4" modo="$5" pkg="$6" filtro="$7"
  if ! ancora_ok "$file" "$old"; then
    echo "✗ AGULHA     $name — a mutação NÃO foi aplicada" | tee -a "$out"
    return
  fi
  if [ "${MUTA_SO_ANCORAS:-0}" = 1 ]; then
    echo "ancora ok    $name" | tee -a "$out"
    return
  fi
  local log="target/lei_do_dono_${name}.log"
  cp "$file" "$file.muta_bak" || return
  python3 - "$file" "$old" "$new" <<'PY'
import sys
path, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
src = open(path, encoding="utf-8").read()
open(path, "w", encoding="utf-8").write(src.replace(old, new, 1))
PY
  corre "$modo" "$pkg" "$filtro" "$log"
  local rc=$?
  mv "$file.muta_bak" "$file"
  touch "$file"
  veredito "$modo" "$name" "$rc" "$log" | tee -a "$out"
}

GATES_PLACA='uma_forma_nova_recompila_as_sondas_e_nao_o_pintor a_primeira_cor_diferente_nao_compila_nada o_quadro_diz_quanto_foi_compilacao forma_nova::lote'

# 4.º CONTROLO — a corrida LIMPA tem de estar verde, senão toda mutação se lê como MORTA.
if [ "${MUTA_SO_ANCORAS:-0}" != 1 ]; then
  corre cpu ph2d-field-eval 'test(o_texto_da_lei_do_dono_nao_depende_da_peca)' target/lei_do_dono_limpa_cpu.log
  rc=$?
  veredito cpu limpa_cpu "$rc" target/lei_do_dono_limpa_cpu.log | tee -a "$out"
  corre gpu ph2d-app-field3d "$GATES_PLACA" target/lei_do_dono_limpa_gpu.log
  rc=$?
  veredito gpu limpa_gpu "$rc" target/lei_do_dono_limpa_gpu.log | tee -a "$out"
  if ! grep -qE '^SOBREVIVEU   limpa_cpu' "$out" || ! grep -qE '^SOBREVIVEU   limpa_gpu' "$out"; then
    echo "⛔ a corrida LIMPA não está verde — o placar seria fabricado; aborto" | tee -a "$out"
    exit 3
  fi
fi

# L1 — a lei volta a ser COMPILADA no texto (o caminho de antes de 2026-10-01).
mutate l1_lei_compilada \
  crates/ph2d-field-eval/src/owners_wgsl.rs \
  '        self.interpretada(const_base)
            .or_else(|| self.compilada(const_base))' \
  '        self.compilada(const_base)
            .or_else(|| self.interpretada(const_base))' \
  cpu ph2d-field-eval 'test(o_texto_da_lei_do_dono_nao_depende_da_peca)'

# L2 — o pintor volta a levar a fita REAL (o texto muda a cada forma).
mutate l2_pintor_com_a_fita_real \
  crates/ph2d-field-gpu/src/paint_entradas.rs \
  '    let pintura = if pintor.le_o_campo { fita } else { inerte };' \
  '    let pintura = if pintor.le_o_campo || true { fita } else { inerte };' \
  gpu ph2d-app-field3d 'uma_forma_nova_recompila_as_sondas_e_nao_o_pintor'

# L3 — sem a lei da peça sem donos, a 1.ª cor diferente troca o texto (sem bloco → com bloco).
mutate l3_sem_a_lei_sem_donos \
  crates/ph2d-field-gpu/src/trace_marcha_com.rs \
  '            .or_else(|| ph2d_field_eval::owners::wgsl::sem_donos(consts.len()))' \
  '            .or_else(|| None)' \
  gpu ph2d-app-field3d 'a_primeira_cor_diferente_nao_compila_nada'

# L4 — o quadro pintado deixa de dizer quanto foi compilação.
mutate l4_o_quadro_cala_a_compilacao \
  crates/ph2d-field-gpu/src/trace.rs \
  '            Pintura::Material(pintor),
        ) {
            Saida::Imagem(mut p) => {
                p.compilado_ms = self.cache.compilado_ms() - antes;' \
  '            Pintura::Material(pintor),
        ) {
            Saida::Imagem(mut p) => {
                p.compilado_ms = 0.0 * (self.cache.compilado_ms() - antes);' \
  gpu ph2d-app-field3d 'o_quadro_diz_quanto_foi_compilacao'

# L5 — CONTROLO NEGATIVO do censo: muda um COMENTÁRIO do ramo do material e deixa as duas
# subtracções intactas. Tem de SOBREVIVER, senão o censo lê prosa e não código.
mutate l5_controlo_comentario \
  crates/ph2d-app-field3d/src/smoke_draw_thread.rs \
  '        // ficou no dispositivo, e o alfa do fundo é o discriminador que já vive ali.' \
  '        // ficou no dispositivo (controlo de mutação: esta linha é só prosa).' \
  cpu ph2d-app-field3d 'test(o_laco_da_resolucao_desconta_a_compilacao)'

# L6 — o 1.º quadro da placa (o do material) volta a medir a compilação como custo do quadro.
# ⚠️ A agulha leva o comentário que SÓ o ramo do material tem: as duas subtracções são texto
# idêntico, e uma agulha só com a expressão casaria duas vezes.
mutate l6_sem_a_subtraccao_no_material \
  crates/ph2d-app-field3d/src/smoke_draw_thread.rs \
  '        // ficou no dispositivo, e o alfa do fundo é o discriminador que já vive ali.
        let hits = pintura
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[3] > BACKGROUND[3])
            .count();
        let (iw, ih) = tamanho_entregue(p, pintura.rgba.len());
        let _ = p.tx.try_send(Ready {
            rgba: pintura.rgba,
            width: iw,
            height: ih,
            tracado_px: u64::from(p.tw) * u64::from(p.th),
            hits,
            edges: pintura.edges,
            // ⏱️⭐⭐⭐ **Sem a COMPILAÇÃO** — ver [`ph2d_field_gpu::FieldPipelines::compilado_ms`]:
            // este número decide o tamanho do quadro seguinte, que não a paga.
            millis: (t0.elapsed().as_secs_f64() * 1000.0 - pintura.compilado_ms).max(0.0),' \
  '        // ficou no dispositivo, e o alfa do fundo é o discriminador que já vive ali.
        let hits = pintura
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[3] > BACKGROUND[3])
            .count();
        let (iw, ih) = tamanho_entregue(p, pintura.rgba.len());
        let _ = p.tx.try_send(Ready {
            rgba: pintura.rgba,
            width: iw,
            height: ih,
            tracado_px: u64::from(p.tw) * u64::from(p.th),
            hits,
            edges: pintura.edges,
            // ⏱️⭐⭐⭐ **Sem a COMPILAÇÃO** — ver [`ph2d_field_gpu::FieldPipelines::compilado_ms`]:
            // este número decide o tamanho do quadro seguinte, que não a paga.
            millis: t0.elapsed().as_secs_f64() * 1000.0,' \
  cpu ph2d-app-field3d 'test(o_laco_da_resolucao_desconta_a_compilacao)'

# ── A 2.ª metade do report (*«ainda 1 ou 2 segundos»*): o lote ÚNICO e as sondas a mexer ──
GATES_LOTE='forma_nova::lote'

# L7 — o lote do quadro deixa de levar o céu no tempo.
mutate l7_lote_sem_o_ceu \
  crates/ph2d-field-gpu/src/trace_marcha_com.rs \
  '            pedidos.extend(' \
  '            let _ = &mut pedidos;
            Vec::<crate::PedidoDeLote<'"'"'_>>::new().extend(' \
  gpu ph2d-app-field3d "$GATES_LOTE"

# L8 — o lote do quadro deixa de levar as sondas.
mutate l8_lote_sem_as_sondas \
  crates/ph2d-field-gpu/src/trace_marcha_com.rs \
  '                pedidos.push((f.as_str(), fita, "assa_sondas", Some(l)));' \
  '                let _ = (f, l);' \
  gpu ph2d-app-field3d "$GATES_LOTE"

# L9 — o quadro de movimento volta a ASSAR as sondas (ignora as guardadas).
mutate l9_movimento_assa \
  crates/ph2d-field-gpu/src/paint.rs \
  '            Some(Some(guardadas)) => (guardadas, false),' \
  '            Some(Some(_)) => cache.sondas(device, chave_sondas.clone(), bytes_sondas),' \
  gpu ph2d-app-field3d "$GATES_LOTE"

# L10 — as guardadas servem SEMPRE (a tolerância sai).
mutate l10_sem_tolerancia \
  crates/ph2d-field-gpu/src/sondas_na_placa.rs \
  '            || guardadas.chave.deslocamento_em_celulas(c) <= self.tolerancia_das_sondas)' \
  '            || guardadas.chave.deslocamento_em_celulas(c) >= -1.0)' \
  gpu ph2d-app-field3d "$GATES_LOTE"

# L11 — as guardadas NUNCA servem (só a mesma chave).
mutate l11_nunca_servem \
  crates/ph2d-field-gpu/src/sondas_na_placa.rs \
  '            || guardadas.chave.deslocamento_em_celulas(c) <= self.tolerancia_das_sondas)' \
  '            || guardadas.chave.deslocamento_em_celulas(c) < -1.0)' \
  gpu ph2d-app-field3d "$GATES_LOTE"

# L12 — sem sondas que sirvam, o movimento leva o ricochete na mesma (e assa).
mutate l12_ricochete_sem_sondas \
  crates/ph2d-field-gpu/src/paint_entradas.rs \
  '        && !(sondas_servem' \
  '        && !((sondas_servem || true)' \
  gpu ph2d-app-field3d "$GATES_LOTE"

echo "---"
echo "esperado: L1–L4 e L6–L12 MORTAS · L5 SOBREVIVE (controlo) · limpa_cpu/limpa_gpu SOBREVIVEM" | tee -a "$out"
