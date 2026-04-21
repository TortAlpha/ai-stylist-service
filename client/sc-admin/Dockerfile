ARG NODE_VERSION=20.18
ARG NGINX_VERSION=1.27

FROM node:${NODE_VERSION}-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build -- --configuration=production

FROM nginx:${NGINX_VERSION}-alpine AS runtime

COPY --from=builder /app/dist/sc-admin/browser /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
