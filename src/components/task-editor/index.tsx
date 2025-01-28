import Document from '@tiptap/extension-document';
import Paragraph from '@tiptap/extension-paragraph';
import Text from '@tiptap/extension-text';
import History from '@tiptap/extension-history';
import { EditorContent, useEditor } from '@tiptap/react';
import Placeholder from '@tiptap/extension-placeholder';
import './styles.css';

interface TaskEditorProps {
  onSubmit: (content: string) => void;
  onChange?: (content: string) => void;
}

export const TaskEditor = ({ onSubmit, onChange }: TaskEditorProps) => {
  const editor = useEditor({
    extensions: [
      Document,
      Paragraph,
      Text,
      History,
      Placeholder.configure({
        placeholder: 'Add a new task...'
      })
    ],
    editorProps: {
      attributes: {
        class:
          'min-h-[100px] backdrop-blur-sm bg-white/10 dark:bg-black/10 px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-800 shadow-sm focus:outline-none'
      }
    },
    onUpdate: ({ editor }) => {
      onChange?.(editor.getText());
    }
  });

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey && editor) {
      e.preventDefault();
      const content = editor.getText().trim();
      if (content) {
        onSubmit(content);
        editor.commands.clearContent();
      }
    }
  };

  if (!editor) {
    return null;
  }

  return (
    <div onKeyDown={handleKeyDown}>
      <EditorContent editor={editor} />
    </div>
  );
};
