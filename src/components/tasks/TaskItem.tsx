import { cn } from '@/lib/utils';
import { Task } from './types';
import { AnimatePresence } from 'motion/react';

interface TaskItemProps {
  task: Task;
  deletingTaskId: number | null;
  onToggle: (id: number) => void;
  onDelete: (id: number) => void;
  onLongPressStart: (id: number) => void;
  onLongPressEnd: () => void;
  onHoverDeleteArea: (isHovering: boolean) => void;
}

export function TaskItem({
  task,
  deletingTaskId,
  onToggle,
  onDelete,
  onLongPressStart,
  onLongPressEnd,
  onHoverDeleteArea
}: TaskItemProps) {
  return (
    <div className="flex items-start space-x-2">
      <div className="flex-shrink-0 pt-1">
        <input
          type="checkbox"
          className="task-checkbox"
          checked={task.completed}
          onChange={() => onToggle(task.id)}
        />
      </div>
      <div className="flex-grow relative">
        <div
          className={cn(
            'flex-grow text-gray-900 dark:text-gray-100 text-base select-none',
            task.completed &&
              'line-through text-gray-400 dark:text-gray-600 decoration-[#ea580c]',
            deletingTaskId === task.id && 'text-red-500 dark:text-red-500'
          )}
          onTouchStart={() => onLongPressStart(task.id)}
          onTouchEnd={onLongPressEnd}
          onMouseDown={() => onLongPressStart(task.id)}
          onMouseUp={onLongPressEnd}
          onMouseEnter={() => onHoverDeleteArea(true)}
          onMouseLeave={() => onHoverDeleteArea(false)}
        >
          {task.content}
        </div>
        {deletingTaskId === task.id && (
          <AnimatePresence>
            <button
              className="absolute right-0 top-1/2 -translate-y-1/2 text-red-500 hover:text-red-700 ml-2 bg-gray-50 dark:bg-gray-800 rounded-full p-1 shadow-md"
              onClick={() => onDelete(task.id)}
              onMouseEnter={() => onHoverDeleteArea(true)}
              onMouseLeave={() => onHoverDeleteArea(false)}
              aria-label="Delete task"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" />
              </svg>
            </button>
          </AnimatePresence>
        )}
      </div>
    </div>
  );
}
