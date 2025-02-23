import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { useCallback, useEffect, useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '@/lib/utils';
import dayjs from 'dayjs';
import { AnimatePresence } from 'motion/react';
import { TaskEditor } from '@/components/task-editor';
import useSound from 'use-sound';
import recordSfx from '@/assets/cassette_tape_record.mp3';
import { listen } from '@tauri-apps/api/event';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { Button } from '@/components/ui/button';
import * as motion from 'motion/react-client';

interface Task {
  id: number;
  content: string;
  completed: boolean;
  assignedDate: string;
  // This is used to preserve the animation when the optimistic task is mounted
  __tempId?: number;
  isNewTask?: boolean;
}

interface DaySection {
  day: string;
  date?: string;
  temperature?: string;
}

interface TranscriptionEvent {
  payload: string;
}

interface ApiTask {
  id: string;
  task: string;
  done: boolean;
  assignedDate: string;
  createdAt?: string;
  updatedAt?: string;
}

function transformApiTask(apiTask: ApiTask): Task {
  return {
    id: Number(apiTask.id),
    content: apiTask.task,
    completed: apiTask.done,
    assignedDate: apiTask.assignedDate || new Date().toISOString()
  };
}

export default function Tasks() {
  const { isAuthenticated, getToken } = KindeAuth.useKindeAuth();

  const [selectedDay, setSelectedDay] = useState<string>('MONDAY');
  const [tasks, setTasks] = useState<Task[]>([]);
  const [play] = useSound(recordSfx);
  const [isRecording, setIsRecording] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [deletingTaskId, setDeletingTaskId] = useState<number | null>(null);
  const deleteTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const longPressTimeout = useRef<NodeJS.Timeout | null>(null);
  const [isHoveringDeleteArea, setIsHoveringDeleteArea] = useState(false);

  const [weekSections, setWeekSections] = useState<DaySection[]>([]);

  useEffect(() => {
    const generateWeekSections = () => {
      const today = dayjs();
      return ['MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY'].map(
        (day, index) => {
          const date = today
            .startOf('week')
            .add(index + 1, 'day')
            .format('MMMM D, YYYY');
          return {
            day,
            date,
            temperature: '' // todo: get temperature
          };
        }
      );
    };
    setWeekSections(generateWeekSections());
  }, []);

  const fetchTasks = useCallback(async () => {
    if (!isAuthenticated || !getToken) {
      return;
    }
    const token = await getToken();

    try {
      const response = await invoke<ApiTask[]>('fetch_tasks', { token });
      if (Array.isArray(response)) {
        const transformedTasks = response.map(transformApiTask);
        setTasks(transformedTasks);
      } else {
        console.error('Unexpected response format:', response);
      }
    } catch (error) {
      console.error('Error fetching tasks:', error);
    }
  }, [getToken, isAuthenticated]);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  useEffect(() => {
    const unlisten = listen(
      'transcription-complete',
      (event: TranscriptionEvent) => {
        const selectedSection = weekSections.find((s) => s.day === selectedDay);
        if (!selectedSection || !selectedSection.date) return;

        const newTask: Task = {
          id: Date.now(),
          content: event.payload,
          completed: false,
          assignedDate: dayjs(selectedSection.date).toISOString()
        };
        setTasks((prev) => [...prev, newTask]);
      }
    );

    return () => {
      unlisten.then((unlistenFn) => unlistenFn());
    };
  }, [selectedDay, weekSections]);

  const handleRecording = async () => {
    if (!isAuthenticated || !getToken) return;
    const token = await getToken();

    if (!isRecording) {
      play();
      await invoke('start_recording');
      setIsRecording(true);
    } else {
      play();
      setIsRecording(false);
      setIsProcessing(true);
      try {
        await invoke('stop_recording', { token, refine: true });
      } finally {
        setIsProcessing(false);
      }
    }
  };

  const handleAddTask = async (content: string, isNewTask?: boolean) => {
    if (!content.trim() || !isAuthenticated || !getToken) return;

    const tempId = Date.now();
    const optimisticTask: Task = {
      id: tempId,
      content,
      completed: false,
      assignedDate: dayjs(
        weekSections.find((s) => s.day === selectedDay)?.date
      ).toISOString(),
      __tempId: tempId,
      isNewTask
    };

    // Optimistically update state
    setTasks((prev) => [...prev, optimisticTask]);

    try {
      const token = await getToken();
      const response = await invoke<ApiTask>('create_task', {
        token,
        content,
        assignedDate: optimisticTask.assignedDate
      });

      // Replace optimistic task with server response
      setTasks((prev) =>
        prev.map((task) =>
          task.__tempId === tempId ? transformApiTask(response) : task
        )
      );
    } catch (error) {
      console.error('Error creating task:', error);
      // Rollback optimistic update if there's an error
      setTasks((prev) => prev.filter((task) => task.__tempId !== tempId));
    }
  };

  const handleToggleTask = async (id: number) => {
    const task = tasks.find((t) => t.id === id);
    if (!task || !isAuthenticated || !getToken) return;

    // Optimistically update the local state
    setTasks((prev) =>
      prev.map((task) =>
        task.id === id ? { ...task, completed: !task.completed } : task
      )
    );

    try {
      const token = await getToken();
      await invoke('update_task', {
        token,
        taskId: id,
        completed: !task.completed
      });
    } catch (error) {
      console.error('Error updating task:', error);
      // Rollback optimistic update if there's an error
      setTasks((prev) =>
        prev.map((task) =>
          task.id === id ? { ...task, completed: task.completed } : task
        )
      );
    }
  };

  // Filter tasks for the selected day
  const tasksForSelectedDay = tasks.filter((t) =>
    dayjs(t.assignedDate).isSame(
      weekSections.find((s) => s.day === selectedDay)?.date,
      'day'
    )
  );

  const handleKeyDown = (e: React.KeyboardEvent, day: string) => {
    // Only prevent default and handle the key press if the target is a day section
    const isDaySection =
      (e.target as HTMLElement).getAttribute('role') === 'button';

    if (isDaySection && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      setSelectedDay(day);
    }
  };

  const handleLongPressStart = (taskId: number) => {
    longPressTimeout.current = setTimeout(() => {
      setDeletingTaskId(taskId);
    }, 500);
  };

  const handleLongPressEnd = () => {
    if (longPressTimeout.current) {
      clearTimeout(longPressTimeout.current);
      longPressTimeout.current = null;
    }
  };

  const handleDeleteTask = async (taskId: number) => {
    if (!isAuthenticated || !getToken) return;

    // Optimistically remove the task from state
    setTasks((prev) => prev.filter((task) => task.id !== taskId));

    try {
      const token = await getToken();
      const result = await invoke('delete_task', {
        token,
        taskId
      });
      console.log('Delete task result:', result);
    } catch (error) {
      console.error('Error deleting task:', error);
      // Rollback optimistic update if there's an error
      fetchTasks(); // Refetch tasks to restore the deleted one
    } finally {
      setDeletingTaskId(null);
    }
  };

  useEffect(() => {
    if (deletingTaskId && !isHoveringDeleteArea) {
      // Start 2s timer only if we're not hovering
      const timer = setTimeout(() => {
        setDeletingTaskId(null);
      }, 2000);
      deleteTimeoutRef.current = timer;

      return () => {
        if (deleteTimeoutRef.current) {
          clearTimeout(deleteTimeoutRef.current);
        }
      };
    }
  }, [deletingTaskId, isHoveringDeleteArea]);

  return (
    <div className="max-w-2xl mx-auto">
      <div className="space-y-0">
        {weekSections.map((section, index) => (
          <div
            key={section.day}
            className={cn(
              'p-4 rounded-none transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-orange-500 focus-visible:ring-offset-2',
              index === 0 && 'rounded-t-lg',
              index === 4 && 'rounded-b-lg',
              selectedDay === section.day
                ? 'bg-gray-100 dark:bg-gray-800/50'
                : cn(
                    'hover:bg-gray-100 dark:hover:bg-gray-800/50 cursor-pointer',
                    index === 0 && 'bg-gray-50 dark:bg-gray-900/30',
                    index === 1 && 'bg-gray-100 dark:bg-gray-900/40',
                    index === 2 && 'bg-gray-200 dark:bg-gray-900/50',
                    index === 3 && 'bg-gray-300 dark:bg-gray-900/60',
                    index === 4 && 'bg-gray-400 dark:bg-gray-900/70'
                  )
            )}
            onClick={() => setSelectedDay(section.day)}
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
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {section.date}
                  </p>
                )}
              </div>

              {selectedDay === section.day && (
                <MicrophoneToggle
                  isActive={isRecording}
                  isProcessing={isProcessing}
                  onClick={() => handleRecording()}
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
                  {/* Display tasks for this day */}
                  <div className="space-y-2">
                    {tasksForSelectedDay.map((task) => (
                      <div key={task.id} className="flex items-start space-x-2">
                        <div className="flex-shrink-0 pt-1">
                          <input
                            type="checkbox"
                            className="task-checkbox"
                            checked={task.completed}
                            onChange={() => handleToggleTask(task.id)}
                          />
                        </div>
                        <div className="flex-grow relative">
                          <div
                            className={cn(
                              'flex-grow text-gray-900 dark:text-gray-100 text-base select-none',
                              task.completed &&
                                'line-through text-gray-400 dark:text-gray-600 decoration-[#ea580c]',
                              deletingTaskId === task.id &&
                                'text-red-500 dark:text-red-500'
                            )}
                            onTouchStart={() => handleLongPressStart(task.id)}
                            onTouchEnd={handleLongPressEnd}
                            onMouseDown={() => handleLongPressStart(task.id)}
                            onMouseUp={handleLongPressEnd}
                            onMouseEnter={() => setIsHoveringDeleteArea(true)}
                            onMouseLeave={() => setIsHoveringDeleteArea(false)}
                          >
                            {task.content}
                          </div>
                          {deletingTaskId === task.id && (
                            <AnimatePresence>
                              <button
                                className="absolute right-0 top-1/2 -translate-y-1/2 text-red-500 hover:text-red-700 ml-2 bg-gray-50 dark:bg-gray-800 rounded-full p-1 shadow-md"
                                onClick={() => handleDeleteTask(task.id)}
                                onMouseEnter={() =>
                                  setIsHoveringDeleteArea(true)
                                }
                                onMouseLeave={() =>
                                  setIsHoveringDeleteArea(false)
                                }
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
                    ))}
                  </div>

                  {/* Task input area */}
                  <div className="mt-4">
                    <TaskEditor
                      onSubmit={async (content) => {
                        await handleAddTask(content);
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

                          await handleAddTask(editor.textContent.trim());
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
        ))}
      </div>
    </div>
  );
}
