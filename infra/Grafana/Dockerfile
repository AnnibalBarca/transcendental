FROM grafana/grafana:latest

# ENV GF_SECURITY_ADMIN_PASSWORD=admin
ENV GF_SECURITY_ADMIN_USER=admin
ENV GF_INSTALL_PLUGINS=grafana-piechart-panel

RUN mkdir -p /etc/grafana/provisioning/dashboards
RUN mkdir -p /etc/grafana/provisioning/datasources

COPY provisioning/datasources/prometheus.yml /etc/grafana/provisioning/datasources/prometheus.yml
COPY provisioning/dashboards/dashboards.yml /etc/grafana/provisioning/dashboards/dashboards.yml
COPY dashboards/ /etc/grafana/provisioning/dashboards/

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

CMD ["grafana-server", "--config=/etc/grafana/grafana.ini"]
