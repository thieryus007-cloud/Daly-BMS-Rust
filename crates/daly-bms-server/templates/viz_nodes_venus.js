// ── MPPT GROUP NODE ───────────────────────────────────────────────────────────
function MPPTGroupNode({ data }) {
  const mppts       = data.mppts       ?? [];
  const totalPowerW = data.totalPowerW ?? 0;
  const totalDcA    = data.totalDcA    ?? 0;

  const stateClass = (s) => {
    if (!s) return '';
    const l = s.toLowerCase();
    if (l === 'off')   return 'off';
    if (l === 'fault') return 'fault';
    return '';
  };

  return h('div', { className: 'mppt-group-card' },
    mkHandle('target', Position.Top,    'tt'),
    mkHandle('source', Position.Bottom, 'sb'),
    mkHandle('target', Position.Left,   'tl'),
    mkHandle('source', Position.Right,  'sr'),
    mkHandle('target', Position.Right,  'tr', { top: '65%' }),
    mkHandle('source', Position.Left,   'sl', { top: '65%' }),
    h('div', { className: 'mg-header' },
      h('span', { className: 'mg-header-icon' }, '☀️'),
      h('span', { className: 'mg-header-title' }, 'SmartSolar MPPT'),
      h('span', { style: { fontSize:'0.65rem', opacity:0.8, marginLeft:'auto' } }, `${mppts.length} chargeur${mppts.length > 1 ? 's' : ''}`)
    ),
    h('div', { className: 'mg-totals' },
      h('div', { className: 'mg-total-chip' },
        h('span', { className: 'mg-total-lbl' }, 'Total Puissance'),
        h('span', { className: 'mg-total-val' }, `${Math.round(totalPowerW)} W`)
      ),
      h('div', { className: 'mg-total-chip' },
        h('span', { className: 'mg-total-lbl' }, 'Total DC'),
        h('span', { className: 'mg-total-val' }, `${Math.round(totalDcA)} A`)
      )
    ),
    mppts.length > 0 ? h('div', { className: 'mg-mppt-list' },
      mppts.map((m, i) => {
        const inst  = m.instance ?? i;
        const name  = inst > 0 ? `MPPT-${inst}` : `MPPT-${i+1}`;
        const s     = m.state ?? null;
        const pvV   = m.pv_voltage_v ?? null;
        const dcA   = m.dc_current_a ?? null;
        const pw    = m.power_w ?? null;
        return h('div', { key: inst, className: 'mg-mppt-row' },
          h('span', { className: 'mg-mppt-name' }, name),
          h('span', { className: `mg-mppt-state ${stateClass(s)}` }, s ?? '—'),
          h('div', { className: 'mg-mppt-metrics' },
            h('div', { className: 'mg-metric' },
              h('span', { className: 'mg-metric-lbl' }, 'PV'),
              h('span', { className: 'mg-metric-val' }, pvV != null ? `${pvV.toFixed(1)}V` : '—')
            ),
            h('div', { className: 'mg-metric' },
              h('span', { className: 'mg-metric-lbl' }, 'DC'),
              h('span', { className: 'mg-metric-val' }, dcA != null ? `${dcA.toFixed(1)}A` : '—')
            ),
            h('div', { className: 'mg-metric' },
              h('span', { className: 'mg-metric-lbl' }, 'W'),
              h('span', { className: 'mg-metric-val' }, pw != null ? `${Math.round(pw)}` : '—')
            )
          )
        );
      })
    ) : h('div', { className: 'mg-wait' }, 'En attente des chargeurs…')
  );
}

// ── SMARTSHUNT NODE ───────────────────────────────────────────────────────────
function SmartShuntNode({ data }) {
  const live    = data.live;
  const soc     = live?.soc_percent   ?? null;
  const voltage = live?.voltage_v     ?? null;
  const current = live?.current_a     ?? null;
  const power   = live?.power_w       ?? null;
  const eIn     = live?.energy_in_kwh ?? null;
  const eOut    = live?.energy_out_kwh ?? null;
  const state   = live?.state         ?? null;
  const ttgMin  = live?.time_to_go_min ?? null;

  const stateClass = state === 'Charging' ? 'charging' : state === 'Discharging' ? 'discharging' : 'idle';

  const fmtTtg = (min) => {
    if (min == null) return '—';
    if (min >= 1440) return `${(min/1440).toFixed(0)}j`;
    if (min >= 60)   return `${Math.floor(min/60)}h${Math.round(min%60).toString().padStart(2,'0')}`;
    return `${Math.round(min)} min`;
  };

  return h('div', { className: 'ss-card' },
    mkHandle('target', Position.Top,    'tt'),
    mkHandle('source', Position.Bottom, 'sb'),
    mkHandle('target', Position.Left,   'tl'),
    mkHandle('source', Position.Right,  'sr'),
    mkHandle('target', Position.Right,  'tr', { top: '65%' }),
    mkHandle('source', Position.Left,   'sl', { top: '65%' }),
    h('div', { className: 'ss-hdr' },
      h('span', { className: 'ss-hdr-icon' }, '⚡'),
      h('span', { className: 'ss-hdr-title' }, 'SmartShunt'),
      state && h('span', { className: `ss-state-badge ${stateClass}` }, state),
      h('div', { className: `ss-dot${live ? ' live' : ''}` })
    ),
    h('div', { className: 'ss-bar-bg' },
      h('div', { className: 'ss-bar-fill', style: { width: `${soc ?? 0}%`, background: soc != null ? socColor(soc) : '#e2e8f0' } })
    ),
    live ? h('div', null,
      h('div', { className: 'ss-grid' },
        h('div', { className: 'ss-cell' },
          h('span', { className: 'ss-cell-lbl' }, 'SOC'),
          h('span', { className: 'ss-cell-val', style: { color: soc != null ? socColor(soc) : undefined } }, soc != null ? `${soc.toFixed(1)}%` : '—')
        ),
        h('div', { className: 'ss-cell' },
          h('span', { className: 'ss-cell-lbl' }, 'Tension'),
          h('span', { className: 'ss-cell-val' }, voltage != null ? `${voltage.toFixed(1)}V` : '—')
        ),
        h('div', { className: 'ss-cell' },
          h('span', { className: 'ss-cell-lbl' }, 'Courant'),
          h('span', { className: `ss-cell-val ${current != null ? (current < 0 ? 'neg' : current > 0.1 ? 'pos' : '') : ''}` },
            current != null ? `${current > 0 ? '+' : ''}${current.toFixed(1)}A` : '—')
        )
      ),
      h('div', { className: 'ss-row' },
        h('span', { className: 'ss-row-lbl' }, 'Puissance'),
        h('span', { className: 'ss-row-val' }, power != null ? `${power > 0 ? '+' : ''}${power.toFixed(0)} W` : '—')
      ),
      h('div', { style: { display: 'flex', gap: '1rem', flexWrap: 'wrap' } },
        h('div', { className: 'ss-row', style: { flex: '1 1 0', minWidth: 0 } },
          h('span', { className: 'ss-row-lbl' }, 'Temps restant'),
          h('span', { className: 'ss-row-val ttg' }, fmtTtg(ttgMin))
        ),
        h('div', { className: 'ss-row', style: { flex: '1 1 0', minWidth: 0 } },
          h('span', { className: 'ss-row-lbl' }, '⬆ Chargée'),
          h('span', { className: 'ss-row-val' }, eIn != null ? `${eIn.toFixed(1)} kWh` : '—')
        ),
        h('div', { className: 'ss-row', style: { flex: '1 1 0', minWidth: 0 } },
          h('span', { className: 'ss-row-lbl' }, '⬇ Déchargée'),
          h('span', { className: 'ss-row-val' }, eOut != null ? `${eOut.toFixed(1)} kWh` : '—')
        )
      )
    ) : h('div', { className: 'ss-wait' }, 'En attente…')
  );
}

// ── TEMPERATURE NODE ──────────────────────────────────────────────────────────
function TemperatureNode({ data }) {
  const live = data.live;
  const temp = live?.temp_c ?? null;
  const humidity = live?.humidity_percent ?? null;

  return h('div', { className: 'dev-nd' },
    mkHandle('target', Position.Top,    'tt'),
    mkHandle('source', Position.Bottom, 'sb'),
    mkHandle('target', Position.Left,   'tl'),
    mkHandle('source', Position.Right,  'sr'),
    mkHandle('target', Position.Right,  'tr', { top: '65%' }),
    mkHandle('source', Position.Left,   'sl', { top: '65%' }),
    h('div', { className: 'dev-hdr' },
      h('span', null, data.icon ?? '🌡️'),
      h('span', { className: 'dev-lbl' }, data.label),
      h('div', { className: `dev-dot${live ? ' on' : ''}` })
    ),
    live ? h('div', null,
      h('div', { className: 'dev-pwr' }, temp != null ? `${temp.toFixed(1)}°C` : '—'),
      h('div', { className: 'dev-nrj' }, humidity != null ? `${humidity.toFixed(0)}% HR` : '—')
    ) : h('div', { className: 'dev-wait' }, 'En attente…')
  );
}
