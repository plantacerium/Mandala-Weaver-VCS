import React, { useState } from 'react';

interface AddProjectDialogProps {
  onAdd: (path: string, name: string) => void;
  onClose: () => void;
}

export const AddProjectDialog: React.FC<AddProjectDialogProps> = ({
  onAdd,
  onClose,
}) => {
  const [path, setPath] = useState('');
  const [name, setName] = useState('');
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (path) {
      onAdd(path, name || path.split(/[/\\]/).pop() || 'Unnamed');
    }
  };
  
  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h2>Add Project to Synarchy</h2>
        
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Project Path</label>
            <input
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="/path/to/project"
              autoFocus
            />
          </div>
          
          <div className="form-group">
            <label>Display Name (optional)</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Auto-detected"
            />
          </div>
          
          <div className="dialog-actions">
            <button type="button" onClick={onClose} className="btn-cancel">
              Cancel
            </button>
            <button type="submit" className="btn-primary" disabled={!path}>
              Add Project
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default AddProjectDialog;