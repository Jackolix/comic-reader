import React, { useState } from 'react';
import { ChevronDown, ChevronRight, Search } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';

const FolderTree = ({ folder, onComicSelect, theme, level = 0, searchQuery = '' }) => {
  const [isExpanded, setIsExpanded] = useState(level === 0);

  // Filter comics and subfolders based on search query
  const matchesSearch = (comic) => {
    const query = searchQuery.toLowerCase();
    return comic.name.toLowerCase().includes(query) ||
        (comic.series && comic.series.toLowerCase().includes(query)) ||
        (comic.folder_path && comic.folder_path.some(folder =>
            folder.toLowerCase().includes(query)
        ));
  };

  const filteredComics = folder.comics.filter(matchesSearch);
  const filteredSubfolders = searchQuery
      ? folder.subfolders.filter(subfolder =>
          subfolder.comics.some(matchesSearch) ||
          subfolder.subfolders.some(sub => sub.comics.some(matchesSearch))
      )
      : folder.subfolders;

  const hasContent = filteredComics.length > 0 || filteredSubfolders.length > 0;
  if (!hasContent && searchQuery) return null;

  return (
      <div className="ml-4" style={{ marginLeft: level === 0 ? 0 : '1rem' }}>
        {level > 0 && (
            <div
                className={`flex items-center gap-2 py-2 px-2 rounded-lg cursor-pointer ${
                    theme === 'dark' ? 'hover:bg-slate-700' : 'hover:bg-slate-100'
                }`}
                onClick={() => setIsExpanded(!isExpanded)}
            >
              {isExpanded ? (
                  <ChevronDown className="h-4 w-4" />
              ) : (
                  <ChevronRight className="h-4 w-4" />
              )}
              <span className="font-medium">{folder.name}</span>
              <span className="text-sm text-slate-500">({filteredComics.length})</span>
            </div>
        )}

        {isExpanded && (
            <div className="space-y-2">
              {filteredSubfolders.map((subfolder, index) => (
                  <FolderTree
                      key={`${subfolder.name}-${index}`}
                      folder={subfolder}
                      onComicSelect={onComicSelect}
                      theme={theme}
                      level={level + 1}
                      searchQuery={searchQuery}
                  />
              ))}

              {filteredComics.map((comic) => (
                  <Card
                      key={comic.id}
                      className={`cursor-pointer ${
                          theme === 'dark'
                              ? 'bg-[#2a324a] hover:bg-[#3a4258] border-[#3a4258]'
                              : 'hover:bg-slate-100'
                      }`}
                      onClick={() => onComicSelect(comic)}
                  >
                    <CardContent className="p-4 flex gap-4">
                      {comic.cover ? (
                          <img
                              src={comic.cover}
                              alt={`Cover of ${comic.name}`}
                              className="w-20 h-28 object-cover rounded-sm"
                              onError={(e) => {
                                e.target.style.display = 'none';
                                e.target.nextSibling.style.display = 'flex';
                              }}
                              crossOrigin="anonymous"
                          />
                      ) : (
                          <div className={`w-20 h-28 rounded-sm flex items-center justify-center p-1 ${
                              theme === 'dark' ? 'bg-[#1a2234]' : 'bg-slate-200'
                          }`}>
                            <span className="text-xs text-slate-500">No cover</span>
                          </div>
                      )}
                      <div>
                        <div className={`font-medium ${
                            theme === 'dark' ? 'text-white' : ''
                        }`}>{comic.name}</div>
                        {comic.folder_path && comic.folder_path.length > 0 && (
                            <div className={`text-sm ${
                                theme === 'dark' ? 'text-slate-300' : 'text-slate-500'
                            }`}>
                              {comic.folder_path.join(' / ')}
                            </div>
                        )}
                      </div>
                    </CardContent>
                  </Card>
              ))}
            </div>
        )}
      </div>
  );
};

const ComicLibrary = ({ library, onComicSelect, theme }) => {
  const [searchQuery, setSearchQuery] = useState('');

  const buildFolderStructure = (comics) => {
    const root = {
      name: 'Library',
      path: [],
      comics: [],
      subfolders: []
    };

    // First sort comics between root and folders
    comics.forEach(comic => {
      if (!comic.folder_path || comic.folder_path.length === 0) {
        // Comics with no folder path go directly in root
        root.comics.push(comic);
      } else {
        let currentFolder = root;

        // Navigate or create folder path
        for (const folderName of comic.folder_path) {
          let folder = currentFolder.subfolders.find(f => f.name === folderName);
          if (!folder) {
            folder = {
              name: folderName,
              path: [...currentFolder.path, folderName],
              comics: [],
              subfolders: []
            };
            currentFolder.subfolders.push(folder);
          }
          currentFolder = folder;
        }
        currentFolder.comics.push(comic);
      }
    });

    // Sort comics in each folder by name
    const sortComics = (folder) => {
      folder.comics.sort((a, b) => a.name.localeCompare(b.name));
      folder.subfolders.forEach(sortComics);
    };
    sortComics(root);

    return root;
  };

  const folderStructure = buildFolderStructure(library);
  console.log('Folder structure:', folderStructure); // Debug log


  return (
      <div className="space-y-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-400 h-4 w-4" />
          <Input
              type="text"
              placeholder="Search comics and folders..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={`pl-10 ${
                  theme === 'dark'
                      ? 'bg-[#2a324a] border-[#3a4258] text-white placeholder-slate-400'
                      : ''
              }`}
          />
        </div>

        {/* Render root level comics first */}
        {folderStructure.comics.length > 0 && (
            <div className="mb-4">
              {folderStructure.comics.map((comic) => (
                  <Card
                      key={comic.id}
                      className={`cursor-pointer mb-2 ${
                          theme === 'dark'
                              ? 'bg-[#2a324a] hover:bg-[#3a4258] border-[#3a4258]'
                              : 'hover:bg-slate-100'
                      }`}
                      onClick={() => onComicSelect(comic)}
                  >
                    <CardContent className="p-4 flex gap-4">
                      {comic.cover ? (
                          <img
                              src={comic.cover}
                              alt={`Cover of ${comic.name}`}
                              className="w-20 h-28 object-cover rounded-sm"
                              onError={(e) => {
                                e.target.style.display = 'none';
                                e.target.nextSibling.style.display = 'flex';
                              }}
                              crossOrigin="anonymous"
                          />
                      ) : (
                          <div className={`w-20 h-28 rounded-sm flex items-center justify-center p-1 ${
                              theme === 'dark' ? 'bg-[#1a2234]' : 'bg-slate-200'
                          }`}>
                            <span className="text-xs text-slate-500">No cover</span>
                          </div>
                      )}
                      <div>
                        <div className={`font-medium ${
                            theme === 'dark' ? 'text-white' : ''
                        }`}>{comic.name}</div>
                      </div>
                    </CardContent>
                  </Card>
              ))}
            </div>
        )}

        {/* Then render folders */}
        <FolderTree
            folder={folderStructure}
            onComicSelect={onComicSelect}
            theme={theme}
            searchQuery={searchQuery}
        />
      </div>
  );
};

export default ComicLibrary;