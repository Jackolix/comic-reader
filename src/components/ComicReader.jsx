import React, { useState, useEffect, useRef } from 'react';
import { ZoomIn, ZoomOut, Menu, FolderPlus, Globe } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useTheme } from 'next-themes';
import JSZip from 'jszip';

const ComicReader = () => {
  const [library, setLibrary] = useState([]);
  const [currentComic, setCurrentComic] = useState(null);
  const [images, setImages] = useState([]);
  const [zoom, setZoom] = useState(1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [serverUrl, setServerUrl] = useState('');
  const [isServerDialogOpen, setIsServerDialogOpen] = useState(false);
  const [isLibraryOpen, setIsLibraryOpen] = useState(true);
  const scrollContainerRef = useRef(null);
  const { theme, setTheme } = useTheme();
  const [savedServers, setSavedServers] = useState([]);

  // Load saved servers from localStorage on mount
  useEffect(() => {
    const servers = JSON.parse(localStorage.getItem('comicReaderServers')) || [];
    setSavedServers(servers);
    // Auto-load comics from saved servers
    servers.forEach(server => loadServerComics(server));
  }, []);

  // Save servers to localStorage when they change
  useEffect(() => {
    localStorage.setItem('comicReaderServers', JSON.stringify(savedServers));
  }, [savedServers]);

  // Load cover for a remote comic
  const loadCover = async (comic) => {
    if (comic.type !== 'remote' || comic.cover) return;
    
    try {
      const comicId = comic.id.replace('remote-', '');
      const response = await fetch(`${comic.serverUrl}/covers/${encodeURIComponent(comicId)}`);
      if (!response.ok) throw new Error('Failed to load cover');
      
      const blob = await response.blob();
      const coverUrl = URL.createObjectURL(blob);
      
      setLibrary(prev => prev.map(item => 
        item.id === comic.id ? { ...item, cover: coverUrl } : item
      ));
    } catch (error) {
      console.error('Error loading cover:', error);
    }
  };

  // Load covers when comics are added
  useEffect(() => {
    library.forEach(comic => {
      if (comic.type === 'remote' && !comic.cover) {
        loadCover(comic);
      }
    });
  }, [library.length]);

  const loadServerComics = async (serverUrl) => {
    setLoading(true);
    setError(null);
    try {
      // Remove trailing slash if present
      const normalizedUrl = serverUrl.endsWith('/') ? serverUrl.slice(0, -1) : serverUrl;
      const response = await fetch(`${normalizedUrl}/comics`);
      if (!response.ok) throw new Error('Could not connect to server');
      
      const comics = await response.json();
      
      const serverComics = comics.map(comic => ({
        id: `remote-${comic.id}`,
        name: comic.name,
        type: 'remote',
        serverUrl: normalizedUrl,
        path: comic.path, // Use the exact path from the server
        cover: null,
        progress: 0
      }));
      
      setLibrary(prev => {
        // Remove existing comics from this server
        const filteredLibrary = prev.filter(comic => 
          comic.type !== 'remote' || comic.serverUrl !== normalizedUrl
        );
        return [...filteredLibrary, ...serverComics];
      });
      return true; // Return success
    } catch (error) {
      console.error('Server connection error:', error);
      setError('Could not connect to server: ' + error.message);
      return false; // Return failure
    } finally {
      setLoading(false);
    }
  };

  const handleServerAdd = async () => {
    if (!serverUrl) return;
    
    const success = await loadServerComics(serverUrl);
    
    if (success) {
      // Only save the server and close dialog if connection was successful
      if (!savedServers.includes(serverUrl)) {
        setSavedServers(prev => [...prev, serverUrl]);
      }
      setServerUrl('');
      setIsServerDialogOpen(false);
    }
  };

  const handleFileUpload = async (event) => {
    const files = Array.from(event.target.files);
    
    for (const file of files) {
      if (!file) continue;

      try {
        if (file.name.endsWith('.cbz')) {
          const zip = new JSZip();
          const zipContent = await zip.loadAsync(file);
          
          // Get first image for cover
          const allFiles = Object.entries(zipContent.files)
            .filter(([name, entry]) => !entry.dir && name.match(/\.(jpg|jpeg|png|gif)$/i))
            .sort(([a], [b]) => a.localeCompare(b, undefined, { numeric: true }));
            
          let coverUrl = null;
          if (allFiles.length > 0) {
            const coverBlob = await allFiles[0][1].async('blob');
            coverUrl = URL.createObjectURL(coverBlob);
          }

          const newComic = {
            id: `local-${Date.now()}-${file.name}`,
            name: file.name.replace(/\.cbz$/i, ''),
            file: file,
            type: 'local',
            cover: coverUrl,
            progress: 0
          };

          setLibrary(prev => [...prev, newComic]);
        }
      } catch (error) {
        console.error('Error processing file:', error);
        setError(error.message);
      }
    }
  };

  const loadComic = async (comic) => {
    setLoading(true);
    setError(null);
    try {
      if (comic.type === 'remote') {
        // Correctly encode the comic path, preserving the original structure
        const filename = comic.path.split('/').pop();
        const encodedPath = `/comics/${encodeURIComponent(decodeURIComponent(filename))}`;
        
        const response = await fetch(`${comic.serverUrl}${encodedPath}`, {
          method: 'GET',
          headers: {
            'Accept': 'application/zip',
          }
        });

        if (!response.ok) throw new Error('Failed to fetch comic');
        
        const blob = await response.blob();
        const zip = new JSZip();
        const zipContent = await zip.loadAsync(blob);
        
        const imageFiles = Object.entries(zipContent.files)
          .filter(([name, entry]) => !entry.dir && name.match(/\.(jpg|jpeg|png|gif)$/i))
          .sort(([a], [b]) => a.localeCompare(b, undefined, { numeric: true }));

        const imageUrls = await Promise.all(
          imageFiles.map(async ([_, entry]) => {
            const blob = await entry.async('blob');
            return URL.createObjectURL(blob);
          })
        );
        
        setImages(imageUrls);
        setCurrentComic(comic);
      } else if (comic.type === 'local') {
        const zip = new JSZip();
        const zipContent = await zip.loadAsync(comic.file);
        
        const imageFiles = Object.entries(zipContent.files)
          .filter(([name, entry]) => !entry.dir && name.match(/\.(jpg|jpeg|png|gif)$/i))
          .sort(([a], [b]) => a.localeCompare(b, undefined, { numeric: true }));

        const imageUrls = await Promise.all(
          imageFiles.map(async ([_, entry]) => {
            const blob = await entry.async('blob');
            return URL.createObjectURL(blob);
          })
        );
        
        setImages(imageUrls);
        setCurrentComic(comic);
      }
    } catch (error) {
      console.error('Error loading comic:', error);
      setError(error.message);
    }
    setLoading(false);
  };

  // Cleanup URLs when unmounting
  useEffect(() => {
    return () => {
      images.forEach(URL.revokeObjectURL);
      library.forEach(item => {
        if (item.cover) {
          URL.revokeObjectURL(item.cover);
        }
      });
    };
  }, [images, library]);

  return (
    <div className={`flex h-screen ${theme === 'dark' ? 'bg-slate-900' : 'bg-slate-50'}`}>
      <Sheet open={isLibraryOpen} onOpenChange={setIsLibraryOpen}>
        <SheetTrigger asChild>
          <Button 
            variant="ghost" 
            className={`fixed top-4 left-4 z-50 ${theme === 'dark' ? 'text-white' : ''}`}
          >
            <Menu className="h-6 w-6" />
          </Button>
        </SheetTrigger>
        <SheetContent 
          side="left" 
          className={`w-80 ${theme === 'dark' ? 'bg-[#1a2234] text-white border-none' : ''}`}
        >
          <SheetHeader>
            <SheetTitle className={theme === 'dark' ? 'text-white' : ''}>
              Comic Library
            </SheetTitle>
          </SheetHeader>
          <div className="mt-4 space-y-4 h-[calc(100vh-6rem)] flex flex-col">
            <div className="flex gap-2 flex-shrink-0">
              <Dialog>
                <DialogTrigger asChild>
                  <Button 
                    variant="outline" 
                    className={`w-full ${theme === 'dark' ? 'bg-[#2a324a] text-white border-[#3a4258] hover:bg-[#3a4258]' : ''}`}
                  >
                    <FolderPlus className="mr-2 h-4 w-4" />
                    Add Files
                  </Button>
                </DialogTrigger>
                <DialogContent className={theme === 'dark' ? 'bg-[#1a2234] text-white border-[#3a4258]' : ''}>
                  <DialogHeader>
                    <DialogTitle>Add Comics</DialogTitle>
                  </DialogHeader>
                  <Input
                    type="file"
                    accept=".cbz"
                    multiple
                    onChange={handleFileUpload}
                    className={`mt-4 ${theme === 'dark' ? 'bg-[#2a324a] border-[#3a4258] text-white' : ''}`}
                  />
                </DialogContent>
              </Dialog>
              
              <Dialog open={isServerDialogOpen} onOpenChange={setIsServerDialogOpen}>
                <DialogTrigger asChild>
                  <Button 
                    variant="outline" 
                    className={`w-full ${theme === 'dark' ? 'bg-[#2a324a] text-white border-[#3a4258] hover:bg-[#3a4258]' : ''}`}
                  >
                    <Globe className="mr-2 h-4 w-4" />
                    Add Server
                  </Button>
                </DialogTrigger>
                <DialogContent className={theme === 'dark' ? 'bg-[#1a2234] text-white border-[#3a4258]' : ''}>
                  <DialogHeader>
                    <DialogTitle>Add Server</DialogTitle>
                  </DialogHeader>
                  <div className="flex flex-col gap-4 mt-4">
                    <div className="flex gap-2">
                      <Input
                        type="url"
                        placeholder="Server URL"
                        value={serverUrl}
                        onChange={(e) => setServerUrl(e.target.value)}
                      />
                      <Button 
                        onClick={handleServerAdd}
                        className={theme === 'dark' ? 'text-white hover:bg-slate-700' : ''}
                      >Add</Button>
                    </div>
                    {savedServers.length > 0 && (
                      <div>
                        <h3 className="text-sm font-medium mb-2">Saved Servers</h3>
                        {savedServers.map((server, index) => (
                          <div key={index} className="flex items-center justify-between gap-2 mb-2">
                            <span className="text-sm truncate flex-1">{server}</span>
                            <Button
                              variant="destructive"
                              size="sm"
                              onClick={() => {
                                setSavedServers(prev => prev.filter(s => s !== server));
                                setLibrary(prev => prev.filter(comic => 
                                  comic.type !== 'remote' || comic.serverUrl !== server
                                ));
                              }}
                            >
                              Remove
                            </Button>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </DialogContent>
              </Dialog>
            </div>
            
            <div className="space-y-2 overflow-y-auto flex-1">
              {library.map((item) => (
                <Card 
                  key={item.id} 
                  className={`cursor-pointer ${
                    theme === 'dark' 
                      ? 'bg-[#2a324a] hover:bg-[#3a4258] border-[#3a4258]' 
                      : 'hover:bg-slate-100'
                  }`} 
                  onClick={() => loadComic(item)}
                >
                  <CardContent className="p-4 flex gap-4">
                    {item.cover ? (
                      <img 
                        src={item.cover} 
                        alt={`Cover of ${item.name}`}
                        className="w-20 h-28 object-cover rounded-sm"
                        onError={(e) => {
                          console.error('Error loading cover for:', item.name);
                          e.target.style.display = 'none';
                          e.target.nextSibling.style.display = 'flex';
                        }}
                      />
                    ) : (
                      <div className={`w-20 h-28 rounded-sm flex items-center justify-center p-1 ${
                        theme === 'dark' ? 'bg-[#1a2234]' : 'bg-slate-200'
                      }`}>
                        <span className={`text-xs ${
                          theme === 'dark' ? 'text-slate-300' : 'text-slate-500'
                        }`}>No cover found</span>
                      </div>
                    )}
                    <div>
                      <div className={`font-medium ${
                        theme === 'dark' ? 'text-white' : ''
                      }`}>{item.name}</div>
                      <div className={`text-sm ${
                        theme === 'dark' ? 'text-slate-300' : 'text-slate-500'
                      }`}>{item.type}</div>
                      {item.progress > 0 && (
                        <div className={`mt-2 h-1 rounded-full overflow-hidden ${
                          theme === 'dark' ? 'bg-[#1a2234]' : 'bg-slate-200'
                        }`}>
                          <div 
                            className="h-full bg-blue-500 transition-all duration-300" 
                            style={{ width: `${item.progress}%` }}
                          />
                        </div>
                      )}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
            
            <div className="flex justify-end pt-4">
              <Button
                variant="outline"
                onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                className={`w-full ${theme === 'dark' ? 'bg-[#2a324a] text-white border-[#3a4258] hover:bg-[#3a4258]' : ''}`}
              >
                Toggle {theme === 'dark' ? 'Light' : 'Dark'} Mode
              </Button>
            </div>
          </div>
        </SheetContent>
      </Sheet>

      <div className="flex-1 p-4">
        <Card className={`w-full h-full shadow-lg max-w-5xl mx-auto ${
          theme === 'dark' ? 'bg-slate-800 text-white' : 'bg-white'
        }`}>
          <CardContent className="p-4 h-full flex flex-col">
            {currentComic ? (
              <>
                <div className="flex justify-between items-center mb-4 gap-4">
                  <h2 className="text-lg font-medium">{currentComic.name}</h2>
                  <div className="flex gap-2">
                    <button
                      onClick={() => setZoom(prev => Math.max(prev - 0.2, 0.5))}
                      className={`p-2 rounded-lg transition-colors ${
                        theme === 'dark' 
                          ? 'hover:bg-slate-700 text-white' 
                          : 'hover:bg-slate-100'
                      }`}
                      aria-label="Zoom out"
                    >
                      <ZoomOut className="w-6 h-6" />
                    </button>
                    <button
                      onClick={() => setZoom(prev => Math.min(prev + 0.2, 3))}
                      className={`p-2 rounded-lg transition-colors ${
                        theme === 'dark' 
                          ? 'hover:bg-slate-700 text-white' 
                          : 'hover:bg-slate-100'
                      }`}
                      aria-label="Zoom in"
                    >
                      <ZoomIn className="w-6 h-6" />
                    </button>
                  </div>
                </div>

                <div 
                  ref={scrollContainerRef}
                  className={`flex-1 relative rounded-lg overflow-y-auto ${
                    theme === 'dark' ? 'bg-slate-700' : 'bg-slate-50'
                  }`}
                  style={{
                    scrollBehavior: 'smooth',
                    scrollSnapType: 'y mandatory'
                  }}
                >
                  {loading ? (
                    <div className="absolute inset-0 flex items-center justify-center">
                      <div className={`animate-spin rounded-full h-12 w-12 border-b-2 ${
                        theme === 'dark' ? 'border-white' : 'border-slate-900'
                      }`}></div>
                    </div>
                  ) : (
                    <div className="flex flex-col items-center">
                      {images.map((imgSrc, index) => (
                        <div
                          key={index}
                          className="w-full flex justify-center"
                          style={{ scrollSnapAlign: 'start' }}
                        >
                          <img
                            src={imgSrc}
                            alt={`Page ${index + 1}`}
                            className="max-w-full h-auto transition-transform duration-200"
                            style={{ 
                              transform: `scale(${zoom})`,
                              maxHeight: '85vh'
                            }}
                          />
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </>
            ) : (
              <div className={`flex-1 flex items-center justify-center ${
                theme === 'dark' ? 'text-slate-400' : 'text-slate-500'
              }`}>
                Select a comic from your library to begin reading
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default ComicReader;