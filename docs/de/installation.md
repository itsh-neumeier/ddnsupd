# Installation und erste Schritte

Dies ist ein Entwicklungsstand. Bis zur freigegebenen Veröffentlichung wird
der Stack aus den Quellen gebaut. Zielumgebungen sind Docker Engine unter
Linux und Docker Desktop. Für den Test-Override ist Compose 2.24.4+ nötig.

Die Startbefehle stehen in der [README](../../README.md). `.env` enthält private
Laufzeitwerte. Unter Windows zusätzlich die NTFS-Zugriffsrechte auf dein Konto
beschränken. Im Named Volume `postgres-data` liegen die Anwendungsdaten.
API und Engine laufen als UID/GID 10001 mit schreibgeschütztem Root-Dateisystem,
temporärem Speicher und ohne Linux-Capabilities. PostgreSQL nutzt seinen eigenen
Benutzer. Nur der Webport ist standardmäßig an 127.0.0.1:8080 veröffentlicht.

## Einrichtung

Öffne nach dem Start [http://localhost:8080](http://localhost:8080). Verwende
`DDNSUPD_SETUP_TOKEN` aus deiner privaten `.env`, um den ersten Administrator
anzulegen. Das Passwort braucht mindestens zwölf Zeichen. Der Benutzername
darf Kleinbuchstaben, Ziffern sowie `.`, `_`, `-` und `@` enthalten. Nach der
Ersteinrichtung ist dieser Zugang zur Einrichtung geschlossen.

Die Umgebungserzeugung überschreibt keine vorhandene Datei. Ersetze niemals
den Anwendungsschlüssel bei einer bestehenden Datenbank: Ohne den passenden
`DDNSUPD_SECRET_KEY` lassen sich gespeicherte Providertokens nicht entschlüsseln.

## Reverse Proxy / Zoraxy

Setze `DDNSUPD_EXTERNAL_URL=https://dns.deine-domain.example`. Leite diese
Origin in Zoraxy auf Port 8080 weiter und erhalte Host- und Origin-Header.
Bei einer HTTPS-Basisadresse verwendet DDNSupd sichere Session-Cookies.
Ein Proxy auf dem gleichen Host kann den Loopback-Port verwenden. Bei einem
anderen Proxyhost sind Bind-Adresse und Firewall gezielt anzupassen.
Veröffentliche den Backendport nicht direkt im Internet.

Die Anwendung vertraut keinen Forwarded-Identitätsheadern. Eine Anmeldung über
Proxyheader ist nicht möglich. WebSockets sind nicht erforderlich. Der Betrieb
unter einem URL-Unterpfad wird in diesem Stand nicht unterstützt. Die konfigurierte
Origin muss inklusive Port mit der Browseradresse übereinstimmen.

## Provider und Records

Verbinde unter Provider ein Hetzner-Cloud- oder Cloudflare-Konto. Verwende
beschränkte Tokens: bei Hetzner ein möglichst separates DNS-Projekt mit
Lese-/Schreibzugriff, bei Cloudflare Zone Read und DNS Edit nur für die benötigte
Zone. Die alten Hetzner-DNS-Tokens funktionieren nicht mit der Cloud-API.
Gespeicherte Tokens werden nicht angezeigt. Unter Bearbeiten ersetzt ein neues
Token das bisherige; ein leeres Feld behält es. Ein Austausch hebt eine Sperre
wegen ungültiger Zugangsdaten auf. Provider mit zugeordneten Records können
nicht gelöscht werden.

Lege anschließend eine Zuordnung für einen A- oder AAAA-Record an: Provider-Zonen-ID,
Zonenname, relativer Name (`home`, `@` oder `*`) und bei Cloudflare zusätzlich
die ID des bestehenden Records.
Neue Zuordnungen bleiben bis zur bewussten Aktivierung pausiert. Bei mehreren
RRSet-Werten ist die bisher verwaltete IP explizit anzugeben. Uneindeutige
Zuordnungen werden abgelehnt. Externe Writer auf demselben RRSet müssen
koordiniert werden. Das Löschen einer Zuordnung lässt den DNS-Record bestehen.

Für eine aktive Hetzner-Cloud-Zuordnung kannst du im Record-Detail **Fehlenden
DNS-Record anlegen** auswählen und **Bei Hetzner anlegen** bestätigen. Der
Auftrag verwendet die konfigurierte IP-Quelle, berücksichtigt Wartezeiten und
legt nur einen fehlenden A- oder AAAA-Record in einer primären Zone an. Ein
vorhandener Record desselben Typs bleibt bei diesem Auftrag unverändert.
Das Ergebnis steht unter **Aktivität**.

Eine leere manuelle IP aktiviert die HTTPS-Ermittlung mit einer Ersatzquelle
für die jeweilige Adressfamilie. Dual-Stack verwendet zwei Records. Private,
CGNAT- und reservierte Adressen werden abgewiesen. Erreichbarkeit und
Portweiterleitungen kann DDNS nicht herstellen. IPv6-Suffixe sind noch geplant;
eine ermittelte temporäre IPv6-Adresse ist keine zugesicherte stabile Hostadresse.

Das Compose-Netz für ausgehende Verbindungen vergibt IPv4- und IPv6-Adressen.
Für automatische AAAA-Updates muss der Container AAAA-Antworten auflösen und
einen IPv6-Prüfdienst über HTTPS erreichen können. Fehlt dem Host öffentlicher
IPv6-Zugang, kannst du die stabile öffentliche IPv6 des Zielsystems als
manuelle IP hinterlegen.

## Bestehende Records erkennen und importieren

Wähle unter Provider ein Konto, dann **Zugriff prüfen / Zonen laden** und eine
Zone. Lade deren DNS-Records und blättere bei Bedarf weiter. Der Test bestätigt
nur Lesezugriff. Eine öffentliche A-/AAAA-Adresse lässt sich als pausierte
Zuordnung übernehmen. Entfernte DNS-Records werden dabei nicht angelegt.

Administratoren können eine ddns-updater-`config.json` für das ausgewählte Konto
und die Zone hochladen. Prüfe die Vorschau, behebe Konflikte und wähle die Zeilen
aus. Bei `ipv4 or ipv6` muss A oder AAAA ausdrücklich gewählt werden. Zugangsdaten
aus der Datei werden ignoriert; verwendet wird das gespeicherte Konto.
TTL und Proxy-Einstellungen beim Provider bleiben erhalten. Bestätige nach der
Prüfung den Import. Alle Zuordnungen bleiben bis zur bewussten Aktivierung unter
DNS-Records pausiert; der Import löst keine DNS-Schreibzugriffe aus.

Vorschauen gelten zehn Minuten, sind einmalig und verfallen beim Wechsel der
Zugangsdaten. Grenzen: 60-KiB-Datei in der UI, 200 aufgelöste Einträge, 20
Providerseiten/2000 Adresswerte, 30 Sekunden Ermittlung und 30 Providerseiten
pro Minute. Überschreitungen brechen ausdrücklich ab. Kontoübergreifender Import,
Historienimport und IPv6-Suffixe sind weiterhin geplant.

## Rollen und Betrieb

Unter Verwaltung kann ein Administrator Benutzer, Rollen und Ressourcenzugriffe
anlegen. Die Rolle erlaubt Aktionen; Eigentum oder Ressourcenzuweisung begrenzen
deren Umfang. Viewer dürfen auch freigegebene Records nicht verändern.
Eine Recordfreigabe gibt keine Zugangsdaten frei. Die Verwaltung von Konten,
Rollen und Credentials bleibt in diesem Stand eingebauten Administratoren
vorbehalten. Benutzer deaktivieren/löschen, Rollen bearbeiten und Passwort-Reset
sind geplant. Der eigene Passwortwechsel benötigt das aktuelle Passwort und
beendet alle bestehenden Sitzungen dieses Benutzers.

Aktivität zeigt Prüfungen, Updates und Fehler. Jetzt prüfen reiht einen Auftrag
ein und umgeht keine Cooldowns. Die Origin-Prüfung bestätigt keine weltweite
DNS-Propagation. Authentifizierungsfehler sperren den Provider; Rate-Limits
beachten Retry-After. `/healthz` prüft den Prozess, `/readyz` die Datenbank.
`docker compose ps` zeigt den Zustand der Dienste. Prüfe bei ausbleibenden
Updates Aktivierung, Sperren, nächsten Prüftermin und die IP-Quelle.

## Backup und Wiederherstellung

Unter **Verwaltung → Datensicherung** kannst du als Administrator ein
verschlüsseltes Backup herunterladen. Bewahre Datei und Passphrase getrennt auf.
Das Archiv enthält Benutzer, Rollen, Freigaben, DNS-Zuordnungen, Provider samt
Zugangsdaten, Aktivität und Audit-Historie. Die Provider-Zugangsdaten werden
beim Restore mit dem Schlüssel der Zielinstanz neu verschlüsselt.

Wähle auf einer frischen Instanz **Backup wiederherstellen** in der
Ersteinrichtung und gib das Setup-Token der Zielinstanz, die Datei und ihre
Passphrase ein. Auf einer eingerichteten Instanz ersetzt der Restore unter
**Verwaltung** die Daten nach ausdrücklicher Bestätigung. Melde dich danach
mit einem Benutzer aus dem Backup an und aktiviere die pausierten DNS-Zuordnungen
erst nach Prüfung der IP-Quellen. Alte Sitzungen und ausstehende Aufträge werden
verworfen. Ein beschädigtes Archiv oder eine falsche Passphrase bricht ab.

Das Archiv unterstützt bis zu 16 MiB und benötigt derzeit dieselbe
DDNSupd-Datenbankschemaversion. Richte Ports, URL, Datenbank- und Engine-Zugang
auf der Zielinstanz über deren eigene private Laufzeitkonfiguration ein. Für
größere Installationen sichere einen PostgreSQL-Dump und den dazugehörigen
Anwendungsschlüssel getrennt.

Für tägliche Sicherungen ergänze dein Compose-Kommando um
`-f docker-compose.backup.yml`. Setze `DDNSUPD_BACKUP_HOST_DIR` auf ein
privates Verzeichnis und `DDNSUPD_BACKUP_HOST_PASSPHRASE_FILE` auf eine
geschützte Datei mit der Passphrase. Im Container muss UID 10001 das Verzeichnis
beschreiben und die Datei lesen können. DDNSupd prüft stündlich, ob für den
aktuellen UTC-Tag bereits ein Backup vorliegt, und verwendet sieben rotierende
Dateien. Kopiere wichtige Archive auf ein getrenntes Speichermedium.
