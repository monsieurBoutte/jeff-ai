import { useEffect, useCallback } from 'react';
import { useMachine } from '@xstate/react';
import { invoke } from '@tauri-apps/api/core';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import {
  Copy,
  Layers2,
  Bold as BoldIcon,
  Italic as ItalicIcon,
  PenLine,
  Strikethrough
} from 'lucide-react';
import Bold from '@tiptap/extension-bold';
import Blockquote from '@tiptap/extension-blockquote';
import BulletList from '@tiptap/extension-bullet-list';
import OrderedList from '@tiptap/extension-ordered-list';
import Document from '@tiptap/extension-document';
import Italic from '@tiptap/extension-italic';
import ListItem from '@tiptap/extension-list-item';
import Paragraph from '@tiptap/extension-paragraph';
import Strike from '@tiptap/extension-strike';
import Highlight from '@tiptap/extension-highlight';
import Heading from '@tiptap/extension-heading';
import HardBreak from '@tiptap/extension-hard-break';
import History from '@tiptap/extension-history';
import Placeholder from '@tiptap/extension-placeholder';
import Code from '@tiptap/extension-code';
import CodeBlock from '@tiptap/extension-code-block';
import Text from '@tiptap/extension-text';
import {
  EditorContent,
  BubbleMenu,
  FloatingMenu,
  useEditor
} from '@tiptap/react';
import * as motion from 'motion/react-client';
import { AnimatePresence, useReducedMotion } from 'motion/react';

import { MenuBar } from '@/components/document-editor/menu-bar';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { Button } from '@/components/ui/button';
import { useToast } from '@/hooks/use-toast';
import { modeMachine } from '@/machines/mode-machine';
import { cn } from '@/lib/utils';
import './styles.css';
import { MarkdownResponse, RefineTextResponse } from '@/types/commands';

interface DocumentEditorProps {
  content: string;
}

export const DocumentEditor = ({ content }: DocumentEditorProps) => {
  const [state, send] = useMachine(modeMachine);
  const { toast } = useToast();
  const editor = useEditor({
    editorProps: {
      attributes: {
        class:
          'backdrop-blur-sm bg-white/10 dark:bg-black/10 px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-800 shadow-sm'
      }
    },
    extensions: [
      Document,
      Paragraph,
      Text,
      Bold,
      Blockquote.configure({
        HTMLAttributes: {
          class: 'dark:bg-gray-800 bg-gray-100 p-2 rounded-tl-md rounded-bl-md'
        }
      }),
      Italic,
      Strike,
      Highlight,
      Heading.configure({
        levels: [1, 2, 3]
      }),
      HardBreak,
      Code,
      CodeBlock.configure({
        languageClassPrefix: 'language-',
        HTMLAttributes: {
          class: 'p-2 rounded-md'
        }
      }),
      BulletList,
      OrderedList,
      ListItem,
      Placeholder.configure({
        placeholder: 'Type something...'
      }),
      History
    ]
  });

  const { isAuthenticated, getToken } = KindeAuth.useKindeAuth();

  useEffect(() => {
    if (content !== '' && editor) {
      if (editor.isEmpty) {
        editor.commands.setContent(content);
      } else {
        editor.commands.insertContent(content);
      }
    }
  }, [content, editor]);

  if (!editor) {
    return null;
  }

  const handleMicToggle = useCallback(() => {
    send({ type: 'TOGGLE_RECORDER' });
    if (state.matches('recorder')) {
      invoke('stop_recording');
    } else {
      invoke('start_recording');
    }
  }, [send, state]);

  const shouldReduceMotion = useReducedMotion();

  const animations = {
    initial: shouldReduceMotion
      ? { opacity: 0 }
      : { opacity: 0, y: -20, height: 0 },
    animate: shouldReduceMotion
      ? { opacity: 1 }
      : { opacity: 1, y: 0, height: 'auto' },
    exit: shouldReduceMotion
      ? { opacity: 0 }
      : { opacity: 0, y: -20, height: 0 },
    transition: {
      duration: shouldReduceMotion ? 0.1 : 0.2,
      ease: 'easeInOut'
    }
  };

  const handleRewriteSubmission = useCallback(
    async (text: string, context: string) => {
      if (!isAuthenticated || !getToken) {
        return;
      }
      const token = await getToken();

      const response = await invoke<RefineTextResponse>('refine_text', {
        token,
        text,
        context: context !== '' ? context : undefined
      });
      editor.commands.insertContent(response.refinedText);

      if (response?.explanation)
        toast({
          description: response.explanation
        });
    },
    [editor, isAuthenticated, getToken]
  );

  const handleCopyMarkdown = useCallback(
    async (html: string) => {
      if (!isAuthenticated || !getToken) {
        return;
      }
      const token = await getToken();

      const response = await invoke<MarkdownResponse>('convert_to_markdown', {
        token,
        html
      });

      writeText(response.markdown);

      toast({
        description: 'markdown copied to clipboard'
      });
    },
    [editor, isAuthenticated, getToken]
  );

  const hasNonParagraphContent = useCallback(() => {
    if (!editor) return false;
    const doc = editor.state.doc;
    let hasNonParagraph = false;

    doc.descendants((node) => {
      if (node.type.name !== 'paragraph' && node.type.name !== 'text') {
        hasNonParagraph = true;
        return false; // Stop traversal
      }
      return true;
    });

    return hasNonParagraph;
  }, [editor]);

  return (
    <div>
      <MenuBar editor={editor} />
      {editor && (
        <BubbleMenu editor={editor} tippyOptions={{ duration: 100 }}>
          <div className="flex gap-2 backdrop-blur-sm bg-white/70 dark:bg-gray-800/70 p-1 rounded-md shadow-lg border border-white/20 dark:border-gray-700/30">
            <Button
              variant="outline"
              size="sm"
              onClick={() => editor.chain().focus().toggleBold().run()}
              className={cn(
                'hover:bg-gray-200 dark:hover:bg-gray-700',
                editor.isActive('bold') && 'text-indigo-600'
              )}
            >
              <BoldIcon
                className="h-[1.2rem] w-[1.2rem]"
                color={'currentColor'}
              />
              <span className="sr-only">Bold</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => editor.chain().focus().toggleItalic().run()}
              className={cn(
                'hover:bg-gray-200 dark:hover:bg-gray-700',
                editor.isActive('italic') && 'text-indigo-600'
              )}
            >
              <ItalicIcon
                className="h-[1.2rem] w-[1.2rem]"
                color={'currentColor'}
              />
              <span className="sr-only">Italic</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => editor.chain().focus().toggleStrike().run()}
              className={cn(
                'hover:bg-gray-200 dark:hover:bg-gray-700',
                editor.isActive('strike') && 'text-indigo-600'
              )}
            >
              <Strikethrough
                className="h-[1.2rem] w-[1.2rem]"
                color={'currentColor'}
              />
              <span className="sr-only">Strike</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={async () => {
                const selection = editor.state.selection;
                if (!selection.empty) {
                  const selectedText = editor.state.doc.textBetween(
                    selection.from,
                    selection.to
                  );
                  const additionalContext = editor.getText();
                  await handleRewriteSubmission(
                    selectedText,
                    additionalContext
                  );
                }
              }}
              className={cn('hover:bg-gray-200 dark:hover:bg-gray-700')}
            >
              <PenLine className="h-[1.2rem] w-[1.2rem]" />
              <span>Refine</span>
            </Button>
          </div>
        </BubbleMenu>
      )}
      {editor && (
        <FloatingMenu
          editor={editor}
          tippyOptions={{ duration: 100 }}
          shouldShow={({ editor }) => {
            if (!editor.isEditable || !editor.isFocused || editor.isEmpty) {
              return false;
            }

            const { selection } = editor.state;
            const lineText = editor.state.doc.textBetween(
              selection.$head.start(),
              selection.$head.end()
            );

            // Check if the line contains any non-whitespace characters
            const hasWords = /\S/.test(lineText);

            return (
              selection.empty && selection.$head.parentOffset === 0 && !hasWords
            );
          }}
        >
          <div className="flex gap-2">
            <MicrophoneToggle
              onClick={handleMicToggle}
              isActive={state.matches('recorder')}
            />
          </div>
        </FloatingMenu>
      )}
      <EditorContent editor={editor} />
      <div className="flex gap-2 justify-end my-2">
        <AnimatePresence>
          {editor && hasNonParagraphContent() && (
            <motion.div {...animations}>
              <Button
                variant="outline"
                size="sm"
                onClick={async () => {
                  const html = editor.getHTML();
                  await handleCopyMarkdown(html);
                }}
              >
                <Layers2 className="h-[1.2rem] w-[1.2rem]" />
                Copy markdown
              </Button>
            </motion.div>
          )}
        </AnimatePresence>
        <AnimatePresence>
          {editor && !editor.isEmpty && (
            <motion.div {...animations}>
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  const plainText = editor.getText();
                  const cleanText = plainText.replace(/\n{3,}/g, '\n\n').trim();
                  writeText(cleanText);
                  toast({
                    description: 'plain text copied to clipboard'
                  });
                }}
              >
                <Copy className="h-[1.2rem] w-[1.2rem]" />
                Copy plain text
              </Button>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};
