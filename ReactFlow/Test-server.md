GUIDE COMPLET - Installation Serveur Node.js + React Flow sur Windows

Version : 1.0
Date : Avril 2026
Système : Windows 10/11

---

1. Prérequis - Installation des outils nécessaires

1.1 Installer Node.js

1. Téléchargez Node.js depuis le site officiel : https://nodejs.org/
2. Choisissez la version LTS (recommandée)
3. Lancez l'installateur .msi
4. Cochez "Automatically install the necessary tools" (optionnel mais recommandé)
5. Terminez l'installation

Vérification dans PowerShell ou CMD :

```powershell
node --version
# Doit afficher : v20.x.x ou supérieur

npm --version
# Doit afficher : 10.x.x ou supérieur
```

1.2 Installer Git (optionnel mais recommandé)

1. Téléchargez Git : https://git-scm.com/download/win
2. Installation par défaut
3. Vérification : git --version

1.3 Choisir un éditeur de code

· VS Code (recommandé) : https://code.visualstudio.com/
· Ou tout autre éditeur (Notepad++, Sublime Text, etc.)

---

2. Création du projet

2.1 Ouvrir un terminal (PowerShell ou CMD)

```powershell
# Aller dans le dossier où vous voulez créer le projet
cd C:\Users\VotreNom\Documents

# Créer le dossier du projet
mkdir reactflow-energie
cd reactflow-energie
```

2.2 Créer l'application Vite + React

```powershell
# Créer le projet avec Vite
npm create vite@latest . -- --template react

# Si on vous demande "Install the following package", tapez "y"
```

2.3 Installer les dépendances

```powershell
# Installer React Flow et autres dépendances
npm install @xyflow/react

# Installer les dépendances pour le serveur (optionnel - développement)
npm install -D vite-plugin-restart
```

2.4 Lancer l'application pour vérifier

```powershell
npm run dev
```

Vous devriez voir :

```
  VITE v5.x.x  ready in xxx ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

Ouvrez http://localhost:5173/ dans votre navigateur.

Arrêtez le serveur avec Ctrl + C pour continuer.

---

3. Structure des dossiers à créer

```powershell
# Dans le dossier du projet, créer l'arborescence
mkdir src\components\nodes
mkdir src\pages
mkdir src\styles
```

Structure finale attendue :

```
C:\Users\VotreNom\Documents\reactflow-energie\
├── index.html
├── package.json
├── vite.config.js
├── src\
│   ├── App.jsx
│   ├── main.jsx
│   ├── index.css
│   ├── components\
│   │   └── nodes\
│   │       ├── BatteryNode.jsx
│   │       ├── ET112Node.jsx
│   │       ├── SwitchNode.jsx
│   │       ├── ShuntNode.jsx
│   │       ├── MeteoNode.jsx
│   │       ├── TemperatureNode.jsx
│   │       └── MPPTNode.jsx
│   ├── pages\
│   │   └── VisualisationComplete.jsx
│   └── styles\
│       ├── batteryAnimations.css
│       ├── et112Animations.css
│       ├── switchAnimations.css
│       ├── shuntAnimations.css
│       ├── meteoAnimations.css
│       ├── temperatureAnimations.css
│       ├── mpptAnimations.css
│       └── energyCommon.css
```

---

4. Création des fichiers - Copier/Coller

4.1 Fichier src/App.jsx

```jsx
import VisualisationComplete from './pages/VisualisationComplete';
import './index.css';

function App() {
  return <VisualisationComplete />;
}

export default App;
```

4.2 Fichier src/main.jsx

```jsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

4.3 Fichier src/index.css

```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  background: #0a0a0a;
  overflow: hidden;
}
```

4.4 Maintenant, copiez tous les composants NodeType

Pour chaque composant, créez le fichier et copiez le code correspondant :

Fichier Source (section dans ce document)
src/components/nodes/BatteryNode.jsx Section 4 (avec énergie)
src/components/nodes/ET112Node.jsx Section 4
src/components/nodes/SwitchNode.jsx Section 4
src/components/nodes/ShuntNode.jsx Section 4 (version enrichie)
src/components/nodes/MeteoNode.jsx Section 4
src/components/nodes/TemperatureNode.jsx Section 6
src/components/nodes/MPPTNode.jsx Section 3

4.5 Fichier src/pages/VisualisationComplete.jsx

Copiez le code complet de la Section 6 (Intégration dans la page de visualisation globale) du document MPPT.

4.6 Fichiers CSS

Pour chaque composant, créez le fichier CSS correspondant :

```powershell
# Créer tous les fichiers CSS
type nul > src\styles\batteryAnimations.css
type nul > src\styles\et112Animations.css
type nul > src\styles\switchAnimations.css
type nul > src\styles\shuntAnimations.css
type nul > src\styles\meteoAnimations.css
type nul > src\styles\temperatureAnimations.css
type nul > src\styles\mpptAnimations.css
type nul > src\styles\energyCommon.css
```

Copiez les styles correspondants dans chaque fichier.

---

5. Configuration Vite (optionnel mais recommandé)

Fichier : vite.config.js (à la racine du projet)

```javascript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    open: true,
    host: true
  }
});
```

---

6. Lancement du projet

6.1 Démarrer le serveur de développement

```powershell
# Dans le dossier du projet
npm run dev
```

6.2 Accéder à l'application

Ouvrez votre navigateur à l'adresse : http://localhost:3000 (ou http://localhost:5173)

Vous devriez voir tous les nœuds affichés :

· MPPT (chargeur solaire)
· Shunt (mesure)
· Batterie (stockage)
· Switch (interrupteur)
· ET112 (compteur)
· Météo (irradiance)
· Température (conditions)

---

7. Simulation de données temps réel (optionnel)

Pour tester le comportement dynamique, ajoutez ce code dans VisualisationComplete.jsx juste avant le return :

```jsx
// Simulation de variation de la production solaire
useEffect(() => {
  const interval = setInterval(() => {
    setNodes((nds) =>
      nds.map((node) => {
        if (node.type === 'mppt') {
          // Variation aléatoire de la production
          const variation = 0.8 + Math.random() * 0.4;
          const newPower = Math.round(1169 * variation);
          return {
            ...node,
            data: {
              ...node.data,
              totalPower: newPower,
              mppts: node.data.mppts.map((mppt, idx) => ({
                ...mppt,
                power: idx === 0 ? Math.round(777 * variation) : Math.round(423 * variation),
                current: idx === 0 ? +(1.9 * variation).toFixed(1) : +(4.3 * variation).toFixed(1)
              }))
            }
          };
        }
        return node;
      })
    );
  }, 3000);
  
  return () => clearInterval(interval);
}, []);
```

N'oubliez pas d'ajouter import { useEffect } from 'react'; en haut du fichier.

---

8. Dépannage - Erreurs fréquentes sous Windows

Erreur 1 : "node" n'est pas reconnu

Solution : Redémarrez votre terminal ou votre PC après installation de Node.js

Erreur 2 : "cannot find module" ou "module not found"

Solution : Supprimez node_modules et réinstallez :

```powershell
rm -r node_modules
rm package-lock.json
npm install
```

Erreur 3 : Erreur de permission (EPERM)

Solution : Lancez PowerShell en Administrateur puis réessayez

Erreur 4 : Port déjà utilisé

Solution : Changez le port dans vite.config.js :

```javascript
server: { port: 3001 }  // au lieu de 3000
```

Erreur 5 : Les styles CSS ne chargent pas

Solution : Vérifiez les imports dans chaque composant :

```jsx
import './meteoAnimations.css';  // Chemin correct
```

---

9. Package.json complet

Fichier : package.json (vérifiez qu'il ressemble à ceci)

```json
{
  "name": "reactflow-energie",
  "private": true,
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@xyflow/react": "^12.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.43",
    "@types/react-dom": "^18.2.17",
    "@vitejs/plugin-react": "^4.2.1",
    "vite": "^5.0.8"
  }
}
```

---

10. Récapitulatif des commandes Windows

Action Commande
Créer un dossier mkdir nom-dossier
Entrer dans un dossier cd nom-dossier
Créer un fichier vide type nul > fichier.jsx
Lancer le serveur npm run dev
Arrêter le serveur Ctrl + C
Installer une dépendance npm install nom-package
Nettoyer l'installation rm -r node_modules && npm install

---

11. Test rapide - Vérification que tout fonctionne

Après avoir tout installé et lancé npm run dev, vous devriez voir :

1. Une page avec plusieurs nœuds organisés horizontalement
2. Le nœud MPPT affichant 1169W avec deux sous-MPPT
3. La batterie avec son SOC et ses compteurs d'énergie
4. Le switch avec son toggle ON/OFF
5. Les nœuds météo avec irradiance et température
6. Les connexions entre les nœuds (edges)

Si vous voyez des erreurs dans la console (F12 dans le navigateur), vérifiez :

· Les imports dans chaque fichier
· Les chemins des fichiers CSS
· Que tous les fichiers sont bien créés

---

Fin du guide - Vous avez maintenant un serveur Node.js fonctionnel avec React Flow sur Windows, prêt à tester tous vos NodeTypes.
