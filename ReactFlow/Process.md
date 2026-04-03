SCRIPT POWERSSHELL - Création complète de l'arborescence

Instructions d'utilisation

1. Ouvrez PowerShell (en tant qu'utilisateur normal, pas besoin d'administrateur)
2. Copiez-collez l'intégralité du script ci-dessous
3. Appuyez sur Entrée

Le script va :

· Créer le dossier du projet reactflow-energie
· Créer toute l'arborescence des dossiers
· Créer tous les fichiers vides avec les bons noms
· Afficher la liste des fichiers créés

---

Le script

```powershell
# SCRIPT DE CRÉATION DE L'ARBORESCENCE REACTFLOW ENERGIE
# À exécuter dans PowerShell sous Windows

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "CREATION DE L'ARBORESCENCE REACTFLOW" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Définir le chemin du projet (dossier courant)
$projectPath = "$PSScriptRoot\reactflow-energie"

Write-Host "`nCréation du projet dans : $projectPath" -ForegroundColor Yellow

# 1. Création du dossier principal
New-Item -ItemType Directory -Path $projectPath -Force | Out-Null

# 2. Création de l'arborescence src
New-Item -ItemType Directory -Path "$projectPath\src" -Force | Out-Null
New-Item -ItemType Directory -Path "$projectPath\src\components" -Force | Out-Null
New-Item -ItemType Directory -Path "$projectPath\src\components\nodes" -Force | Out-Null
New-Item -ItemType Directory -Path "$projectPath\src\pages" -Force | Out-Null
New-Item -ItemType Directory -Path "$projectPath\src\styles" -Force | Out-Null

# 3. Création des fichiers à la racine
New-Item -ItemType File -Path "$projectPath\index.html" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\package.json" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\vite.config.js" -Force | Out-Null

# 4. Création des fichiers src
New-Item -ItemType File -Path "$projectPath\src\App.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\main.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\index.css" -Force | Out-Null

# 5. Création des composants NodeType (dossier nodes)
New-Item -ItemType File -Path "$projectPath\src\components\nodes\BatteryNode.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\components\nodes\ET112Node.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\components\nodes\SwitchNode.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\components\nodes\ShuntNode.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\components\nodes\MeteoNode.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\components\nodes\TemperatureNode.jsx" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\components\nodes\MPPTNode.jsx" -Force | Out-Null

# 6. Création de la page de visualisation
New-Item -ItemType File -Path "$projectPath\src\pages\VisualisationComplete.jsx" -Force | Out-Null

# 7. Création des fichiers CSS (styles)
New-Item -ItemType File -Path "$projectPath\src\styles\batteryAnimations.css" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\styles\et112Animations.css" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\styles\switchAnimations.css" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\styles\shuntAnimations.css" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\styles\meteoAnimations.css" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\styles\temperatureAnimations.css" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\styles\mpptAnimations.css" -Force | Out-Null
New-Item -ItemType File -Path "$projectPath\src\styles\energyCommon.css" -Force | Out-Null

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "ARBORESCENCE CREEE AVEC SUCCES !" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

Write-Host "`nListe des fichiers créés :" -ForegroundColor Cyan
Write-Host "------------------------" -ForegroundColor Cyan

Get-ChildItem -Path $projectPath -Recurse -File | ForEach-Object {
    $relativePath = $_.FullName.Replace($projectPath, "")
    Write-Host "  📄 $relativePath" -ForegroundColor White
}

Write-Host "`n========================================" -ForegroundColor Yellow
Write-Host "PROCHAINES ETAPES :" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "1. Ouvrir le dossier : cd $projectPath" -ForegroundColor White
Write-Host "2. Installer les dépendances : npm install @xyflow/react" -ForegroundColor White
Write-Host "3. Lancer le serveur : npm run dev" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Yellow

# Ouvrir l'explorateur Windows à l'emplacement du projet
Start-Process explorer.exe $projectPath
```

---

Instructions détaillées d'utilisation

Étape 1 : Ouvrir PowerShell

· Appuyez sur Win + R
· Tapez powershell
· Appuyez sur Entrée

Étape 2 : Se placer dans le dossier souhaité

```powershell
cd C:\Users\VotreNom\Documents
```

(remplacez VotreNom par votre nom d'utilisateur)

Étape 3 : Copier et exécuter le script

· Copiez l'intégralité du script ci-dessus
· Collez-le dans PowerShell
· Appuyez sur Entrée

Étape 4 : Vérification

L'explorateur Windows s'ouvre automatiquement avec le dossier reactflow-energie

---

Tableau de correspondance - Fichiers et contenu

Après avoir exécuté le script, voici où trouver le contenu à copier pour chaque fichier :

Fichier créé Où trouver le contenu (section dans les docs)
package.json Guide installation - Section 9
vite.config.js Guide installation - Section 5
index.html (Créer manuellement - voir ci-dessous)
src/App.jsx Guide installation - Section 4.1
src/main.jsx Guide installation - Section 4.2
src/index.css Guide installation - Section 4.3
src/components/nodes/BatteryNode.jsx Doc Battery - Section 4 (version enrichie)
src/components/nodes/ET112Node.jsx Doc ET112 - Section 4
src/components/nodes/SwitchNode.jsx Doc Switch - Section 4
src/components/nodes/ShuntNode.jsx Doc Shunt - Section 4 (version enrichie)
src/components/nodes/MeteoNode.jsx Doc Meteo - Section 4
src/components/nodes/TemperatureNode.jsx Doc Temperature - Section 6
src/components/nodes/MPPTNode.jsx Doc MPPT - Section 3
src/pages/VisualisationComplete.jsx Doc MPPT - Section 6
src/styles/*.css Chaque doc a sa section CSS correspondante

---

Fichier index.html manquant - À créer à la racine

Fichier : index.html (à la racine du projet)

```html
<!DOCTYPE html>
<html lang="fr">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Supervision Énergétique - React Flow</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.jsx"></script>
  </body>
</html>
```

---

Commande unique pour tout créer (copier-coller dans PowerShell)

Si vous voulez tout créer et ouvrir VS Code directement :

```powershell
# Création du projet + ouverture dans VS Code
$projectPath = "$PSScriptRoot\reactflow-energie"
New-Item -ItemType Directory -Path $projectPath -Force
cd $projectPath
npm init -y
npm install @xyflow/react react react-dom
npm install -D vite @vitejs/plugin-react

# Création des dossiers
mkdir src\components\nodes, src\pages, src\styles -Force

# Création des fichiers vides
$files = @(
    "src/App.jsx", "src/main.jsx", "src/index.css",
    "src/components/nodes/BatteryNode.jsx",
    "src/components/nodes/ET112Node.jsx",
    "src/components/nodes/SwitchNode.jsx",
    "src/components/nodes/ShuntNode.jsx",
    "src/components/nodes/MeteoNode.jsx",
    "src/components/nodes/TemperatureNode.jsx",
    "src/components/nodes/MPPTNode.jsx",
    "src/pages/VisualisationComplete.jsx",
    "src/styles/batteryAnimations.css",
    "src/styles/et112Animations.css",
    "src/styles/switchAnimations.css",
    "src/styles/shuntAnimations.css",
    "src/styles/meteoAnimations.css",
    "src/styles/temperatureAnimations.css",
    "src/styles/mpptAnimations.css",
    "src/styles/energyCommon.css"
)
foreach ($file in $files) { New-Item -Path $projectPath\$file -Force }

code .
```

---

Récapitulatif - Ce que vous devez faire APRÈS le script

Ordre Action
1 Exécuter le script PowerShell ci-dessus
2 Ouvrir le dossier reactflow-energie dans VS Code
3 Pour CHAQUE fichier, copier le contenu depuis les documents précédents
4 Ouvrir un terminal dans VS Code (`Ctrl + ``)
5 Taper npm install @xyflow/react
6 Taper npm run dev
7 Ouvrir http://localhost:5173/

---

Fin du script - Exécutez-le et vous aurez toute l'arborescence prête à recevoir le contenu des fichiers.
