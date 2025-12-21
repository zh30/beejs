# Beejs Runtime Docker Image
# This image provides a ready-to-use Beejs runtime environment

FROM node:18-alpine AS base

# Install system dependencies
RUN apk add --no-cache \
    curl \
    tar \
    ca-certificates \
    && update-ca-certificates

# Install Beejs Runtime
ARG BEEJS_VERSION=0.1.0
RUN curl -L "https://github.com/beejs-team/beejs/releases/download/v${BEEJS_VERSION}/beejs-linux-x64.tar.gz" \
    -o beejs.tar.gz \
    && tar -xzf beejs.tar.gz \
    && chmod +x beejs \
    && mv beejs /usr/local/bin/ \
    && rm beejs.tar.gz

# Verify installation
RUN beejs --version

# Create app directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production && npm cache clean --force

# Copy application code
COPY . .

# Production stage
FROM base AS production

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S beejs -u 1001

# Set ownership of app directory
RUN chown -R beejs:nodejs /app
USER beejs

# Expose port for server mode
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD beejs --version || exit 1

# Default command
CMD ["beejs", "start"]

# Development stage
FROM base AS development

# Install development dependencies
RUN npm ci

# Install development tools
RUN npm install -g \
    typescript \
    ts-node \
    @types/node

# Create volume for source code
VOLUME ["/app"]

# Default to interactive mode
CMD ["beejs", "repl"]

# Test stage
FROM development AS test

# Run tests
RUN beejs test --reporter=json --output=/tmp/test-results.json

# Save test results
VOLUME ["/tmp/test-results.json"]

# Build stage
FROM development AS build

# Build application
RUN beejs bundle src/index.js --outfile=dist/bundle.js

# Export build artifacts
VOLUME ["/app/dist"]

# Bundle stage
FROM node:18-alpine AS bundle

WORKDIR /app

# Copy Beejs binary
COPY --from=base /usr/local/bin/beejs /usr/local/bin/beejs

# Copy built bundle
COPY --from=build /app/dist/bundle.js /app/bundle.js

# Set permissions
RUN chmod +x /usr/local/bin/beejs

# Verify
RUN beejs --version

# Export bundle
VOLUME ["/app/bundle.js"]

CMD ["cat", "/app/bundle.js"]
