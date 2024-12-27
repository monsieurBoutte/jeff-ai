import { EditorProvider, FloatingMenu, BubbleMenu } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';

// define your extension array
const extensions = [StarterKit];

interface DocumentEditorProps {
  content: string;
}

export const DocumentEditor = ({ content }: DocumentEditorProps) => {
  console.log('incoming content', { content });
  return (
    <div className="w-full h-[calc(100vh-8rem)] backdrop-blur-sm bg-white/10 dark:bg-black/10 px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-800 shadow-sm">
      <EditorProvider
        extensions={extensions}
        content={content !== '' ? content : '<p>Edit your document here</p>'}
        editorProps={{
          attributes: {
            class:
              'h-full w-full outline-none prose prose-sm max-w-none dark:prose-invert'
          }
        }}
      >
        <FloatingMenu editor={null}>This is the floating menu</FloatingMenu>
        <BubbleMenu editor={null}>This is the bubble menu</BubbleMenu>
      </EditorProvider>
    </div>
  );
};
