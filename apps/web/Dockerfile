FROM node:20-alpine

WORKDIR /workspace
RUN corepack enable
ENV CI=true

COPY . .
RUN pnpm install --frozen-lockfile

EXPOSE 3000
CMD ["pnpm", "--filter", "@loglens/web", "exec", "next", "dev", "--hostname", "0.0.0.0", "--port", "3000"]
