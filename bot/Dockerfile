FROM node:22-alpine
WORKDIR /app

# Install bot dependencies
COPY bot/package.json bot/package-lock.json* ./bot/
RUN cd bot && npm install

# Install client dependencies (needed for module_bindings to resolve spacetimedb)
COPY client/package.json client/package-lock.json* ./client/
RUN cd client && npm install

# Copy source files
COPY bot/index.ts ./bot/
COPY client/module_bindings/ ./client/module_bindings/

WORKDIR /app/bot
CMD ["npx", "tsx", "index.ts"]
