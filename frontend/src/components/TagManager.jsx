import { useState, useEffect } from 'react';
import { X, Tag as TagIcon, Plus, Hash } from 'lucide-react';
import { downloadAPI } from '../api/client';
import './TagManager.css';

const TagManager = ({ download, isOpen, onClose, onUpdate }) => {
  const [tags, setTags] = useState([]);
  const [allTags, setAllTags] = useState([]);
  const [selectedTagIds, setSelectedTagIds] = useState(new Set());
  const [newTagName, setNewTagName] = useState('');
  const [newTagColor, setNewTagColor] = useState('#3b82f6');
  const [newTagCategory, setNewTagCategory] = useState('');
  const [showCreateTag, setShowCreateTag] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isOpen && download?.id) {
      loadTags();
      loadAllTags();
    }
  }, [isOpen, download?.id]);

  useEffect(() => {
    if (download?.tags) {
      setSelectedTagIds(new Set(download.tags.map(t => t.id)));
      setTags(download.tags);
    }
  }, [download?.tags]);

  const loadTags = async () => {
    if (!download?.id) return;
    try {
      const downloadTags = await downloadAPI.getDownloadTags(download.id);
      setTags(downloadTags);
      setSelectedTagIds(new Set(downloadTags.map(t => t.id)));
    } catch (error) {
      console.error('Failed to load tags:', error);
    }
  };

  const loadAllTags = async () => {
    try {
      const all = await downloadAPI.getTags();
      setAllTags(all);
    } catch (error) {
      console.error('Failed to load all tags:', error);
    }
  };

  const handleToggleTag = async (tagId) => {
    if (!download?.id) return;
    
    setLoading(true);
    try {
      const newSelected = new Set(selectedTagIds);
      if (newSelected.has(tagId)) {
        newSelected.delete(tagId);
        await downloadAPI.removeTagFromDownload(download.id, tagId);
      } else {
        newSelected.add(tagId);
        await downloadAPI.addTagToDownload(download.id, tagId);
      }
      setSelectedTagIds(newSelected);
      await loadTags();
      if (onUpdate) {
        // Reload download with updated tags
        const updatedDownload = await downloadAPI.getDownload(download.id);
        onUpdate(updatedDownload);
      }
    } catch (error) {
      console.error('Failed to toggle tag:', error);
      alert('Erreur lors de la modification des tags : ' + (error.message || 'Erreur inconnue'));
    } finally {
      setLoading(false);
    }
  };

  const handleCreateTag = async () => {
    if (!newTagName.trim()) {
      alert('Veuillez saisir un nom de tag');
      return;
    }

    setLoading(true);
    try {
      const newTag = await downloadAPI.createTag({
        name: newTagName.trim(),
        color: newTagColor,
        category: newTagCategory.trim() || null,
      });
      
      setAllTags([...allTags, newTag]);
      setNewTagName('');
      setNewTagColor('#3b82f6');
      setNewTagCategory('');
      setShowCreateTag(false);
      
      // Automatically add to download
      if (download?.id) {
        await downloadAPI.addTagToDownload(download.id, newTag.id);
        await loadTags();
        if (onUpdate) {
          const updatedDownload = await downloadAPI.getDownload(download.id);
          onUpdate(updatedDownload);
        }
      }
    } catch (error) {
      console.error('Failed to create tag:', error);
      alert('Erreur lors de la création du tag : ' + (error.message || 'Erreur inconnue'));
    } finally {
      setLoading(false);
    }
  };

  const predefinedCategories = ['Musique', 'Films', 'Éducatif', 'Divertissement', 'Podcast', 'Documentaire', 'Tutoriel', 'Autre'];

  // Auto-suggest tags based on URL/metadata
  const suggestTags = (download) => {
    const suggestions = [];
    if (!download) return suggestions;

    const url = (download.url || '').toLowerCase();
    const title = (download.title || '').toLowerCase();
    const author = (download.author || '').toLowerCase();
    const type = (download.download_type || '').toLowerCase();

    // Platform-based suggestions
    if (url.includes('youtube.com') || url.includes('youtu.be')) {
      suggestions.push({ name: 'YouTube', category: 'Plateforme', color: '#ff0000' });
    } else if (url.includes('vimeo.com')) {
      suggestions.push({ name: 'Vimeo', category: 'Plateforme', color: '#1ab7ea' });
    } else if (url.includes('tiktok.com')) {
      suggestions.push({ name: 'TikTok', category: 'Plateforme', color: '#000000' });
    } else if (url.includes('soundcloud.com')) {
      suggestions.push({ name: 'SoundCloud', category: 'Plateforme', color: '#ff7700' });
    }

    // Type-based suggestions
    if (type === 'video') {
      suggestions.push({ name: 'Vidéo', category: 'Type', color: '#3b82f6' });
    } else if (type === 'audio' || type === 'instrumental') {
      suggestions.push({ name: 'Audio', category: 'Type', color: '#10b981' });
    }

    // Content-based suggestions (simple keyword matching)
    const content = `${title} ${author}`.toLowerCase();
    if (content.includes('musique') || content.includes('music') || content.includes('song')) {
      suggestions.push({ name: 'Musique', category: 'Contenu', color: '#8b5cf6' });
    }
    if (content.includes('tutoriel') || content.includes('tutorial') || content.includes('how to')) {
      suggestions.push({ name: 'Tutoriel', category: 'Contenu', color: '#f59e0b' });
    }
    if (content.includes('documentaire') || content.includes('documentary')) {
      suggestions.push({ name: 'Documentaire', category: 'Contenu', color: '#06b6d4' });
    }
    if (content.includes('podcast')) {
      suggestions.push({ name: 'Podcast', category: 'Contenu', color: '#ec4899' });
    }

    return suggestions;
  };

  const [suggestedTags, setSuggestedTags] = useState([]);

  useEffect(() => {
    if (isOpen && download && allTags.length > 0) {
      const suggestions = suggestTags(download);
      if (suggestions.length > 0) {
        // Filter out suggestions that already exist as tags
        const newSuggestions = suggestions.filter(s => 
          !allTags.some(t => t.name.toLowerCase() === s.name.toLowerCase())
        );
        setSuggestedTags(newSuggestions);
      } else {
        setSuggestedTags([]);
      }
    } else {
      setSuggestedTags([]);
    }
  }, [isOpen, download, allTags]);

  const handleCreateSuggestedTag = async (suggestion) => {
    setLoading(true);
    try {
      const newTag = await downloadAPI.createTag({
        name: suggestion.name,
        color: suggestion.color,
        category: suggestion.category || null,
      });
      
      setAllTags([...allTags, newTag]);
      setSuggestedTags(suggestedTags.filter(s => s.name !== suggestion.name));
      
      // Automatically add to download
      if (download?.id) {
        await downloadAPI.addTagToDownload(download.id, newTag.id);
        await loadTags();
        if (onUpdate) {
          const updatedDownload = await downloadAPI.getDownload(download.id);
          onUpdate(updatedDownload);
        }
      }
    } catch (error) {
      console.error('Failed to create suggested tag:', error);
      alert('Erreur lors de la création du tag suggéré : ' + (error.message || 'Erreur inconnue'));
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="tag-manager-overlay" onClick={onClose}>
      <div className="tag-manager" onClick={(e) => e.stopPropagation()}>
        <div className="tag-manager-header">
          <h3>Gérer les tags</h3>
          <button className="btn-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="tag-manager-content">
          {/* Current tags */}
          <div className="tag-section">
            <h4>Tags actuels</h4>
            {tags.length === 0 ? (
              <p className="tag-empty">Aucun tag assigné</p>
            ) : (
              <div className="tags-list">
                {tags.map(tag => (
                  <span
                    key={tag.id}
                    className="tag-badge active"
                    style={{
                      backgroundColor: tag.color || '#3b82f6',
                      color: '#fff',
                    }}
                  >
                    {tag.name}
                    {tag.category && <span className="tag-category"> ({tag.category})</span>}
                  </span>
                ))}
              </div>
            )}
          </div>

          {/* Suggested tags */}
          {suggestedTags.length > 0 && (
            <div className="tag-section tag-suggestions">
              <h4>Tags suggérés</h4>
              <p className="tag-suggestion-hint">Ces tags sont suggérés en fonction de votre téléchargement</p>
              <div className="tags-selector">
                {suggestedTags.map((suggestion, index) => (
                  <button
                    key={index}
                    className="tag-suggestion-button"
                    onClick={() => handleCreateSuggestedTag(suggestion)}
                    disabled={loading}
                    style={{
                      borderColor: suggestion.color || '#3b82f6',
                      backgroundColor: 'transparent',
                      color: suggestion.color || '#3b82f6',
                    }}
                  >
                    <Plus size={12} />
                    <TagIcon size={14} />
                    {suggestion.name}
                    {suggestion.category && <span className="tag-category"> ({suggestion.category})</span>}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Available tags */}
          <div className="tag-section">
            <h4>Tags disponibles</h4>
            {allTags.length === 0 ? (
              <p className="tag-empty">Aucun tag disponible. Créez-en un nouveau !</p>
            ) : (
              <div className="tags-selector">
                {allTags.map(tag => {
                  const isSelected = selectedTagIds.has(tag.id);
                  return (
                    <button
                      key={tag.id}
                      className={`tag-select-button ${isSelected ? 'selected' : ''}`}
                      onClick={() => handleToggleTag(tag.id)}
                      disabled={loading}
                      style={{
                        borderColor: tag.color || '#3b82f6',
                        backgroundColor: isSelected ? (tag.color || '#3b82f6') : 'transparent',
                        color: isSelected ? '#fff' : (tag.color || '#3b82f6'),
                      }}
                    >
                      <TagIcon size={14} />
                      {tag.name}
                      {tag.category && <span className="tag-category"> ({tag.category})</span>}
                    </button>
                  );
                })}
              </div>
            )}
          </div>

          {/* Create new tag */}
          {!showCreateTag ? (
            <button
              className="btn-create-tag"
              onClick={() => setShowCreateTag(true)}
            >
              <Plus size={16} />
              Créer un nouveau tag
            </button>
          ) : (
            <div className="create-tag-form">
              <h4>Créer un nouveau tag</h4>
              <div className="form-group">
                <label>Nom du tag *</label>
                <input
                  type="text"
                  value={newTagName}
                  onChange={(e) => setNewTagName(e.target.value)}
                  placeholder="Ex: Musique, Films, etc."
                  maxLength={50}
                />
              </div>
              <div className="form-group">
                <label>Couleur</label>
                <input
                  type="color"
                  value={newTagColor}
                  onChange={(e) => setNewTagColor(e.target.value)}
                />
              </div>
              <div className="form-group">
                <label>Catégorie (optionnel)</label>
                <select
                  value={newTagCategory}
                  onChange={(e) => setNewTagCategory(e.target.value)}
                >
                  <option value="">Aucune</option>
                  {predefinedCategories.map(cat => (
                    <option key={cat} value={cat}>{cat}</option>
                  ))}
                </select>
              </div>
              <div className="form-actions">
                <button
                  className="btn-primary"
                  onClick={handleCreateTag}
                  disabled={loading || !newTagName.trim()}
                >
                  Créer
                </button>
                <button
                  className="btn-secondary"
                  onClick={() => {
                    setShowCreateTag(false);
                    setNewTagName('');
                    setNewTagColor('#3b82f6');
                    setNewTagCategory('');
                  }}
                >
                  Annuler
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TagManager;

