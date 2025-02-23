import { Task } from './types';
import { TaskItem } from './TaskItem';

interface TaskListProps {
  tasks: Task[];
  deletingTaskId: number | null;
  onToggleTask: (id: number) => void;
  onDeleteTask: (id: number) => void;
  onLongPressStart: (id: number) => void;
  onLongPressEnd: () => void;
  onHoverDeleteArea: (isHovering: boolean) => void;
}

export function TaskList({
  tasks,
  deletingTaskId,
  onToggleTask,
  onDeleteTask,
  onLongPressStart,
  onLongPressEnd,
  onHoverDeleteArea
}: TaskListProps) {
  return (
    <div className="space-y-2">
      {tasks.map((task) => (
        <TaskItem
          key={task.id}
          task={task}
          deletingTaskId={deletingTaskId}
          onToggle={onToggleTask}
          onDelete={onDeleteTask}
          onLongPressStart={onLongPressStart}
          onLongPressEnd={onLongPressEnd}
          onHoverDeleteArea={onHoverDeleteArea}
        />
      ))}
    </div>
  );
}
