# Development Dockerfile for Next.js dashboard
FROM node:22-alpine

# Install pnpm
RUN npm install -g pnpm

# Set working directory
WORKDIR /app

# Copy package files
COPY package.json ./

# Copy lockfile if it exists (optional)
COPY pnpm-lock.yaml* ./

# Install dependencies
# Use --frozen-lockfile if lockfile exists, otherwise regular install
RUN if [ -f pnpm-lock.yaml ]; then \
      pnpm install --frozen-lockfile; \
    else \
      pnpm install; \
    fi

# Expose port
EXPOSE 3000

# For development, we'll use volume mounts and run dev server
# The command will be overridden by docker-compose
CMD ["pnpm", "dev"]
