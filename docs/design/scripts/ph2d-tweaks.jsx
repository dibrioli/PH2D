/* global React, ReactDOM */
const { useState, useEffect, useRef } = React;

function useTweaks(defaults) {
  const [t, setT] = useState(defaults);
  const setTweak = (k, v) => {
    const edits = (typeof k === 'object') ? k : { [k]: v };
    setT(prev => ({ ...prev, ...edits }));
    try { window.parent.postMessage({ type: '__edit_mode_set_keys', edits }, '*'); } catch {}
  };
  return [t, setTweak];
}

function TweaksPanel({ open, onClose, children }) {
  const ref = useRef(null);
  useEffect(() => {
    const onMsg = (e) => {
      if (e.data?.type === '__activate_edit_mode') window.dispatchEvent(new CustomEvent('ph2d-tweaks-open'));
      if (e.data?.type === '__deactivate_edit_mode') window.dispatchEvent(new CustomEvent('ph2d-tweaks-close'));
    };
    window.addEventListener('message', onMsg);
    try { window.parent.postMessage({ type: '__edit_mode_available' }, '*'); } catch {}
    return () => window.removeEventListener('message', onMsg);
  }, []);
  if (!open) return null;
  return (
    <div ref={ref} style={{
      position:'fixed', right:20, bottom:20, width:300, zIndex:100,
      background:'var(--bg-elev)', backdropFilter:'blur(24px) saturate(140%)',
      WebkitBackdropFilter:'blur(24px) saturate(140%)',
      border:'1px solid var(--border)', borderRadius:'var(--r-lg)',
      boxShadow:'var(--shadow-lg), var(--inset-hi)',
      maxHeight:'80vh', overflow:'hidden', display:'flex', flexDirection:'column',
      fontFamily:'var(--font-sans)'
    }}>
      <div style={{display:'flex', alignItems:'center', justifyContent:'space-between', padding:'12px 14px', borderBottom:'1px solid var(--border)'}}>
        <div style={{font:'var(--fw-semibold) var(--fs-md)/1.2 var(--font-display)', letterSpacing:'-0.02em'}}>Tweaks</div>
        <button onClick={() => { onClose(); try { window.parent.postMessage({type:'__edit_mode_dismissed'},'*'); } catch{} }}
          style={{appearance:'none', border:0, background:'transparent', color:'var(--text-3)', cursor:'pointer', width:24, height:24, borderRadius:6}}>×</button>
      </div>
      <div style={{padding:'12px 14px', overflowY:'auto', display:'flex', flexDirection:'column', gap:18}}>
        {children}
      </div>
    </div>
  );
}

function Section({ title, children }) {
  return (
    <div>
      <div style={{font:'var(--fw-medium) var(--fs-xxs)/1 var(--font-mono)', letterSpacing:'var(--tr-caps)', textTransform:'uppercase', color:'var(--text-3)', marginBottom:8}}>{title}</div>
      <div style={{display:'flex', flexDirection:'column', gap:8}}>{children}</div>
    </div>
  );
}

function Row({ label, children }) {
  return (
    <div style={{display:'flex', alignItems:'center', justifyContent:'space-between', gap:12}}>
      <span style={{fontSize:'var(--fs-sm)', color:'var(--text-2)'}}>{label}</span>
      <div>{children}</div>
    </div>
  );
}

function Seg({ value, options, onChange }) {
  return (
    <div style={{display:'inline-flex', background:'var(--bg-1)', border:'1px solid var(--border)', borderRadius:'var(--r-md)', padding:2, gap:2}}>
      {options.map(o => (
        <button key={o.v} onClick={() => onChange(o.v)}
          style={{appearance:'none', border:0, background: value===o.v? 'var(--bg-3)':'transparent',
            color: value===o.v?'var(--text-1)':'var(--text-2)',
            font:'var(--fw-medium) var(--fs-xs)/1 var(--font-sans)', padding:'0 10px', height:22, borderRadius:'var(--r-sm)', cursor:'pointer'}}>
          {o.l}
        </button>
      ))}
    </div>
  );
}

function ColorChips({ value, options, onChange }) {
  return (
    <div style={{display:'flex', gap:6}}>
      {options.map(o => (
        <button key={o} onClick={() => onChange(o)} title={o}
          style={{width:24, height:24, borderRadius:'var(--r-sm)', border:'1px solid var(--border)', cursor:'pointer',
            background:`oklch(0.74 0.16 ${({magenta:340,cyan:205,orange:55,blue:250,lime:130})[o]})`,
            boxShadow: value===o? `0 0 0 2px var(--bg-1), 0 0 0 4px oklch(0.74 0.16 ${({magenta:340,cyan:205,orange:55,blue:250,lime:130})[o]})` : 'none'}}/>
      ))}
    </div>
  );
}

window.PH2DTweaks = { TweaksPanel, Section, Row, Seg, ColorChips, useTweaks };
