# Development Guide

This guide provides detailed information for developers who want to contribute to or modify the Comic Reader application.

## Prerequisites

- Node.js 18 or higher
- Docker and Docker Compose (for containerized development)
- Git

## Project Structure

```
comic-reader/
├── client/          # React frontend
│   ├── src/         # Source files
│   └── public/      # Static assets
├── server/          # Express backend
└── docker-compose.yml
```

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/jackolix/comic-reader.git
   cd comic-reader
   ```

2. **Install dependencies**
   ```bash
   # Client dependencies
   cd client && npm install

   # Server dependencies
   cd server && npm install
   ```

3. **Start development servers**
   ```bash
   # Start the frontend
   npm run dev

   # Start the backend
   node ./server.js
   ```

## Environment Configuration

### Server Environment Variables
```bash
COMICS_DIR=/path/to/comics    # Comics directory path
PORT=3000                     # Server port (optional)
SERVER_PASSWORD=yourpassword  # Optional password protection
```

## Building for Production

### Frontend Build
```bash
cd client
npm run build
```

### Backend Build
```bash
cd server
npm run build
```

### Docker Build
```bash
# Build the frontend image
docker build -t comic-reader:latest -f client/Dockerfile .

# Build the backend image
docker build -t comic-server:latest -f server/Dockerfile .
```