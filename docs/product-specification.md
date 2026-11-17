# DDNSupd – Master Development Prompt
- **Version:** 1.0 – Initial Architecture & Product Specification
- **Produkt:** DDNSupd
- **Beschreibung:** Self-hosted Dynamic DNS Management Platform
- **Referenzprojekt UI/Architektur:** https://github.com/itsh-neumeier/talkops
- **Referenzprojekt DDNS Engine:** https://github.com/qdm12/ddns-updater/tree/master
- **Vorgeschlagener Repositoryname:** `itsh-neumeier/DDNSupd`
**Status:** Konzept / Interviewphase

---

## 1. Deine Rolle

Du arbeitest als autonomer Senior Software Architect, Full-Stack Developer, DevOps Engineer und QA Engineer an der Entwicklung von DDNSupd.

Deine Aufgabe ist die Entwicklung einer modernen, sicheren, vollständig containerisierten Dynamic-DNS-Management-Plattform mit webbasiertem Administrationsportal.

Das Produkt soll eigenständig entwickelt, langfristig wartbar und über GitHub versioniert werden.

Verwende bewährte Softwarekomponenten und vermeide unnötige Eigenentwicklungen.

Arbeite inkrementell, testgetrieben und mit nachvollziehbaren Entwicklungsschritten.

**Grundprinzip:** Nicht nur eine grafische Oberfläche für einen bestehenden DDNS-Updater bauen, sondern eine vollständige Management-Plattform entwickeln.

## 2. Produktvision

DDNSupd ist eine zentrale Verwaltungsoberfläche für Dynamic DNS.

Die Software soll DNS-Einträge verschiedener Anbieter automatisch überwachen und bei Änderungen öffentlicher IPv4- oder IPv6-Adressen aktualisieren.

Der gesamte Betrieb erfolgt über Docker und ein intuitives Webportal.

Wesentliche Produktmerkmale:

- Unterstützung mehrerer DNS-Anbieter und Zugangsdaten.
- Verwaltung beliebig vieler Domains und DNS-Records im Rahmen der Systemressourcen.
- Automatische Aktualisierung von A- und AAAA-Einträgen.
- Benutzerverwaltung mit konfigurierbaren Rollen und Berechtigungen.
- Transparente Überwachung aller Aktualisierungsvorgänge.
- Zentrale Verwaltung von DNS-Anbieterzugängen.
- Fehlerprotokollierung und Benachrichtigungen.
- Sichere Speicherung von Zugangsdaten.
- Import und Export bestehender DDNS-Konfigurationen.
- Vollständig selbst gehosteter Betrieb.
- Responsive Weboberfläche für Desktop, Tablet und Smartphone.
- Erweiterbarkeit um externe Agenten und weitere Provider.
- Dokumentierte Schnittstellen für Automatisierungen.

Die Anwendung muss nach der Ersteinrichtung vollständig über das Webportal bedienbar sein.

Manuelle Änderungen an JSON-Dateien dürfen für den regulären Betrieb nicht erforderlich sein.

---

## 3. Architektur- und Designreferenz TalkOps

Analysiere das aktuelle Repository:

https://github.com/itsh-neumeier/talkops

Lies insbesondere:

- `README.md`
- `AGENTS.md`
- `CLAUDE.md`
- `docs/architecture.md`
- `docs/ui-guidelines.md`
- `docs/adr/`
- `docs/release-workflow.md`
- `web/src/app.css`
- `web/src/routes/`
- `web/src/lib/`

Übernimm die nachvollziehbaren architektonischen und organisatorischen Prinzipien von TalkOps.

### 3.1 Designsystem

DDNSupd soll visuell zur TalkOps-Produktfamilie gehören.

Anforderungen:

- Vergleichbare Login-Oberfläche.
- Gleichartiges Navigations- und Seitenlayout.
- Einheitliche Topbar und Seitenleiste.
- Dark Mode und Light Mode.
- Teal als primäre Akzentfarbe.
- Slate-basierte Hintergrund- und Kartenfarben.
- Vergleichbare Buttons, Eingabefelder, Tabellen und Dialoge.
- Responsive Datenansichten.
- Konsistente Status-Badges.
- Einheitliches Icon-System.
- Moderne Ladeanimationen und Skeleton-Zustände.
- Seitliche Detailpanels für Datensatzbearbeitung.
- Vergleichbare Footer-Struktur.
- Deutsche und englische Oberfläche.

Die konkrete UI orientiert sich an den aktuellen TalkOps-Designrichtlinien.

Keine unnötigen alternativen UI-Frameworks einführen.

Das Design wird eigenständig für DDNSupd implementiert. Bei direkter Codeübernahme sind die jeweiligen Open-Source-Lizenzbedingungen zu berücksichtigen.

### 3.2 Technologie

Bevorzugter Technologie-Stack:

Frontend:
- SvelteKit
- Svelte 5
- TypeScript
- Tailwind CSS
- Responsive SPA
- API-basierte Kommunikation

Backend:
- Rust
- Axum
- Tokio
- SQLx
- PostgreSQL
- REST API mit OpenAPI-Spezifikation

DDNS-Engine:
- Technologische Entscheidung nach Analyse des Upstream-Projekts.
- Bevorzugt Wiederverwendung oder gezielte Adaption der bewährten Go-Provider-Implementierungen.
- Saubere Trennung zwischen Provider-Engine und Management-API.
- Kein stillschweigender Funktionsverlust gegenüber der Referenz.

Infrastruktur:
- Docker Compose
- Multi-Stage Docker Builds
- Persistente Docker Volumes
- GitHub Actions
- GitHub Container Registry
- Reverse-Proxy-Unterstützung

Jede Abweichung von diesem Stack muss fachlich begründet und in einem Architecture Decision Record dokumentiert werden.

---

## 4. DDNS-Referenzprojekt

Analysiere das aktuelle Repository:

https://github.com/qdm12/ddns-updater

Untersuche insbesondere:

- Provider-Implementierungen
- Provider-Konfigurationsschema
- Scheduling
- Öffentliche IP-Ermittlung
- DNS-Auflösung
- Record-Vergleich
- Update-Verfahren
- Cooldown und Rate Limiting
- IPv4- und IPv6-Verarbeitung
- Persistence
- Healthchecks
- Benachrichtigungen
- Bestehende API-Endpunkte
- Fehlerbehandlung
- Docker-Betrieb
- Konfigurationsimport

### 4.1 Funktionsparität

Erstelle vor der Implementierung eine Funktionsmatrix aller relevanten Upstream-Funktionen.

Jede Funktion erhält einen Status:

- Unterstützt
- In Entwicklung
- Geplant
- Nicht relevant
- Technisch eingeschränkt

Implementiere keine Funktion nur dem Namen nach.

Jede als unterstützt ausgewiesene Funktion muss durch geeignete Tests nachgewiesen sein.

Dokumentiere Unterschiede gegenüber dem Referenzprojekt.

### 4.2 Provider-Architektur

Implementiere eine modulare Provider-Registry.

Jeder Provider verfügt über eine definierte Schnittstelle mit:

- Eindeutiger Provider-ID
- Anzeigename
- Authentifizierungsschema
- Unterstützten Recordtypen
- Unterstützter IP-Version
- Berechtigungsanforderungen
- Zone- und Record-Funktionen
- Optionalen TTL-Einstellungen
- Provider-spezifischen Optionen
- Rate-Limit-Vorgaben
- Fehlerklassifizierung
- Dokumentationsreferenz

Provider dürfen keine Geschäftslogik der Weboberfläche enthalten.

Implementiere zunächst eine überprüfbare Kernmenge von Providern. Erweitere diese systematisch auf die vollständige, zum festgehaltenen Upstream-Stand dokumentierte Provider-Liste.

**Priorisierte Anbieter:**

1. Hetzner Cloud DNS
2. Hetzner Legacy DNS, sofern die API noch verfügbar ist
3. Cloudflare
4. IONOS
5. INWX
6. Netcup
7. Strato
8. deSEC
9. DuckDNS
10. DynDNS-kompatible Anbieter

Die Priorisierung ist keine Beschränkung der späteren Provider-Unterstützung.

Die Anwendung muss unterschiedliche APIs, Zugangsdatenformate und Berechtigungsmodelle berücksichtigen.

Ein Provider-Supportstatus darf nur dann als verifiziert gelten, wenn die entsprechende Integration getestet wurde.

---

## 5. DDNS-Engine

Entwickle einen stabilen Hintergrunddienst zur regelmäßigen Aktualisierung der DNS-Einträge.

### 5.1 IP-Adressermittlung

Unterstütze:

- Öffentliche IPv4-Adressen.
- Öffentliche IPv6-Adressen.
- IPv4-only.
- IPv6-only.
- Dual-Stack.
- Getrennte IPv4- und IPv6-Quellen.
- IP-Ermittlung per HTTPS.
- IP-Ermittlung per DNS.
- Konfigurierbare externe IP-Dienste.
- Fallback-Dienste.
- Timeouts.
- Mehrere IP-Quellen mit Plausibilitätsprüfung.
- Manuelle IP-Vorgaben für Sonderfälle.

Unterscheide zwischen öffentlich erreichbaren Adressen und privaten Adressen.

Berücksichtige CGNAT, DS-Lite und IPv6-Präfixwechsel.

Bei IPv6 muss die Verwendung eines stabilen Interface-Identifier beziehungsweise eines konfigurierten Suffixes möglich sein.

### 5.2 Aktualisierungslogik

Für jeden aktiven Datensatz:

1. Öffentliche IP ermitteln.
2. Letzten bekannten Zustand abrufen.
3. DNS-Zustand beziehungsweise tatsächlichen Provider-Record ermitteln.
4. Veränderungen bewerten.
5. Notwendigkeit eines Updates bestimmen.
6. Provider-spezifische Update-Anfrage durchführen.
7. Ergebnis überprüfen.
8. Status und Historie speichern.
9. Benachrichtigung bei relevanten Ereignissen auslösen.

Verhindere unnötige Updates.

Berücksichtige DNS-TTL, Propagation, Rate Limits und Provider-Cooldowns.

Bei Cloudflare-Proxied-Records muss die tatsächlich hinterlegte Origin-IP berücksichtigt werden. Die öffentlich aufgelöste Cloudflare-Proxy-IP ist dafür nicht ausreichend.

### 5.3 Zeitsteuerung

Unterstütze:

- Globales Standardintervall.
- Provider-spezifische Intervalle.
- Record-spezifische Intervalle.
- Manuelle Aktualisierung.
- Startprüfung beim Anwendungsstart.
- Konfigurierbare Retry-Strategie.
- Exponential Backoff.
- Rate-Limit-Behandlung.
- Schutz gegen parallele Konfliktupdates.
- Persistente Auftragszustände.

Manuelle Aktualisierungen müssen gegen unzulässige oder unbeabsichtigt wiederholte Provider-Anfragen abgesichert werden.

### 5.4 Sicherheitsmechanismen

- Keine unerwarteten DNS-Änderungen.
- Bei Fehlern der IP-Ermittlung keinen ungültigen Datensatz schreiben.
- Kein Überschreiben von Records außerhalb des verwalteten Umfangs.
- Schreiboperationen müssen idempotent ausgelegt sein.
- Fehlerhafte Credentials dürfen keine endlosen Anfragen auslösen.
- Provider-Zugriffe benötigen Least-Privilege-Berechtigungen.
- Kompatible DNS-Records müssen bei Mehrfachupdates erhalten bleiben.

---

## 6. Webportal

Implementiere ein modernes, vollständiges Administrationsportal.

### 6.1 Dashboard

Das Dashboard zeigt:

- Anzahl konfigurierter Domains.
- Anzahl aktiver DNS-Records.
- Anzahl gesunder Einträge.
- Fehlerhafte Einträge.
- Letzte erfolgreiche Aktualisierungen.
- Aktuelle öffentliche IPv4-Adresse.
- Aktuelle öffentliche IPv6-Adresse.
- Zuletzt erkannte IP-Änderung.
- Providerstatus.
- Anstehende Prüfungen.
- Warnungen.

Zusätzlich:

- Update-Aktivitätsdiagramm.
- IP-Änderungsverlauf.
- Erfolgs- und Fehlerraten.
- Übersicht der Provider.
- Letzte Ereignisse.

Alle Daten sollen möglichst ohne vollständiges Neuladen aktualisiert werden.

### 6.2 DNS-Records

Zentrale Datensatzverwaltung mit:

- Domainname
- Recordname beziehungsweise FQDN
- Provider
- Recordtyp
- Aktuelle öffentliche IP
- Letzter verwalteter DNS-Wert
- Tatsächlicher DNS-/Provider-Wert
- Letzter Check
- Letztes Update
- Nächste Prüfung
- Aktualisierungsintervall
- TTL
- Status
- Aktiv/Inaktiv
- Verantwortlicher Benutzer
- Zugeordneter Standort

Funktionen:

- Datensatz hinzufügen
- Bearbeiten
- Löschen
- Aktivieren/Deaktivieren
- Jetzt aktualisieren
- Verbindung prüfen
- Historie öffnen
- Mehrfachauswahl
- Massenaktionen
- Suche
- Filter
- Sortierung
- Export

Datensätze sollen analog zu TalkOps in Tabellen mit seitlichem Detailpanel verwaltet werden.

### 6.3 Providerverwaltung

Administrationsseite für DNS-Anbieter:

- Provider hinzufügen.
- Credentials hinterlegen.
- Verbindung testen.
- API-Berechtigungen überprüfen, soweit technisch möglich.
- Verfügbare Zonen abfragen.
- DNS-Records importieren.
- Providerkonfiguration bearbeiten.
- Credentials aktualisieren.
- Anbieter deaktivieren.
- Letzte Providerfehler anzeigen.

Zugangsdaten dürfen nach dem Speichern nicht mehr im Klartext ausgegeben werden.

### 6.4 Domains und Zonen

Implementiere eine strukturierte Domainverwaltung.

Beispiel:

`example.org`
- `home.example.org`
- `vpn.example.org`
- `nas.example.org`
- `*.example.org`

Die Oberfläche zeigt Provider, DNS-Zone, verwaltete Records und deren Status.

Wildcard-Records müssen entsprechend den Fähigkeiten des Providers unterstützt werden.

---

## 7. Benutzer- und Rollenverwaltung

Orientiere dich am Berechtigungssystem von TalkOps.

### 7.1 Authentifizierung

- Lokale Benutzerkonten.
- Sichere Passwort-Hashes mit Argon2id.
- Sitzungsverwaltung.
- Passwortwechsel.
- Passwortzurücksetzung.
- TOTP-Zwei-Faktor-Authentifizierung.
- Brute-Force-Schutz.
- Anmeldung über OIDC als Erweiterung.
- LDAP/Active Directory als optionale Erweiterung.

### 7.2 Rollen

Standardrollen:

**Administrator:** Vollständige Systemverwaltung einschließlich Provider, Benutzer, Einstellungen, Backup und Restore.

**Operator:** Verwaltung freigegebener Provider und DNS-Datensätze.

**Benutzer:** Verwaltung ausdrücklich zugeordneter DNS-Records.

**Viewer:** Lesender Zugriff auf freigegebene Ressourcen.

Zusätzlich muss ein Administrator eigene Rollen mit granularen Berechtigungen anlegen können.

### 7.3 Berechtigungssystem

Definiere eigene Berechtigungen für:

- Dashboard lesen
- DNS-Records lesen
- DNS-Records erstellen
- DNS-Records bearbeiten
- DNS-Records löschen
- DNS-Updates auslösen
- Provider lesen
- Provider verwalten
- Credentials verwalten
- Benutzer verwalten
- Rollen verwalten
- Historie lesen
- Audit-Logs lesen
- Systemeinstellungen verwalten
- Backup durchführen
- Restore durchführen
- API-Token verwalten

Alle Berechtigungen müssen serverseitig überprüft werden.

Eine ausgeblendete Schaltfläche ist keine ausreichende Zugriffskontrolle.

Die Zuweisung von Ressourcen zu Benutzern muss sauber von den globalen Rollen getrennt sein.

---

## 8. Monitoring und Historie

Speichere einen nachvollziehbaren Verlauf aller relevanten DDNS-Vorgänge.

### 8.1 Update-Historie

Jeder Vorgang dokumentiert:

- Zeitpunkt
- Provider
- DNS-Record
- Vorherige IP-Adresse
- Neue IP-Adresse
- Auslöser
- Ergebnis
- Verarbeitungsdauer
- Fehlerklasse
- Fehlerbeschreibung ohne Geheimnisse

Unterscheide zwischen:

- Automatischer Prüfung ohne Änderung
- Erfolgreichem Update
- Manuellem Update
- Fehlgeschlagenem Update
- Nicht erreichbarem Provider
- Nicht auflösbarem DNS-Record
- Ungültigen Zugangsdaten
- Rate Limit
- Übersprungener Ausführung

### 8.2 Monitoring

Implementiere:

- Liveness-Endpoint.
- Readiness-Endpoint.
- Konfigurierbare Healthchecks.
- Status pro Record.
- Status pro Provider.
- Metriken für Prometheus.
- Strukturierte Logs.
- Fehlerzähler.
- Verlauf für IPv4 und IPv6.

Zeige eine erfolgreiche API-Antwort nicht automatisch als vollständig erfolgte DNS-Propagation an.

---

## 9. Benachrichtigungen

Erstelle ein zentrales Benachrichtigungssystem.

Unterstützte Ereignisse:

- Öffentliche IP geändert.
- DNS-Eintrag erfolgreich aktualisiert.
- DNS-Update fehlgeschlagen.
- DNS-Provider nicht erreichbar.
- Zugangsdaten ungültig.
- Mehrfach fehlgeschlagene Aktualisierungen.
- Anhaltende Abweichung zwischen Soll- und Ist-Zustand.
- Systemfehler.

Geplante Kanäle:

- SMTP/E-Mail.
- Webhooks.
- Shoutrrr-kompatible Dienste.
- Optional ntfy, Telegram oder Gotify.

Jeder Benutzer kann abhängig von seinen Berechtigungen eigene Benachrichtigungsregeln konfigurieren.

Implementiere Deduplizierung und Eskalation, um Benachrichtigungsfluten zu verhindern.

---

## 10. API und Automatisierung

Implementiere eine versionierte REST API unter `/api/v1/`.

Funktionsbereiche:

- Authentifizierung
- Benutzer
- Rollen
- Provider
- Domains
- DNS-Records
- Updates
- Status
- Historie
- Ereignisse
- Monitoring
- Systemeinstellungen

Zusätzliche Anforderungen:

- OpenAPI-Dokumentation.
- API-Token mit Scopes.
- Sichere Tokenrotation.
- Pagination.
- Filter.
- Einheitliche Fehlermodelle.
- Auditierung mutierender Operationen.
- Rate Limiting.
- Webhooks für Ereignisse.

Optional sollen Router, NAS oder externe Dienste ihre aktuelle öffentliche IP an DDNSupd übermitteln können.

Dies muss über eine authentifizierte, auf einzelne Ressourcen beschränkte Schnittstelle erfolgen.

---

## 11. Docker-Architektur

DDNSupd wird vollständig über Docker Compose betrieben.

Bevorzugte Services:

`ddnsupd`
- Rust Backend
- REST API
- Statisches Webfrontend
- Authentifizierung
- Scheduler/Control Plane

`ddnsupd-worker`
- Provider-Engine
- Öffentliche IP-Ermittlung
- DNS-Updates
- Hintergrundaufträge

`postgres`
- Benutzer
- Provider
- DNS-Records
- Audit-Log
- Update-Historie
- Aufträge
- Konfiguration

Der Worker darf in die Hauptanwendung integriert werden, wenn dies die Wartbarkeit verbessert und die Architektur sauber bleibt.

### 11.1 Compose-Anforderungen

- Persistente Named Volumes.
- Healthchecks.
- Restart Policies.
- Konfiguration über `.env`.
- Beispieldatei `.env.example`.
- Keine produktiven Secrets im Git-Repository.
- Unprivilegierte Container.
- Minimale Linux-Capabilities.
- Möglichst Read-only Root Filesystem.
- Dokumentierte Portbelegung.
- Unterstützte Proxy-Konfiguration.
- Explizite interne Netzwerke.
- ARM64 und AMD64 nach Möglichkeit.
- Unterstützung für Docker Engine auf Linux.
- Testbarkeit unter Docker Desktop.

Produktive Compose-Dateien dürfen keine zwingende lokale Build-Phase benötigen.

### 11.2 Reverse Proxy

Unterstütze HTTPS hinter vorhandenen Reverse Proxies, insbesondere Zoraxy.

Berücksichtige:

- Forwarded Headers aus vertrauenswürdigen Proxy-Netzen.
- Sichere Cookies.
- WebSocket-Unterstützung, falls benötigt.
- Konfigurierbare externe Basis-URL.
- Keine unsichere Authentifizierungsumgehung.
- TLS-Termination durch Reverse Proxy.

---

## 12. Backup und Recovery

Entwickle ein Backupkonzept ähnlich TalkOps.

Sichere:

- Datenbank.
- Anwendungskonfiguration.
- Providerkonfigurationen.
- Verschlüsselte Credentials.
- Benutzer.
- Rollen.
- Audit- und Update-Historie entsprechend Aufbewahrungsregeln.
- Wiederherstellungsrelevantes Schlüsselmaterial.

Unterstütze:

- Manuelles Backup.
- Geplante Backups.
- Verschlüsselte Backup-Dateien.
- Download aus dem Webportal.
- Restore-Assistent.
- Versionskompatibilitätsprüfung.
- Backup-Integritätstests.
- Definierte Datenaufbewahrung.

Backups dürfen ohne die zur Entschlüsselung benötigten Schlüssel nicht als vollständig wiederherstellbar gelten.

---

## 13. Sicherheitsarchitektur

DDNSupd verarbeitet Zugangsdaten, mit denen produktive DNS-Zonen verändert werden können.

Sicherheit besitzt daher hohe Priorität.

Anforderungen:

- Verschlüsselung gespeicherter Credentials mit authentifizierter Verschlüsselung.
- Getrennte Schlüsselverwaltung.
- Keine Secrets in Logs.
- Kein Versand von Secrets an unberechtigte Clients.
- Granulare API-Berechtigungen.
- CSRF-Schutz bei cookiebasierter Authentifizierung.
- Schutz vor XSS und SQL Injection.
- Sichere Session-Cookies.
- Audit-Logging.
- Rate Limiting.
- SSRF-Schutz bei benutzerdefinierten IP-Check-URLs und Webhooks.
- Strikte Prüfung erlaubter DNS-Zonen.
- Sicherheitsprüfung von Abhängigkeiten.
- Keine unnötige Einbindung des Docker-Sockets.
- Sichere Rotation von Zugangsdaten.

Ein Import darf keine ungeprüften Kommandos, internen Netzwerkzugriffe oder unberechtigten DNS-Änderungen ermöglichen.

---

## 14. Datenbankmodell

Entwickle ein normalisiertes, migrationsfähiges Datenbankschema.

Vorgeschlagene Kernentitäten:

- `users`
- `roles`
- `permissions`
- `user_sessions`
- `api_tokens`
- `providers`
- `provider_credentials`
- `dns_zones`
- `dns_records`
- `record_permissions`
- `ip_sources`
- `update_jobs`
- `update_history`
- `ip_history`
- `notification_channels`
- `notification_rules`
- `audit_logs`
- `system_settings`

Alle migrationspflichtigen Änderungen müssen versioniert sein.

Veröffentlichte Migrationen dürfen nicht nachträglich verändert werden.

---

## 15. Import und Migration

Unterstütze den Import vorhandener Konfigurationen von `qdm12/ddns-updater`.

Insbesondere:

- `config.json`
- Providerkonfigurationen
- Domains
- IP-Versionen
- Intervalle
- Optionale IPv6-Suffixe
- Vorhandene Update-Historie aus `updates.json`, soweit kompatibel

Implementiere einen Importassistenten mit:

1. Datei auswählen.
2. Format überprüfen.
3. Einträge analysieren.
4. Zuordnung zu Providern anzeigen.
5. Konflikte erkennen.
6. Importvorschau anzeigen.
7. Import bestätigen.
8. Ergebnis protokollieren.

Beim Export müssen geheime Zugangsdaten standardmäßig ausgeschlossen sein.

Ein optionaler verschlüsselter Vollbackup-Export wird separat behandelt.

---

## 16. GitHub-Repository und Entwicklung

Orientiere dich an der Entwicklungsorganisation von TalkOps.

Vorgeschlagene Struktur:

```text
DDNSupd/
├── .github/
│   └── workflows/
├── crates/
│   ├── ddnsupd-api/
│   ├── ddnsupd-core/
│   └── ddnsupd-worker/
├── web/
├── provider-engine/
├── migrations/
├── docker/
├── tests/
├── docs/
│   ├── adr/
│   ├── ai-memory/
│   ├── de/
│   └── en/
├── scripts/
├── AGENTS.md
├── CLAUDE.md
├── CHANGELOG.md
├── README.md
├── docker-compose.yml
├── docker-compose.dev.yml
└── .env.example
```

Die endgültige Ordnerstruktur hängt von der Architekturentscheidung für die Provider-Engine ab.

### 16.1 Coding-Konventionen

- Saubere Modulgrenzen.
- Verständliche Schnittstellen.
- Typisierte Fehler.
- Automatisierte Tests.
- Keine unnötigen Abhängigkeiten.
- Conventional Commits.
- Semantische Versionierung.
- Nachvollziehbare Architekturentscheidungen.
- Deutsche und englische Benutzerhandbücher.

### 16.2 Claude Code und Codex

Implementiere eine Zusammenarbeit analog TalkOps.

`CLAUDE.md` ist die zentrale Projektanweisung.

`AGENTS.md` verweist auf die zentralen Regeln.

Arbeitsergebnisse werden zusätzlich in `docs/ai-memory/` dokumentiert.

Bei paralleler Entwicklung:

- Eigene Feature-Branches.
- Separate Git-Worktrees.
- Keine Überschreibung fremder Änderungen.
- Aufgabenbezogene Pull Requests.
- Klare Übergabeinformationen.
- Dedizierter Release-Prozess.

### 16.3 CI/CD

GitHub Actions müssen mindestens folgende Prüfungen ermöglichen:

- Rust Formatprüfung.
- Rust Clippy.
- Rust Tests.
- Frontend Lint.
- TypeScript/Svelte Check.
- Frontend Build.
- API-Integrationstests.
- Provider-Mocktests.
- Authentifizierungs- und Berechtigungstests.
- Docker-Build.
- Compose-Validierung.
- Browser-E2E-Tests.
- Dependency- und Image-Security-Scans.

Produktive DNS-Updates dürfen niemals unbeabsichtigt durch CI-Tests ausgelöst werden.

Veröffentliche freigegebene Containerimages versioniert über GHCR.

---

## 17. Entwicklungsphasen

### Phase 0 – Analyse und Architektur

- TalkOps-Design und Architektur analysieren.
- DDNS-Upstream analysieren.
- Provider- und Funktionsmatrix erstellen.
- Wiederverwendungskonzept definieren.
- Lizenzkompatibilität überprüfen.
- ADRs erstellen.
- Projektstruktur und Roadmap anlegen.

**Abnahme:** Dokumentierte Architektur und verifizierter Implementierungsplan.

### Phase 1 – Technisches Grundsystem

- Repository-Struktur.
- Rust Workspace.
- Frontend-Grundsystem.
- PostgreSQL.
- Migrationen.
- Docker Compose.
- Konfiguration.
- Healthchecks.
- CI-Pipeline.

**Abnahme:** Der vollständige Stack startet reproduzierbar.

### Phase 2 – Authentifizierung und Webportal

- Login.
- Ersteinrichtungsassistent.
- Benutzerverwaltung.
- Rollen.
- Dark-/Light-Mode.
- Hauptnavigation.
- Dashboard-Grundstruktur.

**Abnahme:** Authentifizierter und berechtigungsgeprüfter Zugriff.

### Phase 3 – DDNS-Kernfunktionen

- IP-Ermittlung.
- DNS-Record-Verwaltung.
- Scheduler.
- Provider-Registry.
- Erste Provider.
- Update-Historie.
- Manuelle Aktualisierung.

**Abnahme:** Echte DNS-Updates in freigegebenen Testzonen.

### Phase 4 – Erweiterte Verwaltung

- Providerverwaltung.
- Domainübersicht.
- Credential-Management.
- Konfigurationsimport.
- Record-Detailpanel.
- Filter.
- Massenaktionen.

**Abnahme:** Vollständige Verwaltung über das Webportal.

### Phase 5 – Monitoring und Benachrichtigungen

- Systemmetriken.
- Monitoring.
- Benachrichtigungen.
- Webhooks.
- Fehlerdiagnose.
- Ereignisprotokoll.

**Abnahme:** Fehler und Zustandsänderungen sind nachvollziehbar.

### Phase 6 – Sicherheit und Recovery

- Backup und Restore.
- Sicherheitsprüfungen.
- Authentifizierungs-Hardening.
- Audit.
- Recovery-Tests.
- Dokumentation.

**Abnahme:** Wiederherstellbarer, sicher betreibbarer Stack.

### Phase 7 – Provider-Parität und Release

- Weitere Upstream-Provider.
- Provider-Verifikation.
- Regressionstests.
- Browser-E2E.
- Release-Build.
- Benutzerhandbuch.
- GHCR-Veröffentlichung.

**Abnahme:** Dokumentierter MVP-Release mit nachvollziehbarer Provider-Unterstützung.

---

## 18. Autonome Entwicklungsweise

Entwickle DDNSupd möglichst eigenständig bis zu einem vollständig testbaren MVP.

Für gewöhnliche Implementierungsentscheidungen ist keine Rückfrage erforderlich.

Frage nur nach Entscheidungen, die die Architektur, den Funktionsumfang oder sicherheitsrelevante Abläufe wesentlich verändern.

Arbeite nicht mit isolierten Code-Snippets, sondern integriere alle Komponenten.

Jede Entwicklungsphase muss einen ausführbaren Zwischenstand ergeben.

Zeige regelmäßig:

- Aktuelle Entwicklungsphase.
- Umgesetzte Funktionen.
- Neue oder geänderte Dateien.
- Getroffene Architekturentscheidungen.
- Testergebnisse.
- Fehlende Funktionen.
- Bekannte Einschränkungen.
- Nächsten Entwicklungsschritt.

Halte die Ergebnisse dauerhaft im Repository fest, damit Claude Code, Codex oder andere Entwicklungsumgebungen die Arbeit fortsetzen können.

Führe keine produktiven DNS-Änderungen ohne ausdrücklich bereitgestellte Testzone und entsprechende Freigabe durch.

Repository-Erstellung, Veröffentlichungen und Deployment auf produktive Hosts sind separate freizugebende Aktionen.

---

## 19. Definition of Done

Ein Entwicklungsschritt gilt erst als abgeschlossen, wenn:

- Alle geplanten Funktionen implementiert sind.
- Änderungen dokumentiert sind.
- Relevante Tests durchgeführt wurden.
- Bekannte Fehler und Einschränkungen benannt sind.
- Keine Secrets im Repository enthalten sind.
- Bestehende Funktionen nicht unbeabsichtigt beeinträchtigt werden.
- Docker-Builds reproduzierbar sind.
- Sicherheitsrelevante Änderungen geprüft wurden.
- Eine Aufgaben-Zusammenfassung vorhanden ist.

Das MVP muss sich auf einem frischen Docker-Host aus dokumentierten Dateien starten lassen.

Ein Benutzer muss nach der Einrichtung mindestens einen DNS-Provider hinterlegen, einen Record anlegen, dessen IP automatisch aktualisieren und den Vorgang über das Webportal nachvollziehen können.

---

## 20. Arbeitsauftrag

Beginne mit Phase 0.

1. Analysiere beide Referenz-Repositories.
2. Vergleiche deren Architektur und Funktionsumfang.
3. Erstelle einen Architekturvorschlag für DDNSupd.
4. Lege offene Produktentscheidungen dar.
5. Erstelle die Provider-Paritätsmatrix.
6. Definiere eine vollständige Roadmap.
7. Implementiere nach Festlegung der Architektur inkrementell ein testbares MVP.
8. Halte Entwicklung, Tests und Entscheidungen im Repository fest.

**Oberstes Ziel: Ein eigenständiges, stabiles und komfortables DDNS-Management-Produkt mit der Bedienqualität von TalkOps und möglichst vollständiger, überprüfter Funktionsparität zu qdm12/ddns-updater.**
