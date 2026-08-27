# DEVOPS PART

pour le moment en francais 


Ce fichier est la pour expliquer chaque partie pour que vous compreniez chaque module DevOps

Deja on va faire un bridge, un bridge est un reseau virtuel Docker qui relie plusieurs conteneurs sur une meme machine en leur attribuant chacun une IP pour communiquer entre eux comme si ils etaient sur un meme reseau.

On l'utilise pour que les conteneurs puissent se parler par leur nom de service (ex: elasticsearch:9200) sans exposer leurs ports sur l'hote ou se soucier des IP. Ca isole le trafic du reste du systeme.

J'expliquerais ces module dans le dossier [README](README/) :

- [ELKF](README/ELKF.md)
- [Healthcheck](README/healthcheck.md)
- [Microservices](README/microservices.md)
- [Prometheus/Grafana](README/prometheus_grafana.md)

```
DevOps

- [+] ELK — 2 pts
- [~] Prometheus / Grafana — 2 pts
- [~] Microservices — 2 pts
- [] Healthcheck — 1 pt

Total : 7 pts

Module que je voudrais rajouter pour le fun :

- [~] Thanos pour conserver les métriques
- [+] Filebeat pour envoyer les logs à ELK (ELKF)
- [ ] Alertmanager pour les alertes
- [~] cAdvisor pour les métriques Docker

Pas fix, a voir ce que je rajouterais ou retirerais

'+' == fait
' ' == pas fait
'~' == en cours

2/7 pour le moment
```
