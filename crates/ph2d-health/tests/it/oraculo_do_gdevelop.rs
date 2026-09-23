//! ⭐⭐⭐ **A PARIDADE com o oráculo, quadro a quadro e AO BIT** (plano 28, W1).
//!
//! As 19 fixturas de `docs/Components/ferramentas/gdevelop_health/fixtures/` são o código que o
//! próprio gerador do GDevelop produz para a extensão *Health* `0.4.0`, a correr no runtime dele sem
//! interface. Esta bancada corre a [`ph2d_health::Vida`] sob [`Regras::GDEVELOP`] com os MESMOS
//! pedidos, pela MESMA ordem, e compara **três leituras por quadro** — `antes` (depois do
//! `doStepPreEvents`, antes das acções), `depois` (depois das acções) e `fim` (entre quadros) — em
//! todos os campos públicos, nos internos e nos dois relógios. A igualdade é **exacta** (`to_bits`):
//! a lei é só `+ − × min max` em `f64`, as mesmas operações pela mesma ordem, e um `≈` aqui
//! esconderia uma ordem trocada.
//!
//! ⚠️ **O sorteio da esquiva é o GRAVADO:** cada passo diz quantos `Math.random` houve e (no cenário
//! semeado) os valores. A bancada entrega-os pela ordem e exige que a lei os consuma TODOS — um
//! sorteio a mais ou a menos é um defeito de ordem do pipeline, não de aritmética.

use std::path::PathBuf;

use super::json_exacto::{Json as Value, ler};
use ph2d_health::{Config, Regras, Vida};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Components/ferramentas/gdevelop_health/fixtures")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — o gémeo das bancadas do tecido e da pose.
fn inflar(p: &std::path::Path) -> String {
    let raw = std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "não é gzip"
    );
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0x04 != 0 {
        let xlen = usize::from(raw[off]) | (usize::from(raw[off + 1]) << 8);
        off += 2 + xlen;
    }
    for bit in [0x08u8, 0x10] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
    }
    if flg & 0x02 != 0 {
        off += 2;
    }
    let bytes = miniz_oxide::inflate::decompress_to_vec(&raw[off..raw.len() - 8])
        .unwrap_or_else(|e| panic!("não inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("não é número: {v:?}"))
}

/// Um relógio gravado: um número, ou a string `"NaN"` quando o relógio não existe.
fn relogio(v: &Value) -> Option<f64> {
    v.as_f64()
}

/// Compara UMA leitura gravada com o estado da lei. Devolve as diferenças (vazio = paridade).
fn compara(fase: &str, g: &Value, cfg: &Config, v: &Vida) -> Vec<String> {
    let p = &g["publico"];
    let mut dif = Vec::new();
    let mut n = |nome: &str, nosso: f64| {
        let deles = num(&p[nome]);
        if nosso.to_bits() != deles.to_bits() {
            dif.push(format!("{fase}.{nome}: nós {nosso:?}, alvo {deles:?}"));
        }
    };
    n("Health", v.pontos);
    n("MaxHealth", cfg.maximo);
    n("ShieldPoints", v.escudo);
    n("MaxShield", cfg.escudo_max);
    n("HealthRegenRate", cfg.regen);
    n("HealthRegenDelay", cfg.regen_atraso_s);
    n("DamageCooldownDuration", cfg.invencivel_s);
    n("DamageCooldownRemaining", v.invencivel_resta_s(cfg));
    n("ChanceToDodge", cfg.esquiva);
    n("FlatDamageReduction", cfg.armadura_fixa);
    n("PercentDamageReduction", cfg.armadura_pct);
    n("TimeSinceLastHit", v.desde_golpe_s());
    n("PreviousDamageTaken", v.dano_anterior);
    n("PreviousDamageToShield", v.dano_ao_escudo_anterior);
    n("PreviousHealAmount", v.cura_anterior);
    n("ShieldRegenRate", cfg.escudo_regen);
    n("ShieldRegenDelay", cfg.escudo_regen_atraso_s);
    n("ShieldDuration", cfg.escudo_duracao_s);
    n("ShieldTimeRemaining", v.escudo_resta_s(cfg));
    let mut b = |nome: &str, nosso: bool| {
        let deles = p[nome]
            .as_bool()
            .unwrap_or_else(|| panic!("{nome} não é booleano"));
        if nosso != deles {
            dif.push(format!("{fase}.{nome}: nós {nosso}, alvo {deles}"));
        }
    };
    b("IsDead", v.morta());
    b("IsJustDamaged", v.acabou_de_levar_dano);
    b("IsJustHealed", v.acabou_de_ser_curada);
    b("IsJustDodged", v.acabou_de_esquivar);
    b("IsShieldJustDamaged", v.escudo_acabou_de_levar_dano);
    b("IsShieldActive", v.escudo_activo(cfg));
    b("IsDamageCooldownActive", v.invencivel(cfg));
    b("HitAtLeastOnce", v.golpeada_alguma_vez);
    // Os internos que as expressões públicas não expõem.
    let br = &g["bruto"];
    if br["AllowOverHealing"].as_bool() != Some(cfg.sobre_cura) {
        dif.push(format!("{fase}.AllowOverHealing"));
    }
    if br["BlockExcessDamage"].as_bool() != Some(cfg.escudo_bloqueia_excesso) {
        dif.push(format!("{fase}.BlockExcessDamage"));
    }
    // Os dois relógios (em segundos, `NaN` = não existe).
    let t = &g["timers"];
    let tslh = relogio(&t["__Health.TimeSinceLastHit"]);
    if tslh.map(f64::to_bits) != Some(v.desde_golpe_s().to_bits()) {
        dif.push(format!(
            "{fase}.relógio do golpe: nós {:?}, alvo {tslh:?}",
            v.desde_golpe_s()
        ));
    }
    let esc = relogio(&t["__Health.ShieldDuration"]);
    if esc.map(f64::to_bits) != v.escudo_relogio_s().map(f64::to_bits) {
        dif.push(format!(
            "{fase}.relógio do escudo: nós {:?}, alvo {esc:?}",
            v.escudo_relogio_s()
        ));
    }
    dif
}

/// Aplica UMA acção do vocabulário do oráculo (`comandos.mjs`).
fn aplica(acao: &[Value], cfg: &mut Config, v: &mut Vida, sorteio: &mut impl FnMut() -> f64) {
    let nome = acao[0].as_str().expect("nome");
    let x = acao.get(1).map_or(0.0, num);
    let r = Regras::GDEVELOP;
    match nome {
        "Hit" => v.golpe(cfg, r, x, false, false, sorteio),
        "Hit+Shield" => v.golpe(cfg, r, x, true, false, sorteio),
        "Hit+Armor" => v.golpe(cfg, r, x, false, true, sorteio),
        "Hit+Shield+Armor" => v.golpe(cfg, r, x, true, true, sorteio),
        "SetHealth" => v.define(cfg, r, x),
        "Heal" => v.cura(cfg, r, x),
        "AllowOverHealing:yes" => cfg.sobre_cura = true,
        "AllowOverHealing:no" => cfg.sobre_cura = false,
        "TriggerDamageCooldown" => v.arma_invencibilidade(),
        "RenewShieldDuration" => v.renova_escudo(),
        "ActivateShield:renew" => v.activa_escudo(cfg, x, true),
        "ActivateShield:norenew" => v.activa_escudo(cfg, x, false),
        "SetShieldBlockExcessDamage:yes" => cfg.escudo_bloqueia_excesso = true,
        "SetShieldBlockExcessDamage:no" => cfg.escudo_bloqueia_excesso = false,
        "SetMaxHealthOp" => v.muda_maximo(cfg, x),
        "SetHealthRegenRateOp" => cfg.regen = x,
        "SetHealthRegenDelayOp" => cfg.regen_atraso_s = x,
        "SetCooldownDurationOp" => cfg.invencivel_s = x,
        "SetChanceToDodgeOp" => cfg.esquiva = x,
        "SetFlatDamageReductionOp" => cfg.armadura_fixa = x,
        "SetPercentDamageReductionOp" => cfg.armadura_pct = x,
        "SetMaxShieldOp" => cfg.escudo_max = x,
        "SetShieldPointsOp" => v.escudo = x,
        "SetShieldRegenRateOp" => cfg.escudo_regen = x,
        "SetShieldRegenDelayOp" => cfg.escudo_regen_atraso_s = x,
        "SetShieldDurationOp" => cfg.escudo_duracao_s = x,
        outro => panic!("acção desconhecida no oráculo: {outro} — o vocabulário mudou"),
    }
}

/// Corre UMA fixtura e devolve as diferenças.
fn corre(texto: &str) -> (usize, Vec<String>) {
    let f: Value = ler(texto);
    let mut cfg = Config::default();
    let mut v = Vida::nasce(num(&f["inicial"]["publico"]["Health"]), &cfg);
    let mut dif = compara("inicial", &f["inicial"], &cfg, &v);
    let passos = f["passos"].as_array().expect("passos");
    for p in passos {
        let i = p["passo"].as_u64().expect("passo");
        let dt_ms = num(&p["dt_efectivo_ms"]);
        v.anda(dt_ms);
        v.pre_quadro(&cfg, Regras::GDEVELOP, dt_ms / 1000.0);
        dif.extend(compara(&format!("#{i}.antes"), &p["antes"], &cfg, &v));
        let esperados = p["sorteios"]["n"].as_u64().expect("n") as usize;
        let valores: Vec<f64> = p["sorteios"]["valores"]
            .as_array()
            .map(|a| a.iter().map(num).collect())
            .unwrap_or_default();
        let mut usados = 0usize;
        // ⚠️ Nas fixturas sem semente o alvo não grava o valor (a chance é `0` ou `1`, e qualquer
        // valor em `[0, 1)` dá o mesmo veredicto) — só a CONTAGEM é comparada ali.
        let mut sorteio = || {
            let r = valores.get(usados).copied().unwrap_or(0.5);
            usados += 1;
            r
        };
        for a in p["acoes"].as_array().expect("acoes") {
            aplica(a.as_array().expect("acção"), &mut cfg, &mut v, &mut sorteio);
        }
        if usados != esperados {
            dif.push(format!(
                "#{i}: a lei sorteou {usados} vezes, o alvo {esperados}"
            ));
        }
        dif.extend(compara(&format!("#{i}.depois"), &p["depois"], &cfg, &v));
        dif.extend(compara(&format!("#{i}.fim"), &p["fim"], &cfg, &v));
    }
    (passos.len(), dif)
}

/// ⭐⭐⭐ **As 19 fixturas, quadro a quadro, ao bit.**
#[test]
fn a_lei_reproduz_o_oraculo_ao_bit_em_todos_os_quadros() {
    let mut fixturas: Vec<_> = std::fs::read_dir(dir())
        .expect("a pasta das fixturas")
        .map(|e| e.expect("entrada").path())
        .filter(|p| p.to_string_lossy().ends_with(".json.gz"))
        .collect();
    fixturas.sort();
    // ⚠️ O PISO de população: uma pasta vazia deixaria o gate verde a medir nada. E ele é o NÚMERO
    // do corpus, não um mínimo folgado: a 19.ª (`d3_regeneracao_passa_do_maximo`) nasceu de uma
    // mutação SOBREVIVENTE, e apagá-la devolveria o corte da regeneração a ninguém o medir.
    assert!(fixturas.len() >= 19, "só {} fixturas", fixturas.len());
    let mut quadros = 0;
    let mut falhas = Vec::new();
    for f in &fixturas {
        let (n, dif) = corre(&inflar(f));
        quadros += n;
        if !dif.is_empty() {
            let nome = f.file_name().expect("nome").to_string_lossy().into_owned();
            falhas.push(format!(
                "{nome} ({} diferenças):\n  {}",
                dif.len(),
                dif[..dif.len().min(6)].join("\n  ")
            ));
        }
    }
    assert!(quadros > 300, "só {quadros} quadros");
    assert!(
        falhas.is_empty(),
        "{} de {} fixturas divergem:\n{}",
        falhas.len(),
        fixturas.len(),
        falhas.join("\n")
    );
}
