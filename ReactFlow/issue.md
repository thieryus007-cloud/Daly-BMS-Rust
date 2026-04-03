Voici la solution unique qui fonctionne :

---

Dans PowerShell, exécutez cette commande :

```powershell
cd C:\reactflow-energie && New-Item -ItemType Directory -Path "src\styles" -Force | Out-Null && @'
.mppt-node {
  min-width: 260px;
  background: linear-gradient(135deg, #1a2a1a 0%, #0d1a0d 100%);
  border-radius: 20px;
  padding: 16px;
  border: 2px solid #4caf50;
  font-family: "Segoe UI", monospace;
}
.mppt-total-power { text-align: center; margin-bottom: 16px; padding: 8px; background: #1a2a1a; border-radius: 16px; }
.total-value { font-size: 36px; font-weight: bold; color: #4caf50; }
.mppt-list { display: flex; flex-direction: column; gap: 12px; }
.mppt-item { background: #1a2a1a; border-radius: 12px; padding: 10px; }
.mppt-id { font-size: 11px; font-weight: bold; color: #88ff88; }
.mppt-metrics { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.metric { background: #0d1a0d; border-radius: 8px; padding: 6px; text-align: center; }
.metric-label { display: block; font-size: 8px; color: #888; }
.metric-value { display: block; font-size: 12px; font-weight: bold; color: #ddd; }
'@ | Out-File -FilePath "src\styles\mpptAnimations.css" -Encoding utf8 && (Get-Content "src\components\nodes\MPPTNode.jsx") -replace 'import "./mpptAnimations.css"', 'import "../styles/mpptAnimations.css"' | Set-Content "src\components\nodes\MPPTNode.jsx" && Write-Host "✅ Fichier créé et import corrigé. Redémarrez le serveur avec : npm run dev" -ForegroundColor Green
```

---

Après exécution, redémarrez le serveur :

```powershell
npm run dev
```

Puis ouvrez http://localhost:3000/
