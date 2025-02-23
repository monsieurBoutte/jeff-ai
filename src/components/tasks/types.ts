export interface Task {
  id: number;
  content: string;
  completed: boolean;
  assignedDate: string;
  __tempId?: number;
  isNewTask?: boolean;
}

export interface DaySection {
  day: string;
  date?: string;
  temperature?: string;
}

export interface TranscriptionEvent {
  payload: string;
}

export interface ApiTask {
  id: string;
  task: string;
  done: boolean;
  assignedDate: string;
  createdAt?: string;
  updatedAt?: string;
}

export function transformApiTask(apiTask: ApiTask): Task {
  return {
    id: Number(apiTask.id),
    content: apiTask.task,
    completed: apiTask.done,
    assignedDate: apiTask.assignedDate || new Date().toISOString()
  };
}
