FROM node:18-alpine

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm install

# Copy server code
COPY server.js ./

# Create directory for comics
RUN mkdir -p /comics

# Expose port
EXPOSE 3000

CMD ["node", "server.js"]