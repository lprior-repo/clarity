# Dockerfile for Playwright E2E Testing of Clarity
#
# This container runs Playwright tests against the Clarity web interface.
# The Clarity app runs in web mode (http://localhost:8080) for testing.
#
# IMPORTANT: The Clarity desktop app cannot run directly in Docker.
# This setup requires a --web flag to be added to clarity-client.
#
# Build:
#   docker build -t clarity-e2e .
#
# Run:
#   docker run --rm --network host clarity-e2e
#
# With coverage:
#   docker run --rm -v $(pwd)/coverage:/app/coverage --network host clarity-e2e

FROM mcr.microsoft.com/playwright:v1.49.1-jammy

# Set working directory
WORKDIR /app

# Install Node.js dependencies
COPY package.json package-lock.json* ./
RUN npm ci

# Copy test files
COPY tests/ ./tests/
COPY playwright.config.ts ./
COPY tsconfig.json ./

# Install Playwright browsers
RUN npx playwright install --with-deps chromium

# Set environment variables
ENV PLAYWRIGHT_BROWSERS_PATH=/ms-playwright
ENV NODE_ENV=test

# Default command: run all tests
CMD ["npx", "playwright", "test"]
