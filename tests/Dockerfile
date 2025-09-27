FROM mcr.microsoft.com/playwright:v1.40.0-jammy

WORKDIR /tests

# Copy package files
COPY package*.json ./
RUN npm ci

# Copy test files
COPY . .

# Install browsers
RUN npx playwright install --with-deps

CMD ["npm", "test"]