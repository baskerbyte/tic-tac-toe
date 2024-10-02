FROM node:lts-alpine

RUN npm install -g pnpm

WORKDIR /usr/src/app
COPY . .

RUN pnpm install
RUN pnpm recursive run build
CMD ["pnpm", "recursive", "run", "preview"]