1. cd ~/Daly-BMS-Rust
2. git fetch origin main
3. git checkout main                 # S'assurer d'être sur la branche main.
4. git reset --hard origin/main      # Écraser toute modification locale avec l'état distant de main.
5. make build-arm

6. sudo systemctl stop daly-bms
7. sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/
8. . sudo cp Config.toml /etc/daly-bms/config.toml
9. sudo systemctl start daly-bms

## Vérifier

· Accédez à http://192.168.1.141:8080/dashboard/visualization

# Explication des différences

· git checkout main assure que vous travaillez sur la bonne branche.
· git reset --hard origin/main force l'alignement avec le dépôt distant, écrasant tout fichier local modifié (ce qui est souhaité car vous voulez exactement ce qui est sur GitHub).

Remarque : si le Makefile utilise une cible différente pour la compilation ARM (ex. make release-arm), ajustez en conséquence. 
La commande make build-arm est celle que vous aviez dans l'exemple.

# Pi5

# 1. Sur Pi5, aller dans le répertoire du projet
1. cd ~/Daly-BMS-Rust

# 2. Récupérer les derniers changements depuis GitHub
2. git fetch origin claude/update-visualization-html-bTjgf
3. git checkout claude/update-visualization-html-bTjgf
4. git reset --hard origin/claude/update-visualization-html-bTjgf

# 3. Compiler pour aarch64 (Pi5)

5. make build-arm

# 4. Arrêter le service BMS en cours
6. sudo systemctl stop daly-bms

# 5. Remplacer le binaire
7. sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/

# 6. Copier la configuration mise à jour (si nécessaire)
8. sudo cp Config.toml /etc/daly-bms/config.toml

# 7. Redémarrer le service
9. sudo systemctl start daly-bms

# 8. Vérifier que le service est actif
10. systemctl status daly-bms
11. journalctl -u daly-bms -f

# 9. Accéder au dashboard
-- http://192.168.1.141:8080/dashboard/visualization
