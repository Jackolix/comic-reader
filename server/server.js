const express = require('express');
const cors = require('cors');
const fs = require('fs').promises;
const path = require('path');
const JSZip = require('jszip');

const app = express();

// Configure CORS to allow file downloads
app.use(cors({
  exposedHeaders: ['Content-Length', 'Content-Type', 'Content-Disposition']
}));

// Configuration - use correct Windows path
const COMICS_DIR = process.env.COMICS_DIR || '/comics';

// Add headers for binary file transfer
app.use((req, res, next) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Content-Disposition');
  next();
});

// List all comics
app.get('/api/comics', async (req, res) => {
  try {
    const files = await fs.readdir(COMICS_DIR);
    const comics = files
      .filter(file => file.toLowerCase().endsWith('.cbz'))
      .map(file => ({
        id: encodeURIComponent(file),
        name: file.replace(/\.cbz$/i, ''),
        path: `/api/comics/${encodeURIComponent(file)}`
      }));
    res.json(comics);
  } catch (error) {
    console.error('Error reading directory:', error);
    res.status(500).json({ error: error.message });
  }
});

// Get comic cover
app.get('/api/covers/:filename', async (req, res) => {
  try {
    const filename = decodeURIComponent(req.params.filename);
    const filePath = path.join(COMICS_DIR, filename);

    // Verify file exists
    await fs.access(filePath);

    // Read the CBZ file
    const data = await fs.readFile(filePath);
    const zip = await JSZip.loadAsync(data);
    
    // Get first image from the archive
    const imageFile = Object.entries(zip.files)
      .find(([name, file]) => !file.dir && name.match(/\.(jpg|jpeg|png|gif)$/i));

    if (!imageFile) {
      throw new Error('No cover image found');
    }

    // Extract and send the cover image
    const [_, entry] = imageFile;
    const imageData = await entry.async('nodebuffer');
    
    res.setHeader('Content-Type', 'image/jpeg');
    res.send(imageData);
  } catch (error) {
    console.error('Error serving cover:', error);
    res.status(404).json({ error: 'Cover not found' });
  }
});

// Get a specific comic
app.get('/api/comics/:filename', async (req, res) => {
  try {
    const filename = decodeURIComponent(req.params.filename);
    const filePath = path.join(COMICS_DIR, filename);

    // Verify file exists
    await fs.access(filePath);

    // Set appropriate headers for file download
    res.setHeader('Content-Type', 'application/zip');
    res.setHeader('Content-Disposition', `attachment; filename="${filename}"`);
    
    // Stream the file instead of loading it all at once
    const stream = require('fs').createReadStream(filePath);
    stream.pipe(res);
  } catch (error) {
    console.error('Error serving comic:', error);
    res.status(404).json({ error: 'Comic not found' });
  }
});

// Status endpoint for debugging
app.get('/api/status', (req, res) => {
  res.json({
    status: 'running',
    comicsDir: COMICS_DIR
  });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
  console.log(`Comics directory: ${COMICS_DIR}`);
});