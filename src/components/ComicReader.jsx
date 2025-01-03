import React, { useState, useEffect, useRef } from 'react';
import { ZoomIn, ZoomOut } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import JSZip from 'jszip';

const ComicReader = () => {
  const [images, setImages] = useState([]);
  const [zoom, setZoom] = useState(1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const scrollContainerRef = useRef(null);

  const handleFileUpload = async (event) => {
    const file = event.target.files[0];
    if (!file) return;

    setLoading(true);
    setError(null);
    try {
      if (file.name.endsWith('.cbz')) {
        const zip = new JSZip();
        const zipContent = await zip.loadAsync(file);
        
        const imageFiles = [];
        for (const [filename, zipEntry] of Object.entries(zipContent.files)) {
          if (!zipEntry.dir && filename.match(/\.(jpg|jpeg|png|gif)$/i)) {
            imageFiles.push({
              name: filename,
              entry: zipEntry
            });
          }
        }
        
        imageFiles.sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true }));
        
        const imageUrls = await Promise.all(
          imageFiles.map(async ({ entry }) => {
            const blob = await entry.async('blob');
            return URL.createObjectURL(blob);
          })
        );
        
        setImages(imageUrls);
      } else if (file.type.startsWith('image/')) {
        const imageUrl = URL.createObjectURL(file);
        setImages([imageUrl]);
      } else {
        throw new Error('Unsupported file type. Please upload a CBZ file or an image.');
      }
    } catch (error) {
      console.error('Error processing file:', error);
      setError(error.message);
    }
    setLoading(false);
  };

  useEffect(() => {
    return () => {
      images.forEach(URL.revokeObjectURL);
    };
  }, [images]);

  const zoomIn = () => {
    setZoom(prev => Math.min(prev + 0.2, 3));
  };

  const zoomOut = () => {
    setZoom(prev => Math.max(prev - 0.2, 0.5));
  };

  // Handle keyboard events for zooming and scrolling
  useEffect(() => {
    const handleKeyPress = (e) => {
      const scrollAmount = 200; // Adjust this value to change scroll speed
      
      switch(e.key) {
        case '+':
          zoomIn();
          break;
        case '-':
          zoomOut();
          break;
        case 'ArrowDown':
          if (scrollContainerRef.current) {
            scrollContainerRef.current.scrollBy({
              top: scrollAmount,
              behavior: 'smooth'
            });
          }
          break;
        case 'ArrowUp':
          if (scrollContainerRef.current) {
            scrollContainerRef.current.scrollBy({
              top: -scrollAmount,
              behavior: 'smooth'
            });
          }
          break;
        case ' ': // Space bar
          if (scrollContainerRef.current) {
            scrollContainerRef.current.scrollBy({
              top: window.innerHeight * 0.8,
              behavior: 'smooth'
            });
          }
          e.preventDefault(); // Prevent default space bar behavior
          break;
        default:
          break;
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, []);

  return (
    <div className="flex flex-col items-center w-full h-screen">
      <Card className="w-full h-full bg-white shadow-lg max-w-5xl">
        <CardContent className="p-4 h-full flex flex-col">
          <div className="flex justify-between items-center mb-4 gap-4">
            <input
              type="file"
              accept=".cbz,image/*"
              onChange={handleFileUpload}
              className="flex-1 text-sm text-slate-500 file:mr-4 file:py-2 file:px-4 
                       file:rounded-full file:border-0 file:text-sm file:font-semibold
                       file:bg-slate-50 file:text-slate-700 hover:file:bg-slate-100
                       focus:outline-none"
            />
            <div className="flex gap-2">
              <button
                onClick={zoomOut}
                className="p-2 rounded-lg hover:bg-slate-100 transition-colors"
                aria-label="Zoom out"
              >
                <ZoomOut className="w-6 h-6" />
              </button>
              <button
                onClick={zoomIn}
                className="p-2 rounded-lg hover:bg-slate-100 transition-colors"
                aria-label="Zoom in"
              >
                <ZoomIn className="w-6 h-6" />
              </button>
            </div>
          </div>

          <div 
            ref={scrollContainerRef}
            className="flex-1 relative bg-slate-50 rounded-lg overflow-y-auto"
            style={{
              scrollBehavior: 'smooth',
              scrollSnapType: 'y mandatory'
            }}
          >
            {loading ? (
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-slate-900"></div>
              </div>
            ) : error ? (
              <div className="absolute inset-0 flex items-center justify-center text-red-500 px-4 text-center">
                {error}
              </div>
            ) : images.length > 0 ? (
              <div className="flex flex-col items-center">
                {images.map((imgSrc, index) => (
                  <div
                    key={index}
                    className="w-full flex justify-center"
                    style={{ scrollSnapAlign: 'start' }}
                  >
                    <img
                      src={imgSrc}
                      alt={`Comic page ${index + 1}`}
                      className="max-w-full h-auto transition-transform duration-200"
                      style={{ 
                        transform: `scale(${zoom})`,
                        maxHeight: '85vh'
                      }}
                    />
                  </div>
                ))}
              </div>
            ) : (
              <div className="absolute inset-0 flex items-center justify-center text-slate-500">
                Upload a comic file to begin reading
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default ComicReader;