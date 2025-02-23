import { cn } from '@/lib/utils';
import { DaySection } from './types';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { AnimatePresence } from 'motion/react';
import * as motion from 'motion/react-client';
import { TaskEditor } from '@/components/task-editor';
import { Button } from '@/components/ui/button';

interface WeekSectionProps {
  section: DaySection;
  index: number;
  selectedDay: string;
  isRecording: boolean;
  isProcessing: boolean;
  onDaySelect: (day: string) => void;
  onRecording: () => void;
  onAddTask: (content: string) => Promise<void>;
  children?: React.ReactNode;
}

export function WeekSection({
  section,
  index,
  selectedDay,
  isRecording,
  isProcessing,
  onDaySelect,
  onRecording,
  onAddTask,
  children
}: WeekSectionProps) {
  const handleKeyDown = (e: React.KeyboardEvent, day: string) => {
    const isDaySection =
      (e.target as HTMLElement).getAttribute('role') === 'button';
    if (isDaySection && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      onDaySelect(day);
    }
  };

  return (
    <div
      className={cn(
        'p-4 cursor-default rounded-none transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-orange-500 focus-visible:ring-offset-2',
        index === 0 && 'rounded-t-lg',
        index === 4 && 'rounded-b-lg',
        selectedDay === section.day
          ? 'bg-gray-100 dark:bg-gray-800/50'
          : cn(
              'hover:bg-gray-100 dark:hover:bg-gray-800/50',
              index === 0 && 'bg-gray-50 dark:bg-gray-900/30',
              index === 1 && 'bg-gray-100 dark:bg-gray-900/40',
              index === 2 && 'bg-gray-200 dark:bg-gray-900/50',
              index === 3 && 'bg-gray-300 dark:bg-gray-900/60',
              index === 4 && 'bg-gray-400 dark:bg-gray-900/70'
            )
      )}
      onClick={() => onDaySelect(section.day)}
      onKeyDown={(e) => handleKeyDown(e, section.day)}
      tabIndex={0}
      role="button"
      aria-pressed={selectedDay === section.day}
    >
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2
            className={cn(
              'text-xl font-extrabold text-gray-700 dark:text-gray-300',
              selectedDay === section.day &&
                'text-orange-500 dark:text-orange-500'
            )}
          >
            {section.day}
          </h2>
          {section.date && (
            <p className="text-sm text-gray-500 dark:text-gray-400 flex items-center gap-2">
              <span>{section.date}</span>
              {section.temperature && (
                <span className="text-xs text-gray-600 dark:text-gray-500">
                  {section.temperature}
                </span>
              )}
            </p>
          )}
        </div>

        {selectedDay === section.day && (
          <MicrophoneToggle
            isActive={isRecording}
            isProcessing={isProcessing}
            onClick={onRecording}
          />
        )}
      </div>

      <AnimatePresence mode="wait">
        {selectedDay === section.day && (
          <motion.div
            className="mt-4"
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{
              duration: 0.2,
              ease: 'easeInOut'
            }}
          >
            {children}

            <div className="mt-4">
              <TaskEditor
                onSubmit={async (content) => {
                  await onAddTask(content);
                }}
              />
              <div className="flex justify-end mt-2">
                <Button
                  variant="outline"
                  onClick={async () => {
                    const editor = document.querySelector(
                      '[contenteditable="true"]'
                    ) as HTMLElement;

                    if (!editor?.textContent?.trim()) return;

                    await onAddTask(editor.textContent.trim());
                    editor.textContent = '';
                  }}
                  aria-label="Add task"
                >
                  Add Task
                </Button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
