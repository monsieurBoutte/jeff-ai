import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import {
  Bold,
  Italic,
  Strikethrough,
  Code,
  FileJson2,
  Heading1,
  Heading2,
  Heading3,
  List,
  ListOrdered,
  Undo,
  Redo,
  Settings2,
  WrapText,
  Quote
} from 'lucide-react';
import { useMachine } from '@xstate/react';
import { Editor } from '@tiptap/react';
import * as motion from 'motion/react-client';
import { AnimatePresence, useReducedMotion } from 'motion/react';

import { SystemOutputToggle } from '@/components/system-output-toggle';
import { modeMachine } from '@/machines/mode-machine';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { MicrophoneToggle } from '../microphone-toggle';

interface MenuBarProps {
  editor: Editor;
}

interface StyleButton {
  label: string;
  action: (editor: Editor) => boolean;
  isActive: (editor: Editor) => boolean;
  isDisabled?: (editor: Editor) => boolean;
  icon: React.ReactNode;
}

const styleButtons: StyleButton[] = [
  {
    label: 'Bold',
    action: (editor) => editor.chain().focus().toggleBold().run(),
    isActive: (editor) => editor.isActive('bold'),
    isDisabled: (editor) => !editor.can().chain().focus().toggleBold().run(),
    icon: <Bold className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'Italic',
    action: (editor) => editor.chain().focus().toggleItalic().run(),
    isActive: (editor) => editor.isActive('italic'),
    isDisabled: (editor) => !editor.can().chain().focus().toggleItalic().run(),
    icon: <Italic className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'Strike',
    action: (editor) => editor.chain().focus().toggleStrike().run(),
    isActive: (editor) => editor.isActive('strike'),
    isDisabled: (editor) => !editor.can().chain().focus().toggleStrike().run(),
    icon: (
      <Strikethrough className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
    )
  },
  {
    label: 'Code',
    action: (editor) => editor.chain().focus().toggleCode().run(),
    isActive: (editor) => editor.isActive('code'),
    isDisabled: (editor) => !editor.can().chain().focus().toggleCode().run(),
    icon: <Code className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'Code Block',
    action: (editor) => editor.chain().focus().toggleCodeBlock().run(),
    isActive: (editor) => editor.isActive('codeBlock'),
    isDisabled: (editor) =>
      !editor.can().chain().focus().toggleCodeBlock().run(),
    icon: <FileJson2 className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'Paragraph',
    action: (editor) => editor.chain().focus().setParagraph().run(),
    isActive: (editor) => editor.isActive('paragraph'),
    icon: <WrapText className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'H1',
    action: (editor) =>
      editor.chain().focus().toggleHeading({ level: 1 }).run(),
    isActive: (editor) => editor.isActive('heading', { level: 1 }),
    icon: <Heading1 className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'H2',
    action: (editor) =>
      editor.chain().focus().toggleHeading({ level: 2 }).run(),
    isActive: (editor) => editor.isActive('heading', { level: 2 }),
    icon: <Heading2 className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'H3',
    action: (editor) =>
      editor.chain().focus().toggleHeading({ level: 3 }).run(),
    isActive: (editor) => editor.isActive('heading', { level: 3 }),
    icon: <Heading3 className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'Bullet list',
    action: (editor) => editor.chain().focus().toggleBulletList().run(),
    isActive: (editor) => editor.isActive('bulletList'),
    icon: <List className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  },
  {
    label: 'Ordered list',
    action: (editor) => editor.chain().focus().toggleOrderedList().run(),
    isActive: (editor) => editor.isActive('orderedList'),
    icon: (
      <ListOrdered className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
    )
  },
  {
    label: 'Blockquote',
    action: (editor) => editor.chain().focus().toggleBlockquote().run(),
    isActive: (editor) => editor.isActive('blockquote'),
    icon: <Quote className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
  }
];

export const MenuBar = ({ editor }: MenuBarProps) => {
  const [state, send] = useMachine(modeMachine);
  const [isExpanded, setIsExpanded] = useState(false);
  const shouldReduceMotion = useReducedMotion();

  if (!editor) {
    return null;
  }

  const animations = {
    initial: shouldReduceMotion ? { opacity: 0 } : { opacity: 0, height: 0 },
    animate: shouldReduceMotion
      ? { opacity: 1 }
      : { opacity: 1, height: 'auto' },
    exit: shouldReduceMotion ? { opacity: 0 } : { opacity: 0, height: 0 },
    transition: { duration: shouldReduceMotion ? 0.1 : 0.2 }
  };

  const { isAuthenticated, getToken } = KindeAuth.useKindeAuth();

  const handleMicToggle = useCallback(async () => {
    if (!isAuthenticated || !getToken) {
      return;
    }
    const token = await getToken();

    send({ type: 'TOGGLE_RECORDER' });
    if (state.matches('recorder')) {
      invoke('stop_recording', { token, refine: false });
    } else {
      invoke('start_recording');
    }
  }, [send, state, isAuthenticated, getToken]);

  const handleSystemOutputToggle = useCallback(async () => {
    if (!isAuthenticated || !getToken) {
      return;
    }
    const token = await getToken();
    send({ type: 'TOGGLE_SYSTEM_OUTPUT' });
    if (state.matches('systemOutput')) {
      invoke('stop_recording_system_output', { token, refine: false });
    } else {
      invoke('start_recording_system_output');
    }
  }, [send, state, isAuthenticated, getToken]);

  return (
    <div className="flex flex-col mb-2 gap-2">
      <div className="flex gap-2 items-center justify-between">
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setIsExpanded(!isExpanded)}
          >
            <Settings2
              className="h-[1.2rem] w-[1.2rem]"
              color={'currentColor'}
            />
            <span className="sr-only">Styles</span>
          </Button>
          <MicrophoneToggle
            isActive={state.matches('recorder')}
            onClick={handleMicToggle}
          />
          <SystemOutputToggle
            onClick={handleSystemOutputToggle}
            isActive={state.matches('systemOutput')}
          />
        </div>

        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => editor.chain().focus().undo().run()}
            disabled={!editor.can().chain().focus().undo().run()}
            className={editor.isActive('undo') ? 'is-active' : ''}
          >
            <Undo className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
            <span className="sr-only">Undo</span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => editor.chain().focus().redo().run()}
            disabled={!editor.can().chain().focus().redo().run()}
          >
            <Redo className="h-[1.2rem] w-[1.2rem]" color={'currentColor'} />
            <span className="sr-only">Redo</span>
          </Button>
        </div>
      </div>

      <AnimatePresence>
        {isExpanded && (
          <motion.div {...animations} className="grid grid-cols-6 gap-2">
            {styleButtons.map((button) => (
              <Button
                variant="outline"
                size="sm"
                key={button.label}
                onClick={() => button.action(editor)}
                disabled={button.isDisabled?.(editor)}
                className={cn(button.isActive(editor) && 'text-indigo-600')}
              >
                {button.icon}
                <span className="sr-only">{button.label}</span>
              </Button>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};
