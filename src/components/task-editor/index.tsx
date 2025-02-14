import Document from '@tiptap/extension-document';
import Paragraph from '@tiptap/extension-paragraph';
import Text from '@tiptap/extension-text';
import History from '@tiptap/extension-history';
import { EditorContent, useEditor, BubbleMenu } from '@tiptap/react';
import Placeholder from '@tiptap/extension-placeholder';
import { Button } from '@/components/ui/button';
import { Plus } from 'lucide-react';
import { cn } from '@/lib/utils';
import './styles.css';

interface TaskEditorProps {
  onSubmit: (content: string, isNewTask?: boolean) => void;
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
        onSubmit(content, true);
        editor.commands.clearContent();
      }
    }
  };

  if (!editor) {
    return null;
  }

  return (
    <div onKeyDown={handleKeyDown}>
      {editor && (
        <BubbleMenu editor={editor} tippyOptions={{ duration: 100 }}>
          <div className="flex gap-2 backdrop-blur-sm bg-white/70 dark:bg-gray-800/70 p-1 rounded-md shadow-lg border border-white/20 dark:border-gray-700/30">
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                const selection = editor.state.selection;
                if (!selection.empty) {
                  const selectedText = editor.state.doc.textBetween(
                    selection.from,
                    selection.to
                  );
                  if (selectedText.trim()) {
                    onSubmit(selectedText.trim(), true);
                  }
                }
              }}
              className={cn('hover:bg-gray-200 dark:hover:bg-gray-700')}
            >
              <Plus className="h-[1.2rem] w-[1.2rem] mr-1" />
              <span>Add as task</span>
            </Button>
          </div>
        </BubbleMenu>
      )}
      <EditorContent editor={editor} />
    </div>
  );
};
