FROM nginx:alpine

RUN rm /etc/nginx/nginx.conf
COPY infra/Nginx/nginx.conf /etc/nginx/nginx.conf
RUN mkdir -p /usr/share/nginx/html
EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://127.0.0.1:8000/health || exit 1

CMD ["nginx", "-g", "daemon off;"]
