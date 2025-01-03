# Comic Reader

A web-based comic reader application built with React and Node.js that supports both local and server-hosted CBZ files.

## Features

- Read CBZ comic files
- Smooth scrolling between pages
- Zoom controls
- Library management
- Support for both local files and remote server
- Cover image previews
- Progress tracking
- Docker support for easy deployment

## Structure

```
comic-reader/
├── client/          # React frontend
├── server/          # Express backend
└── docker-compose.yml
```

## Prerequisites

- Node.js 18 or higher
- Docker and Docker Compose (for containerized setup)
- A modern web browser

## Quick Start

### Development

1. Clone the repository:
```bash
git clone https://github.com/jackolix/comic-reader.git
cd comic-reader
```

2. Install dependencies:
```bash
# Install root dependencies
npm install

# Install client dependencies
npm install

# Install server dependencies
cd server
npm install
```

3. Start the development servers:
```bash
# Start both frontend and backend
npm run dev

# Or start separately:
npm run client  # Frontend only
npm run server  # Backend only
```

### Docker Setup

1. Configure your comics directory in `docker-compose.yml`:
```yaml
services:
  server:
    volumes:
      - /path/to/your/comics:/comics  # Change this path
```

2. Build and start the containers:
```bash
docker-compose up -d
```

3. Access the application:
- Frontend: http://localhost:5173
- Backend API: http://localhost:3000

## Configuration

### Server Configuration
- Comics directory can be configured via environment variable:
  ```
  COMICS_DIR=/path/to/comics
  ```

### Client Configuration
- API URL can be configured in the environment:
  ```
  VITE_API_URL=http://localhost:3000
  ```

## Usage

1. Open the application in your browser
2. Add comics either by:
   - Uploading local CBZ files
   - Connecting to a comic server
3. Click on a comic in the library to start reading
4. Use scroll or arrow keys to navigate
5. Use zoom controls to adjust view

## API Endpoints

- `GET /api/comics` - List all available comics
- `GET /api/comics/:filename` - Get a specific comic
- `GET /api/covers/:filename` - Get a comic's cover image
- `GET /api/status` - Check server status

## Development

### Frontend
- Built with React + Vite
- Uses shadcn/ui for components
- State management with React hooks

### Backend
- Express.js server
- File streaming for comic delivery
- CORS enabled for development

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Uses [JSZip](https://stuk.github.io/jszip/) for CBZ file handling
- UI components from [shadcn/ui](https://ui.shadcn.com/)
- Icons from [Lucide](https://lucide.dev/)