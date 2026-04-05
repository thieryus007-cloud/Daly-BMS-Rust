Ce guide est structuré en 5 phases :

1. Prérequis et Installation (Ollama, Python, Git)
2. Configuration du modèle DeepSeek (Téléchargement et test)
3. Installation d'Aider (L'IA développeur)
4. Connexion à GitHub (Clonage, Branches, Commits)
5. Workflow complet (Itérations et amélioration du code)

---

Phase 1 : Prérequis et Installation des outils

Avant de toucher au code, il faut préparer la machine.

1. Vérification des prérequis matériels
DeepSeek V3 (671B paramètres) ne tourne pas sur un PC standard. Vous utiliserez donc DeepSeek V3.2 via Ollama.

· Modèle recommandé : deepseek-coder-v2 ou deepseek-r1:32b (selon votre RAM).
· RAM requise : 16 Go minimum (32+ Go recommandé pour les modèles 30B+).

2. Installer Ollama (Le moteur AI)

· Rendez-vous sur ollama.com.
· Téléchargez OllamaSetup.exe et installez-le.
· Vérification : Ouvrez un terminal (PowerShell) et tapez ollama. Si la commande n'est pas reconnue, redémarrez votre PC.

3. Installer Python et Git

· Python 3.10+ : Cochez "Add Python to PATH" pendant l'installation.
· Git for Windows : Téléchargez sur git-scm.com (installation par défaut).

---

Phase 2 : Téléchargement et test de DeepSeek via Ollama

Ollama sert de "backend" qui tourne en arrière-plan.

1. Démarrer le modèle DeepSeek
Ouvrez un terminal (PowerShell) et lancez :

```bash
ollama run deepseek-coder-v2
```

Note : Si vous voulez un modèle plus petit mais rapide : ollama run deepseek-r1:7b.

· Premier lancement : Ollama va télécharger le fichier du modèle (plusieurs Go). Attendez la fin du téléchargement.
· Test : Une fois le >>> affiché, tapez "Hello, write a python loop" pour vérifier que cela fonctionne.
· Sortie : Tapez /bye pour quitter, mais laissez le terminal ouvert. (Ollama reste en mémoire).

⚠️ Important : Pour qu'Aider puisse parler à DeepSeek, Ollama doit tourner en continu. Laissez ce terminal ouvert ou exécutez ollama serve dans un terminal dédié.

---

Phase 3 : Installation et configuration d'Aider

Aider est l'outil qui lit votre code, propose des modifications et génère les commits Git.

1. Installation via pip
Dans un nouveau terminal (Admin si besoin), tapez :

```bash
pip install aider-chat
```

2. Configuration de la connexion à Ollama
Aider doit savoir où trouver DeepSeek. Configurez-le avec la commande suivante :

```bash
aider --model ollama/deepseek-coder-v2 --api-base http://localhost:11434
```

Si vous utilisez un autre modèle : Remplacez deepseek-coder-v2 par le nom exact que vous avez utilisé dans ollama run.

Astuce : Pour ne pas retaper cette longue commande à chaque fois, créez une variable d'environnement ou un fichier .aider.model.yml.

---

Phase 4 : Connexion à GitHub (Clonage et accès)

Vous allez maintenant lier cela à votre dépôt pour "étudier l'application sur le reproche Git" (comprendre : le dépôt Git).

1. Cloner le dépôt
Sur GitHub, copiez l'URL de votre dépôt (HTTPS ou SSH). Dans votre terminal, allez dans le dossier où vous voulez stocker le code :

```bash
git clone https://github.com/VOTRE_NOM/le-repo-git.git
cd le-repo-git
```

2. Lancer Aider sur ce dépôt
Maintenant, lancez Aider en lui donnant le contexte du code :

```bash
aider --model ollama/deepseek-coder-v2 --api-base http://localhost:11434
```

(Aider analyse automatiquement les fichiers du dossier)

3. Vérification de l'état Git
Dans Aider, tapez :

```text
/run git status
```

Cela vous montre l'état actuel du code par rapport à GitHub.

---

Phase 5 : Workflow complet (Étude, Itérations et Commits)

Voici le process standard que vous allez répéter pour améliorer l'application.

Étape A : L'étude du code (Compréhension)

Vous venez d'arriver sur le projet, vous ne comprenez pas un fichier.

· Prompt Aider :
  "Analyse le fichier main.py (ou app.js). Explique moi l'architecture de cette application et identifie les bugs potentiels."

Aider lira le fichier et vous fera un rapport.

Étape B : Proposition de modification (Feature request)

Vous voulez ajouter une fonction.

· Prompt Aider :
  "Ajoute une fonction de log qui enregistre la date et l'heure dans un fichier logs.txt à chaque fois que l'utilisateur clique sur le bouton 'Sauvegarder'."

Aider va générer le diff (les modifications exactes du code) et vous les montrer.

Étape C : Validation et Commit Git

Si la modification vous convient :

· Prompt Aider :
  "Valide ces changements. Fais un commit avec le message 'feat: add logging for save button'."

Aider exécute automatiquement git add et git commit -m "..." pour vous.

Étape D : Itération (Correction d'erreur)

Si le code généré plante ou ne fait pas exactement ce que vous voulez :

· Ne recopiez pas tout le code. Dites simplement :
  "Non, ta fonction de log écrit en UTF-8, mais le système attend de l'ASCII. Corrige ça."

Aider comprend l'erreur, modifie le code existant (sans toucher au reste) et propose un nouveau commit.

Étape E : Push vers GitHub

Une fois que plusieurs commits sont faits localement, poussez-les vers le serveur GitHub.

· Prompt Aider (ou terminal classique) :
  ```text
  /run git push origin main
  ```

Résumé des commandes utiles dans Aider

Commande Fonction
/add fichier.py Ajoute un fichier au contexte (pour que l'IA le lise)
/drop Enlève un fichier du contexte
/commit Crée un commit des changements en cours
/diff Montre les modifications en attente (en couleur)
/run <cmd> Exécute une commande shell (ex: git log)
/help Affiche l'aide

Dépannage rapide

· Erreur "Connection refused" : Ollama n'est pas lancé. Retournez au terminal et tapez ollama serve.
· Mémoire insuffisante : Si votre PC rame, stoppez DeepSeek (Ctrl + C) et relancez un modèle plus petit (deepseek-r1:7b).
· Aider ne voit pas Git : Assurez-vous d'avoir lancé Aider depuis le dossier qui contient le dossier .git.
