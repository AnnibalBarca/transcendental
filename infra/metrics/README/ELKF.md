# ELKF

ELK (+F) se compose en plusieurs modules qui vont gerer de maniere efficace nos log.
On va stocker, collecter et visualiser nos log a grande echelle pour note monitoring.


## Elasticsearch:

Elasticsearch est notre point d'entree pour ce module. C'est la base de donnee + le moteur.
Il stock tous les logs en index, ce qui permet de faire des recherches ultra-rapides dessus, meme sur des millions de lignes.
Normalement plusieurs noeuds mais pour notre projet un seul suffit 
``discovery.type: single-node``

On va le mettre sur le port ``9200`` le faire tourner sur un conteneur java ``et - "ES_JAVA_OPTS=-Xms512m -Xmx512m"``

En gros Elasticsearch c'est la fondation: si il crash ou met trop de temps a demarrer, tout le reste du module (Logstash, Kibana) ne pourra pas fonctionner correctement.


## Logstash

Logstash recoit les logs envoyes par Filebeat, les transforme/enrichit, puis les envoie vers Elasticsearch pour stockage : `Filebeat → Logstash (port 5044) → Elasticsearch`

Le port 5044 (convension) est utilise par le protocole Beats pour envoyer des donnees a Logstash. Les deux cotes (Filebeat et Logstash) doivent etre configures sur le meme port.

**`elk/logstash/pipeline/logstash.conf`** — definit le pipeline de traitement :
- `input { beats { port => 5044 } }` : ecoute les logs envoyes par Filebeat
- `filter {}` : vide pour l'instant, c'est ici qu'on ajoutera le parsing (extraction JSON, nom du service...)
- `output` : double sortie — Elasticsearch (stockage reel) + stdout (affichage dans la console Logstash pour debug)


## Kibana

Kibana sera notre interface visuelle.
Il se connecte à Elasticsearch et te permet de creer des dashboards, graphiques, recherches, alertes... C'est ce qu'on verra dans le navigateur.

**Acces** : `http://localhost:5601`
**Username** : `elastic`
**Password** : `CHANGE_LATER`

Le menu principal se trouve dans les trois traits en haut à gauche (☰).
La gestion de la stack (Data Views, utilisateurs, index) se trouve dans **Stack Management**.

Kibana utilise un compte technique dédié (`kibana_system`) pour se connecter lui-même à Elasticsearch. `kibana_system` n'a que les permissions nécessaires au fonctionnement interne de Kibana, pas un accès admin complet.

Mot de passe généré avec :
```bash
docker exec -it elasticsearch /usr/share/elasticsearch/bin/elasticsearch-reset-password -u kibana_system
```

Pour confirmer que Kibana est lie a Elastic on va dans ☰ --> DevTools --> console

On tape ``GET /`` puis executer la console, ce qui devrait donner :

```json
{
  "name": "es-node-1",
  "cluster_name": "ft-transcendence-cluster",
  "cluster_uuid": "XuXcVMxtSnGby2a3ZQiluw",
  "version": {
    "number": "8.13.0",
    "build_flavor": "default",
    "build_type": "docker",
    "build_hash": "09df99393193b2c53d92899662a8b8b3c55b45cd",
    "build_date": "2024-03-22T03:35:46.757803203Z",
    "build_snapshot": false,
    "lucene_version": "9.10.0",
    "minimum_wire_compatibility_version": "7.17.0",
    "minimum_index_compatibility_version": "7.0.0"
  },
  "tagline": "You Know, for Search"
}
```

## Filebeat

Filebeat collecte les logs de tous les conteneurs Docker et les envoie a Logstash : `Conteneurs Docker → Filebeat → Logstash (port 5044)`

**Pourquoi pas l'autodiscover Docker classique ?**
La méthode standard se connecte au socket Docker (`/var/run/docker.sock`), mais ca necessite les droits root sur l'hote — qu'on n'a pas sur la VM 42 ;-;.

On lit donc directement les fichiers `.log` que Docker écrit déjà sur disque, ce qui ne demande qu'un accès en lecture (déjà fourni par le bind mount).

**Volume utilisé** :
```yaml
- /goinfre/tarini/docker/containers:/var/lib/docker/containers:ro
```
(chemin obtenu avec `docker info | grep "Docker Root Dir"`)

---


### Data View & Dashboard

#### Data View

Créée dans Kibana pour pointer vers tous les index de logs :
- **Name** : `ft-transcendence-logs`
- **Index pattern** : `ft_transcendence-logs-*`
- **Timestamp field** : `@timestamp`

☰ → Stack Management → Data Views → Create data view

#### Dashboard : `ft-transcendence-logs-dashboard`

☰ → Dashboard → `ft-transcendence-logs-dashboard`

3 panels :
- **Bar chart** : volume de logs dans le temps (`@timestamp` × Count)
- **Metric** : nombre total de logs
- **Donut** : répartition stdout/stderr

#### ILM (rétention des logs)

Politique créée via Dev Tools :

```json
PUT _ilm/policy/ft-transcendence-logs-policy
```

3 phases :
- **hot** : index actif, rollover si > 1 jour ou > 5 Go
- **warm** : après 3 jours, compression (shrink à 1 shard)
- **delete** : suppression automatique après 30 jours

Index template associé pour appliquer automatiquement la politique aux nouveaux index :

```json
PUT _index_template/ft-transcendence-logs-template
```

- `index_patterns` : `ft_transcendence-logs-*`
- `number_of_replicas` : 0 (single-node, évite le statut yellow)

#### État actuel

- [x] Elasticsearch démarré et authentifié
- [x] Kibana connecté à Elasticsearch
- [x] Logstash démarré, connecté à Elasticsearch
- [x] Filebeat collecte les logs Docker
- [x] +1 000 000 logs stockés dans Elasticsearch
- [x] Data View créée
- [x] Dashboard avec 3 visualisations
- [x] Politique ILM (rétention 30 jours)
- [x] Index template (appliqué aux futurs index)
- [ ] Sécurisation finale (TLS, mots de passe définitifs)