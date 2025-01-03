const express = require('express');
const cors = require('cors');
const fs = require('fs').promises;
const path = require('path');
const JSZip = require('jszip');

const app = express();

// Configure CORS
app.use(cors({
  exposedHeaders: ['Content-Length', 'Content-Type', 'Content-Disposition']
}));

const COMICS_DIR = process.env.COMICS_DIR || '/comics';

// List all comics
app.get('/comics', async (req, res) => {
  try {
    const files = await fs.readdir(COMICS_DIR);
    const comics = files
      .filter(file => file.toLowerCase().endsWith('.cbz'))
      .map(file => ({
        id: encodeURIComponent(file),
        name: file.replace(/\.cbz$/i, ''),
        path: `/comics/${encodeURIComponent(file)}`
      }));
    res.json(comics);
  } catch (error) {
    console.error('Error reading directory:', error);
    res.status(500).json({ error: 'Failed to read comics directory' });
  }
});

// Get comic cover
app.get('/covers/:filename', async (req, res) => {
  try {
    const filename = decodeURIComponent(req.params.filename);
    const filePath = path.join(COMICS_DIR, filename);

    // Verify file exists
    await fs.access(filePath);

    const data = await fs.readFile(filePath);
    const zip = await JSZip.loadAsync(data);
    
    const imageFile = Object.entries(zip.files)
      .find(([name, file]) => !file.dir && name.match(/\.(jpg|jpeg|png|gif)$/i));

    if (!imageFile) {
      return res.status(404).json({ error: 'No cover image found in comic' });
    }

    const [_, entry] = imageFile;
    const imageData = await entry.async('nodebuffer');
    
    res.setHeader('Content-Type', 'image/jpeg');
    res.send(imageData);
  } catch (error) {
    console.error('Error serving cover:', error);
    if (error.code === 'ENOENT') {
      res.status(404).json({ error: 'Comic file not found' });
    } else {
      res.status(500).json({ error: 'Failed to process comic cover' });
    }
  }
});

// Get a specific comic
app.get('/comics/:filename', async (req, res) => {
  try {
    const filename = decodeURIComponent(req.params.filename);
    const filePath = path.join(COMICS_DIR, filename);

    // Verify file exists
    await fs.access(filePath);

    res.setHeader('Content-Type', 'application/zip');
    res.setHeader('Content-Disposition', `attachment; filename="${filename}"`);
    
    const stream = require('fs').createReadStream(filePath);
    stream.on('error', (error) => {
      console.error('Stream error:', error);
      if (!res.headersSent) {
        res.status(500).json({ error: 'Failed to stream comic file' });
      }
    });
    
    stream.pipe(res);
  } catch (error) {
    console.error('Error serving comic:', error);
    if (error.code === 'ENOENT') {
      res.status(404).json({ error: 'Comic file not found' });
    } else {
      res.status(500).json({ error: 'Failed to serve comic file' });
    }
  }
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
  console.log(`Comics directory: ${COMICS_DIR}`);
});