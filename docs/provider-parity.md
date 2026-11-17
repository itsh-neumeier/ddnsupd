# Provider parity

Status at 2026-11-03. Frozen source inventory: [qdm12/ddns-updater](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers).

The source has 60 registry entries, including the development example. The inventory comes from constants, the factory and the provider directory. Vercel/custom occur in source; Netcup is absent from ProviderChoices() but present in the factory.

Statuses: **Unterstützt**, **In Entwicklung**, **Geplant**, **Nicht relevant**, **Technisch eingeschränkt**. Mock tests and live verification are separate. No live integration is verified. The two implemented adapters remain In Entwicklung until their broader provider integration scope is complete.

The JSON file is authoritative. Run `python scripts/render-provider-matrix.py` after a catalog change. See [the feature matrix](feature-parity.md) for bounded implemented behavior.

| ID | Provider | Priority | DDNSupd status | Verification | Notes |
| --- | --- | --- | --- | --- | --- |
| aliyun | [aliyun](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/aliyun) | — | Geplant | not_tested |  |
| allinkl | [allinkl](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/allinkl) | — | Geplant | not_tested |  |
| bunny | [bunny](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/bunny) | — | Geplant | not_tested |  |
| changeip | [changeip](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/changeip) | — | Geplant | not_tested |  |
| cloudflare | [Cloudflare](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/cloudflare) | 3 | In Entwicklung | mock_tested | Existing A/AAAA origin updates, preservation, paginated read-only discovery and paused config import are mock-tested. Remote creation and live verification remain pending. |
| custom | [Custom HTTPS](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/custom) | — | Geplant | not_tested | Restricted until SSRF-safe transport and administrator endpoint policy exist. |
| dd24 | [dd24](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dd24) | — | Geplant | not_tested |  |
| ddnss | [ddnss](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/ddnss) | — | Geplant | not_tested |  |
| desec | [deSEC](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/desec) | 8 | Geplant | not_tested |  |
| digitalocean | [digitalocean](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/digitalocean) | — | Geplant | not_tested |  |
| dnsomatic | [dnsomatic](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dnsomatic) | — | Geplant | not_tested |  |
| dnspod | [dnspod](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dnspod) | — | Geplant | not_tested |  |
| domeneshop | [domeneshop](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/domeneshop) | — | Geplant | not_tested |  |
| dondominio | [dondominio](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dondominio) | — | Geplant | not_tested |  |
| dreamhost | [dreamhost](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dreamhost) | — | Geplant | not_tested |  |
| duckdns | [DuckDNS](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/duckdns) | 9 | Geplant | not_tested |  |
| dyn | [DynDNS](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dyn) | 10 | Geplant | not_tested |  |
| dynu | [dynu](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dynu) | — | Geplant | not_tested |  |
| dynv6 | [dynv6](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/dynv6) | — | Geplant | not_tested |  |
| easydns | [easydns](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/easydns) | — | Geplant | not_tested |  |
| example | [Example (development)](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/example) | — | Nicht relevant | not_tested | Upstream development scaffold; excluded from production parity. |
| freedns | [freedns](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/freedns) | — | Geplant | not_tested |  |
| gandi | [gandi](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/gandi) | — | Geplant | not_tested |  |
| gcp | [gcp](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/gcp) | — | Geplant | not_tested |  |
| gigahostno | [gigahostno](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/gigahostno) | — | Geplant | not_tested |  |
| godaddy | [godaddy](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/godaddy) | — | Geplant | not_tested |  |
| goip | [goip](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/goip) | — | Geplant | not_tested |  |
| he | [he](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/he) | — | Geplant | not_tested |  |
| hetzner | [Hetzner Legacy DNS](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/hetzner) | 2 | Technisch eingeschränkt | not_tested | API removed on 2026-05-27; import migration to Cloud requires new credentials. |
| hetznercloud | [Hetzner Cloud DNS](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/hetznercloud) | 1 | In Entwicklung | mock_tested | RRSet preservation, paginated discovery, paused config import and confirmed A/AAAA creation in primary zones are mock-tested. Live verification remains pending. |
| hostinger | [hostinger](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/hostinger) | — | Geplant | not_tested |  |
| infomaniak | [infomaniak](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/infomaniak) | — | Geplant | not_tested |  |
| inwx | [INWX](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/inwx) | 5 | Geplant | not_tested |  |
| ionos | [IONOS](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/ionos) | 4 | Geplant | not_tested |  |
| ipv64 | [ipv64](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/ipv64) | — | Geplant | not_tested |  |
| linode | [linode](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/linode) | — | Geplant | not_tested |  |
| loopia | [loopia](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/loopia) | — | Geplant | not_tested |  |
| luadns | [luadns](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/luadns) | — | Geplant | not_tested |  |
| myaddr | [myaddr](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/myaddr) | — | Geplant | not_tested |  |
| namecheap | [namecheap](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/namecheap) | — | Geplant | not_tested |  |
| name.com | [name.com](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/namecom) | — | Geplant | not_tested |  |
| namesilo | [namesilo](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/namesilo) | — | Geplant | not_tested |  |
| netcup | [Netcup](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/netcup) | 6 | Geplant | not_tested | Present in factory and constants, omitted from ProviderChoices(). |
| njalla | [njalla](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/njalla) | — | Geplant | not_tested |  |
| noip | [noip](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/noip) | — | Geplant | not_tested |  |
| nowdns | [nowdns](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/nowdns) | — | Geplant | not_tested |  |
| opendns | [opendns](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/opendns) | — | Geplant | not_tested |  |
| ovh | [ovh](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/ovh) | — | Geplant | not_tested |  |
| porkbun | [porkbun](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/porkbun) | — | Geplant | not_tested |  |
| route53 | [route53](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/route53) | — | Geplant | not_tested |  |
| scaleway | [scaleway](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/scaleway) | — | Geplant | not_tested |  |
| selfhost.de | [selfhost.de](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/selfhostde) | — | Geplant | not_tested |  |
| servercow | [servercow](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/servercow) | — | Geplant | not_tested |  |
| spaceship | [spaceship](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/spaceship) | — | Geplant | not_tested |  |
| spdyn | [spdyn](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/spdyn) | — | Geplant | not_tested |  |
| strato | [Strato](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/strato) | 7 | Geplant | not_tested |  |
| variomedia | [variomedia](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/variomedia) | — | Geplant | not_tested |  |
| vercel | [Vercel](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/vercel) | — | Geplant | not_tested | Present in source registry; omitted from README provider list. |
| vultr | [vultr](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/vultr) | — | Geplant | not_tested |  |
| zoneedit | [zoneedit](https://github.com/qdm12/ddns-updater/tree/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2/internal/provider/providers/zoneedit) | — | Geplant | not_tested |  |
