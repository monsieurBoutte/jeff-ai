// src/Tiptap.tsx
import { EditorProvider, FloatingMenu, BubbleMenu } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';

// define your extension array
const extensions = [StarterKit];

const content = '<p>Hello World!</p>';

export const DocumentEditor = () => {
  return (
    <div className="w-full h-[calc(100vh-8rem)] backdrop-blur-sm bg-white/10 dark:bg-black/10 px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-800 shadow-sm">
      <EditorProvider
        extensions={extensions}
        content={content}
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
