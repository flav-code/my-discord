FROM node:22-alpine
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm install
COPY index.ts ./
ENV PORT=3000
EXPOSE 3000
CMD ["npx", "tsx", "index.ts"]
