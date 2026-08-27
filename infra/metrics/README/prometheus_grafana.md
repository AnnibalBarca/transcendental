# Prometheus et Grafana

Prometheus collecte les métriques des conteneurs Docker et les stock
Grafana agit avec Prometheus grace au langage PromQL afin d'afficher des tableaux de bord et des graphiques

## Comment Prometheus stocke les donnees ?

Chaque métrique est enregistrée sous la forme : nom_metrique{labels} timestamp valeur

On peut le représenter comme :

```
Key                          |  Timestam  | Value |
                             |            |       |
cpu_usage{container="web"}   | 1715000000 |  0.35 |
cpu_usage{container="web"}	 | 1715000060 |	 0.42 |
memory_usage{container="db"} | 1715000060 |  512  |
```

PromQL (Prometheus Query Language) sert a recupérer et analyser les metriques

Grafana affichera une courbe représentant la memoire consommee au fil du temps.

```
         Exemple d'architecture
            Docker Containers
                    │
                    ▼
           cAdvisor / Exporters
                    │
                    ▼
                Prometheus
                    │
                 (PromQL)
                    ▼
                 Grafana
                    │
                    ▼
            Dashboards / Alertes
```

## cadvisor

## Thanos

