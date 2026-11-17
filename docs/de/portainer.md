# Portainer mit IPv6

Verwende ein Docker-Standalone-Environment auf Linux/amd64. Der Docker-Host
benötigt funktionierenden öffentlichen IPv6-Zugang für automatische AAAA-Updates.

1. Öffne **Stacks → Add stack → Git repository**.
2. Verwende als Repository URL `https://github.com/itsh-neumeier/ddnsupd`.
3. Setze Repository reference auf `refs/tags/v0.1.0`.
4. Setze Compose path auf `docker-compose.portainer.yml`.
5. Hinterlege die folgenden Environment variables und wähle **Deploy the stack**.

| Variable | Wert |
| --- | --- |
| DDNSUPD_VERSION | `0.1.0` |
| DDNSUPD_EXTERNAL_URL | Die URL im Browser, etwa `http://192.168.10.20:8080` oder deine HTTPS-URL |
| DDNSUPD_PORT | `8080` oder dein freier Host-Port |
| POSTGRES_PASSWORD | Zufällige Hex-Zeichenfolge mit 48 Zeichen |
| DDNSUPD_SECRET_KEY | Zufällige Hex-Zeichenfolge mit 64 Zeichen |
| DDNSUPD_SETUP_TOKEN | Zufällige Hex-Zeichenfolge mit 64 Zeichen |
| DDNSUPD_ENGINE_TOKEN | Zufällige Hex-Zeichenfolge mit 64 Zeichen |

`python scripts/init-env.py` erzeugt die vier Geheimnisse in einer privaten
`.env`. Portainer kann sie über **Load variables from .env file** importieren.
Passe URL und Port für deinen Zielhost an. Bewahre diese Werte geschützt auf.

Der Portainer-Stack veröffentlicht den Webport standardmäßig auf allen
Host-Schnittstellen. Mit `DDNSUPD_BIND` kannst du eine bestimmte Adresse
vorgeben. Datenbank und Engine sind intern. Das Egress-Netz hat IPv6 aktiviert.
Prüfe die erste Anmeldung oder wähle die Wiederherstellung eines Backups.

Für tägliche Backups kannst du `docker-compose.backup.yml` als Additional path
angeben. Die nötigen absoluten Pfade befinden sich auf dem Docker-Zielhost.
Siehe [Backup und Wiederherstellung](installation.md#backup-und-wiederherstellung).

Für Aktualisierungen wähle die neue Release-Referenz und den zugehörigen
`DDNSUPD_VERSION`-Wert. Die Referenz `refs/heads/main` verfolgt den
Entwicklungsstand.
