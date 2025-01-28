import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '@/lib/utils';
import { AnimatePresence, useReducedMotion } from 'motion/react';
import * as motion from 'motion/react-client';
import { TaskEditor } from '@/components/task-editor';
import useSound from 'use-sound';
import recordSfx from '@/assets/cassette_tape_record.mp3';
import { listen } from '@tauri-apps/api/event';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { Button } from '@/components/ui/button';

interface Task {
  id: string;
  day: string;
  content: string;
  completed: boolean;
}

interface DaySection {
  day: string;
  date?: string;
  temperature?: string;
}

interface TranscriptionEvent {
  payload: string;
}

export default function Tasks() {
  const { isAuthenticated, getToken } = KindeAuth.useKindeAuth();

  const [selectedDay, setSelectedDay] = useState<string>('MONDAY');
  const [tasks, setTasks] = useState<Task[]>([]);
  // Keep track of the editor content for the new task
  const [newTaskContent, setNewTaskContent] = useState<string>('');
  const [play] = useSound(recordSfx);
  const [isRecording, setIsRecording] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const shouldReduceMotion = useReducedMotion();

  // Animation configurations
  const containerAnimations = {
    initial: shouldReduceMotion
      ? { opacity: 0 }
      : { opacity: 0, height: 0, scale: 0.95 },
    animate: shouldReduceMotion
      ? { opacity: 1 }
      : { opacity: 1, height: 'auto', scale: 1 },
    exit: shouldReduceMotion
      ? { opacity: 0 }
      : { opacity: 0, height: 0, scale: 0.95 },
    transition: {
      duration: shouldReduceMotion ? 0.1 : 0.3,
      ease: 'easeInOut'
    }
  };

  // Example: your day "sections" array
  const weekSections: DaySection[] = [
    { day: 'MONDAY', date: 'April, 14 2025', temperature: '64°' },
    { day: 'TUESDAY' },
    { day: 'WEDNESDAY' },
    { day: 'THURSDAY' },
    { day: 'FRIDAY' }
  ];

  const fetchTasks = useCallback(async () => {
    if (!isAuthenticated || !getToken) {
      return;
    }
    const token = await getToken();

    const response = await invoke('fetch_tasks', {
      token
    });
    console.log('response', response);

    // If your backend returns tasks with structure similar to:
    //   [{ id: '1', day: 'MONDAY', content: 'Some task', completed: false }, ...]
    // then just do:
    // setTasks(response as Task[]);
  }, [getToken, isAuthenticated]);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  // Add transcription listener
  useEffect(() => {
    const unlisten = listen(
      'transcription-complete',
      (event: TranscriptionEvent) => {
        // Create a new task with the transcribed text
        const newTask: Task = {
          id: crypto.randomUUID(),
          day: selectedDay,
          content: event.payload,
          completed: false
        };
        setTasks((prev) => [...prev, newTask]);
      }
    );

    return () => {
      unlisten.then((unlistenFn) => unlistenFn());
    };
  }, [selectedDay]);

  // Handle recording
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

  // Handler to add a new task
  const handleAddTask = () => {
    if (!newTaskContent.trim()) return;

    const newTask: Task = {
      id: crypto.randomUUID(), // or any ID generator you prefer
      day: selectedDay,
      content: newTaskContent,
      completed: false
    };

    setTasks((prev) => [...prev, newTask]);
    setNewTaskContent(''); // clear the input after adding
  };

  // Handler to toggle "completed" state of a task
  const handleToggleTask = (id: string) => {
    setTasks((prev) =>
      prev.map((task) =>
        task.id === id ? { ...task, completed: !task.completed } : task
      )
    );
  };

  // Filter tasks for the selected day
  const tasksForSelectedDay = tasks.filter((t) => t.day === selectedDay);

  const handleKeyDown = (e: React.KeyboardEvent, day: string) => {
    // Only prevent default and handle the key press if the target is a day section
    const isDaySection =
      (e.target as HTMLElement).getAttribute('role') === 'button';

    if (isDaySection && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      setSelectedDay(day);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="space-y-0">
        {weekSections.map((section, index) => (
          <motion.div
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
            layout
            transition={{ duration: 0.3 }}
          >
            <motion.div
              className="flex justify-between items-center"
              layout="position"
            >
              <div className="space-y-1">
                <motion.h2
                  className={cn(
                    'text-xl font-extrabold text-gray-700 dark:text-gray-300',
                    selectedDay === section.day &&
                      'text-orange-500 dark:text-orange-500'
                  )}
                  layout="position"
                >
                  {section.day}
                </motion.h2>
                {section.date && (
                  <motion.p
                    className="text-sm text-gray-500 dark:text-gray-400"
                    layout="position"
                  >
                    {section.date} — {section.temperature}
                  </motion.p>
                )}
              </div>

              {/* Add microphone button */}
              {selectedDay === section.day && (
                <MicrophoneToggle
                  isActive={isRecording}
                  isProcessing={isProcessing}
                  onClick={() => handleRecording()}
                />
              )}
            </motion.div>

            <AnimatePresence mode="wait">
              {selectedDay === section.day && (
                <motion.div className="mt-4" {...containerAnimations} layout>
                  {/* Display tasks for this day */}
                  <motion.div
                    className="space-y-2"
                    initial="hidden"
                    animate="visible"
                    variants={{
                      visible: {
                        transition: {
                          staggerChildren: 0.1
                        }
                      }
                    }}
                  >
                    {tasksForSelectedDay.map((task) => (
                      <motion.div
                        key={task.id}
                        className="flex items-start space-x-2"
                        variants={{
                          hidden: { opacity: 0, y: -20 },
                          visible: { opacity: 1, y: 0 }
                        }}
                        transition={{
                          duration: 0.3,
                          type: 'spring',
                          stiffness: 500,
                          damping: 25
                        }}
                      >
                        <div className="flex-shrink-0 pt-1">
                          <input
                            type="checkbox"
                            className="task-checkbox"
                            checked={task.completed}
                            onChange={() => handleToggleTask(task.id)}
                          />
                        </div>
                        <span
                          className={cn(
                            'flex-grow text-gray-900 dark:text-gray-100 text-base',
                            task.completed &&
                              'line-through text-gray-400 dark:text-gray-600 decoration-[#ea580c]'
                          )}
                        >
                          {task.content}
                        </span>
                      </motion.div>
                    ))}
                  </motion.div>

                  {/* Task input area */}
                  <motion.div
                    className="mt-4"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 0.5 }}
                  >
                    <TaskEditor
                      onSubmit={(content) => {
                        const newTask: Task = {
                          id: crypto.randomUUID(),
                          day: selectedDay,
                          content: content,
                          completed: false
                        };
                        setTasks((prev) => [...prev, newTask]);
                      }}
                    />
                    <motion.div
                      className="flex justify-end mt-2"
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      transition={{ delay: 0.6 }}
                    >
                      <Button
                        variant="outline"
                        onClick={() => {
                          // Trigger the same submit handler as the TaskEditor
                          const editor = document.querySelector(
                            '[contenteditable="true"]'
                          );
                          if (editor && editor.textContent?.trim()) {
                            const newTask: Task = {
                              id: crypto.randomUUID(),
                              day: selectedDay,
                              content: editor.textContent.trim(),
                              completed: false
                            };
                            setTasks((prev) => [...prev, newTask]);
                            editor.textContent = ''; // Clear the editor
                          }
                        }}
                        aria-label="Add task"
                      >
                        Add Task
                      </Button>
                    </motion.div>
                  </motion.div>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
