//! Générateurs d'options ECharts côté serveur (Rust → JSON → template HTML).
//!
//! Chaque fonction retourne une `String` JSON représentant un objet `option`
//! ECharts complet, prêt à être injecté dans `echarts.setOption(...)`.
//!
//! Le rendu graphique est assuré par la bibliothèque ECharts (JS) côté navigateur ;
//! Rust ne fait que construire la configuration, sans aucune dépendance JS.

use daly_bms_core::types::BmsSnapshot;
use std::collections::BTreeMap;

// ─── Palette couleurs (dark theme GitHub-like) ───────────────────────────────
const C_BG:      &str = "transparent";
const C_MUTED:   &str = "#8b949e";
const C_GRID:    &str = "#21262d";
const C_AXIS:    &str = "#30363d";
const C_BLUE:    &str = "#58a6ff";
const C_GREEN:   &str = "#3fb950";
const C_YELLOW:  &str = "#d29922";
const C_RED:     &str = "#f85149";
const C_ORANGE:  &str = "#fb8500";

// =============================================================================
// Jauge SOC (page d'accueil — mini, et détail — grande)
// =============================================================================

/// Génère l'option ECharts pour une jauge SOC.
/// `size` indique le style : "mini" pour les cartes, "full" pour le détail.
pub fn soc_gauge(soc: f32, size: &str) -> String {
    let font_size  = if size == "full" { 36 } else { 22 };
    let title_size = if size == "full" { 13  } else { 10 };
    let radius     = if size == "full" { "88%" } else { "85%" };
    let line_width = if size == "full" { 16 } else { 10 };

    // Couleur de la valeur selon seuils SOC
    let color = match soc as u32 {
        0..=14  => C_RED,
        15..=24 => C_ORANGE,
        25..=39 => C_YELLOW,
        _       => C_GREEN,
    };

    format!(r#"{{
  "backgroundColor": "{bg}",
  "series": [{{
    "type": "gauge",
    "startAngle": 205,
    "endAngle": -25,
    "min": 0,
    "max": 100,
    "splitNumber": 5,
    "radius": "{radius}",
    "center": ["50%", "55%"],
    "axisLine": {{
      "lineStyle": {{
        "width": {lw},
        "color": [
          [0.15, "{c_red}"],
          [0.25, "{c_orange}"],
          [0.40, "{c_yellow}"],
          [1.00, "{c_green}"]
        ]
      }}
    }},
    "pointer": {{
      "show": true,
      "length": "58%",
      "width": 4,
      "itemStyle": {{ "color": "auto" }}
    }},
    "axisTick":  {{ "show": false }},
    "splitLine": {{ "show": false }},
    "axisLabel": {{ "show": false }},
    "detail": {{
      "valueAnimation": true,
      "formatter": "{{value}}%",
      "color":     "{c_val}",
      "fontSize":  {fs},
      "fontWeight": "bold",
      "offsetCenter": [0, "20%"]
    }},
    "title": {{
      "color":        "{c_title}",
      "fontSize":     {ts},
      "offsetCenter": [0, "50%"]
    }},
    "data": [{{"value": {soc:.1}, "name": "SOC"}}]
  }}]
}}"#,
        bg      = C_BG,
        radius  = radius,
        lw      = line_width,
        c_red   = C_RED,
        c_orange= C_ORANGE,
        c_yellow= C_YELLOW,
        c_green = C_GREEN,
        c_val   = color,
        c_title = C_MUTED,
        fs      = font_size,
        ts      = title_size,
        soc     = soc,
    )
}

// =============================================================================
// Barres — tensions des cellules
// =============================================================================

/// Génère l'option ECharts pour le graphe de tensions des cellules (bar chart).
pub fn cell_voltages_bar(voltages: &BTreeMap<String, f32>) -> String {
    let labels: Vec<String> = voltages.keys()
        .map(|k| {
            // "Cell1" → "C1"
            let n = k.trim_start_matches("Cell");
            format!("\"C{}\"", n)
        })
        .collect();

    let values: Vec<String> = voltages.values()
        .map(|&v| {
            // Colorier en rouge si hors plage normale (< 2.95V ou > 3.62V)
            let color = if v < 2.95 || v > 3.62 { C_RED }
                        else if (v - voltages.values().cloned()
                            .fold(f32::INFINITY, f32::min)).abs() < 0.001 { C_YELLOW }
                        else { C_BLUE };
            format!(r#"{{"value": {:.3}, "itemStyle": {{"color": "{}"}}}}"#, v, color)
        })
        .collect();

    format!(r#"{{
  "backgroundColor": "{bg}",
  "animation": false,
  "grid": {{ "left": "2%", "right": "2%", "top": "8%", "bottom": "12%", "containLabel": true }},
  "xAxis": {{
    "type":      "category",
    "data":      [{labels}],
    "axisLabel": {{ "color": "{muted}", "fontSize": 9 }},
    "axisLine":  {{ "lineStyle": {{ "color": "{axis}" }} }}
  }},
  "yAxis": {{
    "type":      "value",
    "min":       2.9,
    "max":       3.7,
    "splitNumber": 4,
    "axisLabel": {{ "color": "{muted}", "formatter": "{{value}}V", "fontSize": 9 }},
    "splitLine": {{ "lineStyle": {{ "color": "{grid}", "type": "dashed" }} }}
  }},
  "series": [{{
    "type": "bar",
    "data": [{values}],
    "barMaxWidth": 20,
    "itemStyle": {{ "borderRadius": [3, 3, 0, 0] }},
    "markLine": {{
      "silent": true,
      "symbol": "none",
      "data": [
        {{ "yAxis": 2.95, "lineStyle": {{ "color": "{red}",    "type": "dashed" }}, "label": {{ "show": false }} }},
        {{ "yAxis": 3.62, "lineStyle": {{ "color": "{red}",    "type": "dashed" }}, "label": {{ "show": false }} }},
        {{ "yAxis": 3.40, "lineStyle": {{ "color": "{yellow}", "type": "dotted" }}, "label": {{ "show": false }} }}
      ]
    }}
  }}]
}}"#,
        bg     = C_BG,
        labels = labels.join(", "),
        values = values.join(", "),
        muted  = C_MUTED,
        axis   = C_AXIS,
        grid   = C_GRID,
        red    = C_RED,
        yellow = C_YELLOW,
    )
}

// =============================================================================
// Lignes — historique SOC + courant
// =============================================================================

/// Données d'historique extraites de la série de snapshots.
pub struct HistoryData {
    pub timestamps: Vec<String>,
    pub soc:        Vec<f32>,
    pub current:    Vec<f32>,
    pub voltage:    Vec<f32>,
    pub temp_max:   Vec<f32>,
}

impl HistoryData {
    /// Construit depuis une liste de snapshots (ordre chronologique, du plus ancien au plus récent).
    pub fn from_snapshots(snaps: &[BmsSnapshot]) -> Self {
        let mut timestamps = Vec::with_capacity(snaps.len());
        let mut soc        = Vec::with_capacity(snaps.len());
        let mut current    = Vec::with_capacity(snaps.len());
        let mut voltage    = Vec::with_capacity(snaps.len());
        let mut temp_max   = Vec::with_capacity(snaps.len());

        for s in snaps {
            timestamps.push(s.timestamp.format("%H:%M:%S").to_string());
            soc.push(s.soc);
            current.push(s.dc.current);
            voltage.push(s.dc.voltage);
            temp_max.push(s.system.max_cell_temperature);
        }
        Self { timestamps, soc, current, voltage, temp_max }
    }
}

/// Génère l'option ECharts pour l'historique SOC (line chart avec aire).
pub fn soc_history_line(data: &HistoryData) -> String {
    let ts_json  = json_str_array(&data.timestamps);
    let soc_json = json_f32_array(&data.soc);

    format!(r#"{{
  "backgroundColor": "{bg}",
  "animation": false,
  "grid": {{ "left": "3%", "right": "2%", "top": "8%", "bottom": "18%", "containLabel": true }},
  "xAxis": {{
    "type":      "category",
    "data":      {ts},
    "axisLabel": {{ "color": "{muted}", "fontSize": 8, "rotate": 30, "interval": "auto" }},
    "axisLine":  {{ "lineStyle": {{ "color": "{axis}" }} }}
  }},
  "yAxis": {{
    "type":      "value",
    "min":       0,
    "max":       100,
    "axisLabel": {{ "color": "{muted}", "formatter": "{{value}}%", "fontSize": 9 }},
    "splitLine": {{ "lineStyle": {{ "color": "{grid}", "type": "dashed" }} }}
  }},
  "series": [{{
    "type":   "line",
    "data":   {soc},
    "smooth": true,
    "symbol": "none",
    "lineStyle": {{ "color": "{green}", "width": 2 }},
    "areaStyle": {{
      "color": {{
        "type": "linear", "x": 0, "y": 0, "x2": 0, "y2": 1,
        "colorStops": [
          {{ "offset": 0, "color": "rgba(63,185,80,0.35)" }},
          {{ "offset": 1, "color": "rgba(63,185,80,0.02)" }}
        ]
      }}
    }}
  }}],
  "dataZoom": [{{ "type": "inside" }}, {{ "type": "slider", "height": 16, "bottom": 0 }}]
}}"#,
        bg    = C_BG,
        ts    = ts_json,
        soc   = soc_json,
        muted = C_MUTED,
        axis  = C_AXIS,
        grid  = C_GRID,
        green = C_GREEN,
    )
}

/// Génère l'option ECharts pour l'historique courant (+ charge, - décharge).
pub fn current_history_line(data: &HistoryData) -> String {
    let ts_json      = json_str_array(&data.timestamps);
    let current_json = json_f32_array(&data.current);

    format!(r#"{{
  "backgroundColor": "{bg}",
  "animation": false,
  "grid": {{ "left": "3%", "right": "2%", "top": "8%", "bottom": "18%", "containLabel": true }},
  "xAxis": {{
    "type":      "category",
    "data":      {ts},
    "axisLabel": {{ "color": "{muted}", "fontSize": 8, "rotate": 30, "interval": "auto" }},
    "axisLine":  {{ "lineStyle": {{ "color": "{axis}" }} }}
  }},
  "yAxis": {{
    "type":      "value",
    "axisLabel": {{ "color": "{muted}", "formatter": "{{value}}A", "fontSize": 9 }},
    "splitLine": {{ "lineStyle": {{ "color": "{grid}", "type": "dashed" }} }}
  }},
  "series": [{{
    "type":   "line",
    "data":   {cur},
    "smooth": true,
    "symbol": "none",
    "lineStyle": {{ "color": "{blue}", "width": 2 }},
    "markLine": {{
      "silent": true,
      "symbol": "none",
      "data": [{{ "yAxis": 0, "lineStyle": {{ "color": "{muted}", "type": "dashed" }} }}]
    }}
  }}],
  "dataZoom": [{{ "type": "inside" }}, {{ "type": "slider", "height": 16, "bottom": 0 }}]
}}"#,
        bg    = C_BG,
        ts    = ts_json,
        cur   = current_json,
        muted = C_MUTED,
        axis  = C_AXIS,
        grid  = C_GRID,
        blue  = C_BLUE,
    )
}

/// Génère l'option ECharts pour l'historique tension + température (double axe Y).
pub fn voltage_temp_line(data: &HistoryData) -> String {
    let ts_json   = json_str_array(&data.timestamps);
    let volt_json = json_f32_array(&data.voltage);
    let temp_json = json_f32_array(&data.temp_max);

    format!(r#"{{
  "backgroundColor": "{bg}",
  "animation": false,
  "legend": {{
    "data": ["Tension (V)", "Temp max (°C)"],
    "textStyle": {{ "color": "{muted}", "fontSize": 10 }},
    "top": 0
  }},
  "grid": {{ "left": "3%", "right": "5%", "top": "18%", "bottom": "18%", "containLabel": true }},
  "xAxis": {{
    "type":      "category",
    "data":      {ts},
    "axisLabel": {{ "color": "{muted}", "fontSize": 8, "rotate": 30, "interval": "auto" }},
    "axisLine":  {{ "lineStyle": {{ "color": "{axis}" }} }}
  }},
  "yAxis": [
    {{
      "type":      "value",
      "name":      "V",
      "nameTextStyle": {{ "color": "{muted}", "fontSize": 9 }},
      "axisLabel": {{ "color": "{muted}", "formatter": "{{value}}V", "fontSize": 9 }},
      "splitLine": {{ "lineStyle": {{ "color": "{grid}", "type": "dashed" }} }}
    }},
    {{
      "type":      "value",
      "name":      "°C",
      "nameTextStyle": {{ "color": "{muted}", "fontSize": 9 }},
      "axisLabel": {{ "color": "{muted}", "formatter": "{{value}}°C", "fontSize": 9 }},
      "splitLine": {{ "show": false }}
    }}
  ],
  "series": [
    {{
      "name":   "Tension (V)",
      "type":   "line",
      "yAxisIndex": 0,
      "data":   {volt},
      "smooth": true,
      "symbol": "none",
      "lineStyle": {{ "color": "{blue}", "width": 2 }}
    }},
    {{
      "name":   "Temp max (°C)",
      "type":   "line",
      "yAxisIndex": 1,
      "data":   {temp},
      "smooth": true,
      "symbol": "none",
      "lineStyle": {{ "color": "{orange}", "width": 2 }}
    }}
  ],
  "dataZoom": [{{ "type": "inside" }}, {{ "type": "slider", "height": 16, "bottom": 0 }}]
}}"#,
        bg     = C_BG,
        ts     = ts_json,
        volt   = volt_json,
        temp   = temp_json,
        muted  = C_MUTED,
        axis   = C_AXIS,
        grid   = C_GRID,
        blue   = C_BLUE,
        orange = C_ORANGE,
    )
}

// =============================================================================
// Utilitaires de sérialisation JSON
// =============================================================================

fn json_str_array(v: &[String]) -> String {
    let inner: Vec<String> = v.iter()
        .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
        .collect();
    format!("[{}]", inner.join(","))
}

fn json_f32_array(v: &[f32]) -> String {
    let inner: Vec<String> = v.iter()
        .map(|f| format!("{:.3}", f))
        .collect();
    format!("[{}]", inner.join(","))
}
